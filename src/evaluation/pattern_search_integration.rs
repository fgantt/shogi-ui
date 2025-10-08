//! Pattern Recognition Search Integration Module
//!
//! This module integrates pattern recognition with the search algorithm:
//! - Pattern-based move ordering
//! - Pattern-based pruning decisions
//! - Pattern recognition in quiescence search
//! - Pattern-aware alpha-beta enhancements
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::pattern_search_integration::PatternSearchIntegrator;
//!
//! let integrator = PatternSearchIntegrator::new();
//! let ordered_moves = integrator.order_moves_by_patterns(&board, &moves, player);
//! ```

use crate::types::*;
use crate::bitboards::BitboardBoard;

/// Pattern-based search integrator
pub struct PatternSearchIntegrator {
    config: PatternSearchConfig,
    stats: PatternSearchStats,
}

impl PatternSearchIntegrator {
    /// Create new pattern search integrator
    pub fn new() -> Self {
        Self {
            config: PatternSearchConfig::default(),
            stats: PatternSearchStats::default(),
        }
    }

    /// Order moves based on pattern recognition
    pub fn order_moves_by_patterns(
        &mut self,
        board: &BitboardBoard,
        moves: &[Move],
        player: Player,
    ) -> Vec<(Move, i32)> {
        self.stats.move_orderings += 1;

        let mut scored_moves = Vec::with_capacity(moves.len());

        for mv in moves {
            let score = self.score_move_by_patterns(board, mv, player);
            scored_moves.push((mv.clone(), score));
        }

        // Sort by score (highest first)
        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));

        scored_moves
    }

    /// Score a move based on tactical and positional patterns
    fn score_move_by_patterns(&self, board: &BitboardBoard, mv: &Move, player: Player) -> i32 {
        let mut score = 0;

        // Bonus for moves that create forks
        if self.creates_fork(board, mv, player) {
            score += self.config.fork_bonus;
        }

        // Bonus for moves to center
        if self.is_central_move(mv) {
            score += self.config.center_move_bonus;
        }

        // Bonus for moves that improve piece activity
        if self.improves_activity(mv, player) {
            score += self.config.activity_bonus;
        }

        // Bonus for captures
        if mv.is_capture {
            score += self.config.capture_bonus;
        }

        // Bonus for promotions
        if mv.is_promotion {
            score += self.config.promotion_bonus;
        }

        score
    }

    /// Check if move creates a fork
    fn creates_fork(&self, _board: &BitboardBoard, _mv: &Move, _player: Player) -> bool {
        // Simplified - would check if move attacks 2+ pieces
        false
    }

    /// Check if move is to center
    fn is_central_move(&self, mv: &Move) -> bool {
        let row = mv.to.row;
        let col = mv.to.col;
        row >= 3 && row <= 5 && col >= 3 && col <= 5
    }

    /// Check if move improves piece activity
    fn improves_activity(&self, mv: &Move, player: Player) -> bool {
        // Check if moving forward (advancing)
        if player == Player::Black {
            mv.to.row < mv.from.unwrap_or(Position::new(8, 4)).row
        } else {
            mv.to.row > mv.from.unwrap_or(Position::new(0, 4)).row
        }
    }

    /// Check if position should be pruned based on patterns
    pub fn should_prune_by_patterns(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> bool {
        self.stats.prune_checks += 1;

        // Don't prune at high depths
        if depth >= self.config.min_depth_for_pruning {
            return false;
        }

        // Check for quiet position (no tactical patterns)
        if self.is_quiet_position(board, player) {
            // Can potentially prune
            let margin = self.config.pruning_margin;
            if beta - alpha < margin {
                self.stats.pruned_positions += 1;
                return true;
            }
        }

        false
    }

    /// Check if position is quiet (no immediate tactics)
    fn is_quiet_position(&self, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified - would check for tactical patterns
        // No forks, pins, or immediate threats
        true
    }

    /// Evaluate patterns in quiescence search
    pub fn evaluate_in_quiescence(
        &mut self,
        board: &BitboardBoard,
        player: Player,
    ) -> i32 {
        self.stats.quiescence_evals += 1;

        // In quiescence, only evaluate tactical patterns
        // Look for immediate tactical threats or opportunities
        
        let mut score = 0;

        // Check for hanging pieces
        score += self.evaluate_hanging_pieces(board, player);

        // Check for immediate tactical threats
        score += self.evaluate_immediate_threats(board, player);

        score
    }

    /// Evaluate hanging pieces (undefended pieces)
    fn evaluate_hanging_pieces(&self, _board: &BitboardBoard, _player: Player) -> i32 {
        // Simplified implementation
        0
    }

    /// Evaluate immediate tactical threats
    fn evaluate_immediate_threats(&self, _board: &BitboardBoard, _player: Player) -> i32 {
        // Simplified implementation
        0
    }

    /// Get statistics
    pub fn stats(&self) -> &PatternSearchStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = PatternSearchStats::default();
    }
}

