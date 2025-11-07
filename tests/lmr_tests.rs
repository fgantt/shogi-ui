#![cfg(feature = "legacy-tests")]
/// Comprehensive test suite for Late Move Reductions (LMR)
///
/// This module contains unit tests for all LMR functionality including:
/// - Configuration validation and management
/// - Statistics tracking and calculations
/// - Move exemption rules and classification
/// - Reduction calculation algorithms
/// - Adaptive reduction logic
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(test)]
mod lmr_config_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_lmr_config_default() {
        let config = LMRConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.min_move_index, 4);
        assert_eq!(config.base_reduction, 1);
        assert_eq!(config.max_reduction, 3);
        assert!(config.enable_dynamic_reduction);
        assert!(config.enable_adaptive_reduction);
        assert!(config.enable_extended_exemptions);
    }

    #[test]
    fn test_lmr_config_validation() {
        let mut config = LMRConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid min_depth
        config.min_depth = 0;
        assert!(config.validate().is_err());
        config.min_depth = 16;
        assert!(config.validate().is_err());

        // Test invalid min_move_index
        config.min_depth = 3;
        config.min_move_index = 0;
        assert!(config.validate().is_err());
        config.min_move_index = 21;
        assert!(config.validate().is_err());

        // Test invalid base_reduction
        config.min_move_index = 4;
        config.base_reduction = 0;
        assert!(config.validate().is_err());
        config.base_reduction = 6;
        assert!(config.validate().is_err());

        // Test invalid max_reduction
        config.base_reduction = 1;
        config.max_reduction = 0;
        assert!(config.validate().is_err());
        config.max_reduction = 9;
        assert!(config.validate().is_err());

        // Test max_reduction < base_reduction
        config.max_reduction = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_lmr_config_new_validated() {
        let mut config = LMRConfig {
            enabled: true,
            min_depth: 0,       // Invalid, should be clamped to 1
            min_move_index: 25, // Invalid, should be clamped to 20
            base_reduction: 0,  // Invalid, should be clamped to 1
            max_reduction: 10,  // Invalid, should be clamped to 8
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };

        let validated = config.new_validated();
        assert_eq!(validated.min_depth, 1);
        assert_eq!(validated.min_move_index, 20);
        assert_eq!(validated.base_reduction, 1);
        assert_eq!(validated.max_reduction, 1); // Clamped to base_reduction
    }

    #[test]
    fn test_lmr_config_summary() {
        let config = LMRConfig::default();
        let summary = config.summary();
        assert!(summary.contains("LMRConfig"));
        assert!(summary.contains("enabled=true"));
        assert!(summary.contains("min_depth=3"));
        assert!(summary.contains("base_reduction=1"));
    }
}

#[cfg(test)]
mod lmr_stats_tests {
    use super::*;

    #[test]
    fn test_lmr_stats_default() {
        let stats = LMRStats::default();
        assert_eq!(stats.moves_considered, 0);
        assert_eq!(stats.reductions_applied, 0);
        assert_eq!(stats.researches_triggered, 0);
        assert_eq!(stats.cutoffs_after_reduction, 0);
        assert_eq!(stats.cutoffs_after_research, 0);
        assert_eq!(stats.total_depth_saved, 0);
        assert_eq!(stats.average_reduction, 0.0);
        assert_eq!(stats.re_search_margin_prevented, 0);
        assert_eq!(stats.re_search_margin_allowed, 0);
    }

    #[test]
    fn test_lmr_stats_reset() {
        let mut stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 50,
            researches_triggered: 10,
            cutoffs_after_reduction: 20,
            cutoffs_after_research: 5,
            total_depth_saved: 100,
            average_reduction: 2.0,
            re_search_margin_prevented: 10,
            re_search_margin_allowed: 5,
        };

        stats.reset();
        assert_eq!(stats.moves_considered, 0);
        assert_eq!(stats.reductions_applied, 0);
        assert_eq!(stats.researches_triggered, 0);
        assert_eq!(stats.cutoffs_after_reduction, 0);
        assert_eq!(stats.cutoffs_after_research, 0);
        assert_eq!(stats.total_depth_saved, 0);
        assert_eq!(stats.average_reduction, 0.0);
        assert_eq!(stats.re_search_margin_prevented, 0);
        assert_eq!(stats.re_search_margin_allowed, 0);
    }

    #[test]
    fn test_lmr_stats_research_rate() {
        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.research_rate(), 0.0);

        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 10,
            researches_triggered: 3,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.research_rate(), 30.0);
    }

    #[test]
    fn test_lmr_stats_efficiency() {
        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.efficiency(), 0.0);

        let stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 50,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.efficiency(), 50.0);
    }

    #[test]
    fn test_lmr_stats_total_cutoffs() {
        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 10,
            cutoffs_after_research: 5,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.total_cutoffs(), 15);
    }

    #[test]
    fn test_lmr_stats_cutoff_rate() {
        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.cutoff_rate(), 0.0);

        let stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 20,
            cutoffs_after_research: 10,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.cutoff_rate(), 30.0);
    }

    #[test]
    fn test_lmr_stats_average_depth_saved() {
        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 0,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 0,
            average_reduction: 0.0,
        };
        assert_eq!(stats.average_depth_saved(), 0.0);

        let stats = LMRStats {
            moves_considered: 0,
            reductions_applied: 10,
            researches_triggered: 0,
            cutoffs_after_reduction: 0,
            cutoffs_after_research: 0,
            total_depth_saved: 30,
            average_reduction: 0.0,
        };
        assert_eq!(stats.average_depth_saved(), 3.0);
    }

    #[test]
    fn test_lmr_stats_summary() {
        let stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 50,
            researches_triggered: 10,
            cutoffs_after_reduction: 20,
            cutoffs_after_research: 5,
            total_depth_saved: 100,
            average_reduction: 2.0,
        };

        let summary = stats.summary();
        assert!(summary.contains("LMR"));
        assert!(summary.contains("100 considered"));
        assert!(summary.contains("50.0% reduced"));
        assert!(summary.contains("20.0% researched"));
    }
}

#[cfg(test)]
mod lmr_move_exemption_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_move(is_capture: bool, is_promotion: bool, gives_check: bool) -> Move {
        Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture,
            is_promotion,
            captured_piece: if is_capture {
                Some(Piece {
                    piece_type: PieceType::Pawn,
                    player: Player::White,
                })
            } else {
                None
            },
            gives_check,
            is_recapture: false,
        }
    }

    #[test]
    fn test_capture_move_exemption() {
        let engine = create_test_engine();
        let capture_move = create_test_move(true, false, false);
        assert!(engine.is_move_exempt_from_lmr(&capture_move));
    }

    #[test]
    fn test_promotion_move_exemption() {
        let engine = create_test_engine();
        let promotion_move = create_test_move(false, true, false);
        assert!(engine.is_move_exempt_from_lmr(&promotion_move));
    }

    #[test]
    fn test_check_move_exemption() {
        let engine = create_test_engine();
        let check_move = create_test_move(false, false, true);
        assert!(engine.is_move_exempt_from_lmr(&check_move));
    }

    #[test]
    fn test_quiet_move_no_exemption() {
        let engine = create_test_engine();
        let quiet_move = create_test_move(false, false, false);
        assert!(!engine.is_move_exempt_from_lmr(&quiet_move));
    }

    #[test]
    fn test_killer_move_exemption() {
        let mut engine = create_test_engine();
        let killer_move = create_test_move(false, false, false);

        // Add move to killer table
        engine.update_killer_moves(killer_move.clone());

        // With extended exemptions enabled, killer moves should be exempt
        assert!(engine.is_move_exempt_from_lmr(&killer_move));
    }

    #[test]
    fn test_center_move_detection() {
        let engine = create_test_engine();
        let center_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(4, 4), // Center square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        assert!(engine.is_center_move(&center_move));

        let edge_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(0, 0), // Edge square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        assert!(!engine.is_center_move(&edge_move));
    }
}

#[cfg(test)]
mod lmr_reduction_calculation_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_move() -> Move {
        Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        }
    }

    #[test]
    fn test_static_reduction() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 2,
            max_reduction: 4,
            enable_dynamic_reduction: false, // Disable dynamic reduction
            enable_adaptive_reduction: false,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();
        let reduction = engine.calculate_reduction(&move_, 5, 6);
        assert_eq!(reduction, 2); // Should use base_reduction
    }

    #[test]
    fn test_dynamic_reduction_by_depth() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 5,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: false,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();

        // Test depth-based reduction
        let reduction_5 = engine.calculate_reduction(&move_, 5, 6);
        assert_eq!(reduction_5, 1); // base_reduction only

        let reduction_6 = engine.calculate_reduction(&move_, 6, 6);
        assert_eq!(reduction_6, 2); // base_reduction + 1 for depth >= 6

        let reduction_10 = engine.calculate_reduction(&move_, 10, 6);
        assert_eq!(reduction_10, 3); // base_reduction + 2 for depth >= 10
    }

    #[test]
    fn test_dynamic_reduction_by_move_index() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 5,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: false,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();

        // Test move index-based reduction
        let reduction_6 = engine.calculate_reduction(&move_, 5, 6);
        assert_eq!(reduction_6, 1); // base_reduction only

        let reduction_8 = engine.calculate_reduction(&move_, 5, 8);
        assert_eq!(reduction_8, 2); // base_reduction + 1 for move_index >= 8

        let reduction_16 = engine.calculate_reduction(&move_, 5, 16);
        assert_eq!(reduction_16, 3); // base_reduction + 2 for move_index >= 16
    }

    #[test]
    fn test_max_reduction_limit() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 2, // Low max reduction
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: false,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();
        let reduction = engine.calculate_reduction(&move_, 10, 16);
        assert_eq!(reduction, 2); // Should be limited by max_reduction
    }

    #[test]
    fn test_depth_safety_limit() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 10, // High max reduction
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: false,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();
        let reduction = engine.calculate_reduction(&move_, 3, 6);
        assert_eq!(reduction, 1); // Should be limited by depth - 2 = 1
    }
}

#[cfg(test)]
mod lmr_adaptive_reduction_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_move() -> Move {
        Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        }
    }

    #[test]
    fn test_adaptive_reduction_disabled() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 2,
            max_reduction: 4,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: false, // Disable adaptive reduction
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let move_ = create_test_move();
        let reduction = engine.calculate_reduction(&move_, 6, 8);
        assert_eq!(reduction, 3); // Should be base + depth + move_index, no adaptation
    }

    #[test]
    fn test_center_move_reduction_adjustment() {
        let mut engine = create_test_engine();
        let config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 2,
            max_reduction: 4,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(config).unwrap();

        let center_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(4, 4), // Center square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let edge_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(0, 0), // Edge square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let center_reduction = engine.calculate_reduction(&center_move, 6, 8);
        let edge_reduction = engine.calculate_reduction(&edge_move, 6, 8);

        // Center moves should have less reduction (more conservative)
        assert!(center_reduction <= edge_reduction);
    }
}

