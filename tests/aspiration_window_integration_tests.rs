#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::{IterativeDeepening, SearchEngine};
use shogi_engine::types::Player;
use shogi_engine::types::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(test)]
mod aspiration_window_integration_tests {
    use super::*;

    // ===== SEARCH ENGINE INTEGRATION TESTS =====

    #[test]
    fn test_aspiration_windows_with_iterative_deepening() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result = searcher.search(&mut engine, &board, &captured_pieces, player);

        // Should return a valid move
        assert!(result.is_some());
        let (mv, score) = result.unwrap();
        assert!(score > -100000 && score < 100000);

        // Check that aspiration window statistics were updated
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
    }

    #[test]
    fn test_aspiration_windows_with_different_depths() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test different depths
        for depth in 1..=5 {
            let mut searcher = IterativeDeepening::new(depth, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);

            assert!(result.is_some());
            let (mv, score) = result.unwrap();
            assert!(score > -100000 && score < 100000);
        }

        // Check that statistics were collected
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
    }

    #[test]
    fn test_aspiration_windows_with_different_configurations() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test aggressive configuration
        let mut config = AspirationWindowConfig::default();
        config.base_window_size = 25;
        engine.update_aspiration_window_config(config).unwrap();

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result1 = searcher.search(&mut engine, &board, &captured_pieces, player);
        assert!(result1.is_some());

        // Test conservative configuration
        let mut config = AspirationWindowConfig::default();
        config.base_window_size = 100;
        engine.update_aspiration_window_config(config).unwrap();

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result2 = searcher.search(&mut engine, &board, &captured_pieces, player);
        assert!(result2.is_some());

        // Both should return valid moves
        assert!(result1.is_some());
        assert!(result2.is_some());
    }

    #[test]
    fn test_aspiration_windows_with_disabled_configuration() {
        let mut engine = SearchEngine::new();
        let mut config = AspirationWindowConfig::default();
        config.enabled = false;
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result = searcher.search(&mut engine, &board, &captured_pieces, player);

        // Should still work but with full-width search
        assert!(result.is_some());

        // Check that no aspiration window statistics were collected
        let stats = engine.get_aspiration_window_stats();
        assert_eq!(stats.total_searches, 0);
    }

    // ===== PERFORMANCE INTEGRATION TESTS =====

    #[test]
    fn test_aspiration_windows_performance_improvement() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with aspiration windows enabled
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let start_time = std::time::Instant::now();
        let mut searcher = IterativeDeepening::new(4, 2000, None, None);
        let result1 = searcher.search(&mut engine, &board, &captured_pieces, player);
        let time_with_aspiration = start_time.elapsed();

        // Test with aspiration windows disabled
        let mut config = AspirationWindowConfig::default();
        config.enabled = false;
        engine.update_aspiration_window_config(config).unwrap();

        let start_time = std::time::Instant::now();
        let mut searcher = IterativeDeepening::new(4, 2000, None, None);
        let result2 = searcher.search(&mut engine, &board, &captured_pieces, player);
        let time_without_aspiration = start_time.elapsed();

        // Both should return valid moves
        assert!(result1.is_some());
        assert!(result2.is_some());

        // Aspiration windows should provide some performance benefit
        // (though this may vary depending on the position)
        println!("Time with aspiration: {:?}", time_with_aspiration);
        println!("Time without aspiration: {:?}", time_without_aspiration);
    }

    #[test]
    fn test_aspiration_windows_statistics_collection() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform multiple searches
        for _ in 0..5 {
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Check that statistics were collected
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
        assert!(stats.average_window_size > 0.0);

        // Check performance metrics
        let metrics = engine.get_aspiration_window_performance_metrics();
        assert!(metrics.total_searches > 0);
        assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
    }

    // ===== ERROR HANDLING INTEGRATION TESTS =====

    #[test]
    fn test_aspiration_windows_error_recovery() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Simulate error conditions by using extreme configurations
        let mut config = AspirationWindowConfig::default();
        config.base_window_size = 1; // Very small window
        config.max_researches = 1; // Very few re-searches
        engine.update_aspiration_window_config(config).unwrap();

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result = searcher.search(&mut engine, &board, &captured_pieces, player);

        // Should still return a valid result
        assert!(result.is_some());

        // Check that error recovery was handled
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
    }

    #[test]
    fn test_aspiration_windows_graceful_degradation() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Simulate poor performance by setting up high failure rates
        engine.aspiration_stats.total_searches = 200;
        engine.aspiration_stats.fail_lows = 100;
        engine.aspiration_stats.fail_highs = 80;

        // Should disable aspiration windows due to poor performance
        assert!(engine.should_disable_aspiration_windows());

        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result = searcher.search(&mut engine, &board, &captured_pieces, player);

        // Should still work with fallback to full-width search
        assert!(result.is_some());
    }

    // ===== COMPREHENSIVE WINDOW SIZE CALCULATION TESTS =====

    #[test]
    fn test_comprehensive_window_size_calculation() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test comprehensive window size calculation with various parameters
        let window_size = engine.calculate_comprehensive_window_size(
            3,    // depth
            100,  // previous_score
            1,    // recent_failures
            0.5,  // position_complexity
            500,  // time_remaining_ms
            1000, // total_time_ms
            0.8,  // recent_success_rate
            25,   // move_count
        );

        assert!(window_size >= 10);
        assert!(window_size <= 200);

        // Test with different parameters
        let window_size2 = engine.calculate_comprehensive_window_size(
            5,    // depth
            -50,  // previous_score
            3,    // recent_failures
            0.8,  // position_complexity
            200,  // time_remaining_ms
            1000, // total_time_ms
            0.6,  // recent_success_rate
            40,   // move_count
        );

        assert!(window_size2 >= 10);
        assert!(window_size2 <= 200);
        assert!(window_size2 != window_size); // Should be different due to different parameters
    }

    #[test]
    fn test_window_size_presets_integration() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test aggressive preset
        engine
            .apply_aspiration_window_preset(AspirationWindowPlayingStyle::Aggressive)
            .unwrap();
        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result1 = searcher.search(&mut engine, &board, &captured_pieces, player);
        assert!(result1.is_some());

        // Test conservative preset
        engine
            .apply_aspiration_window_preset(AspirationWindowPlayingStyle::Conservative)
            .unwrap();
        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result2 = searcher.search(&mut engine, &board, &captured_pieces, player);
        assert!(result2.is_some());

        // Test balanced preset
        engine
            .apply_aspiration_window_preset(AspirationWindowPlayingStyle::Balanced)
            .unwrap();
        let mut searcher = IterativeDeepening::new(3, 1000, None, None);
        let result3 = searcher.search(&mut engine, &board, &captured_pieces, player);
        assert!(result3.is_some());
    }

    // ===== MEMORY MANAGEMENT INTEGRATION TESTS =====

    #[test]
    fn test_aspiration_windows_memory_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Simulate large statistics
        engine.aspiration_stats.total_searches = 1_000_000;
        engine.aspiration_stats.average_window_size = 45.5;
        engine.previous_scores = vec![100; 2000]; // Large vector

        // Test memory optimization
        engine.optimize_aspiration_window_memory();

        // Statistics should be reset
        assert_eq!(engine.aspiration_stats.total_searches, 0);
        assert_eq!(engine.aspiration_stats.average_window_size, 0.0);

        // Previous scores should be cleared
        assert!(engine.previous_scores.is_empty());
    }

    // ===== CONCURRENT ACCESS TESTS =====

    #[test]
    fn test_aspiration_windows_concurrent_access() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test multiple searches in sequence (simulating concurrent access)
        for i in 0..10 {
            let mut searcher = IterativeDeepening::new(2, 500, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());

            // Check that statistics are being updated correctly
            let stats = engine.get_aspiration_window_stats();
            assert!(stats.total_searches >= i + 1);
        }
    }
}
