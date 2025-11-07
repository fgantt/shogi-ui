//! Performance Benchmarks for Unified Time Pressure Management
//!
//! This benchmark suite measures the effectiveness of time pressure coordination
//!
//! Task 7.0.2.12: Measure timeout rate before and after time pressure improvements
//!
//! Metrics:
//! - Timeout rate at various time limits
//! - Search completion rate
//! - Time pressure coordination effectiveness

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with time pressure management enabled
fn create_test_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64); // 64MB hash table

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

    engine
}

/// Benchmark time pressure coordination with various time limits
fn benchmark_time_pressure_at_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_pressure_coordination");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test with various time limits
    for time_limit_ms in [50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("search_with_time_pressure", time_limit_ms),
            &time_limit_ms,
            |b, &time_limit| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    let result = engine.search_at_depth(
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                        black_box(time_limit),
                    );

                    // Get statistics to measure coordination
                    let nmp_stats = engine.get_null_move_stats();
                    let iid_stats = engine.get_iid_stats();

                    black_box((
                        result,
                        nmp_stats.skipped_time_pressure,
                        iid_stats.positions_skipped_time_pressure,
                    ))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark search completion rate at various depths and time limits
fn benchmark_search_completion_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_completion_rate");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test completion rate at depth 5 with tight time limits
    group.bench_function("completion_at_depth_5_50ms", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(5),
                black_box(50),
            );
            black_box(result)
        })
    });

    // Test completion rate at depth 6 with moderate time limits
    group.bench_function("completion_at_depth_6_200ms", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            let result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(6),
                black_box(200),
            );
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark NMP and IID skip rates under time pressure
fn benchmark_algorithm_skip_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_skip_rates");
    group.measurement_time(Duration::from_secs(12));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 6;

    // Test with very tight time limit (high pressure)
    group.bench_function("skip_rate_high_pressure_10ms", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();

            let _result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(10), // Very tight - should trigger High pressure
            );

            // Measure skip rates
            let nmp_stats = engine.get_null_move_stats();
            let iid_stats = engine.get_iid_stats();

            let nmp_skip_rate = if nmp_stats.attempts + nmp_stats.skipped_time_pressure > 0 {
                nmp_stats.skipped_time_pressure as f64
                    / (nmp_stats.attempts + nmp_stats.skipped_time_pressure) as f64
            } else {
                0.0
            };

            let iid_skip_rate = if iid_stats.iid_searches_performed
                + iid_stats.positions_skipped_time_pressure
                > 0
            {
                iid_stats.positions_skipped_time_pressure as f64
                    / (iid_stats.iid_searches_performed + iid_stats.positions_skipped_time_pressure)
                        as f64
            } else {
                0.0
            };

            black_box((nmp_skip_rate, iid_skip_rate))
        })
    });

    // Test with moderate time limit (medium pressure)
    group.bench_function("skip_rate_medium_pressure_100ms", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();

            let _result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(100),
            );

            let nmp_stats = engine.get_null_move_stats();
            let iid_stats = engine.get_iid_stats();

            black_box((
                nmp_stats.skipped_time_pressure,
                iid_stats.positions_skipped_time_pressure,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_time_pressure_at_limits,
    benchmark_search_completion_rate,
    benchmark_algorithm_skip_rates
);
criterion_main!(benches);
