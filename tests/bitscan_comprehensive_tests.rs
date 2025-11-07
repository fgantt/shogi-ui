#![cfg(feature = "legacy-tests")]
//! Comprehensive testing suite for bit-scanning optimization system
//!
//! This module provides comprehensive tests for all bit-scanning optimization
//! functions, including unit tests, integration tests, performance benchmarks,
//! cross-platform testing, and regression testing.

use shogi_engine::bitboards::*;
use shogi_engine::types::{Bitboard, Player, Position};
use std::time::Instant;

/// Comprehensive unit tests for all bit-scanning functions
#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test population count correctness across all implementations
    #[test]
    fn test_popcount_correctness() {
        let test_cases = vec![
            (0u128, 0),
            (1u128, 1),
            (0b1010u128, 2),
            (0b1111u128, 4),
            (0b1010_1010u128, 4),
            (0x1234_5678_9ABC_DEF0u128, 32),
            (0xFFFF_FFFF_FFFF_FFFFu128, 64),
            (!0u128, 128),
        ];

        for (bb, expected) in test_cases {
            // Test standard implementation
            assert_eq!(
                popcount(bb),
                expected,
                "Standard popcount failed for 0x{:X}",
                bb
            );

            // Test cache-optimized implementation
            assert_eq!(
                popcount_cache_optimized(bb),
                expected,
                "Cache-optimized popcount failed for 0x{:X}",
                bb
            );

            // Test branch-optimized implementation
            assert_eq!(
                popcount_branch_optimized(bb),
                expected,
                "Branch-optimized popcount failed for 0x{:X}",
                bb
            );

            // Test critical path implementation
            assert_eq!(
                popcount_critical(bb),
                expected,
                "Critical path popcount failed for 0x{:X}",
                bb
            );
        }
    }

    /// Test bit scan forward correctness across all implementations
    #[test]
    fn test_bit_scan_forward_correctness() {
        let test_cases = vec![
            (0u128, None),
            (1u128, Some(0)),
            (0b1000u128, Some(3)),
            (0b1010u128, Some(1)),
            (0b1000_0000u128, Some(7)),
            (1u128 << 63, Some(63)),
            (1u128 << 64, Some(64)),
            (1u128 << 127, Some(127)),
        ];

        for (bb, expected) in test_cases {
            // Test standard implementation
            assert_eq!(
                bit_scan_forward(bb),
                expected,
                "Standard bit_scan_forward failed for 0x{:X}",
                bb
            );

            // Test branch-optimized implementation
            assert_eq!(
                bit_scan_forward_optimized(bb),
                expected,
                "Branch-optimized bit_scan_forward failed for 0x{:X}",
                bb
            );

            // Test critical path implementation
            assert_eq!(
                bit_scan_forward_critical(bb),
                expected,
                "Critical path bit_scan_forward failed for 0x{:X}",
                bb
            );
        }
    }

    /// Test bit scan reverse correctness across all implementations
    #[test]
    fn test_bit_scan_reverse_correctness() {
        let test_cases = vec![
            (0u128, None),
            (1u128, Some(0)),
            (0b1000u128, Some(3)),
            (0b1010u128, Some(3)),
            (0b1000_0000u128, Some(7)),
            (1u128 << 63, Some(63)),
            (1u128 << 64, Some(64)),
            (1u128 << 127, Some(127)),
        ];

        for (bb, expected) in test_cases {
            // Test standard implementation
            assert_eq!(
                bit_scan_reverse(bb),
                expected,
                "Standard bit_scan_reverse failed for 0x{:X}",
                bb
            );

            // Test branch-optimized implementation
            assert_eq!(
                bit_scan_reverse_optimized(bb),
                expected,
                "Branch-optimized bit_scan_reverse failed for 0x{:X}",
                bb
            );
        }
    }

    /// Test bit manipulation utilities
    #[test]
    fn test_bit_manipulation_utilities() {
        let bb = 0b1010u128;

        // Test bit isolation
        let (lsb, cleared) = extract_lsb(bb);
        assert_eq!(lsb, 0b0010);
        assert_eq!(cleared, 0b1000);

        let (msb, cleared) = extract_msb(bb);
        assert_eq!(msb, 0b1000);
        assert_eq!(cleared, 0b0010);

        // Test bit operations
        assert!(overlaps(0b1010, 0b0101));
        assert!(!overlaps(0b1010, 0b0001));

        assert!(is_subset(0b0010, 0b1111));
        assert!(!is_subset(0b1111, 0b1010));

        // Test set operations
        assert_eq!(intersection(0b1010, 0b0110), 0b0010);
        assert_eq!(union(0b1010, 0b0110), 0b1110);
        assert_eq!(symmetric_difference(0b1010, 0b0110), 0b1100);
        assert_eq!(difference(0b1010, 0b0110), 0b1000);
    }

    /// Test square coordinate conversion utilities
    #[test]
    fn test_square_coordinate_conversion() {
        // Test bit position to square conversion
        let pos = bit_to_square(40); // Center of 9x9 board
        assert_eq!(pos.row, 4);
        assert_eq!(pos.col, 4);

        // Test square to bit position conversion
        let square = Position::new(4, 4);
        assert_eq!(square_to_bit(square), 40);

        // Test algebraic notation
        assert_eq!(bit_to_square_name(40), "5e");
        assert_eq!(square_name_to_bit("5e"), 40);

        // Test coordinate conversion
        let (file, rank) = bit_to_coords(40);
        assert_eq!(file, 4);
        assert_eq!(rank, 4);

        assert_eq!(coords_to_bit(4, 4), 40);
    }

    /// Test Shogi-specific utilities
    #[test]
    fn test_shogi_specific_utilities() {
        // Test promotion zones
        assert!(is_promotion_zone(63, Player::Black)); // Rank 7
        assert!(is_promotion_zone(0, Player::White)); // Rank 1
        assert!(!is_promotion_zone(40, Player::Black)); // Center

        // Test square validation
        assert!(is_valid_shogi_square(40)); // Valid square
        assert!(!is_valid_shogi_square(128)); // Invalid square

        // Test center squares
        let center_squares = get_center_squares();
        assert!(center_squares.contains(&40)); // 5e is center
        assert!(is_center_square(40));
    }

    /// Test common case optimization functions
    #[test]
    fn test_common_case_optimization() {
        // Test empty bitboard detection
        assert!(is_empty_optimized(0));
        assert!(!is_empty_optimized(1));

        // Test single piece detection
        assert!(is_single_piece_optimized(1));
        assert!(is_single_piece_optimized(0b1000));
        assert!(!is_single_piece_optimized(0b1010));

        // Test multiple pieces detection
        assert!(!is_multiple_pieces_optimized(0));
        assert!(!is_multiple_pieces_optimized(1));
        assert!(is_multiple_pieces_optimized(0b1010));
    }

    /// Test cache optimization functions
    #[test]
    fn test_cache_optimization() {
        // Test cache-aligned lookup tables
        assert!(cache_opt::validation::validate_cache_alignment());
        assert!(cache_opt::validation::validate_lookup_tables());

        // Test cache-optimized population count
        assert_eq!(popcount_cache_optimized(0b1010), 2);

        // Test cache-optimized bit positions
        let positions = get_bit_positions_cache_optimized(0b1010);
        assert_eq!(positions, vec![1, 3]);
    }

    /// Test platform detection
    #[test]
    fn test_platform_detection() {
        let caps = get_platform_capabilities();

        // Platform capabilities should be detected
        assert!(caps.has_popcnt || caps.has_bmi1); // Should have hardware acceleration

        // Test best implementation selection
        let popcount_impl = get_best_popcount_impl();
        let bitscan_impl = get_best_bitscan_impl();

        // Should return valid implementations
        assert!(matches!(
            popcount_impl,
            PopcountImpl::Hardware | PopcountImpl::BitParallel | PopcountImpl::Software
        ));
        assert!(matches!(
            bitscan_impl,
            BitscanImpl::Hardware | BitscanImpl::DeBruijn | BitscanImpl::Software
        ));
    }
}

