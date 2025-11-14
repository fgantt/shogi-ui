//! Integration module for all bit-scanning optimizations
//!
//! This module provides a unified interface that integrates all bit-scanning
//! optimizations including De Bruijn sequences, 4-bit lookup tables, and
//! precomputed masks. It automatically selects the best algorithm based on
//! platform capabilities and performance characteristics.
//!
//! ## Adaptive Selection (Task 4.0.4.6)
//!
//! The `BitScanningOptimizer` uses adaptive algorithm selection based on:
//! - **Platform capabilities**: Hardware support for POPCNT and BMI1 instructions
//! - **Bit density**: Sparse boards (< 16 bits) use 4-bit lookup tables,
//!   medium density (16-32 bits) use De Bruijn sequences, dense boards use SWAR
//! - **Bit distribution**: The estimator counts high and low halves independently
//!   to avoid misclassifying boards with bits concentrated in one half
//!
//! ## Configuration
//!
//! - Use `BitScanningOptimizer::new()` for automatic adaptive selection
//! - Use `BitScanningOptimizer::with_config(false)` to disable adaptive selection
//!   and always use De Bruijn sequences
//! - Access `get_strategy_counters()` to see which strategies are being used
//! - Call `reset_counters()` to reset telemetry
//!
//! ## Performance Tuning
//!
//! The adaptive system automatically selects optimal algorithms, but you can
//! monitor strategy selection via `StrategyCounters` to understand which paths
//! are taken. For platforms without hardware acceleration, the system falls back
//! to software implementations optimized for different bit densities.

use crate::bitboards::{
    debruijn::{
        bit_scan_forward_debruijn, bit_scan_reverse_debruijn, get_all_bit_positions_debruijn,
    },
    lookup_tables::{bit_positions_4bit_lookup, popcount_4bit_optimized},
    masks::{get_diagonal_mask, get_file_mask, get_rank_mask},
    platform_detection::get_platform_capabilities,
};
use crate::types::Bitboard;

/// Unified bit-scanning interface that automatically selects the best algorithm
///
/// This struct provides a high-level interface that automatically chooses
/// the most appropriate bit-scanning algorithm based on platform capabilities,
/// bitboard characteristics, and performance requirements.
/// Task 4.0.4.3: Added telemetry counters for strategy selection
pub struct BitScanningOptimizer {
    platform_caps: crate::bitboards::platform_detection::PlatformCapabilities,
    use_adaptive_selection: bool,
    /// Task 4.0.4.3: Telemetry counters for strategy selection
    strategy_counters: std::sync::Mutex<StrategyCounters>,
}

/// Task 4.0.4.3: Telemetry counters for tracking strategy selection
/// Task 4.0.4.4: Made public for API access
#[derive(Debug, Default, Clone)]
pub struct StrategyCounters {
    popcount_hardware: u64,
    popcount_4bit: u64,
    popcount_swar: u64,
    popcount_debruijn: u64,
    bitscan_hardware: u64,
    bitscan_debruijn: u64,
    bitscan_software: u64,
    positions_4bit: u64,
    positions_debruijn: u64,
    positions_optimized: u64,
}

impl StrategyCounters {
    fn new() -> Self {
        Self::default()
    }
}

impl BitScanningOptimizer {
    /// Create a new bit-scanning optimizer
    ///
    /// # Returns
    /// A new optimizer instance with platform capabilities detected
    pub fn new() -> Self {
        Self {
            platform_caps: get_platform_capabilities().clone(),
            use_adaptive_selection: true,
            strategy_counters: std::sync::Mutex::new(StrategyCounters::new()),
        }
    }

    /// Create a new optimizer with specific configuration
    ///
    /// # Arguments
    /// * `use_adaptive_selection` - Whether to use adaptive algorithm selection
    ///
    /// # Returns
    /// A new optimizer instance with the specified configuration
    pub fn with_config(use_adaptive_selection: bool) -> Self {
        Self {
            platform_caps: get_platform_capabilities().clone(),
            use_adaptive_selection,
            strategy_counters: std::sync::Mutex::new(StrategyCounters::new()),
        }
    }

