//! Performance optimization benchmarks for complete tapered evaluation
//!
//! This benchmark suite measures:
//! - Complete optimized evaluation
//! - Hot path performance
//! - Cache effectiveness
//! - Profiler overhead
//! - Optimization impact

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::performance::OptimizedEvaluator;
use shogi_engine::types::*;

/// Benchmark optimized evaluator creation
fn benchmark_evaluator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(OptimizedEvaluator::new());
        });
    });

    group.finish();
}

/// Benchmark optimized evaluation
fn benchmark_optimized_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimized_evaluation");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("evaluate_optimized", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("evaluate_both_players", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
            let white = evaluator.evaluate_optimized(&board, Player::White, &captured_pieces);
            black_box((black, white));
        });
    });

    group.finish();
}

/// Benchmark profiler overhead
fn benchmark_profiler_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("profiler_overhead");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("profiler_disabled", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("profiler_enabled", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        evaluator.profiler_mut().enable();
        b.iter(|| {
            black_box(evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark repeated evaluations (cache effectiveness)
fn benchmark_repeated_evaluations(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_evaluations");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("100x_same_position", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces));
            }
        });
    });

    group.bench_function("1000x_same_position", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces));
            }
        });
    });

    group.finish();
}

/// Benchmark hot path functions
fn benchmark_hot_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_paths");

    let board = BitboardBoard::new();

    group.bench_function("phase_calculation", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.calculate_phase_optimized(&board));
        });
    });

    group.bench_function("pst_evaluation", |b| {
        let evaluator = OptimizedEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_pst_optimized(&board, Player::Black));
        });
    });

    group.bench_function("interpolation", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        let score = TaperedScore::new_tapered(100, 200);
        b.iter(|| {
            black_box(evaluator.interpolate_optimized(score, 128));
        });
    });

    group.finish();
}

/// Benchmark performance profiler operations
fn benchmark_profiler_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("profiler_operations");

    group.bench_function("record_evaluation", |b| {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();
        b.iter(|| {
            profiler.record_evaluation(1000);
        });
    });

    group.bench_function("avg_evaluation_time", |b| {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();
        for i in 0..100 {
            profiler.record_evaluation(1000 + i);
        }
        b.iter(|| {
            black_box(profiler.avg_evaluation_time());
        });
    });

    group.bench_function("generate_report", |b| {
        let mut profiler = PerformanceProfiler::new();
        profiler.enable();
        for i in 0..100 {
            profiler.record_evaluation(1000 + i);
            profiler.record_phase_calculation(200 + i);
            profiler.record_pst_lookup(300 + i);
            profiler.record_interpolation(100 + i);
        }
        b.iter(|| {
            black_box(profiler.report());
        });
    });

    group.finish();
}

/// Benchmark complete workflow with profiling
fn benchmark_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("evaluate_and_profile", |b| {
        let mut evaluator = OptimizedEvaluator::new();
        evaluator.profiler_mut().enable();

        b.iter(|| {
            let score = evaluator.evaluate_optimized(&board, Player::Black, &captured_pieces);
            let report = evaluator.profiler().report();
            black_box((score, report));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evaluator_creation,
    benchmark_optimized_evaluation,
    benchmark_profiler_overhead,
    benchmark_repeated_evaluations,
    benchmark_hot_paths,
    benchmark_profiler_operations,
    benchmark_complete_workflow,
);

criterion_main!(benches);
