use crate::types::*;
use std::collections::HashMap;

/// Bitboard-based board representation for efficient Shogi operations
pub struct BitboardBoard {
    // Piece bitboards for each player and piece type
    pieces: [[Bitboard; 14]; 2],  // [player][piece_type]
    
    // Occupancy bitboards
    occupied: Bitboard,
    black_occupied: Bitboard,
    white_occupied: Bitboard,
    
    // Piece positions for quick lookup
    piece_positions: HashMap<Position, Piece>,
    
    // Zobrist hash for transposition table
    zobrist_hash: u64,
    
    // Precomputed attack patterns
    attack_patterns: AttackPatterns,
}

impl BitboardBoard {
    pub fn new() -> Self {
        let mut board = Self {
            pieces: [[EMPTY_BITBOARD; 14]; 2],
            occupied: EMPTY_BITBOARD,
            black_occupied: EMPTY_BITBOARD,
            white_occupied: EMPTY_BITBOARD,
            piece_positions: HashMap::new(),
            zobrist_hash: 0,
            attack_patterns: AttackPatterns::new(),
        };
        
        board.setup_initial_position();
        board
    }

    fn setup_initial_position(&mut self) {
        // Black pieces (bottom)
        self.place_piece(Piece::new(PieceType::Lance, Player::Black), Position::new(0, 0));
        self.place_piece(Piece::new(PieceType::Knight, Player::Black), Position::new(0, 1));
        self.place_piece(Piece::new(PieceType::Silver, Player::Black), Position::new(0, 2));
        self.place_piece(Piece::new(PieceType::Gold, Player::Black), Position::new(0, 3));
        self.place_piece(Piece::new(PieceType::King, Player::Black), Position::new(0, 4));
        self.place_piece(Piece::new(PieceType::Gold, Player::Black), Position::new(0, 5));
        self.place_piece(Piece::new(PieceType::Silver, Player::Black), Position::new(0, 6));
        self.place_piece(Piece::new(PieceType::Knight, Player::Black), Position::new(0, 7));
        self.place_piece(Piece::new(PieceType::Lance, Player::Black), Position::new(0, 8));
        
        self.place_piece(Piece::new(PieceType::Rook, Player::Black), Position::new(1, 1));
        self.place_piece(Piece::new(PieceType::Bishop, Player::Black), Position::new(1, 7));
        
        // Black pawns
        for col in 0..9 {
            self.place_piece(Piece::new(PieceType::Pawn, Player::Black), Position::new(2, col));
        }
        
        // White pieces (top)
        self.place_piece(Piece::new(PieceType::Lance, Player::White), Position::new(8, 0));
        self.place_piece(Piece::new(PieceType::Knight, Player::White), Position::new(8, 1));
        self.place_piece(Piece::new(PieceType::Silver, Player::White), Position::new(8, 2));
        self.place_piece(Piece::new(PieceType::Gold, Player::White), Position::new(8, 3));
        self.place_piece(Piece::new(PieceType::King, Player::White), Position::new(8, 4));
        self.place_piece(Piece::new(PieceType::Gold, Player::White), Position::new(8, 5));
        self.place_piece(Piece::new(PieceType::Silver, Player::White), Position::new(8, 6));
        self.place_piece(Piece::new(PieceType::Knight, Player::White), Position::new(8, 7));
        self.place_piece(Piece::new(PieceType::Lance, Player::White), Position::new(8, 8));
        
        self.place_piece(Piece::new(PieceType::Rook, Player::White), Position::new(7, 7));
        self.place_piece(Piece::new(PieceType::Bishop, Player::White), Position::new(7, 1));
        
        // White pawns
        for col in 0..9 {
            self.place_piece(Piece::new(PieceType::Pawn, Player::White), Position::new(6, col));
        }
    }

