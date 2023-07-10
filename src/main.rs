use std::{sync::{Arc, Mutex}, future::Future};

use async_trait::async_trait;


#[async_trait]
trait ChainLink {
    type TInput;
    type TOutput;

    async fn receive(&mut self, input: Arc<Mutex<Self::TInput>>);
    async fn send(&mut self) -> Option<Arc<Mutex<Self::TOutput>>>;
    async fn poll(&mut self);
}

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
}
