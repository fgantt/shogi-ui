//! Tapered Evaluation Integration Module
//!
//! This module integrates all tapered evaluation components into a unified
//! evaluation system that can be used by the search algorithm.
//!
//! # Overview
//!
//! The integration provides:
//! - Unified evaluation interface
//! - Phase calculation and caching
//! - Component composition (material, PST, patterns, etc.)
//! - Performance monitoring
//! - Statistics tracking
//! - Configuration management
//! - Backward compatibility
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::integration::IntegratedEvaluator;
//!
//! let mut evaluator = IntegratedEvaluator::new();
//! evaluator.enable_statistics();
//!
//! let score = evaluator.evaluate(&board, player, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::evaluation::{
    endgame_patterns::EndgamePatternEvaluator, material::MaterialEvaluator,
    opening_principles::OpeningPrincipleEvaluator, pattern_cache::PatternCache,
    performance::OptimizedEvaluator, phase_transition::PhaseTransition,
    piece_square_tables::PieceSquareTables, position_features::PositionFeatureEvaluator,
    positional_patterns::PositionalPatternAnalyzer, statistics::EvaluationStatistics,
    tactical_patterns::TacticalPatternRecognizer, tapered_eval::TaperedEvaluation,
};
use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

/// Integrated tapered evaluator
pub struct IntegratedEvaluator {
    /// Configuration
    config: IntegratedEvaluationConfig,
    /// Core tapered evaluation (uses interior mutability)
    tapered_eval: RefCell<TaperedEvaluation>,
    /// Material evaluator (uses interior mutability)
    material_eval: RefCell<MaterialEvaluator>,
    /// Piece-square tables
    pst: PieceSquareTables,
    /// Phase transition (uses interior mutability)
    phase_transition: RefCell<PhaseTransition>,
    /// Position features (uses interior mutability)
    position_features: RefCell<PositionFeatureEvaluator>,
    /// Endgame patterns (uses interior mutability)
    endgame_patterns: RefCell<EndgamePatternEvaluator>,
    /// Opening principles (uses interior mutability)
    opening_principles: RefCell<OpeningPrincipleEvaluator>,
    /// Tactical pattern recognizer (Phase 2 - Task 2.1)
    tactical_patterns: RefCell<TacticalPatternRecognizer>,
    /// Positional pattern analyzer (Phase 2 - Task 2.2)
    positional_patterns: RefCell<PositionalPatternAnalyzer>,
    /// Pattern result cache (Phase 2 - Task 2.4, reserved for future optimization)
    #[allow(dead_code)]
    pattern_cache: RefCell<PatternCache>,
    /// Optimized evaluator (for performance mode)
    optimized_eval: Option<OptimizedEvaluator>,
    /// Statistics tracker (uses interior mutability)
    statistics: RefCell<EvaluationStatistics>,
    /// Phase cache (uses interior mutability)
    phase_cache: RefCell<HashMap<u64, i32>>,
    /// Evaluation cache (uses interior mutability)
    eval_cache: RefCell<HashMap<u64, CachedEvaluation>>,
}

