#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::CapturedPieces;
use shogi_engine::search::SearchEngine;
use shogi_engine::types::Player;
use shogi_engine::types::*;

#[cfg(test)]
mod aspiration_window_tests {
    use super::*;

    // ===== CONFIGURATION VALIDATION TESTS =====

    #[test]
    fn test_aspiration_window_config_default() {
        let config = AspirationWindowConfig::default();

        assert!(config.enabled);
        assert_eq!(config.base_window_size, 50);
        assert!(config.dynamic_scaling);
        assert_eq!(config.max_window_size, 200);
        assert_eq!(config.min_depth, 2);
        assert!(config.enable_adaptive_sizing);
        assert_eq!(config.max_researches, 2);
        assert!(config.enable_statistics);
    }

    #[test]
    fn test_aspiration_window_config_validation() {
        let mut config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 25,
            dynamic_scaling: true,
            max_window_size: 100,
            min_depth: 1,
            enable_adaptive_sizing: true,
            max_researches: 3,
            enable_statistics: true,
        };

        assert!(config.validate().is_ok());

        // Test invalid configurations
        config.base_window_size = 0;
        assert!(config.validate().is_err());

        config.base_window_size = 25;
        config.max_window_size = 10; // Less than base_window_size
        assert!(config.validate().is_err());

        config.max_window_size = 100;
        config.min_depth = 0;
        assert!(config.validate().is_err());

        config.min_depth = 1;
        config.max_researches = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_aspiration_window_config_new_validated() {
        let mut config = AspirationWindowConfig {
            enabled: true,
            base_window_size: 0, // Invalid
            dynamic_scaling: true,
            max_window_size: 10, // Invalid (less than base)
            min_depth: 0,        // Invalid
            enable_adaptive_sizing: true,
            max_researches: 0, // Invalid
            enable_statistics: true,
        };

        let validated = config.new_validated();

        assert_eq!(validated.base_window_size, 1); // Clamped to minimum
        assert_eq!(validated.max_window_size, 1); // Clamped to base_window_size
        assert_eq!(validated.min_depth, 1); // Clamped to minimum
        assert_eq!(validated.max_researches, 1); // Clamped to minimum
    }

    #[test]
    fn test_aspiration_window_stats_default() {
        let stats = AspirationWindowStats::default();

        assert_eq!(stats.total_searches, 0);
        assert_eq!(stats.successful_searches, 0);
        assert_eq!(stats.fail_lows, 0);
        assert_eq!(stats.fail_highs, 0);
        assert_eq!(stats.total_researches, 0);
        assert_eq!(stats.average_window_size, 0.0);
        assert_eq!(stats.estimated_time_saved_ms, 0);
        assert_eq!(stats.estimated_nodes_saved, 0);
    }

    #[test]
    fn test_aspiration_window_stats_methods() {
        let mut stats = AspirationWindowStats {
            total_searches: 100,
            successful_searches: 70,
            fail_lows: 20,
            fail_highs: 10,
            total_researches: 30,
            average_window_size: 45.5,
            estimated_time_saved_ms: 1000,
            estimated_nodes_saved: 50000,
        };

        assert_eq!(stats.success_rate(), 70.0);
        assert_eq!(stats.research_rate(), 30.0);
        assert_eq!(stats.fail_low_rate(), 20.0);
        assert_eq!(stats.fail_high_rate(), 10.0);
        assert!(stats.efficiency() > 0.0);

        stats.reset();
        assert_eq!(stats.total_searches, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    // ===== WINDOW SIZE CALCULATION TESTS =====

    #[test]
    fn test_calculate_static_window_size() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test below minimum depth
        let window_size = engine.calculate_static_window_size(1);
        assert_eq!(window_size, i32::MAX);

        // Test at minimum depth
        let window_size = engine.calculate_static_window_size(2);
        assert_eq!(window_size, 50);

        // Test above minimum depth
        let window_size = engine.calculate_static_window_size(5);
        assert_eq!(window_size, 50);
    }

    #[test]
    fn test_calculate_dynamic_window_size() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test with dynamic scaling enabled
        let window_size = engine.calculate_dynamic_window_size(3, 100);
        assert!(window_size > 50); // Should be larger due to depth and score factors

        // Test with dynamic scaling disabled
        let mut config = AspirationWindowConfig::default();
        config.dynamic_scaling = false;
        engine.update_aspiration_window_config(config).unwrap();

        let window_size = engine.calculate_dynamic_window_size(3, 100);
        assert_eq!(window_size, 50); // Should be base size
    }

