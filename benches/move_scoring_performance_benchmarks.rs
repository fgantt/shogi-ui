use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use std::time::Duration;

/// Performance benchmarks for move scoring system
///
/// These benchmarks measure the performance of the move scoring system
/// to ensure it provides the expected speed benefits and can handle
/// high-volume operations efficiently.

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
        PieceType::King,
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

/// Benchmark comprehensive move scoring performance
fn benchmark_comprehensive_move_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_move_scoring");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("score_move", count), count, |b, &count| {
            let mut orderer = MoveOrdering::new();
            let moves = generate_test_moves(count);

            b.iter(|| {
                for move_ in &moves {
                    black_box(orderer.score_move(black_box(move_)));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark move scoring cache performance
fn benchmark_move_scoring_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_cache");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_performance", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                // Pre-populate cache
                for move_ in &moves {
                    orderer.score_move(move_);
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with different piece types
fn benchmark_move_scoring_piece_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_piece_types");
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

                b.iter(|| {
                    for move_ in &specific_moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with different positions
fn benchmark_move_scoring_positions(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_positions");
    group.measurement_time(Duration::from_secs(10));

    let positions = vec![
        Position::new(0, 0), // Corner
        Position::new(0, 4), // Edge center
        Position::new(4, 4), // Center
        Position::new(8, 8), // Opposite corner
    ];

    for position in positions {
        group.bench_with_input(
            BenchmarkId::new("position", format!("{:?}", position)),
            &position,
            |b, &position| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                // Create moves to specific position
                let specific_moves: Vec<Move> = moves
                    .iter()
                    .map(|m| create_benchmark_move(m.from, position, m.piece_type, m.player))
                    .collect();

                b.iter(|| {
                    for move_ in &specific_moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with different weights
fn benchmark_move_scoring_weights(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_weights");
    group.measurement_time(Duration::from_secs(10));

    let weight_configs = vec![
        ("default", OrderingWeights::default()),
        (
            "high_capture",
            OrderingWeights {
                capture_weight: 2000,
                ..Default::default()
            },
        ),
        (
            "high_tactical",
            OrderingWeights {
                tactical_weight: 600,
                ..Default::default()
            },
        ),
        (
            "balanced",
            OrderingWeights {
                capture_weight: 1500,
                promotion_weight: 1200,
                tactical_weight: 500,
                quiet_weight: 50,
                ..Default::default()
            },
        ),
    ];

    for (name, weights) in weight_configs {
        group.bench_with_input(
            BenchmarkId::new("weight_config", name),
            &weights,
            |b, weights| {
                let mut orderer = MoveOrdering::with_config(weights.clone());
                let moves = generate_test_moves(100);

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring cache operations
fn benchmark_move_scoring_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_cache_operations");
    group.measurement_time(Duration::from_secs(10));

    for cache_size in [100, 500, 1000, 2000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_size", cache_size),
            cache_size,
            |b, &cache_size| {
                let mut orderer = MoveOrdering::new();
                orderer.set_cache_size(cache_size);
                let moves = generate_test_moves(cache_size);

                b.iter(|| {
                    // Fill cache
                    for move_ in &moves {
                        orderer.score_move(move_);
                    }

                    // Test cache hits
                    for move_ in &moves[0..cache_size / 2] {
                        black_box(orderer.score_move(move_));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring performance optimization
fn benchmark_move_scoring_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_optimization");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("optimization", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    // Score moves
                    for move_ in &moves {
                        orderer.score_move(move_);
                    }

                    // Optimize performance
                    orderer.optimize_performance();

                    // Test cache warming
                    orderer.warm_up_cache(&moves);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring statistics tracking
fn benchmark_move_scoring_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_statistics");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("statistics_tracking", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    for move_ in &moves {
                        orderer.score_move(move_);
                    }
                    black_box(orderer.get_scoring_stats());
                    black_box(orderer.get_cache_hit_rate());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with all heuristics
fn benchmark_move_scoring_all_heuristics(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_all_heuristics");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("all_heuristics", count),
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
                for i in 0..(count / 20).max(1) {
                    orderer.add_killer_move(moves[i].clone());
                }

                // Pre-populate with history scores
                for i in 0..(count / 10).max(1) {
                    orderer.update_history_score(&moves[i], 3);
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
                    for move_ in &moves {
                        black_box(orderer.score_move_with_all_heuristics(
                            black_box(move_),
                            black_box(&Some(moves[0].clone())),
                            black_box(&moves[0..(count / 20).max(1)].to_vec()),
                        ));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring memory usage
fn benchmark_move_scoring_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_memory_usage");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    // Score moves to populate cache
                    for move_ in &moves {
                        orderer.score_move(move_);
                    }

                    // Update memory usage
                    orderer.update_memory_usage();

                    black_box(orderer.memory_usage.current_bytes);
                    black_box(orderer.memory_usage.peak_bytes);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with different players
fn benchmark_move_scoring_players(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_players");
    group.measurement_time(Duration::from_secs(10));

    let players = vec![Player::Black, Player::White];

    for player in players {
        group.bench_with_input(
            BenchmarkId::new("player", format!("{:?}", player)),
            &player,
            |b, &player| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                // Create moves with specific player
                let specific_moves: Vec<Move> = moves
                    .iter()
                    .map(|m| create_benchmark_move(m.from, m.to, m.piece_type, player))
                    .collect();

                b.iter(|| {
                    for move_ in &specific_moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring cache hit rates
fn benchmark_move_scoring_cache_hit_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_cache_hit_rates");
    group.measurement_time(Duration::from_secs(10));

    let hit_rates = vec![0.0, 25.0, 50.0, 75.0, 90.0, 100.0];

    for hit_rate in hit_rates {
        group.bench_with_input(
            BenchmarkId::new("hit_rate", format!("{:.0}%", hit_rate)),
            &hit_rate,
            |b, &hit_rate| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(100);

                // Pre-populate cache based on hit rate
                let cache_size = (moves.len() as f64 * hit_rate / 100.0) as usize;
                for move_ in &moves[0..cache_size] {
                    orderer.score_move(move_);
                }

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with large move sets
fn benchmark_move_scoring_large_sets(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_large_sets");
    group.measurement_time(Duration::from_secs(15));

    for count in [1000, 2000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large_move_sets", count),
            count,
            |b, &count| {
                let mut orderer = MoveOrdering::new();
                let moves = generate_test_moves(count);

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_move(black_box(move_)));
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_comprehensive_move_scoring,
    benchmark_move_scoring_cache,
    benchmark_move_scoring_piece_types,
    benchmark_move_scoring_positions,
    benchmark_move_scoring_weights,
    benchmark_move_scoring_cache_operations,
    benchmark_move_scoring_optimization,
    benchmark_move_scoring_statistics,
    benchmark_move_scoring_all_heuristics,
    benchmark_move_scoring_memory_usage,
    benchmark_move_scoring_players,
    benchmark_move_scoring_cache_hit_rates,
    benchmark_move_scoring_large_sets
);

criterion_main!(benches);
