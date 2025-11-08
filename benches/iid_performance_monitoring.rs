#![cfg(feature = "legacy-tests")]
//! Benchmarks for IID performance monitoring improvements
//!
//! Measures the overhead introduced by detailed IID monitoring instrumentation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::{CapturedPieces, Player};
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::time_utils::TimeSource;
use std::time::Duration;

/// Task 8.8: Comparison benchmark - IID enabled vs disabled
fn benchmark_iid_enabled_vs_disabled(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_enabled_vs_disabled");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(30);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let time_limit_ms = 2000;

    // Benchmark with IID enabled (default)
    group.bench_function("with_iid", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let start_time = TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let result = engine.negamax_with_context(
                &mut board.clone(),
                &captured_pieces,
                player,
                5, // depth 5
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
                true,
                false,
                false,
                false,
            );
            black_box(result);
        });
    });

    // Benchmark with IID disabled
    group.bench_function("without_iid", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let mut config = engine.get_iid_config().clone();
        config.enabled = false;
        engine.update_iid_config(config).unwrap();

        let start_time = TimeSource::now();
        let mut hash_history = Vec::new();

        b.iter(|| {
            engine.reset_iid_stats();
            let result = engine.negamax_with_context(
                &mut board.clone(),
                &captured_pieces,
                player,
                5, // depth 5
                -10000,
                10000,
                &start_time,
                time_limit_ms,
                &mut hash_history,
                true,
                false,
                false,
                false,
            );
            black_box(result);
        });
    });

    group.finish();
}

/// Task 8.8: Comparison benchmark - Different IID configurations
fn benchmark_iid_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_configurations");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let time_limit_ms = 2000;

    // Test different configurations
    let configs = vec![
        (
            "aggressive",
            |config: &mut shogi_engine::types::IIDConfig| {
                config.max_legal_moves = 50;
                config.time_overhead_threshold = 0.20; // 20%
            },
        ),
        (
            "conservative",
            |config: &mut shogi_engine::types::IIDConfig| {
                config.max_legal_moves = 25;
                config.time_overhead_threshold = 0.10; // 10%
            },
        ),
        ("default", |_config: &mut shogi_engine::types::IIDConfig| {
            // Use default configuration
        }),
    ];

    for (name, config_modifier) in configs {
        group.bench_function(name, |b| {
            let mut engine = SearchEngine::new(None, 64);
            let mut config = engine.get_iid_config().clone();
            config_modifier(&mut config);
            engine.update_iid_config(config).unwrap();

            let start_time = TimeSource::now();
            let mut hash_history = Vec::new();

            b.iter(|| {
                engine.reset_iid_stats();
                let result = engine.negamax_with_context(
                    &mut board.clone(),
                    &captured_pieces,
                    player,
                    5, // depth 5
                    -10000,
                    10000,
                    &start_time,
                    time_limit_ms,
                    &mut hash_history,
                    true,
                    false,
                    false,
                    false,
                );
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Task 8.0: Benchmark overhead monitoring performance
fn benchmark_overhead_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("iid_overhead_monitoring");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("monitor_overhead", |b| {
        let mut engine = SearchEngine::new(None, 64);

        b.iter(|| {
            engine.monitor_iid_overhead(black_box(100), black_box(1000)); // 10% overhead
        });
    });

    group.bench_function("get_overhead_stats", |b| {
        let mut engine = SearchEngine::new(None, 64);

        // Simulate some overhead data
        for _ in 0..50 {
            engine.monitor_iid_overhead(100, 1000);
        }

        b.iter(|| {
            black_box(engine.get_iid_overhead_stats());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_iid_enabled_vs_disabled,
    benchmark_iid_configurations,
    benchmark_overhead_monitoring
);
criterion_main!(benches);