    #[test]
    fn test_calculate_adaptive_window_size() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test with no recent failures
        let window_size = engine.calculate_adaptive_window_size(3, 0);
        assert!(window_size >= 50);

        // Test with recent failures
        let window_size = engine.calculate_adaptive_window_size(3, 2);
        assert!(window_size > 50); // Should be larger due to failures
    }

    #[test]
    fn test_calculate_window_size() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test disabled aspiration windows
        let mut config = AspirationWindowConfig::default();
        config.enabled = false;
        engine.update_aspiration_window_config(config).unwrap();

        let window_size = engine.calculate_window_size(3, 100, 0);
        assert_eq!(window_size, i32::MAX);

        // Test enabled aspiration windows
        let mut config = AspirationWindowConfig::default();
        config.enabled = true;
        engine.update_aspiration_window_config(config).unwrap();

        let window_size = engine.calculate_window_size(3, 100, 0);
        assert!(window_size < i32::MAX);
        assert!(window_size >= 10); // Minimum window size
    }

    #[test]
    fn test_validate_window_size() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test normal window size
        let validated = engine.validate_window_size(75);
        assert_eq!(validated, 75);

        // Test window size too small
        let validated = engine.validate_window_size(5);
        assert_eq!(validated, 10); // Should be clamped to minimum

        // Test window size too large
        let validated = engine.validate_window_size(500);
        assert_eq!(validated, 200); // Should be clamped to maximum
    }

    // ===== RE-SEARCH LOGIC TESTS =====

    #[test]
    fn test_handle_fail_low() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let mut alpha = 100;
        let mut beta = 200;
        let previous_score = 150;
        let window_size = 50;

        engine.handle_fail_low(&mut alpha, &mut beta, previous_score, window_size);

        assert_eq!(alpha, i32::MIN + 1);
        assert_eq!(beta, previous_score + window_size);
        assert_eq!(engine.aspiration_stats.fail_lows, 1);
    }

    #[test]
    fn test_handle_fail_high() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let mut alpha = 100;
        let mut beta = 200;
        let previous_score = 150;
        let window_size = 50;

        engine.handle_fail_high(&mut alpha, &mut beta, previous_score, window_size);

        assert_eq!(alpha, previous_score - window_size);
        assert_eq!(beta, i32::MAX - 1);
        assert_eq!(engine.aspiration_stats.fail_highs, 1);
    }

    #[test]
    fn test_validate_window_parameters() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test valid parameters
        assert!(engine.validate_window_parameters(100, 50));

        // Test invalid score
        assert!(!engine.validate_window_parameters(-200000, 50));
        assert!(!engine.validate_window_parameters(200000, 50));

        // Test invalid window size
        assert!(!engine.validate_window_parameters(100, 0));
        assert!(!engine.validate_window_parameters(100, 500));
    }

    #[test]
    fn test_handle_aspiration_failure() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        let (alpha, beta) = engine.handle_aspiration_failure(3, "test failure");

        assert_eq!(alpha, i32::MIN + 1);
        assert_eq!(beta, i32::MAX - 1);
        assert_eq!(engine.aspiration_stats.total_searches, 1);
    }

    #[test]
    fn test_should_disable_aspiration_windows() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test with no searches
        assert!(!engine.should_disable_aspiration_windows());

        // Test with high failure rate
        engine.aspiration_stats.total_searches = 200;
        engine.aspiration_stats.fail_lows = 100;
        engine.aspiration_stats.fail_highs = 80;

        assert!(engine.should_disable_aspiration_windows());

        // Reset and test with high re-search rate
        engine.aspiration_stats.fail_lows = 10;
        engine.aspiration_stats.fail_highs = 10;
        engine.aspiration_stats.total_researches = 150;

        assert!(engine.should_disable_aspiration_windows());
    }

    // ===== STATISTICS AND METRICS TESTS =====

    #[test]
    fn test_get_research_efficiency() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Set up some test data
        engine.aspiration_stats.total_searches = 100;
        engine.aspiration_stats.successful_searches = 70;
        engine.aspiration_stats.fail_lows = 20;
        engine.aspiration_stats.fail_highs = 10;
        engine.aspiration_stats.total_researches = 30;

        let metrics = engine.get_research_efficiency();

        assert_eq!(metrics.total_searches, 100);
        assert_eq!(metrics.successful_searches, 70);
        assert_eq!(metrics.fail_lows, 20);
        assert_eq!(metrics.fail_highs, 10);
        assert_eq!(metrics.total_researches, 30);
        assert_eq!(metrics.success_rate, 0.7);
        assert_eq!(metrics.research_rate, 0.3);
        assert_eq!(metrics.fail_low_rate, 0.2);
        assert_eq!(metrics.fail_high_rate, 0.1);
    }

    #[test]
    fn test_research_efficiency_metrics() {
        let metrics = ResearchEfficiencyMetrics {
            total_searches: 100,
            successful_searches: 80,
            fail_lows: 10,
            fail_highs: 10,
            total_researches: 20,
            success_rate: 0.8,
            research_rate: 0.2,
            fail_low_rate: 0.1,
            fail_high_rate: 0.1,
        };

        assert!(metrics.is_efficient());

        let summary = metrics.summary();
        assert!(summary.contains("100 searches"));
        assert!(summary.contains("80.0% success"));

        let recommendations = metrics.get_efficiency_recommendations();
        assert!(recommendations.contains(&"Re-search efficiency is good".to_string()));
    }

    #[test]
    fn test_window_size_statistics() {
        let stats = WindowSizeStatistics {
            average_window_size: 45.5,
            min_window_size: 10,
            max_window_size: 200,
            total_calculations: 100,
            success_rate: 0.75,
            fail_low_rate: 0.15,
            fail_high_rate: 0.10,
        };

        assert!(stats.is_well_tuned());

        let summary = stats.summary();
        assert!(summary.contains("avg=45.5"));
        assert!(summary.contains("success=75.0%"));

        let recommendations = stats.get_tuning_recommendations();
        assert!(recommendations.contains(&"Re-search efficiency is good".to_string()));
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_search_engine_aspiration_integration() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test configuration management
        assert!(engine.get_aspiration_window_config().enabled);

        // Test statistics access
        let stats = engine.get_aspiration_window_stats();
        assert_eq!(stats.total_searches, 0);

        // Test performance metrics
        let metrics = engine.get_aspiration_window_performance_metrics();
        assert_eq!(metrics.total_searches, 0);

        // Test preset application
        let result =
            engine.apply_aspiration_window_preset(AspirationWindowPlayingStyle::Aggressive);
        assert!(result.is_ok());

        // Test memory optimization
        engine.optimize_aspiration_window_memory();
    }

    #[test]
    fn test_aspiration_window_presets() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test aggressive preset
        let aggressive_preset =
            engine.get_aspiration_window_preset(AspirationWindowPlayingStyle::Aggressive);
        assert!(aggressive_preset.base_window_size < 50);

        // Test conservative preset
        let conservative_preset =
            engine.get_aspiration_window_preset(AspirationWindowPlayingStyle::Conservative);
        assert!(conservative_preset.base_window_size > 50);

        // Test balanced preset
        let balanced_preset =
            engine.get_aspiration_window_preset(AspirationWindowPlayingStyle::Balanced);
        assert_eq!(balanced_preset.base_window_size, 50);
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_edge_cases() {
        let mut engine = SearchEngine::new();
        let config = AspirationWindowConfig::default();
        engine.update_aspiration_window_config(config).unwrap();

        // Test with extreme scores
        let window_size = engine.calculate_window_size(3, i32::MAX, 0);
        assert!(window_size < i32::MAX);

        let window_size = engine.calculate_window_size(3, i32::MIN, 0);
        assert!(window_size < i32::MAX);

        // Test with maximum failures
        let window_size = engine.calculate_window_size(3, 100, 255);
        assert!(window_size >= 10);

        // Test with depth 0
        let window_size = engine.calculate_window_size(0, 100, 0);
        assert_eq!(window_size, i32::MAX);
    }

    #[test]
    fn test_error_handling() {
        let mut engine = SearchEngine::new();

        // Test invalid configuration
        let mut config = AspirationWindowConfig::default();
        config.base_window_size = -1;
        let result = engine.update_aspiration_window_config(config);
        assert!(result.is_err());

        // Test valid configuration
        let config = AspirationWindowConfig::default();
        let result = engine.update_aspiration_window_config(config);
        assert!(result.is_ok());
    }
}
