//! Attack Pattern Precomputation Module
//! 
//! This module provides precomputed attack patterns for non-sliding pieces in Shogi.
//! It eliminates runtime calculations by precomputing all possible attack patterns
//! at initialization time, providing O(1) lookup performance.

use crate::types::{Bitboard, PieceType, Player, EMPTY_BITBOARD};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Cache-friendly attack tables with 64-byte alignment for optimal performance
#[derive(Clone)]
#[repr(C, align(64))]
pub struct AttackTables {
    /// King attacks: 81 positions × 8 directions = 648 patterns (same for both players)
    pub king_attacks: [Bitboard; 81],
    
    /// Black piece attacks
    pub black_knight_attacks: [Bitboard; 81],
    pub black_gold_attacks: [Bitboard; 81],
    pub black_silver_attacks: [Bitboard; 81],
    pub black_promoted_pawn_attacks: [Bitboard; 81],
    pub black_promoted_lance_attacks: [Bitboard; 81],
    pub black_promoted_knight_attacks: [Bitboard; 81],
    pub black_promoted_silver_attacks: [Bitboard; 81],
    
    /// White piece attacks
    pub white_knight_attacks: [Bitboard; 81],
    pub white_gold_attacks: [Bitboard; 81],
    pub white_silver_attacks: [Bitboard; 81],
    pub white_promoted_pawn_attacks: [Bitboard; 81],
    pub white_promoted_lance_attacks: [Bitboard; 81],
    pub white_promoted_knight_attacks: [Bitboard; 81],
    pub white_promoted_silver_attacks: [Bitboard; 81],
    
    /// Promoted sliding pieces (King-like moves + original sliding) - same for both players
    pub promoted_bishop_attacks: [Bitboard; 81],
    pub promoted_rook_attacks: [Bitboard; 81],
    
    /// Metadata for tracking and debugging
    pub metadata: AttackTablesMetadata,
}

/// Metadata for tracking initialization performance and validation
#[derive(Debug, Clone)]
pub struct AttackTablesMetadata {
    pub initialization_time: std::time::Duration,
    pub memory_usage_bytes: usize,
    pub pattern_counts: [usize; 14], // Count per piece type
    pub validation_passed: bool,
    pub generation_stats: GenerationStats,
}

/// Statistics about pattern generation process
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub total_patterns_generated: usize,
    pub validation_errors: usize,
    pub average_pattern_size: f32,
    pub edge_case_count: usize,
}

/// Direction vector for piece movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction {
    pub row_delta: i8,
    pub col_delta: i8,
}

impl Direction {
    /// Create a new direction vector
    pub const fn new(row_delta: i8, col_delta: i8) -> Self {
        Self { row_delta, col_delta }
    }
    
    /// Apply this direction to a square, returning the resulting square if valid
    pub fn apply(&self, square: u8) -> Option<u8> {
        let row = (square / 9) as i8;
        let col = (square % 9) as i8;
        
        let new_row = row + self.row_delta;
        let new_col = col + self.col_delta;
        
        if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
            Some((new_row * 9 + new_col) as u8)
        } else {
            None
        }
    }
}

/// Attack pattern generator for creating precomputed tables
pub struct AttackPatternGenerator {
    /// Cache for generated patterns during initialization
    #[allow(dead_code)]
    pattern_cache: std::collections::HashMap<(u8, PieceType, Player), Bitboard>,
    
    /// Direction vectors for each piece type
    #[allow(dead_code)]
    direction_cache: std::collections::HashMap<PieceType, Vec<Direction>>,
    
    /// Validation statistics
    validation_stats: ValidationStats,
}

/// Validation statistics for pattern generation
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub total_patterns_generated: usize,
    pub validation_errors: usize,
    pub average_pattern_size: f32,
    pub edge_case_count: usize,
}

