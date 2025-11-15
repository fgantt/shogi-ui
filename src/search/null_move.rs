//! Null-Move Pruning Module
//!
//! This module handles null-move pruning decision logic, reduction calculations, and safety checks.
//! The main null-move search and verification functions remain in `search_engine.rs` due to tight
//! coupling and will be extracted as part of Task 1.8 (coordinator refactoring).
//!
//! Extracted from `search_engine.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use crate::bitboards::BitboardBoard;
use crate::types::board::CapturedPieces;
use crate::types::core::Player;
use crate::types::search::{NullMoveConfig, NullMoveStats, NullMoveReductionStrategy};

/// Null-move pruning helper functions
pub struct NullMoveHelper {
    config: NullMoveConfig,
    stats: NullMoveStats,
}

impl NullMoveHelper {
    /// Create a new NullMoveHelper with the given configuration
    pub fn new(config: NullMoveConfig) -> Self {
        Self {
            config,
            stats: NullMoveStats::default(),
        }
    }

    /// Check if null move pruning should be attempted
    ///
    /// Returns true if:
    /// - Null move is enabled
    /// - Sufficient depth is available
    /// - Not in check
    /// - Position passes endgame detection (if enabled)
    /// - Position passes safety checks (if enabled)
    pub fn should_attempt_null_move(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        can_null_move: bool,
    ) -> bool {
        if !self.config.enabled || !can_null_move {
            return false;
        }

        // Must have sufficient depth
        if depth < self.config.min_depth {
            return false;
        }

        // Cannot be in check
        if board.is_king_in_check(player, captured_pieces) {
            self.stats.disabled_in_check += 1;
            return false;
        }

        // Endgame detection
        if self.config.enable_endgame_detection {
            let piece_count = self.count_pieces_on_board(board);

            // Enhanced endgame type detection if enabled
            if self.config.enable_endgame_type_detection {
                // More sophisticated endgame detection could be added here
                if piece_count <= self.config.max_pieces_threshold {
                    self.stats.disabled_endgame += 1;
                    return false;
                }
            } else {
                // Simple piece count check
                if piece_count <= self.config.max_pieces_threshold {
                    self.stats.disabled_endgame += 1;
                    return false;
                }
            }
        }

        // Safety checks (always enabled - check safety before attempting null move)
        if !self.is_safe_for_null_move(board, captured_pieces, player) {
            self.stats.disabled_endgame += 1; // Use endgame stat as proxy for safety
            return false;
        }

        true
    }

    /// Calculate null move reduction based on configured strategy
    pub fn calculate_null_move_reduction(
        &self,
        board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
        _player: Player,
        depth: u8,
    ) -> u8 {
        match self.config.reduction_strategy {
            NullMoveReductionStrategy::Static => {
                // Static reduction: fixed reduction factor
                self.config.reduction_factor
            }
            NullMoveReductionStrategy::Dynamic => {
                // Dynamic reduction: adjust based on depth
                let base = self.config.reduction_factor as i32;
                let depth_bonus = if depth > 6 {
                    ((depth - 6) as i32) / 2
                } else {
                    0
                };
                // Cap at reasonable maximum (reduction_factor * 2)
                ((base + depth_bonus) as u8).min(self.config.reduction_factor * 2)
            }
            NullMoveReductionStrategy::DepthBased => {
                // Depth-based reduction: different reduction for different depths
                if depth >= 10 {
                    self.config.reduction_factor + 2
                } else if depth >= 7 {
                    self.config.reduction_factor + 1
                } else {
                    self.config.reduction_factor
                }
            }
            NullMoveReductionStrategy::MaterialBased => {
                // Material-based reduction: adjust based on material balance
                let piece_count = self.count_pieces_on_board(board);
                let base = self.config.reduction_factor as i32;
                if piece_count <= 15 {
                    // Endgame: more conservative reduction
                    (base - 1).max(1) as u8
                } else if piece_count >= 30 {
                    // Opening: more aggressive reduction
                    (base + 1).min((self.config.reduction_factor * 2) as i32) as u8
                } else {
                    base as u8
                }
            }
            NullMoveReductionStrategy::PositionTypeBased => {
                // Position-type-based reduction: Different reduction for opening/middlegame/endgame
                let piece_count = self.count_pieces_on_board(board);

                // Classify position type (simplified: use piece count as proxy)
                if piece_count >= 30 {
                    // Opening position: many pieces on board
                    self.config.opening_reduction_factor
                } else if piece_count >= 15 {
                    // Middlegame position: moderate piece count
                    self.config.middlegame_reduction_factor
                } else {
                    // Endgame position: few pieces on board
                    self.config.endgame_reduction_factor
                }
            }
        }
    }

    /// Check if verification search should be performed based on null move score
    ///
    /// Verification is triggered when null move failed (score < beta) but is within the safety margin
    pub fn should_perform_verification(&self, null_move_score: i32, beta: i32) -> bool {
        if self.config.verification_margin == 0 {
            // Verification disabled
            return false;
        }

        // Verification is needed if null move failed (score < beta) but is close to beta
        // i.e., beta - null_move_score <= verification_margin
        null_move_score < beta
            && (beta - null_move_score) <= self.config.verification_margin
    }

