//! Performance benchmarks for the tablebase system
//!
//! This module contains performance benchmarks to measure the efficiency
//! of tablebase operations, caching, and memory usage.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::tablebase::{MicroTablebase, TablebaseConfig};
use shogi_engine::types::{CapturedPieces, Player};
use std::time::Duration;

/// Benchmark tablebase probe performance
fn benchmark_tablebase_probe(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_probe");
    group.measurement_time(Duration::from_secs(10));

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("probe_empty_board", |b| {
        b.iter(|| {
            black_box(tablebase.probe(
                black_box(&board),
                black_box(player),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

/// Benchmark tablebase cache performance
fn benchmark_tablebase_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_cache");
    group.measurement_time(Duration::from_secs(10));

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Warm up cache
    for _ in 0..100 {
        tablebase.probe(&board, player, &captured_pieces);
    }

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            black_box(tablebase.probe(
                black_box(&board),
                black_box(player),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

/// Benchmark tablebase configuration loading
fn benchmark_tablebase_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_config");
    group.measurement_time(Duration::from_secs(5));

    let config = TablebaseConfig::default();
    let json = config.to_json().unwrap();

    group.bench_function("config_serialization", |b| {
        b.iter(|| black_box(config.to_json().unwrap()))
    });

    group.bench_function("config_deserialization", |b| {
        b.iter(|| black_box(TablebaseConfig::from_json(black_box(&json)).unwrap()))
    });

    group.finish();
}

/// Benchmark tablebase statistics collection
fn benchmark_tablebase_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_stats");
    group.measurement_time(Duration::from_secs(5));

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Perform some operations to generate stats
    for _ in 0..1000 {
        tablebase.probe(&board, player, &captured_pieces);
    }

    group.bench_function("stats_collection", |b| {
        b.iter(|| black_box(tablebase.get_stats()))
    });

    group.finish();
}

/// Benchmark tablebase with different cache sizes
fn benchmark_tablebase_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_cache_sizes");
    group.measurement_time(Duration::from_secs(10));

    let cache_sizes = vec![100, 1000, 10000, 100000];

    for size in cache_sizes {
        let mut config = TablebaseConfig::default();
        config.cache_size = size;
        let mut tablebase = MicroTablebase::with_config(config);

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        group.bench_with_input(BenchmarkId::new("cache_size", size), &size, |b, _| {
            b.iter(|| {
                black_box(tablebase.probe(
                    black_box(&board),
                    black_box(player),
                    black_box(&captured_pieces),
                ))
            })
        });
    }

    group.finish();
}

/// Benchmark tablebase memory usage
fn benchmark_tablebase_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_memory");
    group.measurement_time(Duration::from_secs(15));

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("memory_usage_1000_probes", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                tablebase.probe(&board, player, &captured_pieces);
            }
            let stats = tablebase.get_stats().clone();
            black_box(stats)
        })
    });

    group.finish();
}

/// Benchmark tablebase solver performance
fn benchmark_tablebase_solvers(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_solvers");
    group.measurement_time(Duration::from_secs(10));

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("solver_performance", |b| {
        b.iter(|| {
            black_box(tablebase.probe(
                black_box(&board),
                black_box(player),
                black_box(&captured_pieces),
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tablebase_probe,
    benchmark_tablebase_cache,
    benchmark_tablebase_config,
    benchmark_tablebase_stats,
    benchmark_tablebase_cache_sizes,
    benchmark_tablebase_memory,
    benchmark_tablebase_solvers
);

criterion_main!(benches);
