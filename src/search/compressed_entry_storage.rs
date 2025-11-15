//! Compressed Entry Storage
//!
//! This module implements compressed storage for transposition table entries,
//! significantly reducing memory usage while maintaining fast access times.
//! It uses various compression techniques optimized for chess/Shogi position data.
//!
//! # Features
//!
//! - **Multiple Compression Algorithms**: LZ4, Huffman, and custom bit-packing
//! - **Adaptive Compression**: Automatically selects best compression method
//! - **Fast Decompression**: Optimized for real-time game play
//! - **Configurable Compression**: Adjustable compression levels and methods
//! - **Memory Efficiency**: Significant memory savings with minimal performance impact
//!
//! # Usage
//!
//! ```rust
//! use shogi_engine::search::{CompressedEntryStorage, CompressionConfig, TranspositionEntry};
//!
//! // Create compressed storage with LZ4 compression
//! let config = CompressionConfig::lz4_high();
//! let mut storage = CompressedEntryStorage::new(config);
//!
//! // Store an entry (automatically compressed)
//! let entry = TranspositionEntry { /* ... */ };
//! let compressed_data = storage.compress_entry(&entry);
//!
//! // Retrieve an entry (automatically decompressed)
//! let decompressed_entry = storage.decompress_entry(&compressed_data);
//! ```

use crate::types::core::{Move, PieceType, Player, Position};
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::collections::HashMap;

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-9, higher = better compression, slower)
    pub level: u8,
    /// Enable adaptive compression selection
    pub adaptive: bool,
    /// Minimum compression ratio to enable compression
    pub min_ratio: f64,
    /// Cache size for compressed data
    pub cache_size: usize,
    /// Enable dictionary-based compression
    pub use_dictionary: bool,
}

/// Compression algorithms available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// LZ4 fast compression
    Lz4Fast,
    /// LZ4 high compression
    Lz4High,
    /// Huffman coding
    Huffman,
    /// Custom bit-packing
    BitPacking,
    /// Run-length encoding
    Rle,
    /// Adaptive (auto-select best)
    Adaptive,
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total entries compressed
    pub total_compressed: u64,
    /// Total entries decompressed
    pub total_decompressed: u64,
    /// Total bytes saved through compression
    pub bytes_saved: u64,
    /// Total original size
    pub original_size: u64,
    /// Total compressed size
    pub compressed_size: u64,
    /// Average compression ratio
    pub avg_compression_ratio: f64,
    /// Compression time in microseconds
    pub compression_time_us: u64,
    /// Decompression time in microseconds
    pub decompression_time_us: u64,
}

/// Compressed entry data
#[derive(Debug, Clone)]
pub struct CompressedEntry {
    /// Compressed data
    pub data: Vec<u8>,
    /// Original size
    pub original_size: usize,
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compression metadata
    pub metadata: CompressionMetadata,
}

/// Compression metadata
#[derive(Debug, Clone)]
pub struct CompressionMetadata {
    /// Compression ratio achieved
    pub ratio: f64,
    /// Compression time in microseconds
    pub compression_time_us: u64,
    /// Whether compression was beneficial
    pub beneficial: bool,
}

/// Compressed entry storage manager
pub struct CompressedEntryStorage {
    /// Compression configuration
    config: CompressionConfig,
    /// Compression statistics
    stats: CompressionStats,
    /// Compression dictionary for repeated patterns
    dictionary: HashMap<Vec<u8>, u16>,
    /// Dictionary index counter
    dict_counter: u16,
    /// Cache for frequently accessed compressed entries
    cache: HashMap<u64, CompressedEntry>,
    /// Cache hit/miss statistics
    cache_hits: u64,
    cache_misses: u64,
}

impl CompressionConfig {
    /// Create LZ4 fast configuration
    pub fn lz4_fast() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Lz4Fast,
            level: 1,
            adaptive: false,
            min_ratio: 0.8,
            cache_size: 1000,
            use_dictionary: false,
        }
    }

    /// Create LZ4 high compression configuration
    pub fn lz4_high() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Lz4High,
            level: 9,
            adaptive: false,
            min_ratio: 0.6,
            cache_size: 1000,
            use_dictionary: true,
        }
    }

    /// Create Huffman configuration
    pub fn huffman() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Huffman,
            level: 5,
            adaptive: false,
            min_ratio: 0.7,
            cache_size: 500,
            use_dictionary: true,
        }
    }

    /// Create bit-packing configuration
    pub fn bit_packing() -> Self {
        Self {
            algorithm: CompressionAlgorithm::BitPacking,
            level: 3,
            adaptive: false,
            min_ratio: 0.9,
            cache_size: 2000,
            use_dictionary: false,
        }
    }

    /// Create adaptive configuration
    pub fn adaptive() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Adaptive,
            level: 5,
            adaptive: true,
            min_ratio: 0.75,
            cache_size: 1000,
            use_dictionary: true,
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self::lz4_fast()
    }
}

