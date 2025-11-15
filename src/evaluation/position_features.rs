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
//! let king_safety =
//!     evaluator.evaluate_king_safety(&board, Player::Black, &CapturedPieces::new());
//! let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::moves::MoveGenerator;
use crate::types::board::CapturedPieces;
use crate::types::core::{Piece, PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};

fn is_gold_equivalent(piece_type: PieceType) -> bool {
    matches!(
        piece_type,
        PieceType::Gold
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver
    )
}

fn is_promoted_major(piece_type: PieceType) -> bool {
    matches!(
        piece_type,
        PieceType::PromotedBishop | PieceType::PromotedRook
    )
}

fn oriented_coords(pos: Position, player: Player) -> (i8, i8) {
    if player == Player::Black {
        (pos.row as i8, pos.col as i8)
    } else {
        ((8 - pos.row) as i8, (8 - pos.col) as i8)
    }
}

fn oriented_offset_to_actual(base: Position, player: Player, dr: i8, dc: i8) -> Option<Position> {
    let target_row = if player == Player::Black {
        base.row as i8 + dr
    } else {
        base.row as i8 - dr
    };
    let target_col = if player == Player::Black {
        base.col as i8 + dc
    } else {
        base.col as i8 - dc
    };

    if target_row < 0 || target_row >= 9 || target_col < 0 || target_col >= 9 {
        None
    } else {
        Some(Position::new(target_row as u8, target_col as u8))
    }
}

fn is_illegal_pawn_drop_rank(player: Player, row: u8) -> bool {
    match player {
        Player::Black => row == 0,
        Player::White => row == 8,
    }
}

fn is_illegal_lance_drop_rank(player: Player, row: u8) -> bool {
    match player {
        Player::Black => row == 0,
        Player::White => row == 8,
    }
}

fn is_illegal_knight_drop_rank(player: Player, row: u8) -> bool {
    match player {
        Player::Black => row <= 1,
        Player::White => row >= 7,
    }
}

#[derive(Copy, Clone)]
enum HandPieceKind {
    GoldLike,
    Silver,
    Pawn,
    Lance,
    Knight,
}

#[derive(Copy, Clone)]
struct GuardDropTarget {
    dr: i8,
    dc: i8,
    gold_bonus: (i32, i32),
    silver_bonus: Option<(i32, i32)>,
    pawn_bonus: Option<(i32, i32)>,
    lance_bonus: Option<(i32, i32)>,
    knight_bonus: Option<(i32, i32)>,
}

#[derive(Copy, Clone)]
struct CastleGuardRequirement {
    dr: i8,
    dc: i8,
    accept_gold_like: bool,
    accept_silver: bool,
    drop_bonus: (i32, i32),
}

#[derive(Copy, Clone)]
struct HandCounts {
    gold: i32,
    silver: i32,
}

impl HandCounts {
    fn new(captured: &CapturedPieces, player: Player) -> Self {
        Self {
            gold: captured.count(PieceType::Gold, player) as i32,
            silver: captured.count(PieceType::Silver, player) as i32,
        }
    }

    fn has_gold(&self) -> bool {
        self.gold > 0
    }

    fn has_silver(&self) -> bool {
        self.silver > 0
    }

    fn use_gold(&mut self) {
        self.gold -= 1;
    }

    fn use_silver(&mut self) {
        self.silver -= 1;
    }
}

enum GuardState {
    OnBoard,
    FulfilledByDrop(i32, i32),
    Missing,
}

const MOBILITY_BOARD_AREA: usize = 81;
const ALL_PIECE_TYPES: [PieceType; PieceType::COUNT] = [
    PieceType::Pawn,
    PieceType::Lance,
    PieceType::Knight,
    PieceType::Silver,
    PieceType::Gold,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::King,
    PieceType::PromotedPawn,
    PieceType::PromotedLance,
    PieceType::PromotedKnight,
    PieceType::PromotedSilver,
    PieceType::PromotedBishop,
    PieceType::PromotedRook,
];

#[inline]
fn index_for_position(pos: Position) -> usize {
    (pos.row as usize) * 9 + pos.col as usize
}

#[derive(Default, Clone, Copy)]
struct PieceMobilityStats {
    total_moves: i32,
    central_moves: i32,
    attack_moves: i32,
}

#[derive(Default, Clone, Copy)]
struct DropMobilityStats {
    total_moves: i32,
    central_moves: i32,
}

#[derive(Default, Clone, Copy)]
struct ControlStrength {
    mg: i32,
    eg: i32,
}

#[derive(Default)]
struct PositionFeatureInputCache {
    prepared: bool,
    king_positions: [Option<Position>; 2],
    pawns: [Vec<Position>; 2],
}

impl PositionFeatureInputCache {
    fn clear(&mut self) {
        self.prepared = false;
        self.king_positions = [None, None];
        self.pawns[0].clear();
        self.pawns[1].clear();
    }
}

/// Position feature evaluator with phase-aware evaluation
pub struct PositionFeatureEvaluator {
    /// Configuration for position evaluation
    config: PositionFeatureConfig,
    /// Statistics tracking
    stats: PositionFeatureStats,
    /// Shared move generator to avoid repeated construction
    move_generator: MoveGenerator,
    /// Cached board inputs shared across feature evaluators
    inputs_cache: PositionFeatureInputCache,
}

impl PositionFeatureEvaluator {
    /// Create a new PositionFeatureEvaluator with default configuration
    pub fn new() -> Self {
        Self {
            config: PositionFeatureConfig::default(),
            stats: PositionFeatureStats::default(),
            move_generator: MoveGenerator::new(),
            inputs_cache: PositionFeatureInputCache::default(),
        }
    }

    /// Create a new evaluator with custom configuration
    pub fn with_config(config: PositionFeatureConfig) -> Self {
        Self {
            config,
            stats: PositionFeatureStats::default(),
            move_generator: MoveGenerator::new(),
            inputs_cache: PositionFeatureInputCache::default(),
        }
    }

    #[inline]
    fn player_index(player: Player) -> usize {
        match player {
            Player::Black => 0,
            Player::White => 1,
        }
    }

