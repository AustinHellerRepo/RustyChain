use std::{sync::{Arc, Mutex}, future::Future};

use async_trait::async_trait;

#[async_trait]
trait ChainLink {
    type TInput;
    type TOutput;

    async fn receive(&mut self, input: Arc<Mutex<Self::TInput>>);
    async fn send(&mut self) -> Option<Arc<Mutex<Self::TOutput>>>;
    async fn poll(&mut self);
    //async fn chain(self, other: impl ChainLink) -> ChainLnk;
}

#[derive(Debug)]
enum SomeInput {
    First,
    Second
}

struct Test {}

#[async_trait]
impl ChainLink for Test {
    type TInput = SomeInput;
    type TOutput = String;

    async fn receive(&mut self, input: Arc<Mutex<Self::TInput>>) {
        todo!();
    }
    async fn send(&mut self) -> Option<Arc<Mutex<Self::TOutput>>> {
        todo!();
    }
    async fn poll(&mut self) {
        todo!();
    }
}

macro_rules! chain_link {
    ($type:ty, $receive_name:ident: $receive_type:ty => $output_type:ty, $map_block:block) => {
        paste::paste! {
            #[derive(Default)]
            struct $type {
                input_queue: deadqueue::unlimited::Queue<Arc<Mutex<$receive_type>>>,
                output_queue: deadqueue::unlimited::Queue<Arc<Mutex<$output_type>>>
            }

            #[async_trait::async_trait]
            impl ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn receive(&mut self, input: Arc<Mutex<$receive_type>>) -> () {
                    self.input_queue.push(input);
                }
                async fn send(&mut self) -> Option<Arc<Mutex<$output_type>>> {
                    self.output_queue.try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn poll(&mut self) {
                    if let Some($receive_name) = self.input_queue.try_pop() {
                        let $receive_name: &mut $receive_type = &mut $receive_name.lock().unwrap();
                        self.output_queue.push(Arc::new(Mutex::new($map_block)));
                    } 
                }
            }
        }
    };
}

macro_rules! chain {
    ($type:ty,
        $receive_type:ty => $output_type:ty,
        [$first_chain_link_type:ty => $last_chain_link_type:ty]) => {
        
        paste::paste! {
            #[derive(Default)]
            struct $type {
                [<$first_chain_link_type:snake>]: $first_chain_link_type,
                [<$last_chain_link_type:snake>]: $last_chain_link_type
            }

            #[async_trait::async_trait]
            impl ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn receive(&mut self, input: Arc<Mutex<$receive_type>>) -> () {
                    self.[<$first_chain_link_type:snake>].input_queue.push(input);
                }
                async fn send(&mut self) -> Option<Arc<Mutex<$output_type>>> {
                    self.[<$last_chain_link_type:snake>].output_queue.try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn poll(&mut self) {
                    self.[<$first_chain_link_type:snake>].poll().await;
                    let next_input = self.[<$first_chain_link_type:snake>].send().await;
                    if let Some(next_input) = next_input {
                        self.[<$last_chain_link_type:snake>].receive(next_input).await;
                    }
                    self.[<$last_chain_link_type:snake>].poll().await;
                }
            }
        }
    };
    ($type:ty,
        $receive_type:ty => $output_type:ty,
        [$first_chain_link_type:ty => ($($chain_link_type:ty)=>+) => $last_chain_link_type:ty]) => {
        
        paste::paste! {
            struct $type {
                [<$first_chain_link_type:snake>]: $first_chain_link_type,
                $(
                    [<$chain_link_type:snake>]: $chain_link_type,
                )*
                [<$last_chain_link_type:snake>]: $last_chain_link_type
            }

            // fixed by https://stackoverflow.com/questions/33193846/using-macros-how-to-get-unique-names-for-struct-fields

            #[async_trait::async_trait]
            impl ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn receive(&mut self, input: Arc<Mutex<$receive_type>>) -> () {
                    self.[<$first_chain_link_type:snake>].push(input);
                }
                async fn send(&mut self) -> Option<Arc<Mutex<$output_type>>> {
                    self.[<$last_chain_link_type>].try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn poll(&mut self) {
                    self.[<$first_chain_link_type:snake>].poll();
                    let next_input = self.[<$first_chain_link_type>].send().await;
                    $(
                        if let Some(next_input) = next_input {
                            self.[<$chain_link_type:snake>].receive(next_input);
                        }
                        self.[<$chain_link_type:snake>].poll();
                        let next_input = self.[<$chain_link_type>].send().await;
                    )*
                    if let Some(next_input) = next_input {
                        self.[<$last_chain_link_type:snake>].receive(next_input);
                    }
                    self.[<$last_chain_link_type:snake>].poll();
                }
            }
        }
    };
}

chain_link!(TestChainLink, input:SomeInput => String, {
    match input {
        SomeInput::First => {
            String::from("first")
        },
        SomeInput::Second => {
            String::from("second")
        }
    }
});

chain_link!(StringToSomeInput, input:String => SomeInput, {
    match input.as_str() {
        "first" => SomeInput::First,
        "second" => SomeInput::Second,
        _ => panic!("Unexpected value")
    }
});

chain!(ChainTest, SomeInput => SomeInput, [TestChainLink => StringToSomeInput]);

chain!(TripleTest, SomeInput => String, [TestChainLink => (StringToSomeInput) => TestChainLink]);

#[tokio::main]
async fn main() {
    let mut test = TestChainLink::default();
    let value = Arc::new(Mutex::new(SomeInput::Second));
    test.receive(value).await;
    test.poll().await;
    let response = test.send().await;
    match response {
        Some(response) => {
            println!("{}", response.lock().unwrap());
        },
        None => {
            println!("None")
        }
    }

    let mut chain_test = ChainTest::default();
    let value = Arc::new(Mutex::new(SomeInput::Second));
    chain_test.receive(value).await;
    chain_test.poll().await;
    let response = chain_test.send().await;
    match response {
        Some(response) => {
            println!("{:?}", response.lock().unwrap());
        },
        None => {
            println!("None")
        }
    }
}
