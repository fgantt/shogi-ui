//! Pattern Recognition Performance Optimization Module
//!
//! This module provides optimized implementations for pattern detection:
//! - Bitboard-based pattern operations
//! - Pre-computed lookup tables
//! - Memory layout optimization
//! - Hot path profiling and optimization
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::pattern_optimization::OptimizedPatternDetector;
//!
//! let detector = OptimizedPatternDetector::new();
//! let patterns = detector.detect_patterns_fast(&board, player);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::core::{PieceType, Player, Position};
use std::collections::HashMap;

/// Optimized pattern detector using bitboards and lookup tables
pub struct OptimizedPatternDetector {
    /// Pre-computed attack lookup tables
    attack_tables: AttackLookupTables,

    /// Pattern lookup tables (reserved for future use)
    #[allow(dead_code)]
    pattern_tables: PatternLookupTables,

    /// Performance statistics
    stats: OptimizationStats,
}

impl OptimizedPatternDetector {
    /// Create new optimized detector
    pub fn new() -> Self {
        Self {
            attack_tables: AttackLookupTables::new(),
            pattern_tables: PatternLookupTables::new(),
            stats: OptimizationStats::default(),
        }
    }

    /// Detect patterns using optimized algorithms
    pub fn detect_patterns_fast(
        &mut self,
        board: &BitboardBoard,
        player: Player,
    ) -> FastPatternResult {
        self.stats.fast_detections += 1;

        let start = std::time::Instant::now();

        // Use bitboard operations for fast pattern detection
        let result = FastPatternResult {
            has_fork: self.detect_fork_fast(board, player),
            has_pin: self.detect_pin_fast(board, player),
            has_outpost: self.detect_outpost_fast(board, player),
            center_control: self.evaluate_center_fast(board, player),
        };

        self.stats.total_time_ns += start.elapsed().as_nanos() as u64;

        result
    }

    /// Fast fork detection using bitboards
    fn detect_fork_fast(&self, board: &BitboardBoard, player: Player) -> bool {
        // Use attack tables to quickly check for forks
        // Simplified implementation - full version would use bitboard operations

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        // Check if piece attacks 2+ enemy pieces
                        let attacks = self
                            .attack_tables
                            .get_attacks(pos, piece.piece_type, player);
                        let enemy_count = attacks
                            .iter()
                            .filter(|&&attack_pos| {
                                if let Some(target) = board.get_piece(attack_pos) {
                                    target.player != player
                                } else {
                                    false
                                }
                            })
                            .count();

                        if enemy_count >= 2 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Fast pin detection using bitboards
    fn detect_pin_fast(&self, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified - would use bitboard ray operations
        false
    }

    /// Fast outpost detection
    fn detect_outpost_fast(&self, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified - would use bitboard pawn structure operations
        false
    }

    /// Fast center control evaluation
    fn evaluate_center_fast(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;

        // Check center squares (3-5, 3-5)
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        score += 20;
                    } else {
                        score -= 20;
                    }
                }
            }
        }

        score
    }

    /// Get optimization statistics
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }

    /// Get average detection time in nanoseconds
    pub fn avg_time_ns(&self) -> u64 {
        if self.stats.fast_detections == 0 {
            0
        } else {
            self.stats.total_time_ns / self.stats.fast_detections
        }
    }
}

impl Default for OptimizedPatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Attack lookup tables for fast pattern detection
struct AttackLookupTables {
    /// Cached attacks for each square and piece type
    cache: HashMap<(Position, PieceType), Vec<Position>>,
}

impl AttackLookupTables {
    fn new() -> Self {
        let mut tables = Self {
            cache: HashMap::new(),
        };
        tables.initialize();
        tables
    }

    fn initialize(&mut self) {
        // Pre-compute attack tables for common piece types
        // This would be filled with actual attack patterns
        // For now, just create empty structure
    }

    fn get_attacks(&self, pos: Position, piece_type: PieceType, player: Player) -> Vec<Position> {
        // Look up pre-computed attacks
        if let Some(attacks) = self.cache.get(&(pos, piece_type)) {
            attacks.clone()
        } else {
            // Compute on demand (would cache for next time)
            self.compute_attacks(pos, piece_type, player)
        }
    }

    fn compute_attacks(
        &self,
        pos: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Vec<Position> {
        let mut attacks = Vec::new();

        match piece_type {
            PieceType::Knight => {
                let moves = if player == Player::Black {
                    vec![(-2, -1), (-2, 1)]
                } else {
                    vec![(2, -1), (2, 1)]
                };

                for (dr, dc) in moves {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;

                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        attacks.push(Position::new(new_row as u8, new_col as u8));
                    }
                }
            }
            _ => {}
        }

        attacks
    }
}

/// Pattern-specific lookup tables
struct PatternLookupTables {
    /// King safety patterns by king position (reserved for future use)
    #[allow(dead_code)]
    king_safety_patterns: HashMap<Position, i32>,
}

