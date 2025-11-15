//! Static Exchange Evaluation (SEE) calculation
//!
//! This module contains the SEE calculation implementation.
//! SEE evaluates the material gain/loss from a sequence of captures
//! starting with the given move.

use crate::bitboards::BitboardBoard;
use crate::types::core::{Move, Piece, PieceType, Player, Position};
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
/// Task 3.0.3.1: Rewritten to use bitboard iteration instead of nested 9×9 loops
/// Task 3.0.3.4: Uses iter_pieces for efficient iteration over board pieces
pub fn find_attackers_defenders(square: Position, board: &BitboardBoard) -> Vec<(Position, Piece)> {
    let mut all_attackers = Vec::new();

    // Check both players' pieces
    for player in [Player::Black, Player::White] {
        // Get all pieces of this player that can attack the target square
        // We iterate through all squares and check if pieces attack the target
        // This is more efficient than checking every piece individually
        for (position, piece) in board.iter_pieces() {
            // Skip the target square itself and pieces of the wrong player
            if position == square || piece.player != player {
                continue;
            }
            
            // Check if this piece attacks the target square
            if piece_attacks_square(&piece, position, square, board) {
                all_attackers.push((position, piece.clone()));
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
        }
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
        }
        PieceType::Lance => {
            let dir: i8 = if player == Player::Black { 1 } else { -1 };
            check_ray_attack(from_pos, target_pos, (dir, 0), board)
        }
        PieceType::Rook => {
            check_ray_attack(from_pos, target_pos, (1, 0), board)
                || check_ray_attack(from_pos, target_pos, (-1, 0), board)
                || check_ray_attack(from_pos, target_pos, (0, 1), board)
                || check_ray_attack(from_pos, target_pos, (0, -1), board)
        }
        PieceType::Bishop => {
            check_ray_attack(from_pos, target_pos, (1, 1), board)
                || check_ray_attack(from_pos, target_pos, (1, -1), board)
                || check_ray_attack(from_pos, target_pos, (-1, 1), board)
                || check_ray_attack(from_pos, target_pos, (-1, -1), board)
        }
        PieceType::PromotedBishop => {
            // Bishop + King moves
            check_ray_attack(from_pos, target_pos, (1, 1), board)
                || check_ray_attack(from_pos, target_pos, (1, -1), board)
                || check_ray_attack(from_pos, target_pos, (-1, 1), board)
                || check_ray_attack(from_pos, target_pos, (-1, -1), board)
                || check_king_attack(from_pos, target_pos, player)
        }
        PieceType::PromotedRook => {
            // Rook + King moves
            check_ray_attack(from_pos, target_pos, (1, 0), board)
                || check_ray_attack(from_pos, target_pos, (-1, 0), board)
                || check_ray_attack(from_pos, target_pos, (0, 1), board)
                || check_ray_attack(from_pos, target_pos, (0, -1), board)
                || check_king_attack(from_pos, target_pos, player)
        }
        PieceType::Silver
        | PieceType::Gold
        | PieceType::King
        | PieceType::PromotedPawn
        | PieceType::PromotedLance
        | PieceType::PromotedKnight
        | PieceType::PromotedSilver => check_king_attack(from_pos, target_pos, player),
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
fn check_king_attack(from_pos: Position, target_pos: Position, _player: Player) -> bool {
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
    let attackers: Vec<Piece> = all_attackers
        .iter()
        .filter(|(pos, p)| p.player == moving_player && *pos != from)
        .map(|(_, p)| p.clone())
        .collect();
    let defenders: Vec<Piece> = all_attackers
        .iter()
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

/// SEE cache entry with metadata for eviction policies
/// Task 7.0: Enhanced with LRU tracking
#[derive(Debug, Clone)]
pub struct SEECacheEntry {
    /// Cached SEE value
    pub value: i32,
    /// Last access timestamp (for LRU tracking)
    pub last_access: u64,
    /// Number of times this entry was accessed
    pub access_count: u64,
    /// Absolute value of SEE (for value-based eviction)
    pub see_abs_value: i32,
}

/// SEE cache manager
///
/// Manages caching of SEE calculation results for performance optimization.
/// Task 7.0: Enhanced with advanced eviction policies (FIFO, LRU, Value-Based)
#[derive(Debug, Clone)]
pub struct SEECache {
    /// SEE cache: maps (from_square, to_square) -> cache entry
    cache: HashMap<(Position, Position), SEECacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// LRU access counter (incremented on each access)
    lru_access_counter: u64,
}

impl SEECache {
    /// Create a new SEE cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            lru_access_counter: 0,
        }
    }

    /// Get a cached SEE value
    /// Task 7.0: Updates LRU tracking on access
    pub fn get(&mut self, from: Position, to: Position) -> Option<i32> {
        if let Some(entry) = self.cache.get_mut(&(from, to)) {
            self.lru_access_counter += 1;
            entry.last_access = self.lru_access_counter;
            entry.access_count += 1;
            Some(entry.value)
        } else {
            None
        }
    }

    /// Cache a SEE value
    /// Task 7.0: Evicts least valuable entry when cache is full
    pub fn insert(&mut self, from: Position, to: Position, value: i32) -> bool {
        let key = (from, to);

        // If entry already exists, update it
        if let Some(entry) = self.cache.get_mut(&key) {
            self.lru_access_counter += 1;
            entry.value = value;
            entry.last_access = self.lru_access_counter;
            entry.access_count += 1;
            entry.see_abs_value = value.abs();
            return false; // No eviction
        }

        // If cache has room, just insert
        if self.cache.len() < self.max_size {
            self.lru_access_counter += 1;
            let entry = SEECacheEntry {
                value,
                last_access: self.lru_access_counter,
                access_count: 1,
                see_abs_value: value.abs(),
            };
            self.cache.insert(key, entry);
            return false; // No eviction
        }

        // Cache is full - use LRU eviction with value-preference
        // Prefer evicting entries with low absolute SEE values and low access counts
        if let Some(evict_key) = self.select_eviction_candidate() {
            self.cache.remove(&evict_key);

            self.lru_access_counter += 1;
            let entry = SEECacheEntry {
                value,
                last_access: self.lru_access_counter,
                access_count: 1,
                see_abs_value: value.abs(),
            };
            self.cache.insert(key, entry);
            return true; // Eviction occurred
        }

        false
    }

    /// Select a cache entry for eviction
    /// Task 7.0: Hybrid eviction policy combining LRU and value-based eviction
    /// Prefers evicting entries with low absolute SEE values and low recent access
    fn select_eviction_candidate(&self) -> Option<(Position, Position)> {
        if self.cache.is_empty() {
            return None;
        }

        // Score each entry: lower score = more likely to evict
        // Score = (access_age_weight * normalized_age) + (value_weight * normalized_inverse_value)
        let access_age_weight = 0.6;
        let value_weight = 0.4;

        // Find min/max for normalization
        let max_access = self
            .cache
            .values()
            .map(|e| e.last_access)
            .max()
            .unwrap_or(1);
        let min_access = self
            .cache
            .values()
            .map(|e| e.last_access)
            .min()
            .unwrap_or(1);
        let access_range = (max_access - min_access).max(1) as f32;

        let max_value = self
            .cache
            .values()
            .map(|e| e.see_abs_value)
            .max()
            .unwrap_or(1);
        let min_value = self
            .cache
            .values()
            .map(|e| e.see_abs_value)
            .min()
            .unwrap_or(0);
        let value_range = (max_value - min_value).max(1) as f32;

        let mut evict_key = None;
        let mut evict_score = f32::MAX;

        for (key, entry) in &self.cache {
            // Age score: normalized to 0.0 (oldest) .. 1.0 (newest)
            // Invert so older = higher score (more likely to evict)
            let age_score = if access_range > 0.0 {
                1.0 - ((entry.last_access - min_access) as f32 / access_range)
            } else {
                0.5
            };

            // Value score: normalized to 0.0 (lowest value) .. 1.0 (highest value)
            // Invert so lower value = higher score (more likely to evict)
            let value_score = if value_range > 0.0 {
                1.0 - ((entry.see_abs_value - min_value) as f32 / value_range)
            } else {
                0.5
            };

            // Combined score: higher = more likely to evict
            let combined_score = access_age_weight * age_score + value_weight * value_score;

            if combined_score > evict_score {
                evict_score = combined_score;
                evict_key = Some(*key);
            }
        }

        evict_key
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_access_counter = 0;
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.cache.len() >= self.max_size
    }

    /// Get cache utilization as a percentage
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.cache.len() as f64 / self.max_size as f64) * 100.0
        }
    }

    /// Get memory usage estimate for cache
    pub fn memory_bytes(&self) -> usize {
        self.cache.len()
            * (std::mem::size_of::<(Position, Position)>() + std::mem::size_of::<SEECacheEntry>())
    }

    /// Get maximum cache size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Set maximum cache size
    /// If new size is smaller than current cache size, evicts entries
    pub fn set_max_size(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;

        // Evict entries if cache exceeds new max size
        while self.cache.len() > self.max_size {
            if let Some(evict_key) = self.select_eviction_candidate() {
                self.cache.remove(&evict_key);
            } else {
                break;
            }
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> SEECacheStats {
        let total_access_count: u64 = self.cache.values().map(|e| e.access_count).sum();
        let avg_access_count = if !self.cache.is_empty() {
            total_access_count as f64 / self.cache.len() as f64
        } else {
            0.0
        };

        SEECacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            utilization: self.utilization(),
            total_accesses: total_access_count,
            avg_accesses_per_entry: avg_access_count,
            memory_bytes: self.memory_bytes(),
        }
    }
}

/// SEE cache statistics
#[derive(Debug, Clone)]
pub struct SEECacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache utilization percentage
    pub utilization: f64,
    /// Total accesses across all entries
    pub total_accesses: u64,
    /// Average accesses per cache entry
    pub avg_accesses_per_entry: f64,
    /// Memory usage in bytes
    pub memory_bytes: usize,
}

impl Default for SEECache {
    fn default() -> Self {
        Self::new(5000) // Default max size (increased from 1000)
    }
}
