//! Performance benchmarks for position-specific evaluation features
//!
//! This benchmark suite measures the performance of:
//! - King safety evaluation by phase
//! - Pawn structure evaluation by phase
//! - Piece mobility evaluation by phase
//! - Center control evaluation by phase
//! - Development evaluation by phase

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::types::*;
use shogi_engine::evaluation::position_features::PositionFeatureEvaluator;
use shogi_engine::bitboards::BitboardBoard;

/// Benchmark king safety evaluation
fn benchmark_king_safety(c: &mut Criterion) {
    let mut group = c.benchmark_group("king_safety");
    
    let board = BitboardBoard::new();
    
    group.bench_function("evaluate_king_safety", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_king_safety(&board, Player::Black));
        });
    });
    
    group.bench_function("evaluate_both_kings", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_king_safety(&board, Player::Black);
            let white = evaluator.evaluate_king_safety(&board, Player::White);
            black_box((black, white));
        });
    });
    
    group.finish();
}

/// Benchmark pawn structure evaluation
fn benchmark_pawn_structure(c: &mut Criterion) {
    let mut group = c.benchmark_group("pawn_structure");
    
    let board = BitboardBoard::new();
    
    group.bench_function("evaluate_pawn_structure", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_pawn_structure(&board, Player::Black));
        });
    });
    
    group.bench_function("evaluate_both_players", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_pawn_structure(&board, Player::Black);
            let white = evaluator.evaluate_pawn_structure(&board, Player::White);
            black_box((black, white));
        });
    });
    
    group.finish();
}

/// Benchmark mobility evaluation
fn benchmark_mobility(c: &mut Criterion) {
    let mut group = c.benchmark_group("mobility");
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    group.bench_function("evaluate_mobility", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces));
        });
    });
    
    group.finish();
}

/// Benchmark center control evaluation
fn benchmark_center_control(c: &mut Criterion) {
    let mut group = c.benchmark_group("center_control");
    
    let board = BitboardBoard::new();
    
    group.bench_function("evaluate_center_control", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_center_control(&board, Player::Black));
        });
    });
    
    group.bench_function("evaluate_both_players", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            let black = evaluator.evaluate_center_control(&board, Player::Black);
            let white = evaluator.evaluate_center_control(&board, Player::White);
            black_box((black, white));
        });
    });
    
    group.finish();
}

/// Benchmark development evaluation
fn benchmark_development(c: &mut Criterion) {
    let mut group = c.benchmark_group("development");
    
    let board = BitboardBoard::new();
    
    group.bench_function("evaluate_development", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            black_box(evaluator.evaluate_development(&board, Player::Black));
        });
    });
    
    group.finish();
}

/// Benchmark complete position evaluation
fn benchmark_complete_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_evaluation");
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    group.bench_function("all_features", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            let mut total = TaperedScore::default();
            total += evaluator.evaluate_king_safety(&board, Player::Black);
            total += evaluator.evaluate_pawn_structure(&board, Player::Black);
            total += evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
            total += evaluator.evaluate_center_control(&board, Player::Black);
            total += evaluator.evaluate_development(&board, Player::Black);
            black_box(total);
        });
    });
    
    group.bench_function("repeated_evaluations_100x", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            for _ in 0..100 {
                let mut total = TaperedScore::default();
                total += evaluator.evaluate_king_safety(&board, Player::Black);
                total += evaluator.evaluate_pawn_structure(&board, Player::Black);
                black_box(total);
            }
        });
    });
    
    group.finish();
}

/// Benchmark statistics tracking
fn benchmark_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");
    
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    
    group.bench_function("with_stats_tracking", |b| {
        let mut evaluator = PositionFeatureEvaluator::new();
        b.iter(|| {
            evaluator.evaluate_king_safety(&board, Player::Black);
            evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
            black_box(evaluator.stats());
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
        let mut evaluator = PositionFeatureEvaluator::with_config(
            PositionFeatureConfig::default()
        );
        b.iter(|| {
            let mut total = TaperedScore::default();
            total += evaluator.evaluate_king_safety(&board, Player::Black);
            total += evaluator.evaluate_pawn_structure(&board, Player::Black);
            total += evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
            black_box(total);
        });
    });
    
    group.finish();
}

/// Benchmark helper functions
fn benchmark_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("helpers");
    
    let evaluator = PositionFeatureEvaluator::new();
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
    
    group.bench_function("is_pawn_isolated", |b| {
        let pawn_pos = Position::new(4, 4);
        b.iter(|| {
            black_box(evaluator.is_pawn_isolated(&board, pawn_pos, Player::Black));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_king_safety,
    benchmark_pawn_structure,
    benchmark_mobility,
    benchmark_center_control,
    benchmark_development,
    benchmark_complete_evaluation,
    benchmark_statistics,
    benchmark_configurations,
    benchmark_helpers,
);

criterion_main!(benches);

