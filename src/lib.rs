mod types;
mod utils;

use std::{
    borrow::Cow,
    env,
    error::Error,
    sync::{Arc, atomic::AtomicUsize, mpsc},
};

use colored::Colorize;
pub use types::{Args, Config, FileResult, Pattern, ThreadPool};
pub use utils::{print_each_result, print_results, process_file};
use walkdir::{DirEntry, WalkDir};

pub fn count_matches(matches: &[(usize, String)]) -> usize {
    return matches.len();
}

pub trait Matcher {
    fn matches_query(&self, text: &[u8]) -> bool;
}

impl Matcher for Pattern {
    fn matches_query(&self, text: &[u8]) -> bool {
        match self {
            Pattern::Regex(re) => re.is_match(text),
            Pattern::Literal { pattern, .. } => pattern.is_match(text),

            Pattern::MultipleLiteral { pattern, .. } => pattern.is_match(text),
        }
    }
}

pub fn highlight_match(line: &[u8], pat: &Pattern) -> String {
    let mut highlighted_string = String::from("");
    let mut last = 0;

    match pat {
        Pattern::Literal { pattern, .. } | Pattern::MultipleLiteral { pattern, .. } => {
            let matches: Vec<(usize, usize)> = pattern
                .find_iter(line)
                .map(|m| (m.start(), m.end()))
                .collect();

            for (start, end) in matches {
                highlighted_string.push_str(&String::from_utf8_lossy(&line[last..start]));

                highlighted_string.push_str(
                    &String::from_utf8_lossy(&line[start..end])
                        .red()
                        .underline()
                        .bold()
                        .to_string(),
                );

                last = end;
            }

            highlighted_string
        }
        Pattern::Regex(re) => {
            let matches: Vec<(usize, usize)> =
                re.find_iter(line).map(|x| (x.start(), x.end())).collect();

            for (start, end) in matches {
                highlighted_string.push_str(&String::from_utf8_lossy(&line[last..start]));
                highlighted_string.push_str(
                    &String::from_utf8_lossy(&line[start..end])
                        .red()
                        .underline()
                        .bold()
                        .to_string(),
                );
                last = end;
            }

            highlighted_string
        }
    }
}

pub fn process_lines<'a>(
    query: &Pattern,
    contents: &'a [u8],
    invert: bool,
    highlight: bool,
) -> Vec<(usize, Cow<'a, str>)> {
    contents
        .split(|&b| b == b'\n')
        .enumerate()
        .filter_map(|(i, line)| {
            let matched = query.matches_query(line);
            if matched ^ invert {
                if highlight {
                    Some((i + 1, Cow::Owned(highlight_match(line, query))))
                } else {
                    Some((
                        i + 1,
                        Cow::Owned(String::from_utf8_lossy(line).into_owned()),
                    ))
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_counter = Arc::new(AtomicUsize::new(0));
    let current = env::current_dir()?;

    let num_of_cpus = num_cpus::get();
    let pool_size = if num_of_cpus > 1 { num_of_cpus - 1 } else { 1 };

    let (tx, rx) = mpsc::channel::<FileResult>();

    let config = Arc::new(config);
    let file_counter_clone = Arc::clone(&file_counter);
    let thread_pool = Arc::new(ThreadPool::new(pool_size, file_counter_clone));

    if config.recursive {
        let mut entry: DirEntry;

        for entry_walkdir in WalkDir::new(current) {
            entry = entry_walkdir?;

            if !entry.file_type().is_file() {
                continue;
            }

            let config = Arc::clone(&config);
            let tx = tx.clone();

            let thread_pool_c = Arc::clone(&thread_pool);

            let _ = process_file(entry, tx, config, thread_pool_c, pool_size);
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
            Some(Err(e)) => return Err(Box::new(e)),
            None => return Err("Entry was not found in current directory".into()),
        };

        {
            let tx = tx.clone();
            let config = Arc::clone(&config);

            let thread_pool_c = Arc::clone(&thread_pool);
            let _ = process_file(entry, tx, config, thread_pool_c, pool_size);
        } // dropping config
        drop(tx);
        drop(thread_pool);
        print_results(rx, config);
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use aho_corasick::AhoCorasick;

//     #[test]
//     fn literal_match() {
//         use crate::{Matcher, Pattern};
//         use aho_corasick::AhoCorasick;

//         let ac = AhoCorasick::new(&["foo"]).unwrap();
//         let pattern = Pattern::Literal {
//             pattern: ac,
//             case_insensitive: false,
//         };
//         assert!(pattern.matches_query("foo".as_bytes()));
//         assert!(!pattern.matches_query("Foo".as_bytes()));
//     }

//     #[test]
//     fn multiple_literal_match() {
//         use crate::{Matcher, Pattern};
//         let ac = AhoCorasick::new(&["foo", "bar"]).unwrap();
//         let pattern = Pattern::MultipleLiteral {
//             pattern: ac,
//             case_insensitive: false,
//         };
//         assert!(pattern.matches_query("foo".as_bytes()));
//         assert!(pattern.matches_query("bar".as_bytes()));
//         assert!(!pattern.matches_query("baz".as_bytes()));
//     }
//     #[test]
//     fn highlight_literal() {
//         use crate::{Pattern, highlight_match};
//         use aho_corasick::AhoCorasick;
//         use colored::Colorize;

//         let ac = AhoCorasick::new(&["foo"]).unwrap();
//         let pattern = Pattern::Literal {
//             pattern: ac,
//             case_insensitive: false,
//         };
//         let result = highlight_match("foo bar", &pattern);
//         let expected = "foo".red().underline().bold().to_string() + " bar";
//         assert_eq!(result, expected);
//     }

//     #[test]
//     fn process_lines_basic() {
//         use crate::{Pattern, process_lines};
//         use aho_corasick::AhoCorasick;
//         use std::borrow::Cow;

//         let ac = AhoCorasick::new(&["foo"]).unwrap();
//         let pattern = Pattern::Literal {
//             pattern: ac,
//             case_insensitive: false,
//         };
//         let text = "foo\nbar\nfoo bar";
//         let result: Vec<(usize, Cow<str>)> = process_lines(&pattern, text, false, false);

//         assert_eq!(result.len(), 2);
//         assert_eq!(result[0].0, 1);
//         assert_eq!(result[1].0, 3);
//     }

//     #[test]
//     fn invert_lines() {
//         use crate::{Pattern, process_lines};
//         use aho_corasick::AhoCorasick;

//         let ac = AhoCorasick::new(&["foo"]).unwrap();
//         let pattern = Pattern::Literal {
//             pattern: ac,
//             case_insensitive: false,
//         };
//         let text = "foo\nbar\nbaz";
//         let result = process_lines(&pattern, text, true, false);

//         assert_eq!(result.len(), 2);
//         assert_eq!(result[0].1, "bar");
//         assert_eq!(result[1].1, "baz");
//     }

//     #[test]
//     fn ignore_case_literal() {
//         use crate::{Matcher, Pattern};
//         use aho_corasick::AhoCorasickBuilder;

//         let ac = AhoCorasickBuilder::new()
//             .ascii_case_insensitive(true)
//             .build(&["foo"])
//             .unwrap();
//         let pattern = Pattern::Literal {
//             pattern: ac,
//             case_insensitive: true,
//         };
//         assert!(pattern.matches_query("FOO".as_bytes()));
//     }
// }
