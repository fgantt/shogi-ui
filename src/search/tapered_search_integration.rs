//! Tapered Evaluation Search Integration
//!
//! This module provides search enhancements that leverage the tapered evaluation system:
//! - Phase tracking during search
//! - Phase-aware pruning decisions
//! - Phase-aware move ordering
//! - Phase transition detection
//!
//! # Overview
//!
//! The tapered search integration enhances the search algorithm by:
//! - Tracking game phase at each node
//! - Adjusting pruning aggressiveness based on phase
//! - Prioritizing moves differently in different phases
//! - Detecting phase transitions
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::search::tapered_search_integration::TaperedSearchEnhancer;
//!
//! let mut enhancer = TaperedSearchEnhancer::new();
//! let phase = enhancer.track_phase(&board);
//! let can_prune = enhancer.should_prune(phase, depth, score, beta);
//! ```

use crate::bitboards::BitboardBoard;
use crate::types::core::{PieceType, Position};
use std::collections::HashMap;

/// Tapered search enhancer
pub struct TaperedSearchEnhancer {
    /// Phase tracking cache
    phase_cache: HashMap<u64, i32>,
    /// Phase transition detection
    last_phase: Option<i32>,
    /// Statistics
    stats: TaperedSearchStats,
    /// Configuration
    config: TaperedSearchConfig,
}

impl TaperedSearchEnhancer {
    /// Create a new tapered search enhancer
    pub fn new() -> Self {
        Self {
            phase_cache: HashMap::new(),
            last_phase: None,
            stats: TaperedSearchStats::default(),
            config: TaperedSearchConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TaperedSearchConfig) -> Self {
        Self {
            phase_cache: HashMap::new(),
            last_phase: None,
            stats: TaperedSearchStats::default(),
            config,
        }
    }

    /// Track game phase at current search node
    pub fn track_phase(&mut self, board: &BitboardBoard) -> i32 {
        let hash = self.compute_phase_hash(board);

        if let Some(&phase) = self.phase_cache.get(&hash) {
            return phase;
        }

        let phase = self.calculate_phase(board);
        self.phase_cache.insert(hash, phase);

        // Track phase transitions
        if let Some(last) = self.last_phase {
            if self.is_phase_transition(last, phase) {
                self.stats.phase_transitions += 1;
            }
        }
        self.last_phase = Some(phase);

        phase
    }

    /// Calculate game phase from material
    fn calculate_phase(&self, board: &BitboardBoard) -> i32 {
        let mut phase = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    phase += self.piece_phase_value(piece.piece_type);
                }
            }
        }

        phase.min(256)
    }

    /// Get phase value for piece type
    fn piece_phase_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 0,
            PieceType::Lance => 1,
            PieceType::Knight => 1,
            PieceType::Silver => 2,
            PieceType::Gold => 2,
            PieceType::Bishop => 4,
            PieceType::Rook => 5,
            PieceType::King => 0,
            // Promoted pieces
            _ => 3,
        }
    }

    /// Detect phase transition
    fn is_phase_transition(&self, old_phase: i32, new_phase: i32) -> bool {
        let old_category = self.phase_category(old_phase);
        let new_category = self.phase_category(new_phase);
        old_category != new_category
    }

    /// Get phase category
    fn phase_category(&self, phase: i32) -> PhaseCategory {
        if phase >= 192 {
            PhaseCategory::Opening
        } else if phase >= 64 {
            PhaseCategory::Middlegame
        } else {
            PhaseCategory::Endgame
        }
    }

    /// Determine if pruning should be applied based on phase
    pub fn should_prune(&mut self, phase: i32, depth: u8, score: i32, beta: i32) -> bool {
        if !self.config.enable_phase_aware_pruning {
            return false;
        }

        let category = self.phase_category(phase);
        let margin = self.get_pruning_margin(category, depth);

        let can_prune = score - margin >= beta;

        if can_prune {
            self.stats.phase_aware_prunes += 1;
        }

        can_prune
    }

    /// Get pruning margin based on phase and depth
    fn get_pruning_margin(&self, category: PhaseCategory, depth: u8) -> i32 {
        let base_margin = match category {
            PhaseCategory::Opening => self.config.opening_pruning_margin,
            PhaseCategory::Middlegame => self.config.middlegame_pruning_margin,
            PhaseCategory::Endgame => self.config.endgame_pruning_margin,
        };

        base_margin * (depth as i32)
    }

    /// Get move ordering bonus based on phase
    pub fn get_phase_move_bonus(&self, piece_type: PieceType, phase: i32) -> i32 {
        if !self.config.enable_phase_aware_ordering {
            return 0;
        }

        let category = self.phase_category(phase);

        match category {
            PhaseCategory::Opening => {
                // Prioritize development in opening
                match piece_type {
                    PieceType::Knight | PieceType::Silver | PieceType::Bishop | PieceType::Rook => {
                        100
                    }
                    PieceType::Gold => 50,
                    _ => 0,
                }
            }
            PhaseCategory::Middlegame => {
                // Prioritize tactical pieces in middlegame
                match piece_type {
                    PieceType::Bishop | PieceType::Rook => 150,
                    PieceType::Silver | PieceType::Gold => 75,
                    _ => 0,
                }
            }
            PhaseCategory::Endgame => {
                // Prioritize king activity and pawns in endgame
                match piece_type {
                    PieceType::King => 200,
                    PieceType::Pawn => 100,
                    PieceType::Gold => 75,
                    _ => 25,
                }
            }
        }
    }

    /// Get search depth extension based on phase
    pub fn get_phase_extension(&self, phase: i32, is_check: bool, is_capture: bool) -> u8 {
        if !self.config.enable_phase_extensions {
            return 0;
        }

        let category = self.phase_category(phase);

        match category {
            PhaseCategory::Opening => {
                // Minimal extensions in opening
                if is_check {
                    1
                } else {
                    0
                }
            }
            PhaseCategory::Middlegame => {
                // Moderate extensions in middlegame
                if is_check {
                    2
                } else if is_capture {
                    1
                } else {
                    0
                }
            }
            PhaseCategory::Endgame => {
                // Aggressive extensions in endgame
                if is_check {
                    3
                } else if is_capture {
                    2
                } else {
                    1
                }
            }
        }
    }

    /// Clear phase cache
    pub fn clear_cache(&mut self) {
        self.phase_cache.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> &TaperedSearchStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = TaperedSearchStats::default();
    }

    /// Compute hash for phase caching (material-based)
    fn compute_phase_hash(&self, board: &BitboardBoard) -> u64 {
        let mut hash = 0u64;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_hash = (piece.piece_type as u64) << 4 | (piece.player as u64);
                    hash ^= piece_hash.wrapping_mul(0x9e3779b97f4a7c15);
                }
            }
        }

        hash
    }
}

