use crate::types::*;
use std::collections::HashMap;

/// Bitboard-based board representation for efficient Shogi operations
pub struct BitboardBoard {
    pieces: [[Bitboard; 14]; 2],
    occupied: Bitboard,
    black_occupied: Bitboard,
    white_occupied: Bitboard,
    piece_positions: HashMap<Position, Piece>,
    attack_patterns: AttackPatterns,
}

impl BitboardBoard {
    pub fn new() -> Self {
        let mut board = Self::empty();
        board.setup_initial_position();
        board
    }

    pub fn empty() -> Self {
        Self {
            pieces: [[EMPTY_BITBOARD; 14]; 2],
            occupied: EMPTY_BITBOARD,
            black_occupied: EMPTY_BITBOARD,
            white_occupied: EMPTY_BITBOARD,
            piece_positions: HashMap::new(),
            attack_patterns: AttackPatterns::new(),
        }
    }

    fn setup_initial_position(&mut self) {
        // This function is assumed to correctly set up the board
    }

    pub fn place_piece(&mut self, piece: Piece, position: Position) {
        let player_idx = if piece.player == Player::Black { 0 } else { 1 };
        let piece_idx = piece.piece_type.to_u8() as usize;
        set_bit(&mut self.pieces[player_idx][piece_idx], position);
        match piece.player {
            Player::Black => set_bit(&mut self.black_occupied, position),
            Player::White => set_bit(&mut self.white_occupied, position),
        }
        set_bit(&mut self.occupied, position);
        self.piece_positions.insert(position, piece.clone());
    }

    pub fn remove_piece(&mut self, position: Position) -> Option<Piece> {
        if let Some(piece) = self.piece_positions.remove(&position) {
            let player_idx = if piece.player == Player::Black { 0 } else { 1 };
            let piece_idx = piece.piece_type.to_u8() as usize;
            clear_bit(&mut self.pieces[player_idx][piece_idx], position);
            match piece.player {
                Player::Black => clear_bit(&mut self.black_occupied, position),
                Player::White => clear_bit(&mut self.white_occupied, position),
            }
            clear_bit(&mut self.occupied, position);
            Some(piece)
        } else {
            None
        }
    }

    pub fn get_piece(&self, position: Position) -> Option<&Piece> {
        self.piece_positions.get(&position)
    }

    pub fn get_pieces(&self) -> &[[Bitboard; 14]; 2] {
        &self.pieces
    }

    pub fn is_square_occupied(&self, position: Position) -> bool {
        is_bit_set(self.occupied, position)
    }

    pub fn is_square_occupied_by(&self, position: Position, player: Player) -> bool {
        let occupied = if player == Player::Black { self.black_occupied } else { self.white_occupied };
        is_bit_set(occupied, position)
    }

    pub fn make_move(&mut self, move_: &Move) -> Option<Piece> {
        let mut captured_piece = None;
        if let Some(from) = move_.from {
            if let Some(piece_to_move) = self.get_piece(from).cloned() {
                self.remove_piece(from);
                if move_.is_capture {
                    if let Some(cp) = self.remove_piece(move_.to) {
                        captured_piece = Some(cp.unpromoted());
                    }
                }
                let final_piece_type = if move_.is_promotion {
                    piece_to_move.piece_type.promoted_version().unwrap_or(piece_to_move.piece_type)
                } else {
                    piece_to_move.piece_type
                };
                self.place_piece(Piece::new(final_piece_type, piece_to_move.player), move_.to);
            }
        } else {
            self.place_piece(Piece::new(move_.piece_type, move_.player), move_.to);
        }
        captured_piece
    }

    pub fn is_king_in_check(&self, player: Player, _captured_pieces: &CapturedPieces) -> bool {
        if let Some(king_pos) = self.find_king_position(player) {
            return self.is_square_attacked_by(king_pos, player.opposite());
        }
        false
    }

    fn find_king_position(&self, player: Player) -> Option<Position> {
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let king_bb = self.pieces[player_idx][PieceType::King.to_u8() as usize];
        if king_bb == 0 { None } else { get_lsb(king_bb) }
    }

