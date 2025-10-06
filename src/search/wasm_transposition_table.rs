//! WASM-Optimized Transposition Table
//! 
//! This module provides a transposition table implementation specifically
//! optimized for WebAssembly environments. It uses single-threaded data
//! structures and WASM-compatible time handling for optimal performance.
//! 
//! # Features
//! 
//! - **Single-Threaded Optimization**: No synchronization overhead
//! - **WASM-Compatible Time**: Uses performance.now() instead of std::time
//! - **Memory Efficient**: Optimized for WASM memory constraints
//! - **Fast Operations**: Direct memory access without atomic operations
//! - **Compact Storage**: Minimal memory footprint per entry

use crate::types::*;
use crate::search::transposition_config::TranspositionConfig;
use crate::search::wasm_compatibility::{WasmTime, WasmMemoryManager, WasmTranspositionConfig};

/// WASM-optimized transposition table entry
/// 
/// Uses compact storage optimized for WASM environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmTranspositionEntry {
    /// Hash key (48 bits) + depth (8 bits) + flag (8 bits)
    hash_depth_flag: u64,
    /// Score (32 bits) + best move (32 bits)
    score_move: u64,
    /// Age counter
    age: u32,
}

impl WasmTranspositionEntry {
    /// Create a new entry
    pub fn new(hash: u64, depth: u8, score: i32, flag: TranspositionFlag, best_move: Option<Move>, age: u32) -> Self {
        // Pack hash (48 bits), depth (8 bits), flag (8 bits)
        let hash_packed = (hash & 0x0000FFFFFFFFFFFF) << 16;
        let depth_packed = (depth as u64) << 8;
        let flag_packed = flag as u8 as u64;
        let hash_depth_flag = hash_packed | depth_packed | flag_packed;
        
        // Pack score (32 bits) and best move (32 bits)
        let score_packed = (score as u32 as u64) << 32;
        let move_packed = Self::pack_move(best_move);
        let score_move = score_packed | move_packed;
        
        Self {
            hash_depth_flag,
            score_move,
            age,
        }
    }
    
    /// Extract hash key
    pub fn hash(&self) -> u64 {
        (self.hash_depth_flag >> 16) & 0x0000FFFFFFFFFFFF
    }
    
    /// Extract depth
    pub fn depth(&self) -> u8 {
        ((self.hash_depth_flag >> 8) & 0xFF) as u8
    }
    
    /// Extract flag
    pub fn flag(&self) -> TranspositionFlag {
        match self.hash_depth_flag & 0xFF {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            2 => TranspositionFlag::UpperBound,
            _ => TranspositionFlag::Exact,
        }
    }
    
    /// Extract score
    pub fn score(&self) -> i32 {
        ((self.score_move >> 32) & 0xFFFFFFFF) as i32
    }
    
    /// Extract best move
    pub fn best_move(&self) -> Option<Move> {
        let move_data = (self.score_move & 0xFFFFFFFF) as u32;
        if move_data == 0 {
            None
        } else {
            Self::unpack_move(move_data)
        }
    }
    
    /// Get age
    pub fn age(&self) -> u32 {
        self.age
    }
    
    /// Check if entry is valid
    pub fn is_valid(&self) -> bool {
        self.hash_depth_flag != 0 || self.score_move != 0
    }
    
    /// Create empty entry
    pub fn empty() -> Self {
        Self {
            hash_depth_flag: 0,
            score_move: 0,
            age: 0,
        }
    }
    
    /// Pack move into 32 bits
    fn pack_move(move_: Option<Move>) -> u64 {
        if let Some(move_) = move_ {
            let from = move_.from.map(|p| p.to_u8()).unwrap_or(0);
            let to = move_.to.to_u8();
            let piece_type = move_.piece_type as u8;
            let player = move_.player as u8;
            let flags = (move_.is_promotion as u8) << 3 | (move_.is_capture as u8) << 2 | 
                       (move_.gives_check as u8) << 1 | (move_.is_recapture as u8);
            
            (((from as u32) << 24) | ((to as u32) << 16) | ((piece_type as u32) << 8) | 
            ((player as u32) << 4) | (flags as u32)) as u64
        } else {
            0
        }
    }
    
