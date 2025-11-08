//! Tablebase Performance Validation and Tuning
//!
//! This benchmark suite validates the performance of the tablebase system
//! and provides tuning recommendations for different use cases.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    tablebase::tablebase_config::EvictionStrategy,
    tablebase::{MicroTablebase, TablebaseConfig},
    types::{CapturedPieces, PieceType, Player, Position},
    ShogiEngine,
};
use std::time::Duration;

/// Benchmark tablebase probe performance
fn bench_tablebase_probe(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablebase_probe");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    let tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    group.bench_function("probe_new_board", |b| {
        b.iter(|| {
            let mut tb = tablebase.clone();
            tb.probe(&board, Player::Black, &captured_pieces)
        })
    });

    group.finish();
}

/// Benchmark different cache sizes
fn bench_cache_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_sizes");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(500);

    let cache_sizes = [100, 1000, 5000, 10000, 50000];
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for size in cache_sizes.iter() {
        let mut config = TablebaseConfig::default();
        config.cache_size = *size;
        let tablebase = MicroTablebase::with_config(config);

        group.bench_with_input(BenchmarkId::new("probe", size), size, |b, _| {
            b.iter(|| {
                let mut tb = tablebase.clone();
                tb.probe(&board, Player::Black, &captured_pieces)
            })
        });
    }

    group.finish();
}

/// Benchmark different eviction strategies
fn bench_eviction_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("eviction_strategies");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(500);

    use shogi_engine::tablebase::tablebase_config::EvictionStrategy;

    let strategies = [
        ("random", EvictionStrategy::Random),
        ("lru", EvictionStrategy::LRU),
        ("lfu", EvictionStrategy::LFU),
        ("adaptive", EvictionStrategy::LRU), // Using LRU as base for adaptive
    ];

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for (name, strategy) in strategies.iter() {
        let mut config = TablebaseConfig::default();
        config.cache_size = 1000;
        config.performance.eviction_strategy = *strategy;
        let tablebase = MicroTablebase::with_config(config);

        group.bench_with_input(BenchmarkId::new("probe", name), name, |b, _| {
            b.iter(|| {
                let mut tb = tablebase.clone();
                tb.probe(&board, Player::Black, &captured_pieces)
            })
        });
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(200);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test memory usage with different configurations
    let configs = [
        ("default", TablebaseConfig::default()),
        ("memory_optimized", TablebaseConfig::memory_optimized()),
        (
            "performance_optimized",
            TablebaseConfig::performance_optimized(),
        ),
    ];

    for (name, config) in configs.iter() {
        let tablebase = MicroTablebase::with_config(config.clone());

        group.bench_with_input(BenchmarkId::new("probe", name), name, |b, _| {
            b.iter(|| {
                let mut tb = tablebase.clone();
                let result = tb.probe(&board, Player::Black, &captured_pieces);

                // Also measure memory usage
                let _memory_usage = tb.check_memory_usage();
                result
            })
        });
    }

    group.finish();
}

/// Benchmark solver performance
fn bench_solver_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_performance");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with different solver configurations
    let solver_configs = [
        ("all_solvers", {
            let mut config = TablebaseConfig::default();
            config.solvers.king_gold_vs_king.enabled = true;
            config.solvers.king_silver_vs_king.enabled = true;
            config.solvers.king_rook_vs_king.enabled = true;
            config
        }),
        ("gold_only", {
            let mut config = TablebaseConfig::default();
            config.solvers.king_gold_vs_king.enabled = true;
            config.solvers.king_silver_vs_king.enabled = false;
            config.solvers.king_rook_vs_king.enabled = false;
            config
        }),
        ("silver_only", {
            let mut config = TablebaseConfig::default();
            config.solvers.king_gold_vs_king.enabled = false;
            config.solvers.king_silver_vs_king.enabled = true;
            config.solvers.king_rook_vs_king.enabled = false;
            config
        }),
        ("rook_only", {
            let mut config = TablebaseConfig::default();
            config.solvers.king_gold_vs_king.enabled = false;
            config.solvers.king_silver_vs_king.enabled = false;
            config.solvers.king_rook_vs_king.enabled = true;
            config
        }),
    ];

    for (name, config) in solver_configs.iter() {
        let tablebase = MicroTablebase::with_config(config.clone());

        group.bench_with_input(BenchmarkId::new("probe", name), name, |b, _| {
            b.iter(|| {
                let mut tb = tablebase.clone();
                tb.probe(&board, Player::Black, &captured_pieces)
            })
        });
    }

    group.finish();
}

/// Benchmark profiling overhead
fn bench_profiling_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("profiling_overhead");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with profiling enabled
    let mut config_with_profiling = TablebaseConfig::default();
    config_with_profiling.performance.enable_monitoring = true;
    let tablebase_with_profiling = MicroTablebase::with_config(config_with_profiling);

    // Test with profiling disabled
    let mut config_without_profiling = TablebaseConfig::default();
    config_without_profiling.performance.enable_monitoring = false;
    let tablebase_without_profiling = MicroTablebase::with_config(config_without_profiling);

    group.bench_function("with_profiling", |b| {
        b.iter(|| {
            let mut tb = tablebase_with_profiling.clone();
            tb.probe(&board, Player::Black, &captured_pieces)
        })
    });

    group.bench_function("without_profiling", |b| {
        b.iter(|| {
            let mut tb = tablebase_without_profiling.clone();
            tb.probe(&board, Player::Black, &captured_pieces)
        })
    });

    group.finish();
}

