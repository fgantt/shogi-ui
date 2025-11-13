//! Unified Configuration System for Tapered Evaluation
//!
//! This module provides a comprehensive configuration system that unifies all
//! tapered evaluation components. It supports:
//! - Unified configuration struct
//! - Configuration loading from files (JSON/TOML)
//! - Configuration validation
//! - Runtime configuration updates
//! - Configuration presets
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::config::TaperedEvalConfig;
//!
//! // Create default configuration
//! let config = TaperedEvalConfig::default();
//!
//! // Create performance-optimized configuration
//! let config = TaperedEvalConfig::performance_optimized();
//!
//! // Load from file
//! let config = TaperedEvalConfig::load_from_file("eval_config.json")?;
//!
//! // Validate configuration
//! assert!(config.validate().is_ok());
//! ```

use crate::evaluation::advanced_interpolation::AdvancedInterpolationConfig;
use crate::evaluation::material::MaterialEvaluationConfig;
use crate::evaluation::phase_transition::{InterpolationMethod, PhaseTransitionConfig};
use crate::evaluation::position_features::PositionFeatureConfig;
use crate::evaluation::pst_loader::PieceSquareTableConfig;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Unified configuration for all tapered evaluation components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaperedEvalConfig {
    /// Enable or disable tapered evaluation globally
    pub enabled: bool,

    /// Configuration for material evaluation
    pub material: MaterialEvaluationConfig,

    /// Configuration for piece-square tables
    pub pst: PieceSquareTableConfig,

    /// Configuration for phase transitions
    pub phase_transition: PhaseTransitionConfig,

    /// Configuration for position features
    pub position_features: PositionFeatureConfig,

    /// Configuration for tapered evaluation base settings
    pub base: TaperedEvaluationConfig,

    /// Evaluation weights for combining components
    pub weights: EvaluationWeights,

    /// Enable phase-dependent weight scaling (default: false for backward compatibility)
    pub enable_phase_dependent_weights: bool,

    /// Threshold for logging large weight contributions in centipawns (default: 1000.0)
    pub weight_contribution_threshold: f32,

    /// Automatically validate weights after updates (default: true) (Task 20.0 - Task 2.0)
    pub auto_validate_weights: bool,

    /// Automatically normalize weights to ensure cumulative sum is within range (default: false) (Task 20.0 - Task 2.0)
    pub auto_normalize_weights: bool,
}

/// Weights for combining different evaluation components
///
/// # Weight Calibration Methodology
///
/// Evaluation weights control the relative importance of different evaluation components.
/// The final evaluation score is calculated as:
/// ```
/// total_score = material_score * material_weight
///            + position_score * position_weight
///            + king_safety_score * king_safety_weight
///            + ... (for all enabled components)
/// ```
///
/// ## Recommended Weight Ranges
///
/// - **Material weight**: 0.8-1.2 (typically 1.0). Material is fundamental and should be stable.
/// - **Position weight (PST)**: 0.5-1.5 (typically 1.0). Piece-square tables provide positional bonuses.
/// - **King safety weight**: 0.8-1.5 (typically 1.0). Critical for king safety evaluation.
/// - **Pawn structure weight**: 0.6-1.2 (typically 0.8). Important but less critical than material.
/// - **Mobility weight**: 0.4-0.8 (typically 0.6). Piece mobility is important but secondary.
/// - **Center control weight**: 0.5-1.0 (typically 0.7). Center control is valuable but not decisive.
/// - **Development weight**: 0.3-0.7 (typically 0.5). Development matters most in opening.
/// - **Tactical weight**: 0.8-1.5 (typically 1.0). Tactical patterns are important in middlegame.
/// - **Positional weight**: 0.8-1.5 (typically 1.0). Positional patterns matter throughout the game.
/// - **Castle weight**: 0.7-1.3 (typically 1.0). Castle patterns are important for king safety.
///
/// ## Weight Calibration Examples
///
/// ### Aggressive Play Style
/// ```rust
/// let mut weights = EvaluationWeights::default();
/// weights.tactical_weight = 1.5;  // Emphasize tactical patterns
/// weights.mobility_weight = 0.8;   // Value piece activity
/// weights.development_weight = 0.7; // Emphasize quick development
/// ```
///
/// ### Positional Play Style
/// ```rust
/// let mut weights = EvaluationWeights::default();
/// weights.positional_weight = 1.5;      // Emphasize positional patterns
/// weights.pawn_structure_weight = 1.2;   // Value pawn structure
/// weights.center_control_weight = 1.0;    // Emphasize center control
/// ```
///
/// ### Defensive Play Style
/// ```rust
/// let mut weights = EvaluationWeights::default();
/// weights.king_safety_weight = 1.5;      // Emphasize king safety
/// weights.castle_weight = 1.3;            // Value castle formations
/// weights.tactical_weight = 0.8;          // Reduce tactical emphasis
/// ```
///
/// ## Weight Interaction Effects
///
/// Changing one weight affects the overall evaluation balance:
///
/// - **Increasing material_weight**: Makes material advantages more decisive. May reduce
///   the impact of positional factors. Typical range: 0.8-1.2.
///
/// - **Increasing tactical_weight**: Makes tactical patterns (forks, pins, skewers) more
///   influential. Good for aggressive play. Typical range: 0.8-1.5.
///
/// - **Increasing positional_weight**: Makes positional patterns (outposts, weak squares,
///   piece activity) more influential. Good for positional play. Typical range: 0.8-1.5.
///
/// - **Increasing king_safety_weight**: Makes king safety more important. May reduce
///   emphasis on material or positional factors. Typical range: 0.8-1.5.
///
/// - **Increasing pawn_structure_weight**: Makes pawn structure more important. Good for
///   endgame play. Typical range: 0.6-1.2.
///
/// - **Increasing mobility_weight**: Makes piece mobility more important. Good for open
///   positions. Typical range: 0.4-0.8.
///
/// - **Increasing center_control_weight**: Makes center control more important. Good for
///   opening and middlegame. Typical range: 0.5-1.0.
///
/// - **Increasing development_weight**: Makes piece development more important. Primarily
///   affects opening evaluation. Typical range: 0.3-0.7.
///
/// - **Increasing castle_weight**: Makes castle patterns more important. Good for king
///   safety evaluation. Typical range: 0.7-1.3.
///
/// ## Calibration Tips
///
/// 1. **Start with defaults**: The default weights are balanced and work well for most positions.
/// 2. **Adjust incrementally**: Change weights by 0.1-0.2 at a time and test the impact.
/// 3. **Consider game phase**: Use phase-dependent weight scaling (see `enable_phase_dependent_weights`)
///    to adjust weights based on game phase.
/// 4. **Monitor cumulative weights**: Use `validate_cumulative_weights()` to ensure total weight
///    sum is reasonable (typically 5.0-15.0).
/// 5. **Test with positions**: Evaluate test positions to verify weight changes produce expected
///    behavior.
/// 6. **Use telemetry**: Monitor `weight_contributions` in `EvaluationTelemetry` to see which
///    components contribute most to evaluation.
///
/// ## Validation
///
/// Weights are validated to ensure they are:
/// - Non-negative (weights < 0.0 are invalid)
/// - Within reasonable range (typically 0.0-10.0, though 0.5-2.0 is more common)
/// - Finite (NaN and infinity are invalid)
///
/// Use `TaperedEvalConfig::validate()` to check weight validity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationWeights {
    /// Weight for material evaluation (typically 1.0, range: 0.8-1.2)
    /// 
    /// Material is fundamental and should be stable. Increasing this weight makes
    /// material advantages more decisive but may reduce the impact of positional factors.
    pub material_weight: f32,

    /// Weight for piece-square tables (typically 1.0, range: 0.5-1.5)
    /// 
    /// Piece-square tables provide positional bonuses based on piece placement.
    /// Increasing this weight emphasizes piece-square table bonuses.
    pub position_weight: f32,

    /// Weight for king safety (typically 1.0, range: 0.8-1.5)
    /// 
    /// King safety is critical for evaluation. Increasing this weight makes king
    /// safety more important but may reduce emphasis on material or positional factors.
    pub king_safety_weight: f32,

    /// Weight for pawn structure (typically 0.8, range: 0.6-1.2)
    /// 
    /// Pawn structure is important for evaluation, especially in endgame. Increasing
    /// this weight makes pawn structure more important.
    pub pawn_structure_weight: f32,

    /// Weight for mobility (typically 0.6, range: 0.4-0.8)
    /// 
    /// Piece mobility is important but secondary to material and position. Increasing
    /// this weight makes piece activity more important, good for open positions.
    pub mobility_weight: f32,

    /// Weight for center control (typically 0.7, range: 0.5-1.0)
    /// 
    /// Center control is valuable but not decisive. Increasing this weight emphasizes
    /// center control, good for opening and middlegame.
    pub center_control_weight: f32,

    /// Weight for development (typically 0.5, range: 0.3-0.7)
    /// 
    /// Piece development matters most in opening. Increasing this weight emphasizes
    /// quick development, primarily affecting opening evaluation.
    pub development_weight: f32,

    /// Weight for tactical pattern contributions (typically 1.0, range: 0.8-1.5)
    /// 
    /// Tactical patterns (forks, pins, skewers) are important in middlegame.
    /// Increasing this weight makes tactical patterns more influential, good for aggressive play.
    pub tactical_weight: f32,
    
    /// Weight for positional pattern contributions (typically 1.0, range: 0.8-1.5)
    /// 
    /// Positional patterns (outposts, weak squares, piece activity) matter throughout the game.
    /// Increasing this weight makes positional patterns more influential, good for positional play.
    pub positional_weight: f32,
    
    /// Weight for castle pattern contributions (typically 1.0, range: 0.7-1.3)
    /// 
    /// Castle patterns are important for king safety evaluation. Increasing this weight
    /// makes castle formations more important.
    pub castle_weight: f32,
}

