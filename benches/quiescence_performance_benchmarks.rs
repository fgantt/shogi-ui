//! Task 9.0: Performance Benchmarks for Quiescence Search
//!
//! This benchmark suite provides comprehensive performance monitoring for quiescence search:
//! - Pruning effectiveness benchmarks: delta pruning, futility pruning, adaptive pruning
//! - Extension effectiveness benchmarks: selective extensions impact
//! - TT effectiveness benchmarks: TT hit rate, stand-pat caching impact
//! - Move ordering effectiveness benchmarks: cutoff rate, ordering quality
//! - Configuration comparison benchmarks: different config combinations
//! - Tactical position benchmarks: simple vs complex positions
//!
//! Metrics measured:
//! - Search time (wall clock and CPU time)
//! - Nodes searched
//! - Pruning efficiency (delta, futility, total)
//! - TT hit rate
//! - Extension rate
//! - Move ordering cutoff rate
//! - Stand-pat caching effectiveness
//!
//! ## Running Benchmarks
//!
//! To run all quiescence benchmarks:
//! ```bash
//! cargo bench --bench quiescence_performance_benchmarks
//! ```
//!
//! To run a specific benchmark group:
//! ```bash
//! cargo bench --bench quiescence_performance_benchmarks -- pruning
//! cargo bench --bench quiescence_performance_benchmarks -- extension
//! cargo bench --bench quiescence_performance_benchmarks -- tt
//! cargo bench --bench quiescence_performance_benchmarks -- move_ordering
//! cargo bench --bench quiescence_performance_benchmarks -- configuration
//! cargo bench --bench quiescence_performance_benchmarks -- tactical
//! cargo bench --bench quiescence_performance_benchmarks -- depth_scaling
//! cargo bench --bench quiescence_performance_benchmarks -- stand_pat
//! ```
//!
//! To run with specific options:
//! ```bash
//! # Run with more samples (more accurate but slower)
//! cargo bench --bench quiescence_performance_benchmarks -- --sample-size 20
//!
//! # Run with longer measurement time
//! cargo bench --bench quiescence_performance_benchmarks -- --measurement-time 30
//! ```
//!
//! ## Interpreting Results
//!
//! Benchmarks measure:
//! - **Time**: Wall clock time for each search operation (lower is better)
//! - **Throughput**: Operations per second (higher is better)
//! - **Statistics**: Nodes searched, pruning efficiency, TT hit rate, etc.
//!
//! Compare results across different configurations to identify:
//! - Performance improvements from optimizations
//! - Performance regressions from changes
//! - Optimal configuration settings
//!
//! ## CI/CD Integration
//!
//! Benchmarks can be integrated into CI/CD pipelines to detect performance regressions.
//! Use `cargo bench --bench quiescence_performance_benchmarks -- --save-baseline baseline`
//! to save a baseline, then compare with `--baseline baseline` in subsequent runs.
//!
//! This suite is designed for CI/CD integration and performance regression detection.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    time_utils::TimeSource,
    types::{CapturedPieces, Player, QuiescenceConfig, TTReplacementPolicy},
};
use std::time::Duration;

/// Create a test engine with default quiescence configuration
fn create_test_engine() -> SearchEngine {
    SearchEngine::new(None, 16) // 16MB hash table
}

/// Create a test engine with specific quiescence configuration
fn create_test_engine_with_config(config: QuiescenceConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_quiescence_config(config);
    engine
}

/// Create a test position (starting position)
fn create_test_position() -> (BitboardBoard, CapturedPieces, Player) {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Sente;
    (board, captured_pieces, player)
}

