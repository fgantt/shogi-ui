//! Performance benchmarks for zugzwang detection
//!
//! Measures the overhead of zugzwang detection with move generation.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternConfig, EndgamePatternEvaluator};
use shogi_engine::types::{CapturedPieces, PieceType, Player};

fn benchmark_zugzwang_detection_overhead(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("zugzwang_detection_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_zugzwang(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_zugzwang_with_drops(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::empty();
    let mut captured_pieces = CapturedPieces::new();

    // Add captured pieces to enable drops
    captured_pieces.add_piece(PieceType::Pawn, Player::Black);
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Bishop, Player::Black);

    c.bench_function("zugzwang_detection_with_drops", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_zugzwang(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_zugzwang_drop_consideration_disabled(c: &mut Criterion) {
    let mut config = EndgamePatternConfig::default();
    config.enable_zugzwang_drop_consideration = false;
    let mut evaluator = EndgamePatternEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("zugzwang_detection_no_drop_consideration", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_zugzwang(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_count_safe_moves(c: &mut Criterion) {
    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("count_safe_moves", |b| {
        b.iter(|| {
            black_box(evaluator.count_safe_moves(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

criterion_group!(
    benches,
    benchmark_zugzwang_detection_overhead,
    benchmark_zugzwang_with_drops,
    benchmark_zugzwang_drop_consideration_disabled,
    benchmark_count_safe_moves
);
criterion_main!(benches);
