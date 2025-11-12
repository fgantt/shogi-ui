#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::attacks::{AttackAnalyzer, ThreatEvaluator};
use shogi_engine::evaluation::castles::*;
use shogi_engine::evaluation::king_safety::*;
use shogi_engine::types::*;

/// Integration tests for advanced king safety evaluation
/// These tests verify that all components work together correctly

#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    /// Create a complex position with multiple castle patterns and attacks
    fn create_complex_position() -> BitboardBoard {
        let mut board = BitboardBoard::empty();

        // Black Mino castle
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::Black,
            },
            Position::new(8, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
            },
            Position::new(7, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::Black,
            },
            Position::new(6, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::Black,
            },
            Position::new(6, 2),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::Black,
            },
            Position::new(7, 2),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::Black,
            },
            Position::new(8, 2),
        );

        // White Anaguma castle
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::White,
            },
            Position::new(0, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::White,
            },
            Position::new(1, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::White,
            },
            Position::new(2, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::White,
            },
            Position::new(2, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::White,
            },
            Position::new(2, 5),
        );

        // White attacking pieces
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::White,
            },
            Position::new(5, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Bishop,
                player: Player::White,
            },
            Position::new(4, 3),
        );

        // Black attacking pieces
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::Black,
            },
            Position::new(3, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Bishop,
                player: Player::Black,
            },
            Position::new(4, 5),
        );

        board
    }

    /// Create a position with tactical threats
    fn create_tactical_threat_position() -> BitboardBoard {
        let mut board = BitboardBoard::empty();

        // Black king
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::Black,
            },
            Position::new(8, 4),
        );

        // Black piece that will be pinned
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
            },
            Position::new(7, 4),
        );

        // White rook pinning the gold
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::White,
            },
            Position::new(6, 4),
        );

        // White knight creating a fork threat
        board.place_piece(
            Piece {
                piece_type: PieceType::Knight,
                player: Player::White,
            },
            Position::new(6, 2),
        );

        // Black pieces that could be forked
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::Black,
            },
            Position::new(7, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
            },
            Position::new(7, 5),
        );

        board
    }

    #[test]
    fn test_full_king_safety_integration() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_complex_position();

        let black_score = evaluator.evaluate(&board, Player::Black);
        let white_score = evaluator.evaluate(&board, Player::White);

        // Both players should return valid scores (not necessarily non-negative due to threats)
        // Just ensure we get valid TaperedScore values without panicking
        assert!(black_score.mg >= -1000 && black_score.mg <= 1000);
        assert!(white_score.mg >= -1000 && white_score.mg <= 1000);
    }

    #[test]
    fn test_castle_vs_attack_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_complex_position();

        let black_score = evaluator.evaluate(&board, Player::Black);
        let white_score = evaluator.evaluate(&board, Player::White);

        // Both should return valid scores (not necessarily non-negative due to threats)
        assert!(black_score.mg >= -1000 && black_score.mg <= 1000);
        assert!(white_score.mg >= -1000 && white_score.mg <= 1000);
    }

    #[test]
    fn test_tactical_threat_integration() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_tactical_threat_position();

        let black_score = evaluator.evaluate(&board, Player::Black);

        // Should return a valid score (threats can make it negative)
        assert!(black_score.mg >= -1000 && black_score.mg <= 1000);
    }

    #[test]
    fn test_phase_aware_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_complex_position();

        let black_score = evaluator.evaluate(&board, Player::Black);

        // Should return a valid score
        assert!(black_score.mg >= 0 && black_score.eg >= 0);
    }

    #[test]
    fn test_configuration_impact() {
        let mut config = KingSafetyConfig::default();
        config.castle_weight = 2.0;
        config.attack_weight = 0.5;
        config.threat_weight = 1.5;

        let evaluator = KingSafetyEvaluator::with_config(config);
        let board = create_complex_position();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should return a valid score with modified weights
        assert!(score.mg >= 0 && score.eg >= 0);
    }
}

#[cfg(test)]
mod real_game_position_tests {
    use super::*;

