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
//! # Component Coordination
//!
//! This module coordinates between evaluation components to avoid double-counting:
//! - **Passed Pawns**: When `endgame_patterns` is enabled and phase < 64, passed pawn evaluation
//!   is skipped in `position_features` to avoid double-counting (endgame patterns handle passed
//!   pawns with endgame-specific bonuses).
//! - **Center Control**: Both `position_features` and `positional_patterns` evaluate center control,
//!   but with different methods. Position features use control maps, while positional patterns use
//!   more sophisticated evaluation including drop pressure and forward bonuses. A warning is logged
//!   when both are enabled.
//! - **King Safety**: `KingSafetyEvaluator` in position_features evaluates general king safety
//!   (shields, attacks, etc.), while `CastleRecognizer` evaluates specific castle formation patterns.
//!   These are complementary and should both be enabled for comprehensive king safety evaluation.
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
use crate::debug_utils::debug_log;
use crate::evaluation::{
    castles::CastleRecognizer,
    config::EvaluationWeights,
    endgame_patterns::EndgamePatternEvaluator,
    material::{MaterialEvaluationConfig, MaterialEvaluationStats, MaterialEvaluator},
    opening_principles::OpeningPrincipleEvaluator,
    performance::OptimizedEvaluator,
    phase_transition::PhaseTransition,
    piece_square_tables::PieceSquareTables,
    position_features::{PositionFeatureConfig, PositionFeatureEvaluator},
    positional_patterns::PositionalPatternAnalyzer,
    pst_loader::{PieceSquareTableConfig, PieceSquareTableLoader},
    statistics::{EvaluationStatistics, EvaluationTelemetry, PieceSquareTelemetry},
    tactical_patterns::{TacticalConfig, TacticalPatternRecognizer},
    tapered_eval::TaperedEvaluation,
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
    /// Evaluation weighting configuration
    weights: EvaluationWeights,
    /// Endgame patterns (uses interior mutability)
    endgame_patterns: RefCell<EndgamePatternEvaluator>,
    /// Opening principles (uses interior mutability)
    opening_principles: RefCell<OpeningPrincipleEvaluator>,
    /// Tactical pattern recognizer (Phase 2 - Task 2.1)
    tactical_patterns: RefCell<TacticalPatternRecognizer>,
    /// Positional pattern analyzer (Phase 2 - Task 2.2)
    positional_patterns: RefCell<PositionalPatternAnalyzer>,
    /// Castle pattern recognizer (Task 17.0 - Task 1.0)
    castle_recognizer: RefCell<CastleRecognizer>,
    /// Optimized evaluator (for performance mode)
    // Note: Pattern caching is handled per-module. Individual pattern recognizers
    // (CastleRecognizer, TacticalPatternRecognizer, etc.) maintain their own internal
    // caches optimized for their specific needs. A unified pattern cache was considered
    // but removed as unused - each module's cache is more efficient for its use case.
    optimized_eval: Option<OptimizedEvaluator>,
    /// Statistics tracker (uses interior mutability)
    statistics: RefCell<EvaluationStatistics>,
    /// Latest telemetry snapshot
    telemetry: RefCell<Option<EvaluationTelemetry>>,
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
        let pst_tables = match PieceSquareTableLoader::load(&config.pst) {
            Ok(pst) => pst,
            Err(err) => {
                debug_log(&format!(
                    "[IntegratedEvaluator] Failed to load PST definition: {}. Falling back to built-in tables.",
                    err
                ));
                PieceSquareTables::new()
            }
        };

        let optimized_eval = if config.use_optimized_path {
            Some(OptimizedEvaluator::with_components(
                &config.material,
                pst_tables.clone(),
            ))
        } else {
            None
        };

        let evaluator = Self {
            config: config.clone(),
            tapered_eval: RefCell::new(TaperedEvaluation::new()),
            material_eval: RefCell::new(MaterialEvaluator::with_config(config.material.clone())),
            pst: pst_tables,
            phase_transition: RefCell::new(PhaseTransition::new()),
            position_features: RefCell::new(PositionFeatureEvaluator::with_config(
                config.position_features.clone(),
            )),
            weights: config.weights.clone(),
            endgame_patterns: RefCell::new(EndgamePatternEvaluator::new()),
            opening_principles: RefCell::new(OpeningPrincipleEvaluator::new()),
            tactical_patterns: RefCell::new(TacticalPatternRecognizer::with_config(
                config.tactical.clone(),
            )),
            positional_patterns: RefCell::new(PositionalPatternAnalyzer::new()),
            castle_recognizer: RefCell::new(CastleRecognizer::new()),
            optimized_eval,
            statistics: RefCell::new(EvaluationStatistics::new()),
            telemetry: RefCell::new(None),
            phase_cache: RefCell::new(HashMap::new()),
            eval_cache: RefCell::new(HashMap::new()),
        };

        evaluator
            .statistics
            .borrow_mut()
            .set_collect_position_feature_stats(config.collect_position_feature_stats);

        evaluator
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
        let stats_enabled = { self.statistics.borrow().is_enabled() };
        // Calculate phase
        let phase = self.calculate_phase_cached(board, captured_pieces);

        // Apply phase-dependent weight scaling if enabled
        let mut weights = self.weights.clone();
        if self.config.enable_phase_dependent_weights {
            // Create a temporary TaperedEvalConfig to use its phase scaling method
            let mut temp_config = crate::evaluation::config::TaperedEvalConfig::default();
            temp_config.enable_phase_dependent_weights = true;
            temp_config.apply_phase_scaling(&mut weights, phase);
        }

        // Clamp weights to valid range (0.0-10.0) if needed
        weights.material_weight = weights.material_weight.clamp(0.0, 10.0);
        weights.position_weight = weights.position_weight.clamp(0.0, 10.0);
        weights.king_safety_weight = weights.king_safety_weight.clamp(0.0, 10.0);
        weights.pawn_structure_weight = weights.pawn_structure_weight.clamp(0.0, 10.0);
        weights.mobility_weight = weights.mobility_weight.clamp(0.0, 10.0);
        weights.center_control_weight = weights.center_control_weight.clamp(0.0, 10.0);
        weights.development_weight = weights.development_weight.clamp(0.0, 10.0);
        weights.tactical_weight = weights.tactical_weight.clamp(0.0, 10.0);
        weights.positional_weight = weights.positional_weight.clamp(0.0, 10.0);
        weights.castle_weight = weights.castle_weight.clamp(0.0, 10.0);

        // Accumulate component scores
        let mut total = TaperedScore::default();
        let mut pst_telemetry: Option<PieceSquareTelemetry> = None;
        let mut position_feature_stats_snapshot = None;
        let mut tactical_snapshot = None;
        let mut positional_snapshot = None;
        let mut castle_cache_stats = None;

        // Material
        if self.config.components.material {
            total +=
                self.material_eval
                    .borrow_mut()
                    .evaluate_material(board, player, captured_pieces);
        }

        // Piece-square tables
        if self.config.components.piece_square_tables {
            let (pst_score, telemetry) = self.evaluate_pst(board, player);
            total += pst_score;
            pst_telemetry = Some(telemetry);
        }

        // Position features
        // Coordination: Skip passed pawn evaluation in position features when endgame patterns
        // are enabled and we're in endgame (phase < 64) to avoid double-counting.
        // Endgame patterns handle passed pawns with endgame-specific bonuses.
        let skip_passed_pawn_evaluation = self.config.components.endgame_patterns && phase < 64;

        // Coordination warning: Center control is evaluated in both position_features and
        // positional_patterns. This is intentional overlap but should be monitored.
        // Position features use control maps, while positional patterns use more sophisticated
        // center evaluation including drop pressure and forward bonuses.
        if self.config.components.position_features
            && self.config.components.positional_patterns
        {
            debug_log(&format!(
                "WARNING: Both position_features.center_control and positional_patterns are enabled. \
                Center control will be evaluated twice (with different methods). \
                Consider disabling one to avoid potential double-counting."
            ));
        }

        if self.config.components.position_features {
            let mut position_features = self.position_features.borrow_mut();
            position_features.begin_evaluation(board);
            
            // King safety
            let king_safety_score = position_features.evaluate_king_safety(board, player, captured_pieces);
            let contribution = (king_safety_score.interpolate(phase) as f32) * weights.king_safety_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large king_safety contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    king_safety_score.interpolate(phase),
                    weights.king_safety_weight,
                    contribution
                ));
            }
            total += king_safety_score * weights.king_safety_weight;
            
            // Pawn structure
            let pawn_score = position_features
                .evaluate_pawn_structure(board, player, captured_pieces, skip_passed_pawn_evaluation);
            let contribution = (pawn_score.interpolate(phase) as f32) * weights.pawn_structure_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large pawn_structure contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    pawn_score.interpolate(phase),
                    weights.pawn_structure_weight,
                    contribution
                ));
            }
            total += pawn_score * weights.pawn_structure_weight;
            
            // Mobility
            let mobility_score = position_features.evaluate_mobility(board, player, captured_pieces);
            let contribution = (mobility_score.interpolate(phase) as f32) * weights.mobility_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large mobility contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    mobility_score.interpolate(phase),
                    weights.mobility_weight,
                    contribution
                ));
            }
            total += mobility_score * weights.mobility_weight;
            
            // Center control
            // Note: skip_center_control is set to false for now (future use when positional patterns
            // fully replace position_features center control)
            let center_score = position_features
                .evaluate_center_control(board, player, false);
            let contribution = (center_score.interpolate(phase) as f32) * weights.center_control_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large center_control contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    center_score.interpolate(phase),
                    weights.center_control_weight,
                    contribution
                ));
            }
            total += center_score * weights.center_control_weight;
            
            // Development
            let dev_score = position_features.evaluate_development(board, player);
            let contribution = (dev_score.interpolate(phase) as f32) * weights.development_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large development contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    dev_score.interpolate(phase),
                    weights.development_weight,
                    contribution
                ));
            }
            total += dev_score * weights.development_weight;
            
            if stats_enabled && self.config.collect_position_feature_stats {
                position_feature_stats_snapshot = Some(position_features.stats().clone());
            }
            position_features.end_evaluation();
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
            let tactical_score = {
                let mut tactical = self.tactical_patterns.borrow_mut();
                let score = tactical.evaluate_tactics(board, player, captured_pieces);
                tactical_snapshot = Some(tactical.stats().snapshot());
                score
            };
            let contribution = (tactical_score.interpolate(phase) as f32) * weights.tactical_weight;
            // Log large contributions (Task 3.0 - Task 3.12)
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large tactical contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp (threshold={:.1})",
                    tactical_score.interpolate(phase),
                    weights.tactical_weight,
                    contribution,
                    self.config.weight_contribution_threshold
                ));
            }
            total += tactical_score * weights.tactical_weight;
        }

        // Positional patterns (Phase 3 - Task 3.1 Integration)
        if self.config.components.positional_patterns {
            let positional_score = {
                let mut positional = self.positional_patterns.borrow_mut();
                let score = positional.evaluate_position(board, player, captured_pieces);
                if stats_enabled {
                    positional_snapshot = Some(positional.stats().snapshot());
                }
                score
            };
            let contribution = (positional_score.interpolate(phase) as f32) * weights.positional_weight;
            // Log large contributions (Task 3.0 - Task 3.12)
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large positional contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp (threshold={:.1})",
                    positional_score.interpolate(phase),
                    weights.positional_weight,
                    contribution,
                    self.config.weight_contribution_threshold
                ));
            }
            total += positional_score * weights.positional_weight;
        }

        // Castle patterns (Task 17.0 - Task 1.0 Integration)
        // Note: Castle patterns are now separate from king safety evaluation.
        // KingSafetyEvaluator in position_features evaluates general king safety (shields, attacks, etc.),
        // while CastleRecognizer evaluates specific castle formation patterns.
        // These are complementary and should both be enabled for comprehensive king safety evaluation.
        if self.config.components.castle_patterns {
            let castle_score = {
                let mut castle = self.castle_recognizer.borrow_mut();
                // Find king position for castle evaluation
                if let Some(king_pos) = board.find_king_position(player) {
                    let eval = castle.evaluate_castle(board, player, king_pos);
                    if stats_enabled {
                        castle_cache_stats = Some(castle.get_cache_stats());
                    }
                    eval.score()
                } else {
                    TaperedScore::default()
                }
            };
            let contribution = (castle_score.interpolate(phase) as f32) * weights.castle_weight;
            // Log large contributions (Task 3.0 - Task 3.12)
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large castle contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp (threshold={:.1})",
                    castle_score.interpolate(phase),
                    weights.castle_weight,
                    contribution,
                    self.config.weight_contribution_threshold
                ));
            }
            total += castle_score * weights.castle_weight;
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

        let tapered_snapshot = {
            let evaluator = self.tapered_eval.borrow();
            evaluator.stats().snapshot()
        };
        let phase_snapshot = {
            let transition = self.phase_transition.borrow();
            transition.stats().snapshot()
        };
        let performance_snapshot = self.optimized_eval.as_ref().and_then(|opt| {
            let profiler = opt.profiler();
            if profiler.enabled {
                Some(profiler.report())
            } else {
                None
            }
        });
        let material_snapshot = {
            let mut material_eval = self.material_eval.borrow_mut();
            material_eval.stats_mut().record_phase_weighted(final_score);
            material_eval.stats().snapshot()
        };

        let telemetry = EvaluationTelemetry::from_snapshots(
            tapered_snapshot,
            phase_snapshot,
            performance_snapshot,
            Some(material_snapshot),
            pst_telemetry.clone(),
            position_feature_stats_snapshot.clone(),
            positional_snapshot.clone(),
            tactical_snapshot.clone(),
            None, // King safety stats not integrated into IntegratedEvaluator yet
            castle_cache_stats.clone(),
        );
        self.telemetry.borrow_mut().replace(telemetry.clone());
        if stats_enabled {
            if let Some(stats) = position_feature_stats_snapshot {
                self.statistics
                    .borrow_mut()
                    .record_position_feature_stats(stats);
            }
            if let Some(stats) = positional_snapshot {
                self.statistics.borrow_mut().record_positional_stats(stats);
            }
            self.statistics.borrow_mut().update_telemetry(telemetry);
        }

        final_score
    }

    /// Evaluate piece-square tables
    fn evaluate_pst(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> (TaperedScore, PieceSquareTelemetry) {
        let mut score = TaperedScore::default();
        let mut per_piece = [TaperedScore::default(); PieceType::COUNT];

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let pst_value = self.pst.get_value(piece.piece_type, pos, piece.player);
                    let idx = piece.piece_type.as_index();

                    if piece.player == player {
                        score += pst_value;
                        per_piece[idx] += pst_value;
                    } else {
                        score -= pst_value;
                        per_piece[idx] -= pst_value;
                    }
                }
            }
        }

        let telemetry = PieceSquareTelemetry::from_contributions(score, &per_piece);
        (score, telemetry)
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
    fn compute_phase_hash(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> u64 {
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
        self.telemetry.borrow_mut().take();
    }

    /// Get the latest telemetry snapshot (cloned).
    pub fn telemetry_snapshot(&self) -> Option<EvaluationTelemetry> {
        self.telemetry.borrow().clone()
    }

    /// Get current configuration
    pub fn config(&self) -> &IntegratedEvaluationConfig {
        &self.config
    }

    /// Update only the material evaluation configuration.
    pub fn update_material_config(&mut self, material_config: MaterialEvaluationConfig) {
        let mut updated = self.config.clone();
        updated.material = material_config;
        self.set_config(updated);
    }

    /// Update the tactical pattern configuration.
    pub fn update_tactical_config(&mut self, tactical_config: TacticalConfig) {
        let mut updated = self.config.clone();
        updated.tactical = tactical_config;
        self.set_config(updated);
    }

    /// Retrieve material evaluation statistics.
    pub fn material_statistics(&self) -> MaterialEvaluationStats {
        self.material_eval.borrow().stats().clone()
    }

    /// Update configuration
    pub fn set_config(&mut self, config: IntegratedEvaluationConfig) {
        self.config = config.clone();

        {
            let mut material_eval = self.material_eval.borrow_mut();
            material_eval.apply_config(config.material.clone());
        }

        {
            let mut position_features = self.position_features.borrow_mut();
            position_features.set_config(config.position_features.clone());
        }

        {
            let mut tactical = self.tactical_patterns.borrow_mut();
            tactical.set_config(config.tactical.clone());
        }

        self.weights = config.weights.clone();

        let pst_tables = match PieceSquareTableLoader::load(&config.pst) {
            Ok(pst) => pst,
            Err(err) => {
                debug_log(&format!(
                    "[IntegratedEvaluator] Failed to load PST definition: {}. Continuing with previous tables.",
                    err
                ));
                self.pst.clone()
            }
        };

        self.pst = pst_tables.clone();

        if config.use_optimized_path {
            match self.optimized_eval.as_mut() {
                Some(opt) => {
                    opt.apply_material_config(&config.material);
                    opt.apply_piece_square_tables(pst_tables.clone());
                }
                None => {
                    self.optimized_eval = Some(OptimizedEvaluator::with_components(
                        &config.material,
                        pst_tables.clone(),
                    ));
                }
            }
        } else {
            self.optimized_eval = None;
        }

        self.clear_caches();
        {
            let mut stats = self.statistics.borrow_mut();
            stats.reset();
            stats.set_collect_position_feature_stats(config.collect_position_feature_stats);
        }
        self.telemetry.borrow_mut().take();
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
    /// Collect position feature statistics for telemetry
    // Note: Pattern caching is handled per-module. Individual pattern recognizers
    // maintain their own internal caches. No unified pattern cache size configuration
    // is needed as each module manages its own cache size.
    pub collect_position_feature_stats: bool,
    /// Material evaluation configuration
    pub material: MaterialEvaluationConfig,
    /// Piece-square table configuration
    pub pst: PieceSquareTableConfig,
    /// Position feature configuration
    pub position_features: PositionFeatureConfig,
    /// Tactical pattern configuration
    pub tactical: TacticalConfig,
    /// Evaluation weights for combining features
    pub weights: EvaluationWeights,
    /// Enable phase-dependent weight scaling (default: false for backward compatibility)
    pub enable_phase_dependent_weights: bool,
    /// Threshold for logging large weight contributions in centipawns (default: 1000.0)
    pub weight_contribution_threshold: f32,
}

impl Default for IntegratedEvaluationConfig {
    fn default() -> Self {
        Self {
            components: ComponentFlags::all_enabled(),
            enable_phase_cache: true,
            enable_eval_cache: true,
            use_optimized_path: true,
            max_cache_size: 10000,
            collect_position_feature_stats: true,
            material: MaterialEvaluationConfig::default(),
            pst: PieceSquareTableConfig::default(),
            position_features: PositionFeatureConfig::default(),
            tactical: TacticalConfig::default(),
            weights: EvaluationWeights::default(),
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
        }
    }
}

impl IntegratedEvaluationConfig {
    /// Validate cumulative weights for enabled components
    pub fn validate_cumulative_weights(&self) -> Result<(), crate::evaluation::config::ConfigError> {
        use crate::evaluation::config::ComponentFlagsForValidation;
        
        let components = ComponentFlagsForValidation {
            material: self.components.material,
            piece_square_tables: self.components.piece_square_tables,
            position_features: self.components.position_features,
            tactical_patterns: self.components.tactical_patterns,
            positional_patterns: self.components.positional_patterns,
            castle_patterns: self.components.castle_patterns,
        };
        
        // Create a temporary TaperedEvalConfig to use its validation method
        // We only need the weights, so we can create a minimal config
        let mut temp_config = crate::evaluation::config::TaperedEvalConfig::default();
        temp_config.weights = self.weights.clone();
        
        temp_config.validate_cumulative_weights(&components)
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
    pub castle_patterns: bool,
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
            castle_patterns: true,
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
            castle_patterns: false,
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
            castle_patterns: false,
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
        assert!(all_enabled.castle_patterns);

        let all_disabled = ComponentFlags::all_disabled();
        assert!(!all_disabled.material);
        assert!(!all_disabled.piece_square_tables);
        assert!(!all_disabled.castle_patterns);

        let minimal = ComponentFlags::minimal();
        assert!(minimal.material);
        assert!(!minimal.opening_principles);
        assert!(!minimal.castle_patterns);
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

        let (score, telemetry) = evaluator.evaluate_pst(&board, Player::Black);

        // Should have some PST value
        assert!(score.mg != 0 || score.eg != 0);
        assert_eq!(telemetry.total_mg, score.mg);
        assert_eq!(telemetry.total_eg, score.eg);
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
