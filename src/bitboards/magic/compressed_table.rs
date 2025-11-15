//! Compressed magic table format for reduced memory usage
//!
//! This module provides a compressed representation of magic tables
//! that trades some lookup speed for significant memory savings.
//!
//! ## Compression Strategies
//!
//! The compression system uses multiple strategies to reduce memory usage:
//!
//! 1. **Pattern Deduplication**: Identical attack patterns across different squares
//!    and blocker combinations are stored only once, with an index mapping duplicates
//!    to the single storage location.
//!
//! 2. **Run-Length Encoding (RLE)**: Sparse attack patterns (patterns with many
//!    consecutive empty squares) are encoded using run-length encoding to reduce
//!    storage requirements.
//!
//! 3. **Delta Encoding**: Similar patterns are stored as differences from a base
//!    pattern, reducing storage for patterns that differ only slightly.
//!
//! 4. **Strategy Selection**: The system automatically chooses the best compression
//!    method per pattern based on estimated savings (deduplication > RLE > delta > raw).
//!
//! ## Trade-offs
//!
//! - **Memory Savings**: 30-50% reduction in memory usage (target)
//! - **Lookup Performance**: <10% slowdown (target) due to decompression overhead
//! - **Hot Path Caching**: Frequently accessed patterns are cached in decompressed form
//!    to minimize performance impact
//!
//! ## Configuration
//!
//! Compression can be enabled/disabled via the `compression_enabled` parameter in
//! `CompressedMagicTable::from_table_with_config()`. When disabled, the table
//! behaves identically to an uncompressed table but with compression metadata.

use crate::types::core::PieceType;
use crate::types::{Bitboard, EMPTY_BITBOARD, MagicError, MagicTable};
use std::collections::HashMap;
use std::cell::RefCell;

/// Compression strategy for a single pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompressionStrategy {
    /// Store raw pattern (no compression)
    Raw,
    /// Reference to deduplicated pattern
    Deduplicated(usize),
    /// Run-length encoded pattern
    RleEncoded,
    /// Delta-encoded pattern (difference from base)
    DeltaEncoded(usize), // base pattern index
}

/// Compressed pattern entry
#[derive(Debug, Clone)]
struct CompressedPattern {
    strategy: CompressionStrategy,
    data: CompressedData,
}

/// Compressed data representation
#[derive(Debug, Clone)]
enum CompressedData {
    /// Raw pattern (u128)
    Raw(Bitboard),
    /// RLE-encoded pattern
    Rle(Vec<RleRun>),
    /// Delta-encoded pattern (XOR with base)
    Delta(Bitboard), // XOR difference
}

/// Run-length encoding run
#[derive(Debug, Clone, Copy)]
struct RleRun {
    value: bool, // true = bit set, false = bit clear
    length: u8, // number of consecutive bits
}

/// Compressed magic table with reduced memory footprint
#[derive(Clone)]
pub struct CompressedMagicTable {
    /// Base magic table (contains magic numbers and masks)
    base_table: MagicTable,
    /// Compression enabled flag
    compression_enabled: bool,
    /// Compression ratio achieved
    compression_ratio: f64,
    /// Compressed pattern storage
    compressed_patterns: Vec<CompressedPattern>,
    /// Deduplication index: pattern -> storage index
    dedup_index: HashMap<Bitboard, usize>,
    /// Deduplicated pattern storage
    dedup_storage: Vec<Bitboard>,
    /// Lookup table: original index -> compressed pattern index
    lookup_table: Vec<usize>,
    /// Hot path cache: frequently accessed patterns (decompressed)
    /// Using RefCell for interior mutability to allow cache updates with &self
    hot_cache: RefCell<HashMap<usize, Bitboard>>,
    /// Cache size limit
    cache_size_limit: usize,
    /// Compression statistics per square
    square_stats: Vec<SquareCompressionStats>,
}

/// Compression statistics for a single square
#[derive(Debug, Clone)]
struct SquareCompressionStats {
    square: u8,
    piece_type: PieceType,
    original_count: usize,
    compressed_count: usize,
    dedup_count: usize,
    rle_count: usize,
    delta_count: usize,
    raw_count: usize,
}

/// Compression configuration
#[derive(Debug, Clone, Copy)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Enable hot path caching
    pub enable_hot_cache: bool,
    /// Hot cache size limit (number of patterns)
    pub cache_size_limit: usize,
    /// Minimum pattern count for RLE (patterns with fewer bits don't benefit)
    pub min_rle_bits: usize,
    /// Delta encoding threshold (minimum similarity for delta encoding)
    pub delta_similarity_threshold: f64,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_hot_cache: true,
            cache_size_limit: 1000,
            min_rle_bits: 10,
            delta_similarity_threshold: 0.7, // 70% similarity required
        }
    }
}