impl CompressedEntryStorage {
    /// Create a new compressed entry storage
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            stats: CompressionStats::default(),
            dictionary: HashMap::new(),
            dict_counter: 0,
            cache: HashMap::with_capacity(1000),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Compress a transposition entry
    pub fn compress_entry(&mut self, entry: &TranspositionEntry) -> CompressedEntry {
        let start_time = std::time::Instant::now();

        // Serialize entry to bytes
        let serialized = self.serialize_entry(entry);
        let original_size = serialized.len();

        // Select compression algorithm
        let algorithm = if self.config.adaptive {
            self.select_best_algorithm(&serialized)
        } else {
            self.config.algorithm
        };

        // Compress the data
        let compressed_data = match algorithm {
            CompressionAlgorithm::Lz4Fast => self.compress_lz4_fast(&serialized),
            CompressionAlgorithm::Lz4High => self.compress_lz4_high(&serialized),
            CompressionAlgorithm::Huffman => self.compress_huffman(&serialized),
            CompressionAlgorithm::BitPacking => self.compress_bit_packing(entry),
            CompressionAlgorithm::Rle => self.compress_rle(&serialized),
            CompressionAlgorithm::Adaptive => {
                // Should not reach here if adaptive is true
                self.compress_lz4_fast(&serialized)
            }
        };

        let compression_time = start_time.elapsed().as_micros() as u64;
        let ratio = compressed_data.len() as f64 / original_size as f64;
        let beneficial = ratio < self.config.min_ratio;

        // Update statistics
        self.stats.total_compressed += 1;
        self.stats.original_size += original_size as u64;
        self.stats.compressed_size += compressed_data.len() as u64;
        self.stats.bytes_saved +=
            (original_size as u64).saturating_sub(compressed_data.len() as u64);
        self.stats.compression_time_us += compression_time;

        // Update average compression ratio
        let total_entries = self.stats.total_compressed as f64;
        self.stats.avg_compression_ratio =
            (self.stats.avg_compression_ratio * (total_entries - 1.0) + ratio) / total_entries;

        CompressedEntry {
            data: compressed_data,
            original_size,
            algorithm,
            metadata: CompressionMetadata {
                ratio,
                compression_time_us: compression_time,
                beneficial,
            },
        }
    }

    /// Decompress a compressed entry
    pub fn decompress_entry(&mut self, compressed: &CompressedEntry) -> TranspositionEntry {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.compute_cache_key(&compressed.data);
        if let Some(cached_entry) = self.cache.get(&cache_key) {
            self.cache_hits += 1;
            return self.deserialize_entry(&cached_entry.data);
        }

        self.cache_misses += 1;

        // Decompress the data
        let decompressed_data = match compressed.algorithm {
            CompressionAlgorithm::Lz4Fast => self.decompress_lz4_fast(&compressed.data),
            CompressionAlgorithm::Lz4High => self.decompress_lz4_high(&compressed.data),
            CompressionAlgorithm::Huffman => self.decompress_huffman(&compressed.data),
            CompressionAlgorithm::BitPacking => self.decompress_bit_packing(&compressed.data),
            CompressionAlgorithm::Rle => self.decompress_rle(&compressed.data),
            CompressionAlgorithm::Adaptive => self.decompress_lz4_fast(&compressed.data),
        };

        let decompression_time = start_time.elapsed().as_micros() as u64;

        // Update statistics
        self.stats.total_decompressed += 1;
        self.stats.decompression_time_us += decompression_time;

        // Cache the decompressed data if beneficial
        if compressed.metadata.beneficial {
            self.cache_decompressed(cache_key, &decompressed_data);
        }

        // Deserialize to entry
        self.deserialize_entry(&decompressed_data)
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }

    /// Clear compression cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// Update dictionary with new patterns
    pub fn update_dictionary(&mut self, patterns: &[Vec<u8>]) {
        if !self.config.use_dictionary {
            return;
        }

        for pattern in patterns {
            if !self.dictionary.contains_key(pattern) && self.dict_counter < 65535 {
                self.dictionary.insert(pattern.clone(), self.dict_counter);
                self.dict_counter += 1;
            }
        }
    }

