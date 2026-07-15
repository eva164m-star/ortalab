use std::{
    error::Error,
    io::{self, Write},
    path::Path,
};

use crate::{parse_round, score};

/// Runs an optional terminal interface without changing the assignment's
/// original command-line file-scoring behaviour.
pub fn run() -> Result<(), Box<dyn Error>> {
    println!("========================");
    println!("        ORTALAB");
    println!("========================");

    loop {
        println!("\n1) Score a YAML file");
        println!("2) Quit");
        let choice = prompt("Select an option: ")?;

        match choice.trim() {
            "1" => score_file_interactively()?,
            "2" | "q" | "quit" => {
                println!("Goodbye.");
                return Ok(());
            }
            _ => eprintln!("Please enter 1 or 2."),
        }
    }
}

fn score_file_interactively() -> Result<(), Box<dyn Error>> {
    let path = prompt("YAML file path: ")?;
    let path = path.trim();
    if path.is_empty() {
        eprintln!("No file path was entered.");
        return Ok(());
    }

    match parse_round(Path::new(path)) {
        Ok(round) => {
            let (chips, mult) = score(&round);
            println!("Chips: {chips}");
            println!("Mult:  {mult}");
            println!("Score: {}", (chips * mult).floor());
        }
        Err(error) => eprintln!("Could not score '{path}': {error}"),
    }
    Ok(())
}

fn prompt(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
