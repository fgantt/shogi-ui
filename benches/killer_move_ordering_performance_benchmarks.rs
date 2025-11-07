use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use std::time::Duration;

/// Performance benchmarks for killer move ordering operations
///
/// These benchmarks measure the performance of killer move operations
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

/// Benchmark killer move storage performance
fn benchmark_killer_move_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_storage");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_killer_moves", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                b.iter(|| {
                    for move_ in &moves {
                        orderer.add_killer_move(black_box(move_.clone()));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move detection performance
fn benchmark_killer_move_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_detection");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("is_killer_move", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                // Pre-populate with killer moves
                for move_ in &moves {
                    orderer.add_killer_move(move_.clone());
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.is_killer_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move scoring performance
fn benchmark_killer_move_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_scoring");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("score_killer_move", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_killer_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move ordering performance
fn benchmark_killer_move_ordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_ordering");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("order_moves_with_killer", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                // Pre-populate with some killer moves
                for i in 0..(count / 10).max(1) {
                    orderer.add_killer_move(moves[i].clone());
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_killer(black_box(&moves)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move memory usage
fn benchmark_killer_move_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_memory_usage");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                b.iter(|| {
                    // Add killer moves
                    for move_ in &moves {
                        orderer.add_killer_move(move_.clone());
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

/// Benchmark killer move with different depths
fn benchmark_killer_move_different_depths(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_different_depths");
    group.measurement_time(Duration::from_secs(10));

    for depth in [1, 3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("depth_management", depth),
            depth,
            |b, &depth| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for d in 1..=depth {
                        orderer.set_current_depth(d);
                        for move_ in &moves[0..10] {
                            orderer.add_killer_move(move_.clone());
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move configuration changes
fn benchmark_killer_move_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_configuration");
    group.measurement_time(Duration::from_secs(10));

    for max_moves in [1, 2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("max_moves_per_depth", max_moves),
            max_moves,
            |b, &max_moves| {
                let mut orderer = MoveOrdering::new();
                orderer.set_max_killer_moves_per_depth(max_moves);
                orderer.set_current_depth(3);
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for move_ in &moves {
                        orderer.add_killer_move(move_.clone());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move statistics tracking
fn benchmark_killer_move_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_statistics");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("statistics_tracking", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                // Pre-populate with killer moves
                for move_ in &moves {
                    orderer.add_killer_move(move_.clone());
                }

                b.iter(|| {
                    for move_ in &moves {
                        orderer.is_killer_move(move_);
                    }
                    black_box(orderer.get_killer_move_stats());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move cleanup performance
fn benchmark_killer_move_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_cleanup");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("clear_all_killer_moves", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate with killer moves at multiple depths
                for depth in 1..=5 {
                    orderer.set_current_depth(depth);
                    for move_ in &moves[0..(count / 5).max(1)] {
                        orderer.add_killer_move(move_.clone());
                    }
                }

                b.iter(|| {
                    orderer.clear_all_killer_moves();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move with PV move integration
fn benchmark_killer_move_pv_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_pv_integration");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("order_moves_with_pv_and_killer", count),
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

                // Pre-populate with killer moves
                for i in 0..(count / 10).max(1) {
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
                    black_box(orderer.order_moves_with_pv_and_killer(
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

/// Benchmark killer move with different weights
fn benchmark_killer_move_weights(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_weights");
    group.measurement_time(Duration::from_secs(10));

    for weight in [1000, 5000, 10000, 15000, 20000].iter() {
        group.bench_with_input(
            BenchmarkId::new("killer_move_weight", weight),
            weight,
            |b, &weight| {
                let custom_weights = OrderingWeights {
                    killer_move_weight: weight,
                    ..Default::default()
                };
                let mut orderer = MoveOrdering::with_config(custom_weights);
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_killer_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move with large move sets
fn benchmark_killer_move_large_sets(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_large_sets");
    group.measurement_time(Duration::from_secs(15));

    for count in [1000, 2000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large_move_sets", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                orderer.set_current_depth(3);
                let moves = generate_test_moves(count);

                // Pre-populate with killer moves
                for i in 0..(count / 20).max(1) {
                    orderer.add_killer_move(moves[i].clone());
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_killer(black_box(&moves)));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move with different piece types
fn benchmark_killer_move_piece_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_piece_types");
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
                orderer.set_current_depth(3);
                let moves = generate_test_moves(100);

                // Create moves with specific piece type
                let specific_moves: Vec<Move> = moves
                    .iter()
                    .map(|m| create_benchmark_move(m.from, m.to, piece_type, m.player))
                    .collect();

                // Pre-populate with killer moves
                for move_ in &specific_moves[0..10] {
                    orderer.add_killer_move(move_.clone());
                }

                b.iter(|| {
                    black_box(orderer.order_moves_with_killer(black_box(&specific_moves)));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_killer_move_storage,
    benchmark_killer_move_detection,
    benchmark_killer_move_scoring,
    benchmark_killer_move_ordering,
    benchmark_killer_move_memory_usage,
    benchmark_killer_move_different_depths,
    benchmark_killer_move_configuration,
    benchmark_killer_move_statistics,
    benchmark_killer_move_cleanup,
    benchmark_killer_move_pv_integration,
    benchmark_killer_move_weights,
    benchmark_killer_move_large_sets,
    benchmark_killer_move_piece_types
);

criterion_main!(benches);