/// Integration tests for bit-scanning system
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test integration between all optimization modules
    #[test]
    fn test_optimization_integration() {
        let test_bitboard = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0u128;

        // Test that all implementations produce consistent results
        let standard_count = popcount(test_bitboard);
        let cache_count = popcount_cache_optimized(test_bitboard);
        let branch_count = popcount_branch_optimized(test_bitboard);
        let critical_count = popcount_critical(test_bitboard);

        assert_eq!(standard_count, cache_count);
        assert_eq!(standard_count, branch_count);
        assert_eq!(standard_count, critical_count);

        // Test bit scanning consistency
        let standard_forward = bit_scan_forward(test_bitboard);
        let branch_forward = bit_scan_forward_optimized(test_bitboard);
        let critical_forward = bit_scan_forward_critical(test_bitboard);

        assert_eq!(standard_forward, branch_forward);
        assert_eq!(standard_forward, critical_forward);
    }

    /// Test API integration and consistency
    #[test]
    fn test_api_integration() {
        let bb = 0b1010u128;

        // Test API module consistency
        assert_eq!(api::bitscan::popcount(bb), popcount(bb));
        assert_eq!(api::bitscan::bit_scan_forward(bb), bit_scan_forward(bb));
        assert_eq!(api::utils::extract_lsb(bb), extract_lsb(bb));
        assert_eq!(api::squares::bit_to_square(1), bit_to_square(1));
    }

    /// Test global optimizer integration
    #[test]
    fn test_global_optimizer_integration() {
        let bb = 0b1010u128;

        // Test GlobalOptimizer consistency
        assert_eq!(integration::GlobalOptimizer::popcount(bb), popcount(bb));
        assert_eq!(
            integration::GlobalOptimizer::bit_scan_forward(bb),
            bit_scan_forward(bb)
        );
        assert_eq!(
            integration::GlobalOptimizer::get_all_bit_positions(bb),
            get_all_bit_positions(bb)
        );
    }

    /// Test bit iterator integration
    #[test]
    fn test_bit_iterator_integration() {
        let bb = 0b1010u128;

        // Test bit iterator consistency
        let iterator_positions: Vec<u8> = bits(bb).collect();
        let direct_positions = get_all_bit_positions(bb);

        assert_eq!(iterator_positions, direct_positions);

        // Test reverse iteration
        let reverse_positions: Vec<u8> = bb.bits_rev().collect();
        let mut expected_reverse = direct_positions;
        expected_reverse.reverse();

        assert_eq!(reverse_positions, expected_reverse);
    }
}