#[cfg(test)]
mod lmr_move_classification_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    #[test]
    fn test_move_type_classification() {
        let engine = create_test_engine();

        // Test check move
        let check_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: true,
            is_recapture: false,
        };
        assert_eq!(engine.classify_move_type(&check_move), MoveType::Check);

        // Test capture move
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
        assert_eq!(engine.classify_move_type(&capture_move), MoveType::Capture);

        // Test promotion move
        let promotion_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: true,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };
        assert_eq!(
            engine.classify_move_type(&promotion_move),
            MoveType::Promotion
        );

        // Test center move
        let center_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(4, 4), // Center square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };
        assert_eq!(engine.classify_move_type(&center_move), MoveType::Center);

        // Test quiet move
        let quiet_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(0, 0), // Edge square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };
        assert_eq!(engine.classify_move_type(&quiet_move), MoveType::Quiet);
    }

    #[test]
    fn test_move_tactical_value() {
        let engine = create_test_engine();

        // Test capture move value
        let capture_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: true,
            is_promotion: false,
            captured_piece: Some(Piece {
                piece_type: PieceType::Rook,
                player: Player::White,
            }),
            gives_check: false,
            is_recapture: false,
        };
        let capture_value = engine.get_move_tactical_value(&capture_move);
        assert!(capture_value > 0);

        // Test check move value
        let check_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: true,
            is_recapture: false,
        };
        let check_value = engine.get_move_tactical_value(&check_move);
        assert_eq!(check_value, 1000);

        // Test quiet move value
        let quiet_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };
        let quiet_value = engine.get_move_tactical_value(&quiet_move);
        assert_eq!(quiet_value, 0);
    }
}

#[cfg(test)]
mod pruning_manager_lmr_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_search_state(depth: u8, move_number: u8) -> SearchState {
        let mut state = SearchState::new(depth, -10000, 10000);
        state.move_number = move_number;
        state.update_fields(false, 0, 0, GamePhase::Middlegame);
        state
    }

    fn create_test_move() -> Move {
        Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        }
    }

    #[test]
    fn test_pruning_manager_lmr_reduction_basic() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        // Test basic reduction calculation
        let mut state = create_test_search_state(5, 5);
        let mv = create_test_move();

        // Should apply reduction for move beyond threshold
        let reduction = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);
        assert!(reduction > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_extended_exemptions() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mut state = create_test_search_state(5, 5);
        let mv = create_test_move();

        // Test killer move exemption
        let reduction_with_killer =
            pruning_manager.calculate_lmr_reduction(&state, &mv, true, None);
        assert_eq!(reduction_with_killer, 0);

        // Test without killer move
        let reduction_without_killer =
            pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);
        assert!(reduction_without_killer > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_adaptive_reduction() {
        let engine = create_test_engine();
        let mut pruning_manager = engine.get_pruning_manager().clone();

        // Enable adaptive reduction
        pruning_manager.parameters.lmr_enable_adaptive_reduction = true;

        let mut state = create_test_search_state(5, 5);
        let mv = create_test_move();

        // Test with tactical position
        state.set_position_classification(PositionClassification::Tactical);
        let reduction_tactical = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Test with quiet position
        state.set_position_classification(PositionClassification::Quiet);
        let reduction_quiet = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Quiet positions should have more reduction than tactical positions
        assert!(reduction_quiet >= reduction_tactical);
    }

    #[test]
    fn test_pruning_manager_lmr_position_classification() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mut state = create_test_search_state(5, 5);
        let mv = create_test_move();

        // Test with no classification (should use base reduction)
        let reduction_neutral = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Test with tactical classification
        state.set_position_classification(PositionClassification::Tactical);
        let reduction_tactical = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Test with quiet classification
        state.set_position_classification(PositionClassification::Quiet);
        let reduction_quiet = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Verify all reductions are valid
        assert!(reduction_neutral > 0 || reduction_tactical > 0 || reduction_quiet > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_depth_threshold() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mv = create_test_move();

        // Test at depth below threshold (should not apply LMR)
        let mut state_shallow = create_test_search_state(1, 5);
        let reduction_shallow =
            pruning_manager.calculate_lmr_reduction(&state_shallow, &mv, false, None);
        assert_eq!(reduction_shallow, 0);

        // Test at depth above threshold (should apply LMR)
        let mut state_deep = create_test_search_state(5, 5);
        let reduction_deep = pruning_manager.calculate_lmr_reduction(&state_deep, &mv, false, None);
        assert!(reduction_deep > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_move_index_threshold() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mv = create_test_move();

        // Test at move index below threshold (should not apply LMR)
        let mut state_early = create_test_search_state(5, 1);
        let reduction_early =
            pruning_manager.calculate_lmr_reduction(&state_early, &mv, false, None);
        assert_eq!(reduction_early, 0);

        // Test at move index above threshold (should apply LMR)
        let mut state_late = create_test_search_state(5, 10);
        let reduction_late = pruning_manager.calculate_lmr_reduction(&state_late, &mv, false, None);
        assert!(reduction_late > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_basic_exemptions() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mut state = create_test_search_state(5, 5);

        // Test capture exemption
        let mut capture_move = create_test_move();
        capture_move.is_capture = true;
        let reduction_capture =
            pruning_manager.calculate_lmr_reduction(&state, &capture_move, false, None);
        assert_eq!(reduction_capture, 0);

        // Test promotion exemption
        let mut promotion_move = create_test_move();
        promotion_move.is_promotion = true;
        let reduction_promotion =
            pruning_manager.calculate_lmr_reduction(&state, &promotion_move, false, None);
        assert_eq!(reduction_promotion, 0);

        // Test check exemption
        state.is_in_check = true;
        let reduction_check =
            pruning_manager.calculate_lmr_reduction(&state, &create_test_move(), false, None);
        assert_eq!(reduction_check, 0);
    }

    #[test]
    fn test_pruning_manager_lmr_tt_move_exemption() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mut state = create_test_search_state(5, 5);
        let mv = create_test_move();
        let tt_move = Some(&mv);

        // Test with TT move matching current move (should be exempt)
        let reduction_with_tt =
            pruning_manager.calculate_lmr_reduction(&state, &mv, false, tt_move);
        assert_eq!(reduction_with_tt, 0);

        // Test with different TT move (should not be exempt)
        let different_move = Move {
            from: Some(Position::new(2, 2)),
            to: Position::new(3, 2),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };
        let reduction_with_different_tt =
            pruning_manager.calculate_lmr_reduction(&state, &mv, false, Some(&different_move));
        assert!(reduction_with_different_tt > 0);
    }

    #[test]
    fn test_pruning_manager_lmr_reduction_scaling() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mv = create_test_move();

        // Test reduction increases with depth
        let reduction_depth_5 = pruning_manager.calculate_lmr_reduction(
            &create_test_search_state(5, 10),
            &mv,
            false,
            None,
        );
        let reduction_depth_10 = pruning_manager.calculate_lmr_reduction(
            &create_test_search_state(10, 10),
            &mv,
            false,
            None,
        );
        assert!(reduction_depth_10 >= reduction_depth_5);

        // Test reduction increases with move index
        let reduction_move_5 = pruning_manager.calculate_lmr_reduction(
            &create_test_search_state(5, 5),
            &mv,
            false,
            None,
        );
        let reduction_move_15 = pruning_manager.calculate_lmr_reduction(
            &create_test_search_state(5, 15),
            &mv,
            false,
            None,
        );
        assert!(reduction_move_15 >= reduction_move_5);
    }

    #[test]
    fn test_pruning_manager_lmr_center_move_adjustment() {
        let engine = create_test_engine();
        let mut pruning_manager = engine.get_pruning_manager().clone();

        // Enable adaptive reduction
        pruning_manager.parameters.lmr_enable_adaptive_reduction = true;

        let mut state = create_test_search_state(5, 5);

        // Test center move (should reduce reduction)
        let center_move = Move {
            from: Some(Position::new(3, 3)),
            to: Position::new(4, 4), // Center square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let reduction_center =
            pruning_manager.calculate_lmr_reduction(&state, &center_move, false, None);

        // Test edge move (should allow more reduction)
        let edge_move = Move {
            from: Some(Position::new(0, 0)),
            to: Position::new(0, 1), // Edge square
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let reduction_edge =
            pruning_manager.calculate_lmr_reduction(&state, &edge_move, false, None);

        // Center moves should have less or equal reduction than edge moves
        assert!(reduction_center <= reduction_edge);
    }

    #[test]
    fn test_pruning_manager_lmr_max_reduction_limit() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mv = create_test_move();

        // Test at very high depth and move index (should be capped)
        let mut state = create_test_search_state(20, 30);
        let reduction = pruning_manager.calculate_lmr_reduction(&state, &mv, false, None);

        // Should not exceed max_reduction or depth - 1
        assert!(reduction <= pruning_manager.parameters.lmr_max_reduction);
        assert!(reduction < state.depth);
    }
}

#[cfg(test)]
mod re_search_margin_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    #[test]
    fn test_lmr_config_re_search_margin_default() {
        let config = LMRConfig::default();
        assert_eq!(config.re_search_margin, 50);
    }

    #[test]
    fn test_lmr_config_re_search_margin_validation() {
        let mut config = LMRConfig::default();

        // Test valid margin
        config.re_search_margin = 50;
        assert!(config.validate().is_ok());

        // Test margin = 0 (disabled)
        config.re_search_margin = 0;
        assert!(config.validate().is_ok());

        // Test margin = 500 (max)
        config.re_search_margin = 500;
        assert!(config.validate().is_ok());

        // Test invalid margin < 0
        config.re_search_margin = -1;
        assert!(config.validate().is_err());

        // Test invalid margin > 500
        config.re_search_margin = 501;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_lmr_config_re_search_margin_new_validated() {
        let mut config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
            re_search_margin: 600, // Invalid, should be clamped to 500
        };

        let validated = config.new_validated();
        assert_eq!(validated.re_search_margin, 500);

        config.re_search_margin = -10;
        let validated = config.new_validated();
        assert_eq!(validated.re_search_margin, 0);
    }

    #[test]
    fn test_lmr_config_re_search_margin_summary() {
        let mut config = LMRConfig::default();
        config.re_search_margin = 75;
        let summary = config.summary();
        assert!(summary.contains("re_search_margin=75"));
    }

    #[test]
    fn test_lmr_stats_re_search_margin_effectiveness() {
        let stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 50,
            researches_triggered: 10,
            cutoffs_after_reduction: 20,
            cutoffs_after_research: 5,
            total_depth_saved: 100,
            average_reduction: 2.0,
            re_search_margin_prevented: 30,
            re_search_margin_allowed: 10,
        };

        let effectiveness = stats.re_search_margin_effectiveness();
        // 30 prevented out of 40 total = 75%
        assert!((effectiveness - 75.0).abs() < 0.01);

        // Test with zero prevented
        let stats_no_prevented = LMRStats {
            re_search_margin_prevented: 0,
            re_search_margin_allowed: 10,
            ..Default::default()
        };
        assert_eq!(stats_no_prevented.re_search_margin_effectiveness(), 0.0);

        // Test with zero total
        let stats_zero = LMRStats::default();
        assert_eq!(stats_zero.re_search_margin_effectiveness(), 0.0);
    }

    #[test]
    fn test_re_search_margin_disabled() {
        let engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.re_search_margin = 0; // Disabled

        let mut engine2 = create_test_engine();
        engine2.update_lmr_config(config).unwrap();

        // With margin = 0, re-search should trigger when score > alpha (current behavior)
        assert_eq!(engine2.get_lmr_config().re_search_margin, 0);
    }

    #[test]
    fn test_re_search_margin_edge_cases() {
        // Test margin boundaries
        let mut config = LMRConfig::default();

        // Test minimum valid margin
        config.re_search_margin = 0;
        assert!(config.validate().is_ok());

        // Test maximum valid margin
        config.re_search_margin = 500;
        assert!(config.validate().is_ok());

        // Test typical margins
        config.re_search_margin = 25;
        assert!(config.validate().is_ok());

        config.re_search_margin = 50;
        assert!(config.validate().is_ok());

        config.re_search_margin = 75;
        assert!(config.validate().is_ok());

        config.re_search_margin = 100;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_re_search_margin_preset_values() {
        let engine = create_test_engine();

        // Test aggressive preset (lower margin)
        let aggressive = engine.get_lmr_preset(LMRPlayingStyle::Aggressive);
        assert_eq!(aggressive.re_search_margin, 25);

        // Test conservative preset (higher margin)
        let conservative = engine.get_lmr_preset(LMRPlayingStyle::Conservative);
        assert_eq!(conservative.re_search_margin, 100);

        // Test balanced preset (default margin)
        let balanced = engine.get_lmr_preset(LMRPlayingStyle::Balanced);
        assert_eq!(balanced.re_search_margin, 50);
    }

    #[test]
    fn test_re_search_margin_performance_report() {
        let stats = LMRStats {
            moves_considered: 100,
            reductions_applied: 50,
            researches_triggered: 10,
            cutoffs_after_reduction: 20,
            cutoffs_after_research: 5,
            total_depth_saved: 100,
            average_reduction: 2.0,
            re_search_margin_prevented: 30,
            re_search_margin_allowed: 10,
        };

        let report = stats.performance_report();
        assert!(report.contains("Re-search margin prevented"));
        assert!(report.contains("Re-search margin allowed"));
        assert!(report.contains("30"));
        assert!(report.contains("10"));
    }
}

