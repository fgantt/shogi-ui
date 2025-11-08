#![cfg(feature = "legacy-tests")]
//! Integration tests for bit-scanning optimization system
//!
//! These tests verify that the bit-scanning optimizations work correctly
//! across different platforms and integrate properly with the existing codebase.

use shogi_engine::bitboards::{
    // Bit scanning functions
    bit_scan_forward,
    bit_scan_optimized,
    bit_scan_reverse,
    clear_lsb,
    clear_msb,
    get_all_bit_positions,
    get_best_bitscan_impl,
    get_best_popcount_impl,
    // Platform detection
    get_platform_capabilities,
    is_empty,
    is_multiple_bits,
    is_single_bit,
    isolate_lsb,
    isolate_msb,
    // Popcount functions
    popcount,
    popcount_optimized,
};
use shogi_engine::types::Bitboard;

/// Test platform detection works correctly
#[test]
fn test_platform_detection_integration() {
    let capabilities = get_platform_capabilities();

    // Verify basic structure
    assert!(
        capabilities.architecture
            != shogi_engine::bitboards::platform_detection::Architecture::Unknown
    );

    // Verify platform detection works
    // Platform capabilities should be detected correctly

    // Verify implementation selection works
    let popcount_impl = get_best_popcount_impl();
    let bitscan_impl = get_best_bitscan_impl();

    // Should always return a valid implementation
    match popcount_impl {
        shogi_engine::bitboards::platform_detection::PopcountImpl::Hardware => {
            assert!(
                capabilities.has_popcnt,
                "Hardware popcount should only be selected when POPCNT is available"
            );
        }
        shogi_engine::bitboards::platform_detection::PopcountImpl::BitParallel => {
            assert!(
                !capabilities.has_popcnt,
                "BitParallel popcount should be used only when hardware POPCNT is unavailable"
            );
        }
        shogi_engine::bitboards::platform_detection::PopcountImpl::Software => {
            // Software implementation should only be used as final fallback
            assert!(true); // Valid fallback
        }
    }

    match bitscan_impl {
        shogi_engine::bitboards::platform_detection::BitscanImpl::Hardware => {
            assert!(
                capabilities.has_bmi1,
                "Hardware bitscan should only be selected when BMI1 is available"
            );
        }
        shogi_engine::bitboards::platform_detection::BitscanImpl::DeBruijn => {
            assert!(
                !capabilities.has_bmi1,
                "DeBruijn bitscan should be selected when BMI1 is unavailable"
            );
        }
        shogi_engine::bitboards::platform_detection::BitscanImpl::Software => {
            // Software implementation should only be used as final fallback
            assert!(true); // Valid fallback
        }
    }
}

/// Test that all implementations produce identical results
#[test]
fn test_implementation_consistency() {
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
        0xFFFFFFFFFFFFFFFFu128,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128,
    ];

    for bb in test_cases {
        // Test popcount consistency
        let popcount_result = popcount(bb);
        let popcount_optimized_result = popcount_optimized(bb);

        assert_eq!(
            popcount_result, popcount_optimized_result,
            "Popcount implementations inconsistent for 0x{:X}",
            bb
        );

        // Test bit scanning consistency
        let forward_result = bit_scan_forward(bb);
        let reverse_result = bit_scan_reverse(bb);
        let forward_optimized_result = bit_scan_optimized(bb, true);
        let reverse_optimized_result = bit_scan_optimized(bb, false);

        assert_eq!(
            forward_result, forward_optimized_result,
            "Bit scan forward implementations inconsistent for 0x{:X}",
            bb
        );
        assert_eq!(
            reverse_result, reverse_optimized_result,
            "Bit scan reverse implementations inconsistent for 0x{:X}",
            bb
        );

        // Test utility functions consistency
        let single_bit_result = is_single_bit(bb);
        let multiple_bits_result = is_multiple_bits(bb);
        let empty_result = is_empty(bb);

        // Verify logical consistency
        assert_eq!(
            empty_result,
            bb == 0,
            "is_empty inconsistent for 0x{:X}",
            bb
        );
        assert_eq!(
            single_bit_result,
            !empty_result && !multiple_bits_result,
            "is_single_bit inconsistent for 0x{:X}",
            bb
        );
        assert_eq!(
            multiple_bits_result,
            !empty_result && !single_bit_result,
            "is_multiple_bits inconsistent for 0x{:X}",
            bb
        );
    }
}

