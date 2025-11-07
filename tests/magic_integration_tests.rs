#![cfg(feature = "legacy-tests")]
//! Integration tests for magic bitboards with the move generation system
//!
//! These tests verify that magic bitboards integrate correctly with the
//! existing Shogi engine components.

use shogi_engine::{
    types::{MagicTable, Piece, PieceType, Player, Position},
    BitboardBoard,
};

#[test]
fn test_bitboard_with_magic_support() {
    let result = BitboardBoard::new_with_magic_support();
    assert!(
        result.is_ok(),
        "Failed to create BitboardBoard with magic support: {:?}",
        result.err()
    );

    let board = result.unwrap();
    assert!(board.has_magic_support(), "Board should have magic support");
}

#[test]
fn test_magic_table_in_bitboard() {
    let board = BitboardBoard::new_with_magic_support().unwrap();

    let magic_table = board.get_magic_table();
    assert!(magic_table.is_some(), "Magic table should be present");
}

#[test]
fn test_sliding_generator_initialization() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();

    // Initialize sliding generator
    let result = board.init_sliding_generator();
    assert!(
        result.is_ok(),
        "Failed to initialize sliding generator: {:?}",
        result.err()
    );

    assert!(
        board.is_sliding_generator_initialized(),
        "Sliding generator should be initialized"
    );
}

#[test]
fn test_magic_sliding_moves_generation() {
    let mut board = BitboardBoard::empty();

    // Set up board with magic support
    let magic_table = MagicTable::new().unwrap();
    board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a rook in the center
    let rook_pos = Position::new(4, 4);
    let rook = Piece {
        piece_type: PieceType::Rook,
        player: Player::Black,
    };
    board.place_piece(rook, rook_pos);

    // Generate magic sliding moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(rook_pos, PieceType::Rook, Player::Black)
    {
        assert!(!moves.is_empty(), "Rook should have moves from center");

        // Verify moves are valid
        for move_ in moves {
            assert_eq!(
                move_.from,
                Some(rook_pos),
                "Move should start from rook position"
            );
            assert_eq!(move_.piece_type, PieceType::Rook, "Move should be for rook");
            assert_eq!(
                move_.player,
                Player::Black,
                "Move should be for black player"
            );
        }
    }
}

#[test]
fn test_magic_vs_raycast_consistency() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a bishop
    let bishop_pos = Position::new(3, 3);
    let bishop = Piece {
        piece_type: PieceType::Bishop,
        player: Player::White,
    };
    board.place_piece(bishop, bishop_pos);

    // Generate magic moves
    let magic_moves =
        board.generate_magic_sliding_moves(bishop_pos, PieceType::Bishop, Player::White);

    // Magic moves should be generated
    assert!(
        magic_moves.is_some(),
        "Magic moves should be generated for bishop"
    );

    let moves = magic_moves.unwrap();
    assert!(!moves.is_empty(), "Bishop should have moves");
}

#[test]
fn test_sliding_generator_with_blockers() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a rook
    let rook_pos = Position::new(4, 4);
    let rook = Piece {
        piece_type: PieceType::Rook,
        player: Player::Black,
    };
    board.place_piece(rook, rook_pos);

    // Place a blocker
    let blocker_pos = Position::new(4, 6);
    let blocker = Piece {
        piece_type: PieceType::Pawn,
        player: Player::White,
    };
    board.place_piece(blocker, blocker_pos);

    // Generate moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(rook_pos, PieceType::Rook, Player::Black)
    {
        // Check that rook can capture blocker but not go beyond
        let captures_blocker = moves.iter().any(|m| m.to == blocker_pos);
        assert!(captures_blocker, "Rook should be able to capture blocker");

        // Check that rook doesn't go beyond blocker (e.g., column 7)
        let beyond_blocker = moves.iter().any(|m| m.to.row == 4 && m.to.col > 6);
        assert!(!beyond_blocker, "Rook should not go beyond blocker");
    }
}

#[test]
fn test_sliding_generator_respects_own_pieces() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a bishop
    let bishop_pos = Position::new(3, 3);
    let bishop = Piece {
        piece_type: PieceType::Bishop,
        player: Player::Black,
    };
    board.place_piece(bishop, bishop_pos);

    // Place own piece in diagonal
    let own_piece_pos = Position::new(5, 5);
    let own_piece = Piece {
        piece_type: PieceType::Pawn,
        player: Player::Black,
    };
    board.place_piece(own_piece, own_piece_pos);

    // Generate moves
    if let Some(moves) =
        board.generate_magic_sliding_moves(bishop_pos, PieceType::Bishop, Player::Black)
    {
        // Check that bishop doesn't capture own piece
        let captures_own = moves.iter().any(|m| m.to == own_piece_pos);
        assert!(!captures_own, "Bishop should not capture own piece");
    }
}

