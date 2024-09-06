use chain_reaction::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = TimedReactor::input(5)
        .then(add(1))
        .then(square())
        .then(double())
        .then(divide(2))
        .then(to_string())
        .run();

    match result {
        (Ok(output), timings) => {
            println!("Final result: {}", output);
            println!("Timings: {:?}", timings);
        }
        (Err(e), _) => println!("Error: {:?}", e),
    }

    let s = add(5).then(divide(6)).act(1111).unwrap();
    print!("{}", s);

    Ok(())
}