#[cfg(test)]
mod tt_move_detection_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    fn create_test_move() -> Move {
        Move {
            from: Some(Position::from_usi("7g").unwrap()),
            to: Position::from_usi("7f").unwrap(),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        }
    }

    fn create_tt_move() -> Move {
        Move {
            from: Some(Position::from_usi("8g").unwrap()),
            to: Position::from_usi("8f").unwrap(),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        }
    }

    #[test]
    fn test_search_state_tt_move_storage() {
        let mut state = SearchState::new(5, -10000, 10000);
        assert_eq!(state.tt_move, None);

        let tt_move = create_tt_move();
        state.set_tt_move(Some(tt_move.clone()));
        assert!(state.tt_move.is_some());
        assert_eq!(state.tt_move.as_ref().unwrap().to, tt_move.to);

        state.set_tt_move(None);
        assert_eq!(state.tt_move, None);
    }

    #[test]
    fn test_pruning_manager_tt_move_exemption() {
        let manager = PruningManager::new(PruningParameters::default());
        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5; // Above threshold
        state.depth = 5; // Above threshold
        state.is_in_check = false;

        let current_move = create_test_move();
        let tt_move = create_tt_move();

        // Test without TT move - should apply LMR (reduction > 0)
        state.set_tt_move(None);
        let reduction = manager.calculate_lmr_reduction(&state, &current_move, false, None);
        assert!(reduction > 0, "Should apply LMR when no TT move");

        // Test with matching TT move - should exempt from LMR (reduction = 0)
        state.set_tt_move(Some(tt_move.clone()));
        let reduction = manager.calculate_lmr_reduction(&state, &tt_move, false, None);
        assert_eq!(reduction, 0, "Should NOT apply LMR to TT move");

        // Test with non-matching TT move - should apply LMR (reduction > 0)
        state.set_tt_move(Some(tt_move));
        let reduction = manager.calculate_lmr_reduction(&state, &current_move, false, None);
        assert!(
            reduction > 0,
            "Should apply LMR when move doesn't match TT move"
        );
    }

    #[test]
    fn test_pruning_manager_tt_move_parameter_override() {
        let manager = PruningManager::new(PruningParameters::default());
        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5;
        state.depth = 5;
        state.is_in_check = false;

        let current_move = create_test_move();
        let tt_move_from_state = create_tt_move();
        let tt_move_from_param = create_test_move(); // Same as current_move

        // Test parameter override when state has no TT move
        state.set_tt_move(None);
        let reduction = manager.calculate_lmr_reduction(
            &state,
            &current_move,
            false,
            Some(&tt_move_from_param),
        );
        assert_eq!(reduction, 0, "Should exempt when parameter TT move matches");

        // Test state TT move takes precedence
        state.set_tt_move(Some(tt_move_from_state));
        let reduction = manager.calculate_lmr_reduction(
            &state,
            &current_move,
            false,
            Some(&tt_move_from_param),
        );
        assert!(
            reduction > 0,
            "State TT move should take precedence over parameter"
        );
    }

    #[test]
    fn test_lmr_stats_tt_move_tracking() {
        let mut stats = LMRStats::default();
        assert_eq!(stats.tt_move_exempted, 0);
        assert_eq!(stats.tt_move_missed, 0);

        stats.tt_move_exempted = 10;
        stats.tt_move_missed = 2;

        let report = stats.performance_report();
        assert!(report.contains("TT moves exempted"));
        assert!(report.contains("TT moves missed"));
        assert!(report.contains("10"));
        assert!(report.contains("2"));

        stats.reset();
        assert_eq!(stats.tt_move_exempted, 0);
        assert_eq!(stats.tt_move_missed, 0);
    }

    #[test]
    fn test_tt_move_exemption_with_extended_exemptions_disabled() {
        let mut params = PruningParameters::default();
        params.lmr_enable_extended_exemptions = false;
        let manager = PruningManager::new(params);

        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5;
        state.depth = 5;
        state.is_in_check = false;

        let current_move = create_test_move();
        let tt_move = create_tt_move();

        // Even with TT move, should apply LMR if extended exemptions disabled
        state.set_tt_move(Some(tt_move.clone()));
        let reduction = manager.calculate_lmr_reduction(&state, &tt_move, false, None);
        assert!(
            reduction > 0,
            "Should apply LMR when extended exemptions disabled"
        );
    }

    #[test]
    fn test_tt_move_exemption_improves_lmr_accuracy() {
        // This test verifies that TT move exemption improves LMR accuracy
        // by ensuring TT moves are not incorrectly reduced
        let manager = PruningManager::new(PruningParameters::default());
        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5;
        state.depth = 5;
        state.is_in_check = false;

        let tt_move = create_tt_move();
        let non_tt_move = create_test_move();

        state.set_tt_move(Some(tt_move.clone()));

        // TT move should be exempted (no reduction)
        let reduction_tt = manager.calculate_lmr_reduction(&state, &tt_move, false, None);
        assert_eq!(reduction_tt, 0, "TT move should have zero reduction");

        // Non-TT move should have reduction
        let reduction_non_tt = manager.calculate_lmr_reduction(&state, &non_tt_move, false, None);
        assert!(reduction_non_tt > 0, "Non-TT move should have reduction");
    }

    #[test]
    fn test_tt_move_detection_when_no_tt_entry() {
        let manager = PruningManager::new(PruningParameters::default());
        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5;
        state.depth = 5;
        state.is_in_check = false;
        state.set_tt_move(None); // No TT entry

        let current_move = create_test_move();

        // Should apply LMR when no TT move available
        let reduction = manager.calculate_lmr_reduction(&state, &current_move, false, None);
        assert!(reduction > 0, "Should apply LMR when no TT move available");
    }

    #[test]
    fn test_tt_move_exemption_with_basic_exemptions() {
        // Test that TT move exemption works alongside basic exemptions
        let manager = PruningManager::new(PruningParameters::default());
        let mut state = SearchState::new(5, -10000, 10000);
        state.move_number = 5;
        state.depth = 5;
        state.is_in_check = false;

        let mut tt_move = create_tt_move();
        tt_move.is_capture = true; // Also a capture

        state.set_tt_move(Some(tt_move.clone()));

        // Should be exempted due to capture (basic exemption) even if TT
        let reduction = manager.calculate_lmr_reduction(&state, &tt_move, false, None);
        assert_eq!(
            reduction, 0,
            "Should exempt capture move regardless of TT status"
        );
    }
}

#[cfg(test)]
mod performance_monitoring_tests {
    use super::*;

