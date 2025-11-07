#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};

/// Integration tests for move scoring system
///
/// These tests verify that the move scoring system integrates correctly
/// with the overall search system and provides the expected performance benefits.

#[cfg(test)]
mod move_scoring_integration_tests {
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

    /// Test comprehensive move scoring integration
    #[test]
    fn test_comprehensive_move_scoring_integration() {
        let mut orderer = MoveOrdering::new();

        // Test different types of moves
        let capture_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let promotion_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Pawn,
            Player::Black,
        );

        let tactical_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Silver,
            Player::Black,
        );

        let quiet_move = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Gold,
            Player::Black,
        );

        // Score all moves
        let capture_score = orderer.score_move(&capture_move);
        let promotion_score = orderer.score_move(&promotion_move);
        let tactical_score = orderer.score_move(&tactical_move);
        let quiet_score = orderer.score_move(&quiet_move);

        // All scores should be positive
        assert!(capture_score > 0);
        assert!(promotion_score > 0);
        assert!(tactical_score > 0);
        assert!(quiet_score > 0);

        // Verify statistics are updated
        assert_eq!(orderer.stats.scoring_operations, 4);
    }

    /// Test move scoring with different piece types
    #[test]
    fn test_move_scoring_piece_types() {
        let mut orderer = MoveOrdering::new();

        let piece_types = vec![
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::King,
        ];

        let mut scores = Vec::new();

        for piece_type in piece_types {
            let move_ = create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                piece_type,
                Player::Black,
            );
            let score = orderer.score_move(&move_);
            scores.push((piece_type, score));
        }

        // All scores should be positive
        for (piece_type, score) in &scores {
            assert!(score > &0, "Score for {:?} should be positive", piece_type);
        }

        // More valuable pieces should generally score higher
        let king_score = scores
            .iter()
            .find(|(pt, _)| *pt == PieceType::King)
            .unwrap()
            .1;
        let pawn_score = scores
            .iter()
            .find(|(pt, _)| *pt == PieceType::Pawn)
            .unwrap()
            .1;
        assert!(king_score > pawn_score);
    }

    /// Test move scoring with different positions
    #[test]
    fn test_move_scoring_positions() {
        let mut orderer = MoveOrdering::new();

        let positions = vec![
            Position::new(0, 0), // Corner
            Position::new(0, 4), // Edge center
            Position::new(4, 4), // Center
            Position::new(8, 8), // Opposite corner
        ];

        let mut scores = Vec::new();

        for position in positions {
            let move_ = create_test_move(
                Some(Position::new(1, 1)),
                position,
                PieceType::Pawn,
                Player::Black,
            );
            let score = orderer.score_move(&move_);
            scores.push((position, score));
        }

        // All scores should be positive
        for (position, score) in &scores {
            assert!(
                score > &0,
                "Score for position {:?} should be positive",
                position
            );
        }

        // Center positions should generally score higher
        let center_score = scores
            .iter()
            .find(|(pos, _)| pos.row == 4 && pos.col == 4)
            .unwrap()
            .1;
        let corner_score = scores
            .iter()
            .find(|(pos, _)| pos.row == 0 && pos.col == 0)
            .unwrap()
            .1;
        assert!(center_score > corner_score);
    }

    /// Test move scoring cache integration
    #[test]
    fn test_move_scoring_cache_integration() {
        let mut orderer = MoveOrdering::new();

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // First call should populate cache
        let score1 = orderer.score_move(&move_);
        assert_eq!(orderer.stats.cache_misses, 1);
        assert_eq!(orderer.stats.cache_hits, 0);
        assert_eq!(orderer.get_cache_size(), 1);

        // Second call should use cache
        let score2 = orderer.score_move(&move_);
        assert_eq!(score1, score2);
        assert_eq!(orderer.stats.cache_misses, 1);
        assert_eq!(orderer.stats.cache_hits, 1);
        assert_eq!(orderer.get_cache_size(), 1);

        // Cache hit rate should be 50%
        assert!((orderer.get_cache_hit_rate() - 50.0).abs() < 0.1);
    }

    /// Test move scoring with weight configuration
    #[test]
    fn test_move_scoring_weight_configuration() {
        let mut orderer = MoveOrdering::new();

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Score with default weights
        let default_score = orderer.score_move(&move_);

        // Modify weights
        orderer.set_capture_weight(2000);
        orderer.set_promotion_weight(1600);
        orderer.set_tactical_weight(600);

        // Score with modified weights
        let modified_score = orderer.score_move(&move_);

        // Scores should be different if the move involves these heuristics
        // (For a simple pawn move, scores might be similar)
        assert!(modified_score >= 0);

        // Reset to defaults
        orderer.reset_weights_to_default();
        let reset_score = orderer.score_move(&move_);
        assert_eq!(default_score, reset_score);
    }

    /// Test move scoring performance under load
    #[test]
    fn test_move_scoring_performance_under_load() {
        let mut orderer = MoveOrdering::new();

        // Generate many moves
        let mut moves = Vec::new();
        for i in 0..100 {
            let move_ = create_test_move(
                Some(Position::new(i % 9, i / 9)),
                Position::new((i + 1) % 9, (i + 1) / 9),
                PieceType::Pawn,
                Player::Black,
            );
            moves.push(move_);
        }

        // Score all moves
        let start_time = std::time::Instant::now();
        for move_ in &moves {
            orderer.score_move(move_);
        }
        let elapsed = start_time.elapsed();

        // Should be fast
        assert!(elapsed.as_millis() < 100); // Less than 100ms

        // Cache should be populated
        assert!(orderer.get_cache_size() > 0);

        // Statistics should be updated
        assert_eq!(orderer.stats.scoring_operations, 100);
    }

    /// Test move scoring with all heuristics integration
    #[test]
    fn test_move_scoring_all_heuristics_integration() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

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

        // Add killer move
        orderer.add_killer_move(killer_move.clone());

        // Add history score
        orderer.update_history_score(&history_move, 3);

        // Create test position and board
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Store PV move
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

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

        // Verify scoring hierarchy
        assert!(pv_score > killer_score);
        assert!(killer_score > history_score);
        assert!(history_score > regular_score);

        // All scores should be positive
        assert!(pv_score > 0);
        assert!(killer_score > 0);
        assert!(history_score > 0);
        assert!(regular_score > 0);
    }

    /// Test move scoring statistics accuracy
    #[test]
    fn test_move_scoring_statistics_accuracy() {
        let mut orderer = MoveOrdering::new();

        let moves = vec![
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
        ];

        // Score moves
        for move_ in &moves {
            orderer.score_move(move_);
        }

        // Score same moves again (should hit cache)
        for move_ in &moves {
            orderer.score_move(move_);
        }

        // Verify statistics
        let (ops, hits, hit_rate, misses, cache_size, max_cache_size) = orderer.get_scoring_stats();
        assert_eq!(ops, 6); // 3 initial + 3 cached
        assert_eq!(hits, 3); // 3 cache hits
        assert_eq!(misses, 3); // 3 cache misses
        assert!((hit_rate - 50.0).abs() < 0.1); // 50% hit rate
        assert_eq!(cache_size, 3); // 3 unique moves cached
        assert!(max_cache_size > 0);
    }

    /// Test move scoring performance optimization
    #[test]
    fn test_move_scoring_performance_optimization() {
        let mut orderer = MoveOrdering::new();

        // Set initial cache size
        orderer.set_cache_size(100);
        assert_eq!(orderer.get_max_cache_size(), 100);

        // Warm up cache
        let moves = vec![
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
        ];
        orderer.warm_up_cache(&moves);

        // Cache should be populated
        assert!(orderer.get_cache_size() > 0);

        // Optimize performance
        orderer.optimize_performance();

        // Should still be functional
        assert!(orderer.get_max_cache_size() > 0);

        // Test scoring still works
        let move_ = create_test_move(
            Some(Position::new(4, 4)),
            Position::new(5, 4),
            PieceType::Bishop,
            Player::Black,
        );
        let score = orderer.score_move(&move_);
        assert!(score > 0);
    }

    /// Test move scoring with different players
    #[test]
    fn test_move_scoring_different_players() {
        let mut orderer = MoveOrdering::new();

        let black_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let white_move = create_test_move(
            Some(Position::new(7, 7)),
            Position::new(6, 7),
            PieceType::Pawn,
            Player::White,
        );

        let black_score = orderer.score_move(&black_move);
        let white_score = orderer.score_move(&white_move);

        // Both scores should be positive
        assert!(black_score > 0);
        assert!(white_score > 0);

        // Scores should be similar for similar moves
        assert!((black_score - white_score).abs() < 100);
    }

    /// Test move scoring edge cases
    #[test]
    fn test_move_scoring_edge_cases() {
        let mut orderer = MoveOrdering::new();

        // Test drop moves
        let drop_move = create_test_move(None, Position::new(1, 1), PieceType::Pawn, Player::Black);
        let drop_score = orderer.score_move(&drop_move);
        assert!(drop_score > 0);

        // Test moves from/to same position
        let same_position_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(1, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let same_score = orderer.score_move(&same_position_move);
        assert!(same_score >= 0);

        // Test edge positions
        let edge_move = create_test_move(
            Some(Position::new(0, 0)),
            Position::new(8, 8),
            PieceType::Rook,
            Player::Black,
        );
        let edge_score = orderer.score_move(&edge_move);
        assert!(edge_score > 0);
    }

    /// Test move scoring with custom weights
    #[test]
    fn test_move_scoring_custom_weights() {
        let custom_weights = OrderingWeights {
            capture_weight: 2000,
            promotion_weight: 1600,
            tactical_weight: 600,
            quiet_weight: 50,
            ..Default::default()
        };

        let mut orderer = MoveOrdering::with_config(custom_weights);

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let score = orderer.score_move(&move_);
        assert!(score > 0);

        // Verify custom weights are applied
        assert_eq!(orderer.weights.capture_weight, 2000);
        assert_eq!(orderer.weights.promotion_weight, 1600);
        assert_eq!(orderer.weights.tactical_weight, 600);
        assert_eq!(orderer.weights.quiet_weight, 50);
    }
}
