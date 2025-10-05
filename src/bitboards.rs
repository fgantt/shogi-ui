use crate::types::*;
use std::collections::HashMap;

// Include the magic bitboard module
pub mod magic;
pub mod sliding_moves;
pub mod attack_patterns;
pub mod platform_detection;
pub mod popcount;
pub mod bitscan;
pub mod debruijn;
pub mod lookup_tables;
pub mod masks;
pub mod integration;
pub mod bit_iterator;
pub mod bit_utils;
pub mod square_utils;
pub mod api;
pub mod cache_opt;
pub mod branch_opt;

// Re-export commonly used functions for convenience
pub use platform_detection::{get_platform_capabilities, get_best_popcount_impl, get_best_bitscan_impl};
pub use popcount::{popcount, popcount_optimized, is_single_bit, is_multiple_bits, is_empty};
pub use bitscan::{
    bit_scan_forward, bit_scan_reverse, 
    clear_lsb, clear_msb, isolate_lsb, isolate_msb,
    get_all_bit_positions, bit_scan_optimized
};
pub use lookup_tables::{
    popcount_4bit_lookup, bit_positions_4bit_lookup, popcount_4bit_optimized,
    popcount_4bit_small, bit_positions_4bit_small, validate_4bit_lookup_tables
};
pub use masks::{
    get_rank_mask, get_file_mask, get_diagonal_mask,
    get_rank_from_square, get_file_from_square, get_square_from_rank_file,
    get_rank_squares, get_file_squares, get_diagonal_squares,
    same_rank, same_file, same_diagonal, validate_masks
};
pub use integration::{
    BitScanningOptimizer, GlobalOptimizer, GeometricAnalysis
};
pub use bit_iterator::{
    BitIterator, ReverseBitIterator, BitIteratorExt, ReverseBitIteratorExt,
    bits, bits_from
};
pub use bit_utils::{
    bit_positions, extract_lsb, extract_msb, lsb_position, msb_position,
    rotate_left, rotate_right, reverse_bits, overlaps, is_subset,
    intersection, union, symmetric_difference, complement, difference
};
pub use square_utils::{
    bit_to_square, square_to_bit, bit_to_coords, coords_to_bit,
    bit_to_square_name, square_name_to_bit, is_valid_shogi_square,
    is_promotion_zone, square_distance, promotion_zone_mask,
    get_center_squares, is_center_square
};
pub use cache_opt::{
    CACHE_LINE_SIZE, CACHE_ALIGNED_SIZE,
    CacheAlignedPopcountTable, CacheAlignedBitPositionTable,
    CacheAlignedRankMasks, CacheAlignedFileMasks,
    prefetch_bitboard, prefetch_bitboard_sequence, process_bitboard_sequence,
    popcount_cache_optimized, get_bit_positions_cache_optimized
};
pub use branch_opt::{
    optimized::{
        bit_scan_forward_optimized, bit_scan_reverse_optimized, popcount_optimized as popcount_branch_optimized,
        overlaps_optimized, is_subset_optimized
    },
    common_cases::{
        is_single_piece_optimized, is_multiple_pieces_optimized, is_empty_optimized,
        is_not_empty_optimized, single_piece_position_optimized
    },
    critical_paths::{
        popcount_critical, bit_scan_forward_critical
    }
};