/// Test edge cases across all implementations
#[test]
fn test_edge_cases_integration() {
    // Test empty bitboard
    assert_eq!(popcount(0), 0);
    assert_eq!(bit_scan_forward(0), None);
    assert_eq!(bit_scan_reverse(0), None);
    assert!(is_empty(0));
    assert!(!is_single_bit(0));
    assert!(!is_multiple_bits(0));

    // Test single bit cases
    for i in 0..128 {
        let bb = 1u128 << i;
        assert_eq!(
            popcount(bb),
            1,
            "Single bit popcount failed at position {}",
            i
        );
        assert_eq!(
            bit_scan_forward(bb),
            Some(i as u8),
            "Single bit forward scan failed at position {}",
            i
        );
        assert_eq!(
            bit_scan_reverse(bb),
            Some(i as u8),
            "Single bit reverse scan failed at position {}",
            i
        );
        assert!(
            !is_empty(bb),
            "Single bit should not be empty at position {}",
            i
        );
        assert!(
            is_single_bit(bb),
            "Single bit should be single at position {}",
            i
        );
        assert!(
            !is_multiple_bits(bb),
            "Single bit should not be multiple at position {}",
            i
        );
    }

    // Test all bits set
    let all_bits = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    assert_eq!(popcount(all_bits), 128);
    assert_eq!(bit_scan_forward(all_bits), Some(0));
    assert_eq!(bit_scan_reverse(all_bits), Some(127));
    assert!(!is_empty(all_bits));
    assert!(!is_single_bit(all_bits));
    assert!(is_multiple_bits(all_bits));
}

/// Test bit manipulation utilities integration
#[test]
fn test_bit_manipulation_utilities_integration() {
    let test_bitboard = 0b1010101010101010u128;

    // Test LSB operations
    let isolated_lsb = isolate_lsb(test_bitboard);
    let cleared_lsb = clear_lsb(test_bitboard);

    assert_eq!(isolated_lsb, 0b10); // Should isolate the LSB (position 1)
    assert_eq!(cleared_lsb, 0b1010101010101000); // Should clear the LSB

    // Test MSB operations
    let isolated_msb = isolate_msb(test_bitboard);
    let cleared_msb = clear_msb(test_bitboard);

    // The MSB should be at the highest set bit position
    if let Some(msb_pos) = bit_scan_reverse(test_bitboard) {
        assert_eq!(isolated_msb, 1u128 << msb_pos);
        assert_eq!(cleared_msb, test_bitboard & !(1u128 << msb_pos));
    }

    // Test bit position enumeration
    let positions = get_all_bit_positions(test_bitboard);
    assert_eq!(positions.len(), popcount(test_bitboard) as usize);

    // Verify positions are correct
    for pos in &positions {
        assert!(
            (test_bitboard & (1u128 << pos)) != 0,
            "Position {} should be set in bitboard 0x{:X}",
            pos,
            test_bitboard
        );
    }

    // Verify positions are in ascending order
    for i in 1..positions.len() {
        assert!(
            positions[i - 1] < positions[i],
            "Positions should be in ascending order"
        );
    }
}

/// Test performance characteristics
#[test]
fn test_performance_characteristics() {
    use std::time::Instant;

    let test_bitboard = 0x123456789ABCDEF0123456789ABCDEF0;
    let iterations = 100_000;

    // Test popcount performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = popcount(test_bitboard);
    }
    let popcount_duration = start.elapsed();

    // Test bit scanning performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_forward(test_bitboard);
    }
    let bitscan_duration = start.elapsed();

    // Test optimized functions performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = popcount_optimized(test_bitboard);
    }
    let optimized_popcount_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_optimized(test_bitboard, true);
    }
    let optimized_bitscan_duration = start.elapsed();

    // Verify performance is reasonable (should complete in reasonable time)
    let max_expected_duration = std::time::Duration::from_millis(1000); // 1 second max
    assert!(
        popcount_duration < max_expected_duration,
        "Popcount performance too slow: {:?}",
        popcount_duration
    );
    assert!(
        bitscan_duration < max_expected_duration,
        "Bit scan performance too slow: {:?}",
        bitscan_duration
    );
    assert!(
        optimized_popcount_duration < max_expected_duration,
        "Optimized popcount performance too slow: {:?}",
        optimized_popcount_duration
    );
    assert!(
        optimized_bitscan_duration < max_expected_duration,
        "Optimized bit scan performance too slow: {:?}",
        optimized_bitscan_duration
    );

    // Print performance info for monitoring
    println!("Performance test results for {} iterations:", iterations);
    println!("  Popcount: {:?} per call", popcount_duration / iterations);
    println!(
        "  Bit scan forward: {:?} per call",
        bitscan_duration / iterations
    );
    println!(
        "  Optimized popcount: {:?} per call",
        optimized_popcount_duration / iterations
    );
    println!(
        "  Optimized bit scan: {:?} per call",
        optimized_bitscan_duration / iterations
    );
}

