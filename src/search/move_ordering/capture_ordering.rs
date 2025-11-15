//! Capture move ordering
//!
//! This module contains MVV/LVA (Most Valuable Victim / Least Valuable Attacker)
//! and capture move ordering implementation.

use crate::types::core::{Move, PieceType, Position};

/// Score a capture move using MVV/LVA (Most Valuable Victim / Least Valuable Attacker)
///
/// This function scores capture moves based on the value of the captured piece
/// and the value of the capturing piece. Higher scores are given for capturing
/// valuable pieces with less valuable pieces.
///
/// # Arguments
/// * `move_` - The move to score
/// * `capture_weight` - Base weight for capture moves
///
/// # Returns
/// Score for the capture move, or 0 if not a capture
pub fn score_capture_move(move_: &Move, capture_weight: i32) -> i32 {
    if !move_.is_capture {
        return 0;
    }

    let mut score = capture_weight;

    // Add value of captured piece
    if let Some(captured) = &move_.captured_piece {
        score += captured.piece_type.base_value();

        // Bonus for capturing higher-value pieces
        match captured.piece_type {
            PieceType::King => score += 1000,
            PieceType::Rook => score += 500,
            PieceType::Bishop => score += 300,
            PieceType::Gold => score += 200,
            PieceType::Silver => score += 150,
            PieceType::Knight => score += 100,
            PieceType::Lance => score += 80,
            PieceType::Pawn => score += 50,
            // Promoted pieces
            PieceType::PromotedPawn => score += 250,
            PieceType::PromotedLance => score += 230,
            PieceType::PromotedKnight => score += 210,
            PieceType::PromotedSilver => score += 200,
            PieceType::PromotedBishop => score += 350,
            PieceType::PromotedRook => score += 550,
        }
    }

    // Bonus for capturing with lower-value pieces (good exchange)
    match move_.piece_type {
        PieceType::Pawn => score += 100,
        PieceType::Lance => score += 80,
        PieceType::Knight => score += 60,
        PieceType::Silver => score += 40,
        PieceType::Gold => score += 30,
        PieceType::Bishop => score += 20,
        PieceType::Rook => score += 10,
        PieceType::King => score += 5,
        // Promoted pieces
        PieceType::PromotedPawn => score += 110,
        PieceType::PromotedLance => score += 90,
        PieceType::PromotedKnight => score += 70,
        PieceType::PromotedSilver => score += 50,
        PieceType::PromotedBishop => score += 30,
        PieceType::PromotedRook => score += 20,
    }

    score
}

/// Score a promotion move
///
/// Promotions are strategic moves that can significantly change
/// the value and capabilities of a piece.
///
/// # Arguments
/// * `move_` - The move to score
/// * `promotion_weight` - Base weight for promotion moves
/// * `score_position_value` - Function to score position value (for center bonus)
///
/// # Returns
/// Score for the promotion move, or 0 if not a promotion
pub fn score_promotion_move(
    move_: &Move,
    promotion_weight: i32,
    score_position_value: impl FnOnce(&Position) -> i32,
) -> i32 {
    if !move_.is_promotion {
        return 0;
    }

    let mut score = promotion_weight;

    // Add promotion value
    score += move_.promotion_value();

    // Bonus for promoting to more valuable pieces
    match move_.piece_type {
        PieceType::Pawn => score += 200, // Pawn to Gold is very valuable
        PieceType::Lance => score += 180,
        PieceType::Knight => score += 160,
        PieceType::Silver => score += 140,
        PieceType::Gold => score += 120,
        PieceType::Bishop => score += 120,
        PieceType::Rook => score += 100,
        PieceType::King => score += 50,
        // Promoted pieces
        PieceType::PromotedPawn => score += 220,
        PieceType::PromotedLance => score += 200,
        PieceType::PromotedKnight => score += 180,
        PieceType::PromotedSilver => score += 160,
        PieceType::PromotedBishop => score += 140,
        PieceType::PromotedRook => score += 120,
    }

    // Bonus for promoting in center or near enemy king
    let center_bonus = score_position_value(&move_.to);
    score += center_bonus / 2;

    score
}

