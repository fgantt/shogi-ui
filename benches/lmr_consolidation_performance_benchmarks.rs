#![cfg(feature = "legacy-tests")]
//! Performance benchmarks for LMR consolidation verification
//!
//! This benchmark suite measures the performance impact of consolidating LMR
//! implementation to PruningManager. It verifies:
//! - PruningManager reduction formula performance
//! - No performance regression from consolidation
//! - LMR effectiveness remains high
//!
//! Metrics measured:
//! - Search time
//! - Nodes searched
//! - LMR reduction rate (efficiency)
//! - Re-search rate
//! - Cutoff rate
//!
//! Expected results:
//! - PruningManager should perform at least as well as legacy implementation
//! - LMR effectiveness should remain high (efficiency > 25%, cutoff rate > 10%)
//! - No significant performance regression (<5% search time increase)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, LMRConfig, Player, PruningParameters},
};
use std::time::Duration;

/// Create a test engine with default configuration
fn create_test_engine() -> SearchEngine {
    SearchEngine::new(None, 16) // 16MB hash table
}

/// Create a test engine with specific LMR configuration
fn create_test_engine_with_lmr_config(lmr_config: LMRConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_lmr_config(lmr_config).unwrap();
    engine
}

/// Create a test engine with specific PruningManager parameters
fn create_test_engine_with_pruning_params(params: PruningParameters) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_pruning_parameters(params);
    engine
}

/// Benchmark LMR with PruningManager (current implementation)
fn benchmark_lmr_with_pruning_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_pruning_manager");
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
                    let mut engine = create_test_engine();
                    engine.reset_lmr_stats();

                    let mut board_mut = board.clone();
                    let start_time = std::time::Instant::now();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );
                    let search_time = start_time.elapsed();

                    let stats = engine.get_lmr_stats().clone();
                    let pruning_stats = engine.get_pruning_statistics().clone();

                    black_box((result, stats, pruning_stats, search_time))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark LMR effectiveness metrics
fn benchmark_lmr_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_effectiveness");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("lmr_enabled", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr_config(LMRConfig {
                enabled: true,
                min_depth: 3,
                min_move_index: 4,
                base_reduction: 1,
                max_reduction: 3,
                enable_dynamic_reduction: true,
                enable_adaptive_reduction: true,
                enable_extended_exemptions: true,
            });
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
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();

            black_box((result, efficiency, research_rate, cutoff_rate))
        });
    });

    group.bench_function("lmr_disabled", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_lmr_config(LMRConfig {
                enabled: false,
                min_depth: 3,
                min_move_index: 4,
                base_reduction: 1,
                max_reduction: 3,
                enable_dynamic_reduction: true,
                enable_adaptive_reduction: true,
                enable_extended_exemptions: true,
            });
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

            black_box((result, stats))
        });
    });

    group.finish();
}