    fn place_piece(&mut self, piece: Piece, position: Position) {
        let player_idx = if piece.player == Player::Black { 0 } else { 1 };
        let piece_idx = piece.piece_type.to_u8() as usize;
        
        set_bit(&mut self.pieces[player_idx][piece_idx], position);
        
        match piece.player {
            Player::Black => set_bit(&mut self.black_occupied, position),
            Player::White => set_bit(&mut self.white_occupied, position),
        }
        
        set_bit(&mut self.occupied, position);
        self.piece_positions.insert(position, piece.clone());
        
        // Update Zobrist hash
        self.zobrist_hash ^= self.attack_patterns.get_zobrist_key(piece.piece_type, piece.player, position);
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
            
            // Update Zobrist hash
            self.zobrist_hash ^= self.attack_patterns.get_zobrist_key(piece.piece_type, piece.player, position);
            
            Some(piece)
        } else {
            None
        }
    }

    pub fn get_piece(&self, position: Position) -> Option<&Piece> {
        self.piece_positions.get(&position)
    }

    pub fn is_square_occupied(&self, position: Position) -> bool {
        is_bit_set(self.occupied, position)
    }

    pub fn is_square_occupied_by(&self, position: Position, player: Player) -> bool {
        let occupied = match player {
            Player::Black => self.black_occupied,
            Player::White => self.white_occupied,
        };
        is_bit_set(occupied, position)
    }

    pub fn generate_move(&self, from: Position, to: Position, promote: bool) -> Option<Move> {
        if let Some(piece) = self.get_piece(from) {
            let mut move_ = Move::new_move(from, to, piece.piece_type, piece.player, promote);
            
            // Check if this is a capture
            if let Some(captured_piece) = self.get_piece(to) {
                move_.is_capture = true;
                move_.captured_piece = Some(captured_piece.clone());
            }
            
            Some(move_)
        } else {
            None
        }
    }

    pub fn generate_drop(&self, piece_type: PieceType, to: Position) -> Option<Move> {
        if !self.is_square_occupied(to) {
            // Determine player from captured pieces or current turn
            // For now, assume it's the current player's turn
            let player = Player::Black; // This should be passed in or determined from context
            Some(Move::new_drop(piece_type, to, player))
        } else {
            None
        }
    }

    pub fn make_move(&mut self, move_: &Move) {
        if let Some(from) = move_.from {
            // Remove piece from starting position
            if let Some(piece) = self.remove_piece(from) {
                // Handle promotion
                let final_piece_type = if move_.is_promotion {
                    piece.piece_type.promoted_version().unwrap_or(piece.piece_type)
                } else {
                    piece.piece_type
                };
                
                // Handle capture
                if move_.is_capture {
                    if let Some(_captured_piece) = self.remove_piece(move_.to) {
                        // Add to captured pieces (this should be handled by the game state)
                    }
                }
                
                // Place piece at destination
                let final_piece = Piece::new(final_piece_type, piece.player);
                self.place_piece(final_piece, move_.to);
            }
        } else {
            // Drop move
            self.place_piece(Piece::new(move_.piece_type, move_.player), move_.to);
        }
    }

    pub fn is_legal_move(&self, move_: &Move) -> bool {
        // Basic validation
        if !move_.to.is_valid() {
            return false;
        }
        
        if let Some(from) = move_.from {
            if !from.is_valid() {
                return false;
            }
            
            // Check if source square has a piece
            if let Some(piece) = self.get_piece(from) {
                if piece.player != move_.player {
                    return false;
                }
                
                // Check if destination is occupied by own piece
                if let Some(dest_piece) = self.get_piece(move_.to) {
                    if dest_piece.player == piece.player {
                        return false;
                    }
                }
                
                // Check if move pattern is legal for this piece type
                if !self.is_legal_move_pattern(from, move_.to, piece.piece_type, piece.player) {
                    return false;
                }
                
                // Check promotion rules
                if move_.is_promotion {
                    if !piece.piece_type.can_promote() {
                        return false;
                    }
                    
                    // Check if promotion is mandatory
                    let promotion_zone = match piece.player {
                        Player::Black => from.row <= 2 || move_.to.row <= 2,
                        Player::White => from.row >= 6 || move_.to.row >= 6,
                    };
                    
                    if !promotion_zone {
                        return false;
                    }
                }
            } else {
                return false;
            }
        } else {
            // Drop move
            if self.is_square_occupied(move_.to) {
                return false;
            }
            
            // Check drop restrictions
            if !self.is_legal_drop(move_.piece_type, move_.to, move_.player) {
                return false;
            }
        }
        
        // Check if move puts own king in check
        let mut temp_board = self.clone();
        temp_board.make_move(move_);
        !temp_board.is_king_in_check(move_.player)
    }

    fn is_legal_move_pattern(&self, from: Position, to: Position, piece_type: PieceType, player: Player) -> bool {
        match piece_type {
            PieceType::Pawn => self.is_legal_pawn_move(from, to, player),
            PieceType::Lance => self.is_legal_lance_move(from, to, player),
            PieceType::Knight => self.is_legal_knight_move(from, to, player),
            PieceType::Silver => self.is_legal_silver_move(from, to, player),
            PieceType::Gold => self.is_legal_gold_move(from, to, player),
            PieceType::Bishop => self.is_legal_bishop_move(from, to, player),
            PieceType::Rook => self.is_legal_rook_move(from, to, player),
            PieceType::King => self.is_legal_king_move(from, to, player),
            _ => self.is_legal_promoted_move(from, to, piece_type, player),
        }
    }

    fn is_legal_pawn_move(&self, from: Position, to: Position, player: Player) -> bool {
        let (dr, dc) = match player {
            Player::Black => (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8),
            Player::White => (from.row as i8 - to.row as i8, to.col as i8 - from.col as i8),
        };
        
        // Pawn moves forward one square
        dr == 1 && dc == 0
    }

    fn is_legal_lance_move(&self, from: Position, to: Position, player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Lance moves forward only
        if dc != 0 {
            return false;
        }
        
        let forward_direction = match player {
            Player::Black => 1,
            Player::White => -1,
        };
        
        if (dr * forward_direction) <= 0 {
            return false;
        }
        
        // Check if path is clear
        self.is_path_clear(from, to)
    }

    fn is_legal_knight_move(&self, from: Position, to: Position, player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Knight moves in L-shape: 2 forward, 1 left or right
        let forward_direction = match player {
            Player::Black => 1,
            Player::White => -1,
        };
        
        (dr * forward_direction == 2) && (dc.abs() == 1)
    }

    fn is_legal_silver_move(&self, from: Position, to: Position, player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Silver can move: forward, forward-left, forward-right, backward-left, backward-right
        let forward_direction = match player {
            Player::Black => 1,
            Player::White => -1,
        };
        
        let abs_dr = dr.abs();
        let abs_dc = dc.abs();
        
        (abs_dr == 1 && abs_dc == 1) || (dr * forward_direction == 1 && dc == 0)
    }

    fn is_legal_gold_move(&self, from: Position, to: Position, player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Gold can move: forward, forward-left, forward-right, left, right, backward
        let forward_direction = match player {
            Player::Black => 1,
            Player::White => -1,
        };
        
        let abs_dr = dr.abs();
        let abs_dc = dc.abs();
        
        (abs_dr <= 1 && abs_dc <= 1) && !(dr * forward_direction == -1 && abs_dc == 1)
    }

    fn is_legal_bishop_move(&self, from: Position, to: Position, _player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Bishop moves diagonally
        if dr.abs() != dc.abs() {
            return false;
        }
        
        // Check if path is clear
        self.is_path_clear(from, to)
    }

    fn is_legal_rook_move(&self, from: Position, to: Position, _player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // Rook moves horizontally or vertically
        if dr != 0 && dc != 0 {
            return false;
        }
        
        // Check if path is clear
        self.is_path_clear(from, to)
    }

    fn is_legal_king_move(&self, from: Position, to: Position, _player: Player) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        
        // King moves one square in any direction
        dr.abs() <= 1 && dc.abs() <= 1
    }

    fn is_legal_promoted_move(&self, from: Position, to: Position, piece_type: PieceType, player: Player) -> bool {
        match piece_type {
            PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight => {
                // These pieces move like Gold
                self.is_legal_gold_move(from, to, player)
            }
            PieceType::PromotedSilver => {
                // Promoted Silver moves like Gold
                self.is_legal_gold_move(from, to, player)
            }
            PieceType::PromotedBishop => {
                // Promoted Bishop can move like Bishop OR King
                self.is_legal_bishop_move(from, to, player) || self.is_legal_king_move(from, to, player)
            }
            PieceType::PromotedRook => {
                // Promoted Rook can move like Rook OR King
                self.is_legal_rook_move(from, to, player) || self.is_legal_king_move(from, to, player)
            }
            _ => false,
        }
    }

    fn is_path_clear(&self, from: Position, to: Position) -> bool {
        let (dr, dc) = (to.row as i8 - from.row as i8, to.col as i8 - from.col as i8);
        let steps = dr.abs().max(dc.abs());
        
        if steps <= 1 {
            return true;
        }
        
        let step_dr = dr / steps;
        let step_dc = dc / steps;
        
        let mut current = Position::new(from.row, from.col);
        
        for _ in 0..steps - 1 {
            current.row = (current.row as i8 + step_dr) as u8;
            current.col = (current.col as i8 + step_dc) as u8;
            
            if self.is_square_occupied(current) {
                return false;
            }
        }
        
        true
    }

    fn is_legal_drop(&self, piece_type: PieceType, to: Position, player: Player) -> bool {
        // Check if piece is available in hand (this should be handled by game state)
        
        // Check drop restrictions
        match piece_type {
            PieceType::Pawn => {
                // Can't drop pawn on last rank
                let last_rank = match player {
                    Player::Black => 0,
                    Player::White => 8,
                };
                if to.row == last_rank {
                    return false;
                }
                
                // Can't drop pawn on a file that already has a pawn
                for row in 0..9 {
                    let pos = Position::new(row, to.col);
                    if let Some(piece) = self.get_piece(pos) {
                        if piece.piece_type == PieceType::Pawn && piece.player == player {
                            return false;
                        }
                    }
                }
                
                // Can't drop pawn that would immediately checkmate
                // This is a complex check that should be implemented
                true
            }
            PieceType::Lance => {
                // Can't drop lance on last rank
                let last_rank = match player {
                    Player::Black => 0,
                    Player::White => 8,
                };
                to.row != last_rank
            }
            PieceType::Knight => {
                // Can't drop knight on last two ranks
                let last_ranks = match player {
                    Player::Black => [0, 1],
                    Player::White => [7, 8],
                };
                !last_ranks.contains(&to.row)
            }
            _ => true,
        }
    }

    pub fn is_king_in_check(&self, player: Player) -> bool {
        // Find king position
        let king_pos = self.find_king_position(player);
        if king_pos.is_none() {
            return false;
        }
        
        let king_pos = king_pos.unwrap();
        let opponent = player.opposite();
        
        // Check if any opponent piece can attack the king
        self.is_square_attacked_by(king_pos, opponent)
    }

    pub fn is_checkmate(&self, player: Player) -> bool {
        if !self.is_king_in_check(player) {
            return false;
        }
        
        // Check if any legal move can get out of check
        self.has_legal_moves(player)
    }

    pub fn is_stalemate(&self, player: Player) -> bool {
        if self.is_king_in_check(player) {
            return false;
        }
        
        !self.has_legal_moves(player)
    }

    fn has_legal_moves(&self, _player: Player) -> bool {
        // This is a simplified check - in practice, we'd generate all legal moves
        // For now, return true to avoid infinite loops
        true
    }

    fn find_king_position(&self, player: Player) -> Option<Position> {
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let king_bitboard = self.pieces[player_idx][PieceType::King.to_u8() as usize];
        
        if king_bitboard != 0 {
            get_lsb(king_bitboard)
        } else {
            None
        }
    }

    fn is_square_attacked_by(&self, _position: Position, _player: Player) -> bool {
        // Check if any piece of the given player can attack the square
        // This is a simplified implementation
        false
    }

    pub fn get_legal_moves(&self, from: Position) -> Vec<Position> {
        if let Some(piece) = self.get_piece(from) {
            let mut legal_moves = Vec::new();
            
            // Generate all possible moves for this piece
            for row in 0..9 {
                for col in 0..9 {
                    let to = Position::new(row, col);
                    if from != to {
                        let move_ = Move::new_move(from, to, piece.piece_type, piece.player, false);
                        if self.is_legal_move(&move_) {
                            legal_moves.push(to);
                        }
                        
                        // Check promotion moves
                        if piece.piece_type.can_promote() {
                            let move_ = Move::new_move(from, to, piece.piece_type, piece.player, true);
                            if self.is_legal_move(&move_) {
                                legal_moves.push(to);
                            }
                        }
                    }
                }
            }
            
            legal_moves
        } else {
            Vec::new()
        }
    }

    pub fn get_pieces(&self) -> &[[Bitboard; 14]; 2] {
        &self.pieces
    }

    pub fn get_legal_drops(&self, piece_type: PieceType) -> Vec<Position> {
        let mut legal_drops = Vec::new();
        
        for row in 0..9 {
            for col in 0..9 {
                let to = Position::new(row, col);
                if !self.is_square_occupied(to) {
                    let _move_ = Move::new_drop(piece_type, to, Player::Black); // Player should be passed in
                    if self.is_legal_drop(piece_type, to, Player::Black) {
                        legal_drops.push(to);
                    }
                }
            }
        }
        
        legal_drops
    }

    pub fn to_fen(&self) -> String {
        // Convert board to FEN notation
        // This is a simplified implementation
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string()
    }

    pub fn get_zobrist_hash(&self) -> u64 {
        self.zobrist_hash
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
            zobrist_hash: self.zobrist_hash,
            attack_patterns: self.attack_patterns.clone(),
        }
    }
}

