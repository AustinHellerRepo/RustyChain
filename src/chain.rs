use async_trait::async_trait;

#[async_trait]
pub trait ChainLink {
    type TInput;
    type TOutput;

    async fn receive(&mut self, input: std::sync::Arc<tokio::sync::Mutex<Self::TInput>>);
    async fn send(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<Self::TOutput>>>;
    async fn poll(&mut self);
    //async fn chain(self, other: impl ChainLink) -> ChainLink;
}

#[macro_export]
macro_rules! chain_link {
    ($type:ty => ($($property_name:ident: $property_type:ty),*), $receive_name:ident: $receive_type:ty => $output_type:ty, $map_block:block) => {
        paste::paste! {
            pub struct $type {
                initializer: std::sync::Arc<tokio::sync::Mutex<[<$type Initializer>]>>,
                input_queue: deadqueue::unlimited::Queue<std::sync::Arc<tokio::sync::Mutex<$receive_type>>>,
                output_queue: deadqueue::unlimited::Queue<std::sync::Arc<tokio::sync::Mutex<$output_type>>>
            }

            pub struct [<$type Initializer>] {
                $(
                    pub $property_name: $property_type,
                )*
            }

            impl $type {
                pub fn new(initializer: [<$type Initializer>]) -> Self {
                    $type {
                        initializer: std::sync::Arc::new(tokio::sync::Mutex::new(initializer)),
                        input_queue: deadqueue::unlimited::Queue::<std::sync::Arc<tokio::sync::Mutex<$receive_type>>>::default(),
                        output_queue: deadqueue::unlimited::Queue::<std::sync::Arc<tokio::sync::Mutex<$output_type>>>::default()
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

                async fn receive(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$receive_type>>) -> () {
                    self.input_queue.push(input);
                }
                async fn send(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$output_type>>> {
                    self.output_queue.try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn poll(&mut self) {
                    if let Some($receive_name) = self.input_queue.try_pop() {
                        let received: &mut $receive_type = &mut (*$receive_name.lock().await);
                        let initializer: &mut [<$type Initializer>] = &mut (*self.initializer.lock().await);
                        let $receive_name = [<_ $type Input>] {
                            received,
                            initializer
                        };
                        self.output_queue.push(std::sync::Arc::new(tokio::sync::Mutex::new($map_block)));
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
            pub struct $name {
                $first_name: $first,
                $(
                    [<$($field)*>]: $field_type,
                )*
                $last_name: $last
            }

            pub struct [<$name Initializer>] {
                $first_name: [<$first Initializer>],
                $(
                    [<$($field)*>]: [<$field_type Initializer>],
                )*
                $last_name: [<$last Initializer>]
            }

            impl $name {
                pub fn new(initializer: [<$name Initializer>]) -> Self {
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

                async fn receive(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$from>>) -> () {
                    self.$first_name.receive(input).await
                }
                async fn send(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$to>>> {
                    self.$last_name.send().await
                }
                async fn poll(&mut self) {
                    self.$first_name.poll().await;
                    let next_input = self.$first_name.send().await;
                    $(
                        if let Some(next_input) = next_input {
                            self.[<$($field)*>].receive(next_input).await;
                        }
                        self.[<$($field)*>].poll().await;
                        let next_input = self.[<$($field)*>].send().await;
                    )*
                    if let Some(next_input) = next_input {
                        self.$last_name.receive(next_input).await;
                    }
                    self.$last_name.poll().await;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! split_merge {
    ($name:ty, $from:ty => $to:ty, ($($destination:ty),*)) => {
        split_merge_helper!($name, $from, $to, (0) () (x) () (), $($destination),*);
    };
}

#[allow(unused_macros)]
macro_rules! split_merge_helper {
    ($name:ty, $from:ty, $to:ty, ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty) => {
        paste::paste! {
            split_merge_helper!($name, $from, $to, ($index + 1) ($($index_past)* [$index]) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]));
        }
    };
    ($name:ty, $from:ty, $to:ty, ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty, $($destination:ty),*) => {
        paste::paste! {
            split_merge_helper!($name, $from, $to, ($index + 1) ($($index_past)* [$index]) ($($prefix)* x) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]), $($destination),*);
        }
    };
    ($name:ident, $from:ty, $to:ty, ($count:expr) ($([$index:expr])*) ($([$($field:tt)*])*) ($([$field_type:ty])*)) => {
        paste::paste! {
            pub struct $name {
                $(
                    [<$($field)*>]: $field_type,
                )*
                next_send_field_index: tokio::sync::Mutex<usize>
            }

            pub struct [<$name Initializer>] {
                $(
                    [<$($field)* _initializer>]: [<$field_type Initializer>],
                )*
            }

            impl $name {
                pub fn new(initializer: [<$name Initializer>]) -> Self {
                    $name {
                        $(
                            [<$($field)*>]: $field_type::new(initializer.[<$($field)* _initializer>]),
                        )*
                        next_send_field_index: tokio::sync::Mutex::new(0)
                    }
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn receive(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$from>>) -> () {
                    futures::join!($(self.[<$($field)*>].receive(input.clone())),*);
                }
                async fn send(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$to>>> {

                    // loop until we have found `Some` or looped around all internal ChainLink instanes
                    let mut send_attempts_count: usize = 0;
                    while send_attempts_count < $count {

                        // get the next field index to check
                        let next_send_field_index: usize;
                        {
                            let mut next_send_field_index_lock = self.next_send_field_index.lock().await;
                            next_send_field_index = *next_send_field_index_lock;
                            if next_send_field_index + 1 == ($count) {
                                *next_send_field_index_lock = 0;
                            }
                            else {
                                *next_send_field_index_lock = next_send_field_index + 1;
                            }
                        }
                        
                        // get the output for the current field index
                        let output;
                        if false {
                            panic!("False should not be true");
                        }
                        $(
                            else if next_send_field_index == ($index) {
                                output = self.[<$($field)*>].send().await;
                            }
                        )*
                        else {
                            panic!("Index out of bounds: next_send_field_index");
                        }

                        // return the output if `Some`, else try to loop again
                        if output.is_some() {
                            return output;
                        }

                        send_attempts_count += 1;
                    }

                    // if we've exhausted all internal `ChainLink` instances, return None
                    return None;
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
