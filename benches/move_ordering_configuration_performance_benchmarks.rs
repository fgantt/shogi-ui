use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{
    CacheConfig, DebugConfig, HistoryConfig, KillerConfig, MoveOrdering, MoveOrderingConfig,
    OrderingWeights, PerformanceConfig,
};
use std::time::Duration;

/// Performance benchmarks for move ordering configuration system
///
/// These benchmarks measure the performance impact of different configuration
/// options and validate that the configuration system doesn't introduce
/// significant overhead.

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

/// Benchmark configuration creation performance
fn benchmark_configuration_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_creation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("default_config", |b| {
        b.iter(|| {
            black_box(MoveOrderingConfig::new());
        });
    });

    group.bench_function("performance_optimized_config", |b| {
        b.iter(|| {
            black_box(MoveOrderingConfig::performance_optimized());
        });
    });

    group.bench_function("debug_optimized_config", |b| {
        b.iter(|| {
            black_box(MoveOrderingConfig::debug_optimized());
        });
    });

    group.bench_function("memory_optimized_config", |b| {
        b.iter(|| {
            black_box(MoveOrderingConfig::memory_optimized());
        });
    });

    group.finish();
}

/// Benchmark configuration validation performance
fn benchmark_configuration_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_validation");
    group.measurement_time(Duration::from_secs(10));

    let valid_config = MoveOrderingConfig::new();

    group.bench_function("validate_valid_config", |b| {
        b.iter(|| {
            black_box(valid_config.validate());
        });
    });

    let mut invalid_config = MoveOrderingConfig::new();
    invalid_config.weights.capture_weight = -1;

    group.bench_function("validate_invalid_config", |b| {
        b.iter(|| {
            black_box(invalid_config.validate());
        });
    });

    group.finish();
}

/// Benchmark move ordering creation with different configurations
fn benchmark_move_ordering_creation_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_creation_with_configs");
    group.measurement_time(Duration::from_secs(10));

    let configs = vec![
        ("default", MoveOrderingConfig::new()),
        ("performance", MoveOrderingConfig::performance_optimized()),
        ("debug", MoveOrderingConfig::debug_optimized()),
        ("memory", MoveOrderingConfig::memory_optimized()),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::new("creation", name), &config, |b, config| {
            b.iter(|| {
                black_box(MoveOrdering::with_config(config.clone()));
            });
        });
    }

    group.finish();
}

/// Benchmark configuration updates performance
fn benchmark_configuration_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_updates");
    group.measurement_time(Duration::from_secs(10));

    let mut orderer = MoveOrdering::new();

    group.bench_function("update_weights", |b| {
        b.iter(|| {
            orderer.set_capture_weight(black_box(1000));
            orderer.set_promotion_weight(black_box(800));
            orderer.set_tactical_weight(black_box(300));
            orderer.set_quiet_weight(black_box(25));
        });
    });

    group.bench_function("update_cache_config", |b| {
        b.iter(|| {
            orderer.set_cache_size(black_box(1000));
        });
    });

    group.bench_function("update_killer_config", |b| {
        b.iter(|| {
            orderer.set_max_killer_moves_per_depth(black_box(2));
        });
    });

    group.bench_function("update_history_config", |b| {
        b.iter(|| {
            orderer.set_max_history_score(black_box(10000));
            orderer.set_history_aging_factor(black_box(0.9));
        });
    });

    group.finish();
}

