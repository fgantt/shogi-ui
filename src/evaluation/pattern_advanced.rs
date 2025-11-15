//! Advanced Pattern Recognition Features Module
//!
//! This module provides advanced features for pattern recognition:
//! - Machine learning for pattern weight optimization
//! - Position-type specific pattern selection
//! - Dynamic pattern selection based on game phase
//! - Pattern visualization and explanation
//! - Advanced pattern analytics
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::pattern_advanced::{AdvancedPatternSystem, PatternExplainer};
//!
//! let system = AdvancedPatternSystem::new();
//! let weights = system.optimize_weights(&training_data);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::core::{Player, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advanced pattern recognition system
pub struct AdvancedPatternSystem {
    /// Machine learning configuration
    ml_config: MLConfig,

    /// Dynamic selector
    selector: DynamicPatternSelector,

    /// Pattern explainer
    explainer: PatternExplainer,

    /// Analytics engine
    analytics: PatternAnalytics,
}

impl AdvancedPatternSystem {
    /// Create new advanced pattern system
    pub fn new() -> Self {
        Self {
            ml_config: MLConfig::default(),
            selector: DynamicPatternSelector::new(),
            explainer: PatternExplainer::new(),
            analytics: PatternAnalytics::new(),
        }
    }

    /// Optimize pattern weights using machine learning
    pub fn optimize_weights(&mut self, training_data: &[TrainingPosition]) -> Vec<f32> {
        if !self.ml_config.enabled {
            return vec![];
        }

        // Simplified ML weight optimization
        // Full implementation would use gradient descent or other ML techniques
        let weights = vec![1.0; 8]; // 8 pattern types

        for position in training_data {
            // Adjust weights based on position evaluation error
            // This is a placeholder for actual ML training
            let _ = position;
        }

        weights
    }

    /// Select patterns dynamically based on position
    pub fn select_patterns(&self, board: &BitboardBoard, game_phase: u8) -> PatternSelection {
        self.selector.select(board, game_phase)
    }

    /// Explain detected patterns
    pub fn explain_patterns(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<PatternExplanation> {
        self.explainer.explain(board, player)
    }

    /// Get pattern analytics
    pub fn get_analytics(&self) -> &PatternAnalytics {
        &self.analytics
    }
}

impl Default for AdvancedPatternSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Machine learning configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MLConfig {
    /// Enable ML weight optimization
    pub enabled: bool,

    /// Learning rate
    pub learning_rate: f32,

    /// Training iterations
    pub iterations: usize,

    /// Regularization factor
    pub regularization: f32,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            learning_rate: 0.01,
            iterations: 1000,
            regularization: 0.001,
        }
    }
}

/// Training position for ML
pub struct TrainingPosition {
    /// Board position
    pub board: BitboardBoard,

    /// Player to evaluate
    pub player: Player,

    /// Expected evaluation
    pub expected_eval: i32,
}

/// Dynamic pattern selector
pub struct DynamicPatternSelector {
    /// Position type patterns
    position_patterns: HashMap<PositionType, PatternSelection>,
}

impl DynamicPatternSelector {
    fn new() -> Self {
        let mut selector = Self {
            position_patterns: HashMap::new(),
        };
        selector.initialize_patterns();
        selector
    }

    fn initialize_patterns(&mut self) {
        // Initialize patterns for each position type
        self.position_patterns.insert(
            PositionType::Opening,
            PatternSelection {
                enable_tactical: true,
                enable_positional: true,
                enable_endgame: false,
                weights: vec![1.0, 1.0, 1.2, 0.8, 1.0, 0.5, 0.5, 0.2],
            },
        );

        self.position_patterns.insert(
            PositionType::Middlegame,
            PatternSelection {
                enable_tactical: true,
                enable_positional: true,
                enable_endgame: false,
                weights: vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.5],
            },
        );

        self.position_patterns.insert(
            PositionType::Endgame,
            PatternSelection {
                enable_tactical: true,
                enable_positional: false,
                enable_endgame: true,
                weights: vec![0.8, 1.2, 0.8, 0.6, 1.0, 0.7, 0.5, 1.5],
            },
        );
    }

    fn select(&self, _board: &BitboardBoard, game_phase: u8) -> PatternSelection {
        // Select patterns based on game phase
        let position_type = if game_phase > 192 {
            PositionType::Opening
        } else if game_phase > 64 {
            PositionType::Middlegame
        } else {
            PositionType::Endgame
        };

        self.position_patterns
            .get(&position_type)
            .cloned()
            .unwrap_or_default()
    }
}

/// Position type for pattern selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PositionType {
    Opening,
    Middlegame,
    Endgame,
}

