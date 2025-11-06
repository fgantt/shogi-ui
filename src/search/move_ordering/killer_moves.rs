//! Killer moves management
//! 
//! This module contains the killer moves heuristic implementation.
//! Killer moves are moves that caused a beta cutoff at the same depth
//! in a sibling node, and are likely to be good moves in similar positions.

use crate::types::*;
use std::collections::HashMap;

// TODO: Extract from move_ordering.rs:
// - Killer move storage (killer_moves: HashMap<u8, Vec<Move>>)
// - Killer move management methods (add_killer_move, get_killer_moves, clear_killer_moves)
// - Killer move scoring methods (score_killer_move)
// - Killer move configuration (KillerConfig)
// - Depth-based killer move management

