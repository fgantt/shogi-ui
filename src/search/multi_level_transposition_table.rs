//! Multi-Level Transposition Table
//!
//! This module implements a multi-level transposition table system that uses
//! multiple hash tables at different levels to improve cache efficiency and
//! reduce collisions. This approach provides better performance for large
//! search trees and reduces the impact of hash collisions.
//!
//! # Features
//!
//! - **Multiple Table Levels**: Different tables for different search depths
//! - **Collision Reduction**: Reduced hash collisions through level separation
//! - **Memory Efficiency**: Optimized memory usage across levels
//! - **Performance Optimization**: Faster access patterns for different depths
//! - **Configurable Levels**: Adjustable number of levels and table sizes
//!
//! # Usage
//!
//! ```rust
//! use shogi_engine::search::{MultiLevelTranspositionTable, TranspositionEntry, TranspositionFlag};
//!
//! // Create a multi-level table with 3 levels
//! let mut table = MultiLevelTranspositionTable::new(3, 1024);
//!
//! // Store an entry - automatically selects appropriate level
//! let entry = TranspositionEntry {
//!     hash_key: 12345,
//!     depth: 5,
//!     score: 100,
//!     flag: TranspositionFlag::Exact,
//!     best_move: None,
//!     age: 0,
//! };
//! table.store(entry);
//!
//! // Probe for an entry
//! if let Some(found) = table.probe(12345, 5) {
//!     println!("Found entry with score: {}", found.score);
//! }
//! ```

use crate::search::transposition_table::{TranspositionTable, TranspositionTableConfig};
use crate::types::transposition::TranspositionEntry;
use std::collections::HashMap;

/// Multi-level transposition table configuration
#[derive(Debug, Clone)]
pub struct MultiLevelConfig {
    /// Number of levels in the table
    pub levels: usize,
    /// Base table size (will be adjusted per level)
    pub base_size: usize,
    /// Size multiplier for each level (level i has size base_size * (multiplier^i))
    pub size_multiplier: f64,
    /// Minimum size for any level
    pub min_level_size: usize,
    /// Maximum size for any level
    pub max_level_size: usize,
    /// Depth threshold for each level (0-based)
    pub depth_thresholds: Vec<u8>,
    /// Enable level-specific replacement policies
    pub enable_level_policies: bool,
    /// Memory allocation strategy
    pub allocation_strategy: MemoryAllocationStrategy,
}

/// Memory allocation strategy for multi-level tables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryAllocationStrategy {
    /// Equal memory allocation across all levels
    Equal,
    /// Proportional allocation based on level index (higher levels get more memory)
    Proportional,
    /// Custom allocation based on expected usage patterns
    Custom,
}

/// Level-specific configuration
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// Table size for this level
    pub size: usize,
    /// Depth range for this level
    pub min_depth: u8,
    pub max_depth: u8,
    /// Replacement policy for this level
    pub replacement_policy: crate::search::transposition_table::ReplacementPolicy,
    /// Memory priority (higher = more likely to get memory)
    pub memory_priority: u8,
}

impl Default for MultiLevelConfig {
    fn default() -> Self {
        Self {
            levels: 3,
            base_size: 1024,
            size_multiplier: 1.5,
            min_level_size: 256,
            max_level_size: 65536,
            depth_thresholds: vec![2, 6], // Level 0: depth 0-2, Level 1: depth 3-6, Level 2: depth 7+
            enable_level_policies: true,
            allocation_strategy: MemoryAllocationStrategy::Proportional,
        }
    }
}

/// Multi-level transposition table statistics
#[derive(Debug, Clone, Default)]
pub struct MultiLevelStats {
    /// Per-level statistics
    pub level_stats: Vec<LevelStats>,
    /// Overall statistics
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_stores: u64,
    pub total_replacements: u64,
    /// Cross-level statistics
    pub cross_level_hits: u64,
    pub cross_level_misses: u64,
    /// Memory usage per level
    pub level_memory_usage: Vec<u64>,
    /// Total memory usage
    pub total_memory_usage: u64,
}

/// Per-level statistics
#[derive(Debug, Clone, Default)]
pub struct LevelStats {
    /// Level index
    pub level: usize,
    /// Hits at this level
    pub hits: u64,
    /// Misses at this level
    pub misses: u64,
    /// Stores at this level
    pub stores: u64,
    /// Replacements at this level
    pub replacements: u64,
    /// Hit rate at this level
    pub hit_rate: f64,
    /// Average operation time in microseconds
    pub avg_operation_time_us: f64,
    /// Memory usage for this level
    pub memory_usage: u64,
}

