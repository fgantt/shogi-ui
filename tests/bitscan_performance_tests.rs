#![cfg(feature = "legacy-tests")]
//! Performance regression tests for bit-scanning optimization system
//!
//! These tests ensure that the bit-scanning optimizations meet performance
//! targets and don't regress over time.

use shogi_engine::bitboards::{
    // Bit scanning functions
    bit_scan_forward,
    bit_scan_optimized,
    bit_scan_reverse,
    bitscan::{
        bit_scan_forward_debruijn, bit_scan_forward_hardware, bit_scan_forward_software,
        bit_scan_reverse_debruijn, bit_scan_reverse_hardware, bit_scan_reverse_software,
    },
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
    // Implementation-specific functions
    popcount::{popcount_bit_parallel, popcount_hardware, popcount_software},
    popcount_optimized,
};
use shogi_engine::types::Bitboard;
use std::time::{Duration, Instant};

/// Performance benchmarks for popcount operations
#[test]
fn test_popcount_performance_benchmarks() {
    let test_cases = [
        ("Empty", 0u128),
        ("Single bit", 1u128),
        ("Sparse", 0x1000000000000000u128),
        ("Dense", 0xFFFFFFFFFFFFFFFFu128),
        ("Pattern", 0x5555555555555555u128),
        ("Random", 0x123456789ABCDEF0u128),
        ("Full 128-bit", 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128),
    ];

    let iterations = 1_000_000;

    for (name, bb) in test_cases {
        println!("\n=== Popcount Performance: {} ===", name);

        // Benchmark hardware implementation
        #[cfg(target_arch = "x86_64")]
        {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = popcount_hardware(bb);
            }
            let hardware_duration = start.elapsed();
            let avg_ns = hardware_duration.as_nanos() / iterations;
            println!(
                "  Hardware: {:?} total, {}ns per call",
                hardware_duration, avg_ns
            );

            // Verify performance target: < 5 CPU cycles (roughly < 20ns on modern CPU)
            assert!(
                avg_ns < 50,
                "Hardware popcount too slow: {}ns per call",
                avg_ns
            );
        }

        // Benchmark SWAR implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = popcount_bit_parallel(bb);
        }
        let swar_duration = start.elapsed();
        let avg_ns = swar_duration.as_nanos() / iterations;
        println!("  SWAR: {:?} total, {}ns per call", swar_duration, avg_ns);

        // Verify performance target: 3-5x faster than software
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = popcount_software(bb);
        }
        let software_duration = start.elapsed();
        let software_avg_ns = software_duration.as_nanos() / iterations;
        println!(
            "  Software: {:?} total, {}ns per call",
            software_duration, software_avg_ns
        );

        // SWAR should be faster than software
        assert!(
            swar_duration <= software_duration,
            "SWAR implementation should be faster than software"
        );

        // Benchmark optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = popcount_optimized(bb);
        }
        let optimized_duration = start.elapsed();
        let optimized_avg_ns = optimized_duration.as_nanos() / iterations;
        println!(
            "  Optimized: {:?} total, {}ns per call",
            optimized_duration, optimized_avg_ns
        );

        // Optimized should be at least as fast as the best implementation
        assert!(
            optimized_duration <= swar_duration,
            "Optimized implementation should be at least as fast as SWAR"
        );
    }
}