    /// Task 4.0.4.3: Get strategy selection telemetry
    pub fn get_strategy_counters(&self) -> StrategyCounters {
        self.strategy_counters.lock().unwrap().clone()
    }

    /// Task 4.0.4.3: Reset strategy counters
    pub fn reset_counters(&self) {
        *self.strategy_counters.lock().unwrap() = StrategyCounters::new();
    }

    /// Get the best population count implementation for a given bitboard
    ///
    /// # Arguments
    /// * `bb` - The bitboard to count bits in
    ///
    /// # Returns
    /// The number of set bits using the optimal algorithm
    pub fn popcount(&self, bb: Bitboard) -> u32 {
        if !self.use_adaptive_selection {
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.popcount_debruijn += 1;
            }
            return self.popcount_debruijn(bb);
        }

        // Adaptive selection based on bitboard characteristics
        let bit_count = self.estimate_bit_count(bb);

        // Determine best implementation based on platform capabilities
        if self.platform_caps.has_popcnt {
            // Hardware acceleration is available and fastest
            // Task 4.0.4.3: Track strategy selection
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.popcount_hardware += 1;
            }
            self.popcount_hardware(bb)
        } else {
            // Choose between 4-bit lookup and SWAR based on density
            if bit_count < 16 {
                if let Ok(mut counters) = self.strategy_counters.lock() {
                    counters.popcount_4bit += 1;
                }
                popcount_4bit_optimized(bb)
            } else {
                if let Ok(mut counters) = self.strategy_counters.lock() {
                    counters.popcount_swar += 1;
                }
                self.popcount_swar(bb)
            }
        }
    }

    /// Get the best bit scan forward implementation
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the least significant bit, or None if empty
    pub fn bit_scan_forward(&self, bb: Bitboard) -> Option<u8> {
        if !self.use_adaptive_selection {
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.bitscan_debruijn += 1;
            }
            return bit_scan_forward_debruijn(bb);
        }

        // Determine best implementation based on platform capabilities
        if self.platform_caps.has_bmi1 {
            // Hardware acceleration available
            // Task 4.0.4.3: Track strategy selection
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.bitscan_hardware += 1;
            }
            self.bit_scan_forward_hardware(bb)
        } else {
            // De Bruijn sequences - best software fallback
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.bitscan_debruijn += 1;
            }
            bit_scan_forward_debruijn(bb)
        }
    }

    /// Get the best bit scan reverse implementation
    ///
    /// # Arguments
    /// * `bb` - The bitboard to scan
    ///
    /// # Returns
    /// The position of the most significant bit, or None if empty
    pub fn bit_scan_reverse(&self, bb: Bitboard) -> Option<u8> {
        if !self.use_adaptive_selection {
            return bit_scan_reverse_debruijn(bb);
        }

        // Determine best implementation based on platform capabilities
        if self.platform_caps.has_bmi1 {
            // Hardware acceleration available
            self.bit_scan_reverse_hardware(bb)
        } else {
            // De Bruijn sequences - best software fallback
            bit_scan_reverse_debruijn(bb)
        }
    }

    /// Get all bit positions using the optimal algorithm
    ///
    /// # Arguments
    /// * `bb` - The bitboard to process
    ///
    /// # Returns
    /// A vector containing all bit positions
    pub fn get_all_bit_positions(&self, bb: Bitboard) -> Vec<u8> {
        if !self.use_adaptive_selection {
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.positions_debruijn += 1;
            }
            return get_all_bit_positions_debruijn(bb);
        }

        // For position enumeration, choose based on bit density
        let bit_count = self.estimate_bit_count(bb);

        if bit_count <= 8 {
            // Few bits - use 4-bit lookup tables for efficiency
            // Task 4.0.4.3: Track strategy selection
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.positions_4bit += 1;
            }
            bit_positions_4bit_lookup(bb)
        } else if bit_count <= 32 {
            // Medium density - use De Bruijn sequences
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.positions_debruijn += 1;
            }
            get_all_bit_positions_debruijn(bb)
        } else {
            // High density - use optimized enumeration
            if let Ok(mut counters) = self.strategy_counters.lock() {
                counters.positions_optimized += 1;
            }
            self.get_all_bit_positions_optimized(bb)
        }
    }

    /// Optimized combined operations for common patterns
    ///
    /// # Arguments
    /// * `bb` - The bitboard to process
    ///
    /// # Returns
    /// A tuple containing (popcount, first_bit, last_bit)
    pub fn analyze_bitboard(&self, bb: Bitboard) -> (u32, Option<u8>, Option<u8>) {
        let popcount = self.popcount(bb);
        let first_bit = self.bit_scan_forward(bb);
        let last_bit = self.bit_scan_reverse(bb);

        (popcount, first_bit, last_bit)
    }

    /// Get geometric analysis using precomputed masks
    ///
    /// # Arguments
    /// * `bb` - The bitboard to analyze
    ///
    /// # Returns
    /// A struct containing geometric analysis results
    pub fn analyze_geometry(&self, bb: Bitboard) -> GeometricAnalysis {
        let mut rank_counts = [0u32; 9];
        let mut file_counts = [0u32; 9];
        let mut diagonal_counts = [0u32; 15];

        // Analyze ranks
        for rank in 0..9 {
            let rank_mask = get_rank_mask(rank);
            rank_counts[rank as usize] = self.popcount(bb & rank_mask);
        }

        // Analyze files
        for file in 0..9 {
            let file_mask = get_file_mask(file);
            file_counts[file as usize] = self.popcount(bb & file_mask);
        }

        // Analyze diagonals
        for diagonal in 0..15 {
            let diagonal_mask = get_diagonal_mask(diagonal);
            diagonal_counts[diagonal as usize] = self.popcount(bb & diagonal_mask);
        }

        GeometricAnalysis {
            rank_counts,
            file_counts,
            diagonal_counts,
            total_popcount: rank_counts.iter().sum(),
        }
    }

    // Private helper methods for different implementations

    /// Task 4.0.4.1: Corrected to count high/low halves independently
    /// or use count_ones() thresholds to avoid misclassifying dense boards
    /// Task 4.0.4.5: Made public for testing
    pub fn estimate_bit_count(&self, bb: Bitboard) -> u32 {
        // Use actual popcount for accurate estimation
        // This is fast on modern CPUs with hardware popcount support
        let high_bits = (bb >> 64) as u64;
        let low_bits = bb as u64;
        
        // Count high and low halves independently
        // This prevents misclassification when bits are concentrated in one half
        let high_count = high_bits.count_ones();
        let low_count = low_bits.count_ones();
        
        high_count + low_count
    }

    fn popcount_hardware(&self, bb: Bitboard) -> u32 {
        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                let low = bb as u64;
                let high = (bb >> 64) as u64;
                std::arch::x86_64::_popcnt64(low as i64) as u32
                    + std::arch::x86_64::_popcnt64(high as i64) as u32
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.popcount_swar(bb)
        }
    }

    fn popcount_swar(&self, bb: Bitboard) -> u32 {
        // SWAR (SIMD Within A Register) implementation
        let low = bb as u64;
        let high = (bb >> 64) as u64;

        let low_count = {
            let mut x = low;
            x = x - ((x >> 1) & 0x5555555555555555);
            x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
            x = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
            ((x.wrapping_mul(0x0101010101010101)) >> 56) as u32
        };

        let high_count = {
            let mut x = high;
            x = x - ((x >> 1) & 0x5555555555555555);
            x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
            x = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
            ((x.wrapping_mul(0x0101010101010101)) >> 56) as u32
        };

        low_count + high_count
    }

    fn popcount_software(&self, bb: Bitboard) -> u32 {
        // Basic software implementation
        let mut count = 0;
        let mut remaining = bb;
        while remaining != 0 {
            count += 1;
            remaining &= remaining - 1;
        }
        count
    }

    fn popcount_debruijn(&self, bb: Bitboard) -> u32 {
        // Use De Bruijn-based counting
        let mut count = 0;
        let mut remaining = bb;
        while remaining != 0 {
            if let Some(_pos) = bit_scan_forward_debruijn(remaining) {
                count += 1;
                remaining &= remaining - 1;
            } else {
                break;
            }
        }
        count
    }

    fn bit_scan_forward_hardware(&self, bb: Bitboard) -> Option<u8> {
        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            let low = bb as u64;
            if low != 0 {
                unsafe {
                    return Some(std::arch::x86_64::_tzcnt_u64(low) as u8);
                }
            }
            let high = (bb >> 64) as u64;
            if high != 0 {
                unsafe {
                    return Some(std::arch::x86_64::_tzcnt_u64(high) as u8 + 64);
                }
            }
            None
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            bit_scan_forward_debruijn(bb)
        }
    }

    fn bit_scan_reverse_hardware(&self, bb: Bitboard) -> Option<u8> {
        // Use hardware acceleration when available
        #[cfg(target_arch = "x86_64")]
        {
            let high = (bb >> 64) as u64;
            if high != 0 {
                unsafe {
                    return Some(63 - std::arch::x86_64::_lzcnt_u64(high) as u8 + 64);
                }
            }
            let low = bb as u64;
            if low != 0 {
                unsafe {
                    return Some(63 - std::arch::x86_64::_lzcnt_u64(low) as u8);
                }
            }
            None
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            bit_scan_reverse_debruijn(bb)
        }
    }

    fn bit_scan_forward_software(&self, bb: Bitboard) -> Option<u8> {
        // Basic software implementation
        if bb == 0 {
            return None;
        }

        let mut remaining = bb;
        let mut position = 0;

        while remaining != 0 {
            if remaining & 1 != 0 {
                return Some(position);
            }
            remaining >>= 1;
            position += 1;
        }

        None
    }

    fn bit_scan_reverse_software(&self, bb: Bitboard) -> Option<u8> {
        // Basic software implementation
        if bb == 0 {
            return None;
        }

        let mut position = 127;
        let mut remaining = bb;

        while remaining != 0 {
            if remaining & (1u128 << 127) != 0 {
                return Some(position);
            }
            remaining <<= 1;
            position -= 1;
        }

        None
    }

    fn get_all_bit_positions_optimized(&self, bb: Bitboard) -> Vec<u8> {
        // Optimized enumeration for high-density bitboards
        let mut positions = Vec::new();
        let mut remaining = bb;

        // Use the best available bit scan implementation
        while remaining != 0 {
            if let Some(pos) = self.bit_scan_forward(remaining) {
                positions.push(pos);
                remaining &= remaining - 1;
            } else {
                break;
            }
        }

        positions
    }
}

