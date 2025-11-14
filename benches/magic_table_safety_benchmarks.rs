//! Benchmarks for magic table safety and robustness features
//!
//! These benchmarks measure:
//! - Memory usage with bounded vs. unbounded pattern cache
//! - Performance impact of bounds checking and fallback

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::magic::attack_generator::{AttackGenerator, AttackGeneratorConfig};
use shogi_engine::bitboards::magic::MagicTable;
use shogi_engine::types::{Bitboard, PieceType};

fn benchmark_bounded_vs_unbounded_cache_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_memory_usage");

    // Test with different cache sizes
    for cache_size in [100, 1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("bounded_{}", cache_size)),
            cache_size,
            |b, &size| {
                let config = AttackGeneratorConfig { cache_size: size };
                let mut generator = AttackGenerator::with_config(config);
                
                // Generate many patterns to fill cache
                b.iter(|| {
                    for i in 0..(size * 2) {
                        let blockers = (i as u128) * 0x1234567890ABCDEF;
                        let _ = generator.generate_attack_pattern(40, PieceType::Rook, blockers);
                    }
                    black_box(generator.cache_stats());
                });
            },
        );
    }

    group.finish();
}

fn benchmark_bounds_checking_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("bounds_checking");

    let table = MagicTable::new().unwrap();

    // Benchmark with bounds checking (normal operation)
    group.bench_function("with_bounds_checking", |b| {
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
}

fn benchmark_fallback_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("fallback_performance");

    // Create partially initialized table
    let mut table = MagicTable::default();
    table.initialize_rook_square(40).unwrap();

    // Benchmark fallback (uninitialized square)
    group.bench_function("fallback_ray_casting", |b| {
        b.iter(|| {
            for square in (0..81).step_by(10) {
                if square != 40 {
                    let _ = table.get_attacks(square, PieceType::Rook, 0);
                }
            }
        });
    });

    // Benchmark normal lookup (initialized square)
    group.bench_function("normal_lookup", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = table.get_attacks(40, PieceType::Rook, 0);
            }
        });
    });

    group.finish();
}

fn benchmark_integrity_validation(c: &mut Criterion) {
    c.bench_function("validate_integrity", |b| {
        let table = MagicTable::new().unwrap();
        b.iter(|| {
            let _ = table.validate_integrity();
        });
    });
}

criterion_group!(
    benches,
    benchmark_bounded_vs_unbounded_cache_memory,
    benchmark_bounds_checking_overhead,
    benchmark_fallback_performance,
    benchmark_integrity_validation
);
criterion_main!(benches);