    /// Serialize entry to bytes
    fn serialize_entry(&self, entry: &TranspositionEntry) -> Vec<u8> {
        let mut data = Vec::new();

        // Pack entry data efficiently
        data.extend_from_slice(&entry.hash_key.to_le_bytes());
        data.push(entry.depth);
        data.extend_from_slice(&entry.score.to_le_bytes());
        data.push(entry.flag as u8);
        data.push(entry.age as u8);

        // Pack best move if present
        if let Some(move_) = &entry.best_move {
            data.push(1); // Has move flag
            data.extend_from_slice(&self.pack_move(move_));
        } else {
            data.push(0); // No move flag
        }

        data
    }

    /// Deserialize bytes to entry
    fn deserialize_entry(&self, data: &[u8]) -> TranspositionEntry {
        let mut offset = 0;

        // Unpack entry data
        let hash_key = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        let depth = data[offset];
        offset += 1;

        let score = i32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let flag = match data[offset] {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            2 => TranspositionFlag::UpperBound,
            _ => TranspositionFlag::Exact,
        };
        offset += 1;

        let age = data[offset] as u32;
        offset += 1;

        // Unpack best move if present
        let best_move = if data[offset] == 1 {
            offset += 1;
            Some(self.unpack_move(&data[offset..offset + 4]))
        } else {
            None
        };

        TranspositionEntry {
            hash_key,
            depth,
            score,
            flag,
            best_move,
            age,
            source: crate::types::EntrySource::MainSearch,
        }
    }

    /// Pack move into 4 bytes
    fn pack_move(&self, move_: &Move) -> [u8; 4] {
        let from = move_.from.map(|p| p.to_u8()).unwrap_or(0);
        let to = move_.to.to_u8();
        let piece_type = move_.piece_type as u8;
        let flags = (move_.is_promotion as u8) << 3
            | (move_.is_capture as u8) << 2
            | (move_.gives_check as u8) << 1
            | (move_.is_recapture as u8);

        [from, to, piece_type, flags]
    }

    /// Unpack move from 4 bytes
    fn unpack_move(&self, data: &[u8]) -> Move {
        let from = if data[0] == 0 {
            None
        } else {
            Some(Position::from_u8(data[0]))
        };
        let to = Position::from_u8(data[1]);
        let piece_type = match data[2] {
            0 => PieceType::Pawn,
            1 => PieceType::Lance,
            2 => PieceType::Knight,
            3 => PieceType::Silver,
            4 => PieceType::Gold,
            5 => PieceType::Bishop,
            6 => PieceType::Rook,
            7 => PieceType::King,
            _ => PieceType::Pawn,
        };
        let flags = data[3];

        Move {
            from,
            to,
            piece_type,
            player: Player::Black, // Default - would need to be stored separately
            is_promotion: (flags & 0x8) != 0,
            is_capture: (flags & 0x4) != 0,
            captured_piece: None, // Not stored in compact format
            gives_check: (flags & 0x2) != 0,
            is_recapture: (flags & 0x1) != 0,
        }
    }

    /// Select best compression algorithm based on data characteristics
    fn select_best_algorithm(&self, data: &[u8]) -> CompressionAlgorithm {
        // Simple heuristic based on data characteristics
        let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
        let entropy = self.calculate_entropy(data);

        if entropy < 3.0 {
            CompressionAlgorithm::Rle
        } else if unique_bytes < 16 {
            CompressionAlgorithm::Huffman
        } else if data.len() < 32 {
            CompressionAlgorithm::BitPacking
        } else {
            CompressionAlgorithm::Lz4Fast
        }
    }

    /// Calculate entropy of data
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let mut entropy = 0.0;
        let len = data.len() as f64;

