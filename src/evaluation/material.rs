//! Material Evaluation Module
//!
//! This module provides phase-aware material evaluation for the Shogi engine.
//! Material values differ between opening/middlegame and endgame phases, providing
//! more accurate position assessment throughout the game.
//!
//! # Overview
//!
//! The material evaluation system:
//! - Assigns different values to pieces in opening vs endgame
//! - Handles promoted pieces appropriately
//! - Evaluates captured pieces (pieces in hand)
//! - Calculates material balance for both players
//! - Integrates seamlessly with tapered evaluation
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::material::MaterialEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let evaluator = MaterialEvaluator::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
//! ```

use crate::types::*;
use crate::bitboards::BitboardBoard;
use serde::{Deserialize, Serialize};

/// Material evaluator with phase-aware piece values
pub struct MaterialEvaluator {
    /// Configuration for material evaluation
    config: MaterialEvaluationConfig,
    /// Statistics for monitoring
    stats: MaterialEvaluationStats,
}

impl MaterialEvaluator {
    /// Create a new MaterialEvaluator with default configuration
    pub fn new() -> Self {
        Self {
            config: MaterialEvaluationConfig::default(),
            stats: MaterialEvaluationStats::default(),
        }
    }

    /// Create a new MaterialEvaluator with custom configuration
    pub fn with_config(config: MaterialEvaluationConfig) -> Self {
        Self {
            config,
            stats: MaterialEvaluationStats::default(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MaterialEvaluationConfig {
        &self.config
    }

    /// Evaluate material for a player
    ///
    /// Returns a TaperedScore with middlegame and endgame material values
    pub fn evaluate_material(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut score = TaperedScore::default();

        // Evaluate pieces on board
        score += self.evaluate_board_material(board, player);

        // Evaluate captured pieces (pieces in hand)
        if self.config.include_hand_pieces {
            score += self.evaluate_hand_material(captured_pieces, player);
        }

        score
    }

    /// Evaluate material for pieces on the board
    fn evaluate_board_material(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value = self.get_piece_value(piece.piece_type);

                    if piece.player == player {
                        score += piece_value;
                    } else {
                        score -= piece_value;
                    }
                }
            }
        }

        score
    }

    /// Evaluate material for captured pieces (pieces in hand)
    fn evaluate_hand_material(&self, captured_pieces: &CapturedPieces, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();

        // Get captured pieces for this player
        let player_captures = match player {
            Player::Black => &captured_pieces.black,
            Player::White => &captured_pieces.white,
        };

        // Get opponent's captured pieces
        let opponent_captures = match player {
            Player::Black => &captured_pieces.white,
            Player::White => &captured_pieces.black,
        };

        // Add value for pieces we can drop
        for &piece_type in player_captures {
            score += self.get_hand_piece_value(piece_type);
        }

        // Subtract value for pieces opponent can drop
        for &piece_type in opponent_captures {
            score -= self.get_hand_piece_value(piece_type);
        }

        score
    }

    /// Get tapered value for a piece on the board
    ///
    /// Returns a TaperedScore with separate mg/eg values
    pub fn get_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            // Basic pieces - values from research and engine testing
            PieceType::Pawn => TaperedScore::new_tapered(100, 120),
            PieceType::Lance => TaperedScore::new_tapered(300, 280),
            PieceType::Knight => TaperedScore::new_tapered(350, 320),
            PieceType::Silver => TaperedScore::new_tapered(450, 460),
            PieceType::Gold => TaperedScore::new_tapered(500, 520),
            PieceType::Bishop => TaperedScore::new_tapered(800, 850),
            PieceType::Rook => TaperedScore::new_tapered(1000, 1100),
            PieceType::King => TaperedScore::new(20000), // Same in all phases

