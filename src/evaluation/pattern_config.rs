//! Pattern Recognition Configuration Module
//!
//! This module provides comprehensive configuration for all pattern recognition features.
//! It supports:
//! - Individual pattern type enable/disable
//! - Weight configuration for all patterns
//! - Runtime configuration updates
//! - Configuration validation
//! - Serialization/deserialization for persistence
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::pattern_config::{PatternConfig, PatternWeights};
//!
//! let mut config = PatternConfig::default();
//! config.enable_all();
//! config.set_piece_square_table_weight(1.2);
//!
//! if let Err(e) = config.validate() {
//!     println!("Configuration error: {}", e);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for pattern recognition system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Enable/disable individual pattern types
    pub patterns: PatternTypes,

    /// Weights for each pattern type
    pub weights: PatternWeights,

    /// Advanced configuration options
    pub advanced: AdvancedPatternConfig,
}

impl PatternConfig {
    /// Create a new PatternConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable all pattern types
    pub fn enable_all(&mut self) {
        self.patterns.enable_all();
    }

    /// Disable all pattern types
    pub fn disable_all(&mut self) {
        self.patterns.disable_all();
    }

    /// Validate the configuration
    ///
    /// Returns Ok(()) if configuration is valid, Err with description otherwise
    pub fn validate(&self) -> Result<(), String> {
        // Validate weights are in reasonable ranges
        self.weights.validate()?;

        // Validate advanced options
        self.advanced.validate()?;

        // Ensure at least one pattern type is enabled
        if !self.patterns.any_enabled() {
            return Err("At least one pattern type must be enabled".to_string());
        }

        Ok(())
    }

    /// Update configuration from another config (runtime update)
    pub fn update_from(&mut self, other: &PatternConfig) -> Result<(), String> {
        // Validate the new configuration before applying
        other.validate()?;

        // Apply updates
        self.patterns = other.patterns.clone();
        self.weights = other.weights.clone();
        self.advanced = other.advanced.clone();

        Ok(())
    }

    /// Get piece-square table weight
    pub fn piece_square_table_weight(&self) -> f32 {
        self.weights.piece_square_tables
    }

    /// Set piece-square table weight
    pub fn set_piece_square_table_weight(&mut self, weight: f32) {
        self.weights.piece_square_tables = weight;
    }

    /// Get pawn structure weight
    pub fn pawn_structure_weight(&self) -> f32 {
        self.weights.pawn_structure
    }

    /// Set pawn structure weight
    pub fn set_pawn_structure_weight(&mut self, weight: f32) {
        self.weights.pawn_structure = weight;
    }

    /// Get king safety weight
    pub fn king_safety_weight(&self) -> f32 {
        self.weights.king_safety
    }

    /// Set king safety weight
    pub fn set_king_safety_weight(&mut self, weight: f32) {
        self.weights.king_safety = weight;
    }

    /// Get piece coordination weight
    pub fn piece_coordination_weight(&self) -> f32 {
        self.weights.piece_coordination
    }

    /// Set piece coordination weight
    pub fn set_piece_coordination_weight(&mut self, weight: f32) {
        self.weights.piece_coordination = weight;
    }

    /// Get mobility weight
    pub fn mobility_weight(&self) -> f32 {
        self.weights.mobility
    }

    /// Set mobility weight
    pub fn set_mobility_weight(&mut self, weight: f32) {
        self.weights.mobility = weight;
    }

    /// Load configuration from JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Save configuration to JSON string
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            patterns: PatternTypes::default(),
            weights: PatternWeights::default(),
            advanced: AdvancedPatternConfig::default(),
        }
    }
}

/// Enable/disable flags for each pattern type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternTypes {
    /// Enable piece-square tables
    pub piece_square_tables: bool,

    /// Enable pawn structure evaluation
    pub pawn_structure: bool,

    /// Enable king safety patterns
    pub king_safety: bool,

    /// Enable piece coordination patterns
    pub piece_coordination: bool,

    /// Enable mobility patterns
    pub mobility: bool,

    /// Enable tactical patterns (forks, pins, skewers)
    pub tactical_patterns: bool,

    /// Enable positional patterns (outposts, weak squares)
    pub positional_patterns: bool,

    /// Enable endgame patterns
    pub endgame_patterns: bool,
}

impl PatternTypes {
    /// Enable all pattern types
    pub fn enable_all(&mut self) {
        self.piece_square_tables = true;
        self.pawn_structure = true;
        self.king_safety = true;
        self.piece_coordination = true;
        self.mobility = true;
        self.tactical_patterns = true;
        self.positional_patterns = true;
        self.endgame_patterns = true;
    }

