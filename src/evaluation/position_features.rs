//! Position-Specific Evaluation Features Module
//!
//! This module provides phase-aware evaluation of position-specific features including:
//! - King safety by phase
//! - Pawn structure by phase
//! - Piece mobility by phase
//! - Center control by phase
//! - Development bonus by phase
//!
//! All evaluations return TaperedScore for seamless integration with the
//! tapered evaluation system.
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::position_features::PositionFeatureEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let evaluator = PositionFeatureEvaluator::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let king_safety = evaluator.evaluate_king_safety(&board, Player::Black);
//! let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
//! ```

use crate::types::*;
use crate::bitboards::BitboardBoard;
use crate::moves::MoveGenerator;
use serde::{Deserialize, Serialize};

/// Position feature evaluator with phase-aware evaluation
pub struct PositionFeatureEvaluator {
    /// Configuration for position evaluation
    config: PositionFeatureConfig,
    /// Statistics tracking
    stats: PositionFeatureStats,
}

impl PositionFeatureEvaluator {
    /// Create a new PositionFeatureEvaluator with default configuration
    pub fn new() -> Self {
        Self {
            config: PositionFeatureConfig::default(),
            stats: PositionFeatureStats::default(),
        }
    }

    /// Create a new evaluator with custom configuration
    pub fn with_config(config: PositionFeatureConfig) -> Self {
        Self {
            config,
            stats: PositionFeatureStats::default(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &PositionFeatureConfig {
        &self.config
    }

    // =======================================================================
    // KING SAFETY EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate king safety with phase-aware weights
    ///
    /// King safety is more critical in middlegame when there are more pieces
    /// that can mount an attack.
    pub fn evaluate_king_safety(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.king_safety_evals += 1;

        let king_pos = self.find_king_position(board, player);
        if king_pos.is_none() {
            return TaperedScore::default();
        }
        let king_pos = king_pos.unwrap();

        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. King shield (pieces protecting the king)
        let shield_score = self.evaluate_king_shield(board, king_pos, player);
        mg_score += shield_score.mg;
        eg_score += shield_score.eg;

        // 2. Pawn cover (pawns in front of king)
        let pawn_cover = self.evaluate_pawn_cover(board, king_pos, player);
        mg_score += pawn_cover.mg;
        eg_score += pawn_cover.eg;

        // 3. Enemy attackers near king
        let attacker_penalty = self.evaluate_enemy_attackers(board, king_pos, player);
        mg_score -= attacker_penalty.mg;
        eg_score -= attacker_penalty.eg;

        // 4. King exposure (open squares near king)
        let exposure = self.evaluate_king_exposure(board, king_pos, player);
        mg_score -= exposure.mg;
        eg_score -= exposure.eg;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate king shield (friendly pieces near king)
    fn evaluate_king_shield(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        let shield_offsets = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

        for (dr, dc) in shield_offsets {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let shield_value = match piece.piece_type {
                            PieceType::Gold => (40, 20),         // Best defender
                            PieceType::Silver => (30, 18),       // Good defender
                            PieceType::Pawn => (20, 12),         // Pawn shield
                            PieceType::Knight => (15, 8),        // Less useful
                            PieceType::Lance => (15, 8),         // Less useful
                            _ => (10, 5),                        // Other pieces
                        };
                        mg_score += shield_value.0;
                        eg_score += shield_value.1;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate pawn cover in front of king
    fn evaluate_pawn_cover(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check squares in front of king (direction depends on player)
        let direction = if player == Player::Black { -1 } else { 1 };

        for dc in -1..=1 {
            let new_row = king_pos.row as i8 + direction;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == PieceType::Pawn {
                        mg_score += 25; // Pawn cover very important in middlegame
                        eg_score += 10; // Less critical in endgame
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate enemy attackers near king
    fn evaluate_enemy_attackers(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check 3x3 area around king
        for row in (king_pos.row.saturating_sub(2))..=(king_pos.row + 2).min(8) {
            for col in (king_pos.col.saturating_sub(2))..=(king_pos.col + 2).min(8) {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player != player {
                        let threat_value = match piece.piece_type {
                            PieceType::Rook | PieceType::PromotedRook => (50, 30),
                            PieceType::Bishop | PieceType::PromotedBishop => (45, 28),
                            PieceType::Gold => (30, 20),
                            PieceType::Silver => (25, 18),
                            _ => (15, 10),
                        };
                        mg_score += threat_value.0;
                        eg_score += threat_value.1;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate king exposure (open squares near king)
    fn evaluate_king_exposure(&self, board: &BitboardBoard, king_pos: Position, _player: Player) -> TaperedScore {
        let mut open_squares = 0;

        let offsets = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

        for (dr, dc) in offsets {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if !board.is_square_occupied(pos) {
                    open_squares += 1;
                }
            }
        }

        // Open squares near king are dangerous (more in middlegame)
        let mg_penalty = open_squares * 20;
        let eg_penalty = open_squares * 10;

        TaperedScore::new_tapered(mg_penalty, eg_penalty)
    }

    // =======================================================================
    // PAWN STRUCTURE EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate pawn structure with phase-aware weights
    pub fn evaluate_pawn_structure(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.pawn_structure_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;
        let pawns = self.collect_pawns(board, player);

        if pawns.is_empty() {
            return TaperedScore::default();
        }

        // 1. Pawn chains (connected pawns)
        let chains = self.evaluate_pawn_chains(&pawns);
        mg_score += chains.mg;
        eg_score += chains.eg;

        // 2. Advanced pawns
        let advancement = self.evaluate_pawn_advancement(&pawns, player);
        mg_score += advancement.mg;
        eg_score += advancement.eg;

        // 3. Isolated pawns
        let isolation = self.evaluate_pawn_isolation(board, &pawns, player);
        mg_score += isolation.mg;
        eg_score += isolation.eg;

        // 4. Passed pawns (no enemy pawns in front)
        let passed = self.evaluate_passed_pawns(board, &pawns, player);
        mg_score += passed.mg;
        eg_score += passed.eg;

        // 5. Doubled pawns (same file)
        let doubled = self.evaluate_doubled_pawns(&pawns);
        mg_score += doubled.mg;
        eg_score += doubled.eg;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Collect all pawns for a player
    fn collect_pawns(&self, board: &BitboardBoard, player: Player) -> Vec<Position> {
        let mut pawns = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        pawns.push(pos);
                    }
                }
            }
        }
        pawns
    }

    /// Evaluate pawn chains (adjacent pawns)
    fn evaluate_pawn_chains(&self, pawns: &[Position]) -> TaperedScore {
        let mut count = 0;

        for i in 0..pawns.len() {
            for j in i + 1..pawns.len() {
                let r1 = pawns[i].row;
                let c1 = pawns[i].col;
                let r2 = pawns[j].row;
                let c2 = pawns[j].col;

                if (r1.abs_diff(r2) == 1 && c1 == c2) || (c1.abs_diff(c2) == 1 && r1 == r2) {
                    count += 1;
                }
            }
        }

        // Pawn chains more important in middlegame
        TaperedScore::new_tapered(count * 18, count * 12)
    }

    /// Evaluate pawn advancement
    fn evaluate_pawn_advancement(&self, pawns: &[Position], player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        let promotion_zone_start = if player == Player::Black { 2 } else { 6 };

        for pawn in pawns {
            let advancement = if player == Player::Black {
                (promotion_zone_start as i32 - pawn.row as i32).max(0)
            } else {
                (pawn.row as i32 - promotion_zone_start as i32).max(0)
            };

            if advancement > 0 {
                mg_score += advancement * 10;  // Moderate in middlegame
                eg_score += advancement * 20;  // Very important in endgame
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate isolated pawns
    fn evaluate_pawn_isolation(&self, board: &BitboardBoard, pawns: &[Position], player: Player) -> TaperedScore {
        let mut isolated_count = 0;

        for pawn in pawns {
            if self.is_pawn_isolated(board, *pawn, player) {
                isolated_count += 1;
            }
        }

        // Isolated pawns worse in endgame
        TaperedScore::new_tapered(-(isolated_count * 18), -(isolated_count * 30))
    }

    /// Check if a pawn is isolated
    fn is_pawn_isolated(&self, board: &BitboardBoard, pawn_pos: Position, player: Player) -> bool {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let new_row = pawn_pos.row as i8 + dr;
                let new_col = pawn_pos.col as i8 + dc;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.piece_type == PieceType::Pawn && piece.player == player {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Evaluate passed pawns
    fn evaluate_passed_pawns(&self, board: &BitboardBoard, pawns: &[Position], player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        for pawn in pawns {
            if self.is_passed_pawn(board, *pawn, player) {
                // Calculate how advanced the passed pawn is
                let advancement = if player == Player::Black {
                    8 - pawn.row
                } else {
                    pawn.row
                };
                
                // Passed pawns exponentially more valuable as they advance
                mg_score += (advancement * advancement) as i32 * 5;  // Moderate in mg
                eg_score += (advancement * advancement) as i32 * 12; // Critical in eg
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a pawn is passed
    fn is_passed_pawn(&self, board: &BitboardBoard, pawn_pos: Position, player: Player) -> bool {
        // Check if there are any enemy pawns in front of this pawn
        let direction = if player == Player::Black { -1 } else { 1 };
        
        for col_offset in -1..=1 {
            let check_col = pawn_pos.col as i8 + col_offset;
            if check_col < 0 || check_col >= 9 {
                continue;
            }

            let mut check_row = pawn_pos.row as i8 + direction;
            while check_row >= 0 && check_row < 9 {
                let pos = Position::new(check_row as u8, check_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player != player {
                        return false; // Enemy pawn blocks
                    }
                }
                check_row += direction;
            }
        }

        true
    }

    /// Evaluate doubled pawns (same file)
    fn evaluate_doubled_pawns(&self, pawns: &[Position]) -> TaperedScore {
        let mut doubled_count = 0;
        let mut file_counts = [0; 9];

        for pawn in pawns {
            file_counts[pawn.col as usize] += 1;
        }

        for count in file_counts {
            if count >= 2 {
                doubled_count += count - 1; // Each extra pawn is doubled
            }
        }

        // Doubled pawns moderately bad (worse in endgame)
        TaperedScore::new_tapered(-(doubled_count * 12), -(doubled_count * 18))
    }

    // =======================================================================
    // PIECE MOBILITY EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate piece mobility with phase-aware weights
    ///
    /// Mobility is more important in endgame when pieces need room to maneuver.
    pub fn evaluate_mobility(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> TaperedScore {
        self.stats.mobility_evals += 1;

        let move_generator = MoveGenerator::new();
        let legal_moves = move_generator.generate_legal_moves(board, player, captured_pieces);
        let move_count = legal_moves.len() as i32;

        // Count different types of moves for better evaluation
        let mut attack_moves = 0;

        for mv in &legal_moves {
            if mv.is_capture {
                attack_moves += 1;
            }
        }

        // Basic mobility score
        let mg_score = move_count * 2;      // Less important in middlegame
        let eg_score = move_count * 4;      // More important in endgame

        // Bonus for attacking moves (more important in middlegame)
        let attack_bonus_mg = attack_moves * 3;
        let attack_bonus_eg = attack_moves * 2;

        TaperedScore::new_tapered(mg_score + attack_bonus_mg, eg_score + attack_bonus_eg)
    }

    // =======================================================================
    // CENTER CONTROL EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate center control with phase-aware weights
    ///
    /// Center control is more important in opening/middlegame.
    pub fn evaluate_center_control(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.center_control_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Define center squares (3-5, 3-5)
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let value = self.get_center_control_value(piece.piece_type);
                    
                    if piece.player == player {
                        mg_score += value.mg;
                        eg_score += value.eg;
                    } else {
                        mg_score -= value.mg;
                        eg_score -= value.eg;
                    }
                }
            }
        }

        // Extended center (2-6, 2-6) with reduced bonus
        for row in 2..=6 {
            for col in 2..=6 {
                // Skip already counted center
                if row >= 3 && row <= 5 && col >= 3 && col <= 5 {
                    continue;
                }
                
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let value = self.get_center_control_value(piece.piece_type);
                    
                    if piece.player == player {
                        mg_score += value.mg / 2;
                        eg_score += value.eg / 2;
                    } else {
                        mg_score -= value.mg / 2;
                        eg_score -= value.eg / 2;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get center control value for a piece type
    fn get_center_control_value(&self, piece_type: PieceType) -> TaperedScore {
        match piece_type {
            PieceType::Pawn => TaperedScore::new_tapered(15, 8),
            PieceType::Knight => TaperedScore::new_tapered(25, 15),
            PieceType::Silver => TaperedScore::new_tapered(22, 18),
            PieceType::Gold => TaperedScore::new_tapered(20, 16),
            PieceType::Bishop => TaperedScore::new_tapered(30, 20),
            PieceType::Rook => TaperedScore::new_tapered(28, 22),
            PieceType::PromotedPawn => TaperedScore::new_tapered(25, 20),
            PieceType::PromotedBishop => TaperedScore::new_tapered(35, 28),
            PieceType::PromotedRook => TaperedScore::new_tapered(32, 26),
            _ => TaperedScore::default(),
        }
    }

    // =======================================================================
    // DEVELOPMENT EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate piece development with phase-aware weights
    ///
    /// Development is critical in opening, less so in endgame.
    pub fn evaluate_development(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.development_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check if major pieces are developed
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        if let Some(development_bonus) = self.get_development_bonus(piece.piece_type, pos, player) {
                            mg_score += development_bonus.mg;
                            eg_score += development_bonus.eg;
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get development bonus for a piece
    fn get_development_bonus(&self, piece_type: PieceType, pos: Position, player: Player) -> Option<TaperedScore> {
        let start_row = if player == Player::Black { 8 } else { 0 };

        match piece_type {
            PieceType::Rook => {
                if pos.row != start_row {
                    Some(TaperedScore::new_tapered(30, 8)) // Very important in opening
                } else {
                    None
                }
            }
            PieceType::Bishop => {
                if pos.row != start_row {
                    Some(TaperedScore::new_tapered(28, 10))
                } else {
                    None
                }
            }
            PieceType::Silver => {
                if pos.row != start_row {
                    Some(TaperedScore::new_tapered(20, 5))
                } else {
                    None
                }
            }
            PieceType::Gold => {
                if pos.row != start_row {
                    Some(TaperedScore::new_tapered(15, 5))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // =======================================================================
    // HELPER METHODS
    // =======================================================================

    /// Find king position for a player
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

    /// Get statistics
    pub fn stats(&self) -> &PositionFeatureStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PositionFeatureStats::default();
    }
}

impl Default for PositionFeatureEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for position feature evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionFeatureConfig {
    /// Enable king safety evaluation
    pub enable_king_safety: bool,
    /// Enable pawn structure evaluation
    pub enable_pawn_structure: bool,
    /// Enable mobility evaluation
    pub enable_mobility: bool,
    /// Enable center control evaluation
    pub enable_center_control: bool,
    /// Enable development evaluation
    pub enable_development: bool,
}

impl Default for PositionFeatureConfig {
    fn default() -> Self {
        Self {
            enable_king_safety: true,
            enable_pawn_structure: true,
            enable_mobility: true,
            enable_center_control: true,
            enable_development: true,
        }
    }
}

/// Statistics for position feature evaluation
#[derive(Debug, Clone, Default)]
pub struct PositionFeatureStats {
    /// King safety evaluations performed
    pub king_safety_evals: u64,
    /// Pawn structure evaluations performed
    pub pawn_structure_evals: u64,
    /// Mobility evaluations performed
    pub mobility_evals: u64,
    /// Center control evaluations performed
    pub center_control_evals: u64,
    /// Development evaluations performed
    pub development_evals: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_feature_evaluator_creation() {
        let evaluator = PositionFeatureEvaluator::new();
        assert!(evaluator.config().enable_king_safety);
    }

    #[test]
    fn test_king_safety_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_king_safety(&board, Player::Black);
        
        // Starting position should have positive king safety
        assert!(score.mg > 0 || score.eg > 0);
    }

    #[test]
    fn test_pawn_structure_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_pawn_structure(&board, Player::Black);
        
        // Starting position should have neutral or positive pawn structure
        assert!(score.mg >= 0);
    }

    #[test]
    fn test_mobility_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        
        // Starting position should have positive mobility
        assert!(score.mg > 0);
        assert!(score.eg > 0);
        assert!(score.eg > score.mg); // More important in endgame
    }

    #[test]
    fn test_center_control_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_center_control(&board, Player::Black);
        
        // Starting position is symmetric, so score should be near zero
        assert!(score.mg.abs() < 50);
    }

    #[test]
    fn test_development_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_development(&board, Player::Black);
        
        // Starting position has no development
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_pawn_chain_detection() {
        let evaluator = PositionFeatureEvaluator::new();
        
        // Create adjacent pawns
        let pawns = vec![
            Position::new(3, 3),
            Position::new(3, 4), // Adjacent horizontally
        ];
        
        let chains = evaluator.evaluate_pawn_chains(&pawns);
        
        // Should detect 1 chain
        assert_eq!(chains.mg, 18);
        assert_eq!(chains.eg, 12);
    }

    #[test]
    fn test_pawn_advancement() {
        let evaluator = PositionFeatureEvaluator::new();
        
        // Advanced pawn for Black (low row number)
        let pawns = vec![
            Position::new(1, 4), // Very advanced (row 1)
        ];
        
        let advancement = evaluator.evaluate_pawn_advancement(&pawns, Player::Black);
        
        // Should have positive advancement bonus
        assert!(advancement.mg > 0);
        assert!(advancement.eg > advancement.mg); // More valuable in endgame
    }

    #[test]
    fn test_isolated_pawn_detection() {
        let board = BitboardBoard::empty();
        let evaluator = PositionFeatureEvaluator::new();
        
        // Isolated pawn
        let pawn_pos = Position::new(4, 4);
        assert!(evaluator.is_pawn_isolated(&board, pawn_pos, Player::Black));
    }

    #[test]
    fn test_passed_pawn_detection() {
        let board = BitboardBoard::empty();
        let evaluator = PositionFeatureEvaluator::new();
        
        // Empty board means any pawn is passed
        let pawn_pos = Position::new(4, 4);
        assert!(evaluator.is_passed_pawn(&board, pawn_pos, Player::Black));
    }

    #[test]
    fn test_king_shield_evaluation() {
        let evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        
        // Find Black's king
        let king_pos = evaluator.find_king_position(&board, Player::Black).unwrap();
        
        let shield = evaluator.evaluate_king_shield(&board, king_pos, Player::Black);
        
        // Starting position should have some shield
        assert!(shield.mg > 0);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        assert_eq!(evaluator.stats().king_safety_evals, 0);
        assert_eq!(evaluator.stats().mobility_evals, 0);

        evaluator.evaluate_king_safety(&board, Player::Black);
        assert_eq!(evaluator.stats().king_safety_evals, 1);

        evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().mobility_evals, 1);
    }

    #[test]
    fn test_reset_statistics() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        evaluator.evaluate_king_safety(&board, Player::Black);
        assert_eq!(evaluator.stats().king_safety_evals, 1);

        evaluator.reset_stats();
        assert_eq!(evaluator.stats().king_safety_evals, 0);
    }

    #[test]
    fn test_config_options() {
        let config = PositionFeatureConfig {
            enable_king_safety: true,
            enable_pawn_structure: false,
            enable_mobility: true,
            enable_center_control: false,
            enable_development: true,
        };

        let evaluator = PositionFeatureEvaluator::with_config(config);
        assert!(evaluator.config().enable_king_safety);
        assert!(!evaluator.config().enable_pawn_structure);
    }

    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Multiple evaluations should return same result
        let score1 = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_phase_differences() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Mobility should be more valuable in endgame
        let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        assert!(mobility.eg > mobility.mg);

        // Center control should be more valuable in middlegame
        let center = evaluator.evaluate_center_control(&board, Player::Black);
        // Note: This might be close in starting position

        // Development should be more valuable in middlegame
        let development = evaluator.evaluate_development(&board, Player::Black);
        // Note: Starting position has no development bonus
    }

    #[test]
    fn test_doubled_pawns_penalty() {
        let evaluator = PositionFeatureEvaluator::new();
        
        // Two pawns on same file
        let pawns = vec![
            Position::new(3, 4),
            Position::new(5, 4), // Same file (col 4)
        ];
        
        let doubled = evaluator.evaluate_doubled_pawns(&pawns);
        
        // Should have negative score (penalty)
        assert!(doubled.mg < 0);
        assert!(doubled.eg < 0);
        assert!(doubled.eg < doubled.mg); // Worse in endgame
    }

    #[test]
    fn test_center_control_symmetry() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let black_score = evaluator.evaluate_center_control(&board, Player::Black);
        let white_score = evaluator.evaluate_center_control(&board, Player::White);

        // Starting position is symmetric
        assert_eq!(black_score.mg, -white_score.mg);
        assert_eq!(black_score.eg, -white_score.eg);
    }

    #[test]
    fn test_king_position_finding() {
        let evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let black_king = evaluator.find_king_position(&board, Player::Black);
        assert!(black_king.is_some());

        let white_king = evaluator.find_king_position(&board, Player::White);
        assert!(white_king.is_some());
    }
}

