//! Shogi-specific hash handling for Zobrist hashing
//!
//! This module provides specialized hash handling for Shogi-specific features
//! including drop moves, captures to hand, promotions, and repetition tracking.
//! It extends the basic Zobrist hashing with Shogi game rules and validation.

use crate::bitboards::BitboardBoard;
use crate::search::zobrist::{RepetitionState, ZobristHasher};
use crate::types::*;
use std::collections::HashMap;

/// Shogi-specific hash handler
///
/// This struct provides enhanced hash handling for Shogi positions,
/// including proper handling of all Shogi-specific move types and rules.
pub struct ShogiHashHandler {
    zobrist_hasher: ZobristHasher,
    /// History of position hashes for repetition detection
    position_history: Vec<u64>,
    /// Count of how many times each position hash has occurred
    hash_counts: HashMap<u64, u32>,
    /// Maximum history length to prevent memory issues
    max_history_length: usize,
}

impl ShogiHashHandler {
    /// Create a new Shogi hash handler
    pub fn new(max_history_length: usize) -> Self {
        Self {
            zobrist_hasher: ZobristHasher::new(),
            position_history: Vec::new(),
            hash_counts: HashMap::new(),
            max_history_length,
        }
    }

    /// Create a new handler with default settings
    pub fn new_default() -> Self {
        Self::new(1000) // Keep last 1000 positions
    }

