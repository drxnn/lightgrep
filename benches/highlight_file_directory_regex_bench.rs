use criterion::{Criterion, criterion_group, criterion_main};
use lightgrep::{Config, Pattern, run};
use regex::Regex;

fn make_config(file_path: &str) -> Config {
    let regex = Regex::new(r"(said|told|asked)").unwrap();
    Config {
        file_path: file_path.to_string(),
        pattern: Pattern::Regex(regex),
        ignore_case: false,
        invert: false,
        count: true,
        line_number: false,
        recursive: true,
        file_extension: None,
        highlight: true,
    }
}

fn bench_highslight_multiple_literal(c: &mut Criterion) {
    let config = make_config("benches/test_data/large_recursive_data_test");
    c.bench_function("recursive regex match", |b| {
        b.iter(|| run(config.clone()).unwrap())
    });
}

criterion_group!(benches, bench_highslight_multiple_literal);
criterion_main!(benches);
