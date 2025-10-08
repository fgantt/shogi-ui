//! Endgame Patterns Module
//!
//! This module provides endgame-specific evaluation patterns that become more
//! important as the game progresses into the endgame. Includes:
//! - King activity bonus in endgame
//! - Passed pawn evaluation (enhanced from position_features)
//! - Piece coordination in endgame
//! - Mating pattern detection
//! - Endgame-specific bonuses and penalties
//!
//! # Overview
//!
//! Endgame evaluation differs significantly from middlegame:
//! - King becomes an active piece (should advance)
//! - Passed pawns become dominant
//! - Piece coordination critical for mating attacks
//! - Rooks and bishops gain power on open board
//! - Material advantage must be converted to mate
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::endgame_patterns::EndgamePatternEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let mut evaluator = EndgamePatternEvaluator::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let score = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
//! ```

use crate::types::*;
use crate::bitboards::BitboardBoard;
use serde::{Deserialize, Serialize};

/// Endgame pattern evaluator
pub struct EndgamePatternEvaluator {
    /// Configuration
    config: EndgamePatternConfig,
    /// Statistics
    stats: EndgamePatternStats,
}

impl EndgamePatternEvaluator {
    /// Create a new endgame pattern evaluator
    pub fn new() -> Self {
        Self {
            config: EndgamePatternConfig::default(),
            stats: EndgamePatternStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EndgamePatternConfig) -> Self {
        Self {
            config,
            stats: EndgamePatternStats::default(),
        }
    }

    /// Evaluate endgame patterns
    ///
    /// Returns a TaperedScore with emphasis on endgame values
    pub fn evaluate_endgame(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut score = TaperedScore::default();

        // 1. King activity in endgame
        if self.config.enable_king_activity {
            score += self.evaluate_king_activity(board, player);
        }

        // 2. Passed pawn evaluation (endgame-specific)
        if self.config.enable_passed_pawns {
            score += self.evaluate_passed_pawns_endgame(board, player);
        }

        // 3. Piece coordination for mating attacks
        if self.config.enable_piece_coordination {
            score += self.evaluate_piece_coordination(board, player);
        }

        // 4. Mating pattern detection
        if self.config.enable_mating_patterns {
            score += self.evaluate_mating_patterns(board, player, captured_pieces);
        }

        // 5. Rook/Bishop activity in endgame
        if self.config.enable_major_piece_activity {
            score += self.evaluate_major_piece_activity(board, player);
        }

        score
    }

    // =======================================================================
    // KING ACTIVITY IN ENDGAME
    // =======================================================================

    /// Evaluate king activity in endgame
    ///
    /// In endgame, the king should be active and centralized
    fn evaluate_king_activity(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Centralization bonus (more important in endgame)
        let center_distance = self.distance_to_center(king_pos);
        let centralization_bonus = (4 - center_distance.min(4)) * 15;
        mg_score += centralization_bonus / 4; // Small bonus in middlegame
        eg_score += centralization_bonus;      // Large bonus in endgame

        // 2. Activity bonus (king not on back rank)
        let back_rank = if player == Player::Black { 8 } else { 0 };
        if king_pos.row != back_rank {
            mg_score += 5;  // Small bonus in middlegame
            eg_score += 25; // Large bonus in endgame
        }

        // 3. Advanced king bonus (crossing center)
        let is_advanced = if player == Player::Black {
            king_pos.row <= 4
        } else {
            king_pos.row >= 4
        };
        
        if is_advanced {
            mg_score += 5;  // Risky in middlegame
            eg_score += 35; // Excellent in endgame
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Calculate Manhattan distance to center
    fn distance_to_center(&self, pos: Position) -> i32 {
        let center_row = 4;
        let center_col = 4;
        (pos.row as i32 - center_row).abs() + (pos.col as i32 - center_col).abs()
    }

    // =======================================================================
    // PASSED PAWN EVALUATION (ENDGAME-SPECIFIC)
    // =======================================================================

    /// Evaluate passed pawns with endgame emphasis
    fn evaluate_passed_pawns_endgame(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let pawns = self.collect_pawns(board, player);
        let mut mg_score = 0;
        let mut eg_score = 0;

        for pawn in pawns {
            if self.is_passed_pawn(board, pawn, player) {
                let advancement = if player == Player::Black {
                    8 - pawn.row
                } else {
                    pawn.row
                };

                // Quadratic growth - passed pawns exponentially valuable
                let base_mg = (advancement * advancement) as i32 * 8;
                let base_eg = (advancement * advancement) as i32 * 20;

                // Additional bonus if king can support the pawn
                if let Some(king_pos) = self.find_king_position(board, player) {
                    let king_distance = self.manhattan_distance(king_pos, pawn);
                    if king_distance <= 2 {
                        eg_score += 40; // King supports passed pawn
                    }
                }

                // Additional bonus if opponent king is far away
                if let Some(opp_king_pos) = self.find_king_position(board, player.opposite()) {
                    let opp_distance = self.manhattan_distance(opp_king_pos, pawn);
                    if opp_distance >= 4 {
                        eg_score += 50; // Unstoppable passed pawn
                    }
                }

                mg_score += base_mg;
                eg_score += base_eg;
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Calculate Manhattan distance between two positions
    fn manhattan_distance(&self, pos1: Position, pos2: Position) -> i32 {
        (pos1.row as i32 - pos2.row as i32).abs() + (pos1.col as i32 - pos2.col as i32).abs()
    }

    // =======================================================================
    // PIECE COORDINATION IN ENDGAME
    // =======================================================================

    /// Evaluate piece coordination for mating attacks
    fn evaluate_piece_coordination(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Rook and bishop coordination
        let rook_bishop_coord = self.evaluate_rook_bishop_coordination(board, player);
        mg_score += rook_bishop_coord.mg;
        eg_score += rook_bishop_coord.eg;

        // 2. Double rook coordination
        let double_rook_coord = self.evaluate_double_rook_coordination(board, player);
        mg_score += double_rook_coord.mg;
        eg_score += double_rook_coord.eg;

        // 3. Piece proximity to opponent king
        let king_proximity = self.evaluate_piece_proximity_to_opponent_king(board, player);
        mg_score += king_proximity.mg;
        eg_score += king_proximity.eg;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate rook and bishop coordination
    fn evaluate_rook_bishop_coordination(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let rooks = self.find_pieces(board, player, PieceType::Rook);
        let bishops = self.find_pieces(board, player, PieceType::Bishop);

        if rooks.is_empty() || bishops.is_empty() {
            return TaperedScore::default();
        }

        // Check if rook and bishop work together (attacking same area)
        let mut coordination_count = 0;
        for rook_pos in &rooks {
            for bishop_pos in &bishops {
                if self.manhattan_distance(*rook_pos, *bishop_pos) <= 4 {
                    coordination_count += 1;
                }
            }
        }

        // Coordination more valuable in endgame
        TaperedScore::new_tapered(coordination_count * 15, coordination_count * 35)
    }

    /// Evaluate double rook coordination
    fn evaluate_double_rook_coordination(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let rooks = self.find_pieces(board, player, PieceType::Rook);

        if rooks.len() < 2 {
            return TaperedScore::default();
        }

        // Check if rooks are on the same rank or file
        let mut on_same_line = false;
        for i in 0..rooks.len() {
            for j in i + 1..rooks.len() {
                if rooks[i].row == rooks[j].row || rooks[i].col == rooks[j].col {
                    on_same_line = true;
                    break;
                }
            }
        }

        if on_same_line {
            // Double rooks powerful in endgame
            TaperedScore::new_tapered(30, 60)
        } else {
            TaperedScore::default()
        }
    }

    /// Evaluate piece proximity to opponent king
    fn evaluate_piece_proximity_to_opponent_king(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let opp_king_pos = match self.find_king_position(board, player.opposite()) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check major pieces (rook, bishop, promoted pieces)
        for piece_type in [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::PromotedRook,
            PieceType::PromotedBishop,
        ] {
            for piece_pos in self.find_pieces(board, player, piece_type) {
                let distance = self.manhattan_distance(piece_pos, opp_king_pos);
                if distance <= 3 {
                    let proximity_bonus = (4 - distance) * 20;
                    mg_score += proximity_bonus / 2; // Moderate in middlegame
                    eg_score += proximity_bonus;     // Important in endgame
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    // =======================================================================
    // MATING PATTERN DETECTION
    // =======================================================================

    /// Evaluate mating patterns
    fn evaluate_mating_patterns(&self, board: &BitboardBoard, player: Player, _captured_pieces: &CapturedPieces) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Back rank mate threat
        if self.detect_back_rank_mate_threat(board, player.opposite()) {
            mg_score += 50;  // Dangerous in middlegame
            eg_score += 100; // Critical in endgame
        }

        // 2. Ladder mate pattern (rook/lance on file)
        if self.detect_ladder_mate_pattern(board, player) {
            eg_score += 80; // Strong in endgame
        }

        // 3. Bishop and rook mating net
        if self.detect_bishop_rook_mating_net(board, player) {
            eg_score += 90;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Detect back rank mate threat
    fn detect_back_rank_mate_threat(&self, board: &BitboardBoard, player: Player) -> bool {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return false,
        };

        let back_rank = if player == Player::Black { 8 } else { 0 };

        // King on back rank with limited escape squares
        if king_pos.row != back_rank {
            return false;
        }

        // Check if there are pieces blocking escape
        let escape_squares = self.count_escape_squares(board, king_pos, player);
        
        escape_squares <= 2 // Few escape squares = mate threat
    }

    /// Count escape squares for the king
    fn count_escape_squares(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> i32 {
        let mut count = 0;
        
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let new_row = king_pos.row as i8 + dr;
                let new_col = king_pos.col as i8 + dc;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    
                    // Check if square is free or has enemy piece
                    if !board.is_square_occupied(pos) {
                        count += 1;
                    } else if let Some(piece) = board.get_piece(pos) {
                        if piece.player != player {
                            count += 1; // Can capture
                        }
                    }
                }
            }
        }

        count
    }

    /// Detect ladder mate pattern
    fn detect_ladder_mate_pattern(&self, board: &BitboardBoard, player: Player) -> bool {
        let opp_king_pos = match self.find_king_position(board, player.opposite()) {
            Some(pos) => pos,
            None => return false,
        };

        // Check if we have rook or lance on same file as opponent king
        let rooks = self.find_pieces(board, player, PieceType::Rook);
        let lances = self.find_pieces(board, player, PieceType::Lance);

        for rook_pos in rooks {
            if rook_pos.col == opp_king_pos.col {
                // Rook on same file, check if king is trapped
                if opp_king_pos.row == 0 || opp_king_pos.row == 8 {
                    return true; // Ladder mate possible
                }
            }
        }

        for lance_pos in lances {
            if lance_pos.col == opp_king_pos.col {
                // Lance pointing at king
                let pointing_at_king = if player == Player::Black {
                    lance_pos.row > opp_king_pos.row
                } else {
                    lance_pos.row < opp_king_pos.row
                };
                
                if pointing_at_king && (opp_king_pos.row == 0 || opp_king_pos.row == 8) {
                    return true;
                }
            }
        }

        false
    }

    /// Detect bishop and rook mating net
    fn detect_bishop_rook_mating_net(&self, board: &BitboardBoard, player: Player) -> bool {
        let opp_king_pos = match self.find_king_position(board, player.opposite()) {
            Some(pos) => pos,
            None => return false,
        };

        let rooks = self.find_pieces(board, player, PieceType::Rook);
        let bishops = self.find_pieces(board, player, PieceType::Bishop);

        if rooks.is_empty() || bishops.is_empty() {
            return false;
        }

        // Check if opponent king is in corner or edge
        let is_edge = opp_king_pos.row == 0 || opp_king_pos.row == 8 || 
                      opp_king_pos.col == 0 || opp_king_pos.col == 8;

        if !is_edge {
            return false;
        }

        // Check if rook and bishop are close to king
        for rook_pos in &rooks {
            for bishop_pos in &bishops {
                let rook_dist = self.manhattan_distance(*rook_pos, opp_king_pos);
                let bishop_dist = self.manhattan_distance(*bishop_pos, opp_king_pos);

                if rook_dist <= 3 && bishop_dist <= 3 {
                    return true; // Mating net forming
                }
            }
        }

        false
    }

    // =======================================================================
    // MAJOR PIECE ACTIVITY IN ENDGAME
    // =======================================================================

    /// Evaluate major piece (rook/bishop) activity in endgame
    fn evaluate_major_piece_activity(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Rook on 7th rank (or opponent's 2nd rank)
        let seventh_rank = if player == Player::Black { 1 } else { 7 };
        let rooks_on_7th = self.count_pieces_on_rank(board, player, PieceType::Rook, seventh_rank);
        mg_score += rooks_on_7th * 25;
        eg_score += rooks_on_7th * 50;

        // 2. Bishop on long diagonal
        let bishops_on_diagonal = self.count_bishops_on_long_diagonal(board, player);
        mg_score += bishops_on_diagonal * 20;
        eg_score += bishops_on_diagonal * 40;

        // 3. Centralized major pieces
        for piece_type in [PieceType::Rook, PieceType::Bishop, PieceType::PromotedRook, PieceType::PromotedBishop] {
            for piece_pos in self.find_pieces(board, player, piece_type) {
                if self.is_centralized(piece_pos) {
                    mg_score += 15;
                    eg_score += 30;
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Count pieces on a specific rank
    fn count_pieces_on_rank(&self, board: &BitboardBoard, player: Player, piece_type: PieceType, rank: u8) -> i32 {
        let mut count = 0;
        for col in 0..9 {
            let pos = Position::new(rank, col);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player && piece.piece_type == piece_type {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count bishops on long diagonal
    fn count_bishops_on_long_diagonal(&self, board: &BitboardBoard, player: Player) -> i32 {
        let bishops = self.find_pieces(board, player, PieceType::Bishop);
        let mut count = 0;

        for bishop_pos in bishops {
            // Main diagonals: (0,0)-(8,8) and (0,8)-(8,0)
            if bishop_pos.row == bishop_pos.col || bishop_pos.row + bishop_pos.col == 8 {
                count += 1;
            }
        }

        count
    }

    /// Check if position is centralized
    fn is_centralized(&self, pos: Position) -> bool {
        pos.row >= 3 && pos.row <= 5 && pos.col >= 3 && pos.col <= 5
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

    /// Check if pawn is passed
    fn is_passed_pawn(&self, board: &BitboardBoard, pawn_pos: Position, player: Player) -> bool {
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
                        return false;
                    }
                }
                check_row += direction;
            }
        }

        true
    }

    /// Find all pieces of a specific type
    fn find_pieces(&self, board: &BitboardBoard, player: Player, piece_type: PieceType) -> Vec<Position> {
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
    pub fn stats(&self) -> &EndgamePatternStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = EndgamePatternStats::default();
    }
}

impl Default for EndgamePatternEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for endgame pattern evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndgamePatternConfig {
    /// Enable king activity evaluation
    pub enable_king_activity: bool,
    /// Enable passed pawn evaluation
    pub enable_passed_pawns: bool,
    /// Enable piece coordination evaluation
    pub enable_piece_coordination: bool,
    /// Enable mating pattern detection
    pub enable_mating_patterns: bool,
    /// Enable major piece activity evaluation
    pub enable_major_piece_activity: bool,
}

impl Default for EndgamePatternConfig {
    fn default() -> Self {
        Self {
            enable_king_activity: true,
            enable_passed_pawns: true,
            enable_piece_coordination: true,
            enable_mating_patterns: true,
            enable_major_piece_activity: true,
        }
    }
}

/// Statistics for endgame pattern evaluation
#[derive(Debug, Clone, Default)]
pub struct EndgamePatternStats {
    /// Number of evaluations performed
    pub evaluations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endgame_evaluator_creation() {
        let evaluator = EndgamePatternEvaluator::new();
        assert!(evaluator.config.enable_king_activity);
    }

    #[test]
    fn test_king_activity() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_king_activity(&board, Player::Black);
        
        // Starting position: king on back rank, not centralized
        assert!(score.eg >= 0); // Should have some activity potential
    }

    #[test]
    fn test_distance_to_center() {
        let evaluator = EndgamePatternEvaluator::new();
        
        let center = Position::new(4, 4);
        assert_eq!(evaluator.distance_to_center(center), 0);
        
        let corner = Position::new(0, 0);
        assert_eq!(evaluator.distance_to_center(corner), 8);
        
        let edge = Position::new(4, 0);
        assert_eq!(evaluator.distance_to_center(edge), 4);
    }

    #[test]
    fn test_manhattan_distance() {
        let evaluator = EndgamePatternEvaluator::new();
        
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 4);
        
        assert_eq!(evaluator.manhattan_distance(pos1, pos2), 7); // 3 + 4
    }

    #[test]
    fn test_passed_pawn_endgame() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::empty();
        
        let score = evaluator.evaluate_passed_pawns_endgame(&board, Player::Black);
        
        // Empty board, no pawns
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_piece_coordination() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        let score = evaluator.evaluate_piece_coordination(&board, Player::Black);
        
        // Should have some coordination in starting position
        assert!(score.mg >= 0);
    }

    #[test]
    fn test_mating_patterns() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let score = evaluator.evaluate_mating_patterns(&board, Player::Black, &captured_pieces);
        
        // Starting position shouldn't have immediate mate threats
        assert_eq!(score.mg, 0);
    }

    #[test]
    fn test_major_piece_activity() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        let score = evaluator.evaluate_major_piece_activity(&board, Player::Black);
        
        // Starting position has inactive major pieces
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_find_pieces() {
        let evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        let rooks = evaluator.find_pieces(&board, Player::Black, PieceType::Rook);
        assert_eq!(rooks.len(), 1); // One rook per player in starting position
        
        let bishops = evaluator.find_pieces(&board, Player::Black, PieceType::Bishop);
        assert_eq!(bishops.len(), 1);
    }

    #[test]
    fn test_is_centralized() {
        let evaluator = EndgamePatternEvaluator::new();
        
        assert!(evaluator.is_centralized(Position::new(4, 4)));
        assert!(evaluator.is_centralized(Position::new(3, 5)));
        assert!(!evaluator.is_centralized(Position::new(0, 0)));
        assert!(!evaluator.is_centralized(Position::new(8, 8)));
    }

    #[test]
    fn test_statistics() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        assert_eq!(evaluator.stats().evaluations, 0);

        evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);
    }

    #[test]
    fn test_config_options() {
        let config = EndgamePatternConfig {
            enable_king_activity: true,
            enable_passed_pawns: false,
            enable_piece_coordination: true,
            enable_mating_patterns: false,
            enable_major_piece_activity: true,
        };

        let evaluator = EndgamePatternEvaluator::with_config(config);
        assert!(evaluator.config.enable_king_activity);
        assert!(!evaluator.config.enable_passed_pawns);
    }

    #[test]
    fn test_endgame_evaluation_consistency() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score1 = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate_endgame(&board, Player::Black, &captured_pieces);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_escape_squares() {
        let evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::empty();
        
        // King in center has 8 escape squares on empty board
        let king_pos = Position::new(4, 4);
        let escape_count = evaluator.count_escape_squares(&board, king_pos, Player::Black);
        assert_eq!(escape_count, 8);
        
        // King in corner has fewer escape squares
        let corner_king = Position::new(0, 0);
        let corner_escape = evaluator.count_escape_squares(&board, corner_king, Player::Black);
        assert_eq!(corner_escape, 3);
    }
}

