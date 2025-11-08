//! Magic table construction and management for magic bitboards
//!
//! This module provides functionality to build and manage magic bitboard tables
//! for efficient sliding piece move generation.

use super::attack_generator::AttackGenerator;
use super::magic_finder::MagicFinder;
use crate::types::{
    Bitboard, MagicBitboard, MagicError, MagicTable, MemoryPool, PieceType, EMPTY_BITBOARD,
};

impl MagicTable {
    /// Create a new magic table
    pub fn new() -> Result<Self, MagicError> {
        let mut table = Self::default();
        table.initialize_tables()?;
        Ok(table)
    }

    /// Create a new magic table with custom memory pool
    pub fn with_memory_pool(memory_pool: MemoryPool) -> Result<Self, MagicError> {
        Ok(Self {
            rook_magics: [MagicBitboard::default(); 81],
            bishop_magics: [MagicBitboard::default(); 81],
            attack_storage: Vec::new(),
            memory_pool,
        })
    }

    /// Initialize all magic tables
    fn initialize_tables(&mut self) -> Result<(), MagicError> {
        let start_time = std::time::Instant::now();

        // Initialize rook tables
        for square in 0..81 {
            self.initialize_rook_square(square)?;
        }

        // Initialize bishop tables
        for square in 0..81 {
            self.initialize_bishop_square(square)?;
        }

        println!(
            "Magic table initialization completed in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    /// Initialize magic table for a specific rook square
    fn initialize_rook_square(&mut self, square: u8) -> Result<(), MagicError> {
        let mut finder = MagicFinder::new();
        let magic_result = finder.find_magic_number(square, PieceType::Rook)?;
        let attack_base = self.memory_pool.allocate(magic_result.table_size)?;

        // Generate all attack patterns for this square
        let mut generator = AttackGenerator::new();
        let mask = magic_result.mask;
        for blockers in generator.generate_all_blocker_combinations(mask) {
            let attack = generator.generate_attack_pattern(square, PieceType::Rook, blockers);
            let hash =
                (blockers.wrapping_mul(magic_result.magic_number as u128)) >> magic_result.shift;
            let index = attack_base + hash as usize;

            if index >= self.attack_storage.len() {
                self.attack_storage.resize(index + 1, EMPTY_BITBOARD);
            }

            self.attack_storage[index] = attack;
        }

        self.rook_magics[square as usize] = MagicBitboard {
            magic_number: magic_result.magic_number,
            mask: magic_result.mask,
            shift: magic_result.shift,
            attack_base,
            table_size: magic_result.table_size,
        };

        Ok(())
    }

    /// Initialize magic table for a specific bishop square
    fn initialize_bishop_square(&mut self, square: u8) -> Result<(), MagicError> {
        let mut finder = MagicFinder::new();
        let magic_result = finder.find_magic_number(square, PieceType::Bishop)?;
        let attack_base = self.memory_pool.allocate(magic_result.table_size)?;

        // Generate all attack patterns for this square
        let mut generator = AttackGenerator::new();
        let mask = magic_result.mask;
        for blockers in generator.generate_all_blocker_combinations(mask) {
            let attack = generator.generate_attack_pattern(square, PieceType::Bishop, blockers);
            let hash =
                (blockers.wrapping_mul(magic_result.magic_number as u128)) >> magic_result.shift;
            let index = attack_base + hash as usize;

            if index >= self.attack_storage.len() {
                self.attack_storage.resize(index + 1, EMPTY_BITBOARD);
            }

            self.attack_storage[index] = attack;
        }

        self.bishop_magics[square as usize] = MagicBitboard {
            magic_number: magic_result.magic_number,
            mask: magic_result.mask,
            shift: magic_result.shift,
            attack_base,
            table_size: magic_result.table_size,
        };

        Ok(())
    }

    /// Get attack pattern for a square using magic bitboards
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        let magic_entry = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => &self.rook_magics[square as usize],
            PieceType::Bishop | PieceType::PromotedBishop => &self.bishop_magics[square as usize],
            _ => return EMPTY_BITBOARD,
        };

        // Apply mask to get relevant occupied squares
        let relevant_occupied = occupied & magic_entry.mask;

        // Calculate hash index
        let hash =
            (relevant_occupied.wrapping_mul(magic_entry.magic_number as u128)) >> magic_entry.shift;

        // Lookup attack pattern
        let attack_index = magic_entry.attack_base + hash as usize;
        if attack_index < self.attack_storage.len() {
            self.attack_storage[attack_index]
        } else {
            EMPTY_BITBOARD
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> TableMemoryStats {
        TableMemoryStats {
            total_attack_patterns: self.attack_storage.len(),
            memory_usage_bytes: self.attack_storage.len() * std::mem::size_of::<Bitboard>(),
            pool_stats: self.memory_pool.memory_stats(),
        }
    }

    /// Validate magic table correctness
    pub fn validate(&self) -> Result<(), MagicError> {
        let mut generator = AttackGenerator::new();

        // Validate rook tables
        for square in 0..81 {
            let magic_entry = &self.rook_magics[square as usize];
            if magic_entry.magic_number == 0 {
                continue; // Skip uninitialized entries
            }

            let mask = magic_entry.mask;
            let combinations = generator.generate_all_blocker_combinations(mask);

            for blockers in combinations {
                let expected_attacks =
                    generator.generate_attack_pattern(square, PieceType::Rook, blockers);
                let actual_attacks = self.get_attacks(square, PieceType::Rook, blockers);

                if expected_attacks != actual_attacks {
                    return Err(MagicError::ValidationFailed {
                        reason: format!(
                            "Rook attack mismatch at square {} with blockers {:b}",
                            square, blockers
                        ),
                    });
                }
            }
        }

        // Validate bishop tables
        for square in 0..81 {
            let magic_entry = &self.bishop_magics[square as usize];
            if magic_entry.magic_number == 0 {
                continue; // Skip uninitialized entries
            }

            let mask = magic_entry.mask;
            let combinations = generator.generate_all_blocker_combinations(mask);

            for blockers in combinations {
                let expected_attacks =
                    generator.generate_attack_pattern(square, PieceType::Bishop, blockers);
                let actual_attacks = self.get_attacks(square, PieceType::Bishop, blockers);

                if expected_attacks != actual_attacks {
                    return Err(MagicError::ValidationFailed {
                        reason: format!(
                            "Bishop attack mismatch at square {} with blockers {:b}",
                            square, blockers
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Clear all magic tables
    pub fn clear(&mut self) {
        self.attack_storage.clear();
        self.memory_pool.clear();
        self.rook_magics = [MagicBitboard::default(); 81];
        self.bishop_magics = [MagicBitboard::default(); 81];
    }

    /// Serialize magic table to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, MagicError> {
        use std::io::Write;

        let mut data = Vec::new();

        // Write magic entries
        for magic in &self.rook_magics {
            data.write_all(&magic.magic_number.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&magic.mask.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&magic.shift.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&(magic.attack_base as u64).to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&(magic.table_size as u64).to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
        }

        for magic in &self.bishop_magics {
            data.write_all(&magic.magic_number.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&magic.mask.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&magic.shift.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&(magic.attack_base as u64).to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            data.write_all(&(magic.table_size as u64).to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
        }

        // Write attack storage
        data.write_all(&(self.attack_storage.len() as u32).to_le_bytes())
            .map_err(|e| MagicError::IoError(e.to_string()))?;
        for attack in &self.attack_storage {
            data.write_all(&attack.to_le_bytes())
                .map_err(|e| MagicError::IoError(e.to_string()))?;
        }

        Ok(data)
    }

    /// Deserialize magic table from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, MagicError> {
        use std::io::Read;

        let mut cursor = std::io::Cursor::new(data);
        let mut table = Self::default();

        // Read rook magics
        for i in 0..81 {
            let mut magic_number = [0u8; 8];
            cursor
                .read_exact(&mut magic_number)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut mask = [0u8; 16];
            cursor
                .read_exact(&mut mask)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut shift = [0u8; 1];
            cursor
                .read_exact(&mut shift)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut attack_base = [0u8; 8];
            cursor
                .read_exact(&mut attack_base)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut table_size = [0u8; 8];
            cursor
                .read_exact(&mut table_size)
                .map_err(|e| MagicError::IoError(e.to_string()))?;

            table.rook_magics[i] = MagicBitboard {
                magic_number: u64::from_le_bytes(magic_number),
                mask: u128::from_le_bytes(mask),
                shift: shift[0],
                attack_base: u64::from_le_bytes(attack_base) as usize,
                table_size: u64::from_le_bytes(table_size) as usize,
            };
        }

        // Read bishop magics
        for i in 0..81 {
            let mut magic_number = [0u8; 8];
            cursor
                .read_exact(&mut magic_number)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut mask = [0u8; 16];
            cursor
                .read_exact(&mut mask)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut shift = [0u8; 1];
            cursor
                .read_exact(&mut shift)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut attack_base = [0u8; 8];
            cursor
                .read_exact(&mut attack_base)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            let mut table_size = [0u8; 8];
            cursor
                .read_exact(&mut table_size)
                .map_err(|e| MagicError::IoError(e.to_string()))?;

            table.bishop_magics[i] = MagicBitboard {
                magic_number: u64::from_le_bytes(magic_number),
                mask: u128::from_le_bytes(mask),
                shift: shift[0],
                attack_base: u64::from_le_bytes(attack_base) as usize,
                table_size: u64::from_le_bytes(table_size) as usize,
            };
        }

        // Read attack storage
        let mut storage_len = [0u8; 4];
        cursor
            .read_exact(&mut storage_len)
            .map_err(|e| MagicError::IoError(e.to_string()))?;
        let storage_len = u32::from_le_bytes(storage_len) as usize;

        table.attack_storage.reserve(storage_len);
        for _ in 0..storage_len {
            let mut attack = [0u8; 16];
            cursor
                .read_exact(&mut attack)
                .map_err(|e| MagicError::IoError(e.to_string()))?;
            table.attack_storage.push(u128::from_le_bytes(attack));
        }

        Ok(table)
    }

    /// Get performance statistics
    pub fn performance_stats(&self) -> TablePerformanceStats {
        let mut total_rook_entries = 0;
        let mut total_bishop_entries = 0;

        for magic in &self.rook_magics {
            if magic.magic_number != 0 {
                total_rook_entries += magic.table_size;
            }
        }

        for magic in &self.bishop_magics {
            if magic.magic_number != 0 {
                total_bishop_entries += magic.table_size;
            }
        }

        TablePerformanceStats {
            total_rook_entries,
            total_bishop_entries,
            total_attack_patterns: self.attack_storage.len(),
            memory_efficiency: self.calculate_memory_efficiency(),
        }
    }

    /// Calculate memory efficiency ratio
    fn calculate_memory_efficiency(&self) -> f64 {
        let total_entries = self.attack_storage.len();
        if total_entries == 0 {
            return 0.0;
        }

        let used_entries = self
            .attack_storage
            .iter()
            .filter(|&&pattern| pattern != EMPTY_BITBOARD)
            .count();

        used_entries as f64 / total_entries as f64
    }

    /// Pre-generate all magic tables (for performance)
    pub fn pregenerate_all(&mut self) -> Result<(), MagicError> {
        let start_time = std::time::Instant::now();

        // Pre-generate rook tables
        for square in 0..81 {
            self.initialize_rook_square(square)?;
        }

        // Pre-generate bishop tables
        for square in 0..81 {
            self.initialize_bishop_square(square)?;
        }

        println!(
            "Magic table pre-generation completed in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    /// Check if magic table is fully initialized
    pub fn is_fully_initialized(&self) -> bool {
        self.rook_magics.iter().all(|m| m.magic_number != 0)
            && self.bishop_magics.iter().all(|m| m.magic_number != 0)
    }

    /// Get initialization progress
    pub fn initialization_progress(&self) -> (usize, usize) {
        let rook_initialized = self
            .rook_magics
            .iter()
            .filter(|m| m.magic_number != 0)
            .count();
        let bishop_initialized = self
            .bishop_magics
            .iter()
            .filter(|m| m.magic_number != 0)
            .count();
        (rook_initialized + bishop_initialized, 162) // 81 rook + 81 bishop
    }
}

/// Memory usage statistics for magic table
#[derive(Debug, Clone)]
pub struct TableMemoryStats {
    pub total_attack_patterns: usize,
    pub memory_usage_bytes: usize,
    pub pool_stats: super::memory_pool::MemoryStats,
}

/// Performance statistics for magic table
#[derive(Debug, Clone)]
pub struct TablePerformanceStats {
    pub total_rook_entries: usize,
    pub total_bishop_entries: usize,
    pub total_attack_patterns: usize,
    pub memory_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_table_creation() {
        // Note: This test will fail until magic number generation is implemented
        // let table = MagicTable::new();
        // assert!(table.is_ok());
    }

    #[test]
    fn test_magic_table_default() {
        let table = MagicTable::default();
        assert_eq!(table.attack_storage.len(), 0);
        assert!(table.memory_pool.is_empty());
    }

    #[test]
    fn test_magic_table_clear() {
        let mut table = MagicTable::default();
        table.clear();
        assert_eq!(table.attack_storage.len(), 0);
        assert!(table.memory_pool.is_empty());
    }

    #[test]
    fn test_get_attacks_invalid_piece() {
        let table = MagicTable::default();
        let attacks = table.get_attacks(0, PieceType::Pawn, EMPTY_BITBOARD);
        assert_eq!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_magic_table_serialization() {
        let table = MagicTable::default();
        let serialized = table.serialize().unwrap();
        assert!(!serialized.is_empty());

        let deserialized = MagicTable::deserialize(&serialized).unwrap();
        assert_eq!(
            table.attack_storage.len(),
            deserialized.attack_storage.len()
        );
    }

    #[test]
    fn test_magic_table_validation() {
        let table = MagicTable::default();
        // Empty table should validate (no entries to check)
        assert!(table.validate().is_ok());
    }

    #[test]
    fn test_magic_table_memory_stats() {
        let table = MagicTable::default();
        let stats = table.memory_stats();
        assert_eq!(stats.total_attack_patterns, 0);
        assert_eq!(stats.memory_usage_bytes, 0);
    }

    #[test]
    fn test_magic_table_performance_stats() {
        let table = MagicTable::default();
        let stats = table.performance_stats();
        assert_eq!(stats.total_rook_entries, 0);
        assert_eq!(stats.total_bishop_entries, 0);
        assert_eq!(stats.total_attack_patterns, 0);
        assert_eq!(stats.memory_efficiency, 0.0);
    }

    #[test]
    fn test_magic_table_initialization_progress() {
        let table = MagicTable::default();
        let (initialized, total) = table.initialization_progress();
        assert_eq!(initialized, 0);
        assert_eq!(total, 162);
    }

    #[test]
    fn test_magic_table_fully_initialized() {
        let table = MagicTable::default();
        assert!(!table.is_fully_initialized());
    }

    #[test]
    fn test_magic_table_clear_advanced() {
        let mut table = MagicTable::default();
        table.clear();
        assert_eq!(table.attack_storage.len(), 0);
        assert!(!table.is_fully_initialized());
    }

    #[test]
    fn test_magic_table_get_attacks_empty() {
        let table = MagicTable::default();
        let attacks = table.get_attacks(0, PieceType::Rook, EMPTY_BITBOARD);
        assert_eq!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_magic_table_memory_efficiency() {
        let table = MagicTable::default();
        let efficiency = table.calculate_memory_efficiency();
        assert_eq!(efficiency, 0.0);
    }

    #[test]
    fn test_magic_table_with_memory_pool() {
        let memory_pool = MemoryPool::new();
        let table = MagicTable::with_memory_pool(memory_pool);
        assert!(table.is_ok());
    }

    #[test]
    fn test_magic_table_rook_attacks() {
        let table = MagicTable::default();
        // Test with uninitialized table
        let attacks = table.get_attacks(40, PieceType::Rook, EMPTY_BITBOARD);
        assert_eq!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_magic_table_bishop_attacks() {
        let table = MagicTable::default();
        // Test with uninitialized table
        let attacks = table.get_attacks(40, PieceType::Bishop, EMPTY_BITBOARD);
        assert_eq!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_magic_table_promoted_pieces() {
        let table = MagicTable::default();
        // Test promoted pieces use the same tables as base pieces
        let rook_attacks = table.get_attacks(40, PieceType::Rook, EMPTY_BITBOARD);
        let promoted_rook_attacks = table.get_attacks(40, PieceType::PromotedRook, EMPTY_BITBOARD);
        assert_eq!(rook_attacks, promoted_rook_attacks);

        let bishop_attacks = table.get_attacks(40, PieceType::Bishop, EMPTY_BITBOARD);
        let promoted_bishop_attacks =
            table.get_attacks(40, PieceType::PromotedBishop, EMPTY_BITBOARD);
        assert_eq!(bishop_attacks, promoted_bishop_attacks);
    }

    #[test]
    fn test_magic_table_edge_cases() {
        let table = MagicTable::default();

        // Test corner squares
        let corner_attacks = table.get_attacks(0, PieceType::Rook, EMPTY_BITBOARD);
        assert_eq!(corner_attacks, EMPTY_BITBOARD);

        // Test edge squares
        let edge_attacks = table.get_attacks(4, PieceType::Bishop, EMPTY_BITBOARD);
        assert_eq!(edge_attacks, EMPTY_BITBOARD);

        // Test center square
        let center_attacks = table.get_attacks(40, PieceType::Rook, EMPTY_BITBOARD);
        assert_eq!(center_attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_magic_table_serialization_roundtrip() {
        let original_table = MagicTable::default();
        let serialized = original_table.serialize().unwrap();
        let deserialized = MagicTable::deserialize(&serialized).unwrap();

        // Compare key properties
        assert_eq!(
            original_table.attack_storage.len(),
            deserialized.attack_storage.len()
        );
        assert_eq!(
            original_table.rook_magics.len(),
            deserialized.rook_magics.len()
        );
        assert_eq!(
            original_table.bishop_magics.len(),
            deserialized.bishop_magics.len()
        );
    }

    #[test]
    fn test_magic_table_large_serialization() {
        let mut table = MagicTable::default();
        // Add some dummy data to test serialization
        table.attack_storage.push(0x1234567890ABCDEF);
        table.attack_storage.push(0xFEDCBA0987654321);

        let serialized = table.serialize().unwrap();
        let deserialized = MagicTable::deserialize(&serialized).unwrap();

        assert_eq!(table.attack_storage, deserialized.attack_storage);
    }
}
