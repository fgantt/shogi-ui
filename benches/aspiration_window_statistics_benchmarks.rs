//! Benchmarks for aspiration window statistics tracking overhead (Task 7.6)
//!
//! This module benchmarks the performance impact of statistics tracking:
//! - Statistics tracking with tracking enabled vs disabled
//! - Position type tracking overhead
//! - Incremental average update performance
//! - Conditional compilation impact

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::{search::SearchEngine, types::*};

fn bench_statistics_tracking_overhead(c: &mut Criterion) {
    // Task 7.6: Benchmark statistics tracking overhead with/without tracking
    let mut group = c.benchmark_group("aspiration_window_statistics_tracking");

    // Test with statistics enabled
    group.bench_function("with_statistics_tracking", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let mut config = AspirationWindowConfig::default();
        config.enable_statistics = true;
        config.disable_statistics_in_production = false;
        config.enable_position_type_tracking = true;
        engine.update_aspiration_window_config(config).unwrap();

        b.iter(|| {
            // Simulate window size calculation with stats tracking
            for depth in 2..=6 {
                for prev_score in [-100, 0, 100] {
                    for failures in 0..3 {
                        engine.calculate_window_size_with_stats(depth, prev_score, failures);
                    }
                }
            }
        });
    });

    // Test with statistics disabled
    group.bench_function("without_statistics_tracking", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let mut config = AspirationWindowConfig::default();
        config.enable_statistics = false; // Disabled
        config.disable_statistics_in_production = true;
        config.enable_position_type_tracking = false;
        engine.update_aspiration_window_config(config).unwrap();

        b.iter(|| {
            // Simulate window size calculation without stats tracking
            for depth in 2..=6 {
                for prev_score in [-100, 0, 100] {
                    for failures in 0..3 {
                        engine.calculate_window_size(depth, prev_score, failures);
                    }
                }
            }
        });
    });

    group.finish();
}

fn bench_position_type_tracking_overhead(c: &mut Criterion) {
    // Task 7.6: Benchmark position type tracking overhead
    let mut group = c.benchmark_group("aspiration_window_position_type_tracking");

    // Test with position type tracking enabled
    group.bench_function("with_position_type_tracking", |b| {
        let mut stats = AspirationWindowStats::default();
        let phases = vec![
            GamePhase::Opening,
            GamePhase::Middlegame,
            GamePhase::Endgame,
        ];
        let window_sizes = vec![50, 75, 100];

        b.iter(|| {
            for (phase, window_size) in phases.iter().zip(window_sizes.iter()) {
                stats.update_window_size_by_position_type(*phase, *window_size);
                stats.update_success_rate_by_position_type(*phase, true);
            }
        });
    });

    // Test with position type tracking disabled (just update basic stats)
    group.bench_function("without_position_type_tracking", |b| {
        let mut stats = AspirationWindowStats::default();

        b.iter(|| {
            // Just update basic average window size (no position type breakdown)
            let window_size = 75;
            let total = stats.total_searches;
            if total > 0 {
                let diff = (window_size as f64 - stats.average_window_size) / (total + 1) as f64;
                stats.average_window_size += diff;
            } else {
                stats.average_window_size = window_size as f64;
            }
            stats.total_searches += 1;
        });
    });

    group.finish();
}

