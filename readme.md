# ChainReaction

```rust
let increment = |x| Ok(x + 1);
let double = |x| Ok(x * 2);
let stringify = |x| Ok(x.to_string());

let result = TimedReactor::input(5)
    .then(increment)
    .then(double)
    .then(stringify)
    .run();

println!("{:?}", result); // Output: Ok("12")
```

Welcome to **ChainReaction** – a powerful Rust crate designed to simplify complex data processing pipelines, workflows, and real-time systems by providing seamless chaining of operations with built-in error handling and precise timing.

---

## Overview

**ChainReaction** allows you to compose sequences of operations (or "actions") that can be chained together to form complex workflows. Each operation can handle errors gracefully, and the timing of each operation is automatically tracked and recorded, enabling detailed performance analysis.

### Key Features:
- **Seamless Chaining**: Easily chain multiple operations together, with the output of one operation feeding directly into the next.
- **Error Handling**: Built-in error handling ensures that errors are managed gracefully throughout the chain.
- **Timing Mechanism**: Automatically track the duration of each operation in the chain, making performance analysis straightforward.
- **Flexible Usage**: Suitable for a wide range of applications, including data processing, business workflows, automated testing, and real-time systems.

## Getting Started

### Installation

Add ChainReaction to your `Cargo.toml`:

```toml
[dependencies]
chain_reaction = "0.1.0"
```

### Basic Usage

Here's how you can get started with ChainReaction:

```rust
use chain_reaction::{TimedReactor, Failure};

let increment = |x| Ok(x + 1);
let double = |x| Ok(x * 2);
let stringify = |x| Ok(x.to_string());

let result = TimedReactor::input(5)
    .then(increment)
    .then(double)
    .then(stringify)
    .run();

match result {
    Ok(value) => println!("Final result: {}", value),
    Err(e) => println!("An error occurred: {:?}", e),
}
```

### Advanced Usage

#### Conditional Logic

ChainReaction allows you to execute different operations based on conditions using the `if_else` method.

```rust
let (result, timings) = TimedReactor::input(4)
    .if_else(
        |x| *x % 2 == 0,
        |x| Ok(x * 2),
        |x| Ok(x + 1),
    )
    .run();

match result {
    Ok(value) => println!("Conditional result: {}", value),
    Err(e) => println!("An error occurred: {:?}", e),
}
println!("Timings: {:?}", timings);
```

#### Iterating Over Collections

You can process collections of items using the `for_each` method.

```rust
let square = |x| Ok(x * x);

let (result, timings) = TimedReactor::input(vec![1, 2, 3, 4])
    .for_each(square)
    .run();

match result {
    Ok(values) => println!("Squared values: {:?}", values),
    Err(e) => println!("An error occurred: {:?}", e),
}
println!("Timings: {:?}", timings);
```

#### Merging Operations

ChainReaction supports merging operations from a collection, such as summing pairs of elements and then applying further transformations.

```rust
let sum = |a, b| a + b;
let square = |x| Ok(x * x);

let (result, timings) = TimedReactor::input(vec![10, 20, 30, 40])
    .merge(sum)
    .then(square)
    .run();

match result {
    Ok(value) => println!("Merge and square result: {}", value),
    Err(e) => println!("An error occurred: {:?}", e),
}
println!("Timings: {:?}", timings);
```

## Why ChainReaction?

ChainReaction offers a powerful combination of ease of use, error resilience, performance monitoring, and flexibility. It stands out by simplifying the construction of complex workflows and allowing you to focus on business logic without getting bogged down in error handling or performance issues.

Whether you're building data processing pipelines, complex business workflows, or real-time event processing systems, **ChainReaction** provides the tools you need to create efficient, maintainable, and robust systems.