impl Default for TaperedSearchEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for tapered search enhancements
#[derive(Debug, Clone)]
pub struct TaperedSearchConfig {
    /// Enable phase-aware pruning
    pub enable_phase_aware_pruning: bool,
    /// Enable phase-aware move ordering
    pub enable_phase_aware_ordering: bool,
    /// Enable phase-based extensions
    pub enable_phase_extensions: bool,
    /// Opening pruning margin (per ply)
    pub opening_pruning_margin: i32,
    /// Middlegame pruning margin (per ply)
    pub middlegame_pruning_margin: i32,
    /// Endgame pruning margin (per ply)
    pub endgame_pruning_margin: i32,
}

impl Default for TaperedSearchConfig {
    fn default() -> Self {
        Self {
            enable_phase_aware_pruning: true,
            enable_phase_aware_ordering: true,
            enable_phase_extensions: true,
            opening_pruning_margin: 150,
            middlegame_pruning_margin: 100,
            endgame_pruning_margin: 50,
        }
    }
}

/// Phase category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseCategory {
    Opening,
    Middlegame,
    Endgame,
}

/// Statistics for tapered search
#[derive(Debug, Clone, Default)]
pub struct TaperedSearchStats {
    /// Phase transitions detected
    pub phase_transitions: u64,
    /// Phase-aware prunes applied
    pub phase_aware_prunes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhancer_creation() {
        let enhancer = TaperedSearchEnhancer::new();
        assert!(enhancer.config.enable_phase_aware_pruning);
    }

    #[test]
    fn test_phase_tracking() {
        let mut enhancer = TaperedSearchEnhancer::new();
        let board = BitboardBoard::new();

        let phase1 = enhancer.track_phase(&board);
        let phase2 = enhancer.track_phase(&board);

        assert_eq!(phase1, phase2);
        assert!(enhancer.phase_cache.len() > 0);
    }

