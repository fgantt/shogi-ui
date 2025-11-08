#![cfg(feature = "legacy-tests")]
//! Principal Variation move ordering performance benchmarks
//!
//! Measures the effectiveness and overhead of various PV move ordering heuristics

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use shogi_engine::search::{ThreadSafeTranspositionTable, ThreadSafetyMode, TranspositionConfig};
use shogi_engine::types::*;
use std::time::Duration;

/// Generate test moves for PV move ordering benchmarks
fn generate_test_moves_for_pv() -> Vec<Move> {
    let mut moves = Vec::new();

    // Generate a variety of move types
    for row in 0..9 {
        for col in 0..9 {
            let from = Position::new(row, col);

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
                    }
                }
            }
        }
    }

    moves
}

/// Benchmark PV move scoring performance
fn benchmark_pv_move_scoring_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_scoring_performance");
    group.measurement_time(Duration::from_secs(10));

    let mut orderer = MoveOrdering::new();
    let test_moves = generate_test_moves_for_pv();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();

    group.bench_function("score_pv_move", |b| {
        b.iter(|| {
            for move_ in &moves_subset {
                criterion::black_box(orderer.score_pv_move(criterion::black_box(move_)));
            }
        })
    });

    group.finish();
}

/// Benchmark PV move retrieval from transposition table
fn benchmark_pv_move_retrieval_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_retrieval_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    // Create and store PV moves
    let pv_moves = vec![
        Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
        },
        Move {
            from: Some(Position::new(2, 2)),
            to: Position::new(3, 2),
            piece_type: PieceType::Silver,
            player,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
        },
    ];

    // Store PV moves
    for (i, pv_move) in pv_moves.iter().enumerate() {
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100 + i as i32,
        );
    }

    group.bench_function("get_pv_move_with_tt_hit", |b| {
        b.iter(|| {
            criterion::black_box(orderer.get_pv_move(&board, &captured_pieces, player, depth))
        })
    });

    group.finish();
}

/// Benchmark PV move storage performance
fn benchmark_pv_move_storage_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_storage_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    let test_moves = generate_test_moves_for_pv();
    let moves_subset: Vec<Move> = test_moves.iter().take(50).cloned().collect();

    group.bench_function("update_pv_move", |b| {
        b.iter(|| {
            for (i, move_) in moves_subset.iter().enumerate() {
                orderer.update_pv_move(
                    &board,
                    &captured_pieces,
                    player,
                    depth,
                    move_.clone(),
                    100 + i as i32,
                );
            }
        })
    });

    group.finish();
}

/// Benchmark PV move cache performance
fn benchmark_pv_move_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_cache_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    // Create and store PV move
    let pv_move = Move {
        from: Some(Position::new(1, 1)),
        to: Position::new(2, 1),
        piece_type: PieceType::Pawn,
        player,
        is_capture: false,
        is_promotion: false,
        gives_check: false,
        is_recapture: false,
        captured_piece: None,
    };

    orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move, 100);

    // Benchmark cache hits (subsequent lookups)
    group.bench_function("pv_move_cache_hits", |b| {
        b.iter(|| {
            for _ in 0..100 {
                criterion::black_box(orderer.get_pv_move(&board, &captured_pieces, player, depth));
            }
        })
    });

    // Benchmark cache misses (first lookup of new positions)
    group.bench_function("pv_move_cache_misses", |b| {
        b.iter(|| {
            let mut new_orderer = MoveOrdering::new();
            new_orderer.set_transposition_table(&tt);
            criterion::black_box(new_orderer.get_pv_move(&board, &captured_pieces, player, depth));
        })
    });

    group.finish();
}

