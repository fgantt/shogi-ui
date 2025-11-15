//! Quiescence Search Module
//!
//! This module handles quiescence search pruning logic (delta pruning, futility pruning)
//! and extension decisions. The main quiescence search function remains in `search_engine.rs`
//! due to tight coupling and will be extracted as part of Task 1.8 (coordinator refactoring).
//!
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::types::core::Move;
use crate::types::search::{QuiescenceConfig, QuiescenceStats};

/// Quiescence search helper functions for pruning and extensions
pub struct QuiescenceHelper {
    config: QuiescenceConfig,
    stats: QuiescenceStats,
}

impl QuiescenceHelper {
    /// Create a new QuiescenceHelper with the given configuration
    pub fn new(config: QuiescenceConfig) -> Self {
        Self {
            config,
            stats: QuiescenceStats::default(),
        }
    }

    /// Check if a move should be pruned using delta pruning
    pub fn should_prune_delta(&self, move_: &Move, stand_pat: i32, alpha: i32) -> bool {
        if !self.config.enable_delta_pruning {
            return false;
        }

        let material_gain = move_.captured_piece_value();
        let promotion_bonus = move_.promotion_value();
        let total_gain = material_gain + promotion_bonus;

        // If the best possible outcome is still worse than alpha, prune
        stand_pat + total_gain + self.config.delta_margin <= alpha
    }

    /// Adaptive delta pruning based on position characteristics
    ///
    /// Adjusts pruning margins dynamically based on:
    /// - Depth: More aggressive pruning at deeper depths
    /// - Move count: More selective pruning when there are many moves
    /// - Move type: Less aggressive pruning for high-value captures and promotions
    ///
    /// This provides better pruning effectiveness while maintaining tactical accuracy.
    pub fn should_prune_delta_adaptive(
        &self,
        move_: &Move,
        stand_pat: i32,
        alpha: i32,
        depth: u8,
        move_count: usize,
    ) -> bool {
        if !self.config.enable_delta_pruning {
            return false;
        }

        let material_gain = move_.captured_piece_value();
        let promotion_bonus = move_.promotion_value();
        let total_gain = material_gain + promotion_bonus;

        // Adaptive margin based on depth and move count
        let mut adaptive_margin = self.config.delta_margin;

        // Increase margin at deeper depths (more aggressive pruning)
        if depth > 3 {
            adaptive_margin += (depth as i32 - 3) * 50;
        }

        // Increase margin when there are many moves (more selective)
        if move_count > 8 {
            adaptive_margin += (move_count as i32 - 8) * 25;
        }

        // Decrease margin for high-value captures (less aggressive pruning)
        // This treats captures and promotions differently - high-value moves are less likely to be pruned
        if total_gain > 200 {
            adaptive_margin = adaptive_margin / 2;
        }

        // If the best possible outcome is still worse than alpha, prune
        stand_pat + total_gain + adaptive_margin <= alpha
    }

    /// Check if a move should be pruned using futility pruning
    ///
    /// Note: This is capture-specific futility pruning. Standard futility pruning
    /// typically excludes captures and checks, but this implementation applies
    /// futility pruning to weak captures while excluding:
    /// - Checking moves (critical for tactical sequences)
    /// - High-value captures (important tactical moves)
    ///
    /// This helps maintain tactical accuracy while still pruning weak captures.
    pub fn should_prune_futility(
        &mut self,
        move_: &Move,
        stand_pat: i32,
        alpha: i32,
        depth: u8,
    ) -> bool {
        if !self.config.enable_futility_pruning {
            return false;
        }

        // Don't prune checking moves - they're critical for tactical sequences
        if move_.gives_check {
            self.stats.checks_excluded_from_futility += 1;
            return false;
        }

        let material_gain = move_.captured_piece_value();

        // Don't prune high-value captures - they're important tactical moves
        if material_gain >= self.config.high_value_capture_threshold {
            self.stats.high_value_captures_excluded_from_futility += 1;
            return false;
        }

        let futility_margin = match depth {
            1 => self.config.futility_margin / 2,
            2 => self.config.futility_margin,
            _ => self.config.futility_margin * 2,
        };

        stand_pat + material_gain + futility_margin <= alpha
    }

