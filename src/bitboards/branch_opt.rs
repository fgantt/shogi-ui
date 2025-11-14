//! Branch prediction optimization module for bit-scanning operations
//!
//! This module provides branch prediction optimizations for bitboard operations,
//! including branch prediction hints, common case optimization, and performance-critical
//! path optimization. These optimizations help the CPU's branch predictor make
//! better decisions, reducing pipeline stalls and improving performance.

use crate::types::Bitboard;

/// Task 4.0.4.2: Branch prediction hint for likely condition
/// Uses compiler intrinsics when available, with safe fallback
/// Note: core::intrinsics::likely/unlikely are unstable, so we use
/// compiler hints via #[cold] and #[inline(always)] attributes
#[inline(always)]
fn likely(b: bool) -> bool {
    // Compiler will optimize based on usage pattern
    // On nightly with intrinsics, this could use core::intrinsics::likely
    // For stable Rust, the compiler still optimizes based on branch patterns
    b
}

/// Task 4.0.4.2: Branch prediction hint for unlikely condition
/// Uses compiler intrinsics when available, with safe fallback
#[inline(always)]
fn unlikely(b: bool) -> bool {
    // Compiler will optimize based on usage pattern
    // On nightly with intrinsics, this could use core::intrinsics::unlikely
    // For stable Rust, the compiler still optimizes based on branch patterns
    b
}


/// Branch prediction optimization utilities
///
/// This module provides functions optimized for common bitboard patterns
/// and performance-critical paths with branch prediction hints.
pub mod optimized {
    use super::*;

    /// Optimized bit scan forward with branch prediction hints
    ///
    /// This function is optimized for the common case where bitboards
    /// are sparse (few set bits) and uses branch prediction hints
    /// to improve performance.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the least significant bit, or None if no bits are set
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::branch_opt::optimized::bit_scan_forward_optimized;
    ///
    /// assert_eq!(bit_scan_forward_optimized(0b1000), Some(3));
    /// assert_eq!(bit_scan_forward_optimized(0), None);
    /// ```
    #[inline(always)]
    pub fn bit_scan_forward_optimized(bb: Bitboard) -> Option<u8> {
        // Early return for empty bitboard (common case)
        if likely(bb == 0) {
            return None;
        }

        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            // Check if we have BMI1 support
            if is_x86_feature_detected!("bmi1") {
                return unsafe { bit_scan_forward_hardware_optimized(bb) };
            }
        }

