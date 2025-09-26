/// Comprehensive test suite for the opening book implementation
/// 
/// This module contains unit tests, integration tests, and performance tests
/// for all opening book functionality.

use shogi_engine::opening_book::*;
use shogi_engine::types::*;

#[cfg(test)]
mod book_move_tests {
    use super::*;

    #[test]
    fn test_book_move_creation() {
        let from = Position::new(2, 6); // 27
        let to = Position::new(2, 5);   // 26
        let book_move = BookMove::new(
            Some(from),
            to,
            PieceType::Rook,
            false,
            false,
            850,
            15
        );

        assert_eq!(book_move.from, Some(from));
        assert_eq!(book_move.to, to);
        assert_eq!(book_move.piece_type, PieceType::Rook);
        assert_eq!(book_move.is_drop, false);
        assert_eq!(book_move.is_promotion, false);
        assert_eq!(book_move.weight, 850);
        assert_eq!(book_move.evaluation, 15);
        assert!(book_move.opening_name.is_none());
        assert!(book_move.move_notation.is_none());
    }

    #[test]
    fn test_book_move_with_metadata() {
        let from = Position::new(2, 6);
        let to = Position::new(2, 5);
        let book_move = BookMove::new_with_metadata(
            Some(from),
            to,
            PieceType::Rook,
            false,
            false,
            850,
            15,
            Some("Aggressive Rook".to_string()),
            Some("27-26".to_string())
        );

        assert_eq!(book_move.from, Some(from));
        assert_eq!(book_move.to, to);
        assert_eq!(book_move.piece_type, PieceType::Rook);
        assert_eq!(book_move.weight, 850);
        assert_eq!(book_move.evaluation, 15);
        assert_eq!(book_move.opening_name, Some("Aggressive Rook".to_string()));
        assert_eq!(book_move.move_notation, Some("27-26".to_string()));
    }

    #[test]
    fn test_drop_move_creation() {
        let to = Position::new(2, 5);
        let book_move = BookMove::new(
            None, // Drop move
            to,
            PieceType::Pawn,
            true,  // is_drop
            false,
            500,
            10
        );

        assert_eq!(book_move.from, None);
        assert_eq!(book_move.to, to);
        assert_eq!(book_move.piece_type, PieceType::Pawn);
        assert_eq!(book_move.is_drop, true);
        assert_eq!(book_move.is_promotion, false);
    }

    #[test]
    fn test_promotion_move_creation() {
        let from = Position::new(2, 6);
        let to = Position::new(2, 5);
        let book_move = BookMove::new(
            Some(from),
            to,
            PieceType::Pawn,
            false,
            true,  // is_promotion
            750,
            25
        );

        assert_eq!(book_move.from, Some(from));
        assert_eq!(book_move.to, to);
        assert_eq!(book_move.piece_type, PieceType::Pawn);
        assert_eq!(book_move.is_drop, false);
        assert_eq!(book_move.is_promotion, true);
    }

    #[test]
    fn test_to_engine_move_conversion() {
        let from = Position::new(2, 6);
        let to = Position::new(2, 5);
        let book_move = BookMove::new(
            Some(from),
            to,
            PieceType::Rook,
            false,
            false,
            850,
            15
        );

        let engine_move = book_move.to_engine_move(Player::Black);

        assert_eq!(engine_move.from, Some(from));
        assert_eq!(engine_move.to, to);
        assert_eq!(engine_move.piece_type, PieceType::Rook);
        assert_eq!(engine_move.player, Player::Black);
        assert_eq!(engine_move.is_promotion, false);
        assert_eq!(engine_move.is_capture, false);
        assert_eq!(engine_move.gives_check, false);
    }

    #[test]
    fn test_drop_move_to_engine_move() {
        let to = Position::new(2, 5);
        let book_move = BookMove::new(
            None,
            to,
            PieceType::Pawn,
            true,
            false,
            500,
            10
        );

        let engine_move = book_move.to_engine_move(Player::White);

        assert_eq!(engine_move.from, None);
        assert_eq!(engine_move.to, to);
        assert_eq!(engine_move.piece_type, PieceType::Pawn);
        assert_eq!(engine_move.player, Player::White);
        assert_eq!(engine_move.is_promotion, false);
    }
}

