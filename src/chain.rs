use async_trait::async_trait;

#[async_trait]
pub trait ChainLink {
    type TInput;
    type TOutput;

    async fn receive(&mut self, input: std::sync::Arc<std::sync::Mutex<Self::TInput>>);
    async fn send(&mut self) -> Option<std::sync::Arc<std::sync::Mutex<Self::TOutput>>>;
    async fn poll(&mut self);
    //async fn chain(self, other: impl ChainLink) -> ChainLink;
}

#[macro_export]
macro_rules! chain_link {
    ($type:ty => ($($property_name:ident: $property_type:ty),*), $receive_name:ident: $receive_type:ty => $output_type:ty, $map_block:block) => {
        paste::paste! {
            pub struct $type {
                initializer: std::sync::Arc<std::sync::Mutex<[<$type Initializer>]>>,
                input_queue: deadqueue::unlimited::Queue<std::sync::Arc<std::sync::Mutex<$receive_type>>>,
                output_queue: deadqueue::unlimited::Queue<std::sync::Arc<std::sync::Mutex<$output_type>>>
            }

            pub struct [<$type Initializer>] {
                $(
                    pub $property_name: $property_type,
                )*
            }

            impl $type {
                pub fn new(initializer: [<$type Initializer>]) -> Self {
                    $type {
                        initializer: std::sync::Arc::new(std::sync::Mutex::new(initializer)),
                        input_queue: deadqueue::unlimited::Queue::<std::sync::Arc<std::sync::Mutex<$receive_type>>>::default(),
                        output_queue: deadqueue::unlimited::Queue::<std::sync::Arc<std::sync::Mutex<$output_type>>>::default()
                    }
                }
            }

            #[allow(dead_code)]
            pub struct [<_ $type Input>]<'a> {
                received: &'a mut $receive_type,
                initializer: &'a mut [<$type Initializer>]
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn receive(&mut self, input: std::sync::Arc<std::sync::Mutex<$receive_type>>) -> () {
                    self.input_queue.push(input);
                }
                async fn send(&mut self) -> Option<std::sync::Arc<std::sync::Mutex<$output_type>>> {
                    self.output_queue.try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn poll(&mut self) {
                    if let Some($receive_name) = self.input_queue.try_pop() {
                        let received: &mut $receive_type = &mut $receive_name.lock().unwrap();
                        let initializer: &mut [<$type Initializer>] = &mut self.initializer.lock().unwrap();
                        let $receive_name = [<_ $type Input>] {
                            received,
                            initializer
                        };
                        self.output_queue.push(std::sync::Arc::new(std::sync::Mutex::new($map_block)));
                    } 
                }
            }
        }
    };
    ($type:ty, $receive_name:ident: $receive_type:ty => $output_type:ty, $map_block:block) => {
        chain_link!($type => (), $receive_name: $receive_type => $output_type, $map_block);
    };
}

#[macro_export]
macro_rules! chain {
    ($name:ident, $from:ty => $to:ty, $($field:ty)=>*) => {
        chain_first!($name, $from, $to,       (x)              ()        ()                        $($field)=>*);
    };
}

#[allow(unused_macros)]
macro_rules! chain_first {
    (                    $name:ident, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty  ) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $next, [<$($prefix)* _ $next:snake>],  ($($prefix)* x ) ($($past)*) ($($past_type)*)      );
        }
    };
    (                    $name:ident, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty =>   $($rest:ty)=>*) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $next, [<$($prefix)* _ $next:snake>],  ($($prefix)* x ) ($($past)*) ($($past_type)*) $($rest)=>*      );
        }
    };
}

#[allow(unused_macros)]
macro_rules! chain_remaining {
    // In the recursive case: append another `x` into our prefix.
    (                    $name:ident, $from:ty, $to:ty, $first:ty, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $first, $first_name, $next, [<$($prefix)* _ $next:snake>],   () ($($past)*) ($($past_type)*)    );
        }
    };
    (                    $name:ident, $from:ty, $to:ty, $first:ty, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty =>    $($rest:ty)=>*) => {
        paste::paste! {
            chain_remaining!($name, $from, $to, $first, $first_name,    ($($prefix)* x ) ($($past)* [$($prefix)* _ [<$next:snake>]]) ($($past_type)* [$next]) $($rest)=>*      );
        }
    };

    // When there are no fields remaining.
    ($name:ident, $from:ty, $to:ty, $first:ty, $first_name:ident, $last:ty, $last_name:ident,  () ($([$($field:tt)*])*) ($([$field_type:ty])*)) => {
        paste::paste! {
            struct $name {
                $first_name: $first,
                $(
                    [<$($field)*>]: $field_type,
                )*
                $last_name: $last
            }

            struct [<$name Initializer>] {
                $first_name: [<$first Initializer>],
                $(
                    [<$($field)*>]: [<$field_type Initializer>],
                )*
                $last_name: [<$last Initializer>]
            }

            impl $name {
                fn new(initializer: [<$name Initializer>]) -> Self {
                    $name {
                        $first_name: $first::new(initializer.$first_name),
                        $(
                            [<$($field)*>]: $field_type::new(initializer.[<$($field)*>]),
                        )*
                        $last_name: $last::new(initializer.$last_name)
                    }
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn receive(&mut self, input: std::sync::Arc<std::sync::Mutex<$from>>) -> () {
                    self.$first_name.receive(input).await
                }
                async fn send(&mut self) -> Option<std::sync::Arc<std::sync::Mutex<$to>>> {
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

macro_rules! split_merge {
    ($name:ty, $from:ty => $to:ty, ($($destination:ty => $destination_output:ty),*)) => {
        split_merge_helper!($name, $from, $to, (x) () (), $($destination => $destination_output),*);
    };
}

macro_rules! split_merge_helper {
    ($name:ty, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty => $next_output:ty) => {
        paste::paste! {
            split_merge_helper!($name, $from, $to, ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next, $next_output]));
        }
    };
    ($name:ty, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty => $next_output:ty, $($destination:ty => $destination_output:ty),*) => {
        paste::paste! {
            split_merge_helper!($name, $from, $to, ($($prefix)* x) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next, $next_output]), $($destination => $destination_output),*);
        }
    };
    ($name:ident, $from:ty, $to:ty, ($([$($field:tt)*])*) ($([$field_type:ty, $field_output:ty])*)) => {
        paste::paste! {
            pub struct $name {
                $(
                    [<$($field)*>]: $field_type,
                )*
            }

            impl $name {
                pub fn new($([<$($field)* _initializer>]: [<$field_type Initializer>]),*) -> Self {
                    $name {
                        $(
                            [<$($field)*>]: $field_type::new([<$($field)* _initializer>]),
                        )*
                    }
                }
            }

            pub struct [<$name Output>] {
                $(
                    [<$($field)*>]: $field_output,
                )*
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn receive(&mut self, input: std::sync::Arc<std::sync::Mutex<$from>>) -> () {
                    futures::join!($(self.[<$($field)*>].receive(input.clone())),*);
                }
                async fn send(&mut self) -> Option<std::sync::Arc<std::sync::Mutex<($($field_output),*)>>> {
                    let output = futures::join!($(self.[<$($field)*>].send()),*)
                }
                async fn poll(&mut self) {
                    $(
                        self.[<$($field)*>].poll().await;
                    )*
                }
            }
        }
    }
}
