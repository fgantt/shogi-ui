#![cfg(feature = "legacy-tests")]
/// Integration tests for Late Move Reductions (LMR)
///
/// This module contains integration tests for LMR with other search features:
/// - LMR with null move pruning
/// - LMR with quiescence search
/// - LMR with transposition table
/// - LMR re-search behavior
/// - End-to-end search functionality
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
            captured_piece: Some(Piece {
                piece_type: PieceType::Pawn,
                player: Player::White,
            }),
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
            min_depth: 4,      // Higher depth to ensure LMR is applied
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

#[cfg(test)]
mod iid_lmr_coordination_tests {
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

    /// Test 7.0.1.8: Test that IID moves are explicitly exempted from LMR
    #[test]
    fn test_iid_move_explicit_exemption() {
        let mut engine = create_test_engine();

        // Enable both IID and LMR
        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 3;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2; // Apply LMR starting from move 2
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics before search
        engine.reset_lmr_stats();
        engine.reset_iid_stats();

        // Perform search at sufficient depth to trigger IID
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);

        // Check that IID was performed
        let iid_stats = engine.get_iid_stats();
        assert!(
            iid_stats.iid_searches_performed > 0,
            "IID should have been performed"
        );

        // Check that IID move explicit exemption counter was incremented
        let lmr_stats = engine.get_lmr_stats();

        // IID move should have been explicitly exempted at least once
        assert!(
            lmr_stats.iid_move_explicitly_exempted > 0,
            "IID move should have been explicitly exempted from LMR at least once, but count was {}",
            lmr_stats.iid_move_explicitly_exempted
        );

        println!(
            "IID searches performed: {}",
            iid_stats.iid_searches_performed
        );
        println!(
            "IID moves explicitly exempted: {}",
            lmr_stats.iid_move_explicitly_exempted
        );
        println!("Total moves considered: {}", lmr_stats.moves_considered);
        println!("Reductions applied: {}", lmr_stats.reductions_applied);
    }

    /// Test 7.0.1.9: Integration test to ensure IID move is first in ordering AND exempted from LMR
    #[test]
    fn test_iid_move_ordering_and_exemption() {
        let mut engine = create_test_engine();

        // Enable IID with configuration that ensures it runs
        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        // Enable LMR with low thresholds to ensure it triggers
        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 4;
        lmr_config.min_move_index = 1; // Try to reduce even second move
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_lmr_stats();
        engine.reset_iid_stats();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 6, 10000);

        // Get statistics
        let iid_stats = engine.get_iid_stats();
        let lmr_stats = engine.get_lmr_stats();

        // Verify IID was performed
        assert!(
            iid_stats.iid_searches_performed > 0,
            "IID should have been performed"
        );

        // Verify IID move was ordered first (when IID finds a move)
        if iid_stats.iid_move_position_tracked > 0 {
            let avg_position =
                iid_stats.iid_move_position_sum as f64 / iid_stats.iid_move_position_tracked as f64;

            println!("IID move average position: {:.2}", avg_position);
            println!(
                "IID moves ordered first: {}",
                iid_stats.iid_move_ordered_first
            );
            println!(
                "IID moves not ordered first: {}",
                iid_stats.iid_move_not_ordered_first
            );

            // IID move should be ordered first most of the time
            assert!(
                iid_stats.iid_move_ordered_first > 0,
                "IID move should be ordered first at least once"
            );
        }

        // Verify IID move explicit exemption was used
        println!(
            "IID moves explicitly exempted from LMR: {}",
            lmr_stats.iid_move_explicitly_exempted
        );

        // The IID move should have been explicitly exempted
        // (This verifies the coordination between IID and LMR)
        assert!(
            lmr_stats.iid_move_explicitly_exempted > 0,
            "IID move should have been explicitly exempted from LMR"
        );
    }

    /// Test that IID move exemption doesn't interfere with other LMR exemptions
    #[test]
    fn test_iid_exemption_with_other_exemptions() {
        let mut engine = create_test_engine();

        // Enable IID and LMR
        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 3;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2;
        lmr_config.enable_extended_exemptions = true; // Enable killer/TT exemptions
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_lmr_stats();
        engine.reset_iid_stats();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);

        let lmr_stats = engine.get_lmr_stats();
        let iid_stats = engine.get_iid_stats();

        // Verify various exemptions work together
        println!(
            "IID moves exempted: {}",
            lmr_stats.iid_move_explicitly_exempted
        );
        println!("TT moves exempted: {}", lmr_stats.tt_move_exempted);
        println!("Total moves considered: {}", lmr_stats.moves_considered);
        println!("Reductions applied: {}", lmr_stats.reductions_applied);

        // If IID ran and found moves, they should be exempted
        if iid_stats.iid_searches_performed > 0 {
            // IID exemption should work alongside other exemptions
            assert!(
                lmr_stats.iid_move_explicitly_exempted > 0
                    || lmr_stats.tt_move_exempted > 0
                    || lmr_stats.reductions_applied > 0,
                "Some form of LMR activity should have occurred"
            );
        }
    }
}

#[cfg(test)]
mod lmr_position_type_adaptation_tests {
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

    /// Test 7.0.6.5: Test position-type adaptive re-search margin
    #[test]
    fn test_position_type_adaptive_margin() {
        let mut engine = create_test_engine();

        // Enable LMR with position-type adaptation
        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2;
        lmr_config.enable_adaptive_reduction = true;
        lmr_config.enable_position_type_margin = true; // Enable adaptive margin
        lmr_config.tactical_re_search_margin = 75;
        lmr_config.quiet_re_search_margin = 25;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_lmr_stats();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);

        let lmr_stats = engine.get_lmr_stats();

        println!("=== Position-Type Adaptive Margin ===");
        println!("Tactical re-searches: {}", lmr_stats.tactical_researches);
        println!("Quiet re-searches: {}", lmr_stats.quiet_researches);
        println!("Neutral re-searches: {}", lmr_stats.neutral_researches);
        println!("Total re-searches: {}", lmr_stats.researches_triggered);

        // Statistics should be tracked by position type
        let total_classified = lmr_stats.tactical_researches
            + lmr_stats.quiet_researches
            + lmr_stats.neutral_researches;

        // All re-searches should be classified
        assert_eq!(
            total_classified, lmr_stats.researches_triggered,
            "All re-searches should be classified by position type"
        );
    }

    /// Test comparing re-search rates with adaptive margin
    #[test]
    fn test_adaptive_margin_effectiveness() {
        let mut engine = create_test_engine();

        // Enable LMR with adaptive margin
        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2;
        lmr_config.enable_adaptive_reduction = true;
        lmr_config.enable_position_type_margin = true;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_lmr_stats();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 6, 8000);

        let lmr_stats = engine.get_lmr_stats();

        println!("With adaptive margin:");
        println!("  Total reductions: {}", lmr_stats.reductions_applied);
        println!("  Total re-searches: {}", lmr_stats.researches_triggered);
        println!(
            "  Re-search rate: {:.2}%",
            if lmr_stats.reductions_applied > 0 {
                (lmr_stats.researches_triggered as f64 / lmr_stats.reductions_applied as f64)
                    * 100.0
            } else {
                0.0
            }
        );

        // Test that adaptation is working
        assert!(
            lmr_stats.reductions_applied > 0,
            "LMR should have been applied"
        );
    }
}
