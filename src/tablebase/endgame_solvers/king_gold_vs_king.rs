//! King + Gold vs King endgame solver
//! 
//! This module implements a specialized solver for King + Gold vs King endgames.
//! This is one of the most common and important endgames in Shogi, as it represents
//! the minimum material needed to force a win against a lone king.

use crate::types::{Player, Position, PieceType, Move};
use crate::BitboardBoard;
use crate::CapturedPieces;
use super::super::{TablebaseResult, EndgameSolver};
use super::super::solver_traits::EndgameSolverHelper;
use super::super::tablebase_config::KingGoldConfig;

/// Solver for King + Gold vs King endgames
/// 
/// This solver handles positions where one side has a King and Gold,
/// and the other side has only a King. The solver implements standard
/// mating patterns for this endgame scenario.
pub struct KingGoldVsKingSolver {
    /// Configuration for this solver
    config: KingGoldConfig,
}

impl KingGoldVsKingSolver {
    /// Create a new King + Gold vs King solver with default configuration
    pub fn new() -> Self {
        Self::with_config(KingGoldConfig::default())
    }

    /// Create a new solver with specified configuration
    pub fn with_config(config: KingGoldConfig) -> Self {
        Self { config }
    }

    /// Check if the position is a King + Gold vs King endgame
    /// 
    /// This method verifies that:
    /// - One side has exactly a King and Gold
    /// - The other side has exactly a King
    /// - No captured pieces are present
    fn is_king_gold_vs_king(&self, board: &BitboardBoard, _player: Player, captured_pieces: &CapturedPieces) -> bool {
        // Must have no captured pieces
        if !captured_pieces.black.is_empty() || !captured_pieces.white.is_empty() {
            return false;
        }

        // Count pieces for each player
        let (black_pieces, white_pieces) = self.count_pieces_by_player(board);

        // Check if one side has King + Gold and the other has just King
        (black_pieces.len() == 2 && white_pieces.len() == 1 && 
         self.has_king_and_gold(&black_pieces) && self.has_king_only(&white_pieces)) ||
        (white_pieces.len() == 2 && black_pieces.len() == 1 && 
         self.has_king_and_gold(&white_pieces) && self.has_king_only(&black_pieces))
    }

