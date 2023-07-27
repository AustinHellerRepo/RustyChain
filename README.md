# Rusty Chain
This library abstracts over functional processing units represented as chainlinks. Each link in the chain is meant to be independent, idempotent, largely immutable, and highly testable.

## Features

- A `ChainLink` is an independent processing unit that receives an input and sends an output.
  - By using the `chain_link!` macro you can quickly construct the internals of the mapping from input to output.
- A chain is a concatenation of `ChainLink`s (and other chains) and is a natural extension of this methodology for processing.
  - By using the `chain!` macro you can concatenate `ChainLink`s created by `chain_link!` or `chain!`.
- A `chain!` macro permits parallel processing multiple `ChainLink` implementations, round-robin iterating over them per `process` invocation.
  - If a `ChainLink` `try_pop` returns `None`, it will try the next one, etc.

## Usage

You will want to determine what the smallest unit of processing your project consists of so that you can begin to create `ChainLink`s. Defend the quality of your `ChainLink`s by creating rigorous unit tests. After you have created a few `ChainLink`s bring it all together with a `chain!`.

Each type of processing unit (created by the `chain_link!` macro) accept in an optional initializer, allowing for dependency injection. Now, it is possible to share dependencies between `ChainLink`s of a chain, but that is highly discouraged without unit tests around the `ChainLink` constructed by using the `chain!` macro.

## Examples

### Mapper

This example demonstrates how a `ChainLink` may exist to pull records from a database and map them to a model. The pushing of IDs is designed to push faster into the `ChainLink` than the pops occur to pull out the data. The database purposely takes longer to demonstrate how the system behaves asynchronously, pulling from the database while accepting in new IDs.

### ETL

This example demonstrates how a file-loaded ETL process could be separated out into three `ChainLink`s, all connected together as a `Chain`, allowing you to pass in file paths and get back at the end if the current line processed was successful.
This example also covers basic usage of the `nom` crate and how the initializer can be used as a mutable buffer.

### ETL Split

This example is exactly like the ETL example, only that it also demonstrates splitting the final output between two databases using the parallel functionality of the `chain!` macro.

### Madlib

This example demonstrates that an earlier `ChainLink` may take in a group of input that will need to be parsed individually in a later `ChainLink`. In other words, aggregation upstream can be merged together downstream.

### Robotics

This example demonstrates usage of the `chain!` macro in a context where we might want one asynchronous process to run alongside another asynchronous process but such that they are not waiting for each other to complete before input is generally processed. Here, we want the controller to quickly be able to shutdown the robot while the camera sensor may take a while to provide data.

### Fibonacci

This example demonstrates how iterative processes can be utilized, especially with regards to mathematical operations.

### Work Order

This example demonstrates a simple order/work management system where customer orders and worker availability is paired up as they are provided. In a true work management system, the cache would exist in a database.

## Inspiration

I have always wanted highly testable code and to work in an environment where the logic of my processes was absolutely dependable.

## Future work

- chain! parallel conditions
  - This would allow the `try_pop` from one `ChainLink` to make its way to different destination `ChainLink`s based on a conditional block per destination, allowing logical, asynchronous splitting of processing.

- chain! nested sets
  - The idea would be that you could do the following: chain!(SomeChain, String => String, [SomeChainLink => [OneSplit, AnotherSplit]: (one join) => FinalChainLink]: (all join))
