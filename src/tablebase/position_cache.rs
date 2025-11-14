//! Position caching system for tablebase results
//!
//! This module provides efficient caching of tablebase results to avoid
//! repeated calculations for the same positions.

use super::tablebase_config::EvictionStrategy;
use super::TablebaseResult;
use crate::search::BoardTrait;
use crate::time_utils::TimeSource;
use crate::types::Player;
use crate::BitboardBoard;
use crate::CapturedPieces;
use std::collections::HashMap;

/// A cache entry that includes the result and access information
#[derive(Debug, Clone)]
struct CacheEntry {
    result: TablebaseResult,
    last_accessed: u64,
    access_count: u64,
    creation_time: u64,
    position_signature: u64,
    player: Player,
}

/// Configuration for the position cache
#[derive(Debug, Clone, PartialEq)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_size: usize,
    /// Eviction strategy to use
    pub eviction_strategy: EvictionStrategy,
    /// Whether to enable adaptive eviction based on access patterns
    pub enable_adaptive_eviction: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            eviction_strategy: EvictionStrategy::LRU,
            enable_adaptive_eviction: false,
        }
    }
}

/// Cache for storing tablebase results
///
/// This cache stores the results of tablebase probes to avoid
/// repeated calculations for the same positions.
pub struct PositionCache {
    /// The actual cache storage
    cache: HashMap<u64, CacheEntry>,
    /// Maximum number of entries in the cache
    max_size: usize,
    /// Number of cache hits
    hits: u64,
    /// Number of cache misses
    misses: u64,
    /// Eviction strategy to use
    eviction_strategy: EvictionStrategy,
    /// Whether to enable adaptive eviction
    enable_adaptive_eviction: bool,
    /// Number of detected cache key collisions
    collision_count: u64,
}

impl PositionCache {
    /// Create a new position cache with default size
    pub fn new() -> Self {
        Self::with_size(10000)
    }

