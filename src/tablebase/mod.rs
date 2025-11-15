//! Endgame Tablebase System
//!
//! This module provides endgame tablebase functionality for the Shogi engine.
//! It implements a modular system for solving specific endgame positions with
//! perfect play, focusing on common and important endgame scenarios.
//!
//! ## Quick Start
//!
//! ```rust
//! use shogi_engine::tablebase::MicroTablebase;
//! use shogi_engine::{BitboardBoard, Player, CapturedPieces};
//!
//! // Create tablebase with default configuration
//! let mut tablebase = MicroTablebase::new();
//!
//! // Probe for best move
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! if let Some(result) = tablebase.probe(&board, Player::Black, &captured_pieces) {
//!     println!("Best move: {:?}", result.best_move);
//!     println!("Distance to mate: {}", result.distance_to_mate);
//!     println!("Outcome: {:?}", result.outcome);
//! }
//! ```
//!
//! ## Configuration
//!
//! ```rust
//! use shogi_engine::tablebase::{MicroTablebase, TablebaseConfig};
//!
//! // Create optimized configuration
//! let config = TablebaseConfig::performance_optimized();
//! let tablebase = MicroTablebase::with_config(config);
//!
//! // Or customize individual settings
//! let mut config = TablebaseConfig::default();
//! config.cache_size = 10000;
//! config.confidence_threshold = 0.95;
//! let tablebase = MicroTablebase::with_config(config);
//! ```
//!
//! ## Integration with Engine
//!
//! ```rust
//! use shogi_engine::ShogiEngine;
//!
//! let mut engine = ShogiEngine::new();
//! engine.enable_tablebase();
//!
//! // Tablebase will be consulted automatically during search
//! let best_move = engine.get_best_move(1, 1000, None, None);
//! ```
//!
//! ## Performance Monitoring
//!
//! ```rust
//! // Enable profiling
//! tablebase.set_profiling_enabled(true);
//!
//! // Perform operations...
//!
//! // Get performance summary
//! let profiler = tablebase.get_profiler();
//! let summary = profiler.get_summary();
//! println!("Performance: {}", summary);
//! ```
//!
//! Key components:
//! - `micro_tablebase.rs`: Core tablebase implementation
//! - `endgame_solvers/`: Individual endgame solvers for specific scenarios
//! - `position_cache.rs`: Position caching system for performance
//! - `solver_traits.rs`: Common traits for endgame solvers
//! - `tablebase_config.rs`: Configuration management

use crate::types::core::Move;
use serde::{Deserialize, Serialize};

pub mod endgame_solvers;
pub mod micro_tablebase;
pub mod pattern_matching;
pub mod performance_profiler;
pub mod position_analysis;
pub mod position_cache;
pub mod solver_traits;
pub mod tablebase_config;

// Re-export commonly used types
pub use micro_tablebase::MicroTablebase;
pub use pattern_matching::PatternMatcher;
pub use performance_profiler::{OperationProfiler, PerformanceMetrics, TablebaseProfiler};
pub use position_analysis::{PositionAnalysis, PositionAnalyzer, PositionComplexity};
pub use position_cache::PositionCache;
pub use solver_traits::EndgameSolver;
pub use tablebase_config::{TablebaseConfig, TablebaseStats};

/// Result of a tablebase probe containing the optimal move and position analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TablebaseResult {
    /// The optimal move for this position
    pub best_move: Option<Move>,
    /// Distance to mate (positive for winning, negative for losing, None for draw)
    pub distance_to_mate: Option<i32>,
    /// Number of moves to mate (positive for winning, negative for losing, None for draw)
    pub moves_to_mate: Option<u8>,
    /// The outcome of the position
    pub outcome: TablebaseOutcome,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl TablebaseResult {
    /// Create a new tablebase result
    pub fn new(
        best_move: Option<Move>,
        distance_to_mate: Option<i32>,
        outcome: TablebaseOutcome,
        confidence: f32,
    ) -> Self {
        let moves_to_mate = match distance_to_mate {
            Some(d) if d > 0 => Some(d as u8),
            _ => None,
        };

        Self {
            best_move,
            distance_to_mate,
            moves_to_mate,
            outcome,
            confidence,
        }
    }

    /// Create a winning result
    pub fn win(best_move: Option<Move>, moves_to_mate: u8) -> Self {
        Self::new(
            best_move,
            Some(moves_to_mate as i32),
            TablebaseOutcome::Win,
            1.0,
        )
    }

    /// Create a losing result
    pub fn loss(moves_to_mate: u8) -> Self {
        Self::new(
            None,
            Some(-(moves_to_mate as i32)),
            TablebaseOutcome::Loss,
            1.0,
        )
    }

    /// Create a draw result
    pub fn draw() -> Self {
        Self::new(None, None, TablebaseOutcome::Draw, 1.0)
    }

    /// Check if this is a winning position
    pub fn is_winning(&self) -> bool {
        matches!(self.outcome, TablebaseOutcome::Win)
    }

    /// Check if this is a losing position
    pub fn is_losing(&self) -> bool {
        matches!(self.outcome, TablebaseOutcome::Loss)
    }

    /// Check if this is a draw position
    pub fn is_draw(&self) -> bool {
        matches!(self.outcome, TablebaseOutcome::Draw)
    }

    /// Check if the result is unknown
    pub fn is_unknown(&self) -> bool {
        matches!(self.outcome, TablebaseOutcome::Unknown)
    }

    /// Get the score for this result (for evaluation integration)
    pub fn get_score(&self) -> i32 {
        match self.outcome {
            TablebaseOutcome::Win => {
                if let Some(distance) = self.distance_to_mate {
                    10000 - distance
                } else {
                    10000
                }
            }
            TablebaseOutcome::Loss => {
                if let Some(distance) = self.distance_to_mate {
                    -10000 - distance.abs()
                } else {
                    -10000
                }
            }
            TablebaseOutcome::Draw => 0,
            TablebaseOutcome::Unknown => 0,
        }
    }
}

/// Possible outcomes of a tablebase probe
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TablebaseOutcome {
    /// Position is winning for the player to move
    Win,
    /// Position is losing for the player to move
    Loss,
    /// Position is a draw
    Draw,
    /// Position outcome is unknown
    Unknown,
}

impl TablebaseOutcome {
    /// Check if this is a winning outcome
    pub fn is_winning(&self) -> bool {
        matches!(self, TablebaseOutcome::Win)
    }

    /// Check if this is a losing outcome
    pub fn is_losing(&self) -> bool {
        matches!(self, TablebaseOutcome::Loss)
    }

    /// Check if this is a draw outcome
    pub fn is_draw(&self) -> bool {
        matches!(self, TablebaseOutcome::Draw)
    }

    /// Check if this is an unknown outcome
    pub fn is_unknown(&self) -> bool {
        matches!(self, TablebaseOutcome::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, PieceType, Player, Position};

    #[test]
    fn test_tablebase_result_creation() {
        let move_ = Move::new_move(
            Position::new(0, 0),
            Position::new(1, 1),
            PieceType::King,
            Player::Black,
            false,
        );

        let result = TablebaseResult::win(Some(move_), 5);
        assert!(result.is_winning());
        assert_eq!(result.moves_to_mate, Some(5));
        assert_eq!(result.confidence, 1.0);
        assert!(result.best_move.is_some());

        let loss_result = TablebaseResult::loss(3);
        assert!(loss_result.is_losing());
        assert_eq!(loss_result.distance_to_mate, Some(-3));
        assert_eq!(loss_result.confidence, 1.0);
        assert!(loss_result.best_move.is_none());

        let draw_result = TablebaseResult::draw();
        assert!(draw_result.is_draw());
        assert_eq!(loss_result.moves_to_mate, None);
        assert_eq!(draw_result.confidence, 1.0);
        assert!(draw_result.best_move.is_none());
    }

    #[test]
    fn test_tablebase_outcome() {
        assert_eq!(TablebaseOutcome::Win, TablebaseOutcome::Win);
        assert_ne!(TablebaseOutcome::Win, TablebaseOutcome::Loss);
        assert_ne!(TablebaseOutcome::Win, TablebaseOutcome::Draw);
        assert_ne!(TablebaseOutcome::Win, TablebaseOutcome::Unknown);
    }

    #[test]
    fn test_tablebase_result_score() {
        let win_result = TablebaseResult::win(None, 3);
        assert_eq!(win_result.get_score(), 9997); // 10000 - 3

        let loss_result = TablebaseResult::loss(3);
        assert_eq!(loss_result.get_score(), -10003); // -10000 - 3

        let draw_result = TablebaseResult::draw();
        assert_eq!(draw_result.get_score(), 0);
    }
}
