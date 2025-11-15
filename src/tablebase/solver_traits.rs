//! Traits and interfaces for endgame solvers
//!
//! This module defines the common interface that all endgame solvers must implement.
//! It provides a trait-based architecture that allows for easy extension and
//! modular implementation of different endgame scenarios.

use super::TablebaseResult;
use crate::types::core::Player;
use crate::BitboardBoard;
use crate::CapturedPieces;

/// Trait that all endgame solvers must implement
///
/// This trait provides a common interface for solving specific endgame positions.
/// Each solver is responsible for recognizing positions it can handle and
/// providing the optimal move for those positions.
pub trait EndgameSolver: Send + Sync {
    /// Check if this solver can handle the given position
    ///
    /// This method should quickly determine if the position matches the
    /// endgame pattern that this solver is designed to handle.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces for both players
    ///
    /// # Returns
    /// `true` if this solver can handle the position, `false` otherwise
    fn can_solve(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool;

    /// Solve the position and return the optimal move
    ///
    /// This method should only be called if `can_solve()` returns `true`.
    /// It should calculate the optimal move for the position and return
    /// a complete `TablebaseResult` with move, outcome, and distance to mate.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces for both players
    ///
    /// # Returns
    /// `Some(TablebaseResult)` if the position can be solved, `None` otherwise
    fn solve(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TablebaseResult>;

    /// Get the solver's priority (higher = more important)
    ///
    /// When multiple solvers can handle the same position, the one with
    /// the highest priority will be used. This allows for more specific
    /// solvers to take precedence over general ones.
    ///
    /// # Returns
    /// Priority value (0-255, higher is more important)
    fn priority(&self) -> u8;

    /// Get the solver's name for debugging and statistics
    ///
    /// This name will be used in debug output and statistics tracking.
    ///
    /// # Returns
    /// A static string containing the solver's name
    fn name(&self) -> &'static str;

    /// Check if the solver is enabled
    ///
    /// This allows solvers to be temporarily disabled without removing
    /// them from the solver list.
    ///
    /// # Returns
    /// `true` if the solver is enabled, `false` otherwise
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get the maximum depth this solver can handle
    ///
    /// Some solvers may only work for positions within a certain
    /// distance to mate. This method returns the maximum depth
    /// (moves to mate) that this solver can handle.
    ///
    /// # Returns
    /// Maximum depth, or `None` if there's no limit
    fn max_depth(&self) -> Option<u8> {
        None
    }

    /// Get solver-specific configuration
    ///
    /// This method can be used to provide solver-specific configuration
    /// information for debugging or optimization purposes.
    ///
    /// # Returns
    /// A string containing configuration information
    fn get_config_info(&self) -> String {
        format!(
            "{} (priority: {}, enabled: {})",
            self.name(),
            self.priority(),
            self.is_enabled()
        )
    }
}

/// Helper trait for common endgame solver functionality
///
/// This trait provides common helper methods that many endgame solvers
/// will find useful, such as piece counting and position validation.
pub trait EndgameSolverHelper {
    /// Count the total number of pieces on the board
    fn count_pieces(&self, board: &BitboardBoard) -> u8 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if let Some(_) = board.get_piece(crate::types::Position::new(row, col)) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Check if there are any captured pieces
    fn has_captured_pieces(&self, captured_pieces: &CapturedPieces) -> bool {
        !captured_pieces.black.is_empty() || !captured_pieces.white.is_empty()
    }

    /// Check if the position has exactly the specified number of pieces
    fn has_exact_piece_count(&self, board: &BitboardBoard, count: u8) -> bool {
        self.count_pieces(board) == count
    }

    /// Check if the position has no captured pieces and exact piece count
    fn is_clean_endgame(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        piece_count: u8,
    ) -> bool {
        !self.has_captured_pieces(captured_pieces) && self.has_exact_piece_count(board, piece_count)
    }

    /// Extract all pieces from the board as a vector
    fn extract_pieces(
        &self,
        board: &BitboardBoard,
    ) -> Vec<(crate::types::Piece, crate::types::Position)> {
        let mut pieces = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(crate::types::Position::new(row, col)) {
                    pieces.push((piece, crate::types::Position::new(row, col)));
                }
            }
        }
        pieces
    }

    /// Find pieces of a specific type and player
    fn find_pieces(
        &self,
        board: &BitboardBoard,
        piece_type: crate::types::PieceType,
        player: Player,
    ) -> Vec<crate::types::Position> {
        let mut positions = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(crate::types::Position::new(row, col)) {
                    if piece.piece_type == piece_type && piece.player == player {
                        positions.push(crate::types::Position::new(row, col));
                    }
                }
            }
        }
        positions
    }

    /// Calculate the Manhattan distance between two positions
    fn manhattan_distance(&self, pos1: crate::types::Position, pos2: crate::types::Position) -> u8 {
        ((pos1.row as i8 - pos2.row as i8).abs() + (pos1.col as i8 - pos2.col as i8).abs()) as u8
    }

    /// Calculate the Chebyshev distance (max of row and column differences) between two positions
    fn chebyshev_distance(&self, pos1: crate::types::Position, pos2: crate::types::Position) -> u8 {
        ((pos1.row as i8 - pos2.row as i8)
            .abs()
            .max((pos1.col as i8 - pos2.col as i8).abs())) as u8
    }
}

