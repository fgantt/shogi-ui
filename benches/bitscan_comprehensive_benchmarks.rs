//! Comprehensive performance benchmarks for bit-scanning optimization system
//!
//! This module provides detailed performance benchmarks for all bit-scanning
//! optimization functions, including comparative benchmarks, regression tests,
//! and performance validation.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use shogi_engine::bitboards::*;
use shogi_engine::types::Bitboard;

/// Comprehensive population count benchmarks
fn benchmark_popcount_comprehensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("popcount_comprehensive");

    // Generate test data with different patterns
    let test_data = generate_benchmark_test_data();

    // Benchmark standard implementation
    group.bench_function("standard", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount(bb));
            }
        })
    });

    // Benchmark cache-optimized implementation
    group.bench_function("cache_optimized", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_cache_optimized(bb));
            }
        })
    });

    // Benchmark branch-optimized implementation
    group.bench_function("branch_optimized", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_branch_optimized(bb));
            }
        })
    });

    // Benchmark critical path implementation
    group.bench_function("critical_path", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_critical(bb));
            }
        })
    });

    // Benchmark native implementation for comparison
    group.bench_function("native", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bb.count_ones());
            }
        })
    });

    group.finish();
}

/// Comprehensive bit scanning benchmarks
fn benchmark_bitscan_comprehensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitscan_comprehensive");

    let test_data = generate_benchmark_test_data();

    // Benchmark bit scan forward
    group.bench_function("bit_scan_forward_standard", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_forward(bb));
            }
        })
    });

    group.bench_function("bit_scan_forward_optimized", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_forward_optimized(bb));
            }
        })
    });

    group.bench_function("bit_scan_forward_critical", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_forward_critical(bb));
            }
        })
    });

    group.bench_function("bit_scan_forward_native", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(if bb == 0 {
                    None
                } else {
                    Some(bb.trailing_zeros() as u8)
                });
            }
        })
    });

    // Benchmark bit scan reverse
    group.bench_function("bit_scan_reverse_standard", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_reverse(bb));
            }
        })
    });

    group.bench_function("bit_scan_reverse_optimized", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_reverse_optimized(bb));
            }
        })
    });

    group.bench_function("bit_scan_reverse_native", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(if bb == 0 {
                    None
                } else {
                    Some((127 - bb.leading_zeros()) as u8)
                });
            }
        })
    });

    group.finish();
}

/// Common case optimization benchmarks
fn benchmark_common_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("common_cases");

    // Generate data with common Shogi patterns
    let empty_boards = vec![0u128; 1000];
    let single_pieces = (0..1000).map(|i| 1u128 << (i % 81)).collect::<Vec<_>>();
    let mixed_patterns = generate_shogi_patterns(1000);

    // Benchmark empty board detection
    group.bench_function("is_empty_optimized", |b| {
        b.iter(|| {
            for &bb in &empty_boards {
                black_box(is_empty_optimized(bb));
            }
        })
    });

    group.bench_function("is_empty_standard", |b| {
        b.iter(|| {
            for &bb in &empty_boards {
                black_box(bb == 0);
            }
        })
    });

    // Benchmark single piece detection
    group.bench_function("is_single_piece_optimized", |b| {
        b.iter(|| {
            for &bb in &single_pieces {
                black_box(is_single_piece_optimized(bb));
            }
        })
    });

    group.bench_function("is_single_piece_standard", |b| {
        b.iter(|| {
            for &bb in &single_pieces {
                black_box(bb != 0 && (bb & (bb - 1) == 0));
            }
        })
    });

    // Benchmark mixed patterns
    group.bench_function("mixed_patterns_optimized", |b| {
        b.iter(|| {
            for &bb in &mixed_patterns {
                black_box(is_empty_optimized(bb));
                black_box(is_single_piece_optimized(bb));
                black_box(is_multiple_pieces_optimized(bb));
            }
        })
    });

    group.bench_function("mixed_patterns_standard", |b| {
        b.iter(|| {
            for &bb in &mixed_patterns {
                black_box(bb == 0);
                black_box(bb != 0 && (bb & (bb - 1) == 0));
                black_box(bb != 0 && (bb & (bb - 1) != 0));
            }
        })
    });

    group.finish();
}

