//! Iterative Deepening Module
//!
//! This module handles iterative deepening and aspiration window helper functions for window
//! size calculation, validation, and fail-high/fail-low handling. The main iterative deepening
//! search loop remains in `search_engine.rs` due to tight coupling and will be extracted as
//! part of Task 1.8 (coordinator refactoring).
//!
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::types::search::{AspirationWindowConfig, AspirationWindowStats};

/// Constants for score bounds
const MIN_SCORE: i32 = i32::MIN + 1;
const MAX_SCORE: i32 = i32::MAX - 1;

/// Iterative deepening helper for aspiration window calculations
pub struct IterativeDeepeningHelper {
    config: AspirationWindowConfig,
    stats: AspirationWindowStats,
}

impl IterativeDeepeningHelper {
    /// Create a new IterativeDeepeningHelper with the given configuration
    pub fn new(config: AspirationWindowConfig) -> Self {
        Self {
            config,
            stats: AspirationWindowStats::default(),
        }
    }

    /// Calculate static window size
    pub fn calculate_static_window_size(&self, depth: u8) -> i32 {
        if depth < self.config.min_depth {
            return i32::MAX; // Use full-width window
        }
        self.config.base_window_size
    }

    /// Calculate dynamic window size based on depth and score
    pub fn calculate_dynamic_window_size(&self, depth: u8, previous_score: i32) -> i32 {
        let base_size = self.config.base_window_size;

        if !self.config.dynamic_scaling {
            return base_size;
        }

        // Scale based on depth
        let depth_factor = 1.0 + (depth as f64 - 1.0) * 0.1;

        // Scale based on score magnitude (more volatile scores = larger window)
        let score_factor = 1.0 + (previous_score.abs() as f64 / 1000.0) * 0.2;

        // Clamp to i32 range before casting to prevent overflow
        let dynamic_size_f64 = base_size as f64 * depth_factor * score_factor;
        let dynamic_size = dynamic_size_f64.min(i32::MAX as f64).max(i32::MIN as f64) as i32;

        // Apply limits
        dynamic_size.min(self.config.max_window_size)
    }

    /// Calculate adaptive window size based on recent failures
    pub fn calculate_adaptive_window_size(&self, depth: u8, recent_failures: u8) -> i32 {
        let base_size = self.calculate_dynamic_window_size(depth, 0);

        if !self.config.enable_adaptive_sizing {
            return base_size;
        }

        // Increase window size if recent failures
        let failure_factor = 1.0 + (recent_failures as f64 * 0.3);
        // Clamp to i32 range before casting to prevent overflow
        let adaptive_size_f64 = base_size as f64 * failure_factor;
        let adaptive_size = adaptive_size_f64.min(i32::MAX as f64).max(i32::MIN as f64) as i32;

        adaptive_size.min(self.config.max_window_size)
    }

    /// Calculate final window size combining all strategies
    pub fn calculate_window_size(
        &self,
        depth: u8,
        _previous_score: i32,
        recent_failures: u8,
    ) -> i32 {
        if !self.config.enabled {
            return i32::MAX; // Use full-width window
        }

        if depth < self.config.min_depth {
            return i32::MAX; // Use full-width window
        }

        let window_size = self.calculate_adaptive_window_size(depth, recent_failures);
        self.validate_window_size(window_size)
    }

    /// Validate window size to ensure reasonable bounds
    pub fn validate_window_size(&self, window_size: i32) -> i32 {
        // Ensure minimum window size for stability
        let min_size = 10;
        let max_size = self.config.max_window_size;

        let validated_size = window_size.max(min_size).min(max_size);

        // Log extreme values for debugging
        if validated_size != window_size {
            crate::utils::telemetry::debug_log(&format!(
                "Aspiration: Window size clamped from {} to {}",
                window_size, validated_size
            ));
        }

        validated_size
    }

    /// Calculate optimal window size based on historical performance
    pub fn calculate_optimal_window_size(&self, depth: u8, recent_performance: f64) -> i32 {
        let base_size = self.calculate_static_window_size(depth);

        if base_size == i32::MAX {
            return base_size; // Full-width window
        }

        // Adjust based on recent performance
        // Better performance = smaller windows for efficiency
        // Worse performance = larger windows for thoroughness
        let performance_factor = if recent_performance > 0.9 {
            0.7 // High performance: smaller windows
        } else if recent_performance > 0.7 {
            0.85 // Good performance: slightly smaller windows
        } else if recent_performance > 0.5 {
            1.0 // Average performance: standard windows
        } else if recent_performance > 0.3 {
            1.2 // Poor performance: larger windows
        } else {
            1.5 // Very poor performance: much larger windows
        };

        let optimal_size = (base_size as f64 * performance_factor) as i32;
        self.validate_window_size(optimal_size)
    }