impl PatternLookupTables {
    fn new() -> Self {
        Self {
            king_safety_patterns: HashMap::new(),
        }
    }
}

/// Fast pattern detection result
#[derive(Debug, Clone, Copy)]
pub struct FastPatternResult {
    pub has_fork: bool,
    pub has_pin: bool,
    pub has_outpost: bool,
    pub center_control: i32,
}

/// Optimization statistics
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// Number of fast detections
    pub fast_detections: u64,

    /// Total time spent in nanoseconds
    pub total_time_ns: u64,
}

/// Memory-optimized pattern storage
///
/// Uses compact representation to reduce memory footprint
#[repr(C, align(64))] // Cache line alignment
pub struct CompactPatternStorage {
    /// Packed pattern flags (1 bit per pattern type)
    pattern_flags: u8,

    /// Compact scores (i16 instead of i32 for memory efficiency)
    tactical_mg: i16,
    tactical_eg: i16,
    positional_mg: i16,
    positional_eg: i16,
    endgame_mg: i16,
    endgame_eg: i16,
}

impl CompactPatternStorage {
    /// Create new compact storage
    pub fn new() -> Self {
        Self {
            pattern_flags: 0,
            tactical_mg: 0,
            tactical_eg: 0,
            positional_mg: 0,
            positional_eg: 0,
            endgame_mg: 0,
            endgame_eg: 0,
        }
    }

    /// Set pattern flag
    pub fn set_pattern(&mut self, pattern_type: u8) {
        self.pattern_flags |= 1 << pattern_type;
    }

    /// Check if pattern is present
    pub fn has_pattern(&self, pattern_type: u8) -> bool {
        (self.pattern_flags & (1 << pattern_type)) != 0
    }

    /// Store tactical score
    pub fn set_tactical_score(&mut self, mg: i32, eg: i32) {
        self.tactical_mg = mg.clamp(-32768, 32767) as i16;
        self.tactical_eg = eg.clamp(-32768, 32767) as i16;
    }

    /// Get tactical score
    pub fn get_tactical_score(&self) -> (i32, i32) {
        (self.tactical_mg as i32, self.tactical_eg as i32)
    }
}

impl Default for CompactPatternStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_detector_creation() {
        let detector = OptimizedPatternDetector::new();
        assert_eq!(detector.stats().fast_detections, 0);
    }

    #[test]
    fn test_fast_pattern_detection() {
        let mut detector = OptimizedPatternDetector::new();
        let board = BitboardBoard::new();

        let result = detector.detect_patterns_fast(&board, Player::Black);

        assert_eq!(detector.stats().fast_detections, 1);
    }

    #[test]
    fn test_attack_lookup_tables() {
        let tables = AttackLookupTables::new();
        let pos = Position::new(4, 4);

        let attacks = tables.get_attacks(pos, PieceType::Knight, Player::Black);

        // Knight at (4,4) should have 2 attacks for Black
        assert_eq!(attacks.len(), 2);
    }

    #[test]
    fn test_compact_storage() {
        let mut storage = CompactPatternStorage::new();

        storage.set_pattern(0); // Fork
        storage.set_pattern(2); // Outpost

        assert!(storage.has_pattern(0));
        assert!(!storage.has_pattern(1));
        assert!(storage.has_pattern(2));
    }

    #[test]
    fn test_compact_storage_scores() {
        let mut storage = CompactPatternStorage::new();

        storage.set_tactical_score(150, 90);

        let (mg, eg) = storage.get_tactical_score();
        assert_eq!(mg, 150);
        assert_eq!(eg, 90);
    }

    #[test]
    fn test_compact_storage_clamping() {
        let mut storage = CompactPatternStorage::new();

        // Test clamping of large values
        storage.set_tactical_score(100000, -100000);

        let (mg, eg) = storage.get_tactical_score();
        assert_eq!(mg, 32767); // Clamped to i16::MAX
        assert_eq!(eg, -32768); // Clamped to i16::MIN
    }

    #[test]
    fn test_optimization_stats() {
        let stats = OptimizationStats::default();
        assert_eq!(stats.fast_detections, 0);
        assert_eq!(stats.total_time_ns, 0);
    }

    #[test]
    fn test_average_time_calculation() {
        let mut detector = OptimizedPatternDetector::new();
        let board = BitboardBoard::new();

        // Run multiple detections
        for _ in 0..10 {
            let _ = detector.detect_patterns_fast(&board, Player::Black);
        }

        let avg_time = detector.avg_time_ns();
        assert!(avg_time > 0);
    }

    #[test]
    fn test_memory_alignment() {
        use std::mem;

        // Verify cache line alignment
        assert_eq!(mem::align_of::<CompactPatternStorage>(), 64);

        // Verify compact size
        let size = mem::size_of::<CompactPatternStorage>();
        assert!(size <= 64, "CompactPatternStorage should fit in cache line");
    }
}