/// Results of geometric analysis on a bitboard
#[derive(Debug, Clone)]
pub struct GeometricAnalysis {
    /// Population count for each rank (0-8)
    pub rank_counts: [u32; 9],
    /// Population count for each file (0-8)
    pub file_counts: [u32; 9],
    /// Population count for each diagonal (0-14)
    pub diagonal_counts: [u32; 15],
    /// Total population count
    pub total_popcount: u32,
}

impl GeometricAnalysis {
    /// Get the rank with the most bits set
    ///
    /// # Returns
    /// The rank index (0-8) with the highest population count
    pub fn densest_rank(&self) -> u8 {
        self.rank_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(rank, _)| rank as u8)
            .unwrap_or(0)
    }

    /// Get the file with the most bits set
    ///
    /// # Returns
    /// The file index (0-8) with the highest population count
    pub fn densest_file(&self) -> u8 {
        self.file_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(file, _)| file as u8)
            .unwrap_or(0)
    }

    /// Get the diagonal with the most bits set
    ///
    /// # Returns
    /// The diagonal index (0-14) with the highest population count
    pub fn densest_diagonal(&self) -> u8 {
        self.diagonal_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(diagonal, _)| diagonal as u8)
            .unwrap_or(0)
    }

    /// Check if the bitboard has any geometric patterns
    ///
    /// # Returns
    /// True if any rank, file, or diagonal is completely filled
    pub fn has_complete_lines(&self) -> bool {
        // Check for complete ranks
        if self.rank_counts.iter().any(|&count| count == 9) {
            return true;
        }

        // Check for complete files
        if self.file_counts.iter().any(|&count| count == 9) {
            return true;
        }

        // Check for complete diagonals (variable length)
        for (i, &count) in self.diagonal_counts.iter().enumerate() {
            let expected_length = if i < 9 { i + 1 } else { 15 - i };
            if count == expected_length as u32 {
                return true;
            }
        }

        false
    }
}