            // Promoted pieces - generally more valuable in endgame
            PieceType::PromotedPawn => TaperedScore::new_tapered(500, 550),
            PieceType::PromotedLance => TaperedScore::new_tapered(500, 540),
            PieceType::PromotedKnight => TaperedScore::new_tapered(520, 550),
            PieceType::PromotedSilver => TaperedScore::new_tapered(520, 550),
            PieceType::PromotedBishop => TaperedScore::new_tapered(1200, 1300),
            PieceType::PromotedRook => TaperedScore::new_tapered(1400, 1550),
        }
    }

    /// Get tapered value for a piece in hand
    ///
    /// Hand pieces are generally more valuable than board pieces
    /// because they can be dropped anywhere (with restrictions)
    pub fn get_hand_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            // Hand pieces are slightly more valuable due to drop flexibility
            // But only unpromoted pieces can be in hand
            PieceType::Pawn => TaperedScore::new_tapered(110, 130),
            PieceType::Lance => TaperedScore::new_tapered(320, 300),
            PieceType::Knight => TaperedScore::new_tapered(370, 350),
            PieceType::Silver => TaperedScore::new_tapered(480, 490),
            PieceType::Gold => TaperedScore::new_tapered(530, 550),
            PieceType::Bishop => TaperedScore::new_tapered(850, 920),
            PieceType::Rook => TaperedScore::new_tapered(1050, 1180),

            // Promoted pieces and King should never be in hand
            _ => TaperedScore::default(),
        }
    }

    /// Calculate material balance for a player
    ///
    /// Positive value means the player has more material
    /// Negative value means the opponent has more material
    pub fn calculate_material_balance(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
    ) -> TaperedScore {
        self.evaluate_material(board, player, captured_pieces)
    }

    /// Count total material on board (both players)
    pub fn count_total_material(&self, board: &BitboardBoard) -> i32 {
        let mut total = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type != PieceType::King {
                        // Use middlegame value as base
                        total += self.get_piece_value(piece.piece_type).mg;
                    }
                }
            }
        }

        total
    }

    /// Count material by piece type
    pub fn count_material_by_type(
        &self,
        board: &BitboardBoard,
        piece_type: PieceType,
        player: Player,
    ) -> i32 {
        let mut count = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == piece_type && piece.player == player {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Get evaluation statistics
    pub fn stats(&self) -> &MaterialEvaluationStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = MaterialEvaluationStats::default();
    }
}

impl Default for MaterialEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for material evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialEvaluationConfig {
    /// Include hand pieces (captured pieces) in evaluation
    pub include_hand_pieces: bool,
    /// Use research-based values vs classic values
    pub use_research_values: bool,
}

impl Default for MaterialEvaluationConfig {
    fn default() -> Self {
        Self {
            include_hand_pieces: true,
            use_research_values: true,
        }
    }
}

/// Statistics for monitoring material evaluation
#[derive(Debug, Clone, Default)]
pub struct MaterialEvaluationStats {
    /// Number of evaluations performed
    pub evaluations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_evaluator_creation() {
        let evaluator = MaterialEvaluator::new();
        assert!(evaluator.config().include_hand_pieces);
    }

    #[test]
    fn test_material_evaluator_with_config() {
        let config = MaterialEvaluationConfig {
            include_hand_pieces: false,
            use_research_values: false,
        };
        let evaluator = MaterialEvaluator::with_config(config);
        assert!(!evaluator.config().include_hand_pieces);
    }

    #[test]
    fn test_piece_values_basic() {
        let evaluator = MaterialEvaluator::new();

        // Test basic pieces
        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        assert_eq!(pawn.mg, 100);
        assert_eq!(pawn.eg, 120);

        let rook = evaluator.get_piece_value(PieceType::Rook);
        assert_eq!(rook.mg, 1000);
        assert_eq!(rook.eg, 1100);

        let king = evaluator.get_piece_value(PieceType::King);
        assert_eq!(king.mg, 20000);
        assert_eq!(king.eg, 20000);
    }

    #[test]
    fn test_piece_values_promoted() {
        let evaluator = MaterialEvaluator::new();

        let promoted_pawn = evaluator.get_piece_value(PieceType::PromotedPawn);
        assert_eq!(promoted_pawn.mg, 500);
        assert_eq!(promoted_pawn.eg, 550);

        let promoted_rook = evaluator.get_piece_value(PieceType::PromotedRook);
        assert_eq!(promoted_rook.mg, 1400);
        assert_eq!(promoted_rook.eg, 1550);
    }

    #[test]
    fn test_hand_piece_values() {
        let evaluator = MaterialEvaluator::new();

        // Hand pieces should be slightly more valuable
        let board_pawn = evaluator.get_piece_value(PieceType::Pawn);
        let hand_pawn = evaluator.get_hand_piece_value(PieceType::Pawn);
        assert!(hand_pawn.mg > board_pawn.mg);
        assert!(hand_pawn.eg > board_pawn.eg);

        // Promoted pieces shouldn't have hand values
        let hand_promoted = evaluator.get_hand_piece_value(PieceType::PromotedPawn);
        assert_eq!(hand_promoted.mg, 0);
        assert_eq!(hand_promoted.eg, 0);
    }

