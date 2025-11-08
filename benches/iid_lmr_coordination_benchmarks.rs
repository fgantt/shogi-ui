#![cfg(feature = "legacy-tests")]
//! Benchmarks for IID/LMR coordination strategies
//!
//! This suite measures the impact of coordinated IID and LMR heuristics on
//! search performance.
//!
//! Task 7.0.1.10: Measure impact of explicit IID exemption on search performance
//!
//! Metrics:
//! - Search time with/without explicit IID exemption
//! - Node count comparison
//! - IID move exemption statistics

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, LMRConfig, Player},
};
use std::time::Duration;

/// Create a test engine with IID and LMR enabled
fn create_test_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64); // 64MB hash table

    // Enable IID
    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = true;
    iid_config.min_depth = 4;
    engine.update_iid_config(iid_config).unwrap();

    // Enable LMR
    let mut lmr_config = engine.get_lmr_config().clone();
    lmr_config.enabled = true;
    lmr_config.min_depth = 3;
    lmr_config.min_move_index = 2;
    engine.update_lmr_config(lmr_config).unwrap();

    engine
}

/// Benchmark search with IID-LMR coordination
fn benchmark_iid_lmr_coordination(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_lmr_coordination");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Benchmark at different depths
    for depth in [5, 6, 7] {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("with_iid_and_lmr", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    let result = engine.search_at_depth(
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                        black_box(10000),
                    );
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark IID move exemption statistics tracking overhead
fn benchmark_exemption_statistics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_exemption_statistics");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("exemption_tracking", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_lmr_stats();
            engine.reset_iid_stats();

            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(5000),
            );

            // Get statistics to ensure they're computed
            let lmr_stats = engine.get_lmr_stats();
            let _exempted = lmr_stats.iid_move_explicitly_exempted;

            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark comparison: IID effectiveness with explicit exemption
fn benchmark_iid_effectiveness_with_exemption(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_effectiveness");
    group.measurement_time(Duration::from_secs(12));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 6;

    group.bench_function("search_with_coordination", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();

            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(8000),
            );

            // Verify coordination is working
            let iid_stats = engine.get_iid_stats();
            let lmr_stats = engine.get_lmr_stats();

            // Ensure IID ran
            assert!(iid_stats.iid_searches_performed > 0, "IID should have run");

            // Ensure LMR was active
            assert!(
                lmr_stats.moves_considered > 0,
                "LMR should have been active"
            );

            black_box((
                result,
                iid_stats.iid_searches_performed,
                lmr_stats.iid_move_explicitly_exempted,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_iid_lmr_coordination,
    benchmark_exemption_statistics_overhead,
    benchmark_iid_effectiveness_with_exemption
);
criterion_main!(benches);
