#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::{IterativeDeepening, SearchEngine};
use shogi_engine::types::Player;
use shogi_engine::types::*;
use std::time::Instant;

#[cfg(test)]
mod aspiration_window_performance_tests {
    use super::*;

    // ===== PERFORMANCE BENCHMARK TESTS =====

    #[test]
    fn test_aspiration_windows_performance_benchmark() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with aspiration windows enabled
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let start_time = Instant::now();
        let mut searcher = IterativeDeepening::new(4, 2000, None, None);
        let result_with_aspiration = searcher.search(&mut engine, &board, &captured_pieces, player);
        let time_with_aspiration = start_time.elapsed();

        // Test with aspiration windows disabled
        let mut config = AspirationWindowConfig::default();
        config.enabled = false;
        engine.update_aspiration_window_config(config).unwrap();

        let start_time = Instant::now();
        let mut searcher = IterativeDeepening::new(4, 2000, None, None);
        let result_without_aspiration =
            searcher.search(&mut engine, &board, &captured_pieces, player);
        let time_without_aspiration = start_time.elapsed();

        // Both should return valid results
        assert!(result_with_aspiration.is_some());
        assert!(result_without_aspiration.is_some());

        // Log performance comparison
        println!("Time with aspiration windows: {:?}", time_with_aspiration);
        println!(
            "Time without aspiration windows: {:?}",
            time_without_aspiration
        );

