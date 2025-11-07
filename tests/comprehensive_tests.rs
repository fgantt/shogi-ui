#![cfg(feature = "legacy-tests")]
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(test)]
mod comprehensive_tests {
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

    // ===== CONFIGURATION VALIDATION TESTS =====

    #[test]
    fn test_configuration_validation_comprehensive() {
        // Test all configuration parameters
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

        // Test edge cases
        let edge_cases = vec![
            (
                QuiescenceConfig {
                    max_depth: 1,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    max_depth: 20,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    max_depth: 0,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    max_depth: 21,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    futility_margin: 0,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    futility_margin: 1000,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    futility_margin: -1,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    futility_margin: 1001,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    delta_margin: 0,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    delta_margin: 1000,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    delta_margin: -1,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    delta_margin: 1001,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    tt_size_mb: 1,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    tt_size_mb: 1024,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    tt_size_mb: 0,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    tt_size_mb: 1025,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    tt_cleanup_threshold: 1,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    tt_cleanup_threshold: 1000000,
                    ..valid_config.clone()
                },
                true,
            ),
            (
                QuiescenceConfig {
                    tt_cleanup_threshold: 0,
                    ..valid_config.clone()
                },
                false,
            ),
            (
                QuiescenceConfig {
                    tt_cleanup_threshold: 1000001,
                    ..valid_config.clone()
                },
                false,
            ),
        ];

        for (config, should_be_valid) in edge_cases {
            let result = config.validate();
            if should_be_valid {
                assert!(result.is_ok(), "Config should be valid: {:?}", config);
            } else {
                assert!(result.is_err(), "Config should be invalid: {:?}", config);
            }
        }
    }

