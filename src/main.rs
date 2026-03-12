use clap::Parser;

use lightgrep::{Args, Config, run};

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
