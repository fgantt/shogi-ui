//! Reductions Module (LMR and IID)
//!
//! This module handles Late Move Reductions (LMR) and Internal Iterative Deepening (IID)
//! helper functions for depth calculations, effectiveness checks, and adaptive parameter
//! recommendations. The main search functions remain in `search_engine.rs` due to tight
//! coupling and will be extracted as part of Task 1.8 (coordinator refactoring).
//!
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::bitboards::BitboardBoard;
use crate::types::board::CapturedPieces;
use crate::types::search::{IIDConfig, IIDDepthStrategy, IIDStats, LMRStats, PositionComplexity};

/// Reductions helper for IID and LMR calculations
pub struct ReductionsHelper {
    iid_config: IIDConfig,
    iid_stats: IIDStats,
}

impl ReductionsHelper {
    /// Create a new ReductionsHelper with the given IID configuration
    pub fn new(iid_config: IIDConfig) -> Self {
        Self {
            iid_config,
            iid_stats: IIDStats::default(),
        }
    }

    /// Calculate IID depth based on strategy and position characteristics
    pub fn calculate_iid_depth(
        &mut self,
        main_depth: u8,
        board: Option<&BitboardBoard>,
        captured_pieces: Option<&CapturedPieces>,
        assess_complexity: Option<fn(&BitboardBoard, &CapturedPieces) -> PositionComplexity>,
    ) -> u8 {
        let depth = match self.iid_config.depth_strategy {
            IIDDepthStrategy::Fixed => self.iid_config.iid_depth_ply,
            IIDDepthStrategy::Relative => {
                // Add maximum depth cap to Relative strategy
                let relative_depth = std::cmp::max(2, main_depth.saturating_sub(2));
                relative_depth.min(4) // Cap at 4 for performance
            }
            IIDDepthStrategy::Adaptive => {
                // Enhanced Adaptive strategy with position-based adjustments
                let base_depth = if main_depth > 6 { 3 } else { 2 };
                // Enhanced: If we have position info, use it for better depth selection
                if let (Some(board), Some(captured), Some(assess_fn)) =
                    (board, captured_pieces, assess_complexity)
                {
                    let complexity = assess_fn(board, captured);
                    match complexity {
                        PositionComplexity::High => (base_depth as u8).saturating_add(1).min(4),
                        PositionComplexity::Low => (base_depth as u8).saturating_sub(1).max(1),
                        _ => base_depth,
                    }
                } else {
                    base_depth
                }
            }
            IIDDepthStrategy::Dynamic => {
                // Use calculate_dynamic_iid_depth() for Dynamic strategy
                if let (Some(board), Some(captured), Some(assess_fn)) =
                    (board, captured_pieces, assess_complexity)
                {
                    let base_depth = self.iid_config.dynamic_base_depth;
                    let complexity = assess_fn(board, captured);
                    let calculated_depth =
                        self.calculate_dynamic_iid_depth(board, captured, base_depth, assess_fn);

                    // Track statistics for dynamic depth selection
                    *self
                        .iid_stats
                        .dynamic_depth_selections
                        .entry(calculated_depth)
                        .or_insert(0) += 1;
                    match complexity {
                        PositionComplexity::Low => {
                            self.iid_stats.dynamic_depth_low_complexity += 1
                        }
                        PositionComplexity::Medium => {
                            self.iid_stats.dynamic_depth_medium_complexity += 1
                        }
                        PositionComplexity::High => {
                            self.iid_stats.dynamic_depth_high_complexity += 1
                        }
                        PositionComplexity::Unknown => {}
                    }

                    calculated_depth
                } else {
                    // Fallback to base depth if position info not available
                    self.iid_config.dynamic_base_depth
                }
            }
        };

        depth
    }

    /// Calculate dynamic IID depth based on position complexity
    fn calculate_dynamic_iid_depth(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        base_depth: u8,
        assess_complexity: fn(&BitboardBoard, &CapturedPieces) -> PositionComplexity,
    ) -> u8 {
        // Always assess position complexity for Dynamic strategy
        let complexity = assess_complexity(board, captured_pieces);

        // Use configurable depth adjustments if enabled
        let depth = if self.iid_config.enable_complexity_based_adjustments {
            let adjustment = match complexity {
                PositionComplexity::Low => self.iid_config.complexity_depth_adjustment_low,
                PositionComplexity::Medium => self.iid_config.complexity_depth_adjustment_medium,
                PositionComplexity::High => self.iid_config.complexity_depth_adjustment_high,
                PositionComplexity::Unknown => 0,
            };
            ((base_depth as i32) + (adjustment as i32)).max(1) as u8
        } else {
            // Fallback to original logic if complexity-based adjustments disabled
            match complexity {
                PositionComplexity::Low => {
                    // Simple positions: reduce IID depth to save time
                    base_depth.saturating_sub(1).max(1)
                }
                PositionComplexity::Medium => {
                    // Medium positions: use base depth
                    base_depth
                }
                PositionComplexity::High => {
                    // Complex positions: increase IID depth for better move ordering
                    base_depth.saturating_add(1)
                }
                PositionComplexity::Unknown => {
                    // Unknown complexity: use base depth as fallback
                    base_depth
                }
            }
        };

        // Apply maximum depth cap from configuration
        depth.min(self.iid_config.dynamic_max_depth)
    }

