#![cfg(feature = "legacy-tests")]
//! Benchmarks for SEE integration with cache eviction logic
//!
//! Measures the interaction between SEE pruning and TT cache eviction strategies

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position},
    moves::{Move, MoveGenerator},
    search::move_ordering::{CacheEvictionPolicy, MoveOrdering, MoveOrderingConfig},
};
use std::time::Duration;

/// Create a test move for benchmarking
fn create_test_move(
    from: Option<Position>,
    to: Position,
    piece_type: PieceType,
    player: Player,
    is_capture: bool,
) -> Move {
    let mut move_ = Move {
        from,
        to,
        piece_type,
        player,
        promotion: false,
        drop: from.is_none(),
        is_capture,
        captured_piece: if is_capture {
            Some(shogi_engine::types::Piece::new(
                PieceType::Pawn,
                player.opposite(),
            ))
        } else {
            None
        },
        gives_check: false,
    };
    move_
}

/// Generate test moves with mix of captures and non-captures
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
        let from = Position::new((i % 9) as u8, (i / 9) as u8);
        let to = Position::new(((i + 1) % 9) as u8, ((i + 1) / 9) as u8);
        let is_capture = i % 3 == 0; // 1/3 captures

        moves.push(create_test_move(
            Some(from),
            to,
            piece_type,
            if i % 2 == 0 {
                Player::Black
            } else {
                Player::White
            },
            is_capture,
        ));
    }

    moves
}

