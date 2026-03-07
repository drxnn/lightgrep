#![allow(dead_code)]
use colored::Colorize;
use memmap2::Mmap;

extern crate num_cpus;

use crate::{Config, FileResult, process_lines};
use crate::{ThreadPool, count_matches};
// use memmap2::Mmap;
use std::fs::{self, File};

use std::sync::Arc;

use std::sync::mpsc;
const LARGE_FILE_THRESHOLD: u64 = 1073741824;

use walkdir::DirEntry;

pub fn print_results(rx: mpsc::Receiver<FileResult>, config: Arc<Config>) {
    let mut total = 0;
    for file_response in rx {
        match file_response {
            FileResult::Match(n, v) => {
                let config = Arc::clone(&config);
                if config.count {
                    total += count_matches(&v);
                } else {
                    for (key, value) in &v {
                        let config = Arc::clone(&config);
                        print_each_result(config, &n, (*key, value));
                    }
                }
            }
            FileResult::Error(e) => eprintln!("Error: {}", e),
            FileResult::Skip => {}
        }
    }
    if config.count {
        println!("Number of matched lines found: {total}");
    }
}

pub fn normalize_extension(ext: &str) -> &str {
    ext.strip_prefix('.').unwrap_or(ext)
}

pub fn process_file(
    file: DirEntry,
    tx: mpsc::Sender<FileResult>,
    config: Arc<Config>,
    thread_pool: Arc<ThreadPool>,
    pool_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = fs::metadata(file.path())?;
    let file_size_bytes = metadata.len();
    let chunk_size = (file_size_bytes + pool_size as u64) / (pool_size * 8) as u64;

    let f = File::open(&file.path())?;
    let mmap = Arc::new(unsafe { Mmap::map(&f)? });

    if metadata.len() >= 32_000_000 {
        let mmap = Arc::clone(&mmap);
        let chunks = get_chunks(&mmap, chunk_size as usize);

        let mut line_offset = 0;
        for (start, end) in chunks {
            let config = Arc::clone(&config);
            let tx = tx.clone();
            let mmap = Arc::clone(&mmap);

            let chunk_lines = mmap[start..end].iter().filter(|&&b| b == b'\n').count();

            let file_path = file.path().to_string_lossy().to_string();

            thread_pool.execute(move || {
                let temp: Vec<(usize, std::borrow::Cow<'_, str>)> = process_lines(
                    &config.pattern,
                    &mmap[start..end],
                    config.invert,
                    config.highlight,
                )
                .into_iter()
                .map(|(idx, s)| (idx + line_offset, s))
                .collect();

                if !temp.is_empty() {
                    let owned_temp: Vec<(usize, String)> = temp
                        .into_iter()
                        .map(|(idx, s)| (idx, s.to_string()))
                        .collect();

                    if let Err(e) = tx.send(FileResult::Match(file_path, owned_temp)) {
                        eprintln!("failed to send chunk result: {:?}", e);
                    }
                }
            });
            line_offset += chunk_lines;
        }
    } else {
        let config = Arc::clone(&config);
        let tx = tx.clone();
        let mmap = Arc::clone(&mmap);
        let file_path = file.path().to_string_lossy().to_string();

        thread_pool.execute(move || {
            let temp: Vec<(usize, std::borrow::Cow<'_, str>)> =
                process_lines(&config.pattern, &mmap, config.invert, config.highlight);

            if !temp.is_empty() {
                let owned_temp: Vec<(usize, String)> = temp
                    .into_iter()
                    .map(|(idx, s)| (idx, s.to_string()))
                    .collect();

                if let Err(e) = tx.send(FileResult::Match(file_path, owned_temp)) {
                    eprintln!("failed to send chunk result: {:?}", e);
                }
            }
        });
    }

    Ok(())
}

pub fn print_each_result(config: Arc<Config>, name: &str, v: (usize, &String)) {
    if config.line_number {
        println!("{} - line: {}, {}", name.green(), v.0, v.1);
    } else {
        println!("{}: {}", name.green(), v.1);
    }
}

fn get_chunks(bytes: &[u8], chunk_size: usize) -> Vec<(usize, usize)> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let bytes_len = bytes.len();

    if chunk_size == 0 {
        return chunks;
    }

    while start < bytes_len {
        let mut end = std::cmp::min(start + chunk_size, bytes_len);

        while end < bytes_len && bytes[end] != b'\n' {
            end += 1;
        }
        if end < bytes_len {
            end += 1;
        }
        chunks.push((start, end));
        start = end;
    }

    chunks
}