/// Cache optimization benchmarks
fn benchmark_cache_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_optimization");

    let test_data = generate_benchmark_test_data();

    // Benchmark cache-aligned lookup tables
    group.bench_function("cache_aligned_popcount", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_cache_optimized(bb));
            }
        })
    });

    group.bench_function("cache_aligned_bit_positions", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(get_bit_positions_cache_optimized(bb));
            }
        })
    });

    // Benchmark prefetching
    group.bench_function("with_prefetching", |b| {
        b.iter(|| unsafe {
            for &bb in &test_data {
                prefetch_bitboard(bb);
                black_box(popcount_cache_optimized(bb));
            }
        })
    });

    group.bench_function("without_prefetching", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_cache_optimized(bb));
            }
        })
    });

    // Benchmark batch processing with prefetching
    group.bench_function("batch_processing_with_prefetch", |b| {
        b.iter(|| unsafe {
            black_box(process_bitboard_sequence(&test_data));
        })
    });

    group.finish();
}

/// Branch prediction optimization benchmarks
fn benchmark_branch_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_prediction");

    // Generate data with different branch prediction patterns
    let sparse_data = generate_sparse_data(1000);
    let dense_data = generate_dense_data(1000);
    let mixed_data = generate_mixed_data(1000);

    // Benchmark sparse data (good branch prediction)
    group.bench_function("sparse_branch_optimized", |b| {
        b.iter(|| {
            for &bb in &sparse_data {
                black_box(popcount_branch_optimized(bb));
                black_box(bit_scan_forward_optimized(bb));
            }
        })
    });

    group.bench_function("sparse_standard", |b| {
        b.iter(|| {
            for &bb in &sparse_data {
                black_box(bb.count_ones());
                black_box(if bb == 0 {
                    None
                } else {
                    Some(bb.trailing_zeros() as u8)
                });
            }
        })
    });

    // Benchmark dense data (poor branch prediction)
    group.bench_function("dense_branch_optimized", |b| {
        b.iter(|| {
            for &bb in &dense_data {
                black_box(popcount_branch_optimized(bb));
                black_box(bit_scan_forward_optimized(bb));
            }
        })
    });

    group.bench_function("dense_standard", |b| {
        b.iter(|| {
            for &bb in &dense_data {
                black_box(bb.count_ones());
                black_box(if bb == 0 {
                    None
                } else {
                    Some(bb.trailing_zeros() as u8)
                });
            }
        })
    });

    // Benchmark mixed data (realistic patterns)
    group.bench_function("mixed_branch_optimized", |b| {
        b.iter(|| {
            for &bb in &mixed_data {
                black_box(popcount_branch_optimized(bb));
                black_box(bit_scan_forward_optimized(bb));
                black_box(is_empty_optimized(bb));
                black_box(is_single_piece_optimized(bb));
            }
        })
    });

    group.bench_function("mixed_standard", |b| {
        b.iter(|| {
            for &bb in &mixed_data {
                black_box(bb.count_ones());
                black_box(if bb == 0 {
                    None
                } else {
                    Some(bb.trailing_zeros() as u8)
                });
                black_box(bb == 0);
                black_box(bb != 0 && (bb & (bb - 1) == 0));
            }
        })
    });

    group.finish();
}

