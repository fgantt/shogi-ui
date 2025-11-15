//! Time Management Module
//!
//! This module handles time allocation, time limits, timeout handling, and time pressure
//! calculations for the search engine. Extracted from `search_engine.rs` as part of
//! Task 1.0: File Modularization and Structure Improvements.

use crate::time_utils::TimeSource;
use crate::types::search::{TimeAllocationStrategy, TimeBudgetStats, TimeManagementConfig};
// Note: TimePressure and TimePressureThresholds are not yet extracted to a sub-module
// They remain in types/all.rs for now, so we use the root import
use crate::types::{TimePressure, TimePressureThresholds};

/// Time management functionality for search engine
pub struct TimeManager {
    config: TimeManagementConfig,
    time_budget_stats: TimeBudgetStats,
    time_pressure_thresholds: TimePressureThresholds,
    time_check_node_counter: u32,
}

impl TimeManager {
    /// Create a new TimeManager with the given configuration
    pub fn new(
        config: TimeManagementConfig,
        time_pressure_thresholds: TimePressureThresholds,
    ) -> Self {
        Self {
            config,
            time_budget_stats: TimeBudgetStats::default(),
            time_pressure_thresholds,
            time_check_node_counter: 0,
        }
    }

    /// Calculate time pressure level based on remaining time
    pub fn calculate_time_pressure_level(
        &self,
        start_time: &TimeSource,
        time_limit_ms: u32,
    ) -> TimePressure {
        if time_limit_ms == 0 {
            return TimePressure::None;
        }

        let elapsed_ms = start_time.elapsed_ms();
        let remaining_ms = if elapsed_ms >= time_limit_ms {
            0
        } else {
            time_limit_ms - elapsed_ms
        };

        let remaining_percent = (remaining_ms as f64 / time_limit_ms as f64) * 100.0;

        TimePressure::from_remaining_time_percent(
            remaining_percent,
            &self.time_pressure_thresholds,
        )
    }

    /// Check if search should stop due to time limit or stop flag
    /// Uses frequency optimization to avoid checking time on every node
    pub fn should_stop(
        &mut self,
        start_time: &TimeSource,
        time_limit_ms: u32,
        stop_flag: Option<&std::sync::atomic::AtomicBool>,
    ) -> bool {
        // Always check stop flag immediately (user-initiated stop)
        if let Some(flag) = stop_flag {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                return true;
            }
        }

        // Optimize time check frequency
        let frequency = self.config.time_check_frequency;
        self.time_check_node_counter = self.time_check_node_counter.wrapping_add(1);

