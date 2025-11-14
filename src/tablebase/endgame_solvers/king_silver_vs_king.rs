//! King + Silver vs King endgame solver
//!
//! This module implements the King + Silver vs King endgame solver,
//! which can find optimal moves in positions with only a king and silver
//! on one side versus a lone king on the other side.

use crate::bitboards::BitboardBoard;
use crate::tablebase::solver_traits::EndgameSolver;
use crate::tablebase::tablebase_config::KingSilverConfig;
use crate::tablebase::TablebaseResult;
use crate::types::{CapturedPieces, Move, Piece, PieceType, Player, Position};

/// Solver for King + Silver vs King endgames
///
/// This solver handles positions where one side has a king and silver
/// and the other side has only a king. The silver's unique movement
/// pattern (can move diagonally forward and backward, but only forward
/// straight) makes it different from the gold in mating patterns.
#[derive(Debug, Clone)]
pub struct KingSilverVsKingSolver {
    config: KingSilverConfig,
}

impl KingSilverVsKingSolver {
    /// Create a new KingSilverVsKingSolver with default configuration
    pub fn new() -> Self {
        Self {
            config: KingSilverConfig::default(),
        }
    }

    /// Create a new KingSilverVsKingSolver with custom configuration
    pub fn with_config(config: KingSilverConfig) -> Self {
        Self { config }
    }

    /// Check if the position is a King + Silver vs King endgame
    fn is_king_silver_vs_king(&self, board: &BitboardBoard, player: Player) -> bool {
        let pieces = self.extract_pieces(board, player);

        // Check if we have exactly 2 pieces (king + silver)
        if pieces.len() != 2 {
            return false;
        }

        let mut has_king = false;
        let mut has_silver = false;

        for (piece, _) in pieces {
            match piece.piece_type {
                PieceType::King => has_king = true,
                PieceType::Silver => has_silver = true,
                _ => return false, // Other piece types not allowed
            }
        }

        has_king && has_silver
    }

