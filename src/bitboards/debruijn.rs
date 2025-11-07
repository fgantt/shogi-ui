//! De Bruijn sequence implementation for bit scanning optimization
//!
//! This module provides efficient bit position determination using De Bruijn sequences.
//! De Bruijn sequences allow O(1) bit position lookup using multiplication and table lookup,
//! making them ideal for bit scanning operations.

use crate::types::Bitboard;

/// De Bruijn sequence for 64-bit bitboards
///
/// This is a carefully chosen De Bruijn sequence that has the property that when
/// multiplied by any power of 2 (isolated bit), the high-order bits contain a
/// unique identifier for the bit position.
const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;

/// Lookup table for bit positions using De Bruijn sequence
///
/// This table maps the high-order bits from the De Bruijn multiplication
/// to the actual bit position. The table is constructed so that:
/// DEBRUIJN_TABLE[((isolated_bit * DEBRUIJN64) >> 58) as usize] = bit_position
///
/// The magic number 58 is chosen because we need the high-order 6 bits
/// (64 - 58 = 6 bits) to index into our 64-entry table.
const DEBRUIJN_TABLE: [u8; 64] = [
    0, 1, 48, 2, 57, 49, 28, 3, 61, 58, 50, 42, 38, 29, 17, 4, 62, 55, 59, 36, 53, 51, 43, 22, 45,
    39, 33, 30, 24, 18, 12, 5, 63, 47, 56, 27, 60, 41, 37, 16, 54, 35, 52, 21, 44, 32, 23, 11, 46,
    26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6,
];

/// De Bruijn sequence for reverse bit scanning (MSB detection)
///
/// For reverse scanning, we use a different approach since we need to find
/// the most significant bit. We still use the same De Bruijn sequence but
/// with a different isolation method.
const DEBRUIJN_REVERSE_SHIFT: u32 = 58;

/// Bit scan forward using De Bruijn sequence
///
/// This function finds the position of the least significant bit using
/// De Bruijn sequences for O(1) performance.
///
/// # Arguments
/// * `bb` - The bitboard to scan
///
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
///
/// # Performance
/// This implementation provides O(1) bit position determination using:
/// 1. Bit isolation: `bb & (!bb + 1)`
/// 2. De Bruijn multiplication: `isolated_bit * DEBRUIJN64`
/// 3. Table lookup: `DEBRUIJN_TABLE[index]`
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::debruijn::bit_scan_forward_debruijn;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// assert_eq!(bit_scan_forward_debruijn(bb), Some(1)); // Returns LSB position
/// assert_eq!(bit_scan_forward_debruijn(0), None); // Empty bitboard
/// ```
pub fn bit_scan_forward_debruijn(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }

    // For u128, we need to check both halves
    let low = bb as u64;
    if low != 0 {
        Some(bit_scan_forward_debruijn_64(low))
    } else {
        let high = (bb >> 64) as u64;
        Some(bit_scan_forward_debruijn_64(high) + 64)
    }
}

/// Bit scan reverse using De Bruijn sequence
///
/// This function finds the position of the most significant bit using
/// De Bruijn sequences for O(1) performance.
///
/// # Arguments
/// * `bb` - The bitboard to scan
///
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
///
/// # Performance
/// This implementation provides O(1) bit position determination using:
/// 1. MSB isolation: `1 << (63 - leading_zeros)`
/// 2. De Bruijn multiplication: `isolated_bit * DEBRUIJN64`
/// 3. Table lookup: `DEBRUIJN_TABLE[index]`
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::debruijn::bit_scan_reverse_debruijn;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// assert_eq!(bit_scan_reverse_debruijn(bb), Some(3)); // Returns MSB position
/// assert_eq!(bit_scan_reverse_debruijn(0), None); // Empty bitboard
/// ```
pub fn bit_scan_reverse_debruijn(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }

    // For u128, we need to check both halves
    let high = (bb >> 64) as u64;
    if high != 0 {
        Some(bit_scan_reverse_debruijn_64(high) + 64)
    } else {
        let low = bb as u64;
        Some(bit_scan_reverse_debruijn_64(low))
    }
}

