//! Board trait for Zobrist hashing integration
//!
//! This module provides a trait that abstracts board operations needed for
//! Zobrist hashing, allowing the hashing system to work with different board
//! implementations while maintaining performance and correctness.

use crate::search::RepetitionState;
use crate::types::board::{CapturedPieces, GamePhase};
use crate::types::core::{Move, Piece, PieceType, Player, Position};
use crate::types::search::PositionComplexity;
use crate::types::Bitboard;

/// Trait for board operations required by Zobrist hashing
///
/// This trait provides the minimal interface needed for Zobrist hashing
/// operations. It abstracts away board implementation details while providing
/// efficient access to the information needed for hash computation.
pub trait BoardTrait {
    /// Get the piece at the specified position, if any
    fn get_piece_at(&self, position: Position) -> Option<Piece>;

    /// Get all pieces on the board as a vector of (position, piece) pairs
    /// This is useful for efficient iteration over all pieces
    fn get_all_pieces(&self) -> Vec<(Position, Piece)>;

    /// Get the number of pieces of a specific type and player on the board
    fn count_pieces(&self, piece_type: PieceType, player: Player) -> usize;

    /// Check if a square is occupied by any piece
    fn is_square_occupied(&self, position: Position) -> bool;

    /// Check if a square is occupied by a specific player
    fn is_square_occupied_by_player(&self, position: Position, player: Player) -> bool;

    /// Get the occupied bitboard for a specific player
    /// This should return a bitboard where each bit represents an occupied square
    fn get_occupied_bitboard_for_player(&self, player: Player) -> Bitboard;

    /// Get the total occupied bitboard (both players)
    fn get_occupied_bitboard(&self) -> Bitboard;

    /// Check if the board represents a valid Shogi position
    fn is_valid_position(&self) -> bool;

    /// Get the side to move for this position
    /// This should be consistent with the game state
    fn get_side_to_move(&self) -> Player;

    /// Get the current repetition state
    /// This tracks how many times this position has occurred
    fn get_repetition_state(&self) -> RepetitionState;

    /// Get pieces in hand for a specific player
    /// This should return the captured pieces available for dropping
    fn get_captured_pieces(&self, player: Player) -> Vec<PieceType>;

    /// Get the total number of pieces in hand for a specific player
    fn get_captured_pieces_count(&self, player: Player) -> usize;

    /// Get the number of a specific piece type in hand for a player
    fn get_captured_piece_count(&self, piece_type: PieceType, player: Player) -> usize;

    /// Check if a player has a specific piece type in hand
    fn has_captured_piece(&self, piece_type: PieceType, player: Player) -> bool;

    /// Get a unique identifier for this board position
    /// This should incorporate side-to-move, hand pieces, and repetition state
    fn get_position_id(
        &self,
        player: Player,
        captured_pieces: &CapturedPieces,
        repetition_state: RepetitionState,
    ) -> u64;

    /// Clone the board state
    /// This is needed for move generation and position analysis
    fn clone_board(&self) -> Self;

    /// Check if this position is a terminal position (checkmate, stalemate, etc.)
    fn is_terminal_position(&self, captured_pieces: &CapturedPieces) -> bool;

    /// Get the game phase (opening, middlegame, endgame)
    fn get_game_phase(&self) -> GamePhase;

    /// Check if a move is legal in this position
    fn is_legal_move(&self, move_: &Move, captured_pieces: &CapturedPieces) -> bool;

    /// Get the king position for a specific player
    fn get_king_position(&self, player: Player) -> Option<Position>;

    /// Check if a player's king is in check
    fn is_king_in_check(&self, player: Player, captured_pieces: &CapturedPieces) -> bool;

    /// Get the material balance for a specific player
    /// Positive values favor the player, negative values favor the opponent
    fn get_material_balance(&self, player: Player) -> i32;

    /// Get the total material count on the board
    fn get_total_material_count(&self) -> u32;

    /// Check if this position is in a promotion zone for a specific player
    fn is_in_promotion_zone(&self, position: Position, player: Player) -> bool;

    /// Get all possible drop moves for a specific piece type and player
    fn get_drop_moves(&self, piece_type: PieceType, player: Player) -> Vec<Move>;

