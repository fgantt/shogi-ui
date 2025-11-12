use crate::bitboards::*;
use crate::evaluation::castle_geometry::{
    CastlePieceClass, CastlePieceRole, RelativeOffset, BUFFER_RING, FORWARD_SHIELD_ARC,
    KING_ZONE_RING, PAWN_WALL_ARC,
};
use crate::evaluation::patterns::*;
use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub use crate::evaluation::castle_geometry::{
    exact, mirror_descriptors, CastlePieceDescriptor, GOLD_FAMILY, KNIGHT_FAMILY, LANCE_FAMILY,
    PAWN_WALL_FAMILY, SILVER_FAMILY,
};

const CACHE_MAX_ENTRIES: usize = 50;

#[derive(Debug, Clone, Copy)]
pub struct CastleEvaluation {
    pub matched_pattern: Option<&'static str>,
    pub variant_id: Option<&'static str>,
    pub quality: f32,
    pub coverage_ratio: f32,
    pub pattern_coverage_ratio: f32,
    pub zone_coverage_ratio: f32,
    pub primary_ratio: f32,
    pub secondary_ratio: f32,
    pub buffer_ratio: f32,
    pub pawn_shield_ratio: f32,
    pub pattern_shield_ratio: f32,
    pub zone_shield_ratio: f32,
    pub zone_forward_ratio: f32,
    pub zone_pawn_wall_ratio: f32,
    pub infiltration_ratio: f32,
    pub missing_required: usize,
    pub missing_optional: usize,
    pub missing_primary: usize,
    pub missing_shield: usize,
    pub missing_secondary: usize,
    pub missing_buffer: usize,
    pub matched_pieces: usize,
    pub total_pieces: usize,
    pub required_ratio: f32,
    pub base_score: TaperedScore,
}

impl Default for CastleEvaluation {
    fn default() -> Self {
        Self {
            matched_pattern: None,
            variant_id: None,
            quality: 0.0,
            coverage_ratio: 0.0,
            pattern_coverage_ratio: 0.0,
            zone_coverage_ratio: 0.0,
            primary_ratio: 0.0,
            secondary_ratio: 0.0,
            buffer_ratio: 0.0,
            pawn_shield_ratio: 0.0,
            pattern_shield_ratio: 0.0,
            zone_shield_ratio: 0.0,
            zone_forward_ratio: 0.0,
            zone_pawn_wall_ratio: 0.0,
            infiltration_ratio: 0.0,
            missing_required: 0,
            missing_optional: 0,
            missing_primary: 0,
            missing_shield: 0,
            missing_secondary: 0,
            missing_buffer: 0,
            matched_pieces: 0,
            total_pieces: 1,
            required_ratio: 0.0,
            base_score: TaperedScore::default(),
        }
    }
}

impl CastleEvaluation {
    pub fn score(self) -> TaperedScore {
        self.base_score
    }

    fn is_better_than(&self, other: &CastleEvaluation) -> bool {
        const EPS: f32 = 1e-5;
        if (self.quality - other.quality).abs() > EPS {
            return self.quality > other.quality;
        }
        if (self.zone_coverage_ratio - other.zone_coverage_ratio).abs() > EPS {
            return self.zone_coverage_ratio > other.zone_coverage_ratio;
        }
        if (self.coverage_ratio - other.coverage_ratio).abs() > EPS {
            return self.coverage_ratio > other.coverage_ratio;
        }
        if self.matched_pieces != other.matched_pieces {
            return self.matched_pieces > other.matched_pieces;
        }
        if self.missing_required != other.missing_required {
            return self.missing_required < other.missing_required;
        }
        if (self.infiltration_ratio - other.infiltration_ratio).abs() > EPS {
            return self.infiltration_ratio < other.infiltration_ratio;
        }
        self.missing_optional < other.missing_optional
    }
}

#[derive(Clone, Copy, Debug)]
struct CachedEvaluation {
    pattern_index: usize,
    variant_index: usize,
    evaluation: CastleEvaluation,
}