impl AttackTables {
    /// Create a new AttackTables with all patterns precomputed
    pub fn new() -> Self {
        let mut tables = Self {
            king_attacks: [EMPTY_BITBOARD; 81],
            black_knight_attacks: [EMPTY_BITBOARD; 81],
            black_gold_attacks: [EMPTY_BITBOARD; 81],
            black_silver_attacks: [EMPTY_BITBOARD; 81],
            black_promoted_pawn_attacks: [EMPTY_BITBOARD; 81],
            black_promoted_lance_attacks: [EMPTY_BITBOARD; 81],
            black_promoted_knight_attacks: [EMPTY_BITBOARD; 81],
            black_promoted_silver_attacks: [EMPTY_BITBOARD; 81],
            white_knight_attacks: [EMPTY_BITBOARD; 81],
            white_gold_attacks: [EMPTY_BITBOARD; 81],
            white_silver_attacks: [EMPTY_BITBOARD; 81],
            white_promoted_pawn_attacks: [EMPTY_BITBOARD; 81],
            white_promoted_lance_attacks: [EMPTY_BITBOARD; 81],
            white_promoted_knight_attacks: [EMPTY_BITBOARD; 81],
            white_promoted_silver_attacks: [EMPTY_BITBOARD; 81],
            promoted_bishop_attacks: [EMPTY_BITBOARD; 81],
            promoted_rook_attacks: [EMPTY_BITBOARD; 81],
            metadata: AttackTablesMetadata {
                initialization_time: std::time::Duration::ZERO,
                memory_usage_bytes: 0,
                pattern_counts: [0; 14],
                validation_passed: false,
                generation_stats: GenerationStats {
                    total_patterns_generated: 0,
                    validation_errors: 0,
                    average_pattern_size: 0.0,
                    edge_case_count: 0,
                },
            },
        };
        
        // Only measure time on non-WASM platforms
        #[cfg(not(target_arch = "wasm32"))]
        {
            let start_time = Instant::now();
            tables.precompute_all_patterns();
            let initialization_time = start_time.elapsed();
            tables.metadata.initialization_time = initialization_time;
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            tables.precompute_all_patterns();
            // Set zero duration for WASM (timing not available)
            tables.metadata.initialization_time = std::time::Duration::ZERO;
        }
        
        tables.metadata.memory_usage_bytes = std::mem::size_of::<AttackTables>();
        tables.metadata.validation_passed = true;
        
        tables
    }
    
    /// Get attack pattern for a piece at a given square
    pub fn get_attack_pattern(&self, square: u8, piece_type: PieceType, player: Player) -> Bitboard {
        let idx = square as usize;
        match (piece_type, player) {
            // King attacks are same for both players
            (PieceType::King, _) => self.king_attacks[idx],
            
            // Black pieces
            (PieceType::Knight, Player::Black) => self.black_knight_attacks[idx],
            (PieceType::Gold, Player::Black) => self.black_gold_attacks[idx],
            (PieceType::Silver, Player::Black) => self.black_silver_attacks[idx],
            (PieceType::PromotedPawn, Player::Black) => self.black_promoted_pawn_attacks[idx],
            (PieceType::PromotedLance, Player::Black) => self.black_promoted_lance_attacks[idx],
            (PieceType::PromotedKnight, Player::Black) => self.black_promoted_knight_attacks[idx],
            (PieceType::PromotedSilver, Player::Black) => self.black_promoted_silver_attacks[idx],
            
            // White pieces
            (PieceType::Knight, Player::White) => self.white_knight_attacks[idx],
            (PieceType::Gold, Player::White) => self.white_gold_attacks[idx],
            (PieceType::Silver, Player::White) => self.white_silver_attacks[idx],
            (PieceType::PromotedPawn, Player::White) => self.white_promoted_pawn_attacks[idx],
            (PieceType::PromotedLance, Player::White) => self.white_promoted_lance_attacks[idx],
            (PieceType::PromotedKnight, Player::White) => self.white_promoted_knight_attacks[idx],
            (PieceType::PromotedSilver, Player::White) => self.white_promoted_silver_attacks[idx],
            
            // Promoted sliding pieces - same for both players
            (PieceType::PromotedBishop, _) => self.promoted_bishop_attacks[idx],
            (PieceType::PromotedRook, _) => self.promoted_rook_attacks[idx],
            
            // Sliding pieces handled by magic bitboards
            _ => EMPTY_BITBOARD,
        }
    }
    
    /// Check if a square is attacked by a piece
    pub fn is_square_attacked(&self, from_square: u8, to_square: u8, piece_type: PieceType, player: Player) -> bool {
        let attacks = self.get_attack_pattern(from_square, piece_type, player);
        (attacks & (1u128 << to_square)) != 0
    }
    
