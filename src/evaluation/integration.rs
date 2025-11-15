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
//! - **Passed Pawns**: When `endgame_patterns` is enabled and phase < endgame_threshold (default: 64),
//!   passed pawn evaluation is skipped in `position_features` to avoid double-counting (endgame patterns
//!   handle passed pawns with endgame-specific bonuses).
//! - **Center Control**: Both `position_features` and `positional_patterns` evaluate center control,
//!   but with different methods. Position features use control maps, while positional patterns use
//!   more sophisticated evaluation including drop pressure and forward bonuses. When both are enabled,
//!   the `center_control_precedence` configuration option determines which to use:
//!   - `PositionalPatterns` (default): Use positional_patterns evaluation, skip position_features
//!   - `PositionFeatures`: Use position_features evaluation, skip positional_patterns
//!   - `Both`: Evaluate both (not recommended due to double-counting risk)
//! - **Development**: Both `position_features` and `opening_principles` evaluate development,
//!   but opening_principles provides more sophisticated opening-specific evaluation. When `opening_principles`
//!   is enabled and phase >= opening_threshold, development evaluation is automatically skipped in
//!   `position_features` to avoid double-counting.
//! - **King Safety**: `KingSafetyEvaluator` in position_features evaluates general king safety
//!   (shields, attacks, etc.), while `CastleRecognizer` evaluates specific castle formation patterns.
//!   These are complementary and should both be enabled for comprehensive king safety evaluation.
//!
//! # Phase-Aware Gating and Gradual Transitions
//!
//! The evaluator uses phase-aware gating to conditionally evaluate patterns based on game phase:
//! - **Opening Principles**: Evaluated when phase >= opening_threshold (default: 192)
//! - **Endgame Patterns**: Evaluated when phase < endgame_threshold (default: 64)
//!
//! When `enable_gradual_phase_transitions` is enabled, pattern scores are gradually faded out
//! instead of abruptly cut off:
//! - **Opening Principles**: Fade from `opening_fade_start` (default: 192) to `opening_fade_end` (default: 160)
//! - **Endgame Patterns**: Fade from `endgame_fade_start` (default: 80) to `endgame_fade_end` (default: 64)
//!
//! This produces smoother evaluation transitions and avoids sudden score jumps when crossing phase boundaries.
//! Phase boundaries are configurable via `PhaseBoundaryConfig` in `IntegratedEvaluationConfig`.
//!
//! # Phase-Dependent Weight Scaling (Task 20.0 - Task 3.0)
//!
//! When `enable_phase_dependent_weights` is enabled (default: `true`), evaluation weights automatically
//! adjust based on game phase to reflect the changing importance of different evaluation aspects:
//!
//! - **Opening (phase >= 192)**: Development weight is emphasized (1.2x), as piece development matters most
//!   in the opening phase. Pawn structure is de-emphasized (0.8x).
//!
//! - **Middlegame (64 <= phase < 192)**: Tactical patterns (1.2x) and mobility (1.1x) are emphasized, as
//!   tactical opportunities and piece activity are crucial in middlegame play.
//!
//! - **Endgame (phase < 64)**: Positional patterns (1.2x) and pawn structure (1.2x) are emphasized, as
//!   positional factors and pawn structures become decisive in endgame play. Development is de-emphasized (0.6x).
//!
//! Weight scaling supports three curve types for transitions:
//! - **Linear** (default): Smooth linear interpolation between phases
//! - **Sigmoid**: Smooth S-curve transitions for gradual changes
//! - **Step**: Discrete jumps at phase boundaries for abrupt changes
//!
//! Scaling factors are configurable via `phase_scaling_config` in `TaperedEvalConfig`. If `None`, defaults
//! are used which provide good balance across all phases.
//!
//! # Component Dependency Validation and Coordination (Task 20.0 - Task 5.0)
//!
//! The evaluator includes comprehensive component dependency validation to ensure optimal
//! configuration and avoid conflicts:
//!
//! - **Dependency Graph**: Maps component relationships (Conflicts, Complements, Requires, Optional)
//! - **Conflict Detection**: Warns when conflicting components are both enabled (e.g., center control overlap)
//! - **Complement Validation**: Warns when complementary components are not both enabled (e.g., king safety + castle patterns)
//! - **Requirement Validation**: Errors when required components are missing (e.g., endgame patterns requires position features)
//! - **Auto-Resolution**: Optionally automatically resolves conflicts based on precedence rules
//! - **Phase-Aware Validation**: Warns when components are enabled but phase is outside their effective range
//!
//! The dependency graph includes known relationships:
//! - `position_features.center_control` CONFLICTS with `positional_patterns`
//! - `position_features.development` CONFLICTS with `opening_principles` (in opening)
//! - `position_features.passed_pawns` CONFLICTS with `endgame_patterns` (in endgame)
//! - `position_features.king_safety` COMPLEMENTS `castle_patterns`
//! - `endgame_patterns` REQUIRES `position_features` (for pawn structure)
//!
//! Use `validate_configuration()` to perform all validation checks, or enable
//! `auto_resolve_conflicts` for automatic conflict resolution.
//!
//! # Tuning Infrastructure Integration (Task 20.0 - Task 4.0)
//!
//! The evaluator supports weight tuning through the tuning infrastructure:
//!
//! - **`tune_weights()`**: Optimizes evaluation weights using training positions with expected scores.
//!   Uses gradient descent to minimize error between predicted and expected evaluations.
//!
//! - **`tune_from_telemetry()`**: Uses accumulated telemetry to suggest weight adjustments based on
//!   component contributions.
//!
//! - **`telemetry_to_tuning_pipeline()`**: Converts collected telemetry into a tuning position set
//!   for weight optimization.
//!
//! - **`TuningPositionSet`**: Collection of training positions with expected evaluations.
//!
//! - **`TuningConfig`**: Configuration for the tuning process (optimizer, learning rate, iterations, etc.).
//!
//! - **`TuningResult`**: Contains optimized weights and tuning statistics (error, iterations, convergence reason).
//!
//! Example usage:
//! ```rust,ignore
//! let mut evaluator = IntegratedEvaluator::new();
//!
//! // Create training positions
//! let position_set = TuningPositionSet::new(positions);
//! let tuning_config = TuningConfig::default();
//!
//! // Tune weights
//! let result = evaluator.tune_weights(&position_set, &tuning_config)?;
//! evaluator.set_weights(result.optimized_weights);
//! ```
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
    component_coordinator::{
        ComponentCoordination, ComponentContributionTracker, ConflictResolver,
    },
    config::EvaluationWeights,
    dependency_graph::DependencyValidator,
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
use crate::tuning::OptimizationMethod;
use crate::types::board::CapturedPieces;
use crate::types::core::{PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    /// Phase history for phase-aware validation (Task 20.0 - Task 5.14)
    phase_history: RefCell<Vec<i32>>,
}

