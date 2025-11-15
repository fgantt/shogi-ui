//! Tactical Pattern Recognition Module
//!
//! This module implements detection of tactical patterns in Shogi including:
//! - Forks (double attacks)
//! - Pins (pieces that cannot move without exposing king/valuable piece)
//! - Skewers (attacks through less valuable piece to more valuable)
//! - Discovered attacks
//! - Knight forks (special case with unique movement)
//! - Back rank threats
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::tactical_patterns::TacticalPatternRecognizer;
//!
//! let recognizer = TacticalPatternRecognizer::new();
//! let captured_pieces = CapturedPieces::new();
//! let tactical_score = recognizer.evaluate_tactics(&board, Player::Black, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::board::CapturedPieces;
use crate::types::core::{Piece, PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

/// Tactical pattern recognizer
pub struct TacticalPatternRecognizer {
    config: TacticalConfig,
    stats: TacticalStats,
}

#[derive(Clone, Copy)]
struct LineStep {
    position: Position,
    occupant: Option<Piece>,
}

struct TacticalDetectionContext<'a> {
    board: &'a BitboardBoard,
    player: Player,
    opponent: Player,
    player_pieces: Vec<(Position, Piece)>,
    opponent_pieces: Vec<(Position, Piece)>,
    captured_pieces: &'a CapturedPieces,
}

impl<'a> TacticalDetectionContext<'a> {
    fn new(board: &'a BitboardBoard, player: Player, captured_pieces: &'a CapturedPieces) -> Self {
        let opponent = player.opposite();
        let mut player_pieces = Vec::new();
        let mut opponent_pieces = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        player_pieces.push((pos, piece));
                    } else {
                        opponent_pieces.push((pos, piece));
                    }
                }
            }
        }

        Self {
            board,
            player,
            opponent,
            player_pieces,
            opponent_pieces,
            captured_pieces,
        }
    }

    fn player_hand(&self) -> &[PieceType] {
        if self.player == Player::Black {
            &self.captured_pieces.black
        } else {
            &self.captured_pieces.white
        }
    }

    fn player_hand_count(&self, piece_type: PieceType) -> usize {
        self.captured_pieces.count(piece_type, self.player)
    }

    fn empty_squares(&self) -> Vec<Position> {
        let mut squares = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if self.board.get_piece(pos).is_none() {
                    squares.push(pos);
                }
            }
        }
        squares
    }

    fn friendly_unpromoted_pawn_on_file(&self, col: u8) -> bool {
        for row in 0..9 {
            let pos = Position::new(row, col);
            if let Some(piece) = self.board.get_piece(pos) {
                if piece.player == self.player && piece.piece_type == PieceType::Pawn {
                    return true;
                }
            }
        }
        false
    }

    fn trace_line(&self, start: Position, dir: (i8, i8)) -> Vec<LineStep> {
        let mut steps = Vec::new();
        let mut row = start.row as i8 + dir.0;
        let mut col = start.col as i8 + dir.1;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let position = Position::new(row as u8, col as u8);
            let occupant = self.board.get_piece(position);
            steps.push(LineStep { position, occupant });

            if occupant.is_some() {
                break;
            }

            row += dir.0;
            col += dir.1;
        }

        steps
    }

    fn collect_single_steps(&self, origin: Position, offsets: &[(i8, i8)]) -> Vec<LineStep> {
        let mut steps = Vec::new();
        for &(dr, dc) in offsets {
            if let Some(position) = Self::offset_position(origin, dr, dc) {
                steps.push(LineStep {
                    position,
                    occupant: self.board.get_piece(position),
                });
            }
        }
        steps
    }

    fn collect_sliding_steps(&self, origin: Position, directions: &[(i8, i8)]) -> Vec<LineStep> {
        let mut result = Vec::new();
        for &dir in directions {
            result.extend(self.trace_line(origin, dir));
        }
        result
    }

    fn gather_attacks(
        &self,
        origin: Position,
        piece_type: PieceType,
        owner: Player,
    ) -> Vec<LineStep> {
        match piece_type {
            PieceType::Rook => self.collect_sliding_steps(origin, &ROOK_DIRECTIONS),
            PieceType::Bishop => self.collect_sliding_steps(origin, &BISHOP_DIRECTIONS),
            PieceType::PromotedRook => {
                let mut steps = self.collect_sliding_steps(origin, &ROOK_DIRECTIONS);
                steps.extend(self.collect_single_steps(origin, &KING_DIAGONAL_OFFSETS));
                steps
            }
            PieceType::PromotedBishop => {
                let mut steps = self.collect_sliding_steps(origin, &BISHOP_DIRECTIONS);
                steps.extend(self.collect_single_steps(origin, &ORTHOGONAL_OFFSETS));
                steps
            }
            PieceType::Knight => {
                let offsets = if owner == Player::Black {
                    KNIGHT_OFFSETS_BLACK
                } else {
                    KNIGHT_OFFSETS_WHITE
                };
                self.collect_single_steps(origin, &offsets)
            }
            PieceType::Lance => {
                let dir = if owner == Player::Black {
                    LANCE_DIRECTION_BLACK
                } else {
                    LANCE_DIRECTION_WHITE
                };
                self.collect_sliding_steps(origin, &[dir])
            }
            PieceType::Gold
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => {
                let offsets = if owner == Player::Black {
                    GOLD_OFFSETS_BLACK
                } else {
                    GOLD_OFFSETS_WHITE
                };
                self.collect_single_steps(origin, &offsets)
            }
            PieceType::Silver => {
                let offsets = if owner == Player::Black {
                    SILVER_OFFSETS_BLACK
                } else {
                    SILVER_OFFSETS_WHITE
                };
                self.collect_single_steps(origin, &offsets)
            }
            PieceType::Pawn => {
                let dir = if owner == Player::Black { -1 } else { 1 };
                self.collect_single_steps(origin, &[(dir, 0)])
            }
            PieceType::King => self.collect_single_steps(origin, &KING_OFFSETS),
        }
    }

    fn direction_towards(from: Position, to: Position) -> Option<(i8, i8)> {
        let dr = to.row as i8 - from.row as i8;
        let dc = to.col as i8 - from.col as i8;

        let dr_sign = dr.signum();
        let dc_sign = dc.signum();

        if dr == 0 {
            if dc == 0 {
                return None;
            }
            return Some((0, dc_sign));
        }

        if dc == 0 {
            return Some((dr_sign, 0));
        }

        if dr.abs() == dc.abs() {
            return Some((dr_sign, dc_sign));
        }

        None
    }

    fn offset_position(origin: Position, dr: i8, dc: i8) -> Option<Position> {
        let new_row = origin.row as i8 + dr;
        let new_col = origin.col as i8 + dc;
        if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
            Some(Position::new(new_row as u8, new_col as u8))
        } else {
            None
        }
    }
}

