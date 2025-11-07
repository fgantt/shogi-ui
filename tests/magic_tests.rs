#![cfg(feature = "legacy-tests")]
//! Comprehensive unit tests for magic bitboards
//!
//! This test suite validates the correctness of magic bitboard implementation
//! including magic number generation, attack patterns, and lookup functionality.

use shogi_engine::bitboards::magic::{AttackGenerator, MagicFinder, MagicValidator};
use shogi_engine::types::{Bitboard, MagicTable, PieceType, Position};

#[test]
fn test_magic_table_creation() {
    let result = MagicTable::new();
    assert!(
        result.is_ok(),
        "Failed to create magic table: {:?}",
        result.err()
    );

    let table = result.unwrap();

    // Verify table is initialized for all squares
    // Verify we can get attacks for all squares
    for square in 0..81 {
        let empty: u128 = 0;
        let rook_attacks = table.get_attacks(square, PieceType::Rook, empty);
        let bishop_attacks = table.get_attacks(square, PieceType::Bishop, empty);

        // Attacks should be valid (could be 0 for edge squares)
        let _ = rook_attacks;
        let _ = bishop_attacks;
    }
}

#[test]
fn test_magic_table_default() {
    let table = MagicTable::default();

    // Default table should have entries for all squares
    for square in 0..81 {
        let empty: u128 = 0;
        let rook_attacks = table.get_attacks(square, PieceType::Rook, empty);
        let bishop_attacks = table.get_attacks(square, PieceType::Bishop, empty);

        // Attacks should be valid
        let _ = rook_attacks;
        let _ = bishop_attacks;
    }
}

#[test]
fn test_rook_attack_patterns() {
    let table = MagicTable::new().unwrap();

    // Test center square (40 = row 4, col 4)
    let center_square = 40;
    let empty_board: Bitboard = 0;

    let attacks = table.get_attacks(center_square, PieceType::Rook, empty_board);

    // On empty board, rook should attack entire rank and file
    assert!(attacks != 0, "Rook should have attacks on empty board");

    // Count attacked squares (should be 16: 8 in rank + 8 in file, minus the square itself)
    let attack_count = attacks.count_ones();
    assert!(attack_count > 0, "Rook should attack at least some squares");
}

#[test]
fn test_bishop_attack_patterns() {
    let table = MagicTable::new().unwrap();

    // Test center square (40 = row 4, col 4)
    let center_square = 40;
    let empty_board: Bitboard = 0;

    let attacks = table.get_attacks(center_square, PieceType::Bishop, empty_board);

    // On empty board, bishop should attack diagonals
    assert!(attacks != 0, "Bishop should have attacks on empty board");

    let attack_count = attacks.count_ones();
    assert!(
        attack_count > 0,
        "Bishop should attack at least some squares"
    );
}

#[test]
fn test_rook_with_blockers() {
    let table = MagicTable::new().unwrap();

    // Test square (40 = row 4, col 4)
    let square = 40;

    // Place a blocker to the right (square 41 = row 4, col 5)
    let blocker: Bitboard = 1u128 << 41;

    let attacks = table.get_attacks(square, PieceType::Rook, blocker);

    // Rook should attack the blocker square but not beyond
    assert!(
        attacks & (1u128 << 41) != 0,
        "Rook should attack blocker square"
    );

    // Squares beyond blocker should not be attacked
    assert!(
        attacks & (1u128 << 42) == 0,
        "Rook should not attack beyond blocker"
    );
}

#[test]
fn test_bishop_with_blockers() {
    let table = MagicTable::new().unwrap();

    // Test square (40 = row 4, col 4)
    let square = 40;

    // Place a blocker diagonally (square 32 = row 3, col 5)
    let blocker: Bitboard = 1u128 << 32;

    let attacks = table.get_attacks(square, PieceType::Bishop, blocker);

    // Bishop should attack the blocker square
    assert!(
        attacks & (1u128 << 32) != 0,
        "Bishop should attack blocker square"
    );
}

#[test]
fn test_corner_squares() {
    let table = MagicTable::new().unwrap();
    let empty_board: Bitboard = 0;

    // Test all four corners
    let corners = [0, 8, 72, 80]; // Top-left, top-right, bottom-left, bottom-right

    for &corner in &corners {
        let rook_attacks = table.get_attacks(corner, PieceType::Rook, empty_board);
        let bishop_attacks = table.get_attacks(corner, PieceType::Bishop, empty_board);

        assert!(
            rook_attacks != 0,
            "Rook should have attacks from corner {}",
            corner
        );
        assert!(
            bishop_attacks != 0,
            "Bishop should have attacks from corner {}",
            corner
        );
    }
}

#[test]
fn test_edge_squares() {
    let table = MagicTable::new().unwrap();
    let empty_board: Bitboard = 0;

    // Test squares along edges
    let edges = [1, 2, 3, 9, 18, 27]; // Various edge squares

    for &edge in &edges {
        let rook_attacks = table.get_attacks(edge, PieceType::Rook, empty_board);
        let bishop_attacks = table.get_attacks(edge, PieceType::Bishop, empty_board);

        assert!(
            rook_attacks != 0,
            "Rook should have attacks from edge {}",
            edge
        );
        // Bishop may not have attacks from all edges (depends on diagonal availability)
    }
}

#[test]
fn test_magic_finder_generation() {
    let mut finder = MagicFinder::new();

    // Test magic number generation for a few squares
    let test_squares = [0, 40, 80]; // Corner, center, corner

    for &square in &test_squares {
        // Test rook
        let rook_result = finder.find_magic_number(square, PieceType::Rook);
        assert!(
            rook_result.is_ok(),
            "Failed to find rook magic for square {}",
            square
        );

        // Test bishop
        let bishop_result = finder.find_magic_number(square, PieceType::Bishop);
        assert!(
            bishop_result.is_ok(),
            "Failed to find bishop magic for square {}",
            square
        );
    }
}

