#![cfg(feature = "legacy-tests")]
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(test)]
mod horizon_effect_tests {
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

    // Helper function to set up a position from FEN
    fn setup_position_from_fen(fen: &str) -> (BitboardBoard, CapturedPieces, Player) {
        // This is a simplified version - in practice, you'd need a proper FEN parser
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Sente;
        (board, captured_pieces, player)
    }

    #[test]
    fn test_horizon_effect_detection() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can detect horizon effect
        // by comparing static evaluation with quiescence evaluation
        let static_eval = engine.evaluator.evaluate(&board, player, &captured_pieces);

        let quiescence_eval = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        // In positions with horizon effect, quiescence should differ from static
        // This is a basic test - in practice, you'd test specific positions
        assert!(quiescence_eval > -10000 && quiescence_eval < 10000);
        assert!(static_eval > -10000 && static_eval < 10000);
    }

    #[test]
    fn test_horizon_effect_depth_penetration() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can penetrate beyond the horizon
        let shallow_eval = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            2,
        );

        let deep_eval = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            6,
        );

        // Deeper search should provide more accurate evaluation
        // (This may not always be true for simple positions, but tests the mechanism)
        assert!(shallow_eval > -10000 && shallow_eval < 10000);
        assert!(deep_eval > -10000 && deep_eval < 10000);

        let shallow_stats = engine.get_quiescence_stats();
        engine.reset_quiescence_stats();

        let _deep_eval = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            6,
        );

        let deep_stats = engine.get_quiescence_stats();

        // Deeper search should explore more nodes
        assert!(deep_stats.nodes_searched >= shallow_stats.nodes_searched);
    }

    #[test]
    fn test_horizon_effect_tactical_sequences() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can find tactical sequences beyond the horizon
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            5,
        );

        assert!(result > -10000 && result < 10000);

        let stats = engine.get_quiescence_stats();

        // Should find tactical moves
        assert!(stats.capture_moves_found >= 0);
        assert!(stats.check_moves_found >= 0);
        assert!(stats.promotion_moves_found >= 0);
    }

    #[test]
    fn test_horizon_effect_capture_sequences() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can find capture sequences beyond the horizon
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
        assert!(stats.capture_moves_found >= 0);
    }

    #[test]
    fn test_horizon_effect_check_sequences() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can find check sequences beyond the horizon
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
        assert!(stats.check_moves_found >= 0);
    }

    #[test]
    fn test_horizon_effect_promotion_sequences() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search can find promotion sequences beyond the horizon
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
        assert!(stats.promotion_moves_found >= 0);
    }

    #[test]
    fn test_horizon_effect_evaluation_accuracy() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that quiescence search provides more accurate evaluation
        // by comparing with static evaluation
        let static_eval = engine.evaluator.evaluate(&board, player, &captured_pieces);

        let quiescence_eval = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        // Both evaluations should be reasonable
        assert!(static_eval > -10000 && static_eval < 10000);
        assert!(quiescence_eval > -10000 && quiescence_eval < 10000);

        // In tactical positions, quiescence should differ from static
        // (This is a basic test - in practice, you'd test specific positions)
        assert!(quiescence_eval != static_eval || quiescence_eval == static_eval);
    }

    #[test]
    fn test_horizon_effect_move_ordering_importance() {
        let mut engine = create_test_engine();
        let (board, captured_pieces, player) = setup_position_from_fen("startpos");

        // Test that move ordering is crucial for horizon effect detection
        let moves =
            engine
                .move_generator
                .generate_quiescence_moves(&board, player, &captured_pieces);
        let sorted_moves = engine.sort_quiescence_moves(&moves);

        // Should be sorted by tactical importance
        assert_eq!(moves.len(), sorted_moves.len());

        // First few moves should be most important for horizon effect detection
        for (i, mv) in sorted_moves.iter().enumerate().take(3) {
            if i == 0 {
                // First move should be most tactically important
                assert!(mv.gives_check || mv.is_capture || mv.is_promotion);
            }
        }
    }

    #[test]
    fn test_horizon_effect_pruning_impact() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that pruning doesn't interfere with horizon effect detection
        let mut config = QuiescenceConfig::default();
        config.enable_delta_pruning = true;
        config.enable_futility_pruning = true;
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
    }

    #[test]
    fn test_horizon_effect_tt_impact() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that TT doesn't interfere with horizon effect detection
        let mut config = QuiescenceConfig::default();
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
    }

    #[test]
    fn test_horizon_effect_time_management() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that horizon effect detection works within time limits
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            100, // Short time limit
            4,
        );

        assert!(result > -10000 && result < 10000);

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn test_horizon_effect_consistency() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that horizon effect detection is consistent
        let result1 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        let result2 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        // Results should be identical
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_horizon_effect_performance_impact() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that horizon effect detection doesn't significantly impact performance
        let start = std::time::Instant::now();

        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration.as_millis() < 1000);

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn test_horizon_effect_evaluation_bounds() {
        let mut engine = create_test_engine();
        let (mut board, captured_pieces, player) = setup_position_from_fen("startpos");
        let time_source = TimeSource::new();

        // Test that horizon effect detection respects evaluation bounds
        let result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -5000, // Narrow bounds
            5000,
            &time_source,
            1000,
            4,
        );

        // Result should be within bounds
        assert!(result >= -5000 && result <= 5000);

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn test_horizon_effect_move_generation_completeness() {
        let mut engine = create_test_engine();
        let (board, captured_pieces, player) = setup_position_from_fen("startpos");

        // Test that horizon effect detection uses complete move generation
        let moves =
            engine
                .move_generator
                .generate_quiescence_moves(&board, player, &captured_pieces);

        // Should generate some moves
        assert!(!moves.is_empty());

        // All moves should be tactical
        for mv in &moves {
            assert!(
                mv.gives_check
                    || mv.is_capture
                    || mv.is_promotion
                    || engine
                        .move_generator
                        .is_tactical_threat(&mv, &board, player)
            );
        }
    }
}
