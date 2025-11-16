//! Feature extractors: modules that derive features/signals from board state.
//!
//! This namespace groups evaluation components whose primary responsibility is
//! computing features (not final scoring). Public re-exports are provided to
//! avoid breaking existing imports while establishing a clearer structure.
//!
//! Responsibilities:
//! - Position/positional/tactical/endgame/opening feature extraction
//! - Piece-square tables and related loaders
//! - Castle recognition and pattern libraries
//!
//! Aggregation/scoring lives under `crate::evaluation::aggregators`.

pub use crate::evaluation::position_features::*;
pub use crate::evaluation::positional_patterns::*;
pub use crate::evaluation::tactical_patterns::*;
pub use crate::evaluation::endgame_patterns::*;
pub use crate::evaluation::opening_principles::*;
pub use crate::evaluation::castles::*;
pub use crate::evaluation::piece_square_tables::*;
pub use crate::evaluation::pst_loader::*;
pub use crate::evaluation::attacks::*;
pub use crate::evaluation::king_safety::*;
pub use crate::evaluation::patterns::*;


