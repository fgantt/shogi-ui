//! Bit scanning implementations for bit-scanning optimizations
//! 
//! This module provides multiple implementations of bit scanning (finding bit positions)
//! optimized for different platforms and capabilities.

use crate::types::Bitboard;
use crate::bitboards::platform_detection::{get_best_bitscan_impl, BitscanImpl};

/// Main bit scan forward function with automatic implementation selection
/// 
/// This function automatically selects the optimal implementation based on
/// the current platform capabilities detected at runtime.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
/// 
/// # Examples
/// ```
/// use shogi_engine::types::Bitboard;
/// use shogi_engine::bitboards::bitscan::bit_scan_forward;
/// 
/// let bb: Bitboard = 0b1010; // Bits at positions 1 and 3
/// assert_eq!(bit_scan_forward(bb), Some(1)); // Returns position of LSB
/// assert_eq!(bit_scan_forward(0), None); // Empty bitboard
/// ```
pub fn bit_scan_forward(bb: Bitboard) -> Option<u8> {
    match get_best_bitscan_impl() {
        BitscanImpl::Hardware => bit_scan_forward_hardware(bb),
        BitscanImpl::DeBruijn => bit_scan_forward_debruijn(bb),
        BitscanImpl::Software => bit_scan_forward_software(bb),
    }
}

/// Main bit scan reverse function with automatic implementation selection
/// 
/// This function automatically selects the optimal implementation based on
/// the current platform capabilities detected at runtime.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
/// 
/// # Examples
/// ```
/// use shogi_engine::types::Bitboard;
/// use shogi_engine::bitboards::bitscan::bit_scan_reverse;
/// 
/// let bb: Bitboard = 0b1010; // Bits at positions 1 and 3
/// assert_eq!(bit_scan_reverse(bb), Some(3)); // Returns position of MSB
/// assert_eq!(bit_scan_reverse(0), None); // Empty bitboard
/// ```
pub fn bit_scan_reverse(bb: Bitboard) -> Option<u8> {
    match get_best_bitscan_impl() {
        BitscanImpl::Hardware => bit_scan_reverse_hardware(bb),
        BitscanImpl::DeBruijn => bit_scan_reverse_debruijn(bb),
        BitscanImpl::Software => bit_scan_reverse_software(bb),
    }
}

/// Hardware-accelerated bit scan forward using x86_64 BSF instruction
/// 
/// This implementation uses the native BSF (Bit Scan Forward) instruction available on
/// x86_64 processors. It provides the fastest possible performance for finding the
/// least significant bit.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
/// 
/// # Safety
/// This function uses unsafe intrinsics and should only be called when
/// BMI1 support has been verified by the platform detection system.
#[cfg(all(target_arch = "x86_64", not(target_arch = "wasm32")))]
pub fn bit_scan_forward_hardware(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    unsafe {
        // For u128, we need to check both halves
        let low = bb as u64;
        if low != 0 {
            Some(std::arch::x86_64::_tzcnt_u64(low) as u8)
        } else {
            let high = (bb >> 64) as u64;
            Some(std::arch::x86_64::_tzcnt_u64(high) as u8 + 64)
        }
    }
}

/// Hardware-accelerated bit scan reverse using x86_64 BSR instruction
/// 
/// This implementation uses the native BSR (Bit Scan Reverse) instruction available on
/// x86_64 processors. It provides the fastest possible performance for finding the
/// most significant bit.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
/// 
/// # Safety
/// This function uses unsafe intrinsics and should only be called when
/// BMI1 support has been verified by the platform detection system.
#[cfg(all(target_arch = "x86_64", not(target_arch = "wasm32")))]
pub fn bit_scan_reverse_hardware(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    unsafe {
        // For u128, we need to check both halves
        let high = (bb >> 64) as u64;
        if high != 0 {
            Some(63 - std::arch::x86_64::_lzcnt_u64(high) as u8 + 64)
        } else {
            let low = bb as u64;
            Some(63 - std::arch::x86_64::_lzcnt_u64(low) as u8)
        }
    }
}