/// Default implementation of EndgameSolverHelper for all types
impl<T> EndgameSolverHelper for T {}

/// Trait for solvers that can provide additional analysis
///
/// Some solvers may be able to provide additional analysis beyond just
/// the optimal move, such as alternative lines or position evaluation.
pub trait AdvancedEndgameSolver: EndgameSolver {
    /// Get alternative moves for the position
    ///
    /// This method can provide alternative moves that are also good,
    /// which can be useful for move ordering or analysis.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces for both players
    ///
    /// # Returns
    /// Vector of alternative moves with their scores
    fn get_alternative_moves(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> Vec<(crate::types::Move, i32)> {
        // Default implementation returns empty vector
        Vec::new()
    }

    /// Analyze the position and provide detailed information
    ///
    /// This method can provide detailed analysis of the position,
    /// including tactical themes, strategic considerations, etc.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces for both players
    ///
    /// # Returns
    /// String containing detailed analysis
    fn analyze_position(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> String {
        format!(
            "Position analysis by {}: Basic analysis available",
            self.name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, Piece, PieceType, Player, Position};
    use crate::BitboardBoard;
    use crate::CapturedPieces;

    // Mock solver for testing
    struct MockSolver {
        name: &'static str,
        priority: u8,
        enabled: bool,
        can_solve_result: bool,
        solve_result: Option<TablebaseResult>,
    }

    impl MockSolver {
        fn new(name: &'static str, priority: u8) -> Self {
            Self {
                name,
                priority,
                enabled: true,
                can_solve_result: false,
                solve_result: None,
            }
        }

        fn with_can_solve(mut self, can_solve: bool) -> Self {
            self.can_solve_result = can_solve;
            self
        }

        fn with_solve_result(mut self, result: Option<TablebaseResult>) -> Self {
            self.solve_result = result;
            self
        }

        fn with_enabled(mut self, enabled: bool) -> Self {
            self.enabled = enabled;
            self
        }
    }

    impl EndgameSolver for MockSolver {
        fn can_solve(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> bool {
            self.can_solve_result
        }

        fn solve(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> Option<TablebaseResult> {
            self.solve_result.clone()
        }

        fn priority(&self) -> u8 {
            self.priority
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    impl AdvancedEndgameSolver for MockSolver {
        fn get_alternative_moves(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> Vec<(crate::types::Move, i32)> {
            vec![]
        }

        fn analyze_position(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> String {
            format!("Mock analysis by {}", self.name)
        }
    }

    #[test]
    fn test_endgame_solver_trait() {
        let solver = MockSolver::new("TestSolver", 100);
        assert_eq!(solver.name(), "TestSolver");
        assert_eq!(solver.priority(), 100);
        assert!(solver.is_enabled());
        assert_eq!(solver.max_depth(), None);
    }

    #[test]
    fn test_endgame_solver_can_solve() {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let solver = MockSolver::new("TestSolver", 100).with_can_solve(true);

        assert!(solver.can_solve(&board, player, &captured_pieces));

        let solver_disabled = MockSolver::new("TestSolver", 100)
            .with_can_solve(true)
            .with_enabled(false);

        assert!(!solver_disabled.is_enabled());
    }

    #[test]
    fn test_endgame_solver_solve() {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let move_ = Move::new_move(
            Position::new(0, 0),
            Position::new(1, 1),
            PieceType::King,
            Player::Black,
            false,
        );

        let result = TablebaseResult::win(Some(move_), 5);
        let solver = MockSolver::new("TestSolver", 100).with_solve_result(Some(result));

        let solve_result = solver.solve(&board, player, &captured_pieces);
        assert!(solve_result.is_some());
        assert!(solve_result.unwrap().is_winning());
    }

    #[test]
    fn test_endgame_solver_helper() {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let helper = MockSolver::new("TestSolver", 100);

        // Test piece counting
        let count = helper.count_pieces(&board);
        assert!(count > 0); // Initial position has pieces

        // Test captured pieces check
        assert!(!helper.has_captured_pieces(&captured_pieces));

        // Test exact piece count
        assert!(helper.has_exact_piece_count(&board, count));

        // Test clean endgame
        assert!(helper.is_clean_endgame(&board, &captured_pieces, count));

        // Test piece extraction
        let pieces = helper.extract_pieces(&board);
        assert_eq!(pieces.len(), count as usize);

        // Test distance calculations
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(2, 3);
        assert_eq!(helper.manhattan_distance(pos1, pos2), 5);
        assert_eq!(helper.chebyshev_distance(pos1, pos2), 3);
    }

    #[test]
    fn test_advanced_endgame_solver() {
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let solver = MockSolver::new("AdvancedSolver", 100);

        // Test alternative moves
        let alternatives = solver.get_alternative_moves(&board, player, &captured_pieces);
        assert!(alternatives.is_empty());

        // Test position analysis
        let analysis = solver.analyze_position(&board, player, &captured_pieces);
        assert!(analysis.contains("Mock analysis by AdvancedSolver"));
    }

    #[test]
    fn test_solver_config_info() {
        let solver = MockSolver::new("TestSolver", 100);
        let config_info = solver.get_config_info();
        assert!(config_info.contains("TestSolver"));
        assert!(config_info.contains("priority: 100"));
        assert!(config_info.contains("enabled: true"));
    }
}
