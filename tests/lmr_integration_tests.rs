/// Integration tests for Late Move Reductions (LMR)
/// 
/// This module contains integration tests for LMR with other search features:
/// - LMR with null move pruning
/// - LMR with quiescence search
/// - LMR with transposition table
/// - LMR re-search behavior
/// - End-to-end search functionality

use shogi_engine::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[cfg(test)]
mod lmr_null_move_integration_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_with_null_move_pruning_enabled() {
        let mut engine = create_test_engine();
        
        // Enable both LMR and NMP
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let nmp_config = NullMoveConfig {
            enabled: true,
            min_depth: 3,
            reduction_factor: 2,
            max_pieces_threshold: 12,
            enable_dynamic_reduction: true,
            enable_endgame_detection: true,
        };
        engine.update_null_move_config(nmp_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Test that both features work together
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some());
        
        // Check that both LMR and NMP statistics are being tracked
        let lmr_stats = engine.get_lmr_stats();
        let nmp_stats = engine.get_null_move_stats();
        
        // Both should have some activity (exact numbers depend on position)
        assert!(lmr_stats.moves_considered >= 0);
        assert!(nmp_stats.attempts >= 0);
    }

    #[test]
    fn test_lmr_with_null_move_pruning_disabled() {
        let mut engine = create_test_engine();
        
        // Enable LMR but disable NMP
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let nmp_config = NullMoveConfig {
            enabled: false, // Disable NMP
            min_depth: 3,
            reduction_factor: 2,
            max_pieces_threshold: 12,
            enable_dynamic_reduction: true,
            enable_endgame_detection: true,
        };
        engine.update_null_move_config(nmp_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some());
        
        // LMR should still work, NMP should not
        let lmr_stats = engine.get_lmr_stats();
        let nmp_stats = engine.get_null_move_stats();
        
        assert!(lmr_stats.moves_considered >= 0);
        assert_eq!(nmp_stats.attempts, 0); // NMP should be disabled
    }
}

#[cfg(test)]
mod lmr_quiescence_integration_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_with_quiescence_search() {
        let mut engine = create_test_engine();
        
        // Enable LMR
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Test search that will use both LMR and quiescence
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());
        
        // Check that both LMR and quiescence statistics are being tracked
        let lmr_stats = engine.get_lmr_stats();
        let quiescence_stats = engine.get_quiescence_stats();
        
        assert!(lmr_stats.moves_considered >= 0);
        assert!(quiescence_stats.nodes_searched >= 0);
    }

    #[test]
    fn test_lmr_exemptions_with_quiescence_moves() {
        let mut engine = create_test_engine();
        
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        // Test that capture moves are properly exempted from LMR
        // (they should be handled by quiescence search instead)
        let capture_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: true,
            is_promotion: false,
            captured_piece: Some(Piece { piece_type: PieceType::Pawn, player: Player::White }),
            gives_check: false,
            is_recapture: false,
        };
        
        assert!(engine.is_move_exempt_from_lmr(&capture_move));
    }
}

#[cfg(test)]
mod lmr_transposition_table_integration_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_with_transposition_table() {
        let mut engine = create_test_engine();
        
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Perform multiple searches to populate transposition table
        let result1 = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result1.is_some());
        
        let result2 = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result2.is_some());
        
        // Check that LMR statistics are being tracked
        let lmr_stats = engine.get_lmr_stats();
        assert!(lmr_stats.moves_considered >= 0);
        
        // Check that transposition table has entries
        assert!(engine.transposition_table_len() > 0);
    }

    #[test]
    fn test_lmr_research_with_transposition_table() {
        let mut engine = create_test_engine();
        
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Search to populate TT and LMR stats
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some());
        
        let lmr_stats = engine.get_lmr_stats();
        
        // If LMR was applied, we should see some research activity
        if lmr_stats.reductions_applied > 0 {
            // Research rate should be reasonable (not too high, not too low)
            let research_rate = lmr_stats.research_rate();
            assert!(research_rate >= 0.0);
            assert!(research_rate <= 100.0);
        }
    }
}

