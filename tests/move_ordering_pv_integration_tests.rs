#![cfg(feature = "legacy-tests")]
//! Integration tests for PV move ordering with transposition table
//!
//! This module provides comprehensive integration tests to validate
//! the PV move ordering functionality works correctly with the
//! transposition table system.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::search::move_ordering::{MoveOrdering, OrderingWeights};
use shogi_engine::search::{ThreadSafeTranspositionTable, ThreadSafetyMode, TranspositionConfig};
use shogi_engine::types::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

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
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
        }
    }

    #[test]
    fn test_pv_move_integration_with_transposition_table() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create test moves
        let moves = vec![
            create_test_move(
                Some(Position::new(1, 1)),
                Position::new(2, 1),
                PieceType::Pawn,
                player,
            ),
            create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                player,
            ),
            create_test_move(
                Some(Position::new(3, 3)),
                Position::new(4, 3),
                PieceType::Gold,
                player,
            ),
        ];

        // Initially no PV move should be found
        let pv_move = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        assert!(pv_move.is_none());

        // Order moves (should work without PV move)
        let ordered_moves =
            orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);
        assert_eq!(ordered_moves.len(), 3);

        // Verify statistics
        let (hits, misses, hit_rate, tt_lookups, tt_hits) = orderer.get_pv_stats();
        assert_eq!(hits, 0);
        assert!(misses > 0); // Should have misses since no PV move was found
        assert_eq!(hit_rate, 0.0);
        assert!(tt_lookups > 0);
        assert_eq!(tt_hits, 0); // No hits since no entry in TT
    }

    #[test]
    fn test_pv_move_storage_and_retrieval() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create a test move
        let best_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );

        // Store PV move in transposition table
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            best_move.clone(),
            100,
        );

        // Retrieve PV move
        let retrieved_pv_move = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        assert!(retrieved_pv_move.is_some());

        let pv_move = retrieved_pv_move.unwrap();
        assert_eq!(pv_move.from, best_move.from);
        assert_eq!(pv_move.to, best_move.to);
        assert_eq!(pv_move.piece_type, best_move.piece_type);
        assert_eq!(pv_move.player, best_move.player);

        // Verify statistics show a hit
        let (hits, misses, hit_rate, tt_lookups, tt_hits) = orderer.get_pv_stats();
        assert!(hits > 0);
        assert!(tt_hits > 0);
        assert!(hit_rate > 0.0);
    }

    #[test]
    fn test_pv_move_prioritization_in_ordering() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create test moves
        let capture_move = {
            let mut move_ = create_test_move(
                Some(Position::new(2, 2)),
                Position::new(3, 2),
                PieceType::Silver,
                player,
            );
            move_.is_capture = true;
            move_.captured_piece = Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::White,
            });
            move_
        };

        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );

        let quiet_move = create_test_move(
            Some(Position::new(3, 3)),
            Position::new(4, 3),
            PieceType::Gold,
            player,
        );

        let moves = vec![capture_move.clone(), pv_move.clone(), quiet_move.clone()];

        // Store PV move in transposition table
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // Order moves with PV prioritization
        let ordered_moves =
            orderer.order_moves_with_pv(&moves, &board, &captured_pieces, player, depth);

        // PV move should be first (highest priority)
        assert_eq!(ordered_moves[0].from, pv_move.from);
        assert_eq!(ordered_moves[0].to, pv_move.to);
        assert_eq!(ordered_moves[0].piece_type, pv_move.piece_type);

        // Verify all moves are present
        assert_eq!(ordered_moves.len(), 3);
    }

    #[test]
    fn test_pv_move_cache_effectiveness() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create and store PV move
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );
        orderer.update_pv_move(
            &board,
            &captured_pieces,
            player,
            depth,
            pv_move.clone(),
            100,
        );

        // First lookup - should hit transposition table
        let first_lookup = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        assert!(first_lookup.is_some());

        let initial_tt_lookups = orderer.stats.tt_lookups;
        let initial_tt_hits = orderer.stats.tt_hits;

        // Second lookup - should hit cache
        let second_lookup = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        assert!(second_lookup.is_some());

        // TT lookups should not have increased (cache hit)
        assert_eq!(orderer.stats.tt_lookups, initial_tt_lookups);
        assert_eq!(orderer.stats.tt_hits, initial_tt_hits);
    }

    #[test]
    fn test_pv_move_clear_functionality() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create and store PV move
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );
        orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move, 100);

        // Verify PV move is stored
        let retrieved = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        assert!(retrieved.is_some());
        assert!(orderer.pv_move_cache.len() > 0);

        // Clear PV move cache
        orderer.clear_pv_move_cache();

        // Verify cache is cleared
        assert_eq!(orderer.pv_move_cache.len(), 0);
        assert_eq!(orderer.stats.pv_move_hits, 0);
        assert_eq!(orderer.stats.pv_move_misses, 0);
        assert_eq!(orderer.stats.pv_move_hit_rate, 0.0);
        assert_eq!(orderer.stats.tt_lookups, 0);
        assert_eq!(orderer.stats.tt_hits, 0);
    }

    #[test]
    fn test_pv_move_with_different_positions() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Create different board positions
        let board1 = BitboardBoard::new();
        let board2 = BitboardBoard::new(); // Same position for now

        // Create different PV moves for each position
        let pv_move1 = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );

        let pv_move2 = create_test_move(
            Some(Position::new(2, 2)),
            Position::new(3, 2),
            PieceType::Silver,
            player,
        );

        // Store PV moves for different positions
        orderer.update_pv_move(
            &board1,
            &captured_pieces,
            player,
            depth,
            pv_move1.clone(),
            100,
        );
        orderer.update_pv_move(
            &board2,
            &captured_pieces,
            player,
            depth,
            pv_move2.clone(),
            150,
        );

        // Retrieve PV moves for each position
        let retrieved1 = orderer.get_pv_move(&board1, &captured_pieces, player, depth);
        let retrieved2 = orderer.get_pv_move(&board2, &captured_pieces, player, depth);

        // Both should be found
        assert!(retrieved1.is_some());
        assert!(retrieved2.is_some());

        // Verify cache contains both entries
        assert!(orderer.pv_move_cache.len() >= 2);
    }

    #[test]
    fn test_pv_move_statistics_accuracy() {
        // Create transposition table
        let config = TranspositionConfig::default();
        let mut tt = ThreadSafeTranspositionTable::with_thread_mode(
            config,
            ThreadSafetyMode::SingleThreaded,
        );

        // Create move orderer
        let mut orderer = MoveOrdering::new();
        orderer.set_transposition_table(&tt);

        // Create test position
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let depth = 3;

        // Perform multiple lookups
        let pv_move = create_test_move(
            Some(Position::new(1, 1)),
            Position::new(2, 1),
            PieceType::Pawn,
            player,
        );

        // First lookup - should be a miss
        let _ = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        let (hits1, misses1, hit_rate1, tt_lookups1, tt_hits1) = orderer.get_pv_stats();

        // Store PV move
        orderer.update_pv_move(&board, &captured_pieces, player, depth, pv_move, 100);

        // Second lookup - should be a hit
        let _ = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        let (hits2, misses2, hit_rate2, tt_lookups2, tt_hits2) = orderer.get_pv_stats();

        // Third lookup - should be a cache hit
        let _ = orderer.get_pv_move(&board, &captured_pieces, player, depth);
        let (hits3, misses3, hit_rate3, tt_lookups3, tt_hits3) = orderer.get_pv_stats();

        // Verify statistics progression
        assert!(hits2 > hits1);
        assert!(hits3 > hits2);
        assert!(hit_rate3 > hit_rate2);
        assert!(hit_rate2 > hit_rate1);
        assert!(tt_lookups2 > tt_lookups1);
        assert!(tt_lookups3 == tt_lookups2); // Cache hit, no new TT lookup
    }
}