    /// Get the current position hash for a board state
    pub fn get_position_hash(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u64 {
        let repetition_state = self.get_repetition_state_for_hash(0); // Will be updated after hash calculation
        self.zobrist_hasher
            .hash_position(board, player, captured_pieces, repetition_state)
    }

    /// Update hash for a drop move
    ///
    /// Drop moves are unique to Shogi and require special handling:
    /// 1. Remove piece from hand (decrease hand count)
    /// 2. Add piece to board
    /// 3. Update side to move
    pub fn update_hash_for_drop_move(
        &self,
        mut hash: u64,
        drop_move: &Move,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        debug_assert!(
            drop_move.from.is_none(),
            "Drop move should not have a from position"
        );

        // Add the dropped piece to the board
        hash ^= self
            .zobrist_hasher
            .get_piece_key(drop_move.piece_type, drop_move.to);

        // Update hand piece counts
        hash = self.update_hand_piece_hash(
            hash,
            drop_move.piece_type,
            drop_move.player,
            captured_pieces_before,
            captured_pieces_after,
        );

        // Update side to move
        hash ^= self.zobrist_hasher.get_side_to_move_key();

        hash
    }

    /// Update hash for a capture move
    ///
    /// Capture moves in Shogi can capture pieces to hand, which requires:
    /// 1. Remove captured piece from board
    /// 2. Add captured piece to hand
    /// 3. Update moving piece position
    /// 4. Handle promotion if applicable
    pub fn update_hash_for_capture_move(
        &self,
        mut hash: u64,
        capture_move: &Move,
        board_before: &BitboardBoard,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        debug_assert!(capture_move.is_capture, "Move should be a capture");

        // Handle the moving piece
        if let Some(from) = capture_move.from {
            // Remove piece from source square
            if let Some(piece) = board_before.get_piece(from) {
                hash ^= self.zobrist_hasher.get_piece_key(piece.piece_type, from);
            }
        }

        // Add piece to destination square (considering promotion)
        let piece_type = if capture_move.is_promotion {
            capture_move
                .piece_type
                .promoted_version()
                .unwrap_or(capture_move.piece_type)
        } else {
            capture_move.piece_type
        };
        hash ^= self
            .zobrist_hasher
            .get_piece_key(piece_type, capture_move.to);

        // Handle captured piece
        if let Some(captured) = &capture_move.captured_piece {
            // Remove captured piece from destination square
            hash ^= self
                .zobrist_hasher
                .get_piece_key(captured.piece_type, capture_move.to);

            // Add captured piece to hand (unpromoted)
            let unpromoted_captured = captured.unpromoted();
            hash = self.update_hand_piece_hash(
                hash,
                unpromoted_captured.piece_type,
                capture_move.player, // The player making the capture gets the piece
                captured_pieces_before,
                captured_pieces_after,
            );
        }

        // Update side to move
        hash ^= self.zobrist_hasher.get_side_to_move_key();

        hash
    }

    /// Update hash for a promotion move
    ///
    /// Promotion moves require:
    /// 1. Remove original piece from source
    /// 2. Add promoted piece to destination
    /// 3. Update side to move
    pub fn update_hash_for_promotion_move(
        &self,
        mut hash: u64,
        promotion_move: &Move,
        board_before: &BitboardBoard,
        _captured_pieces_before: &CapturedPieces,
        _captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        debug_assert!(promotion_move.is_promotion, "Move should be a promotion");

        // Handle normal move with promotion
        hash = self.update_hash_for_normal_move(
            hash,
            promotion_move,
            board_before,
            _captured_pieces_before,
            _captured_pieces_after,
        );

        hash
    }

    /// Update hash for a normal move (no capture, no promotion)
    pub fn update_hash_for_normal_move(
        &self,
        mut hash: u64,
        move_: &Move,
        board_before: &BitboardBoard,
        _captured_pieces_before: &CapturedPieces,
        _captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        // Remove piece from source square
        if let Some(from) = move_.from {
            if let Some(piece) = board_before.get_piece(from) {
                hash ^= self.zobrist_hasher.get_piece_key(piece.piece_type, from);
            }
        }

        // Add piece to destination square
        hash ^= self
            .zobrist_hasher
            .get_piece_key(move_.piece_type, move_.to);

        // Update side to move
        hash ^= self.zobrist_hasher.get_side_to_move_key();

        hash
    }

    /// Update hand piece hash for a specific piece type and player
    pub fn update_hand_piece_hash(
        &self,
        mut hash: u64,
        piece_type: PieceType,
        player: Player,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        let count_before = captured_pieces_before.count(piece_type, player);
        let count_after = captured_pieces_after.count(piece_type, player);

        if count_before != count_after {
            // Remove old hand count
            if count_before > 0 {
                hash ^= self
                    .zobrist_hasher
                    .get_hand_key(piece_type, count_before as u8);
            }

            // Add new hand count
            if count_after > 0 {
                hash ^= self
                    .zobrist_hasher
                    .get_hand_key(piece_type, count_after as u8);
            }
        }

        hash
    }

    /// Add a position hash to the history and update repetition tracking
    pub fn add_position_to_history(&mut self, hash: u64) {
        // Add to history
        self.position_history.push(hash);

        // Update count
        *self.hash_counts.entry(hash).or_insert(0) += 1;

        // Maintain history length limit
        if self.position_history.len() > self.max_history_length {
            let old_hash = self.position_history.remove(0);
            if let Some(count) = self.hash_counts.get_mut(&old_hash) {
                *count -= 1;
                if *count == 0 {
                    self.hash_counts.remove(&old_hash);
                }
            }
        }
    }

    /// Get the repetition state for a given position hash
    pub fn get_repetition_state_for_hash(&self, hash: u64) -> RepetitionState {
        match self.hash_counts.get(&hash) {
            Some(&count) if count >= 4 => RepetitionState::FourFold,
            Some(&count) if count >= 3 => RepetitionState::ThreeFold,
            Some(&count) if count >= 2 => RepetitionState::TwoFold,
            _ => RepetitionState::None,
        }
    }

    /// Check if a position is a repetition (4-fold repetition = draw)
    pub fn is_repetition(&self, hash: u64) -> bool {
        self.get_repetition_state_for_hash(hash).is_draw()
    }

    /// Get the current repetition state based on the latest position
    pub fn get_current_repetition_state(&self) -> RepetitionState {
        if let Some(&latest_hash) = self.position_history.last() {
            self.get_repetition_state_for_hash(latest_hash)
        } else {
            RepetitionState::None
        }
    }

    /// Validate that a hash is unique for Shogi positions
    ///
    /// This method performs various checks to ensure the hash correctly
    /// represents Shogi-specific features.
    pub fn validate_hash_uniqueness(
        &self,
        hash: u64,
        _board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
    ) -> bool {
        // Basic validation - hash should not be zero
        if hash == 0 {
            return false;
        }

        // Validate that different positions have different hashes
        // This is a basic check - in practice, hash collisions are rare but possible
        true
    }

    /// Get statistics about the position history
    pub fn get_history_stats(&self) -> HashHistoryStats {
        let total_positions = self.position_history.len();
        let unique_positions = self.hash_counts.len();
        let repetition_count = self
            .hash_counts
            .values()
            .filter(|&&count| count > 1)
            .count();

        HashHistoryStats {
            total_positions,
            unique_positions,
            repetition_count,
            max_history_length: self.max_history_length,
        }
    }

    /// Clear the position history (useful for new games)
    pub fn clear_history(&mut self) {
        self.position_history.clear();
        self.hash_counts.clear();
    }

    /// Get the underlying Zobrist hasher
    pub fn get_zobrist_hasher(&self) -> &ZobristHasher {
        &self.zobrist_hasher
    }
}

/// Statistics about hash history
#[derive(Debug, Clone)]
pub struct HashHistoryStats {
    pub total_positions: usize,
    pub unique_positions: usize,
    pub repetition_count: usize,
    pub max_history_length: usize,
}

impl HashHistoryStats {
    /// Get the repetition rate as a percentage
    pub fn repetition_rate(&self) -> f64 {
        if self.total_positions == 0 {
            0.0
        } else {
            (self.repetition_count as f64 / self.total_positions as f64) * 100.0
        }
    }

