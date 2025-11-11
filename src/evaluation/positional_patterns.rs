//! Positional Pattern Recognition Module
//!
//! This module implements detection of positional patterns in Shogi including:
//! - Center control evaluation
//! - Outpost detection (strong pieces on key squares)
//! - Weak square identification
//! - Piece activity bonuses
//! - Space advantage evaluation
//! - Tempo evaluation
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::positional_patterns::PositionalPatternAnalyzer;
//!
//! let analyzer = PositionalPatternAnalyzer::new();
//! let positional_score =
//!     analyzer.evaluate_position(&board, Player::Black, &CapturedPieces::new());
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Positional pattern analyzer
pub struct PositionalPatternAnalyzer {
    config: PositionalConfig,
    stats: PositionalStats,
}

struct ControlCache<'a> {
    board: &'a BitboardBoard,
    cache: [[Option<bool>; 81]; 2],
}

impl<'a> ControlCache<'a> {
    fn new(board: &'a BitboardBoard) -> Self {
        Self {
            board,
            cache: [[None; 81]; 2],
        }
    }

    fn player_index(player: Player) -> usize {
        if player == Player::Black {
            0
        } else {
            1
        }
    }

    fn square_index(pos: Position) -> usize {
        pos.row as usize * 9 + pos.col as usize
    }

    fn controlled_by(&mut self, player: Player, pos: Position) -> bool {
        let player_idx = Self::player_index(player);
        let square_idx = Self::square_index(pos);

        if let Some(value) = self.cache[player_idx][square_idx] {
            return value;
        }

        let value = self.board.is_square_attacked_by(pos, player);
        self.cache[player_idx][square_idx] = Some(value);
        value
    }

    fn count_all_controlled(&mut self, player: Player) -> i32 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if self.controlled_by(player, Position::new(row, col)) {
                    count += 1;
                }
            }
        }
        count
    }

    fn count_subset_controlled(&mut self, player: Player, squares: &[(u8, u8)]) -> i32 {
        let mut count = 0;
        for &(row, col) in squares {
            if self.controlled_by(player, Position::new(row, col)) {
                count += 1;
            }
        }
        count
    }
}