impl Default for EvaluationWeights {
    fn default() -> Self {
        Self {
            material_weight: 1.0,
            position_weight: 1.0,
            king_safety_weight: 1.0,
            pawn_structure_weight: 0.8,
            mobility_weight: 0.6,
            center_control_weight: 0.7,
            development_weight: 0.5,
            tactical_weight: 1.0,
            positional_weight: 1.0,
            castle_weight: 1.0,
        }
    }
}

impl EvaluationWeights {
    /// Normalize weights to ensure cumulative sum is within range while maintaining ratios (Task 20.0 - Task 2.6)
    /// 
    /// Scales all weights proportionally to ensure cumulative sum is within 5.0-15.0 range.
    /// This maintains the relative ratios between weights while fixing the total sum.
    /// 
    /// # Parameters
    /// - `components`: Component flags indicating which weights are enabled
    /// 
    /// # Target Range
    /// - Target cumulative sum: 10.0 (midpoint of 5.0-15.0 range)
    pub fn normalize_weights(&mut self, components: &ComponentFlagsForValidation) {
        let mut sum = 0.0;
        if components.material {
            sum += self.material_weight;
        }
        if components.piece_square_tables {
            sum += self.position_weight;
        }
        if components.position_features {
            sum += self.king_safety_weight;
            sum += self.pawn_structure_weight;
            sum += self.mobility_weight;
            sum += self.center_control_weight;
            sum += self.development_weight;
        }
        if components.tactical_patterns {
            sum += self.tactical_weight;
        }
        if components.positional_patterns {
            sum += self.positional_weight;
        }
        if components.castle_patterns {
            sum += self.castle_weight;
        }

        const MIN_CUMULATIVE_WEIGHT: f32 = 5.0;
        const MAX_CUMULATIVE_WEIGHT: f32 = 15.0;
        const TARGET_CUMULATIVE_WEIGHT: f32 = 10.0; // Midpoint

        // Only normalize if out of range
        if sum < MIN_CUMULATIVE_WEIGHT || sum > MAX_CUMULATIVE_WEIGHT {
            // Calculate scaling factor to bring sum to target
            let scale = if sum > 0.0 {
                TARGET_CUMULATIVE_WEIGHT / sum
            } else {
                1.0 // If sum is 0, don't scale
            };

            // Apply scaling to all enabled weights
            if components.material {
                self.material_weight *= scale;
            }
            if components.piece_square_tables {
                self.position_weight *= scale;
            }
            if components.position_features {
                self.king_safety_weight *= scale;
                self.pawn_structure_weight *= scale;
                self.mobility_weight *= scale;
                self.center_control_weight *= scale;
                self.development_weight *= scale;
            }
            if components.tactical_patterns {
                self.tactical_weight *= scale;
            }
            if components.positional_patterns {
                self.positional_weight *= scale;
            }
            if components.castle_patterns {
                self.castle_weight *= scale;
            }
        }
    }