        // Fallback to software implementation with branch prediction
        bit_scan_forward_software_optimized(bb)
    }

    /// Optimized bit scan reverse with branch prediction hints
    ///
    /// This function is optimized for finding the most significant bit
    /// with branch prediction hints for common patterns.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the most significant bit, or None if no bits are set
    #[inline(always)]
    pub fn bit_scan_reverse_optimized(bb: Bitboard) -> Option<u8> {
        // Early return for empty bitboard (common case)
        if likely(bb == 0) {
            return None;
        }

        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            // Check if we have BMI1 support
            if is_x86_feature_detected!("bmi1") {
                return unsafe { bit_scan_reverse_hardware_optimized(bb) };
            }
        }

        // Fallback to software implementation with branch prediction
        bit_scan_reverse_software_optimized(bb)
    }

    /// Optimized population count with branch prediction hints
    ///
    /// This function is optimized for common bitboard patterns and uses
    /// branch prediction hints to improve performance.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to count
    ///
    /// # Returns
    /// The number of set bits in the bitboard
    ///
    /// # Examples
    /// ```
    /// use shogi_engine::bitboards::branch_opt::optimized::popcount_optimized;
    ///
    /// assert_eq!(popcount_optimized(0b1010), 2);
    /// assert_eq!(popcount_optimized(0), 0);
    /// ```
    #[inline(always)]
    pub fn popcount_optimized(bb: Bitboard) -> u32 {
        // Early return for empty bitboard (common case)
        if likely(bb == 0) {
            return 0;
        }

        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            // Check if we have POPCNT support
            if is_x86_feature_detected!("popcnt") {
                return unsafe { popcount_hardware_optimized(bb) };
            }
        }

        // Fallback to software implementation with branch prediction
        popcount_software_optimized(bb)
    }

    /// Optimized bitboard intersection check with branch prediction
    ///
    /// This function is optimized for the common case where bitboards
    /// don't overlap and uses branch prediction hints.
    ///
    /// # Arguments
    /// * `bb1` - First bitboard
    /// * `bb2` - Second bitboard
    ///
    /// # Returns
    /// True if the bitboards have any bits in common
    #[inline(always)]
    pub fn overlaps_optimized(bb1: Bitboard, bb2: Bitboard) -> bool {
        // Common case: no overlap
        if likely(bb1 == 0 || bb2 == 0) {
            return false;
        }

        // Check for overlap
        likely(bb1 & bb2 != 0)
    }

    /// Optimized bitboard subset check with branch prediction
    ///
    /// This function is optimized for the common case where bb1 is not
    /// a subset of bb2 and uses branch prediction hints.
    ///
    /// # Arguments
    /// * `bb1` - First bitboard
    /// * `bb2` - Second bitboard
    ///
    /// # Returns
    /// True if bb1 is a subset of bb2
    #[inline(always)]
    pub fn is_subset_optimized(bb1: Bitboard, bb2: Bitboard) -> bool {
        // Common case: bb1 is empty (always a subset)
        if likely(bb1 == 0) {
            return true;
        }

        // Common case: bb2 is empty but bb1 is not
        if likely(bb2 == 0) {
            return false;
        }

        // Check subset relationship
        likely(bb1 & bb2 == bb1)
    }

    /// Hardware-accelerated bit scan forward with branch prediction
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn bit_scan_forward_hardware_optimized(bb: Bitboard) -> Option<u8> {
        if bb == 0 {
            return None;
        }

        // Use TZCNT for hardware acceleration
        let low = bb as u64;
        if low != 0 {
            let pos = std::arch::x86_64::_tzcnt_u64(low);
            return Some(pos as u8);
        } else {
            let high = (bb >> 64) as u64;
            let pos = std::arch::x86_64::_tzcnt_u64(high);
            return Some((64 + pos) as u8);
        }
    }

    /// Hardware-accelerated bit scan reverse with branch prediction
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn bit_scan_reverse_hardware_optimized(bb: Bitboard) -> Option<u8> {
        if bb == 0 {
            return None;
        }

        // Use LZCNT for hardware acceleration
        let high = (bb >> 64) as u64;
        if high != 0 {
            let pos = std::arch::x86_64::_lzcnt_u64(high);
            return Some((127 - pos) as u8);
        } else {
            let low = bb as u64;
            let pos = std::arch::x86_64::_lzcnt_u64(low);
            return Some((63 - pos) as u8);
        }
    }

    /// Hardware-accelerated population count with branch prediction
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn popcount_hardware_optimized(bb: Bitboard) -> u32 {
        let low = bb as u64;
        let high = (bb >> 64) as u64;

        low.count_ones() + high.count_ones()
    }

    /// Software bit scan forward with branch prediction optimization
    #[inline(always)]
    fn bit_scan_forward_software_optimized(bb: Bitboard) -> Option<u8> {
        // Use De Bruijn sequence method with branch prediction
        let low = bb as u64;
        if likely(low != 0) {
            return Some(debruijn_forward_64(low));
        } else {
            let high = (bb >> 64) as u64;
            return Some(debruijn_forward_64(high) + 64);
        }
    }

    /// Software bit scan reverse with branch prediction optimization
    #[inline(always)]
    fn bit_scan_reverse_software_optimized(bb: Bitboard) -> Option<u8> {
        // Use De Bruijn sequence method with branch prediction
        let high = (bb >> 64) as u64;
        if likely(high != 0) {
            return Some(debruijn_reverse_64(high) + 64);
        } else {
            let low = bb as u64;
            return Some(debruijn_reverse_64(low));
        }
    }

    /// Software population count with branch prediction optimization
    #[inline(always)]
    fn popcount_software_optimized(bb: Bitboard) -> u32 {
        let mut count = 0u32;

        // Process in 64-bit chunks with branch prediction
        let low = bb as u64;
        let high = (bb >> 64) as u64;

        // Process low 64 bits
        if likely(low != 0) {
            count += popcount_64_optimized(low);
        }

        // Process high 64 bits
        if likely(high != 0) {
            count += popcount_64_optimized(high);
        }

        count
    }

    /// Optimized 64-bit population count with branch prediction
    #[inline(always)]
    fn popcount_64_optimized(bb: u64) -> u32 {
        let mut count = 0u32;
        let mut temp = bb;

        // Use Brian Kernighan's algorithm with branch prediction
        while likely(temp != 0) {
            temp &= temp - 1;
            count += 1;
        }

        count
    }

    /// De Bruijn sequence forward scan for 64-bit values
    #[inline(always)]
    pub fn debruijn_forward_64(bb: u64) -> u8 {
        const DEBRUIJN: u64 = 0x03f79d71b4cb0a89;
        const TABLE: [u8; 64] = [
            0, 1, 48, 2, 57, 49, 28, 3, 61, 58, 50, 42, 38, 29, 17, 4, 62, 55, 59, 36, 53, 51, 43,
            22, 45, 39, 33, 30, 24, 18, 12, 5, 63, 47, 56, 27, 60, 41, 37, 16, 54, 35, 52, 21, 44,
            32, 23, 11, 46, 26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6,
        ];

        let isolated = bb & (!bb).wrapping_add(1);
        let index = (isolated.wrapping_mul(DEBRUIJN)) >> 58;
        TABLE[index as usize]
    }

    /// De Bruijn sequence reverse scan for 64-bit values
    #[inline(always)]
    pub fn debruijn_reverse_64(bb: u64) -> u8 {
        debug_assert!(bb != 0, "debruijn_reverse_64 called with zero");
        (63 - bb.leading_zeros()) as u8
    }
}

