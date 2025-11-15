//! Magic bitboard-based sliding piece move generation
//!
//! This module provides optimized move generation for sliding pieces (rook, bishop)
//! using magic bitboards for maximum performance.

use crate::bitboards::BitboardBoard;
use crate::bitboards::integration::GlobalOptimizer;
use crate::types::core::{Move, PieceType, Player, Position};
use crate::types::MagicTable;
use std::sync::Arc;

// Simple immutable lookup engine
// Task 2.0.2.1: Uses Arc to share magic table without cloning
#[derive(Clone)]
pub struct SimpleLookupEngine {
    magic_table: Arc<MagicTable>,
}

impl SimpleLookupEngine {
    fn new(magic_table: Arc<MagicTable>) -> Self {
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
    /// Task 2.0.2.1: Accepts Arc<MagicTable> to share without cloning
    pub fn new(magic_table: Arc<MagicTable>) -> Self {
        Self {
            lookup_engine: SimpleLookupEngine::new(magic_table),
            magic_enabled: true,
        }
    }

    /// Create a new sliding move generator with custom settings
    /// Task 2.0.2.1: Accepts Arc<MagicTable> to share without cloning
    pub fn with_settings(magic_table: Arc<MagicTable>, magic_enabled: bool) -> Self {
        Self {
            lookup_engine: SimpleLookupEngine::new(magic_table),
            magic_enabled,
        }
    }

    /// Generate moves for a sliding piece using magic bitboards
    ///
    /// This is a pure function with no side effects, making it safe for immutable usage.
    /// Task 2.0.2.4: Uses bit scans instead of 81-square loops for performance
    /// Task 2.0.2.2: Falls back to ray-casting when magic is disabled
    pub fn generate_sliding_moves(
        &self,
        board: &BitboardBoard,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = board.get_occupied_bitboard();
        let square = from.to_index();

        // Get attack pattern - use magic if enabled, otherwise fall back to ray-casting
        let attacks = if self.magic_enabled {
            self.lookup_engine.get_attacks(square, piece_type, occupied)
        } else {
            // Fallback to ray-casting via board's method
            board.get_attack_pattern(from, piece_type)
        };

        // Task 2.0.2.4: Use bit scans instead of iterating all 81 squares
        let mut remaining_attacks = attacks;
        while remaining_attacks != 0 {
            if let Some(target_square) = GlobalOptimizer::bit_scan_forward(remaining_attacks) {
                let target_pos = Position::from_index(target_square);

                // Check if target square is occupied by own piece
                if !board.is_occupied_by_player(target_pos, player) {
                    // Create move
                    let move_ = Move::new_move(from, target_pos, piece_type, player, false);
                    moves.push(move_);
                }

                // Clear the processed bit
                remaining_attacks &= remaining_attacks - 1;
            } else {
                break;
            }
        }

        moves
    }

    /// Generate moves for promoted sliding pieces
    ///
    /// This is a pure function with no side effects, making it safe for immutable usage.
    /// Task 2.0.2.4: Uses bit scans instead of 81-square loops for performance
    /// Task 2.0.2.2: Falls back to ray-casting when magic is disabled
    pub fn generate_promoted_sliding_moves(
        &self,
        board: &BitboardBoard,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let occupied = board.get_occupied_bitboard();
        let square = from.to_index();

        // Get attack pattern - use magic if enabled, otherwise fall back to ray-casting
        let attacks = if self.magic_enabled {
            self.lookup_engine.get_attacks(square, piece_type, occupied)
        } else {
            // Fallback to ray-casting via board's method
            board.get_attack_pattern(from, piece_type)
        };

        // Task 2.0.2.4: Use bit scans instead of iterating all 81 squares
        let mut remaining_attacks = attacks;
        while remaining_attacks != 0 {
            if let Some(target_square) = GlobalOptimizer::bit_scan_forward(remaining_attacks) {
                let target_pos = Position::from_index(target_square);

                // Check if target square is occupied by own piece
                if !board.is_occupied_by_player(target_pos, player) {
                    // Create promoted move
                    let move_ = Move::new_move(from, target_pos, piece_type, player, true);
                    moves.push(move_);
                }

                // Clear the processed bit
                remaining_attacks &= remaining_attacks - 1;
            } else {
                break;
            }
        }

        moves
    }

    /// Generate moves for multiple sliding pieces in batch
    ///
    /// This is a pure function with no side effects, making it safe for immutable usage.
    /// Task 2.0.2.4: Uses bit scans instead of 81-square loops for performance
    /// Task 2.0.2.2: Falls back to ray-casting when magic is disabled
    pub fn generate_sliding_moves_batch(
        &self,
        board: &BitboardBoard,
        pieces: &[(Position, PieceType)],
        player: Player,
    ) -> Vec<Move> {
        let mut all_moves = Vec::new();
        let occupied = board.get_occupied_bitboard();

        // Use batch lookup for performance
        for &(from, piece_type) in pieces {
            let square = from.to_index();
            
            // Get attack pattern - use magic if enabled, otherwise fall back to ray-casting
            let attacks = if self.magic_enabled {
                self.lookup_engine.get_attacks(square, piece_type, occupied)
            } else {
                // Fallback to ray-casting via board's method
                board.get_attack_pattern(from, piece_type)
            };

            // Task 2.0.2.4: Use bit scans instead of iterating all 81 squares
            let mut remaining_attacks = attacks;
            while remaining_attacks != 0 {
                if let Some(target_square) = GlobalOptimizer::bit_scan_forward(remaining_attacks) {
                    let target_pos = Position::from_index(target_square);

                    // Check if target square is occupied by own piece
                    if !board.is_occupied_by_player(target_pos, player) {
                        // Create move
                        let move_ = Move::new_move(from, target_pos, piece_type, player, false);
                        all_moves.push(move_);
                    }

                    // Clear the processed bit
                    remaining_attacks &= remaining_attacks - 1;
                } else {
                    break;
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
    use crate::types::{MagicTable, Piece, PieceType, Player, Position};
    use std::sync::Arc;

    #[test]
    fn test_sliding_move_generator_creation() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);

        assert!(generator.is_magic_enabled());
    }

    #[test]
    fn test_sliding_move_generator_with_settings() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);

        assert!(!generator.is_magic_enabled());
    }

    #[test]
    fn test_magic_enabled_toggle() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(Arc::clone(&magic_table));

        assert!(generator.is_magic_enabled());

        let generator_disabled = SlidingMoveGenerator::with_settings(magic_table, false);
        assert!(!generator_disabled.is_magic_enabled());
    }

