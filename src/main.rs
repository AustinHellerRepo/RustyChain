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


macro_rules! chain {
    ($name:ident, $from:ty => $to:ty, $($field:ident)=>*) => {
        chain_first!($name, $from, $to,       (x)              ()        ()                        $($field)*);
    };
}

macro_rules! chain_first {
    (                    $name:ident, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident      $($rest:ident)*) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $next, [<$($prefix)* _ $next:snake>],  ($($prefix)* x ) ($($past)*) ($($past_type)*) $($rest)*      );
        }
    };
}

macro_rules! chain_remaining {
    // In the recursive case: append another `x` into our prefix.
    (                    $name:ident, $from:ty, $to:ty, $first:ident, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $first, $first_name, $next, [<$($prefix)* _ $next:snake>],   () ($($past)*) ($($past_type)*)    );
        }
    };
    (                    $name:ident, $from:ty, $to:ty, $first:ident, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident      $($rest:ident)*) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $first, $first_name,    ($($prefix)* x ) ($($past)* [$($prefix)* _ [<$next:snake>]]) ($($past_type)* [$next]) $($rest)*      );
        }
    };

    // When there are no fields remaining.
    ($name:ident, $from:ty, $to:ty, $first:ident, $first_name:ident, $last:ident, $last_name:ident,  () ($([$($field:tt)*])*) ($([$field_type:ident])*)) => {
        paste::paste! {
            #[derive(Default)]
            struct $name {
                $first_name: $first,
                $(
                    [<$($field)*>]: $field_type,
                )*
                $last_name: $last
            }

            #[async_trait::async_trait]
            impl ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn receive(&mut self, input: Arc<Mutex<$from>>) -> () {
                    self.$first_name.receive(input).await
                }
                async fn send(&mut self) -> Option<Arc<Mutex<$to>>> {
                    self.$last_name.send().await
                }
                async fn poll(&mut self) {
                    self.$first_name.poll().await;
                    let next_input = self.$first_name.send().await;
                    $(
                        if let Some(next_input) = next_input {
                            self.[<$($field)*>].receive(next_input).await;
                        }
                        else {
                            return;
                        }
                        self.[<$($field)*>].poll().await;
                        let next_input = self.[<$($field)*>].send().await;
                    )*
                    if let Some(next_input) = next_input {
                        self.$last_name.receive(next_input).await;
                    }
                    else {
                        return;
                    }
                    self.$last_name.poll().await;
                }
            }
        }
    };
}

chain!(ChainTest, SomeInput => SomeInput, TestChainLink => StringToSomeInput);

chain!(TripleTest, SomeInput => String, TestChainLink => StringToSomeInput => TestChainLink);



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

    let mut triple_test = TripleTest::default();
    let value = Arc::new(Mutex::new(SomeInput::First));
    triple_test.receive(value).await;
    triple_test.poll().await;
    let response = triple_test.send().await;
    match response {
        Some(response) => {
            println!("{:?}", response.lock().unwrap());
        },
        None => {
            println!("None")
        }
    }

}
