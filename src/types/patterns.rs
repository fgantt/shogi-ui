//! Pattern Recognition Types
//!
//! This module contains types related to pattern recognition that are used across
//! the codebase. Note that the main pattern recognition configuration types
//! (PatternConfig, PatternWeights, etc.) are in the `evaluation` module.
//!
//! Extracted from `types.rs` (now `all.rs`) as part of Task 1.0: File Modularization and Structure Improvements.

use serde::{Deserialize, Serialize};

use super::core::PieceType;

// ============================================================================
// Tactical Pattern Types
// ============================================================================

/// Tactical indicators for move evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TacticalIndicators {
    /// Whether the move is a capture
    pub is_capture: bool,
    /// Whether the move is a promotion
    pub is_promotion: bool,
    /// Whether the move gives check
    pub gives_check: bool,
    /// Whether the move is a recapture
    pub is_recapture: bool,
    /// Piece value involved in the move
    pub piece_value: i32,
    /// Estimated mobility impact
    pub mobility_impact: i32,
    /// Estimated king safety impact
    pub king_safety_impact: i32,
}

impl Default for TacticalIndicators {
    fn default() -> Self {
        Self {
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            piece_value: 0,
            mobility_impact: 0,
            king_safety_impact: 0,
        }
    }
}

impl TacticalIndicators {
    /// Check if the move has any tactical characteristics
    pub fn has_tactical_characteristics(&self) -> bool {
        self.is_capture
            || self.is_promotion
            || self.gives_check
            || self.is_recapture
    }

    /// Get a tactical score based on indicators
    pub fn tactical_score(&self) -> i32 {
        let mut score = 0;
        if self.is_capture {
            score += self.piece_value;
        }
        if self.is_promotion {
            score += 100; // Base promotion bonus
        }
        if self.gives_check {
            score += 50; // Check bonus
        }
        if self.is_recapture {
            score += 30; // Recapture bonus
        }
        score += self.mobility_impact / 10;
        score += self.king_safety_impact / 10;
        score
    }
}

// ============================================================================
// Attack Pattern Types
// ============================================================================

/// Configuration for attack pattern detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttackConfig {
    pub piece_type: PieceType,
    pub square: u8,
    pub include_promoted: bool,
    pub max_distance: Option<u8>,
}

impl Default for AttackConfig {
    fn default() -> Self {
        Self {
            piece_type: PieceType::Pawn,
            square: 0,
            include_promoted: false,
            max_distance: None,
        }
    }
}

impl AttackConfig {
    /// Create a new attack configuration
    pub fn new(piece_type: PieceType, square: u8) -> Self {
        Self {
            piece_type,
            square,
            include_promoted: false,
            max_distance: None,
        }
    }

    /// Create an attack configuration with promoted pieces included
    pub fn with_promoted(piece_type: PieceType, square: u8) -> Self {
        Self {
            piece_type,
            square,
            include_promoted: true,
            max_distance: None,
        }
    }

    /// Create an attack configuration with distance limit
    pub fn with_max_distance(piece_type: PieceType, square: u8, max_distance: u8) -> Self {
        Self {
            piece_type,
            square,
            include_promoted: false,
            max_distance: Some(max_distance),
        }
    }
}

// ============================================================================
// Pattern Recognition Statistics
// ============================================================================

/// Statistics for pattern recognition performance
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternRecognitionStats {
    /// Total number of patterns evaluated
    pub patterns_evaluated: u64,
    /// Number of patterns matched
    pub patterns_matched: u64,
    /// Number of tactical patterns detected
    pub tactical_patterns_detected: u64,
    /// Number of positional patterns detected
    pub positional_patterns_detected: u64,
    /// Average evaluation time per pattern in nanoseconds
    pub average_evaluation_time_ns: u64,
    /// Cache hit rate for pattern matching
    pub cache_hit_rate: f64,
}

impl PatternRecognitionStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = PatternRecognitionStats::default();
    }

    /// Get the pattern match rate as a percentage
    pub fn match_rate(&self) -> f64 {
        if self.patterns_evaluated == 0 {
            return 0.0;
        }
        (self.patterns_matched as f64 / self.patterns_evaluated as f64) * 100.0
    }

    /// Get the tactical pattern detection rate as a percentage
    pub fn tactical_detection_rate(&self) -> f64 {
        if self.patterns_evaluated == 0 {
            return 0.0;
        }
        (self.tactical_patterns_detected as f64 / self.patterns_evaluated as f64) * 100.0
    }

    /// Get the positional pattern detection rate as a percentage
    pub fn positional_detection_rate(&self) -> f64 {
        if self.patterns_evaluated == 0 {
            return 0.0;
        }
        (self.positional_patterns_detected as f64 / self.patterns_evaluated as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tactical_indicators() {
        let indicators = TacticalIndicators {
            is_capture: true,
            is_promotion: false,
            gives_check: true,
            is_recapture: false,
            piece_value: 100,
            mobility_impact: 50,
            king_safety_impact: 30,
        };

        assert!(indicators.has_tactical_characteristics());
        assert!(indicators.tactical_score() > 0);
    }

    #[test]
    fn test_attack_config() {
        let config = AttackConfig::new(PieceType::Rook, 40);
        assert_eq!(config.piece_type, PieceType::Rook);
        assert_eq!(config.square, 40);
        assert!(!config.include_promoted);

        let config_with_promoted = AttackConfig::with_promoted(PieceType::Bishop, 50);
        assert!(config_with_promoted.include_promoted);

        let config_with_distance = AttackConfig::with_max_distance(PieceType::Knight, 30, 3);
        assert_eq!(config_with_distance.max_distance, Some(3));
    }

    #[test]
    fn test_pattern_recognition_stats() {
        let mut stats = PatternRecognitionStats::default();
        stats.patterns_evaluated = 100;
        stats.patterns_matched = 25;
        stats.tactical_patterns_detected = 10;
        stats.positional_patterns_detected = 15;

        assert_eq!(stats.match_rate(), 25.0);
        assert_eq!(stats.tactical_detection_rate(), 10.0);
        assert_eq!(stats.positional_detection_rate(), 15.0);
    }
}