#[cfg(test)]
mod position_entry_tests {
    use super::*;

    #[test]
    fn test_position_entry_creation() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                800,
                10
            )
        ];

        let entry = PositionEntry::new(fen.clone(), moves.clone());

        assert_eq!(entry.fen, fen);
        assert_eq!(entry.moves.len(), 2);
        assert_eq!(entry.moves[0].weight, 850);
        assert_eq!(entry.moves[1].weight, 800);
    }

    #[test]
    fn test_add_move() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let mut entry = PositionEntry::new(fen, vec![]);

        let new_move = BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15
        );

        entry.add_move(new_move.clone());

        assert_eq!(entry.moves.len(), 1);
        assert_eq!(entry.moves[0].weight, 850);
    }

    #[test]
    fn test_get_best_move() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                800,  // Lower weight
                15
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                850,  // Higher weight
                10
            )
        ];

        let entry = PositionEntry::new(fen, moves);

        let best_move = entry.get_best_move();
        assert!(best_move.is_some());
        assert_eq!(best_move.unwrap().weight, 850);
    }

    #[test]
    fn test_get_best_move_by_evaluation() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                10  // Lower evaluation
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                800,
                25  // Higher evaluation
            )
        ];

        let entry = PositionEntry::new(fen, moves);

        let best_move = entry.get_best_move_by_evaluation();
        assert!(best_move.is_some());
        assert_eq!(best_move.unwrap().evaluation, 25);
    }

    #[test]
    fn test_get_random_move() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                100,  // Low weight
                15
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                900,  // High weight
                10
            )
        ];

        let entry = PositionEntry::new(fen, moves);

        // Test multiple times to ensure randomness
        let mut high_weight_count = 0;
        for _ in 0..100 {
            if let Some(random_move) = entry.get_random_move() {
                if random_move.weight == 900 {
                    high_weight_count += 1;
                }
            }
        }

        // High weight move should be selected more often
        assert!(high_weight_count > 50);
    }

    #[test]
    fn test_get_moves_by_weight() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                500,
                15
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                900,
                10
            ),
            BookMove::new(
                Some(Position::new(3, 6)),
                Position::new(3, 5),
                PieceType::Silver,
                false,
                false,
                700,
                20
            )
        ];

        let entry = PositionEntry::new(fen, moves);
        let sorted_moves = entry.get_moves_by_weight();

        assert_eq!(sorted_moves.len(), 3);
        assert_eq!(sorted_moves[0].weight, 900);  // Highest weight first
        assert_eq!(sorted_moves[1].weight, 700);
        assert_eq!(sorted_moves[2].weight, 500);  // Lowest weight last
    }

    #[test]
    fn test_get_moves_by_evaluation() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                10  // Lower evaluation
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                800,
                30  // Higher evaluation
            ),
            BookMove::new(
                Some(Position::new(3, 6)),
                Position::new(3, 5),
                PieceType::Silver,
                false,
                false,
                700,
                20  // Middle evaluation
            )
        ];

        let entry = PositionEntry::new(fen, moves);
        let sorted_moves = entry.get_moves_by_evaluation();

        assert_eq!(sorted_moves.len(), 3);
        assert_eq!(sorted_moves[0].evaluation, 30);  // Highest evaluation first
        assert_eq!(sorted_moves[1].evaluation, 20);
        assert_eq!(sorted_moves[2].evaluation, 10);  // Lowest evaluation last
    }

    #[test]
    fn test_empty_position_entry() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let entry = PositionEntry::new(fen, vec![]);

        assert!(entry.get_best_move().is_none());
        assert!(entry.get_random_move().is_none());
        assert!(entry.get_best_move_by_evaluation().is_none());
        assert!(entry.get_moves_by_weight().is_empty());
        assert!(entry.get_moves_by_evaluation().is_empty());
    }
}

#[cfg(test)]
mod opening_book_tests {
    use super::*;

    #[test]
    fn test_opening_book_creation() {
        let mut book = OpeningBook::new();

        assert!(!book.is_loaded());
        let stats = book.get_stats();
        assert_eq!(stats.position_count, 0);
        assert_eq!(stats.move_count, 0);
    }