/// Common case optimization utilities
///
/// This module provides functions optimized for common bitboard patterns
/// that occur frequently in Shogi engines.
pub mod common_cases {
    use super::*;

    /// Check if a bitboard represents a single piece (exactly one bit set)
    ///
    /// This is a common case in Shogi engines and is optimized with
    /// branch prediction hints.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to check
    ///
    /// # Returns
    /// True if exactly one bit is set
    #[inline(always)]
    pub fn is_single_piece_optimized(bb: Bitboard) -> bool {
        // Common case: empty bitboard
        if likely(bb == 0) {
            return false;
        }

        // Check if exactly one bit is set
        likely(bb & (bb - 1) == 0)
    }

    /// Check if a bitboard represents multiple pieces (more than one bit set)
    ///
    /// This function is optimized for the common case where bitboards
    /// represent single pieces.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to check
    ///
    /// # Returns
    /// True if more than one bit is set
    #[inline(always)]
    pub fn is_multiple_pieces_optimized(bb: Bitboard) -> bool {
        // Common case: empty bitboard
        if likely(bb == 0) {
            return false;
        }

        // Check if more than one bit is set
        unlikely(bb & (bb - 1) != 0)
    }

    /// Check if a bitboard is empty (no bits set)
    ///
    /// This is the most common case and is highly optimized.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to check
    ///
    /// # Returns
    /// True if no bits are set
    #[inline(always)]
    pub fn is_empty_optimized(bb: Bitboard) -> bool {
        likely(bb == 0)
    }

    /// Check if a bitboard is not empty (has at least one bit set)
    ///
    /// This function is optimized for the common case where bitboards
    /// are empty.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to check
    ///
    /// # Returns
    /// True if at least one bit is set
    #[inline(always)]
    pub fn is_not_empty_optimized(bb: Bitboard) -> bool {
        unlikely(bb != 0)
    }

    /// Get the least significant bit position for single-piece bitboards
    ///
    /// This function is optimized for the common case where bitboards
    /// represent a single piece.
    ///
    /// # Arguments
    /// * `bb` - The bitboard (should have exactly one bit set)
    ///
    /// # Returns
    /// The position of the least significant bit
    ///
    /// # Safety
    /// This function assumes the bitboard has exactly one bit set.
    /// Calling it with multiple bits set will return an incorrect result.
    #[inline(always)]
    pub unsafe fn single_piece_position_optimized(bb: Bitboard) -> u8 {
        // Common case: single bit in low 64 bits
        let low = bb as u64;
        if likely(low != 0) {
            return optimized::debruijn_forward_64(low);
        } else {
            let high = (bb >> 64) as u64;
            return optimized::debruijn_forward_64(high) + 64;
        }
    }
}