/// Performance benchmarks and regression tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    /// Benchmark population count performance across implementations
    #[test]
    fn benchmark_popcount_performance() {
        let test_data = generate_test_data(1000);
        let iterations = 1000;

        // Benchmark standard implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount(bb);
            }
        }
        let standard_time = start.elapsed();

        // Benchmark cache-optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount_cache_optimized(bb);
            }
        }
        let cache_time = start.elapsed();

        // Benchmark branch-optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount_branch_optimized(bb);
            }
        }
        let branch_time = start.elapsed();

        // Benchmark critical path implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount_critical(bb);
            }
        }
        let critical_time = start.elapsed();

        println!("Population Count Performance:");
        println!("  Standard: {:?}", standard_time);
        println!("  Cache-optimized: {:?}", cache_time);
        println!("  Branch-optimized: {:?}", branch_time);
        println!("  Critical path: {:?}", critical_time);

        // Verify all implementations produce correct results
        for &bb in &test_data[..10] {
            let standard = popcount(bb);
            assert_eq!(popcount_cache_optimized(bb), standard);
            assert_eq!(popcount_branch_optimized(bb), standard);
            assert_eq!(popcount_critical(bb), standard);
        }
    }

    /// Benchmark bit scanning performance across implementations
    #[test]
    fn benchmark_bit_scan_performance() {
        let test_data = generate_test_data(1000);
        let iterations = 1000;

        // Benchmark standard implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = bit_scan_forward(bb);
                let _ = bit_scan_reverse(bb);
            }
        }
        let standard_time = start.elapsed();

        // Benchmark branch-optimized implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = bit_scan_forward_optimized(bb);
                let _ = bit_scan_reverse_optimized(bb);
            }
        }
        let branch_time = start.elapsed();

        // Benchmark critical path implementation
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = bit_scan_forward_critical(bb);
            }
        }
        let critical_time = start.elapsed();

        println!("Bit Scan Performance:");
        println!("  Standard: {:?}", standard_time);
        println!("  Branch-optimized: {:?}", branch_time);
        println!("  Critical path: {:?}", critical_time);

        // Verify all implementations produce correct results
        for &bb in &test_data[..10] {
            let standard_forward = bit_scan_forward(bb);
            let standard_reverse = bit_scan_reverse(bb);

            assert_eq!(bit_scan_forward_optimized(bb), standard_forward);
            assert_eq!(bit_scan_reverse_optimized(bb), standard_reverse);
            assert_eq!(bit_scan_forward_critical(bb), standard_forward);
        }
    }

    /// Benchmark cache optimization effectiveness
    #[test]
    fn benchmark_cache_optimization() {
        let test_data = generate_test_data(1000);
        let iterations = 500;

        // Benchmark with cache optimization
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount_cache_optimized(bb);
                let _ = get_bit_positions_cache_optimized(bb);
            }
        }
        let cache_optimized_time = start.elapsed();

        // Benchmark without cache optimization (standard)
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = bb.count_ones();
                let _ = get_all_bit_positions(bb);
            }
        }
        let standard_time = start.elapsed();

        println!("Cache Optimization Performance:");
        println!("  Cache-optimized: {:?}", cache_optimized_time);
        println!("  Standard: {:?}", standard_time);

        // Verify correctness
        for &bb in &test_data[..10] {
            assert_eq!(popcount_cache_optimized(bb), bb.count_ones());
            assert_eq!(
                get_bit_positions_cache_optimized(bb),
                get_all_bit_positions(bb)
            );
        }
    }

    /// Benchmark branch prediction optimization effectiveness
    #[test]
    fn benchmark_branch_prediction() {
        let test_data = generate_test_data(1000);
        let iterations = 500;

        // Benchmark branch-optimized implementations
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = popcount_branch_optimized(bb);
                let _ = bit_scan_forward_optimized(bb);
                let _ = is_empty_optimized(bb);
                let _ = is_single_piece_optimized(bb);
            }
        }
        let branch_optimized_time = start.elapsed();

        // Benchmark standard implementations
        let start = Instant::now();
        for _ in 0..iterations {
            for &bb in &test_data {
                let _ = bb.count_ones();
                let _ = bb.trailing_zeros();
                let _ = bb == 0;
                let _ = bb != 0 && (bb & (bb - 1) == 0);
            }
        }
        let standard_time = start.elapsed();

        println!("Branch Prediction Performance:");
        println!("  Branch-optimized: {:?}", branch_optimized_time);
        println!("  Standard: {:?}", standard_time);

        // Verify correctness
        for &bb in &test_data[..10] {
            assert_eq!(popcount_branch_optimized(bb), bb.count_ones());
            assert_eq!(
                bit_scan_forward_optimized(bb),
                if bb == 0 {
                    None
                } else {
                    Some(bb.trailing_zeros() as u8)
                }
            );
            assert_eq!(is_empty_optimized(bb), bb == 0);
            assert_eq!(
                is_single_piece_optimized(bb),
                bb != 0 && (bb & (bb - 1) == 0)
            );
        }
    }

    /// Generate test data for benchmarking
    fn generate_test_data(size: usize) -> Vec<Bitboard> {
        let mut data = Vec::new();

        // Empty bitboards (most common case)
        for _ in 0..size / 4 {
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
        for _ in 0..size / 8 {
            let mut bb = 0u128;
            for _ in 0..3 {
                bb |= 1u128 << (rand::random::<u8>() % 81);
            }
            data.push(bb);
        }

        // Dense bitboards (less common)
        for _ in 0..size / 8 {
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

/// Cross-platform testing and compatibility tests
#[cfg(test)]
mod cross_platform_tests {
    use super::*;

    /// Test cross-platform consistency of results
    #[test]
    fn test_cross_platform_consistency() {
        let test_cases = vec![
            0u128,
            1u128,
            0b1010u128,
            0b1111u128,
            0x1234_5678_9ABC_DEF0u128,
            (1u128 << 63),
            (1u128 << 127),
        ];

        for bb in test_cases {
            // All implementations should produce identical results
            let standard_count = popcount(bb);
            let cache_count = popcount_cache_optimized(bb);
            let branch_count = popcount_branch_optimized(bb);
            let critical_count = popcount_critical(bb);

            assert_eq!(
                standard_count, cache_count,
                "Cache-optimized popcount inconsistent for 0x{:X}",
                bb
            );
            assert_eq!(
                standard_count, branch_count,
                "Branch-optimized popcount inconsistent for 0x{:X}",
                bb
            );
            assert_eq!(
                standard_count, critical_count,
                "Critical path popcount inconsistent for 0x{:X}",
                bb
            );

            // Bit scanning should be consistent
            let standard_forward = bit_scan_forward(bb);
            let branch_forward = bit_scan_forward_optimized(bb);
            let critical_forward = bit_scan_forward_critical(bb);

            assert_eq!(
                standard_forward, branch_forward,
                "Branch-optimized bit_scan_forward inconsistent for 0x{:X}",
                bb
            );
            assert_eq!(
                standard_forward, critical_forward,
                "Critical path bit_scan_forward inconsistent for 0x{:X}",
                bb
            );
        }
    }

    // WASM compatibility tests removed - no longer needed

    /// Test platform detection across architectures
    #[test]
    fn test_platform_detection_cross_platform() {
        let caps = get_platform_capabilities();

        // Platform capabilities should be consistently detected
        assert!(caps.has_popcnt || caps.has_bmi1);

        // Best implementations should be selected appropriately
        let popcount_impl = get_best_popcount_impl();
        let bitscan_impl = get_best_bitscan_impl();

        assert!(matches!(
            popcount_impl,
            PopcountImpl::Hardware | PopcountImpl::BitParallel | PopcountImpl::Software
        ));
        assert!(matches!(
            bitscan_impl,
            BitscanImpl::Hardware | BitscanImpl::DeBruijn | BitscanImpl::Software
        ));
    }
}

/// Regression tests to prevent performance and correctness regressions
#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Test that performance has not regressed
    #[test]
    fn test_performance_regression() {
        let test_data = generate_regression_test_data();

        // Measure current performance
        let start = Instant::now();
        for &bb in &test_data {
            let _ = popcount_critical(bb);
            let _ = bit_scan_forward_critical(bb);
        }
        let current_time = start.elapsed();

        // Performance should be reasonable (less than 1 second for test data)
        assert!(
            current_time.as_millis() < 1000,
            "Performance regression detected: {:?}",
            current_time
        );

        println!("Regression test performance: {:?}", current_time);
    }

    /// Test edge cases that previously caused issues
    #[test]
    fn test_edge_cases_regression() {
        // Test edge cases that have caused problems in the past
        let edge_cases = vec![
            0u128,                     // Empty bitboard
            1u128,                     // Single bit
            1u128 << 63,               // Single bit at 64-bit boundary
            1u128 << 127,              // Single bit at maximum position
            0xFFFF_FFFF_FFFF_FFFFu128, // Full 64-bit chunk
            !0u128,                    // All bits set
        ];

        for bb in edge_cases {
            // All implementations should handle edge cases correctly
            let count = popcount(bb);
            let forward = bit_scan_forward(bb);
            let reverse = bit_scan_reverse(bb);

            // Verify consistency across implementations
            assert_eq!(popcount_cache_optimized(bb), count);
            assert_eq!(popcount_branch_optimized(bb), count);
            assert_eq!(popcount_critical(bb), count);

            assert_eq!(bit_scan_forward_optimized(bb), forward);
            assert_eq!(bit_scan_forward_critical(bb), forward);

            assert_eq!(bit_scan_reverse_optimized(bb), reverse);
        }
    }

    /// Test that all lookup tables are still valid
    #[test]
    fn test_lookup_table_regression() {
        // Validate cache-aligned lookup tables
        assert!(cache_opt::validation::validate_cache_alignment());
        assert!(cache_opt::validation::validate_lookup_tables());

        // Validate De Bruijn sequences
        assert!(debruijn::validate_debruijn_sequences());

        // Validate mask tables
        assert!(masks::validate_masks());
    }

    /// Test that API compatibility is maintained
    #[test]
    fn test_api_compatibility_regression() {
        let bb = 0b1010u128;

        // Test that all API functions still work
        assert_eq!(api::bitscan::popcount(bb), 2);
        assert_eq!(api::bitscan::bit_scan_forward(bb), Some(1));
        assert_eq!(api::utils::extract_lsb(bb), (0b0010, 0b1000));
        assert_eq!(api::squares::bit_to_square(40).row, 4);

        // Test backward compatibility
        assert_eq!(api::compat::count_bits(bb), 2);
        assert_eq!(api::compat::find_first_bit(bb), Some(1));
        assert_eq!(api::compat::find_last_bit(bb), Some(3));
    }

    /// Generate regression test data
    fn generate_regression_test_data() -> Vec<Bitboard> {
        let mut data = Vec::new();

        // Include a variety of patterns
        for i in 0..100 {
            data.push(1u128 << (i % 128));
        }

        for i in 0..100 {
            data.push((1u128 << i) | (1u128 << (i + 64)));
        }

        for i in 0..50 {
            data.push(0x5555_5555_5555_5555_5555_5555_5555_5555 << (i % 2));
        }

        data
    }
}

/// Comprehensive validation tests
#[cfg(test)]
mod validation_tests {
    use super::*;

    /// Test that all optimization modules are working correctly
    #[test]
    fn test_all_optimizations_working() {
        let test_bitboard = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0u128;

        // Test that all optimization modules are functional
        assert!(cache_opt::validation::validate_cache_alignment());
        assert!(cache_opt::validation::validate_lookup_tables());
        assert!(branch_opt::validation::validate_optimization_correctness());
        assert!(branch_opt::validation::validate_critical_path_correctness());

        // Test that all implementations produce correct results
        let expected_count = test_bitboard.count_ones();
        assert_eq!(popcount(test_bitboard), expected_count);
        assert_eq!(popcount_cache_optimized(test_bitboard), expected_count);
        assert_eq!(popcount_branch_optimized(test_bitboard), expected_count);
        assert_eq!(popcount_critical(test_bitboard), expected_count);
    }

    /// Test that all edge cases are handled correctly
    #[test]
    fn test_edge_cases_comprehensive() {
        let edge_cases = vec![
            0u128,                                         // Empty
            1u128,                                         // Single bit
            1u128 << 63,                                   // 64-bit boundary
            1u128 << 127,                                  // Maximum position
            0xFFFF_FFFF_FFFF_FFFFu128,                     // Full 64-bit
            0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000u128, // High 64-bit only
            0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFFu128, // Low 64-bit only
            !0u128,                                        // All bits
        ];

        for bb in edge_cases {
            // Test all implementations handle edge cases
            let count = bb.count_ones();
            let forward = if bb == 0 {
                None
            } else {
                Some(bb.trailing_zeros() as u8)
            };
            let reverse = if bb == 0 {
                None
            } else {
                Some((127 - bb.leading_zeros()) as u8)
            };

            assert_eq!(
                popcount(bb),
                count,
                "Edge case popcount failed for 0x{:X}",
                bb
            );
            assert_eq!(
                popcount_cache_optimized(bb),
                count,
                "Edge case cache popcount failed for 0x{:X}",
                bb
            );
            assert_eq!(
                popcount_branch_optimized(bb),
                count,
                "Edge case branch popcount failed for 0x{:X}",
                bb
            );
            assert_eq!(
                popcount_critical(bb),
                count,
                "Edge case critical popcount failed for 0x{:X}",
                bb
            );

            assert_eq!(
                bit_scan_forward(bb),
                forward,
                "Edge case bit_scan_forward failed for 0x{:X}",
                bb
            );
            assert_eq!(
                bit_scan_forward_optimized(bb),
                forward,
                "Edge case branch bit_scan_forward failed for 0x{:X}",
                bb
            );
            assert_eq!(
                bit_scan_forward_critical(bb),
                forward,
                "Edge case critical bit_scan_forward failed for 0x{:X}",
                bb
            );

            assert_eq!(
                bit_scan_reverse(bb),
                reverse,
                "Edge case bit_scan_reverse failed for 0x{:X}",
                bb
            );
            assert_eq!(
                bit_scan_reverse_optimized(bb),
                reverse,
                "Edge case branch bit_scan_reverse failed for 0x{:X}",
                bb
            );
        }
    }

    /// Test that all functions maintain correctness under stress
    #[test]
    fn test_stress_correctness() {
        // Test with many random bitboards
        for _ in 0..1000 {
            let bb = rand::random::<u128>();

            let expected_count = bb.count_ones();
            let expected_forward = if bb == 0 {
                None
            } else {
                Some(bb.trailing_zeros() as u8)
            };
            let expected_reverse = if bb == 0 {
                None
            } else {
                Some((127 - bb.leading_zeros()) as u8)
            };

            // Verify all implementations produce correct results
            assert_eq!(popcount(bb), expected_count);
            assert_eq!(popcount_cache_optimized(bb), expected_count);
            assert_eq!(popcount_branch_optimized(bb), expected_count);
            assert_eq!(popcount_critical(bb), expected_count);

            assert_eq!(bit_scan_forward(bb), expected_forward);
            assert_eq!(bit_scan_forward_optimized(bb), expected_forward);
            assert_eq!(bit_scan_forward_critical(bb), expected_forward);

            assert_eq!(bit_scan_reverse(bb), expected_reverse);
            assert_eq!(bit_scan_reverse_optimized(bb), expected_reverse);
        }
    }
}

/// Helper functions for testing
mod test_helpers {
    use super::*;

    /// Generate comprehensive test data
    pub fn generate_comprehensive_test_data() -> Vec<Bitboard> {
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
        for _ in 0..100 {
            data.push(rand::random::<u128>());
        }

        data
    }

    /// Validate that all implementations produce consistent results
    pub fn validate_implementation_consistency(bb: Bitboard) -> bool {
        let expected_count = bb.count_ones();
        let expected_forward = if bb == 0 {
            None
        } else {
            Some(bb.trailing_zeros() as u8)
        };
        let expected_reverse = if bb == 0 {
            None
        } else {
            Some((127 - bb.leading_zeros()) as u8)
        };

        popcount(bb) == expected_count
            && popcount_cache_optimized(bb) == expected_count
            && popcount_branch_optimized(bb) == expected_count
            && popcount_critical(bb) == expected_count
            && bit_scan_forward(bb) == expected_forward
            && bit_scan_forward_optimized(bb) == expected_forward
            && bit_scan_forward_critical(bb) == expected_forward
            && bit_scan_reverse(bb) == expected_reverse
            && bit_scan_reverse_optimized(bb) == expected_reverse
    }
}
