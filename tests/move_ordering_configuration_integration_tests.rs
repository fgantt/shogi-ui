#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{
    CacheConfig, DebugConfig, HistoryConfig, KillerConfig, MoveOrdering, MoveOrderingConfig,
    OrderingWeights, PerformanceConfig,
};

/// Integration tests for move ordering configuration system
///
/// These tests verify that the configuration system integrates correctly
/// with the overall move ordering system and provides the expected functionality.

#[cfg(test)]
mod move_ordering_configuration_integration_tests {
    use super::*;

    /// Create a test move for testing purposes
    fn create_test_move(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Move {
        Move {
            from,
            to,
            piece_type,
            player,
            promotion: false,
            drop: from.is_none(),
        }
    }

    /// Test comprehensive configuration integration
    #[test]
    fn test_comprehensive_configuration_integration() {
        let mut config = MoveOrderingConfig::new();

        // Customize all configuration aspects
        config.weights.capture_weight = 2000;
        config.weights.promotion_weight = 1600;
        config.weights.tactical_weight = 600;

        config.cache_config.max_cache_size = 500;
        config.cache_config.enable_cache_warming = true;

        config.killer_config.max_killer_moves_per_depth = 3;
        config.killer_config.enable_depth_based_management = true;

        config.history_config.max_history_score = 15000;
        config.history_config.enable_automatic_aging = true;

        config.performance_config.enable_performance_monitoring = true;
        config.debug_config.enable_debug_logging = true;

        // Validate configuration
        assert!(config.validate().is_ok());

        // Create move orderer with custom configuration
        let mut orderer = MoveOrdering::with_config(config);

        // Verify configuration is applied
        assert_eq!(orderer.get_weights().capture_weight, 2000);
        assert_eq!(orderer.get_weights().promotion_weight, 1600);
        assert_eq!(orderer.get_max_cache_size(), 500);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 3);
        assert_eq!(orderer.get_max_history_score(), 15000);
    }

