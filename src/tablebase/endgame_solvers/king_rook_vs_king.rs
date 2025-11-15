//! King + Rook vs King endgame solver
//!
//! This module implements the King + Rook vs King endgame solver,
//! which can find optimal moves in positions with only a king and rook
//! on one side versus a lone king on the other side.

use crate::bitboards::BitboardBoard;
use crate::tablebase::solver_traits::EndgameSolver;
use crate::tablebase::tablebase_config::KingRookConfig;
use crate::tablebase::TablebaseResult;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Piece, PieceType, Player, Position};

/// Solver for King + Rook vs King endgames
///
/// This solver handles positions where one side has a king and rook
/// and the other side has only a king. The rook's ability to move
/// horizontally and vertically makes it very powerful in endgames.
#[derive(Debug, Clone)]
pub struct KingRookVsKingSolver {
    config: KingRookConfig,
}

impl KingRookVsKingSolver {
    /// Create a new KingRookVsKingSolver with default configuration
    pub fn new() -> Self {
        Self {
            config: KingRookConfig::default(),
        }
    }

    /// Create a new KingRookVsKingSolver with custom configuration
    pub fn with_config(config: KingRookConfig) -> Self {
        Self { config }
    }

    /// Check if the position is a King + Rook vs King endgame
    fn is_king_rook_vs_king(&self, board: &BitboardBoard, player: Player) -> bool {
        let pieces = self.extract_pieces(board, player);

        // Check if we have exactly 2 pieces (king + rook)
        if pieces.len() != 2 {
            return false;
        }

        let mut has_king = false;
        let mut has_rook = false;

        for (piece, _) in pieces {
            match piece.piece_type {
                PieceType::King => has_king = true,
                PieceType::Rook => has_rook = true,
                _ => return false, // Other piece types not allowed
            }
        }

        has_king && has_rook
    }

