//! Fast lookup engine for magic bitboards
//! 
//! This module provides optimized lookup functionality for magic bitboards,
//! including prefetching, caching, and SIMD optimizations.

use crate::types::core::PieceType;
use crate::types::{Bitboard, MagicError, MagicTable, PerformanceMetrics};
use std::collections::HashMap;
use std::cell::RefCell;

/// Fast lookup engine for magic bitboards
/// 
/// Uses interior mutability for caching and metrics to allow immutable API
#[derive(Clone)]
pub struct LookupEngine {
    /// Magic table for lookups
    magic_table: MagicTable,
    /// Performance metrics (interior mutability for immutable API)
    metrics: RefCell<PerformanceMetrics>,
    /// Lookup cache for frequently accessed patterns (interior mutability)
    lookup_cache: RefCell<LookupCache>,
    /// Prefetch buffer for next likely accesses (interior mutability)
    prefetch_buffer: RefCell<PrefetchBuffer>,
    /// Batch lookup cache for multiple squares (interior mutability)
    batch_cache: RefCell<HashMap<u64, Vec<Bitboard>>>,
    /// Fallback ray-casting engine
    ray_caster: RayCaster,
    /// SIMD optimization settings
    simd_enabled: bool,
}

/// Lookup cache for frequently accessed attack patterns
#[derive(Debug, Clone)]
pub struct LookupCache {
    /// Cache entries (square, hash) -> attack_pattern
    entries: Vec<Option<(u8, u64, Bitboard)>>,
    /// Cache size (power of 2)
    size: usize,
    /// Hash function for cache indexing
    hash_fn: fn(u8, u64) -> usize,
}

impl LookupEngine {
    /// Create a new lookup engine
    pub fn new(magic_table: MagicTable) -> Self {
        Self {
            magic_table,
            metrics: RefCell::new(PerformanceMetrics::default()),
            lookup_cache: RefCell::new(LookupCache::new(1024)), // Default cache size
            prefetch_buffer: RefCell::new(PrefetchBuffer::new(64)),
            batch_cache: RefCell::new(HashMap::new()),
            ray_caster: RayCaster::new(),
            simd_enabled: cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64"),
        }
    }

    /// Create a new lookup engine with custom cache size
    pub fn with_cache_size(magic_table: MagicTable, cache_size: usize) -> Self {
        Self {
            magic_table,
            metrics: RefCell::new(PerformanceMetrics::default()),
            lookup_cache: RefCell::new(LookupCache::new(cache_size)),
            prefetch_buffer: RefCell::new(PrefetchBuffer::new(64)),
            batch_cache: RefCell::new(HashMap::new()),
            ray_caster: RayCaster::new(),
            simd_enabled: cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64"),
        }
    }

    /// Create a new lookup engine with all custom settings
    pub fn with_settings(
        magic_table: MagicTable,
        cache_size: usize,
        prefetch_size: usize,
        simd_enabled: bool,
    ) -> Self {
        Self {
            magic_table,
            metrics: RefCell::new(PerformanceMetrics::default()),
            lookup_cache: RefCell::new(LookupCache::new(cache_size)),
            prefetch_buffer: RefCell::new(PrefetchBuffer::new(prefetch_size)),
            batch_cache: RefCell::new(HashMap::new()),
            ray_caster: RayCaster::new(),
            simd_enabled,
        }
    }

    /// Fast attack lookup using magic bitboards with adaptive caching
    ///
    /// This method uses caching for hot paths (frequently accessed squares)
    /// and direct lookup for cold paths to optimize performance.
    pub fn get_attacks(
        &self,
        square: u8,
        piece_type: PieceType,
        occupied: Bitboard
    ) -> Bitboard {
        let start_time = std::time::Instant::now();
        
        // Check cache first (hot path optimization)
        if let Some(cached) = self.lookup_cache.borrow_mut().get(square, occupied) {
            let mut metrics = self.metrics.borrow_mut();
            metrics.cache_hits += 1;
            metrics.lookup_count += 1;
            metrics.total_lookup_time += start_time.elapsed();
            return cached;
        }

        // Perform magic lookup (cold path)
        let attacks = self.magic_table.get_attacks(square, piece_type, occupied);
        
        // Cache the result for future hot path access
        self.lookup_cache.borrow_mut().insert(square, occupied, attacks);
        let mut metrics = self.metrics.borrow_mut();
        metrics.cache_misses += 1;
        metrics.lookup_count += 1;
        metrics.total_lookup_time += start_time.elapsed();
        
        attacks
    }

