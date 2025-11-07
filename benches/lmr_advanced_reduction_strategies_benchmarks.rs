//! Performance benchmarks for advanced reduction strategies
//!
//! This benchmark suite measures the effectiveness and overhead of advanced reduction strategies:
//! - Depth-based reduction scaling (non-linear formulas)
//! - Material-based reduction adjustment
//! - History-based reduction
//! - Combined strategies
//!
//! Metrics measured:
//! - Search time with/without advanced strategies
//! - LMR effectiveness with different strategies
//! - Overhead of advanced strategies
//! - Comparison with basic reduction
//!
//! Expected results:
//! - Advanced strategies may show diminishing returns
//! - Depth-based scaling should be more effective at deeper depths
//! - Material-based adjustment should be more effective in tactical positions
//! - History-based reduction should be more effective for quiet moves
//! - Combined strategies may have diminishing returns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{AdvancedReductionConfig, AdvancedReductionStrategy, CapturedPieces, Player},
};
use std::time::Duration;

/// Create a test engine
fn create_test_engine() -> SearchEngine {
    SearchEngine::new(None, 16)
}

/// Benchmark basic vs advanced reduction strategies
fn benchmark_basic_vs_advanced_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_vs_advanced_reduction");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test basic reduction
    group.bench_function("basic_reduction", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
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
            black_box((result, stats))
        });
    });

    // Test depth-based reduction
    group.bench_function("depth_based_reduction", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable depth-based reduction
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.enable_depth_based = true;
            config.advanced_reduction_config.depth_scaling_factor = 0.15;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            black_box((result, stats))
        });
    });

    // Test material-based reduction
    group.bench_function("material_based_reduction", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable material-based reduction
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.enable_material_based = true;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            black_box((result, stats))
        });
    });

    // Test history-based reduction
    group.bench_function("history_based_reduction", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable history-based reduction
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.enable_history_based = true;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            black_box((result, stats))
        });
    });

    // Test combined strategies
    group.bench_function("combined_strategies", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable all strategies
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.strategy = AdvancedReductionStrategy::Combined;
            config.advanced_reduction_config.enable_depth_based = true;
            config.advanced_reduction_config.enable_material_based = true;
            config.advanced_reduction_config.enable_history_based = true;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            black_box((result, stats))
        });
    });

    group.finish();
}

/// Benchmark depth-based reduction at different depths
fn benchmark_depth_based_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("depth_based_scaling");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths
    for depth in [3, 5, 7, 9] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine();
                engine.reset_lmr_stats();

                // Enable depth-based reduction
                let mut config = engine.get_lmr_config().clone();
                config.advanced_reduction_config.enabled = true;
                config.advanced_reduction_config.enable_depth_based = true;
                config.advanced_reduction_config.depth_scaling_factor = 0.15;
                engine.update_lmr_config(config).unwrap();

                let mut board_mut = board.clone();
                let result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                let stats = engine.get_lmr_stats().clone();
                black_box((result, stats))
            });
        });
    }

    group.finish();
}

/// Benchmark material-based reduction effectiveness
fn benchmark_material_based_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_based_reduction");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("material_based_effectiveness", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable material-based reduction
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.enable_material_based = true;
            config
                .advanced_reduction_config
                .material_imbalance_threshold = 300;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            let metrics = engine.get_lmr_performance_metrics();
            black_box((result, stats, metrics))
        });
    });

    group.finish();
}

/// Benchmark history-based reduction effectiveness
fn benchmark_history_based_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("history_based_reduction");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("history_based_effectiveness", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            // Enable history-based reduction
            let mut config = engine.get_lmr_config().clone();
            config.advanced_reduction_config.enabled = true;
            config.advanced_reduction_config.enable_history_based = true;
            config.advanced_reduction_config.history_score_threshold = 0;
            engine.update_lmr_config(config).unwrap();

            let mut board_mut = board.clone();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats = engine.get_lmr_stats().clone();
            let metrics = engine.get_lmr_performance_metrics();
            black_box((result, stats, metrics))
        });
    });

    group.finish();
}

/// Benchmark comprehensive analysis of advanced strategies
fn benchmark_comprehensive_advanced_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_advanced_strategies");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            let start = std::time::Instant::now();

            // Test with basic reduction
            let mut config_basic = engine.get_lmr_config().clone();
            config_basic.advanced_reduction_config.enabled = false;
            engine.update_lmr_config(config_basic.clone()).unwrap();

            let mut board_mut = board.clone();
            let result_basic = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats_basic = engine.get_lmr_stats().clone();
            let metrics_basic = engine.get_lmr_performance_metrics();

            engine.reset_lmr_stats();

            // Test with combined strategies
            let mut config_advanced = engine.get_lmr_config().clone();
            config_advanced.advanced_reduction_config.enabled = true;
            config_advanced.advanced_reduction_config.strategy =
                AdvancedReductionStrategy::Combined;
            config_advanced.advanced_reduction_config.enable_depth_based = true;
            config_advanced
                .advanced_reduction_config
                .enable_material_based = true;
            config_advanced
                .advanced_reduction_config
                .enable_history_based = true;
            engine.update_lmr_config(config_advanced).unwrap();

            let mut board_mut2 = board.clone();
            let result_advanced = engine.search_at_depth_legacy(
                black_box(&mut board_mut2),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let stats_advanced = engine.get_lmr_stats().clone();
            let metrics_advanced = engine.get_lmr_performance_metrics();

            let elapsed = start.elapsed();

            // Compare effectiveness
            let efficiency_improvement = metrics_advanced.efficiency - metrics_basic.efficiency;
            let research_rate_diff = metrics_advanced.research_rate - metrics_basic.research_rate;
            let cutoff_rate_diff = metrics_advanced.cutoff_rate - metrics_basic.cutoff_rate;

            black_box((
                result_basic,
                result_advanced,
                elapsed,
                stats_basic,
                stats_advanced,
                metrics_basic,
                metrics_advanced,
                efficiency_improvement,
                research_rate_diff,
                cutoff_rate_diff,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_basic_vs_advanced_reduction,
    benchmark_depth_based_scaling,
    benchmark_material_based_reduction,
    benchmark_history_based_reduction,
    benchmark_comprehensive_advanced_strategies
);

criterion_main!(benches);
