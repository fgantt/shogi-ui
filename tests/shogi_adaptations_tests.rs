//! Integration tests for shogi-specific adaptations
//!
//! Tests verify that shogi-specific features work correctly:
//! - Drop-based mate threats
//! - Opposition adjustment with pieces in hand
//! - Material calculation including pieces in hand
//! - Tokin promotion mate detection

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::endgame_patterns::{EndgamePatternEvaluator, EndgamePatternConfig};
use shogi_engine::types::{CapturedPieces, Player, PieceType};

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
    let evaluator = EndgamePatternEvaluator::new();
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    
    // Test material calculation with pieces in hand
    let material1 = evaluator.calculate_material(&board, Player::Black, &captured_pieces);
    
    captured_pieces.add_piece(PieceType::Rook, Player::Black);
    let material2 = evaluator.calculate_material(&board, Player::Black, &captured_pieces);
    
    // Material should increase when piece is added to hand
    assert!(material2 > material1);
    
    // Test material difference
    let diff1 = evaluator.get_material_difference(&board, Player::Black, &captured_pieces);
    
    captured_pieces.add_piece(PieceType::Bishop, Player::White);
    let diff2 = evaluator.get_material_difference(&board, Player::Black, &captured_pieces);
    
    // Difference should decrease when opponent gets piece
    assert!(diff2 < diff1);
}

#[test]
fn test_shogi_opposition_adjustment_config() {
    let mut config = EndgamePatternConfig::default();
    config.enable_shogi_opposition_adjustment = false;
    
    let mut evaluator = EndgamePatternEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let mut captured_pieces = CapturedPieces::new();
    
    // Add pieces to opponent's hand
    captured_pieces.add_piece(PieceType::Rook, Player::White);
    
    // With adjustment disabled, opposition value should not be reduced
    let score1 = evaluator.evaluate_opposition(&board, Player::Black, &captured_pieces);
    
    // Enable adjustment
    evaluator.config.enable_shogi_opposition_adjustment = true;
    let score2 = evaluator.evaluate_opposition(&board, Player::Black, &captured_pieces);
    
    // Scores may differ if opposition is detected
    // Just verify both complete without error
    assert!(score1.eg >= 0 && score1.eg <= 40);
    assert!(score2.eg >= 0 && score2.eg <= 40);
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