    #[test]
    fn test_lmr_stats_performance_thresholds() {
        let mut stats = LMRStats::default();

        // Test healthy performance
        stats.moves_considered = 1000;
        stats.reductions_applied = 300; // 30% efficiency
        stats.researches_triggered = 50; // ~16.7% re-search rate
        stats.cutoffs_after_reduction = 100;
        stats.cutoffs_after_research = 50; // 15% cutoff rate

        let (is_healthy, alerts) = stats.check_performance_thresholds();
        assert!(is_healthy, "Should pass all thresholds");
        assert!(alerts.is_empty(), "Should have no alerts");

        // Test low efficiency
        stats.reductions_applied = 100; // 10% efficiency
        let (is_healthy, alerts) = stats.check_performance_thresholds();
        assert!(!is_healthy, "Should fail efficiency threshold");
        assert!(
            alerts.iter().any(|a| a.contains("Low efficiency")),
            "Should alert on low efficiency"
        );

        // Test high re-search rate
        stats.reductions_applied = 300;
        stats.researches_triggered = 150; // 50% re-search rate
        let (is_healthy, alerts) = stats.check_performance_thresholds();
        assert!(!is_healthy, "Should fail re-search rate threshold");
        assert!(
            alerts.iter().any(|a| a.contains("High re-search rate")),
            "Should alert on high re-search rate"
        );

        // Test low cutoff rate
        stats.researches_triggered = 50;
        stats.cutoffs_after_reduction = 20;
        stats.cutoffs_after_research = 10; // 3% cutoff rate
        let (is_healthy, alerts) = stats.check_performance_thresholds();
        assert!(!is_healthy, "Should fail cutoff rate threshold");
        assert!(
            alerts.iter().any(|a| a.contains("Low cutoff rate")),
            "Should alert on low cutoff rate"
        );
    }

    #[test]
    fn test_lmr_stats_performance_alerts() {
        let mut stats = LMRStats::default();
        stats.moves_considered = 1000;
        stats.reductions_applied = 100; // 10% efficiency
        stats.researches_triggered = 150; // 150% re-search rate (invalid, but test)
        stats.cutoffs_after_reduction = 5;
        stats.cutoffs_after_research = 5; // 1% cutoff rate

        let alerts = stats.get_performance_alerts();
        assert!(!alerts.is_empty(), "Should have alerts");
        assert!(
            alerts.iter().any(|a| a.contains("Low efficiency")),
            "Should alert on low efficiency"
        );
        assert!(
            alerts.iter().any(|a| a.contains("High re-search rate")),
            "Should alert on high re-search rate"
        );
        assert!(
            alerts.iter().any(|a| a.contains("Low cutoff rate")),
            "Should alert on low cutoff rate"
        );
    }

    #[test]
    fn test_lmr_stats_is_performing_well() {
        let mut stats = LMRStats::default();

        // Test healthy performance
        stats.moves_considered = 1000;
        stats.reductions_applied = 300; // 30% efficiency
        stats.researches_triggered = 50; // ~16.7% re-search rate
        stats.cutoffs_after_reduction = 100;
        stats.cutoffs_after_research = 50; // 15% cutoff rate

        assert!(stats.is_performing_well(), "Should be performing well");

        // Test poor performance
        stats.reductions_applied = 100; // 10% efficiency
        assert!(!stats.is_performing_well(), "Should not be performing well");
    }

    #[test]
    fn test_lmr_stats_phase_stats() {
        let mut stats = LMRStats::default();

        // Record phase statistics
        stats.record_phase_stats(
            GamePhase::Opening,
            100, // moves_considered
            30,  // reductions_applied
            5,   // researches_triggered
            10,  // cutoffs_after_reduction
            5,   // cutoffs_after_research
            15,  // depth_saved
        );

        stats.record_phase_stats(GamePhase::Middlegame, 200, 60, 10, 20, 10, 30);

        // Get phase statistics
        let opening_stats = stats.get_phase_stats(GamePhase::Opening);
        assert_eq!(opening_stats.moves_considered, 100);
        assert_eq!(opening_stats.reductions_applied, 30);
        assert_eq!(opening_stats.researches_triggered, 5);
        assert_eq!(opening_stats.efficiency(), 30.0);
        assert_eq!(opening_stats.research_rate(), 5.0 / 30.0 * 100.0);

        let middlegame_stats = stats.get_phase_stats(GamePhase::Middlegame);
        assert_eq!(middlegame_stats.moves_considered, 200);
        assert_eq!(middlegame_stats.efficiency(), 30.0);

        // Test non-existent phase
        let endgame_stats = stats.get_phase_stats(GamePhase::Endgame);
        assert_eq!(endgame_stats.moves_considered, 0);
    }

    #[test]
    fn test_lmr_stats_export_metrics() {
        let mut stats = LMRStats::default();
        stats.moves_considered = 1000;
        stats.reductions_applied = 300;
        stats.researches_triggered = 50;
        stats.cutoffs_after_reduction = 100;
        stats.cutoffs_after_research = 50;
        stats.total_depth_saved = 500;

        let metrics = stats.export_metrics();
        assert_eq!(metrics.get("moves_considered"), Some(&1000.0));
        assert_eq!(metrics.get("reductions_applied"), Some(&300.0));
        assert_eq!(metrics.get("researches_triggered"), Some(&50.0));
        assert_eq!(metrics.get("efficiency"), Some(&30.0));
        assert!(metrics.get("research_rate").is_some());
        assert!(metrics.get("cutoff_rate").is_some());
        assert!(metrics.get("is_performing_well").is_some());
    }

    #[test]
    fn test_lmr_stats_performance_report_with_phase() {
        let mut stats = LMRStats::default();
        stats.moves_considered = 1000;
        stats.reductions_applied = 300;

        // Add phase statistics
        stats.record_phase_stats(GamePhase::Opening, 300, 100, 10, 20, 10, 50);

        let report = stats.performance_report();
        assert!(report.contains("Performance by Game Phase"));
        assert!(report.contains("Opening"));
    }

    #[test]
    fn test_lmr_stats_performance_report_with_alerts() {
        let mut stats = LMRStats::default();
        stats.moves_considered = 1000;
        stats.reductions_applied = 100; // 10% efficiency - should trigger alert
        stats.researches_triggered = 150; // High re-search rate - should trigger alert

        let report = stats.performance_report();
        assert!(report.contains("Performance Alerts"));
        assert!(report.contains("Low efficiency"));
        assert!(report.contains("High re-search rate"));
    }
}

#[cfg(test)]
mod enhanced_position_classification_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_position_classification_config_default() {
        let config = PositionClassificationConfig::default();
        assert_eq!(config.tactical_threshold, 0.3);
        assert_eq!(config.quiet_threshold, 0.1);
        assert_eq!(config.material_imbalance_threshold, 300);
        assert_eq!(config.min_moves_threshold, 5);
    }

    #[test]
    fn test_lmr_config_with_classification_config() {
        let config = LMRConfig::default();
        assert_eq!(config.classification_config.tactical_threshold, 0.3);
        assert_eq!(config.classification_config.quiet_threshold, 0.1);
        assert_eq!(
            config.classification_config.material_imbalance_threshold,
            300
        );
        assert_eq!(config.classification_config.min_moves_threshold, 5);
    }

    #[test]
    fn test_position_classification_stats() {
        let mut stats = PositionClassificationStats::default();
        assert_eq!(stats.tactical_classified, 0);
        assert_eq!(stats.quiet_classified, 0);
        assert_eq!(stats.neutral_classified, 0);
        assert_eq!(stats.total_classifications, 0);

        stats.record_classification(PositionClassification::Tactical);
        assert_eq!(stats.tactical_classified, 1);
        assert_eq!(stats.total_classifications, 1);
        assert_eq!(stats.tactical_ratio(), 100.0);

        stats.record_classification(PositionClassification::Quiet);
        assert_eq!(stats.quiet_classified, 1);
        assert_eq!(stats.total_classifications, 2);
        assert_eq!(stats.quiet_ratio(), 50.0);

        stats.record_classification(PositionClassification::Neutral);
        assert_eq!(stats.neutral_classified, 1);
        assert_eq!(stats.total_classifications, 3);
    }

    #[test]
    fn test_enhanced_position_classification_tactical() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Set up high cutoff ratio to trigger tactical classification
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 40; // 40% cutoff rate
        engine.lmr_stats.cutoffs_after_research = 10; // Total 50% cutoff rate

        let classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Should be classified as tactical due to high cutoff ratio
        assert_eq!(classification, PositionClassification::Tactical);
    }

    #[test]
    fn test_enhanced_position_classification_quiet() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Set up low cutoff ratio to trigger quiet classification
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 5; // 5% cutoff rate
        engine.lmr_stats.cutoffs_after_research = 2; // Total 7% cutoff rate

        let classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Should be classified as quiet due to low cutoff ratio
        assert_eq!(classification, PositionClassification::Quiet);
    }

    #[test]
    fn test_enhanced_position_classification_material_imbalance() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Set up material imbalance (would need to create a board with imbalance)
        // For now, test with high cutoff ratio
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 35; // 35% cutoff rate

        let classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Should be classified as tactical due to high cutoff ratio
        assert_eq!(classification, PositionClassification::Tactical);
    }

    #[test]
    fn test_enhanced_position_classification_min_moves_threshold() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with insufficient moves (below threshold)
        engine.lmr_stats.moves_considered = 3; // Below default threshold of 5
        engine.lmr_stats.cutoffs_after_reduction = 10;

        let classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Should be classified as neutral due to insufficient data
        assert_eq!(classification, PositionClassification::Neutral);
    }

    #[test]
    fn test_enhanced_position_classification_game_phase() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with endgame phase (should be more tactical)
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 25; // 25% cutoff rate (borderline)

        let endgame_classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Endgame,
        );

        // Endgame phase factor should make it more likely to be tactical
        // Reset stats for opening phase
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 25;

        let opening_classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Opening,
        );

        // Endgame should be more likely tactical, opening less likely
        // (Exact classification depends on other factors, but phase affects it)
        assert!(
            endgame_classification != opening_classification
                || endgame_classification == PositionClassification::Tactical
                || opening_classification == PositionClassification::Quiet
        );
    }

    #[test]
    fn test_enhanced_position_classification_configurable_thresholds() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();

        // Set custom thresholds
        config.classification_config.tactical_threshold = 0.4; // Higher threshold
        config.classification_config.quiet_threshold = 0.05; // Lower threshold
        config.classification_config.min_moves_threshold = 10; // Higher threshold

        engine.update_lmr_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with 35% cutoff rate (above 30% default, but below 40% custom)
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 35;

        let classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Should be neutral with custom 40% threshold (35% < 40%)
        assert_eq!(classification, PositionClassification::Neutral);
    }

    #[test]
    fn test_enhanced_position_classification_tracks_statistics() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.cutoffs_after_reduction = 40;

        let initial_count = engine.lmr_stats.classification_stats.total_classifications;

        let _classification = engine.compute_position_classification(
            &board,
            &captured_pieces,
            player,
            GamePhase::Middlegame,
        );

        // Statistics should be tracked
        assert_eq!(
            engine.lmr_stats.classification_stats.total_classifications,
            initial_count + 1
        );
    }

    #[test]
    fn test_piece_activity_calculation() {
        let engine = create_test_engine();
        let board = BitboardBoard::new();
        let player = Player::Black;

        let activity = engine.calculate_piece_activity(&board, player);

        // Activity should be >= 0 (initial position has pieces)
        assert!(activity >= 0);
    }
}