/// Global default optimizer instance
///
/// This provides a convenient way to use the bit-scanning optimizations
/// without explicitly creating an optimizer instance.
pub struct GlobalOptimizer;

impl GlobalOptimizer {
    /// Get the default optimizer instance
    pub fn get() -> BitScanningOptimizer {
        BitScanningOptimizer::new()
    }

    /// Population count using the best available algorithm
    pub fn popcount(bb: Bitboard) -> u32 {
        Self::get().popcount(bb)
    }

    /// Bit scan forward using the best available algorithm
    pub fn bit_scan_forward(bb: Bitboard) -> Option<u8> {
        Self::get().bit_scan_forward(bb)
    }

    /// Bit scan reverse using the best available algorithm
    pub fn bit_scan_reverse(bb: Bitboard) -> Option<u8> {
        Self::get().bit_scan_reverse(bb)
    }

    /// Get all bit positions using the best available algorithm
    pub fn get_all_bit_positions(bb: Bitboard) -> Vec<u8> {
        Self::get().get_all_bit_positions(bb)
    }

    /// Analyze bitboard using the best available algorithms
    pub fn analyze_bitboard(bb: Bitboard) -> (u32, Option<u8>, Option<u8>) {
        Self::get().analyze_bitboard(bb)
    }

    /// Analyze bitboard geometry using precomputed masks
    pub fn analyze_geometry(bb: Bitboard) -> GeometricAnalysis {
        Self::get().analyze_geometry(bb)
    }
}

