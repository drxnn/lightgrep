mod types;
mod utils;

use std::borrow::Cow;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use colored::Colorize;
pub use types::{Args, Config, FileResult, Pattern, ThreadPool};
pub use utils::{print_each_result, print_results, process_batch};

pub fn count_matches(matches: &Vec<(usize, String)>) -> usize {
    // wrong for recursive, fix

    return matches.len();
}

pub trait Matcher {
    fn matches_query(&self, text: &str) -> bool;
}

impl Matcher for Pattern {
    fn matches_query(&self, text: &str) -> bool {
        match self {
            Pattern::Regex(re) => re.is_match(text),
            Pattern::Literal { pattern, .. } => pattern.is_match(text),

            Pattern::MultipleLiteral { pattern, .. } => pattern.is_match(text),
        }
    }
}

pub fn highlight_match(line: &str, pat: &Pattern) -> String {
    let mut highlighted_string = String::from("");

    match pat {
        Pattern::Literal { pattern, .. } | Pattern::MultipleLiteral { pattern, .. } => {
            let matches: Vec<(usize, usize)> = pattern
                .find_iter(line)
                .map(|m| (m.start(), m.end()))
                .collect();

            let mut last = 0;
            for (start, end) in matches {
                highlighted_string.push_str(&line[last..start]);

                highlighted_string.push_str(&line[start..end].red().underline().bold().to_string());

                last = end;
            }
            highlighted_string.push_str(&line[last..]);

            highlighted_string
        }
        Pattern::Regex(re) => {
            let matches: Vec<(usize, usize)> =
                re.find_iter(line).map(|x| (x.start(), x.end())).collect();

            for (index, char) in line.char_indices() {
                let inside_match = matches.iter().any(|(s, e)| index >= *s && index < *e);

                if inside_match {
                    highlighted_string
                        .push_str(&char.to_string().red().underline().bold().to_string());
                } else {
                    highlighted_string.push(char);
                }
            }

            highlighted_string
        }
    }
}

pub fn process_lines<'a>(
    query: &Pattern,
    contents: &'a str,
    invert: bool,
    highlight: bool,
) -> Vec<(usize, Cow<'a, str>)> {
    contents
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let matched = query.matches_query(line);
            if matched ^ invert {
                if highlight {
                    return Some((i + 1, Cow::Owned(highlight_match(line, query))));
                } else {
                    return Some((i + 1, Cow::Borrowed(line)));
                }
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use regex::Regex;

    #[test]
    fn literal_match() {
        use crate::{Matcher, Pattern};
        use aho_corasick::AhoCorasick;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        assert!(pattern.matches_query("foo"));
        assert!(!pattern.matches_query("Foo"));
    }

    #[test]
    fn multiple_literal_match() {
        use crate::{Matcher, Pattern};
        let ac = AhoCorasick::new(&["foo", "bar"]).unwrap();
        let pattern = Pattern::MultipleLiteral {
            pattern: ac,
            case_insensitive: false,
        };
        assert!(pattern.matches_query("foo"));
        assert!(pattern.matches_query("bar"));
        assert!(!pattern.matches_query("baz"));
    }
    #[test]
    fn highlight_literal() {
        use crate::{Pattern, highlight_match};
        use aho_corasick::AhoCorasick;
        use colored::Colorize;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let result = highlight_match("foo bar", &pattern);
        let expected = "foo".red().underline().bold().to_string() + " bar";
        assert_eq!(result, expected);
    }

    #[test]
    fn process_lines_basic() {
        use crate::{Pattern, process_lines};
        use aho_corasick::AhoCorasick;
        use std::borrow::Cow;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let text = "foo\nbar\nfoo bar";
        let result: Vec<(usize, Cow<str>)> = process_lines(&pattern, text, false, false);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].0, 3);
    }

    #[test]
    fn invert_lines() {
        use crate::{Pattern, process_lines};
        use aho_corasick::AhoCorasick;

        let ac = AhoCorasick::new(&["foo"]).unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        };
        let text = "foo\nbar\nbaz";
        let result = process_lines(&pattern, text, true, false);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "bar");
        assert_eq!(result[1].1, "baz");
    }

    #[test]
    fn ignore_case_literal() {
        use crate::{Matcher, Pattern};
        use aho_corasick::AhoCorasickBuilder;

        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&["foo"])
            .unwrap();
        let pattern = Pattern::Literal {
            pattern: ac,
            case_insensitive: true,
        };
        assert!(pattern.matches_query("FOO"));
    }
}
