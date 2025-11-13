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
    /// * `captured_pieces` - Current captured pieces state (for drop pressure evaluation)
    /// * `move_history` - Optional move history for repeated move detection
    pub fn evaluate_opening(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        move_count: u32,
        captured_pieces: Option<&CapturedPieces>,
        move_history: Option<&[Move]>,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut score = TaperedScore::default();

        // 1. Piece development
        if self.config.enable_development {
            let dev_score = self.evaluate_development(board, player, move_count);
            let dev_score_interp = dev_score.interpolate(256);
            self.stats.development_score += dev_score_interp as i64;
            self.stats.development_evaluations += 1;
            score += dev_score;
        }

        // 2. Center control
        if self.config.enable_center_control {
            let mut center_score = self.evaluate_center_control_opening(board, player);
            
            // Drop pressure evaluation (Task 19.0 - Task 5.0)
            if self.config.enable_drop_pressure_evaluation {
                if let Some(captured) = captured_pieces {
                    let drop_score = self.evaluate_drop_pressure_on_center(board, player, captured);
                    center_score += drop_score;
                }
            }
            
            let center_score_interp = center_score.interpolate(256);
            self.stats.center_control_score += center_score_interp as i64;
            self.stats.center_control_evaluations += 1;
            score += center_score;
        }

        // 3. Castle formation (defensive structure)
        if self.config.enable_castle_formation {
            let castle_score = self.evaluate_castle_formation(board, player);
            let castle_score_interp = castle_score.interpolate(256);
            self.stats.castle_formation_score += castle_score_interp as i64;
            self.stats.castle_formation_evaluations += 1;
            score += castle_score;
        }

        // 4. Tempo evaluation
        if self.config.enable_tempo {
            let tempo_score = self.evaluate_tempo(board, player, move_count);
            let tempo_score_interp = tempo_score.interpolate(256);
            self.stats.tempo_score += tempo_score_interp as i64;
            self.stats.tempo_evaluations += 1;
            score += tempo_score;
        }

        // 5. Opening-specific penalties
        if self.config.enable_opening_penalties {
            let penalties_score = self.evaluate_opening_penalties(board, player, move_count, move_history);
            let penalties_score_interp = penalties_score.interpolate(256);
            self.stats.penalties_score += penalties_score_interp as i64;
            self.stats.penalties_evaluations += 1;
            score += penalties_score;
        }

        // 6. Piece coordination (Task 19.0 - Task 2.0)
        if self.config.enable_piece_coordination {
            let coord_score = self.evaluate_piece_coordination(board, player);
            let coord_score_interp = coord_score.interpolate(256);
            self.stats.piece_coordination_score += coord_score_interp as i64;
            self.stats.piece_coordination_evaluations += 1;
            score += coord_score;
        }

        // Telemetry: Log component contributions that exceed threshold (Task 19.0 - Task 5.0)
        self.log_component_contributions(&score, move_count);

        score
    }

    /// Log component contributions when they exceed threshold (Task 19.0 - Task 5.0)
    fn log_component_contributions(&self, total_score: &TaperedScore, move_count: u32) {
        const THRESHOLD_CP: i32 = 100; // Log when component contributes > 100cp
        
        #[cfg(feature = "verbose-debug")]
        {
            use crate::debug_utils::debug_log_fast;
            
            // Check each component's contribution (simplified - would need to track per-component in real-time)
            // For now, we log when total score is significant
            let total_interp = total_score.interpolate(256);
            if total_interp.abs() > THRESHOLD_CP {
                debug_log_fast!(&format!(
                    "[OPENING_PRINCIPLES] Significant contribution at move {}: total={}cp (mg={}, eg={})",
                    move_count,
                    total_interp,
                    total_score.mg,
                    total_score.eg
                ));
            }
        }
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

        // Core center (4,4) and surrounding squares (occupied squares)
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

        // Extended center squares (occupied squares)
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

        // Attack-based center control (Task 19.0 - Task 4.0)
        if self.config.enable_attack_based_center_control {
            let attack_score = self.evaluate_center_control_via_attacks(board, player);
            mg_score += attack_score.mg;
            eg_score += attack_score.eg;
        }

        // Drop pressure evaluation (Task 19.0 - Task 5.0)
        // Note: This requires captured_pieces, so we'll add it as a parameter later
        // For now, we'll skip it in this method signature

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate center control via drop pressure (Task 19.0 - Task 5.0)
    ///
    /// This method evaluates center control based on potential piece drops.
    /// It checks which center squares could be controlled via drops of captured pieces.
    ///
    /// # Arguments
    ///
    /// * `board` - Current board state
    /// * `player` - Player to evaluate for
    /// * `captured_pieces` - Current captured pieces state
    ///
    /// # Returns
    ///
    /// TaperedScore representing drop pressure on center squares
    fn evaluate_drop_pressure_on_center(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Center squares to evaluate
        let center_squares = [
            Position::new(4, 4), // Core center
            Position::new(3, 4), Position::new(5, 4), // Vertical
            Position::new(4, 3), Position::new(4, 5), // Horizontal
            Position::new(3, 3), Position::new(3, 5), // Diagonals
            Position::new(5, 3), Position::new(5, 5),
        ];

        // Pieces that can control center via drops (bishop, rook, silver, gold, knight)
        let drop_pieces = [
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Knight,
        ];

        // Check each center square
        for &center_sq in &center_squares {
            let mut our_drop_pressure = 0;
            let mut their_drop_pressure = 0;

            // Check if square is empty (can be dropped on)
            if !board.is_square_occupied(center_sq) {
                // Check our captured pieces
                for &piece_type in &drop_pieces {
                    let count = captured_pieces.count(piece_type, player);
                    if count > 0 {
                        // Check if this piece could control center square if dropped
                        if self.can_piece_control_square_via_drop(piece_type, center_sq, player, board) {
                            our_drop_pressure += count as i32;
                        }
                    }
                }

                // Check opponent's captured pieces
                let opponent = player.opposite();
                for &piece_type in &drop_pieces {
                    let count = captured_pieces.count(piece_type, opponent);
                    if count > 0 {
                        if self.can_piece_control_square_via_drop(piece_type, center_sq, opponent, board) {
                            their_drop_pressure += count as i32;
                        }
                    }
                }
            }

            // Score based on drop pressure difference
            let pressure_diff = our_drop_pressure - their_drop_pressure;
            let value = if center_sq == Position::new(4, 4) {
                pressure_diff * 6 // Core center is more valuable
            } else {
                pressure_diff * 4 // Extended center
            };

            mg_score += value;
            eg_score += value / 4; // Less important in endgame
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a piece dropped at a position could control a target square (Task 19.0 - Task 5.0)
    fn can_piece_control_square_via_drop(
        &self,
        piece_type: PieceType,
        drop_pos: Position,
        player: Player,
        board: &BitboardBoard,
    ) -> bool {
        // Get attack pattern for this piece type at drop position
        let attacks = board.get_attack_pattern_precomputed(drop_pos, piece_type, player);
        
        // Check if target square is in attack pattern
        // For now, we'll check if the piece could attack center squares
        // This is a simplified check - in reality, we'd need to check if the drop is legal
        // and if the piece could actually control the center from that position
        
        // Center squares
        let center_squares = [
            Position::new(4, 4),
            Position::new(3, 4), Position::new(5, 4),
            Position::new(4, 3), Position::new(4, 5),
            Position::new(3, 3), Position::new(3, 5),
            Position::new(5, 3), Position::new(5, 5),
        ];

        for &center_sq in &center_squares {
            let center_bit = 1u128 << center_sq.to_u8();
            if (attacks & center_bit) != 0 {
                return true; // Piece could attack at least one center square
            }
        }

        false
    }

    /// Evaluate center control via piece attacks (Task 19.0 - Task 4.0)
    ///
    /// This method evaluates center control based on pieces that attack center squares,
    /// not just pieces that occupy them. This provides a more nuanced view of center control.
    fn evaluate_center_control_via_attacks(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Center squares to evaluate
        let center_squares = [
            Position::new(4, 4), // Core center
            Position::new(3, 4), Position::new(5, 4), // Vertical
            Position::new(4, 3), Position::new(4, 5), // Horizontal
            Position::new(3, 3), Position::new(3, 5), // Diagonals
            Position::new(5, 3), Position::new(5, 5),
        ];

        // Check which pieces attack center squares
        for &center_sq in &center_squares {
            let mut our_attacks = 0;
            let mut their_attacks = 0;

            // Check all pieces that might attack this square
            for piece_type in [
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Silver,
                PieceType::Gold,
                PieceType::Knight,
                PieceType::Lance,
            ] {
                // Check our pieces
                for piece_pos in self.find_pieces(board, player, piece_type) {
                    if let Some(piece) = board.get_piece(piece_pos) {
                        // Get attack pattern for this piece
                        let attacks = board.get_attack_pattern_precomputed(
                            piece_pos,
                            piece.piece_type,
                            player,
                        );
                        
                        // Check if center square is in attack pattern
                        // Use bitwise operation: check if bit is set
                        let center_bit = 1u128 << center_sq.to_u8();
                        if (attacks & center_bit) != 0 {
                            our_attacks += 1;
                        }
                    }
                }

                // Check opponent pieces
                let opponent = player.opposite();
                for piece_pos in self.find_pieces(board, opponent, piece_type) {
                    if let Some(piece) = board.get_piece(piece_pos) {
                        let attacks = board.get_attack_pattern_precomputed(
                            piece_pos,
                            piece.piece_type,
                            opponent,
                        );
                        
                        // Check if center square is in attack pattern
                        let center_bit = 1u128 << center_sq.to_u8();
                        if (attacks & center_bit) != 0 {
                            their_attacks += 1;
                        }
                    }
                }
            }

            // Score based on attack count difference
            let attack_diff = our_attacks as i32 - their_attacks as i32;
            let value = if center_sq == Position::new(4, 4) {
                attack_diff * 8 // Core center is more valuable
            } else {
                attack_diff * 5 // Extended center
            };
            
            mg_score += value;
            eg_score += value / 4; // Less important in endgame
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
        move_history: Option<&[Move]>,
    ) -> TaperedScore {
        let mut mg_penalty = 0;

        // Early in opening (first 10 moves)
        if move_count <= 10 {
            // 1. Penalty for moving the same piece multiple times (Task 19.0 - Task 5.0)
            if let Some(history) = move_history {
                let repeated_move_penalty = self.detect_repeated_piece_moves(history, player);
                mg_penalty += repeated_move_penalty;
            }

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

    /// Detect repeated piece moves in opening (Task 19.0 - Task 5.0)
    ///
    /// Returns penalty for moving the same piece multiple times in the opening.
    /// This addresses the TODO in evaluate_opening_penalties.
    fn detect_repeated_piece_moves(&self, move_history: &[Move], player: Player) -> i32 {
        let mut penalty = 0;
        
        // Track how many times each piece position has been moved
        let mut piece_move_count: std::collections::HashMap<(Option<Position>, PieceType), u32> = 
            std::collections::HashMap::new();
        
        // Count moves for this player
        for move_ in move_history.iter() {
            if move_.player == player {
                // Get the piece that was moved (from position and piece type)
                let key = (move_.from, move_.piece_type);
                let count = piece_move_count.entry(key).or_insert(0);
                *count += 1;
                
                // Penalty increases with number of times same piece is moved
                if *count > 1 {
                    // Penalty: 15 cp for 2nd move, 30 cp for 3rd move, etc.
                    penalty += 15 * (*count - 1) as i32;
                    
                    #[cfg(debug_assertions)]
                    crate::debug_utils::debug_log(&format!(
                        "[OPENING_PRINCIPLES] Repeated piece move detected: {:?} moved {} times (penalty: {}cp)",
                        move_.piece_type,
                        *count,
                        15 * (*count - 1)
                    ));
                }
            }
        }
        
        penalty
    }

    // =======================================================================
    // PIECE COORDINATION EVALUATION (Task 19.0 - Task 2.0)
    // =======================================================================

    /// Evaluate piece coordination in opening
    ///
    /// Evaluates coordination bonuses for:
    /// - Rook-lance batteries (same file, both developed)
    /// - Bishop-lance combinations (same diagonal, both developed)
    /// - Gold-silver defensive coordination (near king)
    /// - Rook-bishop coordination (attacking combinations)
    /// - Piece synergy bonuses (developed pieces supporting each other)
    fn evaluate_piece_coordination(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        // 1. Rook-lance battery detection (same file, both developed)
        let rooks = self.find_pieces(board, player, PieceType::Rook);
        let lances = self.find_pieces(board, player, PieceType::Lance);
        
        for rook_pos in &rooks {
            // Check if rook is developed
            let rook_developed = rook_pos.row != start_row;
            
            for lance_pos in &lances {
                // Check if lance is developed
                let lance_developed = lance_pos.row != start_row;
                
                // Check if on same file (same column)
                if rook_pos.col == lance_pos.col && rook_developed && lance_developed {
                    // Rook-lance battery bonus
                    // Rook supports lance on same file
                    mg_score += 25; // Good bonus for rook-lance coordination
                }
            }
        }

        // 2. Bishop-lance combination detection (same diagonal, both developed)
        let bishops = self.find_pieces(board, player, PieceType::Bishop);
        
        for bishop_pos in &bishops {
            let bishop_developed = bishop_pos.row != start_row;
            
            for lance_pos in &lances {
                let lance_developed = lance_pos.row != start_row;
                
                // Check if on same diagonal
                if self.same_diagonal(*bishop_pos, *lance_pos) && bishop_developed && lance_developed {
                    // Bishop-lance combination bonus
                    mg_score += 20; // Moderate bonus for bishop-lance coordination
                }
            }
        }

        // 3. Gold-silver defensive coordination (near king)
        if let Some(king_pos) = self.find_king_position(board, player) {
            let golds = self.find_pieces(board, player, PieceType::Gold);
            let silvers = self.find_pieces(board, player, PieceType::Silver);
            
            // Check for gold-silver pairs near king (within 2 squares)
            for gold_pos in &golds {
                for silver_pos in &silvers {
                    let distance = self.square_distance(*gold_pos, *silver_pos);
                    let gold_near_king = self.square_distance(*gold_pos, king_pos) <= 2;
                    let silver_near_king = self.square_distance(*silver_pos, king_pos) <= 2;
                    
                    // Gold and silver near king and close to each other
                    if gold_near_king && silver_near_king && distance <= 2 {
                        mg_score += 15; // Defensive coordination bonus
                    }
                }
            }
        }

        // 4. Rook-bishop coordination (attacking combinations, both developed)
        for rook_pos in &rooks {
            let rook_developed = rook_pos.row != start_row;
            
            for bishop_pos in &bishops {
                let bishop_developed = bishop_pos.row != start_row;
                
                if rook_developed && bishop_developed {
                    // Both major pieces developed - coordination bonus
                    // Stronger if they're in good positions (not too close to edge)
                    let rook_central = rook_pos.col >= 2 && rook_pos.col <= 6;
                    let bishop_central = bishop_pos.col >= 2 && bishop_pos.col <= 6;
                    
                    if rook_central || bishop_central {
                        mg_score += 18; // Rook-bishop coordination bonus
                    }
                }
            }
        }

        // 5. Piece synergy bonuses (developed pieces supporting each other)
        // Check for developed rook supporting developed silver/gold
        for rook_pos in &rooks {
            let rook_developed = rook_pos.row != start_row;
            
            if rook_developed {
                // Check if rook is on same file as developed silver/gold
                for silver_pos in self.find_pieces(board, player, PieceType::Silver) {
                    if silver_pos.row != start_row && silver_pos.col == rook_pos.col {
                        mg_score += 12; // Rook supporting developed silver
                    }
                }
                
                for gold_pos in self.find_pieces(board, player, PieceType::Gold) {
                    if gold_pos.row != start_row && gold_pos.col == rook_pos.col {
                        mg_score += 10; // Rook supporting developed gold
                    }
                }
            }
        }

        // Endgame component is less important (1/4 of middlegame)
        let eg_score = mg_score / 4;
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if two positions are on the same diagonal
    fn same_diagonal(&self, pos1: Position, pos2: Position) -> bool {
        let rank_diff = (pos1.row as i32 - pos2.row as i32).abs();
        let file_diff = (pos1.col as i32 - pos2.col as i32).abs();
        
        // Same diagonal if rank_diff == file_diff (main diagonal) or
        // if rank_diff + file_diff forms anti-diagonal pattern
        rank_diff == file_diff || (pos1.row + pos1.col == pos2.row + pos2.col)
    }

    /// Calculate square distance (Manhattan distance)
    fn square_distance(&self, pos1: Position, pos2: Position) -> u8 {
        let rank_diff = (pos1.row as i32 - pos2.row as i32).abs() as u8;
        let file_diff = (pos1.col as i32 - pos2.col as i32).abs() as u8;
        rank_diff + file_diff
    }

    // =======================================================================
    // OPENING BOOK INTEGRATION (Task 19.0 - Task 3.0)
    // =======================================================================

    /// Evaluate book move quality using opening principles
    ///
    /// This method evaluates how well a move aligns with opening principles by:
    /// 1. Making the move on a temporary board
    /// 2. Evaluating the resulting position using opening principles
    /// 3. Returning a quality score (higher = better alignment with principles)
    ///
    /// # Arguments
    ///
    /// * `board` - Current board state
    /// * `player` - Player making the move
    /// * `move_` - The move to evaluate
    /// * `captured_pieces` - Current captured pieces state
    /// * `move_count` - Number of moves played so far
    ///
    /// # Returns
    ///
    /// Quality score as i32 (higher = better alignment with opening principles)
    pub fn evaluate_book_move_quality(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        captured_pieces: &CapturedPieces,
        move_count: u32,
    ) -> i32 {
        self.stats.book_moves_evaluated += 1;

        // Clone board and captured pieces to make move without modifying original
        let mut temp_board = board.clone();
        let mut temp_captured = captured_pieces.clone();

        // Make the move
        if let Some(captured_piece) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured_piece.piece_type, player);
        }

        // Evaluate the resulting position using opening principles
        // Note: After making the move, it's the opponent's turn, so we evaluate for the opponent
        // But we want to evaluate how good the position is for the player who made the move
        // So we evaluate for the original player (the one who made the move)
        let score = self.evaluate_opening(&temp_board, player, move_count + 1, Some(&temp_captured), None);
        
        // Convert TaperedScore to i32 (use interpolated score at opening phase)
        let quality_score = score.interpolate(256); // Phase 256 = opening phase
        
        // Telemetry: Log book move quality scores (Task 19.0 - Task 5.0)
        #[cfg(feature = "verbose-debug")]
        {
            use crate::debug_utils::debug_log_fast;
            debug_log_fast!(&format!(
                "[OPENING_PRINCIPLES] Book move quality score: {}cp (move_count={})",
                quality_score,
                move_count + 1
            ));
        }
        
        // Track quality scores for statistics
        self.stats.book_move_quality_scores += quality_score as i64;

        quality_score
    }

    /// Validate book move against opening principles
    ///
    /// Checks if a move violates opening principles (e.g., early king move, undeveloped major piece)
    /// Returns true if move is valid, false if it violates principles
    ///
    /// # Arguments
    ///
    /// * `board` - Current board state
    /// * `player` - Player making the move
    /// * `move_` - The move to validate
    /// * `move_count` - Number of moves played so far
    ///
    /// # Returns
    ///
    /// True if move is valid, false if it violates opening principles
    pub fn validate_book_move(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        move_count: u32,
    ) -> bool {
        self.stats.book_moves_validated += 1;

        // Early king move penalty (if move_count <= 10)
        if move_count <= 10 {
            if let Some(from) = move_.from {
                if let Some(piece) = board.get_piece(from) {
                    if piece.piece_type == PieceType::King && piece.player == player {
                        let start_row = if player == Player::Black { 8 } else { 0 };
                        if from.row == start_row {
                            // King is on starting row, check if moving to non-castle position
                            if !self.is_castle_position(move_.to, player) {
                                // Early king move violation
                                #[cfg(debug_assertions)]
                                crate::debug_utils::debug_log(&format!(
                                    "[OPENING_PRINCIPLES] Book move warning: Early king move at move {} violates opening principles",
                                    move_count
                                ));
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // Check for undeveloped major pieces (rook/bishop still on starting row)
        if move_count <= 15 {
            if let Some(from) = move_.from {
                if let Some(piece) = board.get_piece(from) {
                    if piece.player == player {
                        let start_row = if player == Player::Black { 8 } else { 0 };
                        if from.row == start_row {
                            match piece.piece_type {
                                PieceType::Rook | PieceType::Bishop => {
                                    // Major piece still on starting row - not ideal but not a violation
                                    // Just log a warning
                                    #[cfg(debug_assertions)]
                                    crate::debug_utils::debug_log(&format!(
                                        "[OPENING_PRINCIPLES] Book move note: {} still on starting row at move {}",
                                        format!("{:?}", piece.piece_type),
                                        move_count
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        true
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

    /// Find all pieces of a specific type (optimized with bitboard operations)
    fn find_pieces(
        &self,
        board: &BitboardBoard,
        player: Player,
        piece_type: PieceType,
    ) -> Vec<Position> {
        // Use bitboard operations instead of iterating over all 81 squares
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let piece_idx = piece_type.to_u8() as usize;
        
        // Get the bitboard for this piece type and player
        let piece_bitboard = pieces[player_idx][piece_idx];
        
        // Convert bitboard to positions efficiently
        self.bitboard_to_positions(piece_bitboard)
    }

    /// Convert bitboard to list of positions efficiently (Task 19.0 - Task 4.0)
    fn bitboard_to_positions(&self, bitboard: crate::types::Bitboard) -> Vec<Position> {
        use crate::types::get_lsb;
        
        let mut positions = Vec::new();
        let mut remaining = bitboard;
        
        while remaining != 0 {
            if let Some(pos) = get_lsb(remaining) {
                positions.push(pos);
                remaining &= remaining - 1; // Clear the least significant bit
            } else {
                break;
            }
        }
        
        positions
    }

    /// Get statistics
    pub fn stats(&self) -> &OpeningPrincipleStats {
        &self.stats
    }

    /// Get mutable statistics (for external updates)
    pub fn stats_mut(&mut self) -> &mut OpeningPrincipleStats {
        &mut self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = OpeningPrincipleStats::default();
    }

    /// Get per-component statistics breakdown (Task 19.0 - Task 4.0)
    pub fn get_component_statistics(&self) -> ComponentStatistics {
        let stats = &self.stats;
        
        ComponentStatistics {
            development: ComponentStats {
                total_score: stats.development_score,
                evaluation_count: stats.development_evaluations,
                average_score: if stats.development_evaluations > 0 {
                    stats.development_score as f64 / stats.development_evaluations as f64
                } else {
                    0.0
                },
            },
            center_control: ComponentStats {
                total_score: stats.center_control_score,
                evaluation_count: stats.center_control_evaluations,
                average_score: if stats.center_control_evaluations > 0 {
                    stats.center_control_score as f64 / stats.center_control_evaluations as f64
                } else {
                    0.0
                },
            },
            castle_formation: ComponentStats {
                total_score: stats.castle_formation_score,
                evaluation_count: stats.castle_formation_evaluations,
                average_score: if stats.castle_formation_evaluations > 0 {
                    stats.castle_formation_score as f64 / stats.castle_formation_evaluations as f64
                } else {
                    0.0
                },
            },
            tempo: ComponentStats {
                total_score: stats.tempo_score,
                evaluation_count: stats.tempo_evaluations,
                average_score: if stats.tempo_evaluations > 0 {
                    stats.tempo_score as f64 / stats.tempo_evaluations as f64
                } else {
                    0.0
                },
            },
            penalties: ComponentStats {
                total_score: stats.penalties_score,
                evaluation_count: stats.penalties_evaluations,
                average_score: if stats.penalties_evaluations > 0 {
                    stats.penalties_score as f64 / stats.penalties_evaluations as f64
                } else {
                    0.0
                },
            },
            piece_coordination: ComponentStats {
                total_score: stats.piece_coordination_score,
                evaluation_count: stats.piece_coordination_evaluations,
                average_score: if stats.piece_coordination_evaluations > 0 {
                    stats.piece_coordination_score as f64 / stats.piece_coordination_evaluations as f64
                } else {
                    0.0
                },
            },
        }
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
    /// Enable piece coordination evaluation (rook-lance batteries, bishop-lance combinations, etc.)
    pub enable_piece_coordination: bool,
    /// Enable attack-based center control evaluation (evaluates center control from piece attacks, not just occupied squares)
    pub enable_attack_based_center_control: bool,
    /// Enable drop pressure evaluation (evaluates center control via potential piece drops)
    pub enable_drop_pressure_evaluation: bool,
}

impl Default for OpeningPrincipleConfig {
    fn default() -> Self {
        Self {
            enable_development: true,
            enable_center_control: true,
            enable_castle_formation: true,
            enable_tempo: true,
            enable_opening_penalties: true,
            enable_piece_coordination: true,
            enable_attack_based_center_control: true,
            enable_drop_pressure_evaluation: true,
        }
    }
}

/// Statistics for opening principle evaluation
#[derive(Debug, Clone, Default)]
pub struct OpeningPrincipleStats {
    /// Number of evaluations performed
    pub evaluations: u64,
    /// Number of book moves evaluated using opening principles
    pub book_moves_evaluated: u64,
    /// Number of book moves prioritized by opening principles
    pub book_moves_prioritized: u64,
    /// Number of book moves validated (checked for violations)
    pub book_moves_validated: u64,
    /// Sum of book move quality scores (for average calculation)
    pub book_move_quality_scores: i64,
    /// Per-component statistics (Task 19.0 - Task 4.0)
    /// Development component score sum
    pub development_score: i64,
    /// Center control component score sum
    pub center_control_score: i64,
    /// Castle formation component score sum
    pub castle_formation_score: i64,
    /// Tempo component score sum
    pub tempo_score: i64,
    /// Penalties component score sum
    pub penalties_score: i64,
    /// Piece coordination component score sum
    pub piece_coordination_score: i64,
    /// Development component evaluation count
    pub development_evaluations: u64,
    /// Center control component evaluation count
    pub center_control_evaluations: u64,
    /// Castle formation component evaluation count
    pub castle_formation_evaluations: u64,
    /// Tempo component evaluation count
    pub tempo_evaluations: u64,
    /// Penalties component evaluation count
    pub penalties_evaluations: u64,
    /// Piece coordination component evaluation count
    pub piece_coordination_evaluations: u64,
    /// Telemetry: Track if opening principles influenced move selection (Task 19.0 - Task 5.0)
    pub opening_principles_influenced_move: u64,
    /// Telemetry: Moves influenced by development component
    pub moves_influenced_by_development: u64,
    /// Telemetry: Moves influenced by center control component
    pub moves_influenced_by_center_control: u64,
    /// Telemetry: Moves influenced by castle formation component
    pub moves_influenced_by_castle_formation: u64,
    /// Telemetry: Moves influenced by tempo component
    pub moves_influenced_by_tempo: u64,
    /// Telemetry: Moves influenced by penalties component
    pub moves_influenced_by_penalties: u64,
    /// Telemetry: Moves influenced by piece coordination component
    pub moves_influenced_by_piece_coordination: u64,
}

/// Per-component statistics breakdown (Task 19.0 - Task 4.0)
#[derive(Debug, Clone, Default)]
pub struct ComponentStatistics {
    /// Development component statistics
    pub development: ComponentStats,
    /// Center control component statistics
    pub center_control: ComponentStats,
    /// Castle formation component statistics
    pub castle_formation: ComponentStats,
    /// Tempo component statistics
    pub tempo: ComponentStats,
    /// Penalties component statistics
    pub penalties: ComponentStats,
    /// Piece coordination component statistics
    pub piece_coordination: ComponentStats,
}

/// Statistics for a single component
#[derive(Debug, Clone, Default)]
pub struct ComponentStats {
    /// Total score sum for this component
    pub total_score: i64,
    /// Number of evaluations performed
    pub evaluation_count: u64,
    /// Average score (total_score / evaluation_count)
    pub average_score: f64,
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

        let score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

        // Should have some positive opening evaluation
        assert!(score.mg > 0);
    }

    #[test]
    fn test_statistics() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        assert_eq!(evaluator.stats().evaluations, 0);

        evaluator.evaluate_opening(&board, Player::Black, 5, None, None);
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

        let score1 = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);
        let score2 = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_move_count_effects() {
        let mut evaluator = OpeningPrincipleEvaluator::new();
        let board = BitboardBoard::new();

        // Early game (move 5)
        let early_score = evaluator.evaluate_opening(&board, Player::Black, 5, None, None);

        // Later (move 20)
        let late_score = evaluator.evaluate_opening(&board, Player::Black, 20, None, None);

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
