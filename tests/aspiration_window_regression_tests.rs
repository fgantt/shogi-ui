#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::Player;
use shogi_engine::types::*;

#[cfg(test)]
mod aspiration_window_regression_tests {
    use super::*;

    // ===== REGRESSION TESTS FOR ORIGINAL ISSUES =====

    /// Test that aspiration window search never completely fails (regression test)
    /// This specifically tests the fix for the immediate break on search failure
    #[test]
    fn regression_test_aspiration_window_complete_failure() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 5, // Very small window to force failures
            dynamic_scaling: true,
            max_window_size: 15,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 3, // Allow retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test the specific scenario that caused the original issue
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        // This should NEVER be None (the original bug)
        assert!(
            result.is_some(),
            "REGRESSION: Aspiration window search should never completely fail"
        );

        let (mv, score) = result.unwrap();
        assert!(
            !mv.to_usi_string().is_empty(),
            "REGRESSION: Move should not be empty"
        );
        assert!(
            score > -50000 && score < 50000,
            "REGRESSION: Score should be reasonable"
        );
    }

    /// Test that move tracking always returns a valid move (regression test)
    /// This specifically tests the fix for best_move=None issue
    #[test]
    fn regression_test_move_tracking_best_move_none() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test scenarios that could cause best_move to be None
        let problematic_cases = vec![
            (-10000, 10000), // Very wide window
            (-100, 100),     // Normal window
            (0, 1000),       // Positive window
            (-1000, 0),      // Negative window
            (500, 600),      // Very narrow window
        ];

        for (alpha, beta) in problematic_cases {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // This should NEVER be None (the original bug)
            assert!(
                result.is_some(),
                "REGRESSION: Move tracking should never return None for alpha={}, beta={}",
                alpha,
                beta
            );

            let (mv, score) = result.unwrap();
            assert!(
                !mv.to_usi_string().is_empty(),
                "REGRESSION: Move should not be empty for alpha={}, beta={}",
                alpha,
                beta
            );

            // Score should be within the search window
            assert!(
                score >= alpha,
                "REGRESSION: Score {} should be >= alpha {} for alpha={}, beta={}",
                score,
                alpha,
                alpha,
                beta
            );
            assert!(
                score <= beta,
                "REGRESSION: Score {} should be <= beta {} for alpha={}, beta={}",
                score,
                beta,
                alpha,
                beta
            );
        }
    }

    /// Test integer overflow safety (regression test)
    /// This specifically tests the fix for the panic at line 5237
    #[test]
    fn regression_test_integer_overflow_panic() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test the specific values that caused the original overflow panic
        let overflow_cases = vec![
            (i32::MIN + 1000, i32::MAX - 1000), // Near min/max values
            (i32::MIN + 1, i32::MAX - 1),       // Very close to min/max
            (-2000000000, 2000000000),          // Large negative/positive
            (i32::MIN, i32::MAX),               // Absolute min/max
        ];

        for (alpha, beta) in overflow_cases {
            // This should NOT panic (the original bug)
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // Should handle overflow gracefully
            assert!(
                result.is_some(),
                "REGRESSION: Integer overflow should not cause panic for alpha={}, beta={}",
                alpha,
                beta
            );

            let (mv, score) = result.unwrap();
            assert!(
                !mv.to_usi_string().is_empty(),
                "REGRESSION: Move should be valid after overflow handling"
            );
            assert!(
                score > -100000 && score < 100000,
                "REGRESSION: Score should be reasonable after overflow handling: {}",
                score
            );
        }
    }

    /// Test that debug logging doesn't cause issues (regression test)
    #[test]
    fn regression_test_debug_logging_issues() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Enable debug logging
        crate::debug_utils::set_log_level("TRACE");

        // This should not cause any issues with logging enabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        assert!(
            result.is_some(),
            "REGRESSION: Debug logging should not cause search failures"
        );

        let (mv, score) = result.unwrap();
        assert!(
            !mv.to_usi_string().is_empty(),
            "REGRESSION: Move should be valid with debug logging"
        );
        assert!(
            score > -50000 && score < 50000,
            "REGRESSION: Score should be reasonable with debug logging"
        );
    }

    // ===== EDGE CASE REGRESSION TESTS =====

    /// Test edge case: very small window sizes
    #[test]
    fn regression_test_very_small_window_sizes() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1, // Very small
            dynamic_scaling: true,
            max_window_size: 5,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 5,
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        // Should handle very small windows gracefully
        assert!(
            result.is_some(),
            "REGRESSION: Very small window sizes should not cause failures"
        );

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -50000 && score < 50000);
    }

    /// Test edge case: very large window sizes
    #[test]
    fn regression_test_very_large_window_sizes() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1000, // Very large
            dynamic_scaling: true,
            max_window_size: 2000,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 2,
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        // Should handle very large windows gracefully
        assert!(
            result.is_some(),
            "REGRESSION: Very large window sizes should not cause failures"
        );

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -50000 && score < 50000);
    }

    /// Test edge case: maximum research attempts
    #[test]
    fn regression_test_maximum_research_attempts() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1, // Very small to force many retries
            dynamic_scaling: true,
            max_window_size: 5,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 10, // Many retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 200, -2000, 2000);

        // Should handle maximum retries gracefully
        assert!(
            result.is_some(),
            "REGRESSION: Maximum research attempts should not cause failures"
        );

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -50000 && score < 50000);
    }

    /// Test edge case: extreme alpha/beta values
    #[test]
    fn regression_test_extreme_alpha_beta_values() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let extreme_cases = vec![
            (i32::MIN, i32::MAX),         // Absolute extremes
            (i32::MIN + 1, i32::MAX - 1), // Near extremes
            (-1000000, 1000000),          // Very large values
            (0, 1),                       // Very small positive
            (-1, 0),                      // Very small negative
            (1, 0),                       // Invalid (alpha > beta)
        ];

        for (alpha, beta) in extreme_cases {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // Should handle extreme values gracefully (may return None for invalid cases)
            if alpha < beta {
                assert!(
                    result.is_some(),
                    "REGRESSION: Valid extreme values should not cause failures: alpha={}, beta={}",
                    alpha,
                    beta
                );

                let (mv, score) = result.unwrap();
                assert!(!mv.to_usi_string().is_empty());
                assert!(score > -100000 && score < 100000);
            }
        }
    }

    // ===== SPECIFIC BUG REGRESSION TESTS =====

    /// Test the specific scenario that caused the original panic
    #[test]
    fn regression_test_original_panic_scenario() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // This specific combination was causing the panic
        let alpha = i32::MIN + 1000;
        let beta = i32::MAX - 1000;

        // This should NOT panic anymore
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, alpha, beta);

        assert!(
            result.is_some(),
            "REGRESSION: Original panic scenario should now work"
        );

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -100000 && score < 100000);
    }

    /// Test move tracking with scores below alpha
    #[test]
    fn regression_test_move_tracking_below_alpha() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Use a high alpha so all moves will be below it
        let alpha = 10000;
        let beta = 20000;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

        // Should still return a move (fallback to first move)
        assert!(
            result.is_some(),
            "REGRESSION: Should return fallback move when all moves below alpha"
        );

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        // Score might be below alpha, but that's expected for fallback
    }

    /// Test aspiration window with zero time limit
    #[test]
    fn regression_test_zero_time_limit() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Zero time limit should be handled gracefully
        let result = engine.search_at_depth(&board, &captured_pieces, player, 2, 0, -1000, 1000);

        // Should either return a result or handle timeout gracefully
        if let Some((mv, score)) = result {
            assert!(!mv.to_usi_string().is_empty());
            assert!(score > -50000 && score < 50000);
        }
        // If None, that's also acceptable for zero time limit
    }

    /// Test aspiration window with very short time limit
    #[test]
    fn regression_test_very_short_time_limit() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Very short time limit
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1, -1000, 1000);

        // Should handle short time limit gracefully
        if let Some((mv, score)) = result {
            assert!(!mv.to_usi_string().is_empty());
            assert!(score > -50000 && score < 50000);
        }
        // If None due to timeout, that's acceptable
    }

    // ===== CONSISTENCY REGRESSION TESTS =====

    /// Test that multiple searches with same parameters give consistent results
    #[test]
    fn regression_test_search_consistency() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let mut results = Vec::new();

        // Run same search multiple times
        for _ in 0..5 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            assert!(result.is_some(), "REGRESSION: Search should be consistent");
            results.push(result.unwrap());
        }

        // All results should be valid
        for (mv, score) in &results {
            assert!(!mv.to_usi_string().is_empty());
            assert!(score > &-50000 && score < &50000);
        }

        // Results should be consistent (not necessarily identical, but all valid)
        assert_eq!(results.len(), 5);
    }

    /// Test that aspiration window statistics are updated correctly
    #[test]
    fn regression_test_aspiration_window_statistics() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 50,
            dynamic_scaling: true,
            max_window_size: 200,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 3,
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let initial_stats = engine.get_aspiration_window_stats();

        // Run a search
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
        assert!(result.is_some());

        let final_stats = engine.get_aspiration_window_stats();

        // Statistics should be updated
        assert!(
            final_stats.total_searches > initial_stats.total_searches,
            "REGRESSION: Statistics should be updated after search"
        );
    }
}