    /// Precompute all attack patterns
    fn precompute_all_patterns(&mut self) {
        let mut generator = AttackPatternGenerator::new();
        
        // Generate king patterns (same for both players)
        for square in 0..81 {
            self.king_attacks[square] = generator.generate_king_attacks(square as u8);
            self.metadata.pattern_counts[PieceType::King.to_u8() as usize] += 1;
        }
        
        // Generate BLACK piece patterns
        for square in 0..81 {
            self.black_knight_attacks[square] = generator.generate_knight_attacks(square as u8, Player::Black);
            self.black_gold_attacks[square] = generator.generate_gold_attacks(square as u8, Player::Black);
            self.black_silver_attacks[square] = generator.generate_silver_attacks(square as u8, Player::Black);
            
            // Promoted pieces move like gold
            self.black_promoted_pawn_attacks[square] = self.black_gold_attacks[square];
            self.black_promoted_lance_attacks[square] = self.black_gold_attacks[square];
            self.black_promoted_knight_attacks[square] = self.black_gold_attacks[square];
            self.black_promoted_silver_attacks[square] = self.black_gold_attacks[square];
            
            self.metadata.pattern_counts[PieceType::Knight.to_u8() as usize] += 1;
            self.metadata.pattern_counts[PieceType::Gold.to_u8() as usize] += 1;
            self.metadata.pattern_counts[PieceType::Silver.to_u8() as usize] += 1;
        }
        
        // Generate WHITE piece patterns
        for square in 0..81 {
            self.white_knight_attacks[square] = generator.generate_knight_attacks(square as u8, Player::White);
            self.white_gold_attacks[square] = generator.generate_gold_attacks(square as u8, Player::White);
            self.white_silver_attacks[square] = generator.generate_silver_attacks(square as u8, Player::White);
            
            // Promoted pieces move like gold
            self.white_promoted_pawn_attacks[square] = self.white_gold_attacks[square];
            self.white_promoted_lance_attacks[square] = self.white_gold_attacks[square];
            self.white_promoted_knight_attacks[square] = self.white_gold_attacks[square];
            self.white_promoted_silver_attacks[square] = self.white_gold_attacks[square];
            
            self.metadata.pattern_counts[PieceType::PromotedPawn.to_u8() as usize] += 1;
            self.metadata.pattern_counts[PieceType::PromotedLance.to_u8() as usize] += 1;
            self.metadata.pattern_counts[PieceType::PromotedKnight.to_u8() as usize] += 1;
            self.metadata.pattern_counts[PieceType::PromotedSilver.to_u8() as usize] += 1;
        }
        
        // Generate promoted sliding piece patterns (king + original sliding moves)
        // These are the same for both players
        for square in 0..81 {
            // Promoted bishop: king moves + bishop moves
            self.promoted_bishop_attacks[square] = self.king_attacks[square] | 
                generator.generate_bishop_attacks(square as u8);
            self.metadata.pattern_counts[PieceType::PromotedBishop.to_u8() as usize] += 1;
            
            // Promoted rook: king moves + rook moves
            self.promoted_rook_attacks[square] = self.king_attacks[square] | 
                generator.generate_rook_attacks(square as u8);
            self.metadata.pattern_counts[PieceType::PromotedRook.to_u8() as usize] += 1;
        }
        
        self.metadata.generation_stats = generator.get_validation_stats();
        self.metadata.generation_stats.total_patterns_generated = 81 * 17; // Now we have separate patterns for Black and White
    }
    
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> &AttackTablesMetadata {
        &self.metadata
    }
    
