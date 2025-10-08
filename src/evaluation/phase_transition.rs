//! Phase Transition Smoothing Module
//!
//! This module provides advanced interpolation algorithms for smooth phase transitions
//! in tapered evaluation. Different interpolation methods can be used to control how
//! evaluation weights transition from opening/middlegame to endgame.
//!
//! # Overview
//!
//! The phase transition system:
//! - Multiple interpolation algorithms (linear, cubic, sigmoid)
//! - Smooth transitions without discontinuities
//! - Phase boundary handling
//! - Configurable transition curves
//! - Validation and quality metrics
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::phase_transition::{PhaseTransition, InterpolationMethod};
//! use crate::types::TaperedScore;
//!
//! let transition = PhaseTransition::new();
//! let score = TaperedScore::new_tapered(100, 200);
//! let phase = 128; // Mid-game
//!
//! // Linear interpolation (default)
//! let linear_result = transition.interpolate(score, phase, InterpolationMethod::Linear);
//!
//! // Smooth cubic interpolation
//! let cubic_result = transition.interpolate(score, phase, InterpolationMethod::Cubic);
//! ```

use crate::types::*;

use serde::{Deserialize, Serialize};

/// Interpolation methods for phase transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterpolationMethod {
    /// Linear interpolation (default, fastest)
    /// Formula: mg * (phase/256) + eg * (1 - phase/256)
    Linear,
    
    /// Cubic interpolation (smoother transitions)
    /// Formula: Uses cubic easing for smoother curves
    Cubic,
    
    /// Sigmoid interpolation (S-curve, gradual at extremes)
    /// Formula: Uses sigmoid function for natural transitions
    Sigmoid,
    
    /// Smoothstep interpolation (polynomial smoothing)
    /// Formula: 3t² - 2t³ where t = phase/256
    Smoothstep,
}

/// Phase transition coordinator
#[derive(Debug, Clone)]
pub struct PhaseTransition {
    /// Default interpolation method
    default_method: InterpolationMethod,
    /// Configuration for phase transitions
    config: PhaseTransitionConfig,
    /// Statistics tracking
    stats: PhaseTransitionStats,
}

impl PhaseTransition {
    /// Create a new PhaseTransition with default configuration
    pub fn new() -> Self {
        Self {
            default_method: InterpolationMethod::Linear,
            config: PhaseTransitionConfig::default(),
            stats: PhaseTransitionStats::default(),
        }
    }

    /// Create a PhaseTransition with custom configuration
    pub fn with_config(config: PhaseTransitionConfig) -> Self {
        Self {
            default_method: config.default_method,
            config,
            stats: PhaseTransitionStats::default(),
        }
    }

    /// Interpolate a tapered score using specified method
    ///
    /// # Arguments
    ///
    /// * `score` - The tapered score to interpolate
    /// * `phase` - The current game phase (0 = endgame, GAME_PHASE_MAX = opening)
    /// * `method` - The interpolation method to use
    ///
    /// # Returns
    ///
    /// Interpolated score value
    pub fn interpolate(&mut self, score: TaperedScore, phase: i32, method: InterpolationMethod) -> i32 {
        self.stats.interpolations += 1;
        
        // Clamp phase to valid range
        let clamped_phase = phase.max(0).min(GAME_PHASE_MAX);
        
        // Apply phase boundaries if configured
        let adjusted_phase = if self.config.use_phase_boundaries {
            self.apply_phase_boundaries(clamped_phase)
        } else {
            clamped_phase
        };
        
        // Perform interpolation based on method
        let result = match method {
            InterpolationMethod::Linear => self.interpolate_linear(score, adjusted_phase),
            InterpolationMethod::Cubic => self.interpolate_cubic(score, adjusted_phase),
            InterpolationMethod::Sigmoid => self.interpolate_sigmoid(score, adjusted_phase),
            InterpolationMethod::Smoothstep => self.interpolate_smoothstep(score, adjusted_phase),
        };
        
        result
    }

    /// Interpolate using default method
    pub fn interpolate_default(&mut self, score: TaperedScore, phase: i32) -> i32 {
        self.interpolate(score, phase, self.default_method)
    }

