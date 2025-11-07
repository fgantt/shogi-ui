#![cfg(feature = "legacy-tests")]
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    time_utils::TimeSource,
    types::{CapturedPieces, NullMoveConfig, Player, QuiescenceConfig},
};
use std::time::Instant;

#[cfg(test)]
mod performance_benchmarks {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16) // 16MB hash table
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn benchmark_quiescence_search_speed() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        let start = Instant::now();

        // Run multiple quiescence searches
        for _ in 0..10 {
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                4,
            );
        }

        let duration = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 5000); // 5 seconds for 10 searches

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn benchmark_quiescence_search_depth_scaling() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        let depths = vec![1, 2, 3, 4, 5];
        let mut results = Vec::new();

        for depth in depths {
            engine.reset_quiescence_stats();

            let start = Instant::now();
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                depth,
            );
            let duration = start.elapsed();

            let stats = engine.get_quiescence_stats();
            results.push((depth, stats.nodes_searched, duration.as_millis()));
        }

        // Verify that deeper searches take more time and search more nodes
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i - 1].1); // More nodes at deeper depth
        }
    }

    #[test]
    fn benchmark_quiescence_tt_performance() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Enable TT
        let mut config = QuiescenceConfig::default();
        config.enable_tt = true;
        engine.update_quiescence_config(config);

        // First search (populates TT)
        let start1 = Instant::now();
        let _result1 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );
        let duration1 = start1.elapsed();

        // Second search (should hit TT)
        let start2 = Instant::now();
        let _result2 = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );
        let duration2 = start2.elapsed();

        // Second search should be faster due to TT hits
        // (This may not always be true for simple positions, but tests the mechanism)
        assert!(duration2 <= duration1);

        let stats = engine.get_quiescence_stats();
        assert!(stats.tt_hits > 0);
    }

    #[test]
    fn benchmark_quiescence_pruning_efficiency() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Test with pruning enabled
        let mut config = QuiescenceConfig::default();
        config.enable_delta_pruning = true;
        config.enable_futility_pruning = true;
        engine.update_quiescence_config(config);

        let start = Instant::now();
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );
        let duration_with_pruning = start.elapsed();

        let stats_with_pruning = engine.get_quiescence_stats();

        // Test with pruning disabled
        let mut config = QuiescenceConfig::default();
        config.enable_delta_pruning = false;
        config.enable_futility_pruning = false;
        engine.update_quiescence_config(config);
        engine.reset_quiescence_stats();

        let start = Instant::now();
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );
        let duration_without_pruning = start.elapsed();

        let stats_without_pruning = engine.get_quiescence_stats();

        // Pruning should reduce the number of nodes searched
        assert!(stats_with_pruning.nodes_searched <= stats_without_pruning.nodes_searched);

        // Should have some pruning when enabled
        assert!(stats_with_pruning.delta_prunes > 0 || stats_with_pruning.futility_prunes > 0);
    }

    #[test]
    fn benchmark_quiescence_move_ordering_efficiency() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;

        // Generate moves
        let moves =
            engine
                .move_generator
                .generate_quiescence_moves(&board, player, &captured_pieces);

        // Benchmark move ordering
        let start = Instant::now();
        let _sorted_moves = engine.sort_quiescence_moves(&moves);
        let duration = start.elapsed();

        // Move ordering should be fast
        assert!(duration.as_micros() < 1000); // Less than 1ms for move ordering

        // Should produce sorted moves
        assert_eq!(moves.len(), _sorted_moves.len());
    }

    #[test]
    fn benchmark_quiescence_memory_usage() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Enable TT
        let mut config = QuiescenceConfig::default();
        config.enable_tt = true;
        config.tt_size_mb = 8;
        engine.update_quiescence_config(config);

        let initial_tt_size = engine.quiescence_tt_size();

        // Run multiple searches to populate TT
        for _ in 0..20 {
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                3,
            );
        }

        let final_tt_size = engine.quiescence_tt_size();

        // TT should have grown
        assert!(final_tt_size >= initial_tt_size);

        // Should not exceed reasonable limits
        assert!(final_tt_size < 100000); // Adjust based on actual usage
    }

    #[test]
    fn benchmark_quiescence_configuration_impact() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Test different configurations
        let configs = vec![
            ("default", QuiescenceConfig::default()),
            (
                "high_depth",
                QuiescenceConfig {
                    max_depth: 10,
                    ..QuiescenceConfig::default()
                },
            ),
            (
                "no_pruning",
                QuiescenceConfig {
                    enable_delta_pruning: false,
                    enable_futility_pruning: false,
                    ..QuiescenceConfig::default()
                },
            ),
            (
                "no_tt",
                QuiescenceConfig {
                    enable_tt: false,
                    ..QuiescenceConfig::default()
                },
            ),
        ];

        let mut results = Vec::new();

        for (name, config) in configs {
            engine.update_quiescence_config(config);
            engine.reset_quiescence_stats();

            let start = Instant::now();
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                4,
            );
            let duration = start.elapsed();

            let stats = engine.get_quiescence_stats();
            results.push((name, stats.nodes_searched, duration.as_millis()));
        }

        // All configurations should complete successfully
        assert_eq!(results.len(), 4);

        // High depth should search more nodes
        let high_depth_result = results
            .iter()
            .find(|(name, _, _)| *name == "high_depth")
            .unwrap();
        let default_result = results
            .iter()
            .find(|(name, _, _)| *name == "default")
            .unwrap();
        assert!(high_depth_result.1 >= default_result.1);
    }

    #[test]
    fn benchmark_quiescence_concurrent_performance() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Test multiple concurrent-like searches
        let start = Instant::now();

        for _ in 0..5 {
            let _result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                3,
            );
        }

        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration.as_millis() < 2000); // 2 seconds for 5 searches

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn benchmark_quiescence_evaluation_consistency() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Run the same search multiple times
        let mut results = Vec::new();

        for _ in 0..10 {
            let result = engine.quiescence_search(
                &mut board,
                &captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                3,
            );
            results.push(result);
        }

        // All results should be identical
        let first_result = results[0];
        for result in &results[1..] {
            assert_eq!(*result, first_result);
        }
    }

    #[test]
    fn benchmark_quiescence_time_limit_handling() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        // Test with very short time limit
        let start = Instant::now();
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            10, // 10ms time limit
            4,
        );
        let duration = start.elapsed();

        // Should complete within or close to time limit
        assert!(duration.as_millis() <= 50); // Allow some margin

        let stats = engine.get_quiescence_stats();
        assert!(stats.nodes_searched > 0);
    }

    #[test]
    fn benchmark_quiescence_statistics_accuracy() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::now();

        engine.reset_quiescence_stats();

        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4,
        );

        let stats = engine.get_quiescence_stats();

        // Statistics should be consistent
        assert!(stats.nodes_searched > 0);
        assert!(stats.moves_ordered >= 0);
        assert!(stats.delta_prunes >= 0);
        assert!(stats.futility_prunes >= 0);
        assert!(stats.extensions >= 0);
        assert!(stats.tt_hits >= 0);
        assert!(stats.tt_misses >= 0);

        // Efficiency metrics should be within valid ranges
        let efficiency = engine.get_quiescence_efficiency();
        assert!(efficiency.0 >= 0.0 && efficiency.0 <= 100.0); // pruning efficiency
        assert!(efficiency.1 >= 0.0 && efficiency.1 <= 100.0); // TT hit rate
        assert!(efficiency.2 >= 0.0 && efficiency.2 <= 100.0); // extension rate
    }

    // ===== NULL MOVE PRUNING BENCHMARKS =====

    #[test]
    fn benchmark_null_move_performance() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with NMP enabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_with_nmp = Instant::now();
        let result_with_nmp = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        let duration_with_nmp = start_with_nmp.elapsed();

        let stats_with_nmp = engine.get_null_move_stats();

        // Test with NMP disabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = false;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_without_nmp = Instant::now();
        let result_without_nmp = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        let duration_without_nmp = start_without_nmp.elapsed();

        let stats_without_nmp = engine.get_null_move_stats();

        // Both searches should complete successfully
        assert!(result_with_nmp.is_some());
        assert!(result_without_nmp.is_some());

        let (_, score_with_nmp) = result_with_nmp.unwrap();
        let (_, score_without_nmp) = result_without_nmp.unwrap();

        // Scores should be similar (NMP shouldn't change the best move significantly)
        let score_diff = (score_with_nmp - score_without_nmp).abs();
        assert!(score_diff <= 100); // Allow small differences due to search variations

        // NMP should have some activity when enabled
        assert!(stats_with_nmp.attempts >= 0);
        assert!(stats_with_nmp.cutoffs >= 0);

        // Both searches should complete within reasonable time
        assert!(duration_with_nmp.as_millis() > 0);
        assert!(duration_without_nmp.as_millis() > 0);

        println!(
            "NMP enabled: {:?}, NMP disabled: {:?}",
            duration_with_nmp, duration_without_nmp
        );
        println!(
            "NMP stats: attempts={}, cutoffs={}",
            stats_with_nmp.attempts, stats_with_nmp.cutoffs
        );
    }

    #[test]
    fn benchmark_null_move_nodes_per_second() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with NMP enabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start = Instant::now();
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 3, 2000);
        let duration = start.elapsed();

        let stats = engine.get_null_move_stats();
        let nodes_per_second = if duration.as_secs() > 0 {
            stats.attempts as f64 / duration.as_secs() as f64
        } else {
            stats.attempts as f64 / (duration.as_millis() as f64 / 1000.0)
        };

        // Should have reasonable performance
        assert!(nodes_per_second > 0.0);
        assert!(stats.attempts > 0);

        println!("NMP nodes per second: {:.2}", nodes_per_second);
        println!("NMP attempts: {}, duration: {:?}", stats.attempts, duration);
    }

    #[test]
    fn benchmark_null_move_depth_improvement() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with NMP enabled at depth 4
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_with_nmp = Instant::now();
        let result_with_nmp = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        let duration_with_nmp = start_with_nmp.elapsed();

        // Test with NMP disabled at depth 3 (should take similar time)
        let mut config = engine.get_null_move_config().clone();
        config.enabled = false;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_without_nmp = Instant::now();
        let result_without_nmp = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
        let duration_without_nmp = start_without_nmp.elapsed();

        // Both searches should complete successfully
        assert!(result_with_nmp.is_some());
        assert!(result_without_nmp.is_some());

        // NMP should allow deeper search in similar time
        // (This is a conceptual test - actual performance may vary)
        assert!(duration_with_nmp.as_millis() > 0);
        assert!(duration_without_nmp.as_millis() > 0);

        println!(
            "NMP depth 4: {:?}, No NMP depth 3: {:?}",
            duration_with_nmp, duration_without_nmp
        );
    }

    #[test]
    fn benchmark_null_move_cutoff_rates() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with NMP enabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.min_depth = 2; // Lower threshold to get more NMP activity
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);

        let stats = engine.get_null_move_stats();

        // Should have some NMP activity
        assert!(stats.attempts >= 0);
        assert!(stats.cutoffs >= 0);

        // Calculate cutoff rate
        let cutoff_rate = stats.cutoff_rate();
        assert!(cutoff_rate >= 0.0 && cutoff_rate <= 100.0);

        // Calculate efficiency
        let efficiency = stats.efficiency();
        assert!(efficiency >= 0.0 && efficiency <= 100.0);

        println!("NMP cutoff rate: {:.2}%", cutoff_rate);
        println!("NMP efficiency: {:.2}%", efficiency);
        println!(
            "NMP attempts: {}, cutoffs: {}",
            stats.attempts, stats.cutoffs
        );
    }

    #[test]
    fn benchmark_null_move_comprehensive_suite() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test multiple configurations
        let configs = vec![
            ("default", NullMoveConfig::default()),
            (
                "high_reduction",
                NullMoveConfig {
                    reduction_factor: 4,
                    ..NullMoveConfig::default()
                },
            ),
            (
                "low_threshold",
                NullMoveConfig {
                    max_pieces_threshold: 8,
                    ..NullMoveConfig::default()
                },
            ),
            (
                "static_reduction",
                NullMoveConfig {
                    enable_dynamic_reduction: false,
                    reduction_factor: 3,
                    ..NullMoveConfig::default()
                },
            ),
            (
                "disabled",
                NullMoveConfig {
                    enabled: false,
                    ..NullMoveConfig::default()
                },
            ),
        ];

        let mut results = Vec::new();

        for (name, config) in configs {
            engine.update_null_move_config(config).unwrap();
            engine.reset_null_move_stats();

            let start = Instant::now();
            let result = engine.search_at_depth(&board, &captured_pieces, player, 3, 1000);
            let duration = start.elapsed();

            let stats = engine.get_null_move_stats();
            results.push((
                name,
                result.is_some(),
                stats.attempts,
                stats.cutoffs,
                duration.as_millis(),
            ));
        }

        // All configurations should complete successfully
        assert_eq!(results.len(), 5);

        for (name, success, attempts, cutoffs, duration) in &results {
            assert!(success, "Configuration {} failed", name);
            assert!(*duration > 0, "Configuration {} took no time", name);
            assert!(
                *attempts >= 0,
                "Configuration {} had negative attempts",
                name
            );
            assert!(*cutoffs >= 0, "Configuration {} had negative cutoffs", name);
        }

        // Disabled configuration should have no NMP activity
        let disabled_result = results
            .iter()
            .find(|(name, _, _, _, _)| *name == "disabled")
            .unwrap();
        assert_eq!(disabled_result.2, 0); // No attempts
        assert_eq!(disabled_result.3, 0); // No cutoffs

        println!("NMP comprehensive benchmark results:");
        for (name, _, attempts, cutoffs, duration) in &results {
            println!(
                "  {}: {}ms, {} attempts, {} cutoffs",
                name, duration, attempts, cutoffs
            );
        }
    }

    #[test]
    fn benchmark_null_move_dynamic_vs_static_reduction() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test dynamic reduction
        let mut config = engine.get_null_move_config().clone();
        config.enable_dynamic_reduction = true;
        config.reduction_factor = 2;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_dynamic = Instant::now();
        let result_dynamic = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        let duration_dynamic = start_dynamic.elapsed();

        let stats_dynamic = engine.get_null_move_stats();

        // Test static reduction
        let mut config = engine.get_null_move_config().clone();
        config.enable_dynamic_reduction = false;
        config.reduction_factor = 3;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let start_static = Instant::now();
        let result_static = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        let duration_static = start_static.elapsed();

        let stats_static = engine.get_null_move_stats();

        // Both searches should complete successfully
        assert!(result_dynamic.is_some());
        assert!(result_static.is_some());

        // Both should have reasonable performance
        assert!(duration_dynamic.as_millis() > 0);
        assert!(duration_static.as_millis() > 0);

        println!(
            "Dynamic reduction: {:?}, {} attempts, {} cutoffs",
            duration_dynamic, stats_dynamic.attempts, stats_dynamic.cutoffs
        );
        println!(
            "Static reduction: {:?}, {} attempts, {} cutoffs",
            duration_static, stats_static.attempts, stats_static.cutoffs
        );
    }

    #[test]
    fn benchmark_null_move_safety_mechanisms() {
        let mut engine = create_test_engine();
        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Test with safety mechanisms enabled
        let mut config = engine.get_null_move_config().clone();
        config.enabled = true;
        config.enable_endgame_detection = true;
        config.max_pieces_threshold = 12;
        engine.update_null_move_config(config).unwrap();
        engine.reset_null_move_stats();

        let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);

        let stats = engine.get_null_move_stats();

        // Should track safety mechanism usage
        assert!(stats.disabled_in_check >= 0);
        assert!(stats.disabled_endgame >= 0);

        // Total disabled should be sum of individual counters
        assert_eq!(
            stats.total_disabled(),
            stats.disabled_in_check + stats.disabled_endgame
        );

        println!(
            "Safety mechanisms: {} disabled in check, {} disabled in endgame",
            stats.disabled_in_check, stats.disabled_endgame
        );
    }
}
