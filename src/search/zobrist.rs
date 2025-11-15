use crate::bitboards::BitboardBoard;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, PieceType, Player, Position};
use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Zobrist hashing table for Shogi positions
///
/// This struct contains all the random keys needed to generate unique hash values
/// for Shogi positions. It includes keys for piece positions, hand pieces,
/// side to move, and repetition states.
#[derive(Debug, Clone)]
pub struct ZobristTable {
    /// Hash keys for piece positions [piece_type][position]
    /// 14 piece types × 81 positions = 1134 keys
    pub piece_keys: [[u64; 81]; 14],

    /// Hash key for side to move (Black vs White)
    pub side_to_move_key: u64,

    /// Hash keys for pieces in hand [piece_type][count]
    /// 14 piece types × 8 counts = 112 keys (max 8 of any piece type in hand)
    pub hand_keys: [[u64; 8]; 14],

    /// Hash keys for repetition tracking [state]
    /// 4 states: no repetition, 2-fold, 3-fold, 4-fold
    pub repetition_keys: [u64; 4],

    /// Random number generator seed for reproducible keys
    seed: u64,
}

impl ZobristTable {
    /// Create a new Zobrist table with a given seed
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Initialize piece position keys (14 piece types × 81 positions)
        let mut piece_keys = [[0u64; 81]; 14];
        for piece_type in 0..14 {
            for position in 0..81 {
                piece_keys[piece_type][position] = rng.gen::<u64>();
            }
        }

        // Initialize side to move key
        let side_to_move_key = rng.gen::<u64>();

        // Initialize hand piece keys (14 piece types × 8 counts)
        let mut hand_keys = [[0u64; 8]; 14];
        for piece_type in 0..14 {
            for count in 0..8 {
                hand_keys[piece_type][count] = rng.gen::<u64>();
            }
        }

        // Initialize repetition keys (4 states)
        let mut repetition_keys = [0u64; 4];
        for state in 0..4 {
            repetition_keys[state] = rng.gen::<u64>();
        }

        Self {
            piece_keys,
            side_to_move_key,
            hand_keys,
            repetition_keys,
            seed,
        }
    }

    /// Create a new Zobrist table with a default seed
    pub fn new_default() -> Self {
        Self::new(0x1234567890ABCDEF)
    }

    /// Get the hash key for a piece at a specific position
    pub fn get_piece_key(&self, piece_type: PieceType, position: Position) -> u64 {
        let piece_index = piece_type.to_u8() as usize;
        let pos_index = position.to_index() as usize;
        self.piece_keys[piece_index][pos_index]
    }

    /// Get the hash key for side to move
    pub fn get_side_to_move_key(&self) -> u64 {
        self.side_to_move_key
    }

    /// Get the hash key for a specific count of a piece type in hand
    pub fn get_hand_key(&self, piece_type: PieceType, count: u8) -> u64 {
        let piece_index = piece_type.to_u8() as usize;
        let count_index = count.min(7) as usize; // Cap at 7 since array is 0-7
        self.hand_keys[piece_index][count_index]
    }

    /// Get the hash key for a repetition state
    pub fn get_repetition_key(&self, state: RepetitionState) -> u64 {
        let state_index = state as usize;
        self.repetition_keys[state_index]
    }

    /// Get the seed used for this table
    pub fn get_seed(&self) -> u64 {
        self.seed
    }
}

/// Repetition states for tracking position repetition in Shogi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepetitionState {
    /// No repetition detected
    None = 0,
    /// Position has occurred once before
    TwoFold = 1,
    /// Position has occurred twice before
    ThreeFold = 2,
    /// Position has occurred three times before (draw)
    FourFold = 3,
}

impl RepetitionState {
    /// Get the next repetition state
    pub fn next(self) -> Self {
        match self {
            RepetitionState::None => RepetitionState::TwoFold,
            RepetitionState::TwoFold => RepetitionState::ThreeFold,
            RepetitionState::ThreeFold => RepetitionState::FourFold,
            RepetitionState::FourFold => RepetitionState::FourFold, // Stay at four-fold
        }
    }

    /// Check if this state represents a draw
    pub fn is_draw(self) -> bool {
        self == RepetitionState::FourFold
    }
}

/// Zobrist hasher for Shogi positions
///
/// This struct provides methods to compute and update Zobrist hash values
/// for Shogi positions, including support for all Shogi-specific features
/// like drops, captures to hand, and repetition tracking.
pub struct ZobristHasher {
    table: &'static ZobristTable,
}