    /// Disable all pattern types
    pub fn disable_all(&mut self) {
        self.piece_square_tables = false;
        self.pawn_structure = false;
        self.king_safety = false;
        self.piece_coordination = false;
        self.mobility = false;
        self.tactical_patterns = false;
        self.positional_patterns = false;
        self.endgame_patterns = false;
    }

    /// Check if any pattern type is enabled
    pub fn any_enabled(&self) -> bool {
        self.piece_square_tables
            || self.pawn_structure
            || self.king_safety
            || self.piece_coordination
            || self.mobility
            || self.tactical_patterns
            || self.positional_patterns
            || self.endgame_patterns
    }

    /// Count how many pattern types are enabled
    pub fn count_enabled(&self) -> usize {
        let mut count = 0;
        if self.piece_square_tables {
            count += 1;
        }
        if self.pawn_structure {
            count += 1;
        }
        if self.king_safety {
            count += 1;
        }
        if self.piece_coordination {
            count += 1;
        }
        if self.mobility {
            count += 1;
        }
        if self.tactical_patterns {
            count += 1;
        }
        if self.positional_patterns {
            count += 1;
        }
        if self.endgame_patterns {
            count += 1;
        }
        count
    }
}

impl Default for PatternTypes {
    fn default() -> Self {
        Self {
            // Core patterns enabled by default
            piece_square_tables: true,
            pawn_structure: true,
            king_safety: true,
            piece_coordination: true,
            mobility: true,

            // Advanced patterns disabled by default
            tactical_patterns: false,
            positional_patterns: false,
            endgame_patterns: false,
        }
    }
}

/// Weights for each pattern type
///
/// Weights are multiplicative factors applied to pattern scores.
/// Default weight is 1.0 (no scaling).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternWeights {
    /// Piece-square table weight
    pub piece_square_tables: f32,

    /// Pawn structure weight
    pub pawn_structure: f32,

    /// King safety weight
    pub king_safety: f32,

    /// Piece coordination weight
    pub piece_coordination: f32,

    /// Mobility weight
    pub mobility: f32,

    /// Tactical patterns weight
    pub tactical_patterns: f32,

    /// Positional patterns weight
    pub positional_patterns: f32,

    /// Endgame patterns weight
    pub endgame_patterns: f32,
}

impl PatternWeights {
    /// Validate that all weights are in reasonable ranges
    pub fn validate(&self) -> Result<(), String> {
        let weights = [
            ("piece_square_tables", self.piece_square_tables),
            ("pawn_structure", self.pawn_structure),
            ("king_safety", self.king_safety),
            ("piece_coordination", self.piece_coordination),
            ("mobility", self.mobility),
            ("tactical_patterns", self.tactical_patterns),
            ("positional_patterns", self.positional_patterns),
            ("endgame_patterns", self.endgame_patterns),
        ];

        for (name, weight) in weights {
            if weight < 0.0 {
                return Err(format!("Weight '{}' cannot be negative: {}", name, weight));
            }
            if weight > 10.0 {
                return Err(format!(
                    "Weight '{}' is too large (max 10.0): {}",
                    name, weight
                ));
            }
            if !weight.is_finite() {
                return Err(format!("Weight '{}' must be finite: {}", name, weight));
            }
        }

        Ok(())
    }

    /// Reset all weights to default values
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}

impl Default for PatternWeights {
    fn default() -> Self {
        Self {
            piece_square_tables: 1.0,
            pawn_structure: 1.0,
            king_safety: 1.0,
            piece_coordination: 1.0,
            mobility: 1.0,
            tactical_patterns: 1.0,
            positional_patterns: 1.0,
            endgame_patterns: 1.0,
        }
    }
}

/// Advanced pattern configuration options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedPatternConfig {
    /// Enable pattern caching for performance
    pub enable_caching: bool,

    /// Maximum cache size (number of entries)
    pub cache_size: usize,

    /// Enable incremental pattern updates
    pub incremental_updates: bool,

    /// Enable pattern statistics collection
    pub collect_statistics: bool,

    /// Minimum depth to apply patterns (for search)
    pub min_depth: u8,

    /// Maximum depth to apply patterns (for search)
    pub max_depth: u8,
}

impl AdvancedPatternConfig {
    /// Validate advanced configuration options
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth > self.max_depth {
            return Err(format!(
                "min_depth ({}) cannot be greater than max_depth ({})",
                self.min_depth, self.max_depth
            ));
        }

