//! Tapered Evaluation Module
//!
//! This module provides a comprehensive tapered evaluation system for the Shogi engine.
//! Tapered evaluation allows different evaluation weights for opening/middlegame and endgame
//! phases, providing more accurate position assessment throughout the game.
//!
//! # Overview
//!
//! The tapered evaluation system consists of:
//! - **TaperedScore**: A dual-phase score with separate middlegame and endgame values
//! - **TaperedEvaluation**: Coordination struct for managing tapered evaluation
//! - **Game Phase Calculation**: Based on material count to determine current game phase
//! - **Interpolation**: Smooth transition between middlegame and endgame scores
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::tapered_eval::TaperedEvaluation;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let mut evaluator = TaperedEvaluation::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::*;

/// Coordination struct for managing tapered evaluation
///
/// This struct provides a high-level interface for coordinating all aspects
/// of tapered evaluation, including phase calculation, score interpolation,
/// and configuration management.
pub struct TaperedEvaluation {
    /// Configuration for tapered evaluation
    config: TaperedEvaluationConfig,
    /// Cached game phase for performance optimization
    cached_phase: Option<(u64, i32)>, // (position_hash, phase)
    /// Statistics for monitoring and tuning
    stats: TaperedEvaluationStats,
}

impl TaperedEvaluation {
    /// Create a new TaperedEvaluation with default configuration
    pub fn new() -> Self {
        Self {
            config: TaperedEvaluationConfig::default(),
            cached_phase: None,
            stats: TaperedEvaluationStats::default(),
        }
    }

    /// Create a new TaperedEvaluation with custom configuration
    pub fn with_config(config: TaperedEvaluationConfig) -> Self {
        Self {
            config,
            cached_phase: None,
            stats: TaperedEvaluationStats::default(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &TaperedEvaluationConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: TaperedEvaluationConfig) {
        self.config = config;
        // Clear cache when configuration changes
        self.cached_phase = None;
    }

    /// Calculate the current game phase based on material count
    ///
    /// # Arguments
    ///
    /// * `board` - The current board state
    ///
    /// # Returns
    ///
    /// Game phase value (0 = endgame, GAME_PHASE_MAX = opening)
    ///
    /// # Performance
    ///
    /// This function uses caching when enabled in configuration to avoid
    /// recalculating the phase for the same position.
    pub fn calculate_game_phase(&mut self, board: &BitboardBoard) -> i32 {
        self.stats.phase_calculations += 1;

        // Check cache if enabled
        if self.config.cache_game_phase {
            let position_hash = self.get_position_hash(board);
            if let Some((cached_hash, cached_phase)) = self.cached_phase {
                if cached_hash == position_hash {
                    self.stats.cache_hits += 1;
                    return cached_phase;
                }
            }
        }

        // Calculate phase based on material
        let phase = self.calculate_phase_from_material(board);

        // Update cache if enabled
        if self.config.cache_game_phase {
            let position_hash = self.get_position_hash(board);
            self.cached_phase = Some((position_hash, phase));
        }

        phase
    }

    /// Calculate game phase from material count
    ///
    /// This is the core phase calculation algorithm. It assigns phase values
    /// to each piece type and sums them to determine the overall game phase.
    fn calculate_phase_from_material(&self, board: &BitboardBoard) -> i32 {
        let mut phase = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if let Some(phase_value) = self.get_piece_phase_value(piece.piece_type) {
                        phase += phase_value;
                    }
                }
            }
        }

        // Scale to 0-256 range
        // Starting position has 30 total phase value (15 per player)
        // We want this to map to GAME_PHASE_MAX (256)
        let scaled_phase = (phase * GAME_PHASE_MAX) / 30;

