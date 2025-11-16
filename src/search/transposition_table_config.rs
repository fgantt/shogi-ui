//! Unified Transposition Table Configuration
//!
//! This module provides a unified configuration enum for all transposition table types,
//! enabling polymorphic table creation. This is part of Task 3.0 - Integration
//! Synchronization and Coordination Fixes.

use crate::search::compressed_transposition_table::CompressedTranspositionTableConfig;
use crate::search::hierarchical_transposition_table::HierarchicalTranspositionConfig;
use crate::search::multi_level_transposition_table::MultiLevelConfig;
use crate::search::transposition_config::TranspositionConfig;
use crate::search::transposition_table_trait::TranspositionTableTrait;
use std::cell::RefCell;

// Re-export table types for convenience
pub use crate::search::transposition_table::TranspositionTableConfig as BasicTranspositionTableConfig;

// Re-export for convenience
pub use crate::search::transposition_config::TranspositionConfig as BaseTranspositionConfig;

/// Unified configuration for transposition table types
///
/// This enum provides a single way to specify which type of transposition table
/// to create and its configuration. This enables polymorphic table creation
/// through the factory function.
///
/// # Task 3.0 (Task 3.22)
#[derive(Debug, Clone)]
pub enum TranspositionTableConfig {
    /// Basic single-threaded transposition table
    Basic {
        /// Configuration for the basic table
        config: BasicTranspositionTableConfig,
    },
    /// Thread-safe transposition table (default for multi-threaded environments)
    ThreadSafe {
        /// Configuration for the thread-safe table
        config: TranspositionConfig,
    },
    /// Hierarchical table with L1 (fast) and L2 (compressed) tiers
    #[cfg(feature = "hierarchical-tt")]
    Hierarchical {
        /// Configuration for the hierarchical table
        config: HierarchicalTranspositionConfig,
    },
    /// Multi-level table with different levels for different depths
    MultiLevel {
        /// Configuration for the multi-level table
        config: MultiLevelConfig,
    },
    /// Compressed table for memory-efficient storage
    #[cfg(feature = "hierarchical-tt")]
    Compressed {
        /// Configuration for the compressed table
        config: CompressedTranspositionTableConfig,
    },
}

impl Default for TranspositionTableConfig {
    fn default() -> Self {
        Self::ThreadSafe {
            config: TranspositionConfig::default(),
        }
    }
}

impl TranspositionTableConfig {
    /// Create a basic table configuration
    pub fn basic(config: BasicTranspositionTableConfig) -> Self {
        Self::Basic { config }
    }

    /// Create a thread-safe table configuration
    pub fn thread_safe(config: TranspositionConfig) -> Self {
        Self::ThreadSafe { config }
    }

    /// Create a hierarchical table configuration
    #[cfg(feature = "hierarchical-tt")]
    pub fn hierarchical(config: HierarchicalTranspositionConfig) -> Self {
        Self::Hierarchical { config }
    }

    /// Create a multi-level table configuration
    pub fn multi_level(config: MultiLevelConfig) -> Self {
        Self::MultiLevel { config }
    }

    /// Create a compressed table configuration
    #[cfg(feature = "hierarchical-tt")]
    pub fn compressed(config: CompressedTranspositionTableConfig) -> Self {
        Self::Compressed { config }
    }

    /// Create a default thread-safe table configuration
    pub fn default_thread_safe() -> Self {
        Self::ThreadSafe {
            config: TranspositionConfig::default(),
        }
    }

    /// Create a performance-optimized thread-safe table configuration
    pub fn performance_optimized() -> Self {
        Self::ThreadSafe {
            config: TranspositionConfig::performance_optimized(),
        }
    }

    /// Create a memory-optimized thread-safe table configuration
    pub fn memory_optimized() -> Self {
        Self::ThreadSafe {
            config: TranspositionConfig::memory_optimized(),
        }
    }