    /// Check if a score indicates a potential mate threat
    ///
    /// A mate threat is detected when the null move score is very high (close to beta)
    /// suggesting the position might be winning (mate threat present)
    pub fn is_mate_threat_score(&self, null_move_score: i32, beta: i32) -> bool {
        if !self.config.enable_mate_threat_detection {
            return false;
        }
        if self.config.mate_threat_margin == 0 {
            return false;
        }

        // Mate threat detected if score is very close to beta (within mate_threat_margin)
        // This suggests the position is winning and might contain a mate threat
        null_move_score >= (beta - self.config.mate_threat_margin)
    }

    /// Check if position is safe for null move pruning with additional safety checks
    fn is_safe_for_null_move(
        &self,
        board: &BitboardBoard,
        _captured_pieces: &CapturedPieces,
        player: Player,
    ) -> bool {
        // Check if we have major pieces (rooks, bishops, golds) - more conservative in endgame
        let major_piece_count = self.count_major_pieces(board, player);
        if major_piece_count < 2 {
            return false; // Too few major pieces - potential zugzwang risk
        }

        // Check if position is in late endgame (very few pieces)
        if self.is_late_endgame(board) {
            return false; // Late endgame - high zugzwang risk
        }

        true
    }

    /// Enhanced safety check for null move pruning
    pub fn is_enhanced_safe_for_null_move(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
    ) -> bool {
        // Basic safety checks
        if !self.is_safe_for_null_move(board, captured_pieces, player) {
            return false;
        }

        // Additional tactical safety checks
        // Check if opponent has strong attacking pieces
        let opponent = player.opposite();
        let opponent_major_pieces = self.count_major_pieces(board, opponent);
        if opponent_major_pieces >= 3 {
            return false; // Opponent has strong pieces - potential tactical danger
        }

        true
    }

    /// Check if position is in late endgame where zugzwang is common
    fn is_late_endgame(&self, board: &BitboardBoard) -> bool {
        let total_pieces = self.count_pieces_on_board(board);
        total_pieces <= 8 // Very conservative threshold for late endgame
    }

    /// Count pieces on board (both players)
    fn count_pieces_on_board(&self, board: &BitboardBoard) -> u8 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if board.get_piece(crate::types::Position { row, col }).is_some() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count major pieces for a player (rooks, bishops, golds)
    fn count_major_pieces(&self, board: &BitboardBoard, player: Player) -> u8 {
        use crate::types::PieceType;
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(crate::types::Position { row, col }) {
                    if piece.player == player {
                        match piece.piece_type {
                            PieceType::Rook | PieceType::Bishop | PieceType::Gold => count += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        count
    }

    /// Get null move statistics
    pub fn get_stats(&self) -> &NullMoveStats {
        &self.stats
    }

    /// Get mutable reference to statistics (for updating from main search)
    pub fn get_stats_mut(&mut self) -> &mut NullMoveStats {
        &mut self.stats
    }

    /// Reset null move statistics
    pub fn reset_stats(&mut self) {
        self.stats = NullMoveStats::default();
    }

    /// Get null move configuration
    pub fn get_config(&self) -> &NullMoveConfig {
        &self.config
    }

    /// Update null move configuration
    pub fn update_config(&mut self, config: NullMoveConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_move_helper_creation() {
        let config = NullMoveConfig::default();
        let helper = NullMoveHelper::new(config);
        assert_eq!(helper.get_stats().attempts, 0);
    }

    #[test]
    fn test_verification_check() {
        let config = NullMoveConfig {
            verification_margin: 100,
            ..NullMoveConfig::default()
        };
        let helper = NullMoveHelper::new(config);
        
        // Score close to beta should trigger verification
        assert!(helper.should_perform_verification(1900, 2000)); // 100 margin
        // Score far from beta should not trigger verification
        assert!(!helper.should_perform_verification(1500, 2000)); // 500 margin
    }

    #[test]
    fn test_mate_threat_detection() {
        let config = NullMoveConfig {
            enable_mate_threat_detection: true,
            mate_threat_margin: 200,
            ..NullMoveConfig::default()
        };
        let helper = NullMoveHelper::new(config);
        
        // Score close to beta should indicate mate threat
        assert!(helper.is_mate_threat_score(1900, 2000)); // 100 away
        // Score far from beta should not indicate mate threat
        assert!(!helper.is_mate_threat_score(1500, 2000)); // 500 away
    }

    #[test]
    fn test_config_update() {
        let config = NullMoveConfig::default();
        let mut helper = NullMoveHelper::new(config);
        let new_config = NullMoveConfig {
            min_depth: 4,
            ..NullMoveConfig::default()
        };
        helper.update_config(new_config);
        assert_eq!(helper.get_config().min_depth, 4);
    }
}

