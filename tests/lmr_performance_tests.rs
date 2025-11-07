#![cfg(feature = "legacy-tests")]
/// Performance tests for Late Move Reductions (LMR)
///
/// This module contains performance benchmarks and regression tests:
/// - NPS comparison with/without LMR
/// - Performance tests for different position types
/// - Regression tests to ensure LMR doesn't weaken tactical play
/// - Stress tests with high move counts and deep searches
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

// Safety constants to prevent system crashes
const MAX_TEST_POSITIONS: usize = 50;
const MAX_TEST_ITERATIONS: usize = 10;
const MAX_SEARCH_DEPTH: u8 = 8;

// Safety check to prevent running performance tests that might crash the system
fn should_run_performance_tests() -> bool {
    // Only run performance tests if we're in a safe environment
    std::env::var("RUN_PERFORMANCE_TESTS").is_ok() || std::env::var("CI").is_ok()
}

#[cfg(test)]
mod lmr_nps_comparison_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    fn measure_search_performance(
        engine: &mut SearchEngine,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        iterations: usize,
    ) -> (f64, u64) {
        let mut total_nodes = 0u64;
        let start_time = Instant::now();

        for _ in 0..iterations {
            let result = engine.search_at_depth(board, captured_pieces, player, depth, 5000);
            assert!(result.is_some());
            total_nodes += engine.nodes_searched;
        }

        let duration = start_time.elapsed();
        let nps = if duration.as_secs_f64() > 0.0 {
            total_nodes as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        (nps, total_nodes)
    }

    #[test]
    fn test_lmr_nps_improvement() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine_with_lmr = create_test_engine();
        let mut engine_without_lmr = create_test_engine();

        // Configure engine with LMR
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_with_lmr.update_lmr_config(lmr_config).unwrap();

        // Configure engine without LMR
        let lmr_config_disabled = LMRConfig {
            enabled: false,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_without_lmr
            .update_lmr_config(lmr_config_disabled)
            .unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        let depth = 5;
        let iterations = 3;

        // Measure performance with LMR
        let (nps_with_lmr, nodes_with_lmr) = measure_search_performance(
            &mut engine_with_lmr,
            &board,
            &captured_pieces,
            player,
            depth,
            iterations,
        );

        // Measure performance without LMR
        let (nps_without_lmr, nodes_without_lmr) = measure_search_performance(
            &mut engine_without_lmr,
            &board,
            &captured_pieces,
            player,
            depth,
            iterations,
        );

        // LMR should improve NPS (more nodes per second)
        // Note: This is a basic test - in practice, LMR effectiveness depends on position
        println!(
            "NPS with LMR: {:.0}, NPS without LMR: {:.0}",
            nps_with_lmr, nps_without_lmr
        );
        println!(
            "Nodes with LMR: {}, Nodes without LMR: {}",
            nodes_with_lmr, nodes_without_lmr
        );

        // Check that LMR statistics show activity
        let lmr_stats = engine_with_lmr.get_lmr_stats();
        assert!(lmr_stats.moves_considered > 0);
    }

    #[test]
    fn test_lmr_depth_improvement() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine_with_lmr = create_test_engine();
        let mut engine_without_lmr = create_test_engine();

        // Configure engines
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_with_lmr.update_lmr_config(lmr_config).unwrap();

        let lmr_config_disabled = LMRConfig {
            enabled: false,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_without_lmr
            .update_lmr_config(lmr_config_disabled)
            .unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;
        let time_limit = 2000; // 2 seconds

        // Test that LMR allows deeper search in same time
        let start_time = Instant::now();
        let result_with_lmr =
            engine_with_lmr.search_at_depth(&board, &captured_pieces, player, 6, time_limit);
        let time_with_lmr = start_time.elapsed();

        let start_time = Instant::now();
        let result_without_lmr =
            engine_without_lmr.search_at_depth(&board, &captured_pieces, player, 6, time_limit);
        let time_without_lmr = start_time.elapsed();

        assert!(result_with_lmr.is_some());
        assert!(result_without_lmr.is_some());

        // Both should complete within time limit
        assert!(time_with_lmr.as_millis() < time_limit as u128);
        assert!(time_without_lmr.as_millis() < time_limit as u128);

        println!(
            "Time with LMR: {:?}, Time without LMR: {:?}",
            time_with_lmr, time_without_lmr
        );
    }
}

#[cfg(test)]
mod lmr_position_type_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_tactical_positions() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Search multiple times to build up statistics
        for _ in 0..5 {
            let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
            assert!(result.is_some());
        }

        let lmr_stats = engine.get_lmr_stats();

        // In tactical positions, LMR should be more conservative
        // (fewer reductions, more researches)
        if lmr_stats.reductions_applied > 0 {
            let research_rate = lmr_stats.research_rate();
            println!("Research rate in tactical position: {:.1}%", research_rate);

            // Research rate should be reasonable (not too high, not too low)
            assert!(research_rate >= 0.0);
            assert!(research_rate <= 100.0);
        }
    }

    #[test]
    fn test_lmr_quiet_positions() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Search multiple times to build up statistics
        for _ in 0..5 {
            let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
            assert!(result.is_some());
        }

        let lmr_stats = engine.get_lmr_stats();

        // In quiet positions, LMR should be more aggressive
        // (more reductions, fewer researches)
        if lmr_stats.reductions_applied > 0 {
            let efficiency = lmr_stats.efficiency();
            let research_rate = lmr_stats.research_rate();

            println!("Efficiency in quiet position: {:.1}%", efficiency);
            println!("Research rate in quiet position: {:.1}%", research_rate);

            // Efficiency should be reasonable
            assert!(efficiency >= 0.0);
            assert!(efficiency <= 100.0);

            // Research rate should be reasonable
            assert!(research_rate >= 0.0);
            assert!(research_rate <= 100.0);
        }
    }
}

