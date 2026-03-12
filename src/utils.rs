use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use colored::Colorize;
use memmap2::Mmap;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

extern crate num_cpus;

use crate::count_lines_with_matches;
use crate::{Config, FileResult, process_lines};

use std::error::Error;
use std::fs::{self, File};

use std::io::{self, BufWriter, StdoutLock, Write};
use std::sync::Arc;

use walkdir::DirEntry;

pub fn build_ac(patterns: &[String], ignore_case: bool) -> Result<AhoCorasick, Box<dyn Error>> {
    let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
    if ignore_case {
        Ok(AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&pattern_refs)?)
    } else {
        Ok(AhoCorasick::new(&pattern_refs)?)
    }
}

pub fn print_single_result(result: FileResult, config: Arc<Config>) -> io::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock());
    match result {
        FileResult::Match(n, v) => {
            let config = Arc::clone(&config);
            if config.count {
                // total += count_lines_with_matches(&v);
            } else {
                for (key, value) in &v {
                    print_each_result(&mut stdout, &config, &n, (*key, value))?;
                }
            }
        }
        FileResult::Error(e) => eprintln!("Error: {}", e),
        FileResult::Skip => {}
    }
    Ok(())
}
pub fn print_results(results: Vec<FileResult>, config: Arc<Config>) -> io::Result<()> {
    let mut total = 0;
    let mut stdout = BufWriter::new(io::stdout().lock());

    for file_response in results {
        match file_response {
            FileResult::Match(n, v) => {
                let config = Arc::clone(&config);
                if config.count {
                    total += count_lines_with_matches(&v);
                } else {
                    for (key, value) in &v {
                        print_each_result(&mut stdout, &config, &n, (*key, value))?;
                    }
                }
            }
            FileResult::Error(e) => eprintln!("Error: {}", e),
            FileResult::Skip => {}
        }
    }
    if config.count {
        println!("Number of lines with matches: {total}");
    }
    Ok(())
}

pub fn normalize_extension(ext: &str) -> &str {
    ext.strip_prefix('.').unwrap_or(ext)
}

pub fn process_file(
    file: DirEntry,
    config: Arc<Config>,
) -> Result<FileResult, Box<dyn std::error::Error>> {
    let metadata = file.metadata()?;
    let file_size_bytes = metadata.len();
    let chunk_size = (file_size_bytes + config.pool_size as u64) / (config.pool_size * 2) as u64;

    match file_size_bytes {
        0..=1_999_999 => {
            let f_bytes = fs::read(file.path())?;
            let temp: Vec<(usize, std::borrow::Cow<'_, str>)> =
                process_lines(&config.pattern, &f_bytes, config.invert, config.highlight);
            let owned_temp: Vec<(usize, String)> = temp
                .into_iter()
                .map(|(idx, s)| (idx, s.into_owned()))
                .collect();
            let file_path = file.path().to_string_lossy().to_string();
            return Ok(FileResult::Match(file_path, owned_temp));
        }

        _ => {
            let f = File::open(&file.path())?;
            let mmap = Arc::new(unsafe { Mmap::map(&f)? });

            let chunks = get_chunks(&mmap, chunk_size as usize * 2);
            let chunk_lines: Vec<usize> = if config.line_number {
                let counts: Vec<usize> = chunks
                    .par_iter()
                    .map(|&(start, end)| mmap[start..end].iter().filter(|&&b| b == b'\n').count())
                    .collect();
                counts
                    .iter()
                    .scan(0usize, |acc, &c| {
                        let curr = *acc;
                        *acc += c;
                        Some(curr)
                    })
                    .collect()
            } else {
                vec![0usize; chunks.len()]
            };

            let file_path = file.path().to_string_lossy().to_string();
            let matched: Vec<(usize, String)> = chunks
                .par_iter()
                .zip(chunk_lines.par_iter())
                .flat_map(|((start, end), chunk_line)| {
                    process_lines(
                        &config.pattern,
                        &mmap[*start..*end],
                        config.invert,
                        config.highlight,
                    )
                    .into_iter()
                    .map(move |(idx, s)| (idx + chunk_line, s.into_owned()))
                    .collect::<Vec<_>>()
                })
                .collect();

            return Ok(FileResult::Match(file_path, matched));
        }
    }
}

pub fn print_each_result(
    out: &mut BufWriter<StdoutLock>,
    config: &Arc<Config>,
    name: &str,
    v: (usize, &String),
) -> io::Result<()> {
    if config.line_number {
        writeln!(out, "{} - line: {}, {}", name.green(), v.0, v.1)?;
    } else {
        writeln!(out, "{}: {}", name.green(), v.1)?;
    }
    Ok(())
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
