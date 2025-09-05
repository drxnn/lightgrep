# lightgrep

lightgrep is a fast, ergonomic grep-like CLI tool written in Rust. It searches files using regex or string literals, with optional highlighting, line numbers, and recursive searching. It is optimized for performance and can take advantage of multiple CPU cores when processing large files.

---

## Features

- Search files with regular expressions or string literals

- Highlight matches with ANSI colors

- Invert matches to show only non-matching lines

- Recursive directory search

- Multiple patterns (string literals only)

- Filter by file extension (e.g. .rs, .md)

- Parallel processing for faster search on large files

---

## Install

```bash
From crates.io (recommended)
cargo install lightgrep
```

Build from source

```bash
git clone git@github.com:drxnn/lightgrep.git
cd lightgrep
cargo build --release
# binary will be in target/release/lightgrep
```

## Usage

```
lightgrep [OPTIONS] <PATTERN> -F [FILE...]
```

Basic examples:

```
# search for "TODO" in a specific file
lightgrep TODO -F src/test.txt

# search with a regex (escape shell metacharacters)
lightgrep '\bfixme\b' README.md

# Search all files that have a rs extension for "unsafe"
cargo run --  -r --ext rs --query unsafe

# Recursively search and highlight 3 and 5 letter words
❯ cargo run --  -E --query  "\b[a-zA-Z]{3}\b|\b[a-zA-Z]{5}\b" -r   --highlight


# Multiple String Literal Patterns found and highlighted

lightgrep -- --multiple red blue green  --highlight --recursive

```

## Options:

| Flag                          | Description                                      | Maps to               |
| ----------------------------- | ------------------------------------------------ | --------------------- |
| `-q, --query <PATTERN>`       | Query string to search for                       | `Args.query`          |
| `--multiple <PATTERN>...`     | Multiple literal patterns (conflicts with -E)    | `Args.multiple`       |
| `-i, --ignore-case`           | Case-insensitive search                          | `Args.ignore_case`    |
| `-F, --file-path <FILE_PATH>` | File or directory to search                      | `Args.file_path`      |
| `--invert`                    | Invert match — print non-matching lines          | `Args.invert`         |
| `-E, --regex`                 | Treat QUERY as a regex (conflicts with multiple) | `Args.regex`          |
| `-c, --count`                 | Only print count of matching lines               | `Args.count`          |
| `-l, --line-number`           | Show line numbers for matches                    | `Args.line_number`    |
| `-r, --recursive`             | Recurse into directories                         | `Args.recursive`      |
| `--ext <EXTENSION>`           | Filter files by extension (e.g., `.rs`)          | `Args.file_extension` |
| `    --highlight`             | Highlight matches                                | `Args.highlight`      |
| `--help`                      | Show help message                                | —                     |

## Notes

--multiple and --regex cannot be used together.

If you use --multiple, you don’t need --query.

File extensions for --ext can be provided with or without the dot (e.g. rs or .rs).

When using regex queries (--regex), remember to escape shell metacharacters (e.g. '\bword\b').

Highlighting (--highlight) works with both string and regex searches.

### License

MIT OR Apache-2.0
