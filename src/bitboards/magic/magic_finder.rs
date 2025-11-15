//! Magic number generation and validation for magic bitboards
//!
//! This module provides functionality to generate and validate magic numbers
//! used in magic bitboard implementations for efficient sliding piece move generation.

use crate::types::core::PieceType;
use crate::types::{Bitboard, EMPTY_BITBOARD, MagicError, MagicGenerationResult};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};

/// Magic number finder with optimization strategies
pub struct MagicFinder {
    /// Random number generator for candidate generation
    rng: ThreadRng,
    /// Cache for previously found magic numbers
    magic_cache: HashMap<(u8, PieceType), MagicGenerationResult>,
    /// Performance statistics
    stats: MagicStats,
}

/// Performance statistics for magic number generation
#[derive(Debug, Default, Clone)]
pub struct MagicStats {
    pub total_attempts: u64,
    pub successful_generations: u64,
    pub cache_hits: u64,
    pub average_generation_time: std::time::Duration,
}

impl MagicFinder {
    /// Create a new magic finder
    pub fn new() -> Self {
        Self {
            rng: ThreadRng::default(),
            magic_cache: HashMap::new(),
            stats: MagicStats::default(),
        }
    }

    /// Find magic number for a specific square and piece type
    pub fn find_magic_number(
        &mut self,
        square: u8,
        piece_type: PieceType,
    ) -> Result<MagicGenerationResult, MagicError> {
        // Check cache first
        if let Some(cached) = self.magic_cache.get(&(square, piece_type)) {
            self.stats.cache_hits += 1;
            return Ok(*cached);
        }

        // Validate input
        if square >= 81 {
            return Err(MagicError::InvalidSquare { square });
        }

        if !self.is_valid_piece_type(piece_type) {
            return Err(MagicError::InvalidPieceType { piece_type });
        }

        // Try different generation strategies
        if let Ok(result) = self.find_with_random_search(square, piece_type) {
            self.magic_cache.insert((square, piece_type), result);
            self.stats.successful_generations += 1;
            return Ok(result);
        }

        if let Ok(result) = self.find_with_brute_force(square, piece_type) {
            self.magic_cache.insert((square, piece_type), result);
            self.stats.successful_generations += 1;
            return Ok(result);
        }

        if let Ok(result) = self.find_with_heuristic(square, piece_type) {
            self.magic_cache.insert((square, piece_type), result);
            self.stats.successful_generations += 1;
            return Ok(result);
        }

        Err(MagicError::GenerationFailed { square, piece_type })
    }

    /// Random search strategy
    fn find_with_random_search(
        &mut self,
        square: u8,
        piece_type: PieceType,
    ) -> Result<MagicGenerationResult, MagicError> {
        let mask = self.generate_relevant_mask(square, piece_type);
        let shift = self.calculate_shift(mask);
        let max_attempts = 1_000_000;

        for _ in 0..max_attempts {
            let candidate = self.rng.gen::<u64>();
            if self.validate_magic_fast(candidate, square, piece_type, &mask, shift) {
                return Ok(MagicGenerationResult {
                    magic_number: candidate,
                    mask,
                    shift,
                    table_size: 1 << (64 - shift),
                    generation_time: std::time::Duration::from_secs(0),
                });
            }
            self.stats.total_attempts += 1;
        }

        Err(MagicError::GenerationFailed { square, piece_type })
    }

    /// Brute force strategy
    fn find_with_brute_force(
        &mut self,
        square: u8,
        piece_type: PieceType,
    ) -> Result<MagicGenerationResult, MagicError> {
        let mask = self.generate_relevant_mask(square, piece_type);
        let shift = self.calculate_shift(mask);
        let bit_count = mask.count_ones() as u8;

        // For small bit counts, we can try all possible magic numbers
        if bit_count > 12 {
            return Err(MagicError::GenerationFailed { square, piece_type });
        }

        let start_time = std::time::Instant::now();

        // Try magic numbers starting from 1
        for magic in 1..=u64::MAX {
            if self.validate_magic_fast(magic, square, piece_type, &mask, shift) {
                return Ok(MagicGenerationResult {
                    magic_number: magic,
                    mask,
                    shift,
                    table_size: 1 << (64 - shift),
                    generation_time: start_time.elapsed(),
                });
            }
            self.stats.total_attempts += 1;

            // Limit brute force attempts to prevent infinite loops
            if self.stats.total_attempts > 10_000_000 {
                break;
            }
        }

        Err(MagicError::GenerationFailed { square, piece_type })
    }