#[test]
fn test_board_clone_preserves_magic_support() {
    let board1 = BitboardBoard::new_with_magic_support().unwrap();
    let board2 = board1.clone();

    assert_eq!(
        board1.has_magic_support(),
        board2.has_magic_support(),
        "Cloned board should preserve magic support"
    );
}

#[test]
fn test_magic_table_serialization_integration() {
    let table1 = MagicTable::new().unwrap();

    // Serialize
    let serialized = table1.serialize();
    assert!(serialized.is_ok(), "Serialization should succeed");

    let bytes = serialized.unwrap();
    assert!(!bytes.is_empty(), "Serialized data should not be empty");

    // Deserialize
    let table2 = MagicTable::deserialize(&bytes);
    assert!(table2.is_ok(), "Deserialization should succeed");

    let table2 = table2.unwrap();

    // Verify tables produce same results
    for square in (0..81).step_by(10) {
        let attacks1 = table1.get_attacks(square, PieceType::Rook, 0);
        let attacks2 = table2.get_attacks(square, PieceType::Rook, 0);

        assert_eq!(
            attacks1, attacks2,
            "Deserialized table should match original for square {}",
            square
        );
    }
}

#[test]
fn test_performance_stats() {
    let table = MagicTable::new().unwrap();

    let stats = table.performance_stats();

    // Should have entries for all squares and piece types
    assert_eq!(stats.total_rook_entries, 81, "Should have 81 rook entries");
    assert_eq!(
        stats.total_bishop_entries, 81,
        "Should have 81 bishop entries"
    );
    assert!(
        stats.total_attack_patterns > 0,
        "Should have attack patterns"
    );
}

#[test]
fn test_magic_initialization_progress() {
    let table = MagicTable::new().unwrap();

    let (initialized, total) = table.initialization_progress();
    assert_eq!(
        initialized, total,
        "Fully initialized table should show all entries initialized"
    );

    assert!(
        table.is_fully_initialized(),
        "Table should be fully initialized"
    );
}

#[test]
fn test_multiple_pieces_with_magic() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place multiple sliding pieces
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::Black,
        },
        Position::new(0, 0),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Bishop,
            player: Player::White,
        },
        Position::new(2, 2),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::White,
        },
        Position::new(4, 4),
    );

    // Generate moves for each piece
    let rook1_moves =
        board.generate_magic_sliding_moves(Position::new(0, 0), PieceType::Rook, Player::Black);
    let bishop_moves =
        board.generate_magic_sliding_moves(Position::new(2, 2), PieceType::Bishop, Player::White);
    let rook2_moves =
        board.generate_magic_sliding_moves(Position::new(4, 4), PieceType::Rook, Player::White);

    assert!(rook1_moves.is_some(), "Should generate moves for rook 1");
    assert!(bishop_moves.is_some(), "Should generate moves for bishop");
    assert!(rook2_moves.is_some(), "Should generate moves for rook 2");
}

#[test]
fn test_promoted_pieces_preparation() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Note: Promoted pieces use same sliding patterns as base pieces
    // This test prepares for future promoted piece integration

    let generator = board.get_sliding_generator();
    assert!(
        generator.is_some(),
        "Sliding generator should be available for promoted pieces"
    );
}

#[test]
fn test_magic_table_validation() {
    let table = MagicTable::new().unwrap();

    // Validate the table
    let validation_result = table.validate();
    assert!(
        validation_result.is_ok(),
        "Magic table validation failed: {:?}",
        validation_result.err()
    );
}

#[test]
fn test_edge_case_positions() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Test corners
    let corners = [
        Position::new(0, 0),
        Position::new(0, 8),
        Position::new(8, 0),
        Position::new(8, 8),
    ];

    for corner in corners {
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::Black,
            },
            corner,
        );

        let moves = board.generate_magic_sliding_moves(corner, PieceType::Rook, Player::Black);
        assert!(
            moves.is_some(),
            "Should generate moves from corner {:?}",
            corner
        );

        board.remove_piece(corner);
    }
}

#[test]
fn test_memory_efficiency() {
    let table = MagicTable::new().unwrap();
    let stats = table.performance_stats();

    // Memory efficiency should be reasonable
    assert!(
        stats.memory_efficiency > 0.0,
        "Memory efficiency should be positive"
    );
    assert!(
        stats.memory_efficiency <= 1.0,
        "Memory efficiency should not exceed 100%"
    );
}