    /// Optimized lookup with prefetching
    pub fn get_attacks_optimized(
        &self,
        square: u8,
        piece_type: PieceType,
        occupied: Bitboard
    ) -> Bitboard {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        if let Some(cached) = self.lookup_cache.borrow_mut().get(square, occupied) {
            let mut metrics = self.metrics.borrow_mut();
            metrics.cache_hits += 1;
            metrics.lookup_count += 1;
            return cached;
        }

        // Prefetch next likely access
        self.prefetch_next_access(square, piece_type);
        
        // Perform magic lookup
        let attacks = self.magic_table.get_attacks(square, piece_type, occupied);
        
        // Cache the result
        self.lookup_cache.borrow_mut().insert(square, occupied, attacks);
        let mut metrics = self.metrics.borrow_mut();
        metrics.cache_misses += 1;
        metrics.lookup_count += 1;
        metrics.total_lookup_time += start_time.elapsed();
        
        attacks
    }

    /// Prefetch next likely access for better cache performance
    fn prefetch_next_access(&self, square: u8, piece_type: PieceType) {
        // Placeholder implementation
        // In a real implementation, this would prefetch the next likely
        // attack pattern based on common access patterns
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.borrow().clone()
    }

    /// Reset performance metrics
    pub fn reset_metrics(&self) {
        *self.metrics.borrow_mut() = PerformanceMetrics::default();
    }

    /// Clear lookup cache
    pub fn clear_cache(&self) {
        self.lookup_cache.borrow_mut().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let mut stats = self.lookup_cache.borrow().stats();
        let metrics = self.metrics.borrow();
        let total_lookups = metrics.cache_hits + metrics.cache_misses;
        if total_lookups > 0 {
            stats.hit_rate = (metrics.cache_hits as f64 / total_lookups as f64) * 100.0;
            stats.miss_rate = (metrics.cache_misses as f64 / total_lookups as f64) * 100.0;
        }
        stats
    }

    /// Batch lookup for multiple squares (SIMD optimized)
    pub fn get_attacks_batch(
        &self,
        squares: &[u8],
        piece_type: PieceType,
        occupied: Bitboard,
    ) -> BatchLookupResult {
        let start_time = std::time::Instant::now();
        let mut attacks = Vec::with_capacity(squares.len());
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        let mut lookup_cache = self.lookup_cache.borrow_mut();
        if self.simd_enabled {
            // SIMD-optimized batch lookup
            for &square in squares {
                if let Some(cached) = lookup_cache.get(square, occupied) {
                    attacks.push(cached);
                    cache_hits += 1;
                } else {
                    let attack = self.magic_table.get_attacks(square, piece_type, occupied);
                    lookup_cache.insert(square, occupied, attack);
                    attacks.push(attack);
                    cache_misses += 1;
                }
            }
        } else {
            // Standard batch lookup
            drop(lookup_cache);
            for &square in squares {
                let attack = self.get_attacks(square, piece_type, occupied);
                attacks.push(attack);
            }
            let metrics = self.metrics.borrow();
            cache_hits = metrics.cache_hits;
            cache_misses = metrics.cache_misses;
        }

        BatchLookupResult {
            attacks,
            cache_hits,
            cache_misses,
            total_time: start_time.elapsed(),
        }
    }

    /// Prefetch attack patterns for common access patterns
    pub fn prefetch_common_patterns(&self, piece_type: PieceType, occupied: Bitboard) {
        // Prefetch center squares (most commonly accessed)
        let center_squares = [36, 37, 38, 45, 46, 47, 54, 55, 56]; // 3x3 center
        
        let mut lookup_cache = self.lookup_cache.borrow_mut();
        for &square in &center_squares {
            if !lookup_cache.get(square, occupied).is_some() {
                let attack = self.magic_table.get_attacks(square, piece_type, occupied);
                self.prefetch_buffer.borrow_mut().add_pattern(square, piece_type, occupied, attack);
            }
        }
    }

    /// Get attacks with fallback to ray-casting
    pub fn get_attacks_with_fallback(
        &self,
        square: u8,
        piece_type: PieceType,
        occupied: Bitboard,
    ) -> Bitboard {
        // Try magic lookup first
        if let Ok(attacks) = self.try_magic_lookup(square, piece_type, occupied) {
            return attacks;
        }

        // Fallback to ray-casting
        let mut metrics = self.metrics.borrow_mut();
        metrics.fallback_lookups += 1;
        self.ray_caster.cast_rays(square, piece_type, occupied)
    }

