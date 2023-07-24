#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};
    use tokio::sync::RwLock;
    use crate::chain::ChainLink;

    #[derive(Debug, PartialEq)]
    pub enum SomeInput {
        First,
        Second
    }

    chain_link!(TestChainLink, input:SomeInput => String, {
        match input.received {
            Some(received) => {
                Some(match *received.read().await {
                    SomeInput::First => {
                        String::from("first")
                    },
                    SomeInput::Second => {
                        String::from("second")
                    }
                })
            },
            None => None
        }
    });

    chain_link!(StringToSomeInput, input:String => SomeInput, {
        match input.received {
            Some(received) => {
                Some(match received.read().await.as_str() {
                    "first" => SomeInput::First,
                    "second" => SomeInput::Second,
                    _ => panic!("Unexpected value")
                })
            },
            None => None
        }
    });

    chain_link!(HardCoded => (text: String), input:() => String, {
        match input.received {
            Some(_received) => {
                Some(input.initializer.read().await.text.clone())
            },
            None => None
        }
    });

    chain!(ChainTest, SomeInput => SomeInput, TestChainLink => StringToSomeInput);

    chain!(TripleTest, SomeInput => String, TestChainLink => StringToSomeInput => TestChainLink);

    // chaining two chains
    chain!(ChainToChain, SomeInput => String, ChainTest => TripleTest);
    chain!(ChainToChainToLink, SomeInput => SomeInput, ChainTest => TripleTest => StringToSomeInput);

    chain_link!(StringToInt, input: String => i32, {
        match input.received {
            Some(received) => {
                Some(if received.read().await.as_str() == "test" {
                    1
                }
                else {
                    2
                })
            },
            None => {
                None
            }
        }
    });
    chain_link!(StringPrint, input: String => i32, {
        match input.received {
            Some(received) => {
                println!("{}", received.read().await);
                Some(0)
            },
            None => {
                None
            }
        }
    });
    
    split_merge!(SplitMergeTwoChainLinks, String => i32, (StringToInt, StringPrint), all join);
    split_merge!(SplitMergeMultiple, String => i32, (StringToInt, SplitMergeTwoChainLinks, StringPrint), all join);

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_link_enum_to_string() {
        let test = TestChainLink::new_raw(
            TestChainLinkInitializer { }
        ).await;
        let value = Arc::new(RwLock::new(SomeInput::Second));
        test.push(value).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("second", response.read().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_link_using_initializer() {
        let test = HardCoded::new_raw(
            HardCodedInitializer {
                text: String::from("test")
            }
        ).await;
        test.push_raw(()).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("test", Arc::try_unwrap(response).unwrap().into_inner().as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_enum_to_enum() {
        let chain_test = ChainTest::new_raw(
            ChainTestInitializer::new(
                TestChainLinkInitializer { },
                StringToSomeInputInitializer { }
            )
        ).await;
        let value = Arc::new(RwLock::new(SomeInput::Second));
        chain_test.push(value).await;
        chain_test.process().await;
        let response = chain_test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(SomeInput::Second, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_enum_to_string_to_enum() {
        let triple_test = TripleTest::new_raw(
            TripleTestInitializer::new(
                TestChainLinkInitializer { },
                StringToSomeInputInitializer { },
                TestChainLinkInitializer { }
            )
        ).await;
        let value = Arc::new(RwLock::new(SomeInput::First));
        triple_test.push(value).await;
        triple_test.process().await;
        let response = triple_test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("first", response.read().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_to_chain() {
        let test = ChainToChain::new_raw(
            ChainToChainInitializer::new(
                ChainTestInitializer::new(
                    TestChainLinkInitializer { },
                    StringToSomeInputInitializer { }
                ),
                TripleTestInitializer::new(
                    TestChainLinkInitializer { },
                    StringToSomeInputInitializer { },
                    TestChainLinkInitializer { }
                )
            )
        ).await;
        let value = Arc::new(RwLock::new(SomeInput::First));
        test.push(value).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("first", response.read().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_to_chain_to_chain_link() {
        let test = ChainToChainToLink::new_raw(
            ChainToChainToLinkInitializer::new(
                ChainTestInitializer::new(
                    TestChainLinkInitializer { },
                    StringToSomeInputInitializer { }
                ),
                TripleTestInitializer::new(
                    TestChainLinkInitializer { },
                    StringToSomeInputInitializer { },
                    TestChainLinkInitializer { }
                ),
                StringToSomeInputInitializer { }
            )
        ).await;
        let value = Arc::new(RwLock::new(SomeInput::Second));
        test.push(value).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(SomeInput::Second, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn split_to_two_chain_links() {
        let test = SplitMergeTwoChainLinks::new_raw(
            SplitMergeTwoChainLinksInitializer::new(
                StringToIntInitializer { },
                StringPrintInitializer { }
            )
        ).await;
        test.push(Arc::new(RwLock::new(String::from("test")))).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(1, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(0, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                panic!("Unexpected Some response with value {}.", response.read().await);
            },
            None => {
                // expected path
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn split_to_two_chain_links_round_robin_finds_flushed_chainlink() {
        let test = SplitMergeMultiple::new_raw(
            SplitMergeMultipleInitializer::new(
                StringToIntInitializer { },
                SplitMergeTwoChainLinksInitializer::new(
                    StringToIntInitializer { },
                    StringPrintInitializer { }
                ),
                StringPrintInitializer { }
            )
        ).await;
        test.push(Arc::new(RwLock::new(String::from("test")))).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(1, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(1, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(0, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!(0, Arc::try_unwrap(response).unwrap().into_inner());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                panic!("Unexpected Some response with value {}.", response.read().await);
            },
            None => {
                // expected path
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn duplicate_locking() {

        chain_link!(IsUppercase, input: String => bool, {
            match input.received {
                // we will pretend that it takes a little bit of time to check
                Some(text) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let text = text.read().await;
                    if text.is_empty() {
                        Some(false)
                    }
                    else {
                        let at_least_one_lowercase_letter = text
                            .chars()
                            .any(|c| c.is_lowercase());
                        Some(!at_least_one_lowercase_letter)
                    }
                },
                None => None
            }
        });

        duplicate!(ParallelIsUppercase, String => bool, IsUppercase, join);

        let dup = ParallelIsUppercase::new_raw(
            ParallelIsUppercaseInitializer::new(
                2,
                IsUppercaseInitializer { }
            )
        ).await;

        dup.push_raw(String::from("test")).await;
        dup.push_raw(String::from("TEST")).await;

        dup.process().await;
        dup.process().await;

        let output = dup.try_pop().await;
        assert!(output.is_some());

        let output = dup.try_pop().await;
        assert!(output.is_some());
    }
}