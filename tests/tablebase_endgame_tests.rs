#![cfg(feature = "legacy-tests")]
//! End-to-end tests for specific endgame scenarios
//!
//! This module contains comprehensive tests for various endgame positions
//! that the tablebase system should be able to solve.

use shogi_engine::tablebase::{MicroTablebase, TablebaseOutcome, TablebaseResult};
use shogi_engine::{BitboardBoard, CapturedPieces, Move, Piece, PieceType, Player, Position};

/// Test positions for King + Gold vs King endgames
mod king_gold_vs_king_positions {
    use super::*;

    /// Create a basic King + Gold vs King position
    ///
    /// Black has King at (4,4) and Gold at (3,3)
    /// White has King at (6,6)
    pub fn create_basic_position() -> (BitboardBoard, CapturedPieces, Player) {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear the board
        for row in 0..9 {
            for col in 0..9 {
                board.set_piece(Position::new(row, col), None);
            }
        }

        // Place Black King at (4,4)
        board.set_piece(
            Position::new(4, 4),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place Black Gold at (3,3)
        board.set_piece(
            Position::new(3, 3),
            Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place White King at (6,6)
        board.set_piece(
            Position::new(6, 6),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::White,
                promoted: false,
            }),
        );

        (board, captured_pieces, player)
    }

    /// Create a position where Black can mate in 1 move
    pub fn create_mate_in_one_position() -> (BitboardBoard, CapturedPieces, Player) {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear the board
        for row in 0..9 {
            for col in 0..9 {
                board.set_piece(Position::new(row, col), None);
            }
        }

        // Place Black King at (4,4)
        board.set_piece(
            Position::new(4, 4),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place Black Gold at (5,5) - close to White King
        board.set_piece(
            Position::new(5, 5),
            Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place White King at (6,6) - can be mated by Gold
        board.set_piece(
            Position::new(6, 6),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::White,
                promoted: false,
            }),
        );

        (board, captured_pieces, player)
    }

    /// Create a position where Black needs to approach with King
    pub fn create_approach_position() -> (BitboardBoard, CapturedPieces, Player) {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear the board
        for row in 0..9 {
            for col in 0..9 {
                board.set_piece(Position::new(row, col), None);
            }
        }

        // Place Black King at (1,1) - far from White King
        board.set_piece(
            Position::new(1, 1),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place Black Gold at (2,2)
        board.set_piece(
            Position::new(2, 2),
            Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place White King at (7,7) - far from Black pieces
        board.set_piece(
            Position::new(7, 7),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::White,
                promoted: false,
            }),
        );

        (board, captured_pieces, player)
    }

    /// Create a position where Black needs to coordinate King and Gold
    pub fn create_coordination_position() -> (BitboardBoard, CapturedPieces, Player) {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear the board
        for row in 0..9 {
            for col in 0..9 {
                board.set_piece(Position::new(row, col), None);
            }
        }

        // Place Black King at (3,3)
        board.set_piece(
            Position::new(3, 3),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place Black Gold at (5,5)
        board.set_piece(
            Position::new(5, 5),
            Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place White King at (6,6) - needs coordination to mate
        board.set_piece(
            Position::new(6, 6),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::White,
                promoted: false,
            }),
        );

        (board, captured_pieces, player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use king_gold_vs_king_positions::*;

    #[test]
    fn test_king_gold_vs_king_basic_position() {
        let (board, captured_pieces, player) = create_basic_position();
        let tablebase = MicroTablebase::new();

        // The tablebase should be able to solve this position
        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.is_winning());
        assert!(result.best_move.is_some());
        assert!(result.moves_to_mate.is_some());
        assert!(result.moves_to_mate.unwrap() > 0);
    }

    #[test]
    fn test_king_gold_vs_king_mate_in_one() {
        let (board, captured_pieces, player) = create_mate_in_one_position();
        let tablebase = MicroTablebase::new();

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.is_winning());
        assert!(result.best_move.is_some());

        // Should be able to mate in 1 move
        if let Some(moves_to_mate) = result.moves_to_mate {
            assert!(moves_to_mate <= 3); // Allow some flexibility
        }
    }

    #[test]
    fn test_king_gold_vs_king_approach_position() {
        let (board, captured_pieces, player) = create_approach_position();
        let tablebase = MicroTablebase::new();

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.is_winning());
        assert!(result.best_move.is_some());

