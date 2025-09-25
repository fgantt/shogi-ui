use shogi_engine::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
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
        let time_source = TimeSource::new();
        
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
                4
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
        let time_source = TimeSource::new();
        
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
                depth
            );
            let duration = start.elapsed();
            
            let stats = engine.get_quiescence_stats();
            results.push((depth, stats.nodes_searched, duration.as_millis()));
        }
        
        // Verify that deeper searches take more time and search more nodes
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i-1].1); // More nodes at deeper depth
        }
    }

    #[test]
    fn benchmark_quiescence_tt_performance() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
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
            4
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
            4
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
        let time_source = TimeSource::new();
        
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
            4
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
            4
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
        let moves = engine.move_generator.generate_quiescence_moves(&board, player, &captured_pieces);
        
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
        let time_source = TimeSource::new();
        
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
                3
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
        let time_source = TimeSource::new();
        
        // Test different configurations
        let configs = vec![
            ("default", QuiescenceConfig::default()),
            ("high_depth", QuiescenceConfig {
                max_depth: 10,
                ..QuiescenceConfig::default()
            }),
            ("no_pruning", QuiescenceConfig {
                enable_delta_pruning: false,
                enable_futility_pruning: false,
                ..QuiescenceConfig::default()
            }),
            ("no_tt", QuiescenceConfig {
                enable_tt: false,
                ..QuiescenceConfig::default()
            }),
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
                4
            );
            let duration = start.elapsed();
            
            let stats = engine.get_quiescence_stats();
            results.push((name, stats.nodes_searched, duration.as_millis()));
        }
        
        // All configurations should complete successfully
        assert_eq!(results.len(), 4);
        
        // High depth should search more nodes
        let high_depth_result = results.iter().find(|(name, _, _)| *name == "high_depth").unwrap();
        let default_result = results.iter().find(|(name, _, _)| *name == "default").unwrap();
        assert!(high_depth_result.1 >= default_result.1);
    }

    #[test]
    fn benchmark_quiescence_concurrent_performance() {
        let mut engine = create_test_engine();
        let mut board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Sente;
        let time_source = TimeSource::new();
        
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
                3
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
        let time_source = TimeSource::new();
        
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
                3
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
        let time_source = TimeSource::new();
        
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
            4
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
        let time_source = TimeSource::new();
        
        engine.reset_quiescence_stats();
        
        let _result = engine.quiescence_search(
            &mut board,
            &captured_pieces,
            player,
            -10000,
            10000,
            &time_source,
            1000,
            4
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
}