    /// Test configuration validation integration
    #[test]
    fn test_configuration_validation_integration() {
        let mut config = MoveOrderingConfig::new();

        // Test valid configuration
        assert!(config.validate().is_ok());

        // Test invalid weight
        config.weights.capture_weight = -100;
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("Capture weight")));
        }

        // Test invalid cache size
        config.weights.capture_weight = 1000; // Fix previous error
        config.cache_config.max_cache_size = 0;
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("Max cache size")));
        }

        // Test invalid aging factor
        config.cache_config.max_cache_size = 1000; // Fix previous error
        config.history_config.history_aging_factor = 1.5; // Invalid
        let result = config.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("History aging factor")));
        }
    }

    /// Test specialized configuration presets
    #[test]
    fn test_specialized_configuration_presets() {
        // Test performance optimized configuration
        let perf_config = MoveOrderingConfig::performance_optimized();
        assert_eq!(perf_config.cache_config.max_cache_size, 5000);
        assert!(perf_config.cache_config.enable_cache_warming);
        assert_eq!(perf_config.killer_config.max_killer_moves_per_depth, 3);
        assert!(!perf_config.debug_config.enable_debug_logging);

        // Test debug optimized configuration
        let debug_config = MoveOrderingConfig::debug_optimized();
        assert_eq!(debug_config.cache_config.max_cache_size, 500);
        assert!(!debug_config.cache_config.enable_cache_warming);
        assert_eq!(debug_config.killer_config.max_killer_moves_per_depth, 1);
        assert!(debug_config.debug_config.enable_debug_logging);
        assert_eq!(debug_config.debug_config.log_level, 3);

        // Test memory optimized configuration
        let memory_config = MoveOrderingConfig::memory_optimized();
        assert_eq!(memory_config.cache_config.max_cache_size, 100);
        assert!(!memory_config.cache_config.enable_cache_warming);
        assert_eq!(memory_config.killer_config.max_killer_moves_per_depth, 1);
        assert_eq!(memory_config.history_config.max_history_score, 1000);
        assert!(!memory_config.debug_config.enable_debug_logging);
    }

    /// Test configuration merging
    #[test]
    fn test_configuration_merging() {
        let base_config = MoveOrderingConfig::new();
        let mut override_config = MoveOrderingConfig::new();

        // Override specific settings
        override_config.weights.capture_weight = 3000;
        override_config.cache_config.max_cache_size = 2000;
        override_config.killer_config.max_killer_moves_per_depth = 4;

        let merged_config = base_config.merge(&override_config);

        // Should use override values
        assert_eq!(merged_config.weights.capture_weight, 3000);
        assert_eq!(merged_config.cache_config.max_cache_size, 2000);
        assert_eq!(merged_config.killer_config.max_killer_moves_per_depth, 4);

        // Should keep default values for non-overridden settings
        assert_eq!(merged_config.weights.promotion_weight, 800);
        assert_eq!(merged_config.history_config.max_history_score, 10000);
    }

    /// Test runtime configuration updates
    #[test]
    fn test_runtime_configuration_updates() {
        let mut orderer = MoveOrdering::new();

        // Test weight updates
        orderer.set_capture_weight(2500);
        orderer.set_promotion_weight(2000);
        orderer.set_tactical_weight(800);

        assert_eq!(orderer.get_weights().capture_weight, 2500);
        assert_eq!(orderer.get_weights().promotion_weight, 2000);
        assert_eq!(orderer.get_weights().tactical_weight, 800);

        // Test cache configuration updates
        orderer.set_cache_size(1500);
        assert_eq!(orderer.get_max_cache_size(), 1500);

        // Test killer move configuration updates
        orderer.set_max_killer_moves_per_depth(5);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 5);

        // Test history configuration updates
        orderer.set_max_history_score(20000);
        assert_eq!(orderer.get_max_history_score(), 20000);

        orderer.set_history_aging_factor(0.8);
        assert_eq!(orderer.get_history_aging_factor(), 0.8);
    }

    /// Test configuration with move scoring
    #[test]
    fn test_configuration_with_move_scoring() {
        let mut config = MoveOrderingConfig::new();
        config.weights.capture_weight = 5000;
        config.weights.promotion_weight = 4000;
        config.weights.tactical_weight = 1000;

        let mut orderer = MoveOrdering::with_config(config);

        // Test capture move scoring
        let mut capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        capture_move.is_capture = true;

        let capture_score = orderer.score_move(&capture_move);
        assert!(capture_score >= 5000);

        // Test promotion move scoring
        let mut promotion_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Pawn,
            Player::Black,
        );
        promotion_move.is_promotion = true;

        let promotion_score = orderer.score_move(&promotion_move);
        assert!(promotion_score >= 4000);

        // Test tactical move scoring
        let mut tactical_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Silver,
            Player::Black,
        );
        tactical_move.gives_check = true;

        let tactical_score = orderer.score_move(&tactical_move);
        assert!(tactical_score >= 1000);
    }

    /// Test configuration with cache management
    #[test]
    fn test_configuration_with_cache_management() {
        let mut config = MoveOrderingConfig::new();
        config.cache_config.max_cache_size = 100;
        config.cache_config.enable_cache_warming = true;
        config.cache_config.cache_warming_ratio = 0.5;

        let mut orderer = MoveOrdering::with_config(config);

        // Fill cache beyond limit
        let moves = (0..150)
            .map(|i| {
                create_test_move(
                    Some(Position::new(i % 9, i / 9)),
                    Position::new((i + 1) % 9, (i + 1) / 9),
                    PieceType::Pawn,
                    Player::Black,
                )
            })
            .collect::<Vec<_>>();

        for move_ in &moves {
            orderer.score_move(move_);
        }

        // Cache should be limited to configured size
        assert!(orderer.get_cache_size() <= 100);

        // Test cache warming
        orderer.warm_up_cache(&moves[0..50]);
        assert!(orderer.get_cache_size() > 0);
    }

    /// Test configuration with killer moves
    #[test]
    fn test_configuration_with_killer_moves() {
        let mut config = MoveOrderingConfig::new();
        config.killer_config.max_killer_moves_per_depth = 3;
        config.killer_config.enable_depth_based_management = true;

        let mut orderer = MoveOrdering::with_config(config);
        orderer.set_current_depth(2);

        // Add killer moves
        let killer_moves = vec![
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(3, 3)),
                Position::new(4, 3),
                PieceType::Gold,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(4, 4)),
                Position::new(5, 4),
                PieceType::Bishop,
                Player::Black,
            ),
        ];

        for killer_move in killer_moves {
            orderer.add_killer_move(killer_move);
        }

        // Should be limited to configured number
        let current_killer_moves = orderer.get_current_killer_moves();
        assert!(current_killer_moves.is_some());
        assert!(current_killer_moves.unwrap().len() <= 3);
    }

    /// Test configuration with history heuristic
    #[test]
    fn test_configuration_with_history_heuristic() {
        let mut config = MoveOrderingConfig::new();
        config.history_config.max_history_score = 5000;
        config.history_config.history_aging_factor = 0.8;
        config.history_config.enable_automatic_aging = true;

        let mut orderer = MoveOrdering::with_config(config);

        // Update history scores
        let history_moves = vec![
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                Player::Black,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                Player::Black,
            ),
        ];

        for move_ in &history_moves {
            orderer.update_history_score(move_, 3);
        }

        // Test history scoring
        for move_ in &history_moves {
            let history_score = orderer.get_history_score(move_);
            assert!(history_score > 0);
            assert!(history_score <= 5000); // Should respect max score
        }

        // Test aging
        orderer.age_history_table();

        // Scores should be reduced
        for move_ in &history_moves {
            let aged_score = orderer.get_history_score(move_);
            assert!(aged_score < 5000); // Should be aged down
        }
    }

    /// Test configuration error handling
    #[test]
    fn test_configuration_error_handling() {
        let mut invalid_config = MoveOrderingConfig::new();
        invalid_config.weights.capture_weight = -1;

        // Should fail validation
        assert!(invalid_config.validate().is_err());

        // Should fail when setting invalid configuration
        let mut orderer = MoveOrdering::new();
        let result = orderer.set_config(invalid_config);
        assert!(result.is_err());

        // Configuration should remain unchanged
        assert_eq!(orderer.get_weights().capture_weight, 1000); // Default value
    }

    /// Test configuration reset functionality
    #[test]
    fn test_configuration_reset() {
        let mut orderer = MoveOrdering::new();

        // Modify configuration
        orderer.set_capture_weight(5000);
        orderer.set_promotion_weight(4000);
        orderer.set_cache_size(2000);
        orderer.set_max_killer_moves_per_depth(5);
        orderer.set_max_history_score(25000);

        // Verify modifications
        assert_eq!(orderer.get_weights().capture_weight, 5000);
        assert_eq!(orderer.get_weights().promotion_weight, 4000);
        assert_eq!(orderer.get_max_cache_size(), 2000);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 5);
        assert_eq!(orderer.get_max_history_score(), 25000);

        // Reset to defaults
        orderer.reset_config_to_default();

        // Should be back to defaults
        assert_eq!(orderer.get_weights().capture_weight, 1000);
        assert_eq!(orderer.get_weights().promotion_weight, 800);
        assert_eq!(orderer.get_max_cache_size(), 1000);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 2);
        assert_eq!(orderer.get_max_history_score(), 10000);
    }

    /// Test configuration with performance optimization
    #[test]
    fn test_configuration_with_performance_optimization() {
        let mut config = MoveOrderingConfig::new();
        config.performance_config.enable_auto_optimization = true;
        config.cache_config.enable_auto_optimization = true;

        let mut orderer = MoveOrdering::with_config(config);

        // Generate some moves to populate cache
        let moves = (0..200)
            .map(|i| {
                create_test_move(
                    Some(Position::new(i % 9, i / 9)),
                    Position::new((i + 1) % 9, (i + 1) / 9),
                    PieceType::Pawn,
                    Player::Black,
                )
            })
            .collect::<Vec<_>>();

        // Score moves to populate cache
        for move_ in &moves {
            orderer.score_move(move_);
        }

        // Optimize performance
        orderer.optimize_performance();

        // Should still be functional
        assert!(orderer.get_max_cache_size() > 0);
        assert!(orderer.get_cache_size() >= 0);
    }

    /// Test configuration with all heuristics combined
    #[test]
    fn test_configuration_with_all_heuristics() {
        let mut config = MoveOrderingConfig::new();
        config.weights.capture_weight = 3000;
        config.weights.promotion_weight = 2500;
        config.weights.killer_move_weight = 8000;
        config.weights.history_weight = 4000;

        let mut orderer = MoveOrdering::with_config(config);
        orderer.set_current_depth(3);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create moves
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let killer_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );
        let history_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );
        let regular_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black,
        );

        // Set up heuristics
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );
        orderer.add_killer_move(killer_move.clone());
        orderer.update_history_score(&history_move, 3);

        // Test scoring with all heuristics
        // Task 3.0: Updated calls to include IID move parameter (None for tests)
        let pv_score = orderer.score_move_with_all_heuristics(
            &pv_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let killer_score = orderer.score_move_with_all_heuristics(
            &killer_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let history_score = orderer.score_move_with_all_heuristics(
            &history_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );
        let regular_score = orderer.score_move_with_all_heuristics(
            &regular_move,
            None,
            &Some(pv_move.clone()),
            &[killer_move.clone()],
        );

        // Verify scoring hierarchy with custom weights
        assert!(pv_score > killer_score);
        assert!(killer_score > history_score);
        assert!(history_score > regular_score);

        // Verify custom weights are applied
        assert!(pv_score >= 10000); // PV weight
        assert!(killer_score >= 8000); // Killer weight
        assert!(history_score >= 4000); // History weight
    }
}
