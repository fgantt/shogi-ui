//! Magic Bitboards Module
//!
//! This module provides a complete implementation of magic bitboards for efficient
//! sliding piece move generation in Shogi. Magic bitboards use precomputed lookup
//! tables with carefully chosen magic numbers to hash occupied squares, providing
//! 3-5x faster sliding piece move generation compared to traditional ray-casting.
//!
//! # Overview
//!
//! Magic bitboards work by:
//! 1. Precomputing attack patterns for all possible blocker configurations
//! 2. Using magic numbers to hash blocker configurations into table indices
//! 3. Storing attack patterns in lookup tables for O(1) access
//!
//! # Components
//!
//! - `magic_finder`: Magic number generation and validation
//! - `attack_generator`: Attack pattern generation using ray-casting
//! - `magic_table`: Magic table construction and management
//! - `lookup_engine`: Fast lookup implementation with caching
//! - `validator`: Validation and correctness testing
//! - `memory_pool`: Efficient memory management for attack tables
//!
//! # Usage
//!
//! ```rust
//! use crate::bitboards::magic::MagicTable;
//! use crate::types::{PieceType, Bitboard};
//!
//! // Create magic table
//! let magic_table = MagicTable::new()?;
//!
//! // Get attack pattern for a square
//! let attacks = magic_table.get_attacks(square, PieceType::Rook, occupied);
//! ```

pub mod attack_generator;
pub mod magic_finder;
pub mod magic_table;
pub mod lookup_engine;
pub mod adaptive_cache;
pub mod compressed_table;
pub mod lazy_init;
pub mod memory_mapped;
pub mod memory_pool;
pub mod parallel_init;
pub mod performance_monitor;
pub mod validator;

// Re-export main types for convenience
pub use attack_generator::AttackGenerator;
pub use magic_finder::MagicFinder;
pub use lookup_engine::LookupEngine;
pub use memory_mapped::{MemoryMappedMagicTable, MemoryMappedStats};
pub use adaptive_cache::{AdaptiveCache, CacheStats};
pub use compressed_table::CompressedMagicTable;
pub use lazy_init::{LazyInitStats, LazyMagicTable};
pub use parallel_init::ParallelInitializer;
pub use performance_monitor::{AdaptiveOptimizer, MonitorStats, PerformanceMonitor};
pub use validator::MagicValidator;

// Re-export types from the main types module
pub use crate::types::{
    AttackConfig, MagicBitboard, MagicError, MagicGenerationResult, PerformanceMetrics,
};

/// Initialize the magic bitboard system
///
/// This function should be called once during application startup
/// to initialize the magic bitboard system.
pub fn initialize() -> Result<(), MagicError> {
    // Placeholder for system initialization
    // In a real implementation, this might precompute magic tables
    // or perform other initialization tasks
    Ok(())
}

/// Get system information about magic bitboards
pub fn system_info() -> SystemInfo {
    SystemInfo {
        version: env!("CARGO_PKG_VERSION"),
        magic_table_size: 81 * 2, // 81 squares * 2 piece types
        memory_usage: 0,          // Will be updated when tables are created
        initialized: false,       // Will be updated when system is initialized
    }
}

/// System information for magic bitboards
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: &'static str,
    pub magic_table_size: usize,
    pub memory_usage: usize,
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let result = initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_info() {
        let info = system_info();
        assert_eq!(info.magic_table_size, 162); // 81 * 2
        assert!(!info.initialized);
    }
}