    /// Apply a weight preset (Task 20.0 - Task 2.10)
    /// 
    /// Sets all weights based on the specified preset style.
    pub fn apply_preset(&mut self, preset: WeightPreset) {
        match preset {
            WeightPreset::Balanced => {
                // Default balanced weights
                self.material_weight = 1.0;
                self.position_weight = 1.0;
                self.king_safety_weight = 1.0;
                self.pawn_structure_weight = 0.8;
                self.mobility_weight = 0.6;
                self.center_control_weight = 0.7;
                self.development_weight = 0.5;
                self.tactical_weight = 1.0;
                self.positional_weight = 1.0;
                self.castle_weight = 1.0;
            }
            WeightPreset::Aggressive => {
                // Emphasize tactical patterns and mobility
                self.material_weight = 1.0;
                self.position_weight = 1.0;
                self.king_safety_weight = 0.9;
                self.pawn_structure_weight = 0.7;
                self.mobility_weight = 0.8;
                self.center_control_weight = 0.8;
                self.development_weight = 0.7;
                self.tactical_weight = 1.5;
                self.positional_weight = 0.8;
                self.castle_weight = 0.9;
            }
            WeightPreset::Positional => {
                // Emphasize positional patterns and pawn structure
                self.material_weight = 1.0;
                self.position_weight = 1.1;
                self.king_safety_weight = 1.0;
                self.pawn_structure_weight = 1.2;
                self.mobility_weight = 0.6;
                self.center_control_weight = 1.0;
                self.development_weight = 0.5;
                self.tactical_weight = 0.8;
                self.positional_weight = 1.5;
                self.castle_weight = 1.0;
            }
            WeightPreset::Defensive => {
                // Emphasize king safety and castle patterns
                self.material_weight = 1.0;
                self.position_weight = 1.0;
                self.king_safety_weight = 1.5;
                self.pawn_structure_weight = 1.0;
                self.mobility_weight = 0.5;
                self.center_control_weight = 0.6;
                self.development_weight = 0.4;
                self.tactical_weight = 0.8;
                self.positional_weight = 1.0;
                self.castle_weight = 1.3;
            }
        }
    }
}

/// Weight preset styles (Task 20.0 - Task 2.9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightPreset {
    /// Balanced weights (default) - good for general play
    Balanced,
    /// Aggressive play style - emphasizes tactical patterns and mobility
    Aggressive,
    /// Positional play style - emphasizes positional patterns and pawn structure
    Positional,
    /// Defensive play style - emphasizes king safety and castle patterns
    Defensive,
}

/// Recommended weight ranges (Task 20.0 - Task 2.4)
/// Maps weight names to (min, max, default) tuples
const RECOMMENDED_WEIGHT_RANGES: &[(&str, (f32, f32, f32))] = &[
    ("material", (0.8, 1.2, 1.0)),
    ("position", (0.5, 1.5, 1.0)),
    ("king_safety", (0.8, 1.5, 1.0)),
    ("pawn_structure", (0.6, 1.2, 0.8)),
    ("mobility", (0.4, 0.8, 0.6)),
    ("center_control", (0.5, 1.0, 0.7)),
    ("development", (0.3, 0.7, 0.5)),
    ("tactical", (0.8, 1.5, 1.0)),
    ("positional", (0.8, 1.5, 1.0)),
    ("castle", (0.7, 1.3, 1.0)),
];