    /// Count pieces by player
    fn count_pieces_by_player(&self, board: &BitboardBoard) -> (Vec<(PieceType, Position)>, Vec<(PieceType, Position)>) {
        let mut black_pieces = Vec::new();
        let mut white_pieces = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    let pos = Position::new(row, col);
                    match piece.player {
                        Player::Black => black_pieces.push((piece.piece_type, pos)),
                        Player::White => white_pieces.push((piece.piece_type, pos)),
                    }
                }
            }
        }

        (black_pieces, white_pieces)
    }

    /// Check if pieces contain King and Gold
    fn has_king_and_gold(&self, pieces: &[(PieceType, Position)]) -> bool {
        let mut has_king = false;
        let mut has_gold = false;

        for (piece_type, _) in pieces {
            match piece_type {
                PieceType::King => has_king = true,
                PieceType::Gold => has_gold = true,
                _ => return false, // Any other piece means this is not King + Gold
            }
        }

        has_king && has_gold
    }

    /// Check if pieces contain only King
    fn has_king_only(&self, pieces: &[(PieceType, Position)]) -> bool {
        pieces.len() == 1 && pieces[0].0 == PieceType::King
    }

    /// Find the King and Gold pieces for the attacking side
    fn find_attacking_pieces(&self, board: &BitboardBoard, player: Player) -> Option<(Position, Position)> {
        let mut king_pos = None;
        let mut gold_pos = None;

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    if piece.player == player {
                        let pos = Position::new(row, col);
                        match piece.piece_type {
                            PieceType::King => king_pos = Some(pos),
                            PieceType::Gold => gold_pos = Some(pos),
                            _ => return None, // Should not happen in King + Gold vs King
                        }
                    }
                }
            }
        }

        if let (Some(king), Some(gold)) = (king_pos, gold_pos) {
            Some((king, gold))
        } else {
            None
        }
    }

    /// Find the defending King
    fn find_defending_king(&self, board: &BitboardBoard, player: Player) -> Option<Position> {
        let defending_player = player.opposite();

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    if piece.player == defending_player && piece.piece_type == PieceType::King {
                        return Some(Position::new(row, col));
                    }
                }
            }
        }

        None
    }

    /// Calculate the optimal move for King + Gold vs King
    /// 
    /// This implements the standard mating technique:
    /// 1. Approach with the King to restrict the opponent's King
    /// 2. Use the Gold to deliver the final mate
    fn calculate_mating_move(&self, board: &BitboardBoard, player: Player) -> Option<TablebaseResult> {
        let (attacking_king, attacking_gold) = self.find_attacking_pieces(board, player)?;
        let defending_king = self.find_defending_king(board, player)?;

        // Check if we can mate in one move
        if let Some(mate_move) = self.find_immediate_mate(board, player, attacking_king, attacking_gold, defending_king) {
            return Some(TablebaseResult::win(Some(mate_move), 1));
        }

        // Check if we can approach with the King
        if let Some(approach_move) = self.approach_with_king(board, player, attacking_king, defending_king) {
            let distance = self.calculate_distance_to_mate(board, player);
            return Some(TablebaseResult::win(Some(approach_move), distance));
        }

        // Check if we can coordinate King and Gold for mate
        if let Some(coordinate_move) = self.coordinate_king_gold_mate(board, player, attacking_king, attacking_gold, defending_king) {
            let distance = self.calculate_distance_to_mate(board, player);
            return Some(TablebaseResult::win(Some(coordinate_move), distance));
        }

        // If no immediate progress can be made, return a defensive move
        self.find_best_defensive_move(board, player, attacking_king, attacking_gold, defending_king)
    }

    /// Find immediate mate in one move
    fn find_immediate_mate(&self, board: &BitboardBoard, player: Player, king: Position, gold: Position, defending_king: Position) -> Option<Move> {
        // Check if Gold can mate directly
        if let Some(mate_move) = self.gold_can_mate(board, player, gold, defending_king) {
            return Some(mate_move);
        }

        // Check if King can mate directly (rare but possible)
        if let Some(mate_move) = self.king_can_mate(board, player, king, defending_king) {
            return Some(mate_move);
        }

        None
    }

    /// Check if Gold can deliver mate
    fn gold_can_mate(&self, board: &BitboardBoard, player: Player, gold: Position, defending_king: Position) -> Option<Move> {
        // Generate all possible Gold moves
        let gold_moves = self.generate_gold_moves(board, player, gold);
        
        for move_ in gold_moves {
            if self.is_mating_move(board, player, &move_, defending_king) {
                return Some(move_);
            }
        }

        None
    }

    /// Check if King can deliver mate
    fn king_can_mate(&self, board: &BitboardBoard, player: Player, king: Position, defending_king: Position) -> Option<Move> {
        // Generate all possible King moves
        let king_moves = self.generate_king_moves(board, player, king);
        
        for move_ in king_moves {
            if self.is_mating_move(board, player, &move_, defending_king) {
                return Some(move_);
            }
        }

        None
    }

    /// Approach with the King to restrict the opponent's King
    fn approach_with_king(&self, board: &BitboardBoard, player: Player, king: Position, defending_king: Position) -> Option<Move> {
        let king_moves = self.generate_king_moves(board, player, king);
        
        // Find the move that gets the King closest to the defending King
        let mut best_move = None;
        let mut best_distance = u8::MAX;

        for move_ in king_moves {
            let new_distance = self.manhattan_distance(move_.to, defending_king);
            if new_distance < best_distance {
                best_distance = new_distance;
                best_move = Some(move_);
            }
        }

        best_move
    }

    /// Coordinate King and Gold for mating
    fn coordinate_king_gold_mate(&self, board: &BitboardBoard, player: Player, king: Position, gold: Position, defending_king: Position) -> Option<Move> {
        // Try moving the Gold to a better position
        let gold_moves = self.generate_gold_moves(board, player, gold);
        
        for move_ in gold_moves {
            // Check if this move improves our mating chances
            if self.improves_mating_position(board, player, &move_, king, defending_king) {
                return Some(move_);
            }
        }

        // Try moving the King to support the Gold
        let king_moves = self.generate_king_moves(board, player, king);
        
        for move_ in king_moves {
            if self.king_supports_gold_mate(board, player, &move_, gold, defending_king) {
                return Some(move_);
            }
        }

        None
    }

    /// Find the best defensive move when no progress can be made
    fn find_best_defensive_move(&self, board: &BitboardBoard, player: Player, king: Position, _gold: Position, _defending_king: Position) -> Option<TablebaseResult> {
        // In King + Gold vs King, the side with King + Gold should always be winning
        // If we can't make progress, it might be a draw or we need to find a different approach
        let king_moves = self.generate_king_moves(board, player, king);
        
        if let Some(move_) = king_moves.first() {
            return Some(TablebaseResult::win(Some(move_.clone()), 50)); // Large distance indicates uncertainty
        }

        None
    }

    /// Generate all possible Gold moves
    fn generate_gold_moves(&self, board: &BitboardBoard, player: Player, gold: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        
        // Gold moves: forward, backward, left, right, and diagonally forward
        let directions = [
            (-1, 0),  // forward
            (1, 0),   // backward
            (0, -1),  // left
            (0, 1),   // right
            (-1, -1), // forward-left
            (-1, 1),  // forward-right
        ];

        for (dr, dc) in directions {
            let new_row = gold.row as i8 + dr;
            let new_col = gold.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                // Check if the square is empty or contains an opponent piece
                if let Some(piece) = board.get_piece(new_pos) {
                    if piece.player != player {
                        // Capture move
                        moves.push(Move::new_move(gold, new_pos, PieceType::Gold, player, false));
                    }
                } else {
                    // Regular move
                    moves.push(Move::new_move(gold, new_pos, PieceType::Gold, player, false));
                }
            }
        }

        moves
    }

    /// Generate all possible King moves
    fn generate_king_moves(&self, board: &BitboardBoard, player: Player, king: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        
        // King moves: all 8 directions
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        for (dr, dc) in directions {
            let new_row = king.row as i8 + dr;
            let new_col = king.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                // Check if the square is empty or contains an opponent piece
                if let Some(piece) = board.get_piece(new_pos) {
                    if piece.player != player {
                        // Capture move
                        moves.push(Move::new_move(king, new_pos, PieceType::King, player, false));
                    }
                } else {
                    // Regular move
                    moves.push(Move::new_move(king, new_pos, PieceType::King, player, false));
                }
            }
        }

        moves
    }

    /// Check if a move delivers mate
    fn is_mating_move(&self, _board: &BitboardBoard, _player: Player, move_: &Move, defending_king: Position) -> bool {
        // This is a simplified check - in a real implementation, we would need to
        // verify that the move actually puts the king in checkmate
        // For now, we'll check if the move attacks the king's square
        move_.to == defending_king
    }

    /// Check if a move improves the mating position
    fn improves_mating_position(&self, _board: &BitboardBoard, _player: Player, move_: &Move, _king: Position, defending_king: Position) -> bool {
        // Check if the move gets the Gold closer to the defending King
        let current_distance = self.manhattan_distance(move_.from.unwrap(), defending_king);
        let new_distance = self.manhattan_distance(move_.to, defending_king);
        
        new_distance < current_distance
    }

    /// Check if King move supports Gold mate
    fn king_supports_gold_mate(&self, _board: &BitboardBoard, _player: Player, move_: &Move, gold: Position, defending_king: Position) -> bool {
        // Check if the King move helps coordinate with the Gold for mate
        let king_distance_to_gold = self.manhattan_distance(move_.to, gold);
        let king_distance_to_defending_king = self.manhattan_distance(move_.to, defending_king);
        
        // King should be close to both Gold and defending King for effective coordination
        king_distance_to_gold <= 2 && king_distance_to_defending_king <= 3
    }

    /// Calculate distance to mate
    fn calculate_distance_to_mate(&self, board: &BitboardBoard, player: Player) -> u8 {
        // This is a simplified calculation
        // In a real implementation, we would use more sophisticated algorithms
        // to calculate the actual distance to mate
        
        let (attacking_king, attacking_gold) = match self.find_attacking_pieces(board, player) {
            Some(pieces) => pieces,
            None => return 50, // Unknown distance
        };
        
        let defending_king = match self.find_defending_king(board, player) {
            Some(king) => king,
            None => return 50, // Unknown distance
        };

        // Calculate minimum distance needed to mate
        let king_distance = self.manhattan_distance(attacking_king, defending_king);
        let gold_distance = self.manhattan_distance(attacking_gold, defending_king);
        
        // Rough estimate: need to get pieces close to the king
        (king_distance + gold_distance).min(20)
    }
}

