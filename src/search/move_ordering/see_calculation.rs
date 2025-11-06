//! Static Exchange Evaluation (SEE) calculation
//! 
//! This module contains the SEE calculation implementation.
//! SEE evaluates the material gain/loss from a sequence of captures
//! starting with the given move.

use crate::types::*;
use crate::bitboards::BitboardBoard;
use std::collections::HashMap;

/// SEE calculation result
pub type SEEResult<T> = Result<T, String>;

/// Find all attackers and defenders of a given square
/// 
/// This function identifies all pieces that can attack the target square.
/// For SEE calculation, we need to know which pieces can capture on this square.
/// 
/// Returns a vector of all pieces that can attack the square, with their positions.
/// The caller will separate them by player.
pub fn find_attackers_defenders(square: Position, board: &BitboardBoard) -> Vec<(Position, Piece)> {
    let mut all_attackers = Vec::new();
    
    // Iterate through all squares on the board to find pieces
    for row in 0..9 {
        for col in 0..9 {
            let position = Position::new(row, col);
            
            // Skip the target square itself (we're evaluating captures on it)
            if position == square {
                continue;
            }
            
            // Check if there's a piece at this position
            if let Some(piece) = board.get_piece(position) {
                // Check if this specific piece attacks the target square
                if piece_attacks_square(piece, position, square, board) {
                    all_attackers.push((position, piece.clone()));
                }
            }
        }
    }
    
    // Sort by piece value (ascending) - least valuable first for SEE
    all_attackers.sort_by_key(|(_, p)| p.piece_type.base_value());
    
    all_attackers
}

/// Check if a specific piece attacks a square
/// 
/// This duplicates the logic from BitboardBoard::piece_attacks_square
/// since that method is private.
pub fn piece_attacks_square(
    piece: &Piece,
    from_pos: Position,
    target_pos: Position,
    board: &BitboardBoard,
) -> bool {
    let player = piece.player;
    
    // Early bounds check
    if from_pos.row >= 9 || from_pos.col >= 9 || target_pos.row >= 9 || target_pos.col >= 9 {
        return false;
    }

    match piece.piece_type {
        PieceType::Pawn => {
            let dir: i8 = if player == Player::Black { 1 } else { -1 };
            let new_row = from_pos.row as i8 + dir;
            if new_row >= 0 && new_row < 9 {
                let attack_pos = Position::new(new_row as u8, from_pos.col);
                return attack_pos == target_pos;
            }
            false
        },
        PieceType::Knight => {
            let dir: i8 = if player == Player::Black { 1 } else { -1 };
            let move_offsets = [(2 * dir, 1), (2 * dir, -1)];
            for (dr, dc) in move_offsets.iter() {
                let new_row = from_pos.row as i8 + dr;
                let new_col = from_pos.col as i8 + dc;
                if new_row >= 0 && new_col >= 0 && new_row < 9 && new_col < 9 {
                    let attack_pos = Position::new(new_row as u8, new_col as u8);
                    if attack_pos == target_pos {
                        return true;
                    }
                }
            }
            false
        },
        PieceType::Lance => {
            let dir: i8 = if player == Player::Black { 1 } else { -1 };
            check_ray_attack(from_pos, target_pos, (dir, 0), board)
        },
        PieceType::Rook => {
            check_ray_attack(from_pos, target_pos, (1, 0), board) ||
            check_ray_attack(from_pos, target_pos, (-1, 0), board) ||
            check_ray_attack(from_pos, target_pos, (0, 1), board) ||
            check_ray_attack(from_pos, target_pos, (0, -1), board)
        },
        PieceType::Bishop => {
            check_ray_attack(from_pos, target_pos, (1, 1), board) ||
            check_ray_attack(from_pos, target_pos, (1, -1), board) ||
            check_ray_attack(from_pos, target_pos, (-1, 1), board) ||
            check_ray_attack(from_pos, target_pos, (-1, -1), board)
        },
        PieceType::PromotedBishop => {
            // Bishop + King moves
            check_ray_attack(from_pos, target_pos, (1, 1), board) ||
            check_ray_attack(from_pos, target_pos, (1, -1), board) ||
            check_ray_attack(from_pos, target_pos, (-1, 1), board) ||
            check_ray_attack(from_pos, target_pos, (-1, -1), board) ||
            check_king_attack(from_pos, target_pos, player)
        },
        PieceType::PromotedRook => {
            // Rook + King moves
            check_ray_attack(from_pos, target_pos, (1, 0), board) ||
            check_ray_attack(from_pos, target_pos, (-1, 0), board) ||
            check_ray_attack(from_pos, target_pos, (0, 1), board) ||
            check_ray_attack(from_pos, target_pos, (0, -1), board) ||
            check_king_attack(from_pos, target_pos, player)
        },
        PieceType::Silver | 
        PieceType::Gold | 
        PieceType::King | 
        PieceType::PromotedPawn | 
        PieceType::PromotedLance | 
        PieceType::PromotedKnight | 
        PieceType::PromotedSilver => {
            check_king_attack(from_pos, target_pos, player)
        }
    }
}