/// API integration benchmarks
fn benchmark_api_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_integration");

    let test_data = generate_benchmark_test_data();

    // Benchmark API module vs direct access
    group.bench_function("api_module", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(api::bitscan::popcount(bb));
                black_box(api::bitscan::bit_scan_forward(bb));
                black_box(api::utils::extract_lsb(bb));
            }
        })
    });

    group.bench_function("direct_access", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount(bb));
                black_box(bit_scan_forward(bb));
                black_box(extract_lsb(bb));
            }
        })
    });

    // Benchmark GlobalOptimizer vs direct functions
    group.bench_function("global_optimizer", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(integration::GlobalOptimizer::popcount(bb));
                black_box(integration::GlobalOptimizer::bit_scan_forward(bb));
                black_box(integration::GlobalOptimizer::get_all_bit_positions(bb));
            }
        })
    });

    group.bench_function("direct_functions", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount(bb));
                black_box(bit_scan_forward(bb));
                black_box(get_all_bit_positions(bb));
            }
        })
    });

    group.finish();
}

/// Bit iterator benchmarks
fn benchmark_bit_iterator(c: &mut Criterion) {
    let mut group = c.benchmark_group("bit_iterator");

    let test_data = generate_benchmark_test_data();

    // Benchmark bit iterator vs direct enumeration
    group.bench_function("bit_iterator", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let positions: Vec<u8> = bits(bb).collect();
                black_box(positions);
            }
        })
    });

    group.bench_function("direct_enumeration", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let positions = get_all_bit_positions(bb);
                black_box(positions);
            }
        })
    });

    // Benchmark reverse iteration
    group.bench_function("reverse_iterator", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let positions: Vec<u8> = bb.bits_rev().collect();
                black_box(positions);
            }
        })
    });

    group.bench_function("reverse_direct", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let mut positions = get_all_bit_positions(bb);
                positions.reverse();
                black_box(positions);
            }
        })
    });

    group.finish();
}

/// Square coordinate conversion benchmarks
fn benchmark_square_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_conversion");

    let test_positions = (0..81).collect::<Vec<u8>>();

    // Benchmark coordinate conversion
    group.bench_function("bit_to_square", |b| {
        b.iter(|| {
            for &pos in &test_positions {
                black_box(bit_to_square(pos));
            }
        })
    });

    group.bench_function("square_to_bit", |b| {
        b.iter(|| {
            for &pos in &test_positions {
                let square = bit_to_square(pos);
                black_box(square_to_bit(square));
            }
        })
    });

    // Benchmark algebraic notation
    group.bench_function("bit_to_square_name", |b| {
        b.iter(|| {
            for &pos in &test_positions {
                black_box(bit_to_square_name(pos));
            }
        })
    });

    group.bench_function("square_name_to_bit", |b| {
        b.iter(|| {
            for &pos in &test_positions {
                let name = bit_to_square_name(pos);
                black_box(square_name_to_bit(&name));
            }
        })
    });

    group.finish();
}

/// Performance regression benchmarks
fn benchmark_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");

    let test_data = generate_benchmark_test_data();

    // Benchmark critical path functions for regression testing
    group.bench_function("critical_path_popcount", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(popcount_critical(bb));
            }
        })
    });

    group.bench_function("critical_path_bitscan", |b| {
        b.iter(|| {
            for &bb in &test_data {
                black_box(bit_scan_forward_critical(bb));
            }
        })
    });

    // Benchmark with different data sizes
    for size in [100, 500, 1000, 2000].iter() {
        let data = generate_benchmark_test_data_size(*size);
        group.bench_with_input(BenchmarkId::new("variable_size", size), size, |b, &size| {
            b.iter(|| {
                for &bb in &data {
                    black_box(popcount_critical(bb));
                }
            })
        });
    }

    group.finish();
}