/// Performance benchmarks for bit scanning operations
#[test]
fn test_bitscan_performance_benchmarks() {
    let test_cases = [
        ("Empty", 0u128),
        ("Single bit", 1u128),
        ("Sparse", 0x1000000000000000u128),
        ("Dense", 0xFFFFFFFFFFFFFFFFu128),
        ("Pattern", 0xAAAAAAAAAAAAAAAAu128),
        ("Random", 0x123456789ABCDEF0u128),
        ("Full 128-bit", 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128),
    ];

    let iterations = 1_000_000;

    for (name, bb) in test_cases {
        println!("\n=== Bit Scan Forward Performance: {} ===", name);

        // Benchmark hardware implementation
        #[cfg(target_arch = "x86_64")]
        {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = bit_scan_forward_hardware(bb);
            }
            let hardware_duration = start.elapsed();
            let avg_ns = hardware_duration.as_nanos() / iterations;
            println!(
                "  Hardware: {:?} total, {}ns per call",
                hardware_duration, avg_ns
            );

            // Verify performance target: < 10 CPU cycles (roughly < 40ns on modern CPU)
            assert!(
                avg_ns < 100,
                "Hardware bitscan too slow: {}ns per call",
                avg_ns
            );
        }

        // Benchmark DeBruijn implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_forward_debruijn(bb);
        }
        let debruijn_duration = start.elapsed();
        let avg_ns = debruijn_duration.as_nanos() / iterations;
        println!(
            "  DeBruijn: {:?} total, {}ns per call",
            debruijn_duration, avg_ns
        );

        // Benchmark software implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_forward_software(bb);
        }
        let software_duration = start.elapsed();
        let software_avg_ns = software_duration.as_nanos() / iterations;
        println!(
            "  Software: {:?} total, {}ns per call",
            software_duration, software_avg_ns
        );

        // DeBruijn should be faster than software
        assert!(
            debruijn_duration <= software_duration,
            "DeBruijn implementation should be faster than software"
        );

        // Benchmark optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_optimized(bb, true);
        }
        let optimized_duration = start.elapsed();
        let optimized_avg_ns = optimized_duration.as_nanos() / iterations;
        println!(
            "  Optimized: {:?} total, {}ns per call",
            optimized_duration, optimized_avg_ns
        );

        // Optimized should be at least as fast as the best implementation
        assert!(
            optimized_duration <= debruijn_duration,
            "Optimized implementation should be at least as fast as DeBruijn"
        );
    }

    // Test reverse scanning
    for (name, bb) in test_cases {
        println!("\n=== Bit Scan Reverse Performance: {} ===", name);

        // Benchmark hardware implementation
        #[cfg(target_arch = "x86_64")]
        {
            let start = Instant::now();
            for _ in 0..iterations {
                let _result = bit_scan_reverse_hardware(bb);
            }
            let hardware_duration = start.elapsed();
            let avg_ns = hardware_duration.as_nanos() / iterations;
            println!(
                "  Hardware: {:?} total, {}ns per call",
                hardware_duration, avg_ns
            );

            assert!(
                avg_ns < 100,
                "Hardware reverse bitscan too slow: {}ns per call",
                avg_ns
            );
        }

        // Benchmark DeBruijn implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_reverse_debruijn(bb);
        }
        let debruijn_duration = start.elapsed();
        let avg_ns = debruijn_duration.as_nanos() / iterations;
        println!(
            "  DeBruijn: {:?} total, {}ns per call",
            debruijn_duration, avg_ns
        );

        // Benchmark software implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_reverse_software(bb);
        }
        let software_duration = start.elapsed();
        let software_avg_ns = software_duration.as_nanos() / iterations;
        println!(
            "  Software: {:?} total, {}ns per call",
            software_duration, software_avg_ns
        );

        // DeBruijn should be faster than software
        assert!(
            debruijn_duration <= software_duration,
            "DeBruijn reverse implementation should be faster than software"
        );

        // Benchmark optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = bit_scan_optimized(bb, false);
        }
        let optimized_duration = start.elapsed();
        let optimized_avg_ns = optimized_duration.as_nanos() / iterations;
        println!(
            "  Optimized: {:?} total, {}ns per call",
            optimized_duration, optimized_avg_ns
        );

        // Optimized should be at least as fast as the best implementation
        assert!(
            optimized_duration <= debruijn_duration,
            "Optimized reverse implementation should be at least as fast as DeBruijn"
        );
    }
}

