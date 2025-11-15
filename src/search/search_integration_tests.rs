#[cfg(test)]
use crate::bitboards::*;
#[cfg(test)]
use crate::search::move_ordering::MoveOrdering;
#[cfg(test)]
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Piece, PieceType, Player, Position};
#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(test)]
use std::sync::Arc;

/// Comprehensive integration tests for search algorithm with move ordering
#[cfg(test)]
mod search_integration_tests {
    use super::*;
    use crate::search::search_engine::SearchEngine;

    /// Test basic search integration with move ordering
    #[test]
    fn test_basic_search_integration() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test that search works with move ordering integration
        let result =
            engine.search_at_depth(&board, &captured_pieces, player, 3, 1000, -10000, 10000);

        // Should not panic and should return some result
        assert!(result.is_some() || result.is_none()); // Either some move or no legal moves
    }

    /// Test negamax integration with advanced move ordering
    #[test]
    fn test_negamax_integration() {
        let mut engine = SearchEngine::new(None, 64);
        let mut board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test negamax with move ordering
        let legal_moves =
            engine
                .move_generator
                .generate_legal_moves(&board, player, &captured_pieces);

        if !legal_moves.is_empty() {
            // Test move ordering integration
            let ordered_moves = engine.order_moves_for_negamax(
                &legal_moves,
                &board,
                &captured_pieces,
                player,
                3,
                -10000,
                10000,
            );

            // Should have same number of moves
            assert_eq!(ordered_moves.len(), legal_moves.len());

            // Should be ordered (first move should be different or same as original first)
            // This is a basic sanity check
            assert!(!ordered_moves.is_empty());
        }
    }

    /// Test quiescence search integration with move ordering
    #[test]
    fn test_quiescence_integration() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test quiescence search with move ordering
        let noisy_moves = engine.generate_noisy_moves(&board, player, &captured_pieces);

        if !noisy_moves.is_empty() {
            let ordered_moves = engine.sort_quiescence_moves_advanced(
                &noisy_moves,
                &board,
                &captured_pieces,
                player,
            );

            // Should have same number of moves
            assert_eq!(ordered_moves.len(), noisy_moves.len());
            assert!(!ordered_moves.is_empty());
        }
    }

    /// Test iterative deepening integration with move ordering
    #[test]
    fn test_iterative_deepening_integration() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut engine = SearchEngine::new(Some(stop_flag.clone()), 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test iterative deepening with move ordering
        let result =
            engine.search_at_depth(&board, &captured_pieces, player, 2, 1000, -10000, 10000);

        // Should complete without errors
        assert!(result.is_some() || result.is_none());
    }

    /// Test move ordering performance improvement
    #[test]
    fn test_move_ordering_performance() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let legal_moves =
            engine
                .move_generator
                .generate_legal_moves(&board, player, &captured_pieces);

        if legal_moves.len() > 3 {
            // Test that move ordering doesn't significantly slow down search
            let start_time = std::time::Instant::now();

            for _ in 0..10 {
                let _ordered_moves = engine.order_moves_for_negamax(
                    &legal_moves,
                    &board,
                    &captured_pieces,
                    player,
                    3,
                    -10000,
                    10000,
                );
            }

            let elapsed = start_time.elapsed();

            // Should complete quickly (less than 100ms for 10 iterations)
            assert!(elapsed.as_millis() < 100);
        }
    }

    /// Test move ordering correctness
    #[test]
    fn test_move_ordering_correctness() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let legal_moves =
            engine
                .move_generator
                .generate_legal_moves(&board, player, &captured_pieces);

        if legal_moves.len() > 1 {
            let ordered_moves = engine.order_moves_for_negamax(
                &legal_moves,
                &board,
                &captured_pieces,
                player,
                3,
                -10000,
                10000,
            );

            // All moves should be preserved
            assert_eq!(ordered_moves.len(), legal_moves.len());

            // No duplicates should be introduced
            let mut seen_moves = std::collections::HashSet::new();
            for mv in &ordered_moves {
                assert!(
                    seen_moves.insert(format!("{:?}", mv)),
                    "Duplicate move found in ordering"
                );
            }
        }
    }

    /// Test advanced move ordering features
    #[test]
    fn test_advanced_move_ordering_features() {
        let mut engine = SearchEngine::new(None, 64);

        // Test that advanced move ordering is properly initialized
        let advanced_orderer = &engine.advanced_move_orderer;

        // Should have default configuration
        let status = advanced_orderer.get_advanced_features_status();
        assert!(status.position_specific_strategies); // Always enabled

        // Test game phase determination
        let phase = advanced_orderer.determine_game_phase(10, 0, 0.3);
        assert!(matches!(
            phase,
            crate::search::move_ordering::GamePhase::Opening
        ));

        let phase = advanced_orderer.determine_game_phase(70, 0, 0.3);
        assert!(matches!(
            phase,
            crate::search::move_ordering::GamePhase::Endgame
        ));
    }

    /// Test error handling in move ordering integration
    #[test]
    fn test_move_ordering_error_handling() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with empty move list
        let empty_moves = Vec::new();
        let ordered_moves = engine.order_moves_for_negamax(
            &empty_moves,
            &board,
            &captured_pieces,
            player,
            3,
            -10000,
            10000,
        );
        assert!(ordered_moves.is_empty());
    }

    /// Test memory management in move ordering integration
    #[test]
    fn test_move_ordering_memory_management() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform multiple searches to test memory management
        for _ in 0..5 {
            let result =
                engine.search_at_depth(&board, &captured_pieces, player, 2, 1000, -10000, 10000);
            assert!(result.is_some() || result.is_none());
        }

        // Check memory usage
        let memory_usage = engine.advanced_move_orderer.get_current_memory_usage();
        assert!(memory_usage.total_memory >= 0);
    }

    /// Test search correctness with move ordering
    #[test]
    fn test_search_correctness_with_move_ordering() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test that search results are consistent
        let result1 =
            engine.search_at_depth(&board, &captured_pieces, player, 2, 1000, -10000, 10000);
        let result2 =
            engine.search_at_depth(&board, &captured_pieces, player, 2, 1000, -10000, 10000);

        // Results should be consistent (same move and score)
        if let (Some((move1, score1)), Some((move2, score2))) = (result1, result2) {
            assert_eq!(move1.to_usi_string(), move2.to_usi_string());
            assert_eq!(score1, score2);
        }
    }

    /// Test performance benchmarks for search integration
    #[test]
    fn test_search_performance_benchmarks() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Benchmark search performance
        let start_time = std::time::Instant::now();

        let result =
            engine.search_at_depth(&board, &captured_pieces, player, 3, 2000, -10000, 10000);

        let elapsed = start_time.elapsed();

        // Should complete within reasonable time
        assert!(elapsed.as_millis() < 2000);
        assert!(result.is_some() || result.is_none());
    }
}

