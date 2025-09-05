use clap::Parser;

extern crate num_cpus;

mod types;
mod utils;

use drep::{
    Args, Config, FileResult, ThreadPool, count_matches, print_results, process_batch,
    process_lines,
};

use std::env;
use std::error::Error;

use std::process;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

use walkdir::DirEntry;
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let config: Config = args.into();
    let start = Instant::now();

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
    let duration = start.elapsed();
    println!("Finished in {:?}", duration);

    Ok(())
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_counter = Arc::new(Mutex::new(0));
    let current = env::current_dir().unwrap();
    const BATCH_SIZE: usize = 128;
    let num_of_cpus = num_cpus::get();
    let pool_size = if num_of_cpus > 1 { num_of_cpus - 1 } else { 1 };
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let (tx, rx) = mpsc::channel::<FileResult>();

    let config = Arc::new(config);
    let file_counter_clone = Arc::clone(&file_counter);
    let thread_pool = ThreadPool::new(pool_size, file_counter_clone);

    if config.recursive {
        let mut entry: DirEntry;

        for entry_walkdir in WalkDir::new(current) {
            entry = entry_walkdir?;

            batch.push(entry);

            if batch.len() == BATCH_SIZE {
                let config = Arc::clone(&config);
                let tx = tx.clone();
                thread_pool.execute(move || {
                    let _ = process_batch(batch, tx, config, false);
                });
                batch = Vec::with_capacity(BATCH_SIZE); // reset batch
            }
        }
        // if less than 25 files, send the remaining
        if !batch.is_empty() {
            let tx = tx.clone();
            let config = Arc::clone(&config);
            thread_pool.execute(move || {
                let _ = process_batch(batch, tx, config, false);
            });
        }
        drop(tx);
        drop(thread_pool);

        print_results(rx, config);
    } else {
        let entry = match WalkDir::new(&config.file_path)
            .max_depth(1)
            .into_iter()
            .next()
        {
            Some(Ok(e)) => e,
            Some(Err(e)) => {
                eprintln!("Error reading directory: {}", e);
                return Ok(());
            }
            None => {
                eprintln!("Entry was not found in current directory");
                return Ok(());
            }
        };

        batch.push(entry);

        {
            let tx = tx.clone();
            let config = Arc::clone(&config);
            let _ = process_batch(batch, tx, config, true);
        } // dropping config to use later
        drop(tx);
        drop(thread_pool);
        print_results(rx, config);

        // println!("The number of processed files was: {}", &file_counter_clone);
    }

    Ok(())
}
