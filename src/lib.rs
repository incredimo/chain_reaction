//! # chain_reaction
//! 
//! This library provides a flexible and composable way to build data processing pipelines.
//! It includes the `Reactor` and `TimedReactor` structs for building and executing these pipelines,
//! along with various traits and helper functions.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;

/// A type alias for the Result type used throughout the library.
/// `O` is the output type, and `E` is the error type (defaulting to `Failure`).
pub type Out<O, E = Failure> = Result<O, E>;

/// The `Act` trait defines the core behavior for all actions in the Reactor pipeline.
pub trait Act<I, O, E = Failure> {
    /// Performs the action on the input and returns the result.
    fn act(&self, input: I) -> Out<O, E>;
}

/// The `ChainableAct` trait extends `Act` with the ability to chain actions together.
pub trait ChainableAct<I, O, E = Failure>: Act<I, O, E> 
where
    Self: Sized,
{
    /// Chains this action with another, creating a new `Chain` action.
    fn then<O2, T>(self, transform: T) -> Chain<Self, T, I, O, O2, E>
    where
        T: Act<O, O2, E>,
    {
        Chain {
            first: self,
            second: transform,
            _marker: PhantomData,
        }
    }
}

/// The `Chain` struct represents a sequence of two actions.
pub struct Chain<A, B, I, O1, O2, E> 
where 
    A: Act<I, O1, E>,
    B: Act<O1, O2, E>,
{
    first: A,
    second: B,
    _marker: PhantomData<(I, O1, O2, E)>,
}

impl<A, B, I, O1, O2, E> Act<I, O2, E> for Chain<A, B, I, O1, O2, E>
where
    A: Act<I, O1, E>,
    B: Act<O1, O2, E>,
{
    fn act(&self, input: I) -> Out<O2, E> {
        self.first.act(input).and_then(|o1| self.second.act(o1))
    }
}

/// Implement `ChainableAct` for all types that implement `Act`.
impl<I, O, E, F> ChainableAct<I, O, E> for F
where
    F: Act<I, O, E>,
{}

/// Implement `Act` for all functions that take an input and return an `Out`.
impl<I, O, E, F> Act<I, O, E> for F
where
    F: Fn(I) -> Out<O, E>,
{
    fn act(&self, input: I) -> Out<O, E> {
        self(input)
    }
}

/// An enum representing either a left or right value.
#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// The `Reactor` struct represents a data processing pipeline.
pub struct Reactor<I, E = Failure> {
    input: Out<I, E>,
}

/// The `TimedReactor` struct extends `Reactor` with timing capabilities.
pub struct TimedReactor<I, E = Failure> {
    reactor: Rc<RefCell<Reactor<I, E>>>,
    timings: Rc<RefCell<HashMap<String, Duration>>>,
    current_operation: Rc<RefCell<Option<String>>>,
    start_time: Rc<RefCell<Option<Instant>>>,
}

impl<I, E> Reactor<I, E>
where
    E: Debug,
{
    /// Creates a new `Reactor` with the given input.
    pub fn input(input: I) -> Self {
        Self { input: Ok(input) }
    }

    /// Applies a transformation to the current input.
    pub fn then<O, T>(&mut self, transform: T) -> Reactor<O, E>
    where
        T: ChainableAct<I, O, E>,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(|i| transform.act(i)),
        }
    }

    /// Applies one of two transformations based on a condition.
    pub fn if_else<O1, O2, C, T1, T2>(
        &mut self,
        condition: C,
        true_transform: T1,
        false_transform: T2,
    ) -> Reactor<Either<O1, O2>, E>
    where
        C: Fn(&I) -> bool,
        T1: ChainableAct<I, O1, E>,
        T2: ChainableAct<I, O2, E>,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(|i| {
                if condition(&i) {
                    true_transform.act(i).map(Either::Left)
                } else {
                    false_transform.act(i).map(Either::Right)
                }
            }),
        }
    }

    /// Applies a transformation to each item in an iterable input.
    pub fn for_each<O, T>(&mut self, transform: T) -> Reactor<Vec<O>, E>
    where
        I: IntoIterator,
        T: ChainableAct<I::Item, O, E> + Clone,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(|i| {
                i.into_iter()
                    .map(|item| transform.act(item))
                    .collect::<Result<Vec<_>, _>>()
            }),
        }
    }

    /// Applies a function to the current input.
    pub fn map<O, F>(&mut self, f: F) -> Reactor<O, E>
    where
        F: FnOnce(I) -> O,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.map(f),
        }
    }

    /// Applies a fallible function to the current input.
    pub fn and_then<O, F>(&mut self, f: F) -> Reactor<O, E>
    where
        F: FnOnce(I) -> Result<O, E>,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(f),
        }
    }

    /// Merges the first two items of an iterable input using the provided function.
    pub fn merge<O, F>(&mut self, f: F) -> Reactor<O, E>
    where
        I: IntoIterator,
        I::Item: Clone,
        F: Fn(I::Item, I::Item) -> O,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(|i| {
                let mut iter = i.into_iter();
                match (iter.next(), iter.next()) {
                    (Some(a), Some(b)) => Ok(f(a, b)),
                    _ => panic!("Merge operation requires at least two items")
                }
            }),
        }
    }

    /// Runs the reactor and returns the final result.
    pub fn run(&mut self) -> Out<I, E> {
        mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }))
    }
}