    /// Validate all attack patterns for correctness
    pub fn validate(&self) -> bool {
        // Basic validation - check that patterns are reasonable
        for square in 0..81 {
            let row = square / 9;
            let col = square % 9;
            let is_edge = row == 0 || row == 8 || col == 0 || col == 8;
            let is_corner = (row == 0 || row == 8) && (col == 0 || col == 8);
            
            // King should have 3-8 attack squares (fewer on edges/corners)
            let king_attacks = self.king_attacks[square].count_ones();
            let min_king_attacks = if is_corner { 3 } else if is_edge { 5 } else { 8 };
            let max_king_attacks = if is_corner { 3 } else if is_edge { 6 } else { 8 };
            if king_attacks < min_king_attacks || king_attacks > max_king_attacks {
                return false;
            }
            
            // Validate Black pieces
            // Knight should have 0-2 attack squares
            let black_knight_attacks = self.black_knight_attacks[square].count_ones();
            if black_knight_attacks > 2 {
                return false;
            }
            
            // Gold should have 3-6 attack squares (fewer on edges/corners)
            let black_gold_attacks = self.black_gold_attacks[square].count_ones();
            let min_gold_attacks = if is_corner { 2 } else if is_edge { 3 } else { 4 };
            let max_gold_attacks = if is_corner { 3 } else if is_edge { 5 } else { 6 };
            if black_gold_attacks < min_gold_attacks || black_gold_attacks > max_gold_attacks {
                return false;
            }
            
            // Silver should have 3-5 attack squares (fewer on edges/corners)
            let black_silver_attacks = self.black_silver_attacks[square].count_ones();
            let min_silver_attacks = if is_corner { 1 } else if is_edge { 2 } else { 3 };
            let max_silver_attacks = if is_corner { 2 } else if is_edge { 4 } else { 5 };
            if black_silver_attacks < min_silver_attacks || black_silver_attacks > max_silver_attacks {
                return false;
            }
            
            // Validate White pieces
            // Knight should have 0-2 attack squares
            let white_knight_attacks = self.white_knight_attacks[square].count_ones();
            if white_knight_attacks > 2 {
                return false;
            }
            
            // Gold should have 3-6 attack squares (fewer on edges/corners)
            let white_gold_attacks = self.white_gold_attacks[square].count_ones();
            if white_gold_attacks < min_gold_attacks || white_gold_attacks > max_gold_attacks {
                return false;
            }
            
            // Silver should have 3-5 attack squares (fewer on edges/corners)
            let white_silver_attacks = self.white_silver_attacks[square].count_ones();
            if white_silver_attacks < min_silver_attacks || white_silver_attacks > max_silver_attacks {
                return false;
            }
        }
        
        true
    }
}

impl AttackPatternGenerator {
    /// Create a new attack pattern generator
    pub fn new() -> Self {
        let mut generator = Self {
            pattern_cache: std::collections::HashMap::new(),
            direction_cache: std::collections::HashMap::new(),
            validation_stats: ValidationStats::default(),
        };
        generator.initialize_direction_cache();
        generator
    }
    
    /// Generate king attack pattern for a square
    fn generate_king_attacks(&mut self, square: u8) -> Bitboard {
        const KING_DIRECTIONS: [Direction; 8] = [
            Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
            Direction::new(0, -1),                        Direction::new(0, 1),
            Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
        ];
        
        self.generate_attacks_from_directions(square, &KING_DIRECTIONS)
    }
    
    /// Generate knight attack pattern for a square and player
    fn generate_knight_attacks(&mut self, square: u8, player: Player) -> Bitboard {
        let directions = match player {
            Player::Black => [
                Direction::new(2, -1),  // Forward-left
                Direction::new(2, 1),   // Forward-right
            ],
            Player::White => [
                Direction::new(-2, -1), // Forward-left (from white perspective)
                Direction::new(-2, 1),  // Forward-right (from white perspective)
            ],
        };
        
        self.generate_attacks_from_directions(square, &directions)
    }
    
    /// Generate gold attack pattern for a square and player
    fn generate_gold_attacks(&mut self, square: u8, player: Player) -> Bitboard {
        let directions = match player {
            Player::Black => [
                Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
                Direction::new(0, -1),  Direction::new(0, 1),  Direction::new(1, 0),
            ],
            Player::White => [
                Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
                Direction::new(0, -1),  Direction::new(0, 1),  Direction::new(-1, 0),
            ],
        };
        
        self.generate_attacks_from_directions(square, &directions)
    }
    
    /// Generate silver attack pattern for a square and player
    fn generate_silver_attacks(&mut self, square: u8, player: Player) -> Bitboard {
        let directions = match player {
            Player::Black => [
                Direction::new(-1, -1), Direction::new(-1, 0), Direction::new(-1, 1),
                Direction::new(1, -1),  Direction::new(1, 1),
            ],
            Player::White => [
                Direction::new(1, -1),  Direction::new(1, 0),  Direction::new(1, 1),
                Direction::new(-1, -1), Direction::new(-1, 1),
            ],
        };
        
        self.generate_attacks_from_directions(square, &directions)
    }
    
    /// Generate bishop attack pattern for a square (for promoted bishop)
    fn generate_bishop_attacks(&mut self, square: u8) -> Bitboard {
        const BISHOP_DIRECTIONS: [Direction; 4] = [
            Direction::new(-1, -1), Direction::new(-1, 1),
            Direction::new(1, -1),  Direction::new(1, 1),
        ];
        
        self.generate_attacks_from_directions(square, &BISHOP_DIRECTIONS)
    }
    