        // Clamp to valid range
        scaled_phase.min(GAME_PHASE_MAX).max(0)
    }

    /// Get phase value for a piece type
    ///
    /// Returns None for pieces that don't contribute to game phase
    /// (pawns, kings, promoted pieces)
    fn get_piece_phase_value(&self, piece_type: PieceType) -> Option<i32> {
        PIECE_PHASE_VALUES
            .iter()
            .find(|(pt, _)| *pt == piece_type)
            .map(|(_, value)| *value)
    }

    /// Interpolate a tapered score based on game phase
    ///
    /// # Arguments
    ///
    /// * `score` - The tapered score to interpolate
    /// * `phase` - The current game phase (0 = endgame, GAME_PHASE_MAX = opening)
    ///
    /// # Returns
    ///
    /// Interpolated score value
    ///
    /// # Algorithm
    ///
    /// Linear interpolation: `(mg * phase + eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX`
    ///
    /// This provides smooth transitions between game phases without discontinuities.
    pub fn interpolate(&self, score: TaperedScore, phase: i32) -> i32 {
        self.stats
            .interpolations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        score.interpolate(phase)
    }

    /// Create a TaperedScore with equal middlegame and endgame values
    pub fn create_score(&self, value: i32) -> TaperedScore {
        TaperedScore::new(value)
    }

    /// Create a TaperedScore with different middlegame and endgame values
    pub fn create_tapered_score(&self, mg: i32, eg: i32) -> TaperedScore {
        TaperedScore::new_tapered(mg, eg)
    }

    /// Get a simple hash for position caching
    ///
    /// This is a simplified hash for phase caching purposes.
    /// For more sophisticated hashing, use the Zobrist hash system.
    fn get_position_hash(&self, board: &BitboardBoard) -> u64 {
        let mut hash = 0u64;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value =
                        (piece.piece_type.to_u8() as u64) * 100 + (piece.player as u64);
                    hash = hash.wrapping_mul(31).wrapping_add(piece_value);
                }
            }
        }

        hash
    }

    /// Get evaluation statistics
    pub fn stats(&self) -> &TaperedEvaluationStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = TaperedEvaluationStats::default();
    }

    /// Clear the phase cache
    pub fn clear_cache(&mut self) {
        self.cached_phase = None;
    }
}

impl Default for TaperedEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for monitoring tapered evaluation performance
#[derive(Debug, Default)]
pub struct TaperedEvaluationStats {
    /// Number of phase calculations performed
    pub phase_calculations: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of interpolations performed
    pub interpolations: std::sync::atomic::AtomicU64,
}

impl Clone for TaperedEvaluationStats {
    fn clone(&self) -> Self {
        Self {
            phase_calculations: self.phase_calculations,
            cache_hits: self.cache_hits,
            interpolations: std::sync::atomic::AtomicU64::new(
                self.interpolations
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

impl TaperedEvaluationStats {
    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.phase_calculations == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.phase_calculations as f64
        }
    }

    /// Get total interpolations
    pub fn total_interpolations(&self) -> u64 {
        self.interpolations
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tapered_evaluation_creation() {
        let evaluator = TaperedEvaluation::new();
        assert!(evaluator.config().enabled);
    }

    #[test]
    fn test_tapered_evaluation_with_config() {
        let config = TaperedEvaluationConfig::disabled();
        let evaluator = TaperedEvaluation::with_config(config);
        assert!(!evaluator.config().enabled);
    }

    #[test]
    fn test_calculate_game_phase_starting_position() {
        let mut evaluator = TaperedEvaluation::new();
        let board = BitboardBoard::new();

        let phase = evaluator.calculate_game_phase(&board);
        assert_eq!(
            phase, GAME_PHASE_MAX,
            "Starting position should have maximum phase"
        );
    }

    #[test]
    fn test_calculate_game_phase_consistency() {
        let mut evaluator = TaperedEvaluation::new();
        let board = BitboardBoard::new();

        let phase1 = evaluator.calculate_game_phase(&board);
        let phase2 = evaluator.calculate_game_phase(&board);

        assert_eq!(phase1, phase2, "Phase calculation should be consistent");
    }

    #[test]
    fn test_calculate_game_phase_caching() {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        let board = BitboardBoard::new();

        // First call
        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().cache_hits, 0);

        // Second call should hit cache
        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().cache_hits, 1);
    }

    #[test]
    fn test_interpolate_pure_middlegame() {
        let evaluator = TaperedEvaluation::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = evaluator.interpolate(score, GAME_PHASE_MAX);
        assert_eq!(result, 100, "Phase MAX should return mg value");
    }

    #[test]
    fn test_interpolate_pure_endgame() {
        let evaluator = TaperedEvaluation::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = evaluator.interpolate(score, 0);
        assert_eq!(result, 200, "Phase 0 should return eg value");
    }

    #[test]
    fn test_interpolate_middlegame() {
        let evaluator = TaperedEvaluation::new();
        let score = TaperedScore::new_tapered(100, 200);

        let result = evaluator.interpolate(score, GAME_PHASE_MAX / 2);
        assert_eq!(result, 150, "Phase 128 should return average value");
    }

    #[test]
    fn test_smooth_interpolation() {
        let evaluator = TaperedEvaluation::new();
        let score = TaperedScore::new_tapered(100, 200);

        let mut prev_value = evaluator.interpolate(score, 0);

        for phase in 1..=GAME_PHASE_MAX {
            let value = evaluator.interpolate(score, phase);
            let diff = (value - prev_value).abs();

            assert!(
                diff <= 1,
                "Interpolation should be smooth at phase {}: diff = {}",
                phase,
                diff
            );
            prev_value = value;
        }
    }

    #[test]
    fn test_create_score() {
        let evaluator = TaperedEvaluation::new();
        let score = evaluator.create_score(50);

        assert_eq!(score.mg, 50);
        assert_eq!(score.eg, 50);
    }

    #[test]
    fn test_create_tapered_score() {
        let evaluator = TaperedEvaluation::new();
        let score = evaluator.create_tapered_score(100, 200);

        assert_eq!(score.mg, 100);
        assert_eq!(score.eg, 200);
    }

    #[test]
    fn test_stats_tracking() {
        let mut evaluator = TaperedEvaluation::new();
        let board = BitboardBoard::new();

        assert_eq!(evaluator.stats().phase_calculations, 0);

        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().phase_calculations, 1);

        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().phase_calculations, 2);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        let board = BitboardBoard::new();

