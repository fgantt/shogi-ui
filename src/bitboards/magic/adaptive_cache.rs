//! Adaptive caching system for magic bitboard lookups
//!
//! This module provides an LRU cache with adaptive sizing based on usage patterns.

use crate::types::Bitboard;
use std::cell::RefCell;
use std::collections::HashMap;

/// Adaptive LRU cache for magic bitboard lookups
#[derive(Clone)]
pub struct AdaptiveCache {
    /// Cache entries
    cache: RefCell<HashMap<CacheKey, CacheEntry>>,
    /// Maximum cache size
    max_size: usize,
    /// Current cache size
    current_size: RefCell<usize>,
    /// Access counter for LRU tracking
    access_counter: RefCell<u64>,
    /// Hit/miss tracking for adaptation
    hits: RefCell<u64>,
    misses: RefCell<u64>,
    /// Adaptive sizing enabled
    adaptive_sizing: bool,
}

/// Cache key combining square and occupied bitboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    square: u8,
    occupied_hash: u64, // Hash of occupied bitboard for faster lookups
}

/// Cache entry with LRU tracking
#[derive(Debug, Clone, Copy)]
struct CacheEntry {
    attacks: Bitboard,
    last_access: u64,
    access_count: u32,
}

impl AdaptiveCache {
    /// Create a new adaptive cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: RefCell::new(HashMap::with_capacity(max_size)),
            max_size,
            current_size: RefCell::new(0),
            access_counter: RefCell::new(0),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
            adaptive_sizing: true,
        }
    }

    /// Create with fixed size (no adaptation)
    pub fn with_fixed_size(size: usize) -> Self {
        let mut cache = Self::new(size);
        cache.adaptive_sizing = false;
        cache
    }

    /// Get attack pattern from cache
    pub fn get(&self, square: u8, occupied: Bitboard) -> Option<Bitboard> {
        let key = CacheKey {
            square,
            occupied_hash: Self::hash_occupied(occupied),
        };

        let mut cache = self.cache.borrow_mut();

        if let Some(entry) = cache.get_mut(&key) {
            // Update LRU tracking
            let mut counter = self.access_counter.borrow_mut();
            *counter += 1;
            entry.last_access = *counter;
            entry.access_count += 1;

            *self.hits.borrow_mut() += 1;
            Some(entry.attacks)
        } else {
            *self.misses.borrow_mut() += 1;
            None
        }
    }

    /// Insert attack pattern into cache
    pub fn insert(&self, square: u8, occupied: Bitboard, attacks: Bitboard) {
        let key = CacheKey {
            square,
            occupied_hash: Self::hash_occupied(occupied),
        };

        let mut cache = self.cache.borrow_mut();

        // Check if we need to evict
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            self.evict_lru(&mut cache);
        }

        let mut counter = self.access_counter.borrow_mut();
        *counter += 1;

        cache.insert(
            key,
            CacheEntry {
                attacks,
                last_access: *counter,
                access_count: 1,
            },
        );

        *self.current_size.borrow_mut() = cache.len();
    }

    /// Evict least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        if let Some((&key_to_remove, _)) = cache.iter().min_by_key(|(_, entry)| entry.last_access) {
            cache.remove(&key_to_remove);
        }
    }

    /// Hash occupied bitboard for faster comparison
    fn hash_occupied(occupied: Bitboard) -> u64 {
        // Simple hash - use lower 64 bits
        (occupied & 0xFFFFFFFFFFFFFFFF) as u64
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.borrow();
        let misses = *self.misses.borrow();

        if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: *self.current_size.borrow(),
            max_size: self.max_size,
            hits: *self.hits.borrow(),
            misses: *self.misses.borrow(),
            hit_rate: self.hit_rate(),
        }
    }

    /// Clear cache
    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
        *self.current_size.borrow_mut() = 0;
        *self.access_counter.borrow_mut() = 0;
    }

    /// Adapt cache size based on performance
    pub fn adapt_size(&mut self) {
        if !self.adaptive_sizing {
            return;
        }

        let hit_rate = self.hit_rate();
        let total_accesses = *self.hits.borrow() + *self.misses.borrow();

        // Only adapt if we have enough data
        if total_accesses < 1000 {
            return;
        }

        // Increase size if hit rate is low
        if hit_rate < 0.5 && self.max_size < 10_000 {
            self.max_size = (self.max_size * 2).min(10_000);
        }

        // Decrease size if hit rate is very high (cache is too large)
        if hit_rate > 0.95 && self.max_size > 256 {
            self.max_size = (self.max_size / 2).max(256);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_cache_creation() {
        let cache = AdaptiveCache::new(1024);
        let stats = cache.stats();

        assert_eq!(stats.max_size, 1024);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = AdaptiveCache::new(100);

        cache.insert(40, 0, 0xFFFFu128);

        let result = cache.get(40, 0);
        assert_eq!(result, Some(0xFFFFu128));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let cache = AdaptiveCache::new(100);

        let result = cache.get(40, 0);
        assert_eq!(result, None);

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = AdaptiveCache::new(2);

        // Insert 3 items (should evict LRU)
        cache.insert(0, 0, 0xA);
        cache.insert(1, 0, 0xB);
        cache.insert(2, 0, 0xC);

        let stats = cache.stats();
        assert_eq!(stats.size, 2); // Should have evicted one
    }

    #[test]
    fn test_lru_ordering() {
        let cache = AdaptiveCache::new(2);

        cache.insert(0, 0, 0xA);
        cache.insert(1, 0, 0xB);

        // Access first item to make it more recently used
        cache.get(0, 0);

        // Insert third item (should evict item 1, not item 0)
        cache.insert(2, 0, 0xC);

        // Item 0 should still be in cache
        assert_eq!(cache.get(0, 0), Some(0xA));

        // Item 1 should be evicted
        assert_eq!(cache.get(1, 0), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = AdaptiveCache::new(100);

        cache.insert(0, 0, 0xA);
        cache.insert(1, 0, 0xB);

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }
}