    /// Check if a drop move is legal
    fn is_legal_drop(
        &self,
        piece_type: PieceType,
        position: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool;

    /// Get the position hash for this board state
    /// This should be consistent with Zobrist hashing
    fn get_position_hash(&self, captured_pieces: &CapturedPieces) -> u64;

    /// Update the position hash after a move
    /// This should efficiently update the hash without recomputing everything
    fn update_hash_for_move(
        &self,
        current_hash: u64,
        move_: &Move,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64;
}

/// Extension trait for board operations that are commonly used together
pub trait BoardTraitExt: BoardTrait {
    /// Get all pieces of a specific type for a player
    fn get_pieces_of_type(&self, piece_type: PieceType, player: Player) -> Vec<Position> {
        self.get_all_pieces()
            .into_iter()
            .filter(|(_, piece)| piece.piece_type == piece_type && piece.player == player)
            .map(|(pos, _)| pos)
            .collect()
    }

    /// Get all pieces for a specific player
    fn get_player_pieces(&self, player: Player) -> Vec<(Position, Piece)> {
        self.get_all_pieces()
            .into_iter()
            .filter(|(_, piece)| piece.player == player)
            .collect()
    }

    /// Check if the position is a draw by repetition
    fn is_draw_by_repetition(&self) -> bool {
        self.get_repetition_state().is_draw()
    }

    /// Get the total number of pieces on the board
    fn get_total_piece_count(&self) -> usize {
        self.get_all_pieces().len()
    }

    /// Check if the position is in the opening phase
    fn is_opening(&self) -> bool {
        self.get_game_phase() == GamePhase::Opening
    }

    /// Check if the position is in the endgame phase
    fn is_endgame(&self) -> bool {
        self.get_game_phase() == GamePhase::Endgame
    }

    /// Check if the position is in the middlegame phase
    fn is_middlegame(&self) -> bool {
        self.get_game_phase() == GamePhase::Middlegame
    }

    /// Get the position complexity (simplified heuristic)
    fn get_position_complexity(&self) -> PositionComplexity {
        let piece_count = self.get_total_piece_count();
        let material_count = self.get_total_material_count();

        match (piece_count, material_count) {
            (0..=10, _) => PositionComplexity::Low,
            (11..=20, 0..=20) => PositionComplexity::Low,
            (11..=20, 21..=35) => PositionComplexity::Medium,
            (21..=30, _) => PositionComplexity::Medium,
            _ => PositionComplexity::High,
        }
    }

    /// Check if a position is tactically rich
    fn is_tactically_rich(&self) -> bool {
        let complexity = self.get_position_complexity();
        matches!(complexity, PositionComplexity::High)
    }

    /// Get the position evaluation (simplified)
    fn get_simple_evaluation(&self, player: Player) -> i32 {
        self.get_material_balance(player)
    }
}

// Implement the extension trait for all types that implement BoardTrait
impl<T: BoardTrait> BoardTraitExt for T {}

/// Helper struct for board position analysis
#[derive(Debug, Clone)]
pub struct BoardPositionAnalysis {
    /// The position hash
    pub position_hash: u64,
    /// The repetition state
    pub repetition_state: RepetitionState,
    /// The game phase
    pub game_phase: GamePhase,
    /// The position complexity
    pub complexity: PositionComplexity,
    /// Whether the position is terminal
    pub is_terminal: bool,
    /// The material balance
    pub material_balance: i32,
    /// The total piece count
    pub piece_count: usize,
    /// The total material count
    pub material_count: u32,
}

impl BoardPositionAnalysis {
    /// Create a new analysis from a board implementing BoardTrait
    pub fn from_board<T: BoardTrait>(board: &T, captured_pieces: &CapturedPieces) -> Self {
        Self {
            position_hash: board.get_position_hash(captured_pieces),
            repetition_state: board.get_repetition_state(),
            game_phase: board.get_game_phase(),
            complexity: board.get_position_complexity(),
            is_terminal: board.is_terminal_position(captured_pieces),
            material_balance: board.get_material_balance(Player::Black),
            piece_count: board.get_total_piece_count(),
            material_count: board.get_total_material_count(),
        }
    }

    /// Check if this analysis represents a draw
    pub fn is_draw(&self) -> bool {
        self.repetition_state.is_draw()
    }

    /// Check if this analysis represents a terminal position
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Get a summary string for this analysis
    pub fn summary(&self) -> String {
        format!(
            "Hash: 0x{:016x}, Phase: {:?}, Complexity: {:?}, Pieces: {}, Material: {}, Terminal: {}",
            self.position_hash,
            self.game_phase,
            self.complexity,
            self.piece_count,
            self.material_count,
            self.is_terminal
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;

    // Test that BoardTrait methods work correctly
    #[test]
    fn test_board_trait_basic_operations() {
        let board = BitboardBoard::new();

        // Test basic piece access
        assert!(board.get_piece_at(Position::new(0, 0)).is_some());
        assert!(board.get_piece_at(Position::new(4, 4)).is_none());

        // Test square occupation
        assert!(board.is_square_occupied(Position::new(0, 0)));
        assert!(!board.is_square_occupied(Position::new(4, 4)));

        // Test player occupation
        assert!(board.is_square_occupied_by_player(Position::new(0, 0), Player::White));
        assert!(board.is_square_occupied_by_player(Position::new(8, 0), Player::Black));

        // Test piece counting
        assert!(board.count_pieces(PieceType::King, Player::Black) > 0);
        assert!(board.count_pieces(PieceType::King, Player::White) > 0);

        // Test validity
        assert!(board.is_valid_position());

        // Test side to move
        assert_eq!(board.get_side_to_move(), Player::Black);

        // Test repetition state
        assert_eq!(board.get_repetition_state(), RepetitionState::None);

        // Test captured pieces
        assert_eq!(board.get_captured_pieces_count(Player::Black), 0);
        assert_eq!(board.get_captured_pieces_count(Player::White), 0);

        // Test position ID
        assert!(
            board.get_position_id(Player::Black, &CapturedPieces::new(), RepetitionState::None) > 0
        );

        // Test game phase
        assert_eq!(board.get_game_phase(), GamePhase::Opening);

        // Test material balance
        assert_eq!(board.get_material_balance(Player::Black), 0); // Equal material in starting position

        // Test total material count
        assert!(board.get_total_material_count() > 0);

        // Test promotion zone
        assert!(board.is_in_promotion_zone(Position::new(6, 0), Player::Black));
        assert!(!board.is_in_promotion_zone(Position::new(5, 4), Player::Black));
        assert!(board.is_in_promotion_zone(Position::new(2, 0), Player::White));
        assert!(!board.is_in_promotion_zone(Position::new(3, 4), Player::White));

        // Test king position
        assert!(board.get_king_position(Player::Black).is_some());
        assert!(board.get_king_position(Player::White).is_some());

        // Test check detection
        assert!(!board.is_king_in_check(Player::Black, &CapturedPieces::new()));
        assert!(!board.is_king_in_check(Player::White, &CapturedPieces::new()));
    }

    #[test]
    fn test_board_trait_ext_operations() {
        let board = BitboardBoard::new();

        // Test piece type filtering
        let black_kings = board.get_pieces_of_type(PieceType::King, Player::Black);
        assert_eq!(black_kings.len(), 1);

        let white_kings = board.get_pieces_of_type(PieceType::King, Player::White);
        assert_eq!(white_kings.len(), 1);

        // Test player piece filtering
        let black_pieces = board.get_player_pieces(Player::Black);
        assert_eq!(black_pieces.len(), 20); // 20 pieces in starting position

        let white_pieces = board.get_player_pieces(Player::White);
        assert_eq!(white_pieces.len(), 20);

        // Test repetition check
        assert!(!board.is_draw_by_repetition());

        // Test piece count
        assert_eq!(board.get_total_piece_count(), 40); // 40 pieces in starting position

        // Test game phase checks
        assert!(board.is_opening());
        assert!(!board.is_endgame());
        assert!(!board.is_middlegame());

        // Test complexity
        let complexity = board.get_position_complexity();
        assert!(matches!(
            complexity,
            PositionComplexity::Medium | PositionComplexity::High
        ));

        // Test tactical richness
        assert!(board.is_tactically_rich());

        // Test simple evaluation
        let eval = board.get_simple_evaluation(Player::Black);
        assert_eq!(eval, 0); // Equal material in starting position
    }

    #[test]
    fn test_board_position_analysis() {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let analysis = BoardPositionAnalysis::from_board(&board, &captured_pieces);

        // Test analysis fields
        assert!(analysis.position_hash > 0);
        assert_eq!(analysis.repetition_state, RepetitionState::None);
        assert_eq!(analysis.game_phase, GamePhase::Opening);
        assert_eq!(analysis.material_balance, 0);
        assert_eq!(analysis.piece_count, 40);
        assert!(analysis.material_count > 0);

        // Test analysis methods
        assert!(!analysis.is_draw());
        assert!(!analysis.is_terminal());

        // Test summary
        let summary = analysis.summary();
        assert!(summary.contains("Hash:"));
        assert!(summary.contains("Phase:"));
        assert!(summary.contains("Complexity:"));
        assert!(summary.contains("Pieces: 40"));
    }
}
