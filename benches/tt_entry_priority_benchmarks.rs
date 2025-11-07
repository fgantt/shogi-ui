//! Performance Benchmarks for TT Entry Priority System
//!
//! This benchmark suite measures the effectiveness of TT entry priority management
//!
//! Task 7.0.3.15: Compare TT pollution before and after priority system
//!
//! Metrics:
//! - TT hit rate with priority system
//! - Auxiliary overwrites prevented
//! - Main entries preserved
//! - Search performance impact

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with TT priority system enabled
fn create_test_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64); // 64MB hash table

    // Enable NMP to create auxiliary entries
    let mut nmp_config = engine.get_null_move_config().clone();
    nmp_config.enabled = true;
    nmp_config.min_depth = 3;
    engine.update_null_move_config(nmp_config).unwrap();

    // Enable IID to create auxiliary entries
    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = true;
    iid_config.min_depth = 4;
    engine.update_iid_config(iid_config).unwrap();

    engine
}

/// Benchmark TT hit rate with priority system
fn benchmark_tt_hit_rate_with_priority(c: &mut Criterion) {
    let mut group = c.benchmark_group("tt_priority_hit_rate");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Benchmark at different depths
    for depth in [5, 6] {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("search_with_tt_priority", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    let result = engine.search_at_depth(
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                        black_box(5000),
                    );

                    // Get TT statistics
                    let metrics = engine.get_core_search_metrics();
                    let hit_rate = if metrics.total_tt_probes > 0 {
                        (metrics.total_tt_hits as f64 / metrics.total_tt_probes as f64) * 100.0
                    } else {
                        0.0
                    };

                    black_box((result, hit_rate, metrics.tt_auxiliary_overwrites_prevented))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark auxiliary overwrite prevention effectiveness
fn benchmark_overwrite_prevention(c: &mut Criterion) {
    let mut group = c.benchmark_group("tt_overwrite_prevention");
    group.measurement_time(Duration::from_secs(12));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 6;

    group.bench_function("prevention_effectiveness", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();

            let _result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(5000),
            );

            // Measure prevention effectiveness
            let metrics = engine.get_core_search_metrics();

            let prevention_rate = if metrics.total_tt_probes > 0 {
                (metrics.tt_auxiliary_overwrites_prevented as f64 / metrics.total_tt_probes as f64)
                    * 100.0
            } else {
                0.0
            };

            black_box((
                metrics.tt_auxiliary_overwrites_prevented,
                metrics.tt_main_entries_preserved,
                prevention_rate,
            ))
        })
    });

    group.finish();
}

/// Benchmark TT pollution comparison across multiple searches
fn benchmark_tt_pollution_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("tt_pollution_comparison");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("multiple_searches_with_priority", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();

            // Perform multiple searches to stress-test TT priority system
            for _ in 0..3 {
                let _result = engine.search_at_depth(
                    black_box(&board),
                    black_box(&captured_pieces),
                    black_box(player),
                    black_box(depth),
                    black_box(2000),
                );
            }

            // Measure cumulative TT quality
            let metrics = engine.get_core_search_metrics();

            let hit_rate = if metrics.total_tt_probes > 0 {
                (metrics.total_tt_hits as f64 / metrics.total_tt_probes as f64) * 100.0
            } else {
                0.0
            };

            let exact_hit_rate = if metrics.total_tt_hits > 0 {
                (metrics.tt_exact_hits as f64 / metrics.total_tt_hits as f64) * 100.0
            } else {
                0.0
            };

            black_box((
                hit_rate,
                exact_hit_rate,
                metrics.tt_auxiliary_overwrites_prevented,
                metrics.tt_main_entries_preserved,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tt_hit_rate_with_priority,
    benchmark_overwrite_prevention,
    benchmark_tt_pollution_comparison
);
criterion_main!(benches);
