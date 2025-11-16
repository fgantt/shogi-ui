//! Unified Engine Configuration
//!
//! This module provides a unified configuration system for the entire engine,
//! nesting all module-specific configurations as fields. This is part of Task 4.0.
//!
//! # Task 4.0 (Tasks 4.14-4.30)
//!
//! # Examples
//!
//! ## Creating a default configuration
//!
//! ```rust,no_run
//! use shogi_engine::config::EngineConfig;
//!
//! let config = EngineConfig::default();
//! assert!(config.validate().is_ok());
//! ```
//!
//! ## Using a performance preset
//!
//! ```rust,no_run
//! use shogi_engine::config::EngineConfig;
//!
//! let config = EngineConfig::performance();
//! assert!(config.search.max_depth >= 20);
//! ```
//!
//! ## Loading from a file
//!
//! ```rust,no_run
//! use shogi_engine::config::EngineConfig;
//!
//! let config = EngineConfig::from_file("config.json")?;
//! config.validate()?;
//! # Ok::<(), shogi_engine::error::ShogiEngineError>(())
//! ```
//!
//! ## Saving to a file
//!
//! ```rust,no_run
//! use shogi_engine::config::EngineConfig;
//!
//! let config = EngineConfig::default();
//! config.to_file("config.json")?;
//! # Ok::<(), shogi_engine::error::ShogiEngineError>(())
//! ```
//!
//! # Errors
//!
//! - [`ConfigurationError`]: Returned when configuration validation fails or file I/O errors occur
//!
//! # Task 4.0 (Tasks 4.14-4.30)

use crate::error::{ConfigurationError, Result, ShogiEngineError};
use crate::evaluation::config::TaperedEvalConfig;
use crate::search::parallel_search::ParallelSearchConfig;
use crate::search::transposition_table_config::TranspositionTableConfig;
use crate::types::search::TimeManagementConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Search configuration
///
/// Configuration for search-related operations including depth limits,
/// time limits, and search algorithm parameters.
///
/// # Task 4.0 (Task 4.15)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Maximum search depth
    pub max_depth: u8,

    /// Minimum search depth (for quiescence)
    pub min_depth: u8,

    /// Time limit per move in milliseconds (0 = no limit)
    pub time_limit_ms: u64,

    /// Maximum time per move in milliseconds (0 = no limit)
    pub max_time_per_move_ms: u64,

    /// Minimum time per move in milliseconds
    pub min_time_per_move_ms: u64,

    /// Enable iterative deepening
    pub enable_iterative_deepening: bool,

    /// Enable debug logging for search
    pub debug_logging: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_depth: 20,
            min_depth: 1,
            time_limit_ms: 0,
            max_time_per_move_ms: 0,
            min_time_per_move_ms: 100,
            enable_iterative_deepening: true,
            debug_logging: false,
        }
    }
}

impl SearchConfig {
    /// Validate search configuration
    ///
    /// # Task 4.0 (Task 4.25)
    pub fn validate(&self) -> Result<()> {
        if self.max_depth == 0 || self.max_depth > 100 {
            return Err(ConfigurationError::invalid_value(
                "max_depth",
                self.max_depth.to_string(),
                "1-100",
            )
            .into());
        }

        if self.min_depth > self.max_depth {
            return Err(ConfigurationError::invalid_value(
                "min_depth",
                self.min_depth.to_string(),
                format!("must be <= max_depth ({})", self.max_depth),
            )
            .into());
        }

        if self.max_time_per_move_ms > 0
            && self.min_time_per_move_ms > 0
            && self.min_time_per_move_ms > self.max_time_per_move_ms
        {
            return Err(ConfigurationError::invalid_value(
                "min_time_per_move_ms",
                self.min_time_per_move_ms.to_string(),
                format!(
                    "must be <= max_time_per_move_ms ({})",
                    self.max_time_per_move_ms
                ),
            )
            .into());
        }

        Ok(())
    }
}

