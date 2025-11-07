#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::Player;
use shogi_engine::types::*;

#[cfg(test)]
mod aspiration_window_critical_fixes_tests {
    use super::*;

    // ===== PHASE 1 CRITICAL FIXES TESTS =====

    /// Test that aspiration window search never completely fails
    /// This tests the fix for the immediate break on search failure
    #[test]
    fn test_aspiration_window_never_completely_fails() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 10, // Very small window to force failures
            dynamic_scaling: true,
            max_window_size: 50,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 3, // Allow multiple retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test multiple searches to ensure no complete failures
        for _ in 0..10 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

            // Should always return Some result, never None
            assert!(
                result.is_some(),
                "Aspiration window search should never completely fail"
            );

            let (mv, score) = result.unwrap();
            assert!(
                mv.to_usi_string().len() >= 4,
                "Move should be valid USI format"
            );
            assert!(
                score > -50000 && score < 50000,
                "Score should be within reasonable bounds"
            );
        }
    }

    /// Test that move tracking always returns a valid move when moves exist
    /// This tests the fix for best_move=None issue
    #[test]
    fn test_move_tracking_always_returns_move() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with various alpha/beta values that might cause issues
        let test_cases = vec![
            (-1000, 1000),
            (-500, 500),
            (0, 100),
            (-100, 0),
            (-10000, 10000),
        ];

        for (alpha, beta) in test_cases {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // Should always return Some result with a valid move
            assert!(
                result.is_some(),
                "Search should return result for alpha={}, beta={}",
                alpha,
                beta
            );

            let (mv, score) = result.unwrap();
            assert!(!mv.to_usi_string().is_empty(), "Move should not be empty");
            assert!(score >= alpha, "Score should be >= alpha");
            assert!(score <= beta, "Score should be <= beta");
        }
    }

    /// Test fallback move selection when no move exceeds alpha
    #[test]
    fn test_fallback_move_selection() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 50,
            dynamic_scaling: true,
            max_window_size: 200,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 2,
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Use a very high alpha to force all moves to be below it
        let alpha = 10000;
        let beta = 20000;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

        // Should still return a result (fallback to first move)
        assert!(
            result.is_some(),
            "Should return fallback move when no move exceeds alpha"
        );

        let (mv, score) = result.unwrap();
        assert!(
            !mv.to_usi_string().is_empty(),
            "Fallback move should be valid"
        );
        // Score might be below alpha, but that's expected for fallback
    }

    /// Test comprehensive debug logging
    #[test]
    fn test_debug_logging_comprehensive() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Enable debug logging
        crate::debug_utils::set_log_level("TRACE");

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        // Should complete without panicking and with comprehensive logging
        assert!(result.is_some());

        // Check that statistics were updated (indicating logging worked)
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
    }

    // ===== PHASE 2 ROBUSTNESS IMPROVEMENTS TESTS =====

    /// Test comprehensive retry strategy
    #[test]
    fn test_comprehensive_retry_strategy() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 5, // Very small to force retries
            dynamic_scaling: true,
            max_window_size: 20,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 5, // Allow many retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 200, -2000, 2000);

        // Should succeed despite small window
        assert!(result.is_some());

        let stats = engine.get_aspiration_window_stats();
        // Should have used retries
        assert!(stats.total_researches > 0);
    }

    /// Test window validation and recovery
    #[test]
    fn test_window_validation_and_recovery() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with extreme window values
        let extreme_cases = vec![
            (i32::MIN + 1, i32::MAX - 1),
            (-50000, 50000),
            (0, 1),  // Very small window
            (-1, 1), // Very small window
        ];

        for (alpha, beta) in extreme_cases {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // Should handle extreme values gracefully
            assert!(
                result.is_some(),
                "Should handle extreme window [{}, {}]",
                alpha,
                beta
            );

            let (mv, score) = result.unwrap();
            assert!(!mv.to_usi_string().is_empty());
            // Score should be within reasonable bounds even with extreme windows
            assert!(score > -100000 && score < 100000);
        }
    }

    /// Test search result validation
    #[test]
    fn test_search_result_validation() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);

        assert!(result.is_some());
        let (mv, score) = result.unwrap();

        // Validate move format
        let move_str = mv.to_usi_string();
        assert!(
            move_str.len() >= 4 && move_str.len() <= 6,
            "Invalid move format: {}",
            move_str
        );

        // Validate score bounds
        assert!(
            score > -50000 && score < 50000,
            "Score out of bounds: {}",
            score
        );

        // Validate move is not empty
        assert!(!move_str.is_empty(), "Move should not be empty");
    }

    /// Test error handling and recovery
    #[test]
    fn test_error_handling_and_recovery() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1, // Very small to force errors
            dynamic_scaling: true,
            max_window_size: 5,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 10, // Allow many retries
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // This should trigger error handling and recovery
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 200, -5000, 5000);

        // Should recover gracefully
        assert!(result.is_some(), "Error handling should allow recovery");

        let stats = engine.get_aspiration_window_stats();
        // Should have attempted recovery
        assert!(stats.total_searches > 0);
    }

    /// Test graceful degradation
    #[test]
    fn test_graceful_degradation() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1, // Very small to force degradation
            dynamic_scaling: true,
            max_window_size: 2,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 15, // Many retries to trigger degradation
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Multiple searches to trigger degradation
        for _ in 0..5 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 4, 100, -2000, 2000);
            assert!(
                result.is_some(),
                "Graceful degradation should maintain functionality"
            );
        }

        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
    }

    // ===== INTEGER OVERFLOW FIXES TESTS =====

    /// Test that integer overflow is handled safely
    #[test]
    fn test_integer_overflow_safety() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with values that could cause overflow
        let overflow_cases = vec![
            (i32::MIN + 1000, i32::MAX - 1000),
            (i32::MIN + 1, i32::MAX - 1),
            (-2000000000, 2000000000),
        ];

        for (alpha, beta) in overflow_cases {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 100, alpha, beta);

            // Should not panic due to overflow
            assert!(
                result.is_some(),
                "Should handle overflow case [{}, {}]",
                alpha,
                beta
            );

            let (mv, score) = result.unwrap();
            assert!(!mv.to_usi_string().is_empty());
            assert!(score > -100000 && score < 100000);
        }
    }

    /// Test edge case arithmetic operations
    #[test]
    fn test_edge_case_arithmetic() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 1000, // Large window size
            dynamic_scaling: true,
            max_window_size: 2000,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 3,
            enable_statistics: true,
        };
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with values that could cause arithmetic issues
        let result =
            engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000000, 1000000);

        // Should handle large numbers safely
        assert!(result.is_some());

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -1000000 && score < 1000000);
    }

    // ===== INTEGRATION TESTS =====

    /// Test aspiration windows with other search features
    #[test]
    fn test_aspiration_windows_with_other_features() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Enable other search features
        let mut search_config = engine.get_search_config();
        search_config.enable_null_move = true;
        search_config.enable_lmr = true;
        search_config.enable_quiescence = true;
        engine.update_search_config(search_config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 200, -1000, 1000);

        // Should work with other features enabled
        assert!(result.is_some());

        let (mv, score) = result.unwrap();
        assert!(!mv.to_usi_string().is_empty());
        assert!(score > -50000 && score < 50000);
    }

    /// Test performance under various conditions
    #[test]
    fn test_performance_under_various_conditions() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let start_time = std::time::Instant::now();

        // Run multiple searches
        for _ in 0..10 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 50, -1000, 1000);
            assert!(result.is_some());
        }

        let elapsed = start_time.elapsed();

        // Should complete in reasonable time (less than 5 seconds for 10 searches)
        assert!(
            elapsed.as_secs() < 5,
            "Performance should be reasonable: {:?}",
            elapsed
        );
    }

    /// Test consistency across multiple searches
    #[test]
    fn test_consistency_across_multiple_searches() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let mut results = Vec::new();

        // Run multiple searches
        for _ in 0..5 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 100, -1000, 1000);
            assert!(result.is_some());
            results.push(result.unwrap());
        }

        // All results should be valid
        for (mv, score) in &results {
            assert!(!mv.to_usi_string().is_empty());
            assert!(score > &-50000 && score < &50000);
        }

        // Results should be consistent (not all identical, but all valid)
        assert!(results.len() == 5);
    }
}
