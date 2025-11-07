#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::{IterativeDeepening, SearchEngine};
use shogi_engine::types::Player;
use shogi_engine::types::*;
use std::time::Instant;

#[cfg(test)]
mod aspiration_window_optimization_tests {
    use super::*;

    // ===== WINDOW SIZE OPTIMIZATION TESTS =====

    #[test]
    fn test_window_size_optimization_algorithms() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test different window size calculation strategies
        let test_cases = vec![
            (3, 100, 0, 0.5, 500, 1000, 0.8, 25), // Normal case
            (5, -50, 2, 0.8, 200, 1000, 0.6, 40), // Complex case
            (2, 0, 0, 0.0, 1000, 1000, 1.0, 10),  // Simple case
            (8, 500, 4, 1.0, 100, 1000, 0.4, 60), // Extreme case
        ];

        for (
            depth,
            prev_score,
            failures,
            complexity,
            time_remaining,
            total_time,
            success_rate,
            move_count,
        ) in test_cases
        {
            let window_size = engine.calculate_comprehensive_window_size(
                depth,
                prev_score,
                failures,
                complexity,
                time_remaining,
                total_time,
                success_rate,
                move_count,
            );

            // Window size should be within reasonable bounds
            assert!(window_size >= 10);
            assert!(window_size <= 200);

            // Different cases should produce different window sizes
            println!(
                "Depth: {}, Score: {}, Window: {}",
                depth, prev_score, window_size
            );
        }
    }

    #[test]
    fn test_adaptive_window_sizing() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test adaptive window sizing based on recent failures
        let base_window = engine.calculate_adaptive_window_size(3, 0);
        let failure_window = engine.calculate_adaptive_window_size(3, 3);

        // Window should be larger with more failures
        assert!(failure_window >= base_window);

        // Test with different depths
        for depth in 1..=10 {
            let window_size = engine.calculate_adaptive_window_size(depth, 0);
            assert!(window_size >= 10 || window_size == i32::MAX);
        }
    }

    #[test]
    fn test_dynamic_window_scaling() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test depth-based scaling
        let window_1 = engine.calculate_dynamic_window_size(1, 100);
        let window_5 = engine.calculate_dynamic_window_size(5, 100);

        // Higher depth should generally produce larger windows
        assert!(window_5 >= window_1);

        // Test score-based scaling
        let window_low = engine.calculate_dynamic_window_size(3, 50);
        let window_high = engine.calculate_dynamic_window_size(3, 500);

        // Higher score magnitude should generally produce larger windows
        assert!(window_high >= window_low);
    }

    #[test]
    fn test_time_based_window_sizing() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test time pressure effects
        let window_high_time = engine.calculate_time_based_window_size(3, 800, 1000);
        let window_low_time = engine.calculate_time_based_window_size(3, 200, 1000);

        // More time remaining should generally produce larger windows
        assert!(window_high_time >= window_low_time);

        // Test edge cases
        let window_no_time = engine.calculate_time_based_window_size(3, 0, 1000);
        let window_full_time = engine.calculate_time_based_window_size(3, 1000, 1000);

        assert!(window_no_time >= 10);
        assert!(window_full_time >= 10);
    }

    #[test]
    fn test_complexity_based_window_sizing() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test position complexity effects
        let window_simple = engine.calculate_complexity_based_window_size(3, 0.0);
        let window_complex = engine.calculate_complexity_based_window_size(3, 1.0);

        // More complex positions should generally produce larger windows
        assert!(window_complex >= window_simple);

        // Test edge cases
        let window_extreme = engine.calculate_complexity_based_window_size(3, 2.0);
        assert!(window_extreme >= 10);
    }

    // ===== PERFORMANCE OPTIMIZATION TESTS =====

    #[test]
    fn test_aspiration_window_efficiency_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform searches to collect performance data
        for _ in 0..20 {
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Check efficiency metrics
        let metrics = engine.get_research_efficiency();
        assert!(metrics.total_searches > 0);

        // Test optimization recommendations
        let recommendations = metrics.get_efficiency_recommendations();
        assert!(!recommendations.is_empty());

        // Test if efficiency is good
        if metrics.total_searches > 10 {
            // For larger sample sizes, check efficiency
            println!("Efficiency metrics: {}", metrics.summary());
        }
    }

    #[test]
    fn test_window_size_statistics_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform searches to collect window size statistics
        for _ in 0..15 {
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Check window size statistics
        let stats = engine.get_window_size_statistics();
        assert!(stats.total_calculations > 0);

        // Test tuning recommendations
        let recommendations = stats.get_tuning_recommendations();
        assert!(!recommendations.is_empty());

        // Test if window size is well-tuned
        if stats.total_calculations > 10 {
            println!("Window size statistics: {}", stats.summary());
        }
    }

    // ===== CONFIGURATION OPTIMIZATION TESTS =====

    #[test]
    fn test_aspiration_window_preset_optimization() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let presets = vec![
            AspirationWindowPlayingStyle::Aggressive,
            AspirationWindowPlayingStyle::Conservative,
            AspirationWindowPlayingStyle::Balanced,
        ];

        for preset in presets {
            engine.apply_aspiration_window_preset(preset).unwrap();

            // Test window size presets
            let window_size = engine.get_window_size_preset(preset);
            assert!(window_size > 0);

            // Test search performance with preset
            let start_time = Instant::now();
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            let search_time = start_time.elapsed();

            assert!(result.is_some());
            println!("Preset {:?} search time: {:?}", preset, search_time);
        }
    }

    #[test]
    fn test_aspiration_window_configuration_tuning() {
        let mut engine = SearchEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test different configuration parameters
        let configurations = vec![
            (
                "Small window",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 25,
                    dynamic_scaling: true,
                    max_window_size: 100,
                    min_depth: 2,
                    enable_adaptive_sizing: true,
                    max_researches: 2,
                    enable_statistics: true,
                },
            ),
            (
                "Large window",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 100,
                    dynamic_scaling: true,
                    max_window_size: 300,
                    min_depth: 2,
                    enable_adaptive_sizing: true,
                    max_researches: 3,
                    enable_statistics: true,
                },
            ),
            (
                "No dynamic scaling",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 50,
                    dynamic_scaling: false,
                    max_window_size: 200,
                    min_depth: 2,
                    enable_adaptive_sizing: true,
                    max_researches: 2,
                    enable_statistics: true,
                },
            ),
            (
                "No adaptive sizing",
                AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 50,
                    dynamic_scaling: true,
                    max_window_size: 200,
                    min_depth: 2,
                    enable_adaptive_sizing: false,
                    max_researches: 2,
                    enable_statistics: true,
                },
            ),
        ];

        for (name, config) in configurations {
            engine.update_aspiration_window_config(config).unwrap();

            let start_time = Instant::now();
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            let search_time = start_time.elapsed();

            assert!(result.is_some());
            println!("{} search time: {:?}", name, search_time);

            // Check that configuration was applied
            let applied_config = engine.get_aspiration_window_config();
            assert_eq!(applied_config.base_window_size, config.base_window_size);
            assert_eq!(applied_config.dynamic_scaling, config.dynamic_scaling);
            assert_eq!(
                applied_config.enable_adaptive_sizing,
                config.enable_adaptive_sizing
            );
        }
    }

    // ===== ALGORITHM OPTIMIZATION TESTS =====

    #[test]
    fn test_window_size_calculation_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test optimization of window size calculation
        let start_time = Instant::now();

        // Test comprehensive window size calculation with various parameters
        for depth in 1..=8 {
            for score in -500..=500 {
                for failures in 0..=3 {
                    for complexity in [0.0, 0.5, 1.0] {
                        let window_size = engine.calculate_comprehensive_window_size(
                            depth, score, failures, complexity, 500, 1000, 0.8, 30,
                        );
                        assert!(window_size >= 10);
                        assert!(window_size <= 200);
                    }
                }
            }
        }

        let calculation_time = start_time.elapsed();
        println!(
            "Comprehensive window size calculation time: {:?}",
            calculation_time
        );

        // Should complete within reasonable time
        assert!(calculation_time.as_millis() < 5000);
    }

    #[test]
    fn test_aspiration_window_algorithm_efficiency() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test algorithm efficiency with different depths
        for depth in 2..=6 {
            let start_time = Instant::now();

            let mut searcher = IterativeDeepening::new(depth, 2000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);

            let search_time = start_time.elapsed();
            assert!(result.is_some());

            println!("Depth {} search time: {:?}", depth, search_time);

            // Check that search completed within reasonable time
            assert!(search_time.as_millis() < 10000);
        }

        // Check final efficiency metrics
        let metrics = engine.get_research_efficiency();
        assert!(metrics.total_searches > 0);

        // Check that efficiency is reasonable
        if metrics.total_searches > 5 {
            assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0);
            assert!(metrics.research_rate >= 0.0);
        }
    }

    // ===== MEMORY OPTIMIZATION TESTS =====

    #[test]
    fn test_aspiration_window_memory_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Simulate large memory usage
        engine.aspiration_stats.total_searches = 1_000_000;
        engine.aspiration_stats.average_window_size = 45.5;
        engine.previous_scores = vec![100; 10000];

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

    #[test]
    fn test_aspiration_window_statistics_optimization() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform searches to collect statistics
        for _ in 0..25 {
            let mut searcher = IterativeDeepening::new(3, 1000, None, None);
            let result = searcher.search(&mut engine, &board, &captured_pieces, player);
            assert!(result.is_some());
        }

        // Test statistics optimization
        let stats = engine.get_aspiration_window_stats();
        assert!(stats.total_searches > 0);

        // Test performance metrics
        let metrics = engine.get_aspiration_window_performance_metrics();
        assert!(metrics.total_searches > 0);

        // Test window size statistics
        let window_stats = engine.get_window_size_statistics();
        assert!(window_stats.total_calculations > 0);

        // Test efficiency metrics
        let efficiency = engine.get_research_efficiency();
        assert!(efficiency.total_searches > 0);

        // All metrics should be consistent
        assert_eq!(stats.total_searches, metrics.total_searches);
        assert_eq!(stats.total_searches, efficiency.total_searches);
    }
}