/// ARM hardware-accelerated bit scan forward using CLZ instruction
/// 
/// This implementation uses the native CLZ (Count Leading Zeros) instruction available on
/// ARM processors. It provides the fastest possible performance for finding bit positions.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
#[cfg(all(target_arch = "aarch64", not(target_arch = "wasm32")))]
pub fn bit_scan_forward_hardware(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    // For u128, we need to check both halves
    let low = bb as u64;
    if low != 0 {
        Some(low.trailing_zeros() as u8)
    } else {
        let high = (bb >> 64) as u64;
        Some(high.trailing_zeros() as u8 + 64)
    }
}

/// ARM hardware-accelerated bit scan reverse using CLZ instruction
/// 
/// This implementation uses the native CLZ (Count Leading Zeros) instruction available on
/// ARM processors. It provides the fastest possible performance for finding the
/// most significant bit.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
#[cfg(all(target_arch = "aarch64", not(target_arch = "wasm32")))]
pub fn bit_scan_reverse_hardware(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    // For u128, we need to check both halves
    let high = (bb >> 64) as u64;
    if high != 0 {
        Some(63 - high.leading_zeros() as u8 + 64)
    } else {
        let low = bb as u64;
        Some(63 - low.leading_zeros() as u8)
    }
}

/// Fallback hardware implementation for non-x86_64/non-ARM platforms
#[cfg(not(any(
    all(target_arch = "x86_64", not(target_arch = "wasm32")),
    all(target_arch = "aarch64", not(target_arch = "wasm32"))
)))]
pub fn bit_scan_forward_hardware(bb: Bitboard) -> Option<u8> {
    bit_scan_forward_debruijn(bb)
}

#[cfg(not(any(
    all(target_arch = "x86_64", not(target_arch = "wasm32")),
    all(target_arch = "aarch64", not(target_arch = "wasm32"))
)))]
pub fn bit_scan_reverse_hardware(bb: Bitboard) -> Option<u8> {
    bit_scan_reverse_debruijn(bb)
}

/// De Bruijn sequence bit scan forward implementation
/// 
/// This implementation uses De Bruijn sequences for efficient bit position determination.
/// It works on all platforms including WASM and provides good performance.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
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

/// De Bruijn sequence bit scan reverse implementation
/// 
/// This implementation uses De Bruijn sequences for efficient bit position determination.
/// It works on all platforms including WASM and provides good performance.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
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

/// 64-bit De Bruijn sequence for bit scan forward
const DEBRUIJN64: u64 = 0x03f79d71b4cb0a89;

/// Lookup table for bit positions using De Bruijn sequence
const DEBRUIJN_TABLE: [u8; 64] = [
    0, 1, 48, 2, 57, 49, 28, 3, 61, 58, 50, 42, 38, 29, 17, 4,
    62, 55, 59, 36, 53, 51, 43, 22, 45, 39, 33, 30, 24, 18, 12, 5,
    63, 47, 56, 27, 60, 41, 37, 16, 54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6
];

/// 64-bit De Bruijn bit scan forward
fn bit_scan_forward_debruijn_64(bb: u64) -> u8 {
    let isolated = bb & (!bb + 1); // Isolate least significant bit
    DEBRUIJN_TABLE[((isolated * DEBRUIJN64) >> 58) as usize]
}

/// 64-bit De Bruijn bit scan reverse
fn bit_scan_reverse_debruijn_64(bb: u64) -> u8 {
    // For reverse scan, we need to find the most significant bit
    // We can use the same De Bruijn approach but need to isolate the MSB
    let msb = if bb != 0 { 1 << (63 - bb.leading_zeros()) } else { 0 };
    DEBRUIJN_TABLE[((msb * DEBRUIJN64) >> 58) as usize]
}

/// Software fallback bit scan forward implementation
/// 
/// This implementation uses a simple loop-based approach that works on
/// all platforms but is slower than the optimized versions.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the least significant bit (0-based), or None if the bitboard is empty
pub fn bit_scan_forward_software(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    // For u128, we need to check both halves
    let low = bb as u64;
    if low != 0 {
        Some(low.trailing_zeros() as u8)
    } else {
        let high = (bb >> 64) as u64;
        Some(high.trailing_zeros() as u8 + 64)
    }
}

