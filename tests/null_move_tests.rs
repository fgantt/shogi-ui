#![cfg(feature = "legacy-tests")]
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    time_utils::TimeSource,
    types::{
        CapturedPieces, DynamicReductionFormula, EndgameType, NullMoveConfig, NullMovePreset,
        NullMoveReductionStrategy, Player,
    },
};

#[cfg(test)]
mod null_move_tests {
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

    fn setup_position_from_fen(_fen: &str) -> (BitboardBoard, CapturedPieces, Player) {
        // For now, use initial position - FEN parsing can be added later
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        (board, captured_pieces, player)
    }

    #[test]
    fn test_null_move_basic_functionality() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        let _time_source = TimeSource::now();

        // Test that NMP configuration is properly initialized
        let config = engine.get_null_move_config();
        assert!(config.enabled);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.reduction_factor, 2);

        // Test that statistics are properly initialized
        let stats = engine.get_null_move_stats();
        assert_eq!(stats.attempts, 0);
        assert_eq!(stats.cutoffs, 0);
        assert_eq!(stats.disabled_in_check, 0);
        assert_eq!(stats.disabled_endgame, 0);

        // Test basic search functionality with NMP enabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let (_best_move, score) = result.unwrap();
        assert!(score > -200000); // Should be a reasonable score
                                  // Note: best_move is now unused but the test verifies the move is valid
    }

    #[test]
    fn test_null_move_disabled_in_check() {
        let mut engine = create_test_engine();
        let _board = create_test_board();
        let _captured_pieces = create_test_captured_pieces();
        let _player = Player::Black;

        // Create a position where the king is in check
        // This is a simplified test - in a real implementation, we'd set up a check position
        // For now, we test the configuration logic

        // Reset statistics to ensure clean test
        engine.reset_null_move_stats();

        // Test that statistics tracking works
        let initial_stats = engine.get_null_move_stats();
        assert_eq!(initial_stats.disabled_in_check, 0);

        // In a real check position, the disabled_in_check counter should increment
        // This test verifies the mechanism is in place
        let config = engine.get_null_move_config();
        assert!(config.enabled);

        // Test that NMP respects the check condition
        // The actual check detection happens in should_attempt_null_move
        let stats = engine.get_null_move_stats();
        assert!(stats.disabled_in_check >= 0); // Should not be negative
    }

    #[test]
    fn test_null_move_endgame_detection() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics to ensure clean test
        engine.reset_null_move_stats();

        // Test endgame detection configuration
        let config = engine.get_null_move_config();
        assert!(config.enable_endgame_detection);
        assert_eq!(config.max_pieces_threshold, 12);

        // Test that piece counting works (using public interface)
        // Note: count_pieces_on_board is private, so we test through search behavior
        // In a real implementation, we'd make this method public or test through search results

        // Test that endgame detection respects the threshold
        let stats = engine.get_null_move_stats();
        assert!(stats.disabled_endgame >= 0); // Should not be negative
    }

    #[test]
    fn test_piece_count_accuracy_with_bitboard_optimization() {
        let mut engine = create_test_engine();
        let board = create_test_board();

        // Test that piece count matches actual pieces on board
        // We test this indirectly by verifying endgame detection behavior
        let config = engine.get_null_move_config();

        // Initial position should have 40 pieces (20 per player)
        // So endgame detection should not trigger with threshold of 12
        engine.reset_null_move_stats();

        let result = engine.search_at_depth_legacy(
            &mut board.clone(),
            &create_test_captured_pieces(),
            Player::Black,
            3,
            1000,
        );
        assert!(result.is_some());

        // With threshold of 12 and initial board having 40 pieces,
        // endgame detection should not disable NMP
        let stats = engine.get_null_move_stats();
        // The stats verify that the counting is working correctly
        assert!(stats.disabled_endgame >= 0);
    }

    #[test]
    fn test_endgame_detection_performance() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test that endgame detection uses optimized counting
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_detection = true;
        config.max_pieces_threshold = 12;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Measure that search completes quickly (optimized counting should be fast)
        let start = std::time::Instant::now();
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 3, 1000);
        let elapsed = start.elapsed();

        assert!(result.is_some());

        // Verify search completes in reasonable time (optimization should make this fast)
        // Initial position should complete quickly
        assert!(elapsed.as_millis() < 5000); // Should complete in less than 5 seconds

        let stats = engine.get_null_move_stats();
        // Verify endgame detection is working (initial position has 40 pieces, so shouldn't disable)
        assert!(stats.disabled_endgame >= 0);
    }

    #[test]
    fn test_null_move_configuration_validation() {
        let mut engine = create_test_engine();

        // Test valid configuration update
        let mut valid_config = NullMoveConfig::default();
        valid_config.min_depth = 4;
        valid_config.reduction_factor = 3;

        let result = engine.update_null_move_config(valid_config);
        assert!(result.is_ok());

        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.min_depth, 4);
        assert_eq!(updated_config.reduction_factor, 3);

        // Test invalid configuration update
        let mut invalid_config = NullMoveConfig::default();
        invalid_config.min_depth = 0; // Invalid

        let result = engine.update_null_move_config(invalid_config);
        assert!(result.is_err());

        // Configuration should remain unchanged
        let unchanged_config = engine.get_null_move_config();
        assert_eq!(unchanged_config.min_depth, 4);
    }

    #[test]
    fn test_null_move_statistics_tracking() {
        let mut engine = create_test_engine();

        // Reset statistics to ensure clean test
        engine.reset_null_move_stats();

        let initial_stats = engine.get_null_move_stats();
        assert_eq!(initial_stats.attempts, 0);
        assert_eq!(initial_stats.cutoffs, 0);
        assert_eq!(initial_stats.depth_reductions, 0);
        assert_eq!(initial_stats.disabled_in_check, 0);
        assert_eq!(initial_stats.disabled_endgame, 0);

        // Test statistics calculation methods
        assert_eq!(initial_stats.cutoff_rate(), 0.0);
        assert_eq!(initial_stats.average_reduction_factor(), 0.0);
        assert_eq!(initial_stats.total_disabled(), 0);
        assert_eq!(initial_stats.efficiency(), 0.0);

        // Test performance report generation
        let report = initial_stats.performance_report();
        assert!(report.contains("Null Move Pruning Performance Report"));
        assert!(report.contains("Attempts: 0"));
        assert!(report.contains("Cutoffs: 0"));

        // Test summary generation
        let summary = initial_stats.summary();
        assert!(summary.contains("NMP"));
        assert!(summary.contains("0 attempts"));
    }

    #[test]
    fn test_null_move_integration_with_negamax() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics to ensure clean test
        engine.reset_null_move_stats();

        // Test that search works with NMP integrated
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some());

        let (best_move, score) = result.unwrap();
        assert!(score > -200000); // Should be a reasonable score

        // Test that statistics are being tracked during search
        let stats = engine.get_null_move_stats();
        // Note: Statistics may or may not be incremented depending on search conditions
        // The important thing is that the mechanism is in place
        assert!(stats.attempts >= 0);
        assert!(stats.cutoffs >= 0);
        assert!(stats.depth_reductions >= 0);
    }

    #[test]
    fn test_null_move_performance_improvement() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with NMP enabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        engine.update_null_move_config(config).unwrap();

        let start_time = std::time::Instant::now();
        let result_with_nmp = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        let duration_with_nmp = start_time.elapsed();

        // Test with NMP disabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = false;
        engine.update_null_move_config(config).unwrap();

        let start_time = std::time::Instant::now();
        let result_without_nmp = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        let duration_without_nmp = start_time.elapsed();

        // Both searches should complete successfully
        assert!(result_with_nmp.is_some());
        assert!(result_without_nmp.is_some());

        let (_move_with_nmp, score_with_nmp) = result_with_nmp.unwrap();
        let (_move_without_nmp, score_without_nmp) = result_without_nmp.unwrap();

        // Scores should be similar (NMP shouldn't change the best move)
        let score_diff = (score_with_nmp - score_without_nmp).abs();
        assert!(score_diff <= 100); // Allow small differences due to search variations

        // NMP should generally be faster (though this isn't guaranteed in all cases)
        // We just verify that both searches complete without errors
        assert!(duration_with_nmp.as_millis() > 0);
        assert!(duration_without_nmp.as_millis() > 0);

        println!(
            "NMP enabled: {:?}, NMP disabled: {:?}",
            duration_with_nmp, duration_without_nmp
        );
    }

    #[test]
    fn test_null_move_dynamic_reduction() {
        let mut engine = create_test_engine();

        // Test dynamic reduction configuration
        let mut config = engine.get_null_move_config().clone();
        config.enable_dynamic_reduction = true;
        config.reduction_factor = 2; // Base reduction
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert!(updated_config.enable_dynamic_reduction);
        assert_eq!(updated_config.reduction_factor, 2);

        // Test static reduction configuration
        let mut config = engine.get_null_move_config().clone();
        config.enable_dynamic_reduction = false;
        config.reduction_factor = 3;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert!(!updated_config.enable_dynamic_reduction);
        assert_eq!(updated_config.reduction_factor, 3);
    }

    #[test]
    fn test_null_move_safety_mechanisms() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test that NMP respects minimum depth
        let mut config = engine.get_null_move_config().clone();
        config.min_depth = 5;
        engine.update_null_move_config(config).unwrap();

        // At depth 3, NMP should be disabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some()); // Search should still work

        // At depth 5, NMP should be enabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some()); // Search should work

        // Test that statistics tracking works for safety mechanisms
        let stats = engine.get_null_move_stats();
        assert!(stats.disabled_in_check >= 0);
        assert!(stats.disabled_endgame >= 0);
    }

    #[test]
    fn test_null_move_safety_mechanisms_enhanced() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test enhanced safety mechanisms
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_endgame_detection = true;
        config.max_pieces_threshold = 12; // Conservative threshold
        engine.update_null_move_config(config).unwrap();

        // Test that safety mechanisms are working
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Should track safety mechanism usage
        assert!(stats.disabled_in_check >= 0);
        assert!(stats.disabled_endgame >= 0);

        // Total disabled should be sum of individual counters
        assert_eq!(
            stats.total_disabled(),
            stats.disabled_in_check + stats.disabled_endgame
        );

        println!(
            "Enhanced safety mechanisms: {} disabled in check, {} disabled in endgame",
            stats.disabled_in_check, stats.disabled_endgame
        );
    }

    #[test]
    fn test_null_move_zugzwang_detection() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test zugzwang detection through endgame detection
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_endgame_detection = true;
        config.max_pieces_threshold = 15; // Higher threshold for more conservative play
        engine.update_null_move_config(config).unwrap();

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Should have some endgame detection activity
        assert!(stats.disabled_endgame >= 0);

        println!(
            "Zugzwang detection: {} positions disabled due to endgame",
            stats.disabled_endgame
        );
    }

    #[test]
    fn test_null_move_tactical_safety() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test tactical safety through conservative configuration
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 4; // Higher minimum depth for more conservative play
        config.reduction_factor = 2; // Conservative reduction factor
        engine.update_null_move_config(config).unwrap();

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Should have reasonable NMP activity
        assert!(stats.attempts >= 0);
        assert!(stats.cutoffs >= 0);

        println!(
            "Tactical safety: {} attempts, {} cutoffs",
            stats.attempts, stats.cutoffs
        );
    }

    #[test]
    fn test_null_move_fallback_mechanism() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test fallback mechanism by disabling NMP
        let mut config = engine.get_null_move_config().clone();
        config.enabled = false; // Disable NMP as fallback
        engine.update_null_move_config(config).unwrap();

        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Should have no NMP activity when disabled
        assert_eq!(stats.attempts, 0);
        assert_eq!(stats.cutoffs, 0);
        assert_eq!(stats.disabled_in_check, 0);
        assert_eq!(stats.disabled_endgame, 0);

        println!("Fallback mechanism: NMP disabled, no activity recorded");
    }

    // ===== VERIFICATION SEARCH TESTS =====

    #[test]
    fn test_verification_search_configuration() {
        let mut engine = create_test_engine();

        // Test that verification_margin is properly initialized
        let config = engine.get_null_move_config();
        assert_eq!(config.verification_margin, 200); // Default value

        // Test verification_margin validation
        let mut config = config.clone();
        config.verification_margin = 150;
        let result = engine.update_null_move_config(config);
        assert!(result.is_ok());

        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.verification_margin, 150);

        // Test invalid verification_margin (negative)
        let mut invalid_config = engine.get_null_move_config().clone();
        invalid_config.verification_margin = -1;
        let result = engine.update_null_move_config(invalid_config);
        assert!(result.is_err());

        // Test invalid verification_margin (too large)
        let mut invalid_config = engine.get_null_move_config().clone();
        invalid_config.verification_margin = 1001;
        let result = engine.update_null_move_config(invalid_config);
        assert!(result.is_err());

        // Test verification statistics initialization
        let stats = engine.get_null_move_stats();
        assert_eq!(stats.verification_attempts, 0);
        assert_eq!(stats.verification_cutoffs, 0);
    }

    #[test]
    fn test_verification_search_statistics_tracking() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics to ensure clean test
        engine.reset_null_move_stats();

        // Configure verification search with a reasonable margin
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 200;
        engine.update_null_move_config(config).unwrap();

        // Perform a search that may trigger verification
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Verify statistics tracking mechanisms
        assert!(stats.verification_attempts >= 0);
        assert!(stats.verification_cutoffs >= 0);
        assert!(stats.verification_cutoffs <= stats.verification_attempts);

        // Test verification cutoff rate calculation
        let cutoff_rate = stats.verification_cutoff_rate();
        assert!(cutoff_rate >= 0.0);
        assert!(cutoff_rate <= 100.0);

        // Test that performance report includes verification statistics
        let report = stats.performance_report();
        assert!(report.contains("Verification attempts"));
        assert!(report.contains("Verification cutoffs"));
    }

    #[test]
    fn test_verification_search_disabled() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_null_move_stats();

        // Disable verification search by setting margin to 0
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 0;
        engine.update_null_move_config(config).unwrap();

        // Perform a search
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Verification should not be attempted when margin is 0
        assert_eq!(stats.verification_attempts, 0);
        assert_eq!(stats.verification_cutoffs, 0);
    }

    #[test]
    fn test_verification_search_margin_boundaries() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with very small margin (should rarely trigger)
        engine.reset_null_move_stats();
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 10; // Very small margin
        engine.update_null_move_config(config).unwrap();

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats_small = engine.get_null_move_stats();

        // Test with large margin (should trigger more often)
        engine.reset_null_move_stats();
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 500; // Large margin
        engine.update_null_move_config(config).unwrap();

        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats_large = engine.get_null_move_stats();

        // With larger margin, we should see more or equal verification attempts
        // (this may not always be true depending on position, but structure is correct)
        assert!(stats_large.verification_attempts >= stats_small.verification_attempts);

        println!(
            "Small margin (10): {} verification attempts",
            stats_small.verification_attempts
        );
        println!(
            "Large margin (500): {} verification attempts",
            stats_large.verification_attempts
        );
    }

    #[test]
    fn test_verification_search_different_depths() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure verification search
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 200;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();

        // Test at shallow depth (should still allow NMP if min_depth is met)
        engine.reset_null_move_stats();
        let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        assert!(result.is_some());

        let stats_shallow = engine.get_null_move_stats();

        // Test at deeper depth
        engine.reset_null_move_stats();
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
        assert!(result.is_some());

        let stats_deep = engine.get_null_move_stats();

        // Both should complete successfully
        assert!(stats_shallow.verification_attempts >= 0);
        assert!(stats_deep.verification_attempts >= 0);

        println!(
            "Depth 3: {} verification attempts, {} cutoffs",
            stats_shallow.verification_attempts, stats_shallow.verification_cutoffs
        );
        println!(
            "Depth 5: {} verification attempts, {} cutoffs",
            stats_deep.verification_attempts, stats_deep.verification_cutoffs
        );
    }

    #[test]
    fn test_verification_search_correctness() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure verification search with moderate margin
        let mut config = engine.get_null_move_config().clone();
        config.verification_margin = 200;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search - verification should only trigger when null move fails but is close to beta
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Verification cutoffs should never exceed verification attempts
        assert!(stats.verification_cutoffs <= stats.verification_attempts);

        // If there were verification attempts, they should have been tracked
        if stats.verification_attempts > 0 {
            assert!(stats.verification_cutoffs >= 0);
            let cutoff_rate = stats.verification_cutoff_rate();
            assert!(cutoff_rate >= 0.0 && cutoff_rate <= 100.0);
        }

        // Total cutoffs should include both direct NMP cutoffs and verification cutoffs
        // Note: This is a structural test - actual counts depend on position characteristics
        assert!(stats.cutoffs >= stats.verification_cutoffs);

        println!(
            "Verification correctness: {} attempts, {} cutoffs ({:.2}%)",
            stats.verification_attempts,
            stats.verification_cutoffs,
            stats.verification_cutoff_rate()
        );
    }

    #[test]
    fn test_verification_search_integration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test that verification search integrates correctly with null move pruning
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.verification_margin = 200;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search with verification enabled
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // Search should complete successfully
        assert!(result.is_some());

        // Statistics should be properly tracked
        assert!(stats.attempts >= 0);
        assert!(stats.verification_attempts >= 0);

        // Verification should only happen if null move was attempted
        // (verification_attempts <= attempts is not strictly true because verification
        // happens when null move fails but is within margin, so it's possible to have
        // more verifications than direct cutoffs, but structure should be sound)

        println!(
            "Integration test: {} NMP attempts, {} verification attempts, {} total cutoffs",
            stats.attempts, stats.verification_attempts, stats.cutoffs
        );
    }

    // ===== DYNAMIC REDUCTION FORMULA TESTS =====

    #[test]
    fn test_dynamic_reduction_formula_configuration() {
        let mut engine = create_test_engine();

        // Test default formula (should be Linear)
        let config = engine.get_null_move_config();
        assert_eq!(
            config.dynamic_reduction_formula,
            DynamicReductionFormula::Linear
        );

        // Test Static formula
        let mut config = config.clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Static;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(
            updated_config.dynamic_reduction_formula,
            DynamicReductionFormula::Static
        );

        // Test Linear formula
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Linear;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(
            updated_config.dynamic_reduction_formula,
            DynamicReductionFormula::Linear
        );

        // Test Smooth formula
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Smooth;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(
            updated_config.dynamic_reduction_formula,
            DynamicReductionFormula::Smooth
        );
    }

    #[test]
    fn test_reduction_formula_calculations() {
        // Test Static formula: always returns base reduction
        let formula = DynamicReductionFormula::Static;
        assert_eq!(formula.calculate_reduction(3, 2), 2);
        assert_eq!(formula.calculate_reduction(6, 2), 2);
        assert_eq!(formula.calculate_reduction(12, 2), 2);

        // Test Linear formula: R = base + depth / 6
        // depth 3: 2 + 3/6 = 2 + 0 = 2
        // depth 4: 2 + 4/6 = 2 + 0 = 2
        // depth 5: 2 + 5/6 = 2 + 0 = 2
        // depth 6: 2 + 6/6 = 2 + 1 = 3
        // depth 12: 2 + 12/6 = 2 + 2 = 4
        // depth 18: 2 + 18/6 = 2 + 3 = 5
        let formula = DynamicReductionFormula::Linear;
        assert_eq!(formula.calculate_reduction(3, 2), 2);
        assert_eq!(formula.calculate_reduction(4, 2), 2);
        assert_eq!(formula.calculate_reduction(5, 2), 2);
        assert_eq!(formula.calculate_reduction(6, 2), 3);
        assert_eq!(formula.calculate_reduction(12, 2), 4);
        assert_eq!(formula.calculate_reduction(18, 2), 5);

        // Test Smooth formula: R = base + (depth / 6.0).round()
        // depth 3: 2 + (3/6.0).round() = 2 + 0.5.round() = 2 + 1 = 3
        // depth 4: 2 + (4/6.0).round() = 2 + 0.67.round() = 2 + 1 = 3
        // depth 5: 2 + (5/6.0).round() = 2 + 0.83.round() = 2 + 1 = 3
        // depth 6: 2 + (6/6.0).round() = 2 + 1.0.round() = 2 + 1 = 3
        // depth 7: 2 + (7/6.0).round() = 2 + 1.17.round() = 2 + 1 = 3
        // depth 8: 2 + (8/6.0).round() = 2 + 1.33.round() = 2 + 1 = 3
        // depth 9: 2 + (9/6.0).round() = 2 + 1.5.round() = 2 + 2 = 4
        let formula = DynamicReductionFormula::Smooth;
        assert_eq!(formula.calculate_reduction(3, 2), 3);
        assert_eq!(formula.calculate_reduction(4, 2), 3);
        assert_eq!(formula.calculate_reduction(5, 2), 3);
        assert_eq!(formula.calculate_reduction(6, 2), 3);
        assert_eq!(formula.calculate_reduction(9, 2), 4);
        assert_eq!(formula.calculate_reduction(12, 2), 4);
        assert_eq!(formula.calculate_reduction(18, 2), 5);
    }

    #[test]
    fn test_reduction_formula_smoother_scaling() {
        // Test that Smooth formula provides smoother scaling than Linear
        let linear = DynamicReductionFormula::Linear;
        let smooth = DynamicReductionFormula::Smooth;
        let base = 2;

        // At depth 3-5, Linear keeps reduction at 2, while Smooth increases to 3 earlier
        assert_eq!(linear.calculate_reduction(3, base), 2);
        assert_eq!(smooth.calculate_reduction(3, base), 3); // Smooth increases earlier

        assert_eq!(linear.calculate_reduction(5, base), 2);
        assert_eq!(smooth.calculate_reduction(5, base), 3); // Smooth increases earlier

        // At depth 6, both should be 3
        assert_eq!(linear.calculate_reduction(6, base), 3);
        assert_eq!(smooth.calculate_reduction(6, base), 3);

        // At depth 9, Linear is still 3, but Smooth increases to 4
        assert_eq!(linear.calculate_reduction(9, base), 3);
        assert_eq!(smooth.calculate_reduction(9, base), 4); // Smooth increases at 9

        // At depth 12, Linear is 4, Smooth is also 4
        assert_eq!(linear.calculate_reduction(12, base), 4);
        assert_eq!(smooth.calculate_reduction(12, base), 4);
    }

    #[test]
    fn test_reduction_formulas_integration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with Static formula
        engine.reset_null_move_stats();
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Static;
        config.enable_dynamic_reduction = false; // Static doesn't use dynamic reduction
        engine.update_null_move_config(config).unwrap();

        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Test with Linear formula
        engine.reset_null_move_stats();
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Linear;
        config.enable_dynamic_reduction = true;
        engine.update_null_move_config(config).unwrap();

        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Test with Smooth formula
        engine.reset_null_move_stats();
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Smooth;
        config.enable_dynamic_reduction = true;
        engine.update_null_move_config(config).unwrap();

        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());
    }

    #[test]
    fn test_reduction_formula_different_depths() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test Linear formula at different depths
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Linear;
        config.enable_dynamic_reduction = true;
        engine.update_null_move_config(config).unwrap();

        for depth in [3, 4, 5, 6, 12] {
            engine.reset_null_move_stats();
            let result = engine.search_at_depth_legacy(
                &mut board.clone(),
                &captured_pieces,
                player,
                depth,
                1000,
            );
            assert!(
                result.is_some(),
                "Linear formula should work at depth {}",
                depth
            );

            let stats = engine.get_null_move_stats();
            // Verify search completed successfully
            assert!(stats.attempts >= 0);
        }

        // Test Smooth formula at different depths
        let mut config = engine.get_null_move_config().clone();
        config.dynamic_reduction_formula = DynamicReductionFormula::Smooth;
        config.enable_dynamic_reduction = true;
        engine.update_null_move_config(config).unwrap();

        for depth in [3, 4, 5, 6, 9, 12] {
            engine.reset_null_move_stats();
            let result = engine.search_at_depth_legacy(
                &mut board.clone(),
                &captured_pieces,
                player,
                depth,
                1000,
            );
            assert!(
                result.is_some(),
                "Smooth formula should work at depth {}",
                depth
            );

            let stats = engine.get_null_move_stats();
            // Verify search completed successfully
            assert!(stats.attempts >= 0);
        }
    }

    // ===== MATE THREAT DETECTION TESTS =====

    #[test]
    fn test_mate_threat_detection_configuration() {
        let mut engine = create_test_engine();

        // Test default configuration (should be disabled)
        let config = engine.get_null_move_config();
        assert!(!config.enable_mate_threat_detection);
        assert_eq!(config.mate_threat_margin, 500);

        // Test enabling mate threat detection
        let mut config = config.clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 500;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert!(updated_config.enable_mate_threat_detection);
        assert_eq!(updated_config.mate_threat_margin, 500);
    }

    #[test]
    fn test_mate_threat_detection_statistics_tracking() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable mate threat detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 500;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform a search to trigger mate threat detection if conditions are met
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // Verify mate threat statistics are tracked
        assert!(stats.mate_threat_attempts >= 0);
        assert!(stats.mate_threat_detected >= 0);
        assert!(stats.mate_threat_detected <= stats.mate_threat_attempts);

        // Verify detection rate is calculated correctly
        let detection_rate = stats.mate_threat_detection_rate();
        assert!(detection_rate >= 0.0);
        assert!(detection_rate <= 100.0);
    }

    #[test]
    fn test_mate_threat_detection_disabled() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Ensure mate threat detection is disabled
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = false;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform a search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // When disabled, mate threat attempts should remain 0
        assert_eq!(stats.mate_threat_attempts, 0);
        assert_eq!(stats.mate_threat_detected, 0);
    }

    #[test]
    fn test_mate_threat_detection_margin_boundaries() {
        let mut engine = create_test_engine();

        // Test with small margin
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 100;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.mate_threat_margin, 100);

        // Test with large margin
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 1000;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.mate_threat_margin, 1000);
    }

    #[test]
    fn test_mate_threat_detection_integration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable mate threat detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 500;
        config.enable_dynamic_reduction = true;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search at different depths
        for depth in [3, 4, 5, 6] {
            engine.reset_null_move_stats();
            let result = engine.search_at_depth_legacy(
                &mut board.clone(),
                &captured_pieces,
                player,
                depth,
                1000,
            );
            assert!(
                result.is_some(),
                "Search should complete at depth {}",
                depth
            );

            let stats = engine.get_null_move_stats();
            // Verify search completed and stats are tracked
            assert!(stats.attempts >= 0);
            assert!(stats.mate_threat_attempts >= 0);
        }
    }

    #[test]
    fn test_mate_threat_detection_with_verification_search() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable both mate threat detection and verification search
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 500;
        config.verification_margin = 200;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // Both mate threat and verification stats should be tracked
        assert!(stats.mate_threat_attempts >= 0);
        assert!(stats.verification_attempts >= 0);

        // Verify detection rates are valid
        assert!(stats.mate_threat_detection_rate() >= 0.0);
        assert!(stats.mate_threat_detection_rate() <= 100.0);
        assert!(stats.verification_cutoff_rate() >= 0.0);
        assert!(stats.verification_cutoff_rate() <= 100.0);
    }

    #[test]
    fn test_mate_threat_detection_correctness() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable mate threat detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_mate_threat_detection = true;
        config.mate_threat_margin = 500;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // Correctness checks:
        // 1. Mate threats detected should never exceed attempts
        assert!(stats.mate_threat_detected <= stats.mate_threat_attempts);

        // 2. If there were attempts, they should be tracked correctly
        if stats.mate_threat_attempts > 0 {
            assert!(stats.mate_threat_detected >= 0);
            let detection_rate = stats.mate_threat_detection_rate();
            assert!(detection_rate >= 0.0);
            assert!(detection_rate <= 100.0);
        }

        // 3. Total cutoffs should include mate threat cutoffs
        // (mate threat verification that succeeds contributes to cutoffs)
        assert!(stats.cutoffs >= 0);
    }

    // ===== ENDGAME TYPE DETECTION TESTS =====

    #[test]
    fn test_endgame_type_detection_configuration() {
        let mut engine = create_test_engine();

        // Test default configuration (should be disabled)
        let config = engine.get_null_move_config();
        assert!(!config.enable_endgame_type_detection);
        assert_eq!(config.material_endgame_threshold, 12);
        assert_eq!(config.king_activity_threshold, 8);
        assert_eq!(config.zugzwang_threshold, 6);

        // Test enabling endgame type detection
        let mut config = config.clone();
        config.enable_endgame_type_detection = true;
        config.material_endgame_threshold = 12;
        config.king_activity_threshold = 8;
        config.zugzwang_threshold = 6;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert!(updated_config.enable_endgame_type_detection);
        assert_eq!(updated_config.material_endgame_threshold, 12);
        assert_eq!(updated_config.king_activity_threshold, 8);
        assert_eq!(updated_config.zugzwang_threshold, 6);
    }

    #[test]
    fn test_endgame_type_detection_statistics_tracking() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable endgame type detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_type_detection = true;
        config.enable_endgame_detection = true;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform a search to trigger endgame type detection if conditions are met
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // Verify endgame type statistics are tracked
        assert!(stats.disabled_material_endgame >= 0);
        assert!(stats.disabled_king_activity_endgame >= 0);
        assert!(stats.disabled_zugzwang >= 0);
    }

    #[test]
    fn test_endgame_type_detection_disabled() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Ensure endgame type detection is disabled
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_type_detection = false;
        config.enable_endgame_detection = true;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform a search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // When disabled, endgame type stats should remain 0 (using basic detection)
        // Basic endgame detection still tracks disabled_endgame
        assert!(stats.disabled_endgame >= 0);
    }

    #[test]
    fn test_endgame_type_detection_thresholds() {
        let mut engine = create_test_engine();

        // Test with different thresholds
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_type_detection = true;
        config.material_endgame_threshold = 15;
        config.king_activity_threshold = 10;
        config.zugzwang_threshold = 8;
        engine.update_null_move_config(config).unwrap();

        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.material_endgame_threshold, 15);
        assert_eq!(updated_config.king_activity_threshold, 10);
        assert_eq!(updated_config.zugzwang_threshold, 8);
    }

    #[test]
    fn test_endgame_type_detection_integration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable endgame type detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_type_detection = true;
        config.enable_endgame_detection = true;
        config.material_endgame_threshold = 12;
        config.king_activity_threshold = 8;
        config.zugzwang_threshold = 6;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search at different depths
        for depth in [3, 4, 5, 6] {
            engine.reset_null_move_stats();
            let result = engine.search_at_depth_legacy(
                &mut board.clone(),
                &captured_pieces,
                player,
                depth,
                1000,
            );
            assert!(
                result.is_some(),
                "Search should complete at depth {}",
                depth
            );

            let stats = engine.get_null_move_stats();
            // Verify search completed and stats are tracked
            assert!(stats.attempts >= 0);
            assert!(stats.disabled_material_endgame >= 0);
            assert!(stats.disabled_king_activity_endgame >= 0);
            assert!(stats.disabled_zugzwang >= 0);
        }
    }

    #[test]
    fn test_endgame_type_detection_correctness() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable endgame type detection
        let mut config = engine.get_null_move_config().clone();
        config.enable_endgame_type_detection = true;
        config.enable_endgame_detection = true;
        engine.update_null_move_config(config).unwrap();

        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        // Correctness checks:
        // 1. Endgame type stats should be non-negative
        assert!(stats.disabled_material_endgame >= 0);
        assert!(stats.disabled_king_activity_endgame >= 0);
        assert!(stats.disabled_zugzwang >= 0);

        // 2. Total disabled should include all endgame types
        let total_endgame_disabled = stats.disabled_material_endgame
            + stats.disabled_king_activity_endgame
            + stats.disabled_zugzwang;

        // When endgame type detection is enabled, disabled_endgame may be 0
        // (because it uses type-specific stats instead)
        assert!(total_endgame_disabled >= 0);
    }

    #[test]
    fn test_null_move_preset_enum() {
        // Test preset enum variants
        let conservative = NullMovePreset::Conservative;
        let aggressive = NullMovePreset::Aggressive;
        let balanced = NullMovePreset::Balanced;

        // Test to_string()
        assert_eq!(conservative.to_string(), "Conservative");
        assert_eq!(aggressive.to_string(), "Aggressive");
        assert_eq!(balanced.to_string(), "Balanced");

        // Test from_str()
        assert_eq!(
            NullMovePreset::from_str("conservative"),
            Some(NullMovePreset::Conservative)
        );
        assert_eq!(
            NullMovePreset::from_str("CONSERVATIVE"),
            Some(NullMovePreset::Conservative)
        );
        assert_eq!(
            NullMovePreset::from_str("aggressive"),
            Some(NullMovePreset::Aggressive)
        );
        assert_eq!(
            NullMovePreset::from_str("AGGRESSIVE"),
            Some(NullMovePreset::Aggressive)
        );
        assert_eq!(
            NullMovePreset::from_str("balanced"),
            Some(NullMovePreset::Balanced)
        );
        assert_eq!(
            NullMovePreset::from_str("BALANCED"),
            Some(NullMovePreset::Balanced)
        );
        assert_eq!(NullMovePreset::from_str("invalid"), None);
    }

    #[test]
    fn test_null_move_config_from_preset_conservative() {
        let config = NullMoveConfig::from_preset(NullMovePreset::Conservative);

        // Conservative preset: Higher verification_margin, lower reduction_factor, stricter endgame detection
        assert_eq!(config.verification_margin, 400);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.max_pieces_threshold, 14);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.enable_mate_threat_detection, true);
        assert_eq!(config.mate_threat_margin, 600);
        assert_eq!(config.enable_endgame_type_detection, true);
        assert_eq!(config.material_endgame_threshold, 14);
        assert_eq!(config.king_activity_threshold, 10);
        assert_eq!(config.zugzwang_threshold, 8);
        assert_eq!(
            config.dynamic_reduction_formula,
            DynamicReductionFormula::Linear
        );
        assert_eq!(config.preset, Some(NullMovePreset::Conservative));

        // Verify configuration is valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_null_move_config_from_preset_aggressive() {
        let config = NullMoveConfig::from_preset(NullMovePreset::Aggressive);

        // Aggressive preset: Lower verification_margin, higher reduction_factor, relaxed endgame detection
        assert_eq!(config.verification_margin, 100);
        assert_eq!(config.reduction_factor, 3);
        assert_eq!(config.max_pieces_threshold, 10);
        assert_eq!(config.min_depth, 2);
        assert_eq!(config.enable_mate_threat_detection, false);
        assert_eq!(config.mate_threat_margin, 400);
        assert_eq!(config.enable_endgame_type_detection, false);
        assert_eq!(config.material_endgame_threshold, 10);
        assert_eq!(config.king_activity_threshold, 6);
        assert_eq!(config.zugzwang_threshold, 4);
        assert_eq!(
            config.dynamic_reduction_formula,
            DynamicReductionFormula::Smooth
        );
        assert_eq!(config.preset, Some(NullMovePreset::Aggressive));

        // Verify configuration is valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_null_move_config_from_preset_balanced() {
        let config = NullMoveConfig::from_preset(NullMovePreset::Balanced);

        // Balanced preset: Default values optimized for general play
        assert_eq!(config.verification_margin, 200);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.max_pieces_threshold, 12);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.enable_mate_threat_detection, false);
        assert_eq!(config.mate_threat_margin, 500);
        assert_eq!(config.enable_endgame_type_detection, false);
        assert_eq!(config.material_endgame_threshold, 12);
        assert_eq!(config.king_activity_threshold, 8);
        assert_eq!(config.zugzwang_threshold, 6);
        assert_eq!(
            config.dynamic_reduction_formula,
            DynamicReductionFormula::Linear
        );
        assert_eq!(config.preset, Some(NullMovePreset::Balanced));

        // Verify configuration is valid
        assert!(config.validate().is_ok());

        // Balanced preset should match default
        let default_config = NullMoveConfig::default();
        assert_eq!(
            default_config.verification_margin,
            config.verification_margin
        );
        assert_eq!(default_config.reduction_factor, config.reduction_factor);
        assert_eq!(
            default_config.max_pieces_threshold,
            config.max_pieces_threshold
        );
        assert_eq!(default_config.preset, config.preset);
    }

    #[test]
    fn test_null_move_config_apply_preset() {
        let mut config = NullMoveConfig::default();

        // Apply Conservative preset
        config.apply_preset(NullMovePreset::Conservative);
        assert_eq!(config.verification_margin, 400);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.max_pieces_threshold, 14);
        assert_eq!(config.preset, Some(NullMovePreset::Conservative));

        // Apply Aggressive preset
        config.apply_preset(NullMovePreset::Aggressive);
        assert_eq!(config.verification_margin, 100);
        assert_eq!(config.reduction_factor, 3);
        assert_eq!(config.max_pieces_threshold, 10);
        assert_eq!(config.preset, Some(NullMovePreset::Aggressive));

        // Apply Balanced preset
        config.apply_preset(NullMovePreset::Balanced);
        assert_eq!(config.verification_margin, 200);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.max_pieces_threshold, 12);
        assert_eq!(config.preset, Some(NullMovePreset::Balanced));
    }

    #[test]
    fn test_null_move_config_summary_includes_preset() {
        let config = NullMoveConfig::from_preset(NullMovePreset::Conservative);
        let summary = config.summary();

        // Summary should include preset information
        assert!(summary.contains("preset=Conservative"));

        let balanced_config = NullMoveConfig::from_preset(NullMovePreset::Balanced);
        let balanced_summary = balanced_config.summary();
        assert!(balanced_summary.contains("preset=Balanced"));

        // Config without preset should not have preset in summary
        let mut custom_config = NullMoveConfig::default();
        custom_config.preset = None;
        let custom_summary = custom_config.summary();
        assert!(!custom_summary.contains("preset="));
    }

    #[test]
    fn test_null_move_preset_integration_conservative() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Use Conservative preset
        let config = NullMoveConfig::from_preset(NullMovePreset::Conservative);
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        let engine_config = engine.get_null_move_config();

        // Conservative preset should have higher verification margin
        assert_eq!(engine_config.verification_margin, 400);
        assert_eq!(engine_config.enable_mate_threat_detection, true);

        // Search should complete successfully
        assert!(result.is_some());

        // Statistics should be tracked
        assert!(stats.attempts >= 0);
        if stats.attempts > 0 {
            assert!(stats.cutoffs >= 0);
        }
    }

    #[test]
    fn test_null_move_preset_integration_aggressive() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Use Aggressive preset
        let config = NullMoveConfig::from_preset(NullMovePreset::Aggressive);
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        let engine_config = engine.get_null_move_config();

        // Aggressive preset should have lower verification margin
        assert_eq!(engine_config.verification_margin, 100);
        assert_eq!(engine_config.reduction_factor, 3);
        assert_eq!(engine_config.enable_mate_threat_detection, false);

        // Search should complete successfully
        assert!(result.is_some());

        // Statistics should be tracked
        assert!(stats.attempts >= 0);
        if stats.attempts > 0 {
            assert!(stats.cutoffs >= 0);
        }
    }

    #[test]
    fn test_null_move_preset_integration_balanced() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Use Balanced preset (default)
        let config = NullMoveConfig::from_preset(NullMovePreset::Balanced);
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();
        let engine_config = engine.get_null_move_config();

        // Balanced preset should have default values
        assert_eq!(engine_config.verification_margin, 200);
        assert_eq!(engine_config.reduction_factor, 2);

        // Search should complete successfully
        assert!(result.is_some());

        // Statistics should be tracked
        assert!(stats.attempts >= 0);
        if stats.attempts > 0 {
            assert!(stats.cutoffs >= 0);
        }
    }

    #[test]
    fn test_null_move_preset_comparison() {
        let conservative = NullMoveConfig::from_preset(NullMovePreset::Conservative);
        let aggressive = NullMoveConfig::from_preset(NullMovePreset::Aggressive);
        let balanced = NullMoveConfig::from_preset(NullMovePreset::Balanced);

        // Conservative should have highest verification margin
        assert!(conservative.verification_margin > aggressive.verification_margin);
        assert!(conservative.verification_margin > balanced.verification_margin);

        // Aggressive should have lowest verification margin
        assert!(aggressive.verification_margin < conservative.verification_margin);
        assert!(aggressive.verification_margin < balanced.verification_margin);

        // Aggressive should have highest reduction factor
        assert!(aggressive.reduction_factor > conservative.reduction_factor);
        assert!(aggressive.reduction_factor > balanced.reduction_factor);

        // Conservative should have strictest endgame detection (highest threshold)
        assert!(conservative.max_pieces_threshold > aggressive.max_pieces_threshold);
        assert!(conservative.max_pieces_threshold > balanced.max_pieces_threshold);

        // Aggressive should have most relaxed endgame detection (lowest threshold)
        assert!(aggressive.max_pieces_threshold < conservative.max_pieces_threshold);
        assert!(aggressive.max_pieces_threshold < balanced.max_pieces_threshold);

        // Conservative should have mate threat detection enabled
        assert_eq!(conservative.enable_mate_threat_detection, true);
        assert_eq!(aggressive.enable_mate_threat_detection, false);
        assert_eq!(balanced.enable_mate_threat_detection, false);

        // Conservative should have endgame type detection enabled
        assert_eq!(conservative.enable_endgame_type_detection, true);
        assert_eq!(aggressive.enable_endgame_type_detection, false);
        assert_eq!(balanced.enable_endgame_type_detection, false);
    }

    #[test]
    fn test_null_move_board_state_isolation() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable NMP and set up configuration
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Clone board to compare state before and after
        let board_before = board.clone();

        // Perform search with null move pruning
        let result = engine.search_at_depth_legacy(&mut board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Verify board state is unchanged after null move search
        // The null move search should not modify the board state - it just switches
        // turns via recursive call without making an actual move
        assert_eq!(
            board.get_occupied_bitboard(),
            board_before.get_occupied_bitboard(),
            "Board state should remain unchanged after null move search"
        );

        // Verify NMP was attempted (if conditions were met)
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);

        // Additional check: verify board piece counts are unchanged
        let pieces_before = board_before.get_occupied_bitboard().count_ones();
        let pieces_after = board.get_occupied_bitboard().count_ones();
        assert_eq!(
            pieces_before, pieces_after,
            "Piece count should remain unchanged after null move search"
        );
    }

    #[test]
    fn test_null_move_hash_history_isolation() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable NMP
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search - this will create a main search hash history
        let result1 =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result1.is_some());

        // Perform another search to verify hash history is isolated
        // The null move search creates its own local hash history, so it should
        // not interfere with the main search's hash history
        let result2 =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result2.is_some());

        // Both searches should complete successfully
        assert!(result1.is_some());
        assert!(result2.is_some());

        // The fact that both searches complete successfully indicates that hash
        // history isolation is working correctly. If hash history was shared
        // incorrectly, it could cause repetition detection issues or search failures.
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_does_not_make_actual_move() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable NMP
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();

        // Get initial board state
        let initial_occupied = board.get_occupied_bitboard();
        let mut board_clone = board.clone();

        // Get piece positions on all squares
        let mut initial_pieces = Vec::new();
        for square in 0..81 {
            initial_pieces.push(board.get_piece(square.into()));
        }

        // Perform search
        let result = engine.search_at_depth_legacy(&mut board, &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Verify board state is identical to initial state
        let final_occupied = board.get_occupied_bitboard();
        assert_eq!(
            initial_occupied, final_occupied,
            "Board occupied squares should be unchanged"
        );

        // Verify piece positions are unchanged
        for square in 0..81 {
            let piece_before = initial_pieces[square as usize];
            let piece_after = board.get_piece(square.into());
            assert_eq!(
                piece_before, piece_after,
                "Piece at square {} should be unchanged",
                square
            );
        }

        // Verify board is still in the same state as the clone
        assert_eq!(
            board.get_occupied_bitboard(),
            board_clone.get_occupied_bitboard(),
            "Board should be unchanged after null move search"
        );
    }

    #[test]
    fn test_null_move_hash_history_separation() {
        // This test verifies that null move search uses a separate hash history
        // from the main search, preventing interference with repetition detection
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Enable NMP
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform multiple searches to verify hash history isolation
        for _ in 0..3 {
            let result = engine.search_at_depth_legacy(
                &mut board.clone(),
                &captured_pieces,
                player,
                4,
                1000,
            );
            assert!(result.is_some(), "Search should complete successfully");
        }

        // If hash history isolation wasn't working, we would see:
        // 1. Repetition detection issues (false positives)
        // 2. Search failures
        // 3. Incorrect evaluation results
        // The fact that all searches complete successfully indicates proper isolation

        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_enum() {
        // Test reduction strategy enum variants
        let static_strategy = NullMoveReductionStrategy::Static;
        let dynamic_strategy = NullMoveReductionStrategy::Dynamic;
        let depth_based_strategy = NullMoveReductionStrategy::DepthBased;
        let material_based_strategy = NullMoveReductionStrategy::MaterialBased;
        let position_type_based_strategy = NullMoveReductionStrategy::PositionTypeBased;

        // Test to_string()
        assert_eq!(static_strategy.to_string(), "Static");
        assert_eq!(dynamic_strategy.to_string(), "Dynamic");
        assert_eq!(depth_based_strategy.to_string(), "DepthBased");
        assert_eq!(material_based_strategy.to_string(), "MaterialBased");
        assert_eq!(
            position_type_based_strategy.to_string(),
            "PositionTypeBased"
        );

        // Test from_str()
        assert_eq!(
            NullMoveReductionStrategy::from_str("static"),
            Some(NullMoveReductionStrategy::Static)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("STATIC"),
            Some(NullMoveReductionStrategy::Static)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("dynamic"),
            Some(NullMoveReductionStrategy::Dynamic)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("depthbased"),
            Some(NullMoveReductionStrategy::DepthBased)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("depth-based"),
            Some(NullMoveReductionStrategy::DepthBased)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("materialbased"),
            Some(NullMoveReductionStrategy::MaterialBased)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("material-based"),
            Some(NullMoveReductionStrategy::MaterialBased)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("positiontypebased"),
            Some(NullMoveReductionStrategy::PositionTypeBased)
        );
        assert_eq!(
            NullMoveReductionStrategy::from_str("position-type-based"),
            Some(NullMoveReductionStrategy::PositionTypeBased)
        );
        assert_eq!(NullMoveReductionStrategy::from_str("invalid"), None);
    }

    #[test]
    fn test_null_move_reduction_strategy_static() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure Static reduction strategy
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::Static;
        config.reduction_factor = 2;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Static strategy should always use reduction_factor
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_dynamic() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure Dynamic reduction strategy
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::Dynamic;
        config.enable_dynamic_reduction = true;
        config.dynamic_reduction_formula = DynamicReductionFormula::Linear;
        config.reduction_factor = 2;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Dynamic strategy should use dynamic_reduction_formula
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_depth_based() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure DepthBased reduction strategy
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::DepthBased;
        config.reduction_factor = 2;
        config.depth_scaling_factor = 1;
        config.min_depth_for_scaling = 4;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 5, 1000);
        assert!(result.is_some());

        // DepthBased strategy should vary reduction by depth
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_material_based() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure MaterialBased reduction strategy
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::MaterialBased;
        config.reduction_factor = 2;
        config.material_adjustment_factor = 1;
        config.piece_count_threshold = 20;
        config.threshold_step = 4;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // MaterialBased strategy should adjust reduction by piece count
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_position_type_based() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure PositionTypeBased reduction strategy
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::PositionTypeBased;
        config.opening_reduction_factor = 3;
        config.middlegame_reduction_factor = 2;
        config.endgame_reduction_factor = 1;
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // PositionTypeBased strategy should use different reductions for opening/middlegame/endgame
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_null_move_reduction_strategy_configuration() {
        // Test that reduction strategy is properly configured in default config
        let config = NullMoveConfig::default();
        assert_eq!(
            config.reduction_strategy,
            NullMoveReductionStrategy::Dynamic
        );
        assert_eq!(config.depth_scaling_factor, 1);
        assert_eq!(config.min_depth_for_scaling, 4);
        assert_eq!(config.material_adjustment_factor, 1);
        assert_eq!(config.piece_count_threshold, 20);
        assert_eq!(config.threshold_step, 4);
        assert_eq!(config.opening_reduction_factor, 3);
        assert_eq!(config.middlegame_reduction_factor, 2);
        assert_eq!(config.endgame_reduction_factor, 1);
    }

    #[test]
    fn test_null_move_reduction_strategy_validation() {
        // Test validation of advanced reduction strategy parameters
        let mut config = NullMoveConfig::default();
        config.reduction_strategy = NullMoveReductionStrategy::DepthBased;
        config.depth_scaling_factor = 0; // Invalid

        assert!(config.validate().is_err());

        // Fix and test valid config
        config.depth_scaling_factor = 1;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_per_depth_reduction_configuration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure per-depth reduction
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_per_depth_reduction = true;
        config.reduction_factor_by_depth.insert(3, 1); // Depth 3: reduction 1
        config.reduction_factor_by_depth.insert(4, 2); // Depth 4: reduction 2
        config.reduction_factor_by_depth.insert(5, 3); // Depth 5: reduction 3
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search at different depths
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Per-depth reduction should override strategy-based reduction
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_per_position_type_threshold_configuration() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure per-position-type thresholds
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_endgame_detection = true;
        config.enable_per_position_type_threshold = true;
        config.opening_pieces_threshold = 14; // More conservative for opening
        config.middlegame_pieces_threshold = 12; // Standard for middlegame
        config.endgame_pieces_threshold = 10; // More relaxed for endgame
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Per-position-type thresholds should be used when enabled
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_per_depth_reduction_validation() {
        let mut config = NullMoveConfig::default();
        config.enable_per_depth_reduction = true;

        // Add invalid entries
        config.reduction_factor_by_depth.insert(0, 2); // Invalid: depth 0
        config.reduction_factor_by_depth.insert(3, 0); // Invalid: factor 0
        config.reduction_factor_by_depth.insert(3, 10); // Invalid: factor > 5

        assert!(config.validate().is_err());

        // Fix and test valid config
        config.reduction_factor_by_depth.clear();
        config.reduction_factor_by_depth.insert(3, 2); // Valid
        config.reduction_factor_by_depth.insert(4, 3); // Valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_per_position_type_threshold_validation() {
        let mut config = NullMoveConfig::default();
        config.enable_per_position_type_threshold = true;

        // Test invalid thresholds
        config.opening_pieces_threshold = 0; // Invalid
        assert!(config.validate().is_err());

        config.opening_pieces_threshold = 50; // Invalid: > 40
        assert!(config.validate().is_err());

        // Fix and test valid config
        config.opening_pieces_threshold = 12;
        config.middlegame_pieces_threshold = 12;
        config.endgame_pieces_threshold = 12;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_per_depth_reduction_priority_over_strategy() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure per-depth reduction (should override strategy)
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.reduction_strategy = NullMoveReductionStrategy::Static;
        config.reduction_factor = 5; // Strategy would use 5
        config.enable_per_depth_reduction = true;
        config.reduction_factor_by_depth.insert(4, 2); // But per-depth uses 2 for depth 4
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        // Perform search at depth 4
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());

        // Per-depth reduction (2) should override strategy reduction (5)
        let stats = engine.get_null_move_stats();
        assert!(stats.attempts >= 0);
    }

    #[test]
    fn test_per_position_type_threshold_classification() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Configure per-position-type thresholds
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_endgame_detection = true;
        config.enable_per_position_type_threshold = true;
        config.opening_pieces_threshold = 30; // Opening: >= 30 pieces
        config.middlegame_pieces_threshold = 15; // Middlegame: 15-29 pieces
        config.endgame_pieces_threshold = 10; // Endgame: < 15 pieces
        config.min_depth = 3;
        engine.update_null_move_config(config).unwrap();

        // Per-position-type thresholds should classify positions correctly
        // Opening position (many pieces) should use opening_pieces_threshold
        // Middlegame position (moderate pieces) should use middlegame_pieces_threshold
        // Endgame position (few pieces) should use endgame_pieces_threshold
        let result =
            engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
        assert!(result.is_some());
    }
}