    /// Unpack move from 32 bits
    fn unpack_move(move_data: u32) -> Option<Move> {
        if move_data == 0 {
            return None;
        }
        
        let from_raw = ((move_data >> 24) & 0xFF) as u8;
        let to_raw = ((move_data >> 16) & 0xFF) as u8;
        let piece_type_raw = ((move_data >> 8) & 0xFF) as u8;
        let player_raw = ((move_data >> 4) & 0xF) as u8;
        let flags = move_data & 0xF;
        
        let from = if from_raw == 0 { None } else { Some(Position::from_u8(from_raw)) };
        let to = Position::from_u8(to_raw);
        let piece_type = match piece_type_raw {
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
        let player = match player_raw {
            0 => Player::Black,
            1 => Player::White,
            _ => Player::Black,
        };
        
        Some(Move {
            from,
            to,
            piece_type,
            player,
            is_promotion: (flags & 0x8) != 0,
            is_capture: (flags & 0x4) != 0,
            captured_piece: None, // Not stored in compact format
            gives_check: (flags & 0x2) != 0,
            is_recapture: (flags & 0x1) != 0,
        })
    }
}

/// WASM-optimized transposition table statistics
#[derive(Debug, Clone, Default)]
pub struct WasmTranspositionStats {
    /// Number of hits
    pub hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Number of stores
    pub stores: u64,
    /// Number of replacements
    pub replacements: u64,
    /// Number of memory pressure skips
    pub memory_pressure_skips: u64,
    /// Current age
    pub current_age: u32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Hit rate
    pub hit_rate: f64,
    /// Average operation time in microseconds
    pub avg_operation_time_us: f64,
}

impl WasmTranspositionStats {
    /// Update hit rate
    pub fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.stores = 0;
        self.replacements = 0;
        self.hit_rate = 0.0;
        self.avg_operation_time_us = 0.0;
    }
}

/// WASM-optimized transposition table
/// 
/// A high-performance transposition table optimized for single-threaded
/// WASM environments with minimal memory overhead.
pub struct WasmTranspositionTable {
    /// Hash table entries
    entries: Vec<WasmTranspositionEntry>,
    /// Table size (power of 2)
    #[allow(dead_code)]
    size: usize,
    /// Size mask for fast modulo
    mask: usize,
    /// Current age counter
    age: u32,
    /// Statistics
    stats: WasmTranspositionStats,
    /// Memory manager
    memory_manager: WasmMemoryManager,
    /// Configuration
    config: WasmTranspositionConfig,
    /// Operation timing
    #[allow(dead_code)]
    operation_start: WasmTime,
}

impl WasmTranspositionTable {
    /// Create a new WASM-optimized transposition table
    pub fn new(config: WasmTranspositionConfig) -> Self {
        let size = config.base_config.table_size.next_power_of_two();
        let mask = size - 1;
        
        // Initialize entries
        let mut entries = Vec::with_capacity(size);
        for _ in 0..size {
            entries.push(WasmTranspositionEntry::empty());
        }
        
        let memory_usage = size * std::mem::size_of::<WasmTranspositionEntry>();
        let mut memory_manager = WasmMemoryManager::with_limit((config.max_memory_mb * 1024 * 1024) as u64);
        memory_manager.allocate("transposition_table", memory_usage as u64).unwrap_or_default();
        
        Self {
            entries,
            size,
            mask,
            age: 0,
            stats: WasmTranspositionStats {
                memory_usage: memory_usage as u64,
                ..Default::default()
            },
            memory_manager,
            config,
            operation_start: WasmTime::now(),
        }
    }
    
    /// Create from standard configuration
    pub fn from_config(config: TranspositionConfig) -> Self {
        Self::new(config.into())
    }
    
    /// Probe for an entry
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TranspositionEntry> {
        let start_time = WasmTime::now();
        
        let index = self.get_index(hash);
        let entry = &self.entries[index];
        
        // Check if entry matches
        if entry.hash() != hash {
            self.stats.misses += 1;
            self.update_operation_time(start_time);
            return None;
        }
        
        // Check if entry is valid
        if !entry.is_valid() {
            self.stats.misses += 1;
            self.update_operation_time(start_time);
            return None;
        }
        
        // Check depth requirement
        if entry.depth() < depth {
            self.stats.misses += 1;
            self.update_operation_time(start_time);
            return None;
        }
        
        // Convert to standard entry
        let result = TranspositionEntry {
            hash_key: hash,
            depth: entry.depth(),
            score: entry.score(),
            flag: entry.flag(),
            best_move: entry.best_move(),
            age: entry.age(),
        };
        
        self.stats.hits += 1;
        self.update_operation_time(start_time);
        Some(result)
    }
    
    /// Store an entry
    pub fn store(&mut self, entry: TranspositionEntry) {
        let start_time = WasmTime::now();
        
        // Check memory pressure before storing
        if self.is_memory_pressure() {
            // Skip storing if memory pressure is high
            self.stats.memory_pressure_skips += 1;
            return;
        }
        
        let hash = entry.hash_key;
        let index = self.get_index(hash);
        let current_entry = &self.entries[index];
        
        // Check if replacement is needed
        let should_replace = if !current_entry.is_valid() {
            true
        } else if current_entry.hash() == hash {
            true // Same position, always replace
        } else {
            // Use replacement policy
            self.should_replace(current_entry, &entry)
        };
        
        if should_replace {
            if current_entry.is_valid() && current_entry.hash() != hash {
                self.stats.replacements += 1;
            }
            
            self.entries[index] = WasmTranspositionEntry::new(
                hash,
                entry.depth,
                entry.score,
                entry.flag,
                entry.best_move,
                self.age,
            );
            
            self.stats.stores += 1;
        }
        
        self.update_operation_time(start_time);
    }
    