/// Benchmark: SEE calculation performance (Task 9.3)
fn benchmark_see_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("see_calculation");
    group.measurement_time(Duration::from_secs(10));

    let mut orderer = MoveOrdering::new();
    let board = BitboardBoard::new();

    // Test with capture moves
    let capture_move = create_test_move(
        Some(Position::new(1, 1)),
        Position::new(2, 2),
        PieceType::Pawn,
        Player::Black,
        true,
    );

    group.bench_function("calculate_see", |b| {
        b.iter(|| {
            let result = orderer.calculate_see(black_box(&capture_move), black_box(&board));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark: SEE vs MVV/LVA ordering comparison (Task 9.3)
fn benchmark_see_vs_mvvlva_ordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("see_vs_mvvlva_ordering");
    group.measurement_time(Duration::from_secs(15));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let moves = generate_test_moves(30);

    // Benchmark with SEE enabled
    let mut orderer_with_see = MoveOrdering::new();
    orderer_with_see.set_see_cache_enabled(true);

    group.bench_function("ordering_with_see", |b| {
        b.iter(|| {
            let ordered = orderer_with_see.order_moves_with_all_heuristics(
                black_box(&moves),
                black_box(&board),
                black_box(&captured),
                black_box(Player::Black),
                black_box(3),
                black_box(None),
                black_box(None),
            );
            black_box(ordered);
        });
    });

    // Benchmark with SEE disabled (MVV/LVA only)
    let mut orderer_without_see = MoveOrdering::new();
    orderer_without_see.set_see_cache_enabled(false);

    group.bench_function("ordering_without_see", |b| {
        b.iter(|| {
            let ordered = orderer_without_see.order_moves_with_all_heuristics(
                black_box(&moves),
                black_box(&board),
                black_box(&captured),
                black_box(Player::Black),
                black_box(3),
                black_box(None),
                black_box(None),
            );
            black_box(ordered);
        });
    });

    group.finish();
}

/// Benchmark: Cache eviction policy comparison (Task 9.5)
fn benchmark_cache_eviction_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_eviction_policies");
    group.measurement_time(Duration::from_secs(15));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let moves = generate_test_moves(30);

    let policies = vec![
        ("FIFO", CacheEvictionPolicy::FIFO),
        ("LRU", CacheEvictionPolicy::LRU),
        ("DepthPreferred", CacheEvictionPolicy::DepthPreferred),
        ("Hybrid", CacheEvictionPolicy::Hybrid),
    ];

    for (name, policy) in policies {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = policy;
        config.cache_config.max_cache_size = 100; // Small cache to trigger evictions

        let mut orderer = MoveOrdering::with_config(config);

        group.bench_with_input(BenchmarkId::new("eviction_policy", name), &name, |b, _| {
            b.iter(|| {
                // Perform multiple orderings to trigger evictions
                for depth in 1..=5 {
                    let ordered = orderer.order_moves_with_all_heuristics(
                        black_box(&moves),
                        black_box(&board),
                        black_box(&captured),
                        black_box(Player::Black),
                        black_box(depth),
                        black_box(None),
                        black_box(None),
                    );
                    black_box(ordered);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: SEE cache performance (Task 9.3)
fn benchmark_see_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("see_cache_performance");
    group.measurement_time(Duration::from_secs(10));

    let board = BitboardBoard::new();

    // Test with different cache sizes
    for cache_size in [100, 1000, 5000, 10000].iter() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_see_cache_size(*cache_size);
        orderer.set_see_cache_enabled(true);

        // Create capture moves
        let capture_moves: Vec<Move> = (0..20)
            .map(|i| {
                create_test_move(
                    Some(Position::new((i % 9) as u8, (i / 9) as u8)),
                    Position::new(((i + 1) % 9) as u8, ((i + 1) / 9) as u8),
                    PieceType::Pawn,
                    Player::Black,
                    true,
                )
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("see_cache_size", cache_size),
            cache_size,
            |b, _| {
                b.iter(|| {
                    // Calculate SEE for all moves (some should hit cache)
                    for move_ in &capture_moves {
                        let result = orderer.calculate_see(black_box(move_), black_box(&board));
                        black_box(result);
                    }
                    // Repeat to test cache hits
                    for move_ in &capture_moves {
                        let result = orderer.calculate_see(black_box(move_), black_box(&board));
                        black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Counter-move heuristic effectiveness (Task 9.4)
fn benchmark_counter_move_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter_move_effectiveness");
    group.measurement_time(Duration::from_secs(10));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let moves = generate_test_moves(30);

    // Create an opponent move for counter-move context
    let opponent_move = create_test_move(
        Some(Position::new(6, 4)),
        Position::new(5, 4),
        PieceType::Pawn,
        Player::White,
        false,
    );

    // Benchmark with counter-move heuristic
    let mut orderer_with_counter = MoveOrdering::new();

    group.bench_function("with_counter_move", |b| {
        b.iter(|| {
            let ordered = orderer_with_counter.order_moves_with_all_heuristics(
                black_box(&moves),
                black_box(&board),
                black_box(&captured),
                black_box(Player::Black),
                black_box(3),
                black_box(None),
                black_box(Some(&opponent_move)),
            );
            black_box(ordered);
        });
    });

    // Benchmark without counter-move heuristic (no opponent move)
    group.bench_function("without_counter_move", |b| {
        b.iter(|| {
            let ordered = orderer_with_counter.order_moves_with_all_heuristics(
                black_box(&moves),
                black_box(&board),
                black_box(&captured),
                black_box(Player::Black),
                black_box(3),
                black_box(None),
                black_box(None),
            );
            black_box(ordered);
        });
    });

    group.finish();
}

/// Benchmark: History heuristic enhancements (Task 9.6)
fn benchmark_history_heuristic_enhancements(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_heuristic_enhancements");
    group.measurement_time(Duration::from_secs(10));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let moves = generate_test_moves(30);

    // Test with different history configurations
    let configs = vec![
        ("absolute_only", false, false, false),
        ("with_relative", true, false, false),
        ("with_quiet", false, true, false),
        ("with_phase_aware", false, false, true),
        ("all_enabled", true, true, true),
    ];

    for (name, enable_relative, enable_quiet, enable_phase) in configs {
        let mut config = MoveOrderingConfig::default();
        config.history_config.enable_relative = enable_relative;
        config.history_config.enable_quiet_only = enable_quiet;
        config.history_config.enable_phase_aware = enable_phase;

        let mut orderer = MoveOrdering::with_config(config);

        group.bench_with_input(BenchmarkId::new("history_config", name), &name, |b, _| {
            b.iter(|| {
                let ordered = orderer.order_moves_with_all_heuristics(
                    black_box(&moves),
                    black_box(&board),
                    black_box(&captured),
                    black_box(Player::Black),
                    black_box(3),
                    black_box(None),
                    black_box(None),
                );
                black_box(ordered);
            });
        });
    }

    group.finish();
}

/// Benchmark: Overall move ordering with all enhancements (Task 9.2)
fn benchmark_comprehensive_move_ordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_move_ordering");
    group.measurement_time(Duration::from_secs(15));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    // Generate different move list sizes
    for move_count in [10, 30, 60].iter() {
        let moves = generate_test_moves(*move_count);

        // Test with all enhancements enabled
        let mut orderer = MoveOrdering::new();

        group.bench_with_input(
            BenchmarkId::new("all_enhancements", move_count),
            move_count,
            |b, _| {
                b.iter(|| {
                    let ordered = orderer.order_moves_with_all_heuristics(
                        black_box(&moves),
                        black_box(&board),
                        black_box(&captured),
                        black_box(Player::Black),
                        black_box(3),
                        black_box(None),
                        black_box(None),
                    );
                    black_box(ordered);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Cache hit rates with different eviction policies (Task 9.5)
fn benchmark_cache_hit_rates_by_policy(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rates_by_policy");
    group.measurement_time(Duration::from_secs(15));

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let moves = generate_test_moves(30);

    let policies = vec![
        CacheEvictionPolicy::FIFO,
        CacheEvictionPolicy::LRU,
        CacheEvictionPolicy::DepthPreferred,
        CacheEvictionPolicy::Hybrid,
    ];

    for policy in policies {
        let mut config = MoveOrderingConfig::default();
        config.cache_config.cache_eviction_policy = policy;
        config.cache_config.max_cache_size = 50; // Small cache to test eviction

        let mut orderer = MoveOrdering::with_config(config);

        let policy_name = format!("{:?}", policy);

        group.bench_with_input(
            BenchmarkId::new("policy", &policy_name),
            &policy_name,
            |b, _| {
                b.iter(|| {
                    // Perform ordering multiple times at different depths
                    // This simulates real search patterns
                    for depth in 1..=6 {
                        let ordered = orderer.order_moves_with_all_heuristics(
                            black_box(&moves),
                            black_box(&board),
                            black_box(&captured),
                            black_box(Player::Black),
                            black_box(depth),
                            black_box(None),
                            black_box(None),
                        );
                        black_box(ordered);
                    }

                    // Return cache stats for analysis
                    let (hits, misses, hit_rate) = orderer.get_cache_stats();
                    black_box((hits, misses, hit_rate));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: SEE cache eviction overhead (Task 9.3)
fn benchmark_see_cache_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("see_cache_eviction");
    group.measurement_time(Duration::from_secs(10));

    let board = BitboardBoard::new();

    // Test with small cache to force frequent evictions
    let mut orderer = MoveOrdering::new();
    orderer.set_max_see_cache_size(10);
    orderer.set_see_cache_enabled(true);

    // Create many unique capture moves
    let capture_moves: Vec<Move> = (0..100)
        .map(|i| {
            create_test_move(
                Some(Position::new((i % 9) as u8, (i / 9 % 9) as u8)),
                Position::new(((i + 1) % 9) as u8, ((i + 1) / 9 % 9) as u8),
                PieceType::Pawn,
                Player::Black,
                true,
            )
        })
        .collect();

    group.bench_function("with_frequent_evictions", |b| {
        b.iter(|| {
            for move_ in &capture_moves {
                let result = orderer.calculate_see(black_box(move_), black_box(&board));
                black_box(result);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_see_calculation,
    benchmark_see_vs_mvvlva_ordering,
    benchmark_cache_eviction_policies,
    benchmark_cache_hit_rates_by_policy,
    benchmark_counter_move_effectiveness,
    benchmark_history_heuristic_enhancements,
    benchmark_comprehensive_move_ordering,
    benchmark_see_cache_eviction
);
criterion_main!(benches);
