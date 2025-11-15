//! Counter-move heuristic implementation
//!
//! This module contains the counter-move heuristic implementation.
//! Counter-moves are moves that have previously refuted an opponent's move,
//! and are likely to be good responses to similar moves.

use crate::types::core::Move;
use std::collections::HashMap;

/// Counter-move heuristic configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct CounterMoveConfig {
    /// Maximum number of counter-moves per opponent move
    pub max_counter_moves: usize,
    /// Enable counter-move heuristic
    pub enable_counter_move: bool,
    /// Enable counter-move aging
    pub enable_counter_move_aging: bool,
    /// Counter-move aging factor (0.0 to 1.0)
    pub counter_move_aging_factor: f32,
}

impl Default for CounterMoveConfig {
    fn default() -> Self {
        Self {
            max_counter_moves: 2,
            enable_counter_move: true,        // Enabled by default
            enable_counter_move_aging: false, // Disabled by default
            counter_move_aging_factor: 0.9,
        }
    }
}

/// Counter-move manager
///
/// Manages counter-moves organized by opponent move and provides methods for
/// adding, retrieving, and managing counter-moves.
#[derive(Debug, Clone)]
pub struct CounterMoveManager {
    /// Counter-move table: maps opponent's move -> counter-moves that refuted it
    /// Used for quiet move ordering: if opponent played move X, try counter-moves that refuted X
    counter_move_table: HashMap<Move, Vec<Move>>,
}

impl CounterMoveManager {
    /// Create a new counter-move manager
    pub fn new() -> Self {
        Self {
            counter_move_table: HashMap::new(),
        }
    }

    /// Add a counter-move for an opponent's move
    ///
    /// This method stores a move that refuted (caused a cutoff against) an opponent's move.
    /// Counter-moves are used to prioritize moves that were successful against specific opponent moves.
    ///
    /// # Arguments
    /// * `opponent_move` - The opponent's move that was refuted
    /// * `counter_move` - The move that refuted the opponent's move
    /// * `moves_equal` - Function to check if two moves are equal
    /// * `max_counter_moves` - Maximum number of counter-moves per opponent move
    ///
    /// # Returns
    /// True if the counter-move was added (not a duplicate), false otherwise
    pub fn add_counter_move<F>(
        &mut self,
        opponent_move: Move,
        counter_move: Move,
        moves_equal: F,
        max_counter_moves: usize,
    ) -> bool
    where
        F: Fn(&Move, &Move) -> bool,
    {
        // Get or create the counter-moves list for this opponent move
        let counter_list = self
            .counter_move_table
            .entry(opponent_move)
            .or_insert_with(Vec::new);

        // Check if this counter-move is already in the list
        let is_duplicate = counter_list.iter().any(|cm| moves_equal(cm, &counter_move));

        if !is_duplicate {
            // Add the new counter-move
            counter_list.push(counter_move);

            // Limit the number of counter-moves per opponent move (FIFO order)
            if counter_list.len() > max_counter_moves {
                counter_list.remove(0); // Remove oldest counter-move
            }

            true
        } else {
            false
        }
    }

    /// Check if a move is a counter-move for the opponent's last move
    ///
    /// # Arguments
    /// * `move_` - The move to check
    /// * `opponent_last_move` - The opponent's last move (if available)
    /// * `moves_equal` - Function to check if two moves are equal
    ///
    /// # Returns
    /// True if the move is a counter-move for the opponent's last move, false otherwise
    pub fn is_counter_move<F>(
        &self,
        move_: &Move,
        opponent_last_move: Option<&Move>,
        moves_equal: F,
    ) -> bool
    where
        F: Fn(&Move, &Move) -> bool,
    {
        if let Some(opponent_move) = opponent_last_move {
            if let Some(counter_list) = self.counter_move_table.get(opponent_move) {
                return counter_list.iter().any(|cm| moves_equal(cm, move_));
            }
        }

        false
    }

    /// Get all counter-moves for an opponent's move
    pub fn get_counter_moves(&self, opponent_move: &Move) -> Option<&Vec<Move>> {
        self.counter_move_table.get(opponent_move)
    }

    /// Clear all counter-moves for a specific opponent move
    pub fn clear_counter_moves_for_opponent_move(&mut self, opponent_move: &Move) {
        self.counter_move_table.remove(opponent_move);
    }

    /// Clear all counter-moves
    pub fn clear_all_counter_moves(&mut self) {
        self.counter_move_table.clear();
    }

    /// Set the maximum number of counter-moves per opponent move
    ///
    /// Trims existing counter-move lists if necessary.
    pub fn set_max_counter_moves(&mut self, max_moves: usize) {
        for counter_list in self.counter_move_table.values_mut() {
            if counter_list.len() > max_moves {
                counter_list.truncate(max_moves);
            }
        }
    }

    /// Get the number of counter-moves stored for a specific opponent move
    pub fn counter_move_count(&self, opponent_move: &Move) -> usize {
        self.counter_move_table
            .get(opponent_move)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get total number of counter-moves stored
    pub fn total_counter_moves(&self) -> usize {
        self.counter_move_table.values().map(|v| v.len()).sum()
    }

    /// Get memory usage estimate
    pub fn memory_bytes(&self) -> usize {
        let mut total = 0;
        for (_opponent_move, counter_moves) in &self.counter_move_table {
            total += std::mem::size_of::<Move>(); // opponent_move key
            total += std::mem::size_of::<Vec<Move>>(); // vector overhead
            total += counter_moves.len() * std::mem::size_of::<Move>(); // counter_moves
        }
        total
    }
}

impl Default for CounterMoveManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a move that matches a counter-move for the opponent's last move
///
/// Counter-moves get medium-high priority to encourage trying moves that
/// refuted opponent moves in previous searches.
///
/// # Arguments
/// * `counter_move_weight` - Weight for counter-moves
pub fn score_counter_move(counter_move_weight: i32) -> i32 {
    counter_move_weight
}