    /// Extract pieces for the given player
    fn extract_pieces(&self, board: &BitboardBoard, player: Player) -> Vec<(Piece, Position)> {
        let mut pieces = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == player {
                        pieces.push((piece, Position { row, col }));
                    }
                }
            }
        }

        pieces
    }

    /// Find the best move in a King + Rook vs King position
    fn find_best_move(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<Move> {
        if !self.is_king_rook_vs_king(board, player) {
            return None;
        }

        // Get all legal moves for the current player
        let moves = self.generate_moves(board, player, captured_pieces);

        if moves.is_empty() {
            return None;
        }

        // Extract pieces for evaluation
        let pieces = self.extract_pieces(board, player);
        let (king, rook) = self.find_king_and_rook(&pieces);
        let defending_king = self.find_defending_king(board, player);

        if let (Some(king_pos), Some(rook_pos), Some(def_king_pos)) = (king, rook, defending_king) {
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
                let score = self.evaluate_move(
                    board,
                    player,
                    move_,
                    king_pos,
                    rook_pos,
                    def_king_pos,
                    captured_pieces,
                );
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
                            self.generate_piece_moves(board, piece, from, captured_pieces);
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
        captured_pieces: &CapturedPieces,
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
                            let mut candidate =
                                Move::new_move(from, to, piece.piece_type, piece.player, false);
                            candidate.is_capture =
                                matches!(board.get_piece(to), Some(p) if p.player != piece.player);

                            if self.is_legal_move(board, captured_pieces, &candidate) {
                                moves.push(candidate);
                            }
                        }
                    }
                }
            }
            PieceType::Rook => {
                // Rook can move horizontally and vertically
                let directions = vec![(0, 1), (0, -1), (1, 0), (-1, 0)];

                for (dr, dc) in directions {
                    for distance in 1..9 {
                        let new_row = (from.row as i32 + dr * distance) as u8;
                        let new_col = (from.col as i32 + dc * distance) as u8;

                        if new_row >= 9 || new_col >= 9 {
                            break; // Out of bounds
                        }

                        let to = Position {
                            row: new_row,
                            col: new_col,
                        };

                        let mut candidate =
                            Move::new_move(from, to, piece.piece_type, piece.player, false);
                        candidate.is_capture =
                            matches!(board.get_piece(to), Some(p) if p.player != piece.player);

                        if self.is_legal_move(board, captured_pieces, &candidate) {
                            moves.push(candidate);

                            // If we hit a piece, we can't go further
                            if board.get_piece(to).is_some() {
                                break;
                            }
                        } else {
                            break; // Can't move in this direction
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
        captured_pieces: &CapturedPieces,
        move_: &Move,
    ) -> bool {
        let Some(from) = move_.from else {
            return false;
        };

        if from.row >= 9 || from.col >= 9 || move_.to.row >= 9 || move_.to.col >= 9 {
            return false;
        }

        if let Some(target_piece) = board.get_piece(move_.to) {
            if target_piece.player == move_.player {
                return false;
            }
        }

        if !captured_pieces.black.is_empty() || !captured_pieces.white.is_empty() {
            return false;
        }

        let Some(piece_to_move) = board.get_piece(from) else {
            return false;
        };

        if piece_to_move.player != move_.player || piece_to_move.piece_type != move_.piece_type {
            return false;
        }

        let mut temp_board = board.clone();
        let mut temp_captured = captured_pieces.clone();
        let mut temp_move = move_.clone();
        temp_move.is_capture =
            matches!(board.get_piece(move_.to), Some(p) if p.player != move_.player);

        if let Some(captured_piece) = temp_board.make_move(&temp_move) {
            temp_captured.add_piece(captured_piece.piece_type, move_.player);
        } else if temp_move.is_capture {
            return false;
        }

        !temp_board.is_king_in_check(move_.player, &temp_captured)
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

    /// Find the king and rook pieces from the extracted pieces
    fn find_king_and_rook(
        &self,
        pieces: &[(Piece, Position)],
    ) -> (Option<Position>, Option<Position>) {
        let mut king = None;
        let mut rook = None;

        for (piece, pos) in pieces {
            match piece.piece_type {
                PieceType::King => king = Some(*pos),
                PieceType::Rook => rook = Some(*pos),
                _ => {}
            }
        }

        (king, rook)
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

    /// Evaluate a move's quality in the King + Rook vs King endgame
    fn evaluate_move(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        king: Position,
        rook: Position,
        defending_king: Position,
        captured_pieces: &CapturedPieces,
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

        // Prefer moves that coordinate king and rook
        if self.coordinates_king_rook(board, player, move_, king, rook) {
            score += 50;
        }

        // Prefer moves that restrict the defending king's mobility
        if self.restricts_king_mobility(board, player, move_, defending_king, captured_pieces) {
            score += 30;
        }

        // Rook-specific: prefer moves that control key squares
        if self.controls_key_squares(board, player, move_, defending_king) {
            score += 40;
        }

        score
    }

    /// Calculate Manhattan distance between two positions
    fn manhattan_distance(&self, from: Position, to: Position) -> i32 {
        ((from.row as i32 - to.row as i32).abs() + (from.col as i32 - to.col as i32).abs()) as i32
    }

    /// Check if a move coordinates the king and rook effectively
    fn coordinates_king_rook(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        king: Position,
        rook: Position,
    ) -> bool {
        // Check which piece is moving
        let moving_piece = if let Some(from) = move_.from {
            if from == king {
                PieceType::King
            } else if from == rook {
                PieceType::Rook
            } else {
                return false; // Not king or rook
            }
        } else {
            return false;
        };

        let defending_king = match self.find_defending_king(board, player) {
            Some(king_pos) => king_pos,
            None => return false,
        };

        match moving_piece {
            PieceType::King => {
                // King coordinates when it's close to both rook and defending king
                let king_to_rook = self.manhattan_distance(move_.to, rook);
                let king_to_def_king = self.manhattan_distance(move_.to, defending_king);

                // King should be within 3 squares of rook and within 4 of defending king
                // Rook is powerful at range, so coordination can work from further away
                king_to_rook <= 3 && king_to_def_king <= 4
            }
            PieceType::Rook => {
                // Rook coordinates when it can attack defending king and king can support
                let rook_to_king = self.manhattan_distance(move_.to, king);
                let rook_to_def_king = self.manhattan_distance(move_.to, defending_king);

                // Rook should be within 3 squares of king and close to defending king
                // Rook can attack from distance, so check if it's on same rank/file as defending king
                let on_same_rank_or_file =
                    move_.to.row == defending_king.row || move_.to.col == defending_king.col;
                rook_to_king <= 3 && (rook_to_def_king <= 4 || on_same_rank_or_file)
            }
            _ => false,
        }
    }

    /// Check if a move restricts the defending king's mobility
    fn restricts_king_mobility(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        defending_king: Position,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Make the move on a temporary board to see the resulting position
        let mut temp_board = board.clone();
        let mut temp_captured = CapturedPieces::new();

        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, player);
        }

        // Count legal moves for defending king before and after the move
        let opponent = player.opposite();
        let move_generator = crate::moves::MoveGenerator::new();
        let moves_before = move_generator.generate_legal_moves(board, opponent, captured_pieces);
        let moves_after =
            move_generator.generate_legal_moves(&temp_board, opponent, &temp_captured);

        // If the move reduces the number of legal moves available to the defending king, it restricts mobility
        let restricts_by_reducing_moves = moves_after.len() < moves_before.len();

        // Check if move attacks squares adjacent to defending king or controls key squares
        let move_attacks_escape_squares = {
            let mut attacks_escape = false;
            // Check all 8 adjacent squares to defending king
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let escape_row = (defending_king.row as i8 + dr) as u8;
                    let escape_col = (defending_king.col as i8 + dc) as u8;

                    if escape_row < 9 && escape_col < 9 {
                        let escape_pos = Position {
                            row: escape_row,
                            col: escape_col,
                        };
                        // If the move's destination attacks this escape square, it restricts mobility
                        if move_.to == escape_pos
                            || temp_board.is_square_attacked_by(escape_pos, player)
                        {
                            attacks_escape = true;
                            break;
                        }
                    }
                }
                if attacks_escape {
                    break;
                }
            }
            attacks_escape
        };

        restricts_by_reducing_moves || move_attacks_escape_squares
    }

    /// Check if a move controls key squares for rook mating patterns
    fn controls_key_squares(
        &self,
        board: &BitboardBoard,
        player: Player,
        move_: &Move,
        defending_king: Position,
    ) -> bool {
        // For rook mating patterns, key squares are:
        // 1. Squares on the same rank/file as defending king (rook can attack from distance)
        // 2. Squares that cut off the defending king's escape routes
        // 3. Squares that support the king in approaching the defending king

        // Make the move on a temporary board
        let mut temp_board = board.clone();
        let mut temp_captured = CapturedPieces::new();

        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, player);
        }

        // Key square control: Rook should control squares that restrict the defending king
        // Check if the move places rook on same rank or file as defending king
        let on_same_rank_or_file =
            move_.to.row == defending_king.row || move_.to.col == defending_king.col;

        // Check if rook can now attack squares that the defending king might escape to
        // This includes squares adjacent to the defending king
        let controls_escape_squares = {
            let mut controls = false;
            // Check squares adjacent to defending king
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let escape_row = (defending_king.row as i8 + dr) as u8;
                    let escape_col = (defending_king.col as i8 + dc) as u8;

                    if escape_row < 9 && escape_col < 9 {
                        let escape_pos = Position {
                            row: escape_row,
                            col: escape_col,
                        };
                        // Rook can attack if on same rank or file
                        if temp_board.is_square_attacked_by(escape_pos, player) {
                            controls = true;
                            break;
                        }
                    }
                }
                if controls {
                    break;
                }
            }
            controls
        };

        // Rook controls key squares if it's on same rank/file OR if it attacks escape squares
        on_same_rank_or_file || controls_escape_squares
    }

    /// Calculate distance to mate using search-based DTM calculation
    fn calculate_distance_to_mate(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> u8 {
        use super::dtm_calculator::calculate_dtm;

        // Use search-based DTM calculation with a conservative depth limit
        let max_depth = self.config.max_moves_to_mate.min(12);

        // Calculate actual DTM using iterative deepening search
        if let Some(dtm) = calculate_dtm(board, player, captured_pieces, max_depth) {
            dtm
        } else {
            // If search doesn't find mate within max_depth, use heuristic fallback
            let pieces = self.extract_pieces(board, player);
            let (king, rook) = self.find_king_and_rook(&pieces);
            let defending_king = self.find_defending_king(board, player);

            if let (Some(king_pos), Some(rook_pos), Some(def_king_pos)) =
                (king, rook, defending_king)
            {
                // Heuristic: estimate based on piece coordination
                let king_distance = self.manhattan_distance(king_pos, def_king_pos);
                let rook_distance = self.manhattan_distance(rook_pos, def_king_pos);

                // Better estimate: Rook is powerful, but coordination still needed
                let avg_distance = (king_distance + rook_distance) / 2;

                // Estimate: Rook is powerful, usually takes 1.3x the average distance
                ((avg_distance * 13) / 10).min(25) as u8
            } else {
                20 // Unknown distance
            }
        }
    }
}