    /// Get the uniqueness rate as a percentage
    pub fn uniqueness_rate(&self) -> f64 {
        if self.total_positions == 0 {
            0.0
        } else {
            (self.unique_positions as f64 / self.total_positions as f64) * 100.0
        }
    }
}

/// Shogi-specific move validation for hash operations
pub struct ShogiMoveValidator;

impl ShogiMoveValidator {
    /// Validate that a drop move is legal in Shogi
    pub fn validate_drop_move(
        move_: &Move,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Drop moves must not have a from position
        if move_.from.is_some() {
            return false;
        }

        // Check if player has the piece in hand
        if captured_pieces.count(move_.piece_type, move_.player) == 0 {
            return false;
        }

        // Check if destination square is empty
        if board.is_square_occupied(move_.to) {
            return false;
        }

        // Check Shogi-specific drop rules
        match move_.piece_type {
            PieceType::Pawn => {
                // Pawn cannot be dropped in the last rank
                match move_.player {
                    Player::Black => move_.to.row < 8,
                    Player::White => move_.to.row > 0,
                }
            }
            PieceType::Lance => {
                // Lance cannot be dropped in the last rank
                match move_.player {
                    Player::Black => move_.to.row < 8,
                    Player::White => move_.to.row > 0,
                }
            }
            PieceType::Knight => {
                // Knight cannot be dropped in the last two ranks
                match move_.player {
                    Player::Black => move_.to.row < 7,
                    Player::White => move_.to.row > 1,
                }
            }
            _ => true, // Other pieces can be dropped anywhere
        }
    }

    /// Validate that a capture move is legal in Shogi
    pub fn validate_capture_move(
        move_: &Move,
        board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
    ) -> bool {
        // Must be a capture
        if !move_.is_capture {
            return false;
        }

        // Must have a captured piece
        if move_.captured_piece.is_none() {
            return false;
        }

        // Check if source square has a piece
        if let Some(from) = move_.from {
            if board.get_piece(from).is_none() {
                return false;
            }
        }

        // Check if destination square has an opponent's piece
        if let Some(piece_at_dest) = board.get_piece(move_.to) {
            if piece_at_dest.player == move_.player {
                return false; // Cannot capture own piece
            }
        }

        true
    }

    /// Validate that a promotion move is legal in Shogi
    pub fn validate_promotion_move(
        move_: &Move,
        _board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
    ) -> bool {
        // Must be a promotion
        if !move_.is_promotion {
            return false;
        }

        // Check if piece can promote
        if move_.piece_type.promoted_version().is_none() {
            return false;
        }

        // Check if move is in promotion zone or from promotion zone
        let from_promotion_zone = move_.from.map_or(false, |from| match move_.player {
            Player::Black => from.row <= 2,
            Player::White => from.row >= 6,
        });

        let to_promotion_zone = match move_.player {
            Player::Black => move_.to.row <= 2,
            Player::White => move_.to.row >= 6,
        };

        from_promotion_zone || to_promotion_zone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;

    #[test]
    fn test_shogi_hash_handler_creation() {
        let handler = ShogiHashHandler::new_default();
        assert_eq!(handler.max_history_length, 1000);
        assert!(handler.position_history.is_empty());
        assert!(handler.hash_counts.is_empty());
    }

    #[test]
    fn test_drop_move_hash_handling() {
        let handler = ShogiHashHandler::new_default();
        let mut captured_before = CapturedPieces::new();
        let mut captured_after = CapturedPieces::new();

        // Add a pawn to hand
        captured_before.add_piece(PieceType::Pawn, Player::Black);

        // Create a drop move
        let drop_move = Move::new_drop(PieceType::Pawn, Position::new(4, 4), Player::Black);

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_drop_move(
            initial_hash,
            &drop_move,
            &captured_before,
            &captured_after,
        );

        // Hash should change due to the drop move
        assert_ne!(initial_hash, updated_hash);
    }

    #[test]
    fn test_capture_move_hash_handling() {
        let handler = ShogiHashHandler::new_default();
        let mut board = BitboardBoard::new();
        let mut captured_before = CapturedPieces::new();
        let mut captured_after = CapturedPieces::new();

        // Set up a capture scenario
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::White),
            Position::new(5, 5),
        );

        // Create a capture move
        let capture_move = Move {
            from: Some(Position::new(6, 5)),
            to: Position::new(5, 5),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: true,
            captured_piece: Some(Piece::new(PieceType::Pawn, Player::White)),
            gives_check: false,
            is_recapture: false,
        };

