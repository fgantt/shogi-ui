//! Configuration system for transposition table
//!
//! This module provides a flexible configuration system for the transposition table,
//! allowing customization of table size, replacement policies, and other parameters.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Replacement policy for transposition table entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementPolicy {
    /// Always replace (FIFO-like behavior)
    AlwaysReplace,
    /// Replace only if new entry has higher depth
    DepthPreferred,
    /// Replace based on age (newer entries preferred)
    AgeBased,
    /// Replace based on combined depth and age
    DepthAndAge,
    /// Replace only exact scores, keep bounds
    ExactPreferred,
}

impl Default for ReplacementPolicy {
    fn default() -> Self {
        ReplacementPolicy::DepthAndAge
    }
}

impl ReplacementPolicy {
    /// Get a human-readable description of the policy
    pub fn description(&self) -> &'static str {
        match self {
            ReplacementPolicy::AlwaysReplace => "Always replace existing entries",
            ReplacementPolicy::DepthPreferred => "Replace only if new entry has higher depth",
            ReplacementPolicy::AgeBased => "Replace based on age (newer entries preferred)",
            ReplacementPolicy::DepthAndAge => "Replace based on combined depth and age",
            ReplacementPolicy::ExactPreferred => "Replace only exact scores, keep bounds",
        }
    }

    /// Check if this policy considers depth in replacement decisions
    pub fn considers_depth(&self) -> bool {
        matches!(
            self,
            ReplacementPolicy::DepthPreferred | ReplacementPolicy::DepthAndAge
        )
    }

    /// Check if this policy considers age in replacement decisions
    pub fn considers_age(&self) -> bool {
        matches!(
            self,
            ReplacementPolicy::AgeBased | ReplacementPolicy::DepthAndAge
        )
    }
}

/// Configuration for transposition table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspositionConfig {
    /// Size of the transposition table in entries (must be power of 2)
    pub table_size: usize,
    /// Replacement policy to use
    pub replacement_policy: ReplacementPolicy,
    /// Maximum age for entries (0 = no limit)
    pub max_age: u32,
    /// Whether to use prefetching for better cache performance
    pub enable_prefetching: bool,
    /// Whether to use memory mapping for large tables
    pub enable_memory_mapping: bool,
    /// Maximum memory usage in MB (0 = no limit)
    pub max_memory_mb: usize,
    /// Whether to clear table between games
    pub clear_between_games: bool,
    /// Whether to enable statistics collection
    pub enable_statistics: bool,
    /// Hash collision handling strategy
    pub collision_strategy: CollisionStrategy,
    /// Whether to validate hash keys on probe
    pub validate_hash_keys: bool,
    /// Number of lock buckets for parallel write performance (must be power of 2)
    /// Higher values reduce contention but increase memory overhead
    /// Recommended: 256 for 4-8 threads, 512 for 16+ threads
    pub bucket_count: usize,
    /// Weight for depth in depth-and-age replacement policy
    pub depth_weight: f64,
    /// Weight for age in depth-and-age replacement policy
    pub age_weight: f64,
}

/// Strategy for handling hash collisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionStrategy {
    /// Always overwrite on collision
    Overwrite,
    /// Use replacement policy on collision
    UseReplacementPolicy,
    /// Chain entries (not implemented yet)
    Chain,
}

impl Default for CollisionStrategy {
    fn default() -> Self {
        CollisionStrategy::UseReplacementPolicy
    }
}

impl Default for TranspositionConfig {
    fn default() -> Self {
        Self {
            table_size: 1024 * 1024, // 1M entries
            replacement_policy: ReplacementPolicy::default(),
            max_age: 1000,
            enable_prefetching: true,
            enable_memory_mapping: false,
            max_memory_mb: 512, // 512 MB
            clear_between_games: true,
            enable_statistics: false,
            collision_strategy: CollisionStrategy::default(),
            validate_hash_keys: true,
            bucket_count: 256, // 256 buckets for good 4-8 thread scaling
            depth_weight: 4.0, // Depth is 4× more important than age
            age_weight: 1.0,
        }
    }
}

