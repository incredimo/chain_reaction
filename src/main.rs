 
 use std::{fs::DirEntry, path::{Path, PathBuf}};

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

 pub fn append(y: Vec<i32>) -> impl Fn(Vec<i32>) -> Out<Vec<i32>> {
     move |x| Ok(x.into_iter().chain(y.clone().into_iter()).collect())
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

        let input = vec![1, 2, 3, 4, 5];
        let result = Reactor::input(input)
        .then(append(vec![55,68]))
        .for_each(|x : i32| Ok(x.abs()))
        .run(); 

        println!("{:?}", result);

        //now lets use chain_eractor to extract data from a folder of files
        let data = Reactor::input(Path::new("."))
        .then(|x: &Path| x.read_dir())
        .for_each(|x: Result<DirEntry, std::io::Error>| Ok(x.unwrap())  )
        .run();

        println!("{:?}", data);
 }
 