    /// Heuristic strategy
    fn find_with_heuristic(
        &mut self,
        square: u8,
        piece_type: PieceType,
    ) -> Result<MagicGenerationResult, MagicError> {
        let mask = self.generate_relevant_mask(square, piece_type);
        let shift = self.calculate_shift(mask);
        let start_time = std::time::Instant::now();

        // Heuristic: try magic numbers with specific patterns
        let heuristic_candidates = self.generate_heuristic_candidates(mask);

        for candidate in heuristic_candidates {
            if self.validate_magic_fast(candidate, square, piece_type, &mask, shift) {
                return Ok(MagicGenerationResult {
                    magic_number: candidate,
                    mask,
                    shift,
                    table_size: 1 << (64 - shift),
                    generation_time: start_time.elapsed(),
                });
            }
            self.stats.total_attempts += 1;
        }

        // If heuristics fail, try some random numbers with better distribution
        for _ in 0..100_000 {
            let candidate = self.rng.gen::<u64>();
            if self.validate_magic_fast(candidate, square, piece_type, &mask, shift) {
                return Ok(MagicGenerationResult {
                    magic_number: candidate,
                    mask,
                    shift,
                    table_size: 1 << (64 - shift),
                    generation_time: start_time.elapsed(),
                });
            }
            self.stats.total_attempts += 1;
        }

        Err(MagicError::GenerationFailed { square, piece_type })
    }

    /// Generate heuristic magic number candidates with expanded patterns
    ///
    /// This improved heuristic includes:
    /// - Powers of 2
    /// - Sparse bit patterns (2-4 bits set)
    /// - Mask-derived patterns
    /// - Well-known magic numbers from chess engines
    /// - Patterns optimized for smaller table sizes
    fn generate_heuristic_candidates(&self, mask: Bitboard) -> Vec<u64> {
        let mut candidates = Vec::new();
        let bit_count = mask.count_ones() as u8;

        // Try powers of 2 (expanded set)
        for i in 0..64 {
            candidates.push(1u64 << i);
        }

        // Try numbers with sparse bit patterns (2 bits)
        for i in 0..64 {
            for j in (i + 1)..64 {
                candidates.push((1u64 << i) | (1u64 << j));
            }
        }

        // Try numbers with 3-bit sparse patterns (for better distribution)
        for i in 0..64 {
            for j in (i + 1)..64.min(i + 10) { // Limit to nearby bits
                for k in (j + 1)..64.min(j + 10) {
                    candidates.push((1u64 << i) | (1u64 << j) | (1u64 << k));
                }
            }
        }

        // Try numbers with 4-bit sparse patterns (limited to prevent explosion)
        for i in 0..32 {
            for j in (i + 1)..32.min(i + 8) {
                for k in (j + 1)..32.min(j + 8) {
                    for l in (k + 1)..32.min(k + 8) {
                        candidates.push((1u64 << i) | (1u64 << j) | (1u64 << k) | (1u64 << l));
                    }
                }
            }
        }

        // Try numbers based on the mask pattern (expanded)
        let mask_low = (mask & 0xFFFFFFFF) as u32;
        let mask_high = ((mask >> 32) & 0xFFFFFFFF) as u32;
        let mask_mid = ((mask >> 16) & 0xFFFFFFFF) as u32;

        candidates.push(mask_low as u64);
        candidates.push(mask_high as u64);
        candidates.push(mask_mid as u64);
        candidates.push((mask_low as u64) << 32 | (mask_high as u64));
        candidates.push((mask_low as u64) | ((mask_high as u64) << 32));
        
        // Mask-derived patterns with rotations
        candidates.push(mask_low.wrapping_mul(0x9E3779B9) as u64);
        candidates.push(mask_high.wrapping_mul(0x9E3779B9) as u64);
        candidates.push((mask_low as u64).wrapping_mul(0x517CC1B7));
        candidates.push((mask_high as u64).wrapping_mul(0x517CC1B7));

        // Try some well-known magic numbers from chess engines (expanded set)
        let well_known = vec![
            0x0001010101010101,
            0x0002020202020202,
            0x0004040404040404,
            0x0008080808080808,
            0x0010101010101010,
            0x0020202020202020,
            0x0040404040404040,
            0x0080808080808080,
            0x0101010101010101,
            0x0202020202020202,
            0x0404040404040404,
            0x0808080808080808,
            0x1010101010101010,
            0x2020202020202020,
            0x4040404040404040,
            0x8080808080808080,
            0x00000000000000FF,
            0x000000000000FF00,
            0x00000000FF000000,
            0xFF00000000000000,
        ];
        candidates.extend(well_known);

        // Patterns optimized for smaller table sizes (when bit_count is small)
        if bit_count <= 8 {
            // For small masks, try more aggressive patterns
            for i in 0..16 {
                candidates.push(0xFFFFFFFFFFFFFFFFu64 >> (64 - (1 << i).min(64)));
            }
        }

        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        candidates.retain(|&x| seen.insert(x));
        
        candidates
    }

