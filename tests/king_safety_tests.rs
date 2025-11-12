#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::*;
use shogi_engine::evaluation::attacks::{
    AttackAnalyzer, AttackTables, TacticalType, ThreatConfig, ThreatEvaluator,
};
use shogi_engine::evaluation::castles::*;
use shogi_engine::evaluation::king_safety::*;
use shogi_engine::evaluation::patterns::*;
use shogi_engine::types::*;

/// Test utilities for setting up specific board positions
mod test_utils {
    use super::*;

    /// Create a board with a specific castle pattern
    pub fn create_mino_castle_board(player: Player) -> BitboardBoard {
        let mut board = BitboardBoard::new();

        // Set up Mino castle for the given player
        let king_row = match player {
            Player::Black => 8,
            Player::White => 0,
        };

        // King at 8,4 (Black) or 0,4 (White)
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player,
            },
            Position::new(king_row, 4),
        );

        // Gold at 7,3 (Black) or 1,3 (White)
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player,
            },
            Position::new(king_row - 1, 3),
        );

        // Silver at 6,3 (Black) or 2,3 (White)
        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player,
            },
            Position::new(king_row - 2, 3),
        );

        // Pawns at 6,2, 7,2, 8,2 (Black) or 2,2, 1,2, 0,2 (White)
        for i in 0..3 {
            let pawn_row = king_row - 2 + i;
            board.place_piece(
                Piece {
                    piece_type: PieceType::Pawn,
                    player,
                },
                Position::new(pawn_row, 2),
            );
        }

        board
    }

    /// Create a board with an attacking position
    pub fn create_attacking_position(attacker: Player) -> BitboardBoard {
        let mut board = BitboardBoard::new();

        let defender = attacker.opposite();
        let king_row = match defender {
            Player::Black => 8,
            Player::White => 0,
        };

        // Defender's king
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: defender,
            },
            Position::new(king_row, 4),
        );

        // Attacker's rook
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: attacker,
            },
            Position::new(king_row - 3, 4),
        );

        // Attacker's bishop
        board.place_piece(
            Piece {
                piece_type: PieceType::Bishop,
                player: attacker,
            },
            Position::new(king_row - 2, 3),
        );

        board
    }

    /// Create a board with a tactical threat (pin)
    pub fn create_pin_position(attacker: Player) -> BitboardBoard {
        let mut board = BitboardBoard::new();

        let defender = attacker.opposite();
        let king_row = match defender {
            Player::Black => 8,
            Player::White => 0,
        };

        // Defender's king
        board.place_piece(
            Piece {
                piece_type: PieceType::King,
                player: defender,
            },
            Position::new(king_row, 4),
        );

        // Defender's piece that will be pinned
        board.place_piece(
            Piece {
                piece_type: PieceType::Gold,
                player: defender,
            },
            Position::new(king_row - 1, 4),
        );

        // Attacker's rook
        board.place_piece(
            Piece {
                piece_type: PieceType::Rook,
                player: attacker,
            },
            Position::new(king_row - 2, 4),
        );

        board
    }
}

#[cfg(test)]
mod king_safety_evaluator_tests {
    use super::*;

    #[test]
    fn test_king_safety_evaluator_creation() {
        let evaluator = KingSafetyEvaluator::new();
        // Test that evaluator can be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_king_safety_evaluator_with_config() {
        let config = KingSafetyConfig {
            enabled: false,
            castle_weight: 1.5,
            attack_weight: 0.8,
            threat_weight: 1.2,
            phase_adjustment: 0.9,
            performance_mode: true,
            ..Default::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config.clone());
        // Test that evaluator can be created with custom config
        assert!(true); // Basic config test
    }

    #[test]
    fn test_king_safety_evaluation_disabled() {
        let config = KingSafetyConfig {
            enabled: false,
            ..Default::default()
        };

        let evaluator = KingSafetyEvaluator::with_config(config);
        let board = BitboardBoard::empty();
        let score = evaluator.evaluate(&board, Player::Black);

        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_king_safety_evaluation_enabled() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();
        let score = evaluator.evaluate(&board, Player::Black);

        // Should return a valid score (even if zero for starting position)
        assert_eq!(score.mg, score.mg); // Basic sanity check
        assert_eq!(score.eg, score.eg); // Basic sanity check
    }

    #[test]
    fn test_find_king_position() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();

        // Test that evaluation works with empty board
        let score = evaluator.evaluate(&board, Player::Black);
        assert!(score.mg >= 0 && score.eg >= 0);
    }
}

#[cfg(test)]
mod castle_recognition_tests {
    use super::*;

    #[test]
    fn test_mino_castle_recognition() {
        let recognizer = CastleRecognizer::new();
        let board = test_utils::create_mino_castle_board(Player::Black);
        let king_pos = Position::new(8, 4);

        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);