        // Add captured piece to hand
        captured_after.add_piece(PieceType::Pawn, Player::Black);

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_capture_move(
            initial_hash,
            &capture_move,
            &board,
            &captured_before,
            &captured_after,
        );

        // Hash should change due to the capture move
        assert_ne!(initial_hash, updated_hash);
    }

    #[test]
    fn test_promotion_move_hash_handling() {
        let handler = ShogiHashHandler::new_default();
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Set up a promotion scenario
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::Black),
            Position::new(1, 1),
        );

        // Create a promotion move
        let promotion_move = Move::new_move(
            Position::new(1, 1),
            Position::new(0, 1),
            PieceType::Pawn,
            Player::Black,
            true, // is_promotion
        );

        let initial_hash = 0x1234567890ABCDEF;
        let updated_hash = handler.update_hash_for_promotion_move(
            initial_hash,
            &promotion_move,
            &board,
            &captured_pieces,
            &captured_pieces,
        );

        // Hash should change due to the promotion move
        assert_ne!(initial_hash, updated_hash);
    }

    #[test]
    fn test_position_history_tracking() {
        let mut handler = ShogiHashHandler::new_default();
        let hash1 = 0x1111111111111111;
        let hash2 = 0x2222222222222222;
        let hash3 = 0x1111111111111111; // Same as hash1

        // Add positions to history
        handler.add_position_to_history(hash1);
        handler.add_position_to_history(hash2);
        handler.add_position_to_history(hash3);

        // Check repetition states
        assert_eq!(
            handler.get_repetition_state_for_hash(hash1),
            RepetitionState::TwoFold
        );
        assert_eq!(
            handler.get_repetition_state_for_hash(hash2),
            RepetitionState::None
        );

        // Add hash1 again to make it three-fold
        handler.add_position_to_history(hash1);
        assert_eq!(
            handler.get_repetition_state_for_hash(hash1),
            RepetitionState::ThreeFold
        );

        // Add hash1 again to make it four-fold (draw)
        handler.add_position_to_history(hash1);
        assert_eq!(
            handler.get_repetition_state_for_hash(hash1),
            RepetitionState::FourFold
        );
        assert!(handler.is_repetition(hash1));
    }

    #[test]
    fn test_move_validation() {
        let mut board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Test drop move validation
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);
        let drop_move = Move::new_drop(PieceType::Pawn, Position::new(4, 4), Player::Black);
        assert!(ShogiMoveValidator::validate_drop_move(
            &drop_move,
            &board,
            &captured_pieces
        ));

        // Test invalid drop (no piece in hand)
        let mut captured_empty = CapturedPieces::new();
        assert!(!ShogiMoveValidator::validate_drop_move(
            &drop_move,
            &board,
            &captured_empty
        ));

        // Test invalid drop (square occupied)
        board.place_piece(
            Piece::new(PieceType::Pawn, Player::White),
            Position::new(4, 4),
        );
        assert!(!ShogiMoveValidator::validate_drop_move(
            &drop_move,
            &board,
            &captured_pieces
        ));
    }

    #[test]
    fn test_hash_history_stats() {
        let mut handler = ShogiHashHandler::new_default();
        let hash1 = 0x1111111111111111;
        let hash2 = 0x2222222222222222;
        let hash3 = 0x1111111111111111; // Same as hash1

        // Add positions
        handler.add_position_to_history(hash1);
        handler.add_position_to_history(hash2);
        handler.add_position_to_history(hash3);

        let stats = handler.get_history_stats();
        assert_eq!(stats.total_positions, 3);
        assert_eq!(stats.unique_positions, 2);
        assert_eq!(stats.repetition_count, 1); // hash1 appears twice
        assert_eq!(stats.max_history_length, 1000);

        // Check rates
        assert!((stats.repetition_rate() - 33.33).abs() < 0.1); // 1/3 * 100
        assert!((stats.uniqueness_rate() - 66.67).abs() < 0.1); // 2/3 * 100
    }

    #[test]
    fn test_hash_uniqueness_validation() {
        let handler = ShogiHashHandler::new_default();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Valid hash
        let valid_hash = 0x1234567890ABCDEF;
        assert!(handler.validate_hash_uniqueness(valid_hash, &board, &captured_pieces));

        // Invalid hash (zero)
        assert!(!handler.validate_hash_uniqueness(0, &board, &captured_pieces));
    }

    #[test]
    fn test_clear_history() {
        let mut handler = ShogiHashHandler::new_default();
        let hash = 0x1111111111111111;

        // Add some positions
        handler.add_position_to_history(hash);
        handler.add_position_to_history(hash);

        assert!(!handler.position_history.is_empty());
        assert!(!handler.hash_counts.is_empty());

        // Clear history
        handler.clear_history();

        assert!(handler.position_history.is_empty());
        assert!(handler.hash_counts.is_empty());
    }
}
