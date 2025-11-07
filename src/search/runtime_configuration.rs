//! Runtime configuration system for transposition tables
//!
//! This module provides a flexible runtime configuration system that allows
//! dynamic adjustment of transposition table parameters based on performance
//! metrics, system resources, and user preferences.

use crate::search::transposition_config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Runtime configuration manager for transposition tables
pub struct RuntimeConfigurationManager {
    /// Current active configuration
    active_config: Arc<Mutex<TranspositionConfig>>,
    /// Configuration templates
    templates: HashMap<String, TranspositionConfig>,
    /// Performance metrics for adaptive tuning
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Configuration history for rollback
    config_history: Vec<TranspositionConfig>,
    /// Maximum history size
    max_history_size: usize,
}

/// Performance metrics for adaptive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Current hit rate
    pub hit_rate: f64,
    /// Average operation time in microseconds
    pub avg_operation_time_us: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Collision rate
    pub collision_rate: f64,
    /// Number of replacements per second
    pub replacements_per_second: f64,
    /// System load factor (0.0 to 1.0)
    pub system_load: f64,
    /// Available memory in bytes
    pub available_memory_bytes: u64,
}

/// Configuration update strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationUpdateStrategy {
    /// Immediate update
    Immediate,
    /// Gradual update over time
    Gradual { steps: usize, step_duration_ms: u64 },
    /// Update only if performance improves
    Conditional { min_improvement_threshold: f64 },
    /// Update based on system load
    Adaptive { load_threshold: f64 },
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ConfigurationValidationResult {
    /// Whether the configuration is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// List of warnings
    pub warnings: Vec<String>,
    /// Performance impact assessment
    pub performance_impact: PerformanceImpact,
}

/// Performance impact assessment
#[derive(Debug, Clone)]
pub enum PerformanceImpact {
    /// Positive impact expected
    Positive { improvement_percentage: f64 },
    /// Neutral impact
    Neutral,
    /// Negative impact expected
    Negative { degradation_percentage: f64 },
    /// Unknown impact
    Unknown,
}

impl RuntimeConfigurationManager {
    /// Create a new runtime configuration manager
    pub fn new(initial_config: TranspositionConfig) -> Self {
        let mut manager = Self {
            active_config: Arc::new(Mutex::new(initial_config.clone())),
            templates: HashMap::new(),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            config_history: Vec::new(),
            max_history_size: 10,
        };

        // Add default templates
        manager.add_default_templates();

        // Initialize history with initial config
        manager.config_history.push(initial_config);

        manager
    }

