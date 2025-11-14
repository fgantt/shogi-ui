//! Tests for opening principles and opening book integration (Task 19.0 - Task 3.0)
//!
//! These tests verify that opening principles correctly evaluate and prioritize book moves:
//! - Book move quality evaluation
//! - Book move prioritization
//! - Book move validation
//! - Integration between book and principles

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::opening_principles::OpeningPrincipleEvaluator;
use shogi_engine::opening_book::{BookMove, OpeningBook, PositionEntry};
use shogi_engine::types::*;

/// Test book move quality evaluation
#[test]
fn test_book_move_quality_evaluation() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Create a simple move (pawn push)
    let move_ = Move {
        from: Some(Position::new(6, 4)), // Black pawn at starting position
        to: Position::new(5, 4),         // Push forward
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: false,
        is_recapture: false,
    };

    // Evaluate move quality
    let quality_score =
        evaluator.evaluate_book_move_quality(&board, Player::Black, &move_, &captured_pieces, 5);

    // Should return a valid score
    assert!(
        quality_score.abs() < 100000,
        "Quality score should be reasonable"
    );

    // Check statistics
    assert_eq!(evaluator.stats().book_moves_evaluated, 1);
}

/// Test book move validation
#[test]
fn test_book_move_validation() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();

    // Test valid move (pawn push)
    let valid_move = Move {
        from: Some(Position::new(6, 4)),
        to: Position::new(5, 4),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: false,
        is_recapture: false,
    };

    let is_valid = evaluator.validate_book_move(&board, Player::Black, &valid_move, 5);
    assert!(is_valid, "Valid pawn push should pass validation");

    // Check statistics
    assert_eq!(evaluator.stats().book_moves_validated, 1);
}

/// Test book move prioritization
#[test]
fn test_book_move_prioritization() {
    use shogi_engine::opening_book::OpeningBook;

    let mut book = OpeningBook::new();
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Create a position entry with multiple moves
    let fen = board.to_fen(Player::Black, &captured_pieces);

    // Create some test moves
    let move1 = BookMove::new(
        Some(Position::new(6, 4)),
        Position::new(5, 4),
        PieceType::Pawn,
        false,
        false,
        100,
        0,
    );

    let move2 = BookMove::new(
        Some(Position::new(7, 1)),
        Position::new(6, 1),
        PieceType::Silver,
        false,
        false,
        80,
        0,
    );

    let entry = PositionEntry::new(fen.clone(), vec![move1.clone(), move2.clone()]);
    book.add_position(fen.clone(), entry.moves.clone());

    // Get best move with principles
    let best_move =
        book.get_best_move_with_principles(&fen, &board, &captured_pieces, 5, Some(&mut evaluator));

    // Should return a move
    assert!(best_move.is_some(), "Should return a prioritized move");

    // Check statistics
    assert!(evaluator.stats().book_moves_evaluated >= 2);
    assert_eq!(evaluator.stats().book_moves_prioritized, 1);
}

/// Test opening book and principles coordination
#[test]
fn test_opening_book_principles_coordination() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that evaluator can work with book moves
    let move_ = Move {
        from: Some(Position::new(6, 4)),
        to: Position::new(5, 4),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: false,
        is_recapture: false,
    };

    // Evaluate quality
    let _quality1 =
        evaluator.evaluate_book_move_quality(&board, Player::Black, &move_, &captured_pieces, 5);

    // Validate move
    let is_valid = evaluator.validate_book_move(&board, Player::Black, &move_, 5);
    assert!(is_valid);

    // Check that statistics are tracked
    let stats = evaluator.stats();
    assert!(stats.book_moves_evaluated > 0);
    assert!(stats.book_moves_validated > 0);
}

/// Test that quality scores are tracked in statistics
#[test]
fn test_quality_score_statistics() {
    let mut evaluator = OpeningPrincipleEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let move_ = Move {
        from: Some(Position::new(6, 4)),
        to: Position::new(5, 4),
        piece_type: PieceType::Pawn,
        player: Player::Black,
        is_promotion: false,
        is_capture: false,
        captured_piece: None,
        gives_check: false,
        is_recapture: false,
    };

    let quality_score =
        evaluator.evaluate_book_move_quality(&board, Player::Black, &move_, &captured_pieces, 5);

    // Check that score is tracked
    let stats = evaluator.stats();
    assert_eq!(stats.book_moves_evaluated, 1);
    assert_eq!(stats.book_move_quality_scores, quality_score as i64);
}

/// Test fallback to weight-based selection when evaluator is None
#[test]
fn test_fallback_to_weight_based_selection() {
    use shogi_engine::opening_book::OpeningBook;

    let mut book = OpeningBook::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let fen = board.to_fen(Player::Black, &captured_pieces);

    // Create a move with weight
    let move1 = BookMove::new(
        Some(Position::new(6, 4)),
        Position::new(5, 4),
        PieceType::Pawn,
        false,
        false,
        100, // High weight
        0,
    );

    let entry = PositionEntry::new(fen.clone(), vec![move1.clone()]);
    book.add_position(fen.clone(), entry.moves.clone());

    // Get best move without evaluator (should fall back to weight-based)
    let best_move = book.get_best_move_with_principles(
        &fen,
        &board,
        &captured_pieces,
        5,
        None, // No evaluator
    );

    // Should still return a move (using weight-based selection)
    assert!(best_move.is_some());
}