    fn ensure_inputs(&mut self, board: &BitboardBoard) {
        if self.inputs_cache.prepared {
            return;
        }

        self.inputs_cache.clear();
        self.inputs_cache.prepared = true;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let idx = Self::player_index(piece.player);

                    if piece.piece_type == PieceType::King
                        && self.inputs_cache.king_positions[idx].is_none()
                    {
                        self.inputs_cache.king_positions[idx] = Some(pos);
                    }

                    if piece.piece_type == PieceType::Pawn {
                        self.inputs_cache.pawns[idx].push(pos);
                    }
                }
            }
        }
    }

    /// Prepare cached inputs for a single evaluation pass.
    pub fn begin_evaluation(&mut self, board: &BitboardBoard) {
        self.ensure_inputs(board);
    }

    /// Clear cached inputs after an evaluation pass.
    pub fn end_evaluation(&mut self) {
        self.inputs_cache.clear();
    }

    /// Get current configuration
    pub fn config(&self) -> &PositionFeatureConfig {
        &self.config
    }

    /// Update the evaluator configuration.
    pub fn set_config(&mut self, config: PositionFeatureConfig) {
        self.config = config;
    }

    // =======================================================================
    // KING SAFETY EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate king safety with phase-aware weights
    ///
    /// King safety is more critical in middlegame when there are more pieces
    /// that can mount an attack.
    pub fn evaluate_king_safety(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        if !self.config.enable_king_safety {
            return TaperedScore::default();
        }

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

        // 3. Hand pieces that can reinforce the king
        let hand_defense = self.evaluate_hand_defense(board, king_pos, player, captured_pieces);
        mg_score += hand_defense.mg;
        eg_score += hand_defense.eg;

        // 4. Recognise castle structures
        let castle_bonus = self.evaluate_castle_patterns(board, king_pos, player, captured_pieces);
        mg_score += castle_bonus.mg;
        eg_score += castle_bonus.eg;

        let enemy_hand_pressure =
            self.evaluate_enemy_hand_pressure(board, king_pos, player, captured_pieces);
        mg_score -= enemy_hand_pressure.mg;
        eg_score -= enemy_hand_pressure.eg;

        // 5. Enemy attackers near king
        let attacker_penalty = self.evaluate_enemy_attackers(board, king_pos, player);
        mg_score -= attacker_penalty.mg;
        eg_score -= attacker_penalty.eg;

        // 6. King exposure (open squares near king)
        let exposure = self.evaluate_king_exposure(board, king_pos, player);
        mg_score -= exposure.mg;
        eg_score -= exposure.eg;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn evaluate_enemy_hand_pressure(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let opponent = player.opposite();
        let mut mg_penalty = 0;
        let mut eg_penalty = 0;

        let pawn_in_hand = captured_pieces.count(PieceType::Pawn, opponent) as i32;
        if pawn_in_hand > 0 {
            if let Some(front_square) = oriented_offset_to_actual(king_pos, opponent, -1, 0) {
                if !board.is_square_occupied(front_square)
                    && !self.column_has_unpromoted_pawn(board, opponent, front_square.col)
                    && !is_illegal_pawn_drop_rank(opponent, front_square.row)
                {
                    mg_penalty += 32;
                    eg_penalty += 18;
                }
            }
        }

        let gold_in_hand = captured_pieces.count(PieceType::Gold, opponent) as i32;
        let silver_in_hand = captured_pieces.count(PieceType::Silver, opponent) as i32;
        if gold_in_hand > 0 || silver_in_hand > 0 {
            const THREAT_TARGETS: &[GuardDropTarget] = &[
                GuardDropTarget {
                    dr: -1,
                    dc: 0,
                    gold_bonus: (28, 16),
                    silver_bonus: Some((18, 10)),
                    pawn_bonus: None,
                    lance_bonus: None,
                    knight_bonus: None,
                },
                GuardDropTarget {
                    dr: 0,
                    dc: -1,
                    gold_bonus: (20, 12),
                    silver_bonus: Some((16, 10)),
                    pawn_bonus: None,
                    lance_bonus: None,
                    knight_bonus: None,
                },
                GuardDropTarget {
                    dr: 0,
                    dc: 1,
                    gold_bonus: (20, 12),
                    silver_bonus: Some((16, 10)),
                    pawn_bonus: None,
                    lance_bonus: None,
                    knight_bonus: None,
                },
                GuardDropTarget {
                    dr: -1,
                    dc: -1,
                    gold_bonus: (24, 14),
                    silver_bonus: Some((20, 12)),
                    pawn_bonus: None,
                    lance_bonus: None,
                    knight_bonus: None,
                },
                GuardDropTarget {
                    dr: -1,
                    dc: 1,
                    gold_bonus: (24, 14),
                    silver_bonus: Some((20, 12)),
                    pawn_bonus: None,
                    lance_bonus: None,
                    knight_bonus: None,
                },
            ];

            for target in THREAT_TARGETS {
                if let Some(pos) =
                    oriented_offset_to_actual(king_pos, opponent, target.dr, target.dc)
                {
                    if board.is_square_occupied(pos) {
                        continue;
                    }

                    if gold_in_hand > 0 {
                        mg_penalty += target.gold_bonus.0;
                        eg_penalty += target.gold_bonus.1;
                        break;
                    } else if silver_in_hand > 0 {
                        if let Some(bonus) = target.silver_bonus {
                            mg_penalty += bonus.0;
                            eg_penalty += bonus.1;
                            break;
                        }
                    }
                }
            }
        }

        let knight_in_hand = captured_pieces.count(PieceType::Knight, opponent) as i32;
        if knight_in_hand > 0 {
            let knight_sources = [(-2, -1), (-2, 1)];
            for (dr, dc) in knight_sources {
                if let Some(pos) = oriented_offset_to_actual(king_pos, opponent, dr, dc) {
                    if board.is_square_occupied(pos)
                        || is_illegal_knight_drop_rank(opponent, pos.row)
                    {
                        continue;
                    }
                    mg_penalty += 18;
                    eg_penalty += 12;
                    break;
                }
            }
        }

        TaperedScore::new_tapered(mg_penalty, eg_penalty)
    }

    /// Evaluate king shield (friendly pieces near king)
    fn evaluate_king_shield(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        let shield_offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dr, dc) in shield_offsets {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let shield_value = if is_gold_equivalent(piece.piece_type) {
                            (48, 28)
                        } else {
                            match piece.piece_type {
                                PieceType::PromotedBishop | PieceType::PromotedRook => (44, 26),
                                PieceType::Rook | PieceType::Bishop => (26, 18),
                                PieceType::Silver | PieceType::PromotedSilver => (28, 16),
                                PieceType::Pawn => (18, 10),
                                PieceType::Knight | PieceType::Lance => (14, 8),
                                _ => (12, 6),
                            }
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
    fn evaluate_pawn_cover(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> TaperedScore {
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
                    if piece.player == player {
                        if piece.piece_type == PieceType::Pawn {
                            mg_score += 25; // Pawn cover very important in middlegame
                            eg_score += 10; // Less critical in endgame
                        } else if is_gold_equivalent(piece.piece_type) {
                            mg_score += 32;
                            eg_score += 14;
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate enemy attackers near king
    fn evaluate_enemy_attackers(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check 3x3 area around king
        for row in (king_pos.row.saturating_sub(2))..=(king_pos.row + 2).min(8) {
            for col in (king_pos.col.saturating_sub(2))..=(king_pos.col + 2).min(8) {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player != player {
                        let threat_value = if is_gold_equivalent(piece.piece_type) {
                            (32, 22)
                        } else {
                            match piece.piece_type {
                                PieceType::Rook | PieceType::PromotedRook => (52, 32),
                                PieceType::Bishop | PieceType::PromotedBishop => (46, 28),
                                PieceType::Silver | PieceType::PromotedSilver => (26, 18),
                                PieceType::Knight | PieceType::Lance => (18, 12),
                                PieceType::Pawn => (12, 8),
                                _ => (16, 10),
                            }
                        };
                        mg_score += threat_value.0;
                        eg_score += threat_value.1;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn evaluate_hand_defense(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        let mut gold_like = captured_pieces.count(PieceType::Gold, player) as i32;
        let mut silver = captured_pieces.count(PieceType::Silver, player) as i32;
        let mut pawn = captured_pieces.count(PieceType::Pawn, player) as i32;
        let mut lance = captured_pieces.count(PieceType::Lance, player) as i32;
        let mut knight = captured_pieces.count(PieceType::Knight, player) as i32;

        const GUARD_TARGETS: &[GuardDropTarget] = &[
            GuardDropTarget {
                dr: -1,
                dc: 0,
                gold_bonus: (32, 16),
                silver_bonus: Some((18, 10)),
                pawn_bonus: Some((14, 8)),
                lance_bonus: Some((12, 6)),
                knight_bonus: None,
            },
            GuardDropTarget {
                dr: -1,
                dc: -1,
                gold_bonus: (26, 14),
                silver_bonus: Some((22, 12)),
                pawn_bonus: None,
                lance_bonus: None,
                knight_bonus: None,
            },
            GuardDropTarget {
                dr: -1,
                dc: 1,
                gold_bonus: (26, 14),
                silver_bonus: Some((22, 12)),
                pawn_bonus: None,
                lance_bonus: None,
                knight_bonus: None,
            },
            GuardDropTarget {
                dr: 0,
                dc: -1,
                gold_bonus: (20, 10),
                silver_bonus: Some((16, 8)),
                pawn_bonus: None,
                lance_bonus: None,
                knight_bonus: Some((10, 6)),
            },
            GuardDropTarget {
                dr: 0,
                dc: 1,
                gold_bonus: (20, 10),
                silver_bonus: Some((16, 8)),
                pawn_bonus: None,
                lance_bonus: None,
                knight_bonus: Some((10, 6)),
            },
            GuardDropTarget {
                dr: 1,
                dc: 0,
                gold_bonus: (12, 12),
                silver_bonus: None,
                pawn_bonus: None,
                lance_bonus: None,
                knight_bonus: Some((14, 10)),
            },
        ];

        for target in GUARD_TARGETS {
            if let Some(pos) = oriented_offset_to_actual(king_pos, player, target.dr, target.dc) {
                if board.is_square_occupied(pos) {
                    continue;
                }

                let mut best_kind = None;
                let mut best_bonus = (0, 0);

                if gold_like > 0 && target.gold_bonus.0 > best_bonus.0 {
                    best_kind = Some(HandPieceKind::GoldLike);
                    best_bonus = target.gold_bonus;
                }

                if silver > 0 {
                    if let Some(bonus) = target.silver_bonus {
                        if bonus.0 > best_bonus.0 {
                            best_kind = Some(HandPieceKind::Silver);
                            best_bonus = bonus;
                        }
                    }
                }

                if pawn > 0 {
                    if let Some(bonus) = target.pawn_bonus {
                        if self.can_drop_pawn_at(board, player, pos) && bonus.0 > best_bonus.0 {
                            best_kind = Some(HandPieceKind::Pawn);
                            best_bonus = bonus;
                        }
                    }
                }

                if lance > 0 {
                    if let Some(bonus) = target.lance_bonus {
                        if !is_illegal_lance_drop_rank(player, pos.row) && bonus.0 > best_bonus.0 {
                            best_kind = Some(HandPieceKind::Lance);
                            best_bonus = bonus;
                        }
                    }
                }

                if knight > 0 {
                    if let Some(bonus) = target.knight_bonus {
                        if !is_illegal_knight_drop_rank(player, pos.row) && bonus.0 > best_bonus.0 {
                            best_kind = Some(HandPieceKind::Knight);
                            best_bonus = bonus;
                        }
                    }
                }

                if let Some(kind) = best_kind {
                    mg_score += best_bonus.0;
                    eg_score += best_bonus.1;

                    match kind {
                        HandPieceKind::GoldLike => gold_like -= 1,
                        HandPieceKind::Silver => silver -= 1,
                        HandPieceKind::Pawn => pawn -= 1,
                        HandPieceKind::Lance => lance -= 1,
                        HandPieceKind::Knight => knight -= 1,
                    }
                }
            }
        }

        if gold_like > 0 {
            mg_score += gold_like * 6;
            eg_score += gold_like * 3;
        }

        if silver > 0 {
            mg_score += silver * 4;
            eg_score += silver * 2;
        }

        if pawn > 0 {
            mg_score += pawn * 2;
            eg_score += pawn;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn can_drop_pawn_at(&self, board: &BitboardBoard, player: Player, drop_pos: Position) -> bool {
        if board.is_square_occupied(drop_pos) {
            return false;
        }
        if self.column_has_unpromoted_pawn(board, player, drop_pos.col) {
            return false;
        }
        if is_illegal_pawn_drop_rank(player, drop_pos.row) {
            return false;
        }
        true
    }

    fn evaluate_castle_patterns(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut best: Option<TaperedScore> = None;

        best = Self::pick_better_score(
            best,
            self.assess_anaguma(board, king_pos, player, captured_pieces),
        );
        best = Self::pick_better_score(
            best,
            self.assess_yagura(board, king_pos, player, captured_pieces),
        );
        best = Self::pick_better_score(
            best,
            self.assess_mino(board, king_pos, player, captured_pieces),
        );

        best.unwrap_or_default()
    }

    /// Evaluate king exposure (open squares near king)
    fn evaluate_king_exposure(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        _player: Player,
    ) -> TaperedScore {
        let mut open_squares = 0;

        let offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

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

    fn assess_anaguma(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TaperedScore> {
        let (row, col) = oriented_coords(king_pos, player);
        if row != 8 {
            return None;
        }

        let mut best = None;

        if col >= 7 {
            let requirements = [
                CastleGuardRequirement {
                    dr: 0,
                    dc: -1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (6, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 0,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (5, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: -1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (6, 4),
                },
            ];
            best = Self::pick_better_score(
                best,
                self.score_castle_pattern(
                    board,
                    king_pos,
                    player,
                    captured_pieces,
                    &requirements,
                    (60, 40),
                    (46, 32),
                    (30, 20),
                ),
            );
        }

        if col <= 1 {
            let requirements = [
                CastleGuardRequirement {
                    dr: 0,
                    dc: 1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (6, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 0,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (5, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (6, 4),
                },
            ];
            best = Self::pick_better_score(
                best,
                self.score_castle_pattern(
                    board,
                    king_pos,
                    player,
                    captured_pieces,
                    &requirements,
                    (60, 40),
                    (46, 32),
                    (30, 20),
                ),
            );
        }

        best
    }

    fn assess_mino(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TaperedScore> {
        let (row, col) = oriented_coords(king_pos, player);
        if row < 7 {
            return None;
        }

        let mut best = None;

        if col >= 5 {
            let requirements = [
                CastleGuardRequirement {
                    dr: 0,
                    dc: -1,
                    accept_gold_like: true,
                    accept_silver: false,
                    drop_bonus: (5, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 0,
                    accept_gold_like: false,
                    accept_silver: true,
                    drop_bonus: (4, 3),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: -1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (4, 3),
                },
            ];
            best = Self::pick_better_score(
                best,
                self.score_castle_pattern(
                    board,
                    king_pos,
                    player,
                    captured_pieces,
                    &requirements,
                    (45, 26),
                    (34, 20),
                    (22, 14),
                ),
            );
        }

        if col <= 3 {
            let requirements = [
                CastleGuardRequirement {
                    dr: 0,
                    dc: 1,
                    accept_gold_like: true,
                    accept_silver: false,
                    drop_bonus: (5, 4),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 0,
                    accept_gold_like: false,
                    accept_silver: true,
                    drop_bonus: (4, 3),
                },
                CastleGuardRequirement {
                    dr: -1,
                    dc: 1,
                    accept_gold_like: true,
                    accept_silver: true,
                    drop_bonus: (4, 3),
                },
            ];
            best = Self::pick_better_score(
                best,
                self.score_castle_pattern(
                    board,
                    king_pos,
                    player,
                    captured_pieces,
                    &requirements,
                    (45, 26),
                    (34, 20),
                    (22, 14),
                ),
            );
        }

        best
    }

    fn assess_yagura(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TaperedScore> {
        let (row, _col) = oriented_coords(king_pos, player);
        if row != 8 {
            return None;
        }

        let mut best = None;

        let left_requirements = [
            CastleGuardRequirement {
                dr: 0,
                dc: -1,
                accept_gold_like: true,
                accept_silver: false,
                drop_bonus: (6, 4),
            },
            CastleGuardRequirement {
                dr: -1,
                dc: 0,
                accept_gold_like: false,
                accept_silver: true,
                drop_bonus: (4, 3),
            },
            CastleGuardRequirement {
                dr: -1,
                dc: -1,
                accept_gold_like: true,
                accept_silver: false,
                drop_bonus: (6, 4),
            },
        ];

        let right_requirements = [
            CastleGuardRequirement {
                dr: 0,
                dc: 1,
                accept_gold_like: true,
                accept_silver: false,
                drop_bonus: (6, 4),
            },
            CastleGuardRequirement {
                dr: -1,
                dc: 0,
                accept_gold_like: false,
                accept_silver: true,
                drop_bonus: (4, 3),
            },
            CastleGuardRequirement {
                dr: -1,
                dc: 1,
                accept_gold_like: true,
                accept_silver: false,
                drop_bonus: (6, 4),
            },
        ];

        best = Self::pick_better_score(
            best,
            self.score_castle_pattern(
                board,
                king_pos,
                player,
                captured_pieces,
                &left_requirements,
                (52, 32),
                (38, 24),
                (24, 16),
            ),
        );

        best = Self::pick_better_score(
            best,
            self.score_castle_pattern(
                board,
                king_pos,
                player,
                captured_pieces,
                &right_requirements,
                (52, 32),
                (38, 24),
                (24, 16),
            ),
        );

        best
    }

    fn score_castle_pattern(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
        requirements: &[CastleGuardRequirement],
        full_score: (i32, i32),
        partial_score: (i32, i32),
        near_score: (i32, i32),
    ) -> Option<TaperedScore> {
        let mut hand = HandCounts::new(captured_pieces, player);
        let mut on_board = 0;
        let mut drop_filled = 0;
        let mut drop_bonus = (0, 0);
        let total = requirements.len() as i32;

        for requirement in requirements {
            match self.satisfy_castle_guard(board, king_pos, player, requirement, &mut hand) {
                GuardState::OnBoard => on_board += 1,
                GuardState::FulfilledByDrop(mg, eg) => {
                    drop_filled += 1;
                    drop_bonus.0 += mg;
                    drop_bonus.1 += eg;
                }
                GuardState::Missing => {}
            }
        }

        let filled = on_board + drop_filled;

        if filled == total {
            if drop_filled == 0 {
                return Some(TaperedScore::new_tapered(full_score.0, full_score.1));
            }
            return Some(TaperedScore::new_tapered(
                partial_score.0 + drop_bonus.0,
                partial_score.1 + drop_bonus.1,
            ));
        }

        if filled == total - 1 && total >= 2 {
            return Some(TaperedScore::new_tapered(
                near_score.0 + drop_bonus.0,
                near_score.1 + drop_bonus.1,
            ));
        }

        if on_board >= total - 1 && drop_filled > 0 {
            return Some(TaperedScore::new_tapered(
                (near_score.0 / 2).max(0) + drop_bonus.0,
                (near_score.1 / 2).max(0) + drop_bonus.1,
            ));
        }

        None
    }

    fn satisfy_castle_guard(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        requirement: &CastleGuardRequirement,
        hand: &mut HandCounts,
    ) -> GuardState {
        if let Some(pos) =
            oriented_offset_to_actual(king_pos, player, requirement.dr, requirement.dc)
        {
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player {
                    if requirement.accept_gold_like && is_gold_equivalent(piece.piece_type) {
                        return GuardState::OnBoard;
                    }

                    if requirement.accept_silver
                        && (piece.piece_type == PieceType::Silver
                            || piece.piece_type == PieceType::PromotedSilver)
                    {
                        return GuardState::OnBoard;
                    }
                }
                return GuardState::Missing;
            } else {
                if requirement.accept_gold_like && hand.has_gold() {
                    hand.use_gold();
                    return GuardState::FulfilledByDrop(
                        requirement.drop_bonus.0,
                        requirement.drop_bonus.1,
                    );
                }

                if requirement.accept_silver && hand.has_silver() {
                    hand.use_silver();
                    return GuardState::FulfilledByDrop(
                        (requirement.drop_bonus.0 - 1).max(0),
                        (requirement.drop_bonus.1 - 1).max(0),
                    );
                }

                if requirement.accept_gold_like && hand.has_silver() {
                    hand.use_silver();
                    return GuardState::FulfilledByDrop(
                        (requirement.drop_bonus.0 - 2).max(0),
                        (requirement.drop_bonus.1 - 2).max(0),
                    );
                }
            }
        }

        GuardState::Missing
    }

    fn pick_better_score(
        current: Option<TaperedScore>,
        candidate: Option<TaperedScore>,
    ) -> Option<TaperedScore> {
        match (current, candidate) {
            (None, None) => None,
            (Some(existing), None) => Some(existing),
            (None, Some(new_score)) => Some(new_score),
            (Some(existing), Some(new_score)) => {
                if new_score.mg > existing.mg
                    || (new_score.mg == existing.mg && new_score.eg > existing.eg)
                {
                    Some(new_score)
                } else {
                    Some(existing)
                }
            }
        }
    }

    #[allow(dead_code)]
    fn oriented_piece(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
        dr: i8,
        dc: i8,
    ) -> Option<Piece> {
        oriented_offset_to_actual(king_pos, player, dr, dc)
            .and_then(|pos| board.get_piece(pos))
    }

    fn column_has_unpromoted_pawn(
        &self,
        board: &BitboardBoard,
        player: Player,
        column: u8,
    ) -> bool {
        for row in 0..9 {
            let pos = Position::new(row, column);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player && piece.piece_type == PieceType::Pawn {
                    return true;
                }
            }
        }
        false
    }

    fn enemy_can_drop_pawn_blocker(
        &self,
        board: &BitboardBoard,
        pawn_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        let opponent = player.opposite();
        if captured_pieces.count(PieceType::Pawn, opponent) == 0 {
            return false;
        }
        if self.column_has_unpromoted_pawn(board, opponent, pawn_pos.col) {
            return false;
        }

        let direction = if player == Player::Black { -1 } else { 1 };
        let front_row = pawn_pos.row as i8 + direction;
        if front_row < 0 || front_row >= 9 {
            return false;
        }
        if is_illegal_pawn_drop_rank(opponent, front_row as u8) {
            return false;
        }

        let front_pos = Position::new(front_row as u8, pawn_pos.col);
        !board.is_square_occupied(front_pos)
    }

    fn enemy_promoted_blocker(
        &self,
        board: &BitboardBoard,
        pawn_pos: Position,
        player: Player,
    ) -> bool {
        let opponent = player.opposite();
        let direction = if player == Player::Black { -1 } else { 1 };
        let mut row = pawn_pos.row as i8 + direction;

        while row >= 0 && row < 9 {
            let pos = Position::new(row as u8, pawn_pos.col);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == opponent {
                    if is_gold_equivalent(piece.piece_type) || is_promoted_major(piece.piece_type) {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
            row += direction;
        }

        false
    }

    fn enemy_can_drop_gold_wall(
        &self,
        board: &BitboardBoard,
        pawn_pos: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        let opponent = player.opposite();
        let gold_like = captured_pieces.count(PieceType::Gold, opponent);
        let silvers = captured_pieces.count(PieceType::Silver, opponent);
        let lances = captured_pieces.count(PieceType::Lance, opponent);
        let knights = captured_pieces.count(PieceType::Knight, opponent);
        if gold_like + silvers + lances + knights == 0 {
            return false;
        }

        let block_offsets = [(-1, 0), (-1, -1), (-1, 1)];
        for (dr, dc) in block_offsets {
            if let Some(pos) = oriented_offset_to_actual(pawn_pos, opponent, dr, dc) {
                if board.is_square_occupied(pos) {
                    continue;
                }
                if gold_like > 0 {
                    return true;
                }
                if silvers > 0 {
                    return true;
                }
                if lances > 0 && !is_illegal_lance_drop_rank(opponent, pos.row) {
                    return true;
                }
                if knights > 0 && !is_illegal_knight_drop_rank(opponent, pos.row) {
                    return true;
                }
            }
        }

        if lances > 0 {
            let direction = if player == Player::Black { -1 } else { 1 };
            let mut row = pawn_pos.row as i8 + direction;
            while row >= 0 && row < 9 {
                let pos = Position::new(row as u8, pawn_pos.col);
                if board.is_square_occupied(pos) {
                    break;
                }
                if !is_illegal_lance_drop_rank(opponent, pos.row) {
                    return true;
                }
                row += direction;
            }
        }

        false
    }

    // =======================================================================
    // PAWN STRUCTURE EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate pawn structure with phase-aware weights
    ///
    /// # Parameters
    /// - `skip_passed_pawn_evaluation`: If true, skips passed pawn evaluation to avoid double-counting
    ///   when endgame patterns are enabled (endgame patterns handle passed pawns with endgame-specific bonuses)
    pub fn evaluate_pawn_structure(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        skip_passed_pawn_evaluation: bool,
    ) -> TaperedScore {
        if !self.config.enable_pawn_structure {
            return TaperedScore::default();
        }

        self.stats.pawn_structure_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;
        let pawns = self.collect_pawns(board, player);

        if pawns.is_empty() {
            return TaperedScore::default();
        }

        // 1. Pawn chains (connected pawns)
        let chains = self.evaluate_pawn_chains(&pawns, player);
        mg_score += chains.mg;
        eg_score += chains.eg;

        // 1b. Potential chains supported by hand drops
        let hand_chain_support =
            self.evaluate_hand_supported_chains(board, &pawns, player, captured_pieces);
        mg_score += hand_chain_support.mg;
        eg_score += hand_chain_support.eg;

        // 2. Advanced pawns
        let advancement = self.evaluate_pawn_advancement(&pawns, player);
        mg_score += advancement.mg;
        eg_score += advancement.eg;

        // 3. Isolated pawns
        let isolation = self.evaluate_pawn_isolation(board, &pawns, player, captured_pieces);
        mg_score += isolation.mg;
        eg_score += isolation.eg;

        // 4. Passed pawns (no enemy pawns in front)
        // Skip if endgame patterns are handling passed pawns to avoid double-counting
        if !skip_passed_pawn_evaluation {
            let passed = self.evaluate_passed_pawns(board, &pawns, player, captured_pieces);
            mg_score += passed.mg;
            eg_score += passed.eg;
        }

        // 5. Doubled pawns (same file)
        let doubled = self.evaluate_doubled_pawns(&pawns);
        mg_score += doubled.mg;
        eg_score += doubled.eg;

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Collect all pawns for a player
    fn collect_pawns(&mut self, board: &BitboardBoard, player: Player) -> Vec<Position> {
        self.ensure_inputs(board);
        let idx = Self::player_index(player);
        self.inputs_cache.pawns[idx].clone()
    }

    /// Evaluate pawn chains (adjacent pawns)
    fn evaluate_pawn_chains(&self, pawns: &[Position], player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        if pawns.len() < 2 {
            return TaperedScore::default();
        }

        let oriented: Vec<(i8, i8)> = pawns
            .iter()
            .map(|pawn| oriented_coords(*pawn, player))
            .collect();

        for i in 0..pawns.len() {
            for j in i + 1..pawns.len() {
                let (r1, c1) = oriented[i];
                let (r2, c2) = oriented[j];

                let dr = r1 - r2;
                let dc = c1 - c2;

                let abs_dc = dc.abs();
                let abs_dr = dr.abs();

                if abs_dr == 0 && abs_dc == 1 {
                    mg_score += 18;
                    eg_score += 12;
                } else if abs_dr == 0 && abs_dc == 2 {
                    mg_score += 10;
                    eg_score += 8;
                }

                if dr == 1 && abs_dc <= 1 {
                    mg_score += 24;
                    eg_score += 18;
                } else if dr == -1 && abs_dc <= 1 {
                    mg_score += 16;
                    eg_score += 12;
                }

                if abs_dr == 1 && dc == 0 {
                    mg_score += 14;
                    eg_score += 10;
                }

                if abs_dr == 1 && abs_dc == 1 {
                    if dr == 1 {
                        mg_score += 22;
                        eg_score += 16;
                    } else {
                        mg_score += 14;
                        eg_score += 10;
                    }
                }

                if (2..=6).contains(&c1) && (2..=6).contains(&c2) {
                    mg_score += 4;
                    eg_score += 2;
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn evaluate_hand_supported_chains(
        &self,
        board: &BitboardBoard,
        pawns: &[Position],
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let pawn_available = captured_pieces.count(PieceType::Pawn, player) as i32;
        let gold_available = captured_pieces.count(PieceType::Gold, player) as i32;
        if pawn_available == 0 && gold_available == 0 {
            return TaperedScore::default();
        }

        let oriented: Vec<(i8, i8)> = pawns
            .iter()
            .map(|pawn| oriented_coords(*pawn, player))
            .collect();

        let chain_offsets = [
            (0i8, -1i8),
            (0, 1),
            (-1, 0),
            (1, 0),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ];

        let mut pawn_marks = [false; 81];
        let mut gold_marks = [false; 81];
        let mut pawn_unique = 0i32;
        let mut gold_unique = 0i32;

        for (idx, pawn) in pawns.iter().enumerate() {
            let oriented_pawn = oriented[idx];
            for (dr, dc) in chain_offsets {
                let new_row = pawn.row as i8 + dr;
                let new_col = pawn.col as i8 + dc;
                if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                    continue;
                }

                let pos = Position::new(new_row as u8, new_col as u8);
                let board_index = (pos.row as usize) * 9 + pos.col as usize;
                let drop_oriented = (oriented_pawn.0 + dr, oriented_pawn.1 + dc);

                if pawn_available > 0
                    && self.can_drop_pawn_at(board, player, pos)
                    && self.drop_creates_chain_oriented(&oriented, drop_oriented)
                {
                    if !pawn_marks[board_index] {
                        pawn_marks[board_index] = true;
                        pawn_unique += 1;
                    }
                }

                if gold_available > 0
                    && !board.is_square_occupied(pos)
                    && self.drop_creates_chain_oriented(&oriented, drop_oriented)
                {
                    if !gold_marks[board_index] {
                        gold_marks[board_index] = true;
                        gold_unique += 1;
                    }
                }
            }
        }

        let pawn_usable = pawn_unique.min(pawn_available).max(0);
        let gold_usable = gold_unique.min(gold_available).max(0);

        let mg = pawn_usable * 12 + gold_usable * 9;
        let eg = pawn_usable * 8 + gold_usable * 6;
        TaperedScore::new_tapered(mg, eg)
    }

    fn drop_creates_chain_oriented(
        &self,
        oriented_pawns: &[(i8, i8)],
        drop_oriented: (i8, i8),
    ) -> bool {
        let (drop_r, drop_c) = drop_oriented;
        for (pawn_r, pawn_c) in oriented_pawns {
            let dr = pawn_r - drop_r;
            let dc = pawn_c - drop_c;

            if dr == 0 && dc.abs() == 1 {
                return true;
            }
            if dr.abs() == 1 && dc == 0 {
                return true;
            }
            if dr == 1 && dc.abs() <= 1 {
                return true;
            }
            if dr.abs() == 0 && dc.abs() == 2 {
                return true;
            }
        }
        false
    }

    /// Evaluate pawn advancement
    fn evaluate_pawn_advancement(&self, pawns: &[Position], player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;

        const BLACK_MG_TABLE: [i32; 9] = [60, 52, 44, 34, 24, 16, 8, 2, 0];
        const BLACK_EG_TABLE: [i32; 9] = [92, 78, 62, 46, 32, 20, 10, 3, 0];

        for pawn in pawns {
            let idx = if player == Player::Black {
                pawn.row as usize
            } else {
                (8 - pawn.row) as usize
            };
            mg_score += BLACK_MG_TABLE[idx];
            eg_score += BLACK_EG_TABLE[idx];
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate isolated pawns
    fn evaluate_pawn_isolation(
        &self,
        board: &BitboardBoard,
        pawns: &[Position],
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        let mut isolated_count: i32 = 0;

        for pawn in pawns {
            if self.is_pawn_isolated(board, *pawn, player) {
                isolated_count += 1;
            }
        }

        let mitigation = (captured_pieces.count(PieceType::Pawn, player)
            + captured_pieces.count(PieceType::Gold, player)) as i32;
        let effective_isolated = (isolated_count - mitigation).max(0);

        let mg_penalty = effective_isolated * 18;
        let eg_penalty = effective_isolated * 30;

        TaperedScore::new_tapered(-mg_penalty, -eg_penalty)
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
    fn evaluate_passed_pawns(
        &self,
        board: &BitboardBoard,
        pawns: &[Position],
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
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

                let mut bonus_mg = (advancement * advancement) as i32 * 5;
                let mut bonus_eg = (advancement * advancement) as i32 * 12;

                if self.enemy_can_drop_pawn_blocker(board, *pawn, player, captured_pieces) {
                    bonus_mg = (bonus_mg * 2) / 3;
                    bonus_eg = (bonus_eg * 2) / 3;
                }

                if self.enemy_promoted_blocker(board, *pawn, player) {
                    bonus_mg = (bonus_mg * 3) / 4;
                    bonus_eg = (bonus_eg * 3) / 4;
                }

                if self.enemy_can_drop_gold_wall(board, *pawn, player, captured_pieces) {
                    bonus_mg = (bonus_mg * 2) / 3;
                    bonus_eg = (bonus_eg * 2) / 3;
                }

                let enemy_gold = captured_pieces.count(PieceType::Gold, player.opposite()) as i32;
                if enemy_gold > 0 {
                    let reduction = enemy_gold.min(2) * 6;
                    bonus_mg -= reduction;
                    bonus_eg -= reduction * 2;
                }

                bonus_mg = bonus_mg.max(0);
                bonus_eg = bonus_eg.max(0);

                mg_score += bonus_mg;
                eg_score += bonus_eg;
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
                    if col_offset == 0 {
                        return false;
                    } else if piece.player != player
                        && (piece.piece_type == PieceType::Pawn
                            || is_gold_equivalent(piece.piece_type))
                    {
                        return false;
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

        let mut severe_penalty = 0;
        for count in file_counts {
            if count >= 2 {
                let extras = count - 1;
                doubled_count += extras; // Each extra pawn is doubled
                severe_penalty += extras;
            }
        }

        let mut mg = -(doubled_count * 12);
        let mut eg = -(doubled_count * 18);

        if severe_penalty > 0 {
            mg -= severe_penalty * 60;
            eg -= severe_penalty * 60;
        }

        TaperedScore::new_tapered(mg, eg)
    }

    // =======================================================================
    // PIECE MOBILITY EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate piece mobility with phase-aware weights
    ///
    /// Mobility is more important in endgame when pieces need room to maneuver.
    /// This implementation includes:
    /// - Weighted mobility scores by piece type
    /// - Restricted piece penalties
    /// - Central mobility bonuses
    /// - Attack move bonuses
    pub fn evaluate_mobility(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        if !self.config.enable_mobility {
            return TaperedScore::default();
        }

        self.stats.mobility_evals += 1;

        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        let mut piece_stats = vec![PieceMobilityStats::default(); MOBILITY_BOARD_AREA];
        let mut drop_stats = [DropMobilityStats::default(); PieceType::COUNT];

        for mv in &legal_moves {
            if let Some(from) = mv.from {
                let idx = index_for_position(from);
                let stats = &mut piece_stats[idx];
                stats.total_moves += 1;
                if self.is_central_square(mv.to) {
                    stats.central_moves += 1;
                }
                if mv.is_capture {
                    stats.attack_moves += 1;
                }
            } else {
                let stats = &mut drop_stats[mv.piece_type.as_index()];
                stats.total_moves += 1;
                if self.is_central_square(mv.to) {
                    stats.central_moves += 1;
                }
            }
        }

        let mut total = TaperedScore::default();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let stats = &piece_stats[index_for_position(pos)];
                        total += self.evaluate_piece_mobility_from_stats(piece.piece_type, stats);
                    }
                }
            }
        }

        for piece_type in ALL_PIECE_TYPES.iter().copied() {
            let stats = &drop_stats[piece_type.as_index()];
            total += self.evaluate_drop_mobility(piece_type, stats);
        }

        total
    }

    fn evaluate_piece_mobility_from_stats(
        &self,
        piece_type: PieceType,
        stats: &PieceMobilityStats,
    ) -> TaperedScore {
        let move_count = stats.total_moves;
        let mobility_weight = self.get_mobility_weight(piece_type);

        let mut mg_score = move_count * mobility_weight.0;
        let mut eg_score = move_count * mobility_weight.1;

        if move_count <= 2 {
            let restriction_penalty = self.get_restriction_penalty(piece_type);
            mg_score -= restriction_penalty.0;
            eg_score -= restriction_penalty.1;
        }

        if stats.central_moves > 0 {
            let central_bonus = self.get_central_mobility_bonus(piece_type);
            mg_score += stats.central_moves * central_bonus.0;
            eg_score += stats.central_moves * central_bonus.1;
        }

        if stats.attack_moves > 0 {
            mg_score += stats.attack_moves * 4;
            eg_score += stats.attack_moves * 3;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn evaluate_drop_mobility(
        &self,
        piece_type: PieceType,
        stats: &DropMobilityStats,
    ) -> TaperedScore {
        if stats.total_moves == 0 {
            return TaperedScore::default();
        }

        let drop_weight = self.get_drop_mobility_weight(piece_type);
        let mut mg_score = stats.total_moves * drop_weight.0;
        let mut eg_score = stats.total_moves * drop_weight.1;

        if stats.central_moves > 0 {
            let central_bonus = self.get_drop_central_bonus(piece_type);
            mg_score += stats.central_moves * central_bonus.0;
            eg_score += stats.central_moves * central_bonus.1;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Get mobility weight for piece type (mg, eg)
    fn get_mobility_weight(&self, piece_type: PieceType) -> (i32, i32) {
        match piece_type {
            // Major pieces - high mobility value
            PieceType::Rook => (5, 7),
            PieceType::PromotedRook => (6, 8),
            PieceType::Bishop => (4, 6),
            PieceType::PromotedBishop => (5, 7),

            // Minor pieces - moderate mobility value
            PieceType::Gold => (2, 3),
            PieceType::Silver => (2, 3),
            PieceType::Knight => (3, 3),
            PieceType::Lance => (2, 2),

            // Promoted minor pieces
            PieceType::PromotedPawn => (3, 4),
            PieceType::PromotedLance => (3, 4),
            PieceType::PromotedKnight => (3, 4),
            PieceType::PromotedSilver => (3, 4),

            // Pawns and King - low mobility value
            PieceType::Pawn => (1, 1),
            PieceType::King => (1, 2), // King mobility important in endgame
        }
    }

    /// Get restriction penalty for piece type (mg, eg)
    fn get_restriction_penalty(&self, piece_type: PieceType) -> (i32, i32) {
        match piece_type {
            // Major pieces suffer most from restriction
            PieceType::Rook | PieceType::PromotedRook => (18, 24),
            PieceType::Bishop | PieceType::PromotedBishop => (16, 22),

            // Minor pieces moderate penalty
            PieceType::Gold | PieceType::Silver => (6, 8),
            PieceType::Knight | PieceType::Lance => (7, 9),

            // Promoted minor pieces
            PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => (6, 8),

            // Pawns and King less affected
            PieceType::Pawn => (3, 4),
            PieceType::King => (4, 6),
        }
    }

    /// Get central mobility bonus for piece type (mg, eg)
    fn get_central_mobility_bonus(&self, piece_type: PieceType) -> (i32, i32) {
        match piece_type {
            // Major pieces benefit most from central mobility
            PieceType::Rook | PieceType::PromotedRook => (4, 3),
            PieceType::Bishop | PieceType::PromotedBishop => (3, 3),

            // Knights especially strong in center
            PieceType::Knight => (4, 2),

            // Other pieces moderate bonus
            PieceType::Gold | PieceType::Silver => (2, 2),
            PieceType::Lance => (1, 1),

            // Promoted pieces
            PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => (3, 2),

            // Pawns and King minimal bonus
            PieceType::Pawn => (1, 1),
            PieceType::King => (1, 1), // King centralization good in endgame
        }
    }

    fn get_drop_mobility_weight(&self, piece_type: PieceType) -> (i32, i32) {
        let base = self.get_mobility_weight(piece_type);
        let mg = (base.0 + 1) / 2;
        let eg = (base.1 + 1) / 2;
        (mg.max(1), eg.max(1))
    }

    fn get_drop_central_bonus(&self, piece_type: PieceType) -> (i32, i32) {
        let base = self.get_central_mobility_bonus(piece_type);
        let mg = if base.0 == 0 { 0 } else { (base.0 + 1) / 2 };
        let eg = if base.1 == 0 { 0 } else { (base.1 + 1) / 2 };
        (mg, eg)
    }

    /// Check if a square is in the center (3x3 center area)
    fn is_central_square(&self, pos: Position) -> bool {
        pos.row >= 3 && pos.row <= 5 && pos.col >= 3 && pos.col <= 5
    }

    fn compute_control_map(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> [ControlStrength; MOBILITY_BOARD_AREA] {
        let mut control = [ControlStrength::default(); MOBILITY_BOARD_AREA];

        let moves = self.move_generator.generate_all_piece_moves(board, player);

        for mv in moves {
            if mv.is_drop() || mv.is_promotion {
                continue;
            }
            let idx = index_for_position(mv.to);
            let weight = self.get_center_control_value(mv.piece_type);
            control[idx].mg = control[idx].mg.max(weight.mg);
            control[idx].eg = control[idx].eg.max(weight.eg);
        }

        control
    }

    fn castle_anchor_squares(player: Player) -> &'static [(u8, u8)] {
        const BLACK_ANCHORS: &[(u8, u8)] = &[(7, 1), (7, 7), (6, 2), (6, 6), (8, 2), (8, 6)];
        const WHITE_ANCHORS: &[(u8, u8)] = &[(1, 7), (1, 1), (2, 6), (2, 2), (0, 6), (0, 2)];

        match player {
            Player::Black => BLACK_ANCHORS,
            Player::White => WHITE_ANCHORS,
        }
    }

    fn castle_anchor_occupant_bonus(piece: &Piece, player: Player) -> (i32, i32) {
        let magnitude = if is_gold_equivalent(piece.piece_type) {
            (18, 10)
        } else {
            match piece.piece_type {
                PieceType::Silver | PieceType::PromotedSilver => (14, 8),
                PieceType::King => (12, 6),
                PieceType::Knight | PieceType::PromotedKnight => (10, 6),
                _ => (6, 4),
            }
        };

        if piece.player == player {
            magnitude
        } else {
            (-magnitude.0, -magnitude.1)
        }
    }

    // =======================================================================
    // CENTER CONTROL EVALUATION BY PHASE
    // =======================================================================

    /// Evaluate center control with phase-aware weights
    ///
    /// Center control is more important in opening/middlegame.
    ///
    /// # Parameters
    /// - `skip_center_control`: If true, skips center control evaluation (optional, for future use
    ///   when positional patterns handle center control to avoid double-counting)
    pub fn evaluate_center_control(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        skip_center_control: bool,
    ) -> TaperedScore {
        if skip_center_control {
            return TaperedScore::default();
        }
        if !self.config.enable_center_control {
            return TaperedScore::default();
        }

        self.stats.center_control_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        let player_control = self.compute_control_map(board, player);
        let opponent_control = self.compute_control_map(board, player.opposite());

        for row in 3..=5 {
            for col in 3..=5 {
                let idx = index_for_position(Position::new(row, col));
                let diff_mg = player_control[idx].mg - opponent_control[idx].mg;
                let diff_eg = player_control[idx].eg - opponent_control[idx].eg;
                mg_score += diff_mg;
                eg_score += diff_eg;
            }
        }

        for row in 2..=6 {
            for col in 2..=6 {
                if row >= 3 && row <= 5 && col >= 3 && col <= 5 {
                    continue;
                }

                let idx = index_for_position(Position::new(row, col));
                let diff_mg = player_control[idx].mg - opponent_control[idx].mg;
                let diff_eg = player_control[idx].eg - opponent_control[idx].eg;
                mg_score += (diff_mg * 2) / 3;
                eg_score += (diff_eg * 2) / 3;
            }
        }

        const EDGE_COLUMNS: [u8; 2] = [0, 8];
        for col in EDGE_COLUMNS {
            for row in 2..=6 {
                let idx = index_for_position(Position::new(row, col));
                let diff_mg = player_control[idx].mg - opponent_control[idx].mg;
                let diff_eg = player_control[idx].eg - opponent_control[idx].eg;
                mg_score += diff_mg / 2;
                eg_score += diff_eg / 2;
            }
        }

        for &(row, col) in Self::castle_anchor_squares(player) {
            let pos = Position::new(row, col);
            let idx = index_for_position(pos);
            let diff_mg = player_control[idx].mg - opponent_control[idx].mg;
            let diff_eg = player_control[idx].eg - opponent_control[idx].eg;
            mg_score += diff_mg / 2;
            eg_score += diff_eg / 2;

            if let Some(piece) = board.get_piece(pos) {
                let bonus = Self::castle_anchor_occupant_bonus(&piece, player);
                mg_score += bonus.0;
                eg_score += bonus.1;
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
    /// Evaluate piece development
    ///
    /// # Parameters
    /// - `skip_development`: If true, skips development evaluation (optional, for coordination with
    ///   opening_principles to avoid double-counting when opening_principles is enabled in opening phase)
    pub fn evaluate_development(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        skip_development: bool,
    ) -> TaperedScore {
        if skip_development {
            return TaperedScore::default();
        }
        if !self.config.enable_development {
            return TaperedScore::default();
        }

        self.stats.development_evals += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let contribution =
                            self.get_development_contribution(piece.piece_type, pos, player);
                        mg_score += contribution.mg;
                        eg_score += contribution.eg;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    fn get_development_contribution(
        &self,
        piece_type: PieceType,
        pos: Position,
        player: Player,
    ) -> TaperedScore {
        let (oriented_row, _) = oriented_coords(pos, player);
        let advancement = (8 - oriented_row as i32).max(0);

        match piece_type {
            PieceType::Rook => {
                if advancement == 0 {
                    TaperedScore::new_tapered(-24, -6)
                } else {
                    TaperedScore::new_tapered(advancement * 10, advancement * 4)
                }
            }
            PieceType::Bishop => {
                if advancement == 0 {
                    TaperedScore::new_tapered(-20, -6)
                } else {
                    TaperedScore::new_tapered(advancement * 9, advancement * 4)
                }
            }
            PieceType::Gold => {
                if advancement == 0 {
                    TaperedScore::new_tapered(-18, -6)
                } else {
                    TaperedScore::new_tapered(advancement * 6, advancement * 3)
                }
            }
            PieceType::Silver => {
                if advancement == 0 {
                    TaperedScore::new_tapered(-20, -8)
                } else {
                    TaperedScore::new_tapered((advancement * 7).min(24), advancement * 3)
                }
            }
            PieceType::Knight => {
                if oriented_row >= 7 {
                    TaperedScore::new_tapered(-24, -8)
                } else {
                    let bonus = (advancement.max(2) - 1) * 12;
                    TaperedScore::new_tapered(bonus, bonus / 3)
                }
            }
            PieceType::Lance => {
                if advancement == 0 {
                    TaperedScore::new_tapered(-12, -4)
                } else {
                    TaperedScore::new_tapered(advancement * 5, advancement * 2)
                }
            }
            PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => {
                if oriented_row >= 6 {
                    let retreat = oriented_row as i32 - 5;
                    TaperedScore::new_tapered(-retreat * 8, -retreat * 4)
                } else {
                    TaperedScore::new_tapered(12, 6)
                }
            }
            PieceType::PromotedBishop | PieceType::PromotedRook => {
                if oriented_row >= 6 {
                    let retreat = oriented_row as i32 - 5;
                    TaperedScore::new_tapered(-retreat * 10, -retreat * 6)
                } else {
                    TaperedScore::new_tapered(advancement * 6, advancement * 4)
                }
            }
            _ => TaperedScore::default(),
        }
    }

    // =======================================================================
    // HELPER METHODS
    // =======================================================================

    /// Find king position for a player
    fn find_king_position(&mut self, board: &BitboardBoard, player: Player) -> Option<Position> {
        self.ensure_inputs(board);
        let idx = Self::player_index(player);
        self.inputs_cache.king_positions[idx]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {

    #[test]
    fn test_position_feature_evaluator_creation() {
        let evaluator = PositionFeatureEvaluator::new();
        assert!(evaluator.config().enable_king_safety);
    }

    #[test]
    fn test_king_safety_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate_king_safety(&board, Player::Black, &captured_pieces);

        // Starting position should have positive king safety
        assert!(score.mg > 0 || score.eg > 0);
    }

    #[test]
    fn test_pawn_structure_evaluation() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate_pawn_structure(&board, Player::Black, &captured_pieces);

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

        let score = evaluator.evaluate_development(&board, Player::Black, false);

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
        let mut evaluator = PositionFeatureEvaluator::new();
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

        evaluator.evaluate_king_safety(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().king_safety_evals, 1);

        evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().mobility_evals, 1);
    }

    #[test]
    fn test_reset_statistics() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate_king_safety(&board, Player::Black, &captured_pieces);
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
        let development = evaluator.evaluate_development(&board, Player::Black, false);
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
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();

        let black_king = evaluator.find_king_position(&board, Player::Black);
        assert!(black_king.is_some());

        let white_king = evaluator.find_king_position(&board, Player::White);
        assert!(white_king.is_some());
    }

    // ===================================================================
    // MOBILITY PATTERN TESTS (Task 1.5)
    // ===================================================================

    #[test]
    fn test_mobility_weights() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test that major pieces have higher mobility weights
        let rook_weight = evaluator.get_mobility_weight(PieceType::Rook);
        let pawn_weight = evaluator.get_mobility_weight(PieceType::Pawn);

        assert!(rook_weight.0 > pawn_weight.0); // Middlegame
        assert!(rook_weight.1 > pawn_weight.1); // Endgame

        // Test that promoted rook has higher weight than regular rook
        let promoted_rook = evaluator.get_mobility_weight(PieceType::PromotedRook);
        assert!(promoted_rook.0 >= rook_weight.0);
        assert!(promoted_rook.1 >= rook_weight.1);
    }

    #[test]
    fn test_restriction_penalties() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test that major pieces have higher restriction penalties
        let rook_penalty = evaluator.get_restriction_penalty(PieceType::Rook);
        let pawn_penalty = evaluator.get_restriction_penalty(PieceType::Pawn);

        assert!(rook_penalty.0 > pawn_penalty.0); // Middlegame
        assert!(rook_penalty.1 > pawn_penalty.1); // Endgame

        // Penalties should be significant
        assert!(rook_penalty.0 >= 15);
        assert!(rook_penalty.1 >= 20);
    }

    #[test]
    fn test_central_mobility_bonus() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test that knights get highest central bonus
        let knight_bonus = evaluator.get_central_mobility_bonus(PieceType::Knight);
        let pawn_bonus = evaluator.get_central_mobility_bonus(PieceType::Pawn);

        assert!(knight_bonus.0 > pawn_bonus.0);

        // Test that major pieces get good central bonuses
        let rook_bonus = evaluator.get_central_mobility_bonus(PieceType::Rook);
        assert!(rook_bonus.0 >= 2);
    }

    #[test]
    fn test_is_central_square() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test center squares
        assert!(evaluator.is_central_square(Position::new(4, 4))); // Dead center
        assert!(evaluator.is_central_square(Position::new(3, 3))); // Corner of center
        assert!(evaluator.is_central_square(Position::new(5, 5))); // Corner of center

        // Test non-center squares
        assert!(!evaluator.is_central_square(Position::new(0, 0))); // Corner
        assert!(!evaluator.is_central_square(Position::new(2, 2))); // Just outside
        assert!(!evaluator.is_central_square(Position::new(6, 6))); // Just outside
    }

    #[test]
    fn test_piece_mobility_evaluation() {
        let evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test mobility evaluation for a specific piece
        // In starting position, pieces should have limited mobility
        let pos = Position::new(8, 1); // Black's knight
        let mobility = evaluator.evaluate_piece_mobility(
            &board,
            pos,
            PieceType::Knight,
            Player::Black,
            &captured_pieces,
        );

        // Starting knight has limited moves
        assert!(mobility.mg >= 0);
        assert!(mobility.eg >= 0);
    }

    #[test]
    fn test_mobility_by_piece_type() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test overall mobility evaluation
        let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);

        // Starting position should have positive mobility
        assert!(mobility.mg > 0);
        assert!(mobility.eg > 0);

        // Endgame should value mobility more
        assert!(mobility.eg >= mobility.mg);
    }

    #[test]
    fn test_restricted_piece_detection() {
        let evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::empty();
        let captured_pieces = CapturedPieces::new();

        // On empty board, pieces should have high mobility (not restricted)
        let pos = Position::new(4, 4); // Center
        let mobility = evaluator.evaluate_piece_mobility(
            &board,
            pos,
            PieceType::Rook,
            Player::Black,
            &captured_pieces,
        );

        // Rook in center of empty board should have excellent mobility
        assert!(mobility.mg > 0);
        assert!(mobility.eg > 0);
    }

    #[test]
    fn test_mobility_weights_all_pieces() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test that all piece types return valid weights
        let piece_types = [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::King,
            PieceType::PromotedPawn,
            PieceType::PromotedLance,
            PieceType::PromotedKnight,
            PieceType::PromotedSilver,
            PieceType::PromotedBishop,
            PieceType::PromotedRook,
        ];

        for piece_type in piece_types {
            let weight = evaluator.get_mobility_weight(piece_type);

            // All weights should be positive
            assert!(
                weight.0 > 0,
                "MG weight for {:?} should be positive",
                piece_type
            );
            assert!(
                weight.1 > 0,
                "EG weight for {:?} should be positive",
                piece_type
            );

            // Endgame weight should be >= middlegame weight
            assert!(
                weight.1 >= weight.0,
                "EG weight should be >= MG weight for {:?}",
                piece_type
            );
        }
    }

    #[test]
    fn test_mobility_phase_difference() {
        let mut evaluator = PositionFeatureEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);

        // Mobility should be more valuable in endgame
        assert!(
            mobility.eg > mobility.mg,
            "Endgame mobility ({}) should be greater than middlegame ({})",
            mobility.eg,
            mobility.mg
        );
    }

    #[test]
    fn test_central_mobility_detection() {
        let evaluator = PositionFeatureEvaluator::new();

        // Test all central squares
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                assert!(
                    evaluator.is_central_square(pos),
                    "Position ({}, {}) should be central",
                    row,
                    col
                );
            }
        }

        // Test edge squares are not central
        assert!(!evaluator.is_central_square(Position::new(0, 0)));
        assert!(!evaluator.is_central_square(Position::new(8, 8)));
        assert!(!evaluator.is_central_square(Position::new(2, 4)));
        assert!(!evaluator.is_central_square(Position::new(6, 4)));
    }
}
