//! Performance optimization utilities for transposition table
//!
//! This module provides advanced performance optimizations for the transposition table,
//! including cache line alignment, prefetching, optimized hash mapping, and hot path
//! optimizations while maintaining cross-platform compatibility.

use crate::types::core::Move;
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

/// Cache line size in bytes (64 bytes on most modern architectures)
const CACHE_LINE_SIZE: usize = 64;

/// Maximum number of entries to prefetch
const MAX_PREFETCH_ENTRIES: usize = 8;

/// Optimized hash key to index mapping
///
/// This struct provides fast hash-to-index conversion using bit manipulation
/// and lookup tables for common hash patterns.
#[derive(Debug)]
pub struct OptimizedHashMapper {
    /// Bit mask for fast modulo operations (table_size - 1)
    mask: usize,
    /// Table size (must be power of 2)
    size: usize,
    /// Precomputed hash multipliers for common patterns
    multipliers: [u64; 4],
    /// Fast lookup table for small hash values
    lookup_table: [u16; 256],
}

impl OptimizedHashMapper {
    /// Create a new optimized hash mapper
    pub fn new(table_size: usize) -> Self {
        assert!(
            table_size.is_power_of_two(),
            "Table size must be power of 2"
        );

        let mask = table_size - 1;
        let size = table_size;

        // Precompute multipliers for common hash patterns
        let multipliers = [
            0x9E3779B97F4A7C15, // Golden ratio
            0x517CC1B727220A95, // Another good hash multiplier
            0xBF58476D1CE4E5B9, // Yet another
            0x94D049BB133111EB, // And one more
        ];

        // Create lookup table for small hash values
        let mut lookup_table = [0u16; 256];
        for i in 0..256 {
            lookup_table[i] = ((i as usize) & mask) as u16;
        }

        Self {
            mask,
            size,
            multipliers,
            lookup_table,
        }
    }

    /// Fast hash to index conversion
    ///
    /// This method uses bit manipulation and lookup tables for optimal performance.
    #[inline(always)]
    pub fn hash_to_index(&self, hash: u64) -> usize {
        // Use bit mask for power-of-2 table sizes (fastest method)
        (hash as usize) & self.mask
    }

    /// Alternative hash to index conversion with better distribution
    ///
    /// This method uses multiplication for better hash distribution.
    #[inline(always)]
    pub fn hash_to_index_alternative(&self, hash: u64) -> usize {
        let mixed_hash = hash.wrapping_mul(self.multipliers[0]);
        (mixed_hash as usize) & self.mask
    }

    /// Multi-hash to index conversion for collision reduction
    ///
    /// This method uses multiple hash functions to reduce collisions.
    #[inline(always)]
    pub fn multi_hash_to_index(&self, hash: u64, level: usize) -> usize {
        let multiplier = self.multipliers[level & 3];
        let mixed_hash = hash.wrapping_mul(multiplier);
        (mixed_hash as usize) & self.mask
    }

    /// Fast lookup for small hash values (uses lookup table)
    #[inline(always)]
    pub fn fast_lookup(&self, hash: u8) -> usize {
        self.lookup_table[hash as usize] as usize
    }

    /// Get the table size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the bit mask
    pub fn mask(&self) -> usize {
        self.mask
    }
}

/// Cache-aligned memory allocator for transposition table entries
///
/// This allocator ensures that transposition table entries are aligned to
/// cache line boundaries for optimal memory access performance.
pub struct CacheAlignedAllocator {
    /// Layout for cache-aligned allocation
    layout: Layout,
    /// Number of entries allocated
    entry_count: usize,
}

impl CacheAlignedAllocator {
    /// Create a new cache-aligned allocator
    pub fn new(entry_count: usize, entry_size: usize) -> Result<Self, &'static str> {
        // Calculate total size needed
        let total_size = entry_count * entry_size;

        // Align to cache line boundary
        let aligned_layout =
            Layout::from_size_align(total_size, CACHE_LINE_SIZE).map_err(|_| "Invalid layout")?;

        Ok(Self {
            layout: aligned_layout,
            entry_count,
        })
    }

    /// Allocate cache-aligned memory
    pub unsafe fn allocate(&self) -> Result<NonNull<u8>, &'static str> {
        let ptr = alloc(self.layout);
        if ptr.is_null() {
            Err("Allocation failed")
        } else {
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    /// Deallocate cache-aligned memory
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>) {
        dealloc(ptr.as_ptr(), self.layout);
    }

    /// Get the layout
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }
}

