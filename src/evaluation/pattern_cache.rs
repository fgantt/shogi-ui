//! Pattern Result Caching Module
//!
//! This module provides caching for pattern recognition results to improve performance.
//! Features:
//! - Pattern result caching with position hashing
//! - Incremental pattern updates
//! - Cache invalidation strategies
//! - Cache statistics and monitoring
//! - Cache size management with LRU eviction
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::pattern_cache::PatternCache;
//!
//! let mut cache = PatternCache::new(10000);
//! cache.store(hash, pattern_result);
//! if let Some(result) = cache.lookup(hash) {
//!     // Use cached result
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cached pattern recognition result
#[derive(Debug, Clone, Copy)]
pub struct CachedPatternResult {
    /// Tactical pattern score
    pub tactical_score: (i32, i32), // (mg, eg)

    /// Positional pattern score
    pub positional_score: (i32, i32),

    /// Endgame pattern score
    pub endgame_score: (i32, i32),

    /// Age of entry (for LRU eviction)
    pub age: u64,
}

/// Pattern result cache with LRU eviction
pub struct PatternCache {
    /// Cache storage
    cache: HashMap<u64, CachedPatternResult>,

    /// Maximum cache size
    max_size: usize,

    /// Current age counter (for LRU)
    age_counter: u64,

    /// Cache statistics
    stats: PatternCacheStats,

    /// Configuration
    config: PatternCacheConfig,
}

impl PatternCache {
    /// Create a new pattern cache with specified size
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size,
            age_counter: 0,
            stats: PatternCacheStats::default(),
            config: PatternCacheConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PatternCacheConfig) -> Self {
        Self {
            cache: HashMap::with_capacity(config.max_size),
            max_size: config.max_size,
            age_counter: 0,
            stats: PatternCacheStats::default(),
            config,
        }
    }

    /// Store pattern result in cache
    pub fn store(&mut self, position_hash: u64, result: CachedPatternResult) {
        self.stats.stores += 1;

        // Check if we need to evict entries
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        // Store with current age
        let mut result_with_age = result;
        result_with_age.age = self.age_counter;
        self.age_counter += 1;

        self.cache.insert(position_hash, result_with_age);
    }

    /// Lookup pattern result from cache
    pub fn lookup(&mut self, position_hash: u64) -> Option<CachedPatternResult> {
        self.stats.lookups += 1;

        if let Some(mut result) = self.cache.get(&position_hash).copied() {
            self.stats.hits += 1;

            // Update age for LRU (touch entry)
            if self.config.enable_lru_touch {
                result.age = self.age_counter;
                self.age_counter += 1;
                self.cache.insert(position_hash, result);
            }

            Some(result)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Invalidate cache entry
    pub fn invalidate(&mut self, position_hash: u64) {
        if self.cache.remove(&position_hash).is_some() {
            self.stats.invalidations += 1;
        }
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        let entries_cleared = self.cache.len();
        self.cache.clear();
        self.stats.clears += 1;
        self.stats.total_cleared += entries_cleared as u64;
    }

    /// Evict oldest entry (LRU)
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find entry with minimum age
        let mut oldest_hash = 0;
        let mut oldest_age = u64::MAX;

        for (&hash, entry) in &self.cache {
            if entry.age < oldest_age {
                oldest_age = entry.age;
                oldest_hash = hash;
            }
        }

        self.cache.remove(&oldest_hash);
        self.stats.evictions += 1;
    }

    /// Get cache statistics
    pub fn stats(&self) -> &PatternCacheStats {
        &self.stats
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.stats.lookups == 0 {
            0.0
        } else {
            self.stats.hits as f64 / self.stats.lookups as f64
        }
    }

    /// Get cache usage (percentage)
    pub fn usage_percent(&self) -> f64 {
        (self.cache.len() as f64 / self.max_size as f64) * 100.0
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Get maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Resize cache
    pub fn resize(&mut self, new_size: usize) {
        self.max_size = new_size;

        // Evict entries if needed
        while self.cache.len() > self.max_size {
            self.evict_lru();
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PatternCacheStats::default();
    }
}

impl Default for PatternCache {
    fn default() -> Self {
        Self::new(100_000)
    }
}

/// Pattern cache configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternCacheConfig {
    /// Maximum cache size
    pub max_size: usize,

    /// Enable LRU touch on lookup
    pub enable_lru_touch: bool,

    /// Enable incremental updates
    pub enable_incremental_updates: bool,

    /// Auto-clear threshold (clear when usage > threshold)
    pub auto_clear_threshold: f64,
}

impl Default for PatternCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100_000,
            enable_lru_touch: true,
            enable_incremental_updates: true,
            auto_clear_threshold: 0.95, // Clear at 95% full
        }
    }
}

/// Pattern cache statistics
#[derive(Debug, Clone, Default)]
pub struct PatternCacheStats {
    /// Number of lookups
    pub lookups: u64,

    /// Number of hits
    pub hits: u64,

    /// Number of misses
    pub misses: u64,

    /// Number of stores
    pub stores: u64,

    /// Number of evictions
    pub evictions: u64,

    /// Number of invalidations
    pub invalidations: u64,

    /// Number of clears
    pub clears: u64,

    /// Total entries cleared
    pub total_cleared: u64,
}