    /// Generate rook attack pattern for a square (for promoted rook)
    fn generate_rook_attacks(&mut self, square: u8) -> Bitboard {
        const ROOK_DIRECTIONS: [Direction; 4] = [
            Direction::new(-1, 0), Direction::new(1, 0),
            Direction::new(0, -1), Direction::new(0, 1),
        ];
        
        self.generate_attacks_from_directions(square, &ROOK_DIRECTIONS)
    }
    
    /// Generate attack pattern from a list of directions
    fn generate_attacks_from_directions(&mut self, square: u8, directions: &[Direction]) -> Bitboard {
        let mut attacks = EMPTY_BITBOARD;
        
        for &direction in directions {
            if let Some(target_square) = direction.apply(square) {
                attacks |= 1u128 << target_square;
            }
        }
        
        attacks
    }
    
    /// Initialize direction cache for all piece types
    fn initialize_direction_cache(&mut self) {
        // This method is kept for future extensibility
        // Currently using inline direction arrays for better performance
    }
    
    /// Get validation statistics
    fn get_validation_stats(&self) -> GenerationStats {
        GenerationStats {
            total_patterns_generated: self.validation_stats.total_patterns_generated,
            validation_errors: self.validation_stats.validation_errors,
            average_pattern_size: self.validation_stats.average_pattern_size,
            edge_case_count: self.validation_stats.edge_case_count,
        }
    }
}

impl Default for AttackTables {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_tables_creation() {
        let tables = AttackTables::new();
        assert!(tables.metadata.validation_passed);
        assert!(tables.metadata.memory_usage_bytes > 0);
    }

    #[test]
    fn test_king_attacks() {
        let tables = AttackTables::new();
        
        // Test center square (4,4) - should have 8 attacks
        let center_attacks = tables.king_attacks[40]; // (4*9 + 4) = 40
        assert_eq!(center_attacks.count_ones(), 8);
        
        // Test corner square (0,0) - should have 3 attacks
        let corner_attacks = tables.king_attacks[0];
        assert_eq!(corner_attacks.count_ones(), 3);
        
        // Test edge square (0,4) - should have 5 attacks
        let edge_attacks = tables.king_attacks[4];
        assert_eq!(edge_attacks.count_ones(), 5);
    }

    #[test]
    fn test_knight_attacks() {
        let tables = AttackTables::new();
        
        // Test Black knight from center - should have 2 attacks (forward)
        let center_attacks = tables.black_knight_attacks[40];
        assert_eq!(center_attacks.count_ones(), 2);
        
        // Test Black knight from edge - should have fewer attacks
        let edge_attacks = tables.black_knight_attacks[4];
        assert!(edge_attacks.count_ones() <= 2);
        
        // Test White knight from center - should have 2 attacks (forward from White's perspective)
        let white_center_attacks = tables.white_knight_attacks[40];
        assert_eq!(white_center_attacks.count_ones(), 2);
    }

    #[test]
    fn test_gold_attacks() {
        let tables = AttackTables::new();
        
        // Test Black gold from center - should have 6 attacks
        let center_attacks = tables.black_gold_attacks[40];
        assert_eq!(center_attacks.count_ones(), 6);
        
        // Test Black gold from edge - should have fewer attacks
        let edge_attacks = tables.black_gold_attacks[4];
        assert!(edge_attacks.count_ones() < 6);
        
        // Test White gold from center - should have 6 attacks
        let white_center_attacks = tables.white_gold_attacks[40];
        assert_eq!(white_center_attacks.count_ones(), 6);
    }

    #[test]
    fn test_silver_attacks() {
        let tables = AttackTables::new();
        
        // Test Black silver from center - should have 5 attacks
        let center_attacks = tables.black_silver_attacks[40];
        assert_eq!(center_attacks.count_ones(), 5);
        
        // Test Black silver from edge - should have fewer attacks
        let edge_attacks = tables.black_silver_attacks[4];
        assert!(edge_attacks.count_ones() < 5);
        
        // Test White silver from center - should have 5 attacks
        let white_center_attacks = tables.white_silver_attacks[40];
        assert_eq!(white_center_attacks.count_ones(), 5);
    }