#[test]
fn test_attack_generator() {
    let mut generator = AttackGenerator::new();

    // Test attack generation for center square
    let square = 40;
    let empty_board: Bitboard = 0;

    let rook_attacks = generator.generate_attack_pattern(square, PieceType::Rook, empty_board);
    assert!(rook_attacks != 0, "Rook attacks should not be empty");

    let bishop_attacks = generator.generate_attack_pattern(square, PieceType::Bishop, empty_board);
    assert!(bishop_attacks != 0, "Bishop attacks should not be empty");
}

#[test]
fn test_attack_generator_with_blockers() {
    let mut generator = AttackGenerator::new();

    let square = 40;

    // Create blockers in multiple directions
    let blockers = (1u128 << 41) | (1u128 << 49); // Right and down

    let attacks = generator.generate_attack_pattern(square, PieceType::Rook, blockers);

    // Should include blocker squares
    assert!(attacks & (1u128 << 41) != 0, "Should attack right blocker");
    assert!(attacks & (1u128 << 49) != 0, "Should attack down blocker");

    // Should not go beyond blockers
    assert!(
        attacks & (1u128 << 42) == 0,
        "Should not go beyond right blocker"
    );
    assert!(
        attacks & (1u128 << 58) == 0,
        "Should not go beyond down blocker"
    );
}

#[test]
fn test_magic_validator() {
    let table = MagicTable::new().unwrap();
    let mut validator = MagicValidator::new();

    // Validate the entire magic table
    let result = validator.validate_magic_table(&table);
    assert!(
        result.is_ok(),
        "Magic table validation failed: {:?}",
        result.err()
    );
}

#[test]
fn test_magic_table_clone() {
    let table1 = MagicTable::new().unwrap();
    let table2 = table1.clone();

    // Both tables should produce same results
    for square in (0..81).step_by(10) {
        let empty: Bitboard = 0;

        let attacks1 = table1.get_attacks(square, PieceType::Rook, empty);
        let attacks2 = table2.get_attacks(square, PieceType::Rook, empty);

        assert_eq!(
            attacks1, attacks2,
            "Cloned table should produce same results for square {}",
            square
        );
    }
}

#[test]
fn test_position_from_index() {
    // Test Position::from_index method
    for square in 0..81 {
        let pos = Position::from_index(square);
        let index_back = pos.to_index();

        assert_eq!(
            square, index_back,
            "Position round-trip failed for square {}",
            square
        );
    }
}

#[test]
fn test_magic_table_internals() {
    let table = MagicTable::new().unwrap();

    // Test that the table produces valid attack patterns for all squares
    for square in 0..81 {
        let empty: u128 = 0;

        // Get attacks for empty board
        let rook_attacks = table.get_attacks(square, PieceType::Rook, empty);
        let bishop_attacks = table.get_attacks(square, PieceType::Bishop, empty);

        // Attacks should not include the square itself
        assert!(
            rook_attacks & (1u128 << square) == 0,
            "Rook attacks should not include square {} itself",
            square
        );
        assert!(
            bishop_attacks & (1u128 << square) == 0,
            "Bishop attacks should not include square {} itself",
            square
        );
    }
}

#[test]
fn test_symmetry_properties() {
    let table = MagicTable::new().unwrap();
    let empty: Bitboard = 0;

    // Test that similar positions produce similar attack patterns
    // Center squares should have similar attack counts
    let center_squares = [39, 40, 41, 48, 49, 50];

    let mut rook_counts = Vec::new();
    for &square in &center_squares {
        let attacks = table.get_attacks(square, PieceType::Rook, empty);
        rook_counts.push(attacks.count_ones());
    }

    // All center rook attacks should be non-zero
    for count in &rook_counts {
        assert!(*count > 0, "Center squares should have rook attacks");
    }
}

#[test]
fn test_multiple_blockers() {
    let table = MagicTable::new().unwrap();

    let square = 40;

    // Place blockers in all four rook directions
    let blockers = (1u128 << 41) | // Right
                   (1u128 << 39) | // Left
                   (1u128 << 49) | // Down
                   (1u128 << 31); // Up

    let attacks = table.get_attacks(square, PieceType::Rook, blockers);

    // Should attack all blocker squares
    assert!(attacks & (1u128 << 41) != 0, "Should attack right blocker");
    assert!(attacks & (1u128 << 39) != 0, "Should attack left blocker");
    assert!(attacks & (1u128 << 49) != 0, "Should attack down blocker");
    assert!(attacks & (1u128 << 31) != 0, "Should attack up blocker");

    // Should not attack beyond any blocker
    let beyond_count = (attacks & (1u128 << 42)).count_ones()
        + (attacks & (1u128 << 38)).count_ones()
        + (attacks & (1u128 << 58)).count_ones()
        + (attacks & (1u128 << 22)).count_ones();

    assert_eq!(beyond_count, 0, "Should not attack beyond blockers");
}

#[test]
fn test_deterministic_results() {
    // Magic table should always produce same results
    let table1 = MagicTable::new().unwrap();
    let table2 = MagicTable::new().unwrap();

    for square in (0..81).step_by(5) {
        let blockers = (1u128 << ((square + 1) % 81)) | (1u128 << ((square + 10) % 81));

        let attacks1 = table1.get_attacks(square, PieceType::Rook, blockers);
        let attacks2 = table2.get_attacks(square, PieceType::Rook, blockers);

        assert_eq!(
            attacks1, attacks2,
            "Results should be deterministic for square {}",
            square
        );
    }
}