    /// Validate the configuration
    ///
    /// This validates the underlying configuration based on the table type.
    /// Returns an error if the configuration is invalid.
    ///
    /// # Task 3.0 (Task 3.14)
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TranspositionTableConfig::Basic { config } => {
                // Basic table config validation
                if config.max_entries == 0 {
                    return Err("Basic table max_entries must be > 0".to_string());
                }
                if !config.max_entries.is_power_of_two() {
                    return Err(format!(
                        "Basic table max_entries must be a power of 2, got {}",
                        config.max_entries
                    ));
                }
                Ok(())
            }
            TranspositionTableConfig::ThreadSafe { config } => {
                // Thread-safe table config validation (uses TranspositionConfig::validate)
                config.validate().map_err(|e| format!("Thread-safe table config error: {}", e))
            }
            #[cfg(feature = "hierarchical-tt")]
            TranspositionTableConfig::Hierarchical { config } => {
                // Hierarchical table config validation
                // Validate L1 config
                let mut l1_config = crate::search::transposition_config::TranspositionConfig::default();
                l1_config.table_size = config.l1_config.table_size;
                l1_config.enable_statistics = config.l1_config.enable_statistics;
                if let Err(e) = l1_config.validate() {
                    return Err(format!("Hierarchical table L1 config error: {}", e));
                }
                // Validate L2 config
                if config.l2_config.max_entries == 0 {
                    return Err("Hierarchical table L2 max_entries must be > 0".to_string());
                }
                if config.l2_config.segment_count == 0 {
                    return Err("Hierarchical table L2 segment_count must be > 0".to_string());
                }
                Ok(())
            }
            TranspositionTableConfig::MultiLevel { config } => {
                // Multi-level table config validation
                if config.levels == 0 {
                    return Err("Multi-level table levels must be > 0".to_string());
                }
                if config.base_size == 0 {
                    return Err("Multi-level table base_size must be > 0".to_string());
                }
                if config.min_level_size > config.max_level_size {
                    return Err(format!(
                        "Multi-level table min_level_size ({}) must be <= max_level_size ({})",
                        config.min_level_size, config.max_level_size
                    ));
                }
                Ok(())
            }
            #[cfg(feature = "hierarchical-tt")]
            TranspositionTableConfig::Compressed { config } => {
                // Compressed table config validation
                if config.max_entries == 0 {
                    return Err("Compressed table max_entries must be > 0".to_string());
                }
                if config.segment_count == 0 {
                    return Err("Compressed table segment_count must be > 0".to_string());
                }
                if !(0.1..=1.0).contains(&config.target_compression_ratio) {
                    return Err(format!(
                        "Compressed table target_compression_ratio must be between 0.1 and 1.0, got {}",
                        config.target_compression_ratio
                    ));
                }
                Ok(())
            }
        }
    }
}

/// Factory function for creating transposition tables
///
/// This function creates a boxed trait object implementing `TranspositionTableTrait`,
/// allowing polymorphic usage throughout the search engine.
///
/// Note: Due to trait object limitations and different mutability requirements,
/// this function wraps all tables in RefCell for consistency, even though
/// ThreadSafeTranspositionTable could theoretically work without it.
///
/// # Task 3.0 (Task 3.23)
pub fn create_transposition_table(
    config: TranspositionTableConfig,
) -> Box<dyn TranspositionTableTrait> {
    match config {
        TranspositionTableConfig::Basic { config } => {
            let table = crate::search::transposition_table::TranspositionTable::with_config(config);
            Box::new(RefCell::new(table))
        }
        TranspositionTableConfig::ThreadSafe { config } => {
            // Wrap ThreadSafeTranspositionTable in RefCell for consistency with other types
            // even though it doesn't strictly need it (it uses interior mutability)
            let table = crate::search::thread_safe_table::ThreadSafeTranspositionTable::new(config);
            Box::new(RefCell::new(table))
        }
        #[cfg(feature = "hierarchical-tt")]
        TranspositionTableConfig::Hierarchical { config } => {
            let table = crate::search::hierarchical_transposition_table::HierarchicalTranspositionTable::new(config);
            Box::new(RefCell::new(table))
        }
        TranspositionTableConfig::MultiLevel { config } => {
            let table = crate::search::multi_level_transposition_table::MultiLevelTranspositionTable::with_config(config);
            Box::new(RefCell::new(table))
        }
        #[cfg(feature = "hierarchical-tt")]
        TranspositionTableConfig::Compressed { config } => {
            let table = crate::search::compressed_transposition_table::CompressedTranspositionTable::new(config);
            Box::new(RefCell::new(table))
        }
    }
}