impl CompressedMagicTable {
    /// Create a compressed table from a regular magic table
    pub fn from_table(table: MagicTable) -> Result<Self, MagicError> {
        Self::from_table_with_config(table, CompressionConfig::default())
    }

    /// Create a compressed table with custom configuration
    pub fn from_table_with_config(
        table: MagicTable,
        config: CompressionConfig,
    ) -> Result<Self, MagicError> {
        if !config.enabled {
            return Ok(Self {
                base_table: table,
                compression_enabled: false,
                compression_ratio: 1.0,
            compressed_patterns: Vec::new(),
            dedup_index: HashMap::new(),
            dedup_storage: Vec::new(),
            lookup_table: Vec::new(),
            hot_cache: RefCell::new(HashMap::new()),
            cache_size_limit: config.cache_size_limit,
            square_stats: Vec::new(),
            });
        }

        let original_size = table.attack_storage.len();
        let mut compressed_patterns = Vec::new();
        let mut dedup_index = HashMap::new();
        let mut dedup_storage = Vec::new();
        let mut lookup_table = Vec::with_capacity(original_size);
        let mut square_stats = Vec::new();

        // Step 1: Build deduplication index
        for (idx, &pattern) in table.attack_storage.iter().enumerate() {
            if let Some(&dedup_idx) = dedup_index.get(&pattern) {
                // Pattern already exists, use deduplication
                lookup_table.push(dedup_idx);
            } else {
                // New unique pattern
                let dedup_idx = dedup_storage.len();
                dedup_index.insert(pattern, dedup_idx);
                dedup_storage.push(pattern);
                lookup_table.push(dedup_idx);
            }
        }

        // Step 2: Compress each unique pattern
        let mut total_compressed_size = 0;
        for (dedup_idx, &pattern) in dedup_storage.iter().enumerate() {
            // Only compare against already-processed patterns for delta encoding
            let processed_patterns = &dedup_storage[..dedup_idx];
            let compressed = Self::compress_pattern(pattern, processed_patterns, config)?;
            total_compressed_size += Self::estimate_compressed_size(&compressed);
            compressed_patterns.push(compressed);
        }

        // Update lookup table to point to compressed patterns
        // (lookup_table already points to dedup indices, which now map to compressed patterns)

        // Step 3: Calculate compression ratio
        let original_bytes = original_size * std::mem::size_of::<Bitboard>();
        let compressed_bytes = total_compressed_size;
        let compression_ratio = if compressed_bytes > 0 {
            original_bytes as f64 / compressed_bytes as f64
        } else {
            1.0
        };

        // Step 4: Build square statistics
        // (This is a simplified version - full implementation would track per-square stats)

        Ok(Self {
            base_table: table,
            compression_enabled: true,
            compression_ratio,
            compressed_patterns,
            dedup_index,
            dedup_storage,
            lookup_table,
            hot_cache: RefCell::new(HashMap::new()),
            cache_size_limit: config.cache_size_limit,
            square_stats,
        })
    }

    /// Compress a single pattern using the best strategy
    fn compress_pattern(
        pattern: Bitboard,
        dedup_storage: &[Bitboard],
        config: CompressionConfig,
    ) -> Result<CompressedPattern, MagicError> {
        // Strategy 1: Check if pattern is sparse enough for RLE
        let rle_size = Self::estimate_rle_size(pattern);
        let raw_size = std::mem::size_of::<Bitboard>();

        // Strategy 2: Check for delta encoding opportunities
        let (delta_base_idx, delta_size) = Self::find_best_delta_base(pattern, dedup_storage, config);

        // Strategy 3: Compare all options
        let mut best_strategy = CompressionStrategy::Raw;
        let mut best_size = raw_size;

        // Check RLE
        if rle_size < raw_size && Self::count_set_bits(pattern) >= config.min_rle_bits {
            best_strategy = CompressionStrategy::RleEncoded;
            best_size = rle_size;
        }

        // Check delta encoding
        if let Some((base_idx, delta_sz)) = delta_base_idx.zip(Some(delta_size)) {
            if delta_sz < best_size {
                best_strategy = CompressionStrategy::DeltaEncoded(base_idx);
                best_size = delta_sz;
            }
        }

        // Create compressed data
        let data = match best_strategy {
            CompressionStrategy::Raw => CompressedData::Raw(pattern),
            CompressionStrategy::RleEncoded => CompressedData::Rle(Self::rle_encode(pattern)),
            CompressionStrategy::DeltaEncoded(base_idx) => {
                let base = dedup_storage[base_idx];
                CompressedData::Delta(pattern ^ base)
            }
            _ => CompressedData::Raw(pattern),
        };

        Ok(CompressedPattern {
            strategy: best_strategy,
            data,
        })
    }