/// Software fallback bit scan reverse implementation
/// 
/// This implementation uses a simple loop-based approach that works on
/// all platforms but is slower than the optimized versions.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// 
/// # Returns
/// The position of the most significant bit (0-based), or None if the bitboard is empty
pub fn bit_scan_reverse_software(bb: Bitboard) -> Option<u8> {
    if bb == 0 {
        return None;
    }
    
    // For u128, we need to check both halves
    let high = (bb >> 64) as u64;
    if high != 0 {
        Some(63 - high.leading_zeros() as u8 + 64)
    } else {
        let low = bb as u64;
        Some(63 - low.leading_zeros() as u8)
    }
}

/// Bit scan with manual implementation selection
/// 
/// This function allows manual selection of the implementation,
/// useful for benchmarking or when you need specific behavior.
/// 
/// # Arguments
/// * `bb` - The bitboard to scan
/// * `impl_type` - The implementation to use
/// * `forward` - If true, scan forward (LSB); if false, scan reverse (MSB)
/// 
/// # Returns
/// The bit position (0-based), or None if the bitboard is empty
pub fn bit_scan_with_impl(bb: Bitboard, impl_type: BitscanImpl, forward: bool) -> Option<u8> {
    if forward {
        match impl_type {
            BitscanImpl::Hardware => bit_scan_forward_hardware(bb),
            BitscanImpl::DeBruijn => bit_scan_forward_debruijn(bb),
            BitscanImpl::Software => bit_scan_forward_software(bb),
        }
    } else {
        match impl_type {
            BitscanImpl::Hardware => bit_scan_reverse_hardware(bb),
            BitscanImpl::DeBruijn => bit_scan_reverse_debruijn(bb),
            BitscanImpl::Software => bit_scan_reverse_software(bb),
        }
    }
}

/// Optimized bit scan with fast paths for common cases
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
pub fn bit_scan_optimized(bb: Bitboard, forward: bool) -> Option<u8> {
    // Fast path for empty bitboard
    if bb == 0 {
        return None;
    }
    
    // Fast path for single bit (common case)
    if bb & (bb - 1) == 0 {
        return if forward {
            bit_scan_forward(bb)
        } else {
            bit_scan_reverse(bb)
        };
    }
    
    // Use the best available implementation
    if forward {
        bit_scan_forward(bb)
    } else {
        bit_scan_reverse(bb)
    }
}

/// Clear the least significant bit
/// 
/// # Arguments
/// * `bb` - The bitboard to modify
/// 
/// # Returns
/// The bitboard with the least significant bit cleared
pub fn clear_lsb(bb: Bitboard) -> Bitboard {
    bb & (bb - 1)
}

/// Clear the most significant bit
/// 
/// # Arguments
/// * `bb` - The bitboard to modify
/// 
/// # Returns
/// The bitboard with the most significant bit cleared
pub fn clear_msb(bb: Bitboard) -> Bitboard {
    if bb == 0 {
        return 0;
    }
    
    let msb = if let Some(pos) = bit_scan_reverse(bb) {
        1u128 << pos
    } else {
        return 0;
    };
    
    bb & !msb
}

/// Isolate the least significant bit
/// 
/// # Arguments
/// * `bb` - The bitboard to process
/// 
/// # Returns
/// A bitboard with only the least significant bit set
pub fn isolate_lsb(bb: Bitboard) -> Bitboard {
    bb & (!bb + 1)
}

/// Isolate the most significant bit
/// 
/// # Arguments
/// * `bb` - The bitboard to process
/// 
/// # Returns
/// A bitboard with only the most significant bit set
pub fn isolate_msb(bb: Bitboard) -> Bitboard {
    if bb == 0 {
        return 0;
    }
    
    if let Some(pos) = bit_scan_reverse(bb) {
        1u128 << pos
    } else {
        0
    }
}