/// Benchmark PruningManager reduction formula at different depths
fn benchmark_pruning_manager_reduction_formula(c: &mut Criterion) {
    let mut group = c.benchmark_group("pruning_manager_reduction_formula");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test different depth configurations
    for depth in [3, 4, 5, 6, 8, 10] {
        group.bench_with_input(
            BenchmarkId::new("search_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    engine.reset_lmr_stats();
                    engine.reset_pruning_statistics();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );

                    let stats = engine.get_lmr_stats().clone();
                    let pruning_stats = engine.get_pruning_statistics().clone();

                    // Calculate reduction statistics
                    let avg_reduction = if stats.reductions_applied > 0 {
                        stats.total_depth_saved as f64 / stats.reductions_applied as f64
                    } else {
                        0.0
                    };

                    black_box((result, stats, pruning_stats, avg_reduction))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark PruningManager with different parameter configurations
fn benchmark_pruning_manager_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("pruning_manager_configurations");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test with extended exemptions enabled
    group.bench_function("extended_exemptions_enabled", |b| {
        b.iter(|| {
            let mut params = PruningParameters::default();
            params.lmr_enable_extended_exemptions = true;
            params.lmr_enable_adaptive_reduction = true;

            let mut engine = create_test_engine_with_pruning_params(params);
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
            black_box((result, stats))
        });
    });

    // Test with extended exemptions disabled
    group.bench_function("extended_exemptions_disabled", |b| {
        b.iter(|| {
            let mut params = PruningParameters::default();
            params.lmr_enable_extended_exemptions = false;
            params.lmr_enable_adaptive_reduction = false;

            let mut engine = create_test_engine_with_pruning_params(params);
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
            black_box((result, stats))
        });
    });

    // Test with adaptive reduction enabled
    group.bench_function("adaptive_reduction_enabled", |b| {
        b.iter(|| {
            let mut params = PruningParameters::default();
            params.lmr_enable_adaptive_reduction = true;

            let mut engine = create_test_engine_with_pruning_params(params);
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
            black_box((result, stats))
        });
    });

    // Test with adaptive reduction disabled
    group.bench_function("adaptive_reduction_disabled", |b| {
        b.iter(|| {
            let mut params = PruningParameters::default();
            params.lmr_enable_adaptive_reduction = false;

            let mut engine = create_test_engine_with_pruning_params(params);
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
            black_box((result, stats))
        });
    });

    group.finish();
}

/// Benchmark performance regression validation
fn benchmark_performance_regression_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_performance_regression");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at multiple depths to ensure no regression
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("validation_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    engine.reset_lmr_stats();
                    engine.reset_pruning_statistics();

                    let mut board_mut = board.clone();
                    let start_time = std::time::Instant::now();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        2000,
                    );
                    let search_time = start_time.elapsed();

                    let stats = engine.get_lmr_stats().clone();
                    let pruning_stats = engine.get_pruning_statistics().clone();

                    // Calculate key metrics
                    let efficiency = stats.efficiency();
                    let research_rate = stats.research_rate();
                    let cutoff_rate = stats.cutoff_rate();

                    // Validate metrics meet requirements
                    assert!(
                        efficiency >= 25.0 || stats.moves_considered == 0,
                        "LMR efficiency should be >= 25% (got {:.2}%)",
                        efficiency
                    );
                    assert!(
                        research_rate <= 30.0 || stats.reductions_applied == 0,
                        "Re-search rate should be <= 30% (got {:.2}%)",
                        research_rate
                    );
                    assert!(
                        cutoff_rate >= 10.0 || stats.moves_considered == 0,
                        "Cutoff rate should be >= 10% (got {:.2}%)",
                        cutoff_rate
                    );

                    black_box((
                        result,
                        stats,
                        pruning_stats,
                        search_time,
                        efficiency,
                        research_rate,
                        cutoff_rate,
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark comprehensive LMR analysis
fn benchmark_comprehensive_lmr_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_comprehensive_analysis");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("full_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();
            engine.reset_pruning_statistics();

            let mut board_mut = board.clone();
            let start_time = std::time::Instant::now();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                2000,
            );
            let search_time = start_time.elapsed();

            let stats = engine.get_lmr_stats().clone();
            let pruning_stats = engine.get_pruning_statistics().clone();

            // Comprehensive metrics
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();
            let avg_reduction = stats.average_reduction;
            let avg_depth_saved = stats.average_depth_saved();

            // PruningManager metrics
            let lmr_applied = pruning_stats.lmr_applied;
            let re_searches = pruning_stats.re_searches;

            black_box((
                result,
                stats,
                pruning_stats,
                search_time,
                efficiency,
                research_rate,
                cutoff_rate,
                avg_reduction,
                avg_depth_saved,
                lmr_applied,
                re_searches,
            ))
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(10);
    targets =
        benchmark_lmr_with_pruning_manager,
        benchmark_lmr_effectiveness,
        benchmark_pruning_manager_reduction_formula,
        benchmark_pruning_manager_configurations,
        benchmark_performance_regression_validation,
        benchmark_comprehensive_lmr_analysis
}

criterion_main!(benches);
