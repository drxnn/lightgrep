use criterion::{Criterion, criterion_group, criterion_main};
use lightgrep::{Config, Pattern, run};
use regex::bytes::Regex;

fn make_config(file_path: &str) -> Config {
    let regex = Regex::new(r"\b[a-zA-Z]{3}\b|\b[a-zA-Z]{5}\b").unwrap();
    Config {
        file_path: file_path.to_string(),
        pattern: Pattern::Regex(regex),
        ignore_case: false,
        invert: false,
        count: true,
        line_number: false,
        recursive: false,
        file_extension: None,
        highlight: false,
    }
}

fn bench_large_file_literal(c: &mut Criterion) {
    let config = make_config("benches/test_data/large_test_file.txt");
    c.bench_function("recursive literal match", |b| {
        b.iter(|| run(config.clone()).unwrap())
    });
}

criterion_group!(benches, bench_large_file_literal);
criterion_main!(benches);
