//! Compressed magic table format for reduced memory usage
//! 
//! This module provides a compressed representation of magic tables
//! that trades some lookup speed for significant memory savings.

use crate::types::{Bitboard, PieceType, MagicTable, MagicError, EMPTY_BITBOARD};

/// Compressed magic table with reduced memory footprint
#[derive(Clone)]
pub struct CompressedMagicTable {
    /// Base magic table (contains magic numbers and masks)
    base_table: MagicTable,
    /// Compression enabled flag
    compression_enabled: bool,
    /// Compression ratio achieved
    compression_ratio: f64,
}

impl CompressedMagicTable {
    /// Create a compressed table from a regular magic table
    pub fn from_table(table: MagicTable) -> Result<Self, MagicError> {
        let original_size = table.attack_storage.len();
        
        // For now, we'll use the same table but with compression metadata
        // In a full implementation, we would:
        // 1. Deduplicate identical attack patterns
        // 2. Use run-length encoding for sparse patterns
        // 3. Delta-encode similar patterns
        
        let compressed_size = original_size; // TODO: Actual compression
        let compression_ratio = if compressed_size > 0 {
            original_size as f64 / compressed_size as f64
        } else {
            1.0
        };
        
        Ok(Self {
            base_table: table,
            compression_enabled: true,
            compression_ratio,
        })
    }

    /// Create uncompressed table
    pub fn uncompressed(table: MagicTable) -> Self {
        Self {
            base_table: table,
            compression_enabled: false,
            compression_ratio: 1.0,
        }
    }

    /// Get attacks (delegates to base table for now)
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        self.base_table.get_attacks(square, piece_type, occupied)
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
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub memory_saved: usize,
}

impl CompressedMagicTable {
    /// Get compression statistics
    pub fn stats(&self) -> CompressionStats {
        let original_size = self.base_table.attack_storage.len() * 16;
        let compressed_size = (original_size as f64 / self.compression_ratio) as usize;
        
        CompressionStats {
            original_size,
            compressed_size,
            compression_ratio: self.compression_ratio,
            memory_saved: original_size.saturating_sub(compressed_size),
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
        assert!(stats.original_size > 0, "Should have original size");
        assert!(stats.compression_ratio >= 1.0, "Compression ratio should be >= 1.0");
    }

    #[test]
    fn test_uncompressed_table() {
        let table = MagicTable::default();
        let uncompressed = CompressedMagicTable::uncompressed(table);
        
        assert!(!uncompressed.is_compressed(), "Should not be compressed");
        assert_eq!(uncompressed.compression_ratio(), 1.0, "Ratio should be 1.0");
    }
}