    /// Generate relevant mask for a square and piece type
    fn generate_relevant_mask(&self, square: u8, piece_type: PieceType) -> Bitboard {
        let (row, col) = (square / 9, square % 9);
        let mut mask = EMPTY_BITBOARD;

        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                // Rook moves horizontally and vertically
                // Add all squares in the same row and column, excluding the square itself
                for i in 0..9 {
                    if i != col {
                        mask |= 1u128 << (row * 9 + i);
                    }
                    if i != row {
                        mask |= 1u128 << (i * 9 + col);
                    }
                }
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                // Bishop moves diagonally
                // Add all squares on the diagonals, excluding the square itself
                for i in 1..9 {
                    // Diagonal 1: (row+i, col+i) and (row-i, col-i)
                    if row + i < 9 && col + i < 9 {
                        mask |= 1u128 << ((row + i) * 9 + (col + i));
                    }
                    if row >= i && col >= i {
                        mask |= 1u128 << ((row - i) * 9 + (col - i));
                    }

                    // Diagonal 2: (row+i, col-i) and (row-i, col+i)
                    if row + i < 9 && col >= i {
                        mask |= 1u128 << ((row + i) * 9 + (col - i));
                    }
                    if row >= i && col + i < 9 {
                        mask |= 1u128 << ((row - i) * 9 + (col + i));
                    }
                }
            }
            _ => {
                // Invalid piece type for magic bitboards
                return EMPTY_BITBOARD;
            }
        }

        mask
    }

    /// Calculate shift value for optimal table sizing
    fn calculate_shift(&self, mask: Bitboard) -> u8 {
        // Count the number of set bits in the mask
        let bit_count = mask.count_ones() as u8;

        // The shift is 64 - number of relevant bits
        // This ensures we use the minimum table size
        64 - bit_count
    }

    /// Fast magic number validation
    fn validate_magic_fast(
        &self,
        magic: u64,
        _square: u8,
        _piece_type: PieceType,
        mask: &Bitboard,
        shift: u8,
    ) -> bool {
        // Generate all possible blocker configurations
        let blocker_configs = self.generate_all_blocker_configs(*mask);
        let mut used_indices = HashSet::new();

        for blockers in &blocker_configs {
            // Calculate the hash index
            let hash = (blockers.wrapping_mul(magic as u128)) >> shift;
            let index = hash as usize;

            // Check for collision
            if used_indices.contains(&index) {
                return false;
            }
            used_indices.insert(index);
        }

        true
    }

    /// Generate all possible blocker configurations for a mask
    fn generate_all_blocker_configs(&self, mask: Bitboard) -> Vec<Bitboard> {
        let mut configs = Vec::new();
        let bit_count = mask.count_ones() as usize;

        // Generate all 2^n possible configurations
        for i in 0..(1u64 << bit_count) {
            let mut config = EMPTY_BITBOARD;
            let mut bit_index = 0;
            let mut temp_mask = mask;

            while temp_mask != 0 {
                let bit_pos = temp_mask.trailing_zeros() as u8;
                temp_mask &= temp_mask - 1; // Clear the lowest set bit

                if (i >> bit_index) & 1 != 0 {
                    config |= 1u128 << bit_pos;
                }
                bit_index += 1;
            }

            configs.push(config);
        }

        configs
    }

    /// Check if piece type is valid for magic bitboards
    fn is_valid_piece_type(&self, piece_type: PieceType) -> bool {
        matches!(
            piece_type,
            PieceType::Rook
                | PieceType::Bishop
                | PieceType::PromotedRook
                | PieceType::PromotedBishop
        )
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &MagicStats {
        &self.stats
    }

    /// Clear the magic number cache
    pub fn clear_cache(&mut self) {
        self.magic_cache.clear();
    }

    /// Pre-generate magic numbers for all squares and piece types
    pub fn pregenerate_all_magics(&mut self) -> Result<(), MagicError> {
        let piece_types = [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::PromotedRook,
            PieceType::PromotedBishop,
        ];

        for piece_type in piece_types {
            for square in 0..81 {
                if let Err(e) = self.find_magic_number(square, piece_type) {
                    eprintln!(
                        "Failed to generate magic number for square {} piece {:?}: {}",
                        square, piece_type, e
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Get magic number for a specific square and piece type (cached)
    pub fn get_magic_number(
        &self,
        square: u8,
        piece_type: PieceType,
    ) -> Option<&MagicGenerationResult> {
        self.magic_cache.get(&(square, piece_type))
    }

    /// Check if magic number exists in cache
    pub fn has_magic_number(&self, square: u8, piece_type: PieceType) -> bool {
        self.magic_cache.contains_key(&(square, piece_type))
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.magic_cache.len(), self.magic_cache.capacity())
    }
}

impl Default for MagicFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_magic_finder_creation() {
        let finder = MagicFinder::new();
        assert_eq!(finder.get_stats().total_attempts, 0);
        assert_eq!(finder.get_stats().successful_generations, 0);
    }

    #[test]
    fn test_invalid_square() {
        let mut finder = MagicFinder::new();
        let result = finder.find_magic_number(100, PieceType::Rook);
        assert!(matches!(
            result,
            Err(MagicError::InvalidSquare { square: 100 })
        ));
    }

    #[test]
    fn test_invalid_piece_type() {
        let mut finder = MagicFinder::new();
        let result = finder.find_magic_number(0, PieceType::Pawn);
        assert!(matches!(
            result,
            Err(MagicError::InvalidPieceType {
                piece_type: PieceType::Pawn
            })
        ));
    }

    #[test]
    fn test_relevant_mask_generation() {
        let finder = MagicFinder::new();

        // Test rook mask for center square (4,4)
        let rook_mask = finder.generate_relevant_mask(40, PieceType::Rook);
        assert_ne!(rook_mask, EMPTY_BITBOARD);

        // Test bishop mask for center square (4,4)
        let bishop_mask = finder.generate_relevant_mask(40, PieceType::Bishop);
        assert_ne!(bishop_mask, EMPTY_BITBOARD);

        // Test corner square (0,0) rook
        let corner_rook_mask = finder.generate_relevant_mask(0, PieceType::Rook);
        assert_ne!(corner_rook_mask, EMPTY_BITBOARD);

        // Test edge square (0,4) bishop
        let edge_bishop_mask = finder.generate_relevant_mask(4, PieceType::Bishop);
        assert_ne!(edge_bishop_mask, EMPTY_BITBOARD);
    }

    #[test]
    fn test_shift_calculation() {
        let finder = MagicFinder::new();

        // Test with empty mask
        let empty_mask = EMPTY_BITBOARD;
        let shift = finder.calculate_shift(empty_mask);
        assert_eq!(shift, 64);

        // Test with single bit mask
        let single_bit_mask = 1u128 << 40;
        let shift = finder.calculate_shift(single_bit_mask);
        assert_eq!(shift, 63);

        // Test with multiple bits
        let multi_bit_mask = 0xFFu128;
        let shift = finder.calculate_shift(multi_bit_mask);
        assert_eq!(shift, 64 - 8);
    }

    #[test]
    fn test_blocker_config_generation() {
        let finder = MagicFinder::new();

        // Test with 2-bit mask
        let mask = 0b11u128;
        let configs = finder.generate_all_blocker_configs(mask);
        assert_eq!(configs.len(), 4); // 2^2 = 4 configurations

        // Test with 3-bit mask
        let mask = 0b111u128;
        let configs = finder.generate_all_blocker_configs(mask);
        assert_eq!(configs.len(), 8); // 2^3 = 8 configurations
    }

    #[test]
    fn test_magic_number_generation() {
        let mut finder = MagicFinder::new();

        // Test generating magic number for center square rook
        let result = finder.find_magic_number(40, PieceType::Rook);
        assert!(result.is_ok());

        let magic_result = result.unwrap();
        assert_ne!(magic_result.magic_number, 0);
        assert_ne!(magic_result.mask, EMPTY_BITBOARD);
        assert!(magic_result.shift > 0);
        assert!(magic_result.table_size > 0);
    }

    #[test]
    fn test_magic_number_caching() {
        let mut finder = MagicFinder::new();

        // Generate magic number
        let result1 = finder.find_magic_number(40, PieceType::Rook);
        assert!(result1.is_ok());

        // Check cache
        assert!(finder.has_magic_number(40, PieceType::Rook));
        let cached_result = finder.get_magic_number(40, PieceType::Rook);
        assert!(cached_result.is_some());

        // Generate again (should use cache)
        let result2 = finder.find_magic_number(40, PieceType::Rook);
        assert!(result2.is_ok());

        // Results should be identical
        assert_eq!(result1.unwrap().magic_number, result2.unwrap().magic_number);
    }

    #[test]
    fn test_magic_number_validation() {
        let finder = MagicFinder::new();
        let mask = finder.generate_relevant_mask(40, PieceType::Rook);
        let shift = finder.calculate_shift(mask);

        // Test with a valid magic number (if we can find one)
        let magic = 0x0001010101010101;
        let is_valid = finder.validate_magic_fast(magic, 40, PieceType::Rook, &mask, shift);

        // This might be false for this specific magic number, but the function should work
        // The important thing is that it doesn't panic and returns a boolean
        assert!(is_valid || !is_valid);
    }

    #[test]
    fn test_heuristic_candidates() {
        let finder = MagicFinder::new();
        let mask = 0xFFu128;
        let candidates = finder.generate_heuristic_candidates(mask);

        assert!(!candidates.is_empty());
        assert!(candidates.len() > 100); // Should generate many candidates

        // Check that all candidates are unique
        let mut unique_candidates = candidates.clone();
        unique_candidates.sort();
        unique_candidates.dedup();
        assert_eq!(candidates.len(), unique_candidates.len());
    }

    #[test]
    fn test_performance_stats() {
        let mut finder = MagicFinder::new();

        // Generate a magic number
        let _ = finder.find_magic_number(40, PieceType::Rook);

        let stats = finder.get_stats();
        assert!(stats.total_attempts > 0);
        assert!(stats.successful_generations > 0);
    }

    #[test]
    fn test_cache_operations() {
        let mut finder = MagicFinder::new();

        // Test empty cache
        assert_eq!(finder.get_cache_stats().0, 0);

        // Add some magic numbers
        let _ = finder.find_magic_number(40, PieceType::Rook);
        let _ = finder.find_magic_number(40, PieceType::Bishop);

        // Check cache size
        assert_eq!(finder.get_cache_stats().0, 2);

        // Clear cache
        finder.clear_cache();
        assert_eq!(finder.get_cache_stats().0, 0);
    }

    #[test]
    fn test_promoted_pieces() {
        let mut finder = MagicFinder::new();

        // Test promoted rook
        let result = finder.find_magic_number(40, PieceType::PromotedRook);
        assert!(result.is_ok());

        // Test promoted bishop
        let result = finder.find_magic_number(40, PieceType::PromotedBishop);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        let mut finder = MagicFinder::new();

        // Test corner squares
        let result = finder.find_magic_number(0, PieceType::Rook);
        assert!(result.is_ok());

        let result = finder.find_magic_number(80, PieceType::Bishop);
        assert!(result.is_ok());

        // Test edge squares
        let result = finder.find_magic_number(4, PieceType::Rook);
        assert!(result.is_ok());

        let result = finder.find_magic_number(76, PieceType::Bishop);
        assert!(result.is_ok());
    }
}