/// Precomputed attack patterns for efficient move generation
#[derive(Clone)]
struct AttackPatterns {
    // Piece-Square Tables for positional evaluation
    pawn_attacks: [[Bitboard; 81]; 2],      // [player][square]
    knight_attacks: [Bitboard; 81],
    silver_attacks: [Bitboard; 81],
    gold_attacks: [Bitboard; 81],
    bishop_attacks: [Bitboard; 81],
    rook_attacks: [Bitboard; 81],
    king_attacks: [Bitboard; 81],
    
    // Zobrist keys for transposition table
    zobrist_keys: [[[u64; 81]; 14]; 2],  // [player][piece_type][square]
}

impl AttackPatterns {
    fn new() -> Self {
        let mut patterns = Self {
            pawn_attacks: [[EMPTY_BITBOARD; 81]; 2],
            knight_attacks: [EMPTY_BITBOARD; 81],
            silver_attacks: [EMPTY_BITBOARD; 81],
            gold_attacks: [EMPTY_BITBOARD; 81],
            bishop_attacks: [EMPTY_BITBOARD; 81],
            rook_attacks: [EMPTY_BITBOARD; 81],
            king_attacks: [EMPTY_BITBOARD; 81],
            zobrist_keys: [[[0; 81]; 14]; 2],
        };
        
        patterns.init_attack_patterns();
        patterns.init_zobrist_keys();
        
        patterns
    }