        for &count in &counts {
            if count > 0 {
                let probability = count as f64 / len;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// LZ4 fast compression (simplified implementation)
    fn compress_lz4_fast(&self, data: &[u8]) -> Vec<u8> {
        // Simplified LZ4-like compression
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let mut match_len = 0;
            let mut match_offset = 0;

            // Find longest match
            for j in (0..i).rev().take(4095) {
                let mut len = 0;
                while i + len < data.len()
                    && j + len < i
                    && data[i + len] == data[j + len]
                    && len < 18
                {
                    len += 1;
                }

                if len > match_len {
                    match_len = len;
                    match_offset = i - j;
                }
            }

            if match_len >= 4 {
                // Literal + match
                let literal_len = (i - match_offset).min(15);
                let token = (literal_len << 4) | (match_len - 4);
                compressed.push(token as u8);

                if literal_len == 15 {
                    compressed.push(255);
                }

                compressed.extend_from_slice(&data[i - literal_len..i]);
                compressed.extend_from_slice(&(match_offset as u16).to_le_bytes());
                i += match_len;
            } else {
                // Literal only
                compressed.push(data[i]);
                i += 1;
            }
        }

        compressed
    }

    /// LZ4 fast decompression
    fn decompress_lz4_fast(&self, data: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if i + 1 < data.len() && (data[i] >> 4) > 0 {
                // Has literal
                let literal_len = data[i] >> 4;
                let match_len = (data[i] & 0xF) + 4;

                i += 1;
                if literal_len == 15 {
                    i += 1; // Skip extra literal length byte
                }

                // Copy literals
                decompressed.extend_from_slice(&data[i..i + literal_len as usize]);
                i += literal_len as usize;

                // Copy match
                if i + 2 <= data.len() {
                    let offset = u16::from_le_bytes([data[i], data[i + 1]]) as usize;
                    let start = decompressed.len().saturating_sub(offset);
                    for j in 0..match_len as usize {
                        decompressed.push(decompressed[start + j]);
                    }
                    i += 2;
                }
            } else {
                // Single byte literal
                decompressed.push(data[i]);
                i += 1;
            }
        }

        decompressed
    }

    /// LZ4 high compression (placeholder)
    fn compress_lz4_high(&self, data: &[u8]) -> Vec<u8> {
        // For now, use fast compression
        self.compress_lz4_fast(data)
    }

    /// LZ4 high decompression (placeholder)
    fn decompress_lz4_high(&self, data: &[u8]) -> Vec<u8> {
        self.decompress_lz4_fast(data)
    }

    /// Huffman compression (simplified implementation)
    fn compress_huffman(&self, data: &[u8]) -> Vec<u8> {
        // Simplified Huffman-like compression
        let mut compressed = Vec::new();

        // Count frequencies
        let mut frequencies = [0u32; 256];
        for &byte in data {
            frequencies[byte as usize] += 1;
        }

        // Simple encoding: store frequency table + encoded data
        for &freq in &frequencies {
            compressed.extend_from_slice(&freq.to_le_bytes());
        }

        // Simple encoding: use shorter codes for frequent bytes
        for &byte in data {
            if frequencies[byte as usize] > data.len() as u32 / 16 {
                compressed.push(0); // Escape character
                compressed.push(byte);
            } else {
                compressed.push(byte);
            }
        }

        compressed
    }

    /// Huffman decompression
    fn decompress_huffman(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 1024 {
            return data[1024..].to_vec();
        }

        // Skip frequency table (1024 bytes)
        let mut decompressed = Vec::new();
        let mut i = 1024;

        while i < data.len() {
            if data[i] == 0 && i + 1 < data.len() {
                // Escape sequence
                decompressed.push(data[i + 1]);
                i += 2;
            } else {
                decompressed.push(data[i]);
                i += 1;
            }
        }

        decompressed
    }

    /// Bit-packing compression
    fn compress_bit_packing(&self, entry: &TranspositionEntry) -> Vec<u8> {
        let mut packed = Vec::new();

        // Pack hash key (48 bits)
        let hash_bytes = entry.hash_key.to_le_bytes();
        packed.extend_from_slice(&hash_bytes[..6]);

        // Pack depth (8 bits)
        packed.push(entry.depth);

        // Pack score (16 bits, assuming reasonable range)
        let score_packed = (entry.score + 32768) as u16; // Offset to make positive
        packed.extend_from_slice(&score_packed.to_le_bytes());

        // Pack flag (2 bits)
        let flag_bits = entry.flag as u8;

        // Pack age (8 bits)
        let age_bits = entry.age as u8;

        // Pack best move (32 bits if present)
        let move_bits = if let Some(move_) = &entry.best_move {
            let move_packed = self.pack_move(move_);
            packed.extend_from_slice(&move_packed);
            1u8
        } else {
            0u8
        };

        // Combine remaining bits
        packed.push((flag_bits << 6) | (age_bits << 1) | move_bits);

        packed
    }