    /// Get the position complexity for adaptive LMR
    ///
    /// This analyzes LMR statistics to determine position complexity
    pub fn get_position_complexity_from_lmr_stats(&self, lmr_stats: &LMRStats) -> PositionComplexity {
        if lmr_stats.moves_considered == 0 {
            return PositionComplexity::Unknown;
        }

        // Calculate cutoff rate from cutoffs after reduction and research
        let total_cutoffs = lmr_stats.cutoffs_after_reduction + lmr_stats.cutoffs_after_research;
        let cutoff_rate = if lmr_stats.moves_considered > 0 {
            total_cutoffs as f64 / lmr_stats.moves_considered as f64
        } else {
            0.0
        };
        let research_rate = lmr_stats.research_rate() / 100.0;

        if cutoff_rate > 0.4 || research_rate > 0.3 {
            PositionComplexity::High
        } else if cutoff_rate > 0.2 || research_rate > 0.15 {
            PositionComplexity::Medium
        } else {
            PositionComplexity::Low
        }
    }

    /// Check if LMR is effective in current position
    pub fn is_lmr_effective(&self, lmr_stats: &LMRStats) -> bool {
        if lmr_stats.moves_considered < 10 {
            return true; // Not enough data, assume effective
        }

        let efficiency = lmr_stats.efficiency();
        let research_rate = lmr_stats.research_rate() / 100.0;

        // LMR is effective if we're reducing many moves but not re-searching too many
        efficiency > 20.0 && research_rate < 0.4
    }

    /// Get recommended LMR parameters based on position
    pub fn get_adaptive_lmr_params(&self, lmr_stats: &LMRStats) -> (u8, u8) {
        let complexity = self.get_position_complexity_from_lmr_stats(lmr_stats);
        let is_effective = self.is_lmr_effective(lmr_stats);

        let base_reduction = match complexity {
            PositionComplexity::High => {
                if is_effective {
                    2
                } else {
                    1
                }
            }
            PositionComplexity::Medium => 1,
            PositionComplexity::Low => 2,
            PositionComplexity::Unknown => 1,
        };

        let max_reduction = match complexity {
            PositionComplexity::High => 4,
            PositionComplexity::Medium => 3,
            PositionComplexity::Low => 5,
            PositionComplexity::Unknown => 3,
        };

        (base_reduction, max_reduction)
    }

    /// Get IID statistics
    pub fn get_iid_stats(&self) -> &IIDStats {
        &self.iid_stats
    }

    /// Get mutable reference to IID statistics (for updating from main search)
    pub fn get_iid_stats_mut(&mut self) -> &mut IIDStats {
        &mut self.iid_stats
    }

    /// Reset IID statistics
    pub fn reset_iid_stats(&mut self) {
        self.iid_stats = IIDStats::default();
    }

    /// Get IID configuration
    pub fn get_iid_config(&self) -> &IIDConfig {
        &self.iid_config
    }

    /// Update IID configuration
    pub fn update_iid_config(&mut self, config: IIDConfig) {
        self.iid_config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IIDDepthStrategy;

    fn dummy_complexity_assessment(
        _board: &BitboardBoard,
        _captured: &CapturedPieces,
    ) -> PositionComplexity {
        PositionComplexity::Medium
    }

    #[test]
    fn test_reductions_helper_creation() {
        let config = IIDConfig::default();
        let helper = ReductionsHelper::new(config);
        assert_eq!(helper.get_iid_stats().total_iid_searches, 0);
    }

    #[test]
    fn test_fixed_depth_strategy() {
        let config = IIDConfig {
            depth_strategy: IIDDepthStrategy::Fixed,
            iid_depth_ply: 3,
            ..IIDConfig::default()
        };
        let mut helper = ReductionsHelper::new(config);
        let depth = helper.calculate_iid_depth(5, None, None, None);
        assert_eq!(depth, 3);
    }

    #[test]
    fn test_relative_depth_strategy() {
        let config = IIDConfig {
            depth_strategy: IIDDepthStrategy::Relative,
            ..IIDConfig::default()
        };
        let mut helper = ReductionsHelper::new(config);
        let depth = helper.calculate_iid_depth(5, None, None, None);
        // Relative: max(2, 5-2) = 3, capped at 4
        assert_eq!(depth, 3);
    }

    #[test]
    fn test_lmr_effectiveness() {
        let config = IIDConfig::default();
        let helper = ReductionsHelper::new(config);
        let mut stats = LMRStats::default();
        stats.moves_considered = 100;
        stats.reductions_applied = 50;
        stats.researches_triggered = 10;

        // Should be effective with good efficiency and low research rate
        assert!(helper.is_lmr_effective(&stats));
    }

    #[test]
    fn test_config_update() {
        let config = IIDConfig::default();
        let mut helper = ReductionsHelper::new(config);
        let new_config = IIDConfig {
            iid_depth_ply: 4,
            ..IIDConfig::default()
        };
        helper.update_iid_config(new_config);
        assert_eq!(helper.get_iid_config().iid_depth_ply, 4);
    }
}