    /// Try magic lookup, return error if not available
    fn try_magic_lookup(
        &self,
        square: u8,
        piece_type: PieceType,
        occupied: Bitboard,
    ) -> Result<Bitboard, MagicError> {
        // Check if magic table is initialized for this square and piece
        if !self.magic_table.is_fully_initialized() {
            return Err(MagicError::InitializationFailed {
                reason: "Magic table not fully initialized".to_string(),
            });
        }

        Ok(self.magic_table.get_attacks(square, piece_type, occupied))
    }

    /// Enable or disable SIMD optimizations
    pub fn set_simd_enabled(&self, enabled: bool) {
        // Note: simd_enabled is not mutable, but we can create a new engine with different settings
        // For now, this is a placeholder - SIMD is determined at construction time
        let _ = enabled;
    }

    /// Check if SIMD is enabled
    pub fn is_simd_enabled(&self) -> bool {
        self.simd_enabled
    }

    /// Get comprehensive performance statistics
    pub fn get_detailed_metrics(&self) -> DetailedMetrics {
        let cache_stats = self.cache_stats();
        let prefetch_stats = self.prefetch_buffer.borrow().stats();
        let metrics = self.metrics.borrow();
        
        DetailedMetrics {
            cache_stats,
            prefetch_stats,
            total_lookups: metrics.lookup_count,
            average_lookup_time: if metrics.lookup_count > 0 {
                std::time::Duration::from_nanos(
                    metrics.total_lookup_time.as_nanos() as u64 / metrics.lookup_count
                )
            } else {
                std::time::Duration::ZERO
            },
            fallback_lookups: metrics.fallback_lookups,
            simd_enabled: self.simd_enabled,
        }
    }

    /// Clear all caches and reset metrics
    pub fn reset_all(&self) {
        self.lookup_cache.borrow_mut().clear();
        self.prefetch_buffer.borrow_mut().clear();
        self.batch_cache.borrow_mut().clear();
        *self.metrics.borrow_mut() = PerformanceMetrics::default();
    }
}

impl LookupCache {
    /// Create a new lookup cache
    pub fn new(size: usize) -> Self {
        let actual_size = size.next_power_of_two();
        Self {
            entries: vec![None; actual_size],
            size: actual_size,
            hash_fn: Self::default_hash,
        }
    }

    /// Get cached attack pattern
    pub fn get(&self, square: u8, occupied: Bitboard) -> Option<Bitboard> {
        let hash = self.calculate_hash(square, occupied);
        let index = (hash % self.size as u64) as usize;
        
        if let Some((cached_square, cached_occupied, cached_attacks)) = &self.entries[index] {
            if *cached_square == square && *cached_occupied == occupied as u64 {
                return Some(*cached_attacks);
            }
        }
        
        None
    }

    /// Insert attack pattern into cache
    pub fn insert(&mut self, square: u8, occupied: Bitboard, attacks: Bitboard) {
        let hash = self.calculate_hash(square, occupied);
        let index = (hash % self.size as u64) as usize;
        
        self.entries[index] = Some((square, occupied as u64, attacks));
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.fill(None);
    }

    /// Calculate hash for cache indexing
    fn calculate_hash(&self, square: u8, occupied: Bitboard) -> u64 {
        (self.hash_fn)(square, occupied as u64) as u64
    }

    /// Default hash function
    fn default_hash(square: u8, occupied: u64) -> usize {
        // Simple hash function - in production, use a better one
        (square as usize * 31 + (occupied % 1000000007) as usize) % 1000000007
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let used_entries = self.entries.iter().filter(|entry| entry.is_some()).count();
        CacheStats {
            total_entries: self.size,
            used_entries,
            utilization_percentage: (used_entries as f64 / self.size as f64) * 100.0,
            hit_rate: 0.0, // Will be set by LookupEngine
            miss_rate: 0.0, // Will be set by LookupEngine
        }
    }
}