    #[test]
    fn test_configuration_clamping_comprehensive() {
        let mut config = QuiescenceConfig {
            max_depth: 25,                 // Too high
            futility_margin: -50,          // Too low
            delta_margin: 1500,            // Too high
            tt_size_mb: 0,                 // Too low
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
    fn test_configuration_summary() {
        let config = QuiescenceConfig::default();
        let summary = config.summary();

        assert!(summary.contains("depth=8"));
        assert!(summary.contains("delta_pruning=true"));
        assert!(summary.contains("futility_pruning=true"));
        assert!(summary.contains("extensions=true"));
        assert!(summary.contains("tt=true"));
        assert!(summary.contains("tt_size=4MB"));
        assert!(summary.contains("cleanup_threshold=10000"));
    }

    // ===== MOVE ORDERING TESTS =====

    #[test]
    fn test_move_ordering_checks_first() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;

        // Create test moves with different priorities
        let mut moves = vec![
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                false,
                None,
                false,
                false,
            ),
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                true,
                None,
                true,
                false,
            ), // Check
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                true,
                None,
                false,
                false,
            ), // Capture
        ];

        let sorted_moves = engine.sort_quiescence_moves(&moves);

        // Check should be first
        assert!(sorted_moves[0].gives_check);
    }

    #[test]
    fn test_move_ordering_mvv_lva() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;

        // Create test moves with different MVV-LVA values
        let mut moves = vec![
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                true,
                Some(Piece {
                    piece_type: PieceType::Pawn,
                    player: player.opposite(),
                }),
                false,
                false,
            ), // Pawn captures pawn
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                true,
                Some(Piece {
                    piece_type: PieceType::Rook,
                    player: player.opposite(),
                }),
                false,
                false,
            ), // Pawn captures rook
        ];

        let sorted_moves = engine.sort_quiescence_moves(&moves);

        // Rook capture should be first (higher value)
        assert_eq!(
            sorted_moves[0].captured_piece.as_ref().unwrap().piece_type,
            PieceType::Rook
        );
    }

    #[test]
    fn test_move_ordering_promotions() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;

        // Create test moves with promotions
        let mut moves = vec![
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                false,
                false,
                None,
                false,
                false,
            ), // Non-promotion
            Move::new_move(
                Position { row: 0, col: 0 },
                Position { row: 1, col: 1 },
                PieceType::Pawn,
                player,
                true,
                false,
                None,
                false,
                false,
            ), // Promotion
        ];

        let sorted_moves = engine.sort_quiescence_moves(&moves);

        // Promotion should be first
        assert!(sorted_moves[0].is_promotion);
    }

    #[test]
    fn test_move_ordering_tactical_threats() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;

        // Test tactical threat assessment
        let moves =
            engine
                .move_generator
                .generate_quiescence_moves(&board, player, &captured_pieces);
        let sorted_moves = engine.sort_quiescence_moves(&moves);

        // Should be sorted by tactical importance
        assert_eq!(moves.len(), sorted_moves.len());

        // First few moves should be most tactically important
        for (i, mv) in sorted_moves.iter().enumerate().take(3) {
            if i == 0 {
                assert!(mv.gives_check || mv.is_capture || mv.is_promotion);
            }
        }
    }

    // ===== TRANSPOSITION TABLE TESTS =====

    #[test]
    fn test_tt_basic_functionality() {
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
            3,
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
            3,
        );

        // Results should be identical
        assert_eq!(result1, result2);

        let stats = engine.get_quiescence_stats();
        assert!(stats.tt_hits > 0);
    }

    #[test]
    fn test_tt_different_depths() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        // Enable TT
        let mut config = QuiescenceConfig::default();
        config.enable_tt = true;
        engine.update_quiescence_config(config);

        // Search at depth 3
        let result1 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3,
        );

        // Search at depth 5 (should not hit TT from depth 3)
        let result2 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5,
        );

        // Results may be different
        assert!(result1 > -10000 && result1 < 10000);
        assert!(result2 > -10000 && result2 < 10000);
    }

    #[test]
    fn test_tt_cleanup() {
        let mut engine = create_test_engine();

        // Test TT cleanup
        let initial_size = engine.quiescence_tt_size();
        engine.cleanup_quiescence_tt(0); // Force cleanup
        let final_size = engine.quiescence_tt_size();

        assert!(final_size <= initial_size);
    }

    #[test]
    fn test_tt_size_management() {
        let mut engine = create_test_engine();

        // Test TT size management
        let initial_size = engine.quiescence_tt_size();

        // Update TT size
        let result = engine.update_quiescence_tt_size(8);
        assert!(result.is_ok());

        let new_size = engine.quiescence_tt_size();
        assert!(new_size >= initial_size);
    }

    #[test]
    fn test_tt_disabled() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        // Disable TT
        let mut config = QuiescenceConfig::default();
        config.enable_tt = false;
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
            3,
        );

        // Second search (should not hit TT)
        let result2 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3,
        );

        // Results should be identical (but no TT hits)
        assert_eq!(result1, result2);

        let stats = engine.get_quiescence_stats();
        assert_eq!(stats.tt_hits, 0);
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_quiescence_integration_comprehensive() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        // Test with all features enabled
        let mut config = QuiescenceConfig::default();
        config.enable_delta_pruning = true;
        config.enable_futility_pruning = true;
        config.enable_selective_extensions = true;
        config.enable_tt = true;
        engine.update_quiescence_config(config);

        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        assert!(result > -10000 && result < 10000);

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
        assert!(stats.moves_ordered > 0);
    }

    #[test]
    fn test_quiescence_performance_metrics() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        engine.reset_quiescence_stats();

        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3,
        );

        let stats = engine.get_quiescence_stats();

        // Test performance metrics
        assert!(stats.pruning_efficiency() >= 0.0 && stats.pruning_efficiency() <= 100.0);
        assert!(stats.tt_hit_rate() >= 0.0 && stats.tt_hit_rate() <= 100.0);
        assert!(stats.extension_rate() >= 0.0 && stats.extension_rate() <= 100.0);

        let (check_pct, capture_pct, promotion_pct) = stats.move_type_distribution();
        assert!(check_pct >= 0.0 && check_pct <= 100.0);
        assert!(capture_pct >= 0.0 && capture_pct <= 100.0);
        assert!(promotion_pct >= 0.0 && promotion_pct <= 100.0);
    }

    #[test]
    fn test_quiescence_error_handling() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        // Test with invalid parameters
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            10000, // alpha > beta (invalid)
            -10000,
            &time_source,
            1000,
            3,
        );

        // Should handle gracefully
        assert!(result > -10000 && result < 10000);
    }

    #[test]
    fn test_quiescence_time_limit_handling() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        // Test with very short time limit
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1, // 1ms time limit
            4,
        );

        // Should complete within time limit
        assert!(result > -10000 && result < 10000);
    }

    #[test]
    fn test_quiescence_configuration_persistence() {
        let mut engine = create_test_engine();

        // Test that configuration persists across operations
        let mut config = QuiescenceConfig::default();
        config.max_depth = 12;
        config.futility_margin = 300;
        engine.update_quiescence_config(config);

        let current_config = engine.get_quiescence_config();
        assert_eq!(current_config.max_depth, 12);
        assert_eq!(current_config.futility_margin, 300);

        // Configuration should persist after search
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();

        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            3,
        );

        let current_config = engine.get_quiescence_config();
        assert_eq!(current_config.max_depth, 12);
        assert_eq!(current_config.futility_margin, 300);
    }
}