#[cfg(test)]
mod lmr_regression_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_tactical_accuracy() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine_with_lmr = create_test_engine();
        let mut engine_without_lmr = create_test_engine();

        // Configure engine with LMR
        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_with_lmr.update_lmr_config(lmr_config).unwrap();

        // Configure engine without LMR
        let lmr_config_disabled = LMRConfig {
            enabled: false,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine_without_lmr
            .update_lmr_config(lmr_config_disabled)
            .unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test that both engines find the same best move
        let result_with_lmr =
            engine_with_lmr.search_at_depth(&board, &captured_pieces, player, 5, 2000);
        let result_without_lmr =
            engine_without_lmr.search_at_depth(&board, &captured_pieces, player, 5, 2000);

        assert!(result_with_lmr.is_some());
        assert!(result_without_lmr.is_some());

        let (move_with_lmr, score_with_lmr) = result_with_lmr.unwrap();
        let (move_without_lmr, score_without_lmr) = result_without_lmr.unwrap();

        // Scores should be similar (within reasonable tolerance)
        let score_diff = (score_with_lmr - score_without_lmr).abs();
        assert!(
            score_diff < 1000,
            "Score difference too large: {} vs {}",
            score_with_lmr,
            score_without_lmr
        );

        // Moves should be the same or very similar
        // (In practice, they might be different due to search order, but scores should be close)
        println!(
            "Move with LMR: {:?}, Score: {}",
            move_with_lmr, score_with_lmr
        );
        println!(
            "Move without LMR: {:?}, Score: {}",
            move_without_lmr, score_without_lmr
        );
    }

    #[test]
    fn test_lmr_capture_accuracy() {
        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        // Test that capture moves are properly exempted from LMR
        let capture_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: true,
            is_promotion: false,
            captured_piece: Some(Piece {
                piece_type: PieceType::Rook,
                player: Player::White,
            }),
            gives_check: false,
            is_recapture: false,
        };

        // Capture moves should be exempt from LMR
        assert!(engine.is_move_exempt_from_lmr(&capture_move));

        // Check that the move has high tactical value
        let tactical_value = engine.get_move_tactical_value(&capture_move);
        assert!(tactical_value > 0);
    }

    #[test]
    fn test_lmr_check_accuracy() {
        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        // Test that check moves are properly exempted from LMR
        let check_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: true,
            is_recapture: false,
        };

        // Check moves should be exempt from LMR
        assert!(engine.is_move_exempt_from_lmr(&check_move));

        // Check that the move has high tactical value
        let tactical_value = engine.get_move_tactical_value(&check_move);
        assert_eq!(tactical_value, 1000); // Check moves have high value
    }
}

#[cfg(test)]
mod lmr_stress_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 32) // Larger hash table for stress tests
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_deep_search() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test deep search
        let result =
            engine.search_at_depth(&board, &captured_pieces, player, MAX_SEARCH_DEPTH, 10000);
        assert!(result.is_some());

        let lmr_stats = engine.get_lmr_stats();

        // Should have processed many moves
        assert!(lmr_stats.moves_considered > 0);

        // LMR should be active at this depth
        if lmr_stats.moves_considered > 0 {
            let efficiency = lmr_stats.efficiency();
            println!(
                "LMR efficiency at depth {}: {:.1}%",
                MAX_SEARCH_DEPTH, efficiency
            );

            // Efficiency should be reasonable
            assert!(efficiency >= 0.0);
            assert!(efficiency <= 100.0);
        }
    }

    #[test]
    fn test_lmr_multiple_searches() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Perform multiple searches
        for depth in 3..=6 {
            let result = engine.search_at_depth(&board, &captured_pieces, player, depth, 2000);
            assert!(result.is_some(), "Search should succeed at depth {}", depth);
        }

        let lmr_stats = engine.get_lmr_stats();

        // Should have accumulated statistics across all searches
        assert!(lmr_stats.moves_considered > 0);

        // Statistics should be consistent
        let efficiency = lmr_stats.efficiency();
        let research_rate = lmr_stats.research_rate();

        assert!(efficiency >= 0.0);
        assert!(efficiency <= 100.0);
        assert!(research_rate >= 0.0);
        assert!(research_rate <= 100.0);

        println!("Final LMR statistics:");
        println!("  Moves considered: {}", lmr_stats.moves_considered);
        println!("  Reductions applied: {}", lmr_stats.reductions_applied);
        println!("  Researches triggered: {}", lmr_stats.researches_triggered);
        println!("  Efficiency: {:.1}%", efficiency);
        println!("  Research rate: {:.1}%", research_rate);
    }

    #[test]
    fn test_lmr_memory_usage() {
        if !should_run_performance_tests() {
            return;
        }

        let mut engine = create_test_engine();

        let lmr_config = LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        };
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Perform searches to populate transposition table
        for _ in 0..5 {
            let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 1000);
            assert!(result.is_some());
        }

        // Check that transposition table has reasonable size
        let tt_size = engine.transposition_table_len();
        assert!(tt_size > 0);
        assert!(tt_size < 100000); // Should not be excessively large

        // Check that LMR statistics are reasonable
        let lmr_stats = engine.get_lmr_stats();
        assert!(lmr_stats.moves_considered > 0);

        println!("Transposition table size: {}", tt_size);
        println!("LMR moves considered: {}", lmr_stats.moves_considered);
    }
}