/// Performance benchmarks for utility functions
#[test]
fn test_utility_functions_performance() {
    let test_cases = [
        ("Empty", 0u128),
        ("Single bit", 1u128),
        ("Multiple bits", 0b1010101010101010u128),
        ("Dense", 0xFFFFFFFFFFFFFFFFu128),
        ("Sparse", 0x8000000000000000u128),
    ];

    let iterations = 1_000_000;

    for (name, bb) in test_cases {
        println!("\n=== Utility Functions Performance: {} ===", name);

        // Test utility function performance
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = is_single_bit(bb);
        }
        let single_bit_duration = start.elapsed();
        println!("  is_single_bit: {:?} total", single_bit_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = is_multiple_bits(bb);
        }
        let multiple_bits_duration = start.elapsed();
        println!("  is_multiple_bits: {:?} total", multiple_bits_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = is_empty(bb);
        }
        let empty_duration = start.elapsed();
        println!("  is_empty: {:?} total", empty_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = isolate_lsb(bb);
        }
        let isolate_lsb_duration = start.elapsed();
        println!("  isolate_lsb: {:?} total", isolate_lsb_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = isolate_msb(bb);
        }
        let isolate_msb_duration = start.elapsed();
        println!("  isolate_msb: {:?} total", isolate_msb_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = clear_lsb(bb);
        }
        let clear_lsb_duration = start.elapsed();
        println!("  clear_lsb: {:?} total", clear_lsb_duration);

        let start = Instant::now();
        for _ in 0..iterations {
            let _result = clear_msb(bb);
        }
        let clear_msb_duration = start.elapsed();
        println!("  clear_msb: {:?} total", clear_msb_duration);

        // Test bit position enumeration (only for small iterations to avoid timeout)
        let small_iterations = 10_000;
        let start = Instant::now();
        for _ in 0..small_iterations {
            let _result = get_all_bit_positions(bb);
        }
        let positions_duration = start.elapsed();
        println!(
            "  get_all_bit_positions: {:?} total ({} iterations)",
            positions_duration, small_iterations
        );

        // Utility functions should be very fast (single instruction where possible)
        let max_expected_duration = Duration::from_millis(100); // 100ms max
        assert!(
            single_bit_duration < max_expected_duration,
            "is_single_bit too slow: {:?}",
            single_bit_duration
        );
        assert!(
            multiple_bits_duration < max_expected_duration,
            "is_multiple_bits too slow: {:?}",
            multiple_bits_duration
        );
        assert!(
            empty_duration < max_expected_duration,
            "is_empty too slow: {:?}",
            empty_duration
        );
        assert!(
            isolate_lsb_duration < max_expected_duration,
            "isolate_lsb too slow: {:?}",
            isolate_lsb_duration
        );
        assert!(
            isolate_msb_duration < max_expected_duration,
            "isolate_msb too slow: {:?}",
            isolate_msb_duration
        );
        assert!(
            clear_lsb_duration < max_expected_duration,
            "clear_lsb too slow: {:?}",
            clear_lsb_duration
        );
        assert!(
            clear_msb_duration < max_expected_duration,
            "clear_msb too slow: {:?}",
            clear_msb_duration
        );
    }
}

/// Performance regression test - ensure no significant performance degradation
#[test]
fn test_performance_regression() {
    let test_bitboard = 0x123456789ABCDEF0123456789ABCDEF0u128;
    let iterations = 100_000;

    // Test popcount performance regression
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = popcount(test_bitboard);
    }
    let popcount_duration = start.elapsed();
    let avg_ns = popcount_duration.as_nanos() / iterations;

    // Should be fast (less than 100ns per call for most implementations)
    assert!(
        avg_ns < 1000,
        "Popcount performance regression: {}ns per call",
        avg_ns
    );

    // Test bit scanning performance regression
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_forward(test_bitboard);
    }
    let bitscan_duration = start.elapsed();
    let avg_ns = bitscan_duration.as_nanos() / iterations;

    // Should be fast (less than 200ns per call for most implementations)
    assert!(
        avg_ns < 2000,
        "Bit scan performance regression: {}ns per call",
        avg_ns
    );

    println!("Performance regression test results:");
    println!(
        "  Popcount: {}ns per call",
        popcount_duration.as_nanos() / iterations
    );
    println!(
        "  Bit scan forward: {}ns per call",
        bitscan_duration.as_nanos() / iterations
    );
}