    pub fn is_square_attacked_by(&self, target_pos: Position, attacking_player: Player) -> bool {
        for r in 0..9 {
            for c in 0..9 {
                let from_pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(from_pos) {
                    if piece.player == attacking_player {
                        // Check if this piece attacks the target_pos
                        let pseudo_moves = self.generate_pseudo_moves_for_piece(piece, from_pos);
                        for m in pseudo_moves {
                            if m.to == target_pos {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // Helper to generate pseudo-legal moves for a single piece
    fn generate_pseudo_moves_for_piece(&self, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = piece.player;

        let handle_move = |moves: &mut Vec<Move>, to_pos: Position| {
            if !self.is_square_occupied_by(to_pos, player) {
                moves.push(Move::new_move(pos, to_pos, piece.piece_type, player, false));
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
                    if new_row >= 0 && new_col >= 0 && new_row < 9 && new_col < 9 {
                        handle_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            },
            PieceType::Lance | PieceType::Rook | PieceType::Bishop | PieceType::PromotedBishop | PieceType::PromotedRook => {
                let directions = match piece.piece_type {
                    PieceType::Lance => if player == Player::Black { vec![(1, 0)] } else { vec![(-1, 0)] },
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    PieceType::PromotedBishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1), (1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::PromotedRook => vec![(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => vec![]
                };

                for (dr, dc) in directions {
                    let mut current_pos = pos;
                    loop {
                        let new_row = current_pos.row as i8 + dr;
                        let new_col = current_pos.col as i8 + dc;
                        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 { break; } // Out of bounds
                        
                        current_pos = Position::new(new_row as u8, new_col as u8);
                        handle_move(&mut moves, current_pos);

                        if self.is_square_occupied(current_pos) { break; } // Blocked by a piece
                    }
                }
            },
            PieceType::Silver | PieceType::Gold | PieceType::King | PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight | PieceType::PromotedSilver => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let offsets = piece.piece_type.get_move_offsets(dir);
                for (dr, dc) in offsets {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_col >= 0 && new_row < 9 && new_col < 9 {
                        handle_move(&mut moves, Position::new(new_row as u8, new_col as u8));
                    }
                }
            }
        }
        moves
    }

    pub fn is_legal_move(&self, move_: &Move, captured_pieces: &CapturedPieces) -> bool {
        let mut temp_board = self.clone();
        let mut temp_captured = captured_pieces.clone();
        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, move_.player);
        }
        !temp_board.is_king_in_check(move_.player, &temp_captured)
    }
    
    pub fn is_checkmate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        self.is_king_in_check(player, captured_pieces) && !self.has_legal_moves(player, captured_pieces)
    }

    pub fn is_stalemate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        !self.is_king_in_check(player, captured_pieces) && !self.has_legal_moves(player, captured_pieces)
    }

    fn has_legal_moves(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        let move_generator = crate::moves::MoveGenerator::new();
        !move_generator.generate_legal_moves(self, player, captured_pieces).is_empty()
    }

    pub fn to_fen(&self, player: Player, captured_pieces: &CapturedPieces) -> String {
        let mut fen = String::with_capacity(128);
        for r in 0..9 {
            let mut empty_squares = 0;
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(pos) {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    fen.push_str(&piece.to_fen_char());
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if r < 8 { fen.push('/'); }
        }
        fen.push(' ');
        fen.push(if player == Player::Black { 'b' } else { 'w' });
        fen.push(' ');
        let mut captured_str = String::new();
        for p in &captured_pieces.black { captured_str.push_str(&Piece::new(*p, Player::Black).to_fen_char()); }
        for p in &captured_pieces.white { captured_str.push_str(&Piece::new(*p, Player::White).to_fen_char()); }
        if captured_str.is_empty() { fen.push('-'); } else { fen.push_str(&captured_str); }
        fen
    }
}

impl Clone for BitboardBoard {
    fn clone(&self) -> Self {
        Self {
            pieces: self.pieces,
            occupied: self.occupied,
            black_occupied: self.black_occupied,
            white_occupied: self.white_occupied,
            piece_positions: self.piece_positions.clone(),
            attack_patterns: self.attack_patterns.clone(),
        }
    }
}

#[derive(Clone)]
struct AttackPatterns {
    // Simplified for brevity
}

impl AttackPatterns {
    fn new() -> Self { Self {} }
}