impl TaperedEvalConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with tapered evaluation disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            material: MaterialEvaluationConfig::default(),
            pst: PieceSquareTableConfig::default(),
            phase_transition: PhaseTransitionConfig::default(),
            position_features: PositionFeatureConfig::default(),
            base: TaperedEvaluationConfig::disabled(),
            weights: EvaluationWeights::default(),
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
            auto_validate_weights: true,
            auto_normalize_weights: false,
        }
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            material: MaterialEvaluationConfig {
                include_hand_pieces: true,
                enable_fast_loop: true,
                ..MaterialEvaluationConfig::default()
            },
            pst: PieceSquareTableConfig::default(),
            phase_transition: PhaseTransitionConfig {
                default_method: InterpolationMethod::Linear,
                use_phase_boundaries: false, // Disabled for performance
                sigmoid_steepness: 6.0,
                use_advanced_interpolator: false,
                advanced_config: AdvancedInterpolationConfig::default(),
            },
            position_features: PositionFeatureConfig {
                enable_king_safety: true,
                enable_pawn_structure: true,
                enable_mobility: false, // Expensive, disable for speed
                enable_center_control: true,
                enable_development: true,
            },
            base: TaperedEvaluationConfig::performance_optimized(),
            weights: EvaluationWeights::default(),
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
            auto_validate_weights: true,
            auto_normalize_weights: false,
        }
    }

    /// Create a configuration optimized for strength (accuracy over speed)
    pub fn strength_optimized() -> Self {
        Self {
            enabled: true,
            material: MaterialEvaluationConfig::default(),
            pst: PieceSquareTableConfig::default(),
            phase_transition: PhaseTransitionConfig {
                default_method: InterpolationMethod::Advanced,
                use_phase_boundaries: true,
                sigmoid_steepness: 6.0,
                use_advanced_interpolator: true,
                advanced_config: AdvancedInterpolationConfig {
                    use_spline: true,
                    enable_adaptive: true,
                    ..AdvancedInterpolationConfig::default()
                },
            },
            position_features: PositionFeatureConfig {
                enable_king_safety: true,
                enable_pawn_structure: true,
                enable_mobility: true,
                enable_center_control: true,
                enable_development: true,
            },
            base: TaperedEvaluationConfig::default(),
            weights: EvaluationWeights {
                material_weight: 1.0,
                position_weight: 1.0,
                king_safety_weight: 1.2,    // Increased
                pawn_structure_weight: 1.0, // Increased
                mobility_weight: 0.8,       // Increased
                center_control_weight: 0.9, // Increased
                development_weight: 0.7,    // Increased
                tactical_weight: 1.0,
                positional_weight: 1.0,
                castle_weight: 1.0,
            },
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
            auto_validate_weights: true,
            auto_normalize_weights: false,
        }
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            material: MaterialEvaluationConfig {
                include_hand_pieces: true,
                ..MaterialEvaluationConfig::default()
            },
            pst: PieceSquareTableConfig::default(),
            phase_transition: PhaseTransitionConfig {
                default_method: InterpolationMethod::Linear,
                use_phase_boundaries: false,
                sigmoid_steepness: 6.0,
                use_advanced_interpolator: false,
                advanced_config: AdvancedInterpolationConfig::default(),
            },
            position_features: PositionFeatureConfig {
                enable_king_safety: true,
                enable_pawn_structure: true,
                enable_mobility: false,
                enable_center_control: true,
                enable_development: false,
            },
            base: TaperedEvaluationConfig::memory_optimized(),
            weights: EvaluationWeights::default(),
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
            auto_validate_weights: true,
            auto_normalize_weights: false,
        }
    }

    /// Load configuration from a JSON file
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: Self =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn save_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(path, content).map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Validate cumulative weights for enabled components
    /// 
    /// Checks that the sum of all enabled component weights is within a reasonable range (5.0-15.0).
    /// This helps ensure that the evaluation doesn't become too sensitive or too insensitive to
    /// individual components.
    pub fn validate_cumulative_weights(
        &self,
        components: &ComponentFlagsForValidation,
    ) -> Result<(), ConfigError> {
        let mut sum = 0.0;

        if components.material {
            sum += self.weights.material_weight;
        }
        if components.piece_square_tables {
            sum += self.weights.position_weight;
        }
        if components.position_features {
            // Sum all position feature weights
            sum += self.weights.king_safety_weight;
            sum += self.weights.pawn_structure_weight;
            sum += self.weights.mobility_weight;
            sum += self.weights.center_control_weight;
            sum += self.weights.development_weight;
        }
        if components.tactical_patterns {
            sum += self.weights.tactical_weight;
        }
        if components.positional_patterns {
            sum += self.weights.positional_weight;
        }
        if components.castle_patterns {
            sum += self.weights.castle_weight;
        }

        const MIN_CUMULATIVE_WEIGHT: f32 = 5.0;
        const MAX_CUMULATIVE_WEIGHT: f32 = 15.0;

        if sum < MIN_CUMULATIVE_WEIGHT || sum > MAX_CUMULATIVE_WEIGHT {
            return Err(ConfigError::CumulativeWeightOutOfRange {
                sum,
                min: MIN_CUMULATIVE_WEIGHT,
                max: MAX_CUMULATIVE_WEIGHT,
            });
        }

        Ok(())
    }

    /// Apply phase-dependent weight scaling
    /// 
    /// Adjusts weights based on game phase:
    /// - Tactical weights are higher in middlegame
    /// - Positional weights are higher in endgame
    /// 
    /// Phase ranges:
    /// - Opening: phase >= 192
    /// - Middlegame: 64 <= phase < 192
    /// - Endgame: phase < 64
    pub fn apply_phase_scaling(&self, weights: &mut EvaluationWeights, phase: i32) {
        if !self.enable_phase_dependent_weights {
            return;
        }

        // Normalize phase to 0.0-1.0 range (0 = endgame, 1 = opening)
        // Phase range is typically 0-256
        let phase_factor = (phase as f32 / 256.0).clamp(0.0, 1.0);

        // Tactical patterns are more important in middlegame
        // Scale: 0.8 in endgame, 1.2 in middlegame, 1.0 in opening
        let tactical_scale = if phase < 64 {
            0.8 // Endgame: reduce tactical
        } else if phase < 192 {
            1.2 // Middlegame: increase tactical
        } else {
            1.0 // Opening: neutral
        };
        weights.tactical_weight *= tactical_scale;

        // Positional patterns are more important in endgame
        // Scale: 1.2 in endgame, 0.9 in middlegame, 1.0 in opening
        let positional_scale = if phase < 64 {
            1.2 // Endgame: increase positional
        } else if phase < 192 {
            0.9 // Middlegame: reduce positional
        } else {
            1.0 // Opening: neutral
        };
        weights.positional_weight *= positional_scale;
    }

    /// Suggest weight adjustments to maintain balance
    /// 
    /// Analyzes weight ratios and suggests adjustments to maintain a balanced evaluation.
    /// For example, if tactical_weight is 2.0, it might suggest adjusting positional_weight
    /// to maintain balance.
    pub fn suggest_weight_adjustments(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check tactical vs positional balance
        let tactical = self.weights.tactical_weight;
        let positional = self.weights.positional_weight;
        let ratio = if positional > 0.0 {
            tactical / positional
        } else {
            f32::INFINITY
        };

        if ratio > 1.5 {
            suggestions.push(format!(
                "Tactical weight ({:.2}) is significantly higher than positional weight ({:.2}). \
                Consider increasing positional_weight to {:.2} for better balance.",
                tactical,
                positional,
                tactical * 0.8
            ));
        } else if ratio < 0.67 {
            suggestions.push(format!(
                "Positional weight ({:.2}) is significantly higher than tactical weight ({:.2}). \
                Consider increasing tactical_weight to {:.2} for better balance.",
                positional,
                tactical,
                positional * 0.8
            ));
        }

        // Check if any weight is unusually high
        if tactical > 2.0 {
            suggestions.push(format!(
                "Tactical weight ({:.2}) is very high. Consider reducing to maintain evaluation stability.",
                tactical
            ));
        }
        if positional > 2.0 {
            suggestions.push(format!(
                "Positional weight ({:.2}) is very high. Consider reducing to maintain evaluation stability.",
                positional
            ));
        }

        suggestions
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate weights are in reasonable ranges
        if self.weights.material_weight < 0.0 || self.weights.material_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("material_weight".to_string()));
        }

        if self.weights.position_weight < 0.0 || self.weights.position_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("position_weight".to_string()));
        }

        if self.weights.king_safety_weight < 0.0 || self.weights.king_safety_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("king_safety_weight".to_string()));
        }

        if self.weights.pawn_structure_weight < 0.0 || self.weights.pawn_structure_weight > 10.0 {
            return Err(ConfigError::InvalidWeight(
                "pawn_structure_weight".to_string(),
            ));
        }

        if self.weights.mobility_weight < 0.0 || self.weights.mobility_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("mobility_weight".to_string()));
        }

        if self.weights.center_control_weight < 0.0 || self.weights.center_control_weight > 10.0 {
            return Err(ConfigError::InvalidWeight(
                "center_control_weight".to_string(),
            ));
        }

        if self.weights.development_weight < 0.0 || self.weights.development_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("development_weight".to_string()));
        }

        if self.weights.tactical_weight < 0.0 || self.weights.tactical_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("tactical_weight".to_string()));
        }

        if self.weights.castle_weight < 0.0 || self.weights.castle_weight > 10.0 {
            return Err(ConfigError::InvalidWeight("castle_weight".to_string()));
        }

        // Validate sigmoid steepness
        if self.phase_transition.sigmoid_steepness < 1.0
            || self.phase_transition.sigmoid_steepness > 20.0
        {
            return Err(ConfigError::InvalidParameter(
                "sigmoid_steepness must be between 1.0 and 20.0".to_string(),
            ));
        }

        // Note: Cumulative weight validation requires ComponentFlags, which is not available here.
        // It should be called separately with the appropriate component flags, or from
        // IntegratedEvaluationConfig which has both components and weights.

        Ok(())
    }

    /// Update a specific weight at runtime (Task 20.0 - Task 2.0)
    /// 
    /// Automatically validates weights if `auto_validate_weights` is enabled.
    /// Optionally normalizes weights if `auto_normalize_weights` is enabled and
    /// cumulative sum is out of range.
    /// 
    /// # Parameters
    /// - `weight_name`: Name of the weight to update
    /// - `value`: New weight value (must be 0.0-10.0)
    /// - `components`: Optional component flags for cumulative weight validation
    ///   (if None, validation is skipped even if auto_validate_weights is true)
    pub fn update_weight(
        &mut self,
        weight_name: &str,
        value: f32,
        components: Option<&ComponentFlagsForValidation>,
    ) -> Result<(), ConfigError> {
        match weight_name {
            "material" => self.weights.material_weight = value,
            "position" => self.weights.position_weight = value,
            "king_safety" => self.weights.king_safety_weight = value,
            "pawn_structure" => self.weights.pawn_structure_weight = value,
            "mobility" => self.weights.mobility_weight = value,
            "center_control" => self.weights.center_control_weight = value,
            "development" => self.weights.development_weight = value,
            "tactical" => self.weights.tactical_weight = value,
            "positional" => self.weights.positional_weight = value,
            "castle" => self.weights.castle_weight = value,
            _ => return Err(ConfigError::UnknownWeight(weight_name.to_string())),
        }

        // Validate the new weight
        if value < 0.0 || value > 10.0 {
            return Err(ConfigError::InvalidWeight(weight_name.to_string()));
        }

        // Automatic normalization (Task 20.0 - Task 2.8)
        if self.auto_normalize_weights {
            if let Some(components) = components {
                let sum = self.calculate_cumulative_sum(components);
                const MIN_CUMULATIVE_WEIGHT: f32 = 5.0;
                const MAX_CUMULATIVE_WEIGHT: f32 = 15.0;
                if sum < MIN_CUMULATIVE_WEIGHT || sum > MAX_CUMULATIVE_WEIGHT {
                    self.weights.normalize_weights(components);
                }
            }
        }

        // Automatic validation (Task 20.0 - Task 2.1)
        if self.auto_validate_weights {
            if let Some(components) = components {
                self.validate_cumulative_weights(components)?;
            }
        }

        Ok(())
    }

    /// Calculate cumulative sum of enabled component weights
    fn calculate_cumulative_sum(&self, components: &ComponentFlagsForValidation) -> f32 {
        let mut sum = 0.0;
        if components.material {
            sum += self.weights.material_weight;
        }
        if components.piece_square_tables {
            sum += self.weights.position_weight;
        }
        if components.position_features {
            sum += self.weights.king_safety_weight;
            sum += self.weights.pawn_structure_weight;
            sum += self.weights.mobility_weight;
            sum += self.weights.center_control_weight;
            sum += self.weights.development_weight;
        }
        if components.tactical_patterns {
            sum += self.weights.tactical_weight;
        }
        if components.positional_patterns {
            sum += self.weights.positional_weight;
        }
        if components.castle_patterns {
            sum += self.weights.castle_weight;
        }
        sum
    }

    /// Check weight ranges and return warnings for out-of-range weights (Task 20.0 - Task 2.5)
    /// 
    /// Returns a vector of weight names that are outside their recommended ranges.
    /// These are warnings, not errors - weights outside ranges may still be valid.
    pub fn check_weight_ranges(&self) -> Vec<(&'static str, f32, f32, f32)> {
        let mut warnings = Vec::new();
        
        for (name, (min, max, _default)) in RECOMMENDED_WEIGHT_RANGES {
            let value = match *name {
                "material" => self.weights.material_weight,
                "position" => self.weights.position_weight,
                "king_safety" => self.weights.king_safety_weight,
                "pawn_structure" => self.weights.pawn_structure_weight,
                "mobility" => self.weights.mobility_weight,
                "center_control" => self.weights.center_control_weight,
                "development" => self.weights.development_weight,
                "tactical" => self.weights.tactical_weight,
                "positional" => self.weights.positional_weight,
                "castle" => self.weights.castle_weight,
                _ => continue,
            };
            
            if value < *min || value > *max {
                warnings.push((*name, value, *min, *max));
            }
        }
        
        warnings
    }

    /// Backward-compatible wrapper for update_weight without components (Task 20.0 - Task 2.1)
    /// 
    /// This allows existing code to continue working. New code should use the version
    /// with components parameter for automatic validation.
    pub fn update_weight_simple(&mut self, weight_name: &str, value: f32) -> Result<(), ConfigError> {
        self.update_weight(weight_name, value, None)
    }

    /// Apply a weight preset (Task 20.0 - Task 2.11)
    /// 
    /// Sets all weights based on the specified preset style.
    pub fn aggressive_preset(&mut self) {
        self.weights.apply_preset(WeightPreset::Aggressive);
    }

    /// Apply positional preset
    pub fn positional_preset(&mut self) {
        self.weights.apply_preset(WeightPreset::Positional);
    }

    /// Apply defensive preset
    pub fn defensive_preset(&mut self) {
        self.weights.apply_preset(WeightPreset::Defensive);
    }

    /// Apply balanced preset (default)
    pub fn balanced_preset(&mut self) {
        self.weights.apply_preset(WeightPreset::Balanced);
    }

    /// Analyze telemetry for weight recommendations (Task 20.0 - Task 2.12)
    /// 
    /// Takes `EvaluationTelemetry` and suggests weight adjustments based on component
    /// contribution imbalances. Returns a vector of recommendations (component name, suggested adjustment).
    /// 
    /// # Parameters
    /// - `telemetry`: Evaluation telemetry with weight contributions
    /// - `target_contributions`: Optional target contribution percentages (defaults to balanced distribution)
    /// 
    /// # Returns
    /// Vector of recommendations: (component_name, current_contribution, target_contribution, suggested_weight_change)
    pub fn analyze_telemetry_for_recommendations(
        &self,
        telemetry: &crate::evaluation::statistics::EvaluationTelemetry,
        target_contributions: Option<&std::collections::HashMap<String, f32>>,
    ) -> Vec<(String, f32, f32, f32)> {
        use std::collections::HashMap;
        let mut recommendations = Vec::new();
        
        // Default target contributions (balanced distribution)
        let default_targets: HashMap<String, f32> = [
            ("material".to_string(), 0.15),
            ("piece_square_tables".to_string(), 0.12),
            ("position_features".to_string(), 0.25),
            ("tactical_patterns".to_string(), 0.15),
            ("positional_patterns".to_string(), 0.15),
            ("castle_patterns".to_string(), 0.10),
        ]
        .iter()
        .cloned()
        .collect();
        
        let targets = target_contributions.unwrap_or(&default_targets);
        const THRESHOLD: f32 = 0.05; // 5% difference threshold
        
        // Analyze each component
        for (component, current_pct) in &telemetry.weight_contributions {
            if let Some(target_pct) = targets.get(component) {
                let diff = current_pct - target_pct;
                if diff.abs() > THRESHOLD {
                    // Calculate suggested weight adjustment
                    // If contribution is too low, increase weight; if too high, decrease weight
                    let current_weight = self.get_weight(component).unwrap_or(1.0);
                    let adjustment_factor = if current_pct > &0.0 {
                        (target_pct / current_pct).clamp(0.5, 2.0) // Limit to 0.5x-2.0x
                    } else {
                        1.5 // If no contribution, suggest 1.5x increase
                    };
                    let suggested_weight = current_weight * adjustment_factor;
                    let suggested_change = suggested_weight - current_weight;
                    
                    recommendations.push((
                        component.clone(),
                        *current_pct,
                        *target_pct,
                        suggested_change,
                    ));
                }
            }
        }
        
        recommendations
    }

    /// Automatically balance weights using telemetry (Task 20.0 - Task 2.13)
    /// 
    /// Uses telemetry to automatically adjust weights to achieve target contribution percentages.
    /// This method iteratively adjusts weights based on telemetry analysis.
    /// 
    /// # Parameters
    /// - `telemetry`: Evaluation telemetry with weight contributions
    /// - `components`: Component flags for cumulative weight validation
    /// - `target_contributions`: Optional target contribution percentages
    /// - `learning_rate`: Adjustment rate (default: 0.1, range: 0.01-0.5)
    /// 
    /// # Returns
    /// Number of weights adjusted
    pub fn auto_balance_weights(
        &mut self,
        telemetry: &crate::evaluation::statistics::EvaluationTelemetry,
        components: &ComponentFlagsForValidation,
        target_contributions: Option<&std::collections::HashMap<String, f32>>,
        learning_rate: f32,
    ) -> usize {
        let recommendations = self.analyze_telemetry_for_recommendations(telemetry, target_contributions);
        let lr = learning_rate.clamp(0.01, 0.5);
        let mut adjusted = 0;
        
        for (component_name, _current, _target, suggested_change) in recommendations {
            // Map telemetry component names to weight names
            let weight_name = match component_name.as_str() {
                "material" => Some("material"),
                "piece_square_tables" => Some("position"),
                "position_features" => None, // Position features has multiple weights, skip aggregate
                "tactical_patterns" => Some("tactical"),
                "positional_patterns" => Some("positional"),
                "castle_patterns" => Some("castle"),
                _ => None,
            };
            
            if let Some(weight_name) = weight_name {
                if let Some(current_weight) = self.get_weight(weight_name) {
                    // Apply adjustment with learning rate
                    let adjustment = suggested_change * lr;
                    let new_weight = (current_weight + adjustment).clamp(0.0, 10.0);
                    
                    // Update weight (validation happens automatically if enabled)
                    if self.update_weight(weight_name, new_weight, Some(components)).is_ok() {
                        adjusted += 1;
                    }
                }
            }
        }
        
        adjusted
    }

    /// Get a weight value by name
    pub fn get_weight(&self, weight_name: &str) -> Option<f32> {
        match weight_name {
            "material" => Some(self.weights.material_weight),
            "position" => Some(self.weights.position_weight),
            "king_safety" => Some(self.weights.king_safety_weight),
            "pawn_structure" => Some(self.weights.pawn_structure_weight),
            "mobility" => Some(self.weights.mobility_weight),
            "center_control" => Some(self.weights.center_control_weight),
            "development" => Some(self.weights.development_weight),
            "tactical" => Some(self.weights.tactical_weight),
            "positional" => Some(self.weights.positional_weight),
            "castle" => Some(self.weights.castle_weight),
            _ => None,
        }
    }

    /// Enable or disable specific features
    pub fn set_feature_enabled(&mut self, feature: &str, enabled: bool) {
        match feature {
            "king_safety" => self.position_features.enable_king_safety = enabled,
            "pawn_structure" => self.position_features.enable_pawn_structure = enabled,
            "mobility" => self.position_features.enable_mobility = enabled,
            "center_control" => self.position_features.enable_center_control = enabled,
            "development" => self.position_features.enable_development = enabled,
            "hand_pieces" => self.material.include_hand_pieces = enabled,
            _ => {}
        }
    }

    /// Get list of all configurable weights
    pub fn list_weights(&self) -> Vec<(&str, f32)> {
        vec![
            ("material", self.weights.material_weight),
            ("position", self.weights.position_weight),
            ("king_safety", self.weights.king_safety_weight),
            ("pawn_structure", self.weights.pawn_structure_weight),
            ("mobility", self.weights.mobility_weight),
            ("center_control", self.weights.center_control_weight),
            ("development", self.weights.development_weight),
            ("tactical", self.weights.tactical_weight),
        ]
    }
}

