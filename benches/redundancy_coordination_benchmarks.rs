//! Benchmarks for redundancy elimination and coordination in IntegratedEvaluator
//!
//! Measures the performance impact of coordination logic and verifies
//! that double-counting prevention doesn't significantly impact performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{IntegratedEvaluationConfig, IntegratedEvaluator};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

/// Create a position with a passed pawn for benchmarking
fn create_passed_pawn_position() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let captured_pieces = CapturedPieces::new();

    // Black king
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );

    // Black passed pawn (advanced, no enemy pawns blocking)
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4), // Very advanced for Black
    );

    // White king
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );

    (board, captured_pieces)
}

fn benchmark_passed_pawn_coordination(c: &mut Criterion) {
    let (board, captured_pieces) = create_passed_pawn_position();

    // Configuration with both modules enabled (coordination active)
    let mut config_coordinated = IntegratedEvaluationConfig::default();
    config_coordinated.components.position_features = true;
    config_coordinated.components.endgame_patterns = true;
    let evaluator_coordinated = IntegratedEvaluator::with_config(config_coordinated);

    // Configuration with only position_features (no coordination needed)
    let mut config_no_coordination = IntegratedEvaluationConfig::default();
    config_no_coordination.components.position_features = true;
    config_no_coordination.components.endgame_patterns = false;
    let evaluator_no_coordination = IntegratedEvaluator::with_config(config_no_coordination);

    let mut group = c.benchmark_group("passed_pawn_coordination");

    group.bench_function("with_coordination", |b| {
        b.iter(|| {
            black_box(evaluator_coordinated.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("without_coordination", |b| {
        b.iter(|| {
            black_box(evaluator_no_coordination.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

fn benchmark_center_control_overlap(c: &mut Criterion) {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Configuration with both position_features and positional_patterns
    // (center control overlap)
    let mut config_overlap = IntegratedEvaluationConfig::default();
    config_overlap.components.position_features = true;
    config_overlap.components.positional_patterns = true;
    let evaluator_overlap = IntegratedEvaluator::with_config(config_overlap);

    // Configuration with only position_features
    let mut config_single = IntegratedEvaluationConfig::default();
    config_single.components.position_features = true;
    config_single.components.positional_patterns = false;
    let evaluator_single = IntegratedEvaluator::with_config(config_single);

    let mut group = c.benchmark_group("center_control_overlap");

    group.bench_function("with_overlap", |b| {
        b.iter(|| {
            black_box(evaluator_overlap.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("without_overlap", |b| {
        b.iter(|| {
            black_box(evaluator_single.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

fn benchmark_coordination_overhead(c: &mut Criterion) {
    let (board, captured_pieces) = create_passed_pawn_position();

    // Test coordination logic overhead by comparing:
    // 1. Both modules enabled (coordination active)
    // 2. Only endgame_patterns enabled (no coordination needed)
    // 3. Only position_features enabled (no coordination needed)

    let mut config_both = IntegratedEvaluationConfig::default();
    config_both.components.position_features = true;
    config_both.components.endgame_patterns = true;
    let evaluator_both = IntegratedEvaluator::with_config(config_both);

    let mut config_endgame_only = IntegratedEvaluationConfig::default();
    config_endgame_only.components.position_features = false;
    config_endgame_only.components.endgame_patterns = true;
    let evaluator_endgame_only = IntegratedEvaluator::with_config(config_endgame_only);

    let mut config_position_only = IntegratedEvaluationConfig::default();
    config_position_only.components.position_features = true;
    config_position_only.components.endgame_patterns = false;
    let evaluator_position_only = IntegratedEvaluator::with_config(config_position_only);

    let mut group = c.benchmark_group("coordination_overhead");

    group.bench_function("both_modules", |b| {
        b.iter(|| {
            black_box(evaluator_both.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("endgame_only", |b| {
        b.iter(|| {
            black_box(evaluator_endgame_only.evaluate(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ))
        })
    });

    group.bench_function("position_only", |b| {
        b.iter(|| {
            black_box(evaluator_position_only.evaluate(
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
    benchmark_passed_pawn_coordination,
    benchmark_center_control_overlap,
    benchmark_coordination_overhead
);
criterion_main!(benches);
