//! Counter-move heuristic implementation
//! 
//! This module contains the counter-move heuristic implementation.
//! Counter-moves are moves that have previously refuted an opponent's move,
//! and are likely to be good responses to similar moves.

use crate::types::*;
use std::collections::HashMap;

// TODO: Extract from move_ordering.rs:
// - Counter-move table (counter_move_table: HashMap<Move, Vec<Move>>)
// - Counter-move management methods (add_counter_move, get_counter_moves, clear_counter_moves)
// - Counter-move scoring methods (score_counter_move)
// - Counter-move configuration (CounterMoveConfig)
// - Counter-move aging methods

