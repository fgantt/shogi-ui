//! Performance benchmarks for optimizations
//!
//! Measures the performance impact of optimizations:
//! - Caching effectiveness
//! - King-square tables vs Manhattan distance
//! - Bitboard operations vs O(81) scans

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternConfig, EndgamePatternEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

fn benchmark_caching_effectiveness(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Warm up cache
    evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    c.bench_function("caching_effectiveness", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_king_square_tables_vs_manhattan(c: &mut Criterion) {
    let mut config_manhattan = EndgamePatternConfig::default();
    config_manhattan.use_king_square_tables = false;
    let mut evaluator_manhattan = EndgamePatternEvaluator::with_config(config_manhattan);

    let mut config_tables = EndgamePatternConfig::default();
    config_tables.use_king_square_tables = true;
    let mut evaluator_tables = EndgamePatternEvaluator::with_config(config_tables);

    let board = BitboardBoard::new();

    c.bench_function("king_activity_manhattan", |b| {
        b.iter(|| {
            black_box(
                evaluator_manhattan
                    .evaluate_king_activity(black_box(&board), black_box(Player::Black)),
            );
        });
    });

    c.bench_function("king_activity_tables", |b| {
        b.iter(|| {
            black_box(
                evaluator_tables
                    .evaluate_king_activity(black_box(&board), black_box(Player::Black)),
            );
        });
    });
}

fn benchmark_bitboard_optimizations(c: &mut Criterion) {
    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    c.bench_function("find_pieces_bitboard", |b| {
        b.iter(|| {
            black_box(evaluator.find_pieces(
                black_box(&board),
                black_box(Player::Black),
                black_box(shogi_engine::types::PieceType::Rook),
            ));
        });
    });

    c.bench_function("collect_pawns_bitboard", |b| {
        b.iter(|| {
            black_box(evaluator.collect_pawns(black_box(&board), black_box(Player::Black)));
        });
    });

    c.bench_function("count_total_pieces_bitboard", |b| {
        b.iter(|| {
            black_box(evaluator.count_total_pieces(black_box(&board)));
        });
    });
}

fn benchmark_evaluation_with_optimizations(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("evaluation_with_optimizations", |b| {
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
    benchmark_caching_effectiveness,
    benchmark_king_square_tables_vs_manhattan,
    benchmark_bitboard_optimizations,
    benchmark_evaluation_with_optimizations
);
criterion_main!(benches);
