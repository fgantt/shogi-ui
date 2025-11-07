//! Validation and Benchmarking Suite for Tapered Evaluation
//!
//! This benchmark suite validates and measures:
//! - Tapered vs traditional evaluation performance
//! - Evaluation accuracy
//! - Search performance impact
//! - Cache effectiveness
//! - Memory usage

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use shogi_engine::evaluation::PositionEvaluator;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::*;

/// Benchmark tapered vs traditional evaluation
fn benchmark_tapered_vs_traditional(c: &mut Criterion) {
    let mut group = c.benchmark_group("tapered_vs_traditional");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("integrated_evaluator", |b| {
        let mut evaluator = IntegratedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("position_evaluator_integrated", |b| {
        let evaluator = PositionEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("position_evaluator_legacy", |b| {
        let mut evaluator = PositionEvaluator::new();
        evaluator.disable_integrated_evaluator();
        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark cache effectiveness
fn benchmark_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("first_evaluation", |b| {
        b.iter_batched(
            || IntegratedEvaluator::new(),
            |mut evaluator| {
                black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("cached_evaluation", |b| {
        let mut evaluator = IntegratedEvaluator::new();
        // Warm up cache
        evaluator.evaluate(&board, Player::Black, &captured_pieces);

        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark search performance with tapered evaluation
fn benchmark_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance");
    group.sample_size(10); // Fewer samples for slow operations

    let mut board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("search_depth_3", |b| {
        let mut search_engine = SearchEngine::new(None, 64);
        b.iter(|| {
            black_box(search_engine.search_iterative(
                &mut board.clone(),
                &captured_pieces,
                Player::Black,
                3,
                1000,
            ));
        });
    });

    group.finish();
}

/// Benchmark memory usage
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("evaluator_creation", |b| {
        b.iter(|| {
            black_box(IntegratedEvaluator::new());
        });
    });

    group.bench_function("evaluator_with_stats", |b| {
        b.iter(|| {
            let mut evaluator = IntegratedEvaluator::new();
            evaluator.enable_statistics();
            black_box(evaluator);
        });
    });

    group.finish();
}

/// Benchmark evaluation under different game phases
fn benchmark_phase_specific_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase_specific");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Opening position
    group.bench_function("opening_position", |b| {
        let mut evaluator = IntegratedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark component combinations
fn benchmark_component_combinations(c: &mut Criterion) {
    use shogi_engine::evaluation::integration::{ComponentFlags, IntegratedEvaluationConfig};

    let mut group = c.benchmark_group("component_combinations");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("all_components", |b| {
        let mut config = IntegratedEvaluationConfig::default();
        config.components = ComponentFlags::all_enabled();
        let mut evaluator = IntegratedEvaluator::with_config(config);

        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("minimal_components", |b| {
        let mut config = IntegratedEvaluationConfig::default();
        config.components = ComponentFlags::minimal();
        let mut evaluator = IntegratedEvaluator::with_config(config);

        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark baseline comparison
fn benchmark_baseline_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("baseline_comparison");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("integrated_with_cache", |b| {
        let mut evaluator = IntegratedEvaluator::new();
        evaluator.evaluate(&board, Player::Black, &captured_pieces); // Warm cache

        b.iter(|| {
            black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("integrated_no_cache", |b| {
        b.iter_batched(
            || IntegratedEvaluator::new(),
            |mut evaluator| {
                black_box(evaluator.evaluate(&board, Player::Black, &captured_pieces));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tapered_vs_traditional,
    benchmark_cache_effectiveness,
    benchmark_search_performance,
    benchmark_memory_usage,
    benchmark_phase_specific_evaluation,
    benchmark_component_combinations,
    benchmark_baseline_comparison,
);

criterion_main!(benches);