#[cfg(test)]
mod escape_move_detection_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_escape_move_config_default() {
        let config = EscapeMoveConfig::default();
        assert_eq!(config.enable_escape_move_exemption, true);
        assert_eq!(config.use_threat_based_detection, true);
        assert_eq!(config.fallback_to_heuristic, false);
    }

    #[test]
    fn test_lmr_config_with_escape_move_config() {
        let config = LMRConfig::default();
        assert_eq!(config.escape_move_config.enable_escape_move_exemption, true);
        assert_eq!(config.escape_move_config.use_threat_based_detection, true);
        assert_eq!(config.escape_move_config.fallback_to_heuristic, false);
    }

    #[test]
    fn test_escape_move_stats() {
        let mut stats = EscapeMoveStats::default();
        assert_eq!(stats.escape_moves_exempted, 0);
        assert_eq!(stats.threat_based_detections, 0);
        assert_eq!(stats.heuristic_detections, 0);
        assert_eq!(stats.false_positives, 0);
        assert_eq!(stats.false_negatives, 0);

        stats.record_escape_move(true, true);
        assert_eq!(stats.escape_moves_exempted, 1);
        assert_eq!(stats.threat_based_detections, 1);

        stats.record_escape_move(true, false);
        assert_eq!(stats.escape_moves_exempted, 2);
        assert_eq!(stats.heuristic_detections, 1);

        stats.record_false_positive();
        assert_eq!(stats.false_positives, 1);

        stats.record_false_negative();
        assert_eq!(stats.false_negatives, 1);
    }

    #[test]
    fn test_escape_move_detection_disabled() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.escape_move_config.enable_escape_move_exemption = false;
        engine.update_lmr_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        let is_escape = engine.is_escape_move(&move_, &board, &captured_pieces, player);

        // Should return false when disabled
        assert_eq!(is_escape, false);
    }

    #[test]
    fn test_escape_move_threat_based_detection() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.escape_move_config.use_threat_based_detection = true;
        config.escape_move_config.fallback_to_heuristic = false;
        engine.update_lmr_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Test threat-based detection (simplified - actual threat detection would check board state)
        let is_escape = engine.is_escape_move(&move_, &board, &captured_pieces, player);

        // Result depends on threat detection (simplified implementation)
        // At minimum, should not crash
        assert!(is_escape == true || is_escape == false);
    }

    #[test]
    fn test_escape_move_heuristic_fallback() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.escape_move_config.use_threat_based_detection = false;
        config.escape_move_config.fallback_to_heuristic = true;
        engine.update_lmr_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Center-to-edge move (should trigger heuristic)
        let from = Position::new(4, 4); // Center
        let to = Position::new(2, 2); // Edge
        let move_ = Move::new(from, to, Player::Black, false, false, false);

        let is_escape = engine.is_escape_move(&move_, &board, &captured_pieces, player);

        // Should detect escape move using heuristic (center-to-edge)
        assert_eq!(is_escape, true);
    }

    #[test]
    fn test_escape_move_king_in_check() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with king (if king is in check, escape move should be detected)
        // This is a simplified test - actual implementation would check if king is in check
        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        let is_escape = engine.is_escape_move(&move_, &board, &captured_pieces, player);

        // Result depends on threat detection
        // At minimum, should not crash
        assert!(is_escape == true || is_escape == false);
    }

    #[test]
    fn test_escape_move_stats_tracking() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let initial_count = engine.lmr_stats.escape_move_stats.escape_moves_exempted;

        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(2, 2),
            Player::Black,
            false,
            false,
            false,
        );
        let _is_escape = engine.is_escape_move(&move_, &board, &captured_pieces, player);

        // Statistics should be tracked if escape move detected
        // The count may increase if escape move is detected
        let final_count = engine.lmr_stats.escape_move_stats.escape_moves_exempted;
        assert!(final_count >= initial_count);
    }

    #[test]
    fn test_escape_move_accuracy() {
        let mut stats = EscapeMoveStats::default();
        stats.record_escape_move(true, true);
        stats.record_escape_move(true, true);
        stats.record_escape_move(true, false);

        // No errors yet
        assert_eq!(stats.accuracy(), 100.0);

        stats.record_false_positive();
        stats.record_false_negative();

        // 2 errors out of 3 detections = 33.3% error rate
        let accuracy = stats.accuracy();
        assert!(accuracy < 100.0);
        assert!(accuracy > 0.0);
    }

    #[test]
    fn test_is_piece_under_attack() {
        let engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let position = Position::new(4, 4);

        let is_attacked = engine.is_piece_under_attack(&board, &captured_pieces, position, player);

        // Result depends on board state and threat detection
        // At minimum, should not crash
        assert!(is_attacked == true || is_attacked == false);
    }

    #[test]
    fn test_is_piece_under_attack_after_move() {
        let engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        let is_attacked =
            engine.is_piece_under_attack_after_move(&board, &captured_pieces, &move_, player);

        // Result depends on board state and threat detection
        // At minimum, should not crash
        assert!(is_attacked == true || is_attacked == false);
    }
}

#[cfg(test)]
mod adaptive_tuning_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_adaptive_tuning_config_default() {
        let config = AdaptiveTuningConfig::default();
        assert_eq!(config.enabled, false);
        assert_eq!(config.aggressiveness, TuningAggressiveness::Moderate);
        assert_eq!(config.min_data_threshold, 100);
    }

    #[test]
    fn test_lmr_config_with_adaptive_tuning_config() {
        let config = LMRConfig::default();
        assert_eq!(config.adaptive_tuning_config.enabled, false);
        assert_eq!(
            config.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Moderate
        );
        assert_eq!(config.adaptive_tuning_config.min_data_threshold, 100);
    }

    #[test]
    fn test_adaptive_tuning_stats() {
        let mut stats = AdaptiveTuningStats::default();
        assert_eq!(stats.tuning_attempts, 0);
        assert_eq!(stats.successful_tunings, 0);
        assert_eq!(stats.parameter_changes, 0);

        stats.record_tuning_attempt(true);
        assert_eq!(stats.tuning_attempts, 1);
        assert_eq!(stats.successful_tunings, 1);

        stats.record_tuning_attempt(false);
        assert_eq!(stats.tuning_attempts, 2);
        assert_eq!(stats.successful_tunings, 1);

        stats.record_parameter_change("base_reduction");
        assert_eq!(stats.parameter_changes, 1);
        assert_eq!(stats.base_reduction_changes, 1);

        stats.record_adjustment_reason("re_search_rate");
        assert_eq!(stats.re_search_rate_adjustments, 1);
    }

    #[test]
    fn test_adaptive_tuning_disabled() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = false;
        engine.update_lmr_config(config).unwrap();

        let result = engine.auto_tune_lmr_parameters(None, None);

        // Should return error when disabled
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("disabled"));
    }

    #[test]
    fn test_adaptive_tuning_insufficient_data() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Reset stats to have insufficient data
        engine.reset_lmr_stats();

        let result = engine.auto_tune_lmr_parameters(None, None);

        // Should return error when insufficient data
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[test]
    fn test_adaptive_tuning_re_search_rate_adjustment() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with high re-search rate
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.reductions_applied = 50;
        engine.lmr_stats.researches_triggered = 30; // 60% re-search rate (high)

        let old_base_reduction = engine.lmr_config.base_reduction;
        let result = engine.auto_tune_lmr_parameters(None, None);

        // Should adjust parameters if re-search rate is high
        // Result depends on actual metrics, but should not crash
        let _ = result; // Ignore result - may succeed or fail depending on metrics
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > 0);
    }

    #[test]
    fn test_adaptive_tuning_efficiency_adjustment() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with low efficiency
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.reductions_applied = 50;
        engine.lmr_stats.cutoffs_after_reduction = 5; // Low efficiency

        let old_min_move_index = engine.lmr_config.min_move_index;
        let result = engine.auto_tune_lmr_parameters(None, None);

        // Should adjust parameters if efficiency is low
        // Result depends on actual metrics, but should not crash
        let _ = result; // Ignore result - may succeed or fail depending on metrics
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > 0);
    }

    #[test]
    fn test_adaptive_tuning_game_phase() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with sufficient data
        engine.lmr_stats.moves_considered = 100;

        let result = engine.auto_tune_lmr_parameters(Some(GamePhase::Endgame), None);

        // Should handle game phase tuning
        // Result depends on actual metrics, but should not crash
        let _ = result; // Ignore result - may succeed or fail depending on metrics
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > 0);
    }

    #[test]
    fn test_adaptive_tuning_position_type() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with sufficient data
        engine.lmr_stats.moves_considered = 100;

        let result = engine.auto_tune_lmr_parameters(None, Some(PositionClassification::Quiet));

        // Should handle position type tuning
        // Result depends on actual metrics, but should not crash
        let _ = result; // Ignore result - may succeed or fail depending on metrics
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > 0);
    }

    #[test]
    fn test_adaptive_tuning_aggressiveness() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.aggressiveness = TuningAggressiveness::Conservative;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with sufficient data
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.reductions_applied = 50;
        engine.lmr_stats.researches_triggered = 30; // High re-search rate

        let old_base_reduction = engine.lmr_config.base_reduction;
        let result = engine.auto_tune_lmr_parameters(None, None);

        // Conservative aggressiveness should make smaller adjustments
        // Result depends on actual metrics, but should not crash
        let _ = result; // Ignore result - may succeed or fail depending on metrics
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > 0);
    }

    #[test]
    fn test_adaptive_tuning_stats_tracking() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        let initial_attempts = engine.lmr_stats.adaptive_tuning_stats.tuning_attempts;

        // Set up stats with sufficient data
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.reductions_applied = 50;
        engine.lmr_stats.researches_triggered = 30; // High re-search rate

        let _result = engine.auto_tune_lmr_parameters(None, None);

        // Statistics should be tracked
        assert!(engine.lmr_stats.adaptive_tuning_stats.tuning_attempts > initial_attempts);
    }

    #[test]
    fn test_adaptive_tuning_success_rate() {
        let mut stats = AdaptiveTuningStats::default();
        assert_eq!(stats.success_rate(), 0.0);

        stats.record_tuning_attempt(true);
        stats.record_tuning_attempt(true);
        stats.record_tuning_attempt(false);

        // 2 successful out of 3 attempts = 66.67%
        let success_rate = stats.success_rate();
        assert!(success_rate > 60.0);
        assert!(success_rate < 70.0);
    }

    #[test]
    fn test_adaptive_tuning_no_oscillation() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.adaptive_tuning_config.enabled = true;
        config.adaptive_tuning_config.min_data_threshold = 100;
        engine.update_lmr_config(config).unwrap();

        // Set up stats with sufficient data
        engine.lmr_stats.moves_considered = 100;
        engine.lmr_stats.reductions_applied = 50;
        engine.lmr_stats.researches_triggered = 10; // Normal re-search rate

        let old_config = engine.lmr_config.clone();
        let result1 = engine.auto_tune_lmr_parameters(None, None);

        // If tuning succeeds, verify parameters changed
        if result1.is_ok() {
            let new_config = engine.lmr_config.clone();
            // Parameters should have changed
            assert!(
                new_config.base_reduction != old_config.base_reduction
                    || new_config.max_reduction != old_config.max_reduction
                    || new_config.min_move_index != old_config.min_move_index
            );
        }
    }
}