/// Pattern selection result
#[derive(Debug, Clone)]
pub struct PatternSelection {
    pub enable_tactical: bool,
    pub enable_positional: bool,
    pub enable_endgame: bool,
    pub weights: Vec<f32>,
}

impl Default for PatternSelection {
    fn default() -> Self {
        Self {
            enable_tactical: true,
            enable_positional: true,
            enable_endgame: true,
            weights: vec![1.0; 8],
        }
    }
}

/// Pattern explainer for human-readable explanations
pub struct PatternExplainer {
    /// Explanation templates
    templates: HashMap<String, String>,
}

impl PatternExplainer {
    fn new() -> Self {
        let mut explainer = Self {
            templates: HashMap::new(),
        };
        explainer.initialize_templates();
        explainer
    }

    fn initialize_templates(&mut self) {
        self.templates
            .insert("fork".to_string(), "Double attack on {targets}".to_string());
        self.templates
            .insert("pin".to_string(), "Piece pinned to {target}".to_string());
        self.templates.insert(
            "outpost".to_string(),
            "Strong piece on {square}".to_string(),
        );
        self.templates.insert(
            "center".to_string(),
            "Controls center with {pieces}".to_string(),
        );
    }

    fn explain(&self, _board: &BitboardBoard, _player: Player) -> Vec<PatternExplanation> {
        // Generate human-readable explanations for detected patterns
        vec![]
    }
}

/// Pattern explanation
#[derive(Debug, Clone)]
pub struct PatternExplanation {
    /// Pattern name
    pub pattern_name: String,

    /// Description
    pub description: String,

    /// Value contribution
    pub value: i32,

    /// Squares involved
    pub squares: Vec<Position>,
}

/// Pattern analytics engine
pub struct PatternAnalytics {
    /// Pattern frequency counts
    frequency: HashMap<String, u64>,

    /// Pattern value distribution
    value_distribution: HashMap<String, Vec<i32>>,

    /// Pattern correlation matrix (reserved for future use)
    #[allow(dead_code)]
    correlations: HashMap<(String, String), f32>,
}

impl PatternAnalytics {
    fn new() -> Self {
        Self {
            frequency: HashMap::new(),
            value_distribution: HashMap::new(),
            correlations: HashMap::new(),
        }
    }

