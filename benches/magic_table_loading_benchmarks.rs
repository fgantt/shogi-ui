//! Benchmarks for magic table loading vs. generation performance
//!
//! These benchmarks measure the performance difference between loading
//! precomputed magic tables from disk versus generating them at runtime.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::types::MagicTable;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn benchmark_magic_table_generation(c: &mut Criterion) {
    c.bench_function("magic_table_generation", |b| {
        b.iter(|| {
            let table = MagicTable::new().unwrap();
            black_box(table);
        });
    });
}

fn benchmark_magic_table_loading(c: &mut Criterion) {
    // Setup: Generate and save a table once
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_magic_table.bin");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Generate and save table once
    let table = MagicTable::new().unwrap();
    table.save_to_file(&test_file).unwrap();
    
    c.bench_function("magic_table_loading", |b| {
        b.iter(|| {
            let table = MagicTable::load_from_file(&test_file).unwrap();
            black_box(table);
        });
    });
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

fn benchmark_load_vs_generation_comparison(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_comparison_magic_table.bin");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Generate and save table once
    println!("Generating table for benchmark comparison...");
    let gen_start = Instant::now();
    let table = MagicTable::new().unwrap();
    let gen_time = gen_start.elapsed();
    table.save_to_file(&test_file).unwrap();
    println!("Generation time: {:?}", gen_time);
    
    // Measure load time
    println!("Loading table for benchmark comparison...");
    let load_start = Instant::now();
    let _loaded = MagicTable::load_from_file(&test_file).unwrap();
    let load_time = load_start.elapsed();
    println!("Load time: {:?}", load_time);
    
    if gen_time.as_millis() > 0 {
        let speedup = gen_time.as_millis() as f64 / load_time.as_millis().max(1) as f64;
        println!("Load is {:.2}x faster than generation", speedup);
    }
    
    c.bench_function("magic_table_generation_full", |b| {
        b.iter(|| {
            let table = MagicTable::new().unwrap();
            black_box(table);
        });
    });
    
    c.bench_function("magic_table_loading_full", |b| {
        b.iter(|| {
            let table = MagicTable::load_from_file(&test_file).unwrap();
            black_box(table);
        });
    });
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

fn benchmark_serialization_performance(c: &mut Criterion) {
    let table = MagicTable::default(); // Use default for faster benchmark
    
    c.bench_function("magic_table_serialize", |b| {
        b.iter(|| {
            let serialized = table.serialize().unwrap();
            black_box(serialized);
        });
    });
}

fn benchmark_deserialization_performance(c: &mut Criterion) {
    let table = MagicTable::default();
    let serialized = table.serialize().unwrap();
    
    c.bench_function("magic_table_deserialize", |b| {
        b.iter(|| {
            let table = MagicTable::deserialize(&serialized).unwrap();
            black_box(table);
        });
    });
}

criterion_group!(
    benches,
    benchmark_magic_table_generation,
    benchmark_magic_table_loading,
    benchmark_load_vs_generation_comparison,
    benchmark_serialization_performance,
    benchmark_deserialization_performance
);
criterion_main!(benches);

