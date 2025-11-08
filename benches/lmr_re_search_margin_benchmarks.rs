#![cfg(feature = "legacy-tests")]
//! Benchmarks for LMR re-search margin tuning
//!
//! Measures the performance impact of adjusting the re-search margin thresholds
//! on LMR effectiveness. It compares:
//! - LMR with re-search margin disabled (margin = 0)
//! - LMR with different margin values (25, 50, 75, 100 centipawns)
//!
//! Metrics measured:
//! - Search time
//! - Nodes searched
//! - Re-search rate
//! - LMR effectiveness (efficiency, cutoff rate)
//! - Re-search margin effectiveness
//!
//! Expected results:
//! - Re-search margin should reduce re-search rate (fewer unnecessary re-searches)
//! - Re-search margin should improve efficiency without significantly impacting accuracy
//! - Optimal margin value should balance re-search reduction with search accuracy

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, LMRConfig, Player},
};
use std::time::Duration;

/// Create a test engine with specific re-search margin
fn create_test_engine_with_margin(margin: i32) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16); // 16MB hash table
    let mut config = LMRConfig::default();
    config.re_search_margin = margin;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Benchmark LMR with re-search margin disabled (margin = 0)
fn benchmark_lmr_without_margin(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_without_margin");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("search_depth", depth),
            &depth,
            |b, &depth| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_margin(0);
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
                    black_box((result, stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark LMR with different re-search margin values
fn benchmark_lmr_with_margin_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("lmr_with_margin_values");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test different margin values
    for margin in [0, 25, 50, 75, 100] {
        group.bench_with_input(
            BenchmarkId::new("re_search_margin", margin),
            &margin,
            |b, &margin| {
                b.iter(|| {
                    let mut engine = create_test_engine_with_margin(margin);
                    engine.reset_lmr_stats();

                    let mut board_mut = board.clone();
                    let start_time = std::time::Instant::now();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        depth,
                        1000,
                    );
                    let search_time = start_time.elapsed();

                    let stats = engine.get_lmr_stats().clone();
                    let efficiency = stats.efficiency();
                    let research_rate = stats.research_rate();
                    let margin_effectiveness = stats.re_search_margin_effectiveness();

                    black_box((
                        result,
                        stats,
                        search_time,
                        efficiency,
                        research_rate,
                        margin_effectiveness,
                    ))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark re-search margin effectiveness
fn benchmark_re_search_margin_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("re_search_margin_effectiveness");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("margin_0", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_margin(0);
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
            black_box((result, stats))
        });
    });

    group.bench_function("margin_50", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_margin(50);
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
            let margin_effectiveness = stats.re_search_margin_effectiveness();
            black_box((result, stats, margin_effectiveness))
        });
    });

    group.finish();
}

/// Benchmark optimal margin value search
fn benchmark_optimal_margin_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimal_margin_value");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Test margin values: 0, 25, 50, 75, 100
    for margin in [0, 25, 50, 75, 100] {
        group.bench_with_input(BenchmarkId::new("margin", margin), &margin, |b, &margin| {
            b.iter(|| {
                let mut engine = create_test_engine_with_margin(margin);
                engine.reset_lmr_stats();

                let mut board_mut = board.clone();
                let start_time = std::time::Instant::now();
                let result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    2000,
                );
                let search_time = start_time.elapsed();

                let stats = engine.get_lmr_stats().clone();

                // Calculate key metrics
                let efficiency = stats.efficiency();
                let research_rate = stats.research_rate();
                let cutoff_rate = stats.cutoff_rate();
                let margin_effectiveness = stats.re_search_margin_effectiveness();
                let margin_prevented = stats.re_search_margin_prevented;
                let margin_allowed = stats.re_search_margin_allowed;

                black_box((
                    result,
                    stats,
                    search_time,
                    efficiency,
                    research_rate,
                    cutoff_rate,
                    margin_effectiveness,
                    margin_prevented,
                    margin_allowed,
                ))
            });
        });
    }

    group.finish();
}

/// Benchmark re-search rate impact
fn benchmark_re_search_rate_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("re_search_rate_impact");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    // Compare margin = 0 vs margin = 50
    group.bench_function("margin_0_research_rate", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_margin(0);
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
            let research_rate = stats.research_rate();
            black_box((result, stats, research_rate))
        });
    });

    group.bench_function("margin_50_research_rate", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_margin(50);
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
            let research_rate = stats.research_rate();
            let margin_prevented = stats.re_search_margin_prevented;
            black_box((result, stats, research_rate, margin_prevented))
        });
    });

    group.finish();
}

/// Benchmark comprehensive re-search margin analysis
fn benchmark_comprehensive_margin_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_margin_analysis");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(15);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let depth = 5;

    group.bench_function("full_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_with_margin(50);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let start_time = std::time::Instant::now();
            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                depth,
                2000,
            );
            let search_time = start_time.elapsed();

            let stats = engine.get_lmr_stats().clone();

            // Comprehensive metrics
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();
            let margin_effectiveness = stats.re_search_margin_effectiveness();
            let margin_prevented = stats.re_search_margin_prevented;
            let margin_allowed = stats.re_search_margin_allowed;
            let avg_reduction = stats.average_reduction;
            let avg_depth_saved = stats.average_depth_saved();

            black_box((
                result,
                stats,
                search_time,
                efficiency,
                research_rate,
                cutoff_rate,
                margin_effectiveness,
                margin_prevented,
                margin_allowed,
                avg_reduction,
                avg_depth_saved,
            ))
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(10);
    targets =
        benchmark_lmr_without_margin,
        benchmark_lmr_with_margin_values,
        benchmark_re_search_margin_effectiveness,
        benchmark_optimal_margin_value,
        benchmark_re_search_rate_impact,
        benchmark_comprehensive_margin_analysis
}

criterion_main!(benches);