impl Default for TaperedEvalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            material: MaterialEvaluationConfig::default(),
            pst: PieceSquareTableConfig::default(),
            phase_transition: PhaseTransitionConfig::default(),
            position_features: PositionFeatureConfig::default(),
            base: TaperedEvaluationConfig::default(),
            weights: EvaluationWeights::default(),
            enable_phase_dependent_weights: false,
            weight_contribution_threshold: 1000.0,
            auto_validate_weights: true,
            auto_normalize_weights: false,
        }
    }
}

/// Errors that can occur during configuration operations
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// IO error during file operations
    IoError(String),
    /// Parse error during deserialization
    ParseError(String),
    /// Serialize error
    SerializeError(String),
    /// Invalid weight value
    InvalidWeight(String),
    /// Invalid parameter value
    InvalidParameter(String),
    /// Unknown weight name
    UnknownWeight(String),
    /// Cumulative weight sum is out of acceptable range
    CumulativeWeightOutOfRange {
        sum: f32,
        min: f32,
        max: f32,
    },
}

/// Phase boundary configuration for game phase transitions
///
/// Controls when the evaluation transitions between opening, middlegame, and endgame phases.
/// Phase is calculated based on material remaining on the board (0-256 scale).
///
/// # Default Values
///
/// - `opening_threshold`: 192 (phase >= 192 is opening)
/// - `endgame_threshold`: 64 (phase < 64 is endgame)
/// - `opening_fade_start`: 192 (opening principles start fading at this phase)
/// - `opening_fade_end`: 160 (opening principles fully faded by this phase)
/// - `endgame_fade_start`: 80 (endgame patterns start fading at this phase)
/// - `endgame_fade_end`: 64 (endgame patterns fully faded by this phase)
///
/// # Gradual Phase Transitions
///
/// When `enable_gradual_phase_transitions` is enabled, pattern scores are gradually
/// faded out instead of abruptly cut off:
///
/// - Opening principles: Fade from `opening_fade_start` (192) to `opening_fade_end` (160)
/// - Endgame patterns: Fade from `endgame_fade_start` (80) to `endgame_fade_end` (64)
///
/// The fade factor is calculated as:
/// ```
/// fade_factor = (phase - fade_end) / (fade_start - fade_end)
/// ```
/// This produces a linear fade from 1.0 (at fade_start) to 0.0 (at fade_end).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseBoundaryConfig {
    /// Phase threshold for opening (phase >= this is opening, default: 192)
    pub opening_threshold: i32,
    /// Phase threshold for endgame (phase < this is endgame, default: 64)
    pub endgame_threshold: i32,
    /// Phase where opening principles start fading (default: 192)
    pub opening_fade_start: i32,
    /// Phase where opening principles finish fading (default: 160)
    pub opening_fade_end: i32,
    /// Phase where endgame patterns start fading (default: 80)
    pub endgame_fade_start: i32,
    /// Phase where endgame patterns finish fading (default: 64)
    pub endgame_fade_end: i32,
}

