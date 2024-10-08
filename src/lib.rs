#![allow(unused_imports,unused_variables,dead_code, unused_braces, unused_import_braces)]
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug};
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};




/// #chain_reaction
/// chain reaction is a minimal crate that helps you chain together multiple functions that return Result<T, E>
/// and run them in a pipeline.
///
/// # Example
/// ```rust
/// use chain_reaction::*;
/// // functions can do anything, as long as they return a Result<T, E>
/// pub fn add(y: i32) -> impl Fn(i32) -> Out<i32> {
///     move |x| Ok(x + y)
/// }
/// 
/// // let's say we have a function that squares a number, but it only works for non-negative numbers
/// pub fn square() -> impl Fn(i32) -> Out<i32> {
///     |x| {
///         if x < 0 {
///             Err(Failure::InvalidInput(
///                 "Negative input for square function".to_string(),
///             ))
///         } else {
///             Ok(x * x)
///         }
///     }
/// }
/// 
/// // let's say we have a function that converts a number to a string
/// pub fn to_string() -> impl Fn(i32) -> Out<String> {
///     |x| Ok(x.to_string())
/// }
/// 
/// // let's say we have a function that doubles a number
/// pub fn double() -> impl Fn(i32) -> Out<i32> {
///     |x| Ok(x * 2)
/// }
/// 
/// // let's say we have a function that divides two numbers
/// pub fn divide(y: i32) -> impl Fn(i32) -> Out<i32> {
///     move |x| {
///         if y == 0 {
///             Err(Failure::ArithmeticError("Division by zero".to_string()))
///         } else {
///             Ok(x / y)
///         }
///     }
/// }
///
/// 
/// fn main() {
/// 
/// // we can chain them together like this:
/// // 5 -> add(2) -> square() -> to_string() -> double()
/// // in a type safe and composable way
///     let input = 5;
///     let result = Reactor::input(input)
///         .then(add(2))
///         .then(square())
///         .then(double())
///         .then(to_string())
///         .run();
/// }
/// ```
pub trait Act<I, O, E = Failure>
where
    E:  Debug,
{
    fn act(&self, input: I) -> Out<O, E>;
    fn run(&self, input: I) -> O {
        match self.act(input) {
            Ok(output) => output,
            Err(e) => panic!("Error: {:?}", e),
        }
    }
}

pub type Out<O, E = Failure> = Result<O, E>;

pub trait ChainableAct<I, O, E = Failure>: Act<I, O, E>
where
    Self: Sized,
    E: Debug,
{
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

pub struct Chain<A, B, I, O1, O2, E>
where
    A: Act<I, O1, E>,
    B: Act<O1, O2, E>,
    E: Debug,
{
    first: A,
    second: B,
    _marker: PhantomData<(I, O1, O2, E)>,
}

impl<A, B, I, O1, O2, E> Act<I, O2, E> for Chain<A, B, I, O1, O2, E>
where
    A: Act<I, O1, E>,
    B: Act<O1, O2, E>,
    E: Debug,
{
    fn act(&self, input: I) -> Out<O2, E> {
        self.first.act(input).and_then(|o1| self.second.act(o1))
    }
}

impl<I, O, E, F> ChainableAct<I, O, E> for F where F: Act<I, O, E>, E: Debug {}

impl<I, O, E, F> Act<I, O, E> for F
where
    F: Fn(I) -> Out<O, E>,
    E: Debug,
{
    fn act(&self, input: I) -> Out<O, E> {
        self(input)
    }
}

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub struct Reactor<I, E = Failure> {
    input: Out<I, E>,
}

 

impl<I, E> Reactor<I, E>
where
    E: Debug,
{
    pub fn input(input: I) -> Self {
        Self { input: Ok(input) }
    }

    pub fn then<O, T>(&mut self, transform: T) -> Reactor<O, E>
    where
        T: Act<I, O, E>,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(|i| transform.act(i)),
        }
    }


    

    pub fn if_else<O1, O2, C, T1, T2>(
        &mut self,
        condition: C,
        true_transform: T1,
        false_transform: T2,
    ) -> Reactor<Either<O1, O2>, E>
    where
        C: Fn(&I) -> bool,
        T1: Act<I, O1, E>,
        T2: Act<I, O2, E>,
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

    pub fn for_each<O, T>(&mut self, transform: T) -> Reactor<Vec<O>, E>
    where
        I: IntoIterator,
        T: Act<I::Item, O, E> + Clone,
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

    pub fn map<O, F>(&mut self, f: F) -> Reactor<O, E>
    where
        F: FnOnce(I) -> O,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.map(f),
        }
    }

    pub fn and_then<O, F>(&mut self, f: F) -> Reactor<O, E>
    where
        F: FnOnce(I) -> Result<O, E>,
    {
        let input = mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }));
        Reactor {
            input: input.and_then(f),
        }
    }

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
                    _ => panic!("Merge operation requires at least two items"),
                }
            }),
        }
    }

    pub fn run(&mut self) -> Out<I, E> {
        mem::replace(&mut self.input, Err(unsafe { std::mem::zeroed() }))
    }
}
 
#[derive(Debug)]
pub enum Failure {
    InvalidInput(String),
    ArithmeticError(String),
    Custom(String),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Failure::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            Failure::ArithmeticError(s) => write!(f, "Arithmetic error: {}", s),
            Failure::Custom(s) => write!(f, "Custom error: {}", s),
        }
    }
}

impl std::error::Error for Failure {}