    /// Extract pieces for the given player
    fn extract_pieces(&self, board: &BitboardBoard, player: Player) -> Vec<(Piece, Position)> {
        let mut pieces = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == player {
                        pieces.push((*piece, Position { row, col }));
                    }
                }
            }
        }

        pieces
    }

    /// Find the best move in a King + Silver vs King position
    fn find_best_move(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<Move> {
        if !self.is_king_silver_vs_king(board, player) {
            return None;
        }

        // Get all legal moves for the current player
        let moves = self.generate_moves(board, player, captured_pieces);

        if moves.is_empty() {
            return None;
        }

        // Extract pieces for evaluation
        let pieces = self.extract_pieces(board, player);
        let (king, silver) = self.find_king_and_silver(&pieces);
        let defending_king = self.find_defending_king(board, player);

        if let (Some(king_pos), Some(silver_pos), Some(def_king_pos)) =
            (king, silver, defending_king)
        {
            // Look for immediate checkmate
            for move_ in &moves {
                if self.is_mating_move(board, player, move_, def_king_pos) {
                    return Some(move_.clone());
                }
            }

            // Look for moves that improve our mating position
            let mut best_move = None;
            let mut best_score = i32::MIN;

            for move_ in &moves {
                let score =
                    self.evaluate_move(board, player, move_, king_pos, silver_pos, def_king_pos);
                if score > best_score {
                    best_score = score;
                    best_move = Some(move_.clone());
                }
            }

            best_move
        } else {
            moves.first().cloned()
        }
    }

    /// Generate all legal moves for the current player
    fn generate_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == player {
                        let from = Position { row, col };
                        let piece_moves =
                            self.generate_piece_moves(board, *piece, from, captured_pieces);
                        moves.extend(piece_moves);
                    }
                }
            }
        }

        moves
    }

    /// Generate moves for a specific piece
    fn generate_piece_moves(
        &self,
        board: &BitboardBoard,
        piece: Piece,
        from: Position,
        _captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        match piece.piece_type {
            PieceType::King => {
                // King can move to any adjacent square
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }

                        let new_row = (from.row as i32 + dr) as u8;
                        let new_col = (from.col as i32 + dc) as u8;

                        if new_row < 9 && new_col < 9 {
                            let to = Position {
                                row: new_row,
                                col: new_col,
                            };
                            if self.is_legal_move(board, from, to, piece) {
                                moves.push(Move::new_move(
                                    from,
                                    to,
                                    piece.piece_type,
                                    piece.player,
                                    false,
                                ));
                            }
                        }
                    }
                }
            }
            PieceType::Silver => {
                // Silver can move diagonally forward and backward, and straight forward
                let directions = if piece.player == Player::Black {
                    vec![(-1, -1), (-1, 1), (1, -1), (1, 1), (-1, 0)] // Black silver directions
                } else {
                    vec![(1, -1), (1, 1), (-1, -1), (-1, 1), (1, 0)] // White silver directions
                };

                for (dr, dc) in directions {
                    let new_row = (from.row as i32 + dr) as u8;
                    let new_col = (from.col as i32 + dc) as u8;

                    if new_row < 9 && new_col < 9 {
                        let to = Position {
                            row: new_row,
                            col: new_col,
                        };
                        if self.is_legal_move(board, from, to, piece) {
                            moves.push(Move::new_move(
                                from,
                                to,
                                piece.piece_type,
                                piece.player,
                                false,
                            ));
                        }
                    }
                }
            }
            _ => {} // Other piece types not handled in this solver
        }

        moves
    }

    /// Check if a move is legal
    fn is_legal_move(
        &self,
        board: &BitboardBoard,
        _from: Position,
        to: Position,
        piece: Piece,
    ) -> bool {
        // Check if destination is within bounds
        if to.row >= 9 || to.col >= 9 {
            return false;
        }

        // Check if destination is empty or contains opponent piece
        if let Some(target_piece) = board.get_piece(to) {
            if target_piece.player == piece.player {
                return false; // Can't capture own piece
            }
        }

        // TODO: Add more sophisticated legality checks
        true
    }

    /// Check if the position is a checkmate
    fn is_checkmate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Use the board's built-in checkmate detection
        board.is_checkmate(player, captured_pieces)
    }

    /// Check if the position is a stalemate
    fn is_stalemate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Use the board's built-in stalemate detection
        board.is_stalemate(player, captured_pieces)
    }

    /// Find the king and silver pieces from the extracted pieces
    fn find_king_and_silver(
        &self,
        pieces: &[(Piece, Position)],
    ) -> (Option<Position>, Option<Position>) {
        let mut king = None;
        let mut silver = None;

        for (piece, pos) in pieces {
            match piece.piece_type {
                PieceType::King => king = Some(*pos),
                PieceType::Silver => silver = Some(*pos),
                _ => {}
            }
        }

        (king, silver)
    }

    /// Find the defending king (opponent's king)
    fn find_defending_king(&self, board: &BitboardBoard, player: Player) -> Option<Position> {
        let opponent = player.opposite();
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == opponent && piece.piece_type == PieceType::King {
                        return Some(Position { row, col });
                    }
                }
            }
        }
        None
    }

    /// Check if a move results in checkmate
    fn is_mating_move(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        _defending_king: Position,
    ) -> bool {
        // Make the move on a temporary board
        let mut temp_board = board.clone();
        let mut temp_captured = CapturedPieces::new();
        
        // Capture piece if move captures
        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, player);
        }
        
        // Check if the opponent is now in checkmate
        let opponent = player.opposite();
        temp_board.is_checkmate(opponent, &temp_captured)
    }

    /// Evaluate a move's quality in the King + Silver vs King endgame
    fn evaluate_move(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        king: Position,
        silver: Position,
        defending_king: Position,
    ) -> i32 {
        let mut score = 0;

        // Prefer moves that bring pieces closer to the defending king
        if let Some(from) = move_.from {
            let distance_before = self.manhattan_distance(from, defending_king);
            let distance_after = self.manhattan_distance(move_.to, defending_king);

            if distance_after < distance_before {
                score += 100;
            }
        }

        // Prefer moves that coordinate king and silver
        if self.coordinates_king_silver(board, player, move_, king, silver) {
            score += 50;
        }

        // Prefer moves that restrict the defending king's mobility
        if self.restricts_king_mobility(board, player, move_, defending_king) {
            score += 30;
        }

        score
    }

    /// Calculate Manhattan distance between two positions
    fn manhattan_distance(&self, from: Position, to: Position) -> i32 {
        ((from.row as i32 - to.row as i32).abs() + (from.col as i32 - to.col as i32).abs()) as i32
    }

    /// Check if a move coordinates the king and silver effectively
    fn coordinates_king_silver(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        move_: &Move,
        _king: Position,
        silver: Position,
    ) -> bool {
        // TODO: Implement proper coordination logic
        // For now, return true if the move is by the silver piece
        if let Some(from) = move_.from {
            from == silver
        } else {
            false
        }
    }

    /// Check if a move restricts the defending king's mobility
    fn restricts_king_mobility(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _move_: &Move,
        _defending_king: Position,
    ) -> bool {
        // TODO: Implement mobility restriction logic
        // For now, return false
        false
    }

    /// Calculate distance to mate using search-based DTM calculation
    fn calculate_distance_to_mate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u8 {
        use super::dtm_calculator::calculate_dtm;
        
        // Use search-based DTM calculation with max depth limit
        // For K+S vs K, mate is typically achievable within 35 moves
        let max_depth = 35;
        
        // Calculate actual DTM using iterative deepening search
        if let Some(dtm) = calculate_dtm(board, player, captured_pieces, max_depth) {
            dtm
        } else {
            // If search doesn't find mate within max_depth, use heuristic fallback
            let pieces = self.extract_pieces(board, player);
            let (king, silver) = self.find_king_and_silver(&pieces);
            let defending_king = self.find_defending_king(board, player);

            if let (Some(king_pos), Some(silver_pos), Some(def_king_pos)) =
                (king, silver, defending_king)
            {
                // Heuristic: estimate based on piece coordination
                let king_distance = self.manhattan_distance(king_pos, def_king_pos);
                let silver_distance = self.manhattan_distance(silver_pos, def_king_pos);
                
                // Better estimate: consider piece coordination
                let avg_distance = (king_distance + silver_distance) / 2;
                
                // Estimate: Silver is less powerful than Gold, usually takes longer
                // Typically 1.8x the average distance for coordination
                ((avg_distance * 9) / 5).min(35) as u8
            } else {
                30 // Unknown distance
            }
        }
    }
}

