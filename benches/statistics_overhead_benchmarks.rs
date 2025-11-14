//! Performance benchmarks for statistics tracking overhead
//!
//! Measures the performance impact of statistics tracking in endgame pattern evaluation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::EndgamePatternEvaluator;
use shogi_engine::types::{CapturedPieces, PieceType, Player};

fn benchmark_statistics_overhead(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("statistics_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_statistics_reset(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Generate some statistics first
    for _ in 0..100 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    c.bench_function("statistics_reset", |b| {
        b.iter(|| {
            black_box(evaluator.reset_stats());
        });
    });
}

fn benchmark_statistics_summary(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Rook, Player::Black);

    // Generate some statistics
    for _ in 0..100 {
        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
    }

    c.bench_function("statistics_summary", |b| {
        b.iter(|| {
            black_box(evaluator.stats_summary());
        });
    });
}

fn benchmark_statistics_aggregation(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("statistics_aggregation", |b| {
        b.iter(|| {
            for _ in 0..10 {
                black_box(evaluator.evaluate_endgame(
                    black_box(&board),
                    black_box(Player::Black),
                    black_box(&captured_pieces),
                ));
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_statistics_overhead,
    benchmark_statistics_reset,
    benchmark_statistics_summary,
    benchmark_statistics_aggregation
);
criterion_main!(benches);
