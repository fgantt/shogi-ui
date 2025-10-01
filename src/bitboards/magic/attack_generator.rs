//! Attack pattern generation for magic bitboards
//! 
//! This module provides functionality to generate attack patterns for rook and bishop
//! pieces using ray-casting algorithms. These patterns are used to build the magic
//! bitboard lookup tables.

use crate::types::{PieceType, Bitboard, AttackConfig, EMPTY_BITBOARD};
use std::collections::HashMap;

/// Attack pattern generator with optimization
pub struct AttackGenerator {
    /// Precomputed direction vectors
    direction_cache: HashMap<PieceType, Vec<Direction>>,
    /// Attack pattern cache
    pattern_cache: HashMap<(u8, PieceType, Bitboard), Bitboard>,
}

/// Direction vector for piece movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction {
    pub row_delta: i8,
    pub col_delta: i8,
}

impl AttackGenerator {
    /// Create a new attack generator
    pub fn new() -> Self {
        let mut generator = Self {
            direction_cache: HashMap::new(),
            pattern_cache: HashMap::new(),
        };
        generator.initialize_direction_cache();
        generator
    }

    /// Generate attack pattern with caching
    pub fn generate_attack_pattern(
        &mut self,
        square: u8,
        piece_type: PieceType,
        blockers: Bitboard
    ) -> Bitboard {
        // Check cache first
        if let Some(cached) = self.pattern_cache.get(&(square, piece_type, blockers)) {
            return *cached;
        }

        let pattern = self.generate_attack_pattern_internal(square, piece_type, blockers);
        
        // Cache the result
        self.pattern_cache.insert((square, piece_type, blockers), pattern);
        pattern
    }

    /// Internal attack pattern generation
    fn generate_attack_pattern_internal(
        &self,
        square: u8,
        piece_type: PieceType,
        blockers: Bitboard
    ) -> Bitboard {
        let directions = self.get_directions(piece_type);
        let mut attacks = EMPTY_BITBOARD;
        
        for direction in directions {
            let mut current_square = square;
            
            while let Some(next_square) = self.get_next_square(current_square, *direction) {
                set_bit(&mut attacks, next_square);
                
                if is_bit_set(blockers, next_square) {
                    break;
                }
                
                current_square = next_square;
            }
        }
        
        attacks
    }