/// Benchmark configuration application performance
fn benchmark_configuration_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_application");
    group.measurement_time(Duration::from_secs(10));

    let configs = vec![
        ("default", MoveOrderingConfig::new()),
        ("performance", MoveOrderingConfig::performance_optimized()),
        ("debug", MoveOrderingConfig::debug_optimized()),
        ("memory", MoveOrderingConfig::memory_optimized()),
    ];

    for (name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("apply_config", name),
            &config,
            |b, config| {
                let mut orderer = MoveOrdering::new();

                // Pre-populate with data
                let moves = generate_test_moves(100);
                for move_ in &moves {
                    orderer.score_move(move_);
                }

                b.iter(|| {
                    black_box(orderer.set_config(config.clone()));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark move scoring with different configurations
fn benchmark_move_scoring_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_scoring_with_configs");
    group.measurement_time(Duration::from_secs(10));

    let configs = vec![
        ("default", MoveOrderingConfig::new()),
        ("high_weights", {
            let mut config = MoveOrderingConfig::new();
            config.weights.capture_weight = 5000;
            config.weights.promotion_weight = 4000;
            config.weights.tactical_weight = 2000;
            config
        }),
        ("low_weights", {
            let mut config = MoveOrderingConfig::new();
            config.weights.capture_weight = 100;
            config.weights.promotion_weight = 80;
            config.weights.tactical_weight = 30;
            config
        }),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::new("scoring", name), &config, |b, config| {
            let mut orderer = MoveOrdering::with_config(config.clone());
            let moves = generate_test_moves(100);

            b.iter(|| {
                for move_ in &moves {
                    black_box(orderer.score_move(move_));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark cache performance with different cache configurations
fn benchmark_cache_performance_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance_with_configs");
    group.measurement_time(Duration::from_secs(10));

    let cache_sizes = vec![100, 500, 1000, 2000, 5000];

    for cache_size in cache_sizes {
        group.bench_with_input(
            BenchmarkId::new("cache_size", cache_size),
            &cache_size,
            |b, &cache_size| {
                let mut config = MoveOrderingConfig::new();
                config.cache_config.max_cache_size = cache_size;

                let mut orderer = MoveOrdering::with_config(config);
                let moves = generate_test_moves(cache_size * 2); // More moves than cache size

                b.iter(|| {
                    for move_ in &moves {
                        black_box(orderer.score_move(move_));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark killer move performance with different configurations
fn benchmark_killer_move_performance_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("killer_move_performance_with_configs");
    group.measurement_time(Duration::from_secs(10));

    let killer_limits = vec![1, 2, 3, 4, 5];

    for killer_limit in killer_limits {
        group.bench_with_input(
            BenchmarkId::new("killer_limit", killer_limit),
            &killer_limit,
            |b, &killer_limit| {
                let mut config = MoveOrderingConfig::new();
                config.killer_config.max_killer_moves_per_depth = killer_limit;

                let mut orderer = MoveOrdering::with_config(config);
                orderer.set_current_depth(3);

                let killer_moves = generate_test_moves(killer_limit * 2);

                b.iter(|| {
                    for killer_move in &killer_moves {
                        orderer.add_killer_move(killer_move.clone());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history heuristic performance with different configurations
fn benchmark_history_performance_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_performance_with_configs");
    group.measurement_time(Duration::from_secs(10));

    let max_scores = vec![1000, 5000, 10000, 15000, 20000];

    for max_score in max_scores {
        group.bench_with_input(
            BenchmarkId::new("max_score", max_score),
            &max_score,
            |b, &max_score| {
                let mut config = MoveOrderingConfig::new();
                config.history_config.max_history_score = max_score;

                let mut orderer = MoveOrdering::with_config(config);

                let history_moves = generate_test_moves(100);

                b.iter(|| {
                    for (i, move_) in history_moves.iter().enumerate() {
                        orderer.update_history_score(move_, (i % 10) as u8);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark configuration merging performance
fn benchmark_configuration_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_merging");
    group.measurement_time(Duration::from_secs(10));

    let base_config = MoveOrderingConfig::new();
    let mut override_config = MoveOrderingConfig::new();
    override_config.weights.capture_weight = 3000;
    override_config.cache_config.max_cache_size = 2000;
    override_config.killer_config.max_killer_moves_per_depth = 4;

    group.bench_function("merge_configs", |b| {
        b.iter(|| {
            black_box(base_config.merge(&override_config));
        });
    });

    group.finish();
}

/// Benchmark configuration validation with multiple errors
fn benchmark_configuration_validation_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_validation_complex");
    group.measurement_time(Duration::from_secs(10));

    let mut config = MoveOrderingConfig::new();
    // Make multiple validation errors
    config.weights.capture_weight = -1;
    config.cache_config.max_cache_size = 0;
    config.history_config.history_aging_factor = 1.5;
    config.debug_config.log_level = 5;

    group.bench_function("validate_complex_invalid", |b| {
        b.iter(|| {
            black_box(config.validate());
        });
    });

    group.finish();
}

/// Benchmark move ordering performance with specialized configurations
fn benchmark_move_ordering_with_specialized_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_with_specialized_configs");
    group.measurement_time(Duration::from_secs(15));

    let configs = vec![
        ("performance", MoveOrderingConfig::performance_optimized()),
        ("debug", MoveOrderingConfig::debug_optimized()),
        ("memory", MoveOrderingConfig::memory_optimized()),
    ];

    for (name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("full_ordering", name),
            &config,
            |b, config| {
                let mut orderer = MoveOrdering::with_config(config.clone());
                let moves = generate_test_moves(100);

                b.iter(|| {
                    black_box(orderer.order_moves(&moves));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark configuration overhead
fn benchmark_configuration_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_overhead");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark without configuration (using default constructor)
    group.bench_function("default_constructor", |b| {
        b.iter(|| {
            let mut orderer = MoveOrdering::new();
            let moves = generate_test_moves(50);
            black_box(orderer.order_moves(&moves));
        });
    });

    // Benchmark with explicit configuration
    group.bench_function("explicit_config", |b| {
        b.iter(|| {
            let config = MoveOrderingConfig::new();
            let mut orderer = MoveOrdering::with_config(config);
            let moves = generate_test_moves(50);
            black_box(orderer.order_moves(&moves));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_configuration_creation,
    benchmark_configuration_validation,
    benchmark_move_ordering_creation_with_configs,
    benchmark_configuration_updates,
    benchmark_configuration_application,
    benchmark_move_scoring_with_configs,
    benchmark_cache_performance_with_configs,
    benchmark_killer_move_performance_with_configs,
    benchmark_history_performance_with_configs,
    benchmark_configuration_merging,
    benchmark_configuration_validation_complex,
    benchmark_move_ordering_with_specialized_configs,
    benchmark_configuration_overhead
);

criterion_main!(benches);
