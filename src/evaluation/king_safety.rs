use crate::bitboards::*;
use crate::evaluation::attacks::{AttackAnalyzer, ThreatEvaluator};
use crate::evaluation::castles::CastleRecognizer;
use crate::types::*;
use std::collections::HashMap;

/// Main king safety evaluator that combines castle recognition, attack analysis, and threat evaluation
pub struct KingSafetyEvaluator {
    config: KingSafetyConfig,
    castle_recognizer: CastleRecognizer,
    attack_analyzer: AttackAnalyzer,
    threat_evaluator: ThreatEvaluator,
    // Performance optimization: cache for expensive operations
    evaluation_cache: std::cell::RefCell<HashMap<(u64, Player), TaperedScore>>,
    // Fast mode configuration
    fast_mode_threshold: u8,
}

impl KingSafetyEvaluator {
    /// Create a new king safety evaluator with default configuration
    pub fn new() -> Self {
        Self::with_config(KingSafetyConfig::default())
    }

    /// Create a new king safety evaluator with custom configuration
    pub fn with_config(config: KingSafetyConfig) -> Self {
        Self {
            castle_recognizer: CastleRecognizer::new(),
            attack_analyzer: AttackAnalyzer::new(),
            threat_evaluator: ThreatEvaluator::new(),
            evaluation_cache: std::cell::RefCell::new(HashMap::new()),
            fast_mode_threshold: 1, // Use fast mode for depth >= 1 (very aggressive)
            config,
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &KingSafetyConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: KingSafetyConfig) {
        self.config = config;
    }

    /// Main evaluation function that combines all king safety components
    pub fn evaluate(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.evaluate_with_depth(board, player, 0)
    }

    /// Evaluate only at root and key nodes for performance - very aggressive
    pub fn evaluate_selective(
        &self,
        board: &BitboardBoard,
        player: Player,
        depth: u8,
        is_root: bool,
        has_capture: bool,
        has_check: bool,
    ) -> TaperedScore {
        // Only evaluate king safety at:
        // - Root node (depth 0)
        // - Nodes with captures or checks
        // - Very shallow nodes (depth <= 1) - more aggressive
        if is_root || has_capture || has_check || depth <= 1 {
            self.evaluate_with_depth(board, player, depth)
        } else {
            TaperedScore::default()
        }
    }

    /// Evaluate with depth information for performance optimization
    pub fn evaluate_with_depth(
        &self,
        board: &BitboardBoard,
        player: Player,
        depth: u8,
    ) -> TaperedScore {
        if !self.config.enabled {
            return TaperedScore::default();
        }

        // Check cache first
        let board_hash = self.get_board_hash(board);
        if let Some(cached_score) = self.evaluation_cache.borrow().get(&(board_hash, player)) {
            return *cached_score;
        }

        // Determine if we should use fast mode - very aggressive for performance
        let use_fast_mode = self.config.performance_mode || depth >= self.fast_mode_threshold;

        let mut total_score = TaperedScore::default();

        if use_fast_mode {
            // Fast mode: simplified evaluation
            total_score = self.evaluate_fast_mode(board, player);
        } else {
            // Full evaluation
            // Castle structure evaluation
            if let Some(king_pos) = self.find_king_position(board, player) {
                let castle_eval = self
                    .castle_recognizer
                    .evaluate_castle(board, player, king_pos);

                let quality = castle_eval.quality.clamp(0.0, 1.0);

                let coverage_weight_sum =
                    self.config.pattern_coverage_weight + self.config.zone_coverage_weight;
                let coverage_ratio = if coverage_weight_sum > 0.0 {
                    (castle_eval.pattern_coverage_ratio * self.config.pattern_coverage_weight
                        + castle_eval.zone_coverage_ratio * self.config.zone_coverage_weight)
                        / coverage_weight_sum
                } else {
                    castle_eval.coverage_ratio
                }
                .clamp(0.0, 1.0);

                let shield_weight_sum =
                    self.config.pattern_shield_weight + self.config.zone_shield_weight;
                let zone_shield_mix = (castle_eval.zone_shield_ratio * 0.4
                    + castle_eval.zone_pawn_wall_ratio * 0.6)
                    .clamp(0.0, 1.0);
                let shield_ratio = if shield_weight_sum > 0.0 {
                    (castle_eval.pattern_shield_ratio * self.config.pattern_shield_weight
                        + zone_shield_mix * self.config.zone_shield_weight)
                        / shield_weight_sum
                } else {
                    castle_eval.pawn_shield_ratio
                }
                .clamp(0.0, 1.0);

                let primary_ratio = castle_eval.primary_ratio.clamp(0.0, 1.0);
                let secondary_ratio = castle_eval.secondary_ratio.clamp(0.0, 1.0);
                let buffer_ratio = castle_eval.buffer_ratio.clamp(0.0, 1.0);

                let mut castle_score = castle_eval.base_score;

                castle_score += self.config.coverage_bonus * coverage_ratio;
                castle_score += self.config.pawn_shield_bonus * shield_ratio;
                castle_score += self.config.primary_bonus * primary_ratio;

                if castle_eval.missing_primary > 0 {
                    castle_score +=
                        self.config.primary_defender_penalty * castle_eval.missing_primary as f32;
                }
                if castle_eval.missing_shield > 0 {
                    castle_score +=
                        self.config.pawn_shield_penalty * castle_eval.missing_shield as f32;
                }

                let exposure_weight_sum = self.config.exposure_zone_weight
                    + self.config.exposure_shield_weight
                    + self.config.exposure_primary_weight;
                let zone_exposure_component = if exposure_weight_sum > 0.0 {
                    (castle_eval.zone_coverage_ratio * self.config.exposure_zone_weight
                        + zone_shield_mix * self.config.exposure_shield_weight
                        + primary_ratio * self.config.exposure_primary_weight)
                        / exposure_weight_sum
                } else {
                    castle_eval.zone_coverage_ratio
                };

                let shell_integrity = (secondary_ratio * 0.6 + buffer_ratio * 0.4).clamp(0.0, 1.0);

                let exposure_deficit =
                    (1.0 - (0.6 * zone_exposure_component + 0.4 * shell_integrity)).clamp(0.0, 1.0);
                if exposure_deficit > 0.0 {
                    castle_score += self.config.exposed_king_penalty * exposure_deficit;
                }

                let infiltration_pressure =
                    (castle_eval.infiltration_ratio + (1.0 - shell_integrity)).clamp(0.0, 1.0);
                if infiltration_pressure > 0.0 {
                    castle_score += self.config.infiltration_penalty * infiltration_pressure;
                }

                if quality < self.config.castle_quality_threshold {
                    if quality >= self.config.partial_castle_threshold {
                        let span = (self.config.castle_quality_threshold
                            - self.config.partial_castle_threshold)
                            .max(1e-3);
                        let deficit = (self.config.castle_quality_threshold - quality) / span;
                        castle_score += self.config.partial_castle_penalty * deficit;
                    } else {
                        let bare_scale = (self.config.partial_castle_threshold - quality)
                            / self.config.partial_castle_threshold.max(1e-3);
                        castle_score += self.config.partial_castle_penalty;
                        castle_score += self.config.bare_king_penalty * bare_scale.clamp(0.0, 1.0);
                    }
                }

                total_score += castle_score * self.config.castle_weight;
            }

            // Attack analysis
            let attack_score = self.attack_analyzer.evaluate_attacks(board, player);
            total_score += attack_score * self.config.attack_weight;

            // Threat evaluation - use fast mode for depths >= 1 (very aggressive)
            let use_threat_fast_mode = depth >= 1;
            let threat_score = self.threat_evaluator.evaluate_threats_with_mode(
                board,
                player,
                use_threat_fast_mode,
            );
            total_score += threat_score * self.config.threat_weight;
        }

        // Apply phase adjustment
        let final_score = total_score * self.config.phase_adjustment;

        // Cache the result (limit cache size) - very small for performance
        if self.evaluation_cache.borrow().len() < 100 {
            // Reduced from 1000 to 100
            self.evaluation_cache
                .borrow_mut()
                .insert((board_hash, player), final_score);
        }

        final_score
    }

    /// Fast mode evaluation for deep search nodes
    fn evaluate_fast_mode(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();

        // Simplified castle evaluation (only check for basic patterns)
        if let Some(king_pos) = self.find_king_position(board, player) {
            score += self.evaluate_basic_castle(board, player, king_pos);
        }

        // Simplified attack evaluation (only count major pieces near king)
        score += self.evaluate_basic_attacks(board, player);

        // Basic threat evaluation (pins only) with very reduced weight
        let threat_score = self
            .threat_evaluator
            .evaluate_threats_with_mode(board, player, true);
        score += threat_score * 0.1; // Reduced from 0.3 to 0.1

        score
    }

    /// Basic castle evaluation for fast mode
    fn evaluate_basic_castle(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
    ) -> TaperedScore {
        let mut score = 0;

        // Check for basic defensive pieces around king
        let defensive_offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dr, dc) in defensive_offsets.iter() {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        match piece.piece_type {
                            PieceType::Gold | PieceType::Silver => score += 20,
                            PieceType::Pawn => score += 10,
                            _ => score += 5,
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(score, score / 2)
    }

    /// Basic attack evaluation for fast mode
    fn evaluate_basic_attacks(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = 0;

        if let Some(king_pos) = self.find_king_position(board, player) {
            let opponent = player.opposite();

            // Check for major pieces attacking the king zone
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player == opponent {
                            let distance = ((row as i8 - king_pos.row as i8).abs()
                                + (col as i8 - king_pos.col as i8).abs())
                                as u8;

                            if distance <= 3 {
                                match piece.piece_type {
                                    PieceType::Rook | PieceType::PromotedRook => score -= 30,
                                    PieceType::Bishop | PieceType::PromotedBishop => score -= 25,
                                    PieceType::Silver | PieceType::Gold => score -= 15,
                                    _ => score -= 5,
                                }
                            }
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(score, score / 2)
    }

    /// Get a simple hash for the board position
    fn get_board_hash(&self, board: &BitboardBoard) -> u64 {
        // Simple hash based on piece positions
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

    /// Clear the evaluation cache
    pub fn clear_cache(&self) {
        self.evaluation_cache.borrow_mut().clear();
    }

    /// Set the fast mode threshold
    pub fn set_fast_mode_threshold(&mut self, threshold: u8) {
        self.fast_mode_threshold = threshold;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.evaluation_cache.borrow();
        (cache.len(), 1000) // current size, max size
    }

    /// Evaluate castle structure for the given player
    pub fn evaluate_castle_structure(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        if let Some(king_pos) = self.find_king_position(board, player) {
            self.castle_recognizer
                .evaluate_castle(board, player, king_pos)
                .score()
        } else {
            TaperedScore::default()
        }
    }

    /// Evaluate attacks on the king for the given player
    pub fn evaluate_attacks(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // TODO: Implement attack analysis
        // This is a placeholder implementation
        TaperedScore::default()
    }

    /// Evaluate tactical threats to the king for the given player
    pub fn evaluate_threats(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // TODO: Implement threat evaluation
        // This is a placeholder implementation
        TaperedScore::default()
    }

    /// Fast evaluation for nodes deep in search tree
    pub fn evaluate_fast(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        if !self.config.enabled || !self.config.performance_mode {
            return self.evaluate(board, player);
        }

        // Use fast mode evaluation
        self.evaluate_fast_mode(board, player)
    }

    /// Skip king safety evaluation in quiescence search
    pub fn evaluate_quiescence(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // Return zero for quiescence search to avoid expensive evaluation
        TaperedScore::default()
    }

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

    /// Check if king safety evaluation needs to be updated
    pub fn needs_update(
        &self,
        board: &BitboardBoard,
        player: Player,
        last_king_pos: Option<Position>,
        last_material_count: u8,
    ) -> bool {
        // Check if king moved
        let current_king_pos = self.find_king_position(board, player);
        if current_king_pos != last_king_pos {
            return true;
        }

        // Check if material count changed significantly (captures)
        let current_material = self.count_material_near_king(board, player, current_king_pos);
        if current_material != last_material_count {
            return true;
        }

        false
    }

    /// Count material near the king for incremental updates
    fn count_material_near_king(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Option<Position>,
    ) -> u8 {
        if let Some(king_pos) = king_pos {
            let mut count = 0;
            // Count pieces in 3x3 area around king
            for dr in -1..=1 {
                for dc in -1..=1 {
                    let new_row = king_pos.row as i8 + dr;
                    let new_col = king_pos.col as i8 + dc;

                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        let pos = Position::new(new_row as u8, new_col as u8);
                        if let Some(piece) = board.get_piece(pos) {
                            if piece.player == player {
                                count += 1;
                            }
                        }
                    }
                }
            }
            count
        } else {
            0
        }
    }
}

impl Default for KingSafetyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::castles::CastleRecognizer;
    use crate::types::Piece;

    #[test]
    fn test_king_safety_evaluator_creation() {
        let evaluator = KingSafetyEvaluator::new();
        assert!(evaluator.get_config().enabled);
        assert_eq!(evaluator.get_config().castle_weight, 0.3);
        assert_eq!(evaluator.get_config().attack_weight, 0.3);
        assert_eq!(evaluator.get_config().threat_weight, 0.2);
    }

    #[test]
    fn test_king_safety_evaluator_with_config() {
        let config = KingSafetyConfig {
            enabled: false,
            castle_weight: 2.0,
            attack_weight: 1.5,
            threat_weight: 0.5,
            phase_adjustment: 0.6,
            performance_mode: true,
            ..KingSafetyConfig::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config.clone());
        assert_eq!(evaluator.get_config().enabled, false);
        assert_eq!(evaluator.get_config().castle_weight, 2.0);
        assert_eq!(evaluator.get_config().attack_weight, 1.5);
        assert_eq!(evaluator.get_config().threat_weight, 0.5);
    }

    #[test]
    fn test_king_safety_evaluation_disabled() {
        let mut evaluator = KingSafetyEvaluator::new();
        let mut config = evaluator.get_config().clone();
        config.enabled = false;
        evaluator.set_config(config);

        let board = BitboardBoard::new();
        let score = evaluator.evaluate(&board, Player::Black);
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_king_safety_evaluation_enabled() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::new();
        let score = evaluator.evaluate(&board, Player::Black);

        // Should return a score (king safety evaluation is working)
        assert_ne!(score, TaperedScore::default());
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_find_king_position() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::new();

        let black_king = evaluator.find_king_position(&board, Player::Black);
        let white_king = evaluator.find_king_position(&board, Player::White);

        assert!(black_king.is_some());
        assert!(white_king.is_some());

        // Verify king positions are different
        assert_ne!(black_king.unwrap(), white_king.unwrap());
    }

    #[test]
    fn test_full_castle_bonus_is_positive() {
        let mut config = KingSafetyConfig::default();
        config.performance_mode = false;
        config.castle_weight = 1.0;
        config.attack_weight = 0.0;
        config.threat_weight = 0.0;
        config.phase_adjustment = 1.0;
        let evaluator = KingSafetyEvaluator::with_config(config);

        let mut board = BitboardBoard::empty();
        let king_pos = Position::new(8, 6);
        board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        board.place_piece(
            Piece::new(PieceType::Gold, Player::Black),
            Position::new(7, 6),
        );
        board.place_piece(
            Piece::new(PieceType::Silver, Player::Black),
            Position::new(6, 6),
        );
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(6, 5),
        );
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(6, 7),
        );
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(7, 5),
        );
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(7, 7),
        );