impl PatternCacheStats {
    /// Get hit rate percentage
    pub fn hit_rate_percent(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            (self.hits as f64 / self.lookups as f64) * 100.0
        }
    }

    /// Get miss rate percentage
    pub fn miss_rate_percent(&self) -> f64 {
        100.0 - self.hit_rate_percent()
    }
}

/// Incremental pattern update tracker
pub struct IncrementalPatternTracker {
    /// Last position hash
    last_position_hash: Option<u64>,

    /// Last pattern results
    last_results: Option<CachedPatternResult>,

    /// Enable incremental updates
    enabled: bool,
}

impl IncrementalPatternTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            last_position_hash: None,
            last_results: None,
            enabled: true,
        }
    }

    /// Check if can do incremental update
    pub fn can_update_incrementally(&self, _current_hash: u64) -> bool {
        self.enabled && self.last_position_hash.is_some()
    }

    /// Update with new position
    pub fn update(&mut self, hash: u64, result: CachedPatternResult) {
        self.last_position_hash = Some(hash);
        self.last_results = Some(result);
    }

    /// Get last results
    pub fn last_results(&self) -> Option<CachedPatternResult> {
        self.last_results
    }

    /// Clear tracker
    pub fn clear(&mut self) {
        self.last_position_hash = None;
        self.last_results = None;
    }

    /// Enable/disable incremental updates
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for IncrementalPatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_cache_creation() {
        let cache = PatternCache::new(1000);
        assert_eq!(cache.max_size(), 1000);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_store_and_lookup() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(12345, result);

        let retrieved = cache.lookup(12345);
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.tactical_score, (50, 30));
        assert_eq!(retrieved.positional_score, (40, 25));
    }

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(12345, result);

        // Hit
        let _ = cache.lookup(12345);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 0);

        // Miss
        let _ = cache.lookup(99999);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(1, result);
        cache.store(2, result);

        let _ = cache.lookup(1); // Hit
        let _ = cache.lookup(2); // Hit
        let _ = cache.lookup(3); // Miss
        let _ = cache.lookup(4); // Miss

        assert_eq!(cache.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = PatternCache::new(3);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(1, result);
        cache.store(2, result);
        cache.store(3, result);

        assert_eq!(cache.size(), 3);

        // This should trigger eviction
        cache.store(4, result);

        assert_eq!(cache.size(), 3);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(1, result);
        cache.store(2, result);
        cache.store(3, result);

        assert_eq!(cache.size(), 3);

        cache.clear();

        assert_eq!(cache.size(), 0);
        assert_eq!(cache.stats().clears, 1);
        assert_eq!(cache.stats().total_cleared, 3);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(12345, result);
        assert_eq!(cache.size(), 1);

        cache.invalidate(12345);
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.stats().invalidations, 1);
    }

    #[test]
    fn test_cache_resize() {
        let mut cache = PatternCache::new(5);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        for i in 0..5 {
            cache.store(i, result);
        }

        assert_eq!(cache.size(), 5);

        // Resize to smaller
        cache.resize(3);
        assert_eq!(cache.max_size(), 3);
        assert_eq!(cache.size(), 3);
    }

    #[test]
    fn test_cache_usage_percent() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        for i in 0..50 {
            cache.store(i, result);
        }

        assert_eq!(cache.usage_percent(), 50.0);
    }

    #[test]
    fn test_incremental_tracker() {
        let mut tracker = IncrementalPatternTracker::new();

        assert!(!tracker.can_update_incrementally(123));

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        tracker.update(123, result);
        assert!(tracker.can_update_incrementally(456));

        let last = tracker.last_results();
        assert!(last.is_some());
    }

    #[test]
    fn test_incremental_tracker_clear() {
        let mut tracker = IncrementalPatternTracker::new();

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        tracker.update(123, result);
        tracker.clear();

        assert!(!tracker.can_update_incrementally(456));
    }

    #[test]
    fn test_stats_hit_rate_percent() {
        let mut stats = PatternCacheStats::default();

        stats.lookups = 10;
        stats.hits = 7;
        stats.misses = 3;

        assert_eq!(stats.hit_rate_percent(), 70.0);
        assert_eq!(stats.miss_rate_percent(), 30.0);
    }

    #[test]
    fn test_lru_eviction_order() {
        let mut cache = PatternCache::new(3);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(1, result);
        cache.store(2, result);
        cache.store(3, result);

        // Access entry 1 (update its age)
        let _ = cache.lookup(1);

        // Store new entry (should evict 2, the oldest untouched)
        cache.store(4, result);

        // Entry 1 should still be there (was recently accessed)
        assert!(cache.lookup(1).is_some());

        // Entry 2 might be evicted
        assert_eq!(cache.size(), 3);
    }

    #[test]
    fn test_cache_config_validation() {
        let config = PatternCacheConfig::default();
        assert_eq!(config.max_size, 100_000);
        assert!(config.enable_lru_touch);
        assert!(config.enable_incremental_updates);
    }

    #[test]
    fn test_reset_statistics() {
        let mut cache = PatternCache::new(100);

        let result = CachedPatternResult {
            tactical_score: (50, 30),
            positional_score: (40, 25),
            endgame_score: (20, 35),
            age: 0,
        };

        cache.store(1, result);
        let _ = cache.lookup(1);

        assert!(cache.stats().stores > 0);
        assert!(cache.stats().lookups > 0);

        cache.reset_stats();

        assert_eq!(cache.stats().stores, 0);
        assert_eq!(cache.stats().lookups, 0);
    }
}
