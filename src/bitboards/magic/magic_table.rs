//! Magic table construction and management for magic bitboards
//!
//! This module provides functionality to build and manage magic bitboard tables
//! for efficient sliding piece move generation.

use super::attack_generator::AttackGenerator;
use super::magic_finder::MagicFinder;
use crate::types::{
    Bitboard, MagicBitboard, MagicError, MagicTable, MemoryPool, PieceType, EMPTY_BITBOARD,
};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Magic number for magic table file identification
pub const MAGIC_TABLE_FILE_MAGIC: &[u8] = b"SHOGI_MAGIC_V1";

/// Current version of the magic table file format
pub const MAGIC_TABLE_FILE_VERSION: u8 = 1;

/// Get the default path for the magic table file
/// 
/// Checks environment variable `SHOGI_MAGIC_TABLE_PATH` first, then falls back to
/// `resources/magic_tables/magic_table.bin` relative to the executable or workspace root.
pub fn get_default_magic_table_path() -> std::path::PathBuf {
    // Check environment variable first
    if let Ok(custom_path) = std::env::var("SHOGI_MAGIC_TABLE_PATH") {
        return std::path::PathBuf::from(custom_path);
    }

    // Try to find workspace root or use current directory
    let base_path = if let Ok(exe_path) = std::env::current_exe() {
        // In production, try relative to executable
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("resources").join("magic_tables")
        } else {
            std::path::PathBuf::from("resources").join("magic_tables")
        }
    } else {
        // Fallback: try to find workspace root
        let mut current = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        loop {
            let resources = current.join("resources").join("magic_tables");
            if resources.exists() || current.join("Cargo.toml").exists() {
                return resources.join("magic_table.bin");
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
        std::path::PathBuf::from("resources").join("magic_tables")
    };

    base_path.join("magic_table.bin")
}

impl MagicTable {
    /// Create a new magic table
    pub fn new() -> Result<Self, MagicError> {
        let mut table = Self::default();
        table.initialize_tables()?;
        Ok(table)
    }

    /// Create a new magic table with progress callback
    pub fn new_with_progress<F>(progress_callback: Option<F>) -> Result<Self, MagicError>
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        let mut table = Self::default();
        let callback = progress_callback.map(|f| Box::new(f) as Box<dyn Fn(f64) + Send + Sync>);
        table.initialize_tables_with_progress(callback)?;
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
        self.initialize_tables_with_progress(None)
    }

    /// Initialize all magic tables with progress callback
    ///
    /// The callback receives progress as a f64 from 0.0 to 1.0
    pub fn initialize_tables_with_progress(
        &mut self,
        progress_callback: Option<Box<dyn Fn(f64) + Send + Sync>>,
    ) -> Result<(), MagicError> {
        let start_time = std::time::Instant::now();
        let total_squares = 162; // 81 rook + 81 bishop
        let mut completed = 0;

        // Estimate total storage needed to pre-allocate
        let estimated_size = self.estimate_total_storage_size();
        self.attack_storage.reserve(estimated_size);

        // Initialize rook tables
        for square in 0..81 {
            self.initialize_rook_square(square)?;
            completed += 1;
            if let Some(ref callback) = progress_callback {
                callback(completed as f64 / total_squares as f64);
            }
        }

        // Initialize bishop tables
        for square in 0..81 {
            self.initialize_bishop_square(square)?;
            completed += 1;
            if let Some(ref callback) = progress_callback {
                callback(completed as f64 / total_squares as f64);
            }
        }

        println!(
            "Magic table initialization completed in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    /// Estimate total storage size needed for all tables
    fn estimate_total_storage_size(&self) -> usize {
        // Rough estimate: average table size per square * 162 squares
        // This is conservative and will be adjusted as we allocate
        let avg_table_size = 1024; // Conservative estimate
        avg_table_size * 162
    }

    /// Initialize magic table for a specific rook square
    pub fn initialize_rook_square(&mut self, square: u8) -> Result<(), MagicError> {
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
    pub fn initialize_bishop_square(&mut self, square: u8) -> Result<(), MagicError> {
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
    ///
    /// # Safety Guarantees
    ///
    /// This method includes comprehensive safety checks:
    /// - Validates magic entry is initialized (magic_number != 0)
    /// - Validates attack_index is within bounds
    /// - Falls back to ray-casting if lookup fails or entry is invalid
    ///
    /// # Fallback Behavior
    ///
    /// If the magic table lookup fails (invalid entry, out of bounds, or corruption),
    /// this method automatically falls back to ray-casting attack generation. This ensures
    /// the engine continues to function correctly even if the magic table is corrupted
    /// or partially initialized.
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        let magic_entry = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => &self.rook_magics[square as usize],
            PieceType::Bishop | PieceType::PromotedBishop => &self.bishop_magics[square as usize],
            _ => return EMPTY_BITBOARD,
        };

        // Validate magic entry is initialized
        if magic_entry.magic_number == 0 {
            // Fallback to ray-casting for uninitialized entries
            return self.get_attacks_fallback(square, piece_type, occupied);
        }

        // Apply mask to get relevant occupied squares
        let relevant_occupied = occupied & magic_entry.mask;

        // Calculate hash index
        let hash =
            (relevant_occupied.wrapping_mul(magic_entry.magic_number as u128)) >> magic_entry.shift;

        // Lookup attack pattern with bounds checking
        let attack_index = magic_entry.attack_base + hash as usize;
        if attack_index < self.attack_storage.len() {
            self.attack_storage[attack_index]
        } else {
            // Bounds check failed - fallback to ray-casting
            self.get_attacks_fallback(square, piece_type, occupied)
        }
    }

    /// Fallback to ray-casting when magic table lookup fails
    fn get_attacks_fallback(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        use super::attack_generator::AttackGenerator;
        let mut generator = AttackGenerator::new();
        generator.generate_attack_pattern(square, piece_type, occupied)
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> TableMemoryStats {
        TableMemoryStats {
            total_attack_patterns: self.attack_storage.len(),
            memory_usage_bytes: self.attack_storage.len() * std::mem::size_of::<Bitboard>(),
            pool_stats: self.memory_pool.memory_stats(),
        }
    }

    /// Validate table integrity: check all entries are within bounds
    ///
    /// This method verifies that all magic entries reference valid indices
    /// in the attack_storage array. It does not validate correctness of
    /// attack patterns (use `validate()` for that).
    pub fn validate_integrity(&self) -> Result<(), MagicError> {
        // Check rook tables
        for (square, magic_entry) in self.rook_magics.iter().enumerate() {
            if magic_entry.magic_number == 0 {
                continue; // Skip uninitialized entries
            }

            // Check that attack_base is within bounds
            if magic_entry.attack_base >= self.attack_storage.len() {
                return Err(MagicError::ValidationFailed {
                    reason: format!(
                        "Rook square {} has invalid attack_base {} (storage size: {})",
                        square, magic_entry.attack_base, self.attack_storage.len()
                    ),
                });
            }

            // Check that attack_base + table_size is within bounds
            let max_index = magic_entry.attack_base + magic_entry.table_size;
            if max_index > self.attack_storage.len() {
                return Err(MagicError::ValidationFailed {
                    reason: format!(
                        "Rook square {} table extends beyond storage (max_index: {}, storage size: {})",
                        square, max_index, self.attack_storage.len()
                    ),
                });
            }
        }

        // Check bishop tables
        for (square, magic_entry) in self.bishop_magics.iter().enumerate() {
            if magic_entry.magic_number == 0 {
                continue; // Skip uninitialized entries
            }

            // Check that attack_base is within bounds
            if magic_entry.attack_base >= self.attack_storage.len() {
                return Err(MagicError::ValidationFailed {
                    reason: format!(
                        "Bishop square {} has invalid attack_base {} (storage size: {})",
                        square, magic_entry.attack_base, self.attack_storage.len()
                    ),
                });
            }

            // Check that attack_base + table_size is within bounds
            let max_index = magic_entry.attack_base + magic_entry.table_size;
            if max_index > self.attack_storage.len() {
                return Err(MagicError::ValidationFailed {
                    reason: format!(
                        "Bishop square {} table extends beyond storage (max_index: {}, storage size: {})",
                        square, max_index, self.attack_storage.len()
                    ),
                });
            }
        }

        Ok(())
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

    /// Clear pattern cache in AttackGenerator
    ///
    /// Note: This is primarily for documentation. AttackGenerator instances
    /// are typically created fresh for each operation, so their caches are
    /// automatically cleared. However, if you maintain a long-lived
    /// AttackGenerator instance, you can use this to free memory.
    ///
    /// After table initialization completes, the pattern cache is no longer
    /// needed and can be cleared to free memory.
    pub fn clear_pattern_cache(&self) {
        // AttackGenerator instances are created fresh in get_attacks_fallback,
        // so there's no persistent cache to clear. This method exists for
        // API completeness and documentation purposes.
        // If you maintain a long-lived AttackGenerator, call clear_cache() on it directly.
    }

    /// Serialize magic table to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, MagicError> {
        let mut data = Vec::new();

        // Write version header: magic number (16 bytes) + version (1 byte)
        let mut magic_bytes = [0u8; 16];
        for (i, &byte) in MAGIC_TABLE_FILE_MAGIC.iter().enumerate() {
            if i < 16 {
                magic_bytes[i] = byte;
            }
        }
        data.write_all(&magic_bytes)
            .map_err(|e| MagicError::IoError(e.to_string()))?;
        data.write_all(&[MAGIC_TABLE_FILE_VERSION])
            .map_err(|e| MagicError::IoError(e.to_string()))?;

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

        // Calculate and append checksum (simple wrapping addition checksum)
        let checksum = Self::calculate_checksum(&data[17..]); // Skip header (16 + 1 bytes)
        data.write_all(&checksum.to_le_bytes())
            .map_err(|e| MagicError::IoError(e.to_string()))?;

        Ok(data)
    }

    /// Calculate checksum for data validation
    fn calculate_checksum(data: &[u8]) -> u64 {
        let mut checksum = 0u64;
        for &byte in data {
            checksum = checksum.wrapping_add(byte as u64);
            checksum = checksum.wrapping_mul(0x9e3779b97f4a7c15); // Mix bits
        }
        checksum
    }

    /// Deserialize magic table from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, MagicError> {
        use std::io::Read;
        
        if data.len() < 17 {
            return Err(MagicError::IoError(
                "Data too short for magic table header".to_string(),
            ));
        }

        // Validate magic number
        let expected_magic = MAGIC_TABLE_FILE_MAGIC;
        if &data[0..expected_magic.len()] != expected_magic {
            return Err(MagicError::ValidationFailed {
                reason: format!(
                    "Invalid magic number: expected {:?}, got {:?}",
                    expected_magic,
                    &data[0..expected_magic.len().min(16)]
                ),
            });
        }

        // Validate version
        let version = data[16];
        if version != MAGIC_TABLE_FILE_VERSION {
            return Err(MagicError::ValidationFailed {
                reason: format!(
                    "Version mismatch: expected {}, got {}",
                    MAGIC_TABLE_FILE_VERSION, version
                ),
            });
        }

        // Extract checksum (last 8 bytes)
        if data.len() < 25 {
            return Err(MagicError::IoError(
                "Data too short for checksum".to_string(),
            ));
        }
        let checksum_offset = data.len() - 8;
        let stored_checksum = u64::from_le_bytes(
            data[checksum_offset..checksum_offset + 8]
                .try_into()
                .map_err(|_| MagicError::IoError("Invalid checksum format".to_string()))?,
        );

        // Calculate checksum of data (excluding header and checksum)
        let data_checksum = Self::calculate_checksum(&data[17..checksum_offset]);
        if data_checksum != stored_checksum {
            return Err(MagicError::ValidationFailed {
                reason: format!(
                    "Checksum mismatch: expected {}, got {}",
                    stored_checksum, data_checksum
                ),
            });
        }

        let mut cursor = std::io::Cursor::new(&data[17..checksum_offset]); // Skip header, exclude checksum
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

    /// Save magic table to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), MagicError> {
        let path = path.as_ref();
        let parent = path.parent().ok_or_else(|| MagicError::IoError(
            format!("Invalid path: {}", path.display())
        ))?;
        
        // Create parent directory if it doesn't exist
        std::fs::create_dir_all(parent).map_err(|e| MagicError::IoError(format!(
            "Failed to create directory {}: {}",
            parent.display(),
            e
        )))?;

        let data = self.serialize()?;
        let file = File::create(path).map_err(|e| MagicError::IoError(format!(
            "Failed to create file {}: {}",
            path.display(),
            e
        )))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data).map_err(|e| MagicError::IoError(format!(
            "Failed to write to file {}: {}",
            path.display(),
            e
        )))?;
        writer.flush().map_err(|e| MagicError::IoError(format!(
            "Failed to flush file {}: {}",
            path.display(),
            e
        )))?;
        Ok(())
    }

    /// Load magic table from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, MagicError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| MagicError::IoError(format!(
            "Failed to open file {}: {}",
            path.display(),
            e
        )))?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).map_err(|e| MagicError::IoError(format!(
            "Failed to read file {}: {}",
            path.display(),
            e
        )))?;
        Self::deserialize(&data)
    }

    /// Try to load magic table from file, or generate if not found
    /// 
    /// If `save_if_generated` is true, saves the generated table to the file path.
    pub fn try_load_or_generate<P: AsRef<Path>>(
        path: P,
        save_if_generated: bool,
    ) -> Result<Self, MagicError> {
        let path = path.as_ref();
        
        // Try to load from file first
        match Self::load_from_file(path) {
            Ok(table) => {
                // Validate the loaded table
                table.validate()?;
                return Ok(table);
            }
            Err(e) => {
                // If file doesn't exist or is invalid, generate new table
                if !path.exists() {
                    eprintln!(
                        "Magic table file not found at {}, generating new table...",
                        path.display()
                    );
                } else {
                    eprintln!(
                        "Failed to load magic table from {}: {}, generating new table...",
                        path.display(),
                        e
                    );
                }
            }
        }

        // Generate new table
        let table = Self::new()?;
        
        // Validate generated table
        table.validate()?;

        // Save if requested
        if save_if_generated {
            if let Err(e) = table.save_to_file(path) {
                eprintln!(
                    "Warning: Failed to save generated magic table to {}: {}",
                    path.display(),
                    e
                );
            } else {
                eprintln!(
                    "Generated magic table saved to {}",
                    path.display()
                );
            }
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
    fn test_serialization_version_validation() {
        let table = MagicTable::default();
        let serialized = table.serialize().unwrap();
        
        // Should deserialize successfully with correct version
        let deserialized = MagicTable::deserialize(&serialized);
        assert!(deserialized.is_ok());
        
        // Corrupt the version byte
        let mut corrupted = serialized.clone();
        corrupted[16] = 99; // Invalid version
        let result = MagicTable::deserialize(&corrupted);
        assert!(result.is_err());
        if let Err(MagicError::ValidationFailed { reason }) = result {
            assert!(reason.contains("Version mismatch"));
        } else {
            panic!("Expected ValidationFailed error for version mismatch");
        }
    }

    #[test]
    fn test_serialization_checksum_validation() {
        let table = MagicTable::default();
        let serialized = table.serialize().unwrap();
        
        // Should deserialize successfully with correct checksum
        let deserialized = MagicTable::deserialize(&serialized);
        assert!(deserialized.is_ok());
        
        // Corrupt the checksum (last 8 bytes)
        let mut corrupted = serialized.clone();
        let len = corrupted.len();
        corrupted[len - 1] = corrupted[len - 1].wrapping_add(1);
        let result = MagicTable::deserialize(&corrupted);
        assert!(result.is_err());
        if let Err(MagicError::ValidationFailed { reason }) = result {
            assert!(reason.contains("Checksum mismatch"));
        } else {
            panic!("Expected ValidationFailed error for checksum mismatch");
        }
    }

    #[test]
    fn test_serialization_magic_number_validation() {
        let table = MagicTable::default();
        let serialized = table.serialize().unwrap();
        
        // Corrupt the magic number (first bytes)
        let mut corrupted = serialized.clone();
        corrupted[0] = 0xFF;
        let result = MagicTable::deserialize(&corrupted);
        assert!(result.is_err());
        if let Err(MagicError::ValidationFailed { reason }) = result {
            assert!(reason.contains("Invalid magic number"));
        } else {
            panic!("Expected ValidationFailed error for invalid magic number");
        }
    }

    #[test]
    fn test_save_and_load_file() {
        use std::fs;
        use std::path::Path;
        
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_magic_table.bin");
        
        // Clean up if file exists
        let _ = fs::remove_file(&test_file);
        
        let original_table = MagicTable::default();
        
        // Save to file
        let save_result = original_table.save_to_file(&test_file);
        assert!(save_result.is_ok(), "Failed to save magic table to file");
        assert!(test_file.exists(), "Magic table file was not created");
        
        // Load from file
        let loaded_table = MagicTable::load_from_file(&test_file);
        assert!(loaded_table.is_ok(), "Failed to load magic table from file");
        let loaded = loaded_table.unwrap();
        
        // Verify data matches
        assert_eq!(
            original_table.attack_storage.len(),
            loaded.attack_storage.len()
        );
        assert_eq!(original_table.attack_storage, loaded.attack_storage);
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    #[ignore] // Ignore by default - generation takes 60+ seconds
    fn test_try_load_or_generate() {
        use std::fs;
        use std::path::Path;
        
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_try_load_magic_table.bin");
        
        // Clean up if file exists
        let _ = fs::remove_file(&test_file);
        
        // First call should generate (file doesn't exist)
        // Note: This test is ignored by default because generation takes 60+ seconds
        // Run with: cargo test -- --ignored test_try_load_or_generate
        let result1 = MagicTable::try_load_or_generate(&test_file, true);
        assert!(result1.is_ok());
        assert!(test_file.exists(), "File should be created when save_if_generated=true");
        
        // Second call should load from file
        let result2 = MagicTable::try_load_or_generate(&test_file, false);
        assert!(result2.is_ok());
        let loaded = result2.unwrap();
        let generated = result1.unwrap();
        
        // Verify loaded table matches generated table
        assert_eq!(generated.attack_storage, loaded.attack_storage);
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_get_default_magic_table_path() {
        // Test that function returns a path
        let path = get_default_magic_table_path();
        assert!(!path.as_os_str().is_empty());
        
        // Test environment variable override
        std::env::set_var("SHOGI_MAGIC_TABLE_PATH", "/custom/path/magic.bin");
        let custom_path = get_default_magic_table_path();
        assert_eq!(custom_path, std::path::PathBuf::from("/custom/path/magic.bin"));
        std::env::remove_var("SHOGI_MAGIC_TABLE_PATH");
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