/// Multi-level transposition table
///
/// A sophisticated transposition table that uses multiple levels to improve
/// cache efficiency and reduce hash collisions. Each level is optimized for
/// different depth ranges and usage patterns.
pub struct MultiLevelTranspositionTable {
    /// Individual transposition tables for each level
    tables: Vec<TranspositionTable>,
    /// Configuration for each level
    level_configs: Vec<LevelConfig>,
    /// Overall configuration
    config: MultiLevelConfig,
    /// Statistics
    stats: MultiLevelStats,
    /// Current age counter
    age: u32,
    /// Level selection cache for performance optimization
    level_cache: HashMap<u8, usize>,
}

impl MultiLevelTranspositionTable {
    /// Create a new multi-level transposition table
    pub fn new(levels: usize, base_size: usize) -> Self {
        let mut config = MultiLevelConfig::default();
        config.levels = levels;
        config.base_size = base_size;
        Self::with_config(config)
    }

    /// Create with custom configuration
    pub fn with_config(config: MultiLevelConfig) -> Self {
        let mut tables = Vec::new();
        let mut level_configs = Vec::new();
        let mut level_stats = Vec::new();
        let mut level_memory_usage = Vec::new();

        // Create tables and configurations for each level
        for level in 0..config.levels {
            let level_config = Self::create_level_config(&config, level);
            let table_config = TranspositionTableConfig {
                max_entries: level_config.size,
                replacement_policy: level_config.replacement_policy.clone(),
                track_memory: true,
                track_statistics: true,
            };

            let table = TranspositionTable::with_config(table_config);
            let memory_usage = table.get_memory_usage();

            tables.push(table);
            level_configs.push(level_config);
            level_stats.push(LevelStats {
                level,
                memory_usage: memory_usage as u64,
                ..Default::default()
            });
            level_memory_usage.push(memory_usage as u64);
        }

        // Build level selection cache
        let mut level_cache = HashMap::new();
        for depth in 0..=20 {
            // Cache for depths 0-20
            let level = Self::select_level(&config, depth);
            level_cache.insert(depth, level);
        }

        Self {
            tables,
            level_configs,
            config,
            stats: MultiLevelStats {
                level_stats,
                level_memory_usage: level_memory_usage.clone(),
                total_memory_usage: level_memory_usage.iter().sum(),
                ..Default::default()
            },
            age: 0,
            level_cache,
        }
    }

    /// Store an entry in the appropriate level
    pub fn store(&mut self, entry: TranspositionEntry) {
        let level = self.get_level_for_depth(entry.depth);

        // Update statistics
        self.stats.total_stores += 1;
        self.stats.level_stats[level].stores += 1;

        // Check if replacement will occur (simplified check)
        let existing_entry = self.tables[level].probe(entry.hash_key, 0);
        if existing_entry.is_some() {
            self.stats.total_replacements += 1;
            self.stats.level_stats[level].replacements += 1;
        }

        // Store in the appropriate level
        self.tables[level].store(entry.clone());

        // Update age
        self.age = self.age.wrapping_add(1);
    }

    /// Probe for an entry, checking all relevant levels
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TranspositionEntry> {
        let primary_level = self.get_level_for_depth(depth);

        // First, try the primary level for this depth
        if let Some(entry) = self.tables[primary_level].probe(hash, depth) {
            self.stats.total_hits += 1;
            self.stats.level_stats[primary_level].hits += 1;
            self.update_level_hit_rate(primary_level);
            return Some(entry);
        }

        // If not found in primary level, check other levels (cross-level search)
        for level in 0..self.config.levels {
            if level != primary_level {
                if let Some(entry) = self.tables[level].probe(hash, depth) {
                    self.stats.total_hits += 1;
                    self.stats.cross_level_hits += 1;
                    self.stats.level_stats[level].hits += 1;
                    self.update_level_hit_rate(level);
                    return Some(entry);
                }
            }
        }

        // Not found anywhere
        self.stats.total_misses += 1;
        self.stats.cross_level_misses += 1;
        self.stats.level_stats[primary_level].misses += 1;
        self.update_level_hit_rate(primary_level);
        None
    }

