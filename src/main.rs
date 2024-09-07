 
 use chain_reaction::*;
 // functions can do anything, as long as they return a Result<T, E>
 pub fn add(y: i32) -> impl Fn(i32) -> Out<i32> {
     move |x| Ok(x + y)
 }
 
 // let's say we have a function that squares a number, but it only works for non-negative numbers
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
 
 // let's say we have a function that converts a number to a string
 pub fn to_string() -> impl Fn(i32) -> Out<String> {
     |x| Ok(x.to_string())
 }
 
 // let's say we have a function that doubles a number
 pub fn double() -> impl Fn(i32) -> Out<i32> {
     |x| Ok(x * 2)
 }
 
 // let's say we have a function that divides two numbers
 pub fn divide(y: i32) -> impl Fn(i32) -> Out<i32> {
     move |x| {
         if y == 0 {
             Err(Failure::ArithmeticError("Division by zero".to_string()))
         } else {
             Ok(x / y)
         }
     }
 }

 
 fn main() {
 
 // we can chain them together like this:
 // 5 -> add(2) -> square() -> to_string() -> double()
 // in a type safe and composable way
     let input = 5;
     let result = Reactor::input(input)
         .then(add(2))
         .then(square())
         .then(double())
         .then(to_string())
         .run();

        println!("{:?}", result);
 }
 