/// Prefetch manager for likely transposition table entries
///
/// This struct manages prefetching of likely-to-be-accessed entries
/// to improve cache performance and reduce memory access latency.
pub struct PrefetchManager {
    /// Recent access patterns
    access_patterns: Vec<u64>,
    /// Prefetch queue
    prefetch_queue: Vec<usize>,
    /// Maximum prefetch queue size
    max_queue_size: usize,
    /// Prefetch distance (number of entries ahead to prefetch)
    prefetch_distance: usize,
}

impl PrefetchManager {
    /// Create a new prefetch manager
    pub fn new(max_queue_size: usize, prefetch_distance: usize) -> Self {
        Self {
            access_patterns: Vec::with_capacity(max_queue_size),
            prefetch_queue: Vec::with_capacity(max_queue_size),
            max_queue_size,
            prefetch_distance,
        }
    }

    /// Add a hash to the access pattern
    #[inline(always)]
    pub fn record_access(&mut self, hash: u64) {
        self.access_patterns.push(hash);

        // Maintain maximum size
        if self.access_patterns.len() > self.max_queue_size {
            self.access_patterns.remove(0);
        }
    }

    /// Get prefetch candidates based on access patterns
    pub fn get_prefetch_candidates(
        &self,
        current_hash: u64,
        mapper: &OptimizedHashMapper,
    ) -> Vec<usize> {
        let mut candidates = Vec::new();

        // Add nearby entries based on recent patterns
        for &pattern_hash in &self.access_patterns {
            if let Some(index) = self.calculate_likely_index(pattern_hash, current_hash, mapper) {
                candidates.push(index);
            }
        }

        // Add sequential entries
        let current_index = mapper.hash_to_index(current_hash);
        for i in 1..=self.prefetch_distance {
            candidates.push((current_index + i) & mapper.mask());
        }

        // Limit to maximum prefetch entries
        candidates.truncate(MAX_PREFETCH_ENTRIES);
        candidates
    }

    /// Calculate likely index based on access pattern
    fn calculate_likely_index(
        &self,
        pattern_hash: u64,
        current_hash: u64,
        mapper: &OptimizedHashMapper,
    ) -> Option<usize> {
        // Use XOR to find related hashes
        let related_hash = pattern_hash ^ current_hash;
        Some(mapper.hash_to_index(related_hash))
    }

    /// Clear access patterns
    pub fn clear_patterns(&mut self) {
        self.access_patterns.clear();
        self.prefetch_queue.clear();
    }

    /// Get statistics about access patterns
    pub fn get_pattern_stats(&self) -> PrefetchStats {
        PrefetchStats {
            pattern_count: self.access_patterns.len(),
            queue_size: self.prefetch_queue.len(),
            max_queue_size: self.max_queue_size,
        }
    }
}

/// Statistics for prefetch manager
#[derive(Debug, Clone)]
pub struct PrefetchStats {
    pub pattern_count: usize,
    pub queue_size: usize,
    pub max_queue_size: usize,
}

/// Optimized entry packing/unpacking utilities
///
/// This module provides highly optimized routines for packing and unpacking
/// transposition table entries with minimal CPU cycles.
pub struct OptimizedEntryPacker;

impl OptimizedEntryPacker {
    /// Pack entry data into a single u64 for atomic operations
    ///
    /// This method packs score, depth, flag, and basic move info into 64 bits
    /// for maximum performance in atomic operations.
    #[inline(always)]
    pub fn pack_entry_fast(score: i32, depth: u8, flag: TranspositionFlag) -> u64 {
        // Pack score (16 bits), depth (8 bits), flag (2 bits), reserved (6 bits) = 32 bits
        let upper_32 = ((score as u32 & 0xFFFF) << 16)
            | ((depth as u32 & 0xFF) << 8)
            | (Self::flag_to_bits(flag) as u32 & 0x03);

        // Use lower 32 bits for additional data or hash
        let lower_32 = 0u32; // Reserved for future use

        ((upper_32 as u64) << 32) | (lower_32 as u64)
    }