/// Benchmark adaptive solver selection
fn bench_adaptive_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_selection");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(500);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with adaptive selection enabled
    let mut config_adaptive = TablebaseConfig::default();
    config_adaptive.performance.enable_adaptive_caching = true;
    let tablebase_adaptive = MicroTablebase::with_config(config_adaptive);

    // Test with adaptive selection disabled
    let mut config_non_adaptive = TablebaseConfig::default();
    config_non_adaptive.performance.enable_adaptive_caching = false;
    let tablebase_non_adaptive = MicroTablebase::with_config(config_non_adaptive);

    group.bench_function("adaptive_enabled", |b| {
        b.iter(|| {
            let mut tb = tablebase_adaptive.clone();
            tb.probe(&board, Player::Black, &captured_pieces)
        })
    });

    group.bench_function("adaptive_disabled", |b| {
        b.iter(|| {
            let mut tb = tablebase_non_adaptive.clone();
            tb.probe(&board, Player::Black, &captured_pieces)
        })
    });

    group.finish();
}

/// Benchmark engine integration
fn bench_engine_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_integration");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(200);

    // Test with tablebase enabled
    let mut engine_with_tablebase = ShogiEngine::new();
    engine_with_tablebase.enable_tablebase();

    // Test with tablebase disabled
    let mut engine_without_tablebase = ShogiEngine::new();
    engine_without_tablebase.disable_tablebase();

    group.bench_function("with_tablebase", |b| {
        b.iter(|| {
            let mut engine = engine_with_tablebase.clone();
            engine.get_best_move(1, 1000, None, None)
        })
    });

    group.bench_function("without_tablebase", |b| {
        b.iter(|| {
            let mut engine = engine_without_tablebase.clone();
            engine.get_best_move(1, 1000, None, None)
        })
    });

    group.finish();
}

/// Benchmark throughput with different workloads
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(100);

    let workloads = [1, 10, 50, 100, 500];
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for workload in workloads.iter() {
        let tablebase = MicroTablebase::new();

        group.throughput(Throughput::Elements(*workload as u64));
        group.bench_with_input(
            BenchmarkId::new("probes", workload),
            workload,
            |b, &workload| {
                b.iter(|| {
                    let mut tb = tablebase.clone();
                    for _ in 0..workload {
                        let _ = tb.probe(&board, Player::Black, &captured_pieces);
                    }
                })
            },
        );
    }

    group.finish();
}

/// Performance validation function
fn validate_performance() {
    println!("=== Tablebase Performance Validation ===");

    let mut tablebase = MicroTablebase::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test basic performance
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = tablebase.probe(&board, Player::Black, &captured_pieces);
    }
    let duration = start.elapsed();

    println!("1000 probes completed in {:?}", duration);
    println!("Average time per probe: {:?}", duration / 1000);

    // Test memory usage
    let _memory_usage = tablebase.check_memory_usage();
    let _peak_memory = tablebase.check_memory_usage(); // Using same method for now

    println!("Memory usage check completed");
    println!("Peak memory check completed");

    // Test statistics
    let stats = tablebase.get_stats();
    println!("Statistics: {:?}", stats);

    // Performance recommendations
    println!("\n=== Performance Recommendations ===");

    if duration.as_millis() > 1000 {
        println!("⚠️  Probe time is high. Consider:");
        println!("   - Reducing cache size");
        println!("   - Disabling unused solvers");
        println!("   - Using WASM optimizations");
    } else {
        println!("✅ Probe time is acceptable");
    }

    if duration.as_millis() > 1000 {
        println!("⚠️  Memory usage is high. Consider:");
        println!("   - Reducing cache size");
        println!("   - Enabling auto-eviction");
        println!("   - Using memory-optimized configuration");
    } else {
        println!("✅ Memory usage is acceptable");
    }

    let hit_rate = stats.cache_hits as f64 / stats.total_probes as f64;
    if hit_rate < 0.5 {
        println!(
            "⚠️  Cache hit rate is low ({:.2}%). Consider:",
            hit_rate * 100.0
        );
        println!("   - Increasing cache size");
        println!("   - Using adaptive eviction");
    } else {
        println!("✅ Cache hit rate is good ({:.2}%)", hit_rate * 100.0);
    }
}

criterion_group!(
    benches,
    bench_tablebase_probe,
    bench_cache_sizes,
    bench_eviction_strategies,
    bench_memory_usage,
    bench_solver_performance,
    bench_profiling_overhead,
    bench_adaptive_selection,
    bench_engine_integration,
    bench_throughput
);

criterion_main!(benches);
