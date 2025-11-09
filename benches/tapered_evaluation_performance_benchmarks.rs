//! Performance benchmarks for tapered evaluation system
//!
//! This benchmark suite measures the performance of:
//! - TaperedScore creation and operations
//! - Game phase calculation
//! - Score interpolation
//! - Cache performance
//! - Overall evaluation overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::evaluation::tapered_eval::TaperedEvaluation;
use shogi_engine::types::*;

/// Benchmark TaperedScore creation
fn benchmark_tapered_score_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tapered_score_creation");

    group.bench_function("new_equal", |b| {
        b.iter(|| {
            black_box(TaperedScore::new(100));
        });
    });

    group.bench_function("new_tapered", |b| {
        b.iter(|| {
            black_box(TaperedScore::new_tapered(100, 200));
        });
    });

    group.finish();
}

/// Benchmark TaperedScore arithmetic operations
fn benchmark_tapered_score_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tapered_score_operations");

    let score1 = TaperedScore::new_tapered(100, 200);
    let score2 = TaperedScore::new_tapered(50, 75);

    group.bench_function("add", |b| {
        b.iter(|| {
            black_box(score1 + score2);
        });
    });

    group.bench_function("sub", |b| {
        b.iter(|| {
            black_box(score1 - score2);
        });
    });

    group.bench_function("neg", |b| {
        b.iter(|| {
            black_box(-score1);
        });
    });

    group.bench_function("mul", |b| {
        b.iter(|| {
            black_box(score1 * 0.5);
        });
    });

    group.finish();
}

/// Benchmark score interpolation
fn benchmark_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpolation");

    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("opening_phase", |b| {
        b.iter(|| {
            black_box(score.interpolate(GAME_PHASE_MAX));
        });
    });

    group.bench_function("middlegame_phase", |b| {
        b.iter(|| {
            black_box(score.interpolate(GAME_PHASE_MAX / 2));
        });
    });

    group.bench_function("endgame_phase", |b| {
        b.iter(|| {
            black_box(score.interpolate(0));
        });
    });

    // Benchmark interpolation at various phases
    for phase in [0, 64, 128, 192, 256].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(phase), phase, |b, &phase| {
            b.iter(|| {
                black_box(score.interpolate(phase));
            });
        });
    }

    group.finish();
}

/// Benchmark game phase calculation
fn benchmark_game_phase_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_phase_calculation");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("no_cache", |b| {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: false,
            ..Default::default()
        });
        b.iter(|| {
            black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
        });
    });

    group.bench_function("with_cache_cold", |b| {
        b.iter(|| {
            let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
                cache_game_phase: true,
                ..Default::default()
            });
            black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
        });
    });

    group.bench_function("with_cache_hot", |b| {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        // Warm up cache
        evaluator.calculate_game_phase(&board, &captured_pieces);

        b.iter(|| {
            black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark cache performance
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("cache_hit_rate_100", |b| {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        // Warm up cache
        evaluator.calculate_game_phase(&board, &captured_pieces);

        b.iter(|| {
            // All calls should hit cache
            for _ in 0..100 {
                black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
            }
        });
    });

    group.finish();
}

/// Benchmark TaperedEvaluation creation
fn benchmark_tapered_evaluation_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tapered_evaluation_creation");

    group.bench_function("new_default", |b| {
        b.iter(|| {
            black_box(TaperedEvaluation::new());
        });
    });

    group.bench_function("with_config", |b| {
        let config = TaperedEvaluationConfig::performance_optimized();
        b.iter(|| {
            black_box(TaperedEvaluation::with_config(config.clone()));
        });
    });

    group.finish();
}

/// Benchmark complete evaluation workflow
fn benchmark_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_workflow");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("phase_calculation_and_interpolation", |b| {
        let mut evaluator = TaperedEvaluation::new();
        let score = TaperedScore::new_tapered(100, 200);

        b.iter(|| {
            let phase = evaluator.calculate_game_phase(&board, &captured_pieces);
            black_box(evaluator.interpolate(score, phase));
        });
    });

    group.bench_function("multiple_score_accumulation", |b| {
        let mut evaluator = TaperedEvaluation::new();

        b.iter(|| {
            let phase = evaluator.calculate_game_phase(&board, &captured_pieces);

            let mut total = TaperedScore::default();
            for _ in 0..10 {
                total += TaperedScore::new_tapered(10, 20);
            }

            black_box(evaluator.interpolate(total, phase));
        });
    });

    group.finish();
}

/// Benchmark statistics tracking overhead
fn benchmark_statistics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_overhead");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("with_stats_tracking", |b| {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            enable_performance_monitoring: true,
            ..Default::default()
        });

        b.iter(|| {
            black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
        });
    });

    group.bench_function("without_stats_tracking", |b| {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            enable_performance_monitoring: false,
            ..Default::default()
        });

        b.iter(|| {
            black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark different configurations
fn benchmark_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let configs = vec![
        ("default", TaperedEvaluationConfig::default()),
        (
            "performance_optimized",
            TaperedEvaluationConfig::performance_optimized(),
        ),
        (
            "memory_optimized",
            TaperedEvaluationConfig::memory_optimized(),
        ),
        ("disabled", TaperedEvaluationConfig::disabled()),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &config, |b, config| {
            let mut evaluator = TaperedEvaluation::with_config(config.clone());
            b.iter(|| {
                black_box(evaluator.calculate_game_phase(&board, &captured_pieces));
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("create_many_scores", |b| {
        b.iter(|| {
            let scores: Vec<TaperedScore> = (0..1000)
                .map(|i| TaperedScore::new_tapered(i as i32, (i * 2) as i32))
                .collect();
            black_box(scores);
        });
    });

    group.bench_function("accumulate_scores", |b| {
        let scores: Vec<TaperedScore> = (0..1000)
            .map(|i| TaperedScore::new_tapered(i as i32, (i * 2) as i32))
            .collect();

        b.iter(|| {
            let mut total = TaperedScore::default();
            for score in &scores {
                total += *score;
            }
            black_box(total);
        });
    });

    group.finish();
}

/// Benchmark smooth interpolation
fn benchmark_smooth_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("smooth_interpolation");

    let evaluator = TaperedEvaluation::new();
    let score = TaperedScore::new_tapered(100, 200);

    group.bench_function("interpolate_all_phases", |b| {
        b.iter(|| {
            for phase in 0..=GAME_PHASE_MAX {
                black_box(evaluator.interpolate(score, phase));
            }
        });
    });

    group.bench_function("check_smoothness", |b| {
        b.iter(|| {
            let mut prev = evaluator.interpolate(score, 0);
            let mut max_diff = 0;

            for phase in 1..=GAME_PHASE_MAX {
                let curr = evaluator.interpolate(score, phase);
                let diff = (curr - prev).abs();
                max_diff = max_diff.max(diff);
                prev = curr;
            }

            black_box(max_diff);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tapered_score_creation,
    benchmark_tapered_score_operations,
    benchmark_interpolation,
    benchmark_game_phase_calculation,
    benchmark_cache_performance,
    benchmark_tapered_evaluation_creation,
    benchmark_complete_workflow,
    benchmark_statistics_overhead,
    benchmark_configurations,
    benchmark_memory_patterns,
    benchmark_smooth_interpolation,
);

criterion_main!(benches);
