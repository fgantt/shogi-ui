//! History heuristic implementation
//!
//! This module contains the history heuristic implementation for move ordering.
//! The history heuristic tracks how successful moves have been in the past
//! and uses this information to prioritize moves in future searches.
//!
//! Task 4.0: Enhanced with phase-aware, relative, time-based aging, and quiet-move-only history.

use crate::bitboards::BitboardBoard;
use crate::types::board::GamePhase;
use crate::types::core::{Move, PieceType, Position};
use std::collections::HashMap;

/// History entry for enhanced history heuristic (Task 4.0)
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// History score
    pub score: u32,
    /// Last update timestamp (for time-based aging)
    pub last_update: u64,
    /// Number of times this entry was updated
    pub update_count: u64,
}

/// History heuristic configuration
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryConfig {
    /// Maximum history score to prevent overflow
    pub max_history_score: u32,
    /// History aging factor (0.0 to 1.0)
    pub history_aging_factor: f32,
    /// Enable automatic history aging
    pub enable_automatic_aging: bool,
    /// History aging frequency (number of updates between aging)
    pub aging_frequency: u64,
    /// Enable history score clamping
    pub enable_score_clamping: bool,
    /// Enable phase-aware history tables (Task 4.0)
    pub enable_phase_aware: bool,
    /// Enable relative history (key from (piece_type, from, to) to (from, to)) (Task 4.0)
    pub enable_relative: bool,
    /// Enable time-based aging (exponential decay based on entry age) (Task 4.0)
    pub enable_time_based_aging: bool,
    /// Enable quiet-move-only history (separate table for quiet moves) (Task 4.0)
    pub enable_quiet_only: bool,
    /// Time-based aging decay factor (0.0 to 1.0) (Task 4.0)
    pub time_aging_decay_factor: f32,
    /// Time-based aging update frequency (milliseconds) (Task 4.0)
    pub time_aging_update_frequency_ms: u64,
    /// Phase-specific aging factors (Task 4.0)
    pub opening_aging_factor: f32,
    pub middlegame_aging_factor: f32,
    pub endgame_aging_factor: f32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_history_score: 10000,
            history_aging_factor: 0.9,
            enable_automatic_aging: true,
            aging_frequency: 1000, // Age every 1000 updates
            enable_score_clamping: true,
            enable_phase_aware: false, // Task 4.0: Disabled by default (can be enabled)
            enable_relative: false,    // Task 4.0: Disabled by default (can be enabled)
            enable_time_based_aging: false, // Task 4.0: Disabled by default (can be enabled)
            enable_quiet_only: false,  // Task 4.0: Disabled by default (can be enabled)
            time_aging_decay_factor: 0.95, // Task 4.0: Decay factor for time-based aging
            time_aging_update_frequency_ms: 1000, // Task 4.0: Update every 1 second
            opening_aging_factor: 0.9, // Task 4.0: Opening phase aging factor
            middlegame_aging_factor: 0.9, // Task 4.0: Middlegame phase aging factor
            endgame_aging_factor: 0.95, // Task 4.0: Endgame phase aging factor (less aggressive)
        }
    }
}

/// History heuristic manager
///
/// Manages all history tables (absolute, relative, quiet, phase-aware) and provides
/// methods for scoring, updating, and aging history entries.
///
/// Task 4.0: Enhanced to support phase-aware, relative, time-based aging, and quiet-move-only history.
#[derive(Debug, Clone)]
pub struct HistoryHeuristicManager {
    /// History table for move scoring (absolute history)
    /// Maps (piece_type, from_square, to_square) -> history score
    history_table: HashMap<(PieceType, Position, Position), u32>,
    /// Relative history table (Task 4.0)
    /// Maps (from_square, to_square) -> HistoryEntry (when enable_relative is true)
    relative_history_table: HashMap<(Position, Position), HistoryEntry>,
    /// Quiet-move-only history table (Task 4.0)
    /// Maps (piece_type, from_square, to_square) -> HistoryEntry (when enable_quiet_only is true)
    quiet_history_table: HashMap<(PieceType, Position, Position), HistoryEntry>,
    /// Phase-aware history tables (Task 4.0)
    /// Maps GamePhase -> history table
    phase_history_tables:
        HashMap<GamePhase, HashMap<(PieceType, Position, Position), HistoryEntry>>,
    /// Current game phase (Task 4.0)
    current_game_phase: GamePhase,
    /// Time-based aging counter (Task 4.0)
    time_aging_counter: u64,
    /// History update counter for aging
    history_update_counter: u64,
}

impl HistoryHeuristicManager {
    /// Create a new history heuristic manager
    pub fn new() -> Self {
        Self {
            history_table: HashMap::new(),
            relative_history_table: HashMap::new(),
            quiet_history_table: HashMap::new(),
            phase_history_tables: HashMap::new(),
            current_game_phase: GamePhase::Opening,
            time_aging_counter: 0,
            history_update_counter: 0,
        }
    }