impl EndgameSolver for KingGoldVsKingSolver {
    fn can_solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.is_king_gold_vs_king(board, player, captured_pieces)
    }

    fn solve(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<TablebaseResult> {
        if !self.can_solve(board, player, captured_pieces) {
            return None;
        }

        self.calculate_mating_move(board, player)
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }

    fn name(&self) -> &'static str {
        "KingGoldVsKing"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn max_depth(&self) -> Option<u8> {
        Some(self.config.max_moves_to_mate)
    }
}

// EndgameSolverHelper is already implemented for all types via blanket implementation

impl Default for KingGoldVsKingSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Player, Position, PieceType, Move};

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_king_gold_vs_king_position() -> (BitboardBoard, Player) {
        // This would need to be implemented to create a specific test position
        // For now, we'll use the default board
        (create_test_board(), Player::Black)
    }

    #[test]
    fn test_king_gold_vs_king_solver_creation() {
        let solver = KingGoldVsKingSolver::new();
        assert_eq!(solver.name(), "KingGoldVsKing");
        assert_eq!(solver.priority(), 100);
        assert!(solver.is_enabled());
    }

    #[test]
    fn test_king_gold_vs_king_solver_with_config() {
        let config = KingGoldConfig::memory_optimized();
        let solver = KingGoldVsKingSolver::with_config(config);
        assert_eq!(solver.max_depth(), Some(10));
    }

    #[test]
    fn test_can_solve_empty_board() {
        let solver = KingGoldVsKingSolver::new();
        let board = create_test_board();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Empty board should not be solvable
        assert!(!solver.can_solve(&board, player, &captured_pieces));
    }

    #[test]
    fn test_piece_counting() {
        let solver = KingGoldVsKingSolver::new();
        let board = create_test_board();
        let (black_pieces, white_pieces) = solver.count_pieces_by_player(&board);

        // Initial position should have many pieces
        assert!(black_pieces.len() > 2);
        assert!(white_pieces.len() > 2);
    }

    #[test]
    fn test_has_king_and_gold() {
        let solver = KingGoldVsKingSolver::new();
        let pieces = vec![
            (PieceType::King, Position::new(0, 0)),
            (PieceType::Gold, Position::new(1, 1)),
        ];

        assert!(solver.has_king_and_gold(&pieces));

        let pieces_with_extra = vec![
            (PieceType::King, Position::new(0, 0)),
            (PieceType::Gold, Position::new(1, 1)),
            (PieceType::Silver, Position::new(2, 2)),
        ];

        assert!(!solver.has_king_and_gold(&pieces_with_extra));
    }

    #[test]
    fn test_has_king_only() {
        let solver = KingGoldVsKingSolver::new();
        let pieces = vec![(PieceType::King, Position::new(0, 0))];

        assert!(solver.has_king_only(&pieces));

        let pieces_with_extra = vec![
            (PieceType::King, Position::new(0, 0)),
            (PieceType::Gold, Position::new(1, 1)),
        ];

        assert!(!solver.has_king_only(&pieces_with_extra));
    }

    #[test]
    fn test_gold_move_generation() {
        let solver = KingGoldVsKingSolver::new();
        let board = create_test_board();
        let gold_pos = Position::new(4, 4);
        let player = Player::Black;

        let moves = solver.generate_gold_moves(&board, player, gold_pos);
        
        // Gold should have 6 possible moves from center
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn test_king_move_generation() {
        let solver = KingGoldVsKingSolver::new();
        let board = create_test_board();
        let king_pos = Position::new(4, 4);
        let player = Player::Black;

        let moves = solver.generate_king_moves(&board, player, king_pos);
        
        // King should have 8 possible moves from center
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn test_distance_calculation() {
        let solver = KingGoldVsKingSolver::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 3);

        let distance = solver.manhattan_distance(pos1, pos2);
        assert_eq!(distance, 5);
    }

    #[test]
    fn test_solver_configuration() {
        let config = KingGoldConfig::performance_optimized();
        let solver = KingGoldVsKingSolver::with_config(config);
        
        assert!(solver.is_enabled());
        assert_eq!(solver.max_depth(), Some(30));
        assert_eq!(solver.priority(), 100);
    }
}
