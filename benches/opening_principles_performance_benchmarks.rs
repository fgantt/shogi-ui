//! Performance benchmarks for opening principles evaluation
//!
//! This benchmark suite measures the performance of:
//! - Development evaluation
//! - Center control in opening
//! - Castle formation evaluation
//! - Tempo evaluation
//! - Opening-specific penalties
//! - Piece coordination evaluation (Task 19.0 - Task 2.0)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::opening_principles::{
    OpeningPrincipleConfig, OpeningPrincipleEvaluator,
};
use shogi_engine::types::*;

/// Benchmark evaluator creation
fn benchmark_evaluator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(OpeningPrincipleEvaluator::new());
        });
    });

    group.finish();
}

/// Benchmark development evaluation
fn benchmark_development(c: &mut Criterion) {
    let mut group = c.benchmark_group("development");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_development", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_development(&board, Player::Black, 5));
        });
    });

    // Test at different move counts
    for move_count in [1, 5, 10, 15, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(move_count),
            &move_count,
            |b, &mc| {
                let mut evaluator = OpeningPrincipleEvaluator::new();
                b.iter(|| {
                    black_box(evaluator.evaluate_development(&board, Player::Black, mc));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark center control
fn benchmark_center_control(c: &mut Criterion) {
    let mut group = c.benchmark_group("center_control");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_center_control", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_center_control_opening(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark castle formation
fn benchmark_castle_formation(c: &mut Criterion) {
    let mut group = c.benchmark_group("castle_formation");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_castle", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_castle_formation(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark tempo evaluation
fn benchmark_tempo(c: &mut Criterion) {
    let mut group = c.benchmark_group("tempo");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_tempo", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_tempo(&board, Player::Black, 5));
        });
    });

    group.finish();
}

/// Benchmark opening penalties
fn benchmark_opening_penalties(c: &mut Criterion) {
    let mut group = c.benchmark_group("opening_penalties");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_penalties", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_opening_penalties(&board, Player::Black, 5));
        });
    });

    group.finish();
}

/// Benchmark complete opening evaluation
fn benchmark_complete_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_evaluation");

    let board = BitboardBoard::new();

    group.bench_function("all_principles", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
        });
    });

    group.bench_function("repeated_100x", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
            }
        });
    });

    group.finish();
}

/// Benchmark helper functions
fn benchmark_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("helpers");

    let evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    group.bench_function("find_king_position", |b| {
        b.iter(|| {
            black_box(evaluator.find_king_position(&board, Player::Black));
        });
    });

    group.bench_function("count_developed_pieces", |b| {
        b.iter(|| {
            black_box(evaluator.count_developed_pieces(&board, Player::Black));
        });
    });

    group.bench_function("count_active_pieces", |b| {
        b.iter(|| {
            black_box(evaluator.count_active_pieces(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark configuration variations
fn benchmark_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let board = BitboardBoard::new();

    group.bench_function("all_enabled", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
        });
    });

    group.bench_function("minimal", |b| {
        let config = OpeningPrincipleConfig {
            enable_development: true,
            enable_center_control: false,
            enable_castle_formation: false,
            enable_tempo: false,
            enable_opening_penalties: false,
            enable_piece_coordination: false,
        };
        let mut evaluator = OpeningPrincipleEvaluator::with_config(config);
        b.iter(|| {
            black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
        });
    });

    group.finish();
}

/// Benchmark piece coordination evaluation (Task 19.0 - Task 2.0)
fn benchmark_piece_coordination(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_coordination");

    let board = BitboardBoard::new();

    // Benchmark with coordination enabled
    group.bench_function("with_coordination", |b| {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
        });
    });

    // Benchmark with coordination disabled
    group.bench_function("without_coordination", |b| {
        let config = OpeningPrincipleConfig {
            enable_development: true,
            enable_center_control: true,
            enable_castle_formation: true,
            enable_tempo: true,
            enable_opening_penalties: true,
            enable_piece_coordination: false,
        };
        let mut evaluator = OpeningPrincipleEvaluator::with_config(config);
        b.iter(|| {
            black_box(evaluator.evaluate_opening(&board, Player::Black, 5));
        });
    });

    // Compare overhead
    group.bench_function("coordination_overhead", |b| {
        let board = BitboardBoard::new();

        // With coordination
        let mut evaluator_with = OpeningPrincipleEvaluator::new();
        let score_with = evaluator_with.evaluate_opening(&board, Player::Black, 5);

        // Without coordination
        let config = OpeningPrincipleConfig {
            enable_development: true,
            enable_center_control: true,
            enable_castle_formation: true,
            enable_tempo: true,
            enable_opening_penalties: true,
            enable_piece_coordination: false,
        };
        let mut evaluator_without = OpeningPrincipleEvaluator::with_config(config);
        let score_without = evaluator_without.evaluate_opening(&board, Player::Black, 5);

        b.iter(|| {
            black_box(score_with.interpolate(256));
            black_box(score_without.interpolate(256));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evaluator_creation,
    benchmark_development,
    benchmark_center_control,
    benchmark_castle_formation,
    benchmark_tempo,
    benchmark_opening_penalties,
    benchmark_complete_evaluation,
    benchmark_helpers,
    benchmark_configurations,
    benchmark_piece_coordination,
);

criterion_main!(benches);
