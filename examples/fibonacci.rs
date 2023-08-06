
// This example demonstrates a recursive example

use fibonacci::sequence::{FibonacciSequence, FibonacciSequenceInitializer};
use rusty_chain::framework::ChainLink;

mod fibonacci {

    pub mod sequence {
        use rusty_chain::chain_link;

        chain_link!(FibonacciSequence, input: (u32, u32) => (u32, u32), {
            if let Some(fib_tuple) = input.received {
                let (a, b) = &*fib_tuple.read().await;
                Some((*b, *a + *b))
            }
            else {
                None
            }
        });
    }
}

#[tokio::main]
async fn main() {
    
    let fibonacci_sequence = FibonacciSequence::new_raw(
        FibonacciSequenceInitializer { }
    ).await;

    // push on the initial set
    fibonacci_sequence.push_raw((0, 1)).await;

    for i in 0..12 {
        // process the next set of numbers
        fibonacci_sequence.process().await;

        // pull out the next set of numbers
        if let Some(numbers) = fibonacci_sequence.try_pop().await {
            let locked_numbers = numbers.read().await;
            let (a, b) = (locked_numbers.0, locked_numbers.1);
            
            if i == 0 {
                println!("{}", a);
            }
            println!("{}", b);

            // push on the next set
            fibonacci_sequence.push_raw((a, b)).await;
        }
    }
}