    /// Get directions for a piece type
    fn get_directions(&self, piece_type: PieceType) -> &[Direction] {
        self.direction_cache.get(&piece_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get next square in a direction
    fn get_next_square(&self, square: u8, direction: Direction) -> Option<u8> {
        let row = (square / 9) as i8;
        let col = (square % 9) as i8;
        
        let new_row = row + direction.row_delta;
        let new_col = col + direction.col_delta;
        
        if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
            Some((new_row * 9 + new_col) as u8)
        } else {
            None
        }
    }

    /// Initialize direction cache
    fn initialize_direction_cache(&mut self) {
        // Rook directions
        self.direction_cache.insert(PieceType::Rook, vec![
            Direction { row_delta: 1, col_delta: 0 },   // Up
            Direction { row_delta: -1, col_delta: 0 },  // Down
            Direction { row_delta: 0, col_delta: 1 },   // Right
            Direction { row_delta: 0, col_delta: -1 },  // Left
        ]);

        // Bishop directions
        self.direction_cache.insert(PieceType::Bishop, vec![
            Direction { row_delta: 1, col_delta: 1 },   // Up-Right
            Direction { row_delta: 1, col_delta: -1 },  // Up-Left
            Direction { row_delta: -1, col_delta: 1 },  // Down-Right
            Direction { row_delta: -1, col_delta: -1 }, // Down-Left
        ]);

        // Promoted rook directions (rook + king moves)
        self.direction_cache.insert(PieceType::PromotedRook, vec![
            Direction { row_delta: 1, col_delta: 0 },   // Up
            Direction { row_delta: -1, col_delta: 0 },  // Down
            Direction { row_delta: 0, col_delta: 1 },   // Right
            Direction { row_delta: 0, col_delta: -1 },  // Left
            Direction { row_delta: 1, col_delta: 1 },   // Up-Right
            Direction { row_delta: 1, col_delta: -1 },  // Up-Left
            Direction { row_delta: -1, col_delta: 1 },  // Down-Right
            Direction { row_delta: -1, col_delta: -1 }, // Down-Left
        ]);

        // Promoted bishop directions (bishop + king moves)
        self.direction_cache.insert(PieceType::PromotedBishop, vec![
            Direction { row_delta: 1, col_delta: 1 },   // Up-Right
            Direction { row_delta: 1, col_delta: -1 },  // Up-Left
            Direction { row_delta: -1, col_delta: 1 },  // Down-Right
            Direction { row_delta: -1, col_delta: -1 }, // Down-Left
            Direction { row_delta: 1, col_delta: 0 },   // Up
            Direction { row_delta: -1, col_delta: 0 },  // Down
            Direction { row_delta: 0, col_delta: 1 },   // Right
            Direction { row_delta: 0, col_delta: -1 },  // Left
        ]);
    }

    /// Generate all possible blocker combinations for a mask
    pub fn generate_all_blocker_combinations(&self, mask: Bitboard) -> Vec<Bitboard> {
        let bits: Vec<u8> = (0..81)
            .filter(|&i| is_bit_set(mask, i))
            .collect();
        
        (0..(1 << bits.len())).map(|combination| {
            let mut result = EMPTY_BITBOARD;
            for (i, &bit_pos) in bits.iter().enumerate() {
                if (combination >> i) & 1 != 0 {
                    set_bit(&mut result, bit_pos);
                }
            }
            result
        }).collect()
    }

    /// Generate attack pattern for a specific square and piece type without blockers
    pub fn generate_attack_pattern_no_blockers(&mut self, square: u8, piece_type: PieceType) -> Bitboard {
        self.generate_attack_pattern(square, piece_type, EMPTY_BITBOARD)
    }

    /// Generate attack pattern for a specific square and piece type with all possible blockers
    pub fn generate_attack_pattern_all_blockers(&mut self, square: u8, piece_type: PieceType) -> Bitboard {
        let mask = self.get_relevant_mask(square, piece_type);
        self.generate_attack_pattern(square, piece_type, mask)
    }

    /// Get the relevant mask for a square and piece type
    pub fn get_relevant_mask(&self, square: u8, piece_type: PieceType) -> Bitboard {
        let directions = self.get_directions(piece_type);
        let mut mask = EMPTY_BITBOARD;
        
        for direction in directions {
            let mut current_square = square;
            
            while let Some(next_square) = self.get_next_square(current_square, *direction) {
                set_bit(&mut mask, next_square);
                current_square = next_square;
            }
        }
        
        mask
    }

    /// Generate attack pattern for a specific direction
    pub fn generate_directional_attack(
        &self,
        square: u8,
        direction: Direction,
        blockers: Bitboard
    ) -> Bitboard {
        let mut attacks = EMPTY_BITBOARD;
        let mut current_square = square;
        
        while let Some(next_square) = self.get_next_square(current_square, direction) {
            set_bit(&mut attacks, next_square);
            
            if is_bit_set(blockers, next_square) {
                break;
            }
            
            current_square = next_square;
        }
        
        attacks
    }

    /// Generate attack pattern with custom directions
    pub fn generate_attack_with_directions(
        &mut self,
        square: u8,
        directions: &[Direction],
        blockers: Bitboard
    ) -> Bitboard {
        let mut attacks = EMPTY_BITBOARD;
        
        for direction in directions {
            let directional_attacks = self.generate_directional_attack(square, *direction, blockers);
            attacks |= directional_attacks;
        }
        
        attacks
    }

    /// Check if a square is attacked by a piece
    pub fn is_square_attacked(
        &mut self,
        from_square: u8,
        to_square: u8,
        piece_type: PieceType,
        blockers: Bitboard
    ) -> bool {
        let attacks = self.generate_attack_pattern(from_square, piece_type, blockers);
        is_bit_set(attacks, to_square)
    }

    /// Get all attacked squares for a piece
    pub fn get_attacked_squares(
        &mut self,
        square: u8,
        piece_type: PieceType,
        blockers: Bitboard
    ) -> Vec<u8> {
        let attacks = self.generate_attack_pattern(square, piece_type, blockers);
        (0..81).filter(|&i| is_bit_set(attacks, i)).collect()
    }

    /// Count the number of attacked squares
    pub fn count_attacked_squares(
        &mut self,
        square: u8,
        piece_type: PieceType,
        blockers: Bitboard
    ) -> u32 {
        let attacks = self.generate_attack_pattern(square, piece_type, blockers);
        attacks.count_ones()
    }

    /// Generate attack pattern for multiple pieces
    pub fn generate_combined_attacks(
        &mut self,
        squares: &[u8],
        piece_types: &[PieceType],
        blockers: Bitboard
    ) -> Bitboard {
        let mut combined_attacks = EMPTY_BITBOARD;
        
        for (square, piece_type) in squares.iter().zip(piece_types.iter()) {
            let attacks = self.generate_attack_pattern(*square, *piece_type, blockers);
            combined_attacks |= attacks;
        }
        
        combined_attacks
    }

    /// Pre-generate all attack patterns for a piece type
    pub fn pregenerate_attack_patterns(&mut self, piece_type: PieceType) {
        for square in 0..81 {
            let mask = self.get_relevant_mask(square, piece_type);
            let combinations = self.generate_all_blocker_combinations(mask);
            
            for blockers in combinations {
                self.generate_attack_pattern(square, piece_type, blockers);
            }
        }
    }

    /// Get attack pattern statistics
    pub fn get_attack_stats(&mut self, square: u8, piece_type: PieceType) -> AttackStats {
        let mask = self.get_relevant_mask(square, piece_type);
        let max_attacks = self.count_attacked_squares(square, piece_type, EMPTY_BITBOARD);
        let min_attacks = self.count_attacked_squares(square, piece_type, mask);
        
        AttackStats {
            square,
            piece_type,
            relevant_squares: mask.count_ones(),
            max_attacks,
            min_attacks,
            average_attacks: (max_attacks + min_attacks) / 2,
        }
    }

    /// Clear the pattern cache
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cache_size: self.pattern_cache.len(),
            direction_cache_size: self.direction_cache.len(),
        }
    }
}