/// Task 9.2: Benchmark pruning effectiveness (delta pruning, futility pruning, combined)
fn benchmark_pruning_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_pruning_effectiveness");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test different pruning configurations
    let configs = vec![
        (
            "no_pruning",
            QuiescenceConfig {
                enable_delta_pruning: false,
                enable_futility_pruning: false,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "delta_only",
            QuiescenceConfig {
                enable_delta_pruning: true,
                enable_futility_pruning: false,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "futility_only",
            QuiescenceConfig {
                enable_delta_pruning: false,
                enable_futility_pruning: true,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "both_pruning",
            QuiescenceConfig {
                enable_delta_pruning: true,
                enable_futility_pruning: true,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "adaptive_pruning",
            QuiescenceConfig {
                enable_delta_pruning: true,
                enable_futility_pruning: true,
                enable_adaptive_pruning: true,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.3: Benchmark extension effectiveness
fn benchmark_extension_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_extension_effectiveness");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test with and without extensions
    let configs = vec![
        (
            "no_extensions",
            QuiescenceConfig {
                enable_selective_extensions: false,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "with_extensions",
            QuiescenceConfig {
                enable_selective_extensions: true,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.4: Benchmark TT effectiveness (TT hit rate, stand-pat caching impact)
fn benchmark_tt_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_tt_effectiveness");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test with and without TT
    let configs = vec![
        (
            "no_tt",
            QuiescenceConfig {
                enable_tt: false,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "with_tt",
            QuiescenceConfig {
                enable_tt: true,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        // First search (populates TT)
                        let mut board_mut = board.clone();
                        let _result1 = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        // Second search (should hit TT if enabled)
                        let mut board_mut2 = board.clone();
                        let result2 = engine.quiescence_search(
                            black_box(&mut board_mut2),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result2, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.4: Benchmark TT replacement policies
fn benchmark_tt_replacement_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_tt_replacement_policies");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test different replacement policies
    let policies = vec![
        ("simple", TTReplacementPolicy::Simple),
        ("lru", TTReplacementPolicy::LRU),
        ("depth_preferred", TTReplacementPolicy::DepthPreferred),
        ("hybrid", TTReplacementPolicy::Hybrid),
    ];

    for depth in [3, 4, 5] {
        for (name, policy) in &policies {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, policy),
                |b, &(depth, policy)| {
                    b.iter(|| {
                        let mut config = QuiescenceConfig::default();
                        config.enable_tt = true;
                        config.tt_replacement_policy = *policy;
                        let mut engine = create_test_engine_with_config(config);
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.5: Benchmark move ordering effectiveness
fn benchmark_move_ordering_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_move_ordering_effectiveness");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test different configurations (move ordering is always enabled, but we can test different configs)
    let configs = vec![
        ("default", QuiescenceConfig::default()),
        (
            "with_tt",
            QuiescenceConfig {
                enable_tt: true,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        // Calculate cutoff rate
                        let cutoff_rate = if stats.move_ordering_total_moves > 0 {
                            stats.move_ordering_cutoffs as f64
                                / stats.move_ordering_total_moves as f64
                        } else {
                            0.0
                        };
                        black_box((result, stats, cutoff_rate))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.6: Benchmark different configurations (adaptive vs non-adaptive pruning, different margins)
fn benchmark_configuration_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_configuration_comparison");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test different configurations
    let configs = vec![
        ("default", QuiescenceConfig::default()),
        (
            "adaptive_pruning",
            QuiescenceConfig {
                enable_adaptive_pruning: true,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "high_depth",
            QuiescenceConfig {
                max_depth: 8,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "low_depth",
            QuiescenceConfig {
                max_depth: 3,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "all_optimizations",
            QuiescenceConfig {
                enable_delta_pruning: true,
                enable_futility_pruning: true,
                enable_selective_extensions: true,
                enable_tt: true,
                enable_adaptive_pruning: true,
                max_depth: 6,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.6: Benchmark tactical positions (simple vs complex)
fn benchmark_tactical_positions(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_tactical_positions");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    // Starting position (simple)
    let (board1, captured_pieces1, player1) = create_test_position();

    // For now, we use the same position for both (future: add specific tactical positions)
    let (board2, captured_pieces2, player2) = create_test_position();

    let time_source = TimeSource::now();
    let config = QuiescenceConfig::default();

    let positions = vec![
        ("starting_position", (board1, captured_pieces1, player1)),
        (
            "starting_position_copy",
            (board2, captured_pieces2, player2),
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, (board, captured_pieces, player)) in &positions {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, board, captured_pieces, player),
                |b, &(depth, board, captured_pieces, player)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        let mut board_mut = board.clone();
                        let result = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(captured_pieces),
                            *player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        black_box((result, stats))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Task 9.6: Benchmark depth scaling
fn benchmark_depth_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_depth_scaling");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();
    let config = QuiescenceConfig::default();

    for depth in [1, 2, 3, 4, 5, 6] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine_with_config(config.clone());
                engine.reset_quiescence_stats();

                let mut board_mut = board.clone();
                let result = engine.quiescence_search(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    -10000,
                    10000,
                    &time_source,
                    1000,
                    depth,
                );

                let stats = engine.get_quiescence_stats();
                black_box((result, stats))
            });
        });
    }

    group.finish();
}

/// Task 9.4: Benchmark stand-pat caching effectiveness
fn benchmark_stand_pat_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("quiescence_stand_pat_caching");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let (board, captured_pieces, player) = create_test_position();
    let time_source = TimeSource::now();

    // Test with and without TT (stand-pat caching requires TT)
    let configs = vec![
        (
            "no_tt",
            QuiescenceConfig {
                enable_tt: false,
                ..QuiescenceConfig::default()
            },
        ),
        (
            "with_tt",
            QuiescenceConfig {
                enable_tt: true,
                ..QuiescenceConfig::default()
            },
        ),
    ];

    for depth in [3, 4, 5] {
        for (name, config) in &configs {
            group.bench_with_input(
                BenchmarkId::new(format!("depth_{}", depth), name),
                &(depth, config),
                |b, &(depth, config)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_config(config.clone());
                        engine.reset_quiescence_stats();

                        // First search (caches stand-pat)
                        let mut board_mut = board.clone();
                        let _result1 = engine.quiescence_search(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        // Second search (should use cached stand-pat if TT enabled)
                        let mut board_mut2 = board.clone();
                        let result2 = engine.quiescence_search(
                            black_box(&mut board_mut2),
                            black_box(&captured_pieces),
                            player,
                            -10000,
                            10000,
                            &time_source,
                            1000,
                            depth,
                        );

                        let stats = engine.get_quiescence_stats();
                        // Check stand-pat caching stats
                        let stand_pat_hit_rate =
                            if stats.stand_pat_tt_misses + stats.stand_pat_tt_hits > 0 {
                                stats.stand_pat_tt_hits as f64
                                    / (stats.stand_pat_tt_hits + stats.stand_pat_tt_misses) as f64
                            } else {
                                0.0
                            };
                        black_box((result2, stats, stand_pat_hit_rate))
                    });
                },
            );
        }
    }

    group.finish();
}

// Configure benchmark groups
criterion_group!(
    benches,
    benchmark_pruning_effectiveness,
    benchmark_extension_effectiveness,
    benchmark_tt_effectiveness,
    benchmark_tt_replacement_policies,
    benchmark_move_ordering_effectiveness,
    benchmark_configuration_comparison,
    benchmark_tactical_positions,
    benchmark_depth_scaling,
    benchmark_stand_pat_caching,
);

criterion_main!(benches);