impl IntegratedEvaluator {
    /// Create a new integrated evaluator with default configuration
    pub fn new() -> Self {
        Self::with_config(IntegratedEvaluationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: IntegratedEvaluationConfig) -> Self {
        let optimized_eval = if config.use_optimized_path {
            Some(OptimizedEvaluator::new())
        } else {
            None
        };

        Self {
            config: config.clone(),
            tapered_eval: RefCell::new(TaperedEvaluation::new()),
            material_eval: RefCell::new(MaterialEvaluator::new()),
            pst: PieceSquareTables::new(),
            phase_transition: RefCell::new(PhaseTransition::new()),
            position_features: RefCell::new(PositionFeatureEvaluator::new()),
            endgame_patterns: RefCell::new(EndgamePatternEvaluator::new()),
            opening_principles: RefCell::new(OpeningPrincipleEvaluator::new()),
            tactical_patterns: RefCell::new(TacticalPatternRecognizer::new()),
            positional_patterns: RefCell::new(PositionalPatternAnalyzer::new()),
            pattern_cache: RefCell::new(PatternCache::new(config.pattern_cache_size)),
            optimized_eval,
            statistics: RefCell::new(EvaluationStatistics::new()),
            phase_cache: RefCell::new(HashMap::new()),
            eval_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Main evaluation entry point
    pub fn evaluate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        let start = if self.statistics.borrow().is_enabled() {
            Some(Instant::now())
        } else {
            None
        };

        // Check cache if enabled
        if self.config.enable_eval_cache {
            let hash = self.compute_position_hash(board, player, captured_pieces);
            if let Some(cached) = self.eval_cache.borrow().get(&hash).copied() {
                if self.statistics.borrow().is_enabled() {
                    self.statistics
                        .borrow_mut()
                        .record_evaluation(cached.score, cached.phase);
                }
                return cached.score;
            }
        }

        // Use standard path (optimized path would require &mut self)
        let score = self.evaluate_standard(board, player, captured_pieces);

        // Record statistics
        if let Some(start_time) = start {
            let duration = start_time.elapsed().as_nanos() as u64;
            self.statistics.borrow_mut().record_timing(duration);

            if self.config.enable_phase_cache {
                let phase = self.calculate_phase_cached(board, captured_pieces);
                self.statistics.borrow_mut().record_evaluation(score, phase);
            }
        }

        score
    }

    /// Standard evaluation path (all components)
    fn evaluate_standard(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        // Calculate phase
        let phase = self.calculate_phase_cached(board, captured_pieces);

        // Accumulate component scores
        let mut total = TaperedScore::default();

        // Material
        if self.config.components.material {
            total +=
                self.material_eval
                    .borrow_mut()
                    .evaluate_material(board, player, captured_pieces);
        }

        // Piece-square tables
        if self.config.components.piece_square_tables {
            total += self.evaluate_pst(board, player);
        }

        // Position features
        if self.config.components.position_features {
            total += self
                .position_features
                .borrow_mut()
                .evaluate_king_safety(board, player);
            total += self
                .position_features
                .borrow_mut()
                .evaluate_pawn_structure(board, player);
            total += self.position_features.borrow_mut().evaluate_mobility(
                board,
                player,
                captured_pieces,
            );
            total += self
                .position_features
                .borrow_mut()
                .evaluate_center_control(board, player);
            total += self
                .position_features
                .borrow_mut()
                .evaluate_development(board, player);
        }

        // Opening principles (if in opening)
        if self.config.components.opening_principles && phase >= 192 {
            total += self
                .opening_principles
                .borrow_mut()
                .evaluate_opening(board, player, 0);
        }

        // Endgame patterns (if in endgame)
        if self.config.components.endgame_patterns && phase < 64 {
            total +=
                self.endgame_patterns
                    .borrow_mut()
                    .evaluate_endgame(board, player, captured_pieces);
        }

        // Tactical patterns (Phase 3 - Task 3.1 Integration)
        if self.config.components.tactical_patterns {
            total += self
                .tactical_patterns
                .borrow_mut()
                .evaluate_tactics(board, player);
        }

        // Positional patterns (Phase 3 - Task 3.1 Integration)
        if self.config.components.positional_patterns {
            total += self
                .positional_patterns
                .borrow_mut()
                .evaluate_position(board, player);
        }

        // Interpolate to final score
        let final_score = self
            .phase_transition
            .borrow_mut()
            .interpolate_default(total, phase);

        // Cache if enabled
        if self.config.enable_eval_cache {
            let hash = self.compute_position_hash(board, player, captured_pieces);
            self.eval_cache.borrow_mut().insert(
                hash,
                CachedEvaluation {
                    score: final_score,
                    phase,
                },
            );
        }

        final_score
    }

    /// Evaluate piece-square tables
    fn evaluate_pst(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let pst_value = self.pst.get_value(piece.piece_type, pos, piece.player);

                    if piece.player == player {
                        score += pst_value;
                    } else {
                        score -= pst_value;
                    }
                }
            }
        }

