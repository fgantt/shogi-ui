//! Performance benchmarks for move ordering system
//! 
//! This module provides comprehensive benchmarks to measure the performance
//! of the move ordering system and validate that it meets the target
//! performance requirements.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use shogi_engine::types::*;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::MoveGenerator;
use std::time::Duration;

/// Generate test moves for benchmarking
fn generate_test_moves() -> Vec<Move> {
    let mut moves = Vec::new();
    
    // Generate a variety of move types for comprehensive testing
    for row in 0..9 {
        for col in 0..9 {
            let from = Position::new(row, col);
            
            // Add different types of moves
            for target_row in 0..9 {
                for target_col in 0..9 {
                    let to = Position::new(target_row, target_col);
                    
                    if from != to {
                        // Regular move
                        moves.push(Move {
                            from: Some(from),
                            to,
                            piece_type: PieceType::Pawn,
                            player: Player::Black,
                            is_capture: false,
                            is_promotion: false,
                            gives_check: false,
                            is_recapture: false,
                            captured_piece: None,
                        });
                        
                        // Capture move
                        moves.push(Move {
                            from: Some(from),
                            to,
                            piece_type: PieceType::Silver,
                            player: Player::Black,
                            is_capture: true,
                            is_promotion: false,
                            gives_check: false,
                            is_recapture: false,
                            captured_piece: Some(Piece {
                                piece_type: PieceType::Gold,
                                player: Player::White,
                            }),
                        });
                        
                        // Promotion move
                        if to.row == 0 || to.row == 8 {
                            moves.push(Move {
                                from: Some(from),
                                to,
                                piece_type: PieceType::Pawn,
                                player: Player::Black,
                                is_capture: false,
                                is_promotion: true,
                                gives_check: false,
                                is_recapture: false,
                                captured_piece: None,
                            });
                        }
                    }
                }
            }
        }
    }
    
    moves
}

/// Benchmark move ordering performance with different move counts
fn benchmark_move_ordering_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_performance");
    group.measurement_time(Duration::from_secs(10));
    
    let test_moves = generate_test_moves();
    
    // Test with different move counts
    let move_counts = vec![10, 50, 100, 200, 500];
    
    for count in move_counts {
        let moves_subset: Vec<Move> = test_moves.iter().take(count).cloned().collect();
        
        group.bench_with_input(
            BenchmarkId::new("order_moves", count),
            &moves_subset,
            |b, moves| {
                let mut orderer = MoveOrdering::new();
                b.iter(|| {
                    criterion::black_box(orderer.order_moves(criterion::black_box(moves)))
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark move scoring performance
fn benchmark_move_scoring_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_performance");
    group.measurement_time(Duration::from_secs(10));
    
    let test_moves = generate_test_moves();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();
    
    group.bench_function("score_move", |b| {
        let mut orderer = MoveOrdering::new();
        b.iter(|| {
            for move_ in &moves_subset {
                criterion::black_box(orderer.score_move(criterion::black_box(move_)));
            }
        })
    });
    
    group.finish();
}

/// Benchmark cache performance
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_cache_performance");
    group.measurement_time(Duration::from_secs(10));
    
    let test_moves = generate_test_moves();
    let moves_subset: Vec<Move> = test_moves.iter().take(50).cloned().collect();
    
    // Benchmark cache hits (second scoring of same moves)
    group.bench_function("cache_hits", |b| {
        let mut orderer = MoveOrdering::new();
        
        // Pre-populate cache
        for move_ in &moves_subset {
            let _ = orderer.score_move(move_);
        }
        
        b.iter(|| {
            for move_ in &moves_subset {
                criterion::black_box(orderer.score_move(criterion::black_box(move_)));
            }
        })
    });
    
    // Benchmark cache misses (first scoring)
    group.bench_function("cache_misses", |b| {
        b.iter(|| {
            let mut orderer = MoveOrdering::new();
            for move_ in &moves_subset {
                criterion::black_box(orderer.score_move(criterion::black_box(move_)));
            }
        })
    });
    
    group.finish();
}

/// Benchmark memory usage efficiency
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_memory_efficiency");
    group.measurement_time(Duration::from_secs(5));
    
    let test_moves = generate_test_moves();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();
    
    group.bench_function("memory_usage", |b| {
        b.iter(|| {
            let mut orderer = MoveOrdering::new();
            let _ = orderer.order_moves(&moves_subset);
            
            // Measure memory usage
            criterion::black_box(orderer.get_memory_usage().current_bytes)
        })
    });
    
    group.finish();
}

/// Benchmark different configuration weights
fn benchmark_configuration_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_configuration_performance");
    group.measurement_time(Duration::from_secs(10));
    
    let test_moves = generate_test_moves();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();
    
    // Default weights
    group.bench_function("default_weights", |b| {
        let mut orderer = MoveOrdering::new();
        b.iter(|| {
            criterion::black_box(orderer.order_moves(&moves_subset))
        })
    });
    
    // Custom weights
    group.bench_function("custom_weights", |b| {
        let custom_weights = OrderingWeights {
            capture_weight: 2000,
            promotion_weight: 1500,
            center_control_weight: 200,
            development_weight: 300,
            piece_value_weight: 100,
            position_value_weight: 150,
            tactical_weight: 500,
            quiet_weight: 50,
        };
        let mut orderer = MoveOrdering::with_config(custom_weights);
        b.iter(|| {
            criterion::black_box(orderer.order_moves(&moves_subset))
        })
    });
    
    group.finish();
}

/// Benchmark statistics tracking overhead
fn benchmark_statistics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_statistics_overhead");
    group.measurement_time(Duration::from_secs(10));
    
    let test_moves = generate_test_moves();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();
    
    group.bench_function("with_statistics", |b| {
        let mut orderer = MoveOrdering::new();
        b.iter(|| {
            let _ = orderer.order_moves(&moves_subset);
            criterion::black_box(orderer.get_stats());
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_move_ordering_performance,
    benchmark_move_scoring_performance,
    benchmark_cache_performance,
    benchmark_memory_efficiency,
    benchmark_configuration_performance,
    benchmark_statistics_overhead
);

criterion_main!(benches);