    /// Clear all tables
    pub fn clear(&mut self) {
        for table in &mut self.tables {
            table.clear();
        }
        self.age = 0;
        self.stats = MultiLevelStats {
            level_stats: self
                .stats
                .level_stats
                .iter()
                .map(|s| LevelStats {
                    level: s.level,
                    memory_usage: s.memory_usage,
                    ..Default::default()
                })
                .collect(),
            level_memory_usage: self.stats.level_memory_usage.clone(),
            total_memory_usage: self.stats.total_memory_usage,
            ..Default::default()
        };
    }

    /// Get statistics for all levels
    pub fn get_stats(&self) -> &MultiLevelStats {
        &self.stats
    }

    /// Get statistics for a specific level
    pub fn get_level_stats(&self, level: usize) -> Option<&LevelStats> {
        self.stats.level_stats.get(level)
    }

    /// Get memory usage per level
    pub fn get_level_memory_usage(&self) -> Vec<u64> {
        self.stats.level_memory_usage.clone()
    }

    /// Get total memory usage
    pub fn get_total_memory_usage(&self) -> u64 {
        self.stats.total_memory_usage
    }

    /// Resize a specific level
    pub fn resize_level(&mut self, level: usize, new_size: usize) -> Result<(), String> {
        if level >= self.config.levels {
            return Err(format!("Invalid level: {}", level));
        }

        if new_size < self.config.min_level_size || new_size > self.config.max_level_size {
            return Err(format!(
                "Size {} is outside allowed range [{}, {}]",
                new_size, self.config.min_level_size, self.config.max_level_size
            ));
        }

        // Create new table with new size
        let level_config = &self.level_configs[level];
        let new_table_config = TranspositionTableConfig {
            max_entries: new_size,
            replacement_policy: level_config.replacement_policy.clone(),
            track_memory: true,
            track_statistics: true,
        };

        self.tables[level] = TranspositionTable::with_config(new_table_config);

        // Update configuration
        self.level_configs[level].size = new_size;

        // Update memory usage
        let memory_usage = self.tables[level].get_memory_usage();
        self.stats.level_stats[level].memory_usage = memory_usage as u64;
        self.stats.level_memory_usage[level] = memory_usage as u64;
        self.stats.total_memory_usage = self.stats.level_memory_usage.iter().sum();

        Ok(())
    }

    /// Get the optimal level for a given depth
    fn get_level_for_depth(&self, depth: u8) -> usize {
        // Use cached result if available
        if let Some(&level) = self.level_cache.get(&depth) {
            return level;
        }

        // Calculate level (cache miss)
        let level = Self::select_level(&self.config, depth);
        level
    }

    /// Select the appropriate level for a given depth
    fn select_level(config: &MultiLevelConfig, depth: u8) -> usize {
        for (level, &threshold) in config.depth_thresholds.iter().enumerate() {
            if depth <= threshold {
                return level;
            }
        }
        config.levels - 1 // Use the last level for very deep searches
    }

    /// Create configuration for a specific level
    fn create_level_config(config: &MultiLevelConfig, level: usize) -> LevelConfig {
        // Calculate size for this level
        let size = match config.allocation_strategy {
            MemoryAllocationStrategy::Equal => config.base_size,
            MemoryAllocationStrategy::Proportional => {
                let multiplier = config.size_multiplier.powi(level as i32);
                ((config.base_size as f64) * multiplier) as usize
            }
            MemoryAllocationStrategy::Custom => {
                // Custom sizing based on level priority
                let priority_multiplier = (level + 1) as f64;
                ((config.base_size as f64) * priority_multiplier) as usize
            }
        };

        let size = size.max(config.min_level_size).min(config.max_level_size);

        // Calculate depth range
        let min_depth = if level == 0 {
            0
        } else {
            config.depth_thresholds[level - 1] + 1
        };
        let max_depth = if level < config.depth_thresholds.len() {
            config.depth_thresholds[level]
        } else {
            255 // No upper limit for the last level
        };

        // Select replacement policy for this level
        let replacement_policy = if config.enable_level_policies {
            match level {
                0 => crate::search::transposition_table::ReplacementPolicy::AgeBased, // Shallow levels favor newer entries
                1 => crate::search::transposition_table::ReplacementPolicy::DepthPreferred, // Medium levels favor depth
                _ => crate::search::transposition_table::ReplacementPolicy::DepthPreferred, // Deep levels favor depth
            }
        } else {
            crate::search::transposition_table::ReplacementPolicy::DepthPreferred
        };

        LevelConfig {
            size,
            min_depth,
            max_depth,
            replacement_policy,
            memory_priority: level as u8,
        }
    }