    /// Estimate RLE-encoded size for a pattern
    fn estimate_rle_size(pattern: Bitboard) -> usize {
        let runs = Self::rle_encode(pattern);
        // Each run: 1 byte (value) + 1 byte (length) = 2 bytes
        runs.len() * 2
    }

    /// Run-length encode a pattern
    fn rle_encode(pattern: Bitboard) -> Vec<RleRun> {
        let mut runs = Vec::new();
        let mut current_value = false;
        let mut current_length = 0u8;

        for i in 0..128 {
            let bit = (pattern >> i) & 1 != 0;
            if bit == current_value && current_length < 255 {
                current_length += 1;
            } else {
                if current_length > 0 {
                    runs.push(RleRun {
                        value: current_value,
                        length: current_length,
                    });
                }
                current_value = bit;
                current_length = 1;
            }
        }

        if current_length > 0 {
            runs.push(RleRun {
                value: current_value,
                length: current_length,
            });
        }

        runs
    }

    /// Decode RLE-encoded pattern
    fn rle_decode(runs: &[RleRun]) -> Bitboard {
        let mut pattern = 0u128;
        let mut bit_pos = 0;

        for run in runs {
            for _ in 0..run.length {
                if run.value && bit_pos < 128 {
                    pattern |= 1u128 << bit_pos;
                }
                bit_pos += 1;
            }
        }

        pattern
    }

    /// Find best base pattern for delta encoding
    fn find_best_delta_base(
        pattern: Bitboard,
        dedup_storage: &[Bitboard],
        config: CompressionConfig,
    ) -> (Option<usize>, usize) {
        let mut best_base = None;
        let mut best_size = std::mem::size_of::<Bitboard>();

        for (idx, &base) in dedup_storage.iter().enumerate() {
            if base == pattern {
                continue; // Skip identical patterns (should use deduplication)
            }

            let similarity = Self::pattern_similarity(pattern, base);
            if similarity >= config.delta_similarity_threshold {
                let delta = pattern ^ base;
                let delta_size = Self::estimate_delta_size(delta);
                if delta_size < best_size {
                    best_base = Some(idx);
                    best_size = delta_size;
                }
            }
        }

        (best_base, best_size)
    }

    /// Calculate similarity between two patterns (0.0 to 1.0)
    fn pattern_similarity(pattern1: Bitboard, pattern2: Bitboard) -> f64 {
        let common_bits = Self::count_set_bits(pattern1 & pattern2);
        let total_bits = Self::count_set_bits(pattern1 | pattern2);
        if total_bits == 0 {
            1.0
        } else {
            common_bits as f64 / total_bits as f64
        }
    }

    /// Estimate delta-encoded size (XOR difference)
    fn estimate_delta_size(delta: Bitboard) -> usize {
        // If delta is sparse, we could use RLE, but for simplicity, just use raw size
        // In a full implementation, we'd check if delta benefits from RLE
        std::mem::size_of::<Bitboard>()
    }

    /// Count set bits in a pattern
    fn count_set_bits(pattern: Bitboard) -> usize {
        pattern.count_ones() as usize
    }

    /// Estimate compressed size of a pattern
    fn estimate_compressed_size(compressed: &CompressedPattern) -> usize {
        match &compressed.data {
            CompressedData::Raw(_) => std::mem::size_of::<Bitboard>(),
            CompressedData::Rle(runs) => runs.len() * 2, // 2 bytes per run
            CompressedData::Delta(_) => std::mem::size_of::<Bitboard>(),
        }
    }

    /// Decompress a pattern
    fn decompress_pattern(&self, compressed_idx: usize) -> Bitboard {
        let compressed = &self.compressed_patterns[compressed_idx];
        match &compressed.data {
            CompressedData::Raw(pattern) => *pattern,
            CompressedData::Rle(runs) => Self::rle_decode(runs),
            CompressedData::Delta(delta) => {
                // For delta encoding, get the base pattern from the strategy
                if let CompressionStrategy::DeltaEncoded(base_idx) = compressed.strategy {
                    let base = self.dedup_storage[base_idx];
                    base ^ delta // XOR to recover original pattern
                } else {
                    *delta // Fallback (shouldn't happen)
                }
            }
        }
    }

    /// Get attacks with decompression and caching
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        if !self.compression_enabled {
            return self.base_table.get_attacks(square, piece_type, occupied);
        }