impl PositionalPatternAnalyzer {
    /// Create a new positional pattern analyzer
    pub fn new() -> Self {
        Self {
            config: PositionalConfig::default(),
            stats: PositionalStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PositionalConfig) -> Self {
        Self {
            config,
            stats: PositionalStats::default(),
        }
    }

    /// Evaluate all positional patterns for a player
    pub fn evaluate_position(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;
        let mut control_cache = ControlCache::new(board);

        // Center control
        if self.config.enable_center_control {
            let center =
                self.evaluate_center_control(board, player, &mut control_cache, captured_pieces);
            mg_score += center.mg;
            eg_score += center.eg;
        }

        // Outposts
        if self.config.enable_outposts {
            let outposts = self.evaluate_outposts(board, player, captured_pieces);
            mg_score += outposts.mg;
            eg_score += outposts.eg;
        }

        // Weak squares
        if self.config.enable_weak_squares {
            let weak =
                self.evaluate_weak_squares(board, player, &mut control_cache, captured_pieces);
            mg_score += weak.mg;
            eg_score += weak.eg;
        }

        // Piece activity
        if self.config.enable_piece_activity {
            let activity = self.evaluate_piece_activity(board, player);
            mg_score += activity.mg;
            eg_score += activity.eg;
        }

        // Space advantage
        if self.config.enable_space_advantage {
            let space = self.evaluate_space_advantage(board, player, &mut control_cache);
            mg_score += space.mg;
            eg_score += space.eg;
        }

        // Tempo
        if self.config.enable_tempo {
            let tempo = self.evaluate_tempo(board, player);
            mg_score += tempo.mg;
            eg_score += tempo.eg;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    // ===================================================================
    // CENTER CONTROL EVALUATION
    // ===================================================================

    /// Evaluate center control (enhanced version of basic center control)
    fn evaluate_center_control(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
        _captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.center_control_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Define center (3x3 core)
        let center_squares = [
            (3, 3),
            (3, 4),
            (3, 5),
            (4, 3),
            (4, 4),
            (4, 5),
            (5, 3),
            (5, 4),
            (5, 5),
        ];

        // Define extended center (5x5)
        let extended_center = [
            (2, 2),
            (2, 3),
            (2, 4),
            (2, 5),
            (2, 6),
            (3, 2),
            (3, 6),
            (4, 2),
            (4, 6),
            (5, 2),
            (5, 6),
            (6, 2),
            (6, 3),
            (6, 4),
            (6, 5),
            (6, 6),
        ];

        // Evaluate core center occupation
        for (row, col) in center_squares {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                let value = self.get_center_piece_value(piece.piece_type, true);
                if piece.player == player {
                    mg_score += value.0;
                    eg_score += value.1;
                } else {
                    mg_score -= value.0;
                    eg_score -= value.1;
                }
            }
        }

        // Evaluate extended center (half value)
        for (row, col) in extended_center {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                let value = self.get_center_piece_value(piece.piece_type, false);
                if piece.player == player {
                    mg_score += value.0 / 2;
                    eg_score += value.1 / 2;
                } else {
                    mg_score -= value.0 / 2;
                    eg_score -= value.1 / 2;
                }
            }
        }

        // Bonus for controlling center squares (blocker-aware)
        let control_advantage =
            self.center_control_advantage(control_cache, player, &center_squares);
        mg_score += control_advantage * self.config.pawn_center_bonus;
        eg_score += control_advantage * self.config.pawn_center_bonus / 2;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get value of piece in center
    fn get_center_piece_value(&self, piece_type: PieceType, is_core: bool) -> (i32, i32) {
        let multiplier = if is_core { 1 } else { 1 };

        match piece_type {
            PieceType::Knight => (30 * multiplier, 15 * multiplier),
            PieceType::Silver => (25 * multiplier, 20 * multiplier),
            PieceType::Gold => (20 * multiplier, 18 * multiplier),
            PieceType::Bishop | PieceType::PromotedBishop => (35 * multiplier, 25 * multiplier),
            PieceType::Rook | PieceType::PromotedRook => (32 * multiplier, 28 * multiplier),
            PieceType::Pawn => (15 * multiplier, 10 * multiplier),
            _ => (10 * multiplier, 8 * multiplier),
        }
    }

    fn center_control_advantage(
        &self,
        control_cache: &mut ControlCache<'_>,
        player: Player,
        center: &[(u8, u8)],
    ) -> i32 {
        let player_control = control_cache.count_subset_controlled(player, center);
        let opponent_control = control_cache.count_subset_controlled(player.opposite(), center);
        player_control - opponent_control
    }

    // ===================================================================
    // OUTPOST DETECTION
    // ===================================================================

    /// Evaluate outposts (strong pieces on key squares that cannot be easily attacked)
    fn evaluate_outposts(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.outpost_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check each piece for outpost potential
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        if self.is_outpost(board, pos, piece.piece_type, player, captured_pieces) {
                            let value = self.get_outpost_value(piece.piece_type, pos, player);
                            mg_score += value.0;
                            eg_score += value.1;
                            self.stats.outposts_found += 1;
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a square is an outpost for a piece
    fn is_outpost(
        &self,
        board: &BitboardBoard,
        pos: Position,
        piece_type: PieceType,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Outposts are typically:
        // 1. In or near enemy territory
        // 2. Protected by own pawns
        // 3. Cannot be easily attacked by enemy pawns

        // Check if in advanced position
        let is_advanced = if player == Player::Black {
            pos.row <= 5 // Advanced for Black
        } else {
            pos.row >= 3 // Advanced for White
        };

        if !is_advanced {
            return false;
        }

        // Check if protected by own pawn
        let has_pawn_support = self.has_pawn_support(board, pos, player, captured_pieces);

        // Check if enemy pawns can easily attack
        let enemy_pawn_threat =
            self.is_under_enemy_pawn_threat(board, pos, player, captured_pieces);

        let drop_threat = self.has_drop_threat(board, pos, player, captured_pieces);

        // Knights and Silvers make best outposts
        let is_good_piece = matches!(
            piece_type,
            PieceType::Knight | PieceType::Silver | PieceType::Gold
        );

        has_pawn_support && !enemy_pawn_threat && !drop_threat && is_good_piece
    }

    /// Check if position has pawn support
    fn has_pawn_support(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        let behind_row = match player {
            Player::Black => pos.row as i8 + 1,
            Player::White => pos.row as i8 - 1,
        };

        if behind_row < 0 || behind_row >= 9 {
            return false;
        }

        let support_pos = Position::new(behind_row as u8, pos.col);
        if let Some(piece) = board.get_piece(support_pos) {
            if piece.player == player && piece.piece_type == PieceType::Pawn {
                return true;
            }
        } else if self.config.enable_hand_context
            && captured_pieces.count(PieceType::Pawn, player) > 0
            && !self.has_unpromoted_pawn_on_file(board, player, pos.col)
            && !self.is_illegal_drop_rank(PieceType::Pawn, player, support_pos.row)
        {
            return true;
        }

        false
    }

    fn has_drop_threat(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        if !self.config.enable_drop_threats || !self.config.enable_hand_context {
            return false;
        }

        let opponent = player.opposite();
        self.pawn_drop_threat(board, pos, opponent, captured_pieces)
            || self.lance_drop_threat(board, pos, opponent, captured_pieces)
            || self.knight_drop_threat(board, pos, opponent, captured_pieces)
    }

    fn pawn_drop_threat(
        &self,
        board: &BitboardBoard,
        pos: Position,
        opponent: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        if captured_pieces.count(PieceType::Pawn, opponent) == 0 {
            return false;
        }

        let forward = self.player_forward(opponent);
        let drop_row = pos.row as i8 - forward;
        if drop_row < 0 || drop_row >= 9 {
            return false;
        }

        let drop_pos = Position::new(drop_row as u8, pos.col);
        if board.get_piece(drop_pos).is_some() {
            return false;
        }

        if self.is_illegal_drop_rank(PieceType::Pawn, opponent, drop_pos.row)
            || self.has_unpromoted_pawn_on_file(board, opponent, pos.col)
        {
            return false;
        }

        true
    }

    fn lance_drop_threat(
        &self,
        board: &BitboardBoard,
        pos: Position,
        opponent: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        if captured_pieces.count(PieceType::Lance, opponent) == 0 {
            return false;
        }

        let forward = self.player_forward(opponent);
        let mut drop_row = pos.row as i8 - forward;

        while drop_row >= 0 && drop_row < 9 {
            let drop_pos = Position::new(drop_row as u8, pos.col);

            if self.is_illegal_drop_rank(PieceType::Lance, opponent, drop_pos.row) {
                drop_row -= forward;
                continue;
            }

            if let Some(blocker) = board.get_piece(drop_pos) {
                if drop_pos == pos {
                    return false;
                }

                if blocker.player == opponent {
                    return false;
                } else {
                    return false;
                }
            }

            if self.path_clear_vertical(board, drop_pos, pos, forward) {
                return true;
            }

            drop_row -= forward;
        }

        false
    }

    fn knight_drop_threat(
        &self,
        board: &BitboardBoard,
        pos: Position,
        opponent: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        if captured_pieces.count(PieceType::Knight, opponent) == 0 {
            return false;
        }

        let forward = self.player_forward(opponent);
        let drop_row = pos.row as i8 - 2 * forward;
        if drop_row < 0 || drop_row >= 9 {
            return false;
        }

        let candidate_cols = [pos.col as i8 - 1, pos.col as i8 + 1];

        for drop_col in candidate_cols {
            if drop_col < 0 || drop_col >= 9 {
                continue;
            }

            let drop_pos = Position::new(drop_row as u8, drop_col as u8);
            if board.get_piece(drop_pos).is_some() {
                continue;
            }

            if self.is_illegal_drop_rank(PieceType::Knight, opponent, drop_pos.row) {
                continue;
            }

            return true;
        }

        false
    }

    fn path_clear_vertical(
        &self,
        board: &BitboardBoard,
        from: Position,
        to: Position,
        forward: i8,
    ) -> bool {
        let mut current_row = from.row as i8 + forward;

        while current_row >= 0 && current_row < 9 {
            if current_row == to.row as i8 {
                return true;
            }

            if board
                .get_piece(Position::new(current_row as u8, from.col))
                .is_some()
            {
                return false;
            }

            current_row += forward;
        }

        false
    }

    fn has_unpromoted_pawn_on_file(&self, board: &BitboardBoard, player: Player, file: u8) -> bool {
        for row in 0..9 {
            let pos = Position::new(row, file);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player && piece.piece_type == PieceType::Pawn {
                    return true;
                }
            }
        }
        false
    }

    fn is_illegal_drop_rank(&self, piece_type: PieceType, player: Player, row: u8) -> bool {
        match piece_type {
            PieceType::Pawn | PieceType::Lance => match player {
                Player::Black => row == 0,
                Player::White => row == 8,
            },
            PieceType::Knight => match player {
                Player::Black => row <= 1,
                Player::White => row >= 7,
            },
            _ => false,
        }
    }

    fn player_forward(&self, player: Player) -> i8 {
        if player == Player::Black {
            -1
        } else {
            1
        }
    }

    /// Check if under enemy pawn threat
    fn is_under_enemy_pawn_threat(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        let opponent = player.opposite();
        let forward = self.player_forward(opponent);
        let source_row = pos.row as i8 - forward;

        if source_row >= 0 && source_row < 9 {
            let source_pos = Position::new(source_row as u8, pos.col);
            if let Some(piece) = board.get_piece(source_pos) {
                if piece.player == opponent && piece.piece_type == PieceType::Pawn {
                    return true;
                }
            } else if self.config.enable_hand_context
                && self.config.enable_drop_threats
                && captured_pieces.count(PieceType::Pawn, opponent) > 0
                && !self.has_unpromoted_pawn_on_file(board, opponent, pos.col)
                && !self.is_illegal_drop_rank(PieceType::Pawn, opponent, source_pos.row)
                && board.get_piece(source_pos).is_none()
            {
                return true;
            }
        }

        false
    }

    /// Get value of an outpost
    fn get_outpost_value(
        &self,
        piece_type: PieceType,
        pos: Position,
        player: Player,
    ) -> (i32, i32) {
        let base_value = match piece_type {
            PieceType::Knight => (60, 40),
            PieceType::Silver => (50, 45),
            PieceType::Gold => (45, 40),
            _ => (30, 25),
        };

        // Bonus for deeper outposts
        let depth = if player == Player::Black {
            8 - pos.row
        } else {
            pos.row
        };

        let depth_bonus = (depth as i32 * 5, depth as i32 * 3);

        (base_value.0 + depth_bonus.0, base_value.1 + depth_bonus.1)
    }

    // ===================================================================
    // WEAK SQUARE IDENTIFICATION
    // ===================================================================

    /// Evaluate weak squares in player's position
    fn evaluate_weak_squares(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.weak_square_checks += 1;

        let mut penalty = 0;
        let opponent = player.opposite();

        // Check key squares in own territory
        let key_squares = self.get_key_squares(player);

        for pos in key_squares {
            if self.is_weak_square(board, pos, player, captured_pieces) {
                // Check if enemy has piece controlling this square
                if control_cache.controlled_by(opponent, pos) {
                    penalty += self.config.weak_square_penalty;
                    self.stats.weak_squares_found += 1;
                }
            }
        }

        TaperedScore::new_tapered(-penalty, -penalty / 2)
    }

    /// Get key squares to monitor for weaknesses
    fn get_key_squares(&self, player: Player) -> Vec<Position> {
        let mut squares = Vec::new();

        // Squares around king area
        let king_area_rows = if player == Player::Black {
            6..=8
        } else {
            0..=2
        };

        for row in king_area_rows {
            for col in 3..=5 {
                // Central files
                squares.push(Position::new(row, col));
            }
        }

        squares
    }

    /// Check if a square is weak (cannot be defended by pawns)
    fn is_weak_square(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // A square is weak if no friendly pawns can defend it
        !self.can_be_defended_by_pawn(board, pos, player, captured_pieces)
    }

    /// Check if square can be defended by pawn
    fn can_be_defended_by_pawn(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        let pawn_files = [-1, 0, 1];

        for dc in pawn_files {
            let file = pos.col as i8 + dc;
            if file < 0 || file >= 9 {
                continue;
            }

            // Check if there's a pawn on this file that could defend
            for row in 0..9 {
                let check_pos = Position::new(row, file as u8);
                if let Some(piece) = board.get_piece(check_pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        return true;
                    }
                }
            }

            if self.config.enable_hand_context
                && captured_pieces.count(PieceType::Pawn, player) > 0
                && !self.has_unpromoted_pawn_on_file(board, player, file as u8)
            {
                let defensive_row = match player {
                    Player::Black => pos.row.saturating_add(1),
                    Player::White => pos.row.saturating_sub(1),
                };

                if defensive_row < 9
                    && !self.is_illegal_drop_rank(PieceType::Pawn, player, defensive_row as u8)
                {
                    let defensive_pos = Position::new(defensive_row as u8, file as u8);
                    if board.get_piece(defensive_pos).is_none() {
                        return true;
                    }
                }
            }
        }

        false
    }

    // ===================================================================
    // PIECE ACTIVITY EVALUATION
    // ===================================================================

    /// Evaluate piece activity (how active/well-placed pieces are)
    fn evaluate_piece_activity(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.activity_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let activity = self.get_piece_activity_score(pos, piece.piece_type, player);
                        mg_score += activity.0;
                        eg_score += activity.1;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get activity score for a piece
    fn get_piece_activity_score(
        &self,
        pos: Position,
        piece_type: PieceType,
        player: Player,
    ) -> (i32, i32) {
        // Pieces are more active when advanced
        let advancement = if player == Player::Black {
            8 - pos.row
        } else {
            pos.row
        };

        let activity_bonus = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                (advancement as i32 * 3, advancement as i32 * 4)
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                (advancement as i32 * 2, advancement as i32 * 3)
            }
            PieceType::Silver => (advancement as i32 * 2, advancement as i32 * 2),
            PieceType::Gold => (advancement as i32 * 1, advancement as i32 * 2),
            _ => (0, 0),
        };

        activity_bonus
    }

    // ===================================================================
    // SPACE ADVANTAGE EVALUATION
    // ===================================================================

    /// Evaluate space advantage (territory control)
    fn evaluate_space_advantage(
        &mut self,
        _board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
    ) -> TaperedScore {
        self.stats.space_checks += 1;

        let player_squares = control_cache.count_all_controlled(player);
        let opponent_squares = control_cache.count_all_controlled(player.opposite());

        let advantage = player_squares - opponent_squares;
        let mg_score = advantage * self.config.space_advantage_bonus;
        let eg_score = advantage * self.config.space_advantage_bonus / 3; // Less important in endgame

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    // ===================================================================
    // TEMPO EVALUATION
    // ===================================================================

    /// Evaluate tempo (having the initiative/extra moves)
    fn evaluate_tempo(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.tempo_checks += 1;

        // Count developed pieces (pieces that have moved from starting position)
        let developed = self.count_developed_pieces(board, player);
        let opp_developed = self.count_developed_pieces(board, player.opposite());

        let tempo_advantage = developed.saturating_sub(opp_developed);
        let mg_score = tempo_advantage as i32 * self.config.tempo_bonus;
        let eg_score = 0; // Tempo not relevant in endgame

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Count developed pieces (heuristic based on position)
    fn count_developed_pieces(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut count = 0;
        let start_row = if player == Player::Black { 8 } else { 0 };

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        // Consider piece developed if not on starting row
                        match piece.piece_type {
                            PieceType::Rook
                            | PieceType::Bishop
                            | PieceType::Gold
                            | PieceType::Silver => {
                                if pos.row != start_row {
                                    count += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        count
    }

    /// Get statistics
    pub fn stats(&self) -> &PositionalStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PositionalStats::default();
    }
}

impl Default for PositionalPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for positional pattern analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionalConfig {
    pub enable_center_control: bool,
    pub enable_outposts: bool,
    pub enable_weak_squares: bool,
    pub enable_piece_activity: bool,
    pub enable_space_advantage: bool,
    pub enable_tempo: bool,
    pub enable_hand_context: bool,
    pub enable_drop_threats: bool,

    // Bonus/penalty values
    pub pawn_center_bonus: i32,
    pub weak_square_penalty: i32,
    pub space_advantage_bonus: i32,
    pub tempo_bonus: i32,
}

impl Default for PositionalConfig {
    fn default() -> Self {
        Self {
            enable_center_control: true,
            enable_outposts: true,
            enable_weak_squares: true,
            enable_piece_activity: true,
            enable_space_advantage: true,
            enable_tempo: true,
            enable_hand_context: true,
            enable_drop_threats: true,

            pawn_center_bonus: 25,
            weak_square_penalty: 40,
            space_advantage_bonus: 2,
            tempo_bonus: 15,
        }
    }
}

/// Statistics for positional pattern analysis
#[derive(Debug, Clone, Default)]
pub struct PositionalStats {
    pub evaluations: u64,
    pub center_control_checks: u64,
    pub outpost_checks: u64,
    pub weak_square_checks: u64,
    pub activity_checks: u64,
    pub space_checks: u64,
    pub tempo_checks: u64,

    pub outposts_found: u64,
    pub weak_squares_found: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positional_analyzer_creation() {
        let analyzer = PositionalPatternAnalyzer::new();
        assert!(analyzer.config.enable_center_control);
        assert!(analyzer.config.enable_outposts);
    }

    #[test]
    fn test_center_control_evaluation() {
        let mut analyzer = PositionalPatternAnalyzer::new();
        let board = BitboardBoard::new();
        let mut control_cache = ControlCache::new(&board);
        let captured = CapturedPieces::new();

        let _score =
            analyzer.evaluate_center_control(&board, Player::Black, &mut control_cache, &captured);
        assert_eq!(analyzer.stats().center_control_checks, 1);
    }

    #[test]
    fn test_outpost_detection() {
        let mut analyzer = PositionalPatternAnalyzer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let score = analyzer.evaluate_outposts(&board, Player::Black, &captured);
        assert!(score.mg >= 0);
    }

    #[test]
    fn test_evaluate_position() {
        let mut analyzer = PositionalPatternAnalyzer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let _score = analyzer.evaluate_position(&board, Player::Black, &captured);
        assert_eq!(analyzer.stats().evaluations, 1);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut analyzer = PositionalPatternAnalyzer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        analyzer.evaluate_position(&board, Player::Black, &captured);

        let stats = analyzer.stats();
        assert!(stats.center_control_checks >= 1);
        assert!(stats.outpost_checks >= 1);
    }

    #[test]
    fn test_control_cache_matches_board_queries() {
        let mut board = BitboardBoard::empty();
        let rook = Piece::new(PieceType::Rook, Player::Black);
        let pawn = Piece::new(PieceType::Pawn, Player::Black);
        let rook_pos = Position::new(4, 4);
        let blocking_pawn_pos = Position::new(5, 4);
        let lateral_target = Position::new(4, 6);
        let blocked_target = Position::new(6, 4);

        board.place_piece(rook, rook_pos);
        board.place_piece(pawn, blocking_pawn_pos);

        let board_lateral = board.is_square_attacked_by(lateral_target, Player::Black);
        let board_blocked = board.is_square_attacked_by(blocked_target, Player::Black);

        let mut control_cache = ControlCache::new(&board);
        assert_eq!(
            control_cache.controlled_by(Player::Black, lateral_target),
            board_lateral
        );
        assert_eq!(
            control_cache.controlled_by(Player::Black, blocked_target),
            board_blocked
        );
    }

    #[test]
    fn test_outpost_rejected_by_pawn_drop() {
        let mut board = BitboardBoard::empty();
        let mut analyzer = PositionalPatternAnalyzer::new();
        let outpost_pos = Position::new(4, 4);

        board.place_piece(Piece::new(PieceType::Silver, Player::Black), outpost_pos);
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(5, 4),
        );

        let captured_none = CapturedPieces::new();
        let score_without_threat =
            analyzer.evaluate_outposts(&board, Player::Black, &captured_none);
        assert!(score_without_threat.mg > 0);

        let mut captured_with_pawn = CapturedPieces::new();
        captured_with_pawn.add_piece(PieceType::Pawn, Player::White);

        let score_with_threat =
            analyzer.evaluate_outposts(&board, Player::Black, &captured_with_pawn);
        assert_eq!(score_with_threat.mg, 0);
    }

    #[test]
    fn test_weak_square_relieved_by_pawn_drop() {
        let mut board = BitboardBoard::empty();
        let mut analyzer = PositionalPatternAnalyzer::new();
        board.place_piece(
            Piece::new(PieceType::Rook, Player::White),
            Position::new(0, 4),
        );

        let mut cache_no_drop = ControlCache::new(&board);
        let captured_none = CapturedPieces::new();
        let penalty = analyzer.evaluate_weak_squares(
            &board,
            Player::Black,
            &mut cache_no_drop,
            &captured_none,
        );
        assert!(penalty.mg < 0);

        let mut cache_with_drop = ControlCache::new(&board);
        let mut captured_with_pawn = CapturedPieces::new();
        captured_with_pawn.add_piece(PieceType::Pawn, Player::Black);

        let mitigated = analyzer.evaluate_weak_squares(
            &board,
            Player::Black,
            &mut cache_with_drop,
            &captured_with_pawn,
        );
        assert_eq!(mitigated.mg, 0);
    }
}
