//! PV (Principal Variation) move ordering
//!
//! This module contains PV move ordering implementation.
//! PV moves are the best moves from previous searches and are given
//! the highest priority in move ordering.

use crate::types::core::Move;
use std::collections::HashMap;

/// PV move ordering manager
///
/// Manages PV move cache and provides methods for retrieving and updating PV moves.
/// PV moves are cached by position hash for fast lookup.
/// Task 11.0: Enhanced with multiple PV moves and previous iteration support
#[derive(Debug, Clone)]
pub struct PVOrdering {
    /// PV move cache: maps position hash -> PV move
    /// Caches PV moves from transposition table lookups
    pv_move_cache: HashMap<u64, Option<Move>>,
    /// PV moves organized by depth: maps depth -> PV move
    /// Stores the best move found at each search depth
    pv_moves: HashMap<u8, Move>,
    /// Task 11.0: Multiple PV moves per position (top N moves)
    /// Maps position hash -> Vec of PV moves (ordered by quality)
    multiple_pv_cache: HashMap<u64, Vec<Move>>,
    /// Task 11.0: Previous iteration PV moves
    /// Maps position hash -> PV move from previous search iteration
    previous_iteration_pv: HashMap<u64, Move>,
    /// Task 11.0: Maximum number of PV moves to store per position
    max_pv_moves_per_position: usize,
    /// Task 11.4: Sibling node PV moves
    /// Maps parent position hash -> Vec of PV moves from sibling nodes
    sibling_pv_moves: HashMap<u64, Vec<Move>>,
}

impl PVOrdering {
    /// Create a new PV ordering manager
    pub fn new() -> Self {
        Self {
            pv_move_cache: HashMap::new(),
            pv_moves: HashMap::new(),
            multiple_pv_cache: HashMap::new(),
            previous_iteration_pv: HashMap::new(),
            max_pv_moves_per_position: 3, // Default: store top 3 PV moves
            sibling_pv_moves: HashMap::new(), // Task 11.4
        }
    }

    /// Create a new PV ordering manager with custom configuration
    /// Task 11.0: Allows configuration of maximum PV moves per position
    pub fn with_max_pv_moves(max_pv_moves: usize) -> Self {
        Self {
            pv_move_cache: HashMap::new(),
            pv_moves: HashMap::new(),
            multiple_pv_cache: HashMap::new(),
            previous_iteration_pv: HashMap::new(),
            max_pv_moves_per_position: max_pv_moves,
            sibling_pv_moves: HashMap::new(), // Task 11.4
        }
    }

    /// Get a cached PV move for a position hash
    pub fn get_cached_pv_move(&self, position_hash: u64) -> Option<Option<Move>> {
        self.pv_move_cache.get(&position_hash).cloned()
    }

    /// Cache a PV move for a position hash
    pub fn cache_pv_move(&mut self, position_hash: u64, pv_move: Option<Move>) {
        self.pv_move_cache.insert(position_hash, pv_move);
    }

    /// Get PV move for a specific depth
    pub fn get_pv_move_for_depth(&self, depth: u8) -> Option<Move> {
        self.pv_moves.get(&depth).cloned()
    }

    /// Update PV move for a specific depth
    pub fn update_pv_move_for_depth(&mut self, depth: u8, move_: Move) {
        self.pv_moves.insert(depth, move_);
    }

    /// Clear the PV move cache
    pub fn clear_cache(&mut self) {
        self.pv_move_cache.clear();
    }

    /// Clear PV moves by depth
    pub fn clear_depth_moves(&mut self) {
        self.pv_moves.clear();
    }

    /// Clear all PV data
    pub fn clear_all(&mut self) {
        self.clear_cache();
        self.clear_depth_moves();
        self.multiple_pv_cache.clear(); // Task 11.0
        self.previous_iteration_pv.clear(); // Task 11.0
        self.sibling_pv_moves.clear(); // Task 11.4
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.pv_move_cache.len()
    }

    // ==================== Task 11.0: Multiple PV Moves ====================

