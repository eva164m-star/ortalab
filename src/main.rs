use std::{
    error::Error,
    fs::File,
    io::{self, Read, stdin},
    path::{Path, PathBuf},
};

use clap::Parser;
use ortalib::{Chips, Mult, Round};

mod poker;
mod scoring;
mod ui;

#[derive(Parser)]
#[command(about = "Score an Ortalab round from YAML")]
struct Opts {
    /// YAML input file. Use '-' to read from standard input.
    file: Option<PathBuf>,

    /// Start a small terminal menu instead of scoring one file.
    #[arg(short, long)]
    interactive: bool,

    #[arg(long)]
    explain: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    if opts.interactive {
        return ui::run();
    }

    let file = opts.file.as_deref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing input file: provide a YAML path, '-' for stdin, or use --interactive",
        )
    })?;
    let _ = opts.explain;
    let round = parse_round(file)?;
    let (chips, mult) = score(&round);

    println!("{}", (chips * mult).floor());
    Ok(())
}

fn parse_round(path: &Path) -> Result<Round, Box<dyn Error>> {
    let mut input = String::new();
    if path == Path::new("-") {
        stdin().read_to_string(&mut input)?;
    } else {
        File::open(path)?.read_to_string(&mut input)?;
    }

    Ok(serde_yaml::from_str(&input)?)
}

fn score(round: &Round) -> (Chips, Mult) {
    scoring::score_round(round)
}