/// Bitboard-based board representation for efficient Shogi operations
pub struct BitboardBoard {
    pieces: [[Bitboard; 14]; 2],
    occupied: Bitboard,
    black_occupied: Bitboard,
    white_occupied: Bitboard,
    piece_positions: HashMap<Position, Piece>,
    attack_patterns: AttackPatterns,
    /// Precomputed attack tables for non-sliding pieces
    attack_tables: attack_patterns::AttackTables,
    /// Magic bitboard table for sliding piece moves
    magic_table: Option<crate::types::MagicTable>,
    /// Sliding move generator for magic bitboard operations
    sliding_generator: Option<sliding_moves::SlidingMoveGenerator>,
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
            attack_tables: attack_patterns::AttackTables::new(),
            magic_table: None,
            sliding_generator: None,
        }
    }

    fn setup_initial_position(&mut self) {
        let start_fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        if let Ok((board, _, _)) = BitboardBoard::from_fen(start_fen) {
            self.pieces = board.pieces;
            self.occupied = board.occupied;
            self.black_occupied = board.black_occupied;
            self.white_occupied = board.white_occupied;
            self.piece_positions = board.piece_positions;
        }
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

    pub fn from_fen(fen: &str) -> Result<(BitboardBoard, Player, CapturedPieces), &str> {
        let mut board = BitboardBoard::empty();
        let mut captured_pieces = CapturedPieces::new();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid FEN string: not enough parts");
        }

        // 1. Parse board state
        let board_part = parts[0];
        let ranks: Vec<&str> = board_part.split('/').collect();
        if ranks.len() != 9 {
            return Err("Invalid FEN: must have 9 ranks");
        }

        for (r, rank_str) in ranks.iter().enumerate() {
            let mut c = 0;
            let mut chars = rank_str.chars().peekable();
            while let Some(ch) = chars.next() {
                if c >= 9 { return Err("Invalid FEN: rank has more than 9 files"); }
                if let Some(digit) = ch.to_digit(10) {
                    c += digit as usize;
                } else {
                    let is_promoted = ch == '+' ;
                    let piece_char = if is_promoted {
                        if let Some(next_ch) = chars.next() { next_ch } else { return Err("Invalid FEN: '+' must be followed by a piece"); }
                    } else {
                        ch
                    };

                    let player = if piece_char.is_uppercase() { Player::Black } else { Player::White };
                    let piece_type_char = piece_char.to_ascii_lowercase();
                    
                    let piece_type = match piece_type_char {
                        'p' => if is_promoted { PieceType::PromotedPawn } else { PieceType::Pawn },
                        'l' => if is_promoted { PieceType::PromotedLance } else { PieceType::Lance },
                        'n' => if is_promoted { PieceType::PromotedKnight } else { PieceType::Knight },
                        's' => if is_promoted { PieceType::PromotedSilver } else { PieceType::Silver },
                        'g' => PieceType::Gold,
                        'b' => if is_promoted { PieceType::PromotedBishop } else { PieceType::Bishop },
                        'r' => if is_promoted { PieceType::PromotedRook } else { PieceType::Rook },
                        'k' => PieceType::King,
                        _ => return Err("Invalid FEN: unknown piece character"),
                    };
                    
                    board.place_piece(Piece::new(piece_type, player), Position::new(r as u8, c as u8));
                    c += 1;
                }
            }
        }

        // 2. Parse side to move
        let player = match parts[1] {
            "b" => Player::Black,
            "w" => Player::White,
            _ => return Err("Invalid FEN: invalid player"),
        };

        // 3. Parse pieces in hand
        if parts[2] != "-" {
            let mut count = 1;
            for ch in parts[2].chars() {
                if let Some(digit) = ch.to_digit(10) {
                    count = digit;
                } else {
                    let hand_player = if ch.is_uppercase() { Player::Black } else { Player::White };
                    let piece_type = match ch.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'l' => PieceType::Lance,
                        'n' => PieceType::Knight,
                        's' => PieceType::Silver,
                        'g' => PieceType::Gold,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        _ => return Err("Invalid FEN: unknown piece in hand"),
                    };
                    for _ in 0..count {
                        captured_pieces.add_piece(piece_type, hand_player);
                    }
                    count = 1;
                }
            }
        }

        Ok((board, player, captured_pieces))
    }

    pub fn to_string_for_debug(&self) -> String {
        let mut board_str = String::new();
        board_str.push_str("  9  8  7  6  5  4  3  2  1\n");
        board_str.push_str("+--+--+--+--+--+--+--+--+--+\n");
        for r in 0..9 {
            board_str.push('|');
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(pos) {
                    let mut piece_char = piece.to_fen_char();
                    if piece.player == Player::White {
                        piece_char = piece_char.to_lowercase();
                    }

                    if piece_char.starts_with('+') {
                        board_str.push_str(&piece_char);
                    } else {
                        board_str.push(' ');
                        board_str.push_str(&piece_char);
                    }
                } else {
                    board_str.push_str("  ");
                }
                board_str.push('|');
            }
            board_str.push_str(&format!(" {}\n", (b'a' + r) as char));
            board_str.push_str("+--+--+--+--+--+--+--+--+--+\n");
        }
        board_str
    }

    /// Initialize with magic bitboard support
    pub fn new_with_magic_support() -> Result<Self, MagicError> {
        let magic_table = crate::types::MagicTable::new()?;
        Ok(Self {
            pieces: [[EMPTY_BITBOARD; 14]; 2],
            occupied: EMPTY_BITBOARD,
            black_occupied: EMPTY_BITBOARD,
            white_occupied: EMPTY_BITBOARD,
            piece_positions: HashMap::new(),
            attack_patterns: AttackPatterns::new(),
            attack_tables: attack_patterns::AttackTables::new(),
            magic_table: Some(magic_table),
            sliding_generator: None,
        })
    }

    /// Get attack pattern for a piece at a given square using precomputed tables
    pub fn get_attack_pattern_precomputed(
        &self,
        square: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Bitboard {
        self.attack_tables.get_attack_pattern(square.to_u8(), piece_type, player)
    }

    /// Check if a square is attacked by a piece using precomputed tables
    pub fn is_square_attacked_precomputed(
        &self,
        from_square: Position,
        to_square: Position,
        piece_type: PieceType,
        player: Player,
    ) -> bool {
        self.attack_tables.is_square_attacked(from_square.to_u8(), to_square.to_u8(), piece_type, player)
    }

    /// Get attack table statistics and metadata
    pub fn get_attack_table_stats(&self) -> &attack_patterns::AttackTablesMetadata {
        self.attack_tables.memory_stats()
    }

    /// Get attack pattern for a square using magic bitboards
    pub fn get_attack_pattern(
        &self,
        square: Position,
        piece_type: PieceType
    ) -> Bitboard {
        if let Some(ref magic_table) = self.magic_table {
            magic_table.get_attacks(square.to_index(), piece_type, self.occupied)
        } else {
            // Fallback to ray-casting
            self.generate_attack_pattern_raycast(square, piece_type)
        }
    }

    /// Generate attack pattern using ray-casting (fallback method)
    fn generate_attack_pattern_raycast(&self, _square: Position, _piece_type: PieceType) -> Bitboard {
        // Placeholder implementation - would use the existing ray-casting logic
        EMPTY_BITBOARD
    }

    /// Check if magic bitboards are enabled
    pub fn has_magic_support(&self) -> bool {
        self.magic_table.is_some()
    }

    /// Get magic table reference
    pub fn get_magic_table(&self) -> Option<&crate::types::MagicTable> {
        self.magic_table.as_ref()
    }

    /// Initialize sliding move generator with magic table
    pub fn init_sliding_generator(&mut self) -> Result<(), crate::types::MagicError> {
        if let Some(magic_table) = self.magic_table.take() {
            self.sliding_generator = Some(sliding_moves::SlidingMoveGenerator::new(magic_table));
            Ok(())
        } else {
            Err(crate::types::MagicError::InitializationFailed {
                reason: "Magic table not initialized".to_string(),
            })
        }
    }

    /// Initialize sliding move generator with custom settings
    pub fn init_sliding_generator_with_settings(&mut self, magic_enabled: bool) -> Result<(), crate::types::MagicError> {
        if let Some(magic_table) = self.magic_table.take() {
            self.sliding_generator = Some(sliding_moves::SlidingMoveGenerator::with_settings(magic_table, magic_enabled));
            Ok(())
        } else {
            Err(crate::types::MagicError::InitializationFailed {
                reason: "Magic table not initialized".to_string(),
            })
        }
    }

    /// Get sliding move generator reference
    pub fn get_sliding_generator(&self) -> Option<&sliding_moves::SlidingMoveGenerator> {
        self.sliding_generator.as_ref()
    }

    /// Check if sliding generator is initialized
    pub fn is_sliding_generator_initialized(&self) -> bool {
        self.sliding_generator.is_some()
    }

    /// Generate sliding moves for a piece using magic bitboards
    /// Returns None if magic bitboards are not initialized
    pub fn generate_magic_sliding_moves(
        &self,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Option<Vec<Move>> {
        self.sliding_generator.as_ref().map(|gen| {
            gen.generate_sliding_moves(self, from, piece_type, player)
        })
    }

    /// Get occupied bitboard
    pub fn get_occupied_bitboard(&self) -> Bitboard {
        self.occupied
    }

    /// Check if a square is occupied by a specific player
    pub fn is_occupied_by_player(&self, pos: Position, player: Player) -> bool {
        if let Some(piece) = self.get_piece(pos) {
            piece.player == player
        } else {
            false
        }
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
            attack_tables: self.attack_tables.clone(),
            magic_table: self.magic_table.clone(),
            sliding_generator: self.sliding_generator.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Player, PieceType, CapturedPieces, Position};

    #[test]
    fn test_from_fen_startpos() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let (board, player, captured) = BitboardBoard::from_fen(fen).unwrap();

        assert_eq!(player, Player::Black);
        assert!(captured.black.is_empty());
        assert!(captured.white.is_empty());

        // Spot check a few pieces
        let black_lance = board.get_piece(Position::new(8, 0)).unwrap();
        assert_eq!(black_lance.piece_type, PieceType::Lance);
        assert_eq!(black_lance.player, Player::Black);

        let white_king = board.get_piece(Position::new(0, 4)).unwrap();
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.player, Player::White);
        
        let black_pawn = board.get_piece(Position::new(6, 4)).unwrap();
        assert_eq!(black_pawn.piece_type, PieceType::Pawn);
        assert_eq!(black_pawn.player, Player::Black);
    }

    #[test]
    fn test_from_fen_with_drops_and_promotions() {
        let fen = "8l/1l+R2P3/p2pBG1pp/kps1p4/Nn1P2G2/P1P1P2PP/1PS6/1KSG3+r1/LN2+p3L w Sbgn3p 124";
        let (board, player, captured) = BitboardBoard::from_fen(fen).unwrap();

        assert_eq!(player, Player::White);

        // Check captured pieces
        assert_eq!(captured.black.iter().filter(|&&p| p == PieceType::Silver).count(), 1);
        assert_eq!(captured.white.iter().filter(|&&p| p == PieceType::Pawn).count(), 3);
        assert_eq!(captured.white.iter().filter(|&&p| p == PieceType::Knight).count(), 1);
        assert_eq!(captured.white.iter().filter(|&&p| p == PieceType::Gold).count(), 1);

        // Spot check a few pieces on board
        let promoted_rook = board.get_piece(Position::new(1, 2)).unwrap();
        assert_eq!(promoted_rook.piece_type, PieceType::PromotedRook);
        assert_eq!(promoted_rook.player, Player::Black);

        let promoted_pawn = board.get_piece(Position::new(8, 4)).unwrap();
        assert_eq!(promoted_pawn.piece_type, PieceType::PromotedPawn);
        assert_eq!(promoted_pawn.player, Player::White);
    }
}