    /// Linear interpolation (fastest, standard)
    fn interpolate_linear(&self, score: TaperedScore, phase: i32) -> i32 {
        // Standard linear interpolation
        // result = mg * (phase/256) + eg * ((256-phase)/256)
        (score.mg * phase + score.eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX
    }

    /// Cubic interpolation (smoother transitions)
    fn interpolate_cubic(&self, score: TaperedScore, phase: i32) -> i32 {
        // Cubic easing: t³ where t = phase/256
        // This creates a smoother curve than linear
        let t = phase as f64 / GAME_PHASE_MAX as f64;
        let cubic_t = t * t * t;
        
        let mg_weight = cubic_t;
        let eg_weight = 1.0 - cubic_t;
        
        (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32
    }

    /// Sigmoid interpolation (S-curve, gradual at extremes)
    fn interpolate_sigmoid(&self, score: TaperedScore, phase: i32) -> i32 {
        // Sigmoid function: 1 / (1 + exp(-k*(t-0.5)))
        // where t = phase/256, k = steepness (default 6)
        let t = phase as f64 / GAME_PHASE_MAX as f64;
        let k = 6.0; // Steepness parameter
        
        // Sigmoid centered at 0.5
        let sigmoid_t = 1.0 / (1.0 + f64::exp(-k * (t - 0.5)));
        
        let mg_weight = sigmoid_t;
        let eg_weight = 1.0 - sigmoid_t;
        
        (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32
    }

    /// Smoothstep interpolation (polynomial smoothing)
    fn interpolate_smoothstep(&self, score: TaperedScore, phase: i32) -> i32 {
        // Smoothstep: 3t² - 2t³ where t = phase/256
        // Provides smooth acceleration and deceleration
        let t = phase as f64 / GAME_PHASE_MAX as f64;
        let smoothstep_t = t * t * (3.0 - 2.0 * t);
        
        let mg_weight = smoothstep_t;
        let eg_weight = 1.0 - smoothstep_t;
        
        (score.mg as f64 * mg_weight + score.eg as f64 * eg_weight) as i32
    }

    /// Apply phase boundaries for smoother transitions
    fn apply_phase_boundaries(&self, phase: i32) -> i32 {
        // Define phase boundaries
        let opening_threshold = (GAME_PHASE_MAX as f64 * 0.75) as i32; // 192
        let endgame_threshold = (GAME_PHASE_MAX as f64 * 0.25) as i32; // 64
        
        // Apply smoothing at boundaries if configured
        if phase > opening_threshold {
            // In opening, gradually transition
            let ratio = (phase - opening_threshold) as f64 / (GAME_PHASE_MAX - opening_threshold) as f64;
            opening_threshold + (ratio * (GAME_PHASE_MAX - opening_threshold) as f64) as i32
        } else if phase < endgame_threshold {
            // In endgame, gradually transition
            let ratio = phase as f64 / endgame_threshold as f64;
            (ratio * endgame_threshold as f64) as i32
        } else {
            phase
        }
    }

    /// Check if transition is smooth between two phases
    ///
    /// Returns true if the transition between phase1 and phase2 is smooth
    /// (no large discontinuities)
    pub fn is_transition_smooth(&mut self, score: TaperedScore, phase1: i32, phase2: i32, method: InterpolationMethod) -> bool {
        let value1 = self.interpolate(score, phase1, method);
        let value2 = self.interpolate(score, phase2, method);
        
        let diff = (value2 - value1).abs();
        let phase_diff = (phase2 - phase1).abs();
        
        // For adjacent phases, difference should be small
        if phase_diff == 1 {
            diff <= 2 // Allow at most 2 points change between adjacent phases
        } else {
            // For larger phase differences, scale accordingly
            diff <= (phase_diff * 2)
        }
    }

    /// Validate smooth transitions across all phases
    ///
    /// Returns true if transitions are smooth across the entire phase range
    pub fn validate_smooth_transitions(&mut self, score: TaperedScore, method: InterpolationMethod) -> bool {
        let mut prev_value = self.interpolate(score, 0, method);
        
        for phase in 1..=GAME_PHASE_MAX {
            let current_value = self.interpolate(score, phase, method);
            let diff = (current_value - prev_value).abs();
            
            // Difference should be small between adjacent phases
            if diff > 2 {
                return false;
            }
            
            prev_value = current_value;
        }
        
        true
    }

    /// Calculate maximum transition rate
    ///
    /// Returns the maximum change per phase unit across the entire range
    pub fn calculate_max_transition_rate(&mut self, score: TaperedScore, method: InterpolationMethod) -> i32 {
        let mut max_diff = 0;
        let mut prev_value = self.interpolate(score, 0, method);
        
        for phase in 1..=GAME_PHASE_MAX {
            let current_value = self.interpolate(score, phase, method);
            let diff = (current_value - prev_value).abs();
            max_diff = max_diff.max(diff);
            prev_value = current_value;
        }
        
        max_diff
    }

    /// Get configuration
    pub fn config(&self) -> &PhaseTransitionConfig {
        &self.config
    }

    /// Get statistics
    pub fn stats(&self) -> &PhaseTransitionStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PhaseTransitionStats::default();
    }
}

impl Default for PhaseTransition {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for phase transitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransitionConfig {
    /// Default interpolation method
    pub default_method: InterpolationMethod,
    /// Use phase boundaries for smoother transitions
    pub use_phase_boundaries: bool,
    /// Steepness for sigmoid interpolation
    pub sigmoid_steepness: f64,
}

impl Default for PhaseTransitionConfig {
    fn default() -> Self {
        Self {
            default_method: InterpolationMethod::Linear,
            use_phase_boundaries: false,
            sigmoid_steepness: 6.0,
        }
    }
}

/// Statistics for phase transitions
#[derive(Debug, Clone, Default)]
pub struct PhaseTransitionStats {
    /// Number of interpolations performed
    pub interpolations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transition_creation() {
        let transition = PhaseTransition::new();
        assert_eq!(transition.default_method, InterpolationMethod::Linear);
    }

    #[test]
    fn test_linear_interpolation() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test endpoints
        let result_0 = transition.interpolate(score, 0, InterpolationMethod::Linear);
        assert_eq!(result_0, 200); // Pure endgame
        
        let result_256 = transition.interpolate(score, 256, InterpolationMethod::Linear);
        assert_eq!(result_256, 100); // Pure opening
        
        // Test midpoint
        let result_128 = transition.interpolate(score, 128, InterpolationMethod::Linear);
        assert_eq!(result_128, 150); // Average
    }

    #[test]
    fn test_cubic_interpolation() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test endpoints
        let result_0 = transition.interpolate(score, 0, InterpolationMethod::Cubic);
        assert_eq!(result_0, 200);
        
        let result_256 = transition.interpolate(score, 256, InterpolationMethod::Cubic);
        assert_eq!(result_256, 100);
        
        // Cubic should differ from linear in the middle
        let result_128 = transition.interpolate(score, 128, InterpolationMethod::Cubic);
        assert!(result_128 >= 145 && result_128 <= 155); // Close to middle but not exactly 150
    }

    #[test]
    fn test_sigmoid_interpolation() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Sigmoid should have smooth S-curve
        let result_0 = transition.interpolate(score, 0, InterpolationMethod::Sigmoid);
        let result_256 = transition.interpolate(score, 256, InterpolationMethod::Sigmoid);
        
        // Should be close to endpoints
        assert!(result_0 >= 195 && result_0 <= 200);
        assert!(result_256 >= 100 && result_256 <= 105);
    }

