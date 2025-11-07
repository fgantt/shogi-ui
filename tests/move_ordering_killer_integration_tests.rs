#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::{BitboardBoard, CapturedPieces, PieceType, Player, Position};
use shogi_engine::moves::Move;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};

/// Integration tests for killer move heuristic with search algorithm
///
/// These tests verify that the killer move heuristic integrates correctly
/// with the overall search system and provides the expected performance benefits.

#[cfg(test)]
mod killer_move_integration_tests {
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

    /// Test killer move integration with search depth management
    #[test]
    fn test_killer_move_search_depth_integration() {
        let mut orderer = MoveOrdering::new();

        // Simulate search at different depths
        for depth in 1..=5 {
            orderer.set_current_depth(depth);

            // Add killer moves at each depth
            let killer_move = create_test_move(
                Some(Position::new(depth, depth)),
                Position::new(depth + 1, depth + 1),
                PieceType::Pawn,
                Player::Black,
            );

            orderer.add_killer_move(killer_move.clone());

            // Verify killer move is stored at correct depth
            assert!(orderer.is_killer_move(&killer_move));
            assert_eq!(orderer.get_current_depth(), depth);
        }

        // Verify killer moves are properly separated by depth
        for depth in 1..=5 {
            orderer.set_current_depth(depth);

            // Each depth should have exactly one killer move
            let killer_moves = orderer.get_current_killer_moves();
            assert!(killer_moves.is_some());
            assert_eq!(killer_moves.unwrap().len(), 1);
        }
    }

    /// Test killer move effectiveness in move ordering
    #[test]
    fn test_killer_move_ordering_effectiveness() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

        // Create moves with different priorities
        let killer_move = create_test_move(
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

        // Add killer move
        orderer.add_killer_move(killer_move.clone());

        // Order moves - killer move should be prioritized
        let moves = vec![
            quiet_move.clone(),
            capture_move.clone(),
            killer_move.clone(),
        ];
        let ordered = orderer.order_moves_with_killer(&moves);

        // Verify killer move is first
        assert_eq!(ordered.len(), 3);
        assert!(orderer.moves_equal(&ordered[0], &killer_move));

        // Verify statistics are updated
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert!(hits > 0);
        assert!(stored > 0);
    }

    /// Test killer move memory management
    #[test]
    fn test_killer_move_memory_management() {
        let mut orderer = MoveOrdering::new();
        orderer.set_max_killer_moves_per_depth(3);

        let initial_memory = orderer.memory_usage.current_bytes;

        // Add killer moves at multiple depths
        for depth in 1..=5 {
            orderer.set_current_depth(depth);

            for i in 0..5 {
                // Add more than the limit
                let killer_move = create_test_move(
                    Some(Position::new(i, depth)),
                    Position::new(i + 1, depth),
                    PieceType::Pawn,
                    Player::Black,
                );
                orderer.add_killer_move(killer_move);
            }
        }

        // Verify memory usage is tracked
        assert!(orderer.memory_usage.current_bytes > initial_memory);

        // Verify limits are respected
        for depth in 1..=5 {
            orderer.set_current_depth(depth);
            let killer_moves = orderer.get_current_killer_moves();
            assert!(killer_moves.is_some());
            assert!(killer_moves.unwrap().len() <= 3);
        }
    }

    /// Test killer move statistics accuracy
    #[test]
    fn test_killer_move_statistics_accuracy() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

        // Initially no statistics
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(stored, 0);