        let score = evaluator.evaluate(&board, Player::Black);
        assert!(score.mg > 0);
    }

    #[test]
    fn test_bare_king_penalty_is_negative() {
        let mut config = KingSafetyConfig::default();
        config.performance_mode = false;
        config.castle_weight = 1.0;
        config.attack_weight = 0.0;
        config.threat_weight = 0.0;
        config.phase_adjustment = 1.0;
        let evaluator = KingSafetyEvaluator::with_config(config);

        let mut board = BitboardBoard::empty();
        board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(8, 4),
        );

        let score = evaluator.evaluate(&board, Player::Black);
        assert!(score.mg < 0);
    }

    #[test]
    fn test_partial_castle_scores_between_full_and_bare() {
        let mut config = KingSafetyConfig::default();
        config.performance_mode = false;
        config.castle_weight = 1.0;
        config.attack_weight = 0.0;
        config.threat_weight = 0.0;
        config.phase_adjustment = 1.0;
        let evaluator = KingSafetyEvaluator::with_config(config);

        // Full castle setup (basic Mino shape)
        let mut full_board = BitboardBoard::empty();
        full_board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(8, 4),
        );
        full_board.place_piece(
            Piece::new(PieceType::Gold, Player::Black),
            Position::new(7, 3),
        );
        full_board.place_piece(
            Piece::new(PieceType::Silver, Player::Black),
            Position::new(6, 3),
        );
        full_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(6, 2),
        );
        full_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(7, 2),
        );
        full_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(8, 2),
        );

        let full_score = evaluator.evaluate(&full_board, Player::Black);
        let recognizer = crate::evaluation::castles::CastleRecognizer::new();
        let full_eval = recognizer.evaluate_castle(&full_board, Player::Black, Position::new(8, 4));

        // Partial castle missing pawn shield
        let mut partial_board = BitboardBoard::empty();
        partial_board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(8, 4),
        );
        partial_board.place_piece(
            Piece::new(PieceType::Gold, Player::Black),
            Position::new(7, 3),
        );
        partial_board.place_piece(
            Piece::new(PieceType::Silver, Player::Black),
            Position::new(6, 3),
        );
        // Only one pawn defending
        partial_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(7, 2),
        );

        assert!(partial_board.get_piece(Position::new(6, 2)).is_none());
        assert!(partial_board.get_piece(Position::new(8, 2)).is_none());

        let partial_score = evaluator.evaluate(&partial_board, Player::Black);
        let partial_eval =
            recognizer.evaluate_castle(&partial_board, Player::Black, Position::new(8, 4));

        assert!(
            full_eval.matched_pattern,
            "full castle should match a pattern"
        );
        assert!(
            partial_eval.matched_pattern,
            "partial castle should still register the base castle"
        );
        assert!(
            full_eval.zone_pawn_wall_ratio > partial_eval.zone_pawn_wall_ratio,
            "missing pawn shield should reduce the pawn wall ratio"
        );
        assert!(
            full_eval.pattern_shield_ratio > partial_eval.pattern_shield_ratio,
            "missing pawn shield should reduce the pattern shield ratio"
        );

        // Bare king
        let mut bare_board = BitboardBoard::empty();
        bare_board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(8, 4),
        );

        let bare_score = evaluator.evaluate(&bare_board, Player::Black);

        assert!(
            full_score.mg > partial_score.mg,
            "full {} <= partial {}",
            full_score.mg,
            partial_score.mg
        );
        assert!(
            partial_score.mg > bare_score.mg,
            "partial {} <= bare {}",
            partial_score.mg,
            bare_score.mg
        );
    }

    #[test]
    fn test_infiltration_penalty_reduces_score() {
        let mut config = KingSafetyConfig::default();
        config.performance_mode = false;
        config.castle_weight = 1.0;
        config.attack_weight = 0.0;
        config.threat_weight = 0.0;
        config.phase_adjustment = 1.0;
        let evaluator = KingSafetyEvaluator::with_config(config);

        let mut protected_board = BitboardBoard::empty();
        let king_pos = Position::new(8, 4);
        protected_board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
        protected_board.place_piece(
            Piece::new(PieceType::Gold, Player::Black),
            Position::new(7, 4),
        );
        protected_board.place_piece(
            Piece::new(PieceType::Silver, Player::Black),
            Position::new(6, 4),
        );
        protected_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(6, 3),
        );
        protected_board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(6, 5),
        );

        let protected_score = evaluator.evaluate(&protected_board, Player::Black);

        let mut contested_board = protected_board.clone();
        contested_board.place_piece(
            Piece::new(PieceType::Knight, Player::White),
            Position::new(7, 3),
        );

        let contested_score = evaluator.evaluate(&contested_board, Player::Black);
        assert!(
            contested_score.mg < protected_score.mg,
            "contested {} >= protected {}",
            contested_score.mg,
            protected_score.mg
        );
    }
}