impl TranspositionConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        Self {
            table_size: 256 * 1024, // 256K entries
            replacement_policy: ReplacementPolicy::AgeBased,
            max_age: 500,
            enable_prefetching: false,
            enable_memory_mapping: true,
            max_memory_mb: 128, // 128 MB
            clear_between_games: true,
            enable_statistics: false,
            collision_strategy: CollisionStrategy::Overwrite,
            validate_hash_keys: false,
            bucket_count: 128, // Fewer buckets for memory savings
            depth_weight: 4.0,
            age_weight: 1.0,
        }
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            table_size: 4 * 1024 * 1024, // 4M entries
            replacement_policy: ReplacementPolicy::DepthAndAge,
            max_age: 2000,
            enable_prefetching: true,
            enable_memory_mapping: false,
            max_memory_mb: 1024, // 1 GB
            clear_between_games: false,
            enable_statistics: true,
            collision_strategy: CollisionStrategy::UseReplacementPolicy,
            validate_hash_keys: true,
            bucket_count: 512, // More buckets for better high-thread scaling
            depth_weight: 4.0,
            age_weight: 1.0,
        }
    }

    /// Create a configuration for testing/debugging
    pub fn debug_config() -> Self {
        Self {
            table_size: 1024, // Small table for testing
            replacement_policy: ReplacementPolicy::AlwaysReplace,
            max_age: 100,
            enable_prefetching: false,
            enable_memory_mapping: false,
            max_memory_mb: 16, // 16 MB
            clear_between_games: true,
            enable_statistics: true,
            collision_strategy: CollisionStrategy::Overwrite,
            validate_hash_keys: true,
            bucket_count: 16, // Small bucket count for testing
            depth_weight: 4.0,
            age_weight: 1.0,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check table size is power of 2
        if !self.table_size.is_power_of_two() {
            return Err(ConfigError::InvalidTableSize(self.table_size));
        }

        // Check minimum table size
        if self.table_size < 1024 {
            return Err(ConfigError::TableSizeTooSmall(self.table_size));
        }

        // Check maximum table size (16M entries = ~1GB)
        if self.table_size > 16 * 1024 * 1024 {
            return Err(ConfigError::TableSizeTooLarge(self.table_size));
        }

        // Check memory limits
        if self.max_memory_mb > 0 {
            let estimated_memory = self.estimated_memory_usage_mb();
            if estimated_memory > self.max_memory_mb {
                return Err(ConfigError::MemoryLimitExceeded {
                    estimated: estimated_memory,
                    limit: self.max_memory_mb,
                });
            }
        }

        // Check max age is reasonable
        if self.max_age > 10000 {
            return Err(ConfigError::MaxAgeTooLarge(self.max_age));
        }

        // Check bucket count is power of 2
        if !self.bucket_count.is_power_of_two() {
            return Err(ConfigError::InvalidParameter(format!(
                "Bucket count must be a power of 2, got {}",
                self.bucket_count
            )));
        }

        // Check bucket count is reasonable (1-4096)
        if self.bucket_count < 1 || self.bucket_count > 4096 {
            return Err(ConfigError::InvalidParameter(format!(
                "Bucket count must be between 1 and 4096, got {}",
                self.bucket_count
            )));
        }

        Ok(())
    }

    /// Estimate memory usage in MB
    pub fn estimated_memory_usage_mb(&self) -> usize {
        // Each entry is approximately 32 bytes (hash + entry data)
        let entry_size = 32;
        let total_bytes = self.table_size * entry_size;
        total_bytes / (1024 * 1024)
    }

    /// Get the mask for fast modulo operations (table_size - 1)
    pub fn get_mask(&self) -> usize {
        self.table_size - 1
    }

    /// Check if the configuration is valid for the current system
    pub fn is_system_compatible(&self) -> bool {
        // Check if we have enough memory
        if self.max_memory_mb > 0 {
            let estimated = self.estimated_memory_usage_mb();
            if estimated > self.max_memory_mb {
                return false;
            }
        }

        // Additional system checks could be added here
        true
    }

    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::FileReadError(e.to_string()))?;

        let config: TranspositionConfig =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        self.validate()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        fs::write(path, content).map_err(|e| ConfigError::FileWriteError(e.to_string()))?;

        Ok(())
    }

    /// Create a configuration builder for fluent API
    pub fn builder() -> TranspositionConfigBuilder {
        TranspositionConfigBuilder::new()
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> ConfigSummary {
        ConfigSummary {
            table_size: self.table_size,
            estimated_memory_mb: self.estimated_memory_usage_mb(),
            replacement_policy: self.replacement_policy,
            max_age: self.max_age,
            features_enabled: self.get_enabled_features(),
        }
    }

    /// Get list of enabled features
    fn get_enabled_features(&self) -> Vec<&'static str> {
        let mut features = Vec::new();

        if self.enable_prefetching {
            features.push("Prefetching");
        }
        if self.enable_memory_mapping {
            features.push("Memory Mapping");
        }
        if self.enable_statistics {
            features.push("Statistics");
        }
        if self.validate_hash_keys {
            features.push("Hash Validation");
        }
        if self.clear_between_games {
            features.push("Clear Between Games");
        }

        features
    }
}