impl IntegratedEvaluator {
    /// Create a new integrated evaluator with default configuration
    pub fn new() -> Self {
        Self::with_config(IntegratedEvaluationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(mut config: IntegratedEvaluationConfig) -> Self {
        // Validate and optionally auto-resolve conflicts (Task 20.0 - Task 5.11)
        if config.auto_resolve_conflicts {
            let warnings = config.validate_component_dependencies();
            if !warnings.is_empty() {
                debug_log(&format!(
                    "[IntegratedEvaluator] Auto-resolving {} component dependency conflicts",
                    warnings.len()
                ));
                config.auto_resolve_conflicts(); // This logs resolutions but conflicts are handled during evaluation
            }
        }

        // Validate configuration (Task 20.0 - Task 5.16)
        if let Err(err) = config.validate() {
            debug_log(&format!(
                "[IntegratedEvaluator] Configuration validation error: {}",
                err
            ));
        }

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
            phase_history: RefCell::new(Vec::new()), // Task 20.0 - Task 5.14
        };

        evaluator
            .statistics
            .borrow_mut()
            .set_collect_position_feature_stats(config.collect_position_feature_stats);

        evaluator
    }

    /// Main evaluation entry point
    ///
    /// # Arguments
    ///
    /// * `board` - Current board state
    /// * `player` - Player to evaluate for
    /// * `captured_pieces` - Captured pieces for both players
    /// * `move_count` - Optional move count (number of moves played in the game).
    ///                  If None, will be estimated from phase for opening principles evaluation.
    pub fn evaluate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        self.evaluate_with_move_count(board, player, captured_pieces, None)
    }

