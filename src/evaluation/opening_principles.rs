//! Opening Principles Module
//!
//! This module provides opening-specific evaluation principles that are most
//! important in the opening phase of the game. Includes:
//! - Piece development evaluation
//! - Center control in opening
//! - Castle formation (defensive structures)
//! - Tempo evaluation
//! - Opening-specific bonuses and penalties
//!
//! # Overview
//!
//! Opening evaluation emphasizes:
//! - Quick piece development (getting pieces into play)
//! - Center control (controlling key squares early)
//! - Castle formation (building defensive structures)
//! - Tempo (maintaining initiative)
//! - Avoiding premature attacks
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::opening_principles::OpeningPrincipleEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let mut evaluator = OpeningPrincipleEvaluator::new();
//! let board = BitboardBoard::new();
//! let move_count = 5; // 5 moves into the game
//!
//! let score = evaluator.evaluate_opening(&board, Player::Black, move_count);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Opening principle evaluator
pub struct OpeningPrincipleEvaluator {
    /// Configuration
    config: OpeningPrincipleConfig,
    /// Statistics
    stats: OpeningPrincipleStats,
}

impl OpeningPrincipleEvaluator {
    /// Create a new opening principle evaluator
    pub fn new() -> Self {
        Self {
            config: OpeningPrincipleConfig::default(),
            stats: OpeningPrincipleStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: OpeningPrincipleConfig) -> Self {
        Self {
            config,
            stats: OpeningPrincipleStats::default(),
        }
    }

    /// Evaluate opening principles
    ///
    /// Returns a TaperedScore with emphasis on middlegame/opening values
    ///
    /// # Arguments
    ///
    /// * `board` - Current board state
    /// * `player` - Player to evaluate for
    /// * `move_count` - Number of moves played (for tempo/development tracking)
    pub fn evaluate_opening(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        move_count: u32,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut score = TaperedScore::default();

        // 1. Piece development
        if self.config.enable_development {
            score += self.evaluate_development(board, player, move_count);
        }

        // 2. Center control
        if self.config.enable_center_control {
            score += self.evaluate_center_control_opening(board, player);
        }

        // 3. Castle formation (defensive structure)
        if self.config.enable_castle_formation {
            score += self.evaluate_castle_formation(board, player);
        }

        // 4. Tempo evaluation
        if self.config.enable_tempo {
            score += self.evaluate_tempo(board, player, move_count);
        }

        // 5. Opening-specific penalties
        if self.config.enable_opening_penalties {
            score += self.evaluate_opening_penalties(board, player, move_count);
        }

        score
    }

    // =======================================================================
    // PIECE DEVELOPMENT IN OPENING
    // =======================================================================

    /// Evaluate piece development in opening
    ///
    /// Pieces should be developed quickly in the opening
    fn evaluate_development(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_count: u32,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Major piece development (Rook, Bishop)
        let major_dev = self.evaluate_major_piece_development(board, player);
        mg_score += major_dev.mg;
        eg_score += major_dev.eg;

        // 2. Minor piece development (Silver, Gold, Knight)
        let minor_dev = self.evaluate_minor_piece_development(board, player);
        mg_score += minor_dev.mg;
        eg_score += minor_dev.eg;

        // 3. Development tempo bonus (early development is better)
        if move_count <= 10 {
            let developed_count = self.count_developed_pieces(board, player);
            let tempo_bonus = developed_count * 15;
            mg_score += tempo_bonus;
            eg_score += tempo_bonus / 3; // Less important in endgame
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate major piece development
    fn evaluate_major_piece_development(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        // Rook development
        for rook_pos in self.find_pieces(board, player, PieceType::Rook) {
            if rook_pos.row != start_row {
                mg_score += 35; // Strong bonus for developing rook
            } else if rook_pos.col != 0 && rook_pos.col != 8 {
                mg_score += 10; // Small bonus for moving even on back rank
            }
        }

        // Bishop development
        for bishop_pos in self.find_pieces(board, player, PieceType::Bishop) {
            if bishop_pos.row != start_row {
                mg_score += 32; // Strong bonus for developing bishop
            }
        }

        // Penalty for undeveloped major pieces in late opening
        TaperedScore::new_tapered(mg_score, mg_score / 4)
    }

    /// Evaluate minor piece development
    fn evaluate_minor_piece_development(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        // Silver development
        for silver_pos in self.find_pieces(board, player, PieceType::Silver) {
            if silver_pos.row != start_row {
                mg_score += 22; // Good bonus for developing silver
            }
        }

        // Gold development (less critical than silver)
        for gold_pos in self.find_pieces(board, player, PieceType::Gold) {
            if gold_pos.row != start_row {
                mg_score += 18; // Moderate bonus for gold development
            }
        }

        // Knight development
        for knight_pos in self.find_pieces(board, player, PieceType::Knight) {
            if knight_pos.row != start_row {
                mg_score += 20; // Good bonus for knight development
            }
        }

        TaperedScore::new_tapered(mg_score, mg_score / 4)
    }

    /// Count developed pieces
    fn count_developed_pieces(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut count = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        for piece_type in [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Knight,
        ] {
            for piece_pos in self.find_pieces(board, player, piece_type) {
                if piece_pos.row != start_row {
                    count += 1;
                }
            }
        }

        count
    }

    // =======================================================================
    // CENTER CONTROL IN OPENING
    // =======================================================================

    /// Evaluate center control in opening
    ///
    /// Center control is critical in the opening
    fn evaluate_center_control_opening(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Core center (4,4) and surrounding squares
        let core_center = Position::new(4, 4);
        if let Some(piece) = board.get_piece(core_center) {
            if piece.player == player {
                let value = self.get_opening_center_value(piece.piece_type);
                mg_score += value;
                eg_score += value / 3;
            } else {
                let value = self.get_opening_center_value(piece.piece_type);
                mg_score -= value;
                eg_score -= value / 3;
            }
        }

        // Extended center squares
        for row in 3..=5 {
            for col in 3..=5 {
                if row == 4 && col == 4 {
                    continue; // Already counted
                }

                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let value = self.get_opening_center_value(piece.piece_type) * 2 / 3;

                    if piece.player == player {
                        mg_score += value;
                        eg_score += value / 3;
                    } else {
                        mg_score -= value;
                        eg_score -= value / 3;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get center control value for a piece type in opening
    fn get_opening_center_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 20,
            PieceType::Knight => 35,
            PieceType::Silver => 30,
            PieceType::Gold => 28,
            PieceType::Bishop => 40,
            PieceType::Rook => 38,
            _ => 15,
        }
    }

    // =======================================================================
    // CASTLE FORMATION (DEFENSIVE STRUCTURE)
    // =======================================================================

    /// Evaluate castle formation in opening
    ///
    /// Building a solid defensive structure is important
    fn evaluate_castle_formation(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;

        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // 1. King in castle position (corner area)
        if self.is_castle_position(king_pos, player) {
            mg_score += 40; // Good to castle early
        }

        // 2. Gold and silver near king (traditional defense)
        let golds_near_king = self.count_pieces_near_king(board, king_pos, player, PieceType::Gold);
        let silvers_near_king =
            self.count_pieces_near_king(board, king_pos, player, PieceType::Silver);

        mg_score += golds_near_king * 25; // Golds are excellent defenders
        mg_score += silvers_near_king * 22; // Silvers also good

        // 3. Pawn shield in front of king
        let pawn_shield = self.count_pawn_shield(board, king_pos, player);
        mg_score += pawn_shield * 20;

        // Only important in opening/middlegame
        TaperedScore::new_tapered(mg_score, mg_score / 4)
    }

    /// Check if king is in castle position
    fn is_castle_position(&self, king_pos: Position, player: Player) -> bool {
        if player == Player::Black {
            // Black castles in bottom-right or bottom-left
            king_pos.row >= 7 && (king_pos.col <= 2 || king_pos.col >= 6)
        } else {
            // White castles in top-right or top-left
            king_pos.row <= 1 && (king_pos.col <= 2 || king_pos.col >= 6)
        }
    }

    /// Count pieces near king (within 2 squares)
    fn count_pieces_near_king(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        piece_type: PieceType,
    ) -> i32 {
        let mut count = 0;

        for dr in -2..=2 {
            for dc in -2..=2 {
                let new_row = king_pos.row as i8 + dr;
                let new_col = king_pos.col as i8 + dc;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player == player && piece.piece_type == piece_type {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    /// Count pawn shield in front of king
    fn count_pawn_shield(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> i32 {
        let mut count = 0;
        let direction = if player == Player::Black { -1 } else { 1 };

        for dc in -1..=1 {
            let new_row = king_pos.row as i8 + direction;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == PieceType::Pawn {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    // =======================================================================
    // TEMPO EVALUATION
    // =======================================================================

    /// Evaluate tempo (maintaining initiative)
    fn evaluate_tempo(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_count: u32,
    ) -> TaperedScore {
        let mut mg_score = 0;

        // Basic tempo bonus (player to move has advantage)
        mg_score += 10;

        // Development tempo (reward for developing faster than opponent)
        if move_count <= 15 {
            let our_developed = self.count_developed_pieces(board, player);
            let opp_developed = self.count_developed_pieces(board, player.opposite());

            if our_developed > opp_developed {
                let development_lead = (our_developed - opp_developed) * 20;
                mg_score += development_lead;
            }
        }

        // Activity tempo (more active pieces)
        let our_active_pieces = self.count_active_pieces(board, player);
        let opp_active_pieces = self.count_active_pieces(board, player.opposite());

        if our_active_pieces > opp_active_pieces {
            mg_score += (our_active_pieces - opp_active_pieces) * 12;
        }

        // Tempo only matters in opening/middlegame
        TaperedScore::new_tapered(mg_score, mg_score / 5)
    }

    /// Count active pieces (pieces not on starting positions)
    fn count_active_pieces(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut count = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type != PieceType::King {
                        // Piece is active if not on starting row or in center half
                        if pos.row != start_row || (pos.row >= 3 && pos.row <= 5) {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    // =======================================================================
    // OPENING-SPECIFIC PENALTIES
    // =======================================================================

    /// Evaluate opening-specific penalties
    ///
    /// Penalize common opening mistakes
    fn evaluate_opening_penalties(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_count: u32,
    ) -> TaperedScore {
        let mut mg_penalty = 0;

        // Early in opening (first 10 moves)
        if move_count <= 10 {
            // 1. Penalty for moving the same piece multiple times
            // (This would require move history, so we skip for now)

            // 2. Penalty for undeveloped major pieces
            let rooks_developed = self
                .find_pieces(board, player, PieceType::Rook)
                .iter()
                .filter(|p| p.row != if player == Player::Black { 8 } else { 0 })
                .count();

            let bishops_developed = self
                .find_pieces(board, player, PieceType::Bishop)
                .iter()
                .filter(|p| p.row != if player == Player::Black { 8 } else { 0 })
                .count();

            if rooks_developed == 0 && move_count >= 8 {
                mg_penalty += 30; // Penalty for undeveloped rook
            }

            if bishops_developed == 0 && move_count >= 6 {
                mg_penalty += 25; // Penalty for undeveloped bishop
            }

            // 3. Penalty for king moving too early (without castling)
            if let Some(king_pos) = self.find_king_position(board, player) {
                let start_row = if player == Player::Black { 8 } else { 0 };

                if king_pos.row != start_row && !self.is_castle_position(king_pos, player) {
                    mg_penalty += 40; // Big penalty for early king moves
                }
            }
        }

        TaperedScore::new_tapered(-mg_penalty, -mg_penalty / 5)
    }

    // =======================================================================
    // HELPER METHODS
    // =======================================================================

    /// Find king position
    fn find_king_position(&self, board: &BitboardBoard, player: Player) -> Option<Position> {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::King && piece.player == player {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// Find all pieces of a specific type
    fn find_pieces(
        &self,
        board: &BitboardBoard,
        player: Player,
        piece_type: PieceType,
    ) -> Vec<Position> {
        let mut pieces = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == piece_type && piece.player == player {
                        pieces.push(pos);
                    }
                }
            }
        }
        pieces
    }

    /// Get statistics
    pub fn stats(&self) -> &OpeningPrincipleStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = OpeningPrincipleStats::default();
    }
}

impl Default for OpeningPrincipleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for opening principle evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpeningPrincipleConfig {
    /// Enable development evaluation
    pub enable_development: bool,
    /// Enable center control evaluation
    pub enable_center_control: bool,
    /// Enable castle formation evaluation
    pub enable_castle_formation: bool,
    /// Enable tempo evaluation
    pub enable_tempo: bool,
    /// Enable opening penalties
    pub enable_opening_penalties: bool,
}

impl Default for OpeningPrincipleConfig {
    fn default() -> Self {
        Self {
            enable_development: true,
            enable_center_control: true,
            enable_castle_formation: true,
            enable_tempo: true,
            enable_opening_penalties: true,
        }
    }
}

/// Statistics for opening principle evaluation
#[derive(Debug, Clone, Default)]
pub struct OpeningPrincipleStats {
    /// Number of evaluations performed
    pub evaluations: u64,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_opening_evaluator_creation() {
        let evaluator = OpeningPrincipleEvaluator::new();
        assert!(evaluator.config.enable_development);
    }

    #[test]
    fn test_development_evaluation() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_development(&board, Player::Black, 5);

        // Starting position has no development
        assert_eq!(score.mg, 0);
    }

    #[test]
    fn test_center_control_opening() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_center_control_opening(&board, Player::Black);

        // Starting position is symmetric
        assert!(score.mg.abs() < 50);
    }

    #[test]
    fn test_castle_formation() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_castle_formation(&board, Player::Black);

        // Starting position has some defensive structure
        assert!(score.mg > 0);
    }

    #[test]
    fn test_tempo_evaluation() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_tempo(&board, Player::Black, 5);

        // Should have base tempo bonus
        assert!(score.mg >= 10);
    }

    #[test]
    fn test_opening_penalties() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_opening_penalties(&board, Player::Black, 5);

        // Starting position shouldn't have major penalties
        assert!(score.mg >= -50);
    }

    #[test]
    fn test_count_developed_pieces() {
        let evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let count = evaluator.count_developed_pieces(&board, Player::Black);

        // Starting position has no developed pieces
        assert_eq!(count, 0);
    }

    #[test]
    fn test_is_castle_position() {
        let evaluator = OpeningPrincipleEvaluator::new();

        // Black castle positions
        assert!(evaluator.is_castle_position(Position::new(8, 1), Player::Black));
        assert!(evaluator.is_castle_position(Position::new(7, 7), Player::Black));
        assert!(!evaluator.is_castle_position(Position::new(4, 4), Player::Black));
    }

    #[test]
    fn test_opening_center_values() {
        let evaluator = OpeningPrincipleEvaluator::new();

        assert_eq!(evaluator.get_opening_center_value(PieceType::Bishop), 40);
        assert_eq!(evaluator.get_opening_center_value(PieceType::Knight), 35);
        assert_eq!(evaluator.get_opening_center_value(PieceType::Pawn), 20);
    }

    #[test]
    fn test_complete_opening_evaluation() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_opening(&board, Player::Black, 5);

        // Should have some positive opening evaluation
        assert!(score.mg > 0);
    }

    #[test]
    fn test_statistics() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        assert_eq!(evaluator.stats().evaluations, 0);

        evaluator.evaluate_opening(&board, Player::Black, 5);
        assert_eq!(evaluator.stats().evaluations, 1);
    }

    #[test]
    fn test_config_options() {
        let config = OpeningPrincipleConfig {
            enable_development: true,
            enable_center_control: false,
            enable_castle_formation: true,
            enable_tempo: false,
            enable_opening_penalties: true,
        };

        let evaluator = OpeningPrincipleEvaluator::with_config(config);
        assert!(evaluator.config.enable_development);
        assert!(!evaluator.config.enable_center_control);
    }

    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let score1 = evaluator.evaluate_opening(&board, Player::Black, 5);
        let score2 = evaluator.evaluate_opening(&board, Player::Black, 5);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_move_count_effects() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        // Early game (move 5)
        let early_score = evaluator.evaluate_opening(&board, Player::Black, 5);

        // Later (move 20)
        let late_score = evaluator.evaluate_opening(&board, Player::Black, 20);

        // Tempo bonuses should be higher early in the game
        // (though in starting position both might be similar)
        assert!(early_score.mg >= 0);
        assert!(late_score.mg >= 0);
    }

    #[test]
    fn test_major_vs_minor_development() {
        let evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        let major = evaluator.evaluate_major_piece_development(&board, Player::Black);
        let minor = evaluator.evaluate_minor_piece_development(&board, Player::Black);

        // Starting position has no development
        assert_eq!(major.mg, 0);
        assert_eq!(minor.mg, 0);
    }
}
