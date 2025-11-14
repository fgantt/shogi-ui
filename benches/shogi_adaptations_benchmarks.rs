//! Performance benchmarks for shogi-specific adaptations
//!
//! Measures the overhead of shogi-specific features:
//! - Drop-based mate threat detection
//! - Opposition adjustment with pieces in hand
//! - Material calculation including pieces in hand

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::EndgamePatternEvaluator;
use shogi_engine::types::{CapturedPieces, PieceType, Player};

fn benchmark_drop_mate_threats(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Bishop, Player::Black);

    c.bench_function("drop_mate_threats", |b| {
        b.iter(|| {
            black_box(evaluator.check_drop_mate_threats(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_opposition_with_pieces_in_hand(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Gold, Player::White);
    captured_pieces.add_piece(PieceType::Silver, Player::White);

    c.bench_function("opposition_with_pieces_in_hand", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_opposition(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_material_calculation_with_hand(c: &mut Criterion) {
    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Bishop, Player::Black);

    c.bench_function("material_calculation_with_hand", |b| {
        b.iter(|| {
            black_box(evaluator.calculate_material(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_shogi_adaptations_overhead(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Gold, Player::White);

    c.bench_function("shogi_adaptations_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

criterion_group!(
    benches,
    benchmark_drop_mate_threats,
    benchmark_opposition_with_pieces_in_hand,
    benchmark_material_calculation_with_hand,
    benchmark_shogi_adaptations_overhead
);
criterion_main!(benches);