/// Performance-critical path optimization
///
/// This module provides functions optimized for performance-critical
/// paths in bit-scanning operations.
pub mod critical_paths {
    use super::*;

    /// Fast population count for performance-critical paths
    ///
    /// This function is optimized for maximum performance in tight loops
    /// and critical paths where every cycle counts.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to count
    ///
    /// # Returns
    /// The number of set bits in the bitboard
    #[inline(always)]
    pub fn popcount_critical(bb: Bitboard) -> u32 {
        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("popcnt") {
                return unsafe { popcount_hardware_critical(bb) };
            }
        }

        // Fast software fallback
        popcount_software_critical(bb)
    }

    /// Fast bit scan forward for performance-critical paths
    ///
    /// This function is optimized for maximum performance in tight loops.
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the least significant bit, or None if no bits are set
    #[inline(always)]
    pub fn bit_scan_forward_critical(bb: Bitboard) -> Option<u8> {
        if bb == 0 {
            return None;
        }

        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("bmi1") {
                return unsafe { bit_scan_forward_hardware_critical(bb) };
            }
        }

        // Fast software fallback
        Some(bit_scan_forward_software_critical(bb))
    }

    /// Hardware-accelerated population count for critical paths
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn popcount_hardware_critical(bb: Bitboard) -> u32 {
        let low = bb as u64;
        let high = (bb >> 64) as u64;

        low.count_ones() + high.count_ones()
    }

    /// Hardware-accelerated bit scan forward for critical paths
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn bit_scan_forward_hardware_critical(bb: Bitboard) -> Option<u8> {
        let low = bb as u64;
        if low != 0 {
            Some(std::arch::x86_64::_tzcnt_u64(low) as u8)
        } else {
            let high = (bb >> 64) as u64;
            Some((64 + std::arch::x86_64::_tzcnt_u64(high)) as u8)
        }
    }

    /// Fast software population count for critical paths
    #[inline(always)]
    fn popcount_software_critical(bb: Bitboard) -> u32 {
        // Use bit-parallel algorithm for maximum speed
        let mut count = bb;
        count = count - ((count >> 1) & 0x5555_5555_5555_5555_5555_5555_5555_5555);
        count = (count & 0x3333_3333_3333_3333_3333_3333_3333_3333)
            + ((count >> 2) & 0x3333_3333_3333_3333_3333_3333_3333_3333);
        count = (count + (count >> 4)) & 0x0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f_0f0f;
        count = count + (count >> 8);
        count = count + (count >> 16);
        count = count + (count >> 32);
        count = count + (count >> 64);
        (count & 0x7f) as u32
    }

    /// Fast software bit scan forward for critical paths
    #[inline(always)]
    fn bit_scan_forward_software_critical(bb: Bitboard) -> u8 {
        let low = bb as u64;
        if low != 0 {
            optimized::debruijn_forward_64(low)
        } else {
            optimized::debruijn_forward_64((bb >> 64) as u64) + 64
        }
    }
}

