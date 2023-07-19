use async_trait::async_trait;

#[async_trait]
pub trait ChainLink {
    type TInput;
    type TOutput;

    async fn push(&mut self, input: std::sync::Arc<tokio::sync::Mutex<Self::TInput>>);
    async fn push_raw(&mut self, input: Self::TInput);
    async fn try_pop(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<Self::TOutput>>>;
    async fn process(&mut self) -> bool;
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
                received: Option<&'a mut $receive_type>,
                initializer: std::sync::Arc<tokio::sync::Mutex<[<$type Initializer>]>>
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn push(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$receive_type>>) -> () {
                    self.input_queue.push(input);
                }
                async fn push_raw(&mut self, input: $receive_type) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::Mutex::new(input))).await
                }
                async fn try_pop(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$output_type>>> {
                    self.output_queue.try_pop().map(|element| {
                        element.into()
                    })
                }
                async fn process(&mut self) -> bool {
                    async fn get_map_block_result($receive_name: [<_ $type Input>]<'_>) -> Option<$output_type> {
                        $map_block
                    }
                    let $receive_name = [<_ $type Input>] {
                        received: None,
                        initializer: self.initializer.clone()
                    };
                    if let Some(output) = get_map_block_result($receive_name).await {
                        self.output_queue.push(std::sync::Arc::new(tokio::sync::Mutex::new(output)));
                        return true;
                    }
                    else if let Some($receive_name) = self.input_queue.try_pop() {
                        let mut locked_receive_name = $receive_name.lock().await;
                        let $receive_name = [<_ $type Input>] {
                            received: Some(&mut locked_receive_name),
                            initializer: self.initializer.clone()
                        };
                        if let Some(output) = get_map_block_result($receive_name).await {
                            self.output_queue.push(std::sync::Arc::new(tokio::sync::Mutex::new(output)));
                            return true;
                        }
                    } 
                    return false;
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
    ($name:ty, $from:ty => $to:ty, $($field:ty)=>*) => {
        chain!(first $name, $from, $to,       (x)              ()        ()                        $($field)=>*);
    };
    (first                    $name:ty, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty  ) => {
        paste::paste! {
            chain!(middle $name, $from, $to, $next, [<$($prefix)* _ $next:snake>],  ($($prefix)* x ) ($($past)*) ($($past_type)*)      );
        }
    };
    (first                    $name:ty, $from:ty, $to:ty, ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty =>   $($rest:ty)=>*) => {
        paste::paste! {
            chain!(middle $name, $from, $to, $next, [<$($prefix)* _ $next:snake>],  ($($prefix)* x ) ($($past)*) ($($past_type)*) $($rest)=>*      );
        }
    };
    // In the recursive case: append another `x` into our prefix.
    (middle                    $name:ty, $from:ty, $to:ty, $first:ty, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ident) => {
        paste::paste! {
            chain!(end $name, $from, $to, $first, $first_name, $next, [<$($prefix)* _ $next:snake>],   () ($($past)*) ($($past_type)*)    );
        }
    };
    (middle                    $name:ty, $from:ty, $to:ty, $first:ty, $first_name:ident,  ($($prefix:tt)*) ($($past:tt)*)     ($($past_type:tt)*)               $next:ty =>    $($rest:ty)=>*) => {
        paste::paste! {
            chain!(middle $name, $from, $to, $first, $first_name,    ($($prefix)* x ) ($($past)* [$($prefix)* _ [<$next:snake>]]) ($($past_type)* [$next]) $($rest)=>*      );
        }
    };

    // When there are no fields remaining.
    (end $name:ty, $from:ty, $to:ty, $first:ty, $first_name:ident, $last:ty, $last_name:ident,  () ($([$($field:tt)*])*) ($([$field_type:ty])*)) => {
        paste::paste! {

            pub struct $name {
                $first_name: $first,
                $(
                    [<$($field)*>]: $field_type,
                )*
                $last_name: $last
            }

            pub struct [<$name Initializer>] {
                pub $first_name: [<$first Initializer>],
                $(
                    pub [<$($field)*>]: [<$field_type Initializer>],
                )*
                pub $last_name: [<$last Initializer>]
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

                async fn push(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$from>>) -> () {
                    self.$first_name.push(input).await
                }
                async fn push_raw(&mut self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::Mutex::new(input))).await
                }
                async fn try_pop(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$to>>> {
                    self.$last_name.try_pop().await
                }
                async fn process(&mut self) -> bool {
                    let mut is_at_least_one_processed = true;
                    let mut is_last_processed = false;
                    while is_at_least_one_processed && !is_last_processed {
                        is_at_least_one_processed = self.$first_name.process().await;
                        let next_input = self.$first_name.try_pop().await;
                        $(
                            if let Some(next_input) = next_input {
                                self.[<$($field)*>].push(next_input).await;
                            }
                            is_at_least_one_processed |= self.[<$($field)*>].process().await;
                            let next_input = self.[<$($field)*>].try_pop().await;
                        )*
                        if let Some(next_input) = next_input {
                            self.$last_name.push(next_input).await;
                        }
                        is_last_processed = self.$last_name.process().await;
                    }
                    return is_last_processed;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! split_merge {
    ($name:ty, $from:ty => $to:ty, ($($destination:ty),*)) => {
        split_merge!(middle $name, $from, $to, () (0) () (x) () (), $($destination),*);
    };
    (middle $name:ty, $from:ty, $to:ty, ($($bool:tt)*) ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty) => {
        paste::paste! {
            split_merge!(end $name, $from, $to, ($($bool)* [false]) ($index + 1) ($($index_past)* [$index]) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]));
        }
    };
    (middle $name:ty, $from:ty, $to:ty, ($($bool:tt)*) ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty, $($destination:ty),*) => {
        paste::paste! {
            split_merge!(middle $name, $from, $to, ($($bool)* [false]) ($index + 1) ($($index_past)* [$index]) ($($prefix)* x) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]), $($destination),*);
        }
    };
    (end $name:ident, $from:ty, $to:ty, ($([$bool:tt])*) ($count:expr) ($([$index:expr])*) ($([$($field:tt)*])*) ($([$field_type:ty])*)) => {
        paste::paste! {
            pub struct $name {
                $(
                    [<$($field)*>]: $field_type,
                )*
                next_send_field_index: tokio::sync::Mutex<usize>
            }

            pub struct [<$name Initializer>] {
                $(
                    pub [<$($field)* _initializer>]: [<$field_type Initializer>],
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

                async fn push(&mut self, input: std::sync::Arc<tokio::sync::Mutex<$from>>) -> () {
                    futures::join!($(self.[<$($field)*>].push(input.clone())),*);
                }
                async fn push_raw(&mut self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::Mutex::new(input))).await
                }
                async fn try_pop(&mut self) -> Option<std::sync::Arc<tokio::sync::Mutex<$to>>> {

                    // loop until we have found `Some` or looped around all internal ChainLink instanes
                    let mut next_send_field_index_lock = self.next_send_field_index.lock().await;
                    let mut send_attempts_count: usize = 0;
                    while send_attempts_count < $count {

                        // get the next field index to check
                        let next_send_field_index: usize;
                        next_send_field_index = *next_send_field_index_lock;
                        if next_send_field_index + 1 == ($count) {
                            *next_send_field_index_lock = 0;
                        }
                        else {
                            *next_send_field_index_lock = next_send_field_index + 1;
                        }
                        
                        // get the output for the current field index
                        let output;
                        if false {
                            panic!("False should not be true");
                        }
                        $(
                            else if next_send_field_index == ($index) {
                                output = self.[<$($field)*>].try_pop().await;
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
                async fn process(&mut self) -> bool {
                    let bool_tuple = futures::join!($(self.[<$($field)*>].process()),*);
                    let false_tuple = ($($bool),*);
                    return bool_tuple != false_tuple;
                }
            }
        }
    }
}