    #[test]
    fn test_add_position() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen.clone(), moves);

        let stats = book.get_stats();
        assert_eq!(stats.position_count, 1);
        assert_eq!(stats.move_count, 1);
    }

    #[test]
    fn test_get_moves() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen.clone(), moves.clone());

        let retrieved_moves = book.get_moves(&fen);
        assert!(retrieved_moves.is_some());
        assert_eq!(retrieved_moves.unwrap().len(), 1);
    }

    #[test]
    fn test_get_best_move() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        let best_move = book.get_best_move(&fen);
        assert!(best_move.is_some());
        assert_eq!(best_move.unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_get_random_move() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        let random_move = book.get_random_move(&fen);
        assert!(random_move.is_some());
        assert_eq!(random_move.unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_position_not_found() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();

        assert!(book.get_moves(&fen).is_none());
        assert!(book.get_best_move(&fen).is_none());
        assert!(book.get_random_move(&fen).is_none());
    }

    #[test]
    fn test_fen_lookup_consistency() {
        let mut book = OpeningBook::new();
        let fen1 = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let fen2 = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let fen3 = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1";

        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen1.to_string(), moves);
        book = book.mark_loaded();

        // Same FEN should find the same moves
        let moves1 = book.get_moves(fen1);
        let moves2 = book.get_moves(fen2);
        assert!(moves1.is_some());
        assert!(moves2.is_some());
        assert_eq!(moves1.unwrap().len(), moves2.unwrap().len());

        // Different FEN should not find moves
        let moves3 = book.get_moves(fen3);
        assert!(moves3.is_none());
    }

    #[test]
    fn test_validate_empty_book() {
        let mut book = OpeningBook::new();
        let result = book.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_loaded_book() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen, moves);
        book = book.mark_loaded();

        let result = book.validate();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod coordinate_conversion_tests {
    use super::*;
    use shogi_engine::opening_book::coordinate_utils;

    #[test]
    fn test_string_to_position() {
        // Test valid USI coordinates (format: "file+rank" like "1a", "5e", "9i")
        assert_eq!(coordinate_utils::string_to_position("1a").unwrap(), Position::new(0, 8));
        assert_eq!(coordinate_utils::string_to_position("5e").unwrap(), Position::new(4, 4));
        assert_eq!(coordinate_utils::string_to_position("9i").unwrap(), Position::new(8, 0));
        assert_eq!(coordinate_utils::string_to_position("1i").unwrap(), Position::new(8, 8));
        assert_eq!(coordinate_utils::string_to_position("9a").unwrap(), Position::new(0, 0));
    }

    #[test]
    fn test_string_to_position_invalid() {
        // Test invalid USI coordinates
        assert!(coordinate_utils::string_to_position("").is_err());
        assert!(coordinate_utils::string_to_position("1").is_err());
        assert!(coordinate_utils::string_to_position("123").is_err());
        assert!(coordinate_utils::string_to_position("0a").is_err());
        assert!(coordinate_utils::string_to_position("1j").is_err());
        assert!(coordinate_utils::string_to_position("ab").is_err());
    }

    #[test]
    fn test_position_to_string() {
        // Test valid positions (format: "file+rank" like "1a", "5e", "9i")
        assert_eq!(coordinate_utils::position_to_string(Position::new(0, 0)), "9a");
        assert_eq!(coordinate_utils::position_to_string(Position::new(4, 4)), "5e");
        assert_eq!(coordinate_utils::position_to_string(Position::new(8, 0)), "9i");
        assert_eq!(coordinate_utils::position_to_string(Position::new(8, 8)), "1i");
        assert_eq!(coordinate_utils::position_to_string(Position::new(0, 8)), "1a");
    }

    #[test]
    fn test_parse_piece_type() {
        // Test valid piece types
        assert_eq!(coordinate_utils::parse_piece_type("Pawn").unwrap(), PieceType::Pawn);
        assert_eq!(coordinate_utils::parse_piece_type("Rook").unwrap(), PieceType::Rook);
        assert_eq!(coordinate_utils::parse_piece_type("Bishop").unwrap(), PieceType::Bishop);
        assert_eq!(coordinate_utils::parse_piece_type("King").unwrap(), PieceType::King);
    }

    #[test]
    fn test_parse_piece_type_invalid() {
        // Test invalid piece types
        assert!(coordinate_utils::parse_piece_type("").is_err());
        assert!(coordinate_utils::parse_piece_type("Invalid").is_err());
        assert!(coordinate_utils::parse_piece_type("pawn").is_err()); // Case sensitive
    }
}

