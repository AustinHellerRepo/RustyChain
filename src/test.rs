use std::sync::{Arc, Mutex};
use crate::chain::ChainLink;

#[derive(Debug, PartialEq)]
enum SomeInput {
    First,
    Second
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

chain!(ChainTest, SomeInput => SomeInput, TestChainLink => StringToSomeInput);

chain!(TripleTest, SomeInput => String, TestChainLink => StringToSomeInput => TestChainLink);


// chaining two chains
chain!(ChainToChain, SomeInput => String, ChainTest => TripleTest);
chain!(ChainToChainToLink, SomeInput => SomeInput, ChainTest => TripleTest => StringToSomeInput);

#[tokio::test(flavor = "multi_thread")]
async fn chain_link_enum_to_string() {
    let mut test = TestChainLink::default();
    let value = Arc::new(Mutex::new(SomeInput::Second));
    test.receive(value).await;
    test.poll().await;
    let response = test.send().await;
    match response {
        Some(response) => {
            assert_eq!("second", response.lock().unwrap().as_str());
        },
        None => {
            panic!("Unexpected None response.");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn chain_enum_to_enum() {
    let mut chain_test = ChainTest::default();
    let value = Arc::new(Mutex::new(SomeInput::Second));
    chain_test.receive(value).await;
    chain_test.poll().await;
    let response = chain_test.send().await;
    match response {
        Some(response) => {
            assert_eq!(SomeInput::Second, Arc::try_unwrap(response).unwrap().into_inner().unwrap());
        },
        None => {
            panic!("Unexpected None response.");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn chain_enum_to_string_to_enum() {
    let mut triple_test = TripleTest::default();
    let value = Arc::new(Mutex::new(SomeInput::First));
    triple_test.receive(value).await;
    triple_test.poll().await;
    let response = triple_test.send().await;
    match response {
        Some(response) => {
            assert_eq!("first", response.lock().unwrap().as_str());
        },
        None => {
            panic!("Unexpected None response.");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn chain_to_chain() {
    let mut chain_to_chain = ChainToChain::default();
    let value = Arc::new(Mutex::new(SomeInput::First));
    chain_to_chain.receive(value).await;
    chain_to_chain.poll().await;
    let response = chain_to_chain.send().await;
    match response {
        Some(response) => {
            assert_eq!("first", response.lock().unwrap().as_str());
        },
        None => {
            panic!("Unexpected None response.");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn chain_to_chain_to_chain_link() {
    let mut chain_to_chain_to_link = ChainToChainToLink::default();
    let value = Arc::new(Mutex::new(SomeInput::Second));
    chain_to_chain_to_link.receive(value).await;
    chain_to_chain_to_link.poll().await;
    let response = chain_to_chain_to_link.send().await;
    match response {
        Some(response) => {
            assert_eq!(SomeInput::Second, Arc::try_unwrap(response).unwrap().into_inner().unwrap());
        },
        None => {
            panic!("Unexpected None response.");
        }
    }
}