    pub fn evaluate_with_move_count(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        move_count: Option<u32>,
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
        let score = self.evaluate_standard(board, player, captured_pieces, move_count);

        // Record statistics
        if let Some(start_time) = start {
            let duration = start_time.elapsed().as_nanos() as u64;
            self.statistics.borrow_mut().record_timing(duration);

            if self.config.enable_phase_cache {
                let phase = self.calculate_phase_cached(board, captured_pieces);
                self.statistics.borrow_mut().record_evaluation(score, phase);

                // Record phase for phase-aware validation (Task 20.0 - Task 5.14)
                let mut phase_history = self.phase_history.borrow_mut();
                phase_history.push(phase);
                const MAX_PHASE_HISTORY: usize = 100;
                if phase_history.len() > MAX_PHASE_HISTORY {
                    phase_history.remove(0);
                }
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
        move_count: Option<u32>,
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

        // Track component contributions for telemetry (Task 5.0 - Task 5.9, 5.10)
        use std::collections::HashMap;
        let mut component_contributions: HashMap<String, f32> = HashMap::new();

        // Material
        if self.config.components.material {
            let material_score =
                self.material_eval
                    .borrow_mut()
                    .evaluate_material(board, player, captured_pieces);

            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation && material_score == TaperedScore::default()
            {
                debug_log(&format!(
                    "WARNING: material component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            total += material_score;
            // Track contribution for telemetry
            if stats_enabled {
                let material_interp = material_score.interpolate(phase);
                component_contributions.insert("material".to_string(), material_interp as f32);
            }
        }

        // Piece-square tables
        if self.config.components.piece_square_tables {
            let (pst_score, telemetry) = self.evaluate_pst(board, player);

            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation && pst_score == TaperedScore::default() {
                debug_log(&format!(
                    "WARNING: piece_square_tables component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            total += pst_score;
            pst_telemetry = Some(telemetry);
            // Track contribution for telemetry
            if stats_enabled {
                let pst_interp = pst_score.interpolate(phase);
                component_contributions
                    .insert("piece_square_tables".to_string(), pst_interp as f32);
            }
        }

        // Position features
        // Use ComponentCoordination from extracted module to determine coordination decisions
        let coordination = ComponentCoordination::new(
            &self.config.components,
            phase,
            &self.config.phase_boundaries,
            self.config.center_control_precedence,
            self.config.enable_gradual_phase_transitions,
        );

        let skip_passed_pawn_evaluation = coordination.skip_passed_pawn_evaluation;
        let skip_development_in_features = coordination.skip_development_in_features;
        let skip_center_control_in_features = coordination.skip_center_control_in_features;

        if self.config.components.position_features {
            let mut position_features = self.position_features.borrow_mut();
            position_features.begin_evaluation(board);

            // Track position_features aggregate contribution for telemetry
            let mut pf_total = TaperedScore::default();

            // King safety
            let king_safety_score =
                position_features.evaluate_king_safety(board, player, captured_pieces);
            let contribution =
                (king_safety_score.interpolate(phase) as f32) * weights.king_safety_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large king_safety contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    king_safety_score.interpolate(phase),
                    weights.king_safety_weight,
                    contribution
                ));
            }
            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation
                && king_safety_score == TaperedScore::default()
            {
                debug_log(&format!(
                    "WARNING: king_safety component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            let king_safety_weighted = king_safety_score * weights.king_safety_weight;
            total += king_safety_weighted;
            pf_total += king_safety_weighted;

            // Pawn structure
            let pawn_score = position_features.evaluate_pawn_structure(
                board,
                player,
                captured_pieces,
                skip_passed_pawn_evaluation,
            );
            let contribution =
                (pawn_score.interpolate(phase) as f32) * weights.pawn_structure_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large pawn_structure contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    pawn_score.interpolate(phase),
                    weights.pawn_structure_weight,
                    contribution
                ));
            }
            let pawn_weighted = pawn_score * weights.pawn_structure_weight;
            total += pawn_weighted;
            pf_total += pawn_weighted;

            // Mobility
            let mobility_score =
                position_features.evaluate_mobility(board, player, captured_pieces);
            let contribution = (mobility_score.interpolate(phase) as f32) * weights.mobility_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large mobility contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    mobility_score.interpolate(phase),
                    weights.mobility_weight,
                    contribution
                ));
            }
            let mobility_weighted = mobility_score * weights.mobility_weight;
            total += mobility_weighted;
            pf_total += mobility_weighted;

            // Center control (Task 20.0 - Task 1.0)
            // Skip center control in position_features if positional_patterns takes precedence
            let center_score = position_features.evaluate_center_control(
                board,
                player,
                skip_center_control_in_features,
            );
            let contribution =
                (center_score.interpolate(phase) as f32) * weights.center_control_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large center_control contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    center_score.interpolate(phase),
                    weights.center_control_weight,
                    contribution
                ));
            }
            let center_weighted = center_score * weights.center_control_weight;
            total += center_weighted;
            pf_total += center_weighted;

            // Development (Task 20.0 - Task 1.0)
            // Skip development in position_features if opening_principles is enabled in opening phase
            let dev_score =
                position_features.evaluate_development(board, player, skip_development_in_features);
            let contribution = (dev_score.interpolate(phase) as f32) * weights.development_weight;
            if contribution.abs() > self.config.weight_contribution_threshold {
                debug_log(&format!(
                    "Large development contribution: score={:.1} cp, weight={:.2}, contribution={:.1} cp",
                    dev_score.interpolate(phase),
                    weights.development_weight,
                    contribution
                ));
            }
            let dev_weighted = dev_score * weights.development_weight;
            total += dev_weighted;
            pf_total += dev_weighted;

            if stats_enabled && self.config.collect_position_feature_stats {
                position_feature_stats_snapshot = Some(position_features.stats().clone());
            }
            position_features.end_evaluation();

            // Track position_features aggregate contribution for telemetry
            if stats_enabled {
                let pf_interp = pf_total.interpolate(phase);
                component_contributions.insert("position_features".to_string(), pf_interp as f32);
            }
        }

        // Opening principles (if in opening)
        // Task 6.0 - Task 6.7, 6.10, 6.12: Use configurable phase boundaries and gradual transitions
        // Task 19.0 - Task 1.0: Use actual move_count instead of hardcoded 0
        if coordination.evaluate_opening_principles {
            // Estimate move_count from phase if not provided
            // Phase 256 = starting position (move 0), decreases as material is exchanged
            // Rough estimate: phase 256 = 0, phase 240 = ~5, phase 224 = ~10, phase 192 = ~16
            let estimated_move_count = move_count.unwrap_or_else(|| {
                if phase >= self.config.phase_boundaries.opening_threshold {
                    // In opening phase, estimate based on phase
                    // Formula: (256 - phase) / 4, clamped to reasonable range
                    ((256 - phase) / 4).max(0).min(20) as u32
                } else {
                    0
                }
            });

            let mut opening_score = self.opening_principles.borrow_mut().evaluate_opening(
                board,
                player,
                estimated_move_count,
                Some(captured_pieces),
                None,
            );

            // Apply gradual fade factor from coordination
            opening_score = opening_score * coordination.opening_fade_factor;

            total += opening_score;
        }

