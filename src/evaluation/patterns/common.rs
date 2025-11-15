use crate::bitboards::*;
use crate::types::core::{PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;

/// Common attack patterns and tactical threat detection
pub struct TacticalPattern {
    pub name: &'static str,
    pub pattern_type: TacticalType,
    pub danger_level: u8, // 1-10 scale
}

/// Types of tactical patterns
pub enum TacticalType {
    MatingNet,
    Sacrifice,
    Pin,
    Skewer,
    DiscoveredAttack,
    DoubleAttack,
}

/// Threat evaluator for detecting tactical patterns
pub struct ThreatEvaluator {
    patterns: Vec<TacticalPattern>,
}

impl ThreatEvaluator {
    /// Create a new threat evaluator with default patterns
    pub fn new() -> Self {
        Self {
            patterns: vec![
                TacticalPattern {
                    name: "Rook Pin",
                    pattern_type: TacticalType::Pin,
                    danger_level: 7,
                },
                TacticalPattern {
                    name: "Bishop Skewer",
                    pattern_type: TacticalType::Skewer,
                    danger_level: 6,
                },
                TacticalPattern {
                    name: "Knight Fork",
                    pattern_type: TacticalType::DoubleAttack,
                    danger_level: 8,
                },
            ],
        }
    }

    /// Evaluate tactical threats to the king for the given player
    pub fn evaluate_threats(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let mut threat_score = 0;

        for pattern in &self.patterns {
            if self.detect_pattern(board, player.opposite(), king_pos, pattern) {
                threat_score += pattern.danger_level as i32 * 20;
            }
        }

        TaperedScore::new_tapered(-threat_score, -threat_score / 3)
    }

    /// Detect a specific tactical pattern
    fn detect_pattern(
        &self,
        board: &BitboardBoard,
        player: Player,
        king_pos: Position,
        pattern: &TacticalPattern,
    ) -> bool {
        match pattern.pattern_type {
            TacticalType::Pin => self.detect_rook_pin(board, player, king_pos),
            TacticalType::Skewer => self.detect_bishop_skewer(board, player, king_pos),
            TacticalType::DoubleAttack => self.detect_knight_fork(board, player, king_pos),
            _ => false, // TODO: Implement other pattern types
        }
    }

    /// Detect if king is pinned by opponent's rook
    fn detect_rook_pin(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _king_pos: Position,
    ) -> bool {
        // TODO: Implement rook pin detection
        // Check for rook on same rank or file with no blocking pieces
        false
    }

    /// Detect if king is skewered by opponent's bishop
    fn detect_bishop_skewer(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _king_pos: Position,
    ) -> bool {
        // TODO: Implement bishop skewer detection
        false
    }

    /// Detect if opponent's knight can fork king and another piece
    fn detect_knight_fork(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        _king_pos: Position,
    ) -> bool {
        // TODO: Implement knight fork detection
        false
    }

    /// Find king position for a player
    fn find_king_position(&self, board: &BitboardBoard, player: Player) -> Option<Position> {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::King && piece.player == player {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }
}

impl Default for ThreatEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_evaluator_creation() {
        let evaluator = ThreatEvaluator::new();
        assert_eq!(evaluator.patterns.len(), 3);

        let rook_pin = evaluator.patterns.iter().find(|p| p.name == "Rook Pin");
        let bishop_skewer = evaluator
            .patterns
            .iter()
            .find(|p| p.name == "Bishop Skewer");
        let knight_fork = evaluator.patterns.iter().find(|p| p.name == "Knight Fork");

        assert!(rook_pin.is_some());
        assert!(bishop_skewer.is_some());
        assert!(knight_fork.is_some());

        assert_eq!(rook_pin.unwrap().danger_level, 7);
        assert_eq!(bishop_skewer.unwrap().danger_level, 6);
        assert_eq!(knight_fork.unwrap().danger_level, 8);
    }

    #[test]
    fn test_tactical_pattern_creation() {
        let pattern = TacticalPattern {
            name: "Test Pattern",
            pattern_type: TacticalType::Pin,
            danger_level: 5,
        };

        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.danger_level, 5);
    }

    #[test]
    fn test_threat_evaluation_disabled() {
        let evaluator = ThreatEvaluator::new();
        let board = BitboardBoard::new();
        let score = evaluator.evaluate_threats(&board, Player::Black);

        // Should return a score (even if placeholder)
        assert_eq!(score, TaperedScore::default());
    }
}