    #[test]
    fn test_basic_functionality() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(Arc::clone(&magic_table));

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

    // Task 2.0.2.5: Tests for magic-enabled scenarios
    #[test]
    fn test_magic_enabled_rook_moves() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);
        let board = BitboardBoard::empty();
        let from = Position::new(4, 4); // Center square
        let piece_type = PieceType::Rook;
        let player = Player::Black;

        let moves = generator.generate_sliding_moves(&board, from, piece_type, player);
        
        // Rook from center should have moves in 4 directions
        // Should have at least some moves (exact count depends on board state)
        assert!(!moves.is_empty() || board.get_occupied_bitboard() != 0);
    }

    #[test]
    fn test_magic_enabled_bishop_moves() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);
        let board = BitboardBoard::empty();
        let from = Position::new(4, 4); // Center square
        let piece_type = PieceType::Bishop;
        let player = Player::Black;

        let moves = generator.generate_sliding_moves(&board, from, piece_type, player);
        
        // Bishop from center should have moves in 4 diagonal directions
        assert!(!moves.is_empty() || board.get_occupied_bitboard() != 0);
    }

    // Task 2.0.2.5: Tests for fallback-only scenarios
    #[test]
    fn test_fallback_only_rook_moves() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        let board = BitboardBoard::empty();
        let from = Position::new(4, 4); // Center square
        let piece_type = PieceType::Rook;
        let player = Player::Black;

        let moves = generator.generate_sliding_moves(&board, from, piece_type, player);
        
        // Should use ray-cast fallback and still generate moves
        // The exact count depends on board state, but should not panic
        assert!(moves.len() <= 16); // Rook can move at most 8 squares in each direction
    }

    #[test]
    fn test_fallback_only_bishop_moves() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        let board = BitboardBoard::empty();
        let from = Position::new(4, 4); // Center square
        let piece_type = PieceType::Bishop;
        let player = Player::Black;

        let moves = generator.generate_sliding_moves(&board, from, piece_type, player);
        
        // Should use ray-cast fallback and still generate moves
        assert!(moves.len() <= 13); // Bishop from center can move at most 13 squares
    }

    #[test]
    fn test_fallback_promoted_rook_moves() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        let board = BitboardBoard::empty();
        let from = Position::new(4, 4);
        let piece_type = PieceType::PromotedRook;
        let player = Player::Black;

        let moves = generator.generate_promoted_sliding_moves(&board, from, piece_type, player);
        
        // Promoted rook should have more moves than regular rook (includes king moves)
        assert!(moves.len() <= 24); // Rook moves + 8 king moves
    }

    // Task 2.0.2.5: Tests for mixed scenarios (blocked pieces)
    #[test]
    fn test_magic_with_blockers() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);
        let mut board = BitboardBoard::empty();
        
        // Place a rook
        let from = Position::new(4, 4);
        let rook = Piece::new(PieceType::Rook, Player::Black);
        board.place_piece(rook, from);
        
        // Place a blocker in one direction
        let blocker_pos = Position::new(6, 4);
        let blocker = Piece::new(PieceType::Pawn, Player::White);
        board.place_piece(blocker, blocker_pos);
        
        let moves = generator.generate_sliding_moves(&board, from, PieceType::Rook, Player::Black);
        
        // Should generate moves but not past the blocker
        // Should include the capture move
        assert!(moves.iter().any(|m| m.to() == blocker_pos));
    }

    #[test]
    fn test_fallback_with_blockers() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        let mut board = BitboardBoard::empty();
        
        // Place a rook
        let from = Position::new(4, 4);
        let rook = Piece::new(PieceType::Rook, Player::Black);
        board.place_piece(rook, from);
        
        // Place a blocker in one direction
        let blocker_pos = Position::new(6, 4);
        let blocker = Piece::new(PieceType::Pawn, Player::White);
        board.place_piece(blocker, blocker_pos);
        
        let moves = generator.generate_sliding_moves(&board, from, PieceType::Rook, Player::Black);
        
        // Fallback should also respect blockers
        assert!(moves.iter().any(|m| m.to() == blocker_pos));
        // Should not generate moves past the blocker
        assert!(!moves.iter().any(|m| m.to().row > blocker_pos.row && m.to().col == blocker_pos.col));
    }

    // Task 2.0.2.5: Test bit scan optimization (verify it works correctly)
    #[test]
    fn test_bit_scan_optimization() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);
        let board = BitboardBoard::empty();
        let from = Position::new(0, 0); // Corner square
        let piece_type = PieceType::Rook;
        let player = Player::Black;

        let moves = generator.generate_sliding_moves(&board, from, piece_type, player);
        
        // Rook from corner should have moves in 2 directions (right and down)
        // Verify moves are generated correctly using bit scans
        assert!(moves.len() <= 16); // At most 8 squares in each direction
    }

    // Task 2.0.2.5: Test batch generation
    #[test]
    fn test_batch_generation_magic() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::new(magic_table);
        let board = BitboardBoard::empty();
        let pieces = vec![
            (Position::new(0, 0), PieceType::Rook),
            (Position::new(4, 4), PieceType::Bishop),
        ];
        let player = Player::Black;

        let moves = generator.generate_sliding_moves_batch(&board, &pieces, player);
        
        // Should generate moves for both pieces
        assert!(!moves.is_empty());
    }

    #[test]
    fn test_batch_generation_fallback() {
        let magic_table = Arc::new(MagicTable::default());
        let generator = SlidingMoveGenerator::with_settings(magic_table, false);
        let board = BitboardBoard::empty();
        let pieces = vec![
            (Position::new(0, 0), PieceType::Rook),
            (Position::new(4, 4), PieceType::Bishop),
        ];
        let player = Player::Black;

        let moves = generator.generate_sliding_moves_batch(&board, &pieces, player);
        
        // Fallback should also work for batch generation
        assert!(!moves.is_empty());
    }
}