    /// Add default configuration templates
    fn add_default_templates(&mut self) {
        self.templates
            .insert("default".to_string(), TranspositionConfig::default());
        self.templates.insert(
            "performance".to_string(),
            TranspositionConfig::performance_optimized(),
        );
        self.templates.insert(
            "memory".to_string(),
            TranspositionConfig::memory_optimized(),
        );

        // Add custom templates
        self.templates.insert(
            "high_performance".to_string(),
            TranspositionConfig {
                table_size: 1048576, // 1M entries
                replacement_policy: ReplacementPolicy::DepthPreferred,
                max_age: 0,
                enable_prefetching: true,
                enable_memory_mapping: false,
                max_memory_mb: 128,
                clear_between_games: false,
                enable_statistics: true,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: false,
                bucket_count: 512,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
        );

        self.templates.insert(
            "low_memory".to_string(),
            TranspositionConfig {
                table_size: 4096, // 4K entries
                replacement_policy: ReplacementPolicy::AgeBased,
                max_age: 100,
                enable_prefetching: false,
                enable_memory_mapping: false,
                max_memory_mb: 8,
                clear_between_games: true,
                enable_statistics: false,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: false,
                bucket_count: 64,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
        );

        self.templates.insert(
            "balanced".to_string(),
            TranspositionConfig {
                table_size: 65536, // 64K entries
                replacement_policy: ReplacementPolicy::DepthPreferred,
                max_age: 0,
                enable_prefetching: false,
                enable_memory_mapping: false,
                max_memory_mb: 32,
                clear_between_games: false,
                enable_statistics: true,
                collision_strategy: CollisionStrategy::Overwrite,
                validate_hash_keys: false,
                bucket_count: 256,
                depth_weight: 4.0,
                age_weight: 1.0,
            },
        );
    }

    /// Get the current active configuration
    pub fn get_active_config(&self) -> TranspositionConfig {
        self.active_config.lock().unwrap().clone()
    }

    /// Update the active configuration
    pub fn update_config(
        &mut self,
        new_config: TranspositionConfig,
        strategy: ConfigurationUpdateStrategy,
    ) -> Result<(), String> {
        // Validate the new configuration
        let validation = self.validate_configuration(&new_config);
        if !validation.is_valid {
            return Err(format!(
                "Configuration validation failed: {:?}",
                validation.errors
            ));
        }

        // Log warnings if any
        if !validation.warnings.is_empty() {
            println!("Configuration warnings: {:?}", validation.warnings);
        }

        // Store current config in history
        let current_config = self.get_active_config();
        self.config_history.push(current_config);

        // Trim history if needed
        if self.config_history.len() > self.max_history_size {
            self.config_history.remove(0);
        }

        // Apply the update strategy
        match strategy {
            ConfigurationUpdateStrategy::Immediate => {
                *self.active_config.lock().unwrap() = new_config;
            }
            ConfigurationUpdateStrategy::Gradual {
                steps,
                step_duration_ms,
            } => {
                self.apply_gradual_update(new_config, steps, step_duration_ms)?;
            }
            ConfigurationUpdateStrategy::Conditional {
                min_improvement_threshold,
            } => {
                self.apply_conditional_update(new_config, min_improvement_threshold)?;
            }
            ConfigurationUpdateStrategy::Adaptive { load_threshold } => {
                self.apply_adaptive_update(new_config, load_threshold)?;
            }
        }

        Ok(())
    }

    /// Apply gradual configuration update
    fn apply_gradual_update(
        &mut self,
        target_config: TranspositionConfig,
        steps: usize,
        _step_duration_ms: u64,
    ) -> Result<(), String> {
        let current_config = self.get_active_config();

        // Calculate step sizes for each parameter
        let table_size_step =
            (target_config.table_size as i32 - current_config.table_size as i32) / steps as i32;
        let current_size = current_config.table_size as i32;

        // Apply gradual changes (simplified for this example)
        for step in 1..=steps {
            let intermediate_size = current_size + (table_size_step * step as i32);
            let intermediate_size = intermediate_size.max(1024) as usize; // Minimum size

            let intermediate_config = TranspositionConfig {
                table_size: intermediate_size,
                replacement_policy: if step == steps {
                    target_config.replacement_policy
                } else {
                    current_config.replacement_policy
                },
                max_age: target_config.max_age,
                enable_prefetching: target_config.enable_prefetching,
                enable_memory_mapping: target_config.enable_memory_mapping,
                max_memory_mb: target_config.max_memory_mb,
                clear_between_games: target_config.clear_between_games,
                enable_statistics: target_config.enable_statistics,
                collision_strategy: target_config.collision_strategy,
                validate_hash_keys: target_config.validate_hash_keys,
                bucket_count: target_config.bucket_count,
                depth_weight: target_config.depth_weight,
                age_weight: target_config.age_weight,
            };

            if step == steps {
                *self.active_config.lock().unwrap() = target_config.clone();
            } else {
                *self.active_config.lock().unwrap() = intermediate_config;
            }
        }

        Ok(())
    }

    /// Apply conditional configuration update
    fn apply_conditional_update(
        &mut self,
        new_config: TranspositionConfig,
        min_improvement_threshold: f64,
    ) -> Result<(), String> {
        let current_metrics = self.performance_metrics.lock().unwrap().clone();

        // Simulate performance impact assessment
        let performance_impact = self.assess_performance_impact(&new_config, &current_metrics);

        match performance_impact {
            PerformanceImpact::Positive {
                improvement_percentage,
            } => {
                if improvement_percentage >= min_improvement_threshold {
                    *self.active_config.lock().unwrap() = new_config;
                    println!(
                        "Configuration updated with {:.1}% improvement",
                        improvement_percentage
                    );
                } else {
                    return Err(format!(
                        "Improvement ({:.1}%) below threshold ({:.1}%)",
                        improvement_percentage, min_improvement_threshold
                    ));
                }
            }
            PerformanceImpact::Neutral => {
                *self.active_config.lock().unwrap() = new_config;
                println!("Configuration updated with neutral impact");
            }
            PerformanceImpact::Negative {
                degradation_percentage,
            } => {
                return Err(format!(
                    "Configuration would degrade performance by {:.1}%",
                    degradation_percentage
                ));
            }
            PerformanceImpact::Unknown => {
                return Err("Cannot assess performance impact".to_string());
            }
        }

        Ok(())
    }

    /// Apply adaptive configuration update based on system load
    fn apply_adaptive_update(
        &mut self,
        new_config: TranspositionConfig,
        load_threshold: f64,
    ) -> Result<(), String> {
        let metrics = self.performance_metrics.lock().unwrap();

        if metrics.system_load > load_threshold {
            // High system load - use memory-optimized settings
            let memory_optimized = TranspositionConfig::memory_optimized();
            *self.active_config.lock().unwrap() = memory_optimized;
            println!("High system load detected, using memory-optimized configuration");
        } else {
            // Low system load - can use performance-optimized settings
            *self.active_config.lock().unwrap() = new_config;
            println!("Low system load, using performance-optimized configuration");
        }

        Ok(())
    }

    /// Validate a configuration
    pub fn validate_configuration(
        &self,
        config: &TranspositionConfig,
    ) -> ConfigurationValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate table size
        if config.table_size == 0 {
            errors.push("Table size cannot be zero".to_string());
        } else if !config.table_size.is_power_of_two() {
            warnings
                .push("Table size should be a power of two for optimal performance".to_string());
        }

        if config.table_size > 16777216 {
            // 16M entries
            warnings.push("Very large table size may cause memory issues".to_string());
        }

        // Validate memory constraints
        let estimated_memory = config.table_size * 16; // 16 bytes per entry
        if estimated_memory > 268435456 {
            // 256MB
            warnings.push("Large memory usage detected".to_string());
        }

        // Validate statistics setting
        if config.table_size > 65536 && !config.enable_statistics {
            warnings.push("Consider enabling statistics for large tables".to_string());
        }

        // Assess performance impact
        let performance_impact =
            self.assess_performance_impact(config, &PerformanceMetrics::default());

        ConfigurationValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            performance_impact,
        }
    }

