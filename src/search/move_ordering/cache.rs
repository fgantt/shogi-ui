//! Cache management for move ordering
//! 
//! This module contains cache structures, eviction policies, and cache
//! management methods for the move ordering system.

use crate::types::*;
use serde::Serialize;
use std::collections::HashMap;

/// Cache eviction policy for move ordering cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CacheEvictionPolicy {
    /// First-In-First-Out: Remove oldest entries first
    FIFO,
    /// Least Recently Used: Remove least recently accessed entries first
    LRU,
    /// Depth-Preferred: Remove entries with lower search depth first
    DepthPreferred,
    /// Hybrid: Combine LRU and depth-based eviction
    Hybrid,
}

/// Cache entry for move ordering cache
/// Task 3.0: Stores move ordering results with metadata for eviction
#[derive(Debug, Clone)]
pub struct MoveOrderingCacheEntry {
    /// Cached move ordering result
    pub moves: Vec<Move>,
    /// Last access timestamp (for LRU tracking)
    pub last_access: u64,
    /// Search depth at which this entry was created
    pub depth: u8,
    /// Number of times this entry was accessed
    pub access_count: u64,
}

/// Cache configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable cache warming
    pub enable_cache_warming: bool,
    /// Cache warming ratio (percentage of max cache size to warm)
    pub cache_warming_ratio: f32,
    /// Enable automatic cache optimization
    pub enable_auto_optimization: bool,
    /// Hit rate threshold for optimization
    pub optimization_hit_rate_threshold: f64,
    /// Maximum SEE cache size
    pub max_see_cache_size: usize,
    /// Enable SEE cache
    pub enable_see_cache: bool,
    /// Cache eviction policy (Task 3.0)
    pub cache_eviction_policy: CacheEvictionPolicy,
    /// LRU access counter (incremented on each access) (Task 3.0)
    pub lru_access_counter: u64,
    /// Hybrid LRU weight (0.0 to 1.0) for hybrid eviction policy (Task 3.0)
    pub hybrid_lru_weight: f32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            enable_cache_warming: true,
            cache_warming_ratio: 0.5,
            enable_auto_optimization: true,
            optimization_hit_rate_threshold: 70.0,
            max_see_cache_size: 5000,
            enable_see_cache: true,
            cache_eviction_policy: CacheEvictionPolicy::LRU,
            lru_access_counter: 0,
            hybrid_lru_weight: 0.7,
        }
    }
}

// TODO: Extract cache management methods from move_ordering.rs:
// - Cache eviction logic (FIFO, LRU, DepthPreferred, Hybrid)
// - Cache warming methods
// - Cache optimization methods
// - Cache statistics tracking
// - Cache clearing/pruning methods

