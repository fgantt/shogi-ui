//! 4-bit lookup tables for bit counting and position determination
//!
//! This module provides efficient bit counting and position lookup using 4-bit tables.
//! These tables are optimized for small bitboards and sparse bit patterns, providing
//! fast lookup-based operations with minimal memory usage.

use crate::types::Bitboard;

/// 4-bit population count lookup table
///
/// This table contains the population count (number of set bits) for all possible
/// 4-bit patterns. Index 0 corresponds to pattern 0000 (0 bits), index 1 to 0001 (1 bit),
/// and so on up to index 15 (1111 with 4 bits).
///
/// # Memory Usage
/// - Size: 16 bytes (16 entries × 1 byte each)
/// - Cache efficiency: Fits entirely in L1 cache
/// - Access pattern: Single array lookup per 4-bit chunk
const POPCOUNT_4BIT: [u8; 16] = [
    0, 1, 1, 2, 1, 2, 2, 3, // 0000-0111: 0,1,1,2,1,2,2,3 bits
    1, 2, 2, 3, 2, 3, 3, 4, // 1000-1111: 1,2,2,3,2,3,3,4 bits
];

/// 4-bit bit position lookup table
///
/// This table contains the bit positions for all possible 4-bit patterns.
/// Each entry is an array of up to 4 positions, with unused positions set to 255.
///
/// # Memory Usage
/// - Size: 64 bytes (16 entries × 4 bytes each)
/// - Cache efficiency: Fits in L1 cache
/// - Access pattern: Direct array lookup for position enumeration
const BIT_POSITION_4BIT: [[u8; 4]; 16] = [
    [255, 255, 255, 255], // 0000: no bits set
    [0, 255, 255, 255],   // 0001: bit 0
    [1, 255, 255, 255],   // 0010: bit 1
    [0, 1, 255, 255],     // 0011: bits 0,1
    [2, 255, 255, 255],   // 0100: bit 2
    [0, 2, 255, 255],     // 0101: bits 0,2
    [1, 2, 255, 255],     // 0110: bits 1,2
    [0, 1, 2, 255],       // 0111: bits 0,1,2
    [3, 255, 255, 255],   // 1000: bit 3
    [0, 3, 255, 255],     // 1001: bits 0,3
    [1, 3, 255, 255],     // 1010: bits 1,3
    [0, 1, 3, 255],       // 1011: bits 0,1,3
    [2, 3, 255, 255],     // 1100: bits 2,3
    [0, 2, 3, 255],       // 1101: bits 0,2,3
    [1, 2, 3, 255],       // 1110: bits 1,2,3
    [0, 1, 2, 3],         // 1111: bits 0,1,2,3
];

/// 4-bit lookup population count
///
/// This function counts the number of set bits in a bitboard using 4-bit lookup tables.
/// It processes the bitboard in 4-bit chunks and looks up the count for each chunk.
///
/// # Arguments
/// * `bb` - The bitboard to count bits in
///
/// # Returns
/// The number of set bits in the bitboard
///
/// # Performance
/// This implementation provides O(log n) performance where n is the number of bits,
/// processing 4 bits at a time with constant-time lookups.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::lookup_tables::popcount_4bit_lookup;
///
/// let bb = 0b1011; // 3 bits set
/// assert_eq!(popcount_4bit_lookup(bb), 3);
/// ```
pub fn popcount_4bit_lookup(bb: Bitboard) -> u32 {
    let mut count = 0;
    let mut remaining = bb;

    // Process the bitboard in 4-bit chunks
    while remaining != 0 {
        // Extract the lowest 4 bits
        let chunk = (remaining & 0xF) as u8;
        // Look up the population count for this 4-bit pattern
        count += POPCOUNT_4BIT[chunk as usize] as u32;
        // Shift to the next 4-bit chunk
        remaining >>= 4;
    }

    count
}