const ROOK_DIRECTIONS: &[(i8, i8)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRECTIONS: &[(i8, i8)] = &[(1, 1), (-1, 1), (1, -1), (-1, -1)];
const GOLD_OFFSETS_BLACK: &[(i8, i8)] = &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)];
const GOLD_OFFSETS_WHITE: &[(i8, i8)] = &[(1, -1), (1, 0), (1, 1), (0, -1), (0, 1), (-1, 0)];
const SILVER_OFFSETS_BLACK: &[(i8, i8)] = &[(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)];
const SILVER_OFFSETS_WHITE: &[(i8, i8)] = &[(1, -1), (1, 0), (1, 1), (-1, -1), (-1, 1)];
const KNIGHT_OFFSETS_BLACK: &[(i8, i8)] = &[(-2, -1), (-2, 1)];
const KNIGHT_OFFSETS_WHITE: &[(i8, i8)] = &[(2, -1), (2, 1)];
const LANCE_DIRECTION_BLACK: (i8, i8) = (-1, 0);
const LANCE_DIRECTION_WHITE: (i8, i8) = (1, 0);
const KING_OFFSETS: &[(i8, i8)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];
const KING_DIAGONAL_OFFSETS: &[(i8, i8)] = &[(-1, -1), (-1, 1), (1, -1), (1, 1)];
const ORTHOGONAL_OFFSETS: &[(i8, i8)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotifPhaseWeight {
    pub mg: f32,
    pub eg: f32,
}

impl Default for MotifPhaseWeight {
    fn default() -> Self {
        Self { mg: 1.0, eg: 0.7 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TacticalPhaseWeights {
    pub forks: MotifPhaseWeight,
    pub knight_forks: MotifPhaseWeight,
    pub pins: MotifPhaseWeight,
    pub skewers: MotifPhaseWeight,
    pub discovered: MotifPhaseWeight,
    pub back_rank: MotifPhaseWeight,
}

impl Default for TacticalPhaseWeights {
    fn default() -> Self {
        Self {
            forks: MotifPhaseWeight { mg: 1.0, eg: 0.6 },
            knight_forks: MotifPhaseWeight { mg: 1.1, eg: 0.7 },
            pins: MotifPhaseWeight { mg: 1.0, eg: 0.8 },
            skewers: MotifPhaseWeight { mg: 1.0, eg: 0.8 },
            discovered: MotifPhaseWeight { mg: 0.9, eg: 0.6 },
            back_rank: MotifPhaseWeight { mg: 1.2, eg: 0.9 },
        }
    }
}

impl TacticalPatternRecognizer {
    /// Create a new tactical pattern recognizer
    pub fn new() -> Self {
        Self {
            config: TacticalConfig::default(),
            stats: TacticalStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TacticalConfig) -> Self {
        Self {
            config,
            stats: TacticalStats::default(),
        }
    }

    /// Update configuration at runtime.
    pub fn set_config(&mut self, config: TacticalConfig) {
        self.config = config;
    }

    /// Current configuration reference.
    pub fn config(&self) -> &TacticalConfig {
        &self.config
    }

    /// Evaluate all tactical patterns for a player
    pub fn evaluate_tactics(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;
        let context = TacticalDetectionContext::new(board, player, captured_pieces);

        // Detect forks (double attacks)
        if self.config.enable_forks {
            let forks = self.detect_forks(&context);
            mg_score += forks.mg;
            eg_score += forks.eg;
        }

        // Detect pins
        if self.config.enable_pins {
            let pins = self.detect_pins(&context);
            mg_score += pins.mg;
            eg_score += pins.eg;
        }

        // Detect skewers
        if self.config.enable_skewers {
            let skewers = self.detect_skewers(&context);
            mg_score += skewers.mg;
            eg_score += skewers.eg;
        }

        // Detect discovered attacks
        if self.config.enable_discovered_attacks {
            let discovered = self.detect_discovered_attacks(&context);
            mg_score += discovered.mg;
            eg_score += discovered.eg;
        }

        // Detect knight forks (special handling)
        if self.config.enable_knight_forks {
            let knight_forks = self.detect_knight_forks(&context);
            mg_score += knight_forks.mg;
            eg_score += knight_forks.eg;
        }

        // Detect back rank threats
        if self.config.enable_back_rank_threats {
            let back_rank = self.detect_back_rank_threats(&context);
            mg_score += back_rank.mg;
            eg_score += back_rank.eg;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn apply_phase_weights(&self, base_score: i32, weights: &MotifPhaseWeight) -> TaperedScore {
        if base_score == 0 {
            return TaperedScore::default();
        }

        let mg = (base_score as f32 * weights.mg).round() as i32;
        let eg = (base_score as f32 * weights.eg).round() as i32;
        TaperedScore::new_tapered(mg, eg)
    }

    // ===================================================================
    // FORK DETECTION (Double Attacks)
    // ===================================================================

    /// Detect forks (pieces attacking 2+ valuable targets simultaneously)
    fn detect_forks(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.fork_checks += 1;

        let mut total_score = 0;

        for &(pos, piece) in &ctx.player_pieces {
            total_score += self.check_piece_for_forks(ctx, pos, piece.piece_type);
        }

        total_score += self.detect_drop_fork_threats(ctx);

        self.apply_phase_weights(total_score, &self.config.phase_weights.forks)
    }

    /// Check if a piece is forking multiple targets
    fn check_piece_for_forks(
        &self,
        ctx: &TacticalDetectionContext,
        pos: Position,
        piece_type: PieceType,
    ) -> i32 {
        let targets = self.get_attacked_pieces(ctx, pos, piece_type, ctx.player);

        if targets.len() >= 2 {
            let total_value: i32 = targets.iter().map(|(_, value)| *value).sum();
            let fork_bonus = (total_value as f32 * self.config.fork_threat_ratio).round() as i32;

            // Forking king is especially valuable
            let has_king_fork = targets.iter().any(|(pt, _)| *pt == PieceType::King);
            let king_bonus = if has_king_fork {
                self.config.king_fork_bonus_cp
            } else {
                0
            };

            self.stats
                .forks_found
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            fork_bonus + king_bonus
        } else {
            0
        }
    }

    /// Get list of enemy pieces attacked by a piece at given position
    fn get_attacked_pieces(
        &self,
        ctx: &TacticalDetectionContext,
        pos: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<(PieceType, i32)> {
        let mut attacked = Vec::new();
        let opponent = player.opposite();

        for step in ctx.gather_attacks(pos, piece_type, player) {
            if let Some(target_piece) = step.occupant {
                if target_piece.player == opponent {
                    attacked.push((
                        target_piece.piece_type,
                        target_piece.piece_type.base_value(),
                    ));
                }
            }
        }

        attacked
    }

    fn detect_drop_fork_threats(&self, ctx: &TacticalDetectionContext) -> i32 {
        let mut total = 0;
        let drop_pieces = [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Gold,
            PieceType::Silver,
            PieceType::Pawn,
        ];

        if ctx.player_hand().is_empty() {
            return 0;
        }

        let empty_squares = ctx.empty_squares();

        for &piece_type in &drop_pieces {
            if ctx.player_hand_count(piece_type) == 0 {
                continue;
            }

            for &pos in &empty_squares {
                if !self.is_drop_legal(ctx, piece_type, pos) {
                    continue;
                }

                let targets = self.get_attacked_pieces(ctx, pos, piece_type, ctx.player);
                if targets.len() < 2 {
                    continue;
                }

                let total_value: i32 = targets.iter().map(|(_, value)| *value).sum();
                let base_bonus =
                    (total_value as f32 * self.config.fork_threat_ratio).round() as i32;
                let king_bonus = if targets.iter().any(|(pt, _)| *pt == PieceType::King) {
                    self.config.king_fork_bonus_cp
                } else {
                    0
                };
                total += base_bonus + king_bonus;
            }
        }

        total
    }

    fn detect_drop_knight_fork_threats(&self, ctx: &TacticalDetectionContext) -> i32 {
        if ctx.player_hand_count(PieceType::Knight) == 0 {
            return 0;
        }

        let mut total = 0;
        let empty_squares = ctx.empty_squares();

        for &pos in &empty_squares {
            if !self.is_drop_legal(ctx, PieceType::Knight, pos) {
                continue;
            }

            let targets = self.get_attacked_pieces(ctx, pos, PieceType::Knight, ctx.player);
            if targets.len() < 2 {
                continue;
            }

            let total_value: i32 = targets.iter().map(|(_, value)| *value).sum();
            let base_bonus = (total_value as f32 * self.config.knight_fork_ratio).round() as i32;
            let king_bonus = if targets.iter().any(|(pt, _)| *pt == PieceType::King) {
                self.config.king_fork_bonus_cp * 2
            } else {
                0
            };
            total += base_bonus + king_bonus;
        }

        total
    }

    fn detect_drop_pin_threats(&self, ctx: &TacticalDetectionContext) -> i32 {
        let mut total = 0;
        let empty_squares = ctx.empty_squares();
        let drop_pieces = [PieceType::Rook, PieceType::Bishop, PieceType::Lance];

        if ctx.player_hand().is_empty() {
            return 0;
        }

        for &piece_type in &drop_pieces {
            if ctx.player_hand_count(piece_type) == 0 {
                continue;
            }

            let directions: &[(i8, i8)] = match piece_type {
                PieceType::Rook => &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                PieceType::Bishop => &[(1, 1), (-1, 1), (1, -1), (-1, -1)],
                PieceType::Lance => {
                    if ctx.player == Player::Black {
                        &[(-1, 0)]
                    } else {
                        &[(1, 0)]
                    }
                }
                _ => &[],
            };

            for &pos in &empty_squares {
                if !self.is_drop_legal(ctx, piece_type, pos) {
                    continue;
                }

                total += self.evaluate_drop_pin(ctx, pos, piece_type, directions);
            }
        }

        total
    }

    fn evaluate_drop_pin(
        &self,
        ctx: &TacticalDetectionContext,
        origin: Position,
        _piece_type: PieceType,
        directions: &[(i8, i8)],
    ) -> i32 {
        let mut bonus = 0;

        for &(dr, dc) in directions {
            let mut row = origin.row as i8 + dr;
            let mut col = origin.col as i8 + dc;
            let mut first_enemy: Option<Piece> = None;

            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);

                if let Some(piece) = ctx.board.get_piece(pos) {
                    if piece.player == ctx.player {
                        break;
                    }

                    if first_enemy.is_none() {
                        first_enemy = Some(piece);
                    } else {
                        if piece.piece_type == PieceType::King && piece.player == ctx.opponent {
                            if let Some(target) = first_enemy {
                                let value = (target.piece_type.base_value() as f32
                                    * self.config.pin_penalty_ratio)
                                    .round() as i32;
                                bonus += value.max(1);
                            }
                        }
                        break;
                    }
                }

                row += dr;
                col += dc;
            }
        }

        bonus
    }

    fn is_drop_legal(
        &self,
        ctx: &TacticalDetectionContext,
        piece_type: PieceType,
        pos: Position,
    ) -> bool {
        if ctx.board.get_piece(pos).is_some() {
            return false;
        }

        match piece_type {
            PieceType::Pawn => {
                if (ctx.player == Player::Black && pos.row == 0)
                    || (ctx.player == Player::White && pos.row == 8)
                {
                    return false;
                }
                if ctx.friendly_unpromoted_pawn_on_file(pos.col) {
                    return false;
                }
                true
            }
            PieceType::Knight => {
                if ctx.player == Player::Black {
                    pos.row > 1
                } else {
                    pos.row < 7
                }
            }
            PieceType::Lance => {
                if ctx.player == Player::Black {
                    pos.row > 0
                } else {
                    pos.row < 8
                }
            }
            _ => true,
        }
    }

    // ===================================================================
    // PIN DETECTION
    // ===================================================================

    /// Detect pins (pieces that cannot move without exposing valuable piece)
    fn detect_pins(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.pin_checks += 1;

        let mut total_score = 0;
        if let Some(king_pos) = self.find_king_position(ctx.board, ctx.player) {
            total_score += self.check_pins_in_directions(
                ctx,
                king_pos,
                ctx.player,
                &[(1, 0), (-1, 0), (0, 1), (0, -1)],
            );
            total_score += self.check_pins_in_directions(
                ctx,
                king_pos,
                ctx.player,
                &[(1, 1), (-1, 1), (1, -1), (-1, -1)],
            );
        }

        let drop_bonus = self.detect_drop_pin_threats(ctx);
        total_score += drop_bonus;

        if total_score == 0 {
            TaperedScore::default()
        } else {
            self.apply_phase_weights(total_score, &self.config.phase_weights.pins)
        }
    }

    /// Check for pins in given directions
    fn check_pins_in_directions(
        &self,
        ctx: &TacticalDetectionContext,
        king_pos: Position,
        player: Player,
        directions: &[(i8, i8)],
    ) -> i32 {
        let mut pin_penalty = 0;
        let opponent = player.opposite();

        for &(dr, dc) in directions {
            let mut first_friendly: Option<Piece> = None;
            let mut row = king_pos.row as i8 + dr;
            let mut col = king_pos.col as i8 + dc;

            // Scan outward from king
            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);

                if let Some(piece) = ctx.board.get_piece(pos) {
                    if piece.player == player {
                        if first_friendly.is_some() {
                            // Two friendly pieces before encountering attacker: no pin
                            break;
                        }
                        first_friendly = Some(piece);
                    } else {
                        if let Some(friendly) = first_friendly {
                            if piece.player == opponent
                                && self.can_pin_along_line(piece.piece_type, dr, dc)
                            {
                                let pinned_value = friendly.piece_type.base_value();
                                let penalty = (pinned_value as f32 * self.config.pin_penalty_ratio)
                                    .round() as i32;
                                let penalty = penalty.max(1);
                                pin_penalty -= penalty;
                                self.stats
                                    .pins_found
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                        break;
                    }
                }

                row += dr;
                col += dc;
            }
        }

        pin_penalty
    }

    /// Check if piece type can create pins along given direction
    fn can_pin_along_line(&self, piece_type: PieceType, dr: i8, dc: i8) -> bool {
        match piece_type {
            PieceType::Rook | PieceType::PromotedRook | PieceType::Lance => {
                // Can pin along ranks and files
                dr == 0 || dc == 0
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                // Can pin along diagonals
                dr.abs() == dc.abs()
            }
            _ => false,
        }
    }

    // ===================================================================
    // SKEWER DETECTION
    // ===================================================================

    /// Detect skewers (attacking through piece to hit more valuable target)
    fn detect_skewers(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.skewer_checks += 1;

        let mut total_penalty = 0;

        for &(pos, piece) in &ctx.opponent_pieces {
            match piece.piece_type {
                PieceType::Rook | PieceType::PromotedRook => {
                    total_penalty += self.check_skewers_from_piece(
                        ctx,
                        pos,
                        &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                    );
                }
                PieceType::Bishop | PieceType::PromotedBishop => {
                    total_penalty += self.check_skewers_from_piece(
                        ctx,
                        pos,
                        &[(1, 1), (-1, 1), (1, -1), (-1, -1)],
                    );
                }
                _ => {}
            }
        }

        self.apply_phase_weights(total_penalty, &self.config.phase_weights.skewers)
    }

    /// Check for skewers from a specific piece position
    fn check_skewers_from_piece(
        &self,
        ctx: &TacticalDetectionContext,
        pos: Position,
        directions: &[(i8, i8)],
    ) -> i32 {
        let mut penalty = 0;

        for &(dr, dc) in directions {
            let mut row = pos.row as i8 + dr;
            let mut col = pos.col as i8 + dc;
            let mut front_piece: Option<Piece> = None;

            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let check_pos = Position::new(row as u8, col as u8);

                if let Some(piece) = ctx.board.get_piece(check_pos) {
                    if piece.player == ctx.player {
                        if let Some(front) = front_piece {
                            let front_value = front.piece_type.base_value();
                            let back_value = piece.piece_type.base_value();

                            if back_value > front_value {
                                let delta = back_value - front_value;
                                let skew_penalty = (delta as f32 * self.config.skewer_penalty_ratio)
                                    .round()
                                    as i32;
                                let skew_penalty = skew_penalty.max(1);
                                penalty -= skew_penalty;
                                self.stats
                                    .skewers_found
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            break;
                        } else {
                            front_piece = Some(piece);
                        }
                    } else {
                        // Encountered opponent piece blocking line
                        break;
                    }
                }

                row += dr;
                col += dc;
            }
        }

        penalty
    }

    // ===================================================================
    // DISCOVERED ATTACK DETECTION
    // ===================================================================

    /// Detect discovered attack potential
    fn detect_discovered_attacks(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.discovered_checks += 1;

        let mut total_bonus = 0;
        let opponent = ctx.opponent;

        // Find opponent king
        let opp_king_pos = match self.find_king_position(ctx.board, opponent) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check if any of our pieces can create discovered attacks by moving
        for &(pos, _) in &ctx.player_pieces {
            if self.can_create_discovered_attack(ctx, pos, opp_king_pos) {
                total_bonus += self.config.discovered_attack_bonus_cp;
                self.stats
                    .discovered_attacks_found
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        self.apply_phase_weights(total_bonus, &self.config.phase_weights.discovered)
    }

    /// Check if moving a piece can create a discovered attack
    fn can_create_discovered_attack(
        &self,
        ctx: &TacticalDetectionContext,
        piece_pos: Position,
        target_pos: Position,
    ) -> bool {
        // Check if there's a friendly sliding piece behind this piece that would attack target
        let direction = match TacticalDetectionContext::direction_towards(piece_pos, target_pos) {
            Some(dir) => dir,
            None => return false,
        };

        // Path between piece and target must be clear
        let mut row = piece_pos.row as i8 + direction.0;
        let mut col = piece_pos.col as i8 + direction.1;
        let mut reached_target = false;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let check_pos = Position::new(row as u8, col as u8);
            if check_pos == target_pos {
                reached_target = true;
                break;
            }

            if ctx.board.get_piece(check_pos).is_some() {
                return false;
            }

            row += direction.0;
            col += direction.1;
        }

        if !reached_target {
            return false;
        }

        // Look behind for sliding piece that would attack along this line
        let behind_direction = (-direction.0, -direction.1);
        let mut row = piece_pos.row as i8 + behind_direction.0;
        let mut col = piece_pos.col as i8 + behind_direction.1;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let check_pos = Position::new(row as u8, col as u8);
            match ctx.board.get_piece(check_pos) {
                Some(piece) if piece.player == ctx.player => {
                    return self.can_pin_along_line(piece.piece_type, direction.0, direction.1);
                }
                Some(_) => return false,
                None => {
                    row += behind_direction.0;
                    col += behind_direction.1;
                }
            }
        }

        false
    }

    // ===================================================================
    // KNIGHT FORK DETECTION
    // ===================================================================

    /// Detect knight fork patterns (special handling for knight's unique movement)
    fn detect_knight_forks(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.knight_fork_checks += 1;

        let mut total_bonus = 0;

        for &(pos, piece) in &ctx.player_pieces {
            if piece.piece_type == PieceType::Knight {
                total_bonus += self.check_knight_for_forks(ctx, pos);
            }
        }

        total_bonus += self.detect_drop_knight_fork_threats(ctx);

        self.apply_phase_weights(total_bonus, &self.config.phase_weights.knight_forks)
    }

    /// Check if a knight is creating a fork
    fn check_knight_for_forks(&self, ctx: &TacticalDetectionContext, pos: Position) -> i32 {
        let targets = self.get_attacked_pieces(ctx, pos, PieceType::Knight, ctx.player);

        if targets.len() >= 2 {
            let total_value: i32 = targets.iter().map(|(_, value)| *value).sum();
            let has_king = targets.iter().any(|(pt, _)| *pt == PieceType::King);

            let base_bonus = (total_value as f32 * self.config.knight_fork_ratio).round() as i32;
            let king_bonus = if has_king {
                self.config.king_fork_bonus_cp * 2
            } else {
                0
            };

            self.stats
                .knight_forks_found
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            base_bonus + king_bonus
        } else {
            0
        }
    }

    // ===================================================================
    // BACK RANK THREAT DETECTION
    // ===================================================================

    /// Detect back rank threats (king trapped on back rank)
    fn detect_back_rank_threats(&mut self, ctx: &TacticalDetectionContext) -> TaperedScore {
        self.stats.back_rank_checks += 1;

        let king_pos = match self.find_king_position(ctx.board, ctx.player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check if king is on back rank
        let back_rank = if ctx.player == Player::Black { 8 } else { 0 };

        if king_pos.row != back_rank {
            return TaperedScore::default();
        }

        // Check if king is trapped (no escape squares)
        let escape_count = self.count_king_escape_squares(ctx.board, king_pos, ctx.player);

        if escape_count <= 1 {
            // King is trapped - check for enemy threats on back rank
            let threats = self.count_back_rank_threats(ctx, king_pos);

            if threats > 0 {
                let scaling_divisor = (escape_count + 1) as i32;
                let penalty = threats * self.config.back_rank_penalty_cp / scaling_divisor;

                if penalty != 0 {
                    self.stats
                        .back_rank_threats_found
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return self
                        .apply_phase_weights(-penalty, &self.config.phase_weights.back_rank);
                }
            }
        }

        TaperedScore::default()
    }

    /// Count escape squares for king
    fn count_king_escape_squares(
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

                    // Check if square is empty or has enemy piece
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player != player {
                            count += 1;
                        }
                    } else {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Count enemy threats on back rank
    fn count_back_rank_threats(&self, ctx: &TacticalDetectionContext, king_pos: Position) -> i32 {
        let mut threats = 0;

        for &(pos, piece) in &ctx.opponent_pieces {
            if pos.row != king_pos.row {
                continue;
            }

            match piece.piece_type {
                PieceType::Rook | PieceType::PromotedRook => {
                    if let Some(dir) = TacticalDetectionContext::direction_towards(pos, king_pos) {
                        // Ensure the path from attacker to king is unobstructed
                        let mut row = pos.row as i8 + dir.0;
                        let mut col = pos.col as i8 + dir.1;
                        let mut blocked = false;

                        while row >= 0 && row < 9 && col >= 0 && col < 9 {
                            let step_pos = Position::new(row as u8, col as u8);

                            if step_pos == king_pos {
                                break;
                            }

                            if ctx.board.get_piece(step_pos).is_some() {
                                blocked = true;
                                break;
                            }

                            row += dir.0;
                            col += dir.1;
                        }

                        if !blocked && row >= 0 && row < 9 && col >= 0 && col < 9 {
                            threats += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        threats
    }

    // ===================================================================
    // HELPER METHODS
    // ===================================================================

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
    pub fn stats(&self) -> &TacticalStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = TacticalStats::default();
    }
}

impl Default for TacticalPatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for tactical pattern recognition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TacticalConfig {
    pub enable_forks: bool,
    pub enable_pins: bool,
    pub enable_skewers: bool,
    pub enable_discovered_attacks: bool,
    pub enable_knight_forks: bool,
    pub enable_back_rank_threats: bool,

    // Centipawn-scaled scoring parameters
    pub fork_threat_ratio: f32,
    pub knight_fork_ratio: f32,
    pub king_fork_bonus_cp: i32,
    pub pin_penalty_ratio: f32,
    pub skewer_penalty_ratio: f32,
    pub discovered_attack_bonus_cp: i32,
    pub back_rank_penalty_cp: i32,

    pub phase_weights: TacticalPhaseWeights,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TacticalPreset {
    Balanced,
    Aggressive,
    Conservative,
}

impl Default for TacticalConfig {
    fn default() -> Self {
        Self {
            enable_forks: true,
            enable_pins: true,
            enable_skewers: true,
            enable_discovered_attacks: true,
            enable_knight_forks: true,
            enable_back_rank_threats: true,

            fork_threat_ratio: 0.18,
            knight_fork_ratio: 0.22,
            king_fork_bonus_cp: 120,
            pin_penalty_ratio: 0.40,
            skewer_penalty_ratio: 0.28,
            discovered_attack_bonus_cp: 80,
            back_rank_penalty_cp: 140,

            phase_weights: TacticalPhaseWeights::default(),
        }
    }
}

impl TacticalConfig {
    pub fn balanced() -> Self {
        Self::default()
    }

    pub fn aggressive() -> Self {
        let mut config = Self::default();
        config.fork_threat_ratio = 0.24;
        config.knight_fork_ratio = 0.28;
        config.king_fork_bonus_cp = 180;
        config.pin_penalty_ratio = 0.55;
        config.skewer_penalty_ratio = 0.42;
        config.discovered_attack_bonus_cp = 120;
        config.back_rank_penalty_cp = 220;
        config.phase_weights = TacticalPhaseWeights {
            forks: MotifPhaseWeight { mg: 1.3, eg: 0.9 },
            knight_forks: MotifPhaseWeight { mg: 1.4, eg: 1.0 },
            pins: MotifPhaseWeight { mg: 1.2, eg: 1.0 },
            skewers: MotifPhaseWeight { mg: 1.2, eg: 1.0 },
            discovered: MotifPhaseWeight { mg: 1.1, eg: 0.8 },
            back_rank: MotifPhaseWeight { mg: 1.5, eg: 1.2 },
        };
        config
    }

    pub fn conservative() -> Self {
        let mut config = Self::default();
        config.fork_threat_ratio = 0.12;
        config.knight_fork_ratio = 0.15;
        config.king_fork_bonus_cp = 80;
        config.pin_penalty_ratio = 0.25;
        config.skewer_penalty_ratio = 0.2;
        config.discovered_attack_bonus_cp = 40;
        config.back_rank_penalty_cp = 90;
        config.phase_weights = TacticalPhaseWeights {
            forks: MotifPhaseWeight { mg: 0.7, eg: 0.4 },
            knight_forks: MotifPhaseWeight { mg: 0.8, eg: 0.5 },
            pins: MotifPhaseWeight { mg: 0.6, eg: 0.5 },
            skewers: MotifPhaseWeight { mg: 0.6, eg: 0.5 },
            discovered: MotifPhaseWeight { mg: 0.5, eg: 0.3 },
            back_rank: MotifPhaseWeight { mg: 0.8, eg: 0.6 },
        };
        config
    }

    pub fn preset(preset: TacticalPreset) -> Self {
        match preset {
            TacticalPreset::Balanced => TacticalConfig::balanced(),
            TacticalPreset::Aggressive => TacticalConfig::aggressive(),
            TacticalPreset::Conservative => TacticalConfig::conservative(),
        }
    }

    pub fn apply_preset(&mut self, preset: TacticalPreset) {
        *self = TacticalConfig::preset(preset);
    }
}

/// Statistics for tactical pattern recognition
#[derive(Debug, Default)]
pub struct TacticalStats {
    pub evaluations: u64,
    pub fork_checks: u64,
    pub pin_checks: u64,
    pub skewer_checks: u64,
    pub discovered_checks: u64,
    pub knight_fork_checks: u64,
    pub back_rank_checks: u64,

    pub forks_found: std::sync::atomic::AtomicU64,
    pub pins_found: std::sync::atomic::AtomicU64,
    pub skewers_found: std::sync::atomic::AtomicU64,
    pub discovered_attacks_found: std::sync::atomic::AtomicU64,
    pub knight_forks_found: std::sync::atomic::AtomicU64,
    pub back_rank_threats_found: std::sync::atomic::AtomicU64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TacticalStatsSnapshot {
    pub evaluations: u64,
    pub fork_checks: u64,
    pub pin_checks: u64,
    pub skewer_checks: u64,
    pub discovered_checks: u64,
    pub knight_fork_checks: u64,
    pub back_rank_checks: u64,
    pub forks_found: u64,
    pub pins_found: u64,
    pub skewers_found: u64,
    pub discovered_attacks_found: u64,
    pub knight_forks_found: u64,
    pub back_rank_threats_found: u64,
}

impl TacticalStats {
    pub fn snapshot(&self) -> TacticalStatsSnapshot {
        TacticalStatsSnapshot {
            evaluations: self.evaluations,
            fork_checks: self.fork_checks,
            pin_checks: self.pin_checks,
            skewer_checks: self.skewer_checks,
            discovered_checks: self.discovered_checks,
            knight_fork_checks: self.knight_fork_checks,
            back_rank_checks: self.back_rank_checks,
            forks_found: self.forks_found.load(Ordering::Relaxed),
            pins_found: self.pins_found.load(Ordering::Relaxed),
            skewers_found: self.skewers_found.load(Ordering::Relaxed),
            discovered_attacks_found: self.discovered_attacks_found.load(Ordering::Relaxed),
            knight_forks_found: self.knight_forks_found.load(Ordering::Relaxed),
            back_rank_threats_found: self.back_rank_threats_found.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tactical_recognizer_creation() {
        let recognizer = TacticalPatternRecognizer::new();
        assert!(recognizer.config.enable_forks);
        assert!(recognizer.config.enable_pins);
    }

    #[test]
    fn test_tactical_config_default() {
        let config = TacticalConfig::default();
        assert!((config.fork_threat_ratio - 0.18).abs() < f32::EPSILON);
        assert_eq!(config.king_fork_bonus_cp, 120);
    }

    #[test]
    fn test_fork_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
        assert!(score.mg >= 0);
        assert!(score.eg >= 0);
    }

    #[test]
    fn test_pin_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
        assert!(score.mg >= 0);
        assert!(score.eg >= 0);
    }

    #[test]
    fn test_knight_fork_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
        assert!(score.mg >= 0);
    }

    #[test]
    fn test_evaluate_tactics() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
        assert_eq!(recognizer.stats().evaluations, 1);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        recognizer.evaluate_tactics(&board, Player::Black, &captured);

        let stats = recognizer.stats();
        assert!(stats.fork_checks >= 1);
        assert!(stats.pin_checks >= 1);
    }

    #[test]
    fn test_reset_statistics() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        recognizer.evaluate_tactics(&board, Player::Black, &captured);
        recognizer.reset_stats();

        assert_eq!(recognizer.stats().evaluations, 0);
    }
}
