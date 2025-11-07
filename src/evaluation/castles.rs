use crate::bitboards::*;
use crate::evaluation::patterns::*;
use crate::types::*;

/// Castle pattern recognizer for identifying defensive formations
pub struct CastleRecognizer {
    patterns: Vec<CastlePattern>,
    // Performance optimization: cache for pattern matching
    pattern_cache:
        std::cell::RefCell<std::collections::HashMap<(u64, Player, Position), Option<usize>>>,
    // Early termination threshold
    early_termination_threshold: f32,
}

/// Represents a castle pattern with its pieces and scoring
pub struct CastlePattern {
    pub name: &'static str,
    pub pieces: Vec<CastlePiece>,
    pub score: TaperedScore,
    pub flexibility: u8, // How many pieces can be missing and still count
}

/// Represents a piece in a castle pattern
pub struct CastlePiece {
    pub piece_type: PieceType,
    pub relative_pos: (i8, i8), // Relative to king position
    pub required: bool,         // Must be present for pattern match
    pub weight: u8,             // Importance in pattern (1-10)
}

impl CastleRecognizer {
    /// Create a new castle recognizer with default patterns
    pub fn new() -> Self {
        Self {
            patterns: vec![get_mino_castle(), get_anaguma_castle(), get_yagura_castle()],
            pattern_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            early_termination_threshold: 0.8, // Stop if match quality is below 80% (more aggressive)
        }
    }