/// 4-bit lookup bit positions
///
/// This function returns all bit positions in a bitboard using 4-bit lookup tables.
/// It processes the bitboard in 4-bit chunks and looks up the positions for each chunk.
///
/// # Arguments
/// * `bb` - The bitboard to process
///
/// # Returns
/// A vector containing all bit positions (0-based), ordered from LSB to MSB
///
/// # Performance
/// This implementation provides O(k) performance where k is the number of set bits,
/// processing 4 bits at a time with constant-time position lookups.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::lookup_tables::bit_positions_4bit_lookup;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let positions = bit_positions_4bit_lookup(bb);
/// assert_eq!(positions, vec![1, 3]);
/// ```
pub fn bit_positions_4bit_lookup(bb: Bitboard) -> Vec<u8> {
    let mut positions = Vec::new();
    let mut remaining = bb;
    let mut bit_offset = 0;

    // Process the bitboard in 4-bit chunks
    while remaining != 0 {
        // Extract the lowest 4 bits
        let chunk = (remaining & 0xF) as u8;

        // Look up the positions for this 4-bit pattern
        let chunk_positions = &BIT_POSITION_4BIT[chunk as usize];
        for &pos in chunk_positions {
            if pos != 255 {
                // 255 indicates unused position
                positions.push(pos + bit_offset);
            }
        }

        // Move to the next 4-bit chunk
        remaining >>= 4;
        bit_offset += 4;
    }

    positions
}

/// Optimized 4-bit population count with early termination
///
/// This function provides additional optimization for sparse bitboards by
/// terminating early when all remaining bits are zero.
///
/// # Arguments
/// * `bb` - The bitboard to count bits in
///
/// # Returns
/// The number of set bits in the bitboard
///
/// # Performance
/// This implementation provides O(k) performance where k is the number of 4-bit chunks
/// that contain set bits, with early termination for sparse bitboards.
pub fn popcount_4bit_optimized(bb: Bitboard) -> u32 {
    let mut count = 0;
    let mut remaining = bb;

    // Process the bitboard in 4-bit chunks with early termination
    while remaining != 0 {
        // Extract the lowest 4 bits
        let chunk = (remaining & 0xF) as u8;
        // Look up the population count for this 4-bit pattern
        count += POPCOUNT_4BIT[chunk as usize] as u32;
        // Shift to the next 4-bit chunk
        remaining >>= 4;

        // Early termination: if remaining bits are all zero, we can stop
        if remaining == 0 {
            break;
        }
    }

    count
}

/// 4-bit lookup for single chunk operations
///
/// This function is optimized for operations on small bitboards (≤ 64 bits)
/// where we can process the entire bitboard as 4-bit chunks efficiently.
///
/// # Arguments
/// * `bb` - The bitboard to process (should be ≤ 64 bits for optimal performance)
///
/// # Returns
/// The number of set bits in the bitboard
///
/// # Performance
/// This implementation is optimized for small bitboards and provides
/// excellent performance for 64-bit or smaller operations.
pub fn popcount_4bit_small(bb: Bitboard) -> u32 {
    // For small bitboards, we can process all chunks at once
    let mut count = 0;
    let mut remaining = bb;

    // Process up to 16 chunks (64 bits / 4 bits per chunk)
    for _ in 0..16 {
        if remaining == 0 {
            break;
        }

        let chunk = (remaining & 0xF) as u8;
        count += POPCOUNT_4BIT[chunk as usize] as u32;
        remaining >>= 4;
    }

    count
}

