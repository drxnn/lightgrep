# lightgrep

lightgrep is a fast, ergonomic grep-like CLI tool written in Rust. It searches files using regex or string literals, with optional highlighting, line numbers, and recursive searching.

---

## Features

- Search files for regular expressions

- Highlight matches

- Print line numbers, counts, file paths

- Invert matches

- Recursive directory search

- Multiple pattern support (String literals only)

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

Usage

```
lightgrep [OPTIONS] <PATTERN> -F [FILE...]
```

Basic examples:

```
# search for "TODO" in a specific file
lightgrep TODO -F src/test.txt

# search with a regex (escape shell metacharacters)
lightgrep '\bfixme\b' README.md



# Multiple String Literal Patterns found and highlighted

lightgrep -- --multiple red blue green  --highlight --recursive

```

## Options:

| Flag                          | Description                                    | Maps to               |
| ----------------------------- | ---------------------------------------------- | --------------------- |
| `-q, --query <PATTERN>`       | Query string to search for                     | `Args.query`          |
| `--multiple <PATTERN>...`     | Multiple literal string patterns to search for | `Args.multiple`       |
| `-i, --ignore-case`           | Case-insensitive search                        | `Args.ignore_case`    |
| `-F, --file-path <FILE_PATH>` | File or directory to search                    | `Args.file_path`      |
| `--invert`                    | Invert match — print non-matching lines        | `Args.invert`         |
| `-E, --regex`                 | Treat QUERY as a regex                         | `Args.regex`          |
| `-c, --count`                 | Only print count of matching lines             | `Args.count`          |
| `-l, --line-number`           | Show line numbers for matches                  | `Args.line_number`    |
| `-r, --recursive`             | Recurse into directories                       | `Args.recursive`      |
| `--ext <EXTENSION>`           | Filter files by extension (e.g., `.rs`)        | `Args.file_extension` |
| `    --highlight`             | Highlight matches                              | `Args.highlight`      |
| `--help`                      | Show help message                              | —                     |

License

MIT OR Apache-2.0