impl Default for PhaseBoundaryConfig {
    fn default() -> Self {
        Self {
            opening_threshold: 192,
            endgame_threshold: 64,
            opening_fade_start: 192,
            opening_fade_end: 160,
            endgame_fade_start: 80,
            endgame_fade_end: 64,
        }
    }
}

impl PhaseBoundaryConfig {
    /// Calculate fade factor for opening principles based on current phase
    ///
    /// Returns:
    /// - 1.0 if phase >= opening_fade_start (full opening evaluation)
    /// - 0.0 if phase <= opening_fade_end (no opening evaluation)
    /// - Linear interpolation between fade_start and fade_end
    pub fn calculate_opening_fade_factor(&self, phase: i32) -> f32 {
        if phase >= self.opening_fade_start {
            return 1.0;
        }
        if phase <= self.opening_fade_end {
            return 0.0;
        }
        let fade_range = (self.opening_fade_start - self.opening_fade_end) as f32;
        if fade_range <= 0.0 {
            return 1.0;
        }
        let phase_in_range = (phase - self.opening_fade_end) as f32;
        (phase_in_range / fade_range).clamp(0.0, 1.0)
    }

    /// Calculate fade factor for endgame patterns based on current phase
    ///
    /// Returns:
    /// - 1.0 if phase <= endgame_fade_end (full endgame evaluation)
    /// - 0.0 if phase >= endgame_fade_start (no endgame evaluation)
    /// - Linear interpolation between fade_end and fade_start
    pub fn calculate_endgame_fade_factor(&self, phase: i32) -> f32 {
        if phase <= self.endgame_fade_end {
            return 1.0;
        }
        if phase >= self.endgame_fade_start {
            return 0.0;
        }
        let fade_range = (self.endgame_fade_start - self.endgame_fade_end) as f32;
        if fade_range <= 0.0 {
            return 1.0;
        }
        let phase_in_range = (self.endgame_fade_start - phase) as f32;
        (phase_in_range / fade_range).clamp(0.0, 1.0)
    }
}

