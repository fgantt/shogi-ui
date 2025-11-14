//! Performance benchmarks for pattern detection improvements
//!
//! Measures the overhead of additional pattern detection checks:
//! - Opposition pawn count filtering
//! - Triangulation opponent mobility checks
//! - King activity safety checks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternConfig, EndgamePatternEvaluator};
use shogi_engine::types::{CapturedPieces, Player};

fn benchmark_pattern_detection_overhead(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("pattern_detection_overhead", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_endgame(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_opposition_with_pawn_count(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    c.bench_function("opposition_with_pawn_count", |b| {
        b.iter(|| {
            black_box(evaluator.evaluate_opposition(black_box(&board), black_box(Player::Black)));
        });
    });
}

fn benchmark_triangulation_complete(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::empty();

    c.bench_function("triangulation_complete_logic", |b| {
        b.iter(|| {
            black_box(
                evaluator.evaluate_triangulation(black_box(&board), black_box(Player::Black)),
            );
        });
    });
}

fn benchmark_king_activity_safety(c: &mut Criterion) {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    c.bench_function("king_activity_with_safety", |b| {
        b.iter(|| {
            black_box(
                evaluator.evaluate_king_activity(black_box(&board), black_box(Player::Black)),
            );
        });
    });
}

criterion_group!(
    benches,
    benchmark_pattern_detection_overhead,
    benchmark_opposition_with_pawn_count,
    benchmark_triangulation_complete,
    benchmark_king_activity_safety
);
criterion_main!(benches);