    #[test]
    fn test_smoothstep_interpolation() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Smoothstep should have smooth acceleration/deceleration
        let result_0 = transition.interpolate(score, 0, InterpolationMethod::Smoothstep);
        let result_256 = transition.interpolate(score, 256, InterpolationMethod::Smoothstep);
        let result_128 = transition.interpolate(score, 128, InterpolationMethod::Smoothstep);
        
        assert_eq!(result_0, 200);
        assert_eq!(result_256, 100);
        assert_eq!(result_128, 150); // Should be at midpoint
    }

    #[test]
    fn test_interpolation_default() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        let result_default = transition.interpolate_default(score, 128);
        let result_linear = transition.interpolate(score, 128, InterpolationMethod::Linear);
        
        assert_eq!(result_default, result_linear);
    }

    #[test]
    fn test_phase_clamping() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test negative phase
        let result_neg = transition.interpolate(score, -10, InterpolationMethod::Linear);
        assert_eq!(result_neg, 200); // Should clamp to 0
        
        // Test too large phase
        let result_large = transition.interpolate(score, 300, InterpolationMethod::Linear);
        assert_eq!(result_large, 100); // Should clamp to 256
    }

    #[test]
    fn test_smooth_transition_validation() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Linear should always be smooth
        assert!(transition.validate_smooth_transitions(score, InterpolationMethod::Linear));
        
        // Cubic should also be smooth
        assert!(transition.validate_smooth_transitions(score, InterpolationMethod::Cubic));
        
        // Smoothstep should be smooth
        assert!(transition.validate_smooth_transitions(score, InterpolationMethod::Smoothstep));
    }

    #[test]
    fn test_adjacent_phase_smoothness() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test all adjacent phases
        for phase in 0..GAME_PHASE_MAX {
            assert!(
                transition.is_transition_smooth(score, phase, phase + 1, InterpolationMethod::Linear),
                "Transition not smooth between {} and {}", phase, phase + 1
            );
        }
    }

    #[test]
    fn test_max_transition_rate() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        let max_rate = transition.calculate_max_transition_rate(score, InterpolationMethod::Linear);
        
        // For linear interpolation with difference of 100 over 256 steps,
        // max rate should be around 1 per step
        assert!(max_rate <= 2, "Max transition rate too high: {}", max_rate);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        assert_eq!(transition.stats().interpolations, 0);
        
        transition.interpolate(score, 128, InterpolationMethod::Linear);
        assert_eq!(transition.stats().interpolations, 1);
        
        transition.interpolate(score, 64, InterpolationMethod::Cubic);
        assert_eq!(transition.stats().interpolations, 2);
    }

    #[test]
    fn test_reset_statistics() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        transition.interpolate(score, 128, InterpolationMethod::Linear);
        assert_eq!(transition.stats().interpolations, 1);
        
        transition.reset_stats();
        assert_eq!(transition.stats().interpolations, 0);
    }

    #[test]
    fn test_different_score_ranges() {
        let mut transition = PhaseTransition::new();
        
        // Test with small difference
        let small_score = TaperedScore::new_tapered(100, 110);
        let small_result = transition.interpolate(small_score, 128, InterpolationMethod::Linear);
        assert_eq!(small_result, 105);
        
        // Test with large difference
        let large_score = TaperedScore::new_tapered(0, 1000);
        let large_result = transition.interpolate(large_score, 128, InterpolationMethod::Linear);
        assert_eq!(large_result, 500);
        
        // Test with negative values
        let neg_score = TaperedScore::new_tapered(-100, 100);
        let neg_result = transition.interpolate(neg_score, 128, InterpolationMethod::Linear);
        assert_eq!(neg_result, 0);
    }

    #[test]
    fn test_interpolation_consistency() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        // Multiple calls should return same result
        let result1 = transition.interpolate(score, 128, InterpolationMethod::Linear);
        let result2 = transition.interpolate(score, 128, InterpolationMethod::Linear);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_all_methods_at_endpoints() {
        let mut transition = PhaseTransition::new();
        let score = TaperedScore::new_tapered(100, 200);
        
        let methods = [
            InterpolationMethod::Linear,
            InterpolationMethod::Cubic,
            InterpolationMethod::Sigmoid,
            InterpolationMethod::Smoothstep,
        ];
        
        for method in methods {
            let result_0 = transition.interpolate(score, 0, method);
            let result_256 = transition.interpolate(score, 256, method);
            
            // All methods should converge to endpoints
            assert!(result_0 >= 195 && result_0 <= 200, "Method {:?} failed at phase 0: {}", method, result_0);
            assert!(result_256 >= 100 && result_256 <= 105, "Method {:?} failed at phase 256: {}", method, result_256);
        }
    }

    #[test]
    fn test_config_with_custom_method() {
        let config = PhaseTransitionConfig {
            default_method: InterpolationMethod::Cubic,
            use_phase_boundaries: true,
            sigmoid_steepness: 8.0,
        };
        
        let mut transition = PhaseTransition::with_config(config);
        assert_eq!(transition.default_method, InterpolationMethod::Cubic);
        assert!(transition.config().use_phase_boundaries);
    }

    #[test]
    fn test_extreme_score_values() {
        let mut transition = PhaseTransition::new();
        
        // Test with extreme positive values
        let extreme_pos = TaperedScore::new_tapered(10000, 20000);
        let result_pos = transition.interpolate(extreme_pos, 128, InterpolationMethod::Linear);
        assert_eq!(result_pos, 15000);
        
        // Test with extreme negative values
        let extreme_neg = TaperedScore::new_tapered(-10000, -20000);
        let result_neg = transition.interpolate(extreme_neg, 128, InterpolationMethod::Linear);
        assert_eq!(result_neg, -15000);
    }
}