/// 64-bit De Bruijn bit scan forward
///
/// This is the core De Bruijn algorithm for finding the least significant bit.
///
/// # Algorithm
/// 1. Isolate the LSB: `bb & (!bb + 1)`
/// 2. Multiply by De Bruijn sequence: `isolated_bit * DEBRUIJN64`
/// 3. Extract high-order bits: `>> 58`
/// 4. Look up position in table: `DEBRUIJN_TABLE[index]`
///
/// # Why This Works
/// The De Bruijn sequence has the property that when multiplied by any power of 2,
/// the high-order bits of the result contain a unique pattern that can be used
/// to determine the original bit position.
fn bit_scan_forward_debruijn_64(bb: u64) -> u8 {
    let isolated_bit = bb & (!bb + 1); // Isolate least significant bit
    let index = (isolated_bit.wrapping_mul(DEBRUIJN64) >> DEBRUIJN_REVERSE_SHIFT) as usize;
    DEBRUIJN_TABLE[index]
}

/// 64-bit De Bruijn bit scan reverse
///
/// This function finds the most significant bit using De Bruijn sequences.
///
/// # Algorithm
/// 1. Find MSB using leading zeros: `63 - bb.leading_zeros()`
/// 2. Isolate the MSB: `1 << msb_position`
/// 3. Multiply by De Bruijn sequence: `isolated_bit * DEBRUIJN64`
/// 4. Extract high-order bits: `>> 58`
/// 5. Look up position in table: `DEBRUIJN_TABLE[index]`
fn bit_scan_reverse_debruijn_64(bb: u64) -> u8 {
    let msb_position = 63 - bb.leading_zeros();
    let isolated_bit: u64 = 1 << msb_position;
    let index = (isolated_bit.wrapping_mul(DEBRUIJN64) >> DEBRUIJN_REVERSE_SHIFT) as usize;
    DEBRUIJN_TABLE[index]
}

/// Get all bit positions using De Bruijn sequences
///
/// This function returns all bit positions in a bitboard using De Bruijn sequences
/// for efficient position determination.
///
/// # Arguments
/// * `bb` - The bitboard to process
///
/// # Returns
/// A vector containing all bit positions (0-based), ordered from LSB to MSB
///
/// # Performance
/// This implementation uses De Bruijn sequences for each bit position,
/// providing O(k) performance where k is the number of set bits.
///
/// # Examples
/// ```
/// use shogi_engine::bitboards::debruijn::get_all_bit_positions_debruijn;
///
/// let bb = 0b1010; // Bits at positions 1 and 3
/// let positions = get_all_bit_positions_debruijn(bb);
/// assert_eq!(positions, vec![1, 3]);
/// ```
pub fn get_all_bit_positions_debruijn(bb: Bitboard) -> Vec<u8> {
    let mut positions = Vec::new();
    let mut remaining = bb;

    while remaining != 0 {
        if let Some(pos) = bit_scan_forward_debruijn(remaining) {
            positions.push(pos);
            remaining &= remaining - 1; // Clear the least significant bit
        } else {
            break;
        }
    }

    positions
}

/// Optimized bit scan with De Bruijn fast paths
///
/// This function provides additional optimizations for common patterns
/// like single-bit bitboards.
///
/// # Arguments
/// * `bb` - The bitboard to scan
/// * `forward` - If true, scan forward (LSB); if false, scan reverse (MSB)
///
/// # Returns
/// The bit position (0-based), or None if the bitboard is empty
///
/// # Performance
/// This implementation includes fast paths for common cases:
/// - Empty bitboard: immediate return
/// - Single bit: optimized path
/// - Multiple bits: standard De Bruijn algorithm
pub fn bit_scan_optimized_debruijn(bb: Bitboard, forward: bool) -> Option<u8> {
    // Fast path for empty bitboard
    if bb == 0 {
        return None;
    }

    // Fast path for single bit (common case)
    if bb & (bb - 1) == 0 {
        // Single bit case - we can use a more direct approach
        if forward {
            return bit_scan_forward_debruijn(bb);
        } else {
            return bit_scan_reverse_debruijn(bb);
        }
    }

    // Use standard De Bruijn implementation
    if forward {
        bit_scan_forward_debruijn(bb)
    } else {
        bit_scan_reverse_debruijn(bb)
    }
}