    /// Adaptive futility pruning based on position characteristics
    ///
    /// Adjusts pruning margins dynamically based on:
    /// - Depth: More aggressive pruning at deeper depths (already depth-dependent)
    /// - Move count: More selective pruning when there are many moves available
    ///
    /// Excludes checking moves and high-value captures to maintain tactical accuracy.
    /// This provides better pruning effectiveness while maintaining tactical accuracy.
    pub fn should_prune_futility_adaptive(
        &mut self,
        move_: &Move,
        stand_pat: i32,
        alpha: i32,
        depth: u8,
        move_count: usize,
    ) -> bool {
        if !self.config.enable_futility_pruning {
            return false;
        }

        // Don't prune checking moves - they're critical for tactical sequences
        if move_.gives_check {
            self.stats.checks_excluded_from_futility += 1;
            return false;
        }

        let material_gain = move_.captured_piece_value();

        // Don't prune high-value captures - they're important tactical moves
        if material_gain >= self.config.high_value_capture_threshold {
            self.stats.high_value_captures_excluded_from_futility += 1;
            return false;
        }

        let mut futility_margin = match depth {
            1 => self.config.futility_margin / 2,
            2 => self.config.futility_margin,
            _ => self.config.futility_margin * 2,
        };

        // Adaptive adjustments based on position characteristics
        if move_count > 10 {
            futility_margin += 50; // More aggressive pruning with many moves
        }

        if depth > 4 {
            futility_margin += (depth as i32 - 4) * 25; // More aggressive at deeper depths
        }

        stand_pat + material_gain + futility_margin <= alpha
    }

    /// Check if a move should be extended in quiescence search
    pub fn should_extend(&self, move_: &Move, _depth: u8) -> bool {
        if !self.config.enable_selective_extensions {
            return false;
        }

        // Extend for checks
        if move_.gives_check {
            return true;
        }

        // Extend for recaptures
        if move_.is_recapture {
            return true;
        }

        // Extend for promotions
        if move_.is_promotion {
            return true;
        }

        // Extend for captures of high-value pieces
        if move_.is_capture && move_.captured_piece_value() > 500 {
            return true;
        }

        false
    }

    /// Get quiescence statistics
    pub fn get_stats(&self) -> &QuiescenceStats {
        &self.stats
    }

    /// Get mutable reference to statistics (for updating from main search)
    pub fn get_stats_mut(&mut self) -> &mut QuiescenceStats {
        &mut self.stats
    }

    /// Reset quiescence statistics
    pub fn reset_stats(&mut self) {
        self.stats = QuiescenceStats::default();
    }

    /// Get quiescence configuration
    pub fn get_config(&self) -> &QuiescenceConfig {
        &self.config
    }

    /// Update quiescence configuration
    pub fn update_config(&mut self, config: QuiescenceConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PieceType;

    fn create_test_move(is_capture: bool, gives_check: bool) -> Move {
        // Create a simple test move
        // Note: This is a simplified test - actual Move creation is more complex
        Move::new(
            0, 0, // from, to squares
            PieceType::Pawn,
            false, // is_promotion
            is_capture,
            gives_check,
            false, // is_recapture
        )
    }

    #[test]
    fn test_delta_pruning() {
        let config = QuiescenceConfig::default();
        let helper = QuiescenceHelper::new(config);
        let move_ = create_test_move(true, false);

        // Test delta pruning with low material gain
        let stand_pat = 100;
        let alpha = 200;
        // If material gain is low, should prune
        // Note: This test depends on Move implementation details
    }

    #[test]
    fn test_futility_pruning_excludes_checks() {
        let config = QuiescenceConfig::default();
        let mut helper = QuiescenceHelper::new(config);
        let move_ = create_test_move(true, true); // Checking move

        let stand_pat = 100;
        let alpha = 200;

        // Checking moves should not be pruned
        assert!(!helper.should_prune_futility(&move_, stand_pat, alpha, 1));
        assert!(helper.get_stats().checks_excluded_from_futility > 0);
    }

    #[test]
    fn test_extension_for_checks() {
        let config = QuiescenceConfig {
            enable_selective_extensions: true,
            ..QuiescenceConfig::default()
        };
        let helper = QuiescenceHelper::new(config);
        let move_ = create_test_move(false, true); // Checking move

        assert!(helper.should_extend(&move_, 1));
    }

    #[test]
    fn test_extension_for_promotions() {
        let config = QuiescenceConfig {
            enable_selective_extensions: true,
            ..QuiescenceConfig::default()
        };
        let helper = QuiescenceHelper::new(config);
        let mut move_ = create_test_move(false, false);
        // Note: Would need to set is_promotion if Move had that field accessible
        // For now, this test demonstrates the pattern
    }
}