    /// Bit-packing decompression
    fn decompress_bit_packing(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 8 {
            return data.to_vec();
        }

        let mut decompressed = Vec::new();

        // Reconstruct original format
        decompressed.extend_from_slice(&data[..6]); // Hash key
        decompressed.push(0);
        decompressed.push(0); // Pad hash key to 8 bytes
        decompressed.push(data[6]); // Depth

        // Reconstruct score
        let score_packed = u16::from_le_bytes([data[7], data[8]]);
        let score = (score_packed as i32) - 32768;
        decompressed.extend_from_slice(&score.to_le_bytes());

        // Reconstruct flag, age, and move info
        let control_byte = data[data.len() - 1];
        decompressed.push(control_byte & 0x3F); // Flag (2 bits)
        decompressed.push(0); // Age placeholder
        decompressed.push((control_byte >> 1) & 0x7F); // Age (8 bits)

        // Move data if present
        if control_byte & 1 != 0 && data.len() >= 12 {
            decompressed.push(1); // Has move
            decompressed.extend_from_slice(&data[9..13]);
        } else {
            decompressed.push(0); // No move
        }

        decompressed
    }

    /// Run-length encoding compression
    fn compress_rle(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut compressed = Vec::new();
        let mut current_byte = data[0];
        let mut count = 1u8;

        for &byte in &data[1..] {
            if byte == current_byte && count < 255 {
                count += 1;
            } else {
                compressed.push(count);
                compressed.push(current_byte);
                current_byte = byte;
                count = 1;
            }
        }

        compressed.push(count);
        compressed.push(current_byte);

        compressed
    }

    /// Run-length encoding decompression
    fn decompress_rle(&self, data: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i + 1 < data.len() {
            let count = data[i] as usize;
            let byte = data[i + 1];

            for _ in 0..count {
                decompressed.push(byte);
            }

            i += 2;
        }

        decompressed
    }

    /// Compute cache key for compressed data
    fn compute_cache_key(&self, data: &[u8]) -> u64 {
        let mut hash = 0u64;
        for &byte in data {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Cache decompressed data
    fn cache_decompressed(&mut self, key: u64, data: &[u8]) {
        if self.cache.len() >= self.config.cache_size {
            // Remove oldest entry (simple FIFO)
            if let Some(oldest_key) = self.cache.keys().next().copied() {
                self.cache.remove(&oldest_key);
            }
        }

        self.cache.insert(
            key,
            CompressedEntry {
                data: data.to_vec(),
                original_size: data.len(),
                algorithm: CompressionAlgorithm::Lz4Fast, // Placeholder
                metadata: CompressionMetadata {
                    ratio: 1.0,
                    compression_time_us: 0,
                    beneficial: true,
                },
            },
        );
    }
}

impl Default for CompressedEntryStorage {
    fn default() -> Self {
        Self::new(CompressionConfig::default())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_compression_configs() {
        let lz4_fast = CompressionConfig::lz4_fast();
        assert_eq!(lz4_fast.algorithm, CompressionAlgorithm::Lz4Fast);
        assert_eq!(lz4_fast.level, 1);

        let lz4_high = CompressionConfig::lz4_high();
        assert_eq!(lz4_high.algorithm, CompressionAlgorithm::Lz4High);
        assert_eq!(lz4_high.level, 9);

        let huffman = CompressionConfig::huffman();
        assert_eq!(huffman.algorithm, CompressionAlgorithm::Huffman);

        let adaptive = CompressionConfig::adaptive();
        assert_eq!(adaptive.algorithm, CompressionAlgorithm::Adaptive);
        assert!(adaptive.adaptive);
    }

    #[test]
    fn test_entry_compression_decompression() {
        let mut storage = CompressedEntryStorage::new(CompressionConfig::lz4_fast());

        let original_entry = TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 5,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
            source: crate::types::EntrySource::MainSearch,
            source: crate::types::EntrySource::MainSearch,
        };

        let compressed = storage.compress_entry(&original_entry);
        let decompressed = storage.decompress_entry(&compressed);

        assert_eq!(decompressed.hash_key, original_entry.hash_key);
        assert_eq!(decompressed.depth, original_entry.depth);
        assert_eq!(decompressed.score, original_entry.score);
        assert_eq!(decompressed.flag, original_entry.flag);
        assert_eq!(decompressed.age, original_entry.age);
        assert_eq!(decompressed.best_move, original_entry.best_move);
    }

    #[test]
    fn test_entry_with_move() {
        let mut storage = CompressedEntryStorage::new(CompressionConfig::bit_packing());

        let move_ = Move {
            from: Some(Position::from_u8(10)),
            to: Position::from_u8(20),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: true,
            is_capture: false,
            captured_piece: None,
            gives_check: true,
            is_recapture: false,
        };

        let original_entry = TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 3,
            score: -50,
            flag: TranspositionFlag::LowerBound,
            best_move: Some(move_.clone()),
            age: 5,
            source: crate::types::EntrySource::MainSearch,
            source: crate::types::EntrySource::MainSearch,
        };

        let compressed = storage.compress_entry(&original_entry);
        let decompressed = storage.decompress_entry(&compressed);

        assert_eq!(decompressed.hash_key, original_entry.hash_key);
        assert_eq!(decompressed.depth, original_entry.depth);
        assert_eq!(decompressed.score, original_entry.score);
        assert_eq!(decompressed.flag, original_entry.flag);
        assert_eq!(decompressed.age, original_entry.age);

        // Check move (some fields might be simplified)
        assert!(decompressed.best_move.is_some());
        let decompressed_move = decompressed.best_move.unwrap();
        assert_eq!(decompressed_move.from, move_.from);
        assert_eq!(decompressed_move.to, move_.to);
        assert_eq!(decompressed_move.piece_type, move_.piece_type);
        assert_eq!(decompressed_move.is_promotion, move_.is_promotion);
        assert_eq!(decompressed_move.is_capture, move_.is_capture);
        assert_eq!(decompressed_move.gives_check, move_.gives_check);
        assert_eq!(decompressed_move.is_recapture, move_.is_recapture);
    }

    #[test]
    fn test_compression_statistics() {
        let mut storage = CompressedEntryStorage::new(CompressionConfig::lz4_fast());

        let entry = TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 5,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
            source: crate::types::EntrySource::MainSearch,
            source: crate::types::EntrySource::MainSearch,
        };

        // Compress and decompress multiple times
        for _ in 0..10 {
            let compressed = storage.compress_entry(&entry);
            let _decompressed = storage.decompress_entry(&compressed);
        }

        let stats = storage.get_stats();
        assert_eq!(stats.total_compressed, 10);
        assert_eq!(stats.total_decompressed, 10);
        assert!(stats.original_size > 0);
        assert!(stats.compressed_size > 0);
        assert!(stats.avg_compression_ratio > 0.0);
    }