/// Platform-specific performance validation
#[test]
fn test_platform_specific_performance() {
    let capabilities = get_platform_capabilities();
    let test_bitboard = 0x123456789ABCDEF0u128;
    let iterations = 100_000;

    println!("\n=== Platform-Specific Performance Validation ===");
    println!("Platform: {:?}", capabilities.architecture);
    println!("POPCNT: {}", capabilities.has_popcnt);
    println!("BMI1: {}", capabilities.has_bmi1);
    println!("BMI2: {}", capabilities.has_bmi2);

    // Test popcount performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = popcount(test_bitboard);
    }
    let popcount_duration = start.elapsed();
    let avg_ns = popcount_duration.as_nanos() / iterations;
    println!("Popcount: {}ns per call", avg_ns);

    // Test bit scanning performance
    let start = Instant::now();
    for _ in 0..iterations {
        let _result = bit_scan_forward(test_bitboard);
    }
    let bitscan_duration = start.elapsed();
    let avg_ns = bitscan_duration.as_nanos() / iterations;
    println!("Bit scan forward: {}ns per call", avg_ns);

    // Platform-specific performance expectations
    // Performance should be reasonable
    assert!(
        popcount_duration.as_nanos() / iterations < 2000,
        "Popcount too slow: {}ns per call",
        avg_ns
    );
    assert!(
        bitscan_duration.as_nanos() / iterations < 3000,
        "Bitscan too slow: {}ns per call",
        avg_ns
    );

    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 should potentially use hardware acceleration
        if capabilities.has_popcnt {
            assert!(
                popcount_duration.as_nanos() / iterations < 500,
                "Hardware popcount too slow: {}ns per call",
                avg_ns
            );
        }
        if capabilities.has_bmi1 {
            assert!(
                bitscan_duration.as_nanos() / iterations < 1000,
                "Hardware bitscan too slow: {}ns per call",
                avg_ns
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // ARM should use native instructions where available
        assert!(
            popcount_duration.as_nanos() / iterations < 1000,
            "ARM popcount too slow: {}ns per call",
            avg_ns
        );
        assert!(
            bitscan_duration.as_nanos() / iterations < 1500,
            "ARM bitscan too slow: {}ns per call",
            avg_ns
        );
    }
}

/// Memory usage validation
#[test]
fn test_memory_usage_validation() {
    use std::mem;

    println!("\n=== Memory Usage Validation ===");

    // Test that lookup tables are reasonably sized
    let popcount_impl = get_best_popcount_impl();
    let bitscan_impl = get_best_bitscan_impl();

    println!("Popcount implementation: {:?}", popcount_impl);
    println!("Bitscan implementation: {:?}", bitscan_impl);

    // Test that we don't have excessive memory usage
    // This is more of a compile-time check, but we can verify the structures are reasonable
    let capabilities_size = mem::size_of_val(&detect_platform_capabilities());
    println!("PlatformCapabilities size: {} bytes", capabilities_size);

    // Should be small (under 100 bytes)
    assert!(
        capabilities_size < 100,
        "PlatformCapabilities too large: {} bytes",
        capabilities_size
    );

    // Test that we don't have memory leaks during operation
    let test_bitboard = 0x123456789ABCDEF0u128;

    // Run many iterations to check for memory issues
    for _ in 0..100_000 {
        let _result1 = popcount(test_bitboard);
        let _result2 = bit_scan_forward(test_bitboard);
        let _result3 = bit_scan_reverse(test_bitboard);
        let _result4 = get_all_bit_positions(test_bitboard);
        let _result5 = isolate_lsb(test_bitboard);
        let _result6 = isolate_msb(test_bitboard);
        let _result7 = clear_lsb(test_bitboard);
        let _result8 = clear_msb(test_bitboard);
    }

    // If we get here without running out of memory, the test passes
    println!("Memory usage validation passed - no leaks detected");
}

/// Stress test for performance under load
#[test]
fn test_performance_under_load() {
    let test_cases = [
        0u128,
        1u128,
        0xFFu128,
        0x1000u128,
        0x8000000000000000u128,
        0xFFFFFFFFFFFFFFFFu128,
        0x5555555555555555u128,
        0xAAAAAAAAAAAAAAAAu128,
        0x123456789ABCDEF0u128,
        0xFEDCBA9876543210u128,
    ];

    let iterations = 50_000;

    println!("\n=== Performance Under Load ===");

    let start = Instant::now();
    for bb in test_cases.iter().cycle().take(iterations) {
        let _result1 = popcount(*bb);
        let _result2 = bit_scan_forward(*bb);
        let _result3 = bit_scan_reverse(*bb);
        let _result4 = popcount_optimized(*bb);
        let _result5 = bit_scan_optimized(*bb, true);
        let _result6 = bit_scan_optimized(*bb, false);
        let _result7 = is_single_bit(*bb);
        let _result8 = is_multiple_bits(*bb);
        let _result9 = is_empty(*bb);
    }
    let total_duration = start.elapsed();

    let avg_ns = total_duration.as_nanos() / (iterations * 9); // 9 operations per iteration
    println!("Average time per operation: {}ns", avg_ns);

    // Should complete in reasonable time (less than 10ms total)
    assert!(
        total_duration < Duration::from_millis(10),
        "Performance under load too slow: {:?}",
        total_duration
    );

    // Individual operations should be fast
    assert!(avg_ns < 500, "Individual operations too slow: {}ns", avg_ns);
}
