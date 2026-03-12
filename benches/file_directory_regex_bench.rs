use aho_corasick::AhoCorasick;
use criterion::{Criterion, criterion_group, criterion_main};
use lightgrep::{Config, Pattern, run};

fn make_config(file_path: &str) -> Config {
    let ac = AhoCorasick::new(&["said"]).unwrap();
    Config {
        file_path: file_path.to_string(),
        pattern: Pattern::Literal {
            pattern: ac,
            case_insensitive: false,
        },
        ignore_case: false,
        invert: false,
        count: true,
        line_number: false,
        recursive: true,
        file_extension: None,
        highlight: false,
        pool_size: 11,
    }
}

fn bench_multiple_literal(c: &mut Criterion) {
    let config = make_config("benches/test_data/large_recursive_data_test");
    c.bench_function("recursive literal match", |b| {
        b.iter(|| run(config.clone()).unwrap())
    });
}

criterion_group!(benches, bench_multiple_literal);
criterion_main!(benches);
