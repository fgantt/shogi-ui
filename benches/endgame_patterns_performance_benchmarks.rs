//! Performance benchmarks for endgame pattern evaluation
//!
//! This benchmark suite measures the performance of:
//! - King activity evaluation
//! - Passed pawn evaluation in endgame
//! - Piece coordination detection
//! - Mating pattern recognition
//! - Major piece activity evaluation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::EndgamePatternEvaluator;
use shogi_engine::types::*;

/// Benchmark endgame evaluator creation
fn benchmark_evaluator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            black_box(EndgamePatternEvaluator::new());
        });
    });

    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(EndgamePatternEvaluator::default());
        });
    });

    group.finish();
}

/// Benchmark king activity evaluation
fn benchmark_king_activity(c: &mut Criterion) {
    let mut group = c.benchmark_group("king_activity");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_king_activity", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_king_activity(&board, Player::Black));
        });
    });

    group.bench_function("both_kings", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_king_activity(&board, Player::Black);
            let white = evaluator.evaluate_king_activity(&board, Player::White);
            black_box((black, white));
        });
    });

    group.finish();
}

/// Benchmark passed pawn evaluation
fn benchmark_passed_pawns(c: &mut Criterion) {
    let mut group = c.benchmark_group("passed_pawns");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_passed_pawns", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_passed_pawns_endgame(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark piece coordination
fn benchmark_piece_coordination(c: &mut Criterion) {
    let mut group = c.benchmark_group("piece_coordination");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_coordination", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_piece_coordination(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark mating pattern detection
fn benchmark_mating_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("mating_patterns");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("evaluate_mating_patterns", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_mating_patterns(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

/// Benchmark major piece activity
fn benchmark_major_piece_activity(c: &mut Criterion) {
    let mut group = c.benchmark_group("major_piece_activity");

    let board = BitboardBoard::new();

    group.bench_function("evaluate_activity", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_major_piece_activity(&board, Player::Black));
        });
    });

    group.finish();
}

/// Benchmark complete endgame evaluation
fn benchmark_complete_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_evaluation");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("all_patterns", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("repeated_100x", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            for _ in 0..100 {
                black_box(evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces));
            }
        });
    });

    group.finish();
}

/// Benchmark helper functions
fn benchmark_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("helpers");

    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    group.bench_function("find_king_position", |b| {
        b.iter(|| {
            black_box(evaluator.find_king_position(&board, Player::Black));
        });
    });

    group.bench_function("collect_pawns", |b| {
        b.iter(|| {
            black_box(evaluator.collect_pawns(&board, Player::Black));
        });
    });

    group.bench_function("find_pieces_rook", |b| {
        b.iter(|| {
            black_box(evaluator.find_pieces(&board, Player::Black, PieceType::Rook));
        });
    });

    group.finish();
}

/// Benchmark configuration variations
fn benchmark_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("all_enabled", |b| {
        let mut evaluator = EndgamePatternEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces));
        });
    });

    group.bench_function("minimal", |b| {
        let config = EndgamePatternConfig {
            enable_king_activity: true,
            enable_passed_pawns: false,
            enable_piece_coordination: false,
            enable_mating_patterns: false,
            enable_major_piece_activity: false,
        };
        let mut evaluator = EndgamePatternEvaluator::with_config(config);
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evaluator_creation,
    benchmark_king_activity,
    benchmark_passed_pawns,
    benchmark_piece_coordination,
    benchmark_mating_patterns,
    benchmark_major_piece_activity,
    benchmark_complete_evaluation,
    benchmark_helpers,
    benchmark_configurations,
);

criterion_main!(benches);
