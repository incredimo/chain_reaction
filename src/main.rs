use ex::*;

pub fn main() {
    // Example 1: Basic chaining with timing
    let (result, timings) = TimedReactor::input(5)
        .then(add_one)
        .if_else(|x| *x % 2 == 0, square.then(|x| double(x)).then(add_one), double)
        .run();
    
    match result {
        Ok(Either::Left(square_result)) => println!("Squared: {}", square_result),
        Ok(Either::Right(double_result)) => println!("Doubled: {}", double_result),
        Err(e) => println!("An error occurred: {:?}", e),
    }
    println!("Timings: {:?}", timings);

    // Example 2: Complex chaining with timing
    let (complex_result, complex_timings) = TimedReactor::input(10)
        .then(add_one)
        .then(square.then(add_one))
        .and_then(|x| divide(x, 3))
        .map(|x| x.to_string())
        .and_then(|s| {
            if s.len() > 2 {
                Ok(s)
            } else {
                Err(Failure::Custom("Result string too short".to_string()))
            }
        })
        .run();

    match complex_result {
        Ok(result) => println!("Complex operation result: {}", result),
        Err(e) => println!("An error occurred in complex operation: {:?}", e),
    }
    println!("Complex operation timings: {:?}", complex_timings);

    // Example 3: Using merge
    let (merge_result, merge_timings) = TimedReactor::input(vec![100, 200, 300, 400])
        .merge(|a, b| a + b)
        .then(square)
        .run();

    match merge_result {
        Ok(result) => println!("Merge and square result: {}", result),
        Err(e) => println!("An error occurred in merge operation: {:?}", e),
    }
    println!("Merge operation timings: {:?}", merge_timings);


    let x = square.then(add_one).then(|x| Ok(x*2) ).act(2);
    println!("{:?}", x);
}