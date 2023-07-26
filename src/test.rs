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

    chain!(ChainTest,
        SomeInput => SomeInput,
        [
            TestChainLink => StringToSomeInput
        ]: (all join)
    );

    chain!(TripleTest,
        SomeInput => String,
        [
            TestChainLink => StringToSomeInput => TestChainLink
        ]: (all join)
    );

    // chaining two chains
    chain!(ChainToChain,
        SomeInput => String,
        [
            ChainTest => TripleTest
        ]: (all join)
    );

    chain!(ChainToChainToLink,
        SomeInput => SomeInput,
        [
            ChainTest => TripleTest => StringToSomeInput
        ]: (all join)
    );

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
    
    chain!(SplitMergeTwoChainLinks,
        String => i32,
        [
            StringToInt,
            StringPrint
        ]: (all join)
    );

    chain!(SplitMergeMultiple,
        String => i32,
        [
            StringToInt,
            SplitMergeTwoChainLinks,
            StringPrint
        ]: (all join)
    );

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

        chain!(SoloAllJoin, String => String, [ToLower]: (all join));
        chain!(TwoSplitAllJoin, String => String, [ToLower, ToUpper]: (all join));
        chain!(ThreeSplitAllJoin, String => String, [ToLower, ToUpper, ToUpper]: (all join));
        chain!(FourSplitAllJoin, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (all join));
        chain!(TwoChainAllJoin, String => String, [ToLower => ToUpper]: (all join));
        chain!(ThreeChainAllJoin, String => String, [ToLower => ToUpper => ToUpper]: (all join));
        chain!(FourChainAllJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (all join));
        chain!(FourChainSoloAllJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (all join));
        chain!(FourChainTwoChainAllJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (all join));
        chain!(FourChainSoloTwoChainAllJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (all join));
        chain!(TwoChainTwoChainAllJoin, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (all join));

        chain!(SoloOneJoin, String => String, [ToLower]: (one join));
        chain!(TwoSplitOneJoin, String => String, [ToLower, ToUpper]: (one join));
        chain!(ThreeSplitOneJoin, String => String, [ToLower, ToUpper, ToUpper]: (one join));
        chain!(FourSplitOneJoin, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (one join));
        chain!(TwoChainOneJoin, String => String, [ToLower => ToUpper]: (one join));
        chain!(ThreeChainOneJoin, String => String, [ToLower => ToUpper => ToUpper]: (one join));
        chain!(FourChainOneJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (one join));
        chain!(FourChainSoloOneJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (one join));
        chain!(FourChainTwoChainOneJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (one join));
        chain!(FourChainSoloTwoChainOneJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (one join));
        chain!(TwoChainTwoChainOneJoin, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (one join));

        chain!(SoloRandomJoin, String => String, [ToLower]: (random join));
        chain!(TwoSplitRandomJoin, String => String, [ToLower, ToUpper]: (random join));
        chain!(ThreeSplitRandomJoin, String => String, [ToLower, ToUpper, ToUpper]: (random join));
        chain!(FourSplitRandomJoin, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (random join));
        chain!(TwoChainRandomJoin, String => String, [ToLower => ToUpper]: (random join));
        chain!(ThreeChainRandomJoin, String => String, [ToLower => ToUpper => ToUpper]: (random join));
        chain!(FourChainRandomJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (random join));
        chain!(FourChainSoloRandomJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (random join));
        chain!(FourChainTwoChainRandomJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (random join));
        chain!(FourChainSoloTwoChainRandomJoin, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (random join));
        chain!(TwoChainTwoChainRandomJoin, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (random join));

        chain!(SoloAllFree, String => String, [ToLower]: (all free));
        chain!(TwoSplitAllFree, String => String, [ToLower, ToUpper]: (all free));
        chain!(ThreeSplitAllFree, String => String, [ToLower, ToUpper, ToUpper]: (all free));
        chain!(FourSplitAllFree, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (all free));
        chain!(TwoChainAllFree, String => String, [ToLower => ToUpper]: (all free));
        chain!(ThreeChainAllFree, String => String, [ToLower => ToUpper => ToUpper]: (all free));
        chain!(FourChainAllFree, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (all free));
        chain!(FourChainSoloAllFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (all free));
        chain!(FourChainTwoChainAllFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (all free));
        chain!(FourChainSoloTwoChainAllFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (all free));
        chain!(TwoChainTwoChainAllFree, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (all free));

        chain!(SoloOneFree, String => String, [ToLower]: (one free));
        chain!(TwoSplitOneFree, String => String, [ToLower, ToUpper]: (one free));
        chain!(ThreeSplitOneFree, String => String, [ToLower, ToUpper, ToUpper]: (one free));
        chain!(FourSplitOneFree, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (one free));
        chain!(TwoChainOneFree, String => String, [ToLower => ToUpper]: (one free));
        chain!(ThreeChainOneFree, String => String, [ToLower => ToUpper => ToUpper]: (one free));
        chain!(FourChainOneFree, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (one free));
        chain!(FourChainSoloOneFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (one free));
        chain!(FourChainTwoChainOneFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (one free));
        chain!(FourChainSoloTwoChainOneFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (one free));
        chain!(TwoChainTwoChainOneFree, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (one free));

        chain!(SoloRandomFree, String => String, [ToLower]: (random free));
        chain!(TwoSplitRandomFree, String => String, [ToLower, ToUpper]: (random free));
        chain!(ThreeSplitRandomFree, String => String, [ToLower, ToUpper, ToUpper]: (random free));
        chain!(FourSplitRandomFree, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (random free));
        chain!(TwoChainRandomFree, String => String, [ToLower => ToUpper]: (random free));
        chain!(ThreeChainRandomFree, String => String, [ToLower => ToUpper => ToUpper]: (random free));
        chain!(FourChainRandomFree, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (random free));
        chain!(FourChainSoloRandomFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (random free));
        chain!(FourChainTwoChainRandomFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (random free));
        chain!(FourChainSoloTwoChainRandomFree, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (random free));
        chain!(TwoChainTwoChainRandomFree, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (random free));

        chain!(SoloAllUnique, String => String, [ToLower]: (all unique));
        chain!(TwoSplitAllUnique, String => String, [ToLower, ToUpper]: (all unique));
        chain!(ThreeSplitAllUnique, String => String, [ToLower, ToUpper, ToUpper]: (all unique));
        chain!(FourSplitAllUnique, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (all unique));
        chain!(TwoChainAllUnique, String => String, [ToLower => ToUpper]: (all unique));
        chain!(ThreeChainAllUnique, String => String, [ToLower => ToUpper => ToUpper]: (all unique));
        chain!(FourChainAllUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (all unique));
        chain!(FourChainSoloAllUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (all unique));
        chain!(FourChainTwoChainAllUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (all unique));
        chain!(FourChainSoloTwoChainAllUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (all unique));
        chain!(TwoChainTwoChainAllUnique, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (all unique));

        chain!(SoloOneUnique, String => String, [ToLower]: (one unique));
        chain!(TwoSplitOneUnique, String => String, [ToLower, ToUpper]: (one unique));
        chain!(ThreeSplitOneUnique, String => String, [ToLower, ToUpper, ToUpper]: (one unique));
        chain!(FourSplitOneUnique, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (one unique));
        chain!(TwoChainOneUnique, String => String, [ToLower => ToUpper]: (one unique));
        chain!(ThreeChainOneUnique, String => String, [ToLower => ToUpper => ToUpper]: (one unique));
        chain!(FourChainOneUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (one unique));
        chain!(FourChainSoloOneUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (one unique));
        chain!(FourChainTwoChainOneUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (one unique));
        chain!(FourChainSoloTwoChainOneUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (one unique));
        chain!(TwoChainTwoChainOneUnique, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (one unique));

        chain!(SoloRandomUnique, String => String, [ToLower]: (random unique));
        chain!(TwoSplitRandomUnique, String => String, [ToLower, ToUpper]: (random unique));
        chain!(ThreeSplitRandomUnique, String => String, [ToLower, ToUpper, ToUpper]: (random unique));
        chain!(FourSplitRandomUnique, String => String, [ToLower, ToUpper, ToUpper, ToLower]: (random unique));
        chain!(TwoChainRandomUnique, String => String, [ToLower => ToUpper]: (random unique));
        chain!(ThreeChainRandomUnique, String => String, [ToLower => ToUpper => ToUpper]: (random unique));
        chain!(FourChainRandomUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower]: (random unique));
        chain!(FourChainSoloRandomUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower]: (random unique));
        chain!(FourChainTwoChainRandomUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToLower => ToLower]: (random unique));
        chain!(FourChainSoloTwoChainRandomUnique, String => String, [ToLower => ToUpper => ToUpper => ToLower, ToUpper, ToLower => ToLower]: (random unique));
        chain!(TwoChainTwoChainRandomUnique, String => String, [ToLower => ToUpper, ToUpper => ToLower]: (random unique));
    }
}