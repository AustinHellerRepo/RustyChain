use async_trait::async_trait;

#[async_trait]
pub trait ChainLink {
    type TInput;
    type TOutput;

    async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<Self::TInput>>);
    async fn push_raw(&self, input: Self::TInput);
    async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<Self::TInput>>);
    async fn push_raw_if_empty(&self, input: Self::TInput);
    async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<Self::TOutput>>>;
    async fn process(&self) -> bool;
}

#[macro_export]
macro_rules! chain_link {
    ($type:ty => ($($property_name:ident: $property_type:ty),*), $receive_name:ident: $receive_type:ty => $output_type:ty, $map_block:block) => {
        paste::paste! {
            pub struct $type {
                initializer: std::sync::Arc<tokio::sync::RwLock<[<$type Initializer>]>>,
                input_queue: $crate::queue::Queue<std::sync::Arc<tokio::sync::RwLock<$receive_type>>>,
                output_queue: $crate::queue::Queue<std::sync::Arc<tokio::sync::RwLock<$output_type>>>
            }

            pub struct [<$type Initializer>] {
                $(
                    pub $property_name: $property_type,
                )*
            }

            #[allow(dead_code)]
            impl $type {
                pub async fn new(initializer: std::sync::Arc<tokio::sync::RwLock::<[<$type Initializer>]>>) -> Self {
                    $type {
                        initializer,
                        input_queue: $crate::queue::Queue::<std::sync::Arc<tokio::sync::RwLock<$receive_type>>>::default(),
                        output_queue: $crate::queue::Queue::<std::sync::Arc<tokio::sync::RwLock<$output_type>>>::default()
                    }
                }
                pub async fn new_raw(initializer: [<$type Initializer>]) -> Self {
                    $type::new(std::sync::Arc::new(tokio::sync::RwLock::new(initializer))).await
                }
            }

            #[allow(dead_code)]
            pub struct [<_ $type Input>] {
                received: Option<std::sync::Arc<tokio::sync::RwLock<$receive_type>>>,
                initializer: std::sync::Arc<tokio::sync::RwLock<[<$type Initializer>]>>
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $type {
                type TInput = $receive_type;
                type TOutput = $output_type;

                async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<$receive_type>>) -> () {
                    self.input_queue.push(input).await;
                }
                async fn push_raw(&self, input: $receive_type) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<$receive_type>>) -> () {
                    self.input_queue.push_if_empty(input).await;
                }
                async fn push_raw_if_empty(&self, input: $receive_type) -> () {
                    self.push_if_empty(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<$output_type>>> {
                    self.output_queue.try_pop().await.map(|element| {
                        element.into()
                    })
                }
                async fn process(&self) -> bool {
                    async fn get_map_block_result($receive_name: [<_ $type Input>]) -> Option<$output_type> {
                        $map_block
                    }
                    let $receive_name = [<_ $type Input>] {
                        received: None,
                        initializer: self.initializer.clone()
                    };
                    if let Some(output) = get_map_block_result($receive_name).await {
                        self.output_queue.push(std::sync::Arc::new(tokio::sync::RwLock::new(output))).await;
                        return true;
                    }
                    else if let Some($receive_name) = self.input_queue.try_pop().await {
                        let $receive_name = [<_ $type Input>] {
                            received: Some($receive_name),
                            initializer: self.initializer.clone()
                        };
                        if let Some(output) = get_map_block_result($receive_name).await {
                            self.output_queue.push(std::sync::Arc::new(tokio::sync::RwLock::new(output))).await;
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
macro_rules! new_chain {
    ($name:ty, $from:ty => $to:ty, ($($($field:ty)=>*),*) $choice:ident $mode:ident) => {
        new_chain!(apple $name, $from, $to, $choice, $mode, () (0) () () () () () () () () () () (x) $($($field)=>*),*);
    };
    // only one new solo type left
    (apple $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) $next:ty) => {
        paste::paste! {
            new_chain!(end $name, $from, $to, $choice, $mode, ($($bool)* [false]) ($index + 1) ($($solo_index_past)* [$index]) ($($chain_index_past)*) ($($solo)* [$next]) ($($solo_name)* [<$($prefix)* _ $next:snake>]) ($($first)*) ($($first_name)*) ($($mid)*) ($($mid_name)*) ($($last)*) ($($last_name)*));
        }
    };
    // one solo type following by another set
    (apple $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) $next:ty, $($($rest:ty)=>*),*) => {
        paste::paste! {
            // since this is the end of a chain, the next $next will be the first or solo
            new_chain!(apple $name, $from, $to, $choice, $mode, ($($bool)* [false]) ($index + 1) ($($solo_index_past)* [$index]) ($($chain_index_past)*) ($($solo)* [$next]) ($($solo_name)* [<$($prefix)* _ $next:snake>]) ($($first)*) ($($first_name)*) ($($mid)*) ($($mid_name)*) ($($last)*) ($($last_name)*) ($($prefix)* x ) $($($rest)=>*),*);
        }
    };
    // the first type following by a last type (no mid type) with no more types
    (apple $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) $next:ty => $another:ty) => {
        paste::paste! {
            new_chain!(end $name, $from, $to, $choice, $mode, ($($bool)* [false]) ($index + 1) ($($solo_index_past)*) ($($chain_index_past)* [$index]) ($($solo)*) ($($solo_name)*) ($($first)* [$next]) ($($first_name)* [<$($prefix)* _ $next:snake>]) ($($mid)* []) ($($mid_name)* []) ($($last)* [$another]) ($($last_name)* [<$($prefix)* x _ $another:snake>]));
        }
    };
    // the first type following by a last type (no mid type) with more types
    (apple $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) $next:ty => $another:ty, $($($rest:ty)=>*),*) => {
        paste::paste! {
            new_chain!(apple $name, $from, $to, $choice, $mode, ($($bool)* [false]) ($index + 1) ($($solo_index_past)*) ($($chain_index_past)* [$index]) ($($solo)*) ($($solo_name)*) ($($first)* [$next]) ($($first_name)* [<$($prefix)* _ $next:snake>]) ($($mid)* []) ($($mid_name)* []) ($($last)* [$another]) ($($last_name)* [<$($prefix)* x _ $another:snake>]) ($($prefix)* x x ) $($($rest)=>*),*);
        }
    };
    // the first type following by a chain
    (apple $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) $next:ty => $another:ty => $($($rest:ty)=>*),*) => {
        paste::paste! {
            new_chain!(carrot $name, $from, $to, $choice, $mode, ($($bool)* [false]) ($index + 1) ($($solo_index_past)*) ($($chain_index_past)* [$index]) ($($solo)*) ($($solo_name)*) ($($first)* [$next]) ($($first_name)* [<$($prefix)* _ $next:snake>]) ($($mid)*) ($($mid_name)*) ($($last)*) ($($last_name)*) ($($prefix)* x x ) ([$another]) ([<$($prefix)* x _ $another:snake>]) $($($rest)=>*),*);
        }
    };
    // the middle type of a chain after already being in the middle
    (carrot $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_name:tt)*) $next:ty => $($($rest:ty)=>*),*) => {
        paste::paste! {
            new_chain!(carrot $name, $from, $to, $choice, $mode, ($($bool)*) ($index) ($($solo_index_past)*) ($($chain_index_past)*) ($($solo)*) ($($solo_name)*) ($($first)*) ($($first_name)*) ($($mid)*) ($($mid_name)*) ($($last)*) ($($last_name)*) ($($prefix)* x) ($($past)* [$next]) ($($past_name)* [<$($prefix)* _ $next:snake>]) $($($rest)=>*),*);
        }
    };
    // the last type of a chain after already being in the middle and there is another chain
    (carrot $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_name:tt)*) $next:ty, $($($rest:ty)=>*),*) => {
        paste::paste! {
            new_chain!(apple $name, $from, $to, $choice, $mode, ($($bool)*) ($index) ($($solo_index_past)*) ($($chain_index_past)*) ($($solo)*) ($($solo_name)*) ($($first)*) ($($first_name)*) ($($mid)* [$($past)*]) ($($mid_name)* [$($past_name)*]) ($($last)* [$next]) ($($last_name)* [<$($prefix)* _ $next:snake>]) ($($prefix)* x ) $($($rest)=>*),*);
        }
    };
    // the last type of a chain after already being in the middle and the end
    (carrot $name:ty, $from:ty, $to:ty, $choice:ident, $mode:ident, ($($bool:tt)*) ($index:expr) ($($solo_index_past:tt)*) ($($chain_index_past:tt)*) ($($solo:tt)*) ($($solo_name:tt)*) ($($first:tt)*) ($($first_name:tt)*) ($($mid:tt)*) ($($mid_name:tt)*) ($($last:tt)*) ($($last_name:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_name:tt)*) $next:ty) => {
        paste::paste! {
            new_chain!(end $name, $from, $to, $choice, $mode, ($($bool)*) ($index) ($($solo_index_past)*) ($($chain_index_past)*) ($($solo)*) ($($solo_name)*) ($($first)*) ($($first_name)*) ($($mid)* [$($past)*]) ($($mid_name)* [$($past_name)*]) ($($last)* [$next]) ($($last_name)* [<$($prefix)* _ $next:snake>]));
        }
    };
    (end
        $name:ty,
        $from:ty,
        $to:ty,
        $choice:ident,
        $mode:ident,
        ($([$bool:expr])*)
        ($count:expr)
        ($([$solo_index:expr])*)
        ($([$chain_index:expr])*)
        ($([$solo:ty])*)
        ($($solo_name:ident)*)
        ($([$first:ty])*)
        ($($first_name:ident)*)
        ($([$([$mid:ty])*])*)
        ($([$($mid_name:ident)*])*)
        ($([$last:ty])*)
        ($($last_name:ident)*)) => {
        
        paste::paste! {

            #[allow(dead_code)]
            pub struct $name {
                // for internal functionality
                next_try_pop_index: std::sync::Arc<tokio::sync::Mutex<usize>>,
                // for every split
                $(
                    $first_name: std::sync::Arc<$first>,
                    $(
                        $mid_name: std::sync::Arc<$mid>,
                    )*
                    $last_name: std::sync::Arc<$last>,
                )*
                $(
                    $solo_name: std::sync::Arc<$solo>,
                )*
            }

            #[allow(dead_code)]
            pub struct [<$name Initializer>] {
                $(
                    pub $first_name: std::sync::Arc<tokio::sync::RwLock<[<$first Initializer>]>>,
                    $(
                        pub $mid_name: std::sync::Arc<tokio::sync::RwLock<[<$mid Initializer>]>>,
                    )*
                    pub $last_name: std::sync::Arc<tokio::sync::RwLock<[<$last Initializer>]>>,
                )*
                $(
                    $solo_name: std::sync::Arc<tokio::sync::RwLock<[<$solo Initializer>]>>,
                )*
            }

            #[allow(dead_code)]
            impl [<$name Initializer>] {
                pub fn new($($first_name: [<$first Initializer>], $($mid_name: [<$mid Initializer>],)* $last_name: [<$last Initializer>],)* $($solo_name: [<$solo Initializer>],)*) -> Self {
                    [<$name Initializer>] {
                        $(
                            $first_name: std::sync::Arc::new(tokio::sync::RwLock::new($first_name)),
                            $(
                                $mid_name: std::sync::Arc::new(tokio::sync::RwLock::new($mid_name)),
                            )*
                            $last_name: std::sync::Arc::new(tokio::sync::RwLock::new($last_name)),
                        )*
                        $(
                            $solo_name: std::sync::Arc::new(tokio::sync::RwLock::new($solo_name)),
                        )*
                    }
                }
            }

            #[allow(dead_code)]
            impl $name {
                pub async fn new(initializer: std::sync::Arc<tokio::sync::RwLock<[<$name Initializer>]>>) -> Self {
                    $name {
                        next_try_pop_index: std::sync::Arc::new(tokio::sync::Mutex::new(0)),
                        $(
                            $first_name: std::sync::Arc::new($first::new(initializer.read().await.$first_name.clone()).await),
                            $(
                                $mid_name: std::sync::Arc::new($mid::new(initializer.read().await.$mid_name.clone()).await),
                            )*
                            $last_name: std::sync::Arc::new($last::new(initializer.read().await.$last_name.clone()).await),
                        )*
                        $(
                            $solo_name: std::sync::Arc::new($solo::new(initializer.read().await.$solo_name.clone()).await),
                        )*
                    }
                }

                // useful functions for processing chainlinks
                $(
                    async fn [<process_ $first_name>](&self) -> bool {
                        let mut is_at_least_one_processed = true;
                        let mut is_last_processed = false;
                        while is_at_least_one_processed && !is_last_processed {
                            is_at_least_one_processed = self.$first_name.process().await;
                            let next_input = self.$first_name.try_pop().await;
                            $(
                                if let Some(next_input) = next_input {
                                    self.$mid_name.push(next_input).await;
                                }
                                is_at_least_one_processed |= self.$mid_name.process().await;
                                let next_input = self.$mid_name.try_pop().await;
                            )*
                            if let Some(next_input) = next_input {
                                self.$last_name.push(next_input).await;
                            }
                            is_last_processed = self.$last_name.process().await;
                        }
                        return is_last_processed;
                    }
                )*
                async fn process_chain(chainlinks: Vec<Arc<dyn ChainLink<TInput = $from, TOutput = $to> + Send + Sync>>) -> bool {
                    let mut is_at_least_one_processed = true;
                    let mut is_last_processed = false;
                    while is_at_least_one_processed && !is_last_processed {
                        is_at_least_one_processed = chainlinks[0].process().await;
                        let mut next_input = chainlinks[0].try_pop().await;
                        for index in 1..(chainlinks.len() - 1) {
                            if let Some(next_input) = next_input {
                                chainlinks[index].push(next_input).await;
                            }
                            is_at_least_one_processed |= chainlinks[index].process().await;
                            next_input = chainlinks[index].try_pop().await;
                        }
                        if let Some(next_input) = next_input {
                            chainlinks[chainlinks.len() - 1].push(next_input).await;
                        }
                        is_last_processed = chainlinks[chainlinks.len() - 1].process().await;
                    }
                    return is_last_processed;
                }

                // each of these functions represents all of the possible permutations for processing chains
                async fn process_all_join(&self) -> bool {
                    let bool_tuple = futures::join!($(self.$solo_name.process(),)*$(self.[<process_ $first_name>]()),*);
                    let false_tuple = ($($bool,)*);
                    return bool_tuple != false_tuple;
                }
                async fn process_all_free(&self) -> bool {
                    $(
                        {
                            let chainlinks: Vec<Arc<dyn ChainLink<TInput = $from, TOutput = $to> + Send + Sync>> = vec![
                                self.$first_name.clone(),
                                $(
                                    self.$mid_name.clone(),
                                )*
                                self.$last_name.clone()
                            ];
                            std::thread::spawn(move || {
                                let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                    .enable_time()
                                    .build()
                                    .unwrap();

                                tokio_runtime.block_on(async {
                                    $name::process_chain(chainlinks).await;
                                });
                            });
                        }
                    )*
                    $(
                        {
                            let chainlink = self.$solo_name.clone();
                            std::thread::spawn(move || {
                                let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                    .enable_time()
                                    .build()
                                    .unwrap();

                                tokio_runtime.block_on(async {
                                    chainlink.process().await;
                                });
                            });
                        }
                    )*
                    return false;
                }
                async fn process_all_unique(&self) -> bool {
                    todo!();
                }
                async fn process_one_join(&self) -> bool {
                    todo!();
                }
                async fn process_one_free(&self) -> bool {
                    todo!();
                }
                async fn process_one_unique(&self) -> bool {
                    todo!();
                }
                async fn process_random_join(&self) -> bool {
                    todo!();
                }
                async fn process_random_free(&self) -> bool {
                    todo!();
                }
                async fn process_random_unique(&self) -> bool {
                    todo!();
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    let mut push_futures = vec![];
                    $(
                        push_futures.push(self.$first_name.push(input.clone()));
                    )*
                    $(
                        push_futures.push(self.$solo_name.push(input.clone()));
                    )*
                    futures::future::join_all(push_futures).await;
                }
                async fn push_raw(&self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    let mut futures = vec![];
                    $(
                        futures.push(self.$first_name.push_if_empty(input.clone()));
                    )*
                    $(
                        futures.push(self.$solo_name.push_if_empty(input.clone()));
                    )*
                    futures::future::join_all(futures).await;
                }
                async fn push_raw_if_empty(&self, input: $from) -> () {
                    self.push_if_empty(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<$to>>> {
                    
                    let mut locked_next_try_pop_index = self.next_try_pop_index.lock().await;
                    let mut try_pop_attempt_count: usize = 0;
                    while try_pop_attempt_count < ($count) {

                        // get the next index to check
                        let next_try_pop_index: usize = *locked_next_try_pop_index;
                        if next_try_pop_index + 1 == ($count) {
                            *locked_next_try_pop_index = 0;
                        }
                        else {
                            *locked_next_try_pop_index = next_try_pop_index + 1;
                        }

                        // get the popped output for the current index
                        let output;
                        if false {
                            panic!("False should not be true.");
                        }
                        $(
                            else if next_try_pop_index == ($solo_index) {
                                output = self.$solo_name.try_pop().await;
                            }
                        )*
                        $(
                            else if next_try_pop_index == ($chain_index) {
                                output = self.$last_name.try_pop().await;
                            }
                        )*
                        else {
                            panic!("Index out of bound: next_try_pop_index");
                        }

                        if output.is_some() {
                            return output;
                        }

                        try_pop_attempt_count += 1;
                    }

                    // if the parallel set of ChainLinks have been exhausted
                    return None;
                }
                async fn process(&self) -> bool {
                    let mode = stringify!($mode);
                    let choice = stringify!($choice);
                    match mode {
                        "join" => {
                            match choice {
                                "all" => {
                                    self.process_all_join().await
                                },
                                "one" => {
                                    self.process_one_join().await
                                },
                                "random" => {
                                    self.process_random_join().await
                                }
                                _ => {
                                    panic!("Unexpected choice {}", choice);
                                }
                            }
                        },
                        "free" => {
                            match choice {
                                "all" => {
                                    self.process_all_free().await
                                },
                                "one" => {
                                    self.process_one_free().await
                                },
                                "random" => {
                                    self.process_random_free().await
                                }
                                _ => {
                                    panic!("Unexpected choice {}", choice);
                                }
                            }
                        },
                        "unique" => {
                            match choice {
                                "all" => {
                                    self.process_all_unique().await
                                },
                                "one" => {
                                    self.process_one_unique().await
                                },
                                "random" => {
                                    self.process_random_unique().await
                                }
                                _ => {
                                    panic!("Unexpected choice {}", choice);
                                }
                            }
                        },
                        _ => {
                            panic!("Unexpected mode {}", mode);
                        }
                    }
                }
            }
        }
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
                pub $first_name: std::sync::Arc<tokio::sync::RwLock<[<$first Initializer>]>>,
                $(
                    pub [<$($field)*>]: std::sync::Arc<tokio::sync::RwLock<[<$field_type Initializer>]>>,
                )*
                pub $last_name: std::sync::Arc<tokio::sync::RwLock<[<$last Initializer>]>>
            }

            impl [<$name Initializer>] {
                pub fn new($first_name: [<$first Initializer>], $([<$($field)*>]: [<$field_type Initializer>],)* $last_name: [<$last Initializer>]) -> Self {
                    [<$name Initializer>] {
                        $first_name: std::sync::Arc::new(tokio::sync::RwLock::new($first_name)),
                        $(
                            [<$($field)*>]: std::sync::Arc::new(tokio::sync::RwLock::new([<$($field)*>])),
                        )*
                        $last_name: std::sync::Arc::new(tokio::sync::RwLock::new($last_name))
                    }
                }
            }

            impl $name {
                pub async fn new(initializer: std::sync::Arc<tokio::sync::RwLock<[<$name Initializer>]>>) -> Self {
                    $name {
                        $first_name: $first::new(initializer.read().await.$first_name.clone()).await,
                        $(
                            [<$($field)*>]: $field_type::new(initializer.read().await.[<$($field)*>].clone()).await,
                        )*
                        $last_name: $last::new(initializer.read().await.$last_name.clone()).await
                    }
                }
                pub async fn new_raw(initializer: [<$name Initializer>]) -> Self {
                    $name::new(std::sync::Arc::new(tokio::sync::RwLock::new(initializer))).await
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    self.$first_name.push(input).await
                }
                async fn push_raw(&self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    self.$first_name.push_if_empty(input).await
                }
                async fn push_raw_if_empty(&self, input: $from) -> () {
                    self.push_if_empty(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<$to>>> {
                    self.$last_name.try_pop().await
                }
                async fn process(&self) -> bool {
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
        split_merge!(middle $name, $from, $to, false, false, () (0) () (x) () (), $($destination),*);
    };
    ($name:ty, $from:ty => $to:ty, ($($destination:ty),*), join) => {
        split_merge!(middle $name, $from, $to, true, false, () (0) () (x) () (), $($destination),*);
    };
    ($name:ty, $from:ty => $to:ty, ($($destination:ty),*), unique) => {
        split_merge!(middle $name, $from, $to, false, true, () (0) () (x) () (), $($destination),*);
    };
    (middle $name:ty, $from:ty, $to:ty, $is_join:expr, $is_unique:expr, ($($bool:tt)*) ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty) => {
        paste::paste! {
            split_merge!(end $name, $from, $to, $is_join, $is_unique, ($($bool)* [false]) ($index + 1) ($($index_past)* [$index]) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]));
        }
    };
    (middle $name:ty, $from:ty, $to:ty, $is_join:expr, $is_unique:expr, ($($bool:tt)*) ($index:expr) ($($index_past:tt)*) ($($prefix:tt)*) ($($past:tt)*) ($($past_type:tt)*), $next:ty, $($destination:ty),*) => {
        paste::paste! {
            split_merge!(middle $name, $from, $to, $is_join, $is_unique, ($($bool)* [false]) ($index + 1) ($($index_past)* [$index]) ($($prefix)* x) ($($past)* [$($prefix)* _ $next:snake]) ($($past_type)* [$next]), $($destination),*);
        }
    };
    (end $name:ident, $from:ty, $to:ty, $is_join:expr, $is_unique:expr, ($([$bool:tt])*) ($count:expr) ($([$index:expr])*) ($([$($field:tt)*])*) ($([$field_type:ty])*)) => {
        paste::paste! {
            pub struct $name {
                $(
                    [<$($field)*>]: std::sync::Arc<$field_type>,
                )*
                next_send_field_index: tokio::sync::Mutex<usize>,
                $(
                    [<is_running_ $($field)*>]: std::sync::Arc<tokio::sync::Mutex<bool>>,
                )*
            }

            pub struct [<$name Initializer>] {
                $(
                    pub [<$($field)* _initializer>]: std::sync::Arc<tokio::sync::RwLock<[<$field_type Initializer>]>>,
                )*
            }

            impl [<$name Initializer>] {
                pub fn new($([<$($field)* _initializer>]: [<$field_type Initializer>],)*) -> Self {
                    [<$name Initializer>] {
                        $(
                            [<$($field)* _initializer>]: std::sync::Arc::new(tokio::sync::RwLock::new([<$($field)* _initializer>])),
                        )*
                    }
                }
            }

            impl $name {
                pub async fn new(initializer: std::sync::Arc<tokio::sync::RwLock<[<$name Initializer>]>>) -> Self {
                    $name {
                        $(
                            [<$($field)*>]: std::sync::Arc::new($field_type::new(initializer.read().await.[<$($field)* _initializer>].clone()).await),
                        )*
                        next_send_field_index: tokio::sync::Mutex::new(0),
                        $(
                            [<is_running_ $($field)*>]: std::sync::Arc::new(tokio::sync::Mutex::new(false)),
                        )*
                    }
                }
                pub async fn new_raw(initializer: [<$name Initializer>]) -> Self {
                    $name::new(std::sync::Arc::new(tokio::sync::RwLock::new(initializer))).await
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    futures::join!($(self.[<$($field)*>].push(input.clone())),*);
                }
                async fn push_raw(&self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    futures::join!($(self.[<$($field)*>].push_if_empty(input.clone())),*);
                }
                async fn push_raw_if_empty(&self, input: $from) -> () {
                    self.push_if_empty(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<$to>>> {

                    // loop until we have found `Some` or looped around all internal ChainLink instanes
                    let mut next_send_field_index_lock = self.next_send_field_index.lock().await;
                    let mut send_attempts_count: usize = 0;
                    while send_attempts_count < ($count) {

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
                async fn process(&self) -> bool {
                    if $is_join {
                        let bool_tuple = futures::join!($(self.[<$($field)*>].process()),*);
                        let false_tuple = ($($bool),*);
                        return bool_tuple != false_tuple;
                    }
                    else if $is_unique {
                        $(
                            {
                                let mut [<locked_is_running_ $($field)*>] = self.[<is_running_ $($field)*>].lock().await;
                                if !*[<locked_is_running_ $($field)*>] {
                                    *[<locked_is_running_ $($field)*>] = true;
                                    let [<$($field)*>] = self.[<$($field)*>].clone();
                                    let [<is_running_ $($field)*>] = self.[<is_running_ $($field)*>].clone();
                                    std::thread::spawn(move || {
                                        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                            .enable_time()
                                            .build()
                                            .unwrap();

                                        tokio_runtime.block_on(async {
                                            [<$($field)*>].process().await;
                                            *[<is_running_ $($field)*>].lock().await = false;
                                        });
                                    });
                                }
                            }
                        )*
                        return false;
                    }
                    else {
                        $(
                            {
                                let [<$($field)*>] = self.[<$($field)*>].clone();
                                std::thread::spawn(move || {
                                    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                        .enable_time()
                                        .build()
                                        .unwrap();

                                    tokio_runtime.block_on(async {
                                        [<$($field)*>].process().await;
                                    });
                                });
                            }
                        )*
                        return false;
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! duplicate {
    ($name:ty, $from:ty => $to:ty, $duplicate:ty) => {
        duplicate!(end $name, $from => $to, $duplicate, false, false)
    };
    ($name:ty, $from:ty => $to:ty, $duplicate:ty, join) => {
        duplicate!(end $name, $from => $to, $duplicate, true, false)
    };
    ($name:ty, $from:ty => $to:ty, $duplicate:ty, unique) => {
        duplicate!(end $name, $from => $to, $duplicate, false, true)
    };
    (end $name:ty, $from:ty => $to:ty, $duplicate:ty, $is_join:expr, $is_unique:expr) => {
        paste::paste! {
            pub struct $name {
                next_send_field_index: tokio::sync::Mutex<usize>,
                inner_chainlinks: std::vec::Vec<std::sync::Arc<$duplicate>>,
                is_running_inner_chainlinks: std::vec::Vec<std::sync::Arc<tokio::sync::Mutex<bool>>>
            }

            pub struct [<$name Initializer>] {
                count: u32,
                inner_initializer: std::sync::Arc<tokio::sync::RwLock<[<$duplicate Initializer>]>>
            }

            impl [<$name Initializer>] {
                pub fn new(count: u32, initializer: [<$duplicate Initializer>]) -> Self {
                    [<$name Initializer>] {
                        count,
                        inner_initializer: std::sync::Arc::new(tokio::sync::RwLock::new(initializer))
                    }
                }
            }

            impl $name {
                pub async fn new(initializer: std::sync::Arc<tokio::sync::RwLock<[<$name Initializer>]>>) -> Self {
                    let mut inner_chainlinks = vec![];
                    let mut is_running_inner_chainlinks = vec![];
                    for _ in 0..(initializer.read().await.count) {
                        inner_chainlinks.push(std::sync::Arc::new($duplicate::new(initializer.read().await.inner_initializer.clone()).await));
                        is_running_inner_chainlinks.push(std::sync::Arc::new(tokio::sync::Mutex::new(false)));
                    }
                    Self {
                        next_send_field_index: tokio::sync::Mutex::new(0),
                        inner_chainlinks,
                        is_running_inner_chainlinks
                    }
                }
                pub async fn new_raw(initializer: [<$name Initializer>]) -> Self {
                    $name::new(std::sync::Arc::new(tokio::sync::RwLock::new(initializer))).await
                }
            }

            #[async_trait::async_trait]
            impl $crate::chain::ChainLink for $name {
                type TInput = $from;
                type TOutput = $to;

                async fn push(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    for chainlink in self.inner_chainlinks.iter() {
                        $crate::chain::ChainLink::push(chainlink.as_ref(), input.clone()).await;
                        //chainlink.push(input.clone()).await;
                    }
                }
                async fn push_raw(&self, input: $from) -> () {
                    self.push(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn push_if_empty(&self, input: std::sync::Arc<tokio::sync::RwLock<$from>>) -> () {
                    for chainlink in self.inner_chainlinks.iter() {
                        $crate::chain::ChainLink::push_if_empty(chainlink.as_ref(), input.clone()).await;
                    }
                }
                async fn push_raw_if_empty(&self, input: $from) -> () {
                    self.push_if_empty(std::sync::Arc::new(tokio::sync::RwLock::new(input))).await
                }
                async fn try_pop(&self) -> Option<std::sync::Arc<tokio::sync::RwLock<$to>>> {

                    // loop until we have found `Some` or looped around all internal ChainLink in
                    let mut next_send_field_index_lock = self.next_send_field_index.lock().await;
                    let mut send_attempts_count: usize = 0;
                    while send_attempts_count < self.inner_chainlinks.len() {

                        // get the next field index to check
                        let next_send_field_index: usize;
                        next_send_field_index = *next_send_field_index_lock;
                        if next_send_field_index + 1 == self.inner_chainlinks.len() {
                            *next_send_field_index_lock = 0;
                        }
                        else {
                            *next_send_field_index_lock = next_send_field_index + 1;
                        }

                        // get the output for the current field index
                        let output = self.inner_chainlinks[next_send_field_index].try_pop().await;

                        // return the output if `Some`, else try to loop again
                        if output.is_some() {
                            return output;
                        }

                        send_attempts_count += 1;
                    }

                    // if we've exhausted all internal `ChainLink` instances, return None
                    return None;
                }
                async fn process(&self) -> bool {
                    if $is_join {
                        let mut future_collection = vec![];
                        for chainlink in self.inner_chainlinks.iter() {
                            future_collection.push(chainlink.process());
                        }
                        let outcome = futures::future::join_all(future_collection).await;
                        for index in 0..self.inner_chainlinks.len() {
                            let indexed_bool_tuple: bool = *outcome.get(index).expect(&format!("The tuple index {} should exist within the futures::join! of the chainlink processes of length {}.", index, self.inner_chainlinks.len()));
                            if indexed_bool_tuple {
                                return true;
                            }
                        }
                        return false;
                    }
                    else if $is_unique {
                        for (index, chainlink) in self.inner_chainlinks.iter().enumerate() {
                            let mut locked_is_running_inner_chainlink = self.is_running_inner_chainlinks[index].lock().await;
                            if !*locked_is_running_inner_chainlink {
                                *locked_is_running_inner_chainlink = true;
                                let inner_chainlink = chainlink.clone();
                                let is_running_inner_chainlink = self.is_running_inner_chainlinks[index].clone();
                                std::thread::spawn(move || {
                                    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                        .enable_time()
                                        .build()
                                        .unwrap();

                                    tokio_runtime.block_on(async {
                                        inner_chainlink.process().await;
                                        *is_running_inner_chainlink.lock().await = false;
                                    });
                                });
                            }
                        }
                        return false;
                    }
                    else {
                        self.inner_chainlinks
                            .iter()
                            .for_each(|c| {
                                let inner_chainlink = c.clone();
                                std::thread::spawn(move || {
                                    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                                        .enable_time()
                                        .build()
                                        .unwrap();

                                    tokio_runtime.block_on(async {
                                        inner_chainlink.process().await;
                                    });
                                });
                            });
                        return false;
                    }
                }
            }
        }
    };
}