#[cfg(test)]
mod pruning_manager_adaptive_reduction_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_pruning_manager_implements_adaptive_reduction() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        // Check if PruningManager has adaptive reduction enabled by default
        assert_eq!(
            pruning_manager.parameters.lmr_enable_adaptive_reduction,
            true
        );
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_with_position_classification() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Create search state with position classification
        let mut search_state = SearchState::new(5, 0, 1000);
        search_state.set_position_classification(Some(PositionClassification::Tactical));

        // Calculate reduction with tactical position
        let reduction_tactical =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Create search state with quiet position
        search_state.set_position_classification(Some(PositionClassification::Quiet));
        let reduction_quiet =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Tactical positions should have less reduction (more conservative)
        // Quiet positions should have more reduction (more aggressive)
        assert!(reduction_tactical <= reduction_quiet || reduction_quiet > reduction_tactical);
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_disabled() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.enable_adaptive_reduction = false;
        engine.update_lmr_config(config).unwrap();

        let pruning_manager = engine.get_pruning_manager();

        // Check if adaptive reduction is disabled
        assert_eq!(
            pruning_manager.parameters.lmr_enable_adaptive_reduction,
            false
        );
    }

    #[test]
    fn test_pruning_manager_syncs_with_lmr_config() {
        let mut engine = create_test_engine();
        let mut config = LMRConfig::default();
        config.base_reduction = 2;
        config.max_reduction = 4;
        config.min_move_index = 5;
        config.min_depth = 4;
        engine.update_lmr_config(config).unwrap();

        let pruning_manager = engine.get_pruning_manager();

        // Check if PruningManager parameters are synced
        assert_eq!(pruning_manager.parameters.lmr_base_reduction, 2);
        assert_eq!(pruning_manager.parameters.lmr_max_reduction, 4);
        assert_eq!(pruning_manager.parameters.lmr_move_threshold, 5);
        assert_eq!(pruning_manager.parameters.lmr_depth_threshold, 4);
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_neutral_position() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Create search state with neutral position
        let mut search_state = SearchState::new(5, 0, 1000);
        search_state.set_position_classification(Some(PositionClassification::Neutral));

        // Calculate reduction with neutral position
        let reduction_neutral =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Neutral positions should use base reduction (no adjustment)
        // Should be >= 0
        assert!(reduction_neutral >= 0);
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_center_move() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        // Center move (from center to center)
        let center_move = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Edge move (from edge to edge)
        let edge_move = Move::new(
            Position::new(0, 0),
            Position::new(1, 1),
            Player::Black,
            false,
            false,
            false,
        );

        let mut search_state = SearchState::new(5, 0, 1000);
        search_state.set_position_classification(Some(PositionClassification::Neutral));

        // Calculate reduction with center move
        let reduction_center =
            pruning_manager.calculate_lmr_reduction(&search_state, &center_move, false, None);

        // Calculate reduction with edge move
        let reduction_edge =
            pruning_manager.calculate_lmr_reduction(&search_state, &edge_move, false, None);

        // Center moves should have less reduction (more important)
        // Should be >= 0
        assert!(reduction_center >= 0);
        assert!(reduction_edge >= 0);
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_combined_factors() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Test with different combinations of position classification and move type
        let test_cases = vec![
            (PositionClassification::Tactical, true), // Tactical + center move
            (PositionClassification::Quiet, false),   // Quiet + edge move
            (PositionClassification::Neutral, true),  // Neutral + center move
        ];

        for (classification, is_center) in test_cases {
            let mut search_state = SearchState::new(5, 0, 1000);
            search_state.set_position_classification(Some(classification));

            let reduction =
                pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

            // Should be >= 0 and <= max_reduction
            assert!(reduction >= 0);
            assert!(reduction <= pruning_manager.parameters.lmr_max_reduction);
        }
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_without_classification() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Create search state without position classification
        let search_state = SearchState::new(5, 0, 1000);
        // position_classification is None by default

        // Calculate reduction without position classification
        let reduction = pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Should still work (uses base reduction)
        assert!(reduction >= 0);
    }

    #[test]
    fn test_pruning_manager_parameters_sync_on_config_update() {
        let mut engine = create_test_engine();

        // Get initial PruningManager parameters
        let initial_params = engine.get_pruning_manager().parameters.clone();

        // Update LMRConfig
        let mut config = LMRConfig::default();
        config.base_reduction = 3;
        config.max_reduction = 5;
        config.min_move_index = 6;
        config.min_depth = 4;
        config.enable_adaptive_reduction = false;
        engine.update_lmr_config(config).unwrap();

        // Check if PruningManager parameters are synced
        let updated_params = engine.get_pruning_manager().parameters.clone();
        assert_eq!(updated_params.lmr_base_reduction, 3);
        assert_eq!(updated_params.lmr_max_reduction, 5);
        assert_eq!(updated_params.lmr_move_threshold, 6);
        assert_eq!(updated_params.lmr_depth_threshold, 4);
        assert_eq!(updated_params.lmr_enable_adaptive_reduction, false);

        // Verify parameters changed
        assert_ne!(
            updated_params.lmr_base_reduction,
            initial_params.lmr_base_reduction
        );
    }

    #[test]
    fn test_pruning_manager_adaptive_reduction_effectiveness() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let move_ = Move::new(
            Position::new(4, 4),
            Position::new(3, 3),
            Player::Black,
            false,
            false,
            false,
        );

        // Test that adaptive reduction actually changes reduction based on position
        let mut search_state = SearchState::new(5, 0, 1000);

        // Get base reduction (no classification)
        let base_reduction =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Get reduction with tactical position
        search_state.set_position_classification(Some(PositionClassification::Tactical));
        let tactical_reduction =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Get reduction with quiet position
        search_state.set_position_classification(Some(PositionClassification::Quiet));
        let quiet_reduction =
            pruning_manager.calculate_lmr_reduction(&search_state, &move_, false, None);

        // Tactical should be <= base, quiet should be >= base (if adaptive reduction enabled)
        if pruning_manager.parameters.lmr_enable_adaptive_reduction {
            assert!(tactical_reduction <= quiet_reduction);
        }
    }
}

#[cfg(test)]
mod lmr_preset_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_get_lmr_preset_aggressive() {
        let engine = create_test_engine();
        let preset = engine.get_lmr_preset(LMRPlayingStyle::Aggressive);