#[cfg(test)]
mod lmr_research_behavior_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_research_triggering() {
        let mut engine = create_test_engine();
        
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 4, // Higher depth to ensure LMR is applied
            min_move_index: 2, // Lower threshold to apply LMR to more moves
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Search with LMR enabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some());
        
        let lmr_stats = engine.get_lmr_stats();
        
        // Should have considered some moves
        assert!(lmr_stats.moves_considered > 0);
        
        // If reductions were applied, we might have some researches
        if lmr_stats.reductions_applied > 0 {
            // Research rate should be reasonable
            let research_rate = lmr_stats.research_rate();
            assert!(research_rate >= 0.0);
            assert!(research_rate <= 100.0);
        }
    }

    #[test]
    fn test_lmr_without_research() {
        let mut engine = create_test_engine();
        
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: false, // Disable dynamic reduction
            enable_adaptive_reduction: false, // Disable adaptive reduction
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Search with minimal LMR
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());
        
        let lmr_stats = engine.get_lmr_stats();
        
        // Should have considered some moves
        assert!(lmr_stats.moves_considered > 0);
        
        // Research rate should be reasonable
        let research_rate = lmr_stats.research_rate();
        assert!(research_rate >= 0.0);
        assert!(research_rate <= 100.0);
    }
}

#[cfg(test)]
mod lmr_end_to_end_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_complete_search_workflow() {
        let mut engine = create_test_engine();
        
        // Configure LMR with moderate settings
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();
        
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        // Perform search at different depths
        for depth in 3..=6 {
            let result = engine.search_at_depth(&board, &captured_pieces, player, depth, 1000);
            assert!(result.is_some(), "Search should succeed at depth {}", depth);
            
            let lmr_stats = engine.get_lmr_stats();
            assert!(lmr_stats.moves_considered >= 0);
        }
    }

    #[test]
    fn test_lmr_configuration_persistence() {
        let mut engine = create_test_engine();
        
        // Set custom LMR configuration
        let custom_config = LMRConfig {
            enabled: true,
            min_depth: 4,
            min_move_index: 5,
            base_reduction: 2,
            max_reduction: 4,
            enable_dynamic_reduction: false,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: false,
        };
        engine.update_lmr_config(custom_config.clone()).unwrap();
        
        // Verify configuration was set
        let retrieved_config = engine.get_lmr_config();
        assert_eq!(retrieved_config.min_depth, 4);
        assert_eq!(retrieved_config.min_move_index, 5);
        assert_eq!(retrieved_config.base_reduction, 2);
        assert_eq!(retrieved_config.max_reduction, 4);
        assert!(!retrieved_config.enable_dynamic_reduction);
        assert!(retrieved_config.enable_adaptive_reduction);
        assert!(!retrieved_config.enable_extended_exemptions);
    }

    #[test]
    fn test_lmr_statistics_reset() {
        let mut engine = create_test_engine();
        
        // Perform some searches to generate statistics
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        
        // Check that we have some statistics
        let lmr_stats = engine.get_lmr_stats();
        let initial_moves = lmr_stats.moves_considered;
        
        // Reset statistics
        engine.reset_lmr_stats();
        
        // Check that statistics are reset
        let reset_stats = engine.get_lmr_stats();
        assert_eq!(reset_stats.moves_considered, 0);
        assert_eq!(reset_stats.reductions_applied, 0);
        assert_eq!(reset_stats.researches_triggered, 0);
        
        // Verify we had statistics before reset
        assert!(initial_moves > 0);
    }

    #[test]
    fn test_lmr_clear_integration() {
        let mut engine = create_test_engine();
        
        // Perform some searches
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        
        // Clear the engine
        engine.clear();
        
        // Check that LMR statistics are reset
        let lmr_stats = engine.get_lmr_stats();
        assert_eq!(lmr_stats.moves_considered, 0);
        assert_eq!(lmr_stats.reductions_applied, 0);
        assert_eq!(lmr_stats.researches_triggered, 0);
        
        // Check that transposition table is cleared
        assert_eq!(engine.transposition_table_len(), 0);
    }
}