pub struct CastleRecognizer {
    patterns: Vec<CastlePattern>,
    pattern_cache: RefCell<HashMap<(u64, Player, Position), Option<CachedEvaluation>>>,
    early_termination_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct CastlePattern {
    pub name: &'static str,
    pub variants: Vec<CastleVariant>,
    pub score: TaperedScore,
    pub flexibility: u8,
}

#[derive(Debug, Clone)]
pub struct CastleVariant {
    pub id: &'static str,
    pub pieces: Vec<CastlePiece>,
}

#[derive(Debug, Clone)]
pub struct CastlePiece {
    pub class: CastlePieceClass,
    pub offset: RelativeOffset,
    pub required: bool,
    pub weight: u8,
    pub role: CastlePieceRole,
}

impl CastleRecognizer {
    pub fn new() -> Self {
        Self {
            patterns: vec![get_mino_castle(), get_anaguma_castle(), get_yagura_castle()],
            pattern_cache: RefCell::new(HashMap::new()),
            early_termination_threshold: 0.8,
        }
    }

    pub fn recognize_castle(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> Option<&CastlePattern> {
        self.patterns
            .iter()
            .find(|pattern| self.matches_pattern(board, player, king_pos, pattern))
    }

    pub fn evaluate_castle(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> CastleEvaluation {
        let board_hash = self.get_board_hash(board);
        if let Some(cached) = self
            .pattern_cache
            .borrow()
            .get(&(board_hash, player, king_pos))
        {
            if let Some(entry) = cached {
                return entry.evaluation;
            } else {
                return CastleEvaluation::default();
            }
        }

        let mut best_entry: Option<CachedEvaluation> = None;

        for (pattern_index, pattern) in self.patterns.iter().enumerate() {
            for (variant_index, variant) in pattern.variants.iter().enumerate() {
                let totals = VariantTotals::from_variant(variant);
                let stats = self.analyze_variant(board, player, king_pos, variant);

                let zone_metrics = ZoneMetrics::evaluate(board, player, king_pos);

                let base_quality = self.calculate_match_quality(
                    stats.matches,
                    totals.total_pieces,
                    stats.matched_weight,
                    totals.total_weight,
                );

                let required_ratio = if totals.required_total > 0 {
                    stats.required_matches as f32 / totals.required_total as f32
                } else {
                    1.0
                };

                let pattern_coverage_ratio = if totals.total_weight > 0 {
                    stats.matched_weight as f32 / totals.total_weight as f32
                } else {
                    0.0
                };

                let zone_coverage_ratio = zone_metrics.coverage_ratio();
                let zone_pawn_wall_ratio = zone_metrics.pawn_wall_ratio();

                let pattern_shield_ratio = if totals.shield_total > 0 {
                    stats.pawn_shield_matches as f32 / totals.shield_total as f32
                } else {
                    0.0
                };

                let zone_shield_ratio = zone_metrics.forward_ratio();
                let zone_shield_component =
                    (0.4 * zone_shield_ratio + 0.6 * zone_pawn_wall_ratio).clamp(0.0, 1.0);

                let combined_coverage_ratio =
                    0.6 * pattern_coverage_ratio + 0.4 * zone_coverage_ratio;
                let combined_shield_ratio =
                    (0.6 * pattern_shield_ratio + 0.4 * zone_shield_component).clamp(0.0, 1.0);

                let primary_ratio = if totals.primary_total > 0 {
                    stats.primary_matches as f32 / totals.primary_total as f32
                } else {
                    required_ratio
                };

                let secondary_ratio = if totals.secondary_total > 0 {
                    stats.secondary_matches as f32 / totals.secondary_total as f32
                } else {
                    1.0
                };

                let zone_buffer_ratio = zone_metrics.buffer_ratio();

                let buffer_ratio = if totals.buffer_total > 0 {
                    stats.buffer_matches as f32 / totals.buffer_total as f32
                } else {
                    zone_buffer_ratio
                };

                let zone_integrity =
                    (0.4 * zone_coverage_ratio
                        + 0.3 * zone_buffer_ratio
                        + 0.3 * zone_shield_component)
                        .clamp(0.1, 1.0);

                let quality = (base_quality * required_ratio * (0.7 + 0.3 * zone_integrity))
                    .clamp(0.0, 1.0);

                let missing_required =
                    totals.required_total.saturating_sub(stats.required_matches);
                let optional_matches = stats.matches.saturating_sub(stats.required_matches);
                let missing_optional = totals.optional_total().saturating_sub(optional_matches);
                let missing_primary =
                    totals.primary_total.saturating_sub(stats.primary_matches);
                let missing_shield =
                    totals.shield_total.saturating_sub(stats.pawn_shield_matches);
                let missing_secondary =
                    totals.secondary_total.saturating_sub(stats.secondary_matches);
                let missing_buffer = totals.buffer_total.saturating_sub(stats.buffer_matches);

                let infiltration_ratio = zone_metrics.infiltration_ratio();

                let base_score = self.adjust_score_for_quality(pattern.score, quality);

                let matched_pattern = if quality >= self.early_termination_threshold {
                    Some(pattern.name)
                } else {
                    None
                };

                let evaluation = CastleEvaluation {
                    matched_pattern,
                    variant_id: Some(variant.id),
                    quality,
                    coverage_ratio: combined_coverage_ratio,
                    pattern_coverage_ratio,
                    zone_coverage_ratio,
                    primary_ratio,
                    secondary_ratio,
                    buffer_ratio,
                    pawn_shield_ratio: combined_shield_ratio,
                    pattern_shield_ratio,
                    zone_shield_ratio,
                    zone_forward_ratio: zone_shield_ratio,
                    zone_pawn_wall_ratio,
                    infiltration_ratio,
                    missing_required,
                    missing_optional,
                    missing_primary,
                    missing_shield,
                    missing_secondary,
                    missing_buffer,
                    matched_pieces: stats.matches,
                    total_pieces: totals.total_pieces,
                    required_ratio,
                    base_score,
                };

                let should_update = best_entry
                    .map(|entry| evaluation.is_better_than(&entry.evaluation))
                    .unwrap_or(true);

                if should_update {
                    best_entry = Some(CachedEvaluation {
                        pattern_index,
                        variant_index,
                        evaluation,
                    });
                }
            }
        }

        let result = best_entry.map(|entry| entry.evaluation).unwrap_or_default();

        if self.pattern_cache.borrow().len() < CACHE_MAX_ENTRIES {
            self.pattern_cache
                .borrow_mut()
                .insert((board_hash, player, king_pos), best_entry);
        }

        result
    }

    pub fn evaluate_castle_structure(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> TaperedScore {
        self.evaluate_castle(board, player, king_pos).score()
    }

    fn get_board_hash(&self, board: &BitboardBoard) -> u64 {
        let mut hash = 0u64;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_hash = (piece.piece_type as u8 as u64) << (piece.player as u8 * 4);
                    hash ^= piece_hash
                        .wrapping_mul(pos.row as u64 + 1)
                        .wrapping_mul(pos.col as u64 + 1);
                }
            }
        }
        hash
    }