impl EndgameSolver for KingSilverVsKingSolver {
    fn can_solve(
        &self,
        board: &BitboardBoard,
        player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.is_king_silver_vs_king(board, player)
    }

    fn solve(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TablebaseResult> {
        if !self.can_solve(board, player, captured_pieces) {
            return None;
        }

        if let Some(best_move) = self.find_best_move(board, player, captured_pieces) {
            if self.is_checkmate(board, player, captured_pieces) {
                Some(TablebaseResult::win(Some(best_move), 0))
            } else if self.is_stalemate(board, player, captured_pieces) {
                Some(TablebaseResult::draw())
            } else {
                let distance = self.calculate_distance_to_mate(board, player, captured_pieces);
                Some(TablebaseResult::win(Some(best_move), distance))
            }
        } else {
            Some(TablebaseResult::loss(0))
        }
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }

    fn name(&self) -> &'static str {
        "KingSilverVsKing"
    }
}

impl Default for KingSilverVsKingSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_silver_vs_king_detection() {
        let solver = KingSilverVsKingSolver::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test with empty board (should not be K+S vs K)
        assert!(!solver.can_solve(&board, Player::Black, &captured_pieces));
    }

    #[test]
    fn test_solver_creation() {
        let solver = KingSilverVsKingSolver::new();
        assert_eq!(solver.name(), "KingSilverVsKing");
        assert_eq!(solver.priority(), 90); // Default priority for silver solver
    }

    #[test]
    fn test_solver_with_config() {
        let config = KingSilverConfig {
            enabled: true,
            priority: 5,
            max_moves_to_mate: 20,
            use_pattern_matching: true,
            pattern_cache_size: 1000,
        };
        let solver = KingSilverVsKingSolver::with_config(config);
        assert_eq!(solver.priority(), 5);
    }

    #[test]
    fn test_piece_extraction() {
        let solver = KingSilverVsKingSolver::new();
        let board = BitboardBoard::empty();
        let pieces = solver.extract_pieces(&board, Player::Black);

        // Empty board should have no pieces
        assert_eq!(pieces.len(), 0);
    }

    #[test]
    fn test_move_generation() {
        let solver = KingSilverVsKingSolver::new();
        let board = BitboardBoard::empty();
        let captured_pieces = CapturedPieces::new();
        let moves = solver.generate_moves(&board, Player::Black, &captured_pieces);

        // Empty board should have no moves
        assert_eq!(moves.len(), 0);
    }
}