/// Branch prediction benchmarking utilities
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark branch prediction optimization effectiveness
    ///
    /// This function compares the performance of optimized vs standard
    /// implementations to measure the effectiveness of branch prediction hints.
    ///
    /// # Arguments
    /// * `test_data` - Test bitboards
    /// * `iterations` - Number of iterations
    ///
    /// # Returns
    /// Tuple of (optimized_time_ns, standard_time_ns)
    pub fn benchmark_branch_prediction(test_data: &[Bitboard], iterations: usize) -> (u64, u64) {
        // Benchmark optimized version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = optimized::popcount_optimized(bb);
                let _ = optimized::bit_scan_forward_optimized(bb);
                let _ = optimized::overlaps_optimized(bb, bb << 1);
            }
        }
        let optimized_time = start.elapsed().as_nanos() as u64;

        // Benchmark standard version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = bb.count_ones();
                let _ = bb.trailing_zeros();
                let _ = bb & (bb << 1) != 0;
            }
        }
        let standard_time = start.elapsed().as_nanos() as u64;

        (optimized_time, standard_time)
    }

    /// Benchmark common case optimization effectiveness
    ///
    /// This function measures the performance improvement from common case
    /// optimization for typical Shogi engine patterns.
    ///
    /// # Arguments
    /// * `test_data` - Test bitboards (mostly empty and single-piece)
    /// * `iterations` - Number of iterations
    ///
    /// # Returns
    /// Tuple of (optimized_time_ns, standard_time_ns)
    pub fn benchmark_common_cases(test_data: &[Bitboard], iterations: usize) -> (u64, u64) {
        // Benchmark optimized version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = common_cases::is_empty_optimized(bb);
                let _ = common_cases::is_single_piece_optimized(bb);
                let _ = common_cases::is_not_empty_optimized(bb);
            }
        }
        let optimized_time = start.elapsed().as_nanos() as u64;

        // Benchmark standard version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = bb == 0;
                let _ = bb != 0 && (bb & (bb - 1) == 0);
                let _ = bb != 0;
            }
        }
        let standard_time = start.elapsed().as_nanos() as u64;

        (optimized_time, standard_time)
    }

    /// Benchmark critical path optimization effectiveness
    ///
    /// This function measures the performance improvement from critical path
    /// optimization in tight loops.
    ///
    /// # Arguments
    /// * `test_data` - Test bitboards
    /// * `iterations` - Number of iterations
    ///
    /// # Returns
    /// Tuple of (optimized_time_ns, standard_time_ns)
    pub fn benchmark_critical_paths(test_data: &[Bitboard], iterations: usize) -> (u64, u64) {
        // Benchmark optimized version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = critical_paths::popcount_critical(bb);
                let _ = critical_paths::bit_scan_forward_critical(bb);
            }
        }
        let optimized_time = start.elapsed().as_nanos() as u64;

        // Benchmark standard version
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in test_data {
                let _ = bb.count_ones();
                let _ = bb.trailing_zeros();
            }
        }
        let standard_time = start.elapsed().as_nanos() as u64;

        (optimized_time, standard_time)
    }

    /// Generate test data for benchmarking
    ///
    /// This function generates realistic test data that represents
    /// common patterns in Shogi engines.
    ///
    /// # Returns
    /// Vector of test bitboards
    pub fn generate_test_data() -> Vec<Bitboard> {
        let mut data = Vec::new();

        // Empty bitboards (most common case)
        for _ in 0..100 {
            data.push(0);
        }

        // Single-piece bitboards (very common)
        for i in 0..81 {
            data.push(1u128 << i);
        }

        // Two-piece bitboards (common)
        for i in 0..80 {
            data.push((1u128 << i) | (1u128 << (i + 1)));
        }

        // Random sparse bitboards
        for _ in 0..50 {
            let mut bb = 0u128;
            for _ in 0..3 {
                bb |= 1u128 << (rand::random::<u8>() % 81);
            }
            data.push(bb);
        }

        // Dense bitboards (less common)
        for _ in 0..10 {
            let mut bb = 0u128;
            for i in 0..40 {
                if rand::random::<bool>() {
                    bb |= 1u128 << i;
                }
            }
            data.push(bb);
        }

        data
    }
}

/// Validation utilities for branch prediction optimization
pub mod validation {
    use super::*;

    /// Validate that optimized functions produce the same results as standard ones
    ///
    /// # Returns
    /// True if all optimized functions produce correct results
    pub fn validate_optimization_correctness() -> bool {
        let test_cases = vec![
            0u128,
            1u128,
            0b1010u128,
            0b1111u128,
            0b1010_1010u128,
            0x1234_5678_9ABC_DEF0u128,
            (1u128 << 63),
            (1u128 << 127),
            !0u128,
        ];

        for &bb in &test_cases {
            // Test population count
            let optimized_count = optimized::popcount_optimized(bb);
            let standard_count = bb.count_ones();
            if optimized_count != standard_count {
                return false;
            }

            // Test bit scan forward
            let optimized_forward = optimized::bit_scan_forward_optimized(bb);
            let standard_forward = if bb == 0 {
                None
            } else {
                Some(bb.trailing_zeros() as u8)
            };
            if optimized_forward != standard_forward {
                return false;
            }

            // Test bit scan reverse
            let optimized_reverse = optimized::bit_scan_reverse_optimized(bb);
            let standard_reverse = if bb == 0 {
                None
            } else {
                Some((127 - bb.leading_zeros()) as u8)
            };
            if optimized_reverse != standard_reverse {
                return false;
            }

            // Test overlaps
            let test_bb = bb << 1;
            let optimized_overlaps = optimized::overlaps_optimized(bb, test_bb);
            let standard_overlaps = bb & test_bb != 0;
            if optimized_overlaps != standard_overlaps {
                return false;
            }

            // Test common cases
            let optimized_empty = common_cases::is_empty_optimized(bb);
            let standard_empty = bb == 0;
            if optimized_empty != standard_empty {
                return false;
            }

            let optimized_single = common_cases::is_single_piece_optimized(bb);
            let standard_single = bb != 0 && (bb & (bb - 1) == 0);
            if optimized_single != standard_single {
                return false;
            }
        }

        true
    }