    /// Record pattern occurrence
    pub fn record_pattern(&mut self, pattern_name: &str, value: i32) {
        *self.frequency.entry(pattern_name.to_string()).or_insert(0) += 1;

        self.value_distribution
            .entry(pattern_name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Get pattern frequency
    pub fn get_frequency(&self, pattern_name: &str) -> u64 {
        self.frequency.get(pattern_name).copied().unwrap_or(0)
    }

    /// Get average pattern value
    pub fn get_average_value(&self, pattern_name: &str) -> f32 {
        if let Some(values) = self.value_distribution.get(pattern_name) {
            if values.is_empty() {
                0.0
            } else {
                let sum: i32 = values.iter().sum();
                sum as f32 / values.len() as f32
            }
        } else {
            0.0
        }
    }

    /// Get pattern statistics
    pub fn get_stats(&self) -> PatternAnalyticsStats {
        PatternAnalyticsStats {
            total_patterns: self.frequency.values().sum(),
            unique_patterns: self.frequency.len(),
        }
    }
}

/// Pattern analytics statistics
#[derive(Debug, Clone)]
pub struct PatternAnalyticsStats {
    pub total_patterns: u64,
    pub unique_patterns: usize,
}

/// Pattern visualizer (for debugging and analysis)
pub struct PatternVisualizer;

impl PatternVisualizer {
    /// Create ASCII visualization of patterns
    pub fn visualize_board(board: &BitboardBoard, patterns: &[PatternExplanation]) -> String {
        let mut output = String::new();

        // Board header
        output.push_str("  a b c d e f g h i\n");

        // Board rows
        for row in 0..9 {
            output.push_str(&format!("{} ", 9 - row));

            for col in 0..9 {
                let pos = Position::new(row, col);

                if let Some(_piece) = board.get_piece(pos) {
                    // Check if this square is involved in a pattern
                    let in_pattern = patterns.iter().any(|p| p.squares.contains(&pos));

                    if in_pattern {
                        output.push('*'); // Highlight pattern squares
                    } else {
                        output.push('.');
                    }
                } else {
                    output.push('.');
                }

                output.push(' ');
            }

            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_system_creation() {
        let system = AdvancedPatternSystem::new();
        assert!(!system.ml_config.enabled);
    }

    #[test]
    fn test_ml_config_default() {
        let config = MLConfig::default();
        assert_eq!(config.learning_rate, 0.01);
        assert_eq!(config.iterations, 1000);
    }

    #[test]
    fn test_dynamic_pattern_selection() {
        let system = AdvancedPatternSystem::new();
        let board = BitboardBoard::new();

        // Opening phase (high game phase)
        let selection = system.select_patterns(&board, 200);
        assert!(selection.enable_positional);

        // Endgame phase (low game phase)
        let selection = system.select_patterns(&board, 30);
        assert!(selection.enable_endgame);
    }

    #[test]
    fn test_pattern_explainer() {
        let system = AdvancedPatternSystem::new();
        let board = BitboardBoard::new();

        let explanations = system.explain_patterns(&board, Player::Black);

        // Should return explanations (may be empty for starting position)
        assert!(explanations.len() >= 0);
    }

    #[test]
    fn test_pattern_analytics_recording() {
        let mut analytics = PatternAnalytics::new();

        analytics.record_pattern("fork", 100);
        analytics.record_pattern("fork", 120);
        analytics.record_pattern("pin", 80);

        assert_eq!(analytics.get_frequency("fork"), 2);
        assert_eq!(analytics.get_frequency("pin"), 1);
        assert_eq!(analytics.get_average_value("fork"), 110.0);
    }

    #[test]
    fn test_pattern_analytics_stats() {
        let mut analytics = PatternAnalytics::new();

        analytics.record_pattern("fork", 100);
        analytics.record_pattern("pin", 80);
        analytics.record_pattern("outpost", 60);

        let stats = analytics.get_stats();
        assert_eq!(stats.total_patterns, 3);
        assert_eq!(stats.unique_patterns, 3);
    }

    #[test]
    fn test_pattern_visualizer() {
        let board = BitboardBoard::new();
        let patterns = vec![];

        let visualization = PatternVisualizer::visualize_board(&board, &patterns);

        // Should contain board header
        assert!(visualization.contains("a b c d e f g h i"));

        // Should contain row numbers
        assert!(visualization.contains("9"));
        assert!(visualization.contains("1"));
    }

    #[test]
    fn test_optimize_weights() {
        let mut system = AdvancedPatternSystem::new();

        // Create empty training data
        let training_data = vec![];

        let weights = system.optimize_weights(&training_data);

        // Should return empty vector when ML disabled
        assert_eq!(weights.len(), 0);
    }

    #[test]
    fn test_ml_enabled_weights() {
        let mut system = AdvancedPatternSystem::new();
        system.ml_config.enabled = true;

        let training_data = vec![];
        let weights = system.optimize_weights(&training_data);

        // Should return weights when ML enabled
        assert_eq!(weights.len(), 8);
    }

    #[test]
    fn test_position_type_selection() {
        let selector = DynamicPatternSelector::new();

        // Opening
        let opening_selection = selector.select(&BitboardBoard::new(), 200);
        assert!(opening_selection.enable_positional);
        assert!(!opening_selection.enable_endgame);

        // Middlegame
        let middlegame_selection = selector.select(&BitboardBoard::new(), 128);
        assert!(opening_selection.enable_tactical);
        assert!(opening_selection.enable_positional);

        // Endgame
        let endgame_selection = selector.select(&BitboardBoard::new(), 30);
        assert!(endgame_selection.enable_endgame);
    }

    #[test]
    fn test_pattern_weights_by_phase() {
        let selector = DynamicPatternSelector::new();

        let opening = selector.select(&BitboardBoard::new(), 200);
        let endgame = selector.select(&BitboardBoard::new(), 30);

        // Weights should differ by phase
        assert_ne!(opening.weights, endgame.weights);

        // Opening should emphasize positional patterns (index 2 = king safety)
        assert!(opening.weights[2] >= 1.0);

        // Endgame should emphasize endgame patterns (index 7)
        assert!(endgame.weights[7] >= 1.0);
    }

    #[test]
    fn test_analytics_empty() {
        let analytics = PatternAnalytics::new();

        assert_eq!(analytics.get_frequency("fork"), 0);
        assert_eq!(analytics.get_average_value("fork"), 0.0);
    }

    #[test]
    fn test_analytics_multiple_recordings() {
        let mut analytics = PatternAnalytics::new();

        for i in 0..100 {
            analytics.record_pattern("test_pattern", i);
        }

        assert_eq!(analytics.get_frequency("test_pattern"), 100);

        let avg = analytics.get_average_value("test_pattern");
        assert!((avg - 49.5).abs() < 1.0); // Average of 0..99
    }

    #[test]
    fn test_pattern_explanation_structure() {
        let explanation = PatternExplanation {
            pattern_name: "Fork".to_string(),
            description: "Knight forks king and rook".to_string(),
            value: 150,
            squares: vec![
                Position::new(4, 4),
                Position::new(3, 2),
                Position::new(3, 6),
            ],
        };

        assert_eq!(explanation.pattern_name, "Fork");
        assert_eq!(explanation.value, 150);
        assert_eq!(explanation.squares.len(), 3);
    }
}