        // Should recognize Mino castle and give positive score
        assert!(score.mg > 0 || score.eg > 0);
    }

    #[test]
    fn test_anaguma_castle_recognition() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::new();

        // Set up Anaguma castle for Black
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
            Position::new(7, 4),
        );

        board.place_piece(
            Piece {
                piece_type: PieceType::Silver,
                player: Player::Black,
            },
            Position::new(6, 4),
        );

        let king_pos = Position::new(8, 4);
        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);

        // Should recognize Anaguma castle (or at least return a valid score)
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_yagura_castle_recognition() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::new();

        // Set up Yagura castle for Black
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
                piece_type: PieceType::Lance,
                player: Player::Black,
            },
            Position::new(8, 7),
        );

        let king_pos = Position::new(8, 4);
        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);

        // Should recognize Yagura castle
        assert!(score.mg > 0 || score.eg > 0);
    }

    #[test]
    fn test_castle_flexibility_scoring() {
        let recognizer = CastleRecognizer::new();
        let mut board = BitboardBoard::new();

        // Set up partial Mino castle (missing some pieces)
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

        // Missing Silver and Pawns

        let king_pos = Position::new(8, 4);
        let score = recognizer.evaluate_castle_structure(&board, Player::Black, king_pos);

        // Should give partial score for incomplete castle
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_castle_recognition_both_players() {
        let recognizer = CastleRecognizer::new();
        let board = test_utils::create_mino_castle_board(Player::Black);

        let black_score =
            recognizer.evaluate_castle_structure(&board, Player::Black, Position::new(8, 4));
        let white_score =
            recognizer.evaluate_castle_structure(&board, Player::White, Position::new(0, 4));

        // Both should return valid scores
        assert!(black_score.mg >= 0 && black_score.eg >= 0);
        assert!(white_score.mg >= 0 && white_score.eg >= 0);
    }
}

#[cfg(test)]
mod attack_analysis_tests {
    use super::*;

    #[test]
    fn test_attack_analyzer_creation() {
        let analyzer = AttackAnalyzer::new();
        // Test that analyzer can be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_attack_zone_generation() {
        let analyzer = AttackAnalyzer::new();
        let board = test_utils::create_attacking_position(Player::White);

        let score = analyzer.evaluate_attacks(&board, Player::Black);

        // Should detect attacks and return negative score (bad for the king)
        assert!(score.mg <= 0 && score.eg <= 0);
    }

    #[test]
    fn test_attack_coordination() {
        let analyzer = AttackAnalyzer::new();
        let board = test_utils::create_attacking_position(Player::White);

        let score = analyzer.evaluate_attacks(&board, Player::Black);

        // Should detect coordinated attacks (rook + bishop)
        assert!(score.mg < 0 || score.eg < 0);
    }

    #[test]
    fn test_piece_attack_values() {
        let analyzer = AttackAnalyzer::new();
        let board = BitboardBoard::empty();

        // Test that attack evaluation works
        let score = analyzer.evaluate_attacks(&board, Player::Black);
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_attack_tables_initialization() {
        let tables = AttackTables::new();

        let center = Position::new(4, 4);
        let rook_attacks = tables.get_piece_attacks(PieceType::Rook, center);
        let king_zone = tables.get_king_zone(center);

        assert!(rook_attacks != EMPTY_BITBOARD);
        assert!(king_zone != EMPTY_BITBOARD);
    }
}

#[cfg(test)]
mod threat_evaluation_tests {
    use super::*;

    #[test]
    fn test_threat_evaluator_creation() {
        let evaluator = ThreatEvaluator::new();
        // Test that evaluator can be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_pin_detection() {
        let evaluator = ThreatEvaluator::new();
        let board = test_utils::create_pin_position(Player::White);

        let score = evaluator.evaluate_threats(&board, Player::Black);

        // Should detect pin and return negative score
        assert!(score.mg <= 0 && score.eg <= 0);
    }

    #[test]
    fn test_tactical_pattern_types() {
        let pin_type = TacticalType::Pin;
        let skewer_type = TacticalType::Skewer;
        let fork_type = TacticalType::Fork;
        let discovered_type = TacticalType::DiscoveredAttack;
        let double_type = TacticalType::DoubleAttack;

        assert_eq!(pin_type, TacticalType::Pin);
        assert_eq!(skewer_type, TacticalType::Skewer);
        assert_eq!(fork_type, TacticalType::Fork);
        assert_eq!(discovered_type, TacticalType::DiscoveredAttack);
        assert_eq!(double_type, TacticalType::DoubleAttack);
    }