    #[test]
    fn test_evaluate_starting_position() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Starting position should be balanced (both players have equal material)
        let black_score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(black_score.mg, 0);
        assert_eq!(black_score.eg, 0);

        let white_score = evaluator.evaluate_material(&board, Player::White, &captured_pieces);
        assert_eq!(white_score.mg, 0);
        assert_eq!(white_score.eg, 0);
    }

    #[test]
    fn test_evaluate_with_captures() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a captured pawn for Black
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);

        let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        // Black should have extra value from the captured pawn
        let hand_pawn_value = evaluator.get_hand_piece_value(PieceType::Pawn);
        assert_eq!(score.mg, hand_pawn_value.mg);
        assert_eq!(score.eg, hand_pawn_value.eg);
    }

    #[test]
    fn test_evaluate_without_hand_pieces() {
        let config = MaterialEvaluationConfig {
            include_hand_pieces: false,
            use_research_values: true,
        };
        let mut evaluator = MaterialEvaluator::with_config(config);
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a captured pawn
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);

        let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        // Hand pieces should not be counted
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_material_balance() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let balance = evaluator.calculate_material_balance(&board, &captured_pieces, Player::Black);

        // Starting position should have zero balance
        assert_eq!(balance.mg, 0);
        assert_eq!(balance.eg, 0);
    }

    #[test]
    fn test_count_total_material() {
        let evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();

        let total = evaluator.count_total_material(&board);

        // Starting position should have significant material (excluding kings)
        assert!(total > 10000); // Both players have material
    }

    #[test]
    fn test_count_material_by_type() {
        let evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();

        // Starting position has 9 pawns per player
        let pawn_count = evaluator.count_material_by_type(&board, PieceType::Pawn, Player::Black);
        assert_eq!(pawn_count, 9);

        // Starting position has 1 king per player
        let king_count = evaluator.count_material_by_type(&board, PieceType::King, Player::Black);
        assert_eq!(king_count, 1);

        // Starting position has 2 rooks per player
        let rook_count = evaluator.count_material_by_type(&board, PieceType::Rook, Player::Black);
        assert_eq!(rook_count, 1);
    }

    #[test]
    fn test_endgame_values_higher() {
        let evaluator = MaterialEvaluator::new();

        // Most pieces should be more valuable in endgame
        let rook = evaluator.get_piece_value(PieceType::Rook);
        assert!(rook.eg > rook.mg, "Rook should be more valuable in endgame");

        let bishop = evaluator.get_piece_value(PieceType::Bishop);
        assert!(bishop.eg > bishop.mg, "Bishop should be more valuable in endgame");

        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        assert!(pawn.eg > pawn.mg, "Pawn should be more valuable in endgame");
    }

    #[test]
    fn test_promoted_pieces_more_valuable() {
        let evaluator = MaterialEvaluator::new();

        // Promoted pieces should be more valuable than unpromoted
        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        let promoted_pawn = evaluator.get_piece_value(PieceType::PromotedPawn);
        assert!(promoted_pawn.mg > pawn.mg);
        assert!(promoted_pawn.eg > pawn.eg);

        let rook = evaluator.get_piece_value(PieceType::Rook);
        let promoted_rook = evaluator.get_piece_value(PieceType::PromotedRook);
        assert!(promoted_rook.mg > rook.mg);
        assert!(promoted_rook.eg > rook.eg);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        assert_eq!(evaluator.stats().evaluations, 0);

        evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);

        evaluator.evaluate_material(&board, Player::White, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 2);
    }

    #[test]
    fn test_reset_statistics() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);

        evaluator.reset_stats();
        assert_eq!(evaluator.stats().evaluations, 0);
    }

    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Multiple evaluations should return the same result
        let score1 = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_symmetry() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Black and White should have opposite scores in starting position
        let black_score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        let white_score = evaluator.evaluate_material(&board, Player::White, &captured_pieces);

        assert_eq!(black_score.mg, -white_score.mg);
        assert_eq!(black_score.eg, -white_score.eg);
    }
}