impl ZobristHasher {
    /// Create a new Zobrist hasher using the global table
    pub fn new() -> Self {
        Self {
            table: &ZOBRIST_TABLE,
        }
    }

    /// Compute the hash for a complete Shogi position
    pub fn hash_position(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        repetition_state: RepetitionState,
    ) -> u64 {
        let mut hash = 0u64;

        // Hash all pieces on the board
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    hash ^= self.table.get_piece_key(piece.piece_type, pos);
                }
            }
        }

        // Hash side to move
        if player == Player::Black {
            hash ^= self.table.get_side_to_move_key();
        }

        // Hash pieces in hand for Black
        for piece_type in [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ] {
            let count = captured_pieces.count(piece_type, Player::Black);
            if count > 0 {
                hash ^= self.table.get_hand_key(piece_type, count as u8);
            }
        }

        // Hash pieces in hand for White
        for piece_type in [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ] {
            let count = captured_pieces.count(piece_type, Player::White);
            if count > 0 {
                hash ^= self.table.get_hand_key(piece_type, count as u8);
            }
        }

        // Hash repetition state
        hash ^= self.table.get_repetition_key(repetition_state);

        hash
    }

    /// Update hash for a move (incremental update)
    ///
    /// This method efficiently updates a hash value when a move is made,
    /// avoiding the need to recompute the entire position hash.
    pub fn update_hash_for_move(
        &self,
        mut hash: u64,
        move_: &Move,
        board_before: &BitboardBoard,
        _board_after: &BitboardBoard,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        // Check if this is a drop move (no from position)
        if move_.from.is_none() {
            // Drop move: Add dropped piece to board
            hash ^= self.table.get_piece_key(move_.piece_type, move_.to);
        } else {
            // Normal move: Remove piece from source square
            if let Some(from) = move_.from {
                if let Some(piece) = board_before.get_piece(from) {
                    hash ^= self.table.get_piece_key(piece.piece_type, from);
                }
            }

            // Add piece to destination square
            let piece_type = if move_.is_promotion {
                move_
                    .piece_type
                    .promoted_version()
                    .unwrap_or(move_.piece_type)
            } else {
                move_.piece_type
            };
            hash ^= self.table.get_piece_key(piece_type, move_.to);

            // Handle capture
            if move_.is_capture {
                if let Some(captured) = &move_.captured_piece {
                    hash ^= self.table.get_piece_key(captured.piece_type, move_.to);
                }
            }
        }

        // Update side to move
        hash ^= self.table.get_side_to_move_key();

        // Update hand pieces if counts changed
        for piece_type in [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ] {
            let count_before = captured_pieces_before.count(piece_type, Player::Black);
            let count_after = captured_pieces_after.count(piece_type, Player::Black);

            if count_before != count_after {
                if count_before > 0 {
                    hash ^= self.table.get_hand_key(piece_type, count_before as u8);
                }
                if count_after > 0 {
                    hash ^= self.table.get_hand_key(piece_type, count_after as u8);
                }
            }

            let count_before_white = captured_pieces_before.count(piece_type, Player::White);
            let count_after_white = captured_pieces_after.count(piece_type, Player::White);

            if count_before_white != count_after_white {
                if count_before_white > 0 {
                    hash ^= self
                        .table
                        .get_hand_key(piece_type, count_before_white as u8);
                }
                if count_after_white > 0 {
                    hash ^= self.table.get_hand_key(piece_type, count_after_white as u8);
                }
            }
        }

        hash
    }

    /// Update hash for side to move change only
    pub fn update_hash_for_side_to_move(&self, hash: u64) -> u64 {
        hash ^ self.table.get_side_to_move_key()
    }

    /// Update hash for repetition state change
    pub fn update_hash_for_repetition(
        &self,
        hash: u64,
        old_state: RepetitionState,
        new_state: RepetitionState,
    ) -> u64 {
        let mut new_hash = hash;
        if old_state != RepetitionState::None {
            new_hash ^= self.table.get_repetition_key(old_state);
        }
        if new_state != RepetitionState::None {
            new_hash ^= self.table.get_repetition_key(new_state);
        }
        new_hash
    }

    /// Get a piece key for a specific piece type and position
    pub fn get_piece_key(&self, piece_type: PieceType, position: Position) -> u64 {
        self.table.get_piece_key(piece_type, position)
    }

    /// Get the side to move key
    pub fn get_side_to_move_key(&self) -> u64 {
        self.table.get_side_to_move_key()
    }

    /// Get a hand key for a specific piece type and count
    pub fn get_hand_key(&self, piece_type: PieceType, count: u8) -> u64 {
        self.table.get_hand_key(piece_type, count)
    }

    /// Get a repetition key for a specific state
    pub fn get_repetition_key(&self, state: RepetitionState) -> u64 {
        self.table.get_repetition_key(state)
    }
}