fn bench_incremental_average_update(c: &mut Criterion) {
    // Task 7.4, 7.6: Benchmark incremental average update performance
    let mut group = c.benchmark_group("aspiration_window_incremental_average");

    // Test incremental average update (optimized method)
    group.bench_function("incremental_update", |b| {
        let mut stats = AspirationWindowStats::default();
        let window_sizes: Vec<i32> = (50..=150).step_by(10).collect();

        b.iter(|| {
            for window_size in window_sizes.iter() {
                let total = stats.total_searches;
                if total > 0 {
                    let diff =
                        (*window_size as f64 - stats.average_window_size) / (total + 1) as f64;
                    stats.average_window_size += diff;
                } else {
                    stats.average_window_size = *window_size as f64;
                }
                stats.total_searches += 1;
            }
            stats.total_searches = 0;
            stats.average_window_size = 0.0;
        });
    });

    // Test recalculating average from total (non-optimized method)
    group.bench_function("recalculate_from_total", |b| {
        let mut stats = AspirationWindowStats::default();
        let mut window_size_list = Vec::new();
        let window_sizes: Vec<i32> = (50..=150).step_by(10).collect();

        b.iter(|| {
            for window_size in window_sizes.iter() {
                window_size_list.push(*window_size);
                // Recalculate average from all values (slower)
                let sum: i32 = window_size_list.iter().sum();
                stats.average_window_size = sum as f64 / window_size_list.len() as f64;
            }
            window_size_list.clear();
            stats.average_window_size = 0.0;
        });
    });

    group.finish();
}

fn bench_statistics_update_methods(c: &mut Criterion) {
    // Task 7.4, 7.6: Benchmark different statistics update methods
    let mut group = c.benchmark_group("aspiration_window_update_methods");

    // Benchmark update_aspiration_stats (simulated via public methods)
    group.bench_function("update_aspiration_stats", |b| {
        let mut engine = SearchEngine::new(None, 64);
        let mut config = AspirationWindowConfig::default();
        config.enable_statistics = true;
        config.disable_statistics_in_production = false;
        engine.update_aspiration_window_config(config).unwrap();

        b.iter(|| {
            // Simulate statistics update operations
            // We can't call private methods, so we benchmark equivalent operations
            let engine_config = engine.get_aspiration_window_config();
            let should_track_stats =
                engine_config.enable_statistics && !engine_config.disable_statistics_in_production;

            if should_track_stats {
                // In real code, this would call update_aspiration_stats
                // Here we just benchmark the conditional check overhead
            }
            // Core metrics are always updated (not conditional)
        });
    });

    // Benchmark position type tracking methods
    group.bench_function("position_type_tracking_methods", |b| {
        let mut stats = AspirationWindowStats::default();
        let phases = vec![
            GamePhase::Opening,
            GamePhase::Middlegame,
            GamePhase::Endgame,
        ];
        let window_sizes = vec![50, 75, 100];

        b.iter(|| {
            for (phase, window_size) in phases.iter().zip(window_sizes.iter()) {
                stats.update_window_size_by_position_type(*phase, *window_size);
                stats.update_success_rate_by_position_type(*phase, true);
            }
        });
    });

    group.finish();
}

fn bench_conditional_statistics_check(c: &mut Criterion) {
    // Task 7.3, 7.6: Benchmark conditional statistics check overhead
    let mut group = c.benchmark_group("aspiration_window_conditional_check");

    // Test with all conditions checked
    group.bench_function("full_conditional_check", |b| {
        let config = AspirationWindowConfig {
            enable_statistics: true,
            disable_statistics_in_production: false,
            enable_position_type_tracking: true,
            ..AspirationWindowConfig::default()
        };

        b.iter(|| {
            let should_track_stats =
                config.enable_statistics && !config.disable_statistics_in_production;

            #[cfg(not(feature = "statistics"))]
            let should_track_stats = false;

            should_track_stats
        });
    });

    // Test with early return (statistics disabled)
    group.bench_function("early_return_disabled", |b| {
        let config = AspirationWindowConfig {
            enable_statistics: false,
            disable_statistics_in_production: false,
            enable_position_type_tracking: false,
            ..AspirationWindowConfig::default()
        };

        b.iter(|| {
            let should_track_stats =
                config.enable_statistics && !config.disable_statistics_in_production;

            should_track_stats
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_statistics_tracking_overhead,
    bench_position_type_tracking_overhead,
    bench_incremental_average_update,
    bench_statistics_update_methods,
    bench_conditional_statistics_check
);

criterion_main!(benches);