    /// Assess performance impact of a configuration
    fn assess_performance_impact(
        &self,
        config: &TranspositionConfig,
        _current_metrics: &PerformanceMetrics,
    ) -> PerformanceImpact {
        let current_config = self.get_active_config();

        // Simple heuristic-based assessment
        let size_ratio = config.table_size as f64 / current_config.table_size as f64;

        if size_ratio > 1.5 {
            PerformanceImpact::Positive {
                improvement_percentage: 15.0,
            }
        } else if size_ratio > 1.0 {
            PerformanceImpact::Positive {
                improvement_percentage: 5.0,
            }
        } else if size_ratio < 0.5 {
            PerformanceImpact::Negative {
                degradation_percentage: 20.0,
            }
        } else if size_ratio < 1.0 {
            PerformanceImpact::Negative {
                degradation_percentage: 5.0,
            }
        } else {
            PerformanceImpact::Neutral
        }
    }

    /// Get a configuration template by name
    pub fn get_template(&self, name: &str) -> Option<&TranspositionConfig> {
        self.templates.get(name)
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Add a custom template
    pub fn add_template(
        &mut self,
        name: String,
        config: TranspositionConfig,
    ) -> Result<(), String> {
        let validation = self.validate_configuration(&config);
        if !validation.is_valid {
            return Err(format!(
                "Template validation failed: {:?}",
                validation.errors
            ));
        }

        self.templates.insert(name, config);
        Ok(())
    }

    /// Remove a template
    pub fn remove_template(&mut self, name: &str) -> bool {
        if name == "default" || name == "performance" || name == "memory" {
            false // Cannot remove built-in templates
        } else {
            self.templates.remove(name).is_some()
        }
    }

    /// Update performance metrics
    pub fn update_performance_metrics(&self, metrics: PerformanceMetrics) {
        *self.performance_metrics.lock().unwrap() = metrics;
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.lock().unwrap().clone()
    }

    /// Rollback to previous configuration
    pub fn rollback_config(&mut self) -> Result<(), String> {
        if self.config_history.len() < 2 {
            return Err("No previous configuration to rollback to".to_string());
        }

        // Remove current config and get previous one
        self.config_history.pop(); // Remove current
        if let Some(previous_config) = self.config_history.pop() {
            *self.active_config.lock().unwrap() = previous_config.clone();
            self.config_history.push(previous_config); // Keep it in history
            Ok(())
        } else {
            Err("Failed to rollback configuration".to_string())
        }
    }

    /// Get configuration history
    pub fn get_config_history(&self) -> Vec<TranspositionConfig> {
        self.config_history.clone()
    }

    /// Clear configuration history
    pub fn clear_history(&mut self) {
        self.config_history.clear();
        // Add current config as the only entry
        self.config_history.push(self.get_active_config());
    }

    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;

        // Trim history if needed
        while self.config_history.len() > self.max_history_size {
            self.config_history.remove(0);
        }
    }