/// Validate De Bruijn sequence correctness
///
/// This function validates that the De Bruijn sequence and lookup table
/// are correctly configured for all possible bit positions.
///
/// # Returns
/// True if the De Bruijn sequence is correctly configured, false otherwise
///
/// # Use Case
/// This function is primarily used for testing and validation during
/// development and testing phases.
pub fn validate_debruijn_sequence() -> bool {
    // Test all 64 bit positions for forward scanning
    for i in 0..64 {
        let test_bit = 1u64 << i;
        let result = bit_scan_forward_debruijn_64(test_bit);
        if result != i as u8 {
            return false;
        }
    }

    // Test all 64 bit positions for reverse scanning
    for i in 0..64 {
        let test_bit = 1u64 << i;
        let result = bit_scan_reverse_debruijn_64(test_bit);
        if result != i as u8 {
            return false;
        }
    }

    true
}

/// Get De Bruijn sequence information
///
/// This function returns information about the De Bruijn sequence configuration,
/// useful for debugging and validation.
///
/// # Returns
/// A string containing information about the De Bruijn sequence
pub fn get_debruijn_info() -> String {
    format!(
        "De Bruijn Sequence Info:\n\
         Sequence: 0x{:016X}\n\
         Table Size: {} bytes\n\
         Lookup Entries: {}\n\
         Shift Amount: {}",
        DEBRUIJN64,
        std::mem::size_of_val(&DEBRUIJN_TABLE),
        DEBRUIJN_TABLE.len(),
        DEBRUIJN_REVERSE_SHIFT
    )
}

