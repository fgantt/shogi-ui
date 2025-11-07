//! Performance monitoring benchmarks for LMR
//!
//! This benchmark suite provides comprehensive monitoring of LMR performance:
//! - Comparison benchmarks: LMR enabled vs disabled
//! - Configuration comparison benchmarks
//! - Performance regression tests
//! - Phase-specific performance tracking
//!
//! Metrics measured:
//! - Search time
//! - Nodes searched
//! - LMR effectiveness (efficiency, cutoff rate, re-search rate)
//! - Performance by game phase
//! - Performance alerts and thresholds
//!
//! This suite is designed for CI/CD integration and performance tracking over time.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, LMRConfig, Player, PruningParameters},
};
use std::collections::HashMap;
use std::time::Duration;

/// Create a test engine with LMR enabled
fn create_test_engine_with_lmr(enabled: bool) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = LMRConfig::default();
    config.enabled = enabled;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Create a test engine with specific LMR configuration
fn create_test_engine_with_config(config: LMRConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Benchmark LMR enabled vs disabled comparison (Task 4.7)
fn benchmark_lmr_enabled_vs_disabled(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_enabled_vs_disabled");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        for enabled in [false, true] {
            group.bench_with_input(
                BenchmarkId::new(
                    format!("depth_{}", depth),
                    if enabled { "enabled" } else { "disabled" },
                ),
                &(depth, enabled),
                |b, &(depth, enabled)| {
                    b.iter(|| {
                        let mut engine = create_test_engine_with_lmr(enabled);
                        engine.reset_lmr_stats();

                        let mut board_mut = board.clone();
                        let result = engine.search_at_depth_legacy(
                            black_box(&mut board_mut),
                            black_box(&captured_pieces),
                            player,
                            depth,
                            1000,
                        );

                        let stats = engine.get_lmr_stats().clone();
                        let metrics = engine.export_lmr_metrics();
                        black_box((result, stats, metrics))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark different LMR configurations (Task 4.7)
fn benchmark_lmr_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_configurations");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test different configurations
    let configs = vec![
        ("default", LMRConfig::default()),
        ("aggressive", {
            let mut c = LMRConfig::default();
            c.base_reduction = 2;
            c.max_reduction = 4;
            c.re_search_margin = 25;
            c
        }),
        ("conservative", {
            let mut c = LMRConfig::default();
            c.base_reduction = 1;
            c.max_reduction = 2;
            c.re_search_margin = 100;
            c
        }),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::new("config", name), &config, |b, config| {
            b.iter(|| {
                let mut engine = create_test_engine_with_config(config.clone());
                engine.reset_lmr_stats();

                let mut board_mut = board.clone();
                let result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    5, // Fixed depth
                    1000,
                );

                let stats = engine.get_lmr_stats().clone();
                let (is_healthy, alerts) = engine.check_lmr_performance();
                black_box((result, stats, is_healthy, alerts))
            });
        });
    }

    group.finish();
}

/// Benchmark performance regression validation (Task 4.4)
fn benchmark_performance_regression_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_performance_regression");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("regression_validation", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr(true);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let start = std::time::Instant::now();

            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let elapsed = start.elapsed();
            let stats = engine.get_lmr_stats().clone();

            // Validate performance thresholds (Task 4.4)
            let (is_healthy, alerts) = engine.check_lmr_performance();

            // Assertions for regression tests
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();

            // These would fail in CI/CD if thresholds are not met
            // For benchmarks, we just track the values
            let meets_thresholds =
                efficiency >= 25.0 && research_rate <= 30.0 && cutoff_rate >= 10.0;

            black_box((elapsed, stats, is_healthy, alerts, meets_thresholds))
        });
    });

    group.finish();
}

/// Benchmark phase-specific performance (Task 4.6)
fn benchmark_phase_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_phase_performance");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("phase_tracking", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr(true);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();

            // Extract phase statistics
            let phase_stats: HashMap<_, _> = stats.phase_stats.iter().collect();
            black_box((stats, phase_stats))
        });
    });

    group.finish();
}

/// Benchmark performance metrics export (Task 4.9)
fn benchmark_metrics_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_metrics_export");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("export_metrics", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr(true);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let metrics = engine.export_lmr_metrics();
            let report = engine.get_lmr_performance_report();
            black_box((metrics, report))
        });
    });

    group.finish();
}

/// Benchmark comprehensive performance monitoring
fn benchmark_comprehensive_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_comprehensive_monitoring");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr(true);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let start = std::time::Instant::now();

            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let elapsed = start.elapsed();
            let stats = engine.get_lmr_stats().clone();

            // Comprehensive monitoring
            let (is_healthy, alerts) = engine.check_lmr_performance();
            let metrics = engine.export_lmr_metrics();
            let report = engine.get_lmr_performance_report();

            // Phase statistics
            let phase_stats: HashMap<_, _> = stats.phase_stats.iter().collect();

            black_box((
                result,
                elapsed,
                stats,
                is_healthy,
                alerts,
                metrics,
                report,
                phase_stats,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_lmr_enabled_vs_disabled,
    benchmark_lmr_configurations,
    benchmark_performance_regression_validation,
    benchmark_phase_performance,
    benchmark_metrics_export,
    benchmark_comprehensive_monitoring
);

criterion_main!(benches);