#[cfg(test)]
mod binary_format_tests {
    use super::*;

    #[test]
    fn test_binary_serialization_roundtrip() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new_with_metadata(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15,
                Some("Aggressive Rook".to_string()),
                Some("27-26".to_string())
            ),
            BookMove::new_with_metadata(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                800,
                10,
                Some("Aggressive Rook".to_string()),
                Some("77-76".to_string())
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        // Serialize to binary
        let binary_data = book.to_binary().unwrap();

        // Deserialize from binary
        let mut deserialized_book = OpeningBook::from_binary(&binary_data).unwrap();

        // Verify data integrity
        let original_stats = book.get_stats();
        let deserialized_stats = deserialized_book.get_stats();
        assert_eq!(deserialized_stats.position_count, original_stats.position_count);
        assert_eq!(deserialized_stats.move_count, original_stats.move_count);
        assert_eq!(deserialized_book.is_loaded(), book.is_loaded());

        // Verify moves can be retrieved
        let moves = deserialized_book.get_moves(&fen);
        assert!(moves.is_some());
        assert_eq!(moves.unwrap().len(), 2);
    }

    #[test]
    fn test_binary_format_validation() {
        let mut book = OpeningBook::new();
        let binary_data = book.to_binary().unwrap();

        // Test magic number validation
        assert!(binary_data.len() >= 4);
        assert_eq!(&binary_data[0..4], b"SBOB");

        // Test version validation
        let version = u32::from_le_bytes([binary_data[4], binary_data[5], binary_data[6], binary_data[7]]);
        assert_eq!(version, 1);
    }

    #[test]
    fn test_empty_book_serialization() {
        let mut book = OpeningBook::new();
        let binary_data = book.to_binary().unwrap();
        let mut deserialized_book = OpeningBook::from_binary(&binary_data).unwrap();

        let stats = deserialized_book.get_stats();
        assert_eq!(stats.position_count, 0);
        assert_eq!(stats.move_count, 0);
        assert!(!deserialized_book.is_loaded());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_opening_book() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

        assert!(book.get_moves(fen).is_none());
        assert!(book.get_best_move(fen).is_none());
        assert!(book.get_random_move(fen).is_none());
        assert!(!book.is_loaded());
    }

    #[test]
    fn test_invalid_fen_handling() {
        let mut book = OpeningBook::new();
        let invalid_fens = vec![
            "",
            "invalid fen",
            "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL",
            "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1 extra"
        ];

        for invalid_fen in invalid_fens {
            assert!(book.get_moves(invalid_fen).is_none());
            assert!(book.get_best_move(invalid_fen).is_none());
            assert!(book.get_random_move(invalid_fen).is_none());
        }
    }

    #[test]
    fn test_position_with_no_moves() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();

        book.add_position(fen.clone(), vec![]);
        book = book.mark_loaded();

        assert!(book.get_moves(&fen).is_some());
        assert!(book.get_best_move(&fen).is_none());
        assert!(book.get_random_move(&fen).is_none());
    }

    #[test]
    fn test_position_with_single_move() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        let best_move = book.get_best_move(&fen);
        let random_move = book.get_random_move(&fen);

        assert!(best_move.is_some());
        assert!(random_move.is_some());
        assert_eq!(best_move.unwrap().piece_type, PieceType::Rook);
        assert_eq!(random_move.unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_very_high_weight_moves() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                1000, // Maximum weight
                15
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                999,  // Very high weight
                10
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        let best_move = book.get_best_move(&fen);
        assert!(best_move.is_some());
        // Verify it's the correct move by checking piece type
        assert_eq!(best_move.unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_negative_evaluation() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                -10  // Negative evaluation
            )
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        let best_move = book.get_best_move(&fen);
        assert!(best_move.is_some());
        // Verify it's the correct move by checking piece type
        assert_eq!(best_move.unwrap().piece_type, PieceType::Rook);
    }
}