impl EndgameSolver for KingRookVsKingSolver {
    fn can_solve(
        &self,
        board: &BitboardBoard,
        player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.is_king_rook_vs_king(board, player)
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
        "KingRookVsKing"
    }
}

impl Default for KingRookVsKingSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CapturedPieces, Move, Piece, PieceType, Player, Position};

    fn build_board(pieces: &[(Player, PieceType, Position)]) -> BitboardBoard {
        let mut board = BitboardBoard::empty();
        for (player, piece_type, position) in pieces {
            board.place_piece(
                Piece {
                    piece_type: *piece_type,
                    player: *player,
                },
                *position,
            );
        }
        board
    }

    fn rook_checkmate_position() -> (BitboardBoard, CapturedPieces) {
        let board = build_board(&[
            (Player::Black, PieceType::King, Position::new(2, 4)),
            (Player::Black, PieceType::Rook, Position::new(0, 2)),
            (Player::White, PieceType::King, Position::new(0, 4)),
        ]);
        (board, CapturedPieces::new())
    }

    fn rook_stalemate_position() -> (BitboardBoard, CapturedPieces) {
        let board = build_board(&[
            (Player::Black, PieceType::King, Position::new(2, 2)),
            (Player::Black, PieceType::Rook, Position::new(1, 1)),
            (Player::White, PieceType::King, Position::new(0, 0)),
        ]);
        (board, CapturedPieces::new())
    }

    fn rook_distance_position() -> (BitboardBoard, CapturedPieces) {
        let board = build_board(&[
            (Player::Black, PieceType::King, Position::new(4, 4)),
            (Player::Black, PieceType::Rook, Position::new(6, 6)),
            (Player::White, PieceType::King, Position::new(8, 8)),
        ]);
        (board, CapturedPieces::new())
    }

    #[test]
    fn test_king_rook_vs_king_detection() {
        let solver = KingRookVsKingSolver::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test with empty board (should not be K+R vs K)
        assert!(!solver.can_solve(&board, Player::Black, &captured_pieces));
    }

    #[test]
    fn test_solver_creation() {
        let solver = KingRookVsKingSolver::new();
        assert_eq!(solver.name(), "KingRookVsKing");
        assert_eq!(solver.priority(), 80); // Default priority for rook solver
    }

    #[test]
    fn test_solver_with_config() {
        let config = KingRookConfig {
            enabled: true,
            priority: 7,
            max_moves_to_mate: 25,
            use_pattern_matching: true,
            pattern_cache_size: 1500,
        };
        let solver = KingRookVsKingSolver::with_config(config);
        assert_eq!(solver.priority(), 7);
    }

    #[test]
    fn test_piece_extraction() {
        let solver = KingRookVsKingSolver::new();
        let board = BitboardBoard::empty();
        let pieces = solver.extract_pieces(&board, Player::Black);

        // Empty board should have no pieces
        assert_eq!(pieces.len(), 0);
    }

    #[test]
    fn test_move_generation() {
        let solver = KingRookVsKingSolver::new();
        let board = BitboardBoard::empty();
        let captured_pieces = CapturedPieces::new();
        let moves = solver.generate_moves(&board, Player::Black, &captured_pieces);

        // Empty board should have no moves
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_rook_move_cannot_leave_king_in_check() {
        let solver = KingRookVsKingSolver::new();
        let mut board = BitboardBoard::empty();
        let captured = CapturedPieces::new();

        board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(4, 4),
        );
        board.place_piece(
            Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 5),
        );
        board.place_piece(
            Piece::new(PieceType::Rook, Player::White),
            Position::new(4, 8),
        );
        board.place_piece(
            Piece::new(PieceType::King, Player::White),
            Position::new(0, 0),
        );

        let moves = solver.generate_moves(&board, Player::Black, &captured);
        assert!(
            !moves
                .iter()
                .any(|m| m.from == Some(Position::new(4, 5)) && m.to == Position::new(3, 5)),
            "Rook move that exposes the king should be rejected"
        );
    }

    #[test]
    fn test_rook_moves_require_empty_captured_pieces() {
        let solver = KingRookVsKingSolver::new();
        let mut board = BitboardBoard::empty();
        let mut captured = CapturedPieces::new();
        captured.add_piece(PieceType::Pawn, Player::Black);

        board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(4, 4),
        );
        board.place_piece(
            Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 5),
        );
        board.place_piece(
            Piece::new(PieceType::King, Player::White),
            Position::new(0, 0),
        );

        let moves = solver.generate_moves(&board, Player::Black, &captured);
        assert!(
            moves.is_empty(),
            "Solver should not generate moves when captured pieces exist"
        );
    }

    #[test]
    fn test_rook_solver_detects_checkmate_position() {
        let solver = KingRookVsKingSolver::new();
        let (board, captured) = rook_checkmate_position();

        let result = solver
            .solve(&board, Player::Black, &captured)
            .expect("King+Rook vs King should be solvable");

        assert!(result.is_winning());
        assert_eq!(result.moves_to_mate, Some(0));
    }

    #[test]
    fn test_rook_solver_detects_stalemate_position() {
        let solver = KingRookVsKingSolver::new();
        let (board, captured) = rook_stalemate_position();

        assert!(solver.is_stalemate(&board, Player::White, &captured));
    }

    #[test]
    fn test_rook_solver_distance_to_mate_for_far_position() {
        let solver = KingRookVsKingSolver::new();
        let (board, captured) = rook_distance_position();

        let distance = solver.calculate_distance_to_mate(&board, Player::Black, &captured);
        assert!(distance > 0);
    }

    #[test]
    fn test_rook_evaluation_helpers() {
        let solver = KingRookVsKingSolver::new();
        let king_pos = Position::new(3, 4);
        let rook_pos = Position::new(0, 2);
        let board = build_board(&[
            (Player::Black, PieceType::King, king_pos),
            (Player::Black, PieceType::Rook, rook_pos),
            (Player::White, PieceType::King, Position::new(0, 4)),
        ]);
        let coord_move = Move::new_move(
            rook_pos,
            Position::new(0, 4),
            PieceType::Rook,
            Player::Black,
            false,
        );

        assert!(solver.coordinates_king_rook(
            &board,
            Player::Black,
            &coord_move,
            king_pos,
            rook_pos,
        ));

        let mobility_board = build_board(&[
            (Player::Black, PieceType::King, Position::new(2, 2)),
            (Player::Black, PieceType::Rook, Position::new(0, 2)),
            (Player::White, PieceType::King, Position::new(0, 0)),
        ]);
        let mobility_move = Move::new_move(
            Position::new(0, 2),
            Position::new(0, 1),
            PieceType::Rook,
            Player::Black,
            false,
        );
        let captured = CapturedPieces::new();

        assert!(solver.restricts_king_mobility(
            &mobility_board,
            Player::Black,
            &mobility_move,
            Position::new(0, 0),
            &captured,
        ));

        assert!(solver.controls_key_squares(
            &mobility_board,
            Player::Black,
            &mobility_move,
            Position::new(0, 0),
        ));
    }

    #[test]
    fn test_rook_solver_matches_endgame_theory() {
        let solver = KingRookVsKingSolver::new();
        let (board, captured) = rook_distance_position();

        let result = solver
            .solve(&board, Player::Black, &captured)
            .expect("Position should be solvable");
        assert!(result.is_winning());
    }
}
