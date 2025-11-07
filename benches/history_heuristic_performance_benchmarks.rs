use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use std::time::Duration;

/// Performance benchmarks for history heuristic operations
///
/// These benchmarks measure the performance of history heuristic operations
/// to ensure they provide the expected speed benefits without
/// introducing significant overhead.

/// Create a test move for benchmarking
fn create_benchmark_move(
    from: Option<Position>,
    to: Position,
    piece_type: PieceType,
    player: Player,
) -> Move {
    Move {
        from,
        to,
        piece_type,
        player,
        promotion: false,
        drop: from.is_none(),
    }
}

/// Generate a set of test moves for benchmarking
fn generate_test_moves(count: usize) -> Vec<Move> {
    let mut moves = Vec::new();
    let piece_types = vec![
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
    ];

    for i in 0..count {
        let piece_type = piece_types[i % piece_types.len()];
        let from = Position::new(i % 9, i / 9);
        let to = Position::new((i + 1) % 9, (i + 1) / 9);

        moves.push(create_benchmark_move(
            Some(from),
            to,
            piece_type,
            if i % 2 == 0 {
                Player::Black
            } else {
                Player::White
            },
        ));
    }

    moves
}

/// Benchmark history score update performance
fn benchmark_history_score_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_score_update");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("update_history_score", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    for (i, move_) in moves.iter().enumerate() {
                        orderer.update_history_score(black_box(move_), (i % 10 + 1) as u8);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history score lookup performance