    /// Recognize castle pattern for the given player and king position
    pub fn recognize_castle(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> Option<&CastlePattern> {
        for pattern in &self.patterns {
            if self.matches_pattern(board, player, king_pos, pattern) {
                return Some(pattern);
            }
        }
        None
    }

    /// Evaluate castle structure with flexibility scoring for incomplete patterns
    pub fn evaluate_castle_structure(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> TaperedScore {
        // Check cache first
        let board_hash = self.get_board_hash(board);
        if let Some(cached_pattern) = self
            .pattern_cache
            .borrow()
            .get(&(board_hash, player, king_pos))
        {
            if let Some(pattern_index) = cached_pattern {
                return self.patterns[*pattern_index].score;
            } else {
                return TaperedScore::default();
            }
        }

        let mut best_score = TaperedScore::default();
        let mut best_match_quality = 0.0;
        let mut best_pattern_index = None;

        for (pattern_index, pattern) in self.patterns.iter().enumerate() {
            let (matches, required_matches, total_weight) =
                self.analyze_pattern_match(board, player, king_pos, pattern);
            let required_pieces = pattern.pieces.iter().filter(|p| p.required).count();

            // Must have all required pieces
            if required_matches < required_pieces {
                continue;
            }

            // Calculate match quality (0.0 to 1.0)
            let match_quality =
                self.calculate_match_quality(matches, pattern.pieces.len(), total_weight, pattern);

            // Early termination: if match quality is below threshold, skip this pattern
            if match_quality < self.early_termination_threshold {
                continue;
            }

            // Only consider if it's a reasonable match (at least 70% quality) - more aggressive
            if match_quality >= 0.7 {
                let adjusted_score = self.adjust_score_for_quality(pattern.score, match_quality);

                // Take the best match
                if match_quality > best_match_quality {
                    best_score = adjusted_score;
                    best_match_quality = match_quality;
                    best_pattern_index = Some(pattern_index);
                }
            }
        }

        // Cache the result (limit cache size) - very small for performance
        if self.pattern_cache.borrow().len() < 50 {
            // Reduced from 500 to 50
            self.pattern_cache
                .borrow_mut()
                .insert((board_hash, player, king_pos), best_pattern_index);
        }

        best_score
    }

    /// Get a simple hash for the board position around the king
    fn get_board_hash(&self, board: &BitboardBoard) -> u64 {
        let mut hash = 0u64;
        // Only hash the area around the king (3x3 to 5x5 area)
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

    /// Clear the pattern cache
    pub fn clear_cache(&self) {
        self.pattern_cache.borrow_mut().clear();
    }

    /// Set the early termination threshold
    pub fn set_early_termination_threshold(&mut self, threshold: f32) {
        self.early_termination_threshold = threshold;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.pattern_cache.borrow();
        (cache.len(), 500) // current size, max size
    }

    /// Check if a pattern matches the current board position
    fn matches_pattern(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
        pattern: &CastlePattern,
    ) -> bool {
        let (matches, required_matches, _) =
            self.analyze_pattern_match(board, player, king_pos, pattern);
        let required_pieces = pattern.pieces.iter().filter(|p| p.required).count();

        // Check if all required pieces are present
        if required_matches < required_pieces {
            return false;
        }

        // Check if enough pieces match (considering flexibility)
        let min_matches = pattern
            .pieces
            .len()
            .saturating_sub(pattern.flexibility as usize);
        matches >= min_matches
    }

    /// Analyze pattern match and return detailed statistics
    fn analyze_pattern_match(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
        pattern: &CastlePattern,
    ) -> (usize, usize, u32) {
        let mut matches = 0;
        let mut required_matches = 0;
        let mut total_weight = 0u32;

        for castle_piece in &pattern.pieces {
            let check_pos = self.get_relative_position(king_pos, castle_piece.relative_pos, player);

            if let Some(check_pos) = check_pos {
                if let Some(piece) = board.get_piece(check_pos) {
                    if piece.piece_type == castle_piece.piece_type && piece.player == player {
                        matches += 1;
                        total_weight += castle_piece.weight as u32;
                        if castle_piece.required {
                            required_matches += 1;
                        }
                    }
                }
            }
        }

        (matches, required_matches, total_weight)
    }

    /// Get the actual board position for a relative position, considering player orientation
    fn get_relative_position(
        &self,
        king_pos: Position,
        relative_pos: (i8, i8),
        player: Player,
    ) -> Option<Position> {
        let (dr, dc) = relative_pos;

        // For White player, flip the row direction (White moves "up" the board)
        let adjusted_dr = match player {
            Player::Black => dr,
            Player::White => -dr,
        };

        let new_row = king_pos.row as i8 + adjusted_dr;
        let new_col = king_pos.col as i8 + dc;

        // Check bounds
        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
            return None;
        }

        Some(Position::new(new_row as u8, new_col as u8))
    }

    /// Calculate match quality based on pieces found and their weights
    fn calculate_match_quality(
        &self,
        matches: usize,
        total_pieces: usize,
        total_weight: u32,
        pattern: &CastlePattern,
    ) -> f32 {
        if total_pieces == 0 {
            return 0.0;
        }

        // Base quality from piece count
        let piece_quality = matches as f32 / total_pieces as f32;

        // Weight quality (emphasize important pieces)
        let max_weight = pattern.pieces.iter().map(|p| p.weight as u32).sum::<u32>();
        let weight_quality = if max_weight > 0 {
            total_weight as f32 / max_weight as f32
        } else {
            0.0
        };

        // Combine both factors (60% piece count, 40% weight importance)
        0.6 * piece_quality + 0.4 * weight_quality
    }

    /// Adjust score based on match quality
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castle_recognizer_creation() {
        let recognizer = CastleRecognizer::new();
        assert_eq!(recognizer.patterns.len(), 3); // Should have 3 castle patterns
    }

    #[test]
    fn test_castle_pattern_creation() {
        let pattern = CastlePattern {
            name: "Test Castle",
            pieces: vec![CastlePiece {
                piece_type: PieceType::Gold,
                relative_pos: (-1, -1),
                required: true,
                weight: 10,
            }],
            score: TaperedScore::new_tapered(100, 50),
            flexibility: 1,
        };

        assert_eq!(pattern.name, "Test Castle");
        assert_eq!(pattern.pieces.len(), 1);
        assert_eq!(pattern.flexibility, 1);
    }

    #[test]
    fn test_castle_piece_creation() {
        let piece = CastlePiece {
            piece_type: PieceType::Silver,
            relative_pos: (-2, -1),
            required: false,
            weight: 8,
        };

        assert_eq!(piece.piece_type, PieceType::Silver);
        assert_eq!(piece.relative_pos, (-2, -1));
        assert_eq!(piece.required, false);
        assert_eq!(piece.weight, 8);
    }

    #[test]
    fn test_castle_evaluation() {
        let recognizer = CastleRecognizer::new();
        let board = BitboardBoard::new();
        let king_pos = Position::new(8, 4); // Black king position

        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);
        // Should return a score (even if zero for starting position)
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_relative_position_calculation() {
        let recognizer = CastleRecognizer::new();
        let king_pos = Position::new(4, 4);

        // Test valid position for Black
        let pos = recognizer.get_relative_position(king_pos, (-1, -1), Player::Black);
        assert_eq!(pos, Some(Position::new(3, 3)));

        // Test valid position for White (flipped row direction)
        let pos = recognizer.get_relative_position(king_pos, (-1, -1), Player::White);
        assert_eq!(pos, Some(Position::new(5, 3)));

        // Test out of bounds
        let pos = recognizer.get_relative_position(king_pos, (-5, -5), Player::Black);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_match_quality_calculation() {
        let recognizer = CastleRecognizer::new();
        let pattern = get_mino_castle();

        // Calculate max weight for the pattern
        let max_weight = pattern.pieces.iter().map(|p| p.weight as u32).sum::<u32>();

        // Perfect match
        let quality = recognizer.calculate_match_quality(5, 5, max_weight, &pattern);
        assert!((quality - 1.0).abs() < 0.01);

        // Partial match
        let quality = recognizer.calculate_match_quality(3, 5, max_weight * 3 / 5, &pattern);
        assert!(quality > 0.5 && quality < 1.0);
    }

    #[test]
    fn test_castle_recognition_both_players() {
        let recognizer = CastleRecognizer::new();
        let board = BitboardBoard::new();

        // Test Black king position
        let black_king_pos = Position::new(8, 4);
        let black_score =
            recognizer.evaluate_castle_structure(&board, Player::Black, black_king_pos);

        // Test White king position
        let white_king_pos = Position::new(0, 4);
        let white_score =
            recognizer.evaluate_castle_structure(&board, Player::White, white_king_pos);

        // Both should return scores (even if zero for starting position)
        assert_eq!(black_score, TaperedScore::default());
        assert_eq!(white_score, TaperedScore::default());
    }
}
