//! Fast pattern matching algorithms for endgame tablebases
//! 
//! This module provides optimized pattern matching utilities that use bitboard
//! operations and lookup tables for efficient endgame position analysis.

use crate::{BitboardBoard, Player, Position, PieceType, Piece};
use crate::types::{Bitboard, count_bits, get_lsb};
use std::collections::HashMap;

/// Fast pattern matcher for endgame positions
pub struct PatternMatcher {
    /// Cache for piece positions to avoid repeated scanning
    piece_cache: HashMap<u64, Vec<(Piece, Position)>>,
    /// Precomputed distance tables for common patterns
    distance_cache: HashMap<(Position, Position), i32>,
    /// Bitboard masks for common piece patterns
    pattern_masks: PatternMasks,
}

/// Precomputed bitboard masks for common patterns
#[derive(Clone)]
pub struct PatternMasks {
    /// King attack patterns for each position
    king_attacks: [Bitboard; 81],
    /// Gold attack patterns for each position
    gold_attacks: [Bitboard; 81],
    /// Silver attack patterns for each position
    silver_attacks: [Bitboard; 81],
    /// Rook attack patterns for each position
    rook_attacks: [Bitboard; 81],
    /// Bishop attack patterns for each position
    bishop_attacks: [Bitboard; 81],
}

impl PatternMatcher {
    /// Create a new pattern matcher with precomputed patterns
    pub fn new() -> Self {
        Self {
            piece_cache: HashMap::new(),
            distance_cache: HashMap::new(),
            pattern_masks: PatternMasks::new(),
        }
    }

    /// Fast piece detection using bitboard operations
    pub fn find_pieces_fast(&mut self, board: &BitboardBoard, player: Player, piece_type: PieceType) -> Vec<Position> {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let piece_idx = piece_type.to_u8() as usize;
        let piece_bitboard = pieces[player_idx][piece_idx];
        
        self.bitboard_to_positions(piece_bitboard)
    }

    /// Convert bitboard to list of positions efficiently
    fn bitboard_to_positions(&self, bitboard: Bitboard) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut remaining = bitboard;
        
        while remaining != 0 {
            if let Some(pos) = get_lsb(remaining) {
                positions.push(pos);
                remaining &= remaining - 1; // Clear the least significant bit
            } else {
                break;
            }
        }
        