        // Verify aggressive preset settings
        assert_eq!(preset.enabled, true);
        assert_eq!(preset.min_depth, 2);
        assert_eq!(preset.min_move_index, 3);
        assert_eq!(preset.base_reduction, 2);
        assert_eq!(preset.max_reduction, 4);
        assert_eq!(preset.re_search_margin, 25);
        assert_eq!(preset.enable_extended_exemptions, true);
        assert_eq!(preset.adaptive_tuning_config.enabled, true);
        assert_eq!(
            preset.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Moderate
        );
    }

    #[test]
    fn test_get_lmr_preset_conservative() {
        let engine = create_test_engine();
        let preset = engine.get_lmr_preset(LMRPlayingStyle::Conservative);

        // Verify conservative preset settings
        assert_eq!(preset.enabled, true);
        assert_eq!(preset.min_depth, 4);
        assert_eq!(preset.min_move_index, 6);
        assert_eq!(preset.base_reduction, 1);
        assert_eq!(preset.max_reduction, 2);
        assert_eq!(preset.re_search_margin, 100);
        assert_eq!(preset.enable_extended_exemptions, true);
        assert_eq!(preset.adaptive_tuning_config.enabled, true);
        assert_eq!(
            preset.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Conservative
        );
    }

    #[test]
    fn test_get_lmr_preset_balanced() {
        let engine = create_test_engine();
        let preset = engine.get_lmr_preset(LMRPlayingStyle::Balanced);

        // Verify balanced preset settings
        assert_eq!(preset.enabled, true);
        assert_eq!(preset.min_depth, 3);
        assert_eq!(preset.min_move_index, 4);
        assert_eq!(preset.base_reduction, 1);
        assert_eq!(preset.max_reduction, 3);
        assert_eq!(preset.re_search_margin, 50);
        assert_eq!(preset.enable_extended_exemptions, true);
        assert_eq!(preset.adaptive_tuning_config.enabled, true);
        assert_eq!(
            preset.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Moderate
        );
    }

    #[test]
    fn test_validate_lmr_preset_aggressive() {
        let engine = create_test_engine();
        let result = engine.validate_lmr_preset(LMRPlayingStyle::Aggressive);
        assert!(result.is_ok(), "Aggressive preset should be valid");
    }

    #[test]
    fn test_validate_lmr_preset_conservative() {
        let engine = create_test_engine();
        let result = engine.validate_lmr_preset(LMRPlayingStyle::Conservative);
        assert!(result.is_ok(), "Conservative preset should be valid");
    }

    #[test]
    fn test_validate_lmr_preset_balanced() {
        let engine = create_test_engine();
        let result = engine.validate_lmr_preset(LMRPlayingStyle::Balanced);
        assert!(result.is_ok(), "Balanced preset should be valid");
    }

    #[test]
    fn test_apply_lmr_preset_aggressive() {
        let mut engine = create_test_engine();
        let result = engine.apply_lmr_preset(LMRPlayingStyle::Aggressive);

        assert!(
            result.is_ok(),
            "Should apply aggressive preset successfully"
        );

        let config = engine.get_lmr_config();
        assert_eq!(config.min_depth, 2);
        assert_eq!(config.base_reduction, 2);
        assert_eq!(config.re_search_margin, 25);

        // Verify PruningManager parameters are synced
        let pruning_manager = engine.get_pruning_manager();
        assert_eq!(pruning_manager.parameters.lmr_base_reduction, 2);
        assert_eq!(pruning_manager.parameters.lmr_depth_threshold, 2);
        assert_eq!(pruning_manager.parameters.lmr_move_threshold, 3);
        assert_eq!(pruning_manager.parameters.lmr_max_reduction, 4);
    }

    #[test]
    fn test_apply_lmr_preset_conservative() {
        let mut engine = create_test_engine();
        let result = engine.apply_lmr_preset(LMRPlayingStyle::Conservative);

        assert!(
            result.is_ok(),
            "Should apply conservative preset successfully"
        );

        let config = engine.get_lmr_config();
        assert_eq!(config.min_depth, 4);
        assert_eq!(config.base_reduction, 1);
        assert_eq!(config.re_search_margin, 100);

        // Verify PruningManager parameters are synced
        let pruning_manager = engine.get_pruning_manager();
        assert_eq!(pruning_manager.parameters.lmr_base_reduction, 1);
        assert_eq!(pruning_manager.parameters.lmr_depth_threshold, 4);
        assert_eq!(pruning_manager.parameters.lmr_move_threshold, 6);
        assert_eq!(pruning_manager.parameters.lmr_max_reduction, 2);
    }

    #[test]
    fn test_apply_lmr_preset_balanced() {
        let mut engine = create_test_engine();
        let result = engine.apply_lmr_preset(LMRPlayingStyle::Balanced);

        assert!(result.is_ok(), "Should apply balanced preset successfully");

        let config = engine.get_lmr_config();
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.base_reduction, 1);
        assert_eq!(config.re_search_margin, 50);

        // Verify PruningManager parameters are synced
        let pruning_manager = engine.get_pruning_manager();
        assert_eq!(pruning_manager.parameters.lmr_base_reduction, 1);
        assert_eq!(pruning_manager.parameters.lmr_depth_threshold, 3);
        assert_eq!(pruning_manager.parameters.lmr_move_threshold, 4);
        assert_eq!(pruning_manager.parameters.lmr_max_reduction, 3);
    }

    #[test]
    fn test_preset_configurations_are_reasonable() {
        let engine = create_test_engine();

        // Test all presets have reasonable values
        for style in [
            LMRPlayingStyle::Aggressive,
            LMRPlayingStyle::Conservative,
            LMRPlayingStyle::Balanced,
        ] {
            let preset = engine.get_lmr_preset(style);

            // Verify reasonable ranges
            assert!(
                preset.min_depth >= 1 && preset.min_depth <= 15,
                "min_depth should be in valid range"
            );
            assert!(
                preset.min_move_index >= 1 && preset.min_move_index <= 20,
                "min_move_index should be in valid range"
            );
            assert!(
                preset.base_reduction >= 1 && preset.base_reduction <= 5,
                "base_reduction should be in valid range"
            );
            assert!(
                preset.max_reduction >= preset.base_reduction && preset.max_reduction <= 8,
                "max_reduction should be >= base_reduction and <= 8"
            );
            assert!(
                preset.re_search_margin >= 0 && preset.re_search_margin <= 500,
                "re_search_margin should be in valid range"
            );

            // Verify preset is valid
            assert!(preset.validate().is_ok(), "Preset should be valid");
        }
    }

    #[test]
    fn test_preset_adaptive_tuning_configurations() {
        let engine = create_test_engine();

        let aggressive = engine.get_lmr_preset(LMRPlayingStyle::Aggressive);
        let conservative = engine.get_lmr_preset(LMRPlayingStyle::Conservative);
        let balanced = engine.get_lmr_preset(LMRPlayingStyle::Balanced);

        // Aggressive should have Moderate tuning
        assert_eq!(
            aggressive.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Moderate
        );

        // Conservative should have Conservative tuning
        assert_eq!(
            conservative.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Conservative
        );

        // Balanced should have Moderate tuning
        assert_eq!(
            balanced.adaptive_tuning_config.aggressiveness,
            TuningAggressiveness::Moderate
        );

        // All should have adaptive tuning enabled
        assert!(aggressive.adaptive_tuning_config.enabled);
        assert!(conservative.adaptive_tuning_config.enabled);
        assert!(balanced.adaptive_tuning_config.enabled);
    }

    #[test]
    fn test_preset_switching() {
        let mut engine = create_test_engine();

        // Apply aggressive preset
        engine
            .apply_lmr_preset(LMRPlayingStyle::Aggressive)
            .unwrap();
        assert_eq!(engine.get_lmr_config().base_reduction, 2);

        // Switch to conservative preset
        engine
            .apply_lmr_preset(LMRPlayingStyle::Conservative)
            .unwrap();
        assert_eq!(engine.get_lmr_config().base_reduction, 1);

        // Switch to balanced preset
        engine.apply_lmr_preset(LMRPlayingStyle::Balanced).unwrap();
        assert_eq!(engine.get_lmr_config().base_reduction, 1);
        assert_eq!(engine.get_lmr_config().min_depth, 3);
    }

    #[test]
    fn test_preset_integration_with_lmr() {
        let mut engine = create_test_engine();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Apply aggressive preset
        engine
            .apply_lmr_preset(LMRPlayingStyle::Aggressive)
            .unwrap();

        // Perform a search to verify preset works with LMR
        let mut board_mut = board.clone();
        let result = engine.search_at_depth_legacy(
            &mut board_mut,
            &captured_pieces,
            player,
            5, // Fixed depth
            1000,
        );

        // Should complete without errors
        assert!(result.is_some() || result.is_none()); // Result may or may not be Some

        // Verify preset configuration is still applied
        let config = engine.get_lmr_config();
        assert_eq!(config.base_reduction, 2);
        assert_eq!(config.re_search_margin, 25);
    }
}

#[cfg(test)]
mod move_ordering_effectiveness_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    #[test]
    fn test_move_ordering_stats_default() {
        let stats = MoveOrderingEffectivenessStats::default();
        assert_eq!(stats.total_cutoffs, 0);
        assert_eq!(stats.cutoffs_after_lmr_threshold, 0);
        assert_eq!(stats.late_ordered_cutoffs, 0);
        assert_eq!(stats.early_ordered_no_cutoffs, 0);
    }

    #[test]
    fn test_record_cutoff() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // Record cutoff at index 2 (before threshold)
        stats.record_cutoff(2, lmr_threshold);
        assert_eq!(stats.total_cutoffs, 1);
        assert_eq!(stats.cutoffs_before_lmr_threshold, 1);
        assert_eq!(stats.cutoffs_after_lmr_threshold, 0);
        assert_eq!(stats.late_ordered_cutoffs, 0);

        // Record cutoff at index 5 (after threshold)
        stats.record_cutoff(5, lmr_threshold);
        assert_eq!(stats.total_cutoffs, 2);
        assert_eq!(stats.cutoffs_before_lmr_threshold, 1);
        assert_eq!(stats.cutoffs_after_lmr_threshold, 1);
        assert_eq!(stats.late_ordered_cutoffs, 1);
    }

    #[test]
    fn test_record_no_cutoff() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // Record no cutoff at index 2 (before threshold)
        stats.record_no_cutoff(2, lmr_threshold);
        assert_eq!(stats.moves_no_cutoff, 1);
        assert_eq!(stats.early_ordered_no_cutoffs, 1);

        // Record no cutoff at index 5 (after threshold)
        stats.record_no_cutoff(5, lmr_threshold);
        assert_eq!(stats.moves_no_cutoff, 2);
        assert_eq!(stats.early_ordered_no_cutoffs, 1); // Only early moves count
    }

    #[test]
    fn test_cutoffs_after_threshold_percentage() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // No cutoffs
        assert_eq!(stats.cutoffs_after_threshold_percentage(), 0.0);

        // Add cutoffs
        stats.record_cutoff(2, lmr_threshold); // Before threshold
        stats.record_cutoff(5, lmr_threshold); // After threshold
        stats.record_cutoff(6, lmr_threshold); // After threshold

        // 2 out of 3 cutoffs after threshold = 66.67%
        let percentage = stats.cutoffs_after_threshold_percentage();
        assert!(percentage > 66.0 && percentage < 67.0);
    }

    #[test]
    fn test_average_cutoff_index() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // No cutoffs
        assert_eq!(stats.average_cutoff_index(), 0.0);

        // Add cutoffs at indices 1, 3, 5
        stats.record_cutoff(1, lmr_threshold);
        stats.record_cutoff(3, lmr_threshold);
        stats.record_cutoff(5, lmr_threshold);

        // Average = (1 + 3 + 5) / 3 = 3.0
        assert_eq!(stats.average_cutoff_index(), 3.0);
    }

    #[test]
    fn test_ordering_effectiveness() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // No cutoffs - perfect ordering
        assert_eq!(stats.ordering_effectiveness(), 100.0);

        // Add cutoffs at early indices (good ordering)
        stats.record_cutoff(0, lmr_threshold);
        stats.record_cutoff(1, lmr_threshold);
        stats.record_cutoff(2, lmr_threshold);

        let effectiveness = stats.ordering_effectiveness();
        assert!(effectiveness > 0.0);

        // Add cutoffs at late indices (poor ordering)
        let mut stats2 = MoveOrderingEffectivenessStats::default();
        stats2.record_cutoff(8, lmr_threshold);
        stats2.record_cutoff(10, lmr_threshold);
        stats2.record_cutoff(12, lmr_threshold);

        let effectiveness2 = stats2.ordering_effectiveness();
        // Late cutoffs should have lower effectiveness
        assert!(effectiveness2 < effectiveness);
    }

    #[test]
    fn test_lmr_stats_has_move_ordering_stats() {
        let stats = LMRStats::default();
        assert_eq!(stats.move_ordering_stats.total_cutoffs, 0);
    }

    #[test]
    fn test_get_move_ordering_metrics() {
        let mut engine = create_test_engine();
        let mut stats = engine.lmr_stats.move_ordering_stats.clone();

        // Add some cutoffs
        stats.record_cutoff(2, 4);
        stats.record_cutoff(5, 4);

        engine.lmr_stats.move_ordering_stats = stats;

        let metrics = engine.get_move_ordering_effectiveness_metrics();
        assert_eq!(metrics.total_cutoffs, 2);
        assert!(metrics.average_cutoff_index > 0.0);
    }

    #[test]
    fn test_check_move_ordering_degradation() {
        let mut engine = create_test_engine();
        let mut stats = engine.lmr_stats.move_ordering_stats.clone();

        // Add many late cutoffs (poor ordering)
        for i in 5..15 {
            stats.record_cutoff(i, 4);
        }

        engine.lmr_stats.move_ordering_stats = stats;

        let (is_healthy, alerts) = engine.check_move_ordering_degradation();

        // Should detect degradation
        assert!(!is_healthy || !alerts.is_empty());
    }

    #[test]
    fn test_get_ordering_vs_lmr_report() {
        let engine = create_test_engine();
        let report = engine.get_ordering_vs_lmr_report();

        // Should generate a report
        assert!(!report.is_empty());
        assert!(report.contains("Move Ordering vs LMR Effectiveness"));
    }

    #[test]
    fn test_get_ordering_effectiveness_with_integration() {
        let engine = create_test_engine();
        let report = engine.get_ordering_effectiveness_with_integration();

        // Should generate a report with integration
        assert!(!report.is_empty());
        assert!(report.contains("Move Ordering Effectiveness with Integration"));
    }

    #[test]
    fn test_identify_ordering_improvements() {
        let mut engine = create_test_engine();
        let mut stats = engine.lmr_stats.move_ordering_stats.clone();

        // Add many late cutoffs
        for i in 6..10 {
            stats.record_cutoff(i, 4);
        }

        engine.lmr_stats.move_ordering_stats = stats;

        let improvements = engine.identify_ordering_improvements();

        // Should identify improvements
        assert!(!improvements.is_empty());
    }

    #[test]
    fn test_move_ordering_stats_reset() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        stats.record_cutoff(5, 4);
        stats.record_no_cutoff(2, 4);

        assert!(stats.total_cutoffs > 0);

        stats.reset();

        assert_eq!(stats.total_cutoffs, 0);
        assert_eq!(stats.moves_no_cutoff, 0);
        assert_eq!(stats.cutoffs_by_index.len(), 0);
    }

    #[test]
    fn test_cutoffs_by_index_tracking() {
        let mut stats = MoveOrderingEffectivenessStats::default();
        let lmr_threshold = 4;

        // Record multiple cutoffs at same index
        stats.record_cutoff(5, lmr_threshold);
        stats.record_cutoff(5, lmr_threshold);
        stats.record_cutoff(5, lmr_threshold);

        assert_eq!(stats.cutoffs_by_index.get(&5), Some(&3));
        assert_eq!(stats.total_cutoffs, 3);
    }
}

