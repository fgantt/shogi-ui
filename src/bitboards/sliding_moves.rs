//! Magic bitboard-based sliding piece move generation
//! 
//! This module provides optimized move generation for sliding pieces (rook, bishop)
//! using magic bitboards for maximum performance.

use crate::types::{PieceType, Position, Player, Move};
use crate::types::MagicTable;
use crate::bitboards::BitboardBoard;

// Simple immutable lookup engine
#[derive(Clone)]
struct SimpleLookupEngine {
    magic_table: MagicTable,
}

impl SimpleLookupEngine {
    fn new(magic_table: MagicTable) -> Self {
        Self { magic_table }
    }

    fn get_attacks(&self, square: u8, piece_type: PieceType, occupied: u128) -> u128 {
        self.magic_table.get_attacks(square, piece_type, occupied)
    }
}

/// Magic-based sliding move generator
/// 
/// This is a stateless generator that uses magic bitboards for fast move generation.
/// Metrics are tracked externally to maintain immutability.
#[derive(Clone)]
pub struct SlidingMoveGenerator {
    /// Lookup engine for magic bitboard operations
    lookup_engine: SimpleLookupEngine,
    /// Feature flag for enabling/disabling magic bitboards
    magic_enabled: bool,
}

impl SlidingMoveGenerator {
    /// Create a new sliding move generator
    pub fn new(magic_table: MagicTable) -> Self {
        Self {
            lookup_engine: SimpleLookupEngine::new(magic_table),
            magic_enabled: true,
        }
    }

    /// Create a new sliding move generator with custom settings
    pub fn with_settings(magic_table: MagicTable, magic_enabled: bool) -> Self {
        Self {
            lookup_engine: SimpleLookupEngine::new(magic_table),
            magic_enabled,
        }
    }

    /// Generate moves for a sliding piece using magic bitboards
    /// 
    /// This is a pure function with no side effects, making it safe for immutable usage.
    pub fn generate_sliding_moves(
        &self,
        board: &BitboardBoard,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Move> {
        if !self.magic_enabled {
            return Vec::new(); // Fallback to ray-casting handled by caller
        }

        let mut moves = Vec::new();
        let occupied = board.get_occupied_bitboard();
        let square = from.to_index();

        // Get attack pattern using magic bitboards
        let attacks = self.lookup_engine.get_attacks(square, piece_type, occupied);

        // Convert attack pattern to moves
        for target_square in 0..81 {
            if (attacks & (1u128 << target_square)) != 0 {
                let target_pos = Position::from_index(target_square as u8);
                
                // Check if target square is occupied by own piece
                if board.is_occupied_by_player(target_pos, player) {
                    continue;
                }

                // Create move
                let move_ = Move::new_move(from, target_pos, piece_type, player, false);
                moves.push(move_);
            }
        }

        moves
    }

    /// Generate moves for promoted sliding pieces
    /// 
    /// This is a pure function with no side effects, making it safe for immutable usage.
    pub fn generate_promoted_sliding_moves(
        &self,
        board: &BitboardBoard,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Move> {
        if !self.magic_enabled {
            return Vec::new(); // Fallback to ray-casting handled by caller
        }

        let mut moves = Vec::new();
        let occupied = board.get_occupied_bitboard();
        let square = from.to_index();

        // Get attack pattern using magic bitboards
        let attacks = self.lookup_engine.get_attacks(square, piece_type, occupied);

        // Convert attack pattern to moves
        for target_square in 0..81 {
            if (attacks & (1u128 << target_square)) != 0 {
                let target_pos = Position::from_index(target_square as u8);
                
                // Check if target square is occupied by own piece
                if board.is_occupied_by_player(target_pos, player) {
                    continue;
                }

                // Create promoted move
                let move_ = Move::new_move(from, target_pos, piece_type, player, true);
                moves.push(move_);
            }
        }

        moves
    }

    /// Generate moves for multiple sliding pieces in batch
    /// 
    /// This is a pure function with no side effects, making it safe for immutable usage.
    pub fn generate_sliding_moves_batch(
        &self,
        board: &BitboardBoard,
        pieces: &[(Position, PieceType)],
        player: Player,
    ) -> Vec<Move> {
        if !self.magic_enabled {
            return Vec::new(); // Fallback to ray-casting handled by caller
        }

        let mut all_moves = Vec::new();
        let occupied = board.get_occupied_bitboard();

        // Use batch lookup for performance
        for &(from, piece_type) in pieces {
            let square = from.to_index();
            let attacks = self.lookup_engine.get_attacks(square, piece_type, occupied);

            // Convert attack pattern to moves
            for target_square in 0..81 {
                if (attacks & (1u128 << target_square)) != 0 {
                    let target_pos = Position::from_index(target_square as u8);
                    
                    // Check if target square is occupied by own piece
                    if board.is_occupied_by_player(target_pos, player) {
                        continue;
                    }

                    // Create move
                    let move_ = Move::new_move(from, target_pos, piece_type, player, false);
                    all_moves.push(move_);
                }
            }
        }

        all_moves
    }

    /// Check if magic bitboards are enabled
    pub fn is_magic_enabled(&self) -> bool {
        self.magic_enabled
    }

    /// Get lookup engine reference
    pub fn get_lookup_engine(&self) -> &SimpleLookupEngine {
        &self.lookup_engine
    }
}

/// Feature flags for magic bitboard integration
pub struct MagicBitboardFlags {
    pub magic_enabled: bool,
    pub batch_processing: bool,
    pub prefetching: bool,
    pub fallback_enabled: bool,
}

impl Default for MagicBitboardFlags {
    fn default() -> Self {
        Self {
            magic_enabled: true,
            batch_processing: true,
            prefetching: true,
            fallback_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MagicTable;

    #[test]
    fn test_sliding_move_generator_creation() {
        let magic_table = MagicTable::default();
        let generator = SlidingMoveGenerator::new(magic_table);
        
        assert!(generator.is_magic_enabled());
    }

    #[test]
    fn test_sliding_move_generator_with_settings() {
        let magic_table = MagicTable::default();
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        
        assert!(!generator.is_magic_enabled());
    }

    #[test]
    fn test_magic_enabled_toggle() {
        let magic_table = MagicTable::default();
        let generator = SlidingMoveGenerator::new(magic_table.clone());
        
        assert!(generator.is_magic_enabled());
        
        let generator_disabled = SlidingMoveGenerator::with_settings(magic_table, false);
        assert!(!generator_disabled.is_magic_enabled());
    }

    #[test]
    fn test_basic_functionality() {
        let magic_table = MagicTable::default();
        let generator = SlidingMoveGenerator::new(magic_table.clone());
        
        // Test basic functionality
        assert!(generator.is_magic_enabled());
        
        let generator_disabled = SlidingMoveGenerator::with_settings(magic_table, false);
        assert!(!generator_disabled.is_magic_enabled());
    }

    #[test]
    fn test_magic_bitboard_flags() {
        let flags = MagicBitboardFlags::default();
        
        assert!(flags.magic_enabled);
        assert!(flags.batch_processing);
        assert!(flags.prefetching);
        assert!(flags.fallback_enabled);
    }
}
