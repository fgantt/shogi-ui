//! Benchmarks for castle pattern integration in IntegratedEvaluator
//!
//! Measures the overhead of castle pattern evaluation and verifies
//! that integration doesn't significantly impact performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

fn benchmark_castle_pattern_evaluation(c: &mut Criterion) {
    let mut config = IntegratedEvaluationConfig::default();
    config.components.castle_patterns = true;

    let evaluator = IntegratedEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("castle_pattern_evaluation", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });
}

fn benchmark_castle_pattern_overhead(c: &mut Criterion) {
    let mut config_enabled = IntegratedEvaluationConfig::default();
    config_enabled.components.castle_patterns = true;
    let evaluator_enabled = IntegratedEvaluator::with_config(config_enabled);

    let mut config_disabled = IntegratedEvaluationConfig::default();
    config_disabled.components.castle_patterns = false;
    let evaluator_disabled = IntegratedEvaluator::with_config(config_disabled);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let mut group = c.benchmark_group("castle_pattern_overhead");

    group.bench_function("with_castle_patterns", |b| {
        b.iter(|| {
            black_box(evaluator_enabled.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("without_castle_patterns", |b| {
        b.iter(|| {
            black_box(evaluator_disabled.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_castle_pattern_evaluation,
    benchmark_castle_pattern_overhead
);
criterion_main!(benches);