/// Unified engine configuration
///
/// This struct provides a single place to configure all engine components.
/// All module-specific configurations are nested as fields.
///
/// # Task 4.0 (Task 4.14)
///
/// Note: `TranspositionTableConfig` and `ParallelSearchConfig` don't implement
/// `Serialize` or `Deserialize`, so EngineConfig uses `#[serde(skip)]`
/// for these fields when serializing. They can be set programmatically.
///
/// # Task 4.0 (Task 4.20)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Search configuration
    ///
    /// # Task 4.0 (Task 4.15)
    pub search: SearchConfig,

    /// Evaluation configuration
    ///
    /// # Task 4.0 (Task 4.16)
    pub evaluation: TaperedEvalConfig,

    /// Transposition table configuration
    ///
    /// # Task 4.0 (Task 4.17)
    /// Note: Not serializable, must be set programmatically.
    /// When deserializing from JSON, this will use the default value.
    #[serde(skip_serializing, skip_deserializing)]
    pub transposition: TranspositionTableConfig,

    /// Parallel search configuration
    ///
    /// # Task 4.0 (Task 4.18)
    /// Note: Not serializable, must be set programmatically.
    /// When deserializing from JSON, this will use the default value.
    #[serde(skip_serializing, skip_deserializing)]
    pub parallel: ParallelSearchConfig,

    /// Time management configuration
    ///
    /// # Task 4.0 (Task 4.19)
    pub time_management: TimeManagementConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            evaluation: TaperedEvalConfig::default(),
            transposition: TranspositionTableConfig::default(),
            parallel: ParallelSearchConfig::default(),
            time_management: TimeManagementConfig::default(),
        }
    }
}

impl EngineConfig {
    /// Create a performance-optimized configuration preset
    ///
    /// # Task 4.0 (Task 4.21)
    pub fn performance() -> Self {
        Self {
            search: SearchConfig {
                max_depth: 30,
                min_depth: 1,
                time_limit_ms: 0,
                max_time_per_move_ms: 60000, // 60 seconds
                min_time_per_move_ms: 100,
                enable_iterative_deepening: true,
                debug_logging: false,
            },
            evaluation: TaperedEvalConfig::performance_optimized(),
            transposition: TranspositionTableConfig::performance_optimized(),
            parallel: {
                let mut config = ParallelSearchConfig::default();
                config.enable_parallel = true;
                config.num_threads = num_cpus::get().clamp(1, 32);
                config
            },
            time_management: TimeManagementConfig {
                enabled: true,
                buffer_percentage: 0.1,
                min_time_ms: 100,
                max_time_ms: 60000,
                increment_ms: 1000,
                enable_pressure_detection: true,
                pressure_threshold: 0.2,
                allocation_strategy: crate::types::search::TimeAllocationStrategy::Exponential,
                safety_margin: 0.05,
                min_time_per_depth_ms: 50,
                max_time_per_depth_ms: 10000,
                enable_check_optimization: true,
                check_max_depth: 3,
                check_time_limit_ms: 1000,
                enable_time_budget: true,
                ..TimeManagementConfig::default()
            },
        }
    }

    /// Create a memory-optimized configuration preset
    ///
    /// # Task 4.0 (Task 4.21)
    pub fn memory_optimized() -> Self {
        Self {
            search: SearchConfig {
                max_depth: 15,
                min_depth: 1,
                time_limit_ms: 0,
                max_time_per_move_ms: 30000, // 30 seconds
                min_time_per_move_ms: 100,
                enable_iterative_deepening: true,
                debug_logging: false,
            },
            evaluation: TaperedEvalConfig::memory_optimized(),
            transposition: TranspositionTableConfig::memory_optimized(),
            parallel: {
                let mut config = ParallelSearchConfig::default();
                config.enable_parallel = false; // Disable parallel to save memory
                config.num_threads = 1;
                config.hash_size_mb = 8; // Smaller hash size
                config
            },
            time_management: TimeManagementConfig {
                enabled: true,
                buffer_percentage: 0.15, // Larger buffer for slower searches
                min_time_ms: 100,
                max_time_ms: 30000,
                increment_ms: 500,
                enable_pressure_detection: true,
                pressure_threshold: 0.3,
                allocation_strategy: crate::types::search::TimeAllocationStrategy::Equal,
                safety_margin: 0.1, // Larger safety margin
                min_time_per_depth_ms: 50,
                max_time_per_depth_ms: 5000,
                enable_check_optimization: false, // Disable to save memory
                check_max_depth: 2,
                check_time_limit_ms: 500,
                enable_time_budget: false, // Disable to save memory
                ..TimeManagementConfig::default()
            },
        }
    }