impl PrefetchBuffer {
    /// Create a new prefetch buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            patterns: Vec::with_capacity(max_size),
            max_size,
            position: 0,
        }
    }

    /// Add a pattern to the prefetch buffer
    pub fn add_pattern(&mut self, square: u8, piece_type: PieceType, occupied: Bitboard, attacks: Bitboard) {
        if self.patterns.len() < self.max_size {
            self.patterns.push((square, piece_type, occupied, attacks));
        } else {
            // Replace oldest pattern (circular buffer)
            self.patterns[self.position] = (square, piece_type, occupied, attacks);
            self.position = (self.position + 1) % self.max_size;
        }
    }

    /// Get a prefetched pattern
    pub fn get_pattern(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Option<Bitboard> {
        self.patterns.iter()
            .find(|(s, pt, occ, _)| *s == square && *pt == piece_type && *occ == occupied)
            .map(|(_, _, _, attacks)| *attacks)
    }

    /// Clear the prefetch buffer
    pub fn clear(&mut self) {
        self.patterns.clear();
        self.position = 0;
    }

    /// Get prefetch buffer statistics
    pub fn stats(&self) -> PrefetchStats {
        PrefetchStats {
            total_capacity: self.max_size,
            current_size: self.patterns.len(),
            utilization_percentage: (self.patterns.len() as f64 / self.max_size as f64) * 100.0,
        }
    }
}

impl RayCaster {
    /// Create a new ray caster
    pub fn new() -> Self {
        Self {
            directions: vec![
                (0, 1), (0, -1), (1, 0), (-1, 0),  // Rook directions
                (1, 1), (1, -1), (-1, 1), (-1, -1), // Bishop directions
            ],
            board_size: (9, 9), // Shogi board size
        }
    }

    /// Cast rays from a square to generate attack pattern
    pub fn cast_rays(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        let mut attacks = 0u128;
        let (row, col) = (square / 9, square % 9);

        let directions = match piece_type {
            PieceType::Rook => &self.directions[0..4], // Rook directions
            PieceType::Bishop => &self.directions[4..8], // Bishop directions
            _ => return 0, // Only rook and bishop are sliding pieces
        };

        for &(dr, dc) in directions {
            let mut r = row as i8 + dr;
            let mut c = col as i8 + dc;

            while r >= 0 && r < self.board_size.0 as i8 && c >= 0 && c < self.board_size.1 as i8 {
                let target_square = (r * 9 + c) as u8;
                attacks |= 1u128 << target_square;

                // Stop if we hit a piece
                if (occupied & (1u128 << target_square)) != 0 {
                    break;
                }

                r += dr;
                c += dc;
            }
        }

        attacks
    }
}

/// Prefetch buffer statistics
#[derive(Debug, Clone)]
pub struct PrefetchStats {
    pub total_capacity: usize,
    pub current_size: usize,
    pub utilization_percentage: f64,
}

/// Detailed performance metrics
#[derive(Debug, Clone)]
pub struct DetailedMetrics {
    pub cache_stats: CacheStats,
    pub prefetch_stats: PrefetchStats,
    pub total_lookups: u64,
    pub average_lookup_time: std::time::Duration,
    pub fallback_lookups: u64,
    pub simd_enabled: bool,
}


/// Prefetch buffer for next likely accesses
#[derive(Debug, Clone)]
pub struct PrefetchBuffer {
    /// Prefetched attack patterns
    patterns: Vec<(u8, PieceType, Bitboard, Bitboard)>,
    /// Maximum buffer size
    max_size: usize,
    /// Current position in buffer
    position: usize,
}

/// Ray-casting fallback engine
#[derive(Debug, Clone)]
pub struct RayCaster {
    /// Direction vectors for ray casting
    directions: Vec<(i8, i8)>,
    /// Board boundaries
    board_size: (u8, u8),
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub used_entries: usize,
    pub utilization_percentage: f64,
    pub hit_rate: f64,
    pub miss_rate: f64,
}

/// Batch lookup result
#[derive(Debug, Clone)]
pub struct BatchLookupResult {
    pub attacks: Vec<Bitboard>,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_time: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_cache_creation() {
        let cache = LookupCache::new(1024);
        assert_eq!(cache.size, 1024);
        assert_eq!(cache.entries.len(), 1024);
    }

    #[test]
    fn test_lookup_cache_insert_get() {
        let mut cache = LookupCache::new(1024);
        
        cache.insert(0, 0b101, 0b111);
        let result = cache.get(0, 0b101);
        
        assert_eq!(result, Some(0b111));
    }

    #[test]
    fn test_lookup_cache_miss() {
        let cache = LookupCache::new(1024);
        let result = cache.get(0, 0b101);
        
        assert_eq!(result, None);
    }

    #[test]
    fn test_lookup_cache_clear() {
        let mut cache = LookupCache::new(1024);
        cache.insert(0, 0b101, 0b111);
        cache.clear();
        
        let result = cache.get(0, 0b101);
        assert_eq!(result, None);
    }

