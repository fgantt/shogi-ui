use crate::bitboards::*;
use crate::evaluation::castle_geometry::{CastlePieceClass, RelativeOffset};
use crate::evaluation::patterns::*;
use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub use crate::evaluation::castle_geometry::{
    exact, mirror_descriptors, CastlePieceDescriptor, GOLD_FAMILY, KNIGHT_FAMILY, LANCE_FAMILY,
    PAWN_WALL_FAMILY, SILVER_FAMILY,
};

const CACHE_MAX_ENTRIES: usize = 50;

pub struct CastleRecognizer {
    patterns: Vec<CastlePattern>,
    pattern_cache: RefCell<HashMap<(u64, Player, Position), Option<CachedMatch>>>,
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
}

#[derive(Clone, Copy, Debug)]
struct CachedMatch {
    pattern_index: usize,
    variant_index: usize,
    score: TaperedScore,
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

    pub fn evaluate_castle_structure(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> TaperedScore {
        let board_hash = self.get_board_hash(board);
        if let Some(cached) = self
            .pattern_cache
            .borrow()
            .get(&(board_hash, player, king_pos))
        {
            return cached.map(|entry| entry.score).unwrap_or_default();
        }

        let mut best_match: Option<CachedMatch> = None;
        let mut best_quality = 0.0f32;

        for (pattern_index, pattern) in self.patterns.iter().enumerate() {
            for (variant_index, variant) in pattern.variants.iter().enumerate() {
                let stats = self.analyze_variant(board, player, king_pos, variant);
                let required_count = variant_required_count(variant);
                if stats.required_matches < required_count {
                    continue;
                }

                let match_quality = self.calculate_match_quality(
                    stats.matches,
                    variant.pieces.len(),
                    stats.matched_weight,
                    variant_total_weight(variant),
                );

                if match_quality < self.early_termination_threshold {
                    continue;
                }

                if match_quality >= 0.7 && match_quality > best_quality {
                    let score = self.adjust_score_for_quality(pattern.score, match_quality);
                    best_quality = match_quality;
                    best_match = Some(CachedMatch {
                        pattern_index,
                        variant_index,
                        score,
                    });
                }
            }
        }

        if self.pattern_cache.borrow().len() < CACHE_MAX_ENTRIES {
            self.pattern_cache
                .borrow_mut()
                .insert((board_hash, player, king_pos), best_match);
        }

        best_match.map(|entry| entry.score).unwrap_or_default()
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
            let stats = self.analyze_variant(board, player, king_pos, variant);
            let required_count = variant_required_count(variant);
            if stats.required_matches < required_count {
                return false;
            }
            let min_matches = variant
                .pieces
                .len()
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
}

fn variant_required_count(variant: &CastleVariant) -> usize {
    variant.pieces.iter().filter(|p| p.required).count()
}

fn variant_total_weight(variant: &CastleVariant) -> u32 {
    variant.pieces.iter().map(|p| p.weight as u32).sum()
}

impl CastlePiece {
    pub const fn from_descriptor(descriptor: CastlePieceDescriptor) -> Self {
        Self {
            class: descriptor.class,
            offset: descriptor.offset,
            required: descriptor.required,
            weight: descriptor.weight,
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
            ),
            CastlePieceDescriptor::new(
                exact(PieceType::Silver),
                RelativeOffset::new(-2, 0),
                true,
                9,
            ),
            CastlePieceDescriptor::new(
                exact(PieceType::Pawn),
                RelativeOffset::new(-1, -1),
                false,
                6,
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

        let matched = recognizer.recognize_castle(&board, Player::Black, king_pos).
            map(|pattern| pattern.name.to_string());
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

        let matched = recognizer.recognize_castle(&board, Player::White, king_pos)
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

        let matched = recognizer.recognize_castle(&board, Player::Black, king_pos)
            .map(|pattern| pattern.name.to_string());
        assert!(matched.is_some());
    }
}
