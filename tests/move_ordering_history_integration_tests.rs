#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};

/// Integration tests for history heuristic with search algorithm
///
/// These tests verify that the history heuristic integrates correctly
/// with the overall search system and provides the expected performance benefits.

#[cfg(test)]
mod history_heuristic_integration_tests {
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

    /// Test history heuristic integration with search depth
    #[test]
    fn test_history_heuristic_search_depth_integration() {
        let mut orderer = MoveOrdering::new();

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history score at different depths
        for depth in 1..=5 {
            orderer.update_history_score(&move_, depth);
        }

        // Score should accumulate
        let score = orderer.get_history_score(&move_);
        assert!(score > 0);

        // Statistics should be updated
        assert_eq!(orderer.stats.history_updates, 5);
    }

    /// Test history heuristic effectiveness in move ordering
    #[test]
    fn test_history_heuristic_ordering_effectiveness() {
        let mut orderer = MoveOrdering::new();

        // Create moves with different priorities
        let history_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let capture_move = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        let quiet_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );

        // Add history score to one move
        orderer.update_history_score(&history_move, 3);

        // Order moves - history move should be prioritized
        let moves = vec![
            quiet_move.clone(),
            capture_move.clone(),
            history_move.clone(),
        ];
        let ordered = orderer.order_moves_with_history(&moves);

        // Verify history move is first
        assert_eq!(ordered.len(), 3);
        assert!(orderer.moves_equal(&ordered[0], &history_move));

