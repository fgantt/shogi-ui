//! History heuristic implementation
//! 
//! This module contains the history heuristic implementation for move ordering.
//! The history heuristic tracks how successful moves have been in the past
//! and uses this information to prioritize moves in future searches.
//! 
//! Task 4.0: Enhanced with phase-aware, relative, time-based aging, and quiet-move-only history.

use crate::types::*;
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

// TODO: Extract from move_ordering.rs:
// - History table structures (history_table, relative_history_table, quiet_history_table, phase_history_tables)
// - History scoring methods (score_history_move, get_history_score)
// - History updating methods (update_history_score, update_history)
// - History aging methods (age_history_table, apply_time_based_aging_if_enabled)
// - History clearing methods (clear_history_table, clear_history)
// - Phase detection methods (determine_game_phase_from_material)
// - History configuration structures (HistoryConfig)