    /// Handle fail-low by calculating new window bounds
    ///
    /// Returns (new_alpha, new_beta) for fail-low scenario
    pub fn calculate_fail_low_window(
        &mut self,
        previous_score: i32,
        window_size: i32,
    ) -> (i32, i32) {
        // Adaptive window widening based on failure pattern
        let adaptive_factor = 2; // Widen by 2x on fail-low
        let widened_window = window_size * adaptive_factor;

        // Widen window downward with adaptive sizing
        let new_alpha = MIN_SCORE;
        let new_beta = previous_score + widened_window;

        // Ensure valid window bounds
        if new_beta <= new_alpha {
            // Fallback to conservative approach
            (MIN_SCORE, previous_score + window_size)
        } else {
            (new_alpha, new_beta)
        }
    }

    /// Handle fail-high by calculating new window bounds
    ///
    /// Returns (new_alpha, new_beta) for fail-high scenario
    pub fn calculate_fail_high_window(
        &mut self,
        previous_score: i32,
        window_size: i32,
    ) -> (i32, i32) {
        // Adaptive window widening based on failure pattern
        let adaptive_factor = 2; // Widen by 2x on fail-high
        let widened_window = window_size * adaptive_factor;

        // Widen window upward with adaptive sizing
        let new_alpha = previous_score - widened_window;
        let new_beta = MAX_SCORE;

        // Ensure valid window bounds
        if new_alpha >= new_beta {
            // Fallback to conservative approach
            (previous_score - window_size, MAX_SCORE)
        } else {
            (new_alpha, new_beta)
        }
    }

    /// Check if window should be widened based on failure count
    pub fn should_widen_window(&self, failure_count: u8) -> bool {
        failure_count > 0 && failure_count <= self.config.max_researches
    }

    /// Get aspiration window statistics
    pub fn get_stats(&self) -> &AspirationWindowStats {
        &self.stats
    }

    /// Get mutable reference to statistics (for updating from main search)
    pub fn get_stats_mut(&mut self) -> &mut AspirationWindowStats {
        &mut self.stats
    }

    /// Reset aspiration window statistics
    pub fn reset_stats(&mut self) {
        self.stats = AspirationWindowStats::default();
    }

    /// Get aspiration window configuration
    pub fn get_config(&self) -> &AspirationWindowConfig {
        &self.config
    }

    /// Update aspiration window configuration
    pub fn update_config(&mut self, config: AspirationWindowConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterative_deepening_helper_creation() {
        let config = AspirationWindowConfig::default();
        let helper = IterativeDeepeningHelper::new(config);
        assert_eq!(helper.get_stats().total_searches, 0);
    }

    #[test]
    fn test_static_window_size() {
        let config = AspirationWindowConfig {
            min_depth: 2,
            base_window_size: 50,
            ..AspirationWindowConfig::default()
        };
        let helper = IterativeDeepeningHelper::new(config);
        
        // Below min_depth should return full-width
        assert_eq!(helper.calculate_static_window_size(1), i32::MAX);
        // At or above min_depth should return base size
        assert_eq!(helper.calculate_static_window_size(2), 50);
    }

    #[test]
    fn test_window_size_validation() {
        let config = AspirationWindowConfig {
            max_window_size: 200,
            ..AspirationWindowConfig::default()
        };
        let helper = IterativeDeepeningHelper::new(config);
        
        // Too small should be clamped to minimum
        let validated = helper.validate_window_size(5);
        assert!(validated >= 10);
        
        // Too large should be clamped to maximum
        let validated = helper.validate_window_size(500);
        assert!(validated <= 200);
    }

    #[test]
    fn test_fail_low_window() {
        let config = AspirationWindowConfig::default();
        let mut helper = IterativeDeepeningHelper::new(config);
        
        let (alpha, beta) = helper.calculate_fail_low_window(1000, 50);
        assert_eq!(alpha, MIN_SCORE);
        assert!(beta > 1000); // Window widened downward
    }

    #[test]
    fn test_fail_high_window() {
        let config = AspirationWindowConfig::default();
        let mut helper = IterativeDeepeningHelper::new(config);
        
        let (alpha, beta) = helper.calculate_fail_high_window(1000, 50);
        assert_eq!(beta, MAX_SCORE);
        assert!(alpha < 1000); // Window widened upward
    }

    #[test]
    fn test_config_update() {
        let config = AspirationWindowConfig::default();
        let mut helper = IterativeDeepeningHelper::new(config);
        let new_config = AspirationWindowConfig {
            base_window_size: 100,
            ..AspirationWindowConfig::default()
        };
        helper.update_config(new_config);
        assert_eq!(helper.get_config().base_window_size, 100);
    }
}