/// Check if a ray from from_pos in direction (dr, dc) hits target_pos
fn check_ray_attack(
    from_pos: Position,
    target_pos: Position,
    direction: (i8, i8),
    board: &BitboardBoard,
) -> bool {
    let (dr, dc) = direction;
    let mut current_pos = from_pos;
    
    loop {
        let new_row = current_pos.row as i8 + dr;
        let new_col = current_pos.col as i8 + dc;
        
        // Out of bounds
        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
            break;
        }
        
        current_pos = Position::new(new_row as u8, new_col as u8);
        
        // Found target
        if current_pos == target_pos {
            return true;
        }
        
        // Blocked by a piece
        if board.is_square_occupied(current_pos) {
            break;
        }
    }
    
    false
}

/// Check if a king-like piece attacks target_pos
fn check_king_attack(
    from_pos: Position,
    target_pos: Position,
    _player: Player,
) -> bool {
    let row_diff = (from_pos.row as i8 - target_pos.row as i8).abs();
    let col_diff = (from_pos.col as i8 - target_pos.col as i8).abs();
    
    // King attacks adjacent squares (including diagonals)
    row_diff <= 1 && col_diff <= 1 && (row_diff != 0 || col_diff != 0)
}

/// Calculate Static Exchange Evaluation (SEE) for a move
/// 
/// This function simulates the sequence of captures that would follow
/// the given move and returns the net material gain/loss.
/// 
/// # Arguments
/// * `move_` - The move to evaluate
/// * `board` - The current board position
/// 
/// # Returns
/// The net material gain/loss from the exchange sequence
pub fn calculate_see_internal(move_: &Move, board: &BitboardBoard) -> i32 {
    let from = move_.from.unwrap_or(Position::new(0, 0));
    let to = move_.to;
    let moving_player = move_.player;
    let opponent = moving_player.opposite();
    
    // Get the piece being captured
    let captured_piece = match &move_.captured_piece {
        Some(piece) => piece,
        None => return 0, // No capture, no SEE value
    };
    
    // Get the attacking piece (the piece making the capture)
    let attacking_piece = match board.get_piece(from) {
        Some(piece) => piece.clone(),
        None => {
            // Drop move - use the piece type from the move
            Piece::new(move_.piece_type, moving_player)
        }
    };
    
    // Start with the value of the captured piece, subtract the attacker's value
    let mut gain = captured_piece.piece_type.base_value() - attacking_piece.piece_type.base_value();
    
    // Find all pieces that can attack the target square
    let all_attackers = find_attackers_defenders(to, board);
    
    // Separate attackers and defenders by player
    // Attackers: pieces from the moving player that can continue the exchange
    // Defenders: pieces from the opponent that can recapture
    let attackers: Vec<Piece> = all_attackers.iter()
        .filter(|(pos, p)| p.player == moving_player && *pos != from)
        .map(|(_, p)| p.clone())
        .collect();
    let defenders: Vec<Piece> = all_attackers.iter()
        .filter(|(_, p)| p.player == opponent)
        .map(|(_, p)| p.clone())
        .collect();
    
    // If no defenders, it's a winning capture
    if defenders.is_empty() {
        return gain;
    }
    
    // Simulate the exchange sequence
    // The exchange continues with the least valuable piece at each step
    // We alternate between attackers and defenders
    // After the initial capture, the opponent recaptures, then we can recapture, etc.
    
    // Start with defenders (opponent recaptures after the initial capture)
    let mut current_side = defenders; // Current side's pieces (opponent recaptures first)
    let mut other_side = attackers; // Other side's pieces (we can recapture)
    
    // Continue the exchange until one side runs out of pieces
    loop {
        // Find the least valuable piece on the current side
        if current_side.is_empty() {
            break; // Current side can't continue, exchange ends (we win)
        }
        
        // Find least valuable piece
        let mut min_value = i32::MAX;
        let mut min_index = None;
        for (index, piece) in current_side.iter().enumerate() {
            let value = piece.piece_type.base_value();
            if value < min_value {
                min_value = value;
                min_index = Some(index);
            }
        }
        
        if min_index.is_none() {
            break;
        }
        
        let capturing_piece = current_side.remove(min_index.unwrap());
        
        // Subtract the value of the capturing piece (we lose this piece)
        gain -= capturing_piece.piece_type.base_value();
        
        // If the other side can't recapture, we win the exchange
        if other_side.is_empty() {
            break;
        }
        
        // Switch sides - the other side now captures
        std::mem::swap(&mut current_side, &mut other_side);
        
        // Add the value of the captured piece (the piece that was just captured)
        // This is the piece we just captured from the opponent
        gain += capturing_piece.piece_type.base_value();
    }
    
    gain
}

