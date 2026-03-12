#![allow(dead_code)]
use clap::Parser;

use regex::bytes::Regex;
use std::collections::HashMap;
use std::env;
use std::error::Error;

use std::path::Path;
use std::process;

use aho_corasick::AhoCorasick;

use crate::utils::build_ac;
#[derive(Clone)] // for testing
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

#[derive(Clone)] // for test
pub struct Config {
    pub file_path: String,
    pub pattern: Pattern,
    pub ignore_case: bool,
    pub invert: bool,
    pub count: bool,
    pub line_number: bool,
    pub recursive: bool,
    pub pool_size: usize,

    pub file_extension: Option<String>,
    pub highlight: bool,
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
impl TryFrom<Args> for Config {
    type Error = Box<dyn Error>;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
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

        let pattern = if args.regex {
            let q = if let Some(qs) = args.query.clone() {
                qs
            } else {
                args.multiple.first().cloned().unwrap_or_else(|| {
                    eprintln!("--regex requires a query string (use --query or --multiple).");
                    std::process::exit(1);
                })
            };

            match regex::bytes::RegexBuilder::new(&q)
                .case_insensitive(ignore_case)
                .build()
            {
                Ok(re) => Pattern::Regex(re),
                Err(e) => {
                    eprintln!("Invalid regex `{}`: {}", q, e);
                    process::exit(1);
                }
            }
        } else if !args.multiple.is_empty() {
            let ac = build_ac(&args.multiple, ignore_case)?;
            Pattern::MultipleLiteral {
                pattern: ac,
                case_insensitive: ignore_case,
            }
        } else if let Some(q) = args.query {
            let ac = build_ac(&vec![q], ignore_case)?;
            Pattern::Literal {
                pattern: ac,
                case_insensitive: ignore_case,
            }
        } else {
            return Err("--regex requires a query string (use --query or --multiple)".into());
        };

        let num_of_cpus = num_cpus::get();
        let pool_size = if num_of_cpus > 1 { num_of_cpus - 1 } else { 1 };

        Ok(Config {
            pattern,
            file_path,
            ignore_case,
            invert: args.invert,
            count: args.count,
            line_number: args.line_number,
            recursive: args.recursive,
            file_extension,
            highlight: args.highlight,
            pool_size,
        })
    }
}

pub enum FileResult {
    Match(String, Vec<(usize, String)>),
    Skip,
    Error(String),
}