        // Endgame patterns (if in endgame)
        // Task 5.0 - Task 5.3: Warn if endgame_patterns enabled but not in endgame
        // Task 6.0 - Task 6.7, 6.9, 6.12: Use configurable phase boundaries and gradual transitions
        if self.config.components.endgame_patterns {
            let endgame_threshold = self.config.phase_boundaries.endgame_threshold;
            if !coordination.evaluate_endgame_patterns {
                debug_log(&format!(
                    "INFO: endgame_patterns is enabled but phase ({}) is not endgame (< {}). \
                    Endgame patterns will not be evaluated.",
                    phase, endgame_threshold
                ));
            } else {
                let mut endgame_score = self.endgame_patterns.borrow_mut().evaluate_endgame(
                    board,
                    player,
                    captured_pieces,
                );

                // Apply gradual fade factor from coordination
                endgame_score = endgame_score * coordination.endgame_fade_factor;

                // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
                if self.config.enable_component_validation
                    && endgame_score == TaperedScore::default()
                {
                    debug_log(&format!(
                        "WARNING: endgame_patterns is enabled but produced zero score. \
                        This may indicate a configuration issue or bug."
                    ));
                }

                total += endgame_score;
            }
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
            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation && tactical_score == TaperedScore::default()
            {
                debug_log(&format!(
                    "WARNING: tactical_patterns component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            total += tactical_score * weights.tactical_weight;
            // Track contribution for telemetry
            if stats_enabled {
                let tactical_interp =
                    (tactical_score.interpolate(phase) as f32 * weights.tactical_weight) as i32;
                component_contributions
                    .insert("tactical_patterns".to_string(), tactical_interp as f32);
            }
        }

        // Positional patterns (Phase 3 - Task 3.1 Integration)
        // Center control conflict resolution (Task 20.0 - Task 1.0)
        // When PositionFeatures precedence is used, skip center control in positional_patterns
        if self.config.components.positional_patterns {
            let positional_score = {
                let mut positional = self.positional_patterns.borrow_mut();

                // Temporarily disable center control if PositionFeatures takes precedence
                let original_center_control = positional.config_mut().enable_center_control;
                let skip_center_control_in_positional = if self.config.components.position_features
                    && self.config.center_control_precedence
                        == CenterControlPrecedence::PositionFeatures
                {
                    positional.config_mut().enable_center_control = false;
                    true
                } else {
                    false
                };

                let score = positional.evaluate_position(board, player, captured_pieces);

                // Restore original center control setting
                if skip_center_control_in_positional {
                    positional.config_mut().enable_center_control = original_center_control;
                }

                if stats_enabled {
                    positional_snapshot = Some(positional.stats().snapshot());
                }
                score
            };
            let contribution =
                (positional_score.interpolate(phase) as f32) * weights.positional_weight;
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
            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation
                && positional_score == TaperedScore::default()
            {
                debug_log(&format!(
                    "WARNING: positional_patterns component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            total += positional_score * weights.positional_weight;
            // Track contribution for telemetry
            if stats_enabled {
                let positional_interp =
                    (positional_score.interpolate(phase) as f32 * weights.positional_weight) as i32;
                component_contributions
                    .insert("positional_patterns".to_string(), positional_interp as f32);
            }
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
            // Task 5.0 - Task 5.5a, 5.5b: Validate zero scores from enabled components
            if self.config.enable_component_validation && castle_score == TaperedScore::default() {
                debug_log(&format!(
                    "WARNING: castle_patterns component is enabled but produced zero score. \
                    This may indicate a configuration issue or bug."
                ));
            }

            total += castle_score * weights.castle_weight;
            // Track contribution for telemetry
            if stats_enabled {
                let castle_interp =
                    (castle_score.interpolate(phase) as f32 * weights.castle_weight) as i32;
                component_contributions.insert("castle_patterns".to_string(), castle_interp as f32);
            }
        }

        // Interpolate to final score
        let final_score = self
            .phase_transition
            .borrow_mut()
            .interpolate_default(total, phase);

        // Calculate weight contributions for telemetry (Task 5.0 - Task 5.10)
        // Use ComponentContributionTracker from extracted module
        if stats_enabled && final_score != 0 {
            let mut tracker = ComponentContributionTracker::new();
            for (component, contrib) in &component_contributions {
                tracker.record(component, *contrib as i32);
            }
            let contributions_pct = tracker.to_percentages();

            // Task 5.0 - Task 5.11: Log when component contributes >threshold% of total
            for (component, contrib_pct) in &contributions_pct {
                if contrib_pct > &self.config.large_contribution_threshold {
                    debug_log(&format!(
                        "Large component contribution: {} contributes {:.1}% of total evaluation (threshold: {:.1}%)",
                        component,
                        contrib_pct * 100.0,
                        self.config.large_contribution_threshold * 100.0
                    ));
                }
            }
            component_contributions = contributions_pct;
        }

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

        let mut telemetry = EvaluationTelemetry::from_snapshots(
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

        // Add weight contributions to telemetry (Task 5.0 - Task 5.9, 5.10)
        telemetry.weight_contributions = component_contributions;
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

/// Center control precedence when both position_features and positional_patterns evaluate center control
///
/// This enum determines which component takes precedence when both evaluate center control.
/// - `PositionalPatterns`: Use positional_patterns evaluation (more sophisticated, includes drop pressure)
/// - `PositionFeatures`: Use position_features evaluation (control maps)
/// - `Both`: Evaluate both (may cause double-counting, not recommended)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterControlPrecedence {
    /// Use positional_patterns evaluation (recommended default)
    PositionalPatterns,
    /// Use position_features evaluation
    PositionFeatures,
    /// Evaluate both components (not recommended due to double-counting risk)
    Both,
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
    /// Threshold for logging large component contributions as percentage of total (default: 0.20 for 20%)
    pub large_contribution_threshold: f32,
    /// Enable component output validation (debug mode) - checks for zero scores from enabled components
    pub enable_component_validation: bool,
    /// Phase boundary configuration for game phase transitions
    pub phase_boundaries: crate::evaluation::config::PhaseBoundaryConfig,
    /// Enable gradual phase transitions (default: false for backward compatibility)
    ///
    /// When enabled, pattern scores are gradually faded out instead of abruptly cut off:
    /// - Opening principles fade from phase 192 to 160
    /// - Endgame patterns fade from phase 80 to 64
    pub enable_gradual_phase_transitions: bool,
    /// Center control precedence when both position_features and positional_patterns evaluate center control
    ///
    /// Default: `PositionalPatterns` (more sophisticated evaluation)
    pub center_control_precedence: CenterControlPrecedence,
    /// Component dependency graph (Task 20.0 - Task 5.4)
    pub dependency_graph: crate::evaluation::config::ComponentDependencyGraph,
    /// Automatically resolve conflicts when detected (Task 20.0 - Task 5.10)
    pub auto_resolve_conflicts: bool,
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
            large_contribution_threshold: 0.20,
            enable_component_validation: false,
            phase_boundaries: crate::evaluation::config::PhaseBoundaryConfig::default(),
            enable_gradual_phase_transitions: false,
            center_control_precedence: CenterControlPrecedence::PositionalPatterns,
            dependency_graph: crate::evaluation::config::ComponentDependencyGraph::default(), // Task 20.0 - Task 5.4
            auto_resolve_conflicts: false, // Task 20.0 - Task 5.10
        }
    }
}

impl IntegratedEvaluationConfig {
    /// Validate cumulative weights for enabled components
    pub fn validate_cumulative_weights(
        &self,
    ) -> Result<(), crate::evaluation::config::ConfigError> {
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

    /// Convert ComponentFlags to ComponentId set for dependency checking (Task 20.0 - Task 5.5)
    fn get_enabled_component_ids(&self) -> Vec<crate::evaluation::config::ComponentId> {
        use crate::evaluation::config::ComponentId;
        let mut ids = Vec::new();

        if self.components.material {
            ids.push(ComponentId::Material);
        }
        if self.components.piece_square_tables {
            ids.push(ComponentId::PieceSquareTables);
        }
        if self.components.position_features {
            ids.push(ComponentId::PositionFeatures);
            // Add sub-components if they're enabled
            ids.push(ComponentId::PositionFeaturesCenterControl);
            ids.push(ComponentId::PositionFeaturesDevelopment);
            ids.push(ComponentId::PositionFeaturesPassedPawns);
            ids.push(ComponentId::PositionFeaturesKingSafety);
        }
        if self.components.opening_principles {
            ids.push(ComponentId::OpeningPrinciples);
        }
        if self.components.endgame_patterns {
            ids.push(ComponentId::EndgamePatterns);
        }
        if self.components.tactical_patterns {
            ids.push(ComponentId::TacticalPatterns);
        }
        if self.components.positional_patterns {
            ids.push(ComponentId::PositionalPatterns);
        }
        if self.components.castle_patterns {
            ids.push(ComponentId::CastlePatterns);
        }

        ids
    }

    /// Validate component dependencies and check for conflicts (Task 20.0 - Task 5.5)
    ///
    /// Returns a vector of warnings for potential issues. These are informational
    /// and don't prevent the configuration from being used, but may indicate
    /// suboptimal settings.
    ///
    /// Uses the DependencyValidator from the extracted dependency_graph module.
    pub fn validate_component_dependencies(
        &self,
    ) -> Vec<crate::evaluation::config::ComponentDependencyWarning> {
        use crate::evaluation::config::{ComponentDependencyGraph, ComponentDependencyWarning};
        use crate::evaluation::dependency_graph::{ComponentFlags, DependencyValidator};
        let enabled_ids = self.get_enabled_component_ids();
        let graph = ComponentDependencyGraph::default();
        let components = ComponentFlags {
            position_features: self.components.position_features,
            positional_patterns: self.components.positional_patterns,
            opening_principles: self.components.opening_principles,
            endgame_patterns: self.components.endgame_patterns,
        };
        let validator = DependencyValidator::new(&graph, enabled_ids.clone(), components);
        let mut warnings = validator.validate_component_dependencies();

        // Legacy warnings for backward compatibility (Task 20.0 - Task 1.8)
        // Note: Automatically handled via center_control_precedence, but still warn for visibility
        if self.components.position_features && self.components.positional_patterns {
            warnings.push(ComponentDependencyWarning::CenterControlOverlap);
        }

        // Note: Automatically handled during evaluation (opening_principles takes precedence in opening),
        // but still warn for visibility
        if self.components.position_features && self.components.opening_principles {
            warnings.push(ComponentDependencyWarning::DevelopmentOverlap);
        }

        // Note: Endgame patterns phase check (Task 5.3) requires runtime phase calculation,
        // so it's handled during evaluation, not in static validation

        warnings
    }

    /// Suggest component resolution for conflicts (Task 20.0 - Task 5.9)
    pub fn suggest_component_resolution(&self) -> Vec<String> {
        use crate::evaluation::config::ComponentId;
        let mut suggestions = Vec::new();

        let enabled_ids = self.get_enabled_component_ids();

        // Check for conflicts and suggest resolutions
        for (i, &id1) in enabled_ids.iter().enumerate() {
            for &id2 in enabled_ids.iter().skip(i + 1) {
                if self.dependency_graph.conflicts(id1, id2) {
                    // Suggest disabling one based on precedence or importance
                    let suggestion = if matches!(id1, ComponentId::PositionalPatterns)
                        && matches!(id2, ComponentId::PositionFeaturesCenterControl)
                    {
                        format!("Disable position_features.center_control (positional_patterns takes precedence)")
                    } else if matches!(id1, ComponentId::OpeningPrinciples)
                        && matches!(id2, ComponentId::PositionFeaturesDevelopment)
                    {
                        format!("Disable position_features.development in opening (opening_principles takes precedence)")
                    } else if matches!(id1, ComponentId::EndgamePatterns)
                        && matches!(id2, ComponentId::PositionFeaturesPassedPawns)
                    {
                        format!("Disable position_features.passed_pawns in endgame (endgame_patterns takes precedence)")
                    } else {
                        format!("Disable either {:?} or {:?} to resolve conflict", id1, id2)
                    };
                    suggestions.push(suggestion);
                }
            }
        }

        suggestions
    }

    /// Automatically resolve conflicts by disabling components based on precedence (Task 20.0 - Task 5.9)
    pub fn auto_resolve_conflicts(&mut self) -> Vec<String> {
        use crate::evaluation::config::ComponentId;
        let mut resolutions = Vec::new();

        let enabled_ids = self.get_enabled_component_ids();

        // Resolve conflicts based on precedence
        for (i, &id1) in enabled_ids.iter().enumerate() {
            for &id2 in enabled_ids.iter().skip(i + 1) {
                if self.dependency_graph.conflicts(id1, id2) {
                    // Apply resolution based on component types and precedence
                    if matches!(id1, ComponentId::PositionalPatterns)
                        && matches!(id2, ComponentId::PositionFeaturesCenterControl)
                    {
                        // Positional patterns take precedence - handled by center_control_precedence
                        resolutions.push(
                            "Center control conflict resolved via center_control_precedence"
                                .to_string(),
                        );
                    } else if matches!(id1, ComponentId::OpeningPrinciples)
                        && matches!(id2, ComponentId::PositionFeaturesDevelopment)
                    {
                        // Opening principles take precedence in opening - already handled during evaluation
                        resolutions.push("Development conflict resolved (opening_principles takes precedence in opening)".to_string());
                    } else if matches!(id1, ComponentId::EndgamePatterns)
                        && matches!(id2, ComponentId::PositionFeaturesPassedPawns)
                    {
                        // Endgame patterns take precedence in endgame - already handled during evaluation
                        resolutions.push("Passed pawns conflict resolved (endgame_patterns takes precedence in endgame)".to_string());
                    }
                }
            }
        }

        resolutions
    }

    /// Check phase compatibility for component usage (Task 20.0 - Task 5.14)
    ///
    /// Analyzes recent phase history to detect phase-component mismatches.
    /// Returns warnings if components are enabled but phase is consistently outside their effective range.
    pub fn check_phase_compatibility(
        &self,
        phase_history: &[i32],
    ) -> Vec<crate::evaluation::config::ComponentDependencyWarning> {
        use crate::evaluation::config::ComponentDependencyWarning;
        let mut warnings = Vec::new();

        if phase_history.is_empty() {
            return warnings;
        }

        let opening_threshold = self.phase_boundaries.opening_threshold;
        let endgame_threshold = self.phase_boundaries.endgame_threshold;

        // Check if phases are consistently in a particular range
        let avg_phase: i32 = phase_history.iter().sum::<i32>() / phase_history.len() as i32;

        // Warn when opening_principles is enabled but phase is consistently < opening_threshold (Task 20.0 - Task 5.12)
        if self.components.opening_principles && avg_phase < opening_threshold {
            warnings.push(ComponentDependencyWarning::EndgamePatternsNotInEndgame);
            // Reuse for now
        }

        // Warn when endgame_patterns is enabled but phase is consistently >= endgame_threshold (Task 20.0 - Task 5.13)
        if self.components.endgame_patterns && avg_phase >= endgame_threshold {
            warnings.push(ComponentDependencyWarning::EndgamePatternsNotInEndgame);
        }

        warnings
    }

    /// Validate the configuration
    ///
    /// This validates weights and component dependencies. Returns errors for
    /// invalid configurations and warnings for potential issues.
    pub fn validate(
        &self,
    ) -> Result<
        Vec<crate::evaluation::config::ComponentDependencyWarning>,
        crate::evaluation::config::ConfigError,
    > {
        // Validate cumulative weights
        self.validate_cumulative_weights()?;

        // Check component dependencies (warnings, not errors)
        let warnings = self.validate_component_dependencies();

        Ok(warnings)
    }
}

// Validation methods for IntegratedEvaluator (Task 20.0 - Task 5.0)
impl IntegratedEvaluator {
    /// Validate configuration with all checks (Task 20.0 - Task 5.15)
    ///
    /// Performs comprehensive validation including:
    /// - Cumulative weight validation
    /// - Component dependency validation
    /// - Phase compatibility validation (if phase history is available)
    pub fn validate_configuration(
        &self,
    ) -> Result<
        Vec<crate::evaluation::config::ComponentDependencyWarning>,
        crate::evaluation::config::ConfigError,
    > {
        let mut warnings = Vec::new();

        // Validate configuration (weights and dependencies)
        let config_warnings = self.config.validate()?;
        warnings.extend(config_warnings);

        // Phase-aware validation (Task 20.0 - Task 5.14)
        let phase_history = self.phase_history.borrow();
        if !phase_history.is_empty() {
            let phase_warnings = self.config.check_phase_compatibility(&phase_history);
            warnings.extend(phase_warnings);
        }

        Ok(warnings)
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

        let score =
            evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);

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

        let score =
            evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);
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

        let score =
            evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);
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

        let score =
            evaluator.evaluate_with_move_count(&board, Player::Black, &captured_pieces, None);
        assert!(score.abs() < 100000);
    }
}

// ============================================================================
// TUNING INFRASTRUCTURE INTEGRATION (Task 20.0 - Task 4.0)
// ============================================================================

/// Training position with board state and expected evaluation (Task 20.0 - Task 4.2)
///
/// Note: BitboardBoard and CapturedPieces are not serializable, so this struct
/// cannot be directly serialized. Use position hashes or FEN strings for serialization.
#[derive(Clone)]
pub struct TuningPosition {
    /// Board position
    pub board: BitboardBoard,
    /// Captured pieces
    pub captured_pieces: CapturedPieces,
    /// Player to move
    pub player: Player,
    /// Expected evaluation score from the position's perspective (normalized to -1.0 to 1.0)
    pub expected_score: f64,
    /// Game phase (0 = endgame, 256 = opening)
    pub game_phase: i32,
    /// Move number in game (1-indexed)
    pub move_number: u32,
}

/// Collection of training positions for tuning (Task 20.0 - Task 4.2)
#[derive(Clone)]
pub struct TuningPositionSet {
    /// Training positions
    pub positions: Vec<TuningPosition>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl TuningPositionSet {
    /// Create a new tuning position set
    pub fn new(positions: Vec<TuningPosition>) -> Self {
        Self {
            positions,
            metadata: HashMap::new(),
        }
    }

    /// Create an empty tuning position set
    pub fn empty() -> Self {
        Self {
            positions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a position to the set
    pub fn add_position(&mut self, position: TuningPosition) {
        self.positions.push(position);
    }

    /// Get the number of positions
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Tuning configuration (Task 20.0 - Task 4.6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Optimization method to use
    pub method: OptimizationMethod,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
    /// Learning rate (for gradient-based methods)
    pub learning_rate: f64,
    /// K-factor for sigmoid conversion
    pub k_factor: f64,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            method: OptimizationMethod::Adam {
                learning_rate: 0.001,
                beta1: 0.9,
                beta2: 0.999,
                epsilon: 1e-8,
            },
            max_iterations: 1000,
            convergence_threshold: 1e-6,
            learning_rate: 0.001,
            k_factor: 1.0,
        }
    }
}

/// Tuning result containing optimized weights and statistics (Task 20.0 - Task 4.14)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Optimized evaluation weights
    pub optimized_weights: EvaluationWeights,
    /// Final error value
    pub final_error: f64,
    /// Number of iterations completed
    pub iterations: usize,
    /// Convergence reason
    pub convergence_reason: ConvergenceReason,
    /// Total optimization time
    pub optimization_time: Duration,
    /// Error history across iterations
    pub error_history: Vec<f64>,
}

/// Convergence reason (Task 20.0 - Task 4.14)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergenceReason {
    /// Converged successfully
    Converged,
    /// Reached maximum iterations
    MaxIterations,
    /// Early stopping triggered
    EarlyStopping,
    /// Gradient norm below threshold
    GradientNorm,
}

// Weight adapter methods (Task 20.0 - Task 4.4, 4.5)
impl EvaluationWeights {
    /// Convert EvaluationWeights to a vector of f64 (for optimizer compatibility)
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.material_weight as f64,
            self.position_weight as f64,
            self.king_safety_weight as f64,
            self.pawn_structure_weight as f64,
            self.mobility_weight as f64,
            self.center_control_weight as f64,
            self.development_weight as f64,
            self.tactical_weight as f64,
            self.positional_weight as f64,
            self.castle_weight as f64,
        ]
    }

    /// Create EvaluationWeights from a vector of f64
    pub fn from_vector(weights: &[f64]) -> Result<Self, String> {
        if weights.len() != 10 {
            return Err(format!("Expected 10 weights, got {}", weights.len()));
        }

        Ok(Self {
            material_weight: weights[0] as f32,
            position_weight: weights[1] as f32,
            king_safety_weight: weights[2] as f32,
            pawn_structure_weight: weights[3] as f32,
            mobility_weight: weights[4] as f32,
            center_control_weight: weights[5] as f32,
            development_weight: weights[6] as f32,
            tactical_weight: weights[7] as f32,
            positional_weight: weights[8] as f32,
            castle_weight: weights[9] as f32,
        })
    }
}

// Tuning methods for IntegratedEvaluator (Task 20.0 - Task 4.0)
impl IntegratedEvaluator {
    /// Tune evaluation weights using training positions (Task 20.0 - Task 4.3, 4.6-4.9)
    ///
    /// This method optimizes the evaluation weights to minimize the error between
    /// predicted and expected scores on the training positions.
    ///
    /// # Arguments
    ///
    /// * `position_set` - Collection of training positions with expected scores
    /// * `tuning_config` - Configuration for the tuning process
    ///
    /// # Returns
    ///
    /// * `TuningResult` containing optimized weights and statistics
    pub fn tune_weights(
        &mut self,
        position_set: &TuningPositionSet,
        tuning_config: &TuningConfig,
    ) -> Result<TuningResult, String> {
        if position_set.is_empty() {
            return Err("Position set is empty".to_string());
        }

        let start_time = Instant::now();
        let mut weights = self.weights.to_vector();
        let mut error_history = Vec::new();
        let mut prev_error = f64::INFINITY;
        let mut patience_counter = 0;
        const EARLY_STOPPING_PATIENCE: usize = 50;

        // Simple gradient descent optimizer for component weights
        // (Simplified version - full implementation would use the tuning infrastructure's optimizers)
        for iteration in 0..tuning_config.max_iterations {
            let (error, gradients) =
                self.calculate_error_and_gradients(&weights, position_set, tuning_config.k_factor);
            error_history.push(error);

            // Check for convergence
            if error < tuning_config.convergence_threshold {
                let optimized_weights = EvaluationWeights::from_vector(&weights)?;
                return Ok(TuningResult {
                    optimized_weights,
                    final_error: error,
                    iterations: iteration + 1,
                    convergence_reason: ConvergenceReason::Converged,
                    optimization_time: start_time.elapsed(),
                    error_history,
                });
            }

            // Early stopping
            if error < prev_error {
                prev_error = error;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= EARLY_STOPPING_PATIENCE {
                    let optimized_weights = EvaluationWeights::from_vector(&weights)?;
                    return Ok(TuningResult {
                        optimized_weights,
                        final_error: error,
                        iterations: iteration + 1,
                        convergence_reason: ConvergenceReason::EarlyStopping,
                        optimization_time: start_time.elapsed(),
                        error_history,
                    });
                }
            }

            // Update weights using gradient descent
            for (i, gradient) in gradients.iter().enumerate() {
                weights[i] -= tuning_config.learning_rate * gradient;
                // Clamp weights to reasonable range (0.0 to 10.0)
                weights[i] = weights[i].max(0.0).min(10.0);
            }
        }

        let optimized_weights = EvaluationWeights::from_vector(&weights)?;
        Ok(TuningResult {
            optimized_weights,
            final_error: prev_error,
            iterations: tuning_config.max_iterations,
            convergence_reason: ConvergenceReason::MaxIterations,
            optimization_time: start_time.elapsed(),
            error_history,
        })
    }