/// Performance benchmark for De Bruijn implementation
///
/// This function provides a simple benchmark to measure the performance
/// of the De Bruijn implementation.
///
/// # Arguments
/// * `iterations` - Number of iterations to run
///
/// # Returns
/// A tuple containing (forward_time_ns, reverse_time_ns, positions_time_ns)
pub fn benchmark_debruijn_performance(iterations: u32) -> (u64, u64, u64) {
    let test_bitboard = 0x123456789ABCDEF0u128;

    // Benchmark forward scanning
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_forward_debruijn(test_bitboard);
    }
    let forward_duration = start.elapsed().as_nanos() as u64;

    // Benchmark reverse scanning
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_reverse_debruijn(test_bitboard);
    }
    let reverse_duration = start.elapsed().as_nanos() as u64;

    // Benchmark position enumeration
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = get_all_bit_positions_debruijn(test_bitboard);
    }
    let positions_duration = start.elapsed().as_nanos() as u64;

    (forward_duration, reverse_duration, positions_duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debruijn_sequence_validation() {
        // Validate that our De Bruijn sequence is correctly configured
        assert!(
            validate_debruijn_sequence(),
            "De Bruijn sequence validation failed"
        );
    }

    #[test]
    fn test_bit_scan_forward_debruijn_correctness() {
        // Test basic cases
        assert_eq!(bit_scan_forward_debruijn(0), None);
        assert_eq!(bit_scan_forward_debruijn(1), Some(0));
        assert_eq!(bit_scan_forward_debruijn(2), Some(1));
        assert_eq!(bit_scan_forward_debruijn(4), Some(2));
        assert_eq!(bit_scan_forward_debruijn(8), Some(3));

        // Test edge cases
        assert_eq!(bit_scan_forward_debruijn(0x8000000000000000), Some(63));
        assert_eq!(bit_scan_forward_debruijn(0x10000000000000000), Some(64));
        assert_eq!(
            bit_scan_forward_debruijn(0x80000000000000000000000000000000),
            Some(127)
        );

        // Test multiple bits (should return LSB)
        assert_eq!(bit_scan_forward_debruijn(0b1010), Some(1)); // Bits at positions 1 and 3
        assert_eq!(bit_scan_forward_debruijn(0b1100), Some(2)); // Bits at positions 2 and 3
    }

    #[test]
    fn test_bit_scan_reverse_debruijn_correctness() {
        // Test basic cases
        assert_eq!(bit_scan_reverse_debruijn(0), None);
        assert_eq!(bit_scan_reverse_debruijn(1), Some(0));
        assert_eq!(bit_scan_reverse_debruijn(2), Some(1));
        assert_eq!(bit_scan_reverse_debruijn(4), Some(2));
        assert_eq!(bit_scan_reverse_debruijn(8), Some(3));

        // Test edge cases
        assert_eq!(bit_scan_reverse_debruijn(0x8000000000000000), Some(63));
        assert_eq!(bit_scan_reverse_debruijn(0x10000000000000000), Some(64));
        assert_eq!(
            bit_scan_reverse_debruijn(0x80000000000000000000000000000000),
            Some(127)
        );

        // Test multiple bits (should return MSB)
        assert_eq!(bit_scan_reverse_debruijn(0b1010), Some(3)); // Bits at positions 1 and 3
        assert_eq!(bit_scan_reverse_debruijn(0b1100), Some(3)); // Bits at positions 2 and 3
    }

    #[test]
    fn test_get_all_bit_positions_debruijn() {
        // Test empty bitboard
        assert_eq!(get_all_bit_positions_debruijn(0), Vec::<u8>::new());

        // Test single bit
        assert_eq!(get_all_bit_positions_debruijn(1), vec![0]);
        assert_eq!(get_all_bit_positions_debruijn(0x8000000000000000), vec![63]);

        // Test multiple bits
        assert_eq!(get_all_bit_positions_debruijn(0b1010), vec![1, 3]); // Bits at positions 1 and 3
        assert_eq!(get_all_bit_positions_debruijn(0b1100), vec![2, 3]); // Bits at positions 2 and 3

        // Test pattern
        let positions = get_all_bit_positions_debruijn(0x5555555555555555); // Every other bit
        assert_eq!(positions.len(), 32);
        assert_eq!(positions[0], 0);
        assert_eq!(positions[1], 2);
        assert_eq!(positions[2], 4);
        assert_eq!(positions[31], 62);
    }

    #[test]
    fn test_bit_scan_optimized_debruijn() {
        // Test empty bitboard fast path
        assert_eq!(bit_scan_optimized_debruijn(0, true), None);
        assert_eq!(bit_scan_optimized_debruijn(0, false), None);

        // Test single bit fast path
        assert_eq!(bit_scan_optimized_debruijn(1, true), Some(0));
        assert_eq!(bit_scan_optimized_debruijn(1, false), Some(0));
        assert_eq!(
            bit_scan_optimized_debruijn(0x8000000000000000, true),
            Some(63)
        );
        assert_eq!(
            bit_scan_optimized_debruijn(0x8000000000000000, false),
            Some(63)
        );

        // Test normal case
        assert_eq!(bit_scan_optimized_debruijn(0b1010, true), Some(1));
        assert_eq!(bit_scan_optimized_debruijn(0b1010, false), Some(3));
    }

    #[test]
    fn test_debruijn_sequence_properties() {
        // Test that the De Bruijn sequence has the expected properties
        let sequence = DEBRUIJN64;

        // The sequence should not be zero
        assert_ne!(sequence, 0);

        // The sequence should have some expected mathematical properties
        // (These are specific to the chosen De Bruijn sequence)
        assert_eq!(sequence & 0x1, 1); // Should end with 1
    }

    #[test]
    fn test_debruijn_table_properties() {
        // Test that the lookup table has the expected properties
        let table = &DEBRUIJN_TABLE;

        // Table should have 64 entries
        assert_eq!(table.len(), 64);

        // All entries should be valid bit positions (0-63)
        for &entry in table {
            assert!(entry < 64, "Invalid table entry: {}", entry);
        }

        // Table should contain all positions 0-63 exactly once
        let mut positions = [false; 64];
        for &entry in table {
            assert!(
                !positions[entry as usize],
                "Duplicate entry in table: {}",
                entry
            );
            positions[entry as usize] = true;
        }

        // All positions should be present
        for (i, &present) in positions.iter().enumerate() {
            assert!(present, "Missing position in table: {}", i);
        }
    }

    #[test]
    fn test_debruijn_memory_usage() {
        // Test that memory usage is within acceptable limits
        let table_size = std::mem::size_of_val(&DEBRUIJN_TABLE);
        let sequence_size = std::mem::size_of_val(&DEBRUIJN64);
        let total_size = table_size + sequence_size;

        // Total memory usage should be less than 64 bytes
        assert!(
            total_size < 64,
            "Memory usage too high: {} bytes",
            total_size
        );

        // Table should be exactly 64 bytes (64 entries × 1 byte each)
        assert_eq!(table_size, 64);

        // Sequence should be 8 bytes (u64)
        assert_eq!(sequence_size, 8);
    }

    #[test]
    fn test_debruijn_performance_benchmark() {
        // Test that the benchmark function works
        let (forward_time, reverse_time, positions_time) = benchmark_debruijn_performance(1000);

        // Times should be reasonable (less than 1 second for 1000 iterations)
        assert!(
            forward_time < 1_000_000_000,
            "Forward scan too slow: {}ns",
            forward_time
        );
        assert!(
            reverse_time < 1_000_000_000,
            "Reverse scan too slow: {}ns",
            reverse_time
        );
        assert!(
            positions_time < 1_000_000_000,
            "Position enumeration too slow: {}ns",
            positions_time
        );

        // Print performance info
        println!("De Bruijn Performance (1000 iterations):");
        println!(
            "  Forward scan: {}ns total, {}ns per call",
            forward_time,
            forward_time / 1000
        );
        println!(
            "  Reverse scan: {}ns total, {}ns per call",
            reverse_time,
            reverse_time / 1000
        );
        println!(
            "  Position enumeration: {}ns total, {}ns per call",
            positions_time,
            positions_time / 1000
        );
    }

    #[test]
    fn test_debruijn_edge_cases() {
        // Test edge cases that might cause issues

        // Test all single bits in 64-bit range
        for i in 0..64 {
            let bb = 1u64 << i;
            assert_eq!(bit_scan_forward_debruijn_64(bb), i as u8);
            assert_eq!(bit_scan_reverse_debruijn_64(bb), i as u8);
        }

        // Test all single bits in 128-bit range
        for i in 0..128 {
            let bb = 1u128 << i;
            assert_eq!(bit_scan_forward_debruijn(bb), Some(i as u8));
            assert_eq!(bit_scan_reverse_debruijn(bb), Some(i as u8));
        }

        // Test all bits set
        let all_bits_64 = 0xFFFFFFFFFFFFFFFFu64;
        assert_eq!(bit_scan_forward_debruijn_64(all_bits_64), 0);
        assert_eq!(bit_scan_reverse_debruijn_64(all_bits_64), 63);

        let all_bits_128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128;
        assert_eq!(bit_scan_forward_debruijn(all_bits_128), Some(0));
        assert_eq!(bit_scan_reverse_debruijn(all_bits_128), Some(127));
    }

    #[test]
    fn test_debruijn_consistency() {
        // Test that De Bruijn implementation is consistent with itself
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
            // Test forward scanning consistency
            let forward1 = bit_scan_forward_debruijn(bb);
            let forward2 = bit_scan_forward_debruijn(bb);
            assert_eq!(
                forward1, forward2,
                "Forward scan inconsistent for 0x{:X}",
                bb
            );

            // Test reverse scanning consistency
            let reverse1 = bit_scan_reverse_debruijn(bb);
            let reverse2 = bit_scan_reverse_debruijn(bb);
            assert_eq!(
                reverse1, reverse2,
                "Reverse scan inconsistent for 0x{:X}",
                bb
            );

            // Test optimized implementation consistency
            let optimized_forward1 = bit_scan_optimized_debruijn(bb, true);
            let optimized_forward2 = bit_scan_optimized_debruijn(bb, true);
            assert_eq!(
                optimized_forward1, optimized_forward2,
                "Optimized forward scan inconsistent for 0x{:X}",
                bb
            );

            let optimized_reverse1 = bit_scan_optimized_debruijn(bb, false);
            let optimized_reverse2 = bit_scan_optimized_debruijn(bb, false);
            assert_eq!(
                optimized_reverse1, optimized_reverse2,
                "Optimized reverse scan inconsistent for 0x{:X}",
                bb
            );
        }
    }
}