        if self.cache_size == 0 && self.enable_caching {
            return Err("cache_size must be positive when caching is enabled".to_string());
        }

        if self.cache_size > 10_000_000 {
            return Err(format!(
                "cache_size is too large (max 10M): {}",
                self.cache_size
            ));
        }

        Ok(())
    }
}

impl Default for AdvancedPatternConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size: 100_000,
            incremental_updates: true,
            collect_statistics: false,
            min_depth: 0,
            max_depth: 100,
        }
    }
}

impl fmt::Display for PatternConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pattern Recognition Configuration:")?;
        writeln!(f, "  Enabled Patterns: {}", self.patterns.count_enabled())?;
        writeln!(
            f,
            "  - Piece-Square Tables: {} (weight: {:.2})",
            self.patterns.piece_square_tables, self.weights.piece_square_tables
        )?;
        writeln!(
            f,
            "  - Pawn Structure: {} (weight: {:.2})",
            self.patterns.pawn_structure, self.weights.pawn_structure
        )?;
        writeln!(
            f,
            "  - King Safety: {} (weight: {:.2})",
            self.patterns.king_safety, self.weights.king_safety
        )?;
        writeln!(
            f,
            "  - Piece Coordination: {} (weight: {:.2})",
            self.patterns.piece_coordination, self.weights.piece_coordination
        )?;
        writeln!(
            f,
            "  - Mobility: {} (weight: {:.2})",
            self.patterns.mobility, self.weights.mobility
        )?;
        writeln!(
            f,
            "  - Tactical Patterns: {} (weight: {:.2})",
            self.patterns.tactical_patterns, self.weights.tactical_patterns
        )?;
        writeln!(
            f,
            "  - Positional Patterns: {} (weight: {:.2})",
            self.patterns.positional_patterns, self.weights.positional_patterns
        )?;
        writeln!(
            f,
            "  - Endgame Patterns: {} (weight: {:.2})",
            self.patterns.endgame_patterns, self.weights.endgame_patterns
        )?;
        writeln!(f, "  Advanced:")?;
        writeln!(f, "    - Caching: {}", self.advanced.enable_caching)?;
        writeln!(f, "    - Cache Size: {}", self.advanced.cache_size)?;
        writeln!(
            f,
            "    - Incremental Updates: {}",
            self.advanced.incremental_updates
        )?;
        writeln!(
            f,
            "    - Collect Statistics: {}",
            self.advanced.collect_statistics
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_config_creation() {
        let config = PatternConfig::new();
        assert!(config.patterns.piece_square_tables);
        assert!(config.patterns.pawn_structure);
        assert!(config.patterns.king_safety);
    }

    #[test]
    fn test_pattern_config_default() {
        let config = PatternConfig::default();
        assert_eq!(config.weights.piece_square_tables, 1.0);
        assert_eq!(config.weights.mobility, 1.0);
        assert!(config.advanced.enable_caching);
    }

    #[test]
    fn test_enable_all_patterns() {
        let mut config = PatternConfig::new();
        config.enable_all();

        assert!(config.patterns.piece_square_tables);
        assert!(config.patterns.pawn_structure);
        assert!(config.patterns.king_safety);
        assert!(config.patterns.piece_coordination);
        assert!(config.patterns.mobility);
        assert!(config.patterns.tactical_patterns);
        assert!(config.patterns.positional_patterns);
        assert!(config.patterns.endgame_patterns);
    }

    #[test]
    fn test_disable_all_patterns() {
        let mut config = PatternConfig::new();
        config.disable_all();

        assert!(!config.patterns.piece_square_tables);
        assert!(!config.patterns.pawn_structure);
        assert!(!config.patterns.king_safety);
        assert!(!config.patterns.any_enabled());
    }

    #[test]
    fn test_pattern_count() {
        let mut config = PatternConfig::default();

        // Default has 5 enabled (core patterns)
        assert_eq!(config.patterns.count_enabled(), 5);

        config.enable_all();
        assert_eq!(config.patterns.count_enabled(), 8);

        config.disable_all();
        assert_eq!(config.patterns.count_enabled(), 0);
    }

    #[test]
    fn test_weight_validation() {
        let mut weights = PatternWeights::default();
        assert!(weights.validate().is_ok());

        // Test negative weight
        weights.piece_square_tables = -1.0;
        assert!(weights.validate().is_err());

        // Test too large weight
        weights.piece_square_tables = 11.0;
        assert!(weights.validate().is_err());

        // Test infinite weight
        weights.piece_square_tables = f32::INFINITY;
        assert!(weights.validate().is_err());

        // Test NaN weight
        weights.piece_square_tables = f32::NAN;
        assert!(weights.validate().is_err());
    }

    #[test]
    fn test_config_validation() {
        let mut config = PatternConfig::default();
        assert!(config.validate().is_ok());

        // Test with invalid weights
        config.weights.king_safety = -1.0;
        assert!(config.validate().is_err());

        // Reset and test with all patterns disabled
        config = PatternConfig::default();
        config.disable_all();
        assert!(config.validate().is_err()); // Should fail - no patterns enabled
    }

    #[test]
    fn test_weight_getters_setters() {
        let mut config = PatternConfig::new();

        config.set_piece_square_table_weight(1.5);
        assert_eq!(config.piece_square_table_weight(), 1.5);

        config.set_pawn_structure_weight(2.0);
        assert_eq!(config.pawn_structure_weight(), 2.0);

        config.set_king_safety_weight(1.2);
        assert_eq!(config.king_safety_weight(), 1.2);

        config.set_piece_coordination_weight(0.8);
        assert_eq!(config.piece_coordination_weight(), 0.8);

        config.set_mobility_weight(1.1);
        assert_eq!(config.mobility_weight(), 1.1);
    }

    #[test]
    fn test_runtime_update() {
        let mut config1 = PatternConfig::default();
        let mut config2 = PatternConfig::default();

        config2.set_king_safety_weight(2.0);
        config2.patterns.tactical_patterns = true;

        assert!(config1.update_from(&config2).is_ok());
        assert_eq!(config1.king_safety_weight(), 2.0);
        assert!(config1.patterns.tactical_patterns);
    }

    #[test]
    fn test_runtime_update_validation() {
        let mut config1 = PatternConfig::default();
        let mut config2 = PatternConfig::default();

        // Make config2 invalid
        config2.weights.mobility = -1.0;

        // Update should fail
        assert!(config1.update_from(&config2).is_err());

        // config1 should remain unchanged
        assert_eq!(config1.mobility_weight(), 1.0);
    }

    #[test]
    fn test_json_serialization() {
        let config = PatternConfig::default();

        let json = config.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("piece_square_tables"));
        assert!(json_str.contains("king_safety"));
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{
            "patterns": {
                "piece_square_tables": true,
                "pawn_structure": true,
                "king_safety": true,
                "piece_coordination": true,
                "mobility": true,
                "tactical_patterns": false,
                "positional_patterns": false,
                "endgame_patterns": false
            },
            "weights": {
                "piece_square_tables": 1.0,
                "pawn_structure": 1.0,
                "king_safety": 1.5,
                "piece_coordination": 1.0,
                "mobility": 1.0,
                "tactical_patterns": 1.0,
                "positional_patterns": 1.0,
                "endgame_patterns": 1.0
            },
            "advanced": {
                "enable_caching": true,
                "cache_size": 100000,
                "incremental_updates": true,
                "collect_statistics": false,
                "min_depth": 0,
                "max_depth": 100
            }
        }"#;

        let config = PatternConfig::from_json(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.king_safety_weight(), 1.5);
        assert!(config.patterns.piece_square_tables);
    }

    #[test]
    fn test_advanced_config_validation() {
        let mut advanced = AdvancedPatternConfig::default();
        assert!(advanced.validate().is_ok());

        // Test invalid depth range
        advanced.min_depth = 10;
        advanced.max_depth = 5;
        assert!(advanced.validate().is_err());

        // Test zero cache size with caching enabled
        advanced = AdvancedPatternConfig::default();
        advanced.cache_size = 0;
        advanced.enable_caching = true;
        assert!(advanced.validate().is_err());

        // Test too large cache
        advanced = AdvancedPatternConfig::default();
        advanced.cache_size = 20_000_000;
        assert!(advanced.validate().is_err());
    }

    #[test]
    fn test_pattern_weights_reset() {
        let mut weights = PatternWeights::default();
        weights.king_safety = 2.0;
        weights.mobility = 1.5;

        weights.reset_to_default();

        assert_eq!(weights.king_safety, 1.0);
        assert_eq!(weights.mobility, 1.0);
    }

    #[test]
    fn test_display_format() {
        let config = PatternConfig::default();
        let display = format!("{}", config);

        assert!(display.contains("Pattern Recognition Configuration"));
        assert!(display.contains("Enabled Patterns"));
        assert!(display.contains("weight"));
    }
}
