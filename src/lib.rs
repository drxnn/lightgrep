mod types;
mod utils;

use rayon::prelude::*;

use std::{env, error::Error, sync::Arc};

pub use types::{Args, Config, FileResult, Pattern};
pub use utils::{print_each_result, print_results, print_single_result, process_file};
use walkdir::{DirEntry, WalkDir};

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let current = env::current_dir()?;

    let config = Arc::new(config);

    if config.recursive {
        let entries: Vec<DirEntry> = WalkDir::new(&current)
            .into_iter()
            .filter_map(|e: Result<DirEntry, walkdir::Error>| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        let results: Vec<FileResult> = entries
            .par_iter()
            .filter_map(|e| process_file(e.clone(), Arc::clone(&config)).ok())
            .collect();

        print_results(results, config)?;
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

        let config = Arc::clone(&config);

        let result = process_file(entry, Arc::clone(&config))?;

        print_single_result(result, Arc::clone(&config))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::utils::{highlight_match, process_lines};

    use aho_corasick::AhoCorasick;

    #[test]
    fn literal_match() {
        use crate::types::{Matcher, Pattern};
        use aho_corasick::AhoCorasick;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        assert!(pattern.matches_query("foo".as_bytes()));
        assert!(!pattern.matches_query("Foo".as_bytes()));
    }

    #[test]
    fn multiple_literal_match() {
        use crate::types::{Matcher, Pattern};
        let ac = AhoCorasick::new(&["foo", "bar"]).unwrap();
        let pattern = Pattern::MultipleLiteral {
            pattern: ac,
            case_insensitive: false,
        };
        assert!(pattern.matches_query("foo".as_bytes()));
        assert!(pattern.matches_query("bar".as_bytes()));
        assert!(!pattern.matches_query("baz".as_bytes()));
    }
    #[test]
    fn highlight_literal() {
        use crate::types::Pattern;
        use aho_corasick::AhoCorasick;
        use colored::Colorize;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let result = highlight_match("foo bar".as_bytes(), &pattern);
        let expected = "foo".red().underline().bold().to_string() + " bar";
        assert_eq!(result, expected);
    }

    #[test]
    fn process_lines_basic() {
        use crate::Pattern;
        use aho_corasick::AhoCorasick;
        use std::borrow::Cow;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let text = "foo\nbar\nfoo bar";
        let result: Vec<(usize, Cow<str>)> = process_lines(&pattern, text.as_bytes(), false, false);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].0, 3);
    }

    #[test]
    fn invert_lines() {
        use crate::Pattern;
        use aho_corasick::AhoCorasick;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let text = "foo\nbar\nbaz";
        let result = process_lines(&pattern, text.as_bytes(), true, false);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "bar");
        assert_eq!(result[1].1, "baz");
    }

    #[test]
    fn ignore_case_literal() {
        use crate::types::{Matcher, Pattern};
        use aho_corasick::AhoCorasickBuilder;

        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&["foo"])
            .unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: true,
        };
        assert!(pattern.matches_query("FOO".as_bytes()));
    }
}
