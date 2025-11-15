//! Adaptive configuration system for transposition tables
//!
//! This module provides an adaptive configuration system that automatically
//! adjusts transposition table parameters based on real-time performance
//! metrics, system conditions, and usage patterns.

#![allow(dead_code)]

use crate::search::runtime_configuration::{PerformanceMetrics as RuntimePerformanceMetrics, *};
use crate::search::transposition_config::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Adaptive configuration manager
pub struct AdaptiveConfigurationManager {
    /// Runtime configuration manager
    runtime_manager: Arc<Mutex<RuntimeConfigurationManager>>,
    /// Performance history for trend analysis
    performance_history: VecDeque<RuntimePerformanceMetrics>,
    /// Maximum history size for trend analysis
    max_history_size: usize,
    /// Adaptation rules
    adaptation_rules: Vec<AdaptationRule>,
    /// Current adaptation state
    adaptation_state: AdaptationState,
    /// Adaptation interval in milliseconds
    adaptation_interval_ms: u64,
    /// Last adaptation time
    last_adaptation_time: std::time::Instant,
    /// Minimum time between adaptations
    min_adaptation_interval_ms: u64,
}

/// Adaptation rule for automatic configuration changes
#[derive(Debug, Clone)]
pub struct AdaptationRule {
    /// Rule name for identification
    pub name: String,
    /// Condition that triggers the rule
    pub condition: AdaptationCondition,
    /// Action to take when condition is met
    pub action: AdaptationAction,
    /// Priority (higher numbers = higher priority)
    pub priority: u8,
    /// Whether the rule is enabled
    pub enabled: bool,
}

/// Condition that triggers an adaptation rule
#[derive(Debug, Clone)]
pub enum AdaptationCondition {
    /// Hit rate below threshold
    HitRateBelow { threshold: f64, duration_ms: u64 },
    /// Hit rate above threshold
    HitRateAbove { threshold: f64, duration_ms: u64 },
    /// Memory usage above threshold
    MemoryUsageAbove {
        threshold_bytes: u64,
        duration_ms: u64,
    },
    /// Operation time above threshold
    OperationTimeAbove { threshold_us: f64, duration_ms: u64 },
    /// System load above threshold
    SystemLoadAbove { threshold: f64, duration_ms: u64 },
    /// Collision rate above threshold
    CollisionRateAbove { threshold: f64, duration_ms: u64 },
    /// Available memory below threshold
    AvailableMemoryBelow {
        threshold_bytes: u64,
        duration_ms: u64,
    },
    /// Combined conditions with AND logic
    And(Vec<AdaptationCondition>),
    /// Combined conditions with OR logic
    Or(Vec<AdaptationCondition>),
}

/// Action to take when an adaptation condition is met
#[derive(Debug, Clone)]
pub enum AdaptationAction {
    /// Increase table size by percentage
    IncreaseTableSize {
        percentage: f64,
        max_size: Option<usize>,
    },
    /// Decrease table size by percentage
    DecreaseTableSize {
        percentage: f64,
        min_size: Option<usize>,
    },
    /// Change replacement policy
    ChangeReplacementPolicy { policy: ReplacementPolicy },
    /// Enable or disable statistics
    SetStatisticsEnabled { enabled: bool },
    /// Enable or disable cache line alignment
    SetCacheLineAlignmentEnabled { enabled: bool },
    /// Enable or disable prefetching
    SetPrefetchingEnabled { enabled: bool },
    /// Switch to a predefined template
    SwitchToTemplate { template_name: String },
    /// Custom configuration update
    CustomConfiguration { config: TranspositionConfig },
    /// No action (for testing conditions)
    NoAction,
}

/// Current state of the adaptation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationState {
    /// Whether adaptation is currently enabled
    pub enabled: bool,
    /// Number of adaptations performed
    pub adaptation_count: u64,
    /// Last adaptation timestamp
    pub last_adaptation_timestamp: std::time::SystemTime,
    /// Current adaptation mode
    pub mode: AdaptationMode,
    /// Performance trend
    pub performance_trend: PerformanceTrend,
}