/// Inline capture move scoring for hot path optimization
///
/// Optimized version using MVV-LVA (Most Valuable Victim - Least Valuable Attacker).
/// This version is optimized for performance in hot paths.
///
/// # Arguments
/// * `move_` - The move to score
/// * `capture_weight` - Base weight for capture moves
///
/// # Returns
/// Score for the capture move, or 0 if not a capture
pub fn score_capture_move_inline(move_: &Move, capture_weight: i32) -> i32 {
    if let Some(captured_piece) = &move_.captured_piece {
        // MVV-LVA: Most Valuable Victim - Least Valuable Attacker
        let victim_value = captured_piece.piece_type.base_value();
        let attacker_value = move_.piece_type.base_value();

        // Scale the score based on the exchange value
        let exchange_value = victim_value - attacker_value;
        capture_weight + exchange_value / 10
    } else {
        0
    }
}

/// Inline promotion move scoring for hot path optimization
///
/// Optimized version for performance in hot paths.
///
/// # Arguments
/// * `move_` - The move to score
/// * `promotion_weight` - Base weight for promotion moves
/// * `get_center_distance` - Function to get center distance
///
/// # Returns
/// Score for the promotion move, or 0 if not a promotion
pub fn score_promotion_move_inline(
    move_: &Move,
    promotion_weight: i32,
    get_center_distance: impl FnOnce(&Position) -> i32,
) -> i32 {
    if move_.is_promotion {
        // Base promotion bonus
        let mut score = promotion_weight;

        // Bonus for promoting to center squares
        let center_distance = get_center_distance(&move_.to);
        if center_distance <= 1 {
            score += 50;
        }

        score
    } else {
        0
    }
}

/// Calculate MVV/LVA score for a capture
///
/// MVV/LVA (Most Valuable Victim / Least Valuable Attacker) prioritizes
/// captures where a less valuable piece captures a more valuable piece.
///
/// # Arguments
/// * `victim_value` - Base value of the captured piece
/// * `attacker_value` - Base value of the capturing piece
/// * `capture_weight` - Base weight for capture moves
///
/// # Returns
/// MVV/LVA score for the capture
pub fn calculate_mvv_lva_score(victim_value: i32, attacker_value: i32, capture_weight: i32) -> i32 {
    let exchange_value = victim_value - attacker_value;
    capture_weight + exchange_value / 10
}

/// Get capture value bonus for a piece type
///
/// Returns the bonus value for capturing a piece of the given type.
///
/// # Arguments
/// * `piece_type` - The piece type that was captured
///
/// # Returns
/// Capture bonus value
pub fn get_capture_bonus(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::King => 1000,
        PieceType::Rook => 500,
        PieceType::Bishop => 300,
        PieceType::Gold => 200,
        PieceType::Silver => 150,
        PieceType::Knight => 100,
        PieceType::Lance => 80,
        PieceType::Pawn => 50,
        // Promoted pieces
        PieceType::PromotedPawn => 250,
        PieceType::PromotedLance => 230,
        PieceType::PromotedKnight => 210,
        PieceType::PromotedSilver => 200,
        PieceType::PromotedBishop => 350,
        PieceType::PromotedRook => 550,
    }
}

/// Get attacker bonus for capturing with a piece type
///
/// Returns the bonus for capturing with a less valuable piece (good exchange).
///
/// # Arguments
/// * `piece_type` - The piece type doing the capturing
///
/// # Returns
/// Attacker bonus value
pub fn get_attacker_bonus(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,
        PieceType::Lance => 80,
        PieceType::Knight => 60,
        PieceType::Silver => 40,
        PieceType::Gold => 30,
        PieceType::Bishop => 20,
        PieceType::Rook => 10,
        PieceType::King => 5,
        // Promoted pieces
        PieceType::PromotedPawn => 110,
        PieceType::PromotedLance => 90,
        PieceType::PromotedKnight => 70,
        PieceType::PromotedSilver => 50,
        PieceType::PromotedBishop => 30,
        PieceType::PromotedRook => 20,
    }
}
