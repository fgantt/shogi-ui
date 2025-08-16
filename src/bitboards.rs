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

    pub fn is_king_in_check(&self, player: Player) -> bool {
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

    pub fn is_square_attacked_by(&self, position: Position, player: Player) -> bool {
        // This is a complex method, a full implementation is required for correctness.
        // For now, we assume a simplified logic.
        false
    }

    pub fn is_legal_move(&self, move_: &Move) -> bool {
        let mut temp_board = self.clone();
        temp_board.make_move(move_);
        !temp_board.is_king_in_check(move_.player)
    }
    
    pub fn is_checkmate(&self, player: Player) -> bool {
        self.is_king_in_check(player) && !self.has_legal_moves(player)
    }

    pub fn is_stalemate(&self, player: Player) -> bool {
        !self.is_king_in_check(player) && !self.has_legal_moves(player)
    }

    fn has_legal_moves(&self, player: Player) -> bool {
        // Simplified: In a real scenario, you would generate all moves and check for legality.
        true
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
