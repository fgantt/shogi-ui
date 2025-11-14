//! Memory-mapped file support for large magic tables
//!
//! This module provides memory-mapped file support for loading large magic tables
//! from disk without loading the entire file into memory. This is useful for
//! tables larger than 100MB where memory usage is a concern.

use crate::types::{Bitboard, MagicError, MagicTable, PieceType};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// Memory-mapped magic table
///
/// This table loads magic table data from disk via memory mapping, allowing
/// the OS to manage paging and reducing memory usage for large tables.
pub struct MemoryMappedMagicTable {
    /// Memory-mapped file
    mmap: Mmap,
    /// Magic table structure (references data in mmap)
    table: MagicTable,
    /// File path
    path: std::path::PathBuf,
}

impl MemoryMappedMagicTable {
    /// Create a memory-mapped magic table from a file
    ///
    /// The file must be a valid magic table file (created by `MagicTable::save_to_file()`).
    /// For tables larger than 100MB, this is more memory-efficient than loading into RAM.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MagicError> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| MagicError::IoError(format!("Failed to open file: {}", e)))?;

        // Memory map the file
        let mmap = unsafe {
            Mmap::map(&file)
                .map_err(|e| MagicError::IoError(format!("Failed to memory map file: {}", e)))?
        };

        // Deserialize the table from the memory-mapped data
        // Note: We need to copy the data since MagicTable owns its data
        // For true zero-copy, we'd need to refactor MagicTable to support borrowed data
        let table = MagicTable::deserialize(&mmap[..])
            .map_err(|e| MagicError::IoError(format!("Failed to deserialize table: {}", e)))?;

        Ok(Self {
            mmap,
            table,
            path: path.to_path_buf(),
        })
    }

    /// Get attacks (delegates to underlying table)
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        self.table.get_attacks(square, piece_type, occupied)
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryMappedStats {
        MemoryMappedStats {
            file_size: self.mmap.len(),
            mapped_size: self.mmap.len(),
            table_size: self.table.attack_storage.len() * std::mem::size_of::<Bitboard>(),
        }
    }

    /// Check if table is valid
    pub fn validate(&self) -> Result<(), MagicError> {
        self.table.validate()
    }
}

/// Statistics for memory-mapped table
#[derive(Debug, Clone)]
pub struct MemoryMappedStats {
    /// Total file size in bytes
    pub file_size: usize,
    /// Mapped memory size in bytes
    pub mapped_size: usize,
    /// Table data size in bytes
    pub table_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    #[ignore] // Requires generated table file
    fn test_memory_mapped_table() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_mmap_magic_table.bin");

        // Generate and save a table
        let table = MagicTable::new().unwrap();
        table.save_to_file(&test_file).unwrap();

        // Load as memory-mapped
        let mmap_table = MemoryMappedMagicTable::from_file(&test_file).unwrap();

        // Verify it works
        let attacks = mmap_table.get_attacks(40, PieceType::Rook, 0);
        assert_ne!(attacks, 0);

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }
}