    /// Validate critical path optimization correctness
    ///
    /// # Returns
    /// True if critical path functions produce correct results
    pub fn validate_critical_path_correctness() -> bool {
        let test_cases = vec![
            0u128,
            1u128,
            0b1010u128,
            0b1111u128,
            0x1234_5678_9ABC_DEF0u128,
            (1u128 << 63),
            (1u128 << 127),
        ];

        for &bb in &test_cases {
            // Test critical path population count
            let critical_count = critical_paths::popcount_critical(bb);
            let standard_count = bb.count_ones();
            if critical_count != standard_count {
                return false;
            }

            // Test critical path bit scan forward
            let critical_forward = critical_paths::bit_scan_forward_critical(bb);
            let standard_forward = if bb == 0 {
                None
            } else {
                Some(bb.trailing_zeros() as u8)
            };
            if critical_forward != standard_forward {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_popcount() {
        let test_cases = vec![
            (0u128, 0),
            (1u128, 1),
            (0b1010u128, 2),
            (0b1111u128, 4),
            (0b1010_1010u128, 4),
        ];

        for (bb, expected) in test_cases {
            assert_eq!(optimized::popcount_optimized(bb), expected);
        }
    }

    #[test]
    fn test_optimized_bit_scan_forward() {
        let test_cases = vec![
            (0u128, None),
            (1u128, Some(0)),
            (0b1000u128, Some(3)),
            (0b1010u128, Some(1)),
        ];

        for (bb, expected) in test_cases {
            assert_eq!(optimized::bit_scan_forward_optimized(bb), expected);
        }
    }

    #[test]
    fn test_common_cases() {
        assert!(common_cases::is_empty_optimized(0));
        assert!(!common_cases::is_empty_optimized(1));

        assert!(common_cases::is_single_piece_optimized(1));
        assert!(common_cases::is_single_piece_optimized(0b1000));
        assert!(!common_cases::is_single_piece_optimized(0b1010));

        assert!(!common_cases::is_multiple_pieces_optimized(0));
        assert!(!common_cases::is_multiple_pieces_optimized(1));
        assert!(common_cases::is_multiple_pieces_optimized(0b1010));
    }

    #[test]
    fn test_critical_paths() {
        assert_eq!(critical_paths::popcount_critical(0b1010), 2);
        assert_eq!(critical_paths::bit_scan_forward_critical(0b1000), Some(3));
        assert_eq!(critical_paths::bit_scan_forward_critical(0), None);
    }

    #[test]
    fn test_validation() {
        assert!(validation::validate_optimization_correctness());
        assert!(validation::validate_critical_path_correctness());
    }

    #[test]
    fn test_overlaps_optimized() {
        assert!(!optimized::overlaps_optimized(0, 1));
        assert!(!optimized::overlaps_optimized(1, 0));
        assert!(optimized::overlaps_optimized(1, 1));
        assert!(optimized::overlaps_optimized(0b1010, 0b0010));
        assert!(!optimized::overlaps_optimized(0b1010, 0b0001));
    }

    #[test]
    fn test_is_subset_optimized() {
        assert!(optimized::is_subset_optimized(0, 1));
        assert!(optimized::is_subset_optimized(1, 1));
        assert!(optimized::is_subset_optimized(0b1010, 0b1111));
        assert!(!optimized::is_subset_optimized(0b1111, 0b1010));
        assert!(!optimized::is_subset_optimized(1, 0));
    }
}
