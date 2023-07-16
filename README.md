# Rusty Chain
This library abstracts over functional processing units called `ChainLink`s and `Chain`s. Each link in the chain is meant to be independent, immutable, idempotent, parallelizable, and highly testable.

## Features

- A `ChainLink` is an independent processing unit that receives an input and sends an output.
  - By using the `chain_link!` macro you can quickly construct the internals of the mapping from input to output.
- A `Chain` is a concatenation of `ChainLink`s (and other `Chain`s) and is a natural extension of this methodology for processing.
  - By using the `chain!` macro you can concatenate both `ChainLink`s and `Chain`s naturally.
- A `split_merge!` macro permits parallel processing multiple `ChainLink` implementations, round-robin iterating over them per `send`.
  - If a `ChainLink` returns `None`, it will try the next one, etc.

## Usage

You will want to determine what the smallest unit of processing your project consists of so that you can begin to create `ChainLink`s. Defend the quality of your `ChainLink`s by creating rigorous unit tests. After you have created a few `ChainLink`s bring it all together with a `Chain`.

Each type of processing unit (`ChainLink` and `Chain`) accept in an optional initializer, allowing for dependency injection. Now, it is possible to share dependencies between `ChainLink`s of a `Chain`, but that is highly discouraged without unit tests around the `Chain`.

## Examples

### Mapper

This example demonstrates how a `ChainLink` may exist to pull records from a database and map them to a model.

## Inspiration

I have always wanted highly testable code and to work in an environment where the logic of my processes was absolutely dependable.

## Future work

- split_merge! conditions
  - This would allow the `send` from one `ChainLink` to make its way to different destination `ChainLink`s based on a conditional block per destination, allowing logical, asynchronous splitting of processing.