/// Cross-platform consistency benchmarks
fn benchmark_cross_platform_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_platform_consistency");

    let test_data = generate_benchmark_test_data();

    // Benchmark all implementations to ensure consistency
    group.bench_function("all_popcount_implementations", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let standard = popcount(bb);
                let cache = popcount_cache_optimized(bb);
                let branch = popcount_branch_optimized(bb);
                let critical = popcount_critical(bb);

                // Verify consistency
                assert_eq!(standard, cache);
                assert_eq!(standard, branch);
                assert_eq!(standard, critical);

                black_box(standard);
            }
        })
    });

    group.bench_function("all_bitscan_implementations", |b| {
        b.iter(|| {
            for &bb in &test_data {
                let standard_forward = bit_scan_forward(bb);
                let branch_forward = bit_scan_forward_optimized(bb);
                let critical_forward = bit_scan_forward_critical(bb);

                // Verify consistency
                assert_eq!(standard_forward, branch_forward);
                assert_eq!(standard_forward, critical_forward);

                black_box(standard_forward);
            }
        })
    });

    group.finish();
}

/// Helper functions for benchmark data generation
fn generate_benchmark_test_data() -> Vec<Bitboard> {
    let mut data = Vec::new();

    // Edge cases
    data.extend(vec![
        0u128,                     // Empty
        1u128,                     // Single bit
        1u128 << 63,               // 64-bit boundary
        1u128 << 127,              // Maximum position
        0xFFFF_FFFF_FFFF_FFFFu128, // Full 64-bit
        !0u128,                    // All bits
    ]);

    // Common patterns
    for i in 0..128 {
        data.push(1u128 << i);
    }

    for i in 0..127 {
        data.push((1u128 << i) | (1u128 << (i + 1)));
    }

    // Random patterns
    let mut rng = rand::thread_rng();
    for _ in 0..500 {
        data.push(rng.gen::<u128>());
    }

    data
}

fn generate_benchmark_test_data_size(size: usize) -> Vec<Bitboard> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        data.push(rng.gen::<u128>());
    }

    data
}

fn generate_sparse_data(size: usize) -> Vec<Bitboard> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        let mut bb = 0u128;
        // Add 1-3 bits randomly
        let num_bits = rng.gen_range(1..=3);
        for _ in 0..num_bits {
            bb |= 1u128 << rng.gen_range(0..128);
        }
        data.push(bb);
    }

    data
}

fn generate_dense_data(size: usize) -> Vec<Bitboard> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        let mut bb = 0u128;
        // Add 50-80 bits randomly
        let num_bits = rng.gen_range(50..=80);
        for _ in 0..num_bits {
            bb |= 1u128 << rng.gen_range(0..128);
        }
        data.push(bb);
    }

    data
}

fn generate_mixed_data(size: usize) -> Vec<Bitboard> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        match rng.gen_range(0..3) {
            0 => data.push(0u128),                          // Empty
            1 => data.push(1u128 << rng.gen_range(0..128)), // Single bit
            _ => {
                let mut bb = 0u128;
                let num_bits = rng.gen_range(2..=10);
                for _ in 0..num_bits {
                    bb |= 1u128 << rng.gen_range(0..128);
                }
                data.push(bb);
            }
        }
    }

    data
}

fn generate_shogi_patterns(size: usize) -> Vec<Bitboard> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        match rng.gen_range(0..4) {
            0 => data.push(0u128),                         // Empty board
            1 => data.push(1u128 << rng.gen_range(0..81)), // Single piece
            2 => {
                // Two pieces
                let pos1 = rng.gen_range(0..81);
                let pos2 = rng.gen_range(0..81);
                data.push((1u128 << pos1) | (1u128 << pos2));
            }
            _ => {
                // Multiple pieces
                let mut bb = 0u128;
                let num_pieces = rng.gen_range(3..=20);
                for _ in 0..num_pieces {
                    bb |= 1u128 << rng.gen_range(0..81);
                }
                data.push(bb);
            }
        }
    }

    data
}

// Configure criterion benchmarks
criterion_group!(
    benches,
    benchmark_popcount_comprehensive,
    benchmark_bitscan_comprehensive,
    benchmark_common_cases,
    benchmark_cache_optimization,
    benchmark_branch_prediction,
    benchmark_api_integration,
    benchmark_bit_iterator,
    benchmark_square_conversion,
    benchmark_performance_regression,
    benchmark_cross_platform_consistency
);

criterion_main!(benches);
