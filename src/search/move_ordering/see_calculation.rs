//! Static Exchange Evaluation (SEE) calculation
//! 
//! This module contains the SEE calculation implementation.
//! SEE evaluates the material gain/loss from a sequence of captures
//! starting with the given move.

use crate::types::*;
use crate::bitboards::BitboardBoard;

// TODO: Extract from move_ordering.rs:
// - SEE calculation methods (calculate_see, calculate_see_internal)
// - SEE scoring methods (score_see_move)
// - SEE cache management (see_cache: HashMap<(Position, Position), i32>)
// - Helper methods for finding attackers/defenders (find_attackers_defenders)
// - Piece attack checking methods (piece_attacks_square_internal, etc.)