    /// Update hit rate for a specific level
    fn update_level_hit_rate(&mut self, level: usize) {
        let level_stats = &mut self.stats.level_stats[level];
        let total = level_stats.hits + level_stats.misses;
        level_stats.hit_rate = if total > 0 {
            level_stats.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

impl Default for MultiLevelTranspositionTable {
    fn default() -> Self {
        Self::new(3, 1024)
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_multi_level_table_creation() {
        let table = MultiLevelTranspositionTable::new(3, 1024);
        assert_eq!(table.config.levels, 3);
        assert_eq!(table.tables.len(), 3);
        assert_eq!(table.level_configs.len(), 3);
    }

    #[test]
    fn test_multi_level_store_and_probe() {
        let mut table = MultiLevelTranspositionTable::new(3, 1024);

        let entry = TranspositionEntry {
            hash_key: 12345,
            depth: 3,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };

        table.store(entry.clone());

        let found = table.probe(12345, 3);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.score, 100);
        assert_eq!(found.depth, 3);
    }

    #[test]
    fn test_level_selection() {
        let config = MultiLevelConfig {
            levels: 3,
            depth_thresholds: vec![2, 6],
            ..Default::default()
        };

        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 0), 0);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 1), 0);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 2), 0);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 3), 1);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 6), 1);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 7), 2);
        assert_eq!(MultiLevelTranspositionTable::select_level(&config, 20), 2);
    }

    #[test]
    fn test_cross_level_search() {
        let mut table = MultiLevelTranspositionTable::new(3, 1024);

        // Store in level 0 (depth 1)
        let entry1 = TranspositionEntry {
            hash_key: 12345,
            depth: 1,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        table.store(entry1);

        // Store in level 1 (depth 4)
        let entry2 = TranspositionEntry {
            hash_key: 54321,
            depth: 4,
            score: 200,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        table.store(entry2);

        // Should find both entries regardless of search depth
        let found1 = table.probe(12345, 1);
        assert!(found1.is_some());
        assert_eq!(found1.unwrap().score, 100);

        let found2 = table.probe(54321, 4);
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().score, 200);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut table = MultiLevelTranspositionTable::new(3, 1024);

        let entry = TranspositionEntry {
            hash_key: 12345,
            depth: 2,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };

        table.store(entry.clone());
        table.probe(12345, 2);
        table.probe(54321, 2); // Miss

        let stats = table.get_stats();
        assert_eq!(stats.total_stores, 1);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);

        let level_stats = &stats.level_stats[0]; // Should be in level 0
        assert_eq!(level_stats.stores, 1);
        assert_eq!(level_stats.hits, 1);
        assert_eq!(level_stats.misses, 1);
        assert!(level_stats.hit_rate > 0.0);
    }

    #[test]
    fn test_level_resize() {
        let mut table = MultiLevelTranspositionTable::new(3, 1024);

        let initial_size = table.level_configs[0].size;
        let new_size = initial_size * 2;

        let result = table.resize_level(0, new_size);
        assert!(result.is_ok());
        assert_eq!(table.level_configs[0].size, new_size);
    }

    #[test]
    fn test_invalid_level_resize() {
        let mut table = MultiLevelTranspositionTable::new(3, 1024);

        // Invalid level
        let result = table.resize_level(5, 2048);
        assert!(result.is_err());

        // Invalid size
        let result = table.resize_level(0, 50); // Too small
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_allocation_strategies() {
        let mut config = MultiLevelConfig::default();
        config.levels = 3;
        config.base_size = 1000;
        config.size_multiplier = 2.0;

        // Test proportional allocation
        config.allocation_strategy = MemoryAllocationStrategy::Proportional;
        let table1 = MultiLevelTranspositionTable::with_config(config.clone());

        // Level 0: 1000, Level 1: 2000, Level 2: 4000
        assert!(table1.level_configs[0].size <= table1.level_configs[1].size);
        assert!(table1.level_configs[1].size <= table1.level_configs[2].size);

        // Test equal allocation
        config.allocation_strategy = MemoryAllocationStrategy::Equal;
        let table2 = MultiLevelTranspositionTable::with_config(config);

        // All levels should have similar sizes
        assert_eq!(table2.level_configs[0].size, table2.level_configs[1].size);
        assert_eq!(table2.level_configs[1].size, table2.level_configs[2].size);
    }
}