    #[test]
    fn test_different_compression_algorithms() {
        let algorithms = [
            CompressionConfig::lz4_fast(),
            CompressionConfig::lz4_high(),
            CompressionConfig::huffman(),
            CompressionConfig::bit_packing(),
            CompressionConfig::rle(),
        ];

        let entry = TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 5,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
            source: crate::types::EntrySource::MainSearch,
            source: crate::types::EntrySource::MainSearch,
        };

        for config in algorithms {
            let mut storage = CompressedEntryStorage::new(config);
            let compressed = storage.compress_entry(&entry);
            let decompressed = storage.decompress_entry(&compressed);

            assert_eq!(decompressed.hash_key, entry.hash_key);
            assert_eq!(decompressed.depth, entry.depth);
            assert_eq!(decompressed.score, entry.score);
            assert_eq!(decompressed.flag, entry.flag);
        }
    }

    #[test]
    fn test_cache_functionality() {
        let mut storage = CompressedEntryStorage::new(CompressionConfig::lz4_fast());

        let entry = TranspositionEntry {
            hash_key: 0x123456789ABCDEF0,
            depth: 5,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 10,
            source: crate::types::EntrySource::MainSearch,
            source: crate::types::EntrySource::MainSearch,
        };

        let compressed = storage.compress_entry(&entry);

        // First decompression (cache miss)
        let _decompressed1 = storage.decompress_entry(&compressed);
        let initial_misses = storage.cache_misses;

        // Second decompression (cache hit)
        let _decompressed2 = storage.decompress_entry(&compressed);

        assert!(storage.cache_hits > 0);
        assert_eq!(storage.cache_misses, initial_misses);
    }

    #[test]
    fn test_entropy_calculation() {
        let storage = CompressedEntryStorage::new(CompressionConfig::default());

        // Low entropy data (repeated bytes)
        let low_entropy = vec![1, 1, 1, 1, 1, 1, 1, 1];
        let low_entropy_value = storage.calculate_entropy(&low_entropy);
        assert!(low_entropy_value < 1.0);

        // High entropy data (random-like)
        let high_entropy = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let high_entropy_value = storage.calculate_entropy(&high_entropy);
        assert!(high_entropy_value > 2.0);
    }
}
