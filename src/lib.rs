use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub type Out<O, E = Failure> = Result<O, E>;

pub trait Act<I, O, E = Failure> {
    fn act(&self, input: I) -> Out<O, E>;
    fn run(&self, input: I) -> O {
        match self.act(input) {
            Ok(output) => output,
            Err(e) => panic!("Error: "),
        }
    }
}

pub trait ChainableAct<I, O, E = Failure>: Act<I, O, E>
where
    Self: Sized,
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

impl<I, O, E, F> ChainableAct<I, O, E> for F where F: Act<I, O, E> {}

impl<I, O, E, F> Act<I, O, E> for F
where
    F: Fn(I) -> Out<O, E>,
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

impl<I, E> TimedReactor<I, E>
where
    E: Debug,
{
    pub fn input(input: I) -> Self {
        Self {
            reactor: Rc::new(RefCell::new(Reactor::input(input))),
            timings: Rc::new(RefCell::new(HashMap::new())),
            current_operation: Rc::new(RefCell::new(None)),
            start_time: Rc::new(RefCell::new(None)),
        }
    }

    fn start_timing(&self, operation: &str) {
        *self.current_operation.borrow_mut() = Some(operation.to_string());
        *self.start_time.borrow_mut() = Some(Instant::now());
    }

    fn end_timing(&self) {
        if let (Some(operation), Some(start_time)) = (
            self.current_operation.borrow_mut().take(),
            self.start_time.borrow_mut().take(),
        ) {
            let duration = start_time.elapsed();
            self.timings.borrow_mut().insert(operation, duration);
        }
    }

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
        let new_reactor = Rc::new(RefCell::new(self.reactor.borrow_mut().if_else(
            condition,
            true_transform,
            false_transform,
        )));
        self.end_timing();
        TimedReactor {
            reactor: new_reactor,
            timings: Rc::clone(&self.timings),
            current_operation: Rc::clone(&self.current_operation),
            start_time: Rc::clone(&self.start_time),
        }
    }

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

    pub fn run(self) -> (Out<I, E>, HashMap<String, Duration>) {
        (
            self.reactor.borrow_mut().run(),
            self.timings.borrow().clone(),
        )
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

// Modified example functions to work with the new implementation
pub fn add(y: i32) -> impl Fn(i32) -> Out<i32> {
    move |x| Ok(x + y)
}

pub fn square() -> impl Fn(i32) -> Out<i32> {
    |x| {
        if x < 0 {
            Err(Failure::InvalidInput(
                "Negative input for square function".to_string(),
            ))
        } else {
            Ok(x * x)
        }
    }
}

pub fn to_string() -> impl Fn(i32) -> Out<String> {
    |x| Ok(x.to_string())
}

pub fn double() -> impl Fn(i32) -> Out<i32> {
    |x| Ok(x * 2)
}

pub fn divide(y: i32) -> impl Fn(i32) -> Out<i32> {
    move |x| {
        if y == 0 {
            Err(Failure::ArithmeticError("Division by zero".to_string()))
        } else {
            Ok(x / y)
        }
    }
}
