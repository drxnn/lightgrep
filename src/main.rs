use clap::Parser;

extern crate num_cpus;

mod types;
mod utils;

use lightgrep::{Args, Config, FileResult, ThreadPool, count_matches, process_lines, run};

use std::error::Error;

use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config: Config = match args.try_into() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }

    Ok(())
}
