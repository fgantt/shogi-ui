//! PV (Principal Variation) move ordering
//! 
//! This module contains PV move ordering implementation.
//! PV moves are the best moves from previous searches and are given
//! the highest priority in move ordering.

use crate::types::*;
use std::collections::HashMap;

// TODO: Extract from move_ordering.rs:
// - PV move cache (pv_move_cache: HashMap<u64, Option<Move>>)
// - PV moves by depth (pv_moves: HashMap<u8, Move>)
// - PV move retrieval methods (get_pv_move, update_pv_move)
// - PV move scoring methods (score_pv_move)
// - PV move caching and lookup logic