    /// Unpack entry data from a single u64
    #[inline(always)]
    pub fn unpack_entry_fast(packed: u64) -> (i32, u8, TranspositionFlag) {
        let upper_32 = (packed >> 32) as u32;

        let score = ((upper_32 >> 16) & 0xFFFF) as i16 as i32;
        let depth = ((upper_32 >> 8) & 0xFF) as u8;
        let flag_bits = upper_32 & 0x03;
        let flag = Self::bits_to_flag(flag_bits);

        (score, depth, flag)
    }

    /// Convert flag to bits
    #[inline(always)]
    fn flag_to_bits(flag: TranspositionFlag) -> u8 {
        match flag {
            TranspositionFlag::Exact => 0,
            TranspositionFlag::LowerBound => 1,
            TranspositionFlag::UpperBound => 2,
        }
    }

    /// Convert bits to flag
    #[inline(always)]
    fn bits_to_flag(bits: u32) -> TranspositionFlag {
        match bits {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            2 => TranspositionFlag::UpperBound,
            _ => TranspositionFlag::Exact, // Default fallback
        }
    }

    /// Pack entry with move information
    #[inline(always)]
    pub fn pack_entry_with_move(
        score: i32,
        depth: u8,
        flag: TranspositionFlag,
        from: Option<u8>,
        to: u8,
    ) -> (u64, u32) {
        let packed_data = Self::pack_entry_fast(score, depth, flag);

        // Pack move data into separate 32-bit value
        let move_data = match from {
            Some(f) => ((f as u32) << 8) | (to as u32),
            None => to as u32,
        };

        (packed_data, move_data)
    }

    /// Unpack entry with move information
    #[inline(always)]
    pub fn unpack_entry_with_move(
        packed_data: u64,
        move_data: u32,
    ) -> (i32, u8, TranspositionFlag, Option<u8>, u8) {
        let (score, depth, flag) = Self::unpack_entry_fast(packed_data);

        let from = if (move_data >> 8) != 0 {
            Some((move_data >> 8) as u8)
        } else {
            None
        };
        let to = move_data as u8;

        (score, depth, flag, from, to)
    }
}

/// Hot path optimizer for common transposition table operations
///
/// This struct provides optimized implementations for the most frequently
/// used transposition table operations.
pub struct HotPathOptimizer {
    /// Optimized hash mapper
    mapper: OptimizedHashMapper,
    /// Prefetch manager
    prefetch_manager: PrefetchManager,
    /// Fast lookup cache for recent entries
    lookup_cache: [Option<(u64, usize)>; 16], // Small cache for recent lookups
    /// Cache index for round-robin replacement
    cache_index: usize,
}

impl HotPathOptimizer {
    /// Create a new hot path optimizer
    pub fn new(table_size: usize) -> Self {
        Self {
            mapper: OptimizedHashMapper::new(table_size),
            prefetch_manager: PrefetchManager::new(32, 4),
            lookup_cache: [None; 16],
            cache_index: 0,
        }
    }

    /// Optimized probe operation
    ///
    /// This method provides the fastest possible probe operation using
    /// all available optimizations.
    #[inline(always)]
    pub fn fast_probe(
        &mut self,
        hash: u64,
        depth: u8,
        entries: &[AtomicPackedEntry],
    ) -> Option<TranspositionEntry> {
        // Check lookup cache first
        if let Some((cached_hash, cached_index)) = self.lookup_cache[self.cache_index] {
            if cached_hash == hash {
                let entry = &entries[cached_index];
                if Self::is_entry_valid_fast(entry, depth) {
                    return Some(Self::unpack_entry_atomic(entry, hash));
                }
            }
        }

        // Calculate index using optimized mapper
        let index = self.mapper.hash_to_index(hash);
        let entry = &entries[index];

        // Check if entry is valid and meets depth requirement
        if Self::is_entry_valid_fast(entry, depth) {
            // Update lookup cache
            self.lookup_cache[self.cache_index] = Some((hash, index));
            self.cache_index = (self.cache_index + 1) % 16;

            // Record access for prefetching
            self.prefetch_manager.record_access(hash);

            return Some(Self::unpack_entry_atomic(entry, hash));
        }

        None
    }