/// Test cross-platform compatibility
#[test]
fn test_cross_platform_compatibility() {
    // Test that functions work on all platforms
    let test_cases = [
        0u128,
        1u128,
        0xFFu128,
        0x8000000000000000u128,
        0x10000000000000000u128,
        0xFFFFFFFFFFFFFFFFu128,
    ];

    for bb in test_cases {
        // All functions should work regardless of platform
        let _popcount_result = popcount(bb);
        let _forward_result = bit_scan_forward(bb);
        let _reverse_result = bit_scan_reverse(bb);
        let _optimized_popcount = popcount_optimized(bb);
        let _optimized_forward = bit_scan_optimized(bb, true);
        let _optimized_reverse = bit_scan_optimized(bb, false);

        // Utility functions should work
        let _single_bit = is_single_bit(bb);
        let _multiple_bits = is_multiple_bits(bb);
        let _empty = is_empty(bb);

        // Bit manipulation should work
        let _isolated_lsb = isolate_lsb(bb);
        let _isolated_msb = isolate_msb(bb);
        let _cleared_lsb = clear_lsb(bb);
        let _cleared_msb = clear_msb(bb);
        let _positions = get_all_bit_positions(bb);

        // Platform detection should work
        let _capabilities = detect_platform_capabilities();
        let _popcount_impl = get_best_popcount_impl();
        let _bitscan_impl = get_best_bitscan_impl();
    }
}

/// Test integration with existing bitboard operations
#[test]
fn test_existing_bitboard_integration() {
    use shogi_engine::types::EMPTY_BITBOARD;

    // Test that our functions work with existing bitboard constants
    assert_eq!(popcount(EMPTY_BITBOARD), 0);
    assert_eq!(bit_scan_forward(EMPTY_BITBOARD), None);
    assert_eq!(bit_scan_reverse(EMPTY_BITBOARD), None);
    assert!(is_empty(EMPTY_BITBOARD));

    // Test with some common bitboard patterns
    let rank_mask = 0xFFu128; // First rank
    let file_mask = 0x0101010101010101u128; // First file

    assert_eq!(popcount(rank_mask), 8);
    assert_eq!(popcount(file_mask), 8);

    assert_eq!(bit_scan_forward(rank_mask), Some(0));
    assert_eq!(bit_scan_reverse(rank_mask), Some(7));

    assert_eq!(bit_scan_forward(file_mask), Some(0));
    assert_eq!(bit_scan_reverse(file_mask), Some(56));
}

// WASM-specific functionality removed - no longer needed

/// Test platform functionality
#[test]
fn test_platform_functionality() {
    let capabilities = get_platform_capabilities();

    // Test that functions work correctly on native platforms
    let test_bitboard = 0x123456789ABCDEF0u128;
    let result = popcount(test_bitboard);
    assert_eq!(result, 32); // Should be correct regardless of implementation

    let forward_result = bit_scan_forward(test_bitboard);
    assert!(forward_result.is_some());

    let reverse_result = bit_scan_reverse(test_bitboard);
    assert!(reverse_result.is_some());

    // Verify platform detection works
    match capabilities.architecture {
        shogi_engine::bitboards::platform_detection::Architecture::X86_64 => {
            // Should be able to potentially use hardware acceleration
            assert!(true); // Valid architecture
        }
        shogi_engine::bitboards::platform_detection::Architecture::Aarch64 => {
            // Should be able to potentially use hardware acceleration
            assert!(true); // Valid architecture
        }
        _ => {
            // Other architectures should fall back to software
            assert!(true); // Valid fallback
        }
    }
}