        // Add killer moves
        let killer_move_1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let killer_move_2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            Player::Black,
        );

        orderer.add_killer_move(killer_move_1.clone());
        orderer.add_killer_move(killer_move_2.clone());

        // Test killer move detection
        assert!(orderer.is_killer_move(&killer_move_1));
        assert!(orderer.is_killer_move(&killer_move_2));

        // Test non-killer move
        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );
        assert!(!orderer.is_killer_move(&regular_move));

        // Verify statistics are accurate
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert_eq!(hits, 2); // Two killer move detections
        assert_eq!(misses, 1); // One non-killer move
        assert_eq!(stored, 2); // Two killer moves stored
        assert!((hit_rate - 66.67).abs() < 0.1); // Approximately 66.67% hit rate
    }

    /// Test killer move configuration and customization
    #[test]
    fn test_killer_move_configuration() {
        let custom_weights = OrderingWeights {
            killer_move_weight: 8000,
            ..Default::default()
        };

        let mut orderer = MoveOrdering::with_config(custom_weights);
        orderer.set_max_killer_moves_per_depth(5);

        // Verify configuration
        assert_eq!(orderer.weights.killer_move_weight, 8000);
        assert_eq!(orderer.get_max_killer_moves_per_depth(), 5);

        // Test with custom configuration
        orderer.set_current_depth(3);
        let killer_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        orderer.add_killer_move(killer_move.clone());
        let score = orderer.score_killer_move(&killer_move);
        assert_eq!(score, 8000);
    }

    /// Test killer move performance under load
    #[test]
    fn test_killer_move_performance_under_load() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

        // Add many killer moves
        let mut killer_moves = Vec::new();
        for i in 0..100 {
            let killer_move = create_test_move(
                Some(Position::new(i % 9, i / 9)),
                Position::new((i + 1) % 9, (i + 1) / 9),
                PieceType::Pawn,
                Player::Black,
            );
            killer_moves.push(killer_move);
        }

        // Add killer moves (should respect limit)
        for killer_move in &killer_moves {
            orderer.add_killer_move(killer_move.clone());
        }

        // Verify performance is maintained
        let killer_moves_list = orderer.get_current_killer_moves();
        assert!(killer_moves_list.is_some());
        assert!(killer_moves_list.unwrap().len() <= 2); // Default limit

        // Test ordering performance
        let moves = killer_moves[0..10].to_vec();
        let start_time = std::time::Instant::now();
        let _ordered = orderer.order_moves_with_killer(&moves);
        let elapsed = start_time.elapsed();

        // Ordering should be fast even with many killer moves
        assert!(elapsed.as_micros() < 1000); // Less than 1ms
    }

    /// Test killer move integration with PV moves
    #[test]
    fn test_killer_move_pv_integration() {
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

        let regular_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            Player::Black,
        );

        // Add killer move
        orderer.add_killer_move(killer_move.clone());

        // Store PV move
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Order moves with both PV and killer prioritization
        let moves = vec![regular_move.clone(), killer_move.clone(), pv_move.clone()];
        let ordered =
            orderer.order_moves_with_pv_and_killer(&moves, &board, &captured_pieces, player, depth);

        // Verify correct ordering: PV > Killer > Regular
        assert_eq!(ordered.len(), 3);
        assert!(orderer.moves_equal(&ordered[0], &pv_move));
        assert!(orderer.moves_equal(&ordered[1], &killer_move));
        assert!(orderer.moves_equal(&ordered[2], &regular_move));
    }

    /// Test killer move cleanup and reset
    #[test]
    fn test_killer_move_cleanup_and_reset() {
        let mut orderer = MoveOrdering::new();

        // Add killer moves at multiple depths
        for depth in 1..=3 {
            orderer.set_current_depth(depth);
            let killer_move = create_test_move(
                Some(Position::new(depth, depth)),
                Position::new(depth + 1, depth + 1),
                PieceType::Pawn,
                Player::Black,
            );
            orderer.add_killer_move(killer_move);
        }

        // Verify killer moves are stored
        for depth in 1..=3 {
            orderer.set_current_depth(depth);
            assert!(orderer.get_current_killer_moves().is_some());
        }

        // Clear all killer moves
        orderer.clear_all_killer_moves();

        // Verify all killer moves are cleared
        for depth in 1..=3 {
            orderer.set_current_depth(depth);
            assert!(orderer.get_current_killer_moves().is_none());
        }

        // Verify statistics are reset
        let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
        assert_eq!(stored, 0);
    }

    /// Test killer move edge cases
    #[test]
    fn test_killer_move_edge_cases() {
        let mut orderer = MoveOrdering::new();

        // Test with empty move list
        let empty_moves: Vec<Move> = Vec::new();
        let ordered = orderer.order_moves_with_killer(&empty_moves);
        assert!(ordered.is_empty());

        // Test with single move
        let single_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );
        let ordered = orderer.order_moves_with_killer(&[single_move.clone()]);
        assert_eq!(ordered.len(), 1);
        assert!(orderer.moves_equal(&ordered[0], &single_move));

        // Test with maximum depth
        orderer.set_current_depth(255);
        orderer.add_killer_move(single_move.clone());
        assert!(orderer.is_killer_move(&single_move));

        // Test clearing non-existent depth
        orderer.clear_killer_moves_for_depth(999);
        // Should not panic or cause issues
    }

    /// Test killer move with different piece types
    #[test]
    fn test_killer_move_different_piece_types() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

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

        // Add killer moves with different piece types
        for (i, piece_type) in piece_types.iter().enumerate() {
            let killer_move = create_test_move(
                Some(Position::new(i, 0)),
                Position::new(i + 1, 0),
                *piece_type,
                Player::Black,
            );
            orderer.add_killer_move(killer_move.clone());
            assert!(orderer.is_killer_move(&killer_move));
        }

        // Verify all killer moves are stored
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), piece_types.len());
    }

    /// Test killer move with different players
    #[test]
    fn test_killer_move_different_players() {
        let mut orderer = MoveOrdering::new();
        orderer.set_current_depth(3);

        // Add killer moves for both players
        let black_killer = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            Player::Black,
        );

        let white_killer = create_test_move(
            Some(Position::new(7, 7)),
            Position::new(6, 7),
            PieceType::Pawn,
            Player::White,
        );

        orderer.add_killer_move(black_killer.clone());
        orderer.add_killer_move(white_killer.clone());

        // Both should be stored as killer moves
        assert!(orderer.is_killer_move(&black_killer));
        assert!(orderer.is_killer_move(&white_killer));

        // Verify both are in the killer moves list
        let killer_moves = orderer.get_current_killer_moves();
        assert!(killer_moves.is_some());
        assert_eq!(killer_moves.unwrap().len(), 2);
    }
}