/// Cache statistics for attack generator
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_size: usize,
    pub direction_cache_size: usize,
}

/// Attack pattern statistics
#[derive(Debug, Clone)]
pub struct AttackStats {
    pub square: u8,
    pub piece_type: PieceType,
    pub relevant_squares: u32,
    pub max_attacks: u32,
    pub min_attacks: u32,
    pub average_attacks: u32,
}

impl Default for AttackGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for bitboard operations
fn set_bit(bitboard: &mut Bitboard, square: u8) {
    *bitboard |= 1u128 << square;
}

fn is_bit_set(bitboard: Bitboard, square: u8) -> bool {
    (bitboard & (1u128 << square)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_generator_creation() {
        let generator = AttackGenerator::new();
        let stats = generator.cache_stats();
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.direction_cache_size, 4); // Rook, Bishop, PromotedRook, PromotedBishop
    }

    #[test]
    fn test_direction_cache() {
        let generator = AttackGenerator::new();
        
        let rook_directions = generator.get_directions(PieceType::Rook);
        assert_eq!(rook_directions.len(), 4);
        
        let bishop_directions = generator.get_directions(PieceType::Bishop);
        assert_eq!(bishop_directions.len(), 4);
    }

    #[test]
    fn test_get_next_square() {
        let generator = AttackGenerator::new();
        
        // Test moving right from square 0 (top-left corner)
        let direction = Direction { row_delta: 0, col_delta: 1 };
        assert_eq!(generator.get_next_square(0, direction), Some(1));
        
        // Test moving down from square 0
        let direction = Direction { row_delta: 1, col_delta: 0 };
        assert_eq!(generator.get_next_square(0, direction), Some(9));
        
        // Test moving out of bounds
        let direction = Direction { row_delta: -1, col_delta: 0 };
        assert_eq!(generator.get_next_square(0, direction), None);
    }

    #[test]
    fn test_generate_all_blocker_combinations() {
        let generator = AttackGenerator::new();
        let mask = 0b111; // First 3 squares
        let combinations = generator.generate_all_blocker_combinations(mask);
        
        assert_eq!(combinations.len(), 8); // 2^3 = 8 combinations
    }

    #[test]
    fn test_attack_pattern_generation() {
        let mut generator = AttackGenerator::new();
        
        // Test rook attack from center square (40)
        let attacks = generator.generate_attack_pattern_no_blockers(40, PieceType::Rook);
        assert_ne!(attacks, EMPTY_BITBOARD);
        
        // Test bishop attack from center square (40)
        let attacks = generator.generate_attack_pattern_no_blockers(40, PieceType::Bishop);
        assert_ne!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_relevant_mask_generation() {
        let generator = AttackGenerator::new();
        
        // Test rook mask from center square
        let mask = generator.get_relevant_mask(40, PieceType::Rook);
        assert_ne!(mask, EMPTY_BITBOARD);
        
        // Test bishop mask from center square
        let mask = generator.get_relevant_mask(40, PieceType::Bishop);
        assert_ne!(mask, EMPTY_BITBOARD);
    }

    #[test]
    fn test_directional_attack() {
        let generator = AttackGenerator::new();
        
        // Test moving right from square 0
        let direction = Direction { row_delta: 0, col_delta: 1 };
        let attacks = generator.generate_directional_attack(0, direction, EMPTY_BITBOARD);
        assert_ne!(attacks, EMPTY_BITBOARD);
        
        // Test with blockers
        let blockers = 1u128 << 2; // Block square 2
        let attacks = generator.generate_directional_attack(0, direction, blockers);
        assert_ne!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_square_attack_check() {
        let mut generator = AttackGenerator::new();
        
        // Test if square 1 is attacked by rook from square 0
        let is_attacked = generator.is_square_attacked(0, 1, PieceType::Rook, EMPTY_BITBOARD);
        assert!(is_attacked);
        
        // Test if square 9 is attacked by rook from square 0
        let is_attacked = generator.is_square_attacked(0, 9, PieceType::Rook, EMPTY_BITBOARD);
        assert!(is_attacked);
    }

    #[test]
    fn test_attacked_squares() {
        let mut generator = AttackGenerator::new();
        
        // Get all squares attacked by rook from square 0
        let attacked_squares = generator.get_attacked_squares(0, PieceType::Rook, EMPTY_BITBOARD);
        assert!(!attacked_squares.is_empty());
        assert!(attacked_squares.contains(&1)); // Right
        assert!(attacked_squares.contains(&9)); // Down
    }

    #[test]
    fn test_attack_count() {
        let mut generator = AttackGenerator::new();
        
        // Count attacks from corner square (should be fewer)
        let count = generator.count_attacked_squares(0, PieceType::Rook, EMPTY_BITBOARD);
        assert!(count > 0);
        
        // Count attacks from center square (should be more)
        let center_count = generator.count_attacked_squares(40, PieceType::Rook, EMPTY_BITBOARD);
        assert!(center_count > count);
    }

    #[test]
    fn test_combined_attacks() {
        let mut generator = AttackGenerator::new();
        
        let squares = vec![0, 1];
        let piece_types = vec![PieceType::Rook, PieceType::Bishop];
        let combined = generator.generate_combined_attacks(&squares, &piece_types, EMPTY_BITBOARD);
        
        assert_ne!(combined, EMPTY_BITBOARD);
    }

    #[test]
    fn test_attack_with_directions() {
        let mut generator = AttackGenerator::new();
        
        let directions = vec![
            Direction { row_delta: 0, col_delta: 1 },
            Direction { row_delta: 1, col_delta: 0 },
        ];
        
        let attacks = generator.generate_attack_with_directions(0, &directions, EMPTY_BITBOARD);
        assert_ne!(attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_attack_stats() {
        let mut generator = AttackGenerator::new();
        
        let stats = generator.get_attack_stats(40, PieceType::Rook);
        assert_eq!(stats.square, 40);
        assert_eq!(stats.piece_type, PieceType::Rook);
        assert!(stats.max_attacks > 0);
        assert!(stats.min_attacks >= 0);
        assert!(stats.relevant_squares > 0);
    }

    #[test]
    fn test_promoted_pieces() {
        let mut generator = AttackGenerator::new();
        
        // Test promoted rook (should have more directions)
        let promoted_rook_directions = generator.get_directions(PieceType::PromotedRook);
        assert_eq!(promoted_rook_directions.len(), 8);
        
        // Test promoted bishop (should have more directions)
        let promoted_bishop_directions = generator.get_directions(PieceType::PromotedBishop);
        assert_eq!(promoted_bishop_directions.len(), 8);
    }

    #[test]
    fn test_edge_cases() {
        let mut generator = AttackGenerator::new();
        
        // Test corner squares
        let corner_attacks = generator.generate_attack_pattern_no_blockers(0, PieceType::Rook);
        assert_ne!(corner_attacks, EMPTY_BITBOARD);
        
        // Test edge squares
        let edge_attacks = generator.generate_attack_pattern_no_blockers(4, PieceType::Bishop);
        assert_ne!(edge_attacks, EMPTY_BITBOARD);
        
        // Test center square
        let center_attacks = generator.generate_attack_pattern_no_blockers(40, PieceType::Rook);
        assert_ne!(center_attacks, EMPTY_BITBOARD);
    }

    #[test]
    fn test_blocker_combinations() {
        let generator = AttackGenerator::new();
        
        // Test with 2-bit mask
        let mask = 0b11u128;
        let combinations = generator.generate_all_blocker_combinations(mask);
        assert_eq!(combinations.len(), 4); // 2^2 = 4
        
        // Test with 3-bit mask
        let mask = 0b111u128;
        let combinations = generator.generate_all_blocker_combinations(mask);
        assert_eq!(combinations.len(), 8); // 2^3 = 8
    }

    #[test]
    fn test_cache_functionality() {
        let mut generator = AttackGenerator::new();
        
        // Generate some attack patterns
        generator.generate_attack_pattern(0, PieceType::Rook, EMPTY_BITBOARD);
        generator.generate_attack_pattern(1, PieceType::Bishop, EMPTY_BITBOARD);
        
        let stats = generator.cache_stats();
        assert!(stats.cache_size > 0);
        
        // Clear cache
        generator.clear_cache();
        let stats = generator.cache_stats();
        assert_eq!(stats.cache_size, 0);
    }
}