        // Verify statistics are updated
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert!(hits > 0);
        assert!(updates > 0);
    }

    /// Test history heuristic memory management
    #[test]
    fn test_history_heuristic_memory_management() {
        let mut orderer = MoveOrdering::new();

        let initial_memory = orderer.memory_usage.current_bytes;

        // Add history scores for many moves
        for i in 0..100 {
            let move_ = create_test_move(
                Some(Position::new(i % 9, i / 9)),
                Position::new((i + 1) % 9, (i + 1) / 9),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.update_history_score(&move_, 2);
        }

        // Verify memory usage is tracked
        assert!(orderer.memory_usage.current_bytes > initial_memory);

        // Verify all history scores are stored
        assert_eq!(orderer.stats.history_updates, 100);
    }

    /// Test history heuristic statistics accuracy
    #[test]
    fn test_history_heuristic_statistics_accuracy() {
        let mut orderer = MoveOrdering::new();

        // Initially no statistics
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(updates, 0);
        assert_eq!(aging_ops, 0);

        // Add history scores
        let history_move_1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let history_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        orderer.update_history_score(&history_move_1, 3);
        orderer.update_history_score(&history_move_2, 4);

        // Test history move detection
        orderer.score_history_move(&history_move_1);
        orderer.score_history_move(&history_move_2);

        // Test non-history move
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );
        orderer.score_history_move(&regular_move);

        // Verify statistics are accurate
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 2); // Two history move detections
        assert_eq!(misses, 1); // One non-history move
        assert_eq!(updates, 2); // Two history updates
        assert_eq!(aging_ops, 0); // No aging operations yet
    }

    /// Test history heuristic configuration and customization
    #[test]
    fn test_history_heuristic_configuration() {
        let custom_weights = OrderingWeights {
            history_weight: 4000,
            ..Default::default()
        };

        let mut orderer = MoveOrdering::with_config(custom_weights);
        orderer.set_max_history_score(5000);
        orderer.set_history_aging_factor(0.8);

        // Verify configuration
        assert_eq!(orderer.weights.history_weight, 4000);
        assert_eq!(orderer.get_max_history_score(), 5000);
        assert_eq!(orderer.get_history_aging_factor(), 0.8);

        // Test with custom configuration
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        orderer.update_history_score(&move_, 3);
        let score = orderer.score_history_move(&move_);
        assert!(score > 0);
    }

    /// Test history heuristic performance under load
    #[test]
    fn test_history_heuristic_performance_under_load() {
        let mut orderer = MoveOrdering::new();

        // Add many history scores
        let mut history_moves = Vec::new();
        for i in 0..100 {
            let move_ = create_test_move(
                Some(Position::new(i % 9, i / 9)),
                Position::new((i + 1) % 9, (i + 1) / 9),
                PieceType::Pawn,
                Player::Black,
            );
            history_moves.push(move_);
        }

        // Add history scores
        for move_ in &history_moves {
            orderer.update_history_score(move_, 2);
        }

        // Verify performance is maintained
        assert_eq!(orderer.stats.history_updates, 100);

        // Test ordering performance
        let moves = history_moves[0..10].to_vec();
        let start_time = std::time::Instant::now();
        let _ordered = orderer.order_moves_with_history(&moves);
        let elapsed = start_time.elapsed();

        // Ordering should be fast even with many history scores
        assert!(elapsed.as_micros() < 1000); // Less than 1ms
    }

    /// Test history heuristic integration with PV and killer moves
    #[test]
    fn test_history_heuristic_pv_killer_integration() {
        let mut orderer = MoveOrdering::new();
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

        // Add killer move
        orderer.add_killer_move(killer_move.clone());

        // Add history score
        orderer.update_history_score(&history_move, 3);

        // Store PV move
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Order moves with all heuristics
        let moves = vec![
            regular_move.clone(),
            history_move.clone(),
            killer_move.clone(),
            pv_move.clone(),
        ];
        // Task 3.0: No IID move context for this test
        let ordered = orderer.order_moves_with_all_heuristics(
            &moves,
            &board,
            &captured_pieces,
            player,
            depth,
            None,
        );

        // Verify correct ordering: PV > Killer > History > Regular
        assert_eq!(ordered.len(), 4);
        assert!(orderer.moves_equal(&ordered[0], &pv_move));
        assert!(orderer.moves_equal(&ordered[1], &killer_move));
        assert!(orderer.moves_equal(&ordered[2], &history_move));
        assert!(orderer.moves_equal(&ordered[3], &regular_move));
    }

    /// Test history heuristic cleanup and reset
    #[test]
    fn test_history_heuristic_cleanup_and_reset() {
        let mut orderer = MoveOrdering::new();

        // Add history scores
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.update_history_score(&move_, 3);

        // Verify history score is stored
        assert!(orderer.get_history_score(&move_) > 0);
        assert_eq!(orderer.stats.history_updates, 1);

        // Clear history table
        orderer.clear_history_table();

        // Verify history score is cleared
        assert_eq!(orderer.get_history_score(&move_), 0);

        // Verify statistics are reset
        let (hits, misses, hit_rate, updates, aging_ops) = orderer.get_history_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(updates, 0);
        assert_eq!(aging_ops, 0);
    }

    /// Test history heuristic aging mechanism
    #[test]
    fn test_history_heuristic_aging_mechanism() {
        let mut orderer = MoveOrdering::new();
        orderer.set_history_aging_factor(0.5);

        // Add history scores
        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        orderer.update_history_score(&move_, 4); // 4*4 = 16

        // Verify initial score
        assert_eq!(orderer.get_history_score(&move_), 16);

        // Age the table
        orderer.age_history_table();

        // Verify score is reduced
        assert_eq!(orderer.get_history_score(&move_), 8); // 16 * 0.5 = 8

        // Verify statistics are updated
        assert_eq!(orderer.stats.history_aging_operations, 1);
    }

    /// Test history heuristic edge cases
    #[test]
    fn test_history_heuristic_edge_cases() {
        let mut orderer = MoveOrdering::new();

        // Test with empty move list
        let empty_moves: Vec<Move> = Vec::new();
        let ordered = orderer.order_moves_with_history(&empty_moves);
        assert!(ordered.is_empty());

        // Test with single move
        let single_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let ordered = orderer.order_moves_with_history(&[single_move.clone()]);
        assert_eq!(ordered.len(), 1);
        assert!(orderer.moves_equal(&ordered[0], &single_move));

        // Test with drop moves (no from position)
        let drop_move = create_test_move(None, Position::new(1, 1), PieceType::Pawn, Player::Black);
        orderer.update_history_score(&drop_move, 3);
        assert_eq!(orderer.get_history_score(&drop_move), 0); // Should not be stored
    }

    /// Test history heuristic with different piece types
    #[test]
    fn test_history_heuristic_different_piece_types() {
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

        // Add history scores with different piece types
        for (i, piece_type) in piece_types.iter().enumerate() {
            let move_ = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                *piece_type,
                Player::Black,
            );
            orderer.update_history_score(&move_, 2);
            assert!(orderer.get_history_score(&move_) > 0);
        }

        // Verify all history scores are stored
        assert_eq!(orderer.stats.history_updates, piece_types.len() as u64);
    }

    /// Test history heuristic with different players
    #[test]
    fn test_history_heuristic_different_players() {
        let mut orderer = MoveOrdering::new();

        // Add history scores for both players
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

        orderer.update_history_score(&black_move, 3);
        orderer.update_history_score(&white_move, 4);

        // Both should have history scores
        assert!(orderer.get_history_score(&black_move) > 0);
        assert!(orderer.get_history_score(&white_move) > 0);

        // Verify both are in the history table
        assert_eq!(orderer.stats.history_updates, 2);
    }

    /// Test history heuristic score accumulation
    #[test]
    fn test_history_heuristic_score_accumulation() {
        let mut orderer = MoveOrdering::new();

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update history score multiple times
        orderer.update_history_score(&move_, 2);
        orderer.update_history_score(&move_, 3);
        orderer.update_history_score(&move_, 4);

        // Score should accumulate
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 4 + 9 + 16); // 2*2 + 3*3 + 4*4 = 29

        // Verify statistics
        assert_eq!(orderer.stats.history_updates, 3);
    }

    /// Test history heuristic maximum score limit
    #[test]
    fn test_history_heuristic_max_score_limit() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_history_score(100);

        let move_ = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        // Update with large depth to exceed limit
        orderer.update_history_score(&move_, 20); // 20*20 = 400

        // Score should be capped at max
        let score = orderer.get_history_score(&move_);
        assert_eq!(score, 100);

        // Verify statistics
        assert_eq!(orderer.stats.history_updates, 1);
    }
}
