use criterion::{Criterion, criterion_group, criterion_main};
use lightgrep::{Config, Pattern, run};
use regex::bytes::Regex;

fn make_config(file_path: &str) -> Config {
    let regex = Regex::new(r"\b(?:\w+\s+){5}\w+\b").unwrap();
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
        pool_size: 11,
    }
}

fn bench_single_literal(c: &mut Criterion) {
    let config = make_config("benches/test_data/test_file.txt");
    c.bench_function("single literal match", |b| {
        b.iter(|| run(config.clone()).unwrap())
    });
}

criterion_group!(benches, bench_single_literal);
criterion_main!(benches);
