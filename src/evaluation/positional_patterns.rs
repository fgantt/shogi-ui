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
use crate::types::board::CapturedPieces;
use crate::types::core::{Piece, PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

const CORE_CENTER: [(u8, u8); 9] = [
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

const EXTENDED_CENTER: [(u8, u8); 16] = [
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

const SPACE_ROW_WEIGHTS: [i32; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
const WEAK_SQUARE_ROW_WEIGHTS: [i32; 9] = [7, 7, 6, 5, 4, 3, 2, 2, 1];
const CENTER_FORWARD_BONUS: [i32; 9] = [0, 4, 6, 8, 10, 12, 14, 14, 14];
const EXTENDED_CENTER_WEIGHT: f32 = 0.6;
const CORE_CENTER_WEIGHT: f32 = 1.0;

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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

#[derive(Debug, Copy, Clone)]
struct OutpostContext {
    support_margin: i32,
    depth: u8,
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
            let outposts =
                self.evaluate_outposts(board, player, &mut control_cache, captured_pieces);
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
            let space =
                self.evaluate_space_advantage(board, player, &mut control_cache, captured_pieces);
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

    fn bool_to_i32(value: bool) -> i32 {
        if value {
            1
        } else {
            0
        }
    }

    fn forward_steps(pos: Position, player: Player) -> u8 {
        match player {
            Player::Black => 8 - pos.row,
            Player::White => pos.row,
        }
    }

    fn home_steps(pos: Position, player: Player) -> u8 {
        match player {
            Player::Black => pos.row,
            Player::White => 8 - pos.row,
        }
    }

    /// Evaluate center control (shogi-oriented occupancy, mobility, and drop pressure)
    fn evaluate_center_control(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.center_control_checks += 1;
        self.stats.center_control_tiles_scored +=
            (CORE_CENTER.len() + EXTENDED_CENTER.len()) as u64;

        let opponent = player.opposite();
        let mut score = TaperedScore::default();

        for &(row, col) in CORE_CENTER.iter() {
            let pos = Position::new(row, col);
            score += self.center_square_contribution(
                board,
                player,
                opponent,
                control_cache,
                pos,
                CORE_CENTER_WEIGHT,
            );
        }

        for &(row, col) in EXTENDED_CENTER.iter() {
            let pos = Position::new(row, col);
            score += self.center_square_contribution(
                board,
                player,
                opponent,
                control_cache,
                pos,
                EXTENDED_CENTER_WEIGHT,
            );
        }

        if self.config.enable_hand_context {
            let drop_pressure = self.center_drop_pressure(board, player, captured_pieces);
            let drop_relief = self.center_drop_pressure(board, opponent, captured_pieces);
            score += drop_pressure;
            score -= drop_relief;
        }

        self.config.phase_weights.center_control.apply(score)
    }

    fn center_square_contribution(
        &self,
        board: &BitboardBoard,
        player: Player,
        opponent: Player,
        control_cache: &mut ControlCache<'_>,
        pos: Position,
        region_weight: f32,
    ) -> TaperedScore {
        let mut score = TaperedScore::default();

        let occupant = board.get_piece(pos);
        if let Some(piece) = occupant {
            let presence = self.center_piece_presence_score(&piece, pos);
            if piece.player == player {
                score += presence;
            } else {
                score -= presence;
            }
        }

        let player_control = control_cache.controlled_by(player, pos);
        let opponent_control = control_cache.controlled_by(opponent, pos);
        let control_diff = Self::bool_to_i32(player_control) - Self::bool_to_i32(opponent_control);

        if control_diff != 0 {
            let player_forward = Self::forward_steps(pos, player) as usize;
            let opponent_forward = Self::forward_steps(pos, opponent) as usize;
            let index = if control_diff >= 0 {
                player_forward
            } else {
                opponent_forward
            };
            let control_scale_mg = self.config.pawn_center_bonus + CENTER_FORWARD_BONUS[index] / 2;
            let control_scale_eg =
                self.config.pawn_center_bonus / 2 + CENTER_FORWARD_BONUS[index] / 4;
            score += TaperedScore::new_tapered(
                control_diff * control_scale_mg,
                control_diff * control_scale_eg,
            );
        }

        if player_control && occupant.map(|piece| piece.player != player).unwrap_or(true) {
            let forward = Self::forward_steps(pos, player) as usize;
            let mobility_bonus = CENTER_FORWARD_BONUS[forward] / 2 + 4;
            score.mg += mobility_bonus;
            score.eg += mobility_bonus / 2;
        }

        if opponent_control
            && occupant
                .map(|piece| piece.player != opponent)
                .unwrap_or(true)
        {
            let forward = Self::forward_steps(pos, opponent) as usize;
            let mobility_bonus = CENTER_FORWARD_BONUS[forward] / 2 + 4;
            score.mg -= mobility_bonus;
            score.eg -= mobility_bonus / 2;
        }

        score * region_weight
    }

    fn center_piece_presence_score(&self, piece: &Piece, pos: Position) -> TaperedScore {
        let mut score = Self::center_piece_base_score(piece.piece_type);
        let forward = Self::forward_steps(pos, piece.player) as usize;
        let advancement = CENTER_FORWARD_BONUS[forward];
        score.mg += advancement;
        score.eg += advancement / 2;

        if matches!(
            piece.piece_type,
            PieceType::PromotedPawn
                | PieceType::PromotedLance
                | PieceType::PromotedKnight
                | PieceType::PromotedSilver
        ) {
            score.mg += 6;
            score.eg += 4;
        }

        score
    }

    fn center_piece_base_score(piece_type: PieceType) -> TaperedScore {
        match piece_type {
            PieceType::Pawn => TaperedScore::new_tapered(14, 8),
            PieceType::Lance => TaperedScore::new_tapered(18, 12),
            PieceType::Knight => TaperedScore::new_tapered(26, 18),
            PieceType::Silver => TaperedScore::new_tapered(32, 22),
            PieceType::Gold => TaperedScore::new_tapered(36, 26),
            PieceType::Bishop => TaperedScore::new_tapered(44, 34),
            PieceType::Rook => TaperedScore::new_tapered(48, 36),
            PieceType::PromotedBishop => TaperedScore::new_tapered(52, 44),
            PieceType::PromotedRook => TaperedScore::new_tapered(56, 46),
            PieceType::King => TaperedScore::new_tapered(18, 32),
            PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => TaperedScore::new_tapered(34, 26),
        }
    }

    fn center_drop_pressure(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut mg = 0;
        let mut eg = 0;

        let drop_candidates: &[(PieceType, i32, i32)] = &[
            (PieceType::Pawn, 5, 3),
            (PieceType::Silver, 8, 6),
            (PieceType::Gold, 10, 10),
            (PieceType::Knight, 7, 5),
        ];

        for &(piece_type, mg_weight, eg_weight) in drop_candidates {
            let available = captured_pieces.count(piece_type, player) as usize;
            if available == 0 {
                continue;
            }

            let mut used = 0;
            for &(row, col) in CORE_CENTER.iter() {
                let pos = Position::new(row, col);
                if self.drop_is_legal(piece_type, player, board, pos) {
                    let forward = Self::forward_steps(pos, player) as usize;
                    let positional_weight = 4 + CENTER_FORWARD_BONUS[forward] / 2;
                    mg += positional_weight * mg_weight;
                    eg += (positional_weight / 2 + 1) * eg_weight;
                    used += 1;
                    if used >= available {
                        break;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg, eg)
    }

    fn drop_is_legal(
        &self,
        piece_type: PieceType,
        player: Player,
        board: &BitboardBoard,
        pos: Position,
    ) -> bool {
        if board.is_square_occupied(pos) {
            return false;
        }

        if self.is_illegal_drop_rank(piece_type, player, pos.row) {
            return false;
        }

        if piece_type == PieceType::Pawn && self.has_unpromoted_pawn_on_file(board, player, pos.col)
        {
            return false;
        }

        true
    }

    // ===================================================================
    // OUTPOST DETECTION
    // ===================================================================

    /// Evaluate outposts (strong pieces on key squares that cannot be easily attacked)
    fn evaluate_outposts(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.outpost_checks += 1;

        let mut score = TaperedScore::default();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player != player {
                        continue;
                    }
                    self.stats.outpost_candidates += 1;
                    if let Some(context) =
                        self.outpost_context(board, control_cache, pos, &piece, captured_pieces)
                    {
                        score += self.get_outpost_value(&piece, pos, &context);
                        self.stats.outposts_found += 1;
                    }
                }
            }
        }

        self.config.phase_weights.outposts.apply(score)
    }

    /// Check if a square is an outpost for a piece
    fn outpost_context(
        &self,
        board: &BitboardBoard,
        control_cache: &mut ControlCache<'_>,
        pos: Position,
        piece: &Piece,
        captured_pieces: &CapturedPieces,
    ) -> Option<OutpostContext> {
        let player = piece.player;
        let opponent = player.opposite();

        if !self.is_outpost_zone(pos, player) {
            return None;
        }

        if !self.is_outpost_piece(piece.piece_type) {
            return None;
        }

        let support_score = self.outpost_support_score(board, pos, player, captured_pieces);
        if support_score == 0 {
            return None;
        }

        if self.is_under_enemy_pawn_threat(board, pos, player, captured_pieces) {
            return None;
        }

        if self.has_drop_threat(board, pos, player, captured_pieces) {
            return None;
        }

        if !control_cache.controlled_by(player, pos) {
            return None;
        }

        let friendly_support = self.count_controllers(board, player, pos) as i32;
        let enemy_pressure = self.count_controllers(board, opponent, pos) as i32;

        let mut total_support = friendly_support + support_score;
        if self.config.enable_hand_context {
            total_support += self.drop_guard_score(board, pos, player, captured_pieces);
        }

        if total_support <= enemy_pressure {
            return None;
        }

        Some(OutpostContext {
            support_margin: total_support - enemy_pressure,
            depth: Self::forward_steps(pos, player),
        })
    }

    fn outpost_support_score(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        let mut score = 0;
        let behind_row = match player {
            Player::Black => pos.row as i8 + 1,
            Player::White => pos.row as i8 - 1,
        };

        if behind_row < 0 || behind_row >= 9 {
            return 0;
        }

        let support_pos = Position::new(behind_row as u8, pos.col);
        if let Some(piece) = board.get_piece(support_pos) {
            if piece.player == player {
                score += match piece.piece_type {
                    PieceType::Pawn => 2,
                    PieceType::Silver | PieceType::Gold => 3,
                    PieceType::PromotedPawn
                    | PieceType::PromotedLance
                    | PieceType::PromotedKnight
                    | PieceType::PromotedSilver => 3,
                    PieceType::King => 2,
                    _ => 1,
                };
            }
        } else if self.config.enable_hand_context
            && captured_pieces.count(PieceType::Pawn, player) > 0
            && !self.has_unpromoted_pawn_on_file(board, player, pos.col)
            && !self.is_illegal_drop_rank(PieceType::Pawn, player, support_pos.row)
        {
            score += 1;
        }

        for dc in [-1, 1] {
            let diag_col = pos.col as i8 + dc;
            if diag_col < 0 || diag_col >= 9 {
                continue;
            }
            let diag_pos = Position::new(behind_row as u8, diag_col as u8);
            if let Some(piece) = board.get_piece(diag_pos) {
                if piece.player == player
                    && matches!(
                        piece.piece_type,
                        PieceType::Silver
                            | PieceType::Gold
                            | PieceType::PromotedPawn
                            | PieceType::PromotedLance
                            | PieceType::PromotedKnight
                            | PieceType::PromotedSilver
                            | PieceType::King
                    )
                {
                    score += 1;
                }
            }
        }

        score
    }

    fn drop_guard_score(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        if !self.config.enable_drop_threats {
            return 0;
        }

        let mut score = 0;
        let guard_types: &[(PieceType, i32)] = &[
            (PieceType::Gold, 2),
            (PieceType::Silver, 1),
            (PieceType::Pawn, 1),
        ];

        let guard_squares = self.guard_squares_for_player(pos, player);
        for &(piece_type, value) in guard_types {
            if captured_pieces.count(piece_type, player) == 0 {
                continue;
            }
            for target in guard_squares.iter() {
                if self.drop_is_legal(piece_type, player, board, *target) {
                    score += value;
                    break;
                }
            }
        }

        score
    }

    fn guard_squares_for_player(&self, pos: Position, player: Player) -> SmallVec<[Position; 5]> {
        let mut squares: SmallVec<[Position; 5]> = SmallVec::new();
        let behind_row = match player {
            Player::Black => pos.row as i8 + 1,
            Player::White => pos.row as i8 - 1,
        };

        if behind_row < 0 || behind_row >= 9 {
            return squares;
        }

        squares.push(Position::new(behind_row as u8, pos.col));
        for dc in [-1, 1] {
            let diag_col = pos.col as i8 + dc;
            if diag_col < 0 || diag_col >= 9 {
                continue;
            }
            squares.push(Position::new(behind_row as u8, diag_col as u8));
        }

        squares
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
        piece: &Piece,
        pos: Position,
        context: &OutpostContext,
    ) -> TaperedScore {
        let base = match piece.piece_type {
            PieceType::Knight | PieceType::PromotedKnight => TaperedScore::new_tapered(60, 40),
            PieceType::Silver => TaperedScore::new_tapered(58, 46),
            PieceType::Gold
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedSilver => TaperedScore::new_tapered(52, 44),
            PieceType::Bishop | PieceType::PromotedBishop => TaperedScore::new_tapered(64, 58),
            PieceType::Rook | PieceType::PromotedRook => TaperedScore::new_tapered(70, 64),
            _ => TaperedScore::new_tapered(38, 32),
        };

        let depth = context.depth as i32;
        let support_margin = context.support_margin;

        let mut score = base;
        score.mg += depth * 6;
        score.eg += depth * 4;
        score.mg += support_margin * 12;
        score.eg += support_margin * 10;

        if matches!(
            piece.piece_type,
            PieceType::Bishop
                | PieceType::PromotedBishop
                | PieceType::Rook
                | PieceType::PromotedRook
        ) {
            score.mg += depth * 4;
            score.eg += depth * 4;
        }

        if matches!(
            piece.piece_type,
            PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight
        ) {
            score.mg += 6;
            score.eg += 4;
        }

        if Self::home_steps(pos, piece.player) <= 1 {
            score.eg -= 4;
        }

        score
    }

    fn is_outpost_zone(&self, pos: Position, player: Player) -> bool {
        let depth = Self::forward_steps(pos, player);
        depth >= 2 && depth <= 6
    }

    fn is_outpost_piece(&self, piece_type: PieceType) -> bool {
        matches!(
            piece_type,
            PieceType::Knight
                | PieceType::Silver
                | PieceType::Gold
                | PieceType::Bishop
                | PieceType::Rook
                | PieceType::PromotedPawn
                | PieceType::PromotedKnight
                | PieceType::PromotedLance
                | PieceType::PromotedSilver
                | PieceType::PromotedBishop
                | PieceType::PromotedRook
        )
    }

    fn count_controllers(&self, board: &BitboardBoard, player: Player, target: Position) -> u32 {
        let mut count = 0;

        for row in 0..9 {
            for col in 0..9 {
                let from = Position::new(row, col);
                if from == target {
                    continue;
                }
                if let Some(piece) = board.get_piece(from) {
                    if piece.player != player {
                        continue;
                    }
                    if self.piece_controls_square(board, &piece, from, target) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn piece_controls_square(
        &self,
        board: &BitboardBoard,
        piece: &Piece,
        from: Position,
        target: Position,
    ) -> bool {
        if from == target {
            return false;
        }

        let dr = target.row as i8 - from.row as i8;
        let dc = target.col as i8 - from.col as i8;

        match piece.piece_type {
            PieceType::Pawn => {
                let forward = self.player_forward(piece.player);
                dr == forward && dc == 0 && !board.is_square_occupied_by(target, piece.player)
            }
            PieceType::Lance => {
                let forward = self.player_forward(piece.player);
                if dc != 0 || dr == 0 || dr.signum() != forward {
                    return false;
                }
                self.sliding_path_clear(board, from, target, forward, 0, piece.player)
            }
            PieceType::Knight => {
                let forward = self.player_forward(piece.player);
                if dr != 2 * forward || (dc != -1 && dc != 1) {
                    return false;
                }
                !board.is_square_occupied_by(target, piece.player)
            }
            PieceType::Silver => self.step_controls_square(
                board,
                piece,
                dr,
                dc,
                target,
                &self.silver_offsets(piece.player),
            ),
            PieceType::Gold
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => self.step_controls_square(
                board,
                piece,
                dr,
                dc,
                target,
                &self.gold_offsets(piece.player),
            ),
            PieceType::Bishop => {
                if dr.abs() != dc.abs() || dr == 0 {
                    return false;
                }
                self.sliding_path_clear(board, from, target, dr.signum(), dc.signum(), piece.player)
            }
            PieceType::Rook => {
                if dr != 0 && dc != 0 {
                    return false;
                }
                let step_r = dr.signum();
                let step_c = dc.signum();
                if step_r == 0 && step_c == 0 {
                    return false;
                }
                self.sliding_path_clear(board, from, target, step_r, step_c, piece.player)
            }
            PieceType::King => {
                self.step_controls_square(board, piece, dr, dc, target, &self.king_offsets())
            }
            PieceType::PromotedBishop => {
                if dr.abs() == dc.abs() && dr != 0 {
                    return self.sliding_path_clear(
                        board,
                        from,
                        target,
                        dr.signum(),
                        dc.signum(),
                        piece.player,
                    );
                }
                if dr.abs() <= 1
                    && dc.abs() <= 1
                    && (dr == 0 || dc == 0)
                    && !board.is_square_occupied_by(target, piece.player)
                {
                    return true;
                }
                false
            }
            PieceType::PromotedRook => {
                if dr != 0 && dc != 0 {
                    if dr.abs() == 1 && dc.abs() == 1 {
                        return !board.is_square_occupied_by(target, piece.player);
                    }
                    return false;
                }
                let step_r = dr.signum();
                let step_c = dc.signum();
                if step_r == 0 && step_c == 0 {
                    return false;
                }
                self.sliding_path_clear(board, from, target, step_r, step_c, piece.player)
            }
        }
    }

    fn step_controls_square(
        &self,
        board: &BitboardBoard,
        piece: &Piece,
        dr: i8,
        dc: i8,
        target: Position,
        offsets: &[(i8, i8)],
    ) -> bool {
        for &(offset_r, offset_c) in offsets {
            if dr == offset_r && dc == offset_c {
                return !board.is_square_occupied_by(target, piece.player);
            }
        }
        false
    }

    fn sliding_path_clear(
        &self,
        board: &BitboardBoard,
        from: Position,
        target: Position,
        step_r: i8,
        step_c: i8,
        player: Player,
    ) -> bool {
        let mut row = from.row as i8 + step_r;
        let mut col = from.col as i8 + step_c;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let pos = Position::new(row as u8, col as u8);
            if pos == target {
                return !board.is_square_occupied_by(pos, player);
            }
            if board.is_square_occupied(pos) {
                return false;
            }
            row += step_r;
            col += step_c;
        }

        false
    }

    fn gold_offsets(&self, player: Player) -> [(i8, i8); 6] {
        match player {
            Player::Black => [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)],
            Player::White => [(1, -1), (1, 0), (1, 1), (0, -1), (0, 1), (-1, 0)],
        }
    }

    fn silver_offsets(&self, player: Player) -> [(i8, i8); 5] {
        match player {
            Player::Black => [(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)],
            Player::White => [(1, -1), (1, 0), (1, 1), (-1, -1), (-1, 1)],
        }
    }

    fn king_offsets(&self) -> [(i8, i8); 8] {
        [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ]
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

        let opponent = player.opposite();
        let mut penalty = TaperedScore::default();

        let key_squares = self.get_king_zone(board, player);
        self.stats.weak_square_candidates += key_squares.len() as u64;

        for pos in key_squares.into_iter() {
            if !control_cache.controlled_by(opponent, pos) {
                continue;
            }

            let attackers = self.count_controllers(board, opponent, pos);
            if attackers == 0 {
                continue;
            }

            let mut defender_score = self.count_controllers(board, player, pos) as i32;
            defender_score += self.king_zone_guard_bonus(board, pos, player);
            if self.config.enable_hand_context {
                defender_score += self.drop_defense_score(board, player, captured_pieces, pos);
            }

            if defender_score as u32 >= attackers {
                continue;
            }

            let net_pressure = attackers as i32 - defender_score;
            let severity = self.weak_square_severity(board, pos, player, net_pressure);
            if severity.mg == 0 && severity.eg == 0 {
                continue;
            }

            penalty -= severity;
            self.stats.weak_squares_found += 1;
        }

        self.config.phase_weights.weak_squares.apply(penalty)
    }

    fn get_king_zone(&self, board: &BitboardBoard, player: Player) -> SmallVec<[Position; 24]> {
        let mut squares: SmallVec<[Position; 24]> = SmallVec::new();
        if let Some(king_pos) = board.find_king_position(player) {
            for dr in -2..=2 {
                for dc in -2..=2 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let row = king_pos.row as i8 + dr;
                    let col = king_pos.col as i8 + dc;
                    if row < 0 || row >= 9 || col < 0 || col >= 9 {
                        continue;
                    }
                    squares.push(Position::new(row as u8, col as u8));
                }
            }
            squares.push(king_pos);
        } else {
            // Fallback to default castle region if king not found
            let base_rows = if player == Player::Black {
                6..=8
            } else {
                0..=2
            };
            for row in base_rows {
                for col in 3..=5 {
                    squares.push(Position::new(row, col));
                }
            }
        }

        squares
    }

    fn drop_defense_score(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        pos: Position,
    ) -> i32 {
        if !self.config.enable_hand_context {
            return 0;
        }

        let mut score = 0;
        for &(piece_type, value) in &[
            (PieceType::Pawn, 1),
            (PieceType::Silver, 2),
            (PieceType::Gold, 3),
            (PieceType::Knight, 1),
        ] {
            if captured_pieces.count(piece_type, player) == 0 {
                continue;
            }
            if self.drop_is_legal(piece_type, player, board, pos) {
                score += value;
            }
        }

        score
    }

    fn king_zone_guard_bonus(&self, board: &BitboardBoard, pos: Position, player: Player) -> i32 {
        let mut score = 0;

        if let Some(piece) = board.get_piece(pos) {
            if piece.player == player {
                score += match piece.piece_type {
                    PieceType::King => 3,
                    PieceType::Gold
                    | PieceType::Silver
                    | PieceType::PromotedPawn
                    | PieceType::PromotedLance
                    | PieceType::PromotedKnight
                    | PieceType::PromotedSilver => 3,
                    PieceType::Pawn => 1,
                    _ => 2,
                };
            }
        }

        for &(dr, dc) in &self.gold_offsets(player) {
            let row = pos.row as i8 + dr;
            let col = pos.col as i8 + dc;
            if row < 0 || row >= 9 || col < 0 || col >= 9 {
                continue;
            }
            let guard_pos = Position::new(row as u8, col as u8);
            if let Some(piece) = board.get_piece(guard_pos) {
                if piece.player == player {
                    score += match piece.piece_type {
                        PieceType::King => 2,
                        PieceType::Gold
                        | PieceType::PromotedPawn
                        | PieceType::PromotedLance
                        | PieceType::PromotedKnight
                        | PieceType::PromotedSilver => 2,
                        PieceType::Silver => 1,
                        PieceType::Pawn => 1,
                        _ => 1,
                    };
                }
            }
        }

        score
    }

    fn weak_square_severity(
        &self,
        board: &BitboardBoard,
        pos: Position,
        player: Player,
        net_pressure: i32,
    ) -> TaperedScore {
        if net_pressure <= 0 {
            return TaperedScore::default();
        }

        let occupant_penalty = if let Some(piece) = board.get_piece(pos) {
            if piece.player == player {
                match piece.piece_type {
                    PieceType::King => 32,
                    PieceType::Gold
                    | PieceType::Silver
                    | PieceType::PromotedPawn
                    | PieceType::PromotedLance
                    | PieceType::PromotedKnight
                    | PieceType::PromotedSilver => 24,
                    PieceType::Pawn => 12,
                    _ => 16,
                }
            } else {
                10
            }
        } else {
            8
        };

        let row_weight = WEAK_SQUARE_ROW_WEIGHTS[Self::home_steps(pos, player) as usize];
        let base_penalty = self.config.weak_square_penalty + occupant_penalty;
        let mg = base_penalty * net_pressure * row_weight / 4;
        let eg = (base_penalty / 2) * net_pressure * row_weight / 5;

        TaperedScore::new_tapered(mg, eg)
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

        self.config
            .phase_weights
            .piece_activity
            .apply(TaperedScore::new_tapered(mg_score, eg_score))
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
        board: &BitboardBoard,
        player: Player,
        control_cache: &mut ControlCache<'_>,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.space_checks += 1;

        let opponent = player.opposite();
        let mut player_territory = 0;
        let mut opponent_territory = 0;
        let mut player_frontier = 0;
        let mut opponent_frontier = 0;

        self.stats.space_frontier_samples += 81;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                let player_controls = control_cache.controlled_by(player, pos);
                let opponent_controls = control_cache.controlled_by(opponent, pos);

                if player_controls {
                    let depth = Self::forward_steps(pos, player) as usize;
                    let weight = SPACE_ROW_WEIGHTS[depth];
                    player_territory += weight + self.space_occupancy_bonus(board, pos, player);
                    if !opponent_controls {
                        player_frontier += (weight / 2).max(1);
                    }
                }

                if opponent_controls {
                    let depth = Self::forward_steps(pos, opponent) as usize;
                    let weight = SPACE_ROW_WEIGHTS[depth];
                    opponent_territory += weight + self.space_occupancy_bonus(board, pos, opponent);
                    if !player_controls {
                        opponent_frontier += (weight / 2).max(1);
                    }
                }
            }
        }

        let territory_delta = player_territory - opponent_territory;
        let frontier_delta = player_frontier - opponent_frontier;

        let mut mg_score = territory_delta * self.config.space_advantage_bonus;
        let mut eg_score = territory_delta * (self.config.space_advantage_bonus / 2);

        mg_score += frontier_delta * (self.config.space_advantage_bonus * 2);
        eg_score += frontier_delta * self.config.space_advantage_bonus;

        if self.config.enable_hand_context {
            let hand_delta = self.hand_space_pressure(captured_pieces, player)
                - self.hand_space_pressure(captured_pieces, opponent);
            mg_score += hand_delta;
            eg_score += hand_delta / 2;
        }

        self.config
            .phase_weights
            .space_advantage
            .apply(TaperedScore::new_tapered(mg_score, eg_score))
    }

    fn space_occupancy_bonus(&self, board: &BitboardBoard, pos: Position, player: Player) -> i32 {
        if let Some(piece) = board.get_piece(pos) {
            if piece.player == player {
                let depth = Self::forward_steps(pos, player) as i32;
                (depth / 3) + 1
            } else {
                -3
            }
        } else {
            0
        }
    }

    fn hand_space_pressure(&self, captured_pieces: &CapturedPieces, player: Player) -> i32 {
        if !self.config.enable_hand_context {
            return 0;
        }

        let mut pressure = 0;
        for &(piece_type, weight) in &[
            (PieceType::Pawn, 4),
            (PieceType::Lance, 6),
            (PieceType::Knight, 8),
            (PieceType::Silver, 9),
            (PieceType::Gold, 10),
        ] {
            let count = captured_pieces.count(piece_type, player) as i32;
            if count > 0 {
                pressure += count * weight;
            }
        }
        pressure
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

        self.config
            .phase_weights
            .tempo
            .apply(TaperedScore::new_tapered(mg_score, eg_score))
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

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut PositionalConfig {
        &mut self.config
    }
}

impl Default for PositionalPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionalPhaseWeight {
    pub mg: f32,
    pub eg: f32,
}

impl PositionalPhaseWeight {
    fn apply(self, score: TaperedScore) -> TaperedScore {
        TaperedScore::new_tapered(
            ((score.mg as f32) * self.mg).round() as i32,
            ((score.eg as f32) * self.eg).round() as i32,
        )
    }
}

impl Default for PositionalPhaseWeight {
    fn default() -> Self {
        Self { mg: 1.0, eg: 1.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionalPhaseWeights {
    pub center_control: PositionalPhaseWeight,
    pub outposts: PositionalPhaseWeight,
    pub weak_squares: PositionalPhaseWeight,
    pub piece_activity: PositionalPhaseWeight,
    pub space_advantage: PositionalPhaseWeight,
    pub tempo: PositionalPhaseWeight,
}

impl Default for PositionalPhaseWeights {
    fn default() -> Self {
        Self {
            center_control: PositionalPhaseWeight::default(),
            outposts: PositionalPhaseWeight::default(),
            weak_squares: PositionalPhaseWeight::default(),
            piece_activity: PositionalPhaseWeight::default(),
            space_advantage: PositionalPhaseWeight::default(),
            tempo: PositionalPhaseWeight::default(),
        }
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
    pub phase_weights: PositionalPhaseWeights,

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
            phase_weights: PositionalPhaseWeights::default(),

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
    pub center_control_tiles_scored: u64,
    pub outpost_candidates: u64,
    pub weak_square_candidates: u64,
    pub space_frontier_samples: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionalStatsSnapshot {
    pub evaluations: u64,
    pub center_control_checks: u64,
    pub outpost_checks: u64,
    pub weak_square_checks: u64,
    pub activity_checks: u64,
    pub space_checks: u64,
    pub tempo_checks: u64,
    pub outposts_found: u64,
    pub weak_squares_found: u64,
    pub center_control_tiles_scored: u64,
    pub outpost_candidates: u64,
    pub weak_square_candidates: u64,
    pub space_frontier_samples: u64,
}

impl PositionalStats {
    pub fn snapshot(&self) -> PositionalStatsSnapshot {
        PositionalStatsSnapshot {
            evaluations: self.evaluations,
            center_control_checks: self.center_control_checks,
            outpost_checks: self.outpost_checks,
            weak_square_checks: self.weak_square_checks,
            activity_checks: self.activity_checks,
            space_checks: self.space_checks,
            tempo_checks: self.tempo_checks,
            outposts_found: self.outposts_found,
            weak_squares_found: self.weak_squares_found,
            center_control_tiles_scored: self.center_control_tiles_scored,
            outpost_candidates: self.outpost_candidates,
            weak_square_candidates: self.weak_square_candidates,
            space_frontier_samples: self.space_frontier_samples,
        }
    }

    pub fn merge_from(&mut self, snapshot: &PositionalStatsSnapshot) {
        self.evaluations += snapshot.evaluations;
        self.center_control_checks += snapshot.center_control_checks;
        self.outpost_checks += snapshot.outpost_checks;
        self.weak_square_checks += snapshot.weak_square_checks;
        self.activity_checks += snapshot.activity_checks;
        self.space_checks += snapshot.space_checks;
        self.tempo_checks += snapshot.tempo_checks;
        self.outposts_found += snapshot.outposts_found;
        self.weak_squares_found += snapshot.weak_squares_found;
        self.center_control_tiles_scored += snapshot.center_control_tiles_scored;
        self.outpost_candidates += snapshot.outpost_candidates;
        self.weak_square_candidates += snapshot.weak_square_candidates;
        self.space_frontier_samples += snapshot.space_frontier_samples;
    }
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
        let mut control_cache = ControlCache::new(&board);
        let captured = CapturedPieces::new();

        let score =
            analyzer.evaluate_outposts(&board, Player::Black, &mut control_cache, &captured);
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
        let mut cache_no_threat = ControlCache::new(&board);
        let score_without_threat =
            analyzer.evaluate_outposts(&board, Player::Black, &mut cache_no_threat, &captured_none);
        assert!(score_without_threat.mg > 0);

        let mut captured_with_pawn = CapturedPieces::new();
        captured_with_pawn.add_piece(PieceType::Pawn, Player::White);

        let mut cache_with_threat = ControlCache::new(&board);
        let score_with_threat = analyzer.evaluate_outposts(
            &board,
            Player::Black,
            &mut cache_with_threat,
            &captured_with_pawn,
        );
        assert!(score_with_threat.mg < score_without_threat.mg);
    }

    #[test]
    fn test_outpost_requires_structural_support() {
        let mut board = BitboardBoard::empty();
        let mut analyzer = PositionalPatternAnalyzer::new();
        let captured = CapturedPieces::new();

        let outpost_pos = Position::new(4, 4);
        board.place_piece(Piece::new(PieceType::Silver, Player::Black), outpost_pos);

        let mut cache_no_support = ControlCache::new(&board);
        let unsupported =
            analyzer.evaluate_outposts(&board, Player::Black, &mut cache_no_support, &captured);
        assert_eq!(unsupported.mg, 0);

        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(5, 4),
        );

        let mut cache_supported = ControlCache::new(&board);
        let supported =
            analyzer.evaluate_outposts(&board, Player::Black, &mut cache_supported, &captured);
        assert!(supported.mg > unsupported.mg);
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
        assert!(mitigated.mg > penalty.mg);
    }

    #[test]
    fn test_space_advantage_rewards_forward_control() {
        let mut board = BitboardBoard::empty();
        let captured = CapturedPieces::new();

        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(3, 4),
        );
        board.place_piece(
            Piece::new(PieceType::Silver, Player::Black),
            Position::new(4, 3),
        );
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::White),
            Position::new(6, 4),
        );

        let mut analyzer_black = PositionalPatternAnalyzer::new();
        let mut cache_black = ControlCache::new(&board);
        let black_score = analyzer_black.evaluate_space_advantage(
            &board,
            Player::Black,
            &mut cache_black,
            &captured,
        );
        assert!(black_score.mg > 0);

        let mut analyzer_white = PositionalPatternAnalyzer::new();
        let mut cache_white = ControlCache::new(&board);
        let white_score = analyzer_white.evaluate_space_advantage(
            &board,
            Player::White,
            &mut cache_white,
            &captured,
        );
        assert!(white_score.mg < black_score.mg);
    }

    #[test]
    fn test_center_control_phase_weights() {
        let mut board = BitboardBoard::empty();
        board.place_piece(
            Piece::new(PieceType::Gold, Player::Black),
            Position::new(4, 4),
        );

        let mut base_config = PositionalConfig::default();
        base_config.enable_outposts = false;
        base_config.enable_weak_squares = false;
        base_config.enable_piece_activity = false;
        base_config.enable_space_advantage = false;
        base_config.enable_tempo = false;
        base_config.enable_hand_context = false;
        base_config.enable_drop_threats = false;

        let mut baseline_analyzer = PositionalPatternAnalyzer::with_config(base_config.clone());
        let captured = CapturedPieces::new();
        let baseline = baseline_analyzer.evaluate_position(&board, Player::Black, &captured);

        let mut weighted_config = base_config;
        weighted_config.phase_weights.center_control = PositionalPhaseWeight { mg: 2.0, eg: 0.5 };
        let mut weighted_analyzer = PositionalPatternAnalyzer::with_config(weighted_config);
        let weighted = weighted_analyzer.evaluate_position(&board, Player::Black, &captured);

        assert_eq!(weighted.mg, baseline.mg * 2);
        assert_eq!(weighted.eg, ((baseline.eg as f32) * 0.5).round() as i32);
    }

    #[test]
    fn test_positional_stats_snapshot_merge() {
        let mut stats = PositionalStats::default();
        stats.evaluations = 5;
        stats.center_control_checks = 3;
        stats.outposts_found = 1;

        let snapshot = stats.snapshot();

        let mut other = PositionalStats::default();
        other.merge_from(&snapshot);

        assert_eq!(other.evaluations, 5);
        assert_eq!(other.center_control_checks, 3);
        assert_eq!(other.outposts_found, 1);
    }
}
