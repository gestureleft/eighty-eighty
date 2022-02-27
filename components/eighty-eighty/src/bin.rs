use std::fs;

fn main() -> Result<(), eighty_eighty::Error> {
    println!("Executable!");

    let source = std::env::args().nth(1).expect("no source file given");
    let contents = fs::read(source).expect("failed to open source");

    eighty_eighty::emulate(contents)?;

    Ok(())
}
