//! Performance benchmarks for move ordering effectiveness tracking
//!
//! This benchmark suite measures the correlation between move ordering quality and LMR effectiveness:
//! - Correlation between ordering quality and LMR re-search rate
//! - Average move index of cutoff-causing moves
//! - Percentage of cutoffs from moves after LMR threshold
//! - Move ordering effectiveness metrics
//!
//! Metrics measured:
//! - Ordering effectiveness vs LMR effectiveness
//! - Late cutoff rate correlation with re-search rate
//! - Average cutoff index correlation with efficiency
//! - Integration with move ordering statistics
//!
//! Expected results:
//! - Good move ordering should correlate with low late cutoff rate
//! - Poor move ordering should correlate with high LMR re-search rate
//! - Average cutoff index should be < 5.0 for optimal LMR performance
//! - LMR effectiveness depends on good move ordering

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, Player},
};
use std::time::Duration;

/// Create a test engine
fn create_test_engine() -> SearchEngine {
    SearchEngine::new(None, 16)
}

/// Benchmark move ordering effectiveness tracking
fn benchmark_move_ordering_effectiveness_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering_effectiveness_tracking");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine = create_test_engine();
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
                let ordering_metrics = engine.get_move_ordering_effectiveness_metrics();

                black_box((result, stats, ordering_metrics))
            });
        });
    }

    group.finish();
}

/// Benchmark correlation between ordering quality and LMR re-search rate (Task 10.10)
fn benchmark_ordering_correlation_with_research_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordering_correlation_with_research_rate");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("correlation_measurement", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
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
            let ordering_metrics = engine.get_move_ordering_effectiveness_metrics();
            let lmr_metrics = engine.get_lmr_performance_metrics();

            // Calculate correlation metrics
            let late_cutoff_rate = ordering_metrics.cutoffs_after_threshold_percentage;
            let research_rate = lmr_metrics.research_rate;
            let avg_cutoff_index = ordering_metrics.average_cutoff_index;
            let efficiency = lmr_metrics.efficiency;

            // Correlation: high late cutoff rate should correlate with high re-search rate
            let correlation = if late_cutoff_rate > 0.0 && research_rate > 0.0 {
                (late_cutoff_rate + research_rate) / 2.0
            } else {
                0.0
            };

            black_box((
                stats,
                ordering_metrics,
                lmr_metrics,
                late_cutoff_rate,
                research_rate,
                avg_cutoff_index,
                efficiency,
                correlation,
            ))
        });
    });

    group.finish();
}

/// Benchmark average cutoff index calculation
fn benchmark_average_cutoff_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("average_cutoff_index");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("average_cutoff_index_calculation", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let ordering_metrics = engine.get_move_ordering_effectiveness_metrics();
            let avg_cutoff_index = ordering_metrics.average_cutoff_index;
            let total_cutoffs = ordering_metrics.total_cutoffs;

            black_box((ordering_metrics, avg_cutoff_index, total_cutoffs))
        });
    });

    group.finish();
}

/// Benchmark cutoffs after threshold percentage
fn benchmark_cutoffs_after_threshold_percentage(c: &mut Criterion) {
    let mut group = c.benchmark_group("cutoffs_after_threshold_percentage");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("percentage_calculation", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let ordering_metrics = engine.get_move_ordering_effectiveness_metrics();
            let percentage = ordering_metrics.cutoffs_after_threshold_percentage;
            let late_cutoffs = ordering_metrics.late_ordered_cutoffs;
            let total_cutoffs = ordering_metrics.total_cutoffs;

            black_box((ordering_metrics, percentage, late_cutoffs, total_cutoffs))
        });
    });

    group.finish();
}

/// Benchmark ordering effectiveness integration
fn benchmark_ordering_effectiveness_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordering_effectiveness_integration");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("integration_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let integration_report = engine.get_ordering_effectiveness_with_integration();
            let ordering_vs_lmr_report = engine.get_ordering_vs_lmr_report();
            let improvements = engine.identify_ordering_improvements();

            black_box((integration_report, ordering_vs_lmr_report, improvements))
        });
    });

    group.finish();
}

/// Benchmark comprehensive ordering effectiveness analysis
fn benchmark_comprehensive_ordering_effectiveness_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_ordering_effectiveness_analysis");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
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
            let ordering_metrics = engine.get_move_ordering_effectiveness_metrics();
            let lmr_metrics = engine.get_lmr_performance_metrics();
            let ordering_stats = engine.advanced_move_orderer.get_stats();

            // Comprehensive metrics
            let ordering_effectiveness = ordering_metrics.ordering_effectiveness;
            let late_cutoff_rate = ordering_metrics.cutoffs_after_threshold_percentage;
            let avg_cutoff_index = ordering_metrics.average_cutoff_index;
            let efficiency = lmr_metrics.efficiency;
            let research_rate = lmr_metrics.research_rate;
            let cutoff_rate = lmr_metrics.cutoff_rate;

            // Integration metrics
            let pv_hit_rate = ordering_stats.pv_move_hit_rate;
            let killer_hit_rate = ordering_stats.killer_move_hit_rate;
            let cache_hit_rate = ordering_stats.cache_hit_rate;

            // Degradation check
            let (is_healthy, alerts) = engine.check_move_ordering_degradation();

            // Improvements identification
            let improvements = engine.identify_ordering_improvements();

            black_box((
                result,
                elapsed,
                stats,
                ordering_metrics,
                lmr_metrics,
                ordering_stats,
                ordering_effectiveness,
                late_cutoff_rate,
                avg_cutoff_index,
                efficiency,
                research_rate,
                cutoff_rate,
                pv_hit_rate,
                killer_hit_rate,
                cache_hit_rate,
                is_healthy,
                alerts,
                improvements,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_move_ordering_effectiveness_tracking,
    benchmark_ordering_correlation_with_research_rate,
    benchmark_average_cutoff_index,
    benchmark_cutoffs_after_threshold_percentage,
    benchmark_ordering_effectiveness_integration,
    benchmark_comprehensive_ordering_effectiveness_analysis
);

criterion_main!(benches);