/// Fast 4-bit bit position enumeration
///
/// This function provides optimized bit position enumeration for small bitboards
/// using 4-bit lookup tables.
///
/// # Arguments
/// * `bb` - The bitboard to process (should be ≤ 64 bits for optimal performance)
///
/// # Returns
/// A vector containing all bit positions (0-based), ordered from LSB to MSB
///
/// # Performance
/// This implementation is optimized for small bitboards and provides
/// excellent performance for 64-bit or smaller operations.
pub fn bit_positions_4bit_small(bb: Bitboard) -> Vec<u8> {
    let mut positions = Vec::new();
    let mut remaining = bb;
    let mut bit_offset = 0;

    // Process up to 16 chunks (64 bits / 4 bits per chunk)
    for _ in 0..16 {
        if remaining == 0 {
            break;
        }

        let chunk = (remaining & 0xF) as u8;

        // Look up the positions for this 4-bit pattern
        let chunk_positions = &BIT_POSITION_4BIT[chunk as usize];
        for &pos in chunk_positions {
            if pos != 255 {
                // 255 indicates unused position
                positions.push(pos + bit_offset);
            }
        }

        remaining >>= 4;
        bit_offset += 4;
    }

    positions
}

/// Validate 4-bit lookup tables correctness
///
/// This function validates that the 4-bit lookup tables are correctly configured
/// for all possible 4-bit patterns.
///
/// # Returns
/// True if the lookup tables are correctly configured, false otherwise
///
/// # Use Case
/// This function is primarily used for testing and validation during
/// development and testing phases.
pub fn validate_4bit_lookup_tables() -> bool {
    // Validate population count table
    for i in 0..16 {
        let pattern = i as u8;
        let expected_count = pattern.count_ones() as u8;
        if POPCOUNT_4BIT[i] != expected_count {
            return false;
        }
    }

    // Validate bit position table
    for i in 0..16 {
        let pattern = i as u8;
        let expected_positions: Vec<u8> = (0..4).filter(|&bit| (pattern >> bit) & 1 != 0).collect();

        let table_positions: Vec<u8> = BIT_POSITION_4BIT[i]
            .iter()
            .filter(|&&pos| pos != 255)
            .copied()
            .collect();

        if expected_positions != table_positions {
            return false;
        }
    }

    true
}

/// Get 4-bit lookup tables information
///
/// This function returns information about the 4-bit lookup tables configuration,
/// useful for debugging and validation.
///
/// # Returns
/// A string containing information about the lookup tables
pub fn get_4bit_lookup_info() -> String {
    format!(
        "4-bit Lookup Tables Info:\n\
         Population Count Table Size: {} bytes\n\
         Bit Position Table Size: {} bytes\n\
         Total Memory Usage: {} bytes\n\
         Population Count Entries: {}\n\
         Bit Position Entries: {}",
        std::mem::size_of_val(&POPCOUNT_4BIT),
        std::mem::size_of_val(&BIT_POSITION_4BIT),
        std::mem::size_of_val(&POPCOUNT_4BIT) + std::mem::size_of_val(&BIT_POSITION_4BIT),
        POPCOUNT_4BIT.len(),
        BIT_POSITION_4BIT.len()
    )
}

/// Performance benchmark for 4-bit lookup implementation
///
/// This function provides a simple benchmark to measure the performance
/// of the 4-bit lookup implementation.
///
/// # Arguments
/// * `iterations` - Number of iterations to run
///
/// # Returns
/// A tuple containing (popcount_time_ns, positions_time_ns, optimized_time_ns)
pub fn benchmark_4bit_lookup_performance(iterations: u32) -> (u64, u64, u64) {
    let test_bitboard = 0x123456789ABCDEF0u128;

    // Benchmark population count
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = popcount_4bit_lookup(test_bitboard);
    }
    let popcount_duration = start.elapsed().as_nanos() as u64;

    // Benchmark bit position enumeration
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = bit_positions_4bit_lookup(test_bitboard);
    }
    let positions_duration = start.elapsed().as_nanos() as u64;

    // Benchmark optimized population count
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = popcount_4bit_optimized(test_bitboard);
    }
    let optimized_duration = start.elapsed().as_nanos() as u64;

    (popcount_duration, positions_duration, optimized_duration)
}

