use std::collections::HashMap;
use std::path::Path;

use clap::Parser;
use crossbeam::channel::{Receiver, Sender, unbounded};
use regex::{Regex, RegexBuilder};
use std::env;
use std::process;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use std::sync::{Arc, Mutex};
use std::thread;
pub enum Pattern {
    Literal {
        pattern: AhoCorasick,
        case_insensitive: bool,
    },
    Regex(Regex),
    MultipleLiteral {
        pattern: AhoCorasick,

        case_insensitive: bool,
    },
}

pub struct Config {
    pub file_path: String,
    pub pattern: Pattern,
    pub ignore_case: bool,
    pub invert: bool,
    pub count: bool,
    pub line_number: bool,
    pub recursive: bool,

    pub file_extension: Option<String>,
    pub highlight: bool,
}
pub struct Output {
    pub output_map: HashMap<String, Vec<(usize, String)>>,
}
#[derive(Parser)]
pub struct Args {
    #[arg(short = 'q', long)]
    pub query: Option<String>,
    #[arg(long, num_args = 1.., conflicts_with = "regex")]
    pub multiple: Vec<String>,

    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    #[arg(short = 'F', long, value_name = "FILE_PATH")]
    pub file_path: Option<String>,

    #[arg(long)]
    pub invert: bool,
    #[arg(short = 'E', long, conflicts_with = "multiple")]
    pub regex: bool,
    #[arg(short = 'c', long)]
    pub count: bool,
    #[arg(short = 'l', long)]
    pub line_number: bool,
    #[arg(short = 'r', long)]
    pub recursive: bool,

    #[arg(long = "ext", value_name = "EXTENSION")]
    pub file_extension: Option<String>,
    #[arg(long = "highlight")]
    pub highlight: bool,
}
impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let ignore_case = args.ignore_case || env::var("IGNORE_CASE").is_ok();

        let file_path = match args.file_path {
            Some(fp) => fp,
            _ => "".to_string(),
        };

        let file_extension = args.file_extension.or_else(|| {
            Path::new(&file_path)
                .extension()
                .map(|ext| ext.to_string_lossy().to_string())
        });

        // helper function put in utils later
        fn build_ac(patterns: &[String], ignore_case: bool) -> AhoCorasick {
            let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
            if ignore_case {
                AhoCorasickBuilder::new()
                    .ascii_case_insensitive(true)
                    .build(&pattern_refs)
                    .unwrap()
            } else {
                AhoCorasick::new(&pattern_refs).unwrap()
            }
        }

        let pattern = if args.regex {
            let q = if let Some(qs) = args.query.clone() {
                qs
            } else {
                args.multiple.first().cloned().unwrap_or_else(|| {
                    eprintln!("--regex requires a query string (use --query or --multiple).");
                    std::process::exit(1);
                })
            };

            match RegexBuilder::new(&q).case_insensitive(ignore_case).build() {
                Ok(re) => Pattern::Regex(re),
                Err(e) => {
                    eprintln!("Invalid regex `{}`: {}", q, e);
                    process::exit(1);
                }
            }
        } else if !args.multiple.is_empty() {
            let ac = build_ac(&args.multiple, ignore_case);
            Pattern::MultipleLiteral {
                pattern: ac,
                case_insensitive: ignore_case,
            }
        } else if let Some(q) = args.query {
            let ac = build_ac(&vec![q], ignore_case);
            Pattern::Literal {
                pattern: ac,
                case_insensitive: ignore_case,
            }
        } else {
            eprintln!(
                "Error: no query provided. Provide positional argument(1) for query <Q> or --multiple <Q>."
            );
            process::exit(1);
        };

        Config {
            pattern,
            file_path,
            ignore_case,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_extension,
            highlight: args.highlight,
        }
    }
}
pub struct ThreadPool {
    pub workers: Vec<Worker>,
    pub sender: Option<Sender<Job>>,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Receiver<Job>, counter: Arc<Mutex<usize>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(job) => {
                        job();
                        let mut count = counter.lock().unwrap();
                        *count += 1;
                    }
                    Err(_) => break,
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
impl ThreadPool {
    pub fn new(size: usize, counter: Arc<Mutex<usize>>) -> Self {
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = unbounded::<Job>();

        for id in 0..size {
            let counter_clone = Arc::clone(&counter);
            let rec_clone = receiver.clone();
            workers.push(Worker::new(id as usize, rec_clone, counter_clone));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take();
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = &self.sender {
            sender.send(job).expect("Worker thread has shut down");
        } else {
            panic!("ThreadPool has been shut down");
        }
    }
}

pub enum FileResult {
    Match(String, Vec<(usize, String)>),
    Skip,
    Error(String),
}