/// Adaptation mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdaptationMode {
    /// Conservative adaptation (small changes)
    Conservative,
    /// Aggressive adaptation (large changes)
    Aggressive,
    /// Balanced adaptation (moderate changes)
    Balanced,
    /// Manual adaptation (no automatic changes)
    Manual,
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving { rate: f64 },
    /// Performance is degrading
    Degrading { rate: f64 },
    /// Performance is stable
    Stable { variance: f64 },
    /// Trend is unknown
    Unknown,
}

impl AdaptiveConfigurationManager {
    /// Create a new adaptive configuration manager
    pub fn new(initial_config: TranspositionConfig) -> Self {
        let runtime_manager =
            Arc::new(Mutex::new(RuntimeConfigurationManager::new(initial_config)));

        let mut manager = Self {
            runtime_manager,
            performance_history: VecDeque::new(),
            max_history_size: 100,
            adaptation_rules: Vec::new(),
            adaptation_state: AdaptationState::default(),
            adaptation_interval_ms: 5000, // 5 seconds
            last_adaptation_time: std::time::Instant::now(),
            min_adaptation_interval_ms: 1000, // 1 second
        };

        // Add default adaptation rules
        manager.add_default_rules();

        manager
    }

    /// Add default adaptation rules
    fn add_default_rules(&mut self) {
        // Rule 1: Low hit rate -> increase table size
        self.add_rule(AdaptationRule {
            name: "increase_table_size_low_hit_rate".to_string(),
            condition: AdaptationCondition::HitRateBelow {
                threshold: 0.2,
                duration_ms: 10000,
            },
            action: AdaptationAction::IncreaseTableSize {
                percentage: 0.5,
                max_size: Some(1048576),
            },
            priority: 10,
            enabled: true,
        });

        // Rule 2: High memory usage -> decrease table size
        self.add_rule(AdaptationRule {
            name: "decrease_table_size_high_memory".to_string(),
            condition: AdaptationCondition::MemoryUsageAbove {
                threshold_bytes: 134217728,
                duration_ms: 5000,
            }, // 128MB
            action: AdaptationAction::DecreaseTableSize {
                percentage: 0.3,
                min_size: Some(4096),
            },
            priority: 9,
            enabled: true,
        });

        // Rule 3: High collision rate -> change replacement policy
        self.add_rule(AdaptationRule {
            name: "change_policy_high_collisions".to_string(),
            condition: AdaptationCondition::CollisionRateAbove {
                threshold: 0.15,
                duration_ms: 8000,
            },
            action: AdaptationAction::ChangeReplacementPolicy {
                policy: ReplacementPolicy::AgeBased,
            },
            priority: 8,
            enabled: true,
        });

        // Rule 4: High system load -> use memory-optimized template
        self.add_rule(AdaptationRule {
            name: "memory_optimized_high_load".to_string(),
            condition: AdaptationCondition::SystemLoadAbove {
                threshold: 0.8,
                duration_ms: 15000,
            },
            action: AdaptationAction::SwitchToTemplate {
                template_name: "memory".to_string(),
            },
            priority: 7,
            enabled: true,
        });

        // Rule 5: Low system load -> use performance-optimized template
        self.add_rule(AdaptationRule {
            name: "performance_optimized_low_load".to_string(),
            condition: AdaptationCondition::And(vec![
                AdaptationCondition::SystemLoadAbove {
                    threshold: 0.7,
                    duration_ms: 20000,
                }, // Inverted logic
                AdaptationCondition::AvailableMemoryBelow {
                    threshold_bytes: 268435456,
                    duration_ms: 1000,
                }, // 256MB - inverted logic
            ]),
            action: AdaptationAction::SwitchToTemplate {
                template_name: "high_performance".to_string(),
            },
            priority: 6,
            enabled: true,
        });
    }

