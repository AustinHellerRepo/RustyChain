# Rusty Chain
This library abstracts over functional processing units called chain links and chains. Each link in the chain is meant to be independent, immutable, idempotent, parallelizable, and highly testable.

## Features

- A `ChainLink` is an independent processing unit that receives an input and sends an output.
  - By using the `chain_link!` macro you can quickly construct the internals of the mapping from input to output.
- A chain is a concatenation of `ChainLink`s (and other chains) and is a natural extension of this methodology for processing.
  - By using the `chain!` macro you can concatenate `ChainLink`s created by `chain_link!`, `chain!`, or `split_merge!`.
- A `split_merge!` macro permits parallel processing multiple `ChainLink` implementations, round-robin iterating over them per `process` invocation.
  - If a `ChainLink` `try_pop` returns `None`, it will try the next one, etc.

## Usage

You will want to determine what the smallest unit of processing your project consists of so that you can begin to create `ChainLink`s. Defend the quality of your `ChainLink`s by creating rigorous unit tests. After you have created a few `ChainLink`s bring it all together with a `chain!`.

Each type of processing unit (created by the `chain_link!`, `chain!`, and `split_merge!` macro) accept in an optional initializer, allowing for dependency injection. Now, it is possible to share dependencies between `ChainLink`s of a chain, but that is highly discouraged without unit tests around the `ChainLink` constructed by using the `chain!` macro.

## Examples

### Mapper

This example demonstrates how a `ChainLink` may exist to pull records from a database and map them to a model. The pushing of IDs is designed to push faster into the `ChainLink` than the pops occur to pull out the data. The database purposely takes longer to demonstrate how the system behaves.

### ETL

This example demonstrates how a file-loaded ETL process could be separated out into three `ChainLink`s, all connected together as a `Chain`, allowing you to pass in file paths and get back at the end if the current line processed was successful.
This example also covers basic usage of the `nom` crate and how the initializer can be used as a mutable buffer.

### ETL Split

This example is exactly like the ETL example, only that it also demonstrates splitting the final output between two databases using the `split_merge!` macro.

## Inspiration

I have always wanted highly testable code and to work in an environment where the logic of my processes was absolutely dependable.

## Future work

- split_merge! conditions
  - This would allow the `send` from one `ChainLink` to make its way to different destination `ChainLink`s based on a conditional block per destination, allowing logical, asynchronous splitting of processing.