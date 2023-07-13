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

macro_rules! chain_helper {
    // In the recursive case: append another 'x' into our prefix
    ($type:ty, $receive_type:ty => $output_type:ty, ($($prefix:tt)*) ($($past:tt)*) $next:ident $($rest:ident)*) => {
        chain_helper!($type ($($prefix)* x) ($($past)* [$($prefix)* _ $next]) $($rest)*);
    };

    // When there are no fields remaining
    ($type:ty, $receive_type:ty => $output_type:ty, ($($prefix:tt)*) ($([$($field:tt)*])*)) => {
        paste::paste! {
            
        }
    }
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

//chain!(TripleTest, SomeInput => String, [TestChainLink => (StringToSomeInput) => TestChainLink]);

//[$first_chain_link_type:ty => ($($chain_link_type:ty)=>+) => $last_chain_link_type:ty]) => {
macro_rules! my_macro {
    ($name:ident, ($($field:ident),*)) => {
        my_macro_helper!($name       (x)              ()                                $($field)*);
    };
}

macro_rules! my_macro_helper {
    // In the recursive case: append another `x` into our prefix.
    (                    $name:ident ($($prefix:tt)*) ($($past:tt)*)                    $next:ident      $($rest:ident)*) => {
        my_macro_helper!($name       ($($prefix)* x ) ($($past)* [$($prefix)* _ $next]) $($rest)*      );
    };

    // When there are no fields remaining.
    ($name:ident ($($prefix:tt)*) ($([$($field:tt)*])*)) => {
        paste::paste! {
            // Expands to:
            //    pub struct Blah {
            //        x_a: i32,
            //        xx_b: i32,
            //        xxx_c: i32,
            //        xxxx_a: i32,
            //    }
            pub struct $name {
                $(
                    [<$($field)*>]: i32,
                )*
            }

            // Expands to:
            //    impl Blah {
            //        pub fn foo(&self) -> i32 {
            //            0 + self.x_a + self.xx_b + self.xxx_c + self.xxxx_a
            //        }
            //    }
            impl $name {
                pub fn foo(&self) -> i32 {
                    0 $(
                        + self.[<$($field)*>]
                    )*
                }
            }
        }
    };
}

my_macro!(Foobar, (a, b, c, d));


macro_rules! my_macro_expanded {
    ($name:ident, $other:ident, ($($field:ident),*)) => {
        my_macro_expanded_helper!($name, $other,       (x)              ()        ()                        $($field)*);
    };
}

macro_rules! my_macro_expanded_helper {
    // In the recursive case: append another `x` into our prefix.
    (                    $name:ident, $other:ident, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident      $($rest:ident)*) => {
        paste::paste! {
            my_macro_expanded_helper!($name, $other,       ($($prefix)* x ) ($($past)* [$($prefix)* _ [<$next:snake>]]) ($($past_type)* [$next]) $($rest)*      );
        }
    };
    (                    $name:ident, $other:ident, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident) => {
        paste::paste! {
            my_macro_expanded_helper!($name, $other,       ($($prefix)* x ) ($($past)* [$($prefix)* _ [<$next:snake>]]) ($($past_type)* [$next])    );
        }
    };

    // When there are no fields remaining.
    ($name:ident, $other:ident, ($($prefix:tt)*) ($([$($field:tt)*])*) ($([$field_type:ident])*)) => {
        paste::paste! {
            // Expands to:
            //    pub struct Blah {
            //        x_a: i32,
            //        xx_b: i32,
            //        xxx_c: i32,
            //        xxxx_a: i32,
            //    }
            pub struct $name {
                $(
                    [<$($field)*>]: $field_type,
                )*
            }

            $(
                pub struct $field_type {}
            )*

            pub struct $other {}

            // Expands to:
            //    impl Blah {
            //        pub fn foo(&self) -> i32 {
            //            0 + self.x_a + self.xx_b + self.xxx_c + self.xxxx_a
            //        }
            //    }
        }
    };
}

my_macro_expanded!(Asdf, Fdsa, (Cat, Dog));


//macro_rules! example {
//    ($left:ident, [$($inner:ident)=>*], $right:ident) => {
//        example_helper!($left, $right, (x) () $($inner)*);
//    };
//}
//
//macro_rules! example_helper {
//    (                       $left:ident, $right:ident, ($($prefix:tt)*) ($($past:tt)*) $next:ident                               $($rest:ident)*) => {
//        example_helper!($left,       $right,       ($($prefix)* x)  ($($past)*     [$($prefix)* _ $next]) $($rest)*);
//    };
//    ($left:ident, $right:ident, ($($prefix:tt)*) ($([$(field:tt => $field_type:ident)*])*)) => {
//        paste::paste! {
//            struct $left {
//                $(
//                    [<$($field)*>]: $($field_type)*,
//                )*
//            }
//            struct $right {}
//        }
//    }
//}
//
//example!(Left, [SomeA => SomeB], Right);

//macro_rules! chain_iter {
//    ($name:ty, $from:ty => $to:ty, [$first_field:ty => ($($field:ty)=>*) => $last_field:ty]) => {
//        chain_iter_helper!($name,    $from =>    $to,    $first_field   , $last_field    => (x)              ()                                $($field)=>*                       );
//    };
//}

//macro_rules! chain_iter_helper {
//    // In the recursive case: append another `x` into our prefix.
//    (                      $name:ty, $from:ty => $to:ty, $first_field:ty, $last_field:ty => ($($prefix:tt)*) ($($past:tt)*)              $next:ty =>   $($rest:ty)=>*) => {
//        chain_iter_helper!($name,    $from =>    $to,    $first_field   , $last_field    => ($($prefix)* x ) ($($past)* [$($prefix)* _ $next:snake => $next]  ) $($rest)=>* );
//    };
//    (                      $name:ty, $from:ty => $to:ty, $first_field:ty, $last_field:ty => ($($prefix:tt)*) ($($past:tt)*)              $next:ty ) => {
//        chain_iter_helper!($name,    $from =>    $to,    $first_field   , $last_field    => ($($prefix)* x ) ($($past)* [$($prefix)* _ $next:snake => $next]  )  );
//    };
//
//    // When there are no fields remaining.
//    (                      $name:ty, $from:ty => $to:ty, $first_field:ty, $last_field:ty => ($($prefix:tt)*) ($(        [$($field:tt => $field_type:ty)*   ]  )*)) => {
//        paste::paste! {
//            struct $name {
//                [<$first_field:snake>]: $first_field,
//                $(
//                    [<$($field)*>]: $($field_type)*,
//                )*
//                [<$last_field:snake>]: $last_field
//            }
//
//            // fixed by https://stackoverflow.com/questions/33193846/using-macros-how-to-get-unique-names-for-struct-fields
//
//            #[async_trait::async_trait]
//            impl ChainLink for $name {
//                type TInput = $from;
//                type TOutput = $to;
//
//                async fn receive(&mut self, input: Arc<Mutex<$from>>) -> () {
//                    self.[<$first_field:snake>].push(input);
//                }
//                async fn send(&mut self) -> Option<Arc<Mutex<$to>>> {
//                    self.[<$last_field:snake>].try_pop().map(|element| {
//                        element.into()
//                    })
//                }
//                async fn poll(&mut self) {
//                    self.[<$first_field:snake>].poll();
//                    let next_input = self.[<$first_field:snake>].send().await;
//                    $(
//                        if let Some(next_input) = next_input {
//                            self.[<$($field)*>].receive(next_input);
//                        }
//                        self.[<$($field)*>].poll();
//                        let next_input = self.[<$($field)*>].send().await;
//                    )*
//                    if let Some(next_input) = next_input {
//                        self.[<$last_field:snake>].receive(next_input);
//                    }
//                    self.[<$last_field:snake>].poll();
//                }
//            }
//        }
//    };
//}
//
//chain_iter!(Foo, FromA => ToA, [Abc => (Bcd => Cde) => Def]);

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