    pub fn clear_cache(&self) {
        self.pattern_cache.borrow_mut().clear();
    }

    pub fn set_early_termination_threshold(&mut self, threshold: f32) {
        self.early_termination_threshold = threshold;
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.pattern_cache.borrow();
        (cache.len(), CACHE_MAX_ENTRIES)
    }

    fn matches_pattern(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
        pattern: &CastlePattern,
    ) -> bool {
        pattern.variants.iter().any(|variant| {
            let totals = VariantTotals::from_variant(variant);
            let stats = self.analyze_variant(board, player, king_pos, variant);
            if stats.required_matches < totals.required_total {
                return false;
            }
            let min_matches = totals
                .total_pieces
                .saturating_sub(pattern.flexibility as usize);
            stats.matches >= min_matches
        })
    }

    fn analyze_variant(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
        variant: &CastleVariant,
    ) -> VariantMatchStats {
        let mut stats = VariantMatchStats::default();

        for descriptor in &variant.pieces {
            if let Some(target) = descriptor.offset.to_absolute(king_pos, player) {
                if let Some(piece) = board.get_piece(target) {
                    if piece.player == player && descriptor.class.matches(piece.piece_type) {
                        stats.matches += 1;
                        stats.matched_weight += descriptor.weight as u32;
                        if descriptor.required {
                            stats.required_matches += 1;
                        }
                        match descriptor.role {
                            CastlePieceRole::PrimaryDefender => stats.primary_matches += 1,
                            CastlePieceRole::SecondaryDefender => stats.secondary_matches += 1,
                            CastlePieceRole::Buffer => stats.buffer_matches += 1,
                            CastlePieceRole::PawnShield => stats.pawn_shield_matches += 1,
                            _ => {}
                        }
                    }
                }
            }
        }

        stats
    }

    fn calculate_match_quality(
        &self,
        matches: usize,
        total_pieces: usize,
        matched_weight: u32,
        max_weight: u32,
    ) -> f32 {
        if total_pieces == 0 {
            return 0.0;
        }

        let piece_quality = matches as f32 / total_pieces as f32;
        let weight_quality = if max_weight > 0 {
            matched_weight as f32 / max_weight as f32
        } else {
            0.0
        };

        0.6 * piece_quality + 0.4 * weight_quality
    }

