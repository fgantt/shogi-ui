//! Lazy initialization for magic bitboards
//!
//! This module provides lazy initialization that generates magic tables
//! on-demand when first accessed, rather than pre-generating all tables.

use crate::types::{Bitboard, MagicError, MagicTable, PieceType, EMPTY_BITBOARD};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

/// Lazy magic table that initializes squares on-demand
pub struct LazyMagicTable {
    /// Base magic table (partially initialized)
    table: Arc<Mutex<MagicTable>>,
    /// Track which rook squares are initialized
    rook_initialized: Arc<Mutex<HashSet<u8>>>,
    /// Track which bishop squares are initialized
    bishop_initialized: Arc<Mutex<HashSet<u8>>>,
    /// Statistics tracking
    stats: Arc<Mutex<LazyInitStats>>,
}

/// Statistics for lazy initialization
#[derive(Debug, Clone, Default)]
pub struct LazyInitStats {
    /// Number of rook squares initialized
    pub rook_squares_initialized: usize,
    /// Number of bishop squares initialized
    pub bishop_squares_initialized: usize,
    /// Total number of lazy initializations
    pub lazy_init_count: usize,
    /// Squares that were actually accessed
    pub accessed_squares: HashSet<(u8, PieceType)>,
}

impl LazyMagicTable {
    /// Create a new lazy magic table
    pub fn new() -> Result<Self, MagicError> {
        let table = MagicTable::default();
        Ok(Self {
            table: Arc::new(Mutex::new(table)),
            rook_initialized: Arc::new(Mutex::new(HashSet::new())),
            bishop_initialized: Arc::new(Mutex::new(HashSet::new())),
            stats: Arc::new(Mutex::new(LazyInitStats::default())),
        })
    }

    /// Get attacks with lazy initialization
    pub fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: Bitboard) -> Bitboard {
        // Track access
        {
            let mut stats = self.stats.lock().unwrap();
            stats.accessed_squares.insert((square, piece_type));
        }

        // Check if square needs initialization
        let needs_init = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                let initialized = self.rook_initialized.lock().unwrap();
                !initialized.contains(&square)
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                let initialized = self.bishop_initialized.lock().unwrap();
                !initialized.contains(&square)
            }
            _ => false,
        };

        if needs_init {
            // Initialize the square
            if let Err(e) = self.initialize_square(square, piece_type) {
                eprintln!("Failed to initialize square {} for {:?}: {:?}", square, piece_type, e);
                return EMPTY_BITBOARD;
            }
        }

        // Get attacks from table
        let table = self.table.lock().unwrap();
        table.get_attacks(square, piece_type, occupied)
    }

    /// Initialize a specific square
    fn initialize_square(&self, square: u8, piece_type: PieceType) -> Result<(), MagicError> {
        let mut table = self.table.lock().unwrap();
        
        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                table.initialize_rook_square(square)?;
                let mut initialized = self.rook_initialized.lock().unwrap();
                initialized.insert(square);
                let mut stats = self.stats.lock().unwrap();
                stats.rook_squares_initialized += 1;
                stats.lazy_init_count += 1;
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                table.initialize_bishop_square(square)?;
                let mut initialized = self.bishop_initialized.lock().unwrap();
                initialized.insert(square);
                let mut stats = self.stats.lock().unwrap();
                stats.bishop_squares_initialized += 1;
                stats.lazy_init_count += 1;
            }
            _ => {}
        }

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> LazyInitStats {
        self.stats.lock().unwrap().clone()
    }

    /// Check if a square is initialized
    pub fn is_square_initialized(&self, square: u8, piece_type: PieceType) -> bool {
        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                let initialized = self.rook_initialized.lock().unwrap();
                initialized.contains(&square)
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                let initialized = self.bishop_initialized.lock().unwrap();
                initialized.contains(&square)
            }
            _ => false,
        }
    }

    /// Pre-initialize all squares (convert to fully initialized table)
    pub fn pre_initialize_all(&self) -> Result<(), MagicError> {
        let mut table = self.table.lock().unwrap();
        
        // Initialize all rook squares
        for square in 0..81 {
            if !self.is_square_initialized(square, PieceType::Rook) {
                table.initialize_rook_square(square)?;
                let mut initialized = self.rook_initialized.lock().unwrap();
                initialized.insert(square);
            }
        }

        // Initialize all bishop squares
        for square in 0..81 {
            if !self.is_square_initialized(square, PieceType::Bishop) {
                table.initialize_bishop_square(square)?;
                let mut initialized = self.bishop_initialized.lock().unwrap();
                initialized.insert(square);
            }
        }

        Ok(())
    }

    /// Get the underlying table (for full access)
    pub fn into_table(self) -> MagicTable {
        Arc::try_unwrap(self.table)
            .unwrap_or_else(|_| panic!("Multiple references to table"))
            .into_inner()
            .unwrap()
    }
}

impl Default for LazyMagicTable {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_table_creation() {
        let table = LazyMagicTable::new();
        assert!(table.is_ok());
    }

    #[test]
    fn test_lazy_initialization() {
        let table = LazyMagicTable::new().unwrap();
        
        // Initially not initialized
        assert!(!table.is_square_initialized(0, PieceType::Rook));
        
        // Access triggers initialization
        let _attacks = table.get_attacks(0, PieceType::Rook, 0);
        
        // Now initialized
        assert!(table.is_square_initialized(0, PieceType::Rook));
    }

    #[test]
    fn test_lazy_stats() {
        let table = LazyMagicTable::new().unwrap();
        
        // Access a few squares
        let _ = table.get_attacks(0, PieceType::Rook, 0);
        let _ = table.get_attacks(40, PieceType::Bishop, 0);
        
        let stats = table.stats();
        assert!(stats.lazy_init_count >= 2);
        assert!(stats.accessed_squares.len() >= 2);
    }
}