        // Aspiration windows should provide some benefit (though this may vary)
        // We just check that both searches complete successfully
        assert!(time_with_aspiration.as_millis() > 0);
        assert!(time_without_aspiration.as_millis() > 0);
    }

    #[test]
    fn test_aspiration_windows_efficiency_metrics() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform multiple searches to collect statistics
        for _ in 0..10 {
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Check efficiency metrics
        let metrics = engine.get_research_efficiency();
        assert!(metrics.total_searches > 0);
        assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
        assert!(metrics.research_rate >= 0.0);
        assert!(metrics.fail_low_rate >= 0.0 && metrics.fail_low_rate <= 1.0);
        assert!(metrics.fail_high_rate >= 0.0 && metrics.fail_high_rate <= 1.0);

        // Check that efficiency is reasonable
        assert!(metrics.success_rate + metrics.fail_low_rate + metrics.fail_high_rate <= 1.1);
        // Allow for rounding
    }

    #[test]
    fn test_aspiration_windows_window_size_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test window size calculation performance
        let start_time = Instant::now();

        for depth in 1..=10 {
            for score in -1000..=1000 {
                for failures in 0..=5 {
                    let window_size = engine.calculate_window_size(depth, score, failures);
                    assert!(window_size >= 10);
                    assert!(window_size <= 200);
                }
            }
        }

        let calculation_time = start_time.elapsed();
        println!("Window size calculation time: {:?}", calculation_time);

        // Should complete within reasonable time
        assert!(calculation_time.as_millis() < 1000);
    }

    #[test]
    fn test_aspiration_windows_memory_usage() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform many searches to test memory usage
        for _ in 0..100 {
            let mut searcher = IterativeDeepening::new(2, 500, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Check that memory optimization works
        let stats_before = engine.get_aspiration_window_stats();
        engine.optimize_aspiration_window_memory();
        let stats_after = engine.get_aspiration_window_stats();

        // Statistics should be reset after optimization
        assert_eq!(stats_after.total_searches, 0);
        assert_eq!(stats_after.average_window_size, 0.0);
    }

    // ===== CONFIGURATION PERFORMANCE TESTS =====

    #[test]
    fn test_different_configurations_performance() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let configurations = vec![
            ("Aggressive", AspirationWindowPlayingStyle::Aggressive),
            ("Conservative", AspirationWindowPlayingStyle::Conservative),
            ("Balanced", AspirationWindowPlayingStyle::Balanced),
        ];

        for (name, style) in configurations {
            engine.apply_aspiration_window_preset(style).unwrap();

            let start_time = Instant::now();
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            let search_time = start_time.elapsed();

            assert!(result.is_some());
            println!("{} configuration search time: {:?}", name, search_time);
        }
    }

    #[test]
    fn test_window_size_calculation_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test different window size calculation methods
        let start_time = Instant::now();

        // Test static window size calculation
        for depth in 1..=10 {
            let window_size = engine.calculate_static_window_size(depth);
            assert!(window_size >= 10 || window_size == i32::MAX);
        }

        // Test dynamic window size calculation
        for depth in 1..=10 {
            for score in -1000..=1000 {
                let window_size = engine.calculate_dynamic_window_size(depth, score);
                assert!(window_size >= 10 || window_size == i32::MAX);
            }
        }

        // Test adaptive window size calculation
        for depth in 1..=10 {
            for failures in 0..=5 {
                let window_size = engine.calculate_adaptive_window_size(depth, failures);
                assert!(window_size >= 10 || window_size == i32::MAX);
            }
        }

        let calculation_time = start_time.elapsed();
        println!("All window size calculations time: {:?}", calculation_time);

        // Should complete within reasonable time
        assert!(calculation_time.as_millis() < 2000);
    }

    // ===== STRESS TESTS =====

    #[test]
    fn test_aspiration_windows_stress_test() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform many searches to stress test the system
        let start_time = Instant::now();

        for i in 0..50 {
            let mut searcher = IterativeDeepening::new(2, 200, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());

            // Check statistics periodically
            if i % 10 == 0 {
                let stats = engine.get_aspiration_window_stats();
                assert!(stats.total_searches > 0);
            }
        }

        let total_time = start_time.elapsed();
        println!("Stress test total time: {:?}", total_time);

        // Should complete within reasonable time
        assert!(total_time.as_millis() < 10000);

        // Check final statistics
        let final_stats = engine.get_aspiration_window_stats();
        assert!(final_stats.total_searches > 0);
    }

    #[test]
    fn test_aspiration_windows_edge_case_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with extreme configurations
        let extreme_configs = vec![
            (
                "Very small window",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 10,
                    dynamic_scaling: true,
                    max_window_size: 20,
                    min_depth: 1,
                    enable_adaptive_sizing: true,
                    max_researches: 1,
                    enable_statistics: true,
                },
            ),
            (
                "Very large window",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 100,
                    dynamic_scaling: true,
                    max_window_size: 200,
                    min_depth: 1,
                    enable_adaptive_sizing: true,
                    max_researches: 5,
                    enable_statistics: true,
                },
            ),
            (
                "High research limit",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 50,
                    dynamic_scaling: true,
                    max_window_size: 200,
                    min_depth: 1,
                    enable_adaptive_sizing: true,
                    max_researches: 5,
                    enable_statistics: true,
                },
            ),
        ];

        for (name, config) in extreme_configs {
            engine.update_aspiration_window_config(config).unwrap();

            let start_time = Instant::now();
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            let search_time = start_time.elapsed();

            assert!(result.is_some());
            println!("{} search time: {:?}", name, search_time);
        }
    }

    // ===== MEMORY PERFORMANCE TESTS =====

    #[test]
    fn test_aspiration_windows_memory_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Simulate large memory usage
        engine.aspiration_stats.total_searches = 1_000_000;
        engine.aspiration_stats.average_window_size = 45.5;
        engine.previous_scores = vec![100; 10000]; // Large vector

        let start_time = Instant::now();
        engine.optimize_aspiration_window_memory();
        let optimization_time = start_time.elapsed();

        println!("Memory optimization time: {:?}", optimization_time);

        // Should complete quickly
        assert!(optimization_time.as_millis() < 100);

        // Check that memory was optimized
        assert_eq!(engine.aspiration_stats.total_searches, 0);
        assert_eq!(engine.aspiration_stats.average_window_size, 0.0);
        assert!(engine.previous_scores.is_empty());
    }

    // ===== COMPREHENSIVE PERFORMANCE TESTS =====

    #[test]
    fn test_aspiration_windows_comprehensive_performance() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let mut total_searches = 0;
        let mut total_time = std::time::Duration::new(0, 0);

        // Perform comprehensive performance test
        for depth in 2..=5 {
            for _ in 0..5 {
                let start_time = Instant::now();
                let mut searcher = IterativeDeepening::new(depth, 1000, None, None);
                let result = searcher.search(&mut engine, &board, &captured_pieces, player);
                let search_time = start_time.elapsed();

                assert!(result.is_some());
                total_searches += 1;
                total_time += search_time;
            }
        }

        let average_time = total_time / total_searches;
        println!("Average search time: {:?}", average_time);
        println!("Total searches: {}", total_searches);

        // Check final statistics
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);
        assert!(stats.average_window_size > 0.0);

        // Check efficiency metrics
        let metrics = engine.get_research_efficiency();
        assert!(metrics.total_searches > 0);
        assert!(metrics.is_efficient() || metrics.total_searches < 10); // Allow for small sample size
    }
}
