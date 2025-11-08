//! Comprehensive usage examples for bit-scanning optimization system
//!
//! This file demonstrates best practices and common usage patterns for the
//! bit-scanning optimization system in a Shogi engine context.

use shogi_engine::bitboards::*;
use shogi_engine::types::{Bitboard, Player, Position};

/// Basic usage examples for bit-scanning operations
fn basic_usage_examples() {
    println!("=== Basic Usage Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010; // Example bitboard

    // Population count - count the number of set bits
    let count = popcount(bitboard);
    println!("Population count of 0b{:b}: {}", bitboard, count);

    // Bit scan forward - find the position of the least significant bit
    let first_bit = bit_scan_forward(bitboard);
    println!("First bit position: {:?}", first_bit);

    // Bit scan reverse - find the position of the most significant bit
    let last_bit = bit_scan_reverse(bitboard);
    println!("Last bit position: {:?}", last_bit);

    // Get all bit positions
    let positions = get_all_bit_positions(bitboard);
    println!("All bit positions: {:?}", positions);
}

/// Performance-optimized usage examples
fn performance_optimized_examples() {
    println!("\n=== Performance-Optimized Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010;

    // Use critical path functions for maximum performance
    let count = popcount_critical(bitboard);
    let first_pos = bit_scan_forward_critical(bitboard);
    println!("Critical path - Count: {}, First: {:?}", count, first_pos);

    // Use branch-optimized functions for common cases
    let count = popcount_branch_optimized(bitboard);
    let first_pos = bit_scan_forward_optimized(bitboard);
    println!(
        "Branch-optimized - Count: {}, First: {:?}",
        count, first_pos
    );

    // Use cache-optimized functions for memory-intensive operations
    let count = popcount_cache_optimized(bitboard);
    let positions = get_bit_positions_cache_optimized(bitboard);
    println!(
        "Cache-optimized - Count: {}, Positions: {:?}",
        count, positions
    );
}

/// Common case optimization examples
fn common_case_optimization_examples() {
    println!("\n=== Common Case Optimization Examples ===");

    let empty_board: Bitboard = 0;
    let single_piece: Bitboard = 1 << 40; // Piece at center
    let multiple_pieces: Bitboard = (1 << 40) | (1 << 41) | (1 << 42);

    // Optimized empty board detection
    if is_empty_optimized(empty_board) {
        println!("Board is empty (optimized check)");
    }

    // Optimized single piece detection
    if is_single_piece_optimized(single_piece) {
        let pos = single_piece_position_optimized(single_piece);
        println!("Single piece at position: {}", pos);
    }

    // Optimized multiple piece detection
    if is_multiple_pieces_optimized(multiple_pieces) {
        println!("Multiple pieces detected (optimized check)");
    }

    // Non-empty check
    if is_not_empty_optimized(single_piece) {
        println!("Board is not empty (optimized check)");
    }
}

/// Bit manipulation utility examples
fn bit_manipulation_examples() {
    println!("\n=== Bit Manipulation Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010;

    // Extract least significant bit
    let (lsb, remaining) = extract_lsb(bitboard);
    println!("LSB: 0b{:b}, Remaining: 0b{:b}", lsb, remaining);

    // Extract most significant bit
    let (msb, remaining) = extract_msb(bitboard);
    println!("MSB: 0b{:b}, Remaining: 0b{:b}", msb, remaining);

    // Bit operations
    let other: Bitboard = 0b1100_1100_1100_1100;
    println!("Overlaps: {}", overlaps(bitboard, other));
    println!("Is subset: {}", is_subset(lsb, bitboard));

    // Set operations
    println!("Intersection: 0b{:b}", intersection(bitboard, other));
    println!("Union: 0b{:b}", union(bitboard, other));
    println!("Difference: 0b{:b}", difference(bitboard, other));
    println!(
        "Symmetric difference: 0b{:b}",
        symmetric_difference(bitboard, other)
    );
}

/// Bit iterator examples
fn bit_iterator_examples() {
    println!("\n=== Bit Iterator Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010;

    // Forward iteration
    println!("Forward iteration:");
    for (i, pos) in bits(bitboard).enumerate() {
        println!("  {}: position {}", i, pos);
    }

    // Reverse iteration
    println!("Reverse iteration:");
    for (i, pos) in bitboard.bits_rev().enumerate() {
        println!("  {}: position {}", i, pos);
    }

    // Iterator methods
    let first_pos = bits(bitboard).next();
    let last_pos = bits(bitboard).last();
    let count = bits(bitboard).count();
    println!(
        "First: {:?}, Last: {:?}, Count: {}",
        first_pos, last_pos, count
    );

    // Skip and take
    let positions: Vec<u8> = bits(bitboard).skip(1).take(3).collect();
    println!("Skip 1, take 3: {:?}", positions);
}

/// Square coordinate conversion examples
fn square_coordinate_examples() {
    println!("\n=== Square Coordinate Examples ===");

    // Convert between bit positions and squares
    let bit_pos = 40; // Center of 9x9 board
    let square = bit_to_square(bit_pos);
    println!(
        "Bit position {} -> Square ({}, {})",
        bit_pos, square.row, square.col
    );

    let back_to_bit = square_to_bit(square);
    println!(
        "Square ({}, {}) -> Bit position {}",
        square.row, square.col, back_to_bit
    );

    // Algebraic notation
    let square_name = bit_to_square_name(bit_pos);
    println!(
        "Bit position {} -> Algebraic notation: {}",
        bit_pos, square_name
    );

    let from_name = square_name_to_bit(&square_name);
    println!(
        "Algebraic notation {} -> Bit position: {}",
        square_name, from_name
    );

    // Coordinate conversion
    let (file, rank) = bit_to_coords(bit_pos);
    println!("Bit position {} -> File: {}, Rank: {}", bit_pos, file, rank);

    let from_coords = coords_to_bit(file, rank);
    println!(
        "File: {}, Rank: {} -> Bit position: {}",
        file, rank, from_coords
    );
}

/// Shogi-specific examples
fn shogi_specific_examples() {
    println!("\n=== Shogi-Specific Examples ===");

    // Promotion zone detection
    let rank_7_square = 63; // Rank 7 for Black
    let rank_1_square = 0; // Rank 1 for White
    let center_square = 40; // Center square

    println!(
        "Rank 7 is promotion zone for Black: {}",
        is_promotion_zone(rank_7_square, Player::Black)
    );
    println!(
        "Rank 1 is promotion zone for White: {}",
        is_promotion_zone(rank_1_square, Player::White)
    );
    println!(
        "Center is promotion zone: {}",
        is_promotion_zone(center_square, Player::Black)
    );

    // Square validation
    println!("Valid Shogi squares:");
    for i in 0..100 {
        if is_valid_shogi_square(i) {
            println!("  {} is valid", i);
        }
    }

    // Center squares
    let center_squares = get_center_squares();
    println!("Center squares: {:?}", center_squares);

    // Square distance
    let distance = square_distance(0, 80); // Corner to corner
    println!("Distance from 0 to 80: {}", distance);

    // Promotion zone mask
    let promotion_mask = promotion_zone_mask(Player::Black);
    println!("Black promotion zone mask: 0b{:b}", promotion_mask);
}

/// Platform detection and optimization examples
fn platform_optimization_examples() {
    println!("\n=== Platform Optimization Examples ===");

    // Check platform capabilities
    let caps = get_platform_capabilities();
    println!("Platform capabilities:");
    println!("  POPCNT: {}", caps.has_popcnt);
    println!("  BMI1: {}", caps.has_bmi1);
    println!("  BMI2: {}", caps.has_bmi2);
    println!("  Architecture: {:?}", caps.architecture);

    // Get best implementations
    let best_popcount = get_best_popcount_impl();
    let best_bitscan = get_best_bitscan_impl();
    println!("Best popcount implementation: {:?}", best_popcount);
    println!("Best bitscan implementation: {:?}", best_bitscan);

    // Use adaptive selection
    let bitboard: Bitboard = 0b1010_1010_1010_1010;
    let count = popcount(bitboard); // Automatically selects best implementation
    let pos = bit_scan_forward(bitboard); // Automatically selects best implementation
    println!("Adaptive selection - Count: {}, First: {:?}", count, pos);
}

/// Cache optimization examples
fn cache_optimization_examples() {
    println!("\n=== Cache Optimization Examples ===");

    let bitboards = vec![
        0b1010_1010_1010_1010,
        0b1100_1100_1100_1100,
        0b1111_0000_1111_0000,
    ];

    // Batch processing with prefetching
    unsafe {
        let results = process_bitboard_sequence(&bitboards);
        println!("Batch processing results: {:?}", results);
    }

    // Individual prefetching
    for &bb in &bitboards {
        unsafe {
            prefetch_bitboard(bb);
            let count = popcount_cache_optimized(bb);
            println!("Prefetched count: {}", count);
        }
    }

    // Cache-aligned lookup tables
    let cache_aligned_popcount = CacheAlignedPopcountTable::new();
    let cache_aligned_masks = CacheAlignedRankMasks::new();
    println!("Cache-aligned structures created");
}

/// API integration examples
fn api_integration_examples() {
    println!("\n=== API Integration Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010;

    // Use API module functions
    let count = api::bitscan::popcount(bitboard);
    let pos = api::bitscan::bit_scan_forward(bitboard);
    println!("API module - Count: {}, First: {:?}", count, pos);

    // Use utility functions
    let (lsb, remaining) = api::utils::extract_lsb(bitboard);
    println!("API utils - LSB: 0b{:b}, Remaining: 0b{:b}", lsb, remaining);

    // Use square functions
    let square = api::squares::bit_to_square(40);
    println!("API squares - Square: ({}, {})", square.row, square.col);

    // Use backward compatibility functions
    let count = api::compat::count_bits(bitboard);
    let first = api::compat::find_first_bit(bitboard);
    let last = api::compat::find_last_bit(bitboard);
    println!(
        "API compat - Count: {}, First: {:?}, Last: {:?}",
        count, first, last
    );

    // Use analysis functions
    let analysis = api::analysis::analyze_geometry(bitboard);
    println!(
        "API analysis - Ranks: {}, Files: {}, Diagonals: {}",
        analysis.rank_count, analysis.file_count, analysis.diagonal_count
    );
}

/// Global optimizer examples
fn global_optimizer_examples() {
    println!("\n=== Global Optimizer Examples ===");

    let bitboard: Bitboard = 0b1010_1010_1010_1010;

    // Use GlobalOptimizer for automatic optimization
    let count = integration::GlobalOptimizer::popcount(bitboard);
    let pos = integration::GlobalOptimizer::bit_scan_forward(bitboard);
    let positions = integration::GlobalOptimizer::get_all_bit_positions(bitboard);

    println!(
        "GlobalOptimizer - Count: {}, First: {:?}, Positions: {:?}",
        count, pos, positions
    );

    // Geometric analysis
    let geometry = integration::GlobalOptimizer::analyze_geometry(bitboard);
    println!(
        "Geometry analysis - Ranks: {}, Files: {}, Diagonals: {}",
        geometry.rank_count, geometry.file_count, geometry.diagonal_count
    );
}

/// Performance benchmarking examples
fn performance_benchmarking_examples() {
    println!("\n=== Performance Benchmarking Examples ===");

    use std::time::Instant;

    let bitboards = vec![
        0b1010_1010_1010_1010,
        0b1100_1100_1100_1100,
        0b1111_0000_1111_0000,
        0b1010_0101_1010_0101,
        0b1111_1111_0000_0000,
    ];

    let iterations = 1000;

    // Benchmark different implementations
    let start = Instant::now();
    for _ in 0..iterations {
        for &bb in &bitboards {
            let _ = bb.count_ones(); // Standard implementation
        }
    }
    let standard_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        for &bb in &bitboards {
            let _ = popcount_critical(bb); // Critical path implementation
        }
    }
    let optimized_time = start.elapsed();

    println!("Standard implementation: {:?}", standard_time);
    println!("Optimized implementation: {:?}", optimized_time);
    println!(
        "Performance improvement: {:.2}x",
        standard_time.as_nanos() as f64 / optimized_time.as_nanos() as f64
    );
}

/// Error handling and validation examples
fn error_handling_examples() {
    println!("\n=== Error Handling Examples ===");

    // Handle empty bitboards
    let empty: Bitboard = 0;
    match bit_scan_forward(empty) {
        Some(pos) => println!("First bit at position: {}", pos),
        None => println!("No bits set in bitboard"),
    }

    // Validate square coordinates
    for i in 0..100 {
        if is_valid_shogi_square(i) {
            let square = bit_to_square(i);
            println!("Valid square {}: ({}, {})", i, square.row, square.col);
        }
    }

    // Validate bitboard operations
    let bb1: Bitboard = 0b1010;
    let bb2: Bitboard = 0b1100;

    println!("Bitboard 1: 0b{:b}", bb1);
    println!("Bitboard 2: 0b{:b}", bb2);
    println!("Overlaps: {}", overlaps(bb1, bb2));
    println!("Is subset: {}", is_subset(bb1, bb2));
}

/// Main function demonstrating all examples
fn main() {
    println!("Bit-Scanning Optimization System Examples");
    println!("==========================================");

    basic_usage_examples();
    performance_optimized_examples();
    common_case_optimization_examples();
    bit_manipulation_examples();
    bit_iterator_examples();
    square_coordinate_examples();
    shogi_specific_examples();
    platform_optimization_examples();
    cache_optimization_examples();
    api_integration_examples();
    global_optimizer_examples();
    performance_benchmarking_examples();
    error_handling_examples();

    println!("\n=== All Examples Completed ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_examples() {
        basic_usage_examples();
    }

    #[test]
    fn test_performance_examples() {
        performance_optimized_examples();
    }

    #[test]
    fn test_common_case_examples() {
        common_case_optimization_examples();
    }

    #[test]
    fn test_bit_manipulation_examples() {
        bit_manipulation_examples();
    }

    #[test]
    fn test_square_coordinate_examples() {
        square_coordinate_examples();
    }

    #[test]
    fn test_shogi_specific_examples() {
        shogi_specific_examples();
    }
}