        // Only check time every N nodes
        if self.time_check_node_counter >= frequency {
            self.time_check_node_counter = 0;
            start_time.has_exceeded_limit(time_limit_ms)
        } else {
            false // Don't check time yet
        }
    }

    /// Force time check (bypasses frequency optimization)
    /// Used when we must check time regardless of frequency (e.g., at depth boundaries)
    pub fn should_stop_force(
        &self,
        start_time: &TimeSource,
        time_limit_ms: u32,
        stop_flag: Option<&std::sync::atomic::AtomicBool>,
    ) -> bool {
        if let Some(flag) = stop_flag {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                return true;
            }
        }
        start_time.has_exceeded_limit(time_limit_ms)
    }

    /// Calculate time budget for a specific depth
    pub fn calculate_time_budget(
        &mut self,
        depth: u8,
        total_time_ms: u32,
        elapsed_ms: u32,
        max_depth: u8,
    ) -> u32 {
        let config = &self.config;

        if !config.enable_time_budget {
            // If time budget is disabled, use remaining time
            return total_time_ms.saturating_sub(elapsed_ms);
        }

        let remaining_time = total_time_ms.saturating_sub(elapsed_ms);
        let safety_margin_ms = (remaining_time as f64 * config.safety_margin) as u32;
        let available_time = remaining_time.saturating_sub(safety_margin_ms);

        if depth == 1 {
            // First depth: use minimum time
            let budget = config
                .min_time_per_depth_ms
                .max(available_time / (max_depth as u32 * 2));
            return budget.min(available_time);
        }

        match config.allocation_strategy {
            TimeAllocationStrategy::Equal => {
                // Equal allocation: divide remaining time equally among remaining depths
                let remaining_depths = (max_depth + 1).saturating_sub(depth);
                if remaining_depths == 0 {
                    return available_time;
                }
                available_time / remaining_depths as u32
            }
            TimeAllocationStrategy::Exponential => {
                // Exponential allocation: later depths get more time
                self.calculate_exponential_budget(depth, available_time, max_depth)
            }
            TimeAllocationStrategy::Adaptive => {
                // Adaptive allocation: use historical data if available
                self.calculate_adaptive_time_budget(depth, available_time, max_depth)
            }
        }
    }

    /// Calculate adaptive time budget based on depth completion history
    fn calculate_adaptive_time_budget(&self, depth: u8, available_time: u32, max_depth: u8) -> u32 {
        let config = &self.config;
        let stats = &self.time_budget_stats;

        // If we have historical data, use exponential weighting based on past completion times
        if !stats.depth_completion_times_ms.is_empty()
            && depth <= stats.depth_completion_times_ms.len() as u8
        {
            let depth_idx = (depth - 1) as usize;
            if depth_idx < stats.depth_completion_times_ms.len() {
                let avg_completion_time = stats.depth_completion_times_ms[depth_idx];
                // Estimate based on average, with a factor for remaining depths
                let remaining_depths = (max_depth + 1).saturating_sub(depth);
                let estimated_total = avg_completion_time * remaining_depths as u32;

                // Use the average time for this depth, but cap at available time
                let budget = avg_completion_time.min(available_time);

                // Ensure we have enough time for remaining depths
                if estimated_total > available_time {
                    // Scale down proportionally
                    let scale_factor = available_time as f64 / estimated_total as f64;
                    ((budget as f64 * scale_factor) as u32).max(config.min_time_per_depth_ms)
                } else {
                    budget.max(config.min_time_per_depth_ms)
                }
            } else {
                // Fall back to exponential if no data for this depth
                self.calculate_exponential_budget(depth, available_time, max_depth)
            }
        } else {
            // No historical data: use exponential strategy
            self.calculate_exponential_budget(depth, available_time, max_depth)
        }
    }

    /// Helper function for exponential budget calculation
    fn calculate_exponential_budget(&self, depth: u8, available_time: u32, max_depth: u8) -> u32 {
        let config = &self.config;
        let depth_exponent = depth.saturating_sub(1) as i32;
        let total_weight = (2_f64.powi(max_depth.max(1) as i32) - 1.0).max(1.0);
        let depth_weight = 2_f64.powi(depth_exponent.max(0));

        let allocated = (available_time as f64 * depth_weight / total_weight) as u32;
        let lower_bound = config.min_time_per_depth_ms.min(available_time);
        let upper_bound = available_time;
        allocated.max(lower_bound).min(upper_bound)
    }

    /// Record depth completion time for adaptive allocation
    pub fn record_depth_completion(&mut self, depth: u8, completion_time_ms: u32) {
        let stats = &mut self.time_budget_stats;

        // Ensure we have enough space in the vector
        while stats.depth_completion_times_ms.len() < depth as usize {
            stats.depth_completion_times_ms.push(0);
        }

        if depth > 0 {
            let depth_idx = (depth - 1) as usize;
            if depth_idx < stats.depth_completion_times_ms.len() {
                // Update with exponential moving average (weight recent data more)
                let old_time = stats.depth_completion_times_ms[depth_idx];
                if old_time == 0 {
                    stats.depth_completion_times_ms[depth_idx] = completion_time_ms;
                } else {
                    // EMA with alpha = 0.3 (30% weight to new data)
                    stats.depth_completion_times_ms[depth_idx] =
                        ((old_time as f64 * 0.7) + (completion_time_ms as f64 * 0.3)) as u32;
                }
            } else {
                stats.depth_completion_times_ms.push(completion_time_ms);
            }

            // Update statistics
            stats.depths_completed = stats.depths_completed.max(depth);
            if depth_idx < stats.actual_time_per_depth_ms.len() {
                stats.actual_time_per_depth_ms[depth_idx] = completion_time_ms;
            } else {
                while stats.actual_time_per_depth_ms.len() < depth as usize {
                    stats.actual_time_per_depth_ms.push(0);
                }
                stats.actual_time_per_depth_ms.push(completion_time_ms);
            }
        }
    }

    /// Get time budget statistics for analysis
    pub fn get_time_budget_stats(&self) -> &TimeBudgetStats {
        &self.time_budget_stats
    }

    /// Reset time budget statistics
    pub fn reset_time_budget_stats(&mut self) {
        self.time_budget_stats = TimeBudgetStats::default();
    }

    /// Get time management configuration
    pub fn get_config(&self) -> &TimeManagementConfig {
        &self.config
    }

    /// Update time management configuration
    pub fn update_config(&mut self, config: TimeManagementConfig) {
        self.config = config;
    }

    /// Get time pressure thresholds
    pub fn get_time_pressure_thresholds(&self) -> &TimePressureThresholds {
        &self.time_pressure_thresholds
    }

    /// Update time pressure thresholds
    pub fn update_time_pressure_thresholds(&mut self, thresholds: TimePressureThresholds) {
        self.time_pressure_thresholds = thresholds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TimeAllocationStrategy;

    #[test]
    fn test_time_pressure_calculation() {
        let config = TimeManagementConfig::default();
        let thresholds = TimePressureThresholds::default();
        let manager = TimeManager::new(config, thresholds);
        let start_time = TimeSource::now();

        // Test with plenty of time remaining
        let pressure = manager.calculate_time_pressure_level(&start_time, 1000);
        assert!(matches!(pressure, TimePressure::None | TimePressure::Low));

        // Test with very little time remaining (simulated by checking immediately)
        // Note: This test may be flaky due to timing, but demonstrates the API
    }

    #[test]
    fn test_time_budget_calculation() {
        let config = TimeManagementConfig {
            enable_time_budget: true,
            allocation_strategy: TimeAllocationStrategy::Equal,
            min_time_per_depth_ms: 10,
            safety_margin: 0.1,
            time_check_frequency: 100,
        };
        let thresholds = TimePressureThresholds::default();
        let mut manager = TimeManager::new(config, thresholds);

        // Test equal allocation
        let budget = manager.calculate_time_budget(1, 1000, 0, 5);
        assert!(budget > 0);
        assert!(budget <= 1000);
    }

    #[test]
    fn test_record_depth_completion() {
        let config = TimeManagementConfig::default();
        let thresholds = TimePressureThresholds::default();
        let mut manager = TimeManager::new(config, thresholds);

        manager.record_depth_completion(1, 100);
        manager.record_depth_completion(2, 200);

        let stats = manager.get_time_budget_stats();
        assert!(stats.depths_completed >= 2);
    }
}