/// Memory alignment optimization for lookup tables
///
/// This module provides utilities for ensuring optimal memory alignment
/// of lookup tables for cache performance.
pub mod alignment {
    // Memory alignment utilities for lookup table optimization
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Cache line size (typically 64 bytes on modern processors)
    pub const CACHE_LINE_SIZE: usize = 64;

    /// Ensure a value is aligned to cache line boundaries
    ///
    /// # Arguments
    /// * `value` - The value to align
    ///
    /// # Returns
    /// The aligned value
    pub fn align_to_cache_line(value: usize) -> usize {
        (value + CACHE_LINE_SIZE - 1) & !(CACHE_LINE_SIZE - 1)
    }

    /// Check if a value is cache line aligned
    ///
    /// # Arguments
    /// * `value` - The value to check
    ///
    /// # Returns
    /// True if the value is cache line aligned
    pub fn is_cache_line_aligned(value: usize) -> bool {
        value & (CACHE_LINE_SIZE - 1) == 0
    }

    /// Memory usage statistics for optimization tracking
    pub struct MemoryStats {
        total_allocated: AtomicUsize,
        cache_aligned_allocations: AtomicUsize,
    }

    impl MemoryStats {
        pub fn new() -> Self {
            Self {
                total_allocated: AtomicUsize::new(0),
                cache_aligned_allocations: AtomicUsize::new(0),
            }
        }

