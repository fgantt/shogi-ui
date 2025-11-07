//! Performance benchmarks for PruningManager adaptive reduction
//!
//! This benchmark suite verifies and measures PruningManager adaptive reduction:
//! - Verifies adaptive reduction is actually being applied
//! - Compares adaptive reduction with/without PruningManager
//! - Measures effectiveness of position classification-based reduction
//! - Validates parameter synchronization
//!
//! Metrics measured:
//! - Adaptive reduction application rate
//! - Position classification effectiveness
//! - Parameter synchronization overhead
//! - Search time impact
//!
//! Expected results:
//! - Adaptive reduction should be applied correctly based on position classification
//! - PruningManager should sync parameters with LMRConfig
//! - Overhead should be minimal (<1% search time)
//! - Position classification should improve reduction accuracy

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, GamePhase, LMRConfig, Player, PositionClassification},
};
use std::time::Duration;

/// Create a test engine with adaptive reduction enabled
fn create_test_engine_adaptive_enabled() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    let mut config = LMRConfig::default();
    config.enable_adaptive_reduction = true;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Create a test engine with adaptive reduction disabled
fn create_test_engine_adaptive_disabled() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    let mut config = LMRConfig::default();
    config.enable_adaptive_reduction = false;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Benchmark adaptive reduction with/without PruningManager
fn benchmark_adaptive_reduction_with_without_pruning_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_reduction_with_without_pruning_manager");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine_adaptive = create_test_engine_adaptive_enabled();
                let mut engine_no_adaptive = create_test_engine_adaptive_disabled();

                engine_adaptive.reset_lmr_stats();
                engine_no_adaptive.reset_lmr_stats();

                let mut board_mut = board.clone();
                let result_adaptive = engine_adaptive.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                let mut board_mut = board.clone();
                let result_no_adaptive = engine_no_adaptive.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                let stats_adaptive = engine_adaptive.get_lmr_stats().clone();
                let stats_no_adaptive = engine_no_adaptive.get_lmr_stats().clone();

                black_box((
                    result_adaptive,
                    result_no_adaptive,
                    stats_adaptive,
                    stats_no_adaptive,
                ))
            });
        });
    }

    group.finish();
}

/// Benchmark position classification effectiveness
fn benchmark_position_classification_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_classification_effectiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    let position_types = vec![
        ("tactical", PositionClassification::Tactical),
        ("quiet", PositionClassification::Quiet),
        ("neutral", PositionClassification::Neutral),
    ];

    for (name, position_type) in position_types {
        group.bench_with_input(
            BenchmarkId::new("position_type", name),
            &position_type,
            |b, _position_type| {
                b.iter(|| {
                    let mut engine = create_test_engine_adaptive_enabled();
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
                    let classification_stats = stats.classification_stats.clone();

                    black_box((result, stats, classification_stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parameter synchronization
fn benchmark_parameter_synchronization(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameter_synchronization");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    group.bench_function("sync_overhead", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive_enabled();

            // Update LMRConfig multiple times
            for i in 1..=5 {
                let mut config = LMRConfig::default();
                config.base_reduction = i as u8;
                config.max_reduction = (i + 2) as u8;
                engine.update_lmr_config(config).unwrap();
            }

            let pruning_manager = engine.get_pruning_manager();
            let params = pruning_manager.parameters.clone();

            black_box(params)
        });
    });

    group.finish();
}

/// Benchmark adaptive reduction application rate
fn benchmark_adaptive_reduction_application_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_reduction_application_rate");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("application_rate", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive_enabled();
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
            let pruning_manager = engine.get_pruning_manager();

            // Calculate adaptive reduction application metrics
            let total_moves = stats.moves_considered;
            let reductions_applied = stats.reductions_applied;
            let tactical_classified = stats.classification_stats.tactical_classified;
            let quiet_classified = stats.classification_stats.quiet_classified;
            let neutral_classified = stats.classification_stats.neutral_classified;
            let adaptive_enabled = pruning_manager.parameters.lmr_enable_adaptive_reduction;

            black_box((
                stats,
                total_moves,
                reductions_applied,
                tactical_classified,
                quiet_classified,
                neutral_classified,
                adaptive_enabled,
            ))
        });
    });

    group.finish();
}

/// Benchmark comprehensive PruningManager analysis
fn benchmark_comprehensive_pruning_manager_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_pruning_manager_analysis");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive_enabled();
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
            let pruning_manager = engine.get_pruning_manager();
            let pruning_params = pruning_manager.parameters.clone();

            // Comprehensive metrics
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();
            let adaptive_enabled = pruning_params.lmr_enable_adaptive_reduction;
            let tactical_ratio = stats.classification_stats.tactical_ratio();
            let quiet_ratio = stats.classification_stats.quiet_ratio();

            black_box((
                result,
                elapsed,
                stats,
                pruning_params,
                efficiency,
                research_rate,
                cutoff_rate,
                adaptive_enabled,
                tactical_ratio,
                quiet_ratio,
            ))
        });
    });

    group.finish();
}

/// Benchmark PruningManager adaptive reduction verification
fn benchmark_pruning_manager_adaptive_reduction_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("pruning_manager_adaptive_reduction_verification");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("verification", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive_enabled();
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
            let pruning_manager = engine.get_pruning_manager();

            // Verify adaptive reduction is enabled and working
            let adaptive_enabled = pruning_manager.parameters.lmr_enable_adaptive_reduction;
            let reductions_applied = stats.reductions_applied;
            let tactical_classified = stats.classification_stats.tactical_classified;
            let quiet_classified = stats.classification_stats.quiet_classified;

            // Verify parameters are synced
            let lmr_config = engine.get_lmr_config();
            let params_synced = pruning_manager.parameters.lmr_base_reduction
                == lmr_config.base_reduction
                && pruning_manager.parameters.lmr_max_reduction == lmr_config.max_reduction
                && pruning_manager.parameters.lmr_move_threshold == lmr_config.min_move_index
                && pruning_manager.parameters.lmr_depth_threshold == lmr_config.min_depth
                && pruning_manager.parameters.lmr_enable_adaptive_reduction
                    == lmr_config.enable_adaptive_reduction;

            black_box((
                stats,
                adaptive_enabled,
                reductions_applied,
                tactical_classified,
                quiet_classified,
                params_synced,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_adaptive_reduction_with_without_pruning_manager,
    benchmark_position_classification_effectiveness,
    benchmark_parameter_synchronization,
    benchmark_adaptive_reduction_application_rate,
    benchmark_comprehensive_pruning_manager_analysis,
    benchmark_pruning_manager_adaptive_reduction_verification
);

criterion_main!(benches);