    #[test]
    fn test_phase_categories() {
        let enhancer = TaperedSearchEnhancer::new();

        assert_eq!(enhancer.phase_category(256), PhaseCategory::Opening);
        assert_eq!(enhancer.phase_category(128), PhaseCategory::Middlegame);
        assert_eq!(enhancer.phase_category(32), PhaseCategory::Endgame);
    }

    #[test]
    fn test_pruning_decision() {
        let mut enhancer = TaperedSearchEnhancer::new();

        // Opening - more conservative pruning
        let can_prune_opening = enhancer.should_prune(256, 3, 500, 100);

        // Endgame - more aggressive pruning
        let can_prune_endgame = enhancer.should_prune(32, 3, 300, 100);

        // Endgame should prune more readily
        assert!(can_prune_endgame);
    }

    #[test]
    fn test_move_ordering_bonus() {
        let enhancer = TaperedSearchEnhancer::new();

        // Opening bonuses
        let opening_knight = enhancer.get_phase_move_bonus(PieceType::Knight, 256);
        let opening_pawn = enhancer.get_phase_move_bonus(PieceType::Pawn, 256);
        assert!(opening_knight > opening_pawn);

        // Endgame bonuses
        let endgame_king = enhancer.get_phase_move_bonus(PieceType::King, 32);
        let endgame_bishop = enhancer.get_phase_move_bonus(PieceType::Bishop, 32);
        assert!(endgame_king > endgame_bishop);
    }

    #[test]
    fn test_phase_extensions() {
        let enhancer = TaperedSearchEnhancer::new();

        // Opening extensions (conservative)
        let opening_check = enhancer.get_phase_extension(256, true, false);
        let opening_capture = enhancer.get_phase_extension(256, false, true);
        assert_eq!(opening_check, 1);
        assert_eq!(opening_capture, 0);

        // Endgame extensions (aggressive)
        let endgame_check = enhancer.get_phase_extension(32, true, false);
        let endgame_capture = enhancer.get_phase_extension(32, false, true);
        assert_eq!(endgame_check, 3);
        assert_eq!(endgame_capture, 2);
    }

    #[test]
    fn test_phase_transition_detection() {
        let mut enhancer = TaperedSearchEnhancer::new();

        enhancer.track_phase(&BitboardBoard::new());
        let initial_transitions = enhancer.stats.phase_transitions;

        // Tracking same phase shouldn't trigger transition
        enhancer.track_phase(&BitboardBoard::new());
        assert_eq!(enhancer.stats.phase_transitions, initial_transitions);
    }

    #[test]
    fn test_clear_cache() {
        let mut enhancer = TaperedSearchEnhancer::new();
        let board = BitboardBoard::new();

        enhancer.track_phase(&board);
        assert!(enhancer.phase_cache.len() > 0);

        enhancer.clear_cache();
        assert_eq!(enhancer.phase_cache.len(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut enhancer = TaperedSearchEnhancer::new();
        let board = BitboardBoard::new();

        enhancer.track_phase(&board);
        enhancer.should_prune(32, 3, 500, 100);

        let stats = enhancer.stats();
        assert!(stats.phase_aware_prunes > 0);
    }

    #[test]
    fn test_reset_statistics() {
        let mut enhancer = TaperedSearchEnhancer::new();

        enhancer.should_prune(32, 3, 500, 100);
        assert!(enhancer.stats.phase_aware_prunes > 0);

        enhancer.reset_stats();
        assert_eq!(enhancer.stats.phase_aware_prunes, 0);
    }

    #[test]
    fn test_custom_config() {
        let config = TaperedSearchConfig {
            enable_phase_aware_pruning: false,
            enable_phase_aware_ordering: false,
            enable_phase_extensions: false,
            opening_pruning_margin: 200,
            middlegame_pruning_margin: 150,
            endgame_pruning_margin: 100,
        };

        let enhancer = TaperedSearchEnhancer::with_config(config);
        assert!(!enhancer.config.enable_phase_aware_pruning);
    }

    #[test]
    fn test_pruning_margins() {
        let enhancer = TaperedSearchEnhancer::new();

        let opening_margin = enhancer.get_pruning_margin(PhaseCategory::Opening, 5);
        let middlegame_margin = enhancer.get_pruning_margin(PhaseCategory::Middlegame, 5);
        let endgame_margin = enhancer.get_pruning_margin(PhaseCategory::Endgame, 5);

        // Opening should have highest margin (most conservative)
        assert!(opening_margin > middlegame_margin);
        assert!(middlegame_margin > endgame_margin);
    }
}
