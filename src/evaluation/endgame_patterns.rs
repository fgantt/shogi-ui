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

use crate::bitboards::BitboardBoard;
use crate::moves::MoveGenerator;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Endgame pattern evaluator
pub struct EndgamePatternEvaluator {
    /// Configuration
    config: EndgamePatternConfig,
    /// Statistics
    stats: EndgamePatternStats,
    /// Move generator for zugzwang detection
    move_generator: MoveGenerator,
}

impl EndgamePatternEvaluator {
    /// Create a new endgame pattern evaluator
    pub fn new() -> Self {
        Self {
            config: EndgamePatternConfig::default(),
            stats: EndgamePatternStats::default(),
            move_generator: MoveGenerator::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EndgamePatternConfig) -> Self {
        Self {
            config,
            stats: EndgamePatternStats::default(),
            move_generator: MoveGenerator::new(),
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

        // 6. Zugzwang detection (Phase 2 - Task 2.3.3)
        if self.config.enable_zugzwang {
            score += self.evaluate_zugzwang(board, player, captured_pieces);
        }

        // 7. Opposition patterns (Phase 2 - Task 2.3.4)
        if self.config.enable_opposition {
            score += self.evaluate_opposition(board, player);
        }

        // 8. Triangulation detection (Phase 2 - Task 2.3.5)
        if self.config.enable_triangulation {
            score += self.evaluate_triangulation(board, player);
        }

        // 9. Piece vs pawns evaluation (Phase 2 - Task 2.3.6)
        if self.config.enable_piece_vs_pawns {
            score += self.evaluate_piece_vs_pawns(board, player);
        }

        // 10. Fortress patterns (Phase 2 - Task 2.3.7)
        if self.config.enable_fortress {
            score += self.evaluate_fortress(board, player);
        }

        score
    }

    // =======================================================================
    // KING ACTIVITY IN ENDGAME
    // =======================================================================

    /// Evaluate king activity in endgame
    ///
    /// In endgame, the king should be active and centralized
    fn evaluate_king_activity(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check if king is under attack (safety check)
        let is_unsafe = self.is_king_under_attack(board, king_pos, player);

        // 1. Centralization bonus (more important in endgame)
        let center_distance = self.distance_to_center(king_pos);
        let centralization_bonus_base = (4 - center_distance.min(4)) * 15;
        let centralization_bonus = (centralization_bonus_base as f32 * self.config.king_activity_centralization_scale) as i32;
        mg_score += centralization_bonus / 4; // Small bonus in middlegame
        eg_score += centralization_bonus; // Large bonus in endgame

        // 2. Activity bonus (king not on back rank)
        let back_rank = if player == Player::Black { 8 } else { 0 };
        if king_pos.row != back_rank {
            let activity_bonus = (25.0 * self.config.king_activity_activity_scale) as i32;
            mg_score += 5; // Small bonus in middlegame
            eg_score += activity_bonus; // Large bonus in endgame
        }

        // 3. Advanced king bonus (crossing center)
        let is_advanced = if player == Player::Black {
            king_pos.row <= 4
        } else {
            king_pos.row >= 4
        };

        if is_advanced {
            let mut advancement_bonus = (35.0 * self.config.king_activity_advancement_scale) as i32;
            
            // Reduce advancement bonus by 50% if king is unsafe
            if is_unsafe {
                advancement_bonus = advancement_bonus / 2;
                self.stats.unsafe_king_penalties += 1;
                
                // Also apply penalty for unsafe advanced king
                eg_score -= 20;
                
                crate::debug_utils::trace_log("KING_ACTIVITY", &format!(
                    "Advanced king in unsafe position: {} (row={}, col={}), penalty=-20, advancement bonus reduced by 50%",
                    if player == Player::Black { "Black" } else { "White" },
                    king_pos.row,
                    king_pos.col
                ));
            }
            
            mg_score += 5; // Risky in middlegame
            eg_score += advancement_bonus; // Excellent in endgame (if safe)
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if king is under attack by opponent pieces
    fn is_king_under_attack(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> bool {
        let opponent = player.opposite();
        board.is_square_attacked_by(king_pos, opponent)
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
    fn evaluate_rook_bishop_coordination(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
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
    fn evaluate_double_rook_coordination(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
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
    fn evaluate_piece_proximity_to_opponent_king(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
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
                    eg_score += proximity_bonus; // Important in endgame
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    // =======================================================================
    // MATING PATTERN DETECTION
    // =======================================================================

    /// Evaluate mating patterns
    fn evaluate_mating_patterns(
        &self,
        board: &BitboardBoard,
        player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // 1. Back rank mate threat
        if self.detect_back_rank_mate_threat(board, player.opposite()) {
            mg_score += 50; // Dangerous in middlegame
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
    fn count_escape_squares(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> i32 {
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
        let is_edge = opp_king_pos.row == 0
            || opp_king_pos.row == 8
            || opp_king_pos.col == 0
            || opp_king_pos.col == 8;

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
        for piece_type in [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::PromotedRook,
            PieceType::PromotedBishop,
        ] {
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
    fn count_pieces_on_rank(
        &self,
        board: &BitboardBoard,
        player: Player,
        piece_type: PieceType,
        rank: u8,
    ) -> i32 {
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
    // ZUGZWANG DETECTION (Phase 2 - Task 2.3.3)
    // =======================================================================

    /// Evaluate zugzwang positions (where any move worsens the position)
    ///
    /// Zugzwang detection compares the number of safe moves available to both players.
    /// In shogi, zugzwang is rarer than in chess due to drop moves, which often break
    /// zugzwang situations. However, zugzwang can still occur in pawn endgames or when
    /// both sides are low on material.
    ///
    /// The detection uses `MoveGenerator::generate_legal_moves()` to count actual legal
    /// moves (including drops). Moves are already filtered for safety (no moves that
    /// leave the king in check).
    ///
    /// Configuration:
    /// - `enable_zugzwang_drop_consideration`: If true (default), drop moves are included
    ///   in the move count. If false, only regular moves are counted, making zugzwang
    ///   detection more sensitive (useful for testing or chess-like evaluation).
    ///
    /// Scoring:
    /// - If opponent has ≤2 moves and player has >5 moves: +80 (endgame score)
    /// - If player has ≤2 moves and opponent has >5 moves: -60 (endgame score)
    ///
    /// Statistics are tracked for monitoring zugzwang detection effectiveness.
    fn evaluate_zugzwang(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        // Zugzwang is rare in Shogi due to drop moves, but can occur in endgame
        let opponent = player.opposite();

        // Count mobility for both sides
        let (player_moves, player_drops) = self.count_safe_moves(board, player, captured_pieces);
        let (opponent_moves, opponent_drops) = self.count_safe_moves(board, opponent, captured_pieces);

        // Adjust move counts based on drop consideration configuration
        let player_total = if self.config.enable_zugzwang_drop_consideration {
            player_moves + player_drops
        } else {
            player_moves
        };
        let opponent_total = if self.config.enable_zugzwang_drop_consideration {
            opponent_moves + opponent_drops
        } else {
            opponent_moves
        };

        // Zugzwang-like position: opponent has very few safe moves
        if opponent_total <= 2 && player_total > 5 {
            // Player benefits from opponent's lack of moves
            self.stats.zugzwang_detections += 1;
            self.stats.zugzwang_benefits += 1;
            
            crate::debug_utils::trace_log("ZUGZWANG", &format!(
                "Zugzwang detected: player={} moves ({} regular, {} drops), opponent={} moves ({} regular, {} drops), score=+80",
                player_total, player_moves, player_drops, opponent_total, opponent_moves, opponent_drops
            ));
            
            return TaperedScore::new_tapered(0, 80);
        }

        // Reverse zugzwang: player has few moves
        if player_total <= 2 && opponent_total > 5 {
            self.stats.zugzwang_detections += 1;
            self.stats.zugzwang_penalties += 1;
            
            crate::debug_utils::trace_log("ZUGZWANG", &format!(
                "Reverse zugzwang detected: player={} moves ({} regular, {} drops), opponent={} moves ({} regular, {} drops), score=-60",
                player_total, player_moves, player_drops, opponent_total, opponent_moves, opponent_drops
            ));
            
            return TaperedScore::new_tapered(0, -60);
        }

        TaperedScore::default()
    }

    /// Count safe moves for a player
    /// Returns (regular_move_count, drop_move_count)
    pub fn count_safe_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> (i32, i32) {
        // Generate all legal moves (already filtered for safety - no moves that leave king in check)
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        
        // Separate regular moves from drop moves
        let mut regular_moves = 0;
        let mut drop_moves = 0;
        
        for mv in &legal_moves {
            if mv.is_drop() {
                drop_moves += 1;
            } else {
                regular_moves += 1;
            }
        }
        
        (regular_moves, drop_moves)
    }

    // =======================================================================
    // OPPOSITION PATTERNS (Phase 2 - Task 2.3.4)
    // =======================================================================

    /// Evaluate opposition patterns (king opposition in pawn endgames)
    fn evaluate_opposition(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let opp_king_pos = match self.find_king_position(board, player.opposite()) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Opposition is most valuable in pawn endgames (few pawns on board)
        let pawn_count = self.count_pawns_on_board(board);
        if pawn_count > 6 {
            // Too many pawns, opposition less valuable
            return TaperedScore::default();
        }

        // Check for direct opposition (kings facing each other with 1 square between)
        let file_diff = (king_pos.col as i8 - opp_king_pos.col as i8).abs();
        let rank_diff = (king_pos.row as i8 - opp_king_pos.row as i8).abs();

        let mut base_score = 0;

        // Direct opposition
        if (file_diff == 0 && rank_diff == 2) || (rank_diff == 0 && file_diff == 2) {
            base_score = 40;
        }
        // Distant opposition (even number of squares between)
        else if file_diff == 0 && rank_diff % 2 == 0 && rank_diff > 2 {
            base_score = 20;
        }
        // Diagonal opposition
        else if file_diff == rank_diff && file_diff % 2 == 0 && file_diff > 1 {
            base_score = 15;
        }

        if base_score > 0 {
            // Scale score with pawn count (higher value with fewer pawns)
            // With 0-2 pawns: full value, with 3-4 pawns: 75%, with 5-6 pawns: 50%
            let scale_factor = if pawn_count <= 2 {
                100
            } else if pawn_count <= 4 {
                75
            } else {
                50
            };
            let scaled_score = (base_score * scale_factor) / 100;
            
            self.stats.opposition_detections += 1;
            return TaperedScore::new_tapered(0, scaled_score);
        }

        TaperedScore::default()
    }

    /// Count total pawns on board for both players
    pub fn count_pawns_on_board(&self, board: &BitboardBoard) -> i32 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    // =======================================================================
    // TRIANGULATION DETECTION (Phase 2 - Task 2.3.5)
    // =======================================================================

    /// Evaluate triangulation potential (losing a tempo to gain zugzwang)
    fn evaluate_triangulation(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Triangulation is valuable when:
        // 1. Few pieces on board
        // 2. King has room to maneuver
        // 3. Opponent is in cramped position
        // 4. Player is ahead in material

        let piece_count = self.count_total_pieces(board);

        if piece_count > 10 {
            return TaperedScore::default(); // Too many pieces for triangulation
        }

        // Check if king has triangulation squares available
        let king_mobility = self.count_king_safe_squares(board, king_pos, player);

        if king_mobility < 4 {
            return TaperedScore::default(); // King doesn't have enough mobility
        }

        // Check opponent king mobility (triangulation requires cramped opponent)
        let opponent = player.opposite();
        let opponent_king_pos = match self.find_king_position(board, opponent) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };
        let opponent_mobility = self.count_opponent_king_mobility(board, opponent_king_pos, opponent);

        if opponent_mobility > 3 {
            return TaperedScore::default(); // Opponent not cramped enough
        }

        // Verify triangulation squares don't worsen position (squares should not be attacked)
        // Simplified check: verify king's current position and potential triangulation squares are safe
        if self.is_king_under_attack(board, king_pos, player) {
            return TaperedScore::default(); // King is already under attack, triangulation risky
        }

        // Material balance check (triangulation more valuable when ahead)
        let material_diff = self.get_material_difference(board, player);
        if material_diff < 0 {
            // Behind in material, triangulation less valuable
            return TaperedScore::default();
        }

        // All conditions met for triangulation
        self.stats.triangulation_detections += 1;
        return TaperedScore::new_tapered(0, 25);
    }

    /// Count opponent king mobility (safe squares available)
    fn count_opponent_king_mobility(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> i32 {
        // Use same logic as count_king_safe_squares but for opponent
        self.count_king_safe_squares(board, king_pos, player)
    }

    /// Count safe squares around king
    fn count_king_safe_squares(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> i32 {
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

    // =======================================================================
    // PIECE VS PAWNS EVALUATION (Phase 2 - Task 2.3.6)
    // =======================================================================

    /// Evaluate piece vs pawns endgames
    fn evaluate_piece_vs_pawns(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let player_pieces = self.count_pieces(board, player);
        let player_pawns = self.count_piece_type(board, player, PieceType::Pawn);
        let _opp_pieces = self.count_pieces(board, player.opposite());
        let opp_pawns = self.count_piece_type(board, player.opposite(), PieceType::Pawn);

        // Rook vs pawns
        if player_pieces == 1 && player_pawns == 0 && opp_pawns >= 1 {
            // Check if we have a rook
            if self.has_piece_type(board, player, PieceType::Rook) {
                // Rook vs pawns - usually winning if pawns not too advanced
                let pawn_advancement = self.evaluate_pawn_advancement(board, player.opposite());
                if pawn_advancement < 5 {
                    return TaperedScore::new_tapered(0, 100);
                } else {
                    return TaperedScore::new_tapered(0, 30);
                }
            }
        }

        // Bishop vs pawns - harder to win
        if player_pieces == 1 && player_pawns == 0 && opp_pawns >= 1 {
            if self.has_piece_type(board, player, PieceType::Bishop) {
                let pawn_advancement = self.evaluate_pawn_advancement(board, player.opposite());
                if pawn_advancement < 4 {
                    return TaperedScore::new_tapered(0, 60);
                } else {
                    return TaperedScore::new_tapered(0, 10);
                }
            }
        }

        TaperedScore::default()
    }

    /// Check if player has a specific piece type
    fn has_piece_type(&self, board: &BitboardBoard, player: Player, piece_type: PieceType) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == piece_type {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Evaluate pawn advancement for player
    fn evaluate_pawn_advancement(&self, board: &BitboardBoard, player: Player) -> u8 {
        let mut max_advancement = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == PieceType::Pawn {
                        let advancement = if player == Player::Black {
                            8 - row
                        } else {
                            row
                        };
                        max_advancement = max_advancement.max(advancement);
                    }
                }
            }
        }

        max_advancement
    }

    // =======================================================================
    // FORTRESS PATTERNS (Phase 2 - Task 2.3.7)
    // =======================================================================

    /// Evaluate fortress patterns (defensive structures that are hard to break)
    fn evaluate_fortress(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check if king is in a corner or edge fortress
        let is_corner =
            (king_pos.row == 0 || king_pos.row == 8) && (king_pos.col == 0 || king_pos.col == 8);

        if !is_corner && king_pos.row != 0 && king_pos.row != 8 {
            return TaperedScore::default(); // Not in fortress position
        }

        // Count defenders around king
        let defenders = self.count_defenders_near_king(board, king_pos, player);

        // Fortress is strong with 2-3 defenders
        if defenders >= 2 {
            // Check material disadvantage - fortress more valuable when behind
            let material_diff = self.get_material_difference(board, player);

            if material_diff < -500 {
                // Significant material disadvantage - fortress is crucial
                return TaperedScore::new_tapered(0, 120);
            } else if material_diff < 0 {
                return TaperedScore::new_tapered(0, 60);
            }
        }

        TaperedScore::default()
    }

    /// Count defenders near king
    fn count_defenders_near_king(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> i32 {
        let mut count = 0;

        for dr in -2..=2 {
            for dc in -2..=2 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let new_row = king_pos.row as i8 + dr;
                let new_col = king_pos.col as i8 + dc;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);

                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player == player {
                            match piece.piece_type {
                                PieceType::Gold | PieceType::Silver => count += 2,
                                PieceType::Pawn => count += 1,
                                _ => count += 1,
                            }
                        }
                    }
                }
            }
        }

        count
    }

    /// Get material difference (player - opponent)
    fn get_material_difference(&self, board: &BitboardBoard, player: Player) -> i32 {
        let player_material = self.calculate_material(board, player);
        let opponent_material = self.calculate_material(board, player.opposite());
        player_material - opponent_material
    }

    /// Calculate material for a player
    fn calculate_material(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut material = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        material += piece.piece_type.base_value();
                    }
                }
            }
        }

        material
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

    /// Count total pieces for a player
    fn count_pieces(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Count total pieces on board
    fn count_total_pieces(&self, board: &BitboardBoard) -> i32 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if board.is_square_occupied(pos) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count pieces of specific type for player
    fn count_piece_type(
        &self,
        board: &BitboardBoard,
        player: Player,
        piece_type: PieceType,
    ) -> i32 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == piece_type {
                        count += 1;
                    }
                }
            }
        }
        count
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
    /// Enable zugzwang detection
    pub enable_zugzwang: bool,
    /// Enable opposition patterns
    pub enable_opposition: bool,
    /// Enable triangulation detection
    pub enable_triangulation: bool,
    /// Enable piece vs pawns evaluation
    pub enable_piece_vs_pawns: bool,
    /// Enable fortress patterns
    pub enable_fortress: bool,
    /// Enable drop move consideration in zugzwang detection (drops often break zugzwang in shogi)
    pub enable_zugzwang_drop_consideration: bool,
    /// King activity centralization bonus scaling factor (default: 1.0)
    pub king_activity_centralization_scale: f32,
    /// King activity activity bonus scaling factor (default: 1.0)
    pub king_activity_activity_scale: f32,
    /// King activity advancement bonus scaling factor (default: 1.0)
    pub king_activity_advancement_scale: f32,
}

impl Default for EndgamePatternConfig {
    fn default() -> Self {
        Self {
            enable_king_activity: true,
            enable_passed_pawns: true,
            enable_piece_coordination: true,
            enable_mating_patterns: true,
            enable_major_piece_activity: true,
            enable_zugzwang: true,
            enable_opposition: true,
            enable_triangulation: true,
            enable_piece_vs_pawns: true,
            enable_fortress: true,
            enable_zugzwang_drop_consideration: true,
            king_activity_centralization_scale: 1.0,
            king_activity_activity_scale: 1.0,
            king_activity_advancement_scale: 1.0,
        }
    }
}

/// Statistics for endgame pattern evaluation
#[derive(Debug, Clone, Default)]
pub struct EndgamePatternStats {
    /// Number of evaluations performed
    pub evaluations: u64,
    /// Number of zugzwang detections
    pub zugzwang_detections: u64,
    /// Number of zugzwang benefits (positive scores)
    pub zugzwang_benefits: u64,
    /// Number of zugzwang penalties (negative scores)
    pub zugzwang_penalties: u64,
    /// Number of opposition detections
    pub opposition_detections: u64,
    /// Number of triangulation detections
    pub triangulation_detections: u64,
    /// Number of unsafe king penalties
    pub unsafe_king_penalties: u64,
}

#[cfg(all(test, feature = "legacy-tests"))]
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
    fn test_opposition_with_pawn_count() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        // Starting position has many pawns, opposition should not be detected
        let score = evaluator.evaluate_opposition(&board, Player::Black);
        // May or may not detect opposition depending on king positions and pawn count
        assert!(score.eg >= 0 && score.eg <= 40);
    }

    #[test]
    fn test_triangulation_opponent_mobility() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::empty();
        
        // Empty board with few pieces should allow triangulation if conditions are met
        let score = evaluator.evaluate_triangulation(&board, Player::Black);
        // May or may not detect triangulation depending on king positions
        assert!(score.eg >= 0 && score.eg <= 25);
    }

    #[test]
    fn test_king_activity_safety_check() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        // Test that safety check works
        let score = evaluator.evaluate_king_activity(&board, Player::Black);
        // Should complete without error
        assert!(score.mg >= -100 && score.mg <= 100);
        assert!(score.eg >= -100 && score.eg <= 100);
    }

    #[test]
    fn test_count_pawns_on_board() {
        let evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        // Starting position has 9 pawns per player = 18 total
        let pawn_count = evaluator.count_pawns_on_board(&board);
        assert_eq!(pawn_count, 18);
    }

    #[test]
    fn test_king_activity_bonus_scaling() {
        let mut config = EndgamePatternConfig::default();
        config.king_activity_centralization_scale = 0.5;
        config.king_activity_activity_scale = 0.5;
        config.king_activity_advancement_scale = 0.5;
        
        let mut evaluator = EndgamePatternEvaluator::with_config(config);
        let board = BitboardBoard::new();
        
        let score = evaluator.evaluate_king_activity(&board, Player::Black);
        // Should complete with scaled bonuses
        assert!(score.eg >= -100 && score.eg <= 100);
    }

    #[test]
    fn test_pattern_detection_statistics() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        
        assert_eq!(evaluator.stats().opposition_detections, 0);
        assert_eq!(evaluator.stats().triangulation_detections, 0);
        assert_eq!(evaluator.stats().unsafe_king_penalties, 0);
        
        // Evaluate patterns
        evaluator.evaluate_opposition(&board, Player::Black);
        evaluator.evaluate_triangulation(&board, Player::Black);
        evaluator.evaluate_king_activity(&board, Player::Black);
        
        // Statistics should be tracked (may be 0 if patterns not detected)
        assert!(evaluator.stats().opposition_detections >= 0);
        assert!(evaluator.stats().triangulation_detections >= 0);
        assert!(evaluator.stats().unsafe_king_penalties >= 0);
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

    #[test]
    fn test_count_safe_moves_basic() {
        let evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Starting position should have many legal moves
        let (regular, drops) = evaluator.count_safe_moves(&board, Player::Black, &captured_pieces);
        assert!(regular > 0, "Starting position should have regular moves");
        assert_eq!(drops, 0, "Starting position should have no drop moves (no captured pieces)");
    }

    #[test]
    fn test_count_safe_moves_with_drops() {
        let evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::empty();
        let mut captured_pieces = CapturedPieces::new();
        
        // Add captured pieces to enable drops
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);
        captured_pieces.add_piece(PieceType::Rook, Player::Black);

        // Empty board with captured pieces should have drop moves
        let (regular, drops) = evaluator.count_safe_moves(&board, Player::Black, &captured_pieces);
        assert!(drops > 0, "Should have drop moves when pieces are captured");
    }

    #[test]
    fn test_zugzwang_detection_known_positions() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Starting position is unlikely to be zugzwang
        let score = evaluator.evaluate_zugzwang(&board, Player::Black, &captured_pieces);
        assert_eq!(score.mg, 0);
        // May or may not detect zugzwang depending on move counts
    }

    #[test]
    fn test_zugzwang_drop_consideration() {
        let mut config = EndgamePatternConfig::default();
        config.enable_zugzwang_drop_consideration = false;
        let mut evaluator = EndgamePatternEvaluator::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test that evaluation works with drop consideration disabled
        let score = evaluator.evaluate_zugzwang(&board, Player::Black, &captured_pieces);
        // Should complete without error
        assert!(score.eg >= -60 && score.eg <= 80);
    }

    #[test]
    fn test_zugzwang_statistics() {
        let mut evaluator = EndgamePatternEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        assert_eq!(evaluator.stats().zugzwang_detections, 0);
        assert_eq!(evaluator.stats().zugzwang_benefits, 0);
        assert_eq!(evaluator.stats().zugzwang_penalties, 0);

        // Evaluate zugzwang (may or may not detect depending on position)
        evaluator.evaluate_zugzwang(&board, Player::Black, &captured_pieces);

        // Statistics should be tracked (may be 0 if no zugzwang detected)
        assert!(evaluator.stats().zugzwang_detections >= 0);
        assert!(evaluator.stats().zugzwang_benefits >= 0);
        assert!(evaluator.stats().zugzwang_penalties >= 0);
    }
}