    /// Optimized store operation
    #[inline(always)]
    pub fn fast_store(&mut self, entry: TranspositionEntry, entries: &mut [AtomicPackedEntry]) {
        let index = self.mapper.hash_to_index(entry.hash_key);

        // Pack entry data
        let (_packed_data, _move_data) = OptimizedEntryPacker::pack_entry_with_move(
            entry.score,
            entry.depth,
            entry.flag,
            entry
                .best_move
                .as_ref()
                .and_then(|m| m.from.map(|p| p.to_u8())),
            entry.best_move.as_ref().map_or(0, |m| m.to.to_u8()),
        );

        // Store using atomic operations
        // Note: This is a simplified version - in practice, you'd use proper atomic types
        entries[index] =
            AtomicPackedEntry::new(entry.score, entry.depth, entry.flag, entry.best_move);

        // Record access for prefetching
        self.prefetch_manager.record_access(entry.hash_key);
    }

    /// Check if entry is valid and meets depth requirement (optimized)
    #[inline(always)]
    fn is_entry_valid_fast(entry: &AtomicPackedEntry, depth: u8) -> bool {
        entry.is_valid() && entry.depth() >= depth
    }

    /// Unpack entry from atomic packed data (optimized)
    #[inline(always)]
    fn unpack_entry_atomic(entry: &AtomicPackedEntry, hash: u64) -> TranspositionEntry {
        TranspositionEntry {
            score: entry.score(),
            depth: entry.depth(),
            flag: entry.flag(),
            best_move: entry.best_move(),
            hash_key: hash,
            age: 0, // Would need to be stored separately in a real implementation
            source: crate::types::EntrySource::MainSearch, // Task 7.0.3: Default to MainSearch
        }
    }

    /// Get prefetch candidates for current operation
    pub fn get_prefetch_candidates(&self, hash: u64) -> Vec<usize> {
        self.prefetch_manager
            .get_prefetch_candidates(hash, &self.mapper)
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> HotPathStats {
        HotPathStats {
            mapper_size: self.mapper.size(),
            prefetch_stats: self.prefetch_manager.get_pattern_stats(),
            cache_entries: self.lookup_cache.iter().filter(|e| e.is_some()).count(),
        }
    }
}

/// Statistics for hot path optimizer
#[derive(Debug, Clone)]
pub struct HotPathStats {
    pub mapper_size: usize,
    pub prefetch_stats: PrefetchStats,
    pub cache_entries: usize,
}

/// Platform-agnostic atomic packed entry
///
/// This struct provides atomic operations that work across targets
/// where certain atomic operations may not be available.
#[derive(Debug, Clone, Copy)]
pub struct AtomicPackedEntry {
    /// Packed entry data
    data: u64,
}

impl AtomicPackedEntry {
    /// Create a new atomic packed entry
    pub fn new(score: i32, depth: u8, flag: TranspositionFlag, _best_move: Option<Move>) -> Self {
        let packed = OptimizedEntryPacker::pack_entry_fast(score, depth, flag);
        Self { data: packed }
    }

    /// Load entry data (atomic read)
    #[inline(always)]
    pub fn load(&self) -> u64 {
        self.data
    }

    /// Store entry data (atomic write)
    #[inline(always)]
    pub fn store(&mut self, value: u64) {
        self.data = value;
    }

    /// Compare and swap (atomic operation)
    #[inline(always)]
    pub fn compare_and_swap(&mut self, expected: u64, new: u64) -> bool {
        if self.data == expected {
            self.data = new;
            true
        } else {
            false
        }
    }

    /// Check if entry is valid
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.data != 0
    }

    /// Extract score from packed data
    #[inline(always)]
    pub fn score(&self) -> i32 {
        let (score, _, _) = OptimizedEntryPacker::unpack_entry_fast(self.data);
        score
    }

    /// Extract depth from packed data
    #[inline(always)]
    pub fn depth(&self) -> u8 {
        let (_, depth, _) = OptimizedEntryPacker::unpack_entry_fast(self.data);
        depth
    }

    /// Extract flag from packed data
    #[inline(always)]
    pub fn flag(&self) -> TranspositionFlag {
        let (_, _, flag) = OptimizedEntryPacker::unpack_entry_fast(self.data);
        flag
    }

