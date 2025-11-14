//! Benchmarks for magic table initialization strategies
//!
//! These benchmarks measure:
//! - Sequential vs. parallel initialization time
//! - Lazy initialization overhead (first access latency)
//! - Progress reporting overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::magic::{
    lazy_init::LazyMagicTable, parallel_init::ParallelInitializer, MagicTable,
};
use shogi_engine::types::{Bitboard, PieceType};
use std::time::Instant;

fn benchmark_sequential_initialization(c: &mut Criterion) {
    c.bench_function("sequential_initialization", |b| {
        b.iter(|| {
            let table = MagicTable::new().unwrap();
            black_box(table);
        });
    });
}

fn benchmark_parallel_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_initialization");

    // Test with different thread counts
    for threads in [0, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_threads", if *threads == 0 { "auto" } else { &threads.to_string() })),
            threads,
            |b, &thread_count| {
                b.iter(|| {
                    let initializer = if thread_count == 0 {
                        ParallelInitializer::new()
                    } else {
                        ParallelInitializer::with_threads(thread_count)
                    };
                    let table = initializer.initialize().unwrap();
                    black_box(table);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_vs_parallel");

    // Sequential
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let initializer = ParallelInitializer::new();
            let table = initializer.initialize_sequential().unwrap();
            black_box(table);
        });
    });

    // Parallel
    group.bench_function("parallel", |b| {
        b.iter(|| {
            let initializer = ParallelInitializer::new();
            let table = initializer.initialize().unwrap();
            black_box(table);
        });
    });

    group.finish();

    // Print comparison
    let seq_start = Instant::now();
    let _seq_table = ParallelInitializer::new().initialize_sequential().unwrap();
    let seq_time = seq_start.elapsed();

    let par_start = Instant::now();
    let _par_table = ParallelInitializer::new().initialize().unwrap();
    let par_time = par_start.elapsed();

    println!("\n=== Initialization Performance ===");
    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    if seq_time.as_millis() > 0 {
        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();
        println!("Speedup: {:.2}x", speedup);
    }
}

fn benchmark_lazy_initialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_initialization");

    // First access (includes initialization)
    group.bench_function("first_access", |b| {
        b.iter(|| {
            let table = LazyMagicTable::new().unwrap();
            let attacks = table.get_attacks(40, PieceType::Rook, 0);
            black_box(attacks);
        });
    });

    // Subsequent access (no initialization)
    group.bench_function("subsequent_access", |b| {
        let table = LazyMagicTable::new().unwrap();
        // Pre-initialize
        let _ = table.get_attacks(40, PieceType::Rook, 0);
        
        b.iter(|| {
            let attacks = table.get_attacks(40, PieceType::Rook, 0);
            black_box(attacks);
        });
    });

    // Full table access (all squares)
    group.bench_function("full_table_lazy", |b| {
        b.iter(|| {
            let table = LazyMagicTable::new().unwrap();
            // Access all squares
            for square in 0..81 {
                let _ = table.get_attacks(square, PieceType::Rook, 0);
                let _ = table.get_attacks(square, PieceType::Bishop, 0);
            }
            black_box(table);
        });
    });

    group.finish();
}

fn benchmark_progress_reporting_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("progress_reporting");

    // Without progress callback
    group.bench_function("no_callback", |b| {
        b.iter(|| {
            let table = MagicTable::new().unwrap();
            black_box(table);
        });
    });

    // With progress callback
    group.bench_function("with_callback", |b| {
        b.iter(|| {
            let mut progress_values = Vec::new();
            let table = MagicTable::new_with_progress(Some(move |p| {
                progress_values.push(p);
            })).unwrap();
            black_box(table);
            black_box(progress_values);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sequential_initialization,
    benchmark_parallel_initialization,
    benchmark_sequential_vs_parallel,
    benchmark_lazy_initialization_overhead,
    benchmark_progress_reporting_overhead
);
criterion_main!(benches);