    /// Clear the table
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = WasmTranspositionEntry::empty();
        }
        self.age = 0;
        self.stats.reset();
    }
    
    /// Increment age counter
    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
        self.stats.current_age = self.age;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> WasmTranspositionStats {
        let mut stats = self.stats.clone();
        stats.update_hit_rate();
        stats
    }
    
    /// Get memory usage
    pub fn get_memory_usage(&self) -> u64 {
        self.stats.memory_usage
    }
    
    /// Check memory pressure
    pub fn is_memory_pressure(&self) -> bool {
        self.memory_manager.is_memory_pressure()
    }
    
    /// Force garbage collection hint
    pub fn hint_gc(&self) {
        if self.config.enable_gc_hints {
            self.memory_manager.force_gc();
        }
    }
    
    /// Get index for hash
    fn get_index(&self, hash: u64) -> usize {
        (hash as usize) & self.mask
    }
    
    /// Check if entry should be replaced
    fn should_replace(&self, current: &WasmTranspositionEntry, new: &TranspositionEntry) -> bool {
        match self.config.base_config.replacement_policy {
            crate::search::transposition_config::ReplacementPolicy::AlwaysReplace => true,
            crate::search::transposition_config::ReplacementPolicy::DepthPreferred => {
                new.depth >= current.depth()
            },
            crate::search::transposition_config::ReplacementPolicy::AgeBased => {
                // Always replace in age-based (newest wins)
                true
            },
            crate::search::transposition_config::ReplacementPolicy::ExactPreferred => {
                match new.flag {
                    TranspositionFlag::Exact => true,
                    _ => current.flag() != TranspositionFlag::Exact && new.depth >= current.depth(),
                }
            },
            crate::search::transposition_config::ReplacementPolicy::DepthAndAge => {
                // Combine depth and age for replacement decision
                new.depth >= current.depth() || self.age > current.age() + 100
            },
        }
    }
    
    /// Update operation timing
    fn update_operation_time(&mut self, start_time: WasmTime) {
        let elapsed = start_time.elapsed();
        let elapsed_us = elapsed.as_millis() * 1000; // Convert to microseconds
        
        // Update running average
        let total_ops = self.stats.hits + self.stats.misses + self.stats.stores;
        if total_ops > 0 {
            self.stats.avg_operation_time_us = 
                (self.stats.avg_operation_time_us * (total_ops - 1) as f64 + elapsed_us as f64) / total_ops as f64;
        }
    }
}

impl Default for WasmTranspositionTable {
    fn default() -> Self {
        Self::new(WasmTranspositionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_entry_creation() {
        let entry = WasmTranspositionEntry::new(
            0x123456789ABC,
            5,
            100,
            TranspositionFlag::Exact,
            None,
            10,
        );
        
        assert_eq!(entry.hash(), 0x123456789ABC);
        assert_eq!(entry.depth(), 5);
        assert_eq!(entry.score(), 100);
        assert_eq!(entry.flag(), TranspositionFlag::Exact);
        assert_eq!(entry.age(), 10);
    }
    
    #[test]
    fn test_wasm_entry_move_packing() {
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
        
        let entry = WasmTranspositionEntry::new(
            0x123456789ABC,
            5,
            100,
            TranspositionFlag::Exact,
            Some(move_.clone()),
            10,
        );
        
        let unpacked_move = entry.best_move().unwrap();
        assert_eq!(unpacked_move.from, move_.from);
        assert_eq!(unpacked_move.to, move_.to);
        assert_eq!(unpacked_move.piece_type, move_.piece_type);
        assert_eq!(unpacked_move.player, move_.player);
        assert_eq!(unpacked_move.is_promotion, move_.is_promotion);
        assert_eq!(unpacked_move.is_capture, move_.is_capture);
        assert_eq!(unpacked_move.gives_check, move_.gives_check);
        assert_eq!(unpacked_move.is_recapture, move_.is_recapture);
    }
    
    #[test]
    fn test_wasm_table_basic_operations() {
        let mut table = WasmTranspositionTable::default();
        
        let entry = TranspositionEntry {
            hash_key: 12345,
            depth: 3,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        
        // Store and probe
        table.store(entry.clone());
        
        let retrieved = table.probe(12345, 3);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.score, 100);
        assert_eq!(retrieved.depth, 3);
        assert_eq!(retrieved.flag, TranspositionFlag::Exact);
    }
    
    #[test]
    fn test_wasm_table_statistics() {
        let mut table = WasmTranspositionTable::default();
        
        let entry = TranspositionEntry {
            hash_key: 12345,
            depth: 3,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        
        table.store(entry.clone());
        table.probe(12345, 3);
        table.probe(54321, 3); // Miss
        
        let stats = table.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.stores, 1);
        assert!(stats.hit_rate > 0.0);
    }
    
    #[test]
    fn test_wasm_table_clear() {
        let mut table = WasmTranspositionTable::default();
        
        let entry = TranspositionEntry {
            hash_key: 12345,
            depth: 3,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        
        table.store(entry);
        table.clear();
        
        let retrieved = table.probe(12345, 3);
        assert!(retrieved.is_none());
        
        let stats = table.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.stores, 0);
    }
}
