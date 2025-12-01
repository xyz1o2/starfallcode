use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

// 模拟的性能基准测试

fn benchmark_message_history(c: &mut Criterion) {
    c.bench_function("add_message_100", |b| {
        b.iter(|| {
            let mut history = Vec::new();
            for i in 0..100 {
                history.push(format!("Message {}", i));
            }
            black_box(history)
        })
    });
}

fn benchmark_context_building(c: &mut Criterion) {
    c.bench_function("build_context_1000_chars", |b| {
        b.iter(|| {
            let input = "x".repeat(1000);
            let context = format!("Context: {}", input);
            black_box(context)
        })
    });
}

fn benchmark_response_validation(c: &mut Criterion) {
    c.bench_function("validate_response_5000_chars", |b| {
        b.iter(|| {
            let response = "x".repeat(5000);
            let is_valid = !response.is_empty() && response.len() < 100000;
            black_box(is_valid)
        })
    });
}

fn benchmark_modification_detection(c: &mut Criterion) {
    c.bench_function("detect_modifications_10000_chars", |b| {
        b.iter(|| {
            let response = "```rust\nfn main() {}\n```".repeat(100);
            let has_code = response.contains("```");
            black_box(has_code)
        })
    });
}

criterion_group!(
    benches,
    benchmark_message_history,
    benchmark_context_building,
    benchmark_response_validation,
    benchmark_modification_detection
);

criterion_main!(benches);
