#![allow(dead_code)]
use colored::Colorize;

extern crate num_cpus;

use crate::{Config, FileResult, process_lines};
use crate::{ThreadPool, count_matches};
use std::fs::{self, File};

use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use std::sync::mpsc;

use walkdir::DirEntry;

pub fn print_results(rx: mpsc::Receiver<FileResult>, config: Arc<Config>) {
    let mut total = 0;
    for file_response in rx {
        match file_response {
            FileResult::Match(n, v) => {
                let config = Arc::clone(&config);
                if config.count {
                    total += count_matches(&v);
                }
                for (key, value) in &v {
                    let config = Arc::clone(&config);
                    print_each_result(config, &n, (*key, value));
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
pub fn process_batch(
    batch: Vec<DirEntry>,
    tx: mpsc::Sender<FileResult>,
    config: Arc<Config>,
    single_file: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if single_file {
        let entry = batch
            .first()
            .expect("single_file mode requires exactly one entry in batch");

        let mut pool_size = num_cpus::get();
        pool_size = pool_size.saturating_sub(1);
        if pool_size == 0 {
            pool_size = 1;
        }

        let file_counter = Arc::new(AtomicUsize::new(0));
        let thread_pool = ThreadPool::new(pool_size, file_counter);

        let metadata = fs::metadata(entry.path())?;
        let file_size_bytes = metadata.len();
        let chunk_size = (file_size_bytes + pool_size as u64) / pool_size as u64;

        let mut f = File::open(&config.file_path)?;
        let mut file_buffer = vec![0; f.metadata()?.len() as usize];
        f.read_exact(&mut file_buffer)?;
        let chunks = get_chunks(&file_buffer, chunk_size as usize);

        let mut line_offset = 0;
        for (start, end) in chunks {
            let config = Arc::clone(&config);
            let tx = tx.clone();

            let buffer = file_buffer[start..end].to_vec();

            let chunk_lines = buffer.iter().filter(|&&b| b == b'\n').count();

            thread_pool.execute(move || {
                let file_contents = String::from_utf8_lossy(&buffer);

                let temp: Vec<(usize, std::borrow::Cow<'_, str>)> = process_lines(
                    &config.pattern,
                    &file_contents,
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

                    if let Err(e) = tx.send(FileResult::Match(config.file_path.clone(), owned_temp))
                    {
                        eprintln!("failed to send chunk result: {:?}", e);
                    }
                }
            });
            line_offset += chunk_lines;
        }
    } else {
        for entry in batch {
            let res = (|| -> FileResult {
                if !entry.file_type().is_file() {
                    return FileResult::Skip;
                }

                let path = entry.path().to_path_buf();
                let bytes = match fs::read(&path) {
                    Ok(b) => b,
                    _ => {
                        return FileResult::Skip;
                    }
                };

                let file_contents = match std::str::from_utf8(&bytes) {
                    Ok(s) => s,
                    Err(_) => return FileResult::Skip,
                };

                let file_name = entry.file_name();

                if let Some(config_ext) = &config.file_extension {
                    let config_ext = normalize_extension(&config_ext);
                    let curr_ext = entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(normalize_extension);

                    if curr_ext != Some(config_ext) {
                        return FileResult::Skip;
                    }
                }

                let temp = process_lines(
                    &config.pattern,
                    &file_contents,
                    config.invert,
                    config.highlight,
                );

                if temp.is_empty() {
                    return FileResult::Skip;
                }

                let owned_temp: Vec<(usize, String)> = temp
                    .into_iter()
                    .map(|(idx, s)| (idx, s.to_string()))
                    .collect();

                let file_name_owned = file_name.to_string_lossy().into_owned();

                FileResult::Match(file_name_owned, owned_temp)
            })();
            if let Err(send_err) = tx.send(res) {
                eprintln!("failed to send result back to main: {:?}", send_err);
            }
        }
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