    /// Calculate error and gradients for current weights (Task 20.0 - Task 4.7)
    fn calculate_error_and_gradients(
        &self,
        weights: &[f64],
        position_set: &TuningPositionSet,
        k_factor: f64,
    ) -> (f64, Vec<f64>) {
        let mut total_error = 0.0;
        let mut gradients = vec![0.0; 10]; // 10 weights

        // Create a temporary evaluator with the specified weights
        if let Ok(temp_weights) = EvaluationWeights::from_vector(weights) {
            // Create a new evaluator with modified weights
            let mut temp_evaluator = IntegratedEvaluator::with_config(self.config.clone());
            temp_evaluator.weights = temp_weights.clone();

            for position in &position_set.positions {
                // Evaluate position with current weights
                let predicted_score = temp_evaluator.evaluate_with_move_count(
                    &position.board,
                    position.player,
                    &position.captured_pieces,
                    None,
                ) as f64;

                // Convert to probability using sigmoid
                let predicted_prob = sigmoid(predicted_score * k_factor);
                let expected_prob = position.expected_score;

                // Calculate error (mean squared error)
                let error = expected_prob - predicted_prob;
                total_error += error * error;

                // Calculate gradients using finite differences approximation
                // For each weight, calculate gradient contribution
                let epsilon = 1e-5;
                for i in 0..10 {
                    let mut perturbed_weights = weights.to_vec();
                    perturbed_weights[i] += epsilon;

                    if let Ok(perturbed_eval_weights) =
                        EvaluationWeights::from_vector(&perturbed_weights)
                    {
                        let mut perturbed_evaluator =
                            IntegratedEvaluator::with_config(self.config.clone());
                        perturbed_evaluator.weights = perturbed_eval_weights;
                        let perturbed_score = perturbed_evaluator.evaluate_with_move_count(
                            &position.board,
                            position.player,
                            &position.captured_pieces,
                            None,
                        ) as f64;
                        let perturbed_prob = sigmoid(perturbed_score * k_factor);

                        let gradient_contribution =
                            (perturbed_prob - predicted_prob) / epsilon * error * (-2.0);
                        gradients[i] += gradient_contribution;
                    }
                }
            }
        }

        // Average
        let n = position_set.len() as f64;
        total_error /= n;
        for gradient in &mut gradients {
            *gradient /= n;
        }

        (total_error, gradients)
    }