impl Default for PatternSearchIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for pattern-based search
#[derive(Debug, Clone)]
pub struct PatternSearchConfig {
    /// Bonus for fork-creating moves
    pub fork_bonus: i32,
    
    /// Bonus for moves to center
    pub center_move_bonus: i32,
    
    /// Bonus for activity-improving moves
    pub activity_bonus: i32,
    
    /// Bonus for captures
    pub capture_bonus: i32,
    
    /// Bonus for promotions
    pub promotion_bonus: i32,
    
    /// Minimum depth for pattern-based pruning
    pub min_depth_for_pruning: u8,
    
    /// Pruning margin
    pub pruning_margin: i32,
}

impl Default for PatternSearchConfig {
    fn default() -> Self {
        Self {
            fork_bonus: 200,
            center_move_bonus: 50,
            activity_bonus: 30,
            capture_bonus: 100,
            promotion_bonus: 150,
            min_depth_for_pruning: 3,
            pruning_margin: 200,
        }
    }
}

/// Statistics for pattern search integration
#[derive(Debug, Clone, Default)]
pub struct PatternSearchStats {
    pub move_orderings: u64,
    pub prune_checks: u64,
    pub pruned_positions: u64,
    pub quiescence_evals: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_search_integrator_creation() {
        let integrator = PatternSearchIntegrator::new();
        assert_eq!(integrator.config.fork_bonus, 200);
    }

    #[test]
    fn test_move_ordering() {
        let mut integrator = PatternSearchIntegrator::new();
        let board = BitboardBoard::new();
        
        // Create some test moves
        let moves = vec![];
        
        let ordered = integrator.order_moves_by_patterns(&board, &moves, Player::Black);
        
        assert_eq!(integrator.stats().move_orderings, 1);
        assert_eq!(ordered.len(), 0);
    }

    #[test]
    fn test_central_move_detection() {
        let integrator = PatternSearchIntegrator::new();
        
        let central_move = Move {
            from: Some(Position::new(6, 4)),
            to: Position::new(4, 4),
            piece_type: PieceType::Knight,
            captured_piece: None,
            is_promotion: false,
            is_capture: false,
            is_drop: false,
        };
        
        assert!(integrator.is_central_move(&central_move));
        
        let edge_move = Move {
            from: Some(Position::new(6, 0)),
            to: Position::new(4, 0),
            piece_type: PieceType::Lance,
            captured_piece: None,
            is_promotion: false,
            is_capture: false,
            is_drop: false,
        };
        
        assert!(!integrator.is_central_move(&edge_move));
    }

    #[test]
    fn test_pruning_decision() {
        let mut integrator = PatternSearchIntegrator::new();
        let board = BitboardBoard::new();
        
        // Shallow depth - should consider pruning
        let should_prune = integrator.should_prune_by_patterns(
            &board,
            Player::Black,
            2,
            100,
            150,
        );
        
        assert_eq!(integrator.stats().prune_checks, 1);
    }

    #[test]
    fn test_quiescence_evaluation() {
        let mut integrator = PatternSearchIntegrator::new();
        let board = BitboardBoard::new();
        
        let score = integrator.evaluate_in_quiescence(&board, Player::Black);
        
        assert_eq!(integrator.stats().quiescence_evals, 1);
        assert_eq!(score, 0);  // Starting position is quiet
    }

    #[test]
    fn test_statistics_tracking() {
        let mut integrator = PatternSearchIntegrator::new();
        let board = BitboardBoard::new();
        
        // Trigger various operations
        let _ = integrator.order_moves_by_patterns(&board, &[], Player::Black);
        let _ = integrator.should_prune_by_patterns(&board, Player::Black, 2, 100, 150);
        let _ = integrator.evaluate_in_quiescence(&board, Player::Black);
        
        let stats = integrator.stats();
        assert_eq!(stats.move_orderings, 1);
        assert_eq!(stats.prune_checks, 1);
        assert_eq!(stats.quiescence_evals, 1);
    }

    #[test]
    fn test_reset_statistics() {
        let mut integrator = PatternSearchIntegrator::new();
        let board = BitboardBoard::new();
        
        let _ = integrator.order_moves_by_patterns(&board, &[], Player::Black);
        
        integrator.reset_stats();
        assert_eq!(integrator.stats().move_orderings, 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = PatternSearchConfig::default();
        assert_eq!(config.fork_bonus, 200);
        assert_eq!(config.capture_bonus, 100);
        assert_eq!(config.promotion_bonus, 150);
    }
}
