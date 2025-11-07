#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::Player;
use shogi_engine::types::*;
use std::time::{Duration, Instant};

#[cfg(test)]
mod aspiration_window_performance_validation_tests {
    use super::*;

    // ===== PERFORMANCE BENCHMARKS =====

    /// Benchmark aspiration window performance with fixes
    #[test]
    fn benchmark_aspiration_window_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let iterations = 20;
        let mut total_time = Duration::new(0, 0);
        let mut successful_searches = 0;

        for _ in 0..iterations {
            let start = Instant::now();
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 4, 100, -1000, 1000);
            let elapsed = start.elapsed();

            total_time += elapsed;
            if result.is_some() {
                successful_searches += 1;
            }
        }

        let avg_time = total_time / iterations;
        let success_rate = (successful_searches as f64 / iterations as f64) * 100.0;

        println!("Aspiration Window Performance:");
        println!("  Average time per search: {:?}", avg_time);
        println!("  Success rate: {:.1}%", success_rate);
        println!("  Total time: {:?}", total_time);

        // Performance assertions
        assert!(
            avg_time.as_millis() < 1000,
            "Average search time should be under 1 second"
        );
        assert!(success_rate >= 95.0, "Success rate should be at least 95%");
    }

    /// Benchmark with different window sizes
    #[test]
    fn benchmark_different_window_sizes() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let window_sizes = vec![10, 25, 50, 100, 200];
        let mut results = Vec::new();

        for window_size in window_sizes {
            let config = AspirationWindowConfig {
                enabled: true,
                base_window_size: window_size,
                dynamic_scaling: true,
                max_window_size: window_size * 2,
                min_depth: 1,
                enable_adaptive_sizing: true,
                max_researches: 3,
                enable_statistics: true,
            };
            engine.update_aspiration_window_config(config).unwrap();

            let start = Instant::now();
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            let elapsed = start.elapsed();

            results.push((window_size, elapsed, result.is_some()));
        }

        // Print results
        println!("Window Size Performance:");
        for (size, time, success) in &results {
            println!("  Size {}: {:?} (success: {})", size, time, success);
        }

        // All should succeed
        for (_, _, success) in &results {
            assert!(success, "All window sizes should succeed");
        }
    }

    /// Benchmark error recovery performance
    #[test]
    fn benchmark_error_recovery_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 5, // Small window to force retries
            dynamic_scaling: true,
            max_window_size: 20,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 5, // Allow retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let iterations = 10;
        let mut total_time = Duration::new(0, 0);
        let mut successful_searches = 0;

        for _ in 0..iterations {
            let start = Instant::now();
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 4, 100, -2000, 2000);
            let elapsed = start.elapsed();

            total_time += elapsed;
            if result.is_some() {
                successful_searches += 1;
            }
        }

        let avg_time = total_time / iterations;
        let success_rate = (successful_searches as f64 / iterations as f64) * 100.0;

        println!("Error Recovery Performance:");
        println!("  Average time per search: {:?}", avg_time);
        println!("  Success rate: {:.1}%", success_rate);

        // Should still perform well even with retries
        assert!(
            avg_time.as_millis() < 2000,
            "Error recovery should not be too slow"
        );
        assert!(
            success_rate >= 90.0,
            "Error recovery should maintain high success rate"
        );
    }

    /// Benchmark integer overflow safety performance
    #[test]
    fn benchmark_overflow_safety_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let overflow_cases = vec![
            (i32::MIN + 1000, i32::MAX - 1000),
            (-2000000000, 2000000000),
            (i32::MIN + 1, i32::MAX - 1),
        ];

        let mut total_time = Duration::new(0, 0);
        let mut successful_searches = 0;

        for (alpha, beta) in overflow_cases {
            let start = Instant::now();
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, alpha, beta);
            let elapsed = start.elapsed();

            total_time += elapsed;
            if result.is_some() {
                successful_searches += 1;
            }
        }

        let avg_time = total_time / overflow_cases.len() as u32;
        let success_rate = (successful_searches as f64 / overflow_cases.len() as f64) * 100.0;

        println!("Overflow Safety Performance:");
        println!("  Average time per search: {:?}", avg_time);
        println!("  Success rate: {:.1}%", success_rate);

        // Should handle overflow cases efficiently
        assert!(
            avg_time.as_millis() < 1000,
            "Overflow safety should not be slow"
        );
        assert!(
            success_rate >= 95.0,
            "Overflow safety should maintain high success rate"
        );
    }

    // ===== REGRESSION TESTS =====

    /// Test that fixes don't introduce performance regressions
    #[test]
    fn test_no_performance_regression() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Run baseline performance test
        let iterations = 15;
        let mut total_time = Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            let elapsed = start.elapsed();

            total_time += elapsed;
            assert!(result.is_some(), "All searches should succeed");
        }

        let avg_time = total_time / iterations;

        // Performance should be reasonable (under 500ms per search)
        assert!(
            avg_time.as_millis() < 500,
            "Performance regression detected: {:?}",
            avg_time
        );

        println!("No Regression Test - Average time: {:?}", avg_time);
    }

    /// Test memory usage doesn't increase significantly
    #[test]
    fn test_memory_usage_stability() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Run many searches to test memory stability
        for _ in 0..50 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 50, -1000, 1000);
            assert!(result.is_some(), "Memory usage should not cause failures");
        }

        // If we get here without panicking, memory usage is stable
        println!("Memory usage stability test passed");
    }

    /// Test that error handling doesn't impact normal performance
    #[test]
    fn test_error_handling_performance_impact() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test normal case performance
        let normal_start = Instant::now();
        for _ in 0..10 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            assert!(result.is_some());
        }
        let normal_time = normal_start.elapsed();

        // Test with error-prone configuration
        let error_config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1, // Very small to force errors
            dynamic_scaling: true,
            max_window_size: 5,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 10, // Allow many retries
            enable_statistics: true,
        };
        engine
            .update_aspiration_window_config(error_config)
            .unwrap();

        let error_start = Instant::now();
        for _ in 0..10 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            assert!(result.is_some());
        }
        let error_time = error_start.elapsed();

        // Error handling should not make normal case significantly slower
        let ratio = error_time.as_millis() as f64 / normal_time.as_millis() as f64;
        assert!(
            ratio < 3.0,
            "Error handling should not slow normal case by more than 3x: {:.2}x",
            ratio
        );

        println!("Error handling performance impact: {:.2}x", ratio);
    }

    // ===== STRESS TESTS =====

    /// Stress test with many rapid searches
    #[test]
    fn stress_test_rapid_searches() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let start = Instant::now();
        let mut successful = 0;

        // Run many searches rapidly
        for i in 0..100 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 20, -1000, 1000);
            if result.is_some() {
                successful += 1;
            }

            // Print progress every 20 searches
            if i % 20 == 19 {
                println!("Stress test progress: {}/100 searches completed", i + 1);
            }
        }

        let elapsed = start.elapsed();
        let success_rate = (successful as f64 / 100.0) * 100.0;

        println!("Stress Test Results:");
        println!("  Total time: {:?}", elapsed);
        println!("  Success rate: {:.1}%", success_rate);
        println!("  Average time per search: {:?}", elapsed / 100);

        // Should maintain high success rate even under stress
        assert!(
            success_rate >= 95.0,
            "Stress test success rate too low: {:.1}%",
            success_rate
        );
        assert!(
            elapsed.as_secs() < 10,
            "Stress test took too long: {:?}",
            elapsed
        );
    }

    /// Stress test with extreme values
    #[test]
    fn stress_test_extreme_values() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let extreme_cases = vec![
            (i32::MIN + 1, i32::MAX - 1),
            (-1000000, 1000000),
            (0, 1),
            (-1, 0),
            (i32::MIN + 1000, i32::MAX - 1000),
        ];

        let mut successful = 0;
        let start = Instant::now();

        for (alpha, beta) in extreme_cases {
            for _ in 0..10 {
                let result =
                    engine.search_at_depth(&board, &captured_pieces, player, 2, 50, alpha, beta);
                if result.is_some() {
                    successful += 1;
                }
            }
        }

        let elapsed = start.elapsed();
        let success_rate = (successful as f64 / 50.0) * 100.0;

        println!("Extreme Values Stress Test:");
        println!("  Total time: {:?}", elapsed);
        println!("  Success rate: {:.1}%", success_rate);

        // Should handle extreme values gracefully
        assert!(
            success_rate >= 90.0,
            "Extreme values test success rate too low: {:.1}%",
            success_rate
        );
        assert!(
            elapsed.as_secs() < 5,
            "Extreme values test took too long: {:?}",
            elapsed
        );
    }

    // ===== INTEGRATION PERFORMANCE TESTS =====

    /// Test performance with other search features enabled
    #[test]
    fn test_integration_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Enable all search features
        let mut search_config = engine.get_search_config();
        search_config.enable_null_move = true;
        search_config.enable_lmr = true;
        search_config.enable_quiescence = true;
        search_config.enable_aspiration_windows = true;
        engine.update_search_config(search_config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let start = Instant::now();
        let mut successful = 0;

        for _ in 0..20 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 4, 100, -1000, 1000);
            if result.is_some() {
                successful += 1;
            }
        }

        let elapsed = start.elapsed();
        let success_rate = (successful as f64 / 20.0) * 100.0;
        let avg_time = elapsed / 20;

        println!("Integration Performance Test:");
        println!("  Average time per search: {:?}", avg_time);
        println!("  Success rate: {:.1}%", success_rate);

        // Should work well with all features
        assert!(
            success_rate >= 95.0,
            "Integration test success rate too low: {:.1}%",
            success_rate
        );
        assert!(
            avg_time.as_millis() < 1000,
            "Integration test too slow: {:?}",
            avg_time
        );
    }
}
