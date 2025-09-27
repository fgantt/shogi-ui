/// Comprehensive test suite for Late Move Reductions (LMR)
/// 
/// This module contains unit tests for all LMR functionality including:
/// - Configuration validation and management
/// - Statistics tracking and calculations
/// - Move exemption rules and classification
/// - Reduction calculation algorithms
/// - Adaptive reduction logic

use shogi_engine::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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
            min_depth: 0,        // Invalid, should be clamped to 1
            min_move_index: 25,  // Invalid, should be clamped to 20
            base_reduction: 0,   // Invalid, should be clamped to 1
            max_reduction: 10,   // Invalid, should be clamped to 8
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
        };
        
        stats.reset();
        assert_eq!(stats.moves_considered, 0);
        assert_eq!(stats.reductions_applied, 0);
        assert_eq!(stats.researches_triggered, 0);
        assert_eq!(stats.cutoffs_after_reduction, 0);
        assert_eq!(stats.cutoffs_after_research, 0);
        assert_eq!(stats.total_depth_saved, 0);
        assert_eq!(stats.average_reduction, 0.0);
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
            captured_piece: if is_capture { Some(Piece { piece_type: PieceType::Pawn, player: Player::White }) } else { None },
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
            captured_piece: Some(Piece { piece_type: PieceType::Pawn, player: Player::White }),
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
        assert_eq!(engine.classify_move_type(&promotion_move), MoveType::Promotion);
        
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
            captured_piece: Some(Piece { piece_type: PieceType::Rook, player: Player::White }),
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