    /// Tune weights from accumulated telemetry (Task 20.0 - Task 4.12)
    ///
    /// Uses accumulated telemetry to suggest weight adjustments.
    /// Delegates to the weight_tuning module.
    pub fn tune_from_telemetry(
        &mut self,
        telemetry_set: &[EvaluationTelemetry],
        target_contributions: Option<&HashMap<String, f32>>,
        learning_rate: f32,
    ) -> Result<EvaluationWeights, String> {
        // Use the extracted weight_tuning module
        // Note: The weight_tuning module currently has placeholder types that need
        // to be properly integrated. For now, keeping the original implementation.
        // TODO: Update weight_tuning module to accept IntegratedEvaluator properly
        if telemetry_set.is_empty() {
            return Err("Telemetry set is empty".to_string());
        }

        // Use the existing auto_balance_weights functionality
        let config = crate::evaluation::config::TaperedEvalConfig::default();
        let mut temp_config = config.clone();

        // Aggregate telemetry
        let mut aggregated_contributions = HashMap::new();
        for telemetry in telemetry_set {
            for (component, contribution) in &telemetry.weight_contributions {
                *aggregated_contributions
                    .entry(component.clone())
                    .or_insert(0.0) += contribution;
            }
        }

        // Average
        let count = telemetry_set.len() as f32;
        for contribution in aggregated_contributions.values_mut() {
            *contribution /= count;
        }

        // Use auto_balance_weights to suggest adjustments
        // (This is a simplified version - full implementation would use optimizer)
        let components = crate::evaluation::config::ComponentFlagsForValidation {
            material: true,
            piece_square_tables: true,
            position_features: true,
            tactical_patterns: true,
            positional_patterns: true,
            castle_patterns: true,
        };
        temp_config.auto_balance_weights(
            &telemetry_set[0], // Use first telemetry as representative
            &components,
            target_contributions,
            learning_rate,
        );

        Ok(temp_config.weights)
    }