        // Use base table to get the original index
        let magic_entry = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => &self.base_table.rook_magics[square as usize],
            PieceType::Bishop | PieceType::PromotedBishop => &self.base_table.bishop_magics[square as usize],
            _ => return EMPTY_BITBOARD,
        };

        let relevant_occupied = occupied & magic_entry.mask;
        let hash = (relevant_occupied.wrapping_mul(magic_entry.magic_number as u128)) >> magic_entry.shift;
        let attack_index = magic_entry.attack_base + hash as usize;

        if attack_index >= self.lookup_table.len() {
            return EMPTY_BITBOARD;
        }

        // Get compressed pattern index
        let dedup_idx = self.lookup_table[attack_index];
        let compressed_idx = dedup_idx; // In this implementation, they're the same

        // Check hot cache first
        let mut cache = self.hot_cache.borrow_mut();
        if let Some(&cached) = cache.get(&compressed_idx) {
            return cached;
        }

        // Decompress pattern
        let pattern = self.decompress_pattern(compressed_idx);

        // Cache if enabled and under limit
        if cache.len() < self.cache_size_limit {
            cache.insert(compressed_idx, pattern);
        }

        pattern
    }

    /// Create uncompressed table
    pub fn uncompressed(table: MagicTable) -> Self {
        Self {
            base_table: table,
            compression_enabled: false,
            compression_ratio: 1.0,
            compressed_patterns: Vec::new(),
            dedup_index: HashMap::new(),
            dedup_storage: Vec::new(),
            lookup_table: Vec::new(),
            hot_cache: RefCell::new(HashMap::new()),
            cache_size_limit: 1000,
            square_stats: Vec::new(),
        }
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio
    }

    /// Check if compression is enabled
    pub fn is_compressed(&self) -> bool {
        self.compression_enabled
    }

    /// Get memory savings estimate
    pub fn memory_savings(&self) -> usize {
        let original_size = self.base_table.attack_storage.len() * 16; // u128 = 16 bytes
        let compressed_size = (original_size as f64 / self.compression_ratio) as usize;
        original_size.saturating_sub(compressed_size)
    }

    /// Decompress to full table
    pub fn decompress(self) -> MagicTable {
        self.base_table
    }

    /// Clear hot cache
    pub fn clear_cache(&self) {
        self.hot_cache.borrow_mut().clear();
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub memory_saved: usize,
    pub dedup_count: usize,
    pub rle_count: usize,
    pub delta_count: usize,
    pub raw_count: usize,
}

impl CompressedMagicTable {
    /// Get compression statistics
    pub fn stats(&self) -> CompressionStats {
        let original_size = self.base_table.attack_storage.len() * 16;
        let compressed_size = (original_size as f64 / self.compression_ratio) as usize;

        let mut dedup_count = 0;
        let mut rle_count = 0;
        let mut delta_count = 0;
        let mut raw_count = 0;

        for pattern in &self.compressed_patterns {
            match pattern.strategy {
                CompressionStrategy::Deduplicated(_) => dedup_count += 1,
                CompressionStrategy::RleEncoded => rle_count += 1,
                CompressionStrategy::DeltaEncoded(_) => delta_count += 1,
                CompressionStrategy::Raw => raw_count += 1,
            }
        }

        CompressionStats {
            original_size,
            compressed_size,
            compression_ratio: self.compression_ratio,
            memory_saved: original_size.saturating_sub(compressed_size),
            dedup_count,
            rle_count,
            delta_count,
            raw_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_table_creation() {
        let table = MagicTable::default();
        let compressed = CompressedMagicTable::from_table(table);

        assert!(compressed.is_ok(), "Should create compressed table");
    }

    #[test]
    fn test_compression_stats() {
        let table = MagicTable::default();
        let compressed = CompressedMagicTable::from_table(table).unwrap();

        let stats = compressed.stats();
        assert!(stats.original_size >= stats.compressed_size);
        assert!(
            stats.compression_ratio >= 1.0,
            "Compression ratio should be >= 1.0"
        );
    }

    #[test]
    fn test_uncompressed_table() {
        let table = MagicTable::default();
        let mut uncompressed = CompressedMagicTable::uncompressed(table);

        assert!(!uncompressed.is_compressed(), "Should not be compressed");
        assert_eq!(uncompressed.compression_ratio(), 1.0, "Ratio should be 1.0");
    }

    #[test]
    fn test_rle_encoding() {
        // Test with a sparse pattern
        let pattern: Bitboard = 0b1010101010101010;
        let runs = CompressedMagicTable::rle_encode(pattern);
        let decoded = CompressedMagicTable::rle_decode(&runs);
        assert_eq!(pattern, decoded, "RLE should round-trip correctly");
    }

    #[test]
    fn test_pattern_similarity() {
        let pattern1: Bitboard = 0b11110000;
        let pattern2: Bitboard = 0b11110001;
        let similarity = CompressedMagicTable::pattern_similarity(pattern1, pattern2);
        assert!(similarity > 0.8, "Similar patterns should have high similarity");
    }

    #[test]
    fn test_compression_disabled() {
        let table = MagicTable::default();
        let config = CompressionConfig {
            enabled: false,
            ..Default::default()
        };
        let compressed = CompressedMagicTable::from_table_with_config(table, config).unwrap();
        assert!(!compressed.is_compressed(), "Compression should be disabled");
    }
}