    /// Get current timestamp for time-based aging
    ///
    /// Returns a monotonically increasing counter for time-based aging.
    /// This is a simple implementation that increments on each call.
    pub fn get_current_timestamp(&mut self) -> u64 {
        self.time_aging_counter += 1;
        self.time_aging_counter
    }

    /// Determine game phase from board material count
    ///
    /// Uses material count to determine game phase:
    /// - 0-20 pieces: Endgame
    /// - 21-35 pieces: Middlegame
    /// - 36+ pieces: Opening
    pub fn determine_game_phase_from_material(&self, board: &BitboardBoard) -> GamePhase {
        let mut material_count = 0;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if board.get_piece(pos).is_some() {
                    material_count += 1;
                }
            }
        }

        GamePhase::from_material_count(material_count)
    }

    /// Apply time-based aging to history score if enabled
    ///
    /// Task 4.0: Helper method for time-based aging with exponential decay.
    pub fn apply_time_based_aging_if_enabled(
        &self,
        score: u32,
        last_update: u64,
        current_time: u64,
        time_aging_enabled: bool,
        decay_factor: f32,
    ) -> u32 {
        if !time_aging_enabled {
            return score;
        }

        let age = current_time.saturating_sub(last_update);

        // Apply exponential decay based on age
        // Decay factor: (decay_factor ^ age) where age is normalized
        let age_normalized = age.min(1000) as f32 / 1000.0; // Normalize age to 0-1 range
        let decay = decay_factor.powf(age_normalized);

        (score as f32 * decay) as u32
    }

    /// Get history score for a move
    ///
    /// Returns the current history score for the given move, or 0 if not found.
    /// Task 4.0: Enhanced to support all history table types.
    pub fn get_history_score(
        &self,
        move_: &Move,
        config: &HistoryConfig,
        current_time: u64,
    ) -> u32 {
        if let Some(from) = move_.from {
            let key = (move_.piece_type, from, move_.to);
            let relative_key = (from, move_.to);

            // Task 4.0: Check quiet-move-only history first
            if config.enable_quiet_only && !move_.is_capture {
                if let Some(entry) = self.quiet_history_table.get(&key) {
                    return self.apply_time_based_aging_if_enabled(
                        entry.score,
                        entry.last_update,
                        current_time,
                        config.enable_time_based_aging,
                        config.time_aging_decay_factor,
                    );
                }
            }

            // Task 4.0: Check phase-aware history
            if config.enable_phase_aware {
                if let Some(phase_table) = self.phase_history_tables.get(&self.current_game_phase) {
                    if let Some(entry) = phase_table.get(&key) {
                        return self.apply_time_based_aging_if_enabled(
                            entry.score,
                            entry.last_update,
                            current_time,
                            config.enable_time_based_aging,
                            config.time_aging_decay_factor,
                        );
                    }
                }
            }

            // Task 4.0: Check relative history
            if config.enable_relative {
                if let Some(entry) = self.relative_history_table.get(&relative_key) {
                    return self.apply_time_based_aging_if_enabled(
                        entry.score,
                        entry.last_update,
                        current_time,
                        config.enable_time_based_aging,
                        config.time_aging_decay_factor,
                    );
                }
            }

            // Fall back to absolute history (original implementation)
            self.history_table.get(&key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    /// Update history score for a move
    ///
    /// This method should be called when a move causes a cutoff or
    /// improves the alpha bound during search.
    /// Task 4.0: Enhanced to support relative history, phase-aware history, quiet-move-only history, and time-based aging.
    pub fn update_history_score(
        &mut self,
        move_: &Move,
        _depth: u8,
        bonus: u32,
        config: &HistoryConfig,
        board: Option<&BitboardBoard>,
    ) {
        if let Some(from) = move_.from {
            let current_time = self.get_current_timestamp();
            let key = (move_.piece_type, from, move_.to);
            let relative_key = (from, move_.to);

            // Task 4.0: Update quiet-move-only history if enabled
            if config.enable_quiet_only && !move_.is_capture {
                let current_entry = self.quiet_history_table.get(&key).cloned();
                let current_score = current_entry.as_ref().map(|e| e.score).unwrap_or(0);
                let new_score = current_score + bonus;
                let final_score = new_score.min(config.max_history_score);

                let entry = HistoryEntry {
                    score: final_score,
                    last_update: current_time,
                    update_count: current_entry
                        .as_ref()
                        .map(|e| e.update_count + 1)
                        .unwrap_or(1),
                };
                self.quiet_history_table.insert(key, entry);
            }

            // Task 4.0: Update phase-aware history if enabled
            if config.enable_phase_aware {
                // Update current game phase if board is provided
                if let Some(board_ref) = board {
                    let new_phase = self.determine_game_phase_from_material(board_ref);
                    if new_phase != self.current_game_phase {
                        self.current_game_phase = new_phase;
                    }
                }

                // Get or create phase table
                let phase_table = self
                    .phase_history_tables
                    .entry(self.current_game_phase)
                    .or_insert_with(HashMap::new);
                let current_entry = phase_table.get(&key).cloned();
                let current_score = current_entry.as_ref().map(|e| e.score).unwrap_or(0);
                let new_score = current_score + bonus;
                let final_score = new_score.min(config.max_history_score);

                let entry = HistoryEntry {
                    score: final_score,
                    last_update: current_time,
                    update_count: current_entry
                        .as_ref()
                        .map(|e| e.update_count + 1)
                        .unwrap_or(1),
                };
                phase_table.insert(key, entry);
            }

            // Task 4.0: Update relative history if enabled
            if config.enable_relative {
                let current_entry = self.relative_history_table.get(&relative_key).cloned();
                let current_score = current_entry.as_ref().map(|e| e.score).unwrap_or(0);
                let new_score = current_score + bonus;
                let final_score = new_score.min(config.max_history_score);

                let entry = HistoryEntry {
                    score: final_score,
                    last_update: current_time,
                    update_count: current_entry
                        .as_ref()
                        .map(|e| e.update_count + 1)
                        .unwrap_or(1),
                };
                self.relative_history_table.insert(relative_key, entry);
            }

            // Always update absolute history (backward compatibility)
            let current_score = self.history_table.get(&key).copied().unwrap_or(0);
            let new_score = current_score + bonus;
            let final_score = new_score.min(config.max_history_score);
            self.history_table.insert(key, final_score);

            self.history_update_counter += 1;
        }
    }

    /// Age the history table to prevent overflow
    ///
    /// This method reduces all history scores by the aging factor,
    /// helping to prevent overflow and giving more weight to recent moves.
    /// Task 4.0: Enhanced to age all history table types (absolute, relative, quiet, phase-aware).
    pub fn age_history_table(&mut self, config: &HistoryConfig) {
        // Determine aging factor based on current game phase if phase-aware
        let aging_factor = if config.enable_phase_aware {
            match self.current_game_phase {
                GamePhase::Opening => config.opening_aging_factor,
                GamePhase::Middlegame => config.middlegame_aging_factor,
                GamePhase::Endgame => config.endgame_aging_factor,
            }
        } else {
            config.history_aging_factor
        };

        // Age absolute history table
        if !self.history_table.is_empty() {
            let mut entries_to_remove = Vec::new();

            for (key, score) in self.history_table.iter_mut() {
                *score = (*score as f32 * aging_factor) as u32;
                if *score == 0 {
                    entries_to_remove.push(*key);
                }
            }

            // Remove entries with zero scores
            for key in entries_to_remove {
                self.history_table.remove(&key);
            }
        }

        // Task 4.0: Age relative history table
        if config.enable_relative && !self.relative_history_table.is_empty() {
            let mut entries_to_remove = Vec::new();

            for (key, entry) in self.relative_history_table.iter_mut() {
                entry.score = (entry.score as f32 * aging_factor) as u32;
                if entry.score == 0 {
                    entries_to_remove.push(*key);
                }
            }

            for key in entries_to_remove {
                self.relative_history_table.remove(&key);
            }
        }

        // Task 4.0: Age quiet-move-only history table
        if config.enable_quiet_only && !self.quiet_history_table.is_empty() {
            let mut entries_to_remove = Vec::new();

            for (key, entry) in self.quiet_history_table.iter_mut() {
                entry.score = (entry.score as f32 * aging_factor) as u32;
                if entry.score == 0 {
                    entries_to_remove.push(*key);
                }
            }

            for key in entries_to_remove {
                self.quiet_history_table.remove(&key);
            }
        }

        // Task 4.0: Age phase-aware history tables
        if config.enable_phase_aware {
            for phase_table in self.phase_history_tables.values_mut() {
                let mut entries_to_remove = Vec::new();

                for (key, entry) in phase_table.iter_mut() {
                    entry.score = (entry.score as f32 * aging_factor) as u32;
                    if entry.score == 0 {
                        entries_to_remove.push(*key);
                    }
                }

                for key in entries_to_remove {
                    phase_table.remove(&key);
                }
            }
        }
    }

    /// Clear the history table
    ///
    /// This method removes all history entries.
    /// Task 4.0: Enhanced to clear all history table types.
    pub fn clear_history_table(&mut self) {
        self.history_table.clear();
        // Task 4.0: Clear all enhanced history tables
        self.relative_history_table.clear();
        self.quiet_history_table.clear();
        self.phase_history_tables.clear();
        self.current_game_phase = GamePhase::Opening;
        self.time_aging_counter = 0;
        self.history_update_counter = 0;
    }

    /// Get current game phase
    pub fn get_current_game_phase(&self) -> GamePhase {
        self.current_game_phase
    }

    /// Set current game phase
    pub fn set_current_game_phase(&mut self, phase: GamePhase) {
        self.current_game_phase = phase;
    }

    /// Get history update counter
    pub fn get_history_update_counter(&self) -> u64 {
        self.history_update_counter
    }

    /// Reset history update counter
    pub fn reset_history_update_counter(&mut self) {
        self.history_update_counter = 0;
    }

    /// Get time aging counter (for time-based aging)
    pub fn get_time_aging_counter(&self) -> u64 {
        self.time_aging_counter
    }

    /// Clamp all history scores to maximum value
    ///
    /// This method ensures all history scores do not exceed the maximum value.
    pub fn clamp_history_scores(&mut self, max_score: u32) {
        // Clamp absolute history table
        for score in self.history_table.values_mut() {
            if *score > max_score {
                *score = max_score;
            }
        }

        // Clamp relative history table
        for entry in self.relative_history_table.values_mut() {
            if entry.score > max_score {
                entry.score = max_score;
            }
        }

        // Clamp quiet history table
        for entry in self.quiet_history_table.values_mut() {
            if entry.score > max_score {
                entry.score = max_score;
            }
        }

        // Clamp phase-aware history tables
        for phase_table in self.phase_history_tables.values_mut() {
            for entry in phase_table.values_mut() {
                if entry.score > max_score {
                    entry.score = max_score;
                }
            }
        }
    }

    /// Get total number of history entries across all tables
    pub fn total_history_entries(&self) -> usize {
        let mut total = self.history_table.len();
        total += self.relative_history_table.len();
        total += self.quiet_history_table.len();
        for phase_table in self.phase_history_tables.values() {
            total += phase_table.len();
        }
        total
    }

    /// Get absolute history table size
    pub fn absolute_history_size(&self) -> usize {
        self.history_table.len()
    }

    /// Get absolute history score for a move (for testing)
    ///
    /// This method is primarily for testing purposes to verify history scores.
    pub fn get_absolute_history_score(&self, key: (PieceType, Position, Position)) -> Option<u32> {
        self.history_table.get(&key).copied()
    }

    /// Get relative history entry for a move (for testing)
    ///
    /// This method is primarily for testing purposes to verify relative history scores.
    pub fn get_relative_history_entry(&self, key: (Position, Position)) -> Option<&HistoryEntry> {
        self.relative_history_table.get(&key)
    }

    /// Get quiet history entry for a move (for testing)
    ///
    /// This method is primarily for testing purposes to verify quiet history scores.
    pub fn get_quiet_history_entry(
        &self,
        key: (PieceType, Position, Position),
    ) -> Option<&HistoryEntry> {
        self.quiet_history_table.get(&key)
    }

    /// Check if absolute history table is empty (for testing)
    pub fn is_absolute_history_empty(&self) -> bool {
        self.history_table.is_empty()
    }

    /// Check if relative history table is empty (for testing)
    pub fn is_relative_history_empty(&self) -> bool {
        self.relative_history_table.is_empty()
    }

    /// Check if quiet history table is empty (for testing)
    pub fn is_quiet_history_empty(&self) -> bool {
        self.quiet_history_table.is_empty()
    }

    /// Check if phase history tables are empty (for testing)
    pub fn is_phase_history_empty(&self) -> bool {
        self.phase_history_tables.is_empty()
    }

    /// Get memory usage estimate
    pub fn memory_bytes(&self) -> usize {
        let mut total = 0;

        // Absolute history table
        total += self.history_table.len()
            * (std::mem::size_of::<(PieceType, Position, Position)>() + std::mem::size_of::<u32>());

        // Relative history table
        total += self.relative_history_table.len()
            * (std::mem::size_of::<(Position, Position)>() + std::mem::size_of::<HistoryEntry>());

        // Quiet history table
        total += self.quiet_history_table.len()
            * (std::mem::size_of::<(PieceType, Position, Position)>()
                + std::mem::size_of::<HistoryEntry>());

        // Phase-aware history tables
        for phase_table in self.phase_history_tables.values() {
            total += phase_table.len()
                * (std::mem::size_of::<(PieceType, Position, Position)>()
                    + std::mem::size_of::<HistoryEntry>());
        }

        total
    }
}

impl Default for HistoryHeuristicManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Score a history move
///
/// Returns the scaled history score for a move.
///
/// # Arguments
/// * `history_score` - The raw history score
/// * `history_weight` - The weight to apply to history scores
pub fn score_history_move(history_score: u32, history_weight: i32) -> i32 {
    (history_score as i32 * history_weight) / 1000
}