        positions
    }

    /// Fast king detection using bitboard operations
    pub fn find_kings_fast(&mut self, board: &BitboardBoard) -> (Option<Position>, Option<Position>) {
        let black_kings = self.find_pieces_fast(board, Player::Black, PieceType::King);
        let white_kings = self.find_pieces_fast(board, Player::White, PieceType::King);
        
        (black_kings.first().copied(), white_kings.first().copied())
    }

    /// Fast piece counting using bitboard operations
    pub fn count_pieces_fast(&self, board: &BitboardBoard, player: Player, piece_type: PieceType) -> u32 {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let piece_idx = piece_type.to_u8() as usize;
        let piece_bitboard = pieces[player_idx][piece_idx];
        
        count_bits(piece_bitboard)
    }

    /// Fast endgame pattern detection
    pub fn detect_endgame_pattern(&mut self, board: &BitboardBoard, player: Player) -> EndgamePattern {
        let (_black_king, _white_king) = self.find_kings_fast(board);
        
        // Count pieces efficiently
        let black_pieces = self.count_all_pieces(board, Player::Black);
        let white_pieces = self.count_all_pieces(board, Player::White);
        
        // Detect specific endgame patterns
        if black_pieces == 1 && white_pieces == 1 {
            return EndgamePattern::KingVsKing;
        }
        
        if black_pieces == 2 && white_pieces == 1 {
            let attacking_pieces = self.find_attacking_pieces_fast(board, player);
            match attacking_pieces.len() {
                2 => {
                    if attacking_pieces.iter().any(|(_, piece_type)| *piece_type == PieceType::Gold) {
                        return EndgamePattern::KingGoldVsKing;
                    }
                    if attacking_pieces.iter().any(|(_, piece_type)| *piece_type == PieceType::Silver) {
                        return EndgamePattern::KingSilverVsKing;
                    }
                    if attacking_pieces.iter().any(|(_, piece_type)| *piece_type == PieceType::Rook) {
                        return EndgamePattern::KingRookVsKing;
                    }
                }
                _ => {}
            }
        }
        
        EndgamePattern::Unknown
    }

    /// Count all pieces for a player efficiently
    fn count_all_pieces(&self, board: &BitboardBoard, player: Player) -> u32 {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        
        let mut total = 0;
        for piece_bitboard in &pieces[player_idx] {
            total += count_bits(*piece_bitboard);
        }
        
        total
    }

    /// Find attacking pieces (current player's pieces) efficiently
    fn find_attacking_pieces_fast(&mut self, board: &BitboardBoard, player: Player) -> Vec<(Position, PieceType)> {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let mut attacking_pieces = Vec::new();
        
        for piece_type in [
            PieceType::Pawn, PieceType::Lance, PieceType::Knight,
            PieceType::Silver, PieceType::Gold, PieceType::Bishop,
            PieceType::Rook, PieceType::King
        ] {
            let piece_idx = piece_type.to_u8() as usize;
            let piece_bitboard = pieces[player_idx][piece_idx];
            let positions = self.bitboard_to_positions(piece_bitboard);
            
            for pos in positions {
                attacking_pieces.push((pos, piece_type));
            }
        }
        
        attacking_pieces
    }

    /// Fast Manhattan distance calculation with caching
    pub fn manhattan_distance_cached(&mut self, from: Position, to: Position) -> i32 {
        if let Some(&distance) = self.distance_cache.get(&(from, to)) {
            return distance;
        }
        
        let distance = (from.row as i32 - to.row as i32).abs() +
                       (from.col as i32 - to.col as i32).abs();
        
        self.distance_cache.insert((from, to), distance);
        distance
    }

    /// Fast check for piece attacks using precomputed patterns
    pub fn is_square_attacked_by_piece(&self, target: Position, attacker: Position, piece_type: PieceType) -> bool {
        match piece_type {
            PieceType::King => self.pattern_masks.king_attacks[attacker.to_u8() as usize] & (1 << target.to_u8()) != 0,
            PieceType::Gold => self.pattern_masks.gold_attacks[attacker.to_u8() as usize] & (1 << target.to_u8()) != 0,
            PieceType::Silver => self.pattern_masks.silver_attacks[attacker.to_u8() as usize] & (1 << target.to_u8()) != 0,
            PieceType::Rook => self.pattern_masks.rook_attacks[attacker.to_u8() as usize] & (1 << target.to_u8()) != 0,
            PieceType::Bishop => self.pattern_masks.bishop_attacks[attacker.to_u8() as usize] & (1 << target.to_u8()) != 0,
            _ => false, // Other pieces not implemented yet
        }
    }

    /// Fast mate detection using pattern matching
    pub fn is_mate_pattern(&mut self, board: &BitboardBoard, player: Player, defending_king: Position) -> bool {
        let opponent = player.opposite();
        let (black_king, white_king) = self.find_kings_fast(board);
        let attacking_king = if player == Player::Black { black_king } else { white_king };
        
        if let Some(king_pos) = attacking_king {
            // Check if king is close enough to support mate
            let distance = self.manhattan_distance_cached(king_pos, defending_king);
            if distance <= 2 {
                // Check if defending king has no escape squares
                return self.has_no_escape_squares(board, defending_king, opponent);
            }
        }
        
        false
    }

    /// Check if a position has no escape squares
    fn has_no_escape_squares(&self, board: &BitboardBoard, king_pos: Position, _player: Player) -> bool {
        // Check all 8 directions around the king
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 { continue; }
                
                let new_row = king_pos.row as i8 + dr;
                let new_col = king_pos.col as i8 + dc;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let new_pos = Position::new(new_row as u8, new_col as u8);
                    if !board.is_square_occupied(new_pos) {
                        // This is a potential escape square
                        // In a real implementation, we'd check if it's attacked
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Clear caches to free memory
    pub fn clear_caches(&mut self) {
        self.piece_cache.clear();
        self.distance_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.piece_cache.len(), self.distance_cache.len())
    }
}

impl PatternMasks {
    /// Create new pattern masks with precomputed attack patterns
    pub fn new() -> Self {
        let mut masks = Self {
            king_attacks: [0; 81],
            gold_attacks: [0; 81],
            silver_attacks: [0; 81],
            rook_attacks: [0; 81],
            bishop_attacks: [0; 81],
        };
        
        masks.precompute_patterns();
        masks
    }