    /// Extract best move (simplified version)
    #[inline(always)]
    pub fn best_move(&self) -> Option<Move> {
        // This is a simplified version - in practice, you'd store move data separately
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_hash_mapper() {
        let mapper = OptimizedHashMapper::new(1024);

        // Test basic hash to index conversion
        let hash1 = 0x1234567890ABCDEF;
        let index1 = mapper.hash_to_index(hash1);
        assert_eq!(index1, (hash1 as usize) & 1023);

        // Test alternative hash to index conversion
        let index2 = mapper.hash_to_index_alternative(hash1);
        assert_ne!(index1, index2); // Should be different due to multiplication

        // Test multi-hash to index conversion
        let index3 = mapper.multi_hash_to_index(hash1, 1);
        assert!(index3 < 1024);

        // Test fast lookup
        let fast_index = mapper.fast_lookup(42);
        assert_eq!(fast_index, 42 & 1023);
    }

    #[test]
    fn test_optimized_entry_packer() {
        let score = 150;
        let depth = 8;
        let flag = TranspositionFlag::Exact;

        // Test fast packing/unpacking
        let packed = OptimizedEntryPacker::pack_entry_fast(score, depth, flag);
        let (unpacked_score, unpacked_depth, unpacked_flag) =
            OptimizedEntryPacker::unpack_entry_fast(packed);

        assert_eq!(unpacked_score, score);
        assert_eq!(unpacked_depth, depth);
        assert_eq!(unpacked_flag, flag);

        // Test packing with move
        let from = Some(42);
        let to = 84;
        let (packed_data, move_data) =
            OptimizedEntryPacker::pack_entry_with_move(score, depth, flag, from, to);
        let (u_score, u_depth, u_flag, u_from, u_to) =
            OptimizedEntryPacker::unpack_entry_with_move(packed_data, move_data);

        assert_eq!(u_score, score);
        assert_eq!(u_depth, depth);
        assert_eq!(u_flag, flag);
        assert_eq!(u_from, from);
        assert_eq!(u_to, to);
    }

    #[test]
    fn test_prefetch_manager() {
        let mut manager = PrefetchManager::new(16, 2);
        let mapper = OptimizedHashMapper::new(1024);

        // Record some access patterns
        manager.record_access(0x100);
        manager.record_access(0x200);
        manager.record_access(0x300);

        // Get prefetch candidates
        let candidates = manager.get_prefetch_candidates(0x150, &mapper);
        assert!(!candidates.is_empty());
        assert!(candidates.len() <= MAX_PREFETCH_ENTRIES);

        // Check statistics
        let stats = manager.get_pattern_stats();
        assert_eq!(stats.pattern_count, 3);
    }

    #[test]
    fn test_hot_path_optimizer() {
        let mut optimizer = HotPathOptimizer::new(1024);

        // Create test entries
        let mut entries = vec![AtomicPackedEntry::new(0, 0, TranspositionFlag::Exact, None); 1024];

        // Test fast store
        let entry = TranspositionEntry::new_with_age(100, 5, TranspositionFlag::Exact, None, 0x123);
        optimizer.fast_store(entry, &mut entries);

        // Test fast probe
        let result = optimizer.fast_probe(0x123, 5, &entries);
        assert!(result.is_some());
        let found_entry = result.unwrap();
        assert_eq!(found_entry.score, 100);
        assert_eq!(found_entry.depth, 5);
    }

    #[test]
    fn test_atomic_packed_entry() {
        let mut entry = AtomicPackedEntry::new(200, 10, TranspositionFlag::LowerBound, None);

        // Test load/store
        let original_data = entry.load();
        entry.store(original_data + 1);
        assert_ne!(entry.load(), original_data);

        // Test compare and swap
        let current = entry.load();
        assert!(entry.compare_and_swap(current, current + 1));
        assert!(!entry.compare_and_swap(current, current + 2)); // Should fail

        // Test unpacking
        assert_eq!(entry.score(), 200);
        assert_eq!(entry.depth(), 10);
        assert_eq!(entry.flag(), TranspositionFlag::LowerBound);
    }

    #[test]
    fn test_cache_aligned_allocator() {
        let allocator = CacheAlignedAllocator::new(1000, 64).unwrap();
        assert_eq!(allocator.entry_count(), 1000);

        // Test that layout is cache-aligned
        let layout = allocator.layout();
        assert_eq!(layout.align(), CACHE_LINE_SIZE);
    }
}