        score
    }

    /// Calculate phase with caching
    fn calculate_phase_cached(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        if !self.config.enable_phase_cache {
            return self
                .tapered_eval
                .borrow_mut()
                .calculate_game_phase(board, captured_pieces);
        }

        let hash = self.compute_phase_hash(board, captured_pieces);

        if let Some(&phase) = self.phase_cache.borrow().get(&hash) {
            return phase;
        }

        let phase = self
            .tapered_eval
            .borrow_mut()
            .calculate_game_phase(board, captured_pieces);
        self.phase_cache.borrow_mut().insert(hash, phase);
        phase
    }

    /// Compute position hash for caching
    fn compute_position_hash(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        // Simple hash - in production, use Zobrist hashing
        let mut hash = 0u64;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_hash = (piece.piece_type as u64) << 16
                        | (piece.player as u64) << 8
                        | (row as u64) << 4
                        | (col as u64);
                    hash ^= piece_hash.wrapping_mul(0x9e3779b97f4a7c15);
                }
            }
        }

        let mut captured_counts = [[0u8; 14]; 2];

        for &piece in &captured_pieces.black {
            let idx = piece.to_u8() as usize;
            captured_counts[0][idx] = captured_counts[0][idx].saturating_add(1);
        }

        for &piece in &captured_pieces.white {
            let idx = piece.to_u8() as usize;
            captured_counts[1][idx] = captured_counts[1][idx].saturating_add(1);
        }

        for (player_idx, counts) in captured_counts.iter().enumerate() {
            for (piece_idx, count) in counts.iter().enumerate() {
                if *count > 0 {
                    let token =
                        ((player_idx as u64) << 48) ^ ((piece_idx as u64) << 8) ^ (*count as u64);
                    hash ^= token.wrapping_mul(0x94d049bb133111eb);
                }
            }
        }

        hash ^= (player as u64).wrapping_mul(0x517cc1b727220a95);
        hash
    }

    /// Compute phase hash (material-based)
    fn compute_phase_hash(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        let mut hash = 0u64;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_hash = (piece.piece_type as u64) << 4 | (piece.player as u64);
                    hash ^= piece_hash.wrapping_mul(0x9e3779b97f4a7c15);
                }
            }
        }

        let mut captured_counts = [[0u8; 14]; 2];

        for &piece in &captured_pieces.black {
            let idx = piece.to_u8() as usize;
            captured_counts[0][idx] = captured_counts[0][idx].saturating_add(1);
        }

        for &piece in &captured_pieces.white {
            let idx = piece.to_u8() as usize;
            captured_counts[1][idx] = captured_counts[1][idx].saturating_add(1);
        }

        for (player_idx, counts) in captured_counts.iter().enumerate() {
            for (piece_idx, count) in counts.iter().enumerate() {
                if *count > 0 {
                    let token =
                        ((player_idx as u64) << 32) ^ ((piece_idx as u64) << 4) ^ (*count as u64);
                    hash ^= token.wrapping_mul(0x9e3779b97f4a7c15);
                }
            }
        }

        hash
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.phase_cache.borrow_mut().clear();
        self.eval_cache.borrow_mut().clear();
    }

    /// Enable statistics tracking
    pub fn enable_statistics(&self) {
        self.statistics.borrow_mut().enable();
    }

    /// Disable statistics tracking
    pub fn disable_statistics(&self) {
        self.statistics.borrow_mut().disable();
    }

    /// Get statistics report (creates a clone)
    pub fn get_statistics(&self) -> EvaluationStatistics {
        self.statistics.borrow().clone()
    }

    /// Reset statistics
    pub fn reset_statistics(&self) {
        self.statistics.borrow_mut().reset();
    }

    /// Get current configuration
    pub fn config(&self) -> &IntegratedEvaluationConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: IntegratedEvaluationConfig) {
        self.config = config;

        // Recreate optimized evaluator if needed
        if self.config.use_optimized_path && self.optimized_eval.is_none() {
            self.optimized_eval = Some(OptimizedEvaluator::new());
        } else if !self.config.use_optimized_path {
            self.optimized_eval = None;
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStatistics {
        CacheStatistics {
            phase_cache_size: self.phase_cache.borrow().len(),
            eval_cache_size: self.eval_cache.borrow().len(),
            phase_cache_enabled: self.config.enable_phase_cache,
            eval_cache_enabled: self.config.enable_eval_cache,
        }
    }
}

impl Default for IntegratedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for integrated evaluator
#[derive(Debug, Clone)]
pub struct IntegratedEvaluationConfig {
    /// Component flags
    pub components: ComponentFlags,
    /// Enable phase caching
    pub enable_phase_cache: bool,
    /// Enable evaluation caching
    pub enable_eval_cache: bool,
    /// Use optimized evaluation path
    pub use_optimized_path: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Pattern cache size (Phase 3 - Task 3.1)
    pub pattern_cache_size: usize,
}

impl Default for IntegratedEvaluationConfig {
    fn default() -> Self {
        Self {
            components: ComponentFlags::all_enabled(),
            enable_phase_cache: true,
            enable_eval_cache: true,
            use_optimized_path: true,
            max_cache_size: 10000,
            pattern_cache_size: 100_000,
        }
    }
}

/// Component enable/disable flags
#[derive(Debug, Clone)]
pub struct ComponentFlags {
    pub material: bool,
    pub piece_square_tables: bool,
    pub position_features: bool,
    pub opening_principles: bool,
    pub endgame_patterns: bool,
    pub tactical_patterns: bool,
    pub positional_patterns: bool,
}

impl ComponentFlags {
    pub fn all_enabled() -> Self {
        Self {
            material: true,
            piece_square_tables: true,
            position_features: true,
            opening_principles: true,
            endgame_patterns: true,
            tactical_patterns: true,
            positional_patterns: true,
        }
    }

    pub fn all_disabled() -> Self {
        Self {
            material: false,
            piece_square_tables: false,
            position_features: false,
            opening_principles: false,
            endgame_patterns: false,
            tactical_patterns: false,
            positional_patterns: false,
        }
    }

    pub fn minimal() -> Self {
        Self {
            material: true,
            piece_square_tables: true,
            position_features: false,
            opening_principles: false,
            endgame_patterns: false,
            tactical_patterns: false,
            positional_patterns: false,
        }
    }
}

/// Cached evaluation entry
#[derive(Debug, Clone, Copy)]
struct CachedEvaluation {
    score: i32,
    phase: i32,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub phase_cache_size: usize,
    pub eval_cache_size: usize,
    pub phase_cache_enabled: bool,
    pub eval_cache_enabled: bool,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = IntegratedEvaluator::new();
        assert!(evaluator.config.components.material);
        assert!(evaluator.config.enable_phase_cache);
    }

    #[test]
    fn test_basic_evaluation() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        // Should return a valid score
        assert!(score.abs() < 100000);
    }

    #[test]
    fn test_evaluation_caching() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // First evaluation
        let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        // Second evaluation (should be cached)
        let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        assert_eq!(score1, score2);
        assert!(evaluator.eval_cache.len() > 0);
    }

    #[test]
    fn test_phase_caching() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let phase1 = evaluator.calculate_phase_cached(&board, &captured_pieces);
        let phase2 = evaluator.calculate_phase_cached(&board, &captured_pieces);

        assert_eq!(phase1, phase2);
        assert!(evaluator.phase_cache.len() > 0);
    }

    #[test]
    fn test_clear_caches() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(evaluator.eval_cache.len() > 0);

        evaluator.clear_caches();
        assert_eq!(evaluator.eval_cache.len(), 0);
        assert_eq!(evaluator.phase_cache.len(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut evaluator = IntegratedEvaluator::new();
        evaluator.enable_statistics();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate(&board, Player::Black, &captured_pieces);

        let stats = evaluator.get_statistics();
        assert_eq!(stats.count(), 1);
    }

    #[test]
    fn test_component_flags() {
        let all_enabled = ComponentFlags::all_enabled();
        assert!(all_enabled.material);
        assert!(all_enabled.piece_square_tables);

        let all_disabled = ComponentFlags::all_disabled();
        assert!(!all_disabled.material);
        assert!(!all_disabled.piece_square_tables);

        let minimal = ComponentFlags::minimal();
        assert!(minimal.material);
        assert!(!minimal.opening_principles);
    }

    #[test]
    fn test_config_update() {
        let mut evaluator = IntegratedEvaluator::new();

        let mut config = IntegratedEvaluationConfig::default();
        config.use_optimized_path = false;

        evaluator.set_config(config);

        assert!(!evaluator.config.use_optimized_path);
        assert!(evaluator.optimized_eval.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate(&board, Player::Black, &captured_pieces);

        let stats = evaluator.cache_stats();
        assert!(stats.phase_cache_enabled);
        assert!(stats.eval_cache_enabled);
        assert!(stats.eval_cache_size > 0);
    }

    #[test]
    fn test_optimized_path() {
        let mut config = IntegratedEvaluationConfig::default();
        config.use_optimized_path = true;

        let mut evaluator = IntegratedEvaluator::with_config(config);
        assert!(evaluator.optimized_eval.is_some());

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(score.abs() < 100000);
    }

    #[test]
    fn test_standard_path() {
        let mut config = IntegratedEvaluationConfig::default();
        config.use_optimized_path = false;

        let mut evaluator = IntegratedEvaluator::with_config(config);
        assert!(evaluator.optimized_eval.is_none());

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(score.abs() < 100000);
    }

    #[test]
    fn test_pst_evaluation() {
        let evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();

        let score = evaluator.evaluate_pst(&board, Player::Black);

        // Should have some PST value
        assert!(score.mg != 0 || score.eg != 0);
    }

    #[test]
    fn test_position_hash() {
        let evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();

        let hash1 = evaluator.compute_position_hash(&board, Player::Black);
        let hash2 = evaluator.compute_position_hash(&board, Player::Black);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = IntegratedEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        evaluator.clear_caches();
        let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);

        assert_eq!(score1, score2);
    }

    #[test]
    fn test_component_selective_evaluation() {
        let mut config = IntegratedEvaluationConfig::default();
        config.components = ComponentFlags::minimal();
        config.use_optimized_path = false;

        let mut evaluator = IntegratedEvaluator::with_config(config);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        assert!(score.abs() < 100000);
    }
}