/// Score a move using Static Exchange Evaluation (SEE)
/// 
/// SEE evaluates the material gain/loss from a sequence of captures
/// starting with the given move. This provides a more accurate assessment
/// of capture moves than simple piece values.
/// 
/// # Arguments
/// * `move_` - The move to score
/// * `board` - The current board position
/// * `see_weight` - Weight for SEE scores
/// 
/// # Returns
/// The SEE score for the move, scaled by the weight
pub fn score_see_move(move_: &Move, board: &BitboardBoard, see_weight: i32) -> SEEResult<i32> {
    if !move_.is_capture {
        return Ok(0);
    }

    let see_value = calculate_see_internal(move_, board);
    let see_score = (see_value * see_weight) / 1000;
    
    Ok(see_score)
}

/// SEE cache manager
/// 
/// Manages caching of SEE calculation results for performance optimization.
#[derive(Debug, Clone)]
pub struct SEECache {
    /// SEE cache: maps (from_square, to_square) -> SEE value
    cache: HashMap<(Position, Position), i32>,
    /// Maximum cache size
    max_size: usize,
}

impl SEECache {
    /// Create a new SEE cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Get a cached SEE value
    pub fn get(&self, from: Position, to: Position) -> Option<i32> {
        self.cache.get(&(from, to)).copied()
    }

    /// Cache a SEE value
    pub fn insert(&mut self, from: Position, to: Position, value: i32) {
        if self.cache.len() < self.max_size {
            self.cache.insert((from, to), value);
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.cache.len() >= self.max_size
    }

    /// Get memory usage estimate for cache
    pub fn memory_bytes(&self) -> usize {
        self.cache.len() * (std::mem::size_of::<(Position, Position)>() + std::mem::size_of::<i32>())
    }
}

impl Default for SEECache {
    fn default() -> Self {
        Self::new(1000) // Default max size
    }
}