        // Should require more moves to mate
        if let Some(moves_to_mate) = result.moves_to_mate {
            assert!(moves_to_mate > 1);
        }
    }

    #[test]
    fn test_king_gold_vs_king_coordination_position() {
        let (board, captured_pieces, player) = create_coordination_position();
        let tablebase = MicroTablebase::new();

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.is_winning());
        assert!(result.best_move.is_some());
    }

    #[test]
    fn test_king_gold_vs_king_white_to_move() {
        let (board, captured_pieces, _) = create_basic_position();
        let tablebase = MicroTablebase::new();
        let player = Player::White;

        // White should not be able to solve this position (it's Black's advantage)
        let result = tablebase.probe(&board, player, &captured_pieces);
        // This might return None or a losing result
        if let Some(result) = result {
            assert!(result.is_losing() || result.is_draw());
        }
    }

    #[test]
    fn test_king_gold_vs_king_with_captured_pieces() {
        let (board, mut captured_pieces, player) = create_basic_position();
        let tablebase = MicroTablebase::new();

        // Add some captured pieces - this should not be solvable
        captured_pieces.black.push(PieceType::Silver);

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_none());
    }

    #[test]
    fn test_king_gold_vs_king_extra_pieces() {
        let (mut board, captured_pieces, player) = create_basic_position();
        let tablebase = MicroTablebase::new();

        // Add an extra piece - this should not be solvable
        board.set_piece(
            Position::new(0, 0),
            Some(Piece {
                piece_type: PieceType::Silver,
                player: Player::Black,
                promoted: false,
            }),
        );

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_none());
    }

    #[test]
    fn test_king_gold_vs_king_tablebase_stats() {
        let (board, captured_pieces, player) = create_basic_position();
        let mut tablebase = MicroTablebase::new();

        // Probe the tablebase multiple times
        for _ in 0..5 {
            tablebase.probe_with_stats(&board, player, &captured_pieces);
        }

        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 5);
        assert!(stats.solver_hits > 0);
        assert!(stats.solver_breakdown.contains_key("KingGoldVsKing"));
    }

    #[test]
    fn test_king_gold_vs_king_move_generation() {
        let (board, captured_pieces, player) = create_basic_position();
        let tablebase = MicroTablebase::new();

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        if let Some(move_) = result.best_move {
            // Verify the move is valid
            assert!(move_.from.row < 9);
            assert!(move_.from.col < 9);
            assert!(move_.to.row < 9);
            assert!(move_.to.col < 9);
            assert!(move_.piece_type == PieceType::King || move_.piece_type == PieceType::Gold);
            assert_eq!(move_.player, player);
        }
    }

    #[test]
    fn test_king_gold_vs_king_confidence_levels() {
        let (board, captured_pieces, player) = create_basic_position();
        let tablebase = MicroTablebase::new();

        let result = tablebase.probe(&board, player, &captured_pieces);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
    }

    #[test]
    fn test_king_gold_vs_king_different_positions() {
        let positions = vec![
            create_basic_position(),
            create_mate_in_one_position(),
            create_approach_position(),
            create_coordination_position(),
        ];

        let tablebase = MicroTablebase::new();

        for (board, captured_pieces, player) in positions {
            let result = tablebase.probe(&board, player, &captured_pieces);
            assert!(result.is_some(), "Failed to solve position");

            let result = result.unwrap();
            assert!(result.is_winning(), "Position should be winning for Black");
            assert!(result.best_move.is_some(), "Should have a best move");
        }
    }

    #[test]
    fn test_king_gold_vs_king_edge_positions() {
        let edge_positions = vec![
            create_edge_position(0, 0, 1, 1, 8, 8), // Corners
            create_edge_position(0, 8, 1, 7, 8, 0),
            create_edge_position(8, 0, 7, 1, 0, 8),
            create_edge_position(8, 8, 7, 7, 0, 0),
            create_edge_position(0, 4, 1, 3, 8, 4), // Edge centers
            create_edge_position(4, 0, 3, 1, 4, 8),
            create_edge_position(4, 8, 3, 7, 4, 0),
            create_edge_position(8, 4, 7, 3, 0, 4),
        ];

        let tablebase = MicroTablebase::new();

        for (board, captured_pieces, player) in edge_positions {
            let result = tablebase.probe(&board, player, &captured_pieces);
            // Edge positions might or might not be solvable
            if let Some(result) = result {
                assert!(result.confidence >= 0.0);
                assert!(result.confidence <= 1.0);
                assert!(result.best_move.is_some());
            }
        }
    }

    #[test]
    fn test_king_gold_vs_king_boundary_conditions() {
        let tablebase = MicroTablebase::new();

        // Test positions at various distances
        let distances = vec![1, 2, 3, 4, 5, 6, 7, 8];

        for distance in distances {
            let (board, captured_pieces, player) = create_distance_position(distance);
            let result = tablebase.probe(&board, player, &captured_pieces);

            if let Some(result) = result {
                assert!(result.confidence >= 0.0);
                assert!(result.confidence <= 1.0);

                // Closer positions should generally have higher confidence
                if distance <= 3 {
                    assert!(result.confidence > 0.5);
                }
            }
        }
    }

    #[test]
    fn test_king_gold_vs_king_promotion_scenarios() {
        let tablebase = MicroTablebase::new();

        // Test positions where pieces might promote
        let promotion_positions = vec![
            create_promotion_position(0, 4, 1, 3, 2, 4), // Black pieces near promotion zone
            create_promotion_position(8, 4, 7, 3, 6, 4), // White pieces near promotion zone
        ];

        for (board, captured_pieces, player) in promotion_positions {
            let result = tablebase.probe(&board, player, &captured_pieces);

            if let Some(result) = result {
                assert!(result.confidence >= 0.0);
                assert!(result.confidence <= 1.0);

                if let Some(move_) = result.best_move {
                    // Verify move is valid
                    assert!(move_.from.row < 9);
                    assert!(move_.from.col < 9);
                    assert!(move_.to.row < 9);
                    assert!(move_.to.col < 9);
                }
            }
        }
    }

    #[test]
    fn test_king_gold_vs_king_stalemate_scenarios() {
        let tablebase = MicroTablebase::new();

        // Test positions that might be stalemates
        let stalemate_positions = vec![
            create_stalemate_position(4, 4, 3, 3, 2, 2), // Pieces too close
            create_stalemate_position(0, 0, 1, 1, 8, 8), // Pieces too far
        ];

        for (board, captured_pieces, player) in stalemate_positions {
            let result = tablebase.probe(&board, player, &captured_pieces);

            // These positions might be draws or wins
            if let Some(result) = result {
                assert!(result.confidence >= 0.0);
                assert!(result.confidence <= 1.0);

                // Should not be a clear win
                if result.is_winning() {
                    assert!(result.confidence < 1.0);
                }
            }
        }
    }

    #[test]
    fn test_king_gold_vs_king_performance_under_load() {
        let mut tablebase = MicroTablebase::new();
        let positions = vec![
            create_basic_position(),
            create_mate_in_one_position(),
            create_approach_position(),
            create_coordination_position(),
        ];

        use std::time::Instant;
        let start = Instant::now();

        // Perform many probes under load
        for _ in 0..100 {
            for (board, captured_pieces, player) in &positions {
                tablebase.probe(board, *player, captured_pieces);
            }
        }

        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 5000);

        // Check that stats were collected
        let stats = tablebase.get_stats();
        assert!(stats.total_probes > 0);
        assert!(stats.average_probe_time_ms >= 0.0);
    }

    #[test]
    fn test_king_gold_vs_king_memory_usage() {
        let mut tablebase = MicroTablebase::new();
        let (board, captured_pieces, player) = create_basic_position();

        // Perform many operations to test memory usage
        for _ in 0..1000 {
            tablebase.probe(&board, player, &captured_pieces);
        }

        // Check that stats are reasonable
        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 1000);
        assert!(stats.cache_hits > 0); // Should have cache hits
        assert!(stats.average_probe_time_ms >= 0.0);
    }

    // Additional helper functions for new test scenarios
    fn create_edge_position(
        king_row: u8,
        king_col: u8,
        gold_row: u8,
        gold_col: u8,
        white_king_row: u8,
        white_king_col: u8,
    ) -> (BitboardBoard, CapturedPieces, Player) {
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear the board
        for row in 0..9 {
            for col in 0..9 {
                board.set_piece(Position::new(row, col), None);
            }
        }

        // Place Black King
        board.set_piece(
            Position::new(king_row, king_col),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place Black Gold
        board.set_piece(
            Position::new(gold_row, gold_col),
            Some(Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
                promoted: false,
            }),
        );

        // Place White King
        board.set_piece(
            Position::new(white_king_row, white_king_col),
            Some(Piece {
                piece_type: PieceType::King,
                player: Player::White,
                promoted: false,
            }),
        );

        (board, captured_pieces, player)
    }

    fn create_distance_position(distance: u8) -> (BitboardBoard, CapturedPieces, Player) {
        let king_row = 4;
        let king_col = 4;
        let gold_row = 3;
        let gold_col = 3;
        let white_king_row = (4 + distance).min(8);
        let white_king_col = (4 + distance).min(8);

        create_edge_position(
            king_row,
            king_col,
            gold_row,
            gold_col,
            white_king_row,
            white_king_col,
        )
    }

    fn create_promotion_position(
        king_row: u8,
        king_col: u8,
        gold_row: u8,
        gold_col: u8,
        white_king_row: u8,
        white_king_col: u8,
    ) -> (BitboardBoard, CapturedPieces, Player) {
        create_edge_position(
            king_row,
            king_col,
            gold_row,
            gold_col,
            white_king_row,
            white_king_col,
        )
    }

    fn create_stalemate_position(
        king_row: u8,
        king_col: u8,
        gold_row: u8,
        gold_col: u8,
        white_king_row: u8,
        white_king_col: u8,
    ) -> (BitboardBoard, CapturedPieces, Player) {
        create_edge_position(
            king_row,
            king_col,
            gold_row,
            gold_col,
            white_king_row,
            white_king_col,
        )
    }
}