    /// Export current configuration to JSON
    pub fn export_config(&self) -> Result<String, String> {
        let config = self.get_active_config();
        serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))
    }

    /// Import configuration from JSON
    pub fn import_config(&mut self, json: &str) -> Result<(), String> {
        let config: TranspositionConfig = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize configuration: {}", e))?;

        let validation = self.validate_configuration(&config);
        if !validation.is_valid {
            return Err(format!(
                "Imported configuration validation failed: {:?}",
                validation.errors
            ));
        }

        self.update_config(config, ConfigurationUpdateStrategy::Immediate)
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            hit_rate: 0.0,
            avg_operation_time_us: 0.0,
            memory_usage_bytes: 0,
            collision_rate: 0.0,
            replacements_per_second: 0.0,
            system_load: 0.0,
            available_memory_bytes: u64::MAX,
        }
    }
}

/// Configuration builder for easy configuration creation
pub struct ConfigurationBuilder {
    config: TranspositionConfig,
}

impl ConfigurationBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: TranspositionConfig::default(),
        }
    }

    /// Set table size
    pub fn table_size(mut self, size: usize) -> Self {
        self.config.table_size = size;
        self
    }

    /// Set replacement policy
    pub fn replacement_policy(mut self, policy: ReplacementPolicy) -> Self {
        self.config.replacement_policy = policy;
        self
    }

    /// Enable or disable statistics
    pub fn enable_statistics(mut self, enable: bool) -> Self {
        self.config.enable_statistics = enable;
        self
    }

    /// Enable or disable memory mapping
    pub fn enable_memory_mapping(mut self, enable: bool) -> Self {
        self.config.enable_memory_mapping = enable;
        self
    }

    /// Set maximum memory usage in MB
    pub fn max_memory_mb(mut self, max_mb: usize) -> Self {
        self.config.max_memory_mb = max_mb;
        self
    }

    /// Enable or disable prefetching
    pub fn enable_prefetching(mut self, enable: bool) -> Self {
        self.config.enable_prefetching = enable;
        self
    }

    /// Build the configuration
    pub fn build(self) -> TranspositionConfig {
        self.config
    }
}

impl Default for ConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_configuration_manager_creation() {
        let config = TranspositionConfig::default();
        let manager = RuntimeConfigurationManager::new(config);

        assert!(manager.list_templates().contains(&"default".to_string()));
        assert!(manager
            .list_templates()
            .contains(&"performance".to_string()));
        assert!(manager.list_templates().contains(&"memory".to_string()));
    }

    #[test]
    fn test_configuration_validation() {
        let config = TranspositionConfig::default();
        let manager = RuntimeConfigurationManager::new(config);

        let valid_config = TranspositionConfig::default();
        let validation = manager.validate_configuration(&valid_config);
        assert!(validation.is_valid);

        let invalid_config = TranspositionConfig {
            table_size: 0,
            ..TranspositionConfig::default()
        };
        let validation = manager.validate_configuration(&invalid_config);
        assert!(!validation.is_valid);
    }

    #[test]
    fn test_configuration_builder() {
        let config = ConfigurationBuilder::new()
            .table_size(65536)
            .replacement_policy(ReplacementPolicy::DepthPreferred)
            .enable_statistics(true)
            .build();

        assert_eq!(config.table_size, 65536);
        assert_eq!(config.replacement_policy, ReplacementPolicy::DepthPreferred);
        assert!(config.enable_statistics);
    }

    #[test]
    fn test_template_management() {
        let config = TranspositionConfig::default();
        let mut manager = RuntimeConfigurationManager::new(config);

        let custom_config = TranspositionConfig {
            table_size: 32768,
            ..TranspositionConfig::default()
        };

        assert!(manager
            .add_template("custom".to_string(), custom_config)
            .is_ok());
        assert!(manager.get_template("custom").is_some());
        assert!(manager.remove_template("custom"));
        assert!(!manager.remove_template("default")); // Cannot remove built-in
    }

    #[test]
    fn test_configuration_rollback() {
        let config = TranspositionConfig::default();
        let mut manager = RuntimeConfigurationManager::new(config);

        let new_config = TranspositionConfig::performance_optimized();
        manager
            .update_config(new_config, ConfigurationUpdateStrategy::Immediate)
            .unwrap();

        assert!(manager.rollback_config().is_ok());
        assert_eq!(
            manager.get_active_config().table_size,
            TranspositionConfig::default().table_size
        );
    }
}
