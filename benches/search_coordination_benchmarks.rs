//! Performance Benchmarks for Search Algorithm Coordination
//!
//! This benchmark suite measures the cumulative effectiveness of algorithm integration
//!
//! Task 7.0.5.15: Measure cumulative search efficiency (target ~80% node reduction)
//! Task 7.0.5.16: Track time overhead distribution (NMP ~5-10%, IID ~10-15%, etc.)
//!
//! Metrics:
//! - Node count reduction with all algorithms vs. baseline
//! - Time overhead by algorithm
//! - Search efficiency gains

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, LMRConfig, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create engine with all algorithms enabled
fn create_full_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64);

    // Enable NMP
    let mut nmp_config = engine.get_null_move_config().clone();
    nmp_config.enabled = true;
    nmp_config.min_depth = 3;
    engine.update_null_move_config(nmp_config).unwrap();

    // Enable IID
    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = true;
    iid_config.min_depth = 4;
    engine.update_iid_config(iid_config).unwrap();

    // Enable LMR
    let mut lmr_config = engine.get_lmr_config().clone();
    lmr_config.enabled = true;
    lmr_config.min_depth = 3;
    engine.update_lmr_config(lmr_config).unwrap();

    engine
}

/// Create baseline engine (minimal algorithms)
fn create_baseline_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64);

    // Disable NMP, IID, LMR for baseline
    let mut nmp_config = engine.get_null_move_config().clone();
    nmp_config.enabled = false;
    engine.update_null_move_config(nmp_config).unwrap();

    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = false;
    engine.update_iid_config(iid_config).unwrap();

    let mut lmr_config = engine.get_lmr_config().clone();
    lmr_config.enabled = false;
    engine.update_lmr_config(lmr_config).unwrap();

    engine
}

/// Benchmark cumulative search efficiency
fn benchmark_cumulative_search_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cumulative_search_efficiency");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Baseline: minimal algorithms
    group.bench_function("baseline_minimal_algorithms", |b| {
        b.iter(|| {
            let mut engine = create_baseline_engine();

            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(5000),
            );

            let nodes = engine.get_nodes_searched();
            black_box((result, nodes))
        })
    });

    // Full: all algorithms enabled
    group.bench_function("full_all_algorithms", |b| {
        b.iter(|| {
            let mut engine = create_full_engine();

            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(5000),
            );

            let nodes = engine.get_nodes_searched();
            let nmp_stats = engine.get_null_move_stats();
            let iid_stats = engine.get_iid_stats();
            let lmr_stats = engine.get_lmr_stats();

            black_box((
                result,
                nodes,
                nmp_stats.cutoffs,
                iid_stats.iid_searches_performed,
                lmr_stats.reductions_applied,
            ))
        })
    });

    group.finish();
}

/// Benchmark time overhead distribution by algorithm
fn benchmark_time_overhead_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_overhead_distribution");
    group.measurement_time(Duration::from_secs(18));
    group.sample_size(12);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 6;

    group.bench_function("overhead_measurement", |b| {
        b.iter(|| {
            let mut engine = create_full_engine();

            let start = std::time::Instant::now();
            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(8000),
            );
            let total_time = start.elapsed();

            // Get statistics
            let nmp_stats = engine.get_null_move_stats();
            let iid_stats = engine.get_iid_stats();
            let lmr_stats = engine.get_lmr_stats();

            // Calculate overhead percentages (approximate)
            let iid_time_ms = iid_stats.iid_time_ms;
            let total_time_ms = total_time.as_millis() as u64;

            let iid_overhead = if total_time_ms > 0 {
                (iid_time_ms as f64 / total_time_ms as f64) * 100.0
            } else {
                0.0
            };

            black_box((
                result,
                iid_overhead,
                nmp_stats.attempts,
                lmr_stats.reductions_applied,
            ))
        })
    });

    group.finish();
}

/// Benchmark coordination effectiveness
fn benchmark_coordination_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination_effectiveness");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test at different depths
    for depth in [5, 6] {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("coordination_at_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_full_engine();
                    engine.reset_lmr_stats();
                    engine.reset_iid_stats();
                    engine.reset_core_search_metrics();

                    let result = engine.search_at_depth(
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                        black_box(5000),
                    );

                    // Measure coordination effectiveness
                    let lmr_stats = engine.get_lmr_stats();
                    let iid_stats = engine.get_iid_stats();
                    let metrics = engine.get_core_search_metrics();

                    // Coordination quality indicators
                    let iid_exemption_rate = if lmr_stats.moves_considered > 0 {
                        (lmr_stats.iid_move_explicitly_exempted as f64
                            / lmr_stats.moves_considered as f64)
                            * 100.0
                    } else {
                        0.0
                    };

                    let tt_protection_rate = if metrics.total_tt_probes > 0 {
                        (metrics.tt_auxiliary_overwrites_prevented as f64
                            / metrics.total_tt_probes as f64)
                            * 100.0
                    } else {
                        0.0
                    };

                    black_box((
                        result,
                        iid_exemption_rate,
                        tt_protection_rate,
                        lmr_stats.iid_move_reduced_count, // Should always be 0
                        metrics.evaluation_cache_hits,
                    ))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cumulative_search_efficiency,
    benchmark_time_overhead_distribution,
    benchmark_coordination_effectiveness
);
criterion_main!(benches);
