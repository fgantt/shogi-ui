//! Endgame solvers for specific scenarios
//! 
//! This module contains individual solvers for different endgame types.
//! Each solver is specialized for a particular endgame pattern and
//! implements the EndgameSolver trait.

// Individual solver modules
pub mod king_gold_vs_king;
pub mod king_silver_vs_king;
pub mod king_rook_vs_king;

// Re-export solver types
pub use king_gold_vs_king::KingGoldVsKingSolver;
pub use king_silver_vs_king::KingSilverVsKingSolver;
pub use king_rook_vs_king::KingRookVsKingSolver;
