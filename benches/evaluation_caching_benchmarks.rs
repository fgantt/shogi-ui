//! Performance Benchmarks for Evaluation Result Caching
//!
//! This benchmark suite measures the impact of evaluation caching on search performance
//!
//! Task 7.0.4.11: Measure evaluation overhead reduction
//! Task 7.0.4.12: Verify 50-70% reduction in evaluation calls
//!
//! Metrics:
//! - Search time with evaluation caching
//! - Evaluation cache hit rate
//! - Calls saved through caching

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, IIDConfig, NullMoveConfig, Player},
};
use std::time::Duration;

/// Create a test engine with evaluation caching
fn create_test_engine() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 64); // 64MB hash table

    // Enable NMP and IID to benefit from caching
    let mut nmp_config = engine.get_null_move_config().clone();
    nmp_config.enabled = true;
    nmp_config.min_depth = 3;
    engine.update_null_move_config(nmp_config).unwrap();

    let mut iid_config = engine.get_iid_config().clone();
    iid_config.enabled = true;
    iid_config.min_depth = 4;
    engine.update_iid_config(iid_config).unwrap();

    engine
}

/// Benchmark evaluation overhead with caching
fn benchmark_evaluation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_caching");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Benchmark at different depths
    for depth in [5, 6] {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("search_with_eval_caching", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine();
                    engine.reset_core_search_metrics();

                    let result = engine.search_at_depth(
                        black_box(&board),
                        black_box(&captured_pieces),
                        black_box(player),
                        black_box(depth),
                        black_box(5000),
                    );

                    // Get caching statistics
                    let metrics = engine.get_core_search_metrics();

                    black_box((
                        result,
                        metrics.evaluation_cache_hits,
                        metrics.evaluation_calls_saved,
                    ))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark evaluation cache hit rate
fn benchmark_evaluation_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_cache_hit_rate");
    group.measurement_time(Duration::from_secs(12));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("cache_hit_rate_measurement", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_core_search_metrics();

            let _result = engine.search_at_depth(
                black_box(&board),
                black_box(&captured_pieces),
                black_box(player),
                black_box(depth),
                black_box(3000),
            );

            // Calculate cache hit rate
            let metrics = engine.get_core_search_metrics();

            let cache_hit_rate = if metrics.total_nodes > 0 {
                (metrics.evaluation_cache_hits as f64 / metrics.total_nodes as f64) * 100.0
            } else {
                0.0
            };

            let calls_saved_rate = if metrics.total_nodes > 0 {
                (metrics.evaluation_calls_saved as f64 / metrics.total_nodes as f64) * 100.0
            } else {
                0.0
            };

            black_box((
                cache_hit_rate,
                calls_saved_rate,
                metrics.evaluation_cache_hits,
                metrics.evaluation_calls_saved,
            ))
        })
    });

    group.finish();
}

/// Benchmark evaluation caching with multiple searches
fn benchmark_evaluation_caching_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_caching_efficiency");
    group.measurement_time(Duration::from_secs(18));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("multiple_searches_caching", |b| {
        b.iter(|| {
            let mut engine = create_test_engine();
            engine.reset_core_search_metrics();

            // Perform multiple searches to accumulate statistics
            for _ in 0..3 {
                let _result = engine.search_at_depth(
                    black_box(&board),
                    black_box(&captured_pieces),
                    black_box(player),
                    black_box(depth),
                    black_box(2000),
                );
            }

            // Measure cumulative caching effectiveness
            let metrics = engine.get_core_search_metrics();

            let total_cache_benefit = metrics.evaluation_calls_saved;
            let efficiency_rate = if metrics.total_nodes > 0 {
                (total_cache_benefit as f64 / metrics.total_nodes as f64) * 100.0
            } else {
                0.0
            };

            black_box((
                efficiency_rate,
                total_cache_benefit,
                metrics.evaluation_cache_hits,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evaluation_overhead,
    benchmark_evaluation_cache_hit_rate,
    benchmark_evaluation_caching_efficiency
);
criterion_main!(benches);