        // First call - no cache hit
        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().cache_hit_rate(), 0.0);

        // Second call - should hit cache
        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_clear_cache() {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        let board = BitboardBoard::new();

        evaluator.calculate_game_phase(&board);
        assert!(evaluator.cached_phase.is_some());

        evaluator.clear_cache();
        assert!(evaluator.cached_phase.is_none());
    }

    #[test]
    fn test_reset_stats() {
        let mut evaluator = TaperedEvaluation::new();
        let board = BitboardBoard::new();

        evaluator.calculate_game_phase(&board);
        assert_eq!(evaluator.stats().phase_calculations, 1);

        evaluator.reset_stats();
        assert_eq!(evaluator.stats().phase_calculations, 0);
    }

    #[test]
    fn test_config_update_clears_cache() {
        let mut evaluator = TaperedEvaluation::with_config(TaperedEvaluationConfig {
            cache_game_phase: true,
            ..Default::default()
        });
        let board = BitboardBoard::new();

        evaluator.calculate_game_phase(&board);
        assert!(evaluator.cached_phase.is_some());

        let new_config = TaperedEvaluationConfig::default();
        evaluator.set_config(new_config);
        assert!(evaluator.cached_phase.is_none());
    }

    #[test]
    fn test_game_phase_range() {
        let mut evaluator = TaperedEvaluation::new();
        let board = BitboardBoard::new();

        let phase = evaluator.calculate_game_phase(&board);
        assert!(phase >= 0, "Phase should be non-negative");
        assert!(phase <= GAME_PHASE_MAX, "Phase should not exceed maximum");
    }

    #[test]
    fn test_piece_phase_values() {
        let evaluator = TaperedEvaluation::new();

        // Test pieces that contribute to phase
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Knight), Some(1));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Silver), Some(1));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Gold), Some(2));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Bishop), Some(2));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Rook), Some(3));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Lance), Some(1));

        // Test pieces that don't contribute to phase
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Pawn), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::King), None);
    }

    #[test]
    fn test_performance_optimized_config() {
        let config = TaperedEvaluationConfig::performance_optimized();
        let evaluator = TaperedEvaluation::with_config(config);

        assert!(evaluator.config().enabled);
        assert!(evaluator.config().cache_game_phase);
        assert!(evaluator.config().enable_performance_monitoring);
    }

    #[test]
    fn test_memory_optimized_config() {
        let config = TaperedEvaluationConfig::memory_optimized();
        let evaluator = TaperedEvaluation::with_config(config);

        assert!(evaluator.config().enabled);
        assert!(!evaluator.config().cache_game_phase);
        assert_eq!(evaluator.config().memory_pool_size, 100);
    }
}