    /// Store multiple PV moves for a position
    /// Task 11.0: Allows storing top N moves from transposition table
    pub fn store_multiple_pv_moves(&mut self, position_hash: u64, moves: Vec<Move>) {
        let mut pv_moves = moves;
        pv_moves.truncate(self.max_pv_moves_per_position);
        self.multiple_pv_cache.insert(position_hash, pv_moves);
    }

    /// Get multiple PV moves for a position
    /// Task 11.0: Returns top N PV moves if available
    pub fn get_multiple_pv_moves(&self, position_hash: u64) -> Option<&Vec<Move>> {
        self.multiple_pv_cache.get(&position_hash)
    }

    /// Set maximum number of PV moves to store per position
    pub fn set_max_pv_moves_per_position(&mut self, max_pv_moves: usize) {
        self.max_pv_moves_per_position = max_pv_moves;
    }

    // ==================== Task 11.0: Previous Iteration PV ====================

    /// Store PV move from previous iteration
    /// Task 11.0: Called at the start of a new iteration to save previous PV moves
    pub fn save_previous_iteration_pv(&mut self) {
        // Copy current PV moves to previous iteration cache
        self.previous_iteration_pv.clear();
        for (hash, pv_move_opt) in &self.pv_move_cache {
            if let Some(pv_move) = pv_move_opt {
                self.previous_iteration_pv.insert(*hash, pv_move.clone());
            }
        }
    }

    /// Get PV move from previous iteration
    /// Task 11.0: Returns PV move from previous search iteration if available
    pub fn get_previous_iteration_pv(&self, position_hash: u64) -> Option<&Move> {
        self.previous_iteration_pv.get(&position_hash)
    }

    /// Clear previous iteration PV moves
    pub fn clear_previous_iteration(&mut self) {
        self.previous_iteration_pv.clear();
    }

    // ==================== Task 11.4: Sibling Node PV ====================

    /// Store PV move from sibling node
    /// Task 11.4: Stores PV moves discovered in sibling search nodes
    ///
    /// Sibling nodes are nodes at the same depth in the search tree.
    /// When exploring one node, the PV from other sibling nodes can be useful.
    pub fn store_sibling_pv(&mut self, parent_hash: u64, sibling_pv_move: Move) {
        let siblings = self
            .sibling_pv_moves
            .entry(parent_hash)
            .or_insert_with(Vec::new);

        // Only store if not already present
        if !siblings.iter().any(|m| moves_equal(m, &sibling_pv_move)) {
            siblings.push(sibling_pv_move);

            // Limit number of sibling PV moves per parent
            if siblings.len() > self.max_pv_moves_per_position {
                siblings.remove(0); // Remove oldest
            }
        }
    }

    /// Get PV moves from sibling nodes
    /// Task 11.4: Returns PV moves discovered in sibling search nodes
    pub fn get_sibling_pv_moves(&self, parent_hash: u64) -> Option<&Vec<Move>> {
        self.sibling_pv_moves.get(&parent_hash)
    }

    /// Clear sibling PV moves
    pub fn clear_sibling_pv(&mut self) {
        self.sibling_pv_moves.clear();
    }

    /// Clear sibling PV moves for a specific parent
    pub fn clear_sibling_pv_for_parent(&mut self, parent_hash: u64) {
        self.sibling_pv_moves.remove(&parent_hash);
    }

