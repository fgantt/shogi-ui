use crate::types::*;
use crate::bitboards::*;
use std::collections::HashSet;

/// Move generator for the Shogi engine
pub struct MoveGenerator {
    // Precomputed move patterns
    move_patterns: MovePatterns,
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            move_patterns: MovePatterns::new(),
        }
    }

    /// Generate all legal moves for a given position
    pub fn generate_legal_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        web_sys::console::log_1(&format!("generate_legal_moves: start").into());
        let mut legal_moves = Vec::new();
        
        // Generate moves for pieces on the board
        legal_moves.extend(self.generate_piece_moves(board, player));
        
        // Generate drop moves
        legal_moves.extend(self.generate_drop_moves(board, player, captured_pieces));
        
        legal_moves
    }

    /// Generate moves for pieces currently on the board
    fn generate_piece_moves(&self, board: &BitboardBoard, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        
        // Iterate through all piece types
        for piece_type in 0..14 {
            let piece_bitboard = board.get_pieces()[player_idx][piece_type];
            let mut current_bitboard = piece_bitboard;
            
            // Generate moves for each piece of this type
            while let Some(pos) = pop_lsb(&mut current_bitboard) {
                let piece_type_enum = PieceType::from_u8(piece_type as u8);
                let piece = Piece::new(piece_type_enum, player);
                
                web_sys::console::log_1(&format!("generate_piece_moves: piece_type={:?}, player={:?}, pos={:?}", piece.piece_type, piece.player, pos).into());
                // Generate moves for this piece
                let piece_moves = self.generate_moves_for_piece(board, &piece, pos);
                moves.extend(piece_moves);
            }
        }
        
        moves
    }

    /// Generate moves for a specific piece
    fn generate_moves_for_piece(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        web_sys::console::log_1(&format!("generate_moves_for_piece: piece={:?}, pos={:?}", piece, pos).into());
        match piece.piece_type {
            PieceType::Pawn => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_pawn_moves").into());
                moves.extend(self.generate_pawn_moves(board, piece, pos));
            }
            PieceType::Lance => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_lance_moves").into());
                moves.extend(self.generate_lance_moves(board, piece, pos));
            }
            PieceType::Knight => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_knight_moves").into());
                moves.extend(self.generate_knight_moves(board, piece, pos));
            }
            PieceType::Silver => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_silver_moves").into());
                moves.extend(self.generate_silver_moves(board, piece, pos));
            }
            PieceType::Gold => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_gold_moves").into());
                moves.extend(self.generate_gold_moves(board, piece, pos));
            }
            PieceType::Bishop => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_bishop_moves").into());
                moves.extend(self.generate_bishop_moves(board, piece, pos));
            }
            PieceType::Rook => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_rook_moves").into());
                moves.extend(self.generate_rook_moves(board, piece, pos));
            }
            PieceType::King => {
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_king_moves").into());
                moves.extend(self.generate_king_moves(board, piece, pos));
            }
            _ => {
                // Promoted pieces
                web_sys::console::log_1(&format!("generate_moves_for_piece: calling generate_promoted_moves").into());
                moves.extend(self.generate_promoted_moves(board, piece, pos));
            }
        }
        
        moves
    }

    /// Generate pawn moves
    fn generate_pawn_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward_direction = if piece.player == Player::Black { -1 } else { 1 };
        let new_row = pos.row as i8 + forward_direction;
        
        if new_row >= 0 && new_row < 9 {
            let new_pos = Position::new(new_row as u8, pos.col);
            
            if !board.is_square_occupied(new_pos) {
                // Regular move
                let move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                if board.is_legal_move(&move_) {
                    moves.push(move_);
                }
                
                // Check if promotion is possible
                if piece.piece_type.can_promote() {
                    let promotion_zone = match piece.player {
                        Player::Black => new_row <= 2,
                        Player::White => new_row >= 6,
                    };
                    
                    if promotion_zone {
                        let promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                        if board.is_legal_move(&promoted_move) {
                            moves.push(promoted_move);
                        }
                    }
                }
            } else {
                // Capture move
                if let Some(target_piece) = board.get_piece(new_pos) {
                    if target_piece.player != piece.player {
                        let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                        if board.is_legal_move(&move_) {
                            moves.push(move_);
                        }
                        
                        // Check promotion on capture
                        if piece.piece_type.can_promote() {
                            let promotion_zone = match piece.player {
                                Player::Black => new_row <= 2,
                                Player::White => new_row >= 6,
                            };
                            
                            if promotion_zone {
                                let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                                promoted_move.is_capture = true;
                                promoted_move.captured_piece = Some(target_piece.clone());
                                if board.is_legal_move(&promoted_move) {
                                    moves.push(promoted_move);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate lance moves
    fn generate_lance_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward_direction = if piece.player == Player::Black { -1 } else { 1 };
        
        // Generate moves in the forward direction
        let mut current_row = pos.row as i8;
        loop {
            current_row += forward_direction;
            if current_row < 0 || current_row >= 9 {
                break;
            }
            
            let new_pos = Position::new(current_row as u8, pos.col);
            
            if board.is_square_occupied(new_pos) {
                // Square is occupied
                if let Some(target_piece) = board.get_piece(new_pos) {
                    if target_piece.player != piece.player {
                        // Capture move
                        let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                        if board.is_legal_move(&move_) {
                            moves.push(move_);
                        }
                        
                        // Check promotion on capture
                        if piece.piece_type.can_promote() {
                            let promotion_zone = match piece.player {
                                Player::Black => current_row <= 2,
                                Player::White => current_row >= 6,
                            };
                            
                            if promotion_zone {
                                let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                                promoted_move.is_capture = true;
                                promoted_move.captured_piece = Some(target_piece.clone());
                                if board.is_legal_move(&promoted_move) {
                                    moves.push(promoted_move);
                                }
                            }
                        }
                    }
                }
                break; // Can't move further
            } else {
                // Empty square
                let move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                if board.is_legal_move(&move_) {
                    moves.push(move_);
                }
                
                // Check promotion
                if piece.piece_type.can_promote() {
                    let promotion_zone = match piece.player {
                        Player::Black => current_row <= 2,
                        Player::White => current_row >= 6,
                    };
                    
                    if promotion_zone {
                        let promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                        if board.is_legal_move(&promoted_move) {
                            moves.push(promoted_move);
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate knight moves
    fn generate_knight_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward_direction = if piece.player == Player::Black { -1 } else { 1 };
        
        let knight_moves = [
            (2 * forward_direction, -1),
            (2 * forward_direction, 1),
        ];
        
        for (dr, dc) in knight_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                if !board.is_square_occupied_by(new_pos, piece.player) {
                    let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                    }

                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                    
                    // Check promotion
                    if piece.piece_type.can_promote() {
                        let promotion_zone = match piece.player {
                            Player::Black => new_row <= 2,
                            Player::White => new_row >= 6,
                        };
                        
                        if promotion_zone {
                            let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                            if let Some(target_piece) = board.get_piece(new_pos) {
                                promoted_move.is_capture = true;
                                promoted_move.captured_piece = Some(target_piece.clone());
                            }
                            if board.is_legal_move(&promoted_move) {
                                moves.push(promoted_move);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate silver moves
    fn generate_silver_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward_direction = if piece.player == Player::Black { -1 } else { 1 };
        
        let silver_moves = [
            (forward_direction, 0),      // Forward
            (forward_direction, -1),     // Forward-left
            (forward_direction, 1),      // Forward-right
            (-forward_direction, -1),    // Backward-left
            (-forward_direction, 1),     // Backward-right
        ];
        
        for (dr, dc) in silver_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                if !board.is_square_occupied_by(new_pos, piece.player) {
                    let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                    }
                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                    
                    // Check promotion
                    if piece.piece_type.can_promote() {
                        let promotion_zone = match piece.player {
                            Player::Black => new_row <= 2,
                            Player::White => new_row >= 6,
                        };
                        
                        if promotion_zone {
                            let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                            if let Some(target_piece) = board.get_piece(new_pos) {
                                promoted_move.is_capture = true;
                                promoted_move.captured_piece = Some(target_piece.clone());
                            }
                            if board.is_legal_move(&promoted_move) {
                                moves.push(promoted_move);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate gold moves
    fn generate_gold_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward_direction = if piece.player == Player::Black { -1 } else { 1 };
        
        let gold_moves = [
            (forward_direction, 0),      // Forward
            (forward_direction, -1),     // Forward-left
            (forward_direction, 1),      // Forward-right
            (0, -1),                     // Left
            (0, 1),                      // Right
            (-forward_direction, 0),     // Backward
        ];
        
        for (dr, dc) in gold_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                if !board.is_square_occupied_by(new_pos, piece.player) {
                    let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                    }
                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                }
            }
        }
        
        moves
    }

    /// Generate bishop moves
    fn generate_bishop_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        
        let diagonal_directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        
        for (dr, dc) in diagonal_directions.iter() {
            let mut current_row = pos.row as i8;
            let mut current_col = pos.col as i8;
            
            loop {
                current_row += dr;
                current_col += dc;
                
                if current_row < 0 || current_row >= 9 || current_col < 0 || current_col >= 9 {
                    break;
                }
                
                let new_pos = Position::new(current_row as u8, current_col as u8);
                
                if board.is_square_occupied(new_pos) {
                    // Square is occupied
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        if target_piece.player != piece.player {
                            // Capture move
                            let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                            move_.is_capture = true;
                            move_.captured_piece = Some(target_piece.clone());
                            if board.is_legal_move(&move_) {
                                moves.push(move_);
                            }
                            
                            // Check promotion
                            if piece.piece_type.can_promote() {
                                let promotion_zone = match piece.player {
                                    Player::Black => current_row <= 2,
                                    Player::White => current_row >= 6,
                                };
                                
                                if promotion_zone {
                                    let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                                    promoted_move.is_capture = true;
                                    promoted_move.captured_piece = Some(target_piece.clone());
                                    if board.is_legal_move(&promoted_move) {
                                        moves.push(promoted_move);
                                    }
                                }
                            }
                        }
                    }
                    break; // Can't move further
                } else {
                    // Empty square
                    let move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                    
                    // Check promotion
                    if piece.piece_type.can_promote() {
                        let promotion_zone = match piece.player {
                            Player::Black => current_row <= 2,
                            Player::White => current_row >= 6,
                        };
                        
                        if promotion_zone {
                            let promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                            if board.is_legal_move(&promoted_move) {
                                moves.push(promoted_move);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate rook moves
    fn generate_rook_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        
        let orthogonal_directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        
        for (dr, dc) in orthogonal_directions.iter() {
            let mut current_row = pos.row as i8;
            let mut current_col = pos.col as i8;
            
            loop {
                current_row += dr;
                current_col += dc;
                
                if current_row < 0 || current_row >= 9 || current_col < 0 || current_col >= 9 {
                    break;
                }
                
                let new_pos = Position::new(current_row as u8, current_col as u8);
                
                if board.is_square_occupied(new_pos) {
                    // Square is occupied
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        if target_piece.player != piece.player {
                            // Capture move
                            let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                            move_.is_capture = true;
                            move_.captured_piece = Some(target_piece.clone());
                            if board.is_legal_move(&move_) {
                                moves.push(move_);
                            }
                            
                            // Check promotion
                            if piece.piece_type.can_promote() {
                                let promotion_zone = match piece.player {
                                    Player::Black => current_row <= 2,
                                    Player::White => current_row >= 6,
                                };
                                
                                if promotion_zone {
                                    let mut promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                                    promoted_move.is_capture = true;
                                    promoted_move.captured_piece = Some(target_piece.clone());
                                    if board.is_legal_move(&promoted_move) {
                                        moves.push(promoted_move);
                                    }
                                }
                            }
                        }
                    }
                    break; // Can't move further
                } else {
                    // Empty square
                    let move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                    
                    // Check promotion
                    if piece.piece_type.can_promote() {
                        let promotion_zone = match piece.player {
                            Player::Black => current_row <= 2,
                            Player::White => current_row >= 6,
                        };
                        
                        if promotion_zone {
                            let promoted_move = Move::new_move(pos, new_pos, piece.piece_type, piece.player, true);
                            if board.is_legal_move(&promoted_move) {
                                moves.push(promoted_move);
                            }
                        }
                    }
                }
            }
        }
        
        moves
    }

    /// Generate king moves
    fn generate_king_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        
        let king_moves = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1), (0, 1),
            (1, -1), (1, 0), (1, 1),
        ];
        
        for (dr, dc) in king_moves.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                
                if !board.is_square_occupied_by(new_pos, piece.player) {
                    let mut move_ = Move::new_move(pos, new_pos, piece.piece_type, piece.player, false);
                    if let Some(target_piece) = board.get_piece(new_pos) {
                        move_.is_capture = true;
                        move_.captured_piece = Some(target_piece.clone());
                    }
                    if board.is_legal_move(&move_) {
                        moves.push(move_);
                    }
                }
            }
        }
        
        moves
    }

    /// Generate moves for promoted pieces
    fn generate_promoted_moves(&self, board: &BitboardBoard, piece: &Piece, pos: Position) -> Vec<Move> {
        match piece.piece_type {
            PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight => {
                // These pieces move like Gold
                self.generate_gold_moves(board, piece, pos)
            }
            PieceType::PromotedSilver => {
                // Promoted Silver moves like Gold
                self.generate_gold_moves(board, piece, pos)
            }
            PieceType::PromotedBishop => {
                // Promoted Bishop can move like Bishop OR King
                let mut moves = self.generate_bishop_moves(board, piece, pos);
                moves.extend(self.generate_king_moves(board, piece, pos));
                moves
            }
            PieceType::PromotedRook => {
                // Promoted Rook can move like Rook OR King
                let mut moves = self.generate_rook_moves(board, piece, pos);
                moves.extend(self.generate_king_moves(board, piece, pos));
                moves
            }
            _ => Vec::new(),
        }
    }

    /// Generate drop moves
    fn generate_drop_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut processed_pieces = HashSet::new();
        let captured = if player == Player::Black { &captured_pieces.black } else { &captured_pieces.white };

        for &piece_type in captured {
            if !processed_pieces.insert(piece_type) {
                continue; // Already processed this piece type
            }

            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    
                    if !board.is_square_occupied(pos) {
                        let move_ = Move::new_drop(piece_type, pos, player);
                        if board.is_legal_move(&move_) {
                            moves.push(move_);
                        }
                    }
                }
            }
        }
        
        moves
    }
}

/// Precomputed move patterns for efficient move generation
#[derive(Clone)]
struct MovePatterns {
    // This would contain precomputed move patterns
    // For now, it's a placeholder
}

impl MovePatterns {
    fn new() -> Self {
        Self {}
    }
}