impl<I, E> TimedReactor<I, E>
where
    E: Debug,
{
    /// Creates a new `TimedReactor` with the given input.
    pub fn input(input: I) -> Self {
        Self {
            reactor: Rc::new(RefCell::new(Reactor::input(input))),
            timings: Rc::new(RefCell::new(HashMap::new())),
            current_operation: Rc::new(RefCell::new(None)),
            start_time: Rc::new(RefCell::new(None)),
        }
    }

    /// Starts timing for the current operation.
    fn start_timing(&self, operation: &str) {
        *self.current_operation.borrow_mut() = Some(operation.to_string());
        *self.start_time.borrow_mut() = Some(Instant::now());
    }

    /// Ends timing for the current operation and records the duration.
    fn end_timing(&self) {
        if let (Some(operation), Some(start_time)) = (
            self.current_operation.borrow_mut().take(),
            self.start_time.borrow_mut().take(),
        ) {
            let duration = start_time.elapsed();
            self.timings.borrow_mut().insert(operation, duration);
        }
    }

    /// Applies a transformation to the current input and records the timing.
    pub fn then<O, T>(self, transform: T) -> TimedReactor<O, E>
    where
        T: Act<I, O, E>,
    {
        self.start_timing("then");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().then(transform)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Applies one of two transformations based on a condition and records the timing.
    pub fn if_else<O1, O2, C, T1, T2>(
        self,
        condition: C,
        true_transform: T1,
        false_transform: T2,
    ) -> TimedReactor<Either<O1, O2>, E>
    where
        C: Fn(&I) -> bool,
        T1: Act<I, O1, E>,
        T2: Act<I, O2, E>,
    {
        self.start_timing("if_else");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().if_else(condition, true_transform, false_transform)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Applies a transformation to each item in an iterable input and records the timing.
    pub fn for_each<O, T>(self, transform: T) -> TimedReactor<Vec<O>, E>
    where
        I: IntoIterator,
        T: Act<I::Item, O, E> + Clone,
    {
        self.start_timing("for_each");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().for_each(transform)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Applies a function to the current input and records the timing.
    pub fn map<O, F>(self, f: F) -> TimedReactor<O, E>
    where
        F: FnOnce(I) -> O,
    {
        self.start_timing("map");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().map(f)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Applies a fallible function to the current input and records the timing.
    pub fn and_then<O, F>(self, f: F) -> TimedReactor<O, E>
    where
        F: FnOnce(I) -> Result<O, E>,
    {
        self.start_timing("and_then");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().and_then(f)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Merges the first two items of an iterable input using the provided function and records the timing.
    pub fn merge<O, F>(self, f: F) -> TimedReactor<O, E>
    where
        I: IntoIterator,
        I::Item: Clone,
        F: Fn(I::Item, I::Item) -> O,
    {
        self.start_timing("merge");
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().merge(f)));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

    /// Runs the reactor and returns the final result along with the recorded timings.
    pub fn run(self) -> (Out<I, E>, HashMap<String, Duration>) {
        (self.reactor.borrow_mut().run(), self.timings.borrow().clone())
    }
}

/// A generic enum representing various types of failures that can occur in the Reactor pipeline.
#[derive(Debug)]
pub enum Failure {
    /// Represents an invalid input value.
    InvalidInput(String),
    /// Represents an arithmetic error (e.g., division by zero).
    ArithmeticError(String),
    /// Represents a custom error with a message.
    Custom(String),
}

/// Adds one to the input number.
pub fn add_one(x: i32) -> Out<i32> {
    Ok(x + 1)
}

/// Squares the input number, returning an error for negative inputs.
pub fn square(x: i32) -> Out<i32> {
    if x < 0 {
        Err(Failure::InvalidInput("Negative input for square function".to_string()))
    } else {
        Ok(x * x)
    }
}

/// Converts the input number to a string.
pub fn to_string(x: i32) -> Out<String> {
    Ok(x.to_string())
}

/// Doubles the input number.
pub fn double(x: i32) -> Out<i32> {
    Ok(x * 2)
}

/// Divides the first number by the second, returning an error for division by zero.
pub fn divide(x: i32, y: i32) -> Out<i32> {
    if y == 0 {
        Err(Failure::ArithmeticError("Division by zero".to_string()))
    } else {
        Ok(x / y)
    }
}
