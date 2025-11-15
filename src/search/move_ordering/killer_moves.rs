//! Killer moves management
//!
//! This module contains the killer moves heuristic implementation.
//! Killer moves are moves that caused a beta cutoff at the same depth
//! in a sibling node, and are likely to be good moves in similar positions.

use crate::types::core::Move;
use std::collections::HashMap;

/// Killer move configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct KillerConfig {
    /// Maximum number of killer moves per depth
    pub max_killer_moves_per_depth: usize,
    /// Enable killer move aging
    pub enable_killer_aging: bool,
    /// Killer move aging factor (0.0 to 1.0)
    pub killer_aging_factor: f32,
    /// Enable depth-based killer move management
    pub enable_depth_based_management: bool,
}

impl Default for KillerConfig {
    fn default() -> Self {
        Self {
            max_killer_moves_per_depth: 2,
            enable_killer_aging: false, // Disabled by default
            killer_aging_factor: 0.9,
            enable_depth_based_management: true,
        }
    }
}

/// Killer move manager
///
/// Manages killer moves organized by depth and provides methods for
/// adding, retrieving, and managing killer moves.
#[derive(Debug, Clone)]
pub struct KillerMoveManager {
    /// Killer moves organized by depth
    /// Each depth can have multiple killer moves
    killer_moves: HashMap<u8, Vec<Move>>,
    /// Current search depth for killer move management
    current_depth: u8,
}

impl KillerMoveManager {
    /// Create a new killer move manager
    pub fn new() -> Self {
        Self {
            killer_moves: HashMap::new(),
            current_depth: 0,
        }
    }

    /// Set the current search depth for killer move management
    pub fn set_current_depth(&mut self, depth: u8) {
        self.current_depth = depth;
    }

    /// Get the current search depth
    pub fn get_current_depth(&self) -> u8 {
        self.current_depth
    }

    /// Add a killer move for the current depth
    ///
    /// This method stores a move that caused a beta cutoff, making it
    /// a candidate for early consideration in future searches at the same depth.
    ///
    /// # Arguments
    /// * `move_` - The move to add as a killer move
    /// * `moves_equal` - Function to check if two moves are equal
    /// * `max_killer_moves_per_depth` - Maximum number of killer moves per depth
    ///
    /// # Returns
    /// True if the move was added (not a duplicate), false otherwise
    pub fn add_killer_move<F>(
        &mut self,
        move_: Move,
        moves_equal: F,
        max_killer_moves_per_depth: usize,
    ) -> bool
    where
        F: Fn(&Move, &Move) -> bool,
    {
        let depth = self.current_depth;

        // Check if this move is already a killer move at this depth
        let is_duplicate = if let Some(killer_list) = self.killer_moves.get(&depth) {
            killer_list.iter().any(|killer| moves_equal(killer, &move_))
        } else {
            false
        };

        if !is_duplicate {
            // Get or create the killer moves list for this depth
            let killer_list = self.killer_moves.entry(depth).or_insert_with(Vec::new);

            // Add the new killer move
            killer_list.push(move_);

            // Limit the number of killer moves per depth
            if killer_list.len() > max_killer_moves_per_depth {
                killer_list.remove(0); // Remove oldest killer move
            }

            true
        } else {
            false
        }
    }

    /// Check if a move is a killer move at the current depth
    ///
    /// # Arguments
    /// * `move_` - The move to check
    /// * `moves_equal` - Function to check if two moves are equal
    ///
    /// # Returns
    /// True if the move is a killer move at the current depth, false otherwise
    pub fn is_killer_move<F>(&self, move_: &Move, moves_equal: F) -> bool
    where
        F: Fn(&Move, &Move) -> bool,
    {
        let depth = self.current_depth;

        if let Some(killer_list) = self.killer_moves.get(&depth) {
            killer_list.iter().any(|killer| moves_equal(killer, move_))
        } else {
            false
        }
    }

    /// Get all killer moves for a specific depth
    pub fn get_killer_moves(&self, depth: u8) -> Option<&Vec<Move>> {
        self.killer_moves.get(&depth)
    }

    /// Get all killer moves for the current depth
    pub fn get_current_killer_moves(&self) -> Option<&Vec<Move>> {
        self.get_killer_moves(self.current_depth)
    }

    /// Clear killer moves for a specific depth
    pub fn clear_killer_moves_for_depth(&mut self, depth: u8) {
        if let Some(killer_list) = self.killer_moves.get_mut(&depth) {
            killer_list.clear();
        }
    }

    /// Clear all killer moves
    pub fn clear_all_killer_moves(&mut self) {
        self.killer_moves.clear();
    }

    /// Set the maximum number of killer moves per depth
    ///
    /// Trims existing killer move lists if necessary.
    pub fn set_max_killer_moves_per_depth(&mut self, max_moves: usize) {
        for killer_list in self.killer_moves.values_mut() {
            if killer_list.len() > max_moves {
                killer_list.truncate(max_moves);
            }
        }
    }

    /// Get the number of killer moves stored at a specific depth
    pub fn killer_move_count(&self, depth: u8) -> usize {
        self.killer_moves.get(&depth).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total number of killer moves stored
    pub fn total_killer_moves(&self) -> usize {
        self.killer_moves.values().map(|v| v.len()).sum()
    }

    /// Get memory usage estimate
    pub fn memory_bytes(&self) -> usize {
        let mut total = 0;
        for (_depth, moves) in &self.killer_moves {
            total += std::mem::size_of::<u8>(); // depth key
            total += std::mem::size_of::<Vec<Move>>(); // vector overhead
            total += moves.len() * std::mem::size_of::<Move>(); // moves
        }
        total
    }
}

impl Default for KillerMoveManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a move that matches a killer move
///
/// Killer moves get high priority to encourage trying moves that
/// caused beta cutoffs in previous searches at the same depth.
///
/// # Arguments
/// * `killer_move_weight` - Weight for killer moves
pub fn score_killer_move(killer_move_weight: i32) -> i32 {
    killer_move_weight
}