    fn adjust_score_for_quality(&self, base_score: TaperedScore, quality: f32) -> TaperedScore {
        TaperedScore {
            mg: (base_score.mg as f32 * quality) as i32,
            eg: (base_score.eg as f32 * quality) as i32,
        }
    }
}

impl Default for CastleRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct VariantMatchStats {
    matches: usize,
    required_matches: usize,
    matched_weight: u32,
    primary_matches: usize,
    secondary_matches: usize,
    buffer_matches: usize,
    pawn_shield_matches: usize,
}

struct VariantTotals {
    total_pieces: usize,
    required_total: usize,
    total_weight: u32,
    primary_total: usize,
    secondary_total: usize,
    buffer_total: usize,
    shield_total: usize,
}

impl VariantTotals {
    fn from_variant(variant: &CastleVariant) -> Self {
        let mut required_total = 0;
        let mut total_weight = 0u32;
        let mut primary_total = 0;
        let mut secondary_total = 0;
        let mut buffer_total = 0;
        let mut shield_total = 0;

        for piece in &variant.pieces {
            if piece.required {
                required_total += 1;
            }
            total_weight += piece.weight as u32;
            match piece.role {
                CastlePieceRole::PrimaryDefender => primary_total += 1,
                CastlePieceRole::SecondaryDefender => secondary_total += 1,
                CastlePieceRole::Buffer => buffer_total += 1,
                CastlePieceRole::PawnShield => shield_total += 1,
                _ => {}
            }
        }

        Self {
            total_pieces: variant.pieces.len(),
            required_total,
            total_weight,
            primary_total,
            secondary_total,
            buffer_total,
            shield_total,
        }
    }

    fn optional_total(&self) -> usize {
        self.total_pieces.saturating_sub(self.required_total)
    }
}

struct ZoneMetrics {
    ring_slots: usize,
    ring_friendly: usize,
    ring_opponent: usize,
    forward_slots: usize,
    forward_pawn_hits: usize,
    buffer_slots: usize,
    buffer_friendly: usize,
    pawn_wall_slots: usize,
    pawn_wall_friendly: usize,
}

impl ZoneMetrics {
    fn evaluate(board: &BitboardBoard, player: Player, king_pos: Position) -> Self {
        let mut metrics = Self {
            ring_slots: 0,
            ring_friendly: 0,
            ring_opponent: 0,
            forward_slots: 0,
            forward_pawn_hits: 0,
            buffer_slots: 0,
            buffer_friendly: 0,
            pawn_wall_slots: 0,
            pawn_wall_friendly: 0,
        };

        for offset in &KING_ZONE_RING {
            if let Some(target) = offset.to_absolute(king_pos, player) {
                metrics.ring_slots += 1;
                if let Some(piece) = board.get_piece(target) {
                    if piece.player == player {
                        metrics.ring_friendly += 1;
                    } else {
                        metrics.ring_opponent += 1;
                    }
                }
            }
        }

        for offset in &FORWARD_SHIELD_ARC {
            if let Some(target) = offset.to_absolute(king_pos, player) {
                metrics.forward_slots += 1;
                if let Some(piece) = board.get_piece(target) {
                    if piece.player == player
                        && matches!(piece.piece_type, PieceType::Pawn | PieceType::PromotedPawn)
                    {
                        metrics.forward_pawn_hits += 1;
                    }
                }
            }
        }

        for offset in &BUFFER_RING {
            if let Some(target) = offset.to_absolute(king_pos, player) {
                metrics.buffer_slots += 1;
                if let Some(piece) = board.get_piece(target) {
                    if piece.player == player {
                        metrics.buffer_friendly += 1;
                    }
                }
            }
        }

        for offset in &PAWN_WALL_ARC {
            if let Some(target) = offset.to_absolute(king_pos, player) {
                metrics.pawn_wall_slots += 1;
                if let Some(piece) = board.get_piece(target) {
                    if piece.player == player
                        && matches!(piece.piece_type, PieceType::Pawn | PieceType::PromotedPawn)
                    {
                        metrics.pawn_wall_friendly += 1;
                    }
                }
            }
        }

        metrics
    }

    fn coverage_ratio(&self) -> f32 {
        if self.ring_slots == 0 {
            0.0
        } else {
            self.ring_friendly as f32 / self.ring_slots as f32
        }
    }