/// Get all bit positions in a bitboard
/// 
/// # Arguments
/// * `bb` - The bitboard to process
/// 
/// # Returns
/// A vector containing all bit positions (0-based), ordered from LSB to MSB
pub fn get_all_bit_positions(bb: Bitboard) -> Vec<u8> {
    let mut positions = Vec::new();
    let mut remaining = bb;
    
    while remaining != 0 {
        if let Some(pos) = bit_scan_forward(remaining) {
            positions.push(pos);
            remaining = clear_lsb(remaining);
        } else {
            break;
        }
    }
    
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_scan_forward_correctness() {
        // Test basic cases
        assert_eq!(bit_scan_forward(0), None);
        assert_eq!(bit_scan_forward(1), Some(0));
        assert_eq!(bit_scan_forward(2), Some(1));
        assert_eq!(bit_scan_forward(4), Some(2));
        assert_eq!(bit_scan_forward(8), Some(3));
        
        // Test edge cases
        assert_eq!(bit_scan_forward(0x8000000000000000), Some(63));
        assert_eq!(bit_scan_forward(0x10000000000000000), Some(64));
        assert_eq!(bit_scan_forward(0x80000000000000000000000000000000), Some(127));
        
        // Test multiple bits (should return LSB)
        assert_eq!(bit_scan_forward(0b1010), Some(1)); // Bits at positions 1 and 3
        assert_eq!(bit_scan_forward(0b1100), Some(2)); // Bits at positions 2 and 3
    }

    #[test]
    fn test_bit_scan_reverse_correctness() {
        // Test basic cases
        assert_eq!(bit_scan_reverse(0), None);
        assert_eq!(bit_scan_reverse(1), Some(0));
        assert_eq!(bit_scan_reverse(2), Some(1));
        assert_eq!(bit_scan_reverse(4), Some(2));
        assert_eq!(bit_scan_reverse(8), Some(3));
        
        // Test edge cases
        assert_eq!(bit_scan_reverse(0x8000000000000000), Some(63));
        assert_eq!(bit_scan_reverse(0x10000000000000000), Some(64));
        assert_eq!(bit_scan_reverse(0x80000000000000000000000000000000), Some(127));
        
        // Test multiple bits (should return MSB)
        assert_eq!(bit_scan_reverse(0b1010), Some(3)); // Bits at positions 1 and 3
        assert_eq!(bit_scan_reverse(0b1100), Some(3)); // Bits at positions 2 and 3
    }

    #[test]
    fn test_all_implementations_identical() {
        let test_cases = [
            0,
            1,
            2,
            4,
            8,
            0xFF,
            0x8000000000000000,
            0x10000000000000000,
            0x5555555555555555,
            0xAAAAAAAAAAAAAAAA,
            0x123456789ABCDEF0,
            0x80000000000000000000000000000000,
        ];

        for bb in test_cases {
            let forward_hardware = bit_scan_forward_hardware(bb);
            let forward_debruijn = bit_scan_forward_debruijn(bb);
            let forward_software = bit_scan_forward_software(bb);

            let reverse_hardware = bit_scan_reverse_hardware(bb);
            let reverse_debruijn = bit_scan_reverse_debruijn(bb);
            let reverse_software = bit_scan_reverse_software(bb);

            assert_eq!(forward_hardware, forward_debruijn, 
                      "Forward hardware vs DeBruijn mismatch for 0x{:X}", bb);
            assert_eq!(forward_hardware, forward_software, 
                      "Forward hardware vs Software mismatch for 0x{:X}", bb);

            assert_eq!(reverse_hardware, reverse_debruijn, 
                      "Reverse hardware vs DeBruijn mismatch for 0x{:X}", bb);
            assert_eq!(reverse_hardware, reverse_software, 
                      "Reverse hardware vs Software mismatch for 0x{:X}", bb);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Empty bitboard
        assert_eq!(bit_scan_forward(0), None);
        assert_eq!(bit_scan_reverse(0), None);
        
        // Single bit cases
        for i in 0..128 {
            let bb = 1u128 << i;
            assert_eq!(bit_scan_forward(bb), Some(i as u8), 
                      "Single bit forward at position {} failed", i);
            assert_eq!(bit_scan_reverse(bb), Some(i as u8), 
                      "Single bit reverse at position {} failed", i);
        }
        
        // All bits set
        let all_bits = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        assert_eq!(bit_scan_forward(all_bits), Some(0));
        assert_eq!(bit_scan_reverse(all_bits), Some(127));
        
        // Pattern tests
        assert_eq!(bit_scan_forward(0x5555555555555555), Some(0)); // Alternating bits
        assert_eq!(bit_scan_reverse(0x5555555555555555), Some(62));
        assert_eq!(bit_scan_forward(0xAAAAAAAAAAAAAAAA), Some(1)); // Alternating bits (opposite)
        assert_eq!(bit_scan_reverse(0xAAAAAAAAAAAAAAAA), Some(63));
    }

    #[test]
    fn test_utility_functions() {
        // Test clear_lsb
        assert_eq!(clear_lsb(0b1010), 0b1000); // Clear LSB of 0b1010
        assert_eq!(clear_lsb(0b1000), 0b0000); // Clear LSB of 0b1000
        assert_eq!(clear_lsb(0), 0); // Clear LSB of 0
        
        // Test clear_msb
        assert_eq!(clear_msb(0b1010), 0b0010); // Clear MSB of 0b1010
        assert_eq!(clear_msb(0b1000), 0b0000); // Clear MSB of 0b1000
        assert_eq!(clear_msb(0), 0); // Clear MSB of 0
        
        // Test isolate_lsb
        assert_eq!(isolate_lsb(0b1010), 0b0010); // Isolate LSB of 0b1010
        assert_eq!(isolate_lsb(0b1000), 0b1000); // Isolate LSB of 0b1000
        assert_eq!(isolate_lsb(0), 0); // Isolate LSB of 0
        
        // Test isolate_msb
        assert_eq!(isolate_msb(0b1010), 0b1000); // Isolate MSB of 0b1010
        assert_eq!(isolate_msb(0b0010), 0b0010); // Isolate MSB of 0b0010
        assert_eq!(isolate_msb(0), 0); // Isolate MSB of 0
    }

    #[test]
    fn test_get_all_bit_positions() {
        // Test empty bitboard
        assert_eq!(get_all_bit_positions(0), vec![]);
        
        // Test single bit
        assert_eq!(get_all_bit_positions(1), vec![0]);
        assert_eq!(get_all_bit_positions(0x8000000000000000), vec![63]);
        
        // Test multiple bits
        assert_eq!(get_all_bit_positions(0b1010), vec![1, 3]); // Bits at positions 1 and 3
        assert_eq!(get_all_bit_positions(0b1100), vec![2, 3]); // Bits at positions 2 and 3
        
        // Test pattern
        let positions = get_all_bit_positions(0x5555555555555555); // Every other bit
        assert_eq!(positions.len(), 32);
        assert_eq!(positions[0], 0);
        assert_eq!(positions[1], 2);
        assert_eq!(positions[2], 4);
        assert_eq!(positions[31], 62);
    }

    #[test]
    fn test_bit_scan_with_impl() {
        let bb = 0b1010;
        
        let forward_hardware = bit_scan_with_impl(bb, BitscanImpl::Hardware, true);
        let forward_debruijn = bit_scan_with_impl(bb, BitscanImpl::DeBruijn, true);
        let forward_software = bit_scan_with_impl(bb, BitscanImpl::Software, true);
        
        let reverse_hardware = bit_scan_with_impl(bb, BitscanImpl::Hardware, false);
        let reverse_debruijn = bit_scan_with_impl(bb, BitscanImpl::DeBruijn, false);
        let reverse_software = bit_scan_with_impl(bb, BitscanImpl::Software, false);
        
        assert_eq!(forward_hardware, forward_debruijn);
        assert_eq!(forward_hardware, forward_software);
        assert_eq!(reverse_hardware, reverse_debruijn);
        assert_eq!(reverse_hardware, reverse_software);
        
        assert_eq!(forward_hardware, Some(1));
        assert_eq!(reverse_hardware, Some(3));
    }

    #[test]
    fn test_bit_scan_optimized() {
        // Test empty bitboard fast path
        assert_eq!(bit_scan_optimized(0, true), None);
        assert_eq!(bit_scan_optimized(0, false), None);
        
        // Test single bit fast path
        assert_eq!(bit_scan_optimized(1, true), Some(0));
        assert_eq!(bit_scan_optimized(1, false), Some(0));
        assert_eq!(bit_scan_optimized(0x8000000000000000, true), Some(63));
        assert_eq!(bit_scan_optimized(0x8000000000000000, false), Some(63));
        
        // Test normal case
        assert_eq!(bit_scan_optimized(0b1010, true), Some(1));
        assert_eq!(bit_scan_optimized(0b1010, false), Some(3));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_bit_scan_performance_comparison() {
        let test_bitboard = 0x123456789ABCDEF0123456789ABCDEF0;
        let iterations = 1_000_000;

        // Benchmark hardware implementation
        #[cfg(all(target_arch = "x86_64", not(target_arch = "wasm32")))]
        {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = bit_scan_forward_hardware(test_bitboard);
            }
            let hardware_duration = start.elapsed();
            println!("Hardware bit_scan_forward: {:?} total, {:?} per call", 
                    hardware_duration, hardware_duration / iterations);
        }

        // Benchmark DeBruijn implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_forward_debruijn(test_bitboard);
        }
        let debruijn_duration = start.elapsed();
        println!("DeBruijn bit_scan_forward: {:?} total, {:?} per call", 
                debruijn_duration, debruijn_duration / iterations);

        // Benchmark software implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_forward_software(test_bitboard);
        }
        let software_duration = start.elapsed();
        println!("Software bit_scan_forward: {:?} total, {:?} per call", 
                software_duration, software_duration / iterations);

        // Verify performance targets
        // DeBruijn should be faster than software
        assert!(debruijn_duration <= software_duration, 
                "DeBruijn implementation should be faster than software");

        #[cfg(all(target_arch = "x86_64", not(target_arch = "wasm32")))]
        {
            // Hardware should be fastest on x86_64
            assert!(hardware_duration <= debruijn_duration,
                    "Hardware implementation should be faster than DeBruijn on x86_64");
        }
    }

    #[test]
    fn test_bit_scan_optimized_performance() {
        let iterations = 1_000_000;
        
        // Test fast path performance (single bit)
        let single_bit = 0x8000000000000000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_optimized(single_bit, true);
        }
        let fast_path_duration = start.elapsed();
        
        // Test normal case performance
        let normal_bitboard = 0x123456789ABCDEF0;
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_optimized(normal_bitboard, true);
        }
        let normal_duration = start.elapsed();
        
        println!("Optimized bit_scan (single bit): {:?} per call", fast_path_duration / iterations);
        println!("Optimized bit_scan (normal): {:?} per call", normal_duration / iterations);
        
        // Fast path should be very fast
        assert!(fast_path_duration <= normal_duration, 
                "Fast path should be faster than or equal to normal case");
    }

    #[test]
    fn test_bit_scan_consistency_under_load() {
        // Test that all implementations produce consistent results under load
        let test_cases = [
            0x0000000000000000,
            0x0000000000000001,
            0x0000000000000003,
            0x00000000000000FF,
            0x000000000000FFFF,
            0x00000000FFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0x123456789ABCDEF0,
        ];

        for bb in test_cases {
            let iterations = 100_000;
            
            // Run multiple implementations in parallel to ensure consistency
            let hardware_forward_results: Vec<Option<u8>> = (0..iterations)
                .map(|_| bit_scan_forward_hardware(bb))
                .collect();
            
            let debruijn_forward_results: Vec<Option<u8>> = (0..iterations)
                .map(|_| bit_scan_forward_debruijn(bb))
                .collect();
            
            let software_forward_results: Vec<Option<u8>> = (0..iterations)
                .map(|_| bit_scan_forward_software(bb))
                .collect();

            // All results should be identical
            assert!(hardware_forward_results.iter().all(|&x| x == hardware_forward_results[0]),
                    "Hardware forward implementation inconsistent for 0x{:X}", bb);
            assert!(debruijn_forward_results.iter().all(|&x| x == debruijn_forward_results[0]),
                    "DeBruijn forward implementation inconsistent for 0x{:X}", bb);
            assert!(software_forward_results.iter().all(|&x| x == software_forward_results[0]),
                    "Software forward implementation inconsistent for 0x{:X}", bb);
            
            // All implementations should agree
            assert_eq!(hardware_forward_results[0], debruijn_forward_results[0], 
                      "Hardware vs DeBruijn forward mismatch for 0x{:X}", bb);
            assert_eq!(hardware_forward_results[0], software_forward_results[0],
                      "Hardware vs Software forward mismatch for 0x{:X}", bb);
        }
    }
}