/// Builder for TranspositionConfig
pub struct TranspositionConfigBuilder {
    config: TranspositionConfig,
}

impl TranspositionConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TranspositionConfig::new(),
        }
    }

    /// Set the table size
    pub fn table_size(mut self, size: usize) -> Self {
        self.config.table_size = size;
        self
    }

    /// Set the replacement policy
    pub fn replacement_policy(mut self, policy: ReplacementPolicy) -> Self {
        self.config.replacement_policy = policy;
        self
    }

    /// Set the maximum age
    pub fn max_age(mut self, age: u32) -> Self {
        self.config.max_age = age;
        self
    }

    /// Enable or disable prefetching
    pub fn enable_prefetching(mut self, enable: bool) -> Self {
        self.config.enable_prefetching = enable;
        self
    }

    /// Enable or disable memory mapping
    pub fn enable_memory_mapping(mut self, enable: bool) -> Self {
        self.config.enable_memory_mapping = enable;
        self
    }

    /// Set the maximum memory usage in MB
    pub fn max_memory_mb(mut self, mb: usize) -> Self {
        self.config.max_memory_mb = mb;
        self
    }

    /// Enable or disable clearing between games
    pub fn clear_between_games(mut self, enable: bool) -> Self {
        self.config.clear_between_games = enable;
        self
    }

    /// Enable or disable statistics collection
    pub fn enable_statistics(mut self, enable: bool) -> Self {
        self.config.enable_statistics = enable;
        self
    }

    /// Set the collision strategy
    pub fn collision_strategy(mut self, strategy: CollisionStrategy) -> Self {
        self.config.collision_strategy = strategy;
        self
    }

    /// Enable or disable hash key validation
    pub fn validate_hash_keys(mut self, enable: bool) -> Self {
        self.config.validate_hash_keys = enable;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<TranspositionConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Configuration summary for display
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub table_size: usize,
    pub estimated_memory_mb: usize,
    pub replacement_policy: ReplacementPolicy,
    pub max_age: u32,
    pub features_enabled: Vec<&'static str>,
}