    /// Get memory usage estimate for cache
    /// Task 11.0/11.4: Updated to include all PV caches (multiple, previous iteration, sibling)
    pub fn cache_memory_bytes(&self) -> usize {
        let single_pv = self.pv_move_cache.len()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<Option<Move>>());
        let depth_pv =
            self.pv_moves.len() * (std::mem::size_of::<u8>() + std::mem::size_of::<Move>());
        let multiple_pv = self
            .multiple_pv_cache
            .iter()
            .map(|(_, v)| std::mem::size_of::<u64>() + v.len() * std::mem::size_of::<Move>())
            .sum::<usize>();
        let previous_pv = self.previous_iteration_pv.len()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<Move>());
        let sibling_pv = self
            .sibling_pv_moves
            .iter()
            .map(|(_, v)| std::mem::size_of::<u64>() + v.len() * std::mem::size_of::<Move>())
            .sum::<usize>();

        single_pv + depth_pv + multiple_pv + previous_pv + sibling_pv
    }

    /// Check if cache is full (for size management)
    pub fn is_cache_full(&self, max_size: usize) -> bool {
        self.pv_move_cache.len() >= max_size
    }

    /// Remove oldest entries if cache exceeds max size
    /// Simple implementation: clears cache if full (can be enhanced with LRU)
    pub fn trim_cache_if_needed(&mut self, max_size: usize) {
        if self.is_cache_full(max_size) {
            // For now, clear cache if full
            // TODO: Implement LRU eviction if needed
            self.clear_cache();
        }
    }
}

impl Default for PVOrdering {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a PV move
///
/// PV moves get the highest priority weight to ensure they are tried first.
pub fn score_pv_move(pv_move_weight: i32) -> i32 {
    pv_move_weight
}

/// Check if two moves are equal
///
/// Helper function to compare moves for PV matching.
pub fn moves_equal(a: &Move, b: &Move) -> bool {
    a.from == b.from
        && a.to == b.to
        && a.piece_type == b.piece_type
        && a.player == b.player
        && a.is_promotion == b.is_promotion
}

// ==================== Task 11.0: PV Statistics ====================

/// PV move statistics
/// Task 11.0/11.4: Tracks effectiveness of different PV move sources
#[derive(Debug, Clone, Default)]
pub struct PVMoveStatistics {
    /// Number of times primary PV move was used
    pub primary_pv_hits: u64,
    /// Number of times multiple PV moves were used
    pub multiple_pv_hits: u64,
    /// Number of times previous iteration PV was used
    pub previous_iteration_pv_hits: u64,
    /// Task 11.4: Number of times sibling PV was used
    pub sibling_pv_hits: u64,
    /// Number of times PV move not available
    pub pv_misses: u64,
    /// Number of times primary PV move was the best move
    pub primary_pv_best_move_count: u64,
    /// Number of times multiple PV move was the best move
    pub multiple_pv_best_move_count: u64,
    /// Number of times previous iteration PV was the best move
    pub previous_iteration_best_move_count: u64,
    /// Task 11.4: Number of times sibling PV was the best move
    pub sibling_pv_best_move_count: u64,
}

impl PVMoveStatistics {
    /// Create new PV move statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate primary PV hit rate
    pub fn primary_pv_hit_rate(&self) -> f64 {
        let total = self.primary_pv_hits + self.pv_misses;
        if total > 0 {
            (self.primary_pv_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate primary PV effectiveness (percentage of times it was best move)
    pub fn primary_pv_effectiveness(&self) -> f64 {
        if self.primary_pv_hits > 0 {
            (self.primary_pv_best_move_count as f64 / self.primary_pv_hits as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate multiple PV effectiveness
    pub fn multiple_pv_effectiveness(&self) -> f64 {
        if self.multiple_pv_hits > 0 {
            (self.multiple_pv_best_move_count as f64 / self.multiple_pv_hits as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate previous iteration PV effectiveness
    pub fn previous_iteration_effectiveness(&self) -> f64 {
        if self.previous_iteration_pv_hits > 0 {
            (self.previous_iteration_best_move_count as f64
                / self.previous_iteration_pv_hits as f64)
                * 100.0
        } else {
            0.0
        }
    }

    /// Calculate sibling PV effectiveness
    /// Task 11.4: Effectiveness of sibling node PV moves
    pub fn sibling_pv_effectiveness(&self) -> f64 {
        if self.sibling_pv_hits > 0 {
            (self.sibling_pv_best_move_count as f64 / self.sibling_pv_hits as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate sibling PV hit rate
    pub fn sibling_pv_hit_rate(&self) -> f64 {
        let total = self.sibling_pv_hits + self.pv_misses;
        if total > 0 {
            (self.sibling_pv_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}