        pub fn get_total_allocated(&self) -> usize {
            self.total_allocated.load(Ordering::Relaxed)
        }

        pub fn get_cache_aligned_allocations(&self) -> usize {
            self.cache_aligned_allocations.load(Ordering::Relaxed)
        }

        pub fn record_allocation(&self, size: usize, aligned: bool) {
            self.total_allocated.fetch_add(size, Ordering::Relaxed);
            if aligned {
                self.cache_aligned_allocations
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    impl Default for MemoryStats {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    // Task 4.0.4.5: Tests for estimator edge cases and branch hints
    use super::*;

    #[test]
    fn test_bit_scanning_optimizer_creation() {
        let optimizer = BitScanningOptimizer::new();
        // Test that optimizer works (adaptive selection is enabled by default)
        let result = optimizer.popcount(0b1010);
        assert_eq!(result, 2);

        let optimizer_fixed = BitScanningOptimizer::with_config(false);
        // Test that fixed optimizer works (should use De Bruijn)
        let result2 = optimizer_fixed.popcount(0b1010);
        assert_eq!(result2, 2);
    }

    #[test]
    fn test_popcount_consistency() {
        let optimizer = BitScanningOptimizer::new();
        let test_cases = [
            0u128,
            1u128,
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
            let result1 = optimizer.popcount(bb);
            let result2 = optimizer.popcount(bb);
            assert_eq!(result1, result2, "Popcount inconsistent for 0x{:X}", bb);

            // Test global optimizer
            let global_result = GlobalOptimizer::popcount(bb);
            assert_eq!(
                result1, global_result,
                "Global optimizer inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_bit_scan_consistency() {
        let optimizer = BitScanningOptimizer::new();
        let test_cases = [
            1u128,
            2u128,
            4u128,
            8u128,
            0xFFu128,
            0x8000000000000000u128,
            0x10000000000000000u128,
            0x5555555555555555u128,
            0x123456789ABCDEF0u128,
        ];

        for bb in test_cases {
            let forward1 = optimizer.bit_scan_forward(bb);
            let forward2 = optimizer.bit_scan_forward(bb);
            assert_eq!(
                forward1, forward2,
                "Forward scan inconsistent for 0x{:X}",
                bb
            );

            let reverse1 = optimizer.bit_scan_reverse(bb);
            let reverse2 = optimizer.bit_scan_reverse(bb);
            assert_eq!(
                reverse1, reverse2,
                "Reverse scan inconsistent for 0x{:X}",
                bb
            );

            // Test global optimizer
            let global_forward = GlobalOptimizer::bit_scan_forward(bb);
            let global_reverse = GlobalOptimizer::bit_scan_reverse(bb);
            assert_eq!(
                forward1, global_forward,
                "Global forward scan inconsistent for 0x{:X}",
                bb
            );
            assert_eq!(
                reverse1, global_reverse,
                "Global reverse scan inconsistent for 0x{:X}",
                bb
            );
        }
    }

    #[test]
    fn test_analyze_bitboard() {
        let optimizer = BitScanningOptimizer::new();
        let bb = 0b1010u128; // Bits at positions 1 and 3

        let (popcount, first_bit, last_bit) = optimizer.analyze_bitboard(bb);

        assert_eq!(popcount, 2);
        assert_eq!(first_bit, Some(1));
        assert_eq!(last_bit, Some(3));

        // Test global optimizer
        let (global_popcount, global_first_bit, global_last_bit) =
            GlobalOptimizer::analyze_bitboard(bb);
        assert_eq!(popcount, global_popcount);
        assert_eq!(first_bit, global_first_bit);
        assert_eq!(last_bit, global_last_bit);
    }

    #[test]
    fn test_analyze_geometry() {
        let optimizer = BitScanningOptimizer::new();

        // Test with a bitboard that has bits on rank 0 and file 0
        let bb = 0b111111111u128; // Bottom rank (rank 0)

        let analysis = optimizer.analyze_geometry(bb);

        assert_eq!(analysis.total_popcount, 9);
        assert_eq!(analysis.rank_counts[0], 9); // Rank 0 should have 9 bits
        assert_eq!(analysis.file_counts[0], 1); // File 0 should have 1 bit
        assert_eq!(analysis.file_counts[1], 1); // File 1 should have 1 bit
                                                // ... etc for other files

        assert_eq!(analysis.densest_rank(), 0);

        // Test global optimizer
        let global_analysis = GlobalOptimizer::analyze_geometry(bb);
        assert_eq!(analysis.total_popcount, global_analysis.total_popcount);
    }

    #[test]
    fn test_geometric_analysis_utilities() {
        let optimizer = BitScanningOptimizer::new();

        // Test with a complete rank
        let complete_rank = get_rank_mask(0); // All bits on rank 0
        let analysis = optimizer.analyze_geometry(complete_rank);

        assert!(analysis.has_complete_lines());
        assert_eq!(analysis.densest_rank(), 0);

        // Test with a complete file
        let complete_file = get_file_mask(0); // All bits on file 0
        let analysis = optimizer.analyze_geometry(complete_file);

        assert!(analysis.has_complete_lines());
        assert_eq!(analysis.densest_file(), 0);
    }

    #[test]
    fn test_memory_alignment() {
        use alignment::*;

        // Test cache line alignment
        assert!(is_cache_line_aligned(0));
        assert!(is_cache_line_aligned(64));
        assert!(is_cache_line_aligned(128));
        assert!(!is_cache_line_aligned(1));
        assert!(!is_cache_line_aligned(63));

        // Test alignment calculation
        assert_eq!(align_to_cache_line(0), 0);
        assert_eq!(align_to_cache_line(1), 64);
        assert_eq!(align_to_cache_line(63), 64);
        assert_eq!(align_to_cache_line(64), 64);
        assert_eq!(align_to_cache_line(65), 128);
    }

    #[test]
    fn test_memory_stats() {
        use alignment::*;

        let stats = MemoryStats::new();

        assert_eq!(stats.get_total_allocated(), 0);
        assert_eq!(stats.get_cache_aligned_allocations(), 0);

        stats.record_allocation(128, true);
        assert_eq!(stats.get_total_allocated(), 128);
        assert_eq!(stats.get_cache_aligned_allocations(), 1);

        stats.record_allocation(64, false);
        assert_eq!(stats.get_total_allocated(), 192);
        assert_eq!(stats.get_cache_aligned_allocations(), 1);
    }

    #[test]
    fn test_performance_consistency() {
        // Test that different configurations produce consistent results
        let adaptive_optimizer = BitScanningOptimizer::with_config(true);
        let fixed_optimizer = BitScanningOptimizer::with_config(false);

        let test_bitboard = 0x123456789ABCDEF0u128;

        let adaptive_result = adaptive_optimizer.analyze_bitboard(test_bitboard);
        let fixed_result = fixed_optimizer.analyze_bitboard(test_bitboard);

        // Results should be consistent regardless of algorithm selection
        assert_eq!(adaptive_result, fixed_result);
    }

    #[test]
    fn test_edge_cases() {
        let optimizer = BitScanningOptimizer::new();

        // Test empty bitboard
        let (popcount, first_bit, last_bit) = optimizer.analyze_bitboard(0);
        assert_eq!(popcount, 0);
        assert_eq!(first_bit, None);
        assert_eq!(last_bit, None);

        // Test single bit
        let (popcount, first_bit, last_bit) = optimizer.analyze_bitboard(1);
        assert_eq!(popcount, 1);
        assert_eq!(first_bit, Some(0));
        assert_eq!(last_bit, Some(0));

        // Test all bits set
        let all_bits = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128;
        let (popcount, first_bit, last_bit) = optimizer.analyze_bitboard(all_bits);
        assert_eq!(popcount, 128);
        assert_eq!(first_bit, Some(0));
        assert_eq!(last_bit, Some(127));
    }

    // Task 4.0.4.5: Tests for estimator edge cases
    #[test]
    fn test_estimate_bit_count_empty() {
        let optimizer = BitScanningOptimizer::new();
        let empty = 0u128;
        let count = optimizer.estimate_bit_count(empty);
        assert_eq!(count, 0, "Empty bitboard should have 0 bits");
    }

    #[test]
    fn test_estimate_bit_count_bits_only_in_low_half() {
        let optimizer = BitScanningOptimizer::new();
        // Bits only in low 64 bits
        let low_only = 0x5555555555555555u128;
        let count = optimizer.estimate_bit_count(low_only);
        // Should count 32 bits (one bit per 2 bits in pattern)
        assert_eq!(count, 32, "Low half should have 32 bits set");
    }

    #[test]
    fn test_estimate_bit_count_bits_only_in_high_half() {
        let optimizer = BitScanningOptimizer::new();
        // Bits only in high 64 bits
        let high_only = 0x55555555555555550000000000000000u128;
        let count = optimizer.estimate_bit_count(high_only);
        // Should count 32 bits in high half
        assert_eq!(count, 32, "High half should have 32 bits set");
    }

    #[test]
    fn test_estimate_bit_count_dense_board() {
        let optimizer = BitScanningOptimizer::new();
        // Dense board with many bits set
        let dense = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128;
        let count = optimizer.estimate_bit_count(dense);
        assert_eq!(count, 128, "Dense board should have all 128 bits set");
    }

    #[test]
    fn test_estimate_bit_count_sparse_board() {
        let optimizer = BitScanningOptimizer::new();
        // Sparse board with few bits
        let sparse = 0b1010u128; // Only 2 bits set
        let count = optimizer.estimate_bit_count(sparse);
        assert_eq!(count, 2, "Sparse board should have 2 bits set");
    }

    #[test]
    fn test_strategy_counters() {
        let optimizer = BitScanningOptimizer::new();
        
        // Perform some operations to generate counters
        optimizer.popcount(0b1010);
        optimizer.bit_scan_forward(0b1000);
        optimizer.get_all_bit_positions(0b1111);
        
        let counters = optimizer.get_strategy_counters();
        // At least some counters should be non-zero
        let total = counters.popcount_hardware + counters.popcount_4bit + counters.popcount_swar
            + counters.popcount_debruijn + counters.bitscan_hardware + counters.bitscan_debruijn
            + counters.positions_4bit + counters.positions_debruijn + counters.positions_optimized;
        assert!(total > 0, "Strategy counters should track usage");
        
        // Test reset
        optimizer.reset_counters();
        let counters_after_reset = optimizer.get_strategy_counters();
        assert_eq!(counters_after_reset.popcount_hardware, 0);
        assert_eq!(counters_after_reset.bitscan_hardware, 0);
    }

    #[test]
    fn test_adaptive_selection_with_different_densities() {
        let optimizer = BitScanningOptimizer::new();
        
        // Test with different bit densities to verify adaptive selection
        let sparse = 0b1010u128; // 2 bits
        let medium = 0x5555555555555555u128; // 32 bits
        let dense = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128; // 128 bits
        
        // All should produce correct results regardless of density
        assert_eq!(optimizer.popcount(sparse), 2);
        assert_eq!(optimizer.popcount(medium), 32);
        assert_eq!(optimizer.popcount(dense), 128);
    }
}