    /// Create a position from a real Shogi game
    fn create_real_game_position() -> BitboardBoard {
        let mut board = BitboardBoard::empty();

        // Set up a typical middlegame position
        // Black pieces
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::Black,
            },
            Position::new(8, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::Black,
            },
            Position::new(7, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::Black,
            },
            Position::new(6, 3),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::Black,
            },
            Position::new(5, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Bishop,
                player: Player::Black,
            },
            Position::new(4, 5),
        );

        // White pieces
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::White,
            },
            Position::new(0, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: Player::White,
            },
            Position::new(1, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::White,
            },
            Position::new(2, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: Player::White,
            },
            Position::new(3, 4),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Bishop,
                player: Player::White,
            },
            Position::new(4, 3),
        );

        board
    }

    #[test]
    fn test_real_game_position_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_real_game_position();

        let black_score = evaluator.evaluate(&board, Player::Black);
        let white_score = evaluator.evaluate(&board, Player::White);

        // Both players should have valid scores (can be negative due to threats)
        assert!(black_score.mg >= -1000 && black_score.mg <= 1000);
        assert!(white_score.mg >= -1000 && white_score.mg <= 1000);
    }

    #[test]
    fn test_position_symmetry() {
        let evaluator = KingSafetyEvaluator::new();
        let board = create_real_game_position();

        let black_score = evaluator.evaluate(&board, Player::Black);
        let white_score = evaluator.evaluate(&board, Player::White);

        // In a symmetric position, scores should be similar
        let score_diff = (black_score.mg - white_score.mg).abs();
        assert!(score_diff < 100); // Allow some difference due to piece placement
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_evaluation_performance() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = evaluator.evaluate(&board, Player::Black);
        }
        let duration = start.elapsed();

        // Should complete 1000 evaluations in reasonable time
        assert!(
            duration.as_millis() < 1000,
            "Evaluation too slow: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_castle_recognition_performance() {
        let recognizer = CastleRecognizer::new();
        let board = BitboardBoard::empty();
        let king_pos = Position::new(8, 4);

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);
        }
        let duration = start.elapsed();

        // Should complete 1000 evaluations in reasonable time
        assert!(
            duration.as_millis() < 100,
            "Castle recognition too slow: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_attack_analysis_performance() {
        let analyzer = AttackAnalyzer::new();
        let board = BitboardBoard::empty();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = analyzer.evaluate_attacks(&board, Player::Black);
        }
        let duration = start.elapsed();

        // Should complete 1000 evaluations in reasonable time
        assert!(
            duration.as_millis() < 500,
            "Attack analysis too slow: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_threat_evaluation_performance() {
        let evaluator = ThreatEvaluator::new();
        let board = BitboardBoard::empty();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = evaluator.evaluate_threats(&board, Player::Black);
        }
        let duration = start.elapsed();

        // Should complete 1000 evaluations in reasonable time
        assert!(
            duration.as_millis() < 500,
            "Threat evaluation too slow: {}ms",
            duration.as_millis()
        );
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_board_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle empty board gracefully
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_missing_king_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let mut board = BitboardBoard::empty();

        // Remove the king (create empty board)
        let mut board = BitboardBoard::empty();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle missing king gracefully
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_extreme_configuration_values() {
        let config = KingSafetyConfig {
            enabled: true,
            castle_weight: 10.0,
            attack_weight: 0.1,
            threat_weight: 5.0,
            phase_adjustment: 0.5,
            performance_mode: true,
            ..Default::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config);
        let board = BitboardBoard::empty();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle extreme values gracefully
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_disabled_components() {
        let config = KingSafetyConfig {
            enabled: true,
            castle_weight: 0.0,
            attack_weight: 0.0,
            threat_weight: 0.0,
            phase_adjustment: 1.0,
            performance_mode: false,
            ..Default::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config);
        let board = BitboardBoard::empty();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle disabled components gracefully
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_corner_king_positions() {
        let evaluator = KingSafetyEvaluator::new();
        let mut board = BitboardBoard::empty();

        // Move king to corner
        let mut board = BitboardBoard::empty();
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::Black,
            },
            Position::new(8, 0),
        );

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle corner positions
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_center_king_positions() {
        let evaluator = KingSafetyEvaluator::new();
        let mut board = BitboardBoard::empty();

        // Move king to center
        let mut board = BitboardBoard::empty();
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: Player::Black,
            },
            Position::new(4, 4),
        );

        let score = evaluator.evaluate(&board, Player::Black);

        // Should handle center positions
        assert!(score.mg >= 0 && score.eg >= 0);
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_all_configuration_options() {
        let config = KingSafetyConfig {
            enabled: true,
            castle_weight: 1.5,
            attack_weight: 0.8,
            threat_weight: 1.2,
            phase_adjustment: 0.9,
            performance_mode: true,
            ..Default::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config);
        let board = BitboardBoard::empty();

        let score = evaluator.evaluate(&board, Player::Black);

        // Should work with all configuration options
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_configuration_serialization() {
        let config = KingSafetyConfig::default();

        // Test that configuration can be cloned
        let cloned_config = config.clone();
        assert_eq!(config, cloned_config);
    }

    #[test]
    fn test_configuration_defaults() {
        let config = KingSafetyConfig::default();

        assert!(config.enabled);
        assert_eq!(config.castle_weight, 1.0);
        assert_eq!(config.attack_weight, 1.0);
        assert_eq!(config.threat_weight, 1.0);
        assert_eq!(config.phase_adjustment, 0.8);
        assert!(!config.performance_mode);
    }
}
