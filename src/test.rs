#[cfg(test)]
mod test {
    use std::sync::{Arc};
    use tokio::sync::Mutex;
    use crate::chain::ChainLink;

    #[derive(Debug, PartialEq)]
    pub enum SomeInput {
        First,
        Second
    }

    chain_link!(TestChainLink, input:SomeInput => String, {
        match input.received {
            Some(received) => {
                Some(match *received {
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
                Some(match received.as_str() {
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
                Some(if received.as_str() == "test" {
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
                println!("{}", received);
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
        let test = TestChainLink::new(TestChainLinkInitializer { });
        let value = Arc::new(Mutex::new(SomeInput::Second));
        test.push(value).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("second", response.lock().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_link_using_initializer() {
        let test = HardCoded::new(HardCodedInitializer { text: String::from("test") });
        test.push(Arc::new(Mutex::new(()))).await;
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
        let chain_test = ChainTest::new(ChainTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { } });
        let value = Arc::new(Mutex::new(SomeInput::Second));
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
        let triple_test = TripleTest::new(TripleTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { }, xxx_test_chain_link: TestChainLinkInitializer { } });
        let value = Arc::new(Mutex::new(SomeInput::First));
        triple_test.push(value).await;
        triple_test.process().await;
        let response = triple_test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("first", response.lock().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_to_chain() {
        let test = ChainToChain::new(ChainToChainInitializer { x_chain_test: ChainTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { } }, xx_triple_test: TripleTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { }, xxx_test_chain_link: TestChainLinkInitializer { } } });
        let value = Arc::new(Mutex::new(SomeInput::First));
        test.push(value).await;
        test.process().await;
        let response = test.try_pop().await;
        match response {
            Some(response) => {
                assert_eq!("first", response.lock().await.as_str());
            },
            None => {
                panic!("Unexpected None response.");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn chain_to_chain_to_chain_link() {
        let test = ChainToChainToLink::new(ChainToChainToLinkInitializer { x_chain_test: ChainTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { } }, xx_triple_test: TripleTestInitializer { x_test_chain_link: TestChainLinkInitializer { }, xx_string_to_some_input: StringToSomeInputInitializer { }, xxx_test_chain_link: TestChainLinkInitializer { } }, xxx_string_to_some_input: StringToSomeInputInitializer { } });
        let value = Arc::new(Mutex::new(SomeInput::Second));
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
        let test = SplitMergeTwoChainLinks::new(SplitMergeTwoChainLinksInitializer { x_string_to_int_initializer: StringToIntInitializer { }, xx_string_print_initializer: StringPrintInitializer { } });
        test.push(Arc::new(Mutex::new(String::from("test")))).await;
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
                panic!("Unexpected Some response with value {}.", response.lock().await);
            },
            None => {
                // expected path
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn split_to_two_chain_links_round_robin_finds_flushed_chainlink() {
        let test = SplitMergeMultiple::new(SplitMergeMultipleInitializer { x_string_to_int_initializer: StringToIntInitializer { }, xx_split_merge_two_chain_links_initializer: SplitMergeTwoChainLinksInitializer { x_string_to_int_initializer: StringToIntInitializer { }, xx_string_print_initializer: StringPrintInitializer { } }, xxx_string_print_initializer: StringPrintInitializer { } });
        test.push(Arc::new(Mutex::new(String::from("test")))).await;
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
                panic!("Unexpected Some response with value {}.", response.lock().await);
            },
            None => {
                // expected path
            }
        }
    }
}