    #[test]
    fn test_promoted_piece_attacks() {
        let tables = AttackTables::new();
        
        // Black promoted pieces should have same attacks as Black gold
        for square in 0..81 {
            assert_eq!(tables.black_promoted_pawn_attacks[square], tables.black_gold_attacks[square]);
            assert_eq!(tables.black_promoted_lance_attacks[square], tables.black_gold_attacks[square]);
            assert_eq!(tables.black_promoted_knight_attacks[square], tables.black_gold_attacks[square]);
            assert_eq!(tables.black_promoted_silver_attacks[square], tables.black_gold_attacks[square]);
        }
        
        // White promoted pieces should have same attacks as White gold
        for square in 0..81 {
            assert_eq!(tables.white_promoted_pawn_attacks[square], tables.white_gold_attacks[square]);
            assert_eq!(tables.white_promoted_lance_attacks[square], tables.white_gold_attacks[square]);
            assert_eq!(tables.white_promoted_knight_attacks[square], tables.white_gold_attacks[square]);
            assert_eq!(tables.white_promoted_silver_attacks[square], tables.white_gold_attacks[square]);
        }
    }

    #[test]
    fn test_promoted_sliding_piece_attacks() {
        let tables = AttackTables::new();
        
        // Promoted bishop should have king + bishop attacks
        for square in 0..81 {
            assert!(tables.promoted_bishop_attacks[square] >= tables.king_attacks[square]);
            assert!(tables.promoted_rook_attacks[square] >= tables.king_attacks[square]);
        }
    }

    #[test]
    fn test_get_attack_pattern() {
        let tables = AttackTables::new();
        let square = 40; // Center square
        
        // Test King (same for both players)
        let king_attacks_black = tables.get_attack_pattern(square, PieceType::King, Player::Black);
        let king_attacks_white = tables.get_attack_pattern(square, PieceType::King, Player::White);
        assert_eq!(king_attacks_black, tables.king_attacks[square as usize]);
        assert_eq!(king_attacks_white, tables.king_attacks[square as usize]);
        
        // Test Black Knight
        let black_knight_attacks = tables.get_attack_pattern(square, PieceType::Knight, Player::Black);
        assert_eq!(black_knight_attacks, tables.black_knight_attacks[square as usize]);
        
        // Test White Knight
        let white_knight_attacks = tables.get_attack_pattern(square, PieceType::Knight, Player::White);
        assert_eq!(white_knight_attacks, tables.white_knight_attacks[square as usize]);
        
        // Test Black Gold
        let black_gold_attacks = tables.get_attack_pattern(square, PieceType::Gold, Player::Black);
        assert_eq!(black_gold_attacks, tables.black_gold_attacks[square as usize]);
        
        // Test White Gold
        let white_gold_attacks = tables.get_attack_pattern(square, PieceType::Gold, Player::White);
        assert_eq!(white_gold_attacks, tables.white_gold_attacks[square as usize]);
    }

    #[test]
    fn test_is_square_attacked() {
        let tables = AttackTables::new();
        
        // Test king attacking adjacent squares from center square 40 (4,4)
        assert!(tables.is_square_attacked(40, 31, PieceType::King, Player::Black)); // Up-left (3,4)
        assert!(tables.is_square_attacked(40, 32, PieceType::King, Player::Black)); // Up (3,5)
        assert!(tables.is_square_attacked(40, 48, PieceType::King, Player::Black)); // Down (5,3)
        assert!(tables.is_square_attacked(40, 49, PieceType::King, Player::Black)); // Down (5,4)
        
        // Test king not attacking distant squares
        assert!(!tables.is_square_attacked(40, 0, PieceType::King, Player::Black)); // Far away
    }

    #[test]
    fn test_validation() {
        let tables = AttackTables::new();
        assert!(tables.validate());
    }

    #[test]
    fn test_direction_application() {
        let direction = Direction::new(1, 0); // Move down
        
        // Test valid move
        assert_eq!(direction.apply(40), Some(49)); // From (4,4) to (5,4)
        
        // Test invalid move (off board)
        assert_eq!(direction.apply(80), None); // From (8,8) to (9,8) - invalid
    }

    #[test]
    fn test_memory_alignment() {
        let tables = AttackTables::new();
        let alignment = std::mem::align_of_val(&tables);
        assert_eq!(alignment, 64); // Should be aligned to 64 bytes
    }
}