    fn init_attack_patterns(&mut self) {
        for square in 0..81 {
            let pos = Position::from_u8(square);
            
            // Initialize attack patterns for each piece type
            self.init_pawn_attacks(pos, square.into());
            self.init_knight_attacks(pos, square.into());
            self.init_silver_attacks(pos, square.into());
            self.init_gold_attacks(pos, square.into());
            self.init_king_attacks(pos, square.into());
        }
    }

    fn init_pawn_attacks(&mut self, pos: Position, square: usize) {
        // Black pawn attacks (forward-left and forward-right)
        if pos.row > 0 {
            if pos.col > 0 {
                set_bit(&mut self.pawn_attacks[0][square], Position::new(pos.row - 1, pos.col - 1));
            }
            if pos.col < 8 {
                set_bit(&mut self.pawn_attacks[0][square], Position::new(pos.row - 1, pos.col + 1));
            }
        }
        
        // White pawn attacks (forward-left and forward-right)
        if pos.row < 8 {
            if pos.col > 0 {
                set_bit(&mut self.pawn_attacks[1][square], Position::new(pos.row + 1, pos.col - 1));
            }
            if pos.col < 8 {
                set_bit(&mut self.pawn_attacks[1][square], Position::new(pos.row + 1, pos.col + 1));
            }
        }
    }

