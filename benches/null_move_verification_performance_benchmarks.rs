//! Performance benchmarks for null move pruning verification search
//!
//! This benchmark suite measures the performance impact of verification search
//! on null move pruning effectiveness. It compares:
//! - NMP with verification search disabled (verification_margin = 0)
//! - NMP with verification search enabled (verification_margin = 200)
//!
//! Metrics measured:
//! - Search time (overhead)
//! - Nodes searched
//! - NMP cutoff rate (effectiveness)
//! - Verification search attempts and cutoffs
//!
//! Expected results:
//! - Verification search should have minimal overhead (<10% increase in search time)
//! - NMP effectiveness should remain high (cutoff rate reduction <5%)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with specific null move configuration
fn create_test_engine_with_config(nmp_config: NullMoveConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    engine.update_null_move_config(nmp_config.clone()).unwrap();
    engine
}

/// Benchmark NMP with verification search disabled
fn benchmark_nmp_without_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_without_verification");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("search_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_config(NullMoveConfig {
                        enabled: true,
                        min_depth: 3,
                        reduction_factor: 2,
                        max_pieces_threshold: 12,
                        enable_dynamic_reduction: true,
                        enable_endgame_detection: true,
                        verification_margin: 0, // Verification disabled
                    });

                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();

                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark NMP with verification search enabled
fn benchmark_nmp_with_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_with_verification");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("search_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_config(NullMoveConfig {
                        enabled: true,
                        min_depth: 3,
                        reduction_factor: 2,
                        max_pieces_threshold: 12,
                        enable_dynamic_reduction: true,
                        enable_endgame_detection: true,
                        verification_margin: 200, // Verification enabled
                    });

                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();

                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark NMP effectiveness comparison
fn benchmark_nmp_effectiveness_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("nmp_effectiveness_comparison");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Compare at depth 5
    let depth = 5;

    // Benchmark without verification
    group.bench_function("without_verification", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(NullMoveConfig {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 0,
            });

            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats();
            let cutoff_rate = stats.cutoff_rate();

            black_box((result, cutoff_rate))
        });
    });

    // Benchmark with verification
    group.bench_function("with_verification", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(NullMoveConfig {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 200,
            });

            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats().clone();
            let cutoff_rate = stats.cutoff_rate();
            let verification_cutoff_rate = stats.verification_cutoff_rate();

            black_box((result, cutoff_rate, verification_cutoff_rate))
        });
    });

    group.finish();
}

/// Benchmark verification search overhead at different margins
fn benchmark_verification_margin_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_margin_overhead");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test different verification margins
    for margin in [0, 50, 100, 200, 500] {
        group.bench_with_input(
            BenchmarkId::new("verification_margin", margin),
            &margin,
            |b, &margin| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_config(NullMoveConfig {
                        enabled: true,
                        min_depth: 3,
                        reduction_factor: 2,
                        max_pieces_threshold: 12,
                        enable_dynamic_reduction: true,
                        enable_endgame_detection: true,
                        verification_margin: margin,
                    });

                    engine.reset_null_move_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_null_move_stats().clone();

                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark verification search statistics tracking
fn benchmark_verification_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("verification_statistics");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    for depth in [3, 4, 5, 6] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine_with_config(NullMoveConfig {
                    enabled: true,
                    min_depth: 3,
                    reduction_factor: 2,
                    max_pieces_threshold: 12,
                    enable_dynamic_reduction: true,
                    enable_endgame_detection: true,
                    verification_margin: 200,
                });

                engine.reset_null_move_stats();

                let mut board_mut = board.clone();
                let _result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                let stats = engine.get_null_move_stats();

                // Verify statistics are tracked correctly
                let verification_rate = if stats.verification_attempts > 0 {
                    stats.verification_cutoffs as f64 / stats.verification_attempts as f64
                } else {
                    0.0
                };

                black_box((
                    stats.verification_attempts,
                    stats.verification_cutoffs,
                    verification_rate,
                ))
            });
        });
    }

    group.finish();
}

/// Benchmark comprehensive NMP performance analysis
fn benchmark_comprehensive_nmp_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_nmp_analysis");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test configurations
    let configurations = vec![
        ("no_verification", 0),
        ("low_margin", 100),
        ("default_margin", 200),
        ("high_margin", 500),
    ];

    for (name, margin) in configurations {
        group.bench_with_input(BenchmarkId::new("config", name), &margin, |b, &margin| {
            b.iter(|| {
                let mut engine = create_test_engine_with_config(NullMoveConfig {
                    enabled: true,
                    min_depth: 3,
                    reduction_factor: 2,
                    max_pieces_threshold: 12,
                    enable_dynamic_reduction: true,
                    enable_endgame_detection: true,
                    verification_margin: margin,
                });

                engine.reset_null_move_stats();

                let start = std::time::Instant::now();
                let mut board_mut = board.clone();
                let result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    5, // Fixed depth for comparison
                    1000,
                );
                let elapsed = start.elapsed();

                let stats = engine.get_null_move_stats();

                // Calculate metrics
                let nodes_searched = engine.get_nodes_searched();
                let nmp_cutoff_rate = stats.cutoff_rate();
                let verification_attempt_rate = if stats.attempts > 0 {
                    stats.verification_attempts as f64 / stats.attempts as f64
                } else {
                    0.0
                };

                black_box((
                    result,
                    elapsed,
                    nodes_searched,
                    nmp_cutoff_rate,
                    verification_attempt_rate,
                ))
            });
        });
    }

    group.finish();
}

/// Validation benchmark to verify verification search doesn't significantly impact NMP effectiveness
/// This ensures cutoff rate reduction is <5% when verification is enabled
fn benchmark_verify_effectiveness_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_effectiveness_impact");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20); // More samples for statistical significance

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Use thread-local storage or collect stats during benchmark execution
    // For now, we'll just compare the benchmark results directly
    // The actual comparison will be done by analyzing the benchmark output

    // Benchmark without verification
    group.bench_function("collect_stats_without_verification", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(NullMoveConfig {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 0,
            });

            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats();
            let cutoff_rate = if stats.attempts > 0 {
                stats.cutoff_rate()
            } else {
                0.0
            };

            black_box(cutoff_rate)
        });
    });

    // Benchmark with verification
    group.bench_function("collect_stats_with_verification", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_config(NullMoveConfig {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 200,
            });

            engine.reset_null_move_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                1000,
            );

            let stats = engine.get_null_move_stats();
            let cutoff_rate = if stats.attempts > 0 {
                stats.cutoff_rate()
            } else {
                0.0
            };

            black_box(cutoff_rate)
        });
    });

    group.finish();

    // Note: The actual effectiveness validation should be done by comparing
    // the benchmark results manually or through a separate analysis script.
    // This benchmark provides the data needed for verification.
    println!("\n=== Verification Search Effectiveness Validation ===");
    println!("Run this benchmark and compare cutoff rates:");
    println!("- 'collect_stats_without_verification' should show baseline cutoff rate");
    println!("- 'collect_stats_with_verification' should show cutoff rate with verification");
    println!("- Expected: verification should not reduce cutoff rate by more than 5%");
    println!("===================================================\n");
}

criterion_group!(
    benches,
    benchmark_nmp_without_verification,
    benchmark_nmp_with_verification,
    benchmark_nmp_effectiveness_comparison,
    benchmark_verification_margin_overhead,
    benchmark_verification_statistics,
    benchmark_comprehensive_nmp_analysis,
    benchmark_verify_effectiveness_impact
);

criterion_main!(benches);
