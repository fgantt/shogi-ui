//! Benchmarks for advanced magic table optimizations
//!
//! These benchmarks measure:
//! - Heuristic improvements (table size reduction)
//! - LookupEngine vs. SimpleLookupEngine performance
//! - Memory-mapped vs. in-memory table performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::magic::{
    lookup_engine::LookupEngine, memory_mapped::MemoryMappedMagicTable, MagicTable,
};
use shogi_engine::types::{Bitboard, PieceType};
use std::fs;
use std::time::Instant;

fn benchmark_heuristic_improvements(c: &mut Criterion) {
    let mut group = c.benchmark_group("heuristic_improvements");

    // Benchmark magic number generation with improved heuristics
    group.bench_function("find_magic_with_improved_heuristics", |b| {
        use shogi_engine::bitboards::magic::magic_finder::MagicFinder;
        b.iter(|| {
            let mut finder = MagicFinder::new();
            let result = finder.find_magic_number(40, PieceType::Rook);
            black_box(result);
        });
    });

    group.finish();
}

fn benchmark_lookup_engine_vs_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_engine_comparison");

    let table = MagicTable::new().unwrap();
    let engine = LookupEngine::new(table.clone());

    // Benchmark LookupEngine with caching
    group.bench_function("lookup_engine_cached", |b| {
        b.iter(|| {
            for square in (0..81).step_by(5) {
                for piece_type in [PieceType::Rook, PieceType::Bishop] {
                    let occupied = (square as u128) * 0x1234567890ABCDEF;
                    let _ = engine.get_attacks(square, piece_type, occupied);
                }
            }
        });
    });

    // Benchmark direct table lookup (SimpleLookupEngine equivalent)
    group.bench_function("direct_table_lookup", |b| {
        b.iter(|| {
            for square in (0..81).step_by(5) {
                for piece_type in [PieceType::Rook, PieceType::Bishop] {
                    let occupied = (square as u128) * 0x1234567890ABCDEF;
                    let _ = table.get_attacks(square, piece_type, occupied);
                }
            }
        });
    });

    group.finish();

    // Print comparison
    let test_squares: Vec<u8> = (0..81).step_by(5).collect();
    let iterations = 100;

    // LookupEngine (with warmup)
    for _ in 0..10 {
        for &square in &test_squares {
            let _ = engine.get_attacks(square, PieceType::Rook, 0);
        }
    }

    let engine_start = Instant::now();
    for _ in 0..iterations {
        for &square in &test_squares {
            let _ = engine.get_attacks(square, PieceType::Rook, 0);
        }
    }
    let engine_time = engine_start.elapsed();

    // Direct lookup
    let direct_start = Instant::now();
    for _ in 0..iterations {
        for &square in &test_squares {
            let _ = table.get_attacks(square, PieceType::Rook, 0);
        }
    }
    let direct_time = direct_start.elapsed();

    println!("\n=== LookupEngine vs. Direct Lookup ===");
    println!("LookupEngine (cached): {:?}", engine_time);
    println!("Direct lookup: {:?}", direct_time);
    if direct_time.as_millis() > 0 {
        let speedup = direct_time.as_secs_f64() / engine_time.as_secs_f64();
        println!("Speedup: {:.2}x", speedup);
    }

    let metrics = engine.get_metrics();
    println!("Cache hits: {}", metrics.cache_hits);
    println!("Cache misses: {}", metrics.cache_misses);
    if metrics.cache_hits + metrics.cache_misses > 0 {
        let hit_rate = metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64;
        println!("Hit rate: {:.1}%", hit_rate * 100.0);
    }
}

fn benchmark_memory_mapped_vs_in_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_mapped");

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_mmap_magic_table.bin");

    // Setup: Generate and save table
    let table = MagicTable::new().unwrap();
    table.save_to_file(&test_file).unwrap();

    // Benchmark in-memory table
    group.bench_function("in_memory", |b| {
        b.iter(|| {
            for square in (0..81).step_by(10) {
                let _ = table.get_attacks(square, PieceType::Rook, 0);
            }
        });
    });

    // Benchmark memory-mapped table
    group.bench_function("memory_mapped", |b| {
        let mmap_table = MemoryMappedMagicTable::from_file(&test_file).unwrap();
        b.iter(|| {
            for square in (0..81).step_by(10) {
                let _ = mmap_table.get_attacks(square, PieceType::Rook, 0);
            }
        });
    });

    group.finish();

    // Cleanup
    let _ = fs::remove_file(&test_file);
}

criterion_group!(
    benches,
    benchmark_heuristic_improvements,
    benchmark_lookup_engine_vs_simple,
    benchmark_memory_mapped_vs_in_memory
);
criterion_main!(benches);

