//! Benchmarks for magic table compression
//!
//! These benchmarks measure:
//! - Compression ratio achieved (target: 30-50% memory reduction)
//! - Lookup performance impact of compression (target: <10% slowdown)
//! - Memory usage comparison between compressed and uncompressed tables

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::magic::compressed_table::{CompressedMagicTable, CompressionConfig};
use shogi_engine::types::{Bitboard, MagicTable, PieceType};
use std::time::Instant;

fn benchmark_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratio");

    // Generate a magic table
    let table = MagicTable::new().unwrap();
    let original_size = table.attack_storage.len() * 16; // u128 = 16 bytes

    // Benchmark compression
    group.bench_function("compress_table", |b| {
        b.iter(|| {
            let compressed = CompressedMagicTable::from_table(black_box(table.clone())).unwrap();
            black_box(compressed)
        });
    });

    // Measure compression ratio
    let compressed = CompressedMagicTable::from_table(table.clone()).unwrap();
    let stats = compressed.stats();
    let ratio = stats.compression_ratio;
    let savings = stats.memory_saved;

    group.bench_function("compression_stats", |b| {
        b.iter(|| {
            let stats = compressed.stats();
            black_box((stats.compression_ratio, stats.memory_saved));
        });
    });

    group.finish();

    // Print compression results
    println!("\n=== Compression Results ===");
    println!("Original size: {} bytes ({:.2} MB)", original_size, original_size as f64 / 1_000_000.0);
    println!("Compressed size: {} bytes ({:.2} MB)", stats.compressed_size, stats.compressed_size as f64 / 1_000_000.0);
    println!("Compression ratio: {:.2}x", ratio);
    println!("Memory saved: {} bytes ({:.2} MB)", savings, savings as f64 / 1_000_000.0);
    println!("Memory reduction: {:.1}%", (1.0 - 1.0 / ratio) * 100.0);
}

fn benchmark_lookup_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_performance");

    // Generate tables
    let table = MagicTable::new().unwrap();
    let compressed = CompressedMagicTable::from_table(table.clone()).unwrap();
    let uncompressed = CompressedMagicTable::uncompressed(table.clone());

    // Test positions: various squares and occupied bitboards
    let test_cases = vec![
        (0u8, PieceType::Rook, 0u128),
        (40u8, PieceType::Rook, 0b1010101010101010u128),
        (40u8, PieceType::Bishop, 0b1111000011110000u128),
        (72u8, PieceType::Rook, 0b1111111100000000u128),
        (4u8, PieceType::Bishop, 0b0000111100001111u128),
    ];

    // Benchmark uncompressed lookups
    group.bench_function("uncompressed_lookup", |b| {
        b.iter(|| {
            for &(square, piece_type, occupied) in &test_cases {
                let attacks = uncompressed.get_attacks(square, piece_type, occupied);
                black_box(attacks);
            }
        });
    });

    // Benchmark compressed lookups
    group.bench_function("compressed_lookup", |b| {
        b.iter(|| {
            for &(square, piece_type, occupied) in &test_cases {
                let attacks = compressed.get_attacks(square, piece_type, occupied);
                black_box(attacks);
            }
        });
    });

    // Benchmark base table lookups (for comparison)
    group.bench_function("base_table_lookup", |b| {
        b.iter(|| {
            for &(square, piece_type, occupied) in &test_cases {
                let attacks = table.get_attacks(square, piece_type, occupied);
                black_box(attacks);
            }
        });
    });

    group.finish();

    // Measure actual performance difference
    let iterations = 10000;
    
    // Uncompressed
    let start = Instant::now();
    for _ in 0..iterations {
        for &(square, piece_type, occupied) in &test_cases {
            black_box(uncompressed.get_attacks(square, piece_type, occupied));
        }
    }
    let uncompressed_time = start.elapsed();

    // Compressed
    let start = Instant::now();
    for _ in 0..iterations {
        for &(square, piece_type, occupied) in &test_cases {
            black_box(compressed.get_attacks(square, piece_type, occupied));
        }
    }
    let compressed_time = start.elapsed();

    let slowdown = (compressed_time.as_secs_f64() / uncompressed_time.as_secs_f64() - 1.0) * 100.0;
    
    println!("\n=== Lookup Performance ===");
    println!("Uncompressed: {:?} ({:.2} ns/lookup)", uncompressed_time, uncompressed_time.as_nanos() as f64 / (iterations * test_cases.len()) as f64);
    println!("Compressed: {:?} ({:.2} ns/lookup)", compressed_time, compressed_time.as_nanos() as f64 / (iterations * test_cases.len()) as f64);
    println!("Slowdown: {:.2}%", slowdown);
}

fn benchmark_compression_with_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_config");

    let table = MagicTable::new().unwrap();

    // Test different configurations
    let configs = vec![
        ("default", CompressionConfig::default()),
        ("no_cache", CompressionConfig {
            enable_hot_cache: false,
            ..Default::default()
        }),
        ("large_cache", CompressionConfig {
            cache_size_limit: 5000,
            ..Default::default()
        }),
    ];

    for (name, config) in configs {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &config,
            |b, config| {
                b.iter(|| {
                    let compressed = CompressedMagicTable::from_table_with_config(
                        black_box(table.clone()),
                        *config,
                    ).unwrap();
                    black_box(compressed);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_compression_ratio,
    benchmark_lookup_performance,
    benchmark_compression_with_config
);
criterion_main!(benches);

