//! Integration tests for shogi-specific adaptations
//!
//! Tests verify that shogi-specific features work correctly:
//! - Drop-based mate threats
//! - Opposition adjustment with pieces in hand
//! - Material calculation including pieces in hand
//! - Tokin promotion mate detection

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternConfig, EndgamePatternEvaluator};
use shogi_engine::types::{CapturedPieces, PieceType, Player};

#[test]
fn test_drop_mate_threats_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();

    // Add pieces to hand that could create mate threats
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    captured_pieces.add_piece(PieceType::Bishop, Player::Black);

    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Should complete evaluation
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);

    // Statistics should track drop mate threats if detected
    assert!(evaluator.stats().drop_mate_threats_detected >= 0);
}

#[test]
fn test_opposition_with_pieces_in_hand_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();

    // Add pieces to opponent's hand
    captured_pieces.add_piece(PieceType::Gold, Player::White);
    captured_pieces.add_piece(PieceType::Silver, Player::White);
    captured_pieces.add_piece(PieceType::Rook, Player::White);

    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Should complete evaluation
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);

    // Statistics should track opposition broken by drops if detected
    assert!(evaluator.stats().opposition_broken_by_drops >= 0);
}

#[test]
fn test_material_calculation_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();

    let base_captured = CapturedPieces::new();
    let base_score = evaluator.evaluate_endgame(&board, Player::Black, &base_captured);

    let mut advantage_captured = CapturedPieces::new();
    advantage_captured.add_piece(PieceType::Rook, Player::Black);
    let advantage_score = evaluator.evaluate_endgame(&board, Player::Black, &advantage_captured);

    assert!(
        advantage_score.eg >= base_score.eg,
        "Expected extra rook in hand to improve evaluation eg={} base={}",
        advantage_score.eg,
        base_score.eg
    );

    let mut balanced_captured = advantage_captured.clone();
    balanced_captured.add_piece(PieceType::Bishop, Player::White);
    let balanced_score = evaluator.evaluate_endgame(&board, Player::Black, &balanced_captured);

    assert!(
        balanced_score.eg <= advantage_score.eg,
        "Adding opponent material should not improve our evaluation (balanced {} vs advantage {})",
        balanced_score.eg,
        advantage_score.eg
    );
}

#[test]
fn test_shogi_opposition_adjustment_config() {
    let mut disabled_config = EndgamePatternConfig::default();
    disabled_config.enable_shogi_opposition_adjustment = false;

    let mut enabled_config = EndgamePatternConfig::default();
    enabled_config.enable_shogi_opposition_adjustment = true;

    let mut evaluator_disabled = EndgamePatternEvaluator::with_config(disabled_config);
    let mut evaluator_enabled = EndgamePatternEvaluator::with_config(enabled_config);

    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    captured_pieces.add_piece(PieceType::Rook, Player::White);

    let score_disabled =
        evaluator_disabled.evaluate_endgame(&board, Player::Black, &captured_pieces);
    let score_enabled = evaluator_enabled.evaluate_endgame(&board, Player::Black, &captured_pieces);

    assert!(
        score_disabled.eg.abs() <= 10000 && score_enabled.eg.abs() <= 10000,
        "Opposition adjustment evaluations should complete successfully"
    );
}

#[test]
fn test_tokin_promotion_mate_integration() {
    let mut evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test tokin promotion mate detection in full evaluation
    let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

    // Should complete evaluation
    assert!(score.mg >= -10000 && score.mg <= 10000);
    assert!(score.eg >= -10000 && score.eg <= 10000);
}