/// Benchmark PV move ordering with different move counts
fn benchmark_pv_move_ordering_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_ordering_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    let test_moves = generate_test_moves_for_pv();
    let move_counts = vec![10, 25, 50, 100, 200];

    // Create and store PV move
    let pv_move = Move {
        from: Some(Position::new(1, 1)),
        to: Position::new(2, 1),
        piece_type: PieceType::Pawn,
        player,
        is_capture: false,
        is_promotion: false,
        gives_check: false,
        is_recapture: false,
        captured_piece: None,
    };

    orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move, 100);

    for count in move_counts {
        let moves_subset: Vec<Move> = test_moves.iter().take(count).cloned().collect();

        group.bench_with_input(
            BenchmarkId::new("order_moves_with_pv", count),
            &moves_subset,
            |b, moves| {
                b.iter(|| {
                    criterion::black_box(orderer.order_moves_with_pv(
                        criterion::black_box(moves),
                        &board,
                        &captured_pieces,
                        player,
                        depth,
                    ))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark PV move hit rate performance
fn benchmark_pv_move_hit_rate_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_hit_rate_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    // Create and store PV move
    let pv_move = Move {
        from: Some(Position::new(1, 1)),
        to: Position::new(2, 1),
        piece_type: PieceType::Pawn,
        player,
        is_capture: false,
        is_promotion: false,
        gives_check: false,
        is_recapture: false,
        captured_piece: None,
    };

    orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move, 100);

    group.bench_function("pv_move_statistics", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = orderer.get_pv_move(&board, &captured_pieces, player, depth);
                criterion::black_box(orderer.get_pv_stats());
                criterion::black_box(orderer.get_tt_hit_rate());
            }
        })
    });

    group.finish();
}

/// Benchmark memory usage efficiency for PV move ordering
fn benchmark_pv_move_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_memory_efficiency");
    group.measurement_time(Duration::from_secs(5));

    // Create transposition table and move orderer
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);
    let mut orderer = MoveOrdering::new();
    orderer.set_transposition_table(&tt);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    let test_moves = generate_test_moves_for_pv();
    let moves_subset: Vec<Move> = test_moves.iter().take(100).cloned().collect();

    group.bench_function("memory_usage_with_pv_operations", |b| {
        b.iter(|| {
            // Store multiple PV moves
            for (i, move_) in moves_subset.iter().take(10).enumerate() {
                orderer.update_pv_move(
                    &board,
                    &captured_pieces,
                    player,
                    depth,
                    move_.clone(),
                    100 + i as i32,
                );
            }

            // Retrieve PV moves
            for _ in 0..10 {
                let _ = orderer.get_pv_move(&board, &captured_pieces, player, depth);
            }

            // Measure memory usage
            criterion::black_box(orderer.get_memory_usage().current_bytes)
        })
    });

    group.finish();
}

/// Benchmark PV move ordering with different configurations
fn benchmark_pv_move_configuration_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("pv_move_configuration_performance");
    group.measurement_time(Duration::from_secs(10));

    // Create transposition table
    let config = TranspositionConfig::default();
    let mut tt =
        ThreadSafeTranspositionTable::with_thread_mode(config, ThreadSafetyMode::SingleThreaded);

    // Create test position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 3;

    let test_moves = generate_test_moves_for_pv();
    let moves_subset: Vec<Move> = test_moves.iter().take(50).cloned().collect();

    // Default PV move weight
    group.bench_function("default_pv_weight", |b| {
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        b.iter(|| {
            criterion::black_box(orderer.order_moves_with_pv(
                &moves_subset,
                &board,
                &captured_pieces,
                player,
                depth,
            ))
        })
    });

    // High PV move weight
    group.bench_function("high_pv_weight", |b| {
        let custom_weights = OrderingWeights {
            pv_move_weight: 50000,
            ..Default::default()
        };
        let mut orderer = MoveOrdering::with_config(custom_weights);
        orderer.set_transposition_table(&tt);

        b.iter(|| {
            criterion::black_box(orderer.order_moves_with_pv(
                &moves_subset,
                &board,
                &captured_pieces,
                player,
                depth,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_pv_move_scoring_performance,
    benchmark_pv_move_retrieval_performance,
    benchmark_pv_move_storage_performance,
    benchmark_pv_move_cache_performance,
    benchmark_pv_move_ordering_performance,
    benchmark_pv_move_hit_rate_performance,
    benchmark_pv_move_memory_efficiency,
    benchmark_pv_move_configuration_performance
);

criterion_main!(benches);