    /// Create a new position cache with specified size
    pub fn with_size(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
            eviction_strategy: EvictionStrategy::LRU,
            enable_adaptive_eviction: false,
            collision_count: 0,
        }
    }

    /// Create a new position cache with configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            max_size: config.max_size,
            hits: 0,
            misses: 0,
            eviction_strategy: config.eviction_strategy,
            enable_adaptive_eviction: config.enable_adaptive_eviction,
            collision_count: 0,
        }
    }

    /// Get a cached result for a position
    ///
    /// # Arguments
    /// * `board` - The board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces
    ///
    /// # Returns
    /// `Some(TablebaseResult)` if found in cache, `None` otherwise
    pub fn get(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TablebaseResult> {
        let key = self.generate_key(board, player, captured_pieces);
        let timestamp = self.current_timestamp();
        if let Some(entry) = self.cache.get_mut(&key) {
            // Update access information
            entry.last_accessed = timestamp;
            entry.access_count += 1;
            let signature = board.get_position_hash(captured_pieces);
            if entry.position_signature == signature && entry.player == player {
                self.hits += 1;
                Some(entry.result.clone())
            } else {
                self.collision_count += 1;
                self.misses += 1;
                None
            }
        } else {
            self.misses += 1;
            None
        }
    }

    /// Store a result in the cache
    ///
    /// # Arguments
    /// * `board` - The board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces
    /// * `result` - The tablebase result to cache
    pub fn put(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        result: TablebaseResult,
    ) {
        let key = self.generate_key(board, player, captured_pieces);
        let signature = board.get_position_hash(captured_pieces);

        // Check if we need to evict entries
        if self.cache.len() >= self.max_size && !self.cache.contains_key(&key) {
            self.evict_entry();
        }

        let timestamp = self.current_timestamp();
        let entry = CacheEntry {
            result,
            last_accessed: timestamp,
            access_count: 0,
            creation_time: timestamp,
            position_signature: signature,
            player,
        };

        self.cache.insert(key, entry);
    }

    /// Set the maximum cache size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        // Evict entries if we're over the new limit
        while self.cache.len() > self.max_size {
            self.evict_lru();
        }
    }

    /// Get cache configuration
    pub fn get_config(&self) -> CacheConfig {
        CacheConfig {
            max_size: self.max_size,
            eviction_strategy: self.eviction_strategy,
            enable_adaptive_eviction: self.enable_adaptive_eviction,
        }
    }

    /// Get the cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get the number of entries in the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Get the maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
        self.collision_count = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> (u64, u64, f64) {
        (self.hits, self.misses, self.hit_rate())
    }

    /// Number of detected cache key collisions
    pub fn collision_count(&self) -> u64 {
        self.collision_count
    }

    /// Generate a hash key for a position
    ///
    /// This method creates a unique hash key for a position based on
    /// the board state, player to move, and captured pieces.
    /// Optimized for speed using bitboard operations.
    /// Generate a stable cache key using the board's Zobrist hash.
    ///
    /// Tablebase results depend only on the material arrangement and side to move,
    /// not on repetition counters or move history. We therefore combine the board's
    /// position hash (which already incorporates captured pieces) with the player
    /// to move and rely on the Zobrist implementation for uniqueness.
    fn generate_key(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        let mut hash = board.get_position_hash(captured_pieces);
        if player == Player::White {
            hash ^= 0x9E37_79B1_85EB_CA87;
        }
        hash
    }

    /// Get current timestamp for LRU tracking
    fn current_timestamp(&self) -> u64 {
        TimeSource::now().elapsed_ms() as u64
    }

    /// Evict an entry from the cache based on the configured strategy
    fn evict_entry(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        let key_to_remove = match self.eviction_strategy {
            EvictionStrategy::Random => self.evict_random(),
            EvictionStrategy::LRU => self.evict_lru(),
            EvictionStrategy::LFU => self.evict_lfu(),
        };

        if let Some(key) = key_to_remove {
            self.cache.remove(&key);
        }
    }

    /// Evict a random entry (fastest)
    fn evict_random(&self) -> Option<u64> {
        self.cache.keys().next().copied()
    }

    /// Evict the least recently used entry
    fn evict_lru(&self) -> Option<u64> {
        self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| *key)
    }

    /// Evict the least frequently used entry
    fn evict_lfu(&self) -> Option<u64> {
        self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| *key)
    }

    /// Evict multiple entries using adaptive strategy
    fn evict_adaptive(&mut self, count: usize) {
        if self.cache.is_empty() {
            return;
        }

        let mut entries: Vec<(u64, u64, u64, u64)> = self
            .cache
            .iter()
            .map(|(key, entry)| {
                (
                    *key,
                    entry.last_accessed,
                    entry.access_count,
                    entry.creation_time,
                )
            })
            .collect();

        // Sort by adaptive score (combination of recency and frequency)
        entries.sort_by(|a, b| {
            let score_a = self.calculate_adaptive_score(a.1, a.2, a.3);
            let score_b = self.calculate_adaptive_score(b.1, b.2, b.3);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove the lowest scoring entries
        for (key, _, _, _) in entries.iter().take(count) {
            self.cache.remove(key);
        }
    }

    /// Calculate adaptive score for eviction (lower is better for eviction)
    fn calculate_adaptive_score(
        &self,
        last_accessed: u64,
        access_count: u64,
        creation_time: u64,
    ) -> f64 {
        let current_time = self.current_timestamp();
        let age = current_time - creation_time;
        let recency = current_time - last_accessed;

        // Weight factors
        let recency_weight = 0.6;
        let frequency_weight = 0.3;
        let age_weight = 0.1;

        // Normalize and calculate score
        let recency_score = recency as f64 / (age + 1) as f64;
        let frequency_score = 1.0 / (access_count + 1) as f64;
        let age_score = age as f64 / 1000.0; // Normalize age

        recency_weight * recency_score + frequency_weight * frequency_score + age_weight * age_score
    }

    /// Estimate memory usage in bytes
    pub fn estimate_memory_usage(&self) -> usize {
        // Estimate memory usage based on cache size and entry size
        let entry_size = std::mem::size_of::<CacheEntry>()
            + std::mem::size_of::<u64>()
            + std::mem::size_of::<TablebaseResult>();
        let cache_memory = self.cache.len() * entry_size;
        let overhead = std::mem::size_of::<PositionCache>();

        cache_memory + overhead
    }

    /// Clear half of the cache entries (for emergency eviction)
    pub fn clear_half(&mut self) {
        let target_size = self.cache.len() / 2;

        if self.enable_adaptive_eviction {
            self.evict_adaptive(target_size);
        } else {
            // Use simple LRU for emergency eviction
            let mut entries_to_remove: Vec<u64> = Vec::new();

            // Collect entries to remove (oldest first)
            let mut entries: Vec<(u64, u64)> = self
                .cache
                .iter()
                .map(|(key, entry)| (*key, entry.last_accessed))
                .collect();

            entries.sort_by_key(|(_, timestamp)| *timestamp);

            for (key, _) in entries.iter().take(target_size) {
                entries_to_remove.push(*key);
            }

            // Remove the selected entries
            for key in entries_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Get cache utilization percentage
    pub fn utilization_percentage(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.cache.len() as f64 / self.max_size as f64) * 100.0
        }
    }

    /// Set the eviction strategy
    pub fn set_eviction_strategy(&mut self, strategy: EvictionStrategy) {
        self.eviction_strategy = strategy;
    }

    /// Enable or disable adaptive eviction
    pub fn set_adaptive_eviction(&mut self, enabled: bool) {
        self.enable_adaptive_eviction = enabled;
    }

    /// Get current eviction strategy
    pub fn get_eviction_strategy(&self) -> EvictionStrategy {
        self.eviction_strategy
    }

    /// Check if adaptive eviction is enabled
    pub fn is_adaptive_eviction_enabled(&self) -> bool {
        self.enable_adaptive_eviction
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> (u64, u64, f64) {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            self.hits as f64 / total_requests as f64
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }

    /// Reset cache statistics
    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    /// Clear all cache entries
    pub fn clear_all(&mut self) {
        self.cache.clear();
    }
}

impl Default for PositionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, PieceType, Player, Position};
    use crate::BitboardBoard;
    use crate::CapturedPieces;

    #[test]
    fn test_position_cache_creation() {
        let cache = PositionCache::new();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.max_size(), 10000);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_position_cache_with_size() {
        let cache = PositionCache::with_size(100);
        assert_eq!(cache.max_size(), 100);
    }

    #[test]
    fn test_position_cache_put_and_get() {
        let mut cache = PositionCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let move_ = Move::new_move(
            Position::new(0, 0),
            Position::new(1, 1),
            PieceType::King,
            Player::Black,
            false,
        );

        let result = TablebaseResult::win(Some(move_), 5);

        // Initially, cache should be empty
        assert!(cache.get(&board, player, &captured_pieces).is_none());

        // Put result in cache
        cache.put(&board, player, &captured_pieces, result);

        // Now should be able to get it back
        let cached_result = cache.get(&board, player, &captured_pieces);
        assert!(cached_result.is_some());
        assert_eq!(cached_result.unwrap().moves_to_mate, Some(5));
    }

    #[test]
    fn test_position_cache_hit_miss_tracking() {
        let mut cache = PositionCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Record some hits and misses by using get operations
        let result = TablebaseResult::draw();
        cache.put(&board, player, &captured_pieces, result.clone());
        let _ = cache.get(&board, player, &captured_pieces); // Hit
        let _ = cache.get(&board, player, &captured_pieces); // Hit
        let _ = cache.get(&board, Player::White, &captured_pieces); // Miss

        assert_eq!(cache.hits, 2);
        assert_eq!(cache.misses, 1);
        assert_eq!(cache.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_position_cache_clear() {
        let mut cache = PositionCache::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let result = TablebaseResult::draw();
        cache.put(&board, player, &captured_pieces, result.clone());
        let _ = cache.get(&board, player, &captured_pieces); // Hit
        let _ = cache.get(&board, Player::White, &captured_pieces); // Miss

        assert_eq!(cache.size(), 1);
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 1);

        cache.clear();

        assert_eq!(cache.size(), 0);
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
    }

    #[test]
    fn test_position_cache_stats() {
        let mut cache = PositionCache::new();
        // Simulate hits and misses
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;
        let result = TablebaseResult::draw();
        cache.put(&board, player, &captured_pieces, result.clone());
        let _ = cache.get(&board, player, &captured_pieces); // Hit
        let _ = cache.get(&board, player, &captured_pieces); // Hit
        let _ = cache.get(&board, Player::White, &captured_pieces); // Miss

        let (hits, misses, hit_rate) = cache.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 2.0 / 3.0);
    }

    #[test]
    fn test_position_cache_key_generation() {
        let cache = PositionCache::new();
        let board1 = BitboardBoard::new();
        let board2 = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Same position should generate same key
        let key1 = cache.generate_key(&board1, player, &captured_pieces);
        let key2 = cache.generate_key(&board2, player, &captured_pieces);
        assert_eq!(key1, key2);

        // Different player should generate different key
        let key3 = cache.generate_key(&board1, Player::White, &captured_pieces);
        assert_ne!(key1, key3);
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use crate::types::{Piece, PieceType, Position};

    /// Benchmark cache performance with various operations
    #[test]
    fn benchmark_cache_performance() {
        let mut cache = PositionCache::with_size(1000);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Create a test result
        let test_result = TablebaseResult::win(
            Some(crate::types::Move::new_move(
                Position::new(0, 0),
                Position::new(1, 1),
                PieceType::King,
                Player::Black,
                false,
            )),
            5,
        );

        // Benchmark put operations
        let start = TimeSource::now();
        for i in 0..1000 {
            let test_board = board.clone();
            // For now, just use the same board for all iterations
            // TODO: Implement proper board modification when set_piece is available
            cache.put(&test_board, player, &captured_pieces, test_result.clone());
        }
        let put_duration = std::time::Duration::from_millis(start.elapsed_ms() as u64);

        // Benchmark get operations
        let start = TimeSource::now();
        for i in 0..1000 {
            let test_board = board.clone();
            // For now, just use the same board for all iterations
            // TODO: Implement proper board modification when set_piece is available
            let _ = cache.get(&test_board, player, &captured_pieces);
        }
        let get_duration = std::time::Duration::from_millis(start.elapsed_ms() as u64);

        // Print benchmark results
        println!("Cache Performance Benchmarks:");
        println!("  Put 1000 entries: {:?}", put_duration);
        println!("  Get 1000 entries: {:?}", get_duration);
        println!("  Cache hit rate: {:.2}%", cache.hit_rate() * 100.0);
        println!("  Cache size: {}", cache.size());

        // Verify reasonable performance (should complete in under 1 second)
        assert!(put_duration.as_millis() < 1000);
        assert!(get_duration.as_millis() < 1000);
    }

    /// Benchmark cache eviction performance
    #[test]
    fn benchmark_cache_eviction() {
        let mut cache = PositionCache::with_size(100); // Small cache to force evictions
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let test_result = TablebaseResult::win(
            Some(crate::types::Move::new_move(
                Position::new(0, 0),
                Position::new(1, 1),
                PieceType::King,
                Player::Black,
                false,
            )),
            5,
        );

        // Fill cache beyond capacity to trigger evictions
        let start = TimeSource::now();
        for i in 0..200 {
            let test_board = board.clone();
            // For now, just use the same board for all iterations
            // TODO: Implement proper board modification when set_piece is available
            cache.put(&test_board, player, &captured_pieces, test_result.clone());
        }
        let eviction_duration = std::time::Duration::from_millis(start.elapsed_ms() as u64);

        println!("Cache Eviction Benchmark:");
        println!("  Fill 200 entries (100 capacity): {:?}", eviction_duration);
        println!("  Final cache size: {}", cache.size());

        // Verify cache doesn't exceed capacity
        assert!(cache.size() <= 100);
        assert!(eviction_duration.as_millis() < 1000);
    }
}
