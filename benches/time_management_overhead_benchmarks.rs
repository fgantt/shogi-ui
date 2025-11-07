// Benchmark time check overhead and frequency optimization (Task 8.1, 8.4, 8.6)
// This benchmark measures the overhead of time checks in deep searches and the
// performance impact of checking every N nodes instead of every node.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{
    bitboards::BitboardBoard,
    moves::CapturedPieces,
    search::search_engine::SearchEngine,
    time_utils::TimeSource,
    types::{EngineConfig, Player},
};

fn bench_time_check_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_check_overhead");

    // Create a test position
    let board = BitboardBoard::from_starting_position();
    let captured = CapturedPieces::new();
    let player = Player::Sente;

    // Test different time check frequencies
    let frequencies = vec![1, 64, 256, 1024, 4096];

    for frequency in frequencies {
        let mut config = EngineConfig::get_preset("default");
        config.time_management.time_check_frequency = frequency;

        let engine = SearchEngine::new_with_engine_config(None, 64, config);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("frequency_{}", frequency)),
            &frequency,
            |b, _| {
                b.iter(|| {
                    let mut engine = engine.clone();
                    let start_time = TimeSource::now();

                    // Simulate many time checks
                    for _ in 0..10000 {
                        black_box(engine.should_stop(&start_time, 10000));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_time_check_vs_no_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_check_vs_no_check");

    let board = BitboardBoard::from_starting_position();
    let captured = CapturedPieces::new();
    let player = Player::Sente;

    // Test with time checks vs without
    let mut config_with_checks = EngineConfig::get_preset("default");
    config_with_checks.time_management.time_check_frequency = 1024; // Check every 1024 nodes

    let mut config_no_checks = EngineConfig::get_preset("default");
    config_no_checks.time_management.time_check_frequency = 1_000_000; // Effectively no checks

    group.bench_function("with_time_checks_1024", |b| {
        let engine = SearchEngine::new_with_engine_config(None, 64, config_with_checks.clone());
        b.iter(|| {
            let mut engine = engine.clone();
            let start_time = TimeSource::now();

            // Simulate search loop with time checks
            for i in 0..100000 {
                if i % 1024 == 0 {
                    black_box(engine.should_stop_force(&start_time, 10000));
                }
            }
        });
    });

    group.bench_function("without_time_checks", |b| {
        let engine = SearchEngine::new_with_engine_config(None, 64, config_no_checks.clone());
        b.iter(|| {
            let mut engine = engine.clone();
            let start_time = TimeSource::now();

            // Simulate search loop without time checks
            for _ in 0..100000 {
                // Do some work instead of time check
                black_box(engine.nodes_searched);
            }
        });
    });

    group.finish();
}

fn bench_has_exceeded_limit_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_exceeded_limit_overhead");

    // Measure the overhead of has_exceeded_limit calls
    group.bench_function("has_exceeded_limit_call", |b| {
        let start_time = TimeSource::now();
        b.iter(|| {
            black_box(start_time.has_exceeded_limit(10000));
        });
    });

    group.bench_function("elapsed_ms_call", |b| {
        let start_time = TimeSource::now();
        b.iter(|| {
            black_box(start_time.elapsed_ms());
        });
    });

    group.finish();
}

fn bench_cumulative_time_check_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("cumulative_time_check_overhead");

    let board = BitboardBoard::from_starting_position();
    let captured = CapturedPieces::new();
    let player = Player::Sente;

    // Simulate different node counts
    let node_counts = vec![1000, 10000, 100000, 1000000];

    for node_count in node_counts {
        let mut config = EngineConfig::get_preset("default");
        config.time_management.time_check_frequency = 1024;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_nodes", node_count)),
            &node_count,
            |b, &count| {
                let engine = SearchEngine::new_with_engine_config(None, 64, config.clone());
                b.iter(|| {
                    let mut engine = engine.clone();
                    let start_time = TimeSource::now();

                    // Simulate checking time every frequency nodes
                    let frequency = config.time_management.time_check_frequency;
                    for i in 0..count {
                        if i % frequency == 0 {
                            black_box(engine.should_stop_force(&start_time, 10000));
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_safety_margin_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("safety_margin_calculation");

    let time_limits = vec![100, 500, 1000, 5000, 10000];

    for time_limit in time_limits {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}ms", time_limit)),
            &time_limit,
            |b, &limit| {
                let mut config = EngineConfig::get_preset("default");
                b.iter(|| {
                    // Simulate safety margin calculation
                    let percentage_margin =
                        (limit as f64 * config.time_management.safety_margin) as u32;
                    let absolute_margin = config.time_management.absolute_safety_margin_ms;
                    let total_margin = percentage_margin.max(absolute_margin);
                    black_box(limit.saturating_sub(total_margin));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_has_exceeded_limit_overhead,
    bench_safety_margin_calculation,
    bench_time_check_frequency_simulation
);
criterion_main!(benches);