/// Component dependency warnings for configuration validation
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentDependencyWarning {
    /// Center control overlap: both position_features and positional_patterns evaluate center control
    CenterControlOverlap,
    /// Development overlap: both position_features and opening_principles evaluate development
    /// Note: Automatically handled during evaluation (opening_principles takes precedence in opening)
    DevelopmentOverlap,
    /// Endgame patterns enabled but phase is not endgame (informational)
    EndgamePatternsNotInEndgame,
    /// Enabled component produced zero score (may indicate configuration issue)
    ComponentProducedZeroScore(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::InvalidWeight(name) => write!(f, "Invalid weight: {}", name),
            ConfigError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            ConfigError::UnknownWeight(name) => write!(f, "Unknown weight: {}", name),
            ConfigError::CumulativeWeightOutOfRange { sum, min, max } => {
                write!(
                    f,
                    "Cumulative weight sum {} is out of range [{}, {}]",
                    sum, min, max
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Component flags for weight validation
/// 
/// This is a simplified version of ComponentFlags from integration.rs
/// used for cumulative weight validation in TaperedEvalConfig.
#[derive(Debug, Clone)]
pub struct ComponentFlagsForValidation {
    pub material: bool,
    pub piece_square_tables: bool,
    pub position_features: bool,
    pub tactical_patterns: bool,
    pub positional_patterns: bool,
    pub castle_patterns: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = TaperedEvalConfig::new();
        assert!(config.enabled);
    }

    #[test]
    fn test_default_config() {
        let config = TaperedEvalConfig::default();
        assert!(config.enabled);
        assert_eq!(config.weights.material_weight, 1.0);
    }

    #[test]
    fn test_disabled_config() {
        let config = TaperedEvalConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_performance_optimized() {
        let config = TaperedEvalConfig::performance_optimized();
        assert!(config.enabled);
        assert_eq!(
            config.phase_transition.default_method,
            InterpolationMethod::Linear
        );
        assert!(!config.position_features.enable_mobility); // Disabled for speed
    }

    #[test]
    fn test_strength_optimized() {
        let config = TaperedEvalConfig::strength_optimized();
        assert!(config.enabled);
        assert_eq!(
            config.phase_transition.default_method,
            InterpolationMethod::Smoothstep
        );
        assert!(config.position_features.enable_mobility); // Enabled for accuracy
    }

    #[test]
    fn test_memory_optimized() {
        let config = TaperedEvalConfig::memory_optimized();
        assert!(config.enabled);
        assert!(!config.position_features.enable_mobility);
        assert!(!config.position_features.enable_development);
    }

    #[test]
    fn test_validate_default() {
        let config = TaperedEvalConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_weight() {
        let mut config = TaperedEvalConfig::default();
        config.weights.material_weight = -1.0; // Invalid

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_weight_too_large() {
        let mut config = TaperedEvalConfig::default();
        config.weights.mobility_weight = 15.0; // Too large

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_sigmoid() {
        let mut config = TaperedEvalConfig::default();
        config.phase_transition.sigmoid_steepness = 0.5; // Too small

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_update_weight() {
        let mut config = TaperedEvalConfig::default();

        assert!(config.update_weight("material", 1.5, None).is_ok());
        assert_eq!(config.weights.material_weight, 1.5);

        assert!(config.update_weight("king_safety", 0.8, None).is_ok());
        assert_eq!(config.weights.king_safety_weight, 0.8);
    }

    #[test]
    fn test_update_weight_invalid() {
        let mut config = TaperedEvalConfig::default();

        // Invalid weight value
        assert!(config.update_weight("material", -1.0, None).is_err());

        // Unknown weight name
        assert!(config.update_weight("unknown", 1.0, None).is_err());
    }

    #[test]
    fn test_get_weight() {
        let config = TaperedEvalConfig::default();

        assert_eq!(config.get_weight("material"), Some(1.0));
        assert_eq!(config.get_weight("mobility"), Some(0.6));
        assert_eq!(config.get_weight("unknown"), None);
    }

    #[test]
    fn test_set_feature_enabled() {
        let mut config = TaperedEvalConfig::default();

        assert!(config.position_features.enable_mobility);
        config.set_feature_enabled("mobility", false);
        assert!(!config.position_features.enable_mobility);

        assert!(config.material.include_hand_pieces);
        config.set_feature_enabled("hand_pieces", false);
        assert!(!config.material.include_hand_pieces);
    }

    #[test]
    fn test_list_weights() {
        let config = TaperedEvalConfig::default();
        let weights = config.list_weights();

        assert_eq!(weights.len(), 7);
        assert_eq!(weights[0].0, "material");
        assert_eq!(weights[0].1, 1.0);
    }

    #[test]
    fn test_serialization() {
        let config = TaperedEvalConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        // Deserialize back
        let deserialized: Result<TaperedEvalConfig, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(config, deserialized.unwrap());
    }

    #[test]
    fn test_config_clone() {
        let config1 = TaperedEvalConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1, config2);
    }

    #[test]
    fn test_weights_default() {
        let weights = EvaluationWeights::default();

        assert_eq!(weights.material_weight, 1.0);
        assert_eq!(weights.position_weight, 1.0);
        assert!(weights.mobility_weight > 0.0);
        assert!(weights.development_weight > 0.0);
    }

    #[test]
    fn test_runtime_weight_update() {
        let mut config = TaperedEvalConfig::default();

        // Update multiple weights
        assert!(config.update_weight("material", 1.2, None).is_ok());
        assert!(config.update_weight("position", 0.9, None).is_ok());
        assert!(config.update_weight("king_safety", 1.1, None).is_ok());

        // Verify changes
        assert_eq!(config.weights.material_weight, 1.2);
        assert_eq!(config.weights.position_weight, 0.9);
        assert_eq!(config.weights.king_safety_weight, 1.1);

        // Configuration should still be valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_feature_toggles() {
        let mut config = TaperedEvalConfig::default();

        // Disable all features
        config.set_feature_enabled("king_safety", false);
        config.set_feature_enabled("pawn_structure", false);
        config.set_feature_enabled("mobility", false);
        config.set_feature_enabled("center_control", false);
        config.set_feature_enabled("development", false);

        // Verify all disabled
        assert!(!config.position_features.enable_king_safety);
        assert!(!config.position_features.enable_pawn_structure);
        assert!(!config.position_features.enable_mobility);
        assert!(!config.position_features.enable_center_control);
        assert!(!config.position_features.enable_development);
    }

    #[test]
    fn test_preset_configs_valid() {
        // All preset configs should be valid
        assert!(TaperedEvalConfig::default().validate().is_ok());
        assert!(TaperedEvalConfig::disabled().validate().is_ok());
        assert!(TaperedEvalConfig::performance_optimized()
            .validate()
            .is_ok());
        assert!(TaperedEvalConfig::strength_optimized().validate().is_ok());
        assert!(TaperedEvalConfig::memory_optimized().validate().is_ok());
    }

    #[test]
    fn test_config_equality() {
        let config1 = TaperedEvalConfig::default();
        let config2 = TaperedEvalConfig::default();

        assert_eq!(config1, config2);

        let mut config3 = TaperedEvalConfig::default();
        config3.weights.material_weight = 1.5;

        assert_ne!(config1, config3);
    }
}