    /// Telemetry-to-tuning pipeline (Task 20.0 - Task 4.11)
    ///
    /// Collects telemetry from multiple positions and converts them to a tuning position set.
    /// Delegates to the weight_tuning module.
    pub fn telemetry_to_tuning_pipeline(
        &self,
        telemetry_positions: &[(
            BitboardBoard,
            CapturedPieces,
            Player,
            EvaluationTelemetry,
            f64,
        )],
    ) -> crate::evaluation::weight_tuning::TuningPositionSet {
        // Use the extracted weight_tuning module
        // Note: The weight_tuning module currently has placeholder types that need
        // to be properly integrated. For now, keeping the original implementation.
        // TODO: Update weight_tuning module to accept IntegratedEvaluator properly
        use crate::evaluation::weight_tuning::{TuningPosition, TuningPositionSet};
        let mut positions = Vec::new();

        for (board, captured_pieces, player, _telemetry, expected_score) in telemetry_positions {
            // Calculate game phase
            let game_phase = self.calculate_phase_cached(board, captured_pieces);

            // Create tuning position
            let tuning_position = TuningPosition {
                board: board.clone(),
                captured_pieces: captured_pieces.clone(),
                player: *player,
                expected_score: *expected_score,
                game_phase,
                move_number: 1, // Default - should be provided if available
            };

            positions.push(tuning_position);
        }

        TuningPositionSet::new(positions)
    }
}

/// Sigmoid function for probability conversion (Task 20.0 - Task 4.7)
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

// Note: export_for_tuning() is now in the telemetry module (src/evaluation/telemetry.rs)
// This impl block has been removed as part of Task 1.21: Refactor integration.rs to be a thin facade