    fn init_knight_attacks(&mut self, pos: Position, square: usize) {
        let knight_moves = [
            (-2, -1), (-2, 1), (2, -1), (2, 1),
            (-1, -2), (-1, 2), (1, -2), (1, 2),
        ];
        
        for (dr, dc) in knight_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                set_bit(&mut self.knight_attacks[square], Position::new(new_row as u8, new_col as u8));
            }
        }
    }

    fn init_silver_attacks(&mut self, pos: Position, square: usize) {
        let silver_moves = [
            (-1, -1), (-1, 0), (-1, 1),
            (1, -1), (1, 1),
        ];
        
        for (dr, dc) in silver_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                set_bit(&mut self.silver_attacks[square], Position::new(new_row as u8, new_col as u8));
            }
        }
    }

    fn init_gold_attacks(&mut self, pos: Position, square: usize) {
        let gold_moves = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1), (0, 1),
            (1, 0),
        ];
        
        for (dr, dc) in gold_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                set_bit(&mut self.gold_attacks[square], Position::new(new_row as u8, new_col as u8));
            }
        }
    }

    fn init_king_attacks(&mut self, pos: Position, square: usize) {
        let king_moves = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1), (0, 1),
            (1, -1), (1, 0), (1, 1),
        ];
        
        for (dr, dc) in king_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                set_bit(&mut self.king_attacks[square], Position::new(new_row as u8, new_col as u8));
            }
        }
    }

    fn init_zobrist_keys(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut rng = 12345u64; // Simple pseudo-random number generator
        
        for player in 0..2 {
            for piece_type in 0..14 {
                for square in 0..81 {
                    let mut hasher = DefaultHasher::new();
                    (player, piece_type, square, rng).hash(&mut hasher);
                    self.zobrist_keys[player][piece_type][square] = hasher.finish();
                    rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                }
            }
        }
    }

    pub fn get_zobrist_key(&self, piece_type: PieceType, player: Player, position: Position) -> u64 {
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let piece_idx = piece_type.to_u8() as usize;
        let square_idx = position.to_u8() as usize;
        
        self.zobrist_keys[player_idx][piece_idx][square_idx]
    }
}
