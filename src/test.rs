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
    
    split_merge!(SplitMergeTwoChainLinks, String => i32, (StringToInt, StringPrint), join);
    split_merge!(SplitMergeMultiple, String => i32, (StringToInt, SplitMergeTwoChainLinks, StringPrint), join);

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

    #[tokio::test(flavor = "multi_thread")]
    async fn new_chain() {

        chain_link!(ToLower, input: String => String, {
            match input.received {
                Some(text) => {
                    Some(text.read().await.to_lowercase())
                },
                None => None
            }
        });

        chain_link!(ToUpper, input: String => String, {
            match input.received {
                Some(text) => {
                    Some(text.read().await.to_uppercase())
                },
                None => None
            }
        });
        new_chain!(Solo, String => String, (ToLower) all join);
        new_chain!(TwoSplit, String => String, (ToLower, ToUpper) one free);
        new_chain!(ThreeSplit, String => String, (ToLower, ToUpper, ToUpper) random unique);
        new_chain!(FourSplit, String => String, (ToLower, ToUpper, ToUpper, ToLower) all free);
        new_chain!(TwoChain, String => String, (ToLower => ToUpper) one unique);
        new_chain!(ThreeChain, String => String, (ToLower => ToUpper => ToUpper) random join);
        new_chain!(FourChain, String => String, (ToLower => ToUpper => ToUpper => ToLower) random free);
        new_chain!(FourChainSolo, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) all unique);
        new_chain!(FourChainTwoChain, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) one join);
        new_chain!(FourChainSoloTwoChain, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) all join);
        new_chain!(TwoChainTwoChain, String => String, (ToLower => ToUpper, ToUpper => ToLower) all join);

        new_chain!(SoloAllJoin, String => String, (ToLower) all join);
        new_chain!(TwoSplitAllJoin, String => String, (ToLower, ToUpper) all join);
        new_chain!(ThreeSplitAllJoin, String => String, (ToLower, ToUpper, ToUpper) all join);
        new_chain!(FourSplitAllJoin, String => String, (ToLower, ToUpper, ToUpper, ToLower) all join);
        new_chain!(TwoChainAllJoin, String => String, (ToLower => ToUpper) all join);
        new_chain!(ThreeChainAllJoin, String => String, (ToLower => ToUpper => ToUpper) all join);
        new_chain!(FourChainAllJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower) all join);
        new_chain!(FourChainSoloAllJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) all join);
        new_chain!(FourChainTwoChainAllJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) all join);
        new_chain!(FourChainSoloTwoChainAllJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) all join);
        new_chain!(TwoChainTwoChainAllJoin, String => String, (ToLower => ToUpper, ToUpper => ToLower) all join);

        new_chain!(SoloOneJoin, String => String, (ToLower) one join);
        new_chain!(TwoSplitOneJoin, String => String, (ToLower, ToUpper) one join);
        new_chain!(ThreeSplitOneJoin, String => String, (ToLower, ToUpper, ToUpper) one join);
        new_chain!(FourSplitOneJoin, String => String, (ToLower, ToUpper, ToUpper, ToLower) one join);
        new_chain!(TwoChainOneJoin, String => String, (ToLower => ToUpper) one join);
        new_chain!(ThreeChainOneJoin, String => String, (ToLower => ToUpper => ToUpper) one join);
        new_chain!(FourChainOneJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower) one join);
        new_chain!(FourChainSoloOneJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) one join);
        new_chain!(FourChainTwoChainOneJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) one join);
        new_chain!(FourChainSoloTwoChainOneJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) one join);
        new_chain!(TwoChainTwoChainOneJoin, String => String, (ToLower => ToUpper, ToUpper => ToLower) one join);

        new_chain!(SoloRandomJoin, String => String, (ToLower) random join);
        new_chain!(TwoSplitRandomJoin, String => String, (ToLower, ToUpper) random join);
        new_chain!(ThreeSplitRandomJoin, String => String, (ToLower, ToUpper, ToUpper) random join);
        new_chain!(FourSplitRandomJoin, String => String, (ToLower, ToUpper, ToUpper, ToLower) random join);
        new_chain!(TwoChainRandomJoin, String => String, (ToLower => ToUpper) random join);
        new_chain!(ThreeChainRandomJoin, String => String, (ToLower => ToUpper => ToUpper) random join);
        new_chain!(FourChainRandomJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower) random join);
        new_chain!(FourChainSoloRandomJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) random join);
        new_chain!(FourChainTwoChainRandomJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) random join);
        new_chain!(FourChainSoloTwoChainRandomJoin, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) random join);
        new_chain!(TwoChainTwoChainRandomJoin, String => String, (ToLower => ToUpper, ToUpper => ToLower) random join);

        new_chain!(SoloAllFree, String => String, (ToLower) all free);
        new_chain!(TwoSplitAllFree, String => String, (ToLower, ToUpper) all free);
        new_chain!(ThreeSplitAllFree, String => String, (ToLower, ToUpper, ToUpper) all free);
        new_chain!(FourSplitAllFree, String => String, (ToLower, ToUpper, ToUpper, ToLower) all free);
        new_chain!(TwoChainAllFree, String => String, (ToLower => ToUpper) all free);
        new_chain!(ThreeChainAllFree, String => String, (ToLower => ToUpper => ToUpper) all free);
        new_chain!(FourChainAllFree, String => String, (ToLower => ToUpper => ToUpper => ToLower) all free);
        new_chain!(FourChainSoloAllFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) all free);
        new_chain!(FourChainTwoChainAllFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) all free);
        new_chain!(FourChainSoloTwoChainAllFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) all free);
        new_chain!(TwoChainTwoChainAllFree, String => String, (ToLower => ToUpper, ToUpper => ToLower) all free);

        new_chain!(SoloOneFree, String => String, (ToLower) one free);
        new_chain!(TwoSplitOneFree, String => String, (ToLower, ToUpper) one free);
        new_chain!(ThreeSplitOneFree, String => String, (ToLower, ToUpper, ToUpper) one free);
        new_chain!(FourSplitOneFree, String => String, (ToLower, ToUpper, ToUpper, ToLower) one free);
        new_chain!(TwoChainOneFree, String => String, (ToLower => ToUpper) one free);
        new_chain!(ThreeChainOneFree, String => String, (ToLower => ToUpper => ToUpper) one free);
        new_chain!(FourChainOneFree, String => String, (ToLower => ToUpper => ToUpper => ToLower) one free);
        new_chain!(FourChainSoloOneFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) one free);
        new_chain!(FourChainTwoChainOneFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) one free);
        new_chain!(FourChainSoloTwoChainOneFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) one free);
        new_chain!(TwoChainTwoChainOneFree, String => String, (ToLower => ToUpper, ToUpper => ToLower) one free);

        new_chain!(SoloRandomFree, String => String, (ToLower) random free);
        new_chain!(TwoSplitRandomFree, String => String, (ToLower, ToUpper) random free);
        new_chain!(ThreeSplitRandomFree, String => String, (ToLower, ToUpper, ToUpper) random free);
        new_chain!(FourSplitRandomFree, String => String, (ToLower, ToUpper, ToUpper, ToLower) random free);
        new_chain!(TwoChainRandomFree, String => String, (ToLower => ToUpper) random free);
        new_chain!(ThreeChainRandomFree, String => String, (ToLower => ToUpper => ToUpper) random free);
        new_chain!(FourChainRandomFree, String => String, (ToLower => ToUpper => ToUpper => ToLower) random free);
        new_chain!(FourChainSoloRandomFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) random free);
        new_chain!(FourChainTwoChainRandomFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) random free);
        new_chain!(FourChainSoloTwoChainRandomFree, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) random free);
        new_chain!(TwoChainTwoChainRandomFree, String => String, (ToLower => ToUpper, ToUpper => ToLower) random free);

        new_chain!(SoloAllUnique, String => String, (ToLower) all unique);
        new_chain!(TwoSplitAllUnique, String => String, (ToLower, ToUpper) all unique);
        new_chain!(ThreeSplitAllUnique, String => String, (ToLower, ToUpper, ToUpper) all unique);
        new_chain!(FourSplitAllUnique, String => String, (ToLower, ToUpper, ToUpper, ToLower) all unique);
        new_chain!(TwoChainAllUnique, String => String, (ToLower => ToUpper) all unique);
        new_chain!(ThreeChainAllUnique, String => String, (ToLower => ToUpper => ToUpper) all unique);
        new_chain!(FourChainAllUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower) all unique);
        new_chain!(FourChainSoloAllUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) all unique);
        new_chain!(FourChainTwoChainAllUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) all unique);
        new_chain!(FourChainSoloTwoChainAllUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) all unique);
        new_chain!(TwoChainTwoChainAllUnique, String => String, (ToLower => ToUpper, ToUpper => ToLower) all unique);

        new_chain!(SoloOneUnique, String => String, (ToLower) one unique);
        new_chain!(TwoSplitOneUnique, String => String, (ToLower, ToUpper) one unique);
        new_chain!(ThreeSplitOneUnique, String => String, (ToLower, ToUpper, ToUpper) one unique);
        new_chain!(FourSplitOneUnique, String => String, (ToLower, ToUpper, ToUpper, ToLower) one unique);
        new_chain!(TwoChainOneUnique, String => String, (ToLower => ToUpper) one unique);
        new_chain!(ThreeChainOneUnique, String => String, (ToLower => ToUpper => ToUpper) one unique);
        new_chain!(FourChainOneUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower) one unique);
        new_chain!(FourChainSoloOneUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) one unique);
        new_chain!(FourChainTwoChainOneUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) one unique);
        new_chain!(FourChainSoloTwoChainOneUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) one unique);
        new_chain!(TwoChainTwoChainOneUnique, String => String, (ToLower => ToUpper, ToUpper => ToLower) one unique);

        new_chain!(SoloRandomUnique, String => String, (ToLower) random unique);
        new_chain!(TwoSplitRandomUnique, String => String, (ToLower, ToUpper) random unique);
        new_chain!(ThreeSplitRandomUnique, String => String, (ToLower, ToUpper, ToUpper) random unique);
        new_chain!(FourSplitRandomUnique, String => String, (ToLower, ToUpper, ToUpper, ToLower) random unique);
        new_chain!(TwoChainRandomUnique, String => String, (ToLower => ToUpper) random unique);
        new_chain!(ThreeChainRandomUnique, String => String, (ToLower => ToUpper => ToUpper) random unique);
        new_chain!(FourChainRandomUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower) random unique);
        new_chain!(FourChainSoloRandomUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower) random unique);
        new_chain!(FourChainTwoChainRandomUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower) random unique);
        new_chain!(FourChainSoloTwoChainRandomUnique, String => String, (ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower) random unique);
        new_chain!(TwoChainTwoChainRandomUnique, String => String, (ToLower => ToUpper, ToUpper => ToLower) random unique);
    }
}