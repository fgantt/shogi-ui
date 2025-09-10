use crate::types::*;
use crate::bitboards::*;
use std::collections::HashSet;


pub struct MoveGenerator {
    // In a more advanced engine, this could hold precomputed attack tables.
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_legal_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let is_in_check = board.is_king_in_check(player, captured_pieces);

        let pseudo_legal_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);

        let legal_moves: Vec<Move> = pseudo_legal_moves.into_iter().filter(|m| {
            let mut temp_board = board.clone();
            let mut temp_captured = captured_pieces.clone();
            
            if let Some(captured) = temp_board.make_move(m) {
                temp_captured.add_piece(captured.piece_type, player);
            }

            !temp_board.is_king_in_check(player, &temp_captured)
        }).collect();

        if is_in_check {
            // If in check, only moves that resolve the check are legal.
            // The filtering above already handles this.
            // If no moves are found, it's checkmate.
        }
        
        legal_moves
    }

    pub fn generate_legal_captures(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let pseudo_legal_moves = self.generate_pseudo_legal_captures(board, player, captured_pieces);
        
        // Filter out moves that leave the king in check
        pseudo_legal_moves.into_iter().filter(|m| {
            let mut temp_board = board.clone();
            let mut temp_captured = captured_pieces.clone();
            if let Some(captured) = temp_board.make_move(m) {
                temp_captured.add_piece(captured.piece_type, m.player);
            }
            !temp_board.is_king_in_check(player, &temp_captured)
        }).collect()
    }

    fn generate_pseudo_legal_captures(&self, board: &BitboardBoard, player: Player, _captured_pieces: &CapturedPieces) -> Vec<Move> {
        self.generate_capture_piece_moves(board, player)
    }

    fn generate_capture_piece_moves(&self, board: &BitboardBoard, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        moves.extend(self.generate_capture_moves_for_piece(board, piece, pos));
                    }
                }
            }
        }
        moves
    }

    fn generate_capture_moves_for_piece(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = piece.player;

        let handle_capture_move = |moves: &mut Vec<Move>, to_pos: Position| {
            if !board.is_square_occupied_by(to_pos, player) {
                if board.is_square_occupied(to_pos) { // Is a capture
                    let from_in_promo = pos.is_in_promotion_zone(player);
                    let to_in_promo = to_pos.is_in_promotion_zone(player);

                    // Non-promoted move
                    let mut move_ = Move::new_move(pos, to_pos, piece.piece_type, player, false);
                    move_.is_capture = true;
                    move_.captured_piece = board.get_piece(to_pos).cloned();
                    moves.push(move_);

                    // Promoted move
                    if piece.piece_type.can_promote() && (from_in_promo || to_in_promo) {
                        let mut promoted_move = Move::new_move(pos, to_pos, piece.piece_type, player, true);
                        promoted_move.is_capture = true;
                        promoted_move.captured_piece = board.get_piece(to_pos).cloned();
                        moves.push(promoted_move);
                    }
                }
            }
        };

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let new_row = pos.row as i8 + dir;
                if new_row >= 0 && new_row < 9 {
                    handle_capture_move(&mut moves, Position::new(new_row as u8, pos.col));
                }
            },
            PieceType::Knight => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let move_offsets = [(2 * dir, 1), (2 * dir, -1)];
                for (dr, dc) in move_offsets.iter() {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_capture_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            },
            PieceType::Lance | PieceType::Rook | PieceType::Bishop => {
                let directions = match piece.piece_type {
                    PieceType::Lance => if player == Player::Black { vec![(1, 0)] } else { vec![(-1, 0)] },
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => vec![]
                };

                for (dr, dc) in directions {
                    let mut current_pos = pos;
                    loop {
                        let new_row = current_pos.row as i8 + dr;
                        let new_col = current_pos.col as i8 + dc;
                        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 { break; }
                        
                        current_pos = Position::new(new_row as u8, new_col as u8);
                        handle_capture_move(&mut moves, current_pos);

                        if board.is_square_occupied(current_pos) { break; }
                    }
                }
            },
            PieceType::Silver | PieceType::Gold | PieceType::King | PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight | PieceType::PromotedSilver | PieceType::PromotedBishop | PieceType::PromotedRook => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let offsets = piece.piece_type.get_move_offsets(dir);
                for (dr, dc) in offsets {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_capture_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            }
        }
        moves
    }

    fn generate_pseudo_legal_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let mut moves = Vec::new();
        moves.extend(self.generate_piece_moves(board, player));
        moves.extend(self.generate_drop_moves(board, player, captured_pieces));
        moves
    }

    fn generate_piece_moves(&self, board: &BitboardBoard, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        moves.extend(self.generate_moves_for_piece(board, piece, pos));
                    }
                }
            }
        }
        moves
    }

    fn generate_moves_for_piece(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = piece.player;

        let handle_move = |moves: &mut Vec<Move>, to_pos: Position| {
            if !board.is_square_occupied_by(to_pos, player) {
                let is_capture = board.is_square_occupied(to_pos);
                let from_in_promo = pos.is_in_promotion_zone(player);
                let to_in_promo = to_pos.is_in_promotion_zone(player);

                // Non-promoted move
                let mut move_ = Move::new_move(pos, to_pos, piece.piece_type, player, false);
                if is_capture { 
                    move_.is_capture = true;
                    move_.captured_piece = board.get_piece(to_pos).cloned();
                }
                moves.push(move_);

                // Promoted move
                if piece.piece_type.can_promote() && (from_in_promo || to_in_promo) {
                    let mut promoted_move = Move::new_move(pos, to_pos, piece.piece_type, player, true);
                    if is_capture { 
                        promoted_move.is_capture = true;
                        promoted_move.captured_piece = board.get_piece(to_pos).cloned();
                    }
                    moves.push(promoted_move);
                }
            }
        };

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let new_row = pos.row as i8 + dir;
                if new_row >= 0 && new_row < 9 {
                    handle_move(&mut moves, Position::new(new_row as u8, pos.col));
                }
            },
            PieceType::Knight => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let move_offsets = [(2 * dir, 1), (2 * dir, -1)];
                for (dr, dc) in move_offsets.iter() {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            },
            PieceType::Lance | PieceType::Rook | PieceType::Bishop => {
                let directions = match piece.piece_type {
                    PieceType::Lance => if player == Player::Black { vec![(1, 0)] } else { vec![(-1, 0)] },
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => vec![]
                };

                for (dr, dc) in directions {
                    let mut current_pos = pos;
                    loop {
                        let new_row = current_pos.row as i8 + dr;
                        let new_col = current_pos.col as i8 + dc;
                        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 { break; }
                        
                        current_pos = Position::new(new_row as u8, new_col as u8);
                        handle_move(&mut moves, current_pos);

                        if board.is_square_occupied(current_pos) { break; }
                    }
                }
            },
            PieceType::Silver | PieceType::Gold | PieceType::King | PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight | PieceType::PromotedSilver | PieceType::PromotedBishop | PieceType::PromotedRook => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let offsets = piece.piece_type.get_move_offsets(dir);
                for (dr, dc) in offsets {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            }
        }
        moves
    }

    fn generate_drop_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut processed_pieces = HashSet::new();
        let captured = if player == Player::Black { &captured_pieces.black } else { &captured_pieces.white };

        for &piece_type in captured {
            if !processed_pieces.insert(piece_type) { continue; }

            for r in 0..9 {
                for c in 0..9 {
                    let pos = Position::new(r, c);
                    if !board.is_square_occupied(pos) {
                        // Basic legality check for drops (e.g., pawn drops)
                        if is_legal_drop_location(board, piece_type, pos, player) {
                            moves.push(Move::new_drop(piece_type, pos, player));
                        }
                    }
                }
            }
        }
        moves
    }
}

fn is_legal_drop_location(board: &BitboardBoard, piece_type: PieceType, pos: Position, player: Player) -> bool {
    if piece_type == PieceType::Pawn {
        // Cannot drop on a file that already contains an unpromoted pawn of the same color
        for r in 0..9 {
            if let Some(p) = board.get_piece(Position::new(r, pos.col)) {
                if p.piece_type == PieceType::Pawn && p.player == player {
                    return false;
                }
            }
        }
        // Cannot drop pawn to give immediate checkmate (this is a complex rule, simplified here)
    }

    // Cannot drop a piece where it has no legal moves
    let last_rank = if player == Player::Black { 8 } else { 0 };
    let second_last_rank = if player == Player::Black { 7 } else { 1 };
    match piece_type {
        PieceType::Pawn | PieceType::Lance if pos.row == last_rank => return false,
        PieceType::Knight if pos.row == last_rank || pos.row == second_last_rank => return false,
        _ => true
    }
}