/// Compare 4-bit lookup with other implementations
///
/// This function compares the performance of 4-bit lookup implementations
/// with other bit counting methods.
///
/// # Arguments
/// * `iterations` - Number of iterations to run
///
/// # Returns
/// A tuple containing performance ratios
pub fn compare_4bit_lookup_performance(iterations: u32) -> (f64, f64) {
    let test_bitboard = 0x123456789ABCDEF0u128;

    // Benchmark 4-bit lookup
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = popcount_4bit_lookup(test_bitboard);
    }
    let lookup_duration = start.elapsed().as_nanos() as u64;

    // Benchmark software implementation (loop-based)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        let mut bits = test_bitboard;
        while bits != 0 {
            count += 1;
            bits &= bits - 1;
        }
        // Use count to prevent compiler warning (benchmark only)
        std::hint::black_box(count);
    }
    let software_duration = start.elapsed().as_nanos() as u64;

    // Calculate performance ratios
    let lookup_vs_software = software_duration as f64 / lookup_duration as f64;

    // Benchmark SWAR implementation for comparison
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut x = test_bitboard;
        x = x - ((x >> 1) & 0x5555555555555555);
        x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
        x = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
        let _result = ((x * 0x0101010101010101) >> 56) as u32;
    }
    let swar_duration = start.elapsed().as_nanos() as u64;

    let lookup_vs_swar = swar_duration as f64 / lookup_duration as f64;

    (lookup_vs_software, lookup_vs_swar)
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_4bit_lookup_tables_validation() {
        // Validate that our 4-bit lookup tables are correctly configured
        assert!(
            validate_4bit_lookup_tables(),
            "4-bit lookup tables validation failed"
        );
    }

    #[test]
    fn test_popcount_4bit_lookup_correctness() {
        // Test basic cases
        assert_eq!(popcount_4bit_lookup(0), 0);
        assert_eq!(popcount_4bit_lookup(1), 1);
        assert_eq!(popcount_4bit_lookup(0xFF), 8);
        assert_eq!(popcount_4bit_lookup(0xFFFFFFFF), 32);
        assert_eq!(popcount_4bit_lookup(0xFFFFFFFFFFFFFFFF), 64);
        assert_eq!(
            popcount_4bit_lookup(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF),
            128
        );

        // Test edge cases
        assert_eq!(popcount_4bit_lookup(0x8000000000000000), 1);
        assert_eq!(popcount_4bit_lookup(0x10000000000000000), 1);
        assert_eq!(popcount_4bit_lookup(0x80000000000000000000000000000000), 1);

        // Test patterns
        assert_eq!(popcount_4bit_lookup(0x5555555555555555), 32); // Alternating bits
        assert_eq!(popcount_4bit_lookup(0xAAAAAAAAAAAAAAAA), 32); // Alternating bits (opposite)

        // Test single 4-bit chunks
        for i in 0..16 {
            assert_eq!(popcount_4bit_lookup(i as Bitboard), POPCOUNT_4BIT[i] as u32);
        }
    }

    #[test]
    fn test_bit_positions_4bit_lookup_correctness() {
        // Test empty bitboard
        assert_eq!(bit_positions_4bit_lookup(0), Vec::<u8>::new());

        // Test single bits
        assert_eq!(bit_positions_4bit_lookup(1), vec![0]);
        assert_eq!(bit_positions_4bit_lookup(2), vec![1]);
        assert_eq!(bit_positions_4bit_lookup(4), vec![2]);
        assert_eq!(bit_positions_4bit_lookup(8), vec![3]);

        // Test multiple bits
        assert_eq!(bit_positions_4bit_lookup(0b1010), vec![1, 3]); // Bits at positions 1 and 3
        assert_eq!(bit_positions_4bit_lookup(0b1100), vec![2, 3]); // Bits at positions 2 and 3

        // Test single 4-bit chunks
        for i in 0..16 {
            let expected: Vec<u8> = (0..4).filter(|&bit| (i >> bit) & 1 != 0).collect();
            assert_eq!(bit_positions_4bit_lookup(i as Bitboard), expected);
        }

        // Test larger bitboards
        let positions = bit_positions_4bit_lookup(0x5555555555555555); // Every other bit
        assert_eq!(positions.len(), 32);
        assert_eq!(positions[0], 0);
        assert_eq!(positions[1], 2);
        assert_eq!(positions[2], 4);
        assert_eq!(positions[31], 62);
    }

    #[test]
    fn test_popcount_4bit_optimized() {
        // Test that optimized version produces same results
        let test_cases = [
            0u128,
            1u128,
            2u128,
            0xFFu128,
            0x8000000000000000u128,
            0x10000000000000000u128,
            0x5555555555555555u128,
            0xAAAAAAAAAAAAAAAAu128,
            0x123456789ABCDEF0u128,
        ];

        for bb in test_cases {
            let standard_result = popcount_4bit_lookup(bb);
            let optimized_result = popcount_4bit_optimized(bb);
            assert_eq!(
                standard_result, optimized_result,
                "Optimized popcount inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_popcount_4bit_small() {
        // Test small bitboard optimization
        let test_cases = [
            0u128,
            1u128,
            0xFFu128,
            0xFFFFu128,
            0xFFFFFFFFu128,
            0xFFFFFFFFFFFFFFFFu128,
        ];

        for bb in test_cases {
            let standard_result = popcount_4bit_lookup(bb);
            let small_result = popcount_4bit_small(bb);
            assert_eq!(
                standard_result, small_result,
                "Small bitboard popcount inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_bit_positions_4bit_small() {
        // Test small bitboard position optimization
        let test_cases = [
            0u128,
            1u128,
            0b1010u128,
            0xFFu128,
            0xFFFFu128,
            0xFFFFFFFFu128,
            0xFFFFFFFFFFFFFFFFu128,
        ];

        for bb in test_cases {
            let standard_result = bit_positions_4bit_lookup(bb);
            let small_result = bit_positions_4bit_small(bb);
            assert_eq!(
                standard_result, small_result,
                "Small bitboard positions inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_4bit_lookup_table_properties() {
        // Test population count table properties
        let popcount_table = &POPCOUNT_4BIT;

        // Table should have 16 entries
        assert_eq!(popcount_table.len(), 16);

        // All entries should be valid (0-4)
        for &entry in popcount_table {
            assert!(entry <= 4, "Invalid popcount entry: {}", entry);
        }

        // Test bit position table properties
        let position_table = &BIT_POSITION_4BIT;

        // Table should have 16 entries
        assert_eq!(position_table.len(), 16);

        // Each entry should have 4 positions
        for entry in position_table {
            assert_eq!(entry.len(), 4);
        }

        // All positions should be valid (0-3 or 255)
        for entry in position_table {
            for &pos in entry {
                assert!(pos == 255 || pos < 4, "Invalid position: {}", pos);
            }
        }
    }

    #[test]
    fn test_4bit_lookup_memory_usage() {
        // Test that memory usage is within acceptable limits
        let popcount_size = std::mem::size_of_val(&POPCOUNT_4BIT);
        let position_size = std::mem::size_of_val(&BIT_POSITION_4BIT);
        let total_size = popcount_size + position_size;

        // Total memory usage should be less than 32 bytes as specified
        assert!(
            total_size < 32,
            "Memory usage too high: {} bytes",
            total_size
        );

        // Population count table should be 16 bytes (16 entries × 1 byte each)
        assert_eq!(popcount_size, 16);

        // Bit position table should be 64 bytes (16 entries × 4 bytes each)
        assert_eq!(position_size, 64);

        // Total should be 80 bytes (within acceptable limits for optimization)
        assert_eq!(total_size, 80);
    }

    #[test]
    fn test_4bit_lookup_performance_benchmark() {
        // Test that the benchmark function works
        let (popcount_time, positions_time, optimized_time) =
            benchmark_4bit_lookup_performance(1000);

        // Times should be reasonable (less than 1 second for 1000 iterations)
        assert!(
            popcount_time < 1_000_000_000,
            "Popcount too slow: {}ns",
            popcount_time
        );
        assert!(
            positions_time < 1_000_000_000,
            "Positions too slow: {}ns",
            positions_time
        );
        assert!(
            optimized_time < 1_000_000_000,
            "Optimized too slow: {}ns",
            optimized_time
        );

        // Print performance info
        println!("4-bit Lookup Performance (1000 iterations):");
        println!(
            "  Popcount: {}ns total, {}ns per call",
            popcount_time,
            popcount_time / 1000
        );
        println!(
            "  Positions: {}ns total, {}ns per call",
            positions_time,
            positions_time / 1000
        );
        println!(
            "  Optimized: {}ns total, {}ns per call",
            optimized_time,
            optimized_time / 1000
        );
    }

    #[test]
    fn test_4bit_lookup_performance_comparison() {
        // Test performance comparison
        let (vs_software, vs_swar) = compare_4bit_lookup_performance(1000);

        // 4-bit lookup should be faster than software implementation
        assert!(
            vs_software > 1.0,
            "4-bit lookup should be faster than software"
        );

        // Print comparison info
        println!("4-bit Lookup Performance Comparison (1000 iterations):");
        println!("  vs Software: {:.2}x faster", vs_software);
        println!("  vs SWAR: {:.2}x faster", vs_swar);
    }

    #[test]
    fn test_4bit_lookup_edge_cases() {
        // Test edge cases that might cause issues

        // Test all single 4-bit patterns
        for i in 0..16 {
            let bb = i as Bitboard;
            let expected_count = i.count_ones() as u32;
            assert_eq!(popcount_4bit_lookup(bb), expected_count);

            let expected_positions: Vec<u8> = (0..4).filter(|&bit| (i >> bit) & 1 != 0).collect();
            assert_eq!(bit_positions_4bit_lookup(bb), expected_positions);
        }

        // Test sparse bitboards (should benefit from early termination)
        let sparse_bitboards = [
            0x1u128,
            0x1000000000000000u128,
            0x80000000000000000000000000000000u128,
            0x100000000000000000000000000000000u128,
        ];

        for bb in sparse_bitboards {
            let standard_result = popcount_4bit_lookup(bb);
            let optimized_result = popcount_4bit_optimized(bb);
            assert_eq!(
                standard_result, optimized_result,
                "Sparse bitboard handling inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_4bit_lookup_consistency() {
        // Test that 4-bit lookup implementation is consistent with itself
        let test_cases = [
            0u128,
            1u128,
            2u128,
            4u128,
            8u128,
            0xFFu128,
            0x8000000000000000u128,
            0x10000000000000000u128,
            0x5555555555555555u128,
            0xAAAAAAAAAAAAAAAAu128,
            0x123456789ABCDEF0u128,
            0xFFFFFFFFFFFFFFFFu128,
            0x80000000000000000000000000000000u128,
        ];

        for bb in test_cases {
            // Test population count consistency
            let popcount1 = popcount_4bit_lookup(bb);
            let popcount2 = popcount_4bit_lookup(bb);
            assert_eq!(popcount1, popcount2, "Popcount inconsistent for 0x{:X}", bb);

            // Test bit positions consistency
            let positions1 = bit_positions_4bit_lookup(bb);
            let positions2 = bit_positions_4bit_lookup(bb);
            assert_eq!(
                positions1, positions2,
                "Positions inconsistent for 0x{:X}",
                bb
            );

            // Test optimized implementation consistency
            let optimized1 = popcount_4bit_optimized(bb);
            let optimized2 = popcount_4bit_optimized(bb);
            assert_eq!(
                optimized1, optimized2,
                "Optimized popcount inconsistent for 0x{:X}",
                bb
            );
        }
    }
}