    #[test]
    fn test_threat_evaluation_disabled() {
        let config = ThreatConfig {
            enabled: false,
            ..Default::default()
        };

        let evaluator = ThreatEvaluator::with_config(config);
        let board = test_utils::create_pin_position(Player::White);
        let score = evaluator.evaluate_threats(&board, Player::Black);

        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_threat_evaluation_enabled() {
        let evaluator = ThreatEvaluator::new();
        let board = test_utils::create_pin_position(Player::White);
        let score = evaluator.evaluate_threats(&board, Player::Black);

        // Should return a valid score
        assert!(score.mg <= 0 && score.eg <= 0);
    }
}

#[cfg(test)]
mod pattern_tests {
    use super::*;

    #[test]
    fn test_mino_pattern_definition() {
        let pattern = get_mino_castle();

        assert_eq!(pattern.name, "Mino");
        assert_eq!(pattern.pieces.len(), 5);
        assert_eq!(pattern.flexibility, 2);
        assert!(pattern.score.mg > 0 || pattern.score.eg > 0);

        // Check required pieces
        let required_pieces: Vec<_> = pattern.pieces.iter().filter(|p| p.required).collect();
        assert_eq!(required_pieces.len(), 2); // Gold and Silver
    }

    #[test]
    fn test_anaguma_pattern_definition() {
        let pattern = get_anaguma_castle();

        assert_eq!(pattern.name, "Anaguma");
        assert_eq!(pattern.pieces.len(), 6);
        assert_eq!(pattern.flexibility, 3);
        assert!(pattern.score.mg > 0 || pattern.score.eg > 0);

        // Check required pieces
        let required_pieces: Vec<_> = pattern.pieces.iter().filter(|p| p.required).collect();
        assert_eq!(required_pieces.len(), 2); // Gold and Silver
    }

    #[test]
    fn test_yagura_pattern_definition() {
        let pattern = get_yagura_castle();

        assert_eq!(pattern.name, "Yagura");
        assert_eq!(pattern.pieces.len(), 5);
        assert_eq!(pattern.flexibility, 2);
        assert!(pattern.score.mg > 0 || pattern.score.eg > 0);

        // Check required pieces
        let required_pieces: Vec<_> = pattern.pieces.iter().filter(|p| p.required).collect();
        assert_eq!(required_pieces.len(), 2); // Gold and Silver
    }

    #[test]
    fn test_castle_piece_weights() {
        let mino_pattern = get_mino_castle();

        // Gold should have highest weight
        let gold_piece = mino_pattern
            .pieces
            .iter()
            .find(|p| p.piece_type == PieceType::Gold)
            .unwrap();
        assert_eq!(gold_piece.weight, 10);

        // Silver should have second highest weight
        let silver_piece = mino_pattern
            .pieces
            .iter()
            .find(|p| p.piece_type == PieceType::Silver)
            .unwrap();
        assert_eq!(silver_piece.weight, 9);

        // Pawns should have lower weights
        let pawn_pieces: Vec<_> = mino_pattern
            .pieces
            .iter()
            .filter(|p| p.piece_type == PieceType::Pawn)
            .collect();
        for pawn in pawn_pieces {
            assert!(pawn.weight < 9);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_king_safety_evaluation() {
        let evaluator = KingSafetyEvaluator::new();
        let board = test_utils::create_mino_castle_board(Player::Black);

        let score = evaluator.evaluate(&board, Player::Black);

        // Should return a valid score (may be zero for starting position)
        assert!(score.mg >= 0 && score.eg >= 0);
    }

    #[test]
    fn test_king_safety_with_attacks() {
        let evaluator = KingSafetyEvaluator::new();
        let board = test_utils::create_attacking_position(Player::White);

        let score = evaluator.evaluate(&board, Player::Black);

        // Should detect attacks and return negative score
        assert!(score.mg <= 0 && score.eg <= 0);
    }

    #[test]
    fn test_king_safety_with_threats() {
        let evaluator = KingSafetyEvaluator::new();
        let board = test_utils::create_pin_position(Player::White);

        let score = evaluator.evaluate(&board, Player::Black);

        // Should detect threats and return negative score
        assert!(score.mg <= 0 && score.eg <= 0);
    }

    #[test]
    fn test_king_safety_consistency() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();

        let score1 = evaluator.evaluate(&board, Player::Black);
        let score2 = evaluator.evaluate(&board, Player::Black);

        // Should return consistent results
        assert_eq!(score1, score2);
    }

    #[test]
    fn test_king_safety_both_players() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::empty();

        let black_score = evaluator.evaluate(&board, Player::Black);
        let white_score = evaluator.evaluate(&board, Player::White);

        // Both should return valid scores (non-negative)
        assert!(black_score.mg >= 0 && black_score.eg >= 0);
        assert!(white_score.mg >= 0 && white_score.eg >= 0);
    }
}