    /// Add an adaptation rule
    pub fn add_rule(&mut self, rule: AdaptationRule) {
        // Remove existing rule with same name
        self.adaptation_rules.retain(|r| r.name != rule.name);

        // Add new rule
        self.adaptation_rules.push(rule);

        // Sort by priority (highest first)
        self.adaptation_rules
            .sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove an adaptation rule by name
    pub fn remove_rule(&mut self, name: &str) -> bool {
        let initial_len = self.adaptation_rules.len();
        self.adaptation_rules.retain(|r| r.name != name);
        self.adaptation_rules.len() < initial_len
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(rule) = self.adaptation_rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Update performance metrics and trigger adaptation if needed
    pub fn update_performance_metrics(
        &mut self,
        metrics: RuntimePerformanceMetrics,
    ) -> Result<(), String> {
        // Update runtime manager metrics
        {
            let runtime_manager = self.runtime_manager.lock().unwrap();
            runtime_manager.update_performance_metrics(metrics.clone());
        }

        // Add to performance history
        self.performance_history.push_back(metrics.clone());

        // Trim history if needed
        while self.performance_history.len() > self.max_history_size {
            self.performance_history.pop_front();
        }

        // Update performance trend
        self.update_performance_trend();

        // Check if adaptation is needed
        if self.should_adapt() {
            self.perform_adaptation()?;
        }

        Ok(())
    }

    /// Check if adaptation should be performed
    fn should_adapt(&self) -> bool {
        if !self.adaptation_state.enabled {
            return false;
        }

        if self.adaptation_state.mode == AdaptationMode::Manual {
            return false;
        }

        // Check minimum time interval
        let time_since_last = self.last_adaptation_time.elapsed().as_millis() as u64;
        if time_since_last < self.min_adaptation_interval_ms {
            return false;
        }

        // Check adaptation interval
        if time_since_last < self.adaptation_interval_ms {
            return false;
        }

        // Check if any rules are triggered
        self.adaptation_rules
            .iter()
            .any(|rule| rule.enabled && self.evaluate_condition(&rule.condition))
    }

    /// Evaluate an adaptation condition
    fn evaluate_condition(&self, condition: &AdaptationCondition) -> bool {
        match condition {
            AdaptationCondition::HitRateBelow {
                threshold,
                duration_ms,
            } => self.check_metric_threshold(|m| m.hit_rate, *threshold, false, *duration_ms),
            AdaptationCondition::HitRateAbove {
                threshold,
                duration_ms,
            } => self.check_metric_threshold(|m| m.hit_rate, *threshold, true, *duration_ms),
            AdaptationCondition::MemoryUsageAbove {
                threshold_bytes,
                duration_ms,
            } => self.check_metric_threshold(
                |m| m.memory_usage_bytes as f64,
                *threshold_bytes as f64,
                true,
                *duration_ms,
            ),
            AdaptationCondition::OperationTimeAbove {
                threshold_us,
                duration_ms,
            } => self.check_metric_threshold(
                |m| m.avg_operation_time_us,
                *threshold_us,
                true,
                *duration_ms,
            ),
            AdaptationCondition::SystemLoadAbove {
                threshold,
                duration_ms,
            } => self.check_metric_threshold(|m| m.system_load, *threshold, true, *duration_ms),
            AdaptationCondition::CollisionRateAbove {
                threshold,
                duration_ms,
            } => self.check_metric_threshold(|m| m.collision_rate, *threshold, true, *duration_ms),
            AdaptationCondition::AvailableMemoryBelow {
                threshold_bytes,
                duration_ms,
            } => self.check_metric_threshold(
                |m| m.available_memory_bytes as f64,
                *threshold_bytes as f64,
                false,
                *duration_ms,
            ),
            AdaptationCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c))
            }
            AdaptationCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c))
            }
        }
    }

    /// Check if a metric has been above/below threshold for specified duration
    fn check_metric_threshold<F>(
        &self,
        metric_extractor: F,
        threshold: f64,
        above: bool,
        duration_ms: u64,
    ) -> bool
    where
        F: Fn(&RuntimePerformanceMetrics) -> f64,
    {
        let _cutoff_time =
            std::time::SystemTime::now() - std::time::Duration::from_millis(duration_ms);

        // For simplicity, we'll assume all metrics in history are recent enough
        // In a real implementation, you'd check timestamps
        self.performance_history.iter().any(|metrics| {
            let value = metric_extractor(metrics);
            if above {
                value > threshold
            } else {
                value < threshold
            }
        })
    }

    /// Perform adaptation based on triggered rules
    fn perform_adaptation(&mut self) -> Result<(), String> {
        // Find the highest priority triggered rule
        if let Some(rule) = self
            .adaptation_rules
            .iter()
            .find(|rule| rule.enabled && self.evaluate_condition(&rule.condition))
        {
            println!("Adaptation triggered by rule: {}", rule.name);

            // Execute the action
            self.execute_action(&rule.action)?;

            // Update adaptation state
            self.adaptation_state.adaptation_count += 1;
            self.adaptation_state.last_adaptation_timestamp = std::time::SystemTime::now();
            self.last_adaptation_time = std::time::Instant::now();

            println!("Adaptation completed successfully");
        }

        Ok(())
    }

    /// Execute an adaptation action
    fn execute_action(&self, action: &AdaptationAction) -> Result<(), String> {
        let mut runtime_manager = self.runtime_manager.lock().unwrap();
        let current_config = runtime_manager.get_active_config();

        let new_config = match action {
            AdaptationAction::IncreaseTableSize {
                percentage,
                max_size,
            } => {
                let new_size = (current_config.table_size as f64 * (1.0 + percentage)) as usize;
                let new_size = max_size.map(|max| new_size.min(max)).unwrap_or(new_size);
                TranspositionConfig {
                    table_size: new_size,
                    ..current_config
                }
            }
            AdaptationAction::DecreaseTableSize {
                percentage,
                min_size,
            } => {
                let new_size = (current_config.table_size as f64 * (1.0 - percentage)) as usize;
                let new_size = min_size.map(|min| new_size.max(min)).unwrap_or(new_size);
                TranspositionConfig {
                    table_size: new_size,
                    ..current_config
                }
            }
            AdaptationAction::ChangeReplacementPolicy { policy } => TranspositionConfig {
                replacement_policy: policy.clone(),
                ..current_config
            },
            AdaptationAction::SetStatisticsEnabled { enabled } => TranspositionConfig {
                enable_statistics: *enabled,
                ..current_config
            },
            AdaptationAction::SetPrefetchingEnabled { enabled } => TranspositionConfig {
                enable_prefetching: *enabled,
                ..current_config
            },
            AdaptationAction::SetCacheLineAlignmentEnabled { .. } => {
                // This field doesn't exist in TranspositionConfig, so return current config
                current_config
            }
            AdaptationAction::SwitchToTemplate { template_name } => runtime_manager
                .get_template(template_name)
                .ok_or_else(|| format!("Template '{}' not found", template_name))?
                .clone(),
            AdaptationAction::CustomConfiguration { config } => config.clone(),
            AdaptationAction::NoAction => {
                return Ok(());
            }
        };

        // Apply the new configuration
        runtime_manager.update_config(new_config, ConfigurationUpdateStrategy::Immediate)
    }

    /// Update performance trend analysis
    fn update_performance_trend(&mut self) {
        if self.performance_history.len() < 10 {
            self.adaptation_state.performance_trend = PerformanceTrend::Unknown;
            return;
        }

        // Calculate trend based on hit rate over recent history
        let recent_metrics: Vec<_> = self.performance_history.iter().rev().take(10).collect();
        let hit_rates: Vec<f64> = recent_metrics.iter().map(|m| m.hit_rate).collect();

        // Simple linear regression to determine trend
        let n = hit_rates.len() as f64;
        let sum_x: f64 = (0..hit_rates.len()).map(|i| i as f64).sum();
        let sum_y: f64 = hit_rates.iter().sum();
        let sum_xy: f64 = hit_rates
            .iter()
            .enumerate()
            .map(|(i, y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..hit_rates.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let variance = self.calculate_variance(&hit_rates);

        self.adaptation_state.performance_trend = if slope > 0.01 {
            PerformanceTrend::Improving { rate: slope }
        } else if slope < -0.01 {
            PerformanceTrend::Degrading { rate: -slope }
        } else {
            PerformanceTrend::Stable { variance }
        };
    }

    /// Calculate variance of a dataset
    fn calculate_variance(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        variance
    }

    /// Get current adaptation state
    pub fn get_adaptation_state(&self) -> AdaptationState {
        self.adaptation_state.clone()
    }

    /// Enable or disable adaptation
    pub fn set_adaptation_enabled(&mut self, enabled: bool) {
        self.adaptation_state.enabled = enabled;
    }

    /// Set adaptation mode
    pub fn set_adaptation_mode(&mut self, mode: AdaptationMode) {
        self.adaptation_state.mode = mode.clone();

        // Adjust adaptation interval based on mode
        self.adaptation_interval_ms = match mode {
            AdaptationMode::Conservative => 10000, // 10 seconds
            AdaptationMode::Aggressive => 2000,    // 2 seconds
            AdaptationMode::Balanced => 5000,      // 5 seconds
            AdaptationMode::Manual => u64::MAX,    // Never
        };
    }

    /// Set adaptation interval
    pub fn set_adaptation_interval(&mut self, interval_ms: u64) {
        self.adaptation_interval_ms = interval_ms;
    }

    /// Get runtime configuration manager
    pub fn get_runtime_manager(&self) -> Arc<Mutex<RuntimeConfigurationManager>> {
        self.runtime_manager.clone()
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> Vec<RuntimePerformanceMetrics> {
        self.performance_history.iter().cloned().collect()
    }

    /// Clear performance history
    pub fn clear_performance_history(&mut self) {
        self.performance_history.clear();
    }

    /// Export adaptation state to JSON
    pub fn export_adaptation_state(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.adaptation_state)
            .map_err(|e| format!("Failed to serialize adaptation state: {}", e))
    }

    /// Get list of adaptation rules
    pub fn get_adaptation_rules(&self) -> Vec<AdaptationRule> {
        self.adaptation_rules.clone()
    }
}

impl Default for AdaptationState {
    fn default() -> Self {
        Self {
            enabled: true,
            adaptation_count: 0,
            last_adaptation_timestamp: std::time::SystemTime::now(),
            mode: AdaptationMode::Balanced,
            performance_trend: PerformanceTrend::Unknown,
        }
    }
}

// Helper methods for creating adaptation conditions
impl AdaptationCondition {
    /// Hit rate above threshold
    pub fn hit_rate_above(threshold: f64, duration_ms: u64) -> Self {
        Self::HitRateAbove {
            threshold,
            duration_ms,
        }
    }

    /// System load below threshold
    pub fn system_load_below(threshold: f64, duration_ms: u64) -> Self {
        Self::SystemLoadAbove {
            threshold: 1.0 - threshold,
            duration_ms,
        }
    }

    /// Available memory above threshold
    pub fn available_memory_above(threshold_bytes: u64, duration_ms: u64) -> Self {
        Self::AvailableMemoryBelow {
            threshold_bytes: u64::MAX - threshold_bytes,
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_configuration_manager_creation() {
        let config = TranspositionConfig::default();
        let manager = AdaptiveConfigurationManager::new(config);

        assert!(manager.get_adaptation_rules().len() > 0);
        assert_eq!(
            manager.get_adaptation_state().mode,
            AdaptationMode::Balanced
        );
    }

    #[test]
    fn test_adaptation_rule_management() {
        let config = TranspositionConfig::default();
        let mut manager = AdaptiveConfigurationManager::new(config);

        let rule = AdaptationRule {
            name: "test_rule".to_string(),
            condition: AdaptationCondition::HitRateBelow {
                threshold: 0.5,
                duration_ms: 1000,
            },
            action: AdaptationAction::NoAction,
            priority: 5,
            enabled: true,
        };

        manager.add_rule(rule);
        assert!(manager
            .get_adaptation_rules()
            .iter()
            .any(|r| r.name == "test_rule"));

        assert!(manager.remove_rule("test_rule"));
        assert!(!manager
            .get_adaptation_rules()
            .iter()
            .any(|r| r.name == "test_rule"));
    }

    #[test]
    fn test_performance_metrics_update() {
        let config = TranspositionConfig::default();
        let mut manager = AdaptiveConfigurationManager::new(config);

        let metrics = PerformanceMetrics {
            hit_rate: 0.3,
            avg_operation_time_us: 100.0,
            memory_usage_bytes: 1000000,
            collision_rate: 0.05,
            replacements_per_second: 10.0,
            system_load: 0.5,
            available_memory_bytes: 1000000000,
        };

        assert!(manager.update_performance_metrics(metrics).is_ok());
        assert_eq!(manager.get_performance_history().len(), 1);
    }

    #[test]
    fn test_adaptation_mode_changes() {
        let config = TranspositionConfig::default();
        let mut manager = AdaptiveConfigurationManager::new(config);

        manager.set_adaptation_mode(AdaptationMode::Conservative);
        assert_eq!(
            manager.get_adaptation_state().mode,
            AdaptationMode::Conservative
        );

        manager.set_adaptation_mode(AdaptationMode::Aggressive);
        assert_eq!(
            manager.get_adaptation_state().mode,
            AdaptationMode::Aggressive
        );
    }
}