    /// Precompute all attack patterns
    fn precompute_patterns(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                let idx = pos.to_u8() as usize;
                
                self.king_attacks[idx] = self.compute_king_attacks(pos);
                self.gold_attacks[idx] = self.compute_gold_attacks(pos);
                self.silver_attacks[idx] = self.compute_silver_attacks(pos);
                self.rook_attacks[idx] = self.compute_rook_attacks(pos);
                self.bishop_attacks[idx] = self.compute_bishop_attacks(pos);
            }
        }
    }

    /// Compute king attack pattern
    fn compute_king_attacks(&self, pos: Position) -> Bitboard {
        let mut attacks = 0u128;
        
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 { continue; }
                
                let new_row = pos.row as i8 + dr;
                let new_col = pos.col as i8 + dc;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let new_pos = Position::new(new_row as u8, new_col as u8);
                    attacks |= 1 << new_pos.to_u8();
                }
            }
        }
        
        attacks
    }

    /// Compute gold attack pattern
    fn compute_gold_attacks(&self, pos: Position) -> Bitboard {
        let mut attacks = 0u128;
        
        // Gold attacks: forward, diagonally forward, and sideways
        let directions = [(1, 0), (1, 1), (1, -1), (0, 1), (0, -1), (-1, 0)];
        
        for (dr, dc) in directions.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                attacks |= 1 << new_pos.to_u8();
            }
        }
        
        attacks
    }

    /// Compute silver attack pattern
    fn compute_silver_attacks(&self, pos: Position) -> Bitboard {
        let mut attacks = 0u128;
        
        // Silver attacks: forward, diagonally forward, and diagonally backward
        let directions = [(1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];
        
        for (dr, dc) in directions.iter() {
            let new_row = pos.row as i8 + dr;
            let new_col = pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let new_pos = Position::new(new_row as u8, new_col as u8);
                attacks |= 1 << new_pos.to_u8();
            }
        }
        
        attacks
    }

    /// Compute rook attack pattern (simplified - just horizontal and vertical)
    fn compute_rook_attacks(&self, pos: Position) -> Bitboard {
        let mut attacks = 0u128;
        
        // Horizontal and vertical directions
        for &(dr, dc) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            for i in 1..9 {
                let new_row = pos.row as i8 + dr * i;
                let new_col = pos.col as i8 + dc * i;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let new_pos = Position::new(new_row as u8, new_col as u8);
                    attacks |= 1 << new_pos.to_u8();
                } else {
                    break;
                }
            }
        }
        
        attacks
    }

    /// Compute bishop attack pattern (simplified - just diagonal)
    fn compute_bishop_attacks(&self, pos: Position) -> Bitboard {
        let mut attacks = 0u128;
        
        // Diagonal directions
        for &(dr, dc) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            for i in 1..9 {
                let new_row = pos.row as i8 + dr * i;
                let new_col = pos.col as i8 + dc * i;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let new_pos = Position::new(new_row as u8, new_col as u8);
                    attacks |= 1 << new_pos.to_u8();
                } else {
                    break;
                }
            }
        }
        
        attacks
    }
}

/// Endgame pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndgamePattern {
    Unknown,
    KingVsKing,
    KingGoldVsKing,
    KingSilverVsKing,
    KingRookVsKing,
    KingBishopVsKing,
    // Add more patterns as needed
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = PatternMatcher::new();
        assert_eq!(matcher.cache_stats(), (0, 0));
    }

    #[test]
    fn test_fast_piece_detection() {
        let mut matcher = PatternMatcher::new();
        let board = BitboardBoard::new();
        
        let black_kings = matcher.find_pieces_fast(&board, Player::Black, PieceType::King);
        let white_kings = matcher.find_pieces_fast(&board, Player::White, PieceType::King);
        
        assert_eq!(black_kings.len(), 1);
        assert_eq!(white_kings.len(), 1);
    }

    #[test]
    fn test_manhattan_distance_caching() {
        let mut matcher = PatternMatcher::new();
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 4);
        
        let distance1 = matcher.manhattan_distance_cached(pos1, pos2);
        let distance2 = matcher.manhattan_distance_cached(pos1, pos2);
        
        assert_eq!(distance1, 7); // 3 + 4
        assert_eq!(distance1, distance2);
        assert_eq!(matcher.cache_stats().1, 1); // One cached distance
    }

    #[test]
    fn test_endgame_pattern_detection() {
        let mut matcher = PatternMatcher::new();
        let board = BitboardBoard::new();
        
        let pattern = matcher.detect_endgame_pattern(&board, Player::Black);
        // Initial position has more than 2 pieces per side, so should be Unknown
        assert_eq!(pattern, EndgamePattern::Unknown);
    }
}