lazy_static! {
    /// Global Zobrist table instance
    ///
    /// This is the single global instance of the Zobrist table that should be used
    /// throughout the application for consistent hashing.
    pub static ref ZOBRIST_TABLE: ZobristTable = ZobristTable::new_default();
}

/// Convenience function to get the global Zobrist table
pub fn get_zobrist_table() -> &'static ZobristTable {
    &ZOBRIST_TABLE
}

/// Convenience function to create a new Zobrist hasher
pub fn create_hasher() -> ZobristHasher {
    ZobristHasher::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;

    #[test]
    fn test_zobrist_table_creation() {
        let table = ZobristTable::new(42);
        assert_eq!(table.get_seed(), 42);

        // Test that all keys are non-zero (very high probability)
        for piece_type in 0..14 {
            for position in 0..81 {
                assert_ne!(table.piece_keys[piece_type][position], 0);
            }
        }

        assert_ne!(table.side_to_move_key, 0);

        for piece_type in 0..14 {
            for count in 0..8 {
                assert_ne!(table.hand_keys[piece_type][count], 0);
            }
        }

        for state in 0..4 {
            assert_ne!(table.repetition_keys[state], 0);
        }
    }

    #[test]
    fn test_zobrist_table_consistency() {
        let table1 = ZobristTable::new(42);
        let table2 = ZobristTable::new(42);

        // Tables with same seed should be identical
        assert_eq!(table1.piece_keys, table2.piece_keys);
        assert_eq!(table1.side_to_move_key, table2.side_to_move_key);
        assert_eq!(table1.hand_keys, table2.hand_keys);
        assert_eq!(table1.repetition_keys, table2.repetition_keys);
    }

    #[test]
    fn test_repetition_state() {
        assert_eq!(RepetitionState::None.next(), RepetitionState::TwoFold);
        assert_eq!(RepetitionState::TwoFold.next(), RepetitionState::ThreeFold);
        assert_eq!(RepetitionState::ThreeFold.next(), RepetitionState::FourFold);
        assert_eq!(RepetitionState::FourFold.next(), RepetitionState::FourFold);

        assert!(!RepetitionState::None.is_draw());
        assert!(!RepetitionState::TwoFold.is_draw());
        assert!(!RepetitionState::ThreeFold.is_draw());
        assert!(RepetitionState::FourFold.is_draw());
    }

    #[test]
    fn test_global_zobrist_table() {
        let table1 = get_zobrist_table();
        let table2 = get_zobrist_table();

        // Global table should be the same instance
        assert_eq!(
            std::ptr::addr_of!(table1.piece_keys),
            std::ptr::addr_of!(table2.piece_keys)
        );
    }

    #[test]
    fn test_hash_position_consistency() {
        let hasher = create_hasher();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let hash1 = hasher.hash_position(
            &board,
            Player::Black,
            &captured_pieces,
            RepetitionState::None,
        );
        let hash2 = hasher.hash_position(
            &board,
            Player::Black,
            &captured_pieces,
            RepetitionState::None,
        );

        // Same position should give same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_position_different_players() {
        let hasher = create_hasher();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let hash_black = hasher.hash_position(
            &board,
            Player::Black,
            &captured_pieces,
            RepetitionState::None,
        );
        let hash_white = hasher.hash_position(
            &board,
            Player::White,
            &captured_pieces,
            RepetitionState::None,
        );

        // Different players should give different hashes
        assert_ne!(hash_black, hash_white);
    }

    #[test]
    fn test_side_to_move_update() {
        let hasher = create_hasher();
        let hash = 12345u64;

        let updated_hash = hasher.update_hash_for_side_to_move(hash);
        let back_to_original = hasher.update_hash_for_side_to_move(updated_hash);

        // Double side-to-move update should return to original
        assert_eq!(hash, back_to_original);
    }

    #[test]
    fn test_repetition_update() {
        let hasher = create_hasher();
        let hash = 12345u64;

        let updated_hash = hasher.update_hash_for_repetition(
            hash,
            RepetitionState::None,
            RepetitionState::TwoFold,
        );

        let back_to_original = hasher.update_hash_for_repetition(
            updated_hash,
            RepetitionState::TwoFold,
            RepetitionState::None,
        );

        // Reversing repetition update should return to original
        assert_eq!(hash, back_to_original);
    }
}
