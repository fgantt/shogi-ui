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
}

/// Weights for combining different evaluation components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationWeights {
    /// Weight for material evaluation (typically 1.0)
    pub material_weight: f32,

    /// Weight for piece-square tables
    pub position_weight: f32,

    /// Weight for king safety
    pub king_safety_weight: f32,

    /// Weight for pawn structure
    pub pawn_structure_weight: f32,

    /// Weight for mobility
    pub mobility_weight: f32,

    /// Weight for center control
    pub center_control_weight: f32,

    /// Weight for development
    pub development_weight: f32,

    /// Weight for tactical pattern contributions
    pub tactical_weight: f32,
    /// Weight for positional pattern contributions
    pub positional_weight: f32,
    /// Weight for castle pattern contributions
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

        Ok(())
    }

    /// Update a specific weight at runtime
    pub fn update_weight(&mut self, weight_name: &str, value: f32) -> Result<(), ConfigError> {
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

        Ok(())
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
        }
    }
}

impl std::error::Error for ConfigError {}

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

        assert!(config.update_weight("material", 1.5).is_ok());
        assert_eq!(config.weights.material_weight, 1.5);

        assert!(config.update_weight("king_safety", 0.8).is_ok());
        assert_eq!(config.weights.king_safety_weight, 0.8);
    }

    #[test]
    fn test_update_weight_invalid() {
        let mut config = TaperedEvalConfig::default();

        // Invalid weight value
        assert!(config.update_weight("material", -1.0).is_err());

        // Unknown weight name
        assert!(config.update_weight("unknown", 1.0).is_err());
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
        assert!(config.update_weight("material", 1.2).is_ok());
        assert!(config.update_weight("position", 0.9).is_ok());
        assert!(config.update_weight("king_safety", 1.1).is_ok());

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