fn benchmark_history_score_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_score_lookup");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("get_history_score", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for (i, move_) in moves.iter().enumerate() {
                    orderer.update_history_score(move_, (i % 10 + 1) as u8);
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.get_history_score(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history move scoring performance
fn benchmark_history_move_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_move_scoring");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("score_history_move", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for (i, move_) in moves.iter().enumerate() {
                    orderer.update_history_score(move_, (i % 10 + 1) as u8);
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_history_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history move ordering performance
fn benchmark_history_move_ordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_move_ordering");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("order_moves_with_history", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with some history scores
                for i in 0..(count / 10).max(1) {
                    orderer.update_history_score(&moves[i], 3);
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_history(black_box(&moves)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history table aging performance
fn benchmark_history_table_aging(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_table_aging");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("age_history_table", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for (i, move_) in moves.iter().enumerate() {
                    orderer.update_history_score(move_, (i % 10 + 1) as u8);
                }

                b.iter(|| {
                    black_box(orderer.age_history_table());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history table memory usage
fn benchmark_history_table_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_table_memory_usage");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    // Add history scores
                    for (i, move_) in moves.iter().enumerate() {
                        orderer.update_history_score(move_, (i % 10 + 1) as u8);
                    }

                    // Update memory usage
                    orderer.update_memory_usage();

                    black_box(orderer.memory_usage.current_bytes);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic with different depths
fn benchmark_history_heuristic_different_depths(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_different_depths");
    group.measurement_time(Duration::from_secs(10));

    for depth in [1, 3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("depth_updates", depth),
            depth,
            |b, &depth| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for move_ in &moves {
                        orderer.update_history_score(move_, depth);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic configuration changes
fn benchmark_history_heuristic_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_configuration");
    group.measurement_time(Duration::from_secs(10));

    for max_score in [1000, 5000, 10000, 20000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::new("max_history_score", max_score),
            max_score,
            |b, &max_score| {
                let mut orderer = MoveOrdering::new();
                orderer.set_max_history_score(max_score);
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for move_ in &moves {
                        orderer.update_history_score(move_, 10);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic statistics tracking
fn benchmark_history_heuristic_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_statistics");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("statistics_tracking", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for (i, move_) in moves.iter().enumerate() {
                    orderer.update_history_score(move_, (i % 10 + 1) as u8);
                }

                b.iter(|| {
                    for move_ in &moves {
                        orderer.score_history_move(move_);
                    }
                    black_box(orderer.get_history_stats());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic cleanup performance
fn benchmark_history_heuristic_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_cleanup");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("clear_history_table", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for (i, move_) in moves.iter().enumerate() {
                    orderer.update_history_score(move_, (i % 10 + 1) as u8);
                }

                b.iter(|| {
                    orderer.clear_history_table();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic with all heuristics integration
fn benchmark_history_heuristic_all_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_all_integration");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("order_moves_with_all_heuristics", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                // Create test position
                let board = BitboardBoard::new();
                let captured_pieces = CapturedPieces::new();
                let player = Player::Black;
                let depth = 3;

                // Pre-populate with history scores
                for i in 0..(count / 10).max(1) {
                    orderer.update_history_score(&moves[i], 3);
                }

                // Pre-populate with killer moves
                for i in 0..(count / 20).max(1) {
                    orderer.add_killer_move(moves[i].clone());
                }

                // Store PV move
                if !moves.is_empty() {
                    orderer.update_pv_move(
                        &board,
                        &captured_pieces,
                        player,
                        depth,
                        moves[0].clone(),
                        100,
                    );
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_all_heuristics(
                        black_box(&moves),
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                    ));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic with different weights
fn benchmark_history_heuristic_weights(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_weights");
    group.measurement_time(Duration::from_secs(10));

    for weight in [1000, 2500, 5000, 7500, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("history_weight", weight),
            weight,
            |b, &weight| {
                let custom_weights = OrderingWeights {
                    history_weight: weight,
                    ..Default::default()
                };
                let mut orderer = MoveOrdering::with_config(custom_weights);
                let moves = generate_test_moves(100);

                // Pre-populate with history scores
                for move_ in &moves {
                    orderer.update_history_score(move_, 3);
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_history_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic with large move sets
fn benchmark_history_heuristic_large_sets(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_large_sets");
    group.measurement_time(Duration::from_secs(15));

    for count in [1000, 2000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large_move_sets", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with history scores
                for i in 0..(count / 20).max(1) {
                    orderer.update_history_score(&moves[i], 3);
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_history(black_box(&moves)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic with different piece types
fn benchmark_history_heuristic_piece_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_piece_types");
    group.measurement_time(Duration::from_secs(10));

    let piece_types = vec![
        PieceType::Pawn,
        PieceType::Lance,
        PieceType::Knight,
        PieceType::Silver,
        PieceType::Gold,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::King,
    ];

    for piece_type in piece_types {
        group.bench_with_input(
            BenchmarkId::new("piece_type", format!("{:?}", piece_type)),
            &piece_type,
            |b, &piece_type| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                // Create moves with specific piece type
                let specific_moves: Vec<Move> = moves
                    .iter()
                    .map(|m| create_benchmark_move(m.from, m.to, piece_type, m.player))
                    .collect();

                // Pre-populate with history scores
                for move_ in &specific_moves[0..10] {
                    orderer.update_history_score(move_, 3);
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_history(black_box(&specific_moves)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic aging factor performance
fn benchmark_history_heuristic_aging_factor(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_aging_factor");
    group.measurement_time(Duration::from_secs(10));

    for aging_factor in [0.5, 0.7, 0.8, 0.9, 0.95].iter() {
        group.bench_with_input(
            BenchmarkId::new("aging_factor", format!("{:.2}", aging_factor)),
            aging_factor,
            |b, &aging_factor| {
                let mut orderer = MoveOrdering::new();
                orderer.set_history_aging_factor(aging_factor);
                let moves = generate_test_moves(100);

                // Pre-populate with history scores
                for move_ in &moves {
                    orderer.update_history_score(move_, 3);
                }

                b.iter(|| {
                    black_box(orderer.age_history_table());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_history_score_update,
    benchmark_history_score_lookup,
    benchmark_history_move_scoring,
    benchmark_history_move_ordering,
    benchmark_history_table_aging,
    benchmark_history_table_memory_usage,
    benchmark_history_heuristic_different_depths,
    benchmark_history_heuristic_configuration,
    benchmark_history_heuristic_statistics,
    benchmark_history_heuristic_cleanup,
    benchmark_history_heuristic_all_integration,
    benchmark_history_heuristic_weights,
    benchmark_history_heuristic_large_sets,
    benchmark_history_heuristic_piece_types,
    benchmark_history_heuristic_aging_factor
);

criterion_main!(benches);