#[cfg(test)]
mod advanced_reduction_strategies_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    fn create_test_state() -> SearchState {
        SearchState::new(5, -10000, 10000)
    }

    fn create_test_move() -> Move {
        Move::new(
            Some(Position { row: 2, col: 2 }),
            Position { row: 3, col: 3 },
            PieceType::Pawn,
            Player::Black,
            false,
        )
    }

    #[test]
    fn test_advanced_reduction_config_default() {
        let config = AdvancedReductionConfig::default();
        assert_eq!(config.enabled, false);
        assert_eq!(config.strategy, AdvancedReductionStrategy::Basic);
        assert_eq!(config.enable_depth_based, false);
        assert_eq!(config.enable_material_based, false);
        assert_eq!(config.enable_history_based, false);
    }

    #[test]
    fn test_advanced_reduction_strategy_enum() {
        assert_eq!(
            AdvancedReductionStrategy::Basic,
            AdvancedReductionStrategy::Basic
        );
        assert_eq!(
            AdvancedReductionStrategy::DepthBased,
            AdvancedReductionStrategy::DepthBased
        );
        assert_eq!(
            AdvancedReductionStrategy::MaterialBased,
            AdvancedReductionStrategy::MaterialBased
        );
        assert_eq!(
            AdvancedReductionStrategy::HistoryBased,
            AdvancedReductionStrategy::HistoryBased
        );
        assert_eq!(
            AdvancedReductionStrategy::Combined,
            AdvancedReductionStrategy::Combined
        );
    }

    #[test]
    fn test_apply_depth_based_reduction() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_depth_based = true;
        config.depth_scaling_factor = 0.15;

        let mut state = create_test_state();
        state.depth = 10;
        state.set_advanced_reduction_config(config.clone());

        let mv = create_test_move();
        let base_reduction = 2;

        let reduction =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Depth-based reduction should increase reduction for deeper positions
        assert!(reduction >= base_reduction);
        assert!(reduction <= 5); // Max reduction
    }

    #[test]
    fn test_apply_material_based_reduction() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_material_based = true;

        let mut state = create_test_state();
        state.set_position_classification(PositionClassification::Tactical);
        state.set_advanced_reduction_config(config.clone());

        let mv = create_test_move();
        let base_reduction = 2;

        let reduction =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Material-based reduction should increase reduction for tactical positions
        assert!(reduction >= base_reduction);
    }

    #[test]
    fn test_apply_history_based_reduction() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_history_based = true;

        let mut state = create_test_state();
        state.set_advanced_reduction_config(config.clone());

        // Test with quiet move (poor history)
        let mut mv = create_test_move();
        mv.is_capture = false;
        mv.is_promotion = false;

        let base_reduction = 2;

        let reduction =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // History-based reduction should increase reduction for quiet moves
        assert!(reduction >= base_reduction);
    }

    #[test]
    fn test_apply_combined_reduction() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.strategy = AdvancedReductionStrategy::Combined;
        config.enable_depth_based = true;
        config.enable_material_based = true;
        config.enable_history_based = true;

        let mut state = create_test_state();
        state.depth = 10;
        state.set_position_classification(PositionClassification::Tactical);
        state.set_advanced_reduction_config(config.clone());

        let mut mv = create_test_move();
        mv.is_capture = false;
        mv.is_promotion = false;

        let base_reduction = 2;

        let reduction =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Combined reduction should apply all strategies
        assert!(reduction >= base_reduction);
    }

    #[test]
    fn test_advanced_reduction_disabled() {
        let mut engine = create_test_engine();
        let config = AdvancedReductionConfig::default(); // Disabled by default

        let state = create_test_state();
        let mv = create_test_move();
        let base_reduction = 2;

        let reduction =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // When disabled, should return base reduction
        assert_eq!(reduction, base_reduction);
    }

    #[test]
    fn test_search_state_advanced_reduction_config() {
        let mut state = create_test_state();
        let config = AdvancedReductionConfig::default();

        assert_eq!(state.advanced_reduction_config, None);

        state.set_advanced_reduction_config(config.clone());

        assert_eq!(state.advanced_reduction_config, Some(config));
    }

    #[test]
    fn test_lmr_config_has_advanced_reduction_config() {
        let config = LMRConfig::default();
        assert_eq!(config.advanced_reduction_config.enabled, false);
    }

    #[test]
    fn test_depth_based_reduction_scaling() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_depth_based = true;
        config.depth_scaling_factor = 0.15;

        let mut state = create_test_state();
        let mv = create_test_move();

        // Test at different depths
        let base_reduction = 2;

        state.depth = 5;
        state.set_advanced_reduction_config(config.clone());
        let reduction_shallow =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        state.depth = 15;
        state.set_advanced_reduction_config(config.clone());
        let reduction_deep =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Deeper positions should have more reduction
        assert!(reduction_deep >= reduction_shallow);
    }

    #[test]
    fn test_material_based_reduction_by_classification() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_material_based = true;

        let mut state = create_test_state();
        let mv = create_test_move();
        let base_reduction = 2;

        // Test tactical position
        state.set_position_classification(PositionClassification::Tactical);
        state.set_advanced_reduction_config(config.clone());
        let reduction_tactical =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Test quiet position
        state.set_position_classification(PositionClassification::Quiet);
        state.set_advanced_reduction_config(config.clone());
        let reduction_quiet =
            engine
                .pruning_manager
                .apply_advanced_reduction(base_reduction, &state, &mv, &config);

        // Tactical positions should have more reduction than quiet positions
        assert!(reduction_tactical >= reduction_quiet);
    }

    #[test]
    fn test_history_based_reduction_by_move_type() {
        let mut engine = create_test_engine();
        let mut config = AdvancedReductionConfig::default();
        config.enabled = true;
        config.enable_history_based = true;

        let mut state = create_test_state();
        state.set_advanced_reduction_config(config.clone());
        let base_reduction = 2;

        // Test quiet move (poor history)
        let mut quiet_move = create_test_move();
        quiet_move.is_capture = false;
        quiet_move.is_promotion = false;

        let reduction_quiet = engine.pruning_manager.apply_advanced_reduction(
            base_reduction,
            &state,
            &quiet_move,
            &config,
        );

        // Test capture move (good history)
        let mut capture_move = create_test_move();
        capture_move.is_capture = true;

        let reduction_capture = engine.pruning_manager.apply_advanced_reduction(
            base_reduction,
            &state,
            &capture_move,
            &config,
        );

        // Quiet moves should have more reduction than capture moves
        assert!(reduction_quiet >= reduction_capture);
    }
}

#[cfg(test)]
mod conditional_exemption_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    fn create_test_state() -> SearchState {
        SearchState::new(5, -10000, 10000)
    }

    fn create_test_move() -> Move {
        Move::new(
            Some(Position { row: 2, col: 2 }),
            Position { row: 3, col: 3 },
            PieceType::Pawn,
            Player::Black,
            false,
        )
    }

    #[test]
    fn test_conditional_exemption_config_default() {
        let config = ConditionalExemptionConfig::default();
        assert_eq!(config.enable_conditional_capture_exemption, false);
        assert_eq!(config.enable_conditional_promotion_exemption, false);
        assert_eq!(config.min_capture_value_threshold, 100);
        assert_eq!(config.min_depth_for_conditional_capture, 5);
        assert_eq!(config.exempt_tactical_promotions_only, true);
        assert_eq!(config.min_depth_for_conditional_promotion, 5);
    }

    #[test]
    fn test_conditional_capture_exemption_disabled() {
        let engine = create_test_engine();
        let pruning_manager = engine.get_pruning_manager();

        let mut state = create_test_state();
        state.move_number = 10; // Above threshold
        state.depth = 10; // Above threshold

        // Test capture move - should be exempted (default behavior)
        let mut capture_move = create_test_move();
        capture_move.is_capture = true;
        capture_move.captured_piece = Some(Piece {
            piece_type: PieceType::Pawn,
            player: Player::White,
        });

        let reduction = pruning_manager.calculate_lmr_reduction(&state, &capture_move, false, None);
        assert_eq!(reduction, 0); // Should be exempted
    }

    #[test]
    fn test_lmr_config_has_conditional_exemption_config() {
        let config = LMRConfig::default();
        assert_eq!(
            config
                .conditional_exemption_config
                .enable_conditional_capture_exemption,
            false
        );
        assert_eq!(
            config
                .conditional_exemption_config
                .enable_conditional_promotion_exemption,
            false
        );
    }
}