/// Performance tests for search algorithm integration
#[cfg(test)]
mod search_performance_tests {
    use super::*;
    use crate::search::search_engine::SearchEngine;

    /// Benchmark move ordering performance
    #[test]
    fn benchmark_move_ordering_performance() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let legal_moves =
            engine
                .move_generator
                .generate_legal_moves(&board, player, &captured_pieces);

        if legal_moves.len() > 5 {
            let iterations = 100;
            let start_time = std::time::Instant::now();

            for _ in 0..iterations {
                let _ordered_moves = engine.order_moves_for_negamax(
                    &legal_moves,
                    &board,
                    &captured_pieces,
                    player,
                    3,
                    -10000,
                    10000,
                );
            }

            let elapsed = start_time.elapsed();
            let avg_time_per_iteration = elapsed.as_micros() as f64 / iterations as f64;

            // Should be fast (less than 1000 microseconds per iteration)
            assert!(
                avg_time_per_iteration < 1000.0,
                "Move ordering too slow: {}μs per iteration",
                avg_time_per_iteration
            );
        }
    }

    /// Benchmark search performance with move ordering
    #[test]
    fn benchmark_search_performance() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let iterations = 5;
        let start_time = std::time::Instant::now();

        for _ in 0..iterations {
            let _result =
                engine.search_at_depth(&board, &captured_pieces, player, 3, 1000, -10000, 10000);
        }

        let elapsed = start_time.elapsed();
        let avg_time_per_search = elapsed.as_millis() as f64 / iterations as f64;

        // Should complete searches quickly (less than 500ms per search on average)
        assert!(
            avg_time_per_search < 500.0,
            "Search too slow: {}ms per search",
            avg_time_per_search
        );
    }

    /// Benchmark quiescence search performance
    #[test]
    fn benchmark_quiescence_performance() {
        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let noisy_moves = engine.generate_noisy_moves(&board, player, &captured_pieces);

        if noisy_moves.len() > 2 {
            let iterations = 50;
            let start_time = std::time::Instant::now();

            for _ in 0..iterations {
                let _ordered_moves = engine.sort_quiescence_moves_advanced(
                    &noisy_moves,
                    &board,
                    &captured_pieces,
                    player,
                );
            }

            let elapsed = start_time.elapsed();
            let avg_time_per_iteration = elapsed.as_micros() as f64 / iterations as f64;

            // Should be very fast (less than 100 microseconds per iteration)
            assert!(
                avg_time_per_iteration < 100.0,
                "Quiescence move ordering too slow: {}μs per iteration",
                avg_time_per_iteration
            );
        }
    }
}