    /// Load configuration from a JSON file
    ///
    /// # Task 4.0 (Task 4.22)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if the file cannot be read or parsed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|_e| {
            ShogiEngineError::Configuration(ConfigurationError::file_not_found(
                path.to_string_lossy().to_string(),
            ))
        })?;

        let config: Self = serde_json::from_str(&content).map_err(|e| {
            ShogiEngineError::Configuration(ConfigurationError::parse_error(
                path.to_string_lossy().to_string(),
                e.to_string(),
            ))
        })?;

        Ok(config)
    }

    /// Save configuration to a JSON file
    ///
    /// # Task 4.0 (Task 4.23)
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the JSON configuration file
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if the file cannot be written or serialization fails.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ShogiEngineError::Configuration(ConfigurationError::serialization_failed(
                e.to_string(),
            ))
        })?;

        std::fs::write(path, json).map_err(|e| {
            ShogiEngineError::Configuration(ConfigurationError::serialization_failed(format!(
                "Failed to write file: {}",
                e
            )))
        })?;

        Ok(())
    }

    /// Validate the entire configuration
    ///
    /// This validates all nested configurations and returns the first error found,
    /// or Ok(()) if all configurations are valid.
    ///
    /// # Task 4.0 (Task 4.24)
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if any configuration is invalid.
    pub fn validate(&self) -> Result<()> {
        // Validate search configuration
        self.search.validate()?;

        // Validate evaluation configuration
        // TaperedEvalConfig::validate() returns Result<Vec<ComponentDependencyWarning>, ConfigError>
        // We'll wrap any error as a ConfigurationError
        if let Err(e) = self.evaluation.validate() {
            return Err(ShogiEngineError::Configuration(ConfigurationError::validation_failed(
                format!("Evaluation config: {:?}", e),
            )));
        }

        // Validate transposition table configuration
        // TranspositionTableConfig::validate() returns Result<(), String>
        self.transposition.validate().map_err(|e| {
            ShogiEngineError::Configuration(ConfigurationError::validation_failed(format!(
                "Transposition table config: {}",
                e
            )))
        })?;

        // Validate time management configuration
        // TimeManagementConfig doesn't have a validate method, so we'll check basic constraints
        if self.time_management.min_time_ms > 0
            && self.time_management.max_time_ms > 0
            && self.time_management.min_time_ms > self.time_management.max_time_ms
        {
            return Err(ShogiEngineError::Configuration(ConfigurationError::invalid_value(
                "time_management.min_time_ms",
                self.time_management.min_time_ms.to_string(),
                format!(
                    "must be <= max_time_ms ({})",
                    self.time_management.max_time_ms
                ),
            )));
        }

        if !(0.0..=1.0).contains(&self.time_management.buffer_percentage) {
            return Err(ShogiEngineError::Configuration(ConfigurationError::invalid_value(
                "time_management.buffer_percentage",
                self.time_management.buffer_percentage.to_string(),
                "0.0-1.0".to_string(),
            )));
        }

        if !(0.0..=1.0).contains(&self.time_management.pressure_threshold) {
            return Err(ShogiEngineError::Configuration(ConfigurationError::invalid_value(
                "time_management.pressure_threshold",
                self.time_management.pressure_threshold.to_string(),
                "0.0-1.0".to_string(),
            )));
        }

        Ok(())
    }
}

// Task 4.0 (Task 4.20): serde Serialize/Deserialize are automatically derived

