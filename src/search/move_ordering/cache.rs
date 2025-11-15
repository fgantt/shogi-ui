//! Cache management for move ordering
//!
//! This module contains cache structures, eviction policies, and cache
//! management methods for the move ordering system.

use crate::types::core::Move;
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

/// Move ordering cache manager
///
/// Manages the move ordering result cache with support for multiple eviction policies.
/// Task 3.0: Enhanced with LRU, depth-preferred, and hybrid eviction policies.
#[derive(Debug, Clone)]
pub struct MoveOrderingCacheManager {
    /// Move ordering result cache
    /// Maps (position_hash, depth) -> cache entry with metadata
    cache: HashMap<(u64, u8), MoveOrderingCacheEntry>,
    /// LRU access counter (incremented on each cache access for LRU tracking)
    lru_access_counter: u64,
}

impl MoveOrderingCacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            lru_access_counter: 0,
        }
    }

    /// Get a cached entry for a key
    ///
    /// Returns a mutable reference to the cache entry if found, updating LRU tracking.
    pub fn get_mut(&mut self, key: &(u64, u8)) -> Option<&mut MoveOrderingCacheEntry> {
        if let Some(entry) = self.cache.get_mut(key) {
            self.lru_access_counter += 1;
            entry.last_access = self.lru_access_counter;
            entry.access_count += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Insert a new cache entry
    ///
    /// Inserts a new entry into the cache, evicting an entry if necessary.
    /// Returns the evicted key if an eviction occurred.
    /// Updates last_access to current lru_access_counter.
    pub fn insert(
        &mut self,
        key: (u64, u8),
        mut entry: MoveOrderingCacheEntry,
        max_size: usize,
        eviction_policy: CacheEvictionPolicy,
        hybrid_lru_weight: f32,
    ) -> Option<(u64, u8)> {
        // Update last_access to current counter
        self.lru_access_counter += 1;
        entry.last_access = self.lru_access_counter;

        // If cache has room, just insert
        if self.cache.len() < max_size {
            self.cache.insert(key, entry);
            return None;
        }

        // Cache is full - evict an entry
        let evicted_key = self.evict_entry(eviction_policy, hybrid_lru_weight);
        if let Some(evicted) = evicted_key {
            self.cache.remove(&evicted);
        }

        // Insert new entry
        self.cache.insert(key, entry);
        evicted_key
    }

    /// Evict an entry from the cache based on the eviction policy
    ///
    /// Returns the key of the entry to evict, or None if cache is empty.
    pub fn evict_entry(
        &self,
        eviction_policy: CacheEvictionPolicy,
        hybrid_lru_weight: f32,
    ) -> Option<(u64, u8)> {
        if self.cache.is_empty() {
            return None;
        }

        match eviction_policy {
            CacheEvictionPolicy::FIFO => {
                // FIFO: Remove first entry (oldest insertion)
                self.cache.keys().next().copied()
            }
            CacheEvictionPolicy::LRU => {
                // LRU: Remove least recently used entry (lowest last_access)
                let mut lru_key = None;
                let mut lru_access = u64::MAX;

                for (key, entry) in &self.cache {
                    if entry.last_access < lru_access {
                        lru_access = entry.last_access;
                        lru_key = Some(*key);
                    }
                }
                lru_key
            }
            CacheEvictionPolicy::DepthPreferred => {
                // Depth-preferred: Remove entry with lowest depth
                let mut min_depth_key = None;
                let mut min_depth = u8::MAX;

                for (key, entry) in &self.cache {
                    if entry.depth < min_depth {
                        min_depth = entry.depth;
                        min_depth_key = Some(*key);
                    }
                }
                min_depth_key
            }
            CacheEvictionPolicy::Hybrid => {
                // Hybrid: Combine LRU and depth-preferred
                let lru_weight = hybrid_lru_weight;
                let depth_weight = 1.0 - lru_weight;

                // Normalize LRU: older = lower score (more likely to evict)
                let max_access = self
                    .cache
                    .values()
                    .map(|e| e.last_access)
                    .max()
                    .unwrap_or(1);
                let min_access = self
                    .cache
                    .values()
                    .map(|e| e.last_access)
                    .min()
                    .unwrap_or(1);
                let access_range = (max_access - min_access).max(1) as f32;

                // Normalize depth: lower depth = lower score (more likely to evict)
                let max_depth = self
                    .cache
                    .values()
                    .map(|e| e.depth as f32)
                    .fold(0.0, f32::max);
                let min_depth = self
                    .cache
                    .values()
                    .map(|e| e.depth as f32)
                    .fold(u8::MAX as f32, f32::min);
                let depth_range = (max_depth - min_depth).max(1.0);

                let mut evict_key = None;
                let mut evict_score = f32::MAX;

                for (key, entry) in &self.cache {
                    // LRU score: normalized to 0.0 (oldest) .. 1.0 (newest)
                    let lru_score = if access_range > 0.0 {
                        1.0 - ((entry.last_access - min_access) as f32 / access_range)
                    } else {
                        0.5
                    };

                    // Depth score: normalized to 0.0 (shallowest) .. 1.0 (deepest)
                    let depth_score = if depth_range > 0.0 {
                        (entry.depth as f32 - min_depth) / depth_range
                    } else {
                        0.5
                    };

                    // Combined score: lower = more likely to evict
                    let combined_score =
                        depth_weight * (1.0 - depth_score) + lru_weight * lru_score;

                    if combined_score < evict_score {
                        evict_score = combined_score;
                        evict_key = Some(*key);
                    }
                }

                evict_key
            }
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_access_counter = 0;
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Check if cache is full
    pub fn is_full(&self, max_size: usize) -> bool {
        self.cache.len() >= max_size
    }

    /// Get LRU access counter
    pub fn get_lru_access_counter(&self) -> u64 {
        self.lru_access_counter
    }

    /// Increment LRU access counter
    pub fn increment_lru_counter(&mut self) -> u64 {
        self.lru_access_counter += 1;
        self.lru_access_counter
    }

    /// Get memory usage estimate
    pub fn memory_bytes(&self) -> usize {
        let mut total = 0;
        for (_key, entry) in &self.cache {
            total += std::mem::size_of::<(u64, u8)>(); // key
            total += std::mem::size_of::<MoveOrderingCacheEntry>(); // entry overhead
            total += entry.moves.len() * std::mem::size_of::<Move>(); // moves
        }
        total
    }

    /// Get cache entry (for testing)
    ///
    /// This method is primarily for testing purposes to verify cache contents.
    pub fn get(&self, key: &(u64, u8)) -> Option<&MoveOrderingCacheEntry> {
        self.cache.get(key)
    }

    /// Check if cache contains key (for testing)
    pub fn contains_key(&self, key: &(u64, u8)) -> bool {
        self.cache.contains_key(key)
    }
}

impl Default for MoveOrderingCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// Task 1.22: Extracted from mod.rs - Move score cache management

/// Helper struct for managing move score caches
///
/// Manages both the main move_score_cache (HashMap) and fast_score_cache (Vec)
/// for efficient move scoring lookups.
#[derive(Debug, Clone)]
pub struct MoveScoreCache {
    /// Main cache: maps move hash -> score
    cache: HashMap<u64, i32>,
    /// Fast cache (L1 cache simulation): small Vec for hot scores
    fast_cache: Vec<(u64, i32)>,
    /// Maximum size for main cache
    max_size: usize,
    /// Maximum size for fast cache (typically 64)
    fast_cache_max_size: usize,
}

impl MoveScoreCache {
    /// Create a new move score cache
    pub fn new(max_size: usize, fast_cache_max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            fast_cache: Vec::with_capacity(fast_cache_max_size),
            max_size,
            fast_cache_max_size,
        }
    }

    /// Get a cached score, checking fast cache first, then main cache
    ///
    /// Returns the cached score if found, None otherwise.
    pub fn get(&self, move_hash: u64) -> Option<i32> {
        // Check fast cache first (L1 cache simulation)
        for &(hash, score) in &self.fast_cache {
            if hash == move_hash {
                return Some(score);
            }
        }

        // Check main cache (L2 cache simulation)
        self.cache.get(&move_hash).copied()
    }

    /// Insert a score into the cache
    ///
    /// If the cache is full, the oldest entry in fast_cache is evicted.
    /// The new entry is added to both fast_cache (if there's room) and main cache.
    pub fn insert(&mut self, move_hash: u64, score: i32) {
        // Add to fast cache if there's room
        if self.fast_cache.len() < self.fast_cache_max_size {
            self.fast_cache.push((move_hash, score));
        }

        // Add to main cache, evicting if necessary
        if self.cache.len() < self.max_size {
            self.cache.insert(move_hash, score);
        } else {
            // Cache is full - simple eviction: remove first entry
            if let Some(&key) = self.cache.keys().next() {
                self.cache.remove(&key);
            }
            self.cache.insert(move_hash, score);
        }
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.cache.clear();
        self.fast_cache.clear();
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.cache.len() >= self.max_size
    }

    /// Set maximum cache size and trim if necessary
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;

        // Trim cache if necessary
        if self.cache.len() > max_size {
            let excess = self.cache.len() - max_size;
            let keys_to_remove: Vec<u64> =
                self.cache.keys().take(excess).copied().collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    /// Get maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get memory usage estimate in bytes
    pub fn memory_bytes(&self) -> usize {
        let cache_bytes = self.cache.len()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<i32>());
        let fast_cache_bytes = self.fast_cache.len()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<i32>());
        cache_bytes + fast_cache_bytes
    }
}

impl Default for MoveScoreCache {
    fn default() -> Self {
        Self::new(10000, 64)
    }
}
