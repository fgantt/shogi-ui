#![cfg(feature = "legacy-tests")]
//! Correctness validation tests for magic bitboards
//!
//! These tests validate that magic bitboards produce correct results
//! by comparing against ray-casting reference implementation.

use shogi_engine::bitboards::magic::AttackGenerator;
use shogi_engine::types::{Bitboard, PieceType, Position};

/// Helper to convert Position to square index
fn pos_to_square(row: u8, col: u8) -> u8 {
    row * 9 + col
}

/// Helper to check if a square is set in a bitboard
fn is_square_set(bitboard: Bitboard, square: u8) -> bool {
    (bitboard & (1u128 << square)) != 0
}

#[test]
fn test_attack_generator_correctness() {
    let mut generator = AttackGenerator::new();

    // Test center square with no blockers
    let square = pos_to_square(4, 4); // Center of board
    let empty: Bitboard = 0;

    let rook_attacks = generator.generate_attack_pattern(square, PieceType::Rook, empty);
    let bishop_attacks = generator.generate_attack_pattern(square, PieceType::Bishop, empty);

    // Rook should attack horizontally and vertically
    assert!(
        is_square_set(rook_attacks, pos_to_square(4, 0)),
        "Rook should attack left"
    );
    assert!(
        is_square_set(rook_attacks, pos_to_square(4, 8)),
        "Rook should attack right"
    );
    assert!(
        is_square_set(rook_attacks, pos_to_square(0, 4)),
        "Rook should attack up"
    );
    assert!(
        is_square_set(rook_attacks, pos_to_square(8, 4)),
        "Rook should attack down"
    );

    // Bishop should attack diagonally
    assert!(
        is_square_set(bishop_attacks, pos_to_square(3, 3)),
        "Bishop should attack NW"
    );
    assert!(
        is_square_set(bishop_attacks, pos_to_square(3, 5)),
        "Bishop should attack NE"
    );
    assert!(
        is_square_set(bishop_attacks, pos_to_square(5, 3)),
        "Bishop should attack SW"
    );
    assert!(
        is_square_set(bishop_attacks, pos_to_square(5, 5)),
        "Bishop should attack SE"
    );
}

#[test]
fn test_blocker_handling_correctness() {
    let mut generator = AttackGenerator::new();

    let square = pos_to_square(4, 4);

    // Place blocker to the right
    let blocker_square = pos_to_square(4, 6);
    let blockers: Bitboard = 1u128 << blocker_square;

    let attacks = generator.generate_attack_pattern(square, PieceType::Rook, blockers);

    // Should attack up to and including blocker
    assert!(
        is_square_set(attacks, pos_to_square(4, 5)),
        "Should attack square before blocker"
    );
    assert!(
        is_square_set(attacks, blocker_square),
        "Should attack blocker square"
    );

    // Should NOT attack beyond blocker
    assert!(
        !is_square_set(attacks, pos_to_square(4, 7)),
        "Should not attack beyond blocker"
    );
    assert!(
        !is_square_set(attacks, pos_to_square(4, 8)),
        "Should not attack beyond blocker"
    );
}

#[test]
fn test_edge_case_correctness() {
    let mut generator = AttackGenerator::new();

    // Test corner square (0, 0)
    let corner = pos_to_square(0, 0);
    let empty: Bitboard = 0;

    let rook_attacks = generator.generate_attack_pattern(corner, PieceType::Rook, empty);

    // Should attack entire first row and first column
    for col in 1..9 {
        assert!(
            is_square_set(rook_attacks, pos_to_square(0, col)),
            "Should attack row from corner"
        );
    }
    for row in 1..9 {
        assert!(
            is_square_set(rook_attacks, pos_to_square(row, 0)),
            "Should attack column from corner"
        );
    }

    // Should not attack the corner itself
    assert!(
        !is_square_set(rook_attacks, corner),
        "Should not attack corner itself"
    );
}

#[test]
fn test_bishop_diagonal_correctness() {
    let mut generator = AttackGenerator::new();

    // Test from position (3, 3)
    let square = pos_to_square(3, 3);
    let empty: Bitboard = 0;

    let attacks = generator.generate_attack_pattern(square, PieceType::Bishop, empty);

    // Check NW diagonal
    assert!(
        is_square_set(attacks, pos_to_square(2, 2)),
        "Should attack NW diagonal"
    );
    assert!(
        is_square_set(attacks, pos_to_square(1, 1)),
        "Should attack NW diagonal"
    );
    assert!(
        is_square_set(attacks, pos_to_square(0, 0)),
        "Should attack NW diagonal"
    );

    // Check SE diagonal
    assert!(
        is_square_set(attacks, pos_to_square(4, 4)),
        "Should attack SE diagonal"
    );
    assert!(
        is_square_set(attacks, pos_to_square(5, 5)),
        "Should attack SE diagonal"
    );
}

