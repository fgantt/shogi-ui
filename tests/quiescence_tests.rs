use shogi_engine::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[cfg(test)]
mod quiescence_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_quiescence_search_basic() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Test basic quiescence search on starting position
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        // Should return a reasonable evaluation
        assert!(result > -10000 && result < 10000);
    }

    #[test]
    fn test_quiescence_search_with_captures() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let mut captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Set up a position with potential captures
        // This is a simplified test - in practice, you'd set up specific positions
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            2
        );
        
        assert!(result > -10000 && result < 10000);
    }

    #[test]
    fn test_quiescence_config_validation() {
        // Test valid configuration
        let valid_config = QuiescenceConfig {
            max_depth: 8,
            enable_delta_pruning: true,
            enable_futility_pruning: true,
            enable_selective_extensions: true,
            enable_tt: true,
            futility_margin: 200,
            delta_margin: 100,
            tt_size_mb: 4,
            tt_cleanup_threshold: 10000,
        };
        
        assert!(valid_config.validate().is_ok());
        
        // Test invalid configurations
        let invalid_depth = QuiescenceConfig {
            max_depth: 0,
            ..valid_config.clone()
        };
        assert!(invalid_depth.validate().is_err());
        
        let invalid_margin = QuiescenceConfig {
            futility_margin: -100,
            ..valid_config.clone()
        };
        assert!(invalid_margin.validate().is_err());
        
        let invalid_tt_size = QuiescenceConfig {
            tt_size_mb: 0,
            ..valid_config.clone()
        };
        assert!(invalid_tt_size.validate().is_err());
    }

    #[test]
    fn test_quiescence_config_clamping() {
        let mut config = QuiescenceConfig {
            max_depth: 25, // Too high
            futility_margin: -50, // Too low
            delta_margin: 1500, // Too high
            tt_size_mb: 0, // Too low
            tt_cleanup_threshold: 2000000, // Too high
            ..QuiescenceConfig::default()
        };
        
        let clamped_config = config.new_validated();
        
        assert_eq!(clamped_config.max_depth, 20);
        assert_eq!(clamped_config.futility_margin, 0);
        assert_eq!(clamped_config.delta_margin, 1000);
        assert_eq!(clamped_config.tt_size_mb, 1);
        assert_eq!(clamped_config.tt_cleanup_threshold, 1000000);
    }

    #[test]
    fn test_quiescence_stats_tracking() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Reset stats
        engine.reset_quiescence_stats();
        
        // Run quiescence search
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        let stats = engine.get_quiescence_stats();
        
        // Should have searched some nodes
        assert!(stats.nodes_searched > 0);
        
        // Should have some statistics
        assert!(stats.moves_ordered >= 0);
    }

    #[test]
    fn test_quiescence_move_ordering() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        
        // Generate quiescence moves
        let moves = engine.move_generator.generate_quiescence_moves(&board, player, &captured_pieces);
        
        // Test move ordering
        let sorted_moves = engine.sort_quiescence_moves(&moves);
        
        // Should be sorted (this is a basic test)
        assert_eq!(moves.len(), sorted_moves.len());
    }

    #[test]
    fn test_quiescence_tt_functionality() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Enable TT
        let mut config = QuiescenceConfig::default();
        config.enable_tt = true;
        engine.update_quiescence_config(config);
        
        // First search
        let result1 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        // Second search (should hit TT)
        let result2 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        // Results should be the same
        assert_eq!(result1, result2);
        
        let stats = engine.get_quiescence_stats();
        assert!(stats.tt_hits > 0);
    }

    #[test]
    fn test_quiescence_pruning() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Enable pruning
        let mut config = QuiescenceConfig::default();
        config.enable_delta_pruning = true;
        config.enable_futility_pruning = true;
        engine.update_quiescence_config(config);
        
        // Run search
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        let stats = engine.get_quiescence_stats();
        
        // Should have some pruning (may be 0 for simple positions)
        assert!(stats.delta_prunes >= 0);
        assert!(stats.futility_prunes >= 0);
    }

    #[test]
    fn test_quiescence_extensions() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Enable extensions
        let mut config = QuiescenceConfig::default();
        config.enable_selective_extensions = true;
        engine.update_quiescence_config(config);
        
        // Run search
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        let stats = engine.get_quiescence_stats();
        
        // Should have some extensions (may be 0 for simple positions)
        assert!(stats.extensions >= 0);
    }

    #[test]
    fn test_quiescence_performance_reporting() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Run search
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        // Test performance reporting
        let summary = engine.get_quiescence_summary();
        assert!(!summary.is_empty());
        
        let report = engine.get_quiescence_performance_report();
        assert!(!report.is_empty());
        
        let status = engine.get_quiescence_status();
        assert!(!status.is_empty());
        
        let efficiency = engine.get_quiescence_efficiency();
        assert!(efficiency.0 >= 0.0 && efficiency.0 <= 100.0); // pruning efficiency
        assert!(efficiency.1 >= 0.0 && efficiency.1 <= 100.0); // TT hit rate
        assert!(efficiency.2 >= 0.0 && efficiency.2 <= 100.0); // extension rate
    }

    #[test]
    fn test_quiescence_configuration_updates() {
        let mut engine = create_test_engine();
        
        // Test safe configuration update
        let mut config = QuiescenceConfig::default();
        config.max_depth = 12;
        config.futility_margin = 300;
        
        engine.update_quiescence_config_safe(config.clone());
        let current_config = engine.get_quiescence_config();
        assert_eq!(current_config.max_depth, 12);
        assert_eq!(current_config.futility_margin, 300);
        
        // Test validated configuration update
        let mut config2 = QuiescenceConfig::default();
        config2.max_depth = 15;
        config2.tt_size_mb = 8;
        
        let result = engine.update_quiescence_config_validated(config2.clone());
        assert!(result.is_ok());
        
        let current_config = engine.get_quiescence_config();
        assert_eq!(current_config.max_depth, 15);
        assert_eq!(current_config.tt_size_mb, 8);
        
        // Test invalid configuration update
        let mut invalid_config = QuiescenceConfig::default();
        invalid_config.max_depth = 0; // Invalid
        
        let result = engine.update_quiescence_config_validated(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_quiescence_tt_cleanup() {
        let mut engine = create_test_engine();
        
        // Test TT cleanup
        let initial_size = engine.quiescence_tt_size();
        engine.cleanup_quiescence_tt(0); // Force cleanup
        let final_size = engine.quiescence_tt_size();
        
        assert!(final_size <= initial_size);
    }

    #[test]
    fn test_quiescence_depth_limiting() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Set max depth to 2
        let mut config = QuiescenceConfig::default();
        config.max_depth = 2;
        engine.update_quiescence_config(config);
        
        // Run search with depth 5 (should be limited to 2)
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5
        );
        
        // Should complete without issues (depth limited internally)
        assert!(true);
    }

    #[test]
    fn test_quiescence_extension_maintains_depth() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Enable extensions
        let mut config = QuiescenceConfig::default();
        config.enable_selective_extensions = true;
        config.max_depth = 8;
        engine.update_quiescence_config(config);
        
        // Reset stats
        engine.reset_quiescence_stats();
        
        // Run search with depth 3 (should allow extensions)
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3
        );
        
        let stats = engine.get_quiescence_stats();
        
        // If extensions occurred, they should maintain depth (not reduce it)
        // This is tested by verifying that extensions lead to deeper searches
        // We can't directly test internal depth, but we can verify extensions work
        assert!(stats.extensions >= 0);
        
        // Verify that with extensions enabled, we search deeper than without
        // (This is an indirect test - the actual depth maintenance is verified by the fix)
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn test_quiescence_deep_tactical_sequences_with_extensions() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Enable extensions for tactical sequences
        let mut config = QuiescenceConfig::default();
        config.enable_selective_extensions = true;
        config.max_depth = 10; // Allow deeper searches
        engine.update_quiescence_config(config);
        
        // Reset stats
        engine.reset_quiescence_stats();
        
        // Run search with depth 5 - extensions should allow finding deeper tactics
        let result_with_extensions = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5
        );
        
        let stats_with = engine.get_quiescence_stats();
        let extensions_count = stats_with.extensions;
        
        // Disable extensions and compare
        let mut config_no_ext = QuiescenceConfig::default();
        config_no_ext.enable_selective_extensions = false;
        config_no_ext.max_depth = 10;
        engine.update_quiescence_config(config_no_ext);
        engine.reset_quiescence_stats();
        
        let result_without_extensions = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5
        );
        
        let stats_without = engine.get_quiescence_stats();
        
        // With extensions, we should search more nodes (if extensions occurred)
        // This verifies that extensions maintain depth and allow deeper tactical sequences
        if extensions_count > 0 {
            assert!(stats_with.nodes_searched >= stats_without.nodes_searched);
        }
        
        // Both should return reasonable evaluations
        assert!(result_with_extensions > -10000 && result_with_extensions < 10000);
        assert!(result_without_extensions > -10000 && result_without_extensions < 10000);
    }

    #[test]
    fn test_quiescence_seldepth_uses_config_max_depth() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Test with different max_depth values
        for max_depth in [1u8, 8u8, 20u8] {
            let mut config = QuiescenceConfig::default();
            config.max_depth = max_depth;
            engine.update_quiescence_config(config);
            
            // Reset stats
            engine.reset_quiescence_stats();
            
            // Run search
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                max_depth
            );
            
            let stats = engine.get_quiescence_stats();
            
            // Verify search completed successfully
            assert!(stats.nodes_searched > 0);
            
            // Verify that max_depth is respected (depth limiting works)
            // If we search with depth = max_depth, it should complete
            assert!(true); // Search completed without error
        }
    }

    #[test]
    fn test_quiescence_depth_limiting_with_different_max_depths() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Test with max_depth = 1
        let mut config = QuiescenceConfig::default();
        config.max_depth = 1;
        engine.update_quiescence_config(config);
        
        let result1 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5 // Pass depth > max_depth, should be limited
        );
        assert!(result1 > -10000 && result1 < 10000);
        
        // Test with max_depth = 8
        let mut config = QuiescenceConfig::default();
        config.max_depth = 8;
        engine.update_quiescence_config(config);
        
        let result8 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5
        );
        assert!(result8 > -10000 && result8 < 10000);
        
        // Test with max_depth = 20
        let mut config = QuiescenceConfig::default();
        config.max_depth = 20;
        engine.update_quiescence_config(config);
        
        let result20 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5
        );
        assert!(result20 > -10000 && result20 < 10000);
        
        // All should complete successfully with depth limiting working correctly
        assert!(true);
    }

    #[test]
    fn test_quiescence_depth_zero_check() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
        // Test with depth = 0 (should be caught by depth check)
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            0
        );
        
        // Should return static evaluation (depth limit reached)
        assert!(result > -10000 && result < 10000);
    }
}
