//! PV (Principal Variation) move ordering
//! 
//! This module contains PV move ordering implementation.
//! PV moves are the best moves from previous searches and are given
//! the highest priority in move ordering.

use crate::types::*;
use std::collections::HashMap;

/// PV move ordering manager
/// 
/// Manages PV move cache and provides methods for retrieving and updating PV moves.
/// PV moves are cached by position hash for fast lookup.
#[derive(Debug, Clone)]
pub struct PVOrdering {
    /// PV move cache: maps position hash -> PV move
    /// Caches PV moves from transposition table lookups
    pv_move_cache: HashMap<u64, Option<Move>>,
    /// PV moves organized by depth: maps depth -> PV move
    /// Stores the best move found at each search depth
    pv_moves: HashMap<u8, Move>,
}

impl PVOrdering {
    /// Create a new PV ordering manager
    pub fn new() -> Self {
        Self {
            pv_move_cache: HashMap::new(),
            pv_moves: HashMap::new(),
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
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.pv_move_cache.len()
    }

    /// Get memory usage estimate for cache
    pub fn cache_memory_bytes(&self) -> usize {
        self.pv_move_cache.len() * (std::mem::size_of::<u64>() + std::mem::size_of::<Option<Move>>())
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
    a.from == b.from && 
    a.to == b.to && 
    a.piece_type == b.piece_type && 
    a.player == b.player &&
    a.is_promotion == b.is_promotion
}