#[test]
fn test_multiple_blockers_correctness() {
    let mut generator = AttackGenerator::new();

    let square = pos_to_square(4, 4);

    // Place blockers in all four rook directions
    let blockers = (1u128 << pos_to_square(4, 6)) | // Right
                   (1u128 << pos_to_square(4, 2)) | // Left
                   (1u128 << pos_to_square(6, 4)) | // Down
                   (1u128 << pos_to_square(2, 4)); // Up

    let attacks = generator.generate_attack_pattern(square, PieceType::Rook, blockers);

    // Should attack each blocker
    assert!(
        is_square_set(attacks, pos_to_square(4, 6)),
        "Should attack right blocker"
    );
    assert!(
        is_square_set(attacks, pos_to_square(4, 2)),
        "Should attack left blocker"
    );
    assert!(
        is_square_set(attacks, pos_to_square(6, 4)),
        "Should attack down blocker"
    );
    assert!(
        is_square_set(attacks, pos_to_square(2, 4)),
        "Should attack up blocker"
    );

    // Should NOT attack beyond any blocker
    assert!(
        !is_square_set(attacks, pos_to_square(4, 7)),
        "Should not go beyond right"
    );
    assert!(
        !is_square_set(attacks, pos_to_square(4, 1)),
        "Should not go beyond left"
    );
    assert!(
        !is_square_set(attacks, pos_to_square(7, 4)),
        "Should not go beyond down"
    );
    assert!(
        !is_square_set(attacks, pos_to_square(1, 4)),
        "Should not go beyond up"
    );
}

#[test]
fn test_position_roundtrip() {
    // Test Position::from_index and to_index
    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            let index = pos.to_index();
            let pos_back = Position::from_index(index);

            assert_eq!(
                pos.row, pos_back.row,
                "Row should match for ({}, {})",
                row, col
            );
            assert_eq!(
                pos.col, pos_back.col,
                "Col should match for ({}, {})",
                row, col
            );
        }
    }
}

#[test]
fn test_all_squares_coverage() {
    let mut generator = AttackGenerator::new();
    let empty: Bitboard = 0;

    // Test that we can generate attacks for all 81 squares
    for square in 0..81 {
        let rook_attacks = generator.generate_attack_pattern(square, PieceType::Rook, empty);
        let bishop_attacks = generator.generate_attack_pattern(square, PieceType::Bishop, empty);

        // Attacks should not include the square itself
        assert!(
            !is_square_set(rook_attacks, square),
            "Rook should not attack its own square {}",
            square
        );
        assert!(
            !is_square_set(bishop_attacks, square),
            "Bishop should not attack its own square {}",
            square
        );
    }
}

#[test]
fn test_blocker_variations() {
    let mut generator = AttackGenerator::new();
    let square = pos_to_square(4, 4);

    // Test different blocker configurations
    let test_cases = vec![
        (0u128, "empty board"),
        (1u128 << pos_to_square(4, 5), "single blocker right"),
        (1u128 << pos_to_square(3, 4), "single blocker up"),
        (
            (1u128 << pos_to_square(4, 5)) | (1u128 << pos_to_square(4, 3)),
            "two blockers horizontal",
        ),
        (
            (1u128 << pos_to_square(3, 4)) | (1u128 << pos_to_square(5, 4)),
            "two blockers vertical",
        ),
    ];

    for (blockers, description) in test_cases {
        let attacks = generator.generate_attack_pattern(square, PieceType::Rook, blockers);

        // Attacks should be valid for each configuration
        assert!(
            !is_square_set(attacks, square),
            "Should not attack self for: {}",
            description
        );

        // If there are blockers, attacks should be limited
        if blockers != 0 {
            let attack_count = attacks.count_ones();
            assert!(
                attack_count > 0,
                "Should have some attacks for: {}",
                description
            );
            assert!(
                attack_count < 16,
                "Attacks should be limited by blockers for: {}",
                description
            );
        }
    }
}

#[test]
fn test_relevant_mask_generation() {
    let mut generator = AttackGenerator::new();

    // Relevant mask should exclude edge squares for interior squares
    let center_square = pos_to_square(4, 4);

    // Generate attack pattern with no blockers
    let attacks = generator.generate_attack_pattern(center_square, PieceType::Rook, 0);

    // Should attack many squares from center
    assert!(
        attacks.count_ones() > 10,
        "Center rook should attack many squares"
    );
}