    fn forward_ratio(&self) -> f32 {
        if self.forward_slots == 0 {
            0.0
        } else {
            self.forward_pawn_hits as f32 / self.forward_slots as f32
        }
    }

    fn buffer_ratio(&self) -> f32 {
        if self.buffer_slots == 0 {
            0.0
        } else {
            self.buffer_friendly as f32 / self.buffer_slots as f32
        }
    }

    fn pawn_wall_ratio(&self) -> f32 {
        if self.pawn_wall_slots == 0 {
            0.0
        } else {
            self.pawn_wall_friendly as f32 / self.pawn_wall_slots as f32
        }
    }

    fn infiltration_ratio(&self) -> f32 {
        if self.ring_slots == 0 {
            0.0
        } else {
            self.ring_opponent as f32 / self.ring_slots as f32
        }
    }
}

impl CastlePiece {
    pub const fn from_descriptor(descriptor: CastlePieceDescriptor) -> Self {
        Self {
            class: descriptor.class,
            offset: descriptor.offset,
            required: descriptor.required,
            weight: descriptor.weight,
            role: descriptor.role,
        }
    }
}

impl CastleVariant {
    pub fn from_descriptors(id: &'static str, descriptors: &[CastlePieceDescriptor]) -> Self {
        Self {
            id,
            pieces: descriptors
                .iter()
                .copied()
                .map(CastlePiece::from_descriptor)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Piece, PieceType};

    fn build_test_variant() -> CastleVariant {
        let descriptors = vec![
            CastlePieceDescriptor::new(
                exact(PieceType::Gold),
                RelativeOffset::new(-1, 0),
                true,
                10,
                CastlePieceRole::PrimaryDefender,
            ),
            CastlePieceDescriptor::new(
                exact(PieceType::Silver),
                RelativeOffset::new(-2, 0),
                true,
                9,
                CastlePieceRole::PrimaryDefender,
            ),
            CastlePieceDescriptor::new(
                exact(PieceType::Pawn),
                RelativeOffset::new(-1, -1),
                false,
                6,
                CastlePieceRole::PawnShield,
            ),
        ];
        CastleVariant::from_descriptors("base", &descriptors)
    }

    fn place_relative(
        board: &mut BitboardBoard,
        player: Player,
        king: Position,
        offset: RelativeOffset,
        piece_type: PieceType,
    ) {
        let target = offset
            .to_absolute(king, player)
            .expect("offset should stay on board for fixture");
        board.place_piece(Piece::new(piece_type, player), target);
    }

    #[test]
    fn test_castle_recognizer_creation() {
        let recognizer = CastleRecognizer::new();
        assert_eq!(recognizer.patterns.len(), 3);
    }

    #[test]
    fn test_castle_pattern_structure() {
        let variant = build_test_variant();
        let pattern = CastlePattern {
            name: "Test",
            variants: vec![variant],
            score: TaperedScore::new_tapered(100, 60),
            flexibility: 1,
        };

        assert_eq!(pattern.name, "Test");
        assert_eq!(pattern.variants.len(), 1);
        assert_eq!(pattern.flexibility, 1);
        assert_eq!(pattern.score.mg, 100);
        assert_eq!(pattern.score.eg, 60);
        assert_eq!(pattern.variants[0].pieces.len(), 3);
    }

    #[test]
    fn test_relative_offset_application() {
        let king = Position::new(4, 4);
        let offset = RelativeOffset::new(-1, -1);

        let black_target = offset.to_absolute(king, Player::Black).unwrap();
        assert_eq!(black_target, Position::new(3, 3));

        let white_target = offset.to_absolute(king, Player::White).unwrap();
        assert_eq!(white_target, Position::new(5, 5));
    }

    #[test]
    fn test_match_quality_calculation() {
        let recognizer = CastleRecognizer::new();
        let quality = recognizer.calculate_match_quality(3, 5, 30, 50);
        assert!(quality > 0.5 && quality < 1.0);

        let perfect = recognizer.calculate_match_quality(5, 5, 50, 50);
        assert!((perfect - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_adjust_score_for_quality() {
        let recognizer = CastleRecognizer::new();
        let base = TaperedScore::new_tapered(200, 80);
        let adjusted = recognizer.adjust_score_for_quality(base, 0.5);
        assert_eq!(adjusted.mg, 100);
        assert_eq!(adjusted.eg, 40);
    }

    #[test]
    fn test_recognize_anaguma_with_promoted_silver() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 6);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, 0),
            PieceType::Gold,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, 0),
            PieceType::PromotedSilver,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -1),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, 1),
            PieceType::PromotedPawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, -1),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, 1),
            PieceType::Pawn,
        );

        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);
        assert!(score.mg > 0);

        let matched = recognizer
            .recognize_castle(&board, Player::Black, king_pos)
            .map(|pattern| pattern.name.to_string());
        assert_eq!(matched.as_deref(), Some("Anaguma"));
    }

    #[test]
    fn test_mino_recognition_for_white_mirror() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(0, 2);
        board.place_piece(Piece::new(PieceType::King, Player::White), king_pos);
        place_relative(
            &mut board,
            Player::White,
            king_pos,
            RelativeOffset::new(-1, -1),
            PieceType::Gold,
        );
        place_relative(
            &mut board,
            Player::White,
            king_pos,
            RelativeOffset::new(-2, -1),
            PieceType::Silver,
        );
        place_relative(
            &mut board,
            Player::White,
            king_pos,
            RelativeOffset::new(-2, -2),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::White,
            king_pos,
            RelativeOffset::new(-1, -2),
            PieceType::PromotedPawn,
        );
        place_relative(
            &mut board,
            Player::White,
            king_pos,
            RelativeOffset::new(0, -2),
            PieceType::Pawn,
        );

        let score = recognizer.evaluate_castle_structure(&board, Player::White, king_pos);
        assert!(score.mg > 0);

        let matched = recognizer
            .recognize_castle(&board, Player::White, king_pos)
            .map(|pattern| pattern.name.to_string());
        assert_eq!(matched.as_deref(), Some("Mino"));
    }

    #[test]
    fn test_yagura_requires_pawn_wall() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 4);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, -1),
            PieceType::Gold,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -1),
            PieceType::Silver,
        );

        let score_without_wall =
            recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);
        assert_eq!(score_without_wall, TaperedScore::default());

        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -2),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, -2),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -3),
            PieceType::Knight,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(0, -3),
            PieceType::Lance,
        );

        let score_with_wall = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);
        assert!(score_with_wall.mg > 0);

        let matched = recognizer
            .recognize_castle(&board, Player::Black, king_pos)
            .map(|pattern| pattern.name.to_string());
        assert!(matched.is_some());
    }

    #[test]
    fn test_castle_evaluation_reports_quality() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 6);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-1, 0),
            PieceType::Gold,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, 0),
            PieceType::Silver,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -1),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, 1),
            PieceType::Pawn,
        );

        let evaluation = recognizer.evaluate_castle(&board, Player::Black, king_pos);
        assert!(evaluation.quality > 0.6);
        assert!(evaluation.pawn_shield_ratio > 0.4);
        assert_eq!(evaluation.missing_required, 0);
    }

    #[test]
    fn test_castle_evaluation_detects_missing_primary_defenders() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 6);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        // Provide only pawn shield without gold/silver defenders
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, -1),
            PieceType::Pawn,
        );
        place_relative(
            &mut board,
            Player::Black,
            king_pos,
            RelativeOffset::new(-2, 1),
            PieceType::Pawn,
        );

        let evaluation = recognizer.evaluate_castle(&board, Player::Black, king_pos);
        assert!(evaluation.quality < 0.2);
        assert!(evaluation.missing_required >= 1);
        assert!(evaluation.score().mg <= 0);
    }

    #[test]
    fn test_bare_king_zone_metrics_low() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 4);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);

        let evaluation = recognizer.evaluate_castle(&board, Player::Black, king_pos);
        assert!(evaluation.zone_coverage_ratio <= 0.25);
        assert!(evaluation.pawn_shield_ratio <= 0.25);
        assert!(evaluation.quality < 0.2);
    }

    #[test]
    fn test_infiltration_ratio_detects_opponent_piece() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 4);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        board.place_piece(Piece::new(PieceType::Gold, Player::Black), Position::new(7, 4));
        board.place_piece(Piece::new(PieceType::Silver, Player::Black), Position::new(6, 4));
        board.place_piece(Piece::new(PieceType::Pawn, Player::Black), Position::new(6, 3));
        board.place_piece(Piece::new(PieceType::Pawn, Player::Black), Position::new(6, 5));
        // Opponent piece infiltrating the king zone
        board.place_piece(Piece::new(PieceType::Knight, Player::White), Position::new(7, 3));

        let evaluation = recognizer.evaluate_castle(&board, Player::Black, king_pos);
        assert!(evaluation.infiltration_ratio > 0.0);
    }
}