impl ConfigSummary {
    /// Get a formatted string representation
    pub fn to_string(&self) -> String {
        format!(
            "Table Size: {} entries ({} MB)\n\
             Replacement Policy: {:?}\n\
             Max Age: {}\n\
             Features: {}",
            self.table_size,
            self.estimated_memory_mb,
            self.replacement_policy,
            self.max_age,
            self.features_enabled.join(", ")
        )
    }
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    InvalidTableSize(usize),
    TableSizeTooSmall(usize),
    TableSizeTooLarge(usize),
    MemoryLimitExceeded { estimated: usize, limit: usize },
    MaxAgeTooLarge(u32),
    InvalidParameter(String),
    FileReadError(String),
    FileWriteError(String),
    ParseError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidTableSize(size) => {
                write!(f, "Table size {} is not a power of 2", size)
            }
            ConfigError::TableSizeTooSmall(size) => {
                write!(f, "Table size {} is too small (minimum 1024)", size)
            }
            ConfigError::TableSizeTooLarge(size) => {
                write!(f, "Table size {} is too large (maximum 16M)", size)
            }
            ConfigError::MemoryLimitExceeded { estimated, limit } => {
                write!(
                    f,
                    "Estimated memory usage {} MB exceeds limit {} MB",
                    estimated, limit
                )
            }
            ConfigError::MaxAgeTooLarge(age) => {
                write!(f, "Max age {} is too large (maximum 10000)", age)
            }
            ConfigError::InvalidParameter(msg) => {
                write!(f, "Invalid parameter: {}", msg)
            }
            ConfigError::FileReadError(msg) => {
                write!(f, "Failed to read configuration file: {}", msg)
            }
            ConfigError::FileWriteError(msg) => {
                write!(f, "Failed to write configuration file: {}", msg)
            }
            ConfigError::ParseError(msg) => {
                write!(f, "Failed to parse configuration: {}", msg)
            }
            ConfigError::SerializeError(msg) => {
                write!(f, "Failed to serialize configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = TranspositionConfig::new();
        assert_eq!(config.table_size, 1024 * 1024);
        assert_eq!(config.replacement_policy, ReplacementPolicy::DepthAndAge);
        assert_eq!(config.max_age, 1000);
        assert!(config.enable_prefetching);
        assert!(!config.enable_memory_mapping);
        assert_eq!(config.max_memory_mb, 512);
        assert!(config.clear_between_games);
        assert!(!config.enable_statistics);
        assert_eq!(
            config.collision_strategy,
            CollisionStrategy::UseReplacementPolicy
        );
        assert!(config.validate_hash_keys);
    }

    #[test]
    fn test_enable_statistics_via_builder() {
        let config = TranspositionConfigBuilder::new()
            .enable_statistics(true)
            .build()
            .expect("builder should validate");
        assert!(config.enable_statistics);
    }

    #[test]
    fn test_config_validation() {
        // Valid configuration
        let config = TranspositionConfig::new();
        assert!(config.validate().is_ok());

        // Invalid table size (not power of 2)
        let mut invalid_config = TranspositionConfig::new();
        invalid_config.table_size = 1000;
        assert!(matches!(
            invalid_config.validate(),
            Err(ConfigError::InvalidTableSize(1000))
        ));

        // Table size too small
        invalid_config.table_size = 512;
        assert!(matches!(
            invalid_config.validate(),
            Err(ConfigError::TableSizeTooSmall(512))
        ));

        // Table size too large
        invalid_config.table_size = 32 * 1024 * 1024;
        let result = invalid_config.validate();
        assert!(matches!(result, Err(ConfigError::TableSizeTooLarge(_))));

        // Max age too large
        invalid_config.table_size = 1024 * 1024;
        invalid_config.max_age = 20000;
        assert!(matches!(
            invalid_config.validate(),
            Err(ConfigError::MaxAgeTooLarge(20000))
        ));
    }

    #[test]
    fn test_replacement_policy() {
        let policy = ReplacementPolicy::DepthAndAge;
        assert!(policy.considers_depth());
        assert!(policy.considers_age());

        let policy = ReplacementPolicy::DepthPreferred;
        assert!(policy.considers_depth());
        assert!(!policy.considers_age());

        let policy = ReplacementPolicy::AgeBased;
        assert!(!policy.considers_depth());
        assert!(policy.considers_age());
    }

    #[test]
    fn test_config_builder() {
        let config = TranspositionConfig::builder()
            .table_size(2048)
            .replacement_policy(ReplacementPolicy::AlwaysReplace)
            .max_age(500)
            .enable_prefetching(false)
            .build()
            .unwrap();

        assert_eq!(config.table_size, 2048);
        assert_eq!(config.replacement_policy, ReplacementPolicy::AlwaysReplace);
        assert_eq!(config.max_age, 500);
        assert!(!config.enable_prefetching);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create and save configuration
        let config = TranspositionConfig::debug_config();
        config.to_file(&config_path).unwrap();

        // Load configuration
        let loaded_config = TranspositionConfig::from_file(&config_path).unwrap();
        assert_eq!(config.table_size, loaded_config.table_size);
        assert_eq!(config.replacement_policy, loaded_config.replacement_policy);
        assert_eq!(config.max_age, loaded_config.max_age);

        // Clean up
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_config_summary() {
        let config = TranspositionConfig::new();
        let summary = config.summary();

        assert_eq!(summary.table_size, config.table_size);
        assert_eq!(
            summary.estimated_memory_mb,
            config.estimated_memory_usage_mb()
        );
        assert_eq!(summary.replacement_policy, config.replacement_policy);
        assert_eq!(summary.max_age, config.max_age);

        let summary_str = summary.to_string();
        assert!(summary_str.contains("Table Size"));
        assert!(summary_str.contains("Replacement Policy"));
        assert!(summary_str.contains("Max Age"));
        assert!(summary_str.contains("Features"));
    }

    #[test]
    fn test_predefined_configs() {
        // Test memory optimized config
        let mem_config = TranspositionConfig::memory_optimized();
        assert_eq!(mem_config.table_size, 256 * 1024);
        assert_eq!(mem_config.replacement_policy, ReplacementPolicy::AgeBased);
        assert!(mem_config.enable_memory_mapping);
        assert!(!mem_config.enable_prefetching);

        // Test performance optimized config
        let perf_config = TranspositionConfig::performance_optimized();
        assert_eq!(perf_config.table_size, 4 * 1024 * 1024);
        assert_eq!(
            perf_config.replacement_policy,
            ReplacementPolicy::DepthAndAge
        );
        assert!(!perf_config.enable_memory_mapping);
        assert!(perf_config.enable_prefetching);

        // Test debug config
        let debug_config = TranspositionConfig::debug_config();
        assert_eq!(debug_config.table_size, 1024);
        assert_eq!(
            debug_config.replacement_policy,
            ReplacementPolicy::AlwaysReplace
        );
        assert!(debug_config.validate_hash_keys);
    }

    #[test]
    fn test_memory_estimation() {
        let config = TranspositionConfig::new();
        let estimated = config.estimated_memory_usage_mb();

        // Should be approximately 32 MB for 1M entries
        assert!(estimated >= 30 && estimated <= 35);

        // Test with different sizes
        let small_config = TranspositionConfig::debug_config();
        let small_estimated = small_config.estimated_memory_usage_mb();
        assert!(small_estimated < estimated);

        let large_config = TranspositionConfig::performance_optimized();
        let large_estimated = large_config.estimated_memory_usage_mb();
        assert!(large_estimated > estimated);
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::InvalidTableSize(1000);
        let error_str = format!("{}", error);
        assert!(error_str.contains("1000"));
        assert!(error_str.contains("power of 2"));

        let error = ConfigError::MemoryLimitExceeded {
            estimated: 1000,
            limit: 500,
        };
        let error_str = format!("{}", error);
        assert!(error_str.contains("1000"));
        assert!(error_str.contains("500"));
        assert!(error_str.contains("exceeds limit"));
    }
}