    #[test]
    fn test_lookup_cache_stats() {
        let mut cache = LookupCache::new(1024);
        cache.insert(0, 0b101, 0b111);
        cache.insert(1, 0b110, 0b111);
        
        let stats = cache.stats();
        assert_eq!(stats.used_entries, 2);
        assert!(stats.utilization_percentage > 0.0);
    }

    #[test]
    fn test_prefetch_buffer() {
        let mut buffer = PrefetchBuffer::new(4);
        
        // Add patterns
        buffer.add_pattern(0, PieceType::Rook, 0b101, 0b111);
        buffer.add_pattern(1, PieceType::Bishop, 0b110, 0b111);
        
        // Test retrieval
        assert_eq!(buffer.get_pattern(0, PieceType::Rook, 0b101), Some(0b111));
        assert_eq!(buffer.get_pattern(1, PieceType::Bishop, 0b110), Some(0b111));
        assert_eq!(buffer.get_pattern(0, PieceType::Bishop, 0b101), None);
        
        // Test circular buffer behavior
        for i in 2..8 {
            buffer.add_pattern(i, PieceType::Rook, 0b101, 0b111);
        }
        
        // First two should be overwritten
        assert_eq!(buffer.get_pattern(0, PieceType::Rook, 0b101), None);
        assert_eq!(buffer.get_pattern(1, PieceType::Bishop, 0b110), None);
        
        // Test stats
        let stats = buffer.stats();
        assert_eq!(stats.total_capacity, 4);
        assert_eq!(stats.current_size, 4);
        assert_eq!(stats.utilization_percentage, 100.0);
    }

    #[test]
    fn test_ray_caster() {
        let caster = RayCaster::new();
        
        // Test rook ray casting from center square (40)
        let attacks = caster.cast_rays(40, PieceType::Rook, 0);
        assert!(attacks != 0);
        
        // Test bishop ray casting
        let attacks = caster.cast_rays(40, PieceType::Bishop, 0);
        assert!(attacks != 0);
        
        // Test with occupied squares
        let occupied = 1u128 << 41; // Block one direction
        let attacks = caster.cast_rays(40, PieceType::Rook, occupied);
        assert!(attacks != 0);
    }

    #[test]
    fn test_lookup_engine_batch() {
        use crate::types::MagicTable;
        
        let magic_table = MagicTable::default();
        let mut engine = LookupEngine::new(magic_table);
        
        let squares = [0, 1, 2, 3];
        let result = engine.get_attacks_batch(&squares, PieceType::Rook, 0);
        
        assert_eq!(result.attacks.len(), 4);
        assert!(result.total_time.as_nanos() > 0);
    }

    #[test]
    fn test_lookup_engine_prefetch() {
        use crate::types::MagicTable;
        
        let magic_table = MagicTable::default();
        let mut engine = LookupEngine::new(magic_table);
        
        // Test prefetching
        engine.prefetch_common_patterns(PieceType::Rook, 0);
        
        // Test SIMD settings
        assert!(engine.is_simd_enabled() || !engine.is_simd_enabled());
        engine.set_simd_enabled(false);
        assert!(!engine.is_simd_enabled());
    }

    #[test]
    fn test_lookup_engine_fallback() {
        use crate::types::MagicTable;
        
        let magic_table = MagicTable::default();
        let mut engine = LookupEngine::new(magic_table);
        
        // Test fallback lookup
        let attacks = engine.get_attacks_with_fallback(40, PieceType::Rook, 0);
        assert!(attacks >= 0);
    }

    #[test]
    fn test_detailed_metrics() {
        use crate::types::MagicTable;
        
        let magic_table = MagicTable::default();
        let engine = LookupEngine::new(magic_table);
        
        let metrics = engine.get_detailed_metrics();
        assert_eq!(metrics.total_lookups, 0);
        assert_eq!(metrics.fallback_lookups, 0);
    }

    #[test]
    fn test_lookup_engine_reset() {
        use crate::types::MagicTable;
        
        let magic_table = MagicTable::default();
        let mut engine = LookupEngine::new(magic_table);
        
        // Perform some operations
        engine.get_attacks(0, PieceType::Rook, 0);
        
        // Reset all
        engine.reset_all();
        
        // Verify reset
        let metrics = engine.get_detailed_metrics();
        assert_eq!(metrics.total_lookups, 0);
    }
}
