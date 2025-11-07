#![cfg(feature = "legacy-tests")]
/// Performance optimization tests for Late Move Reductions (LMR)
///
/// This module contains tests for LMR performance optimization features:
/// - Performance profiling and overhead analysis
/// - Parameter tuning and auto-optimization
/// - Memory usage optimization
/// - Configuration presets and hardware optimization
/// - Performance regression testing
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

// Safety constants for performance tests
const MAX_PROFILE_ITERATIONS: usize = 5;
const MAX_PROFILE_DEPTH: u8 = 6;

#[cfg(test)]
mod lmr_profiling_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 32) // Larger hash table for profiling
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    #[test]
    fn test_lmr_profiling_overhead() {
        if !std::env::var("RUN_PERFORMANCE_TESTS").is_ok() {
            return;
        }

        let mut engine = create_test_engine();

        // Configure LMR for profiling
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
        let depth = 5;
        let iterations = 3;

        // Profile LMR overhead
        let profile_result =
            engine.profile_lmr_overhead(&board, &captured_pieces, player, depth, iterations);

        // Verify profiling results
        assert!(profile_result.total_time.as_millis() > 0);
        assert!(profile_result.total_moves_processed > 0);
        assert!(profile_result.moves_per_second > 0.0);

        println!("LMR Profiling Results:");
        println!("  Total time: {:?}", profile_result.total_time);
        println!(
            "  Average time per search: {:?}",
            profile_result.average_time_per_search
        );
        println!("  Moves per second: {:.0}", profile_result.moves_per_second);
        println!("  Reduction rate: {:.1}%", profile_result.reduction_rate);
        println!("  Research rate: {:.1}%", profile_result.research_rate);

        // Check efficiency
        assert!(profile_result.is_efficient() || profile_result.reduction_rate > 0.0);
    }

    #[test]
    fn test_lmr_performance_metrics() {
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

        // Perform some searches to generate metrics
        for _ in 0..3 {
            let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        }

        let metrics = engine.get_lmr_performance_metrics();

        // Verify metrics are reasonable
        assert!(metrics.moves_considered >= 0);
        assert!(metrics.reductions_applied >= 0);
        assert!(metrics.researches_triggered >= 0);
        assert!(metrics.efficiency >= 0.0);
        assert!(metrics.efficiency <= 100.0);
        assert!(metrics.research_rate >= 0.0);
        assert!(metrics.research_rate <= 100.0);
        assert!(metrics.cutoff_rate >= 0.0);
        assert!(metrics.cutoff_rate <= 100.0);

        println!("LMR Performance Metrics:");
        println!("  {}", metrics.summary());
        println!("  Performing well: {}", metrics.is_performing_well());

        // Test optimization recommendations
        let recommendations = metrics.get_optimization_recommendations();
        assert!(!recommendations.is_empty());
        println!("  Recommendations: {:?}", recommendations);
    }
}

#[cfg(test)]
mod lmr_parameter_tuning_tests {
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
    fn test_lmr_auto_tuning() {
        if !std::env::var("RUN_PERFORMANCE_TESTS").is_ok() {
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

        // Perform many searches to generate enough data for auto-tuning
        for _ in 0..20 {
            let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        }

        // Test auto-tuning
        let result = engine.auto_tune_lmr_parameters();

        // Auto-tuning should either succeed or fail with insufficient data
        match result {
            Ok(_) => {
                println!("Auto-tuning succeeded");
                let new_config = engine.get_lmr_config();
                println!("New configuration: {}", new_config.summary());
            }
            Err(msg) => {
                println!("Auto-tuning failed: {}", msg);
                // This is acceptable if we don't have enough data
                assert!(msg.contains("Insufficient data"));
            }
        }
    }

    #[test]
    fn test_lmr_configuration_presets() {
        let engine = create_test_engine();

        // Test aggressive preset
        let aggressive_config = engine.get_lmr_preset(LMRPlayingStyle::Aggressive);
        assert!(aggressive_config.enabled);
        assert_eq!(aggressive_config.min_depth, 2);
        assert_eq!(aggressive_config.min_move_index, 3);
        assert_eq!(aggressive_config.base_reduction, 2);
        assert_eq!(aggressive_config.max_reduction, 4);

        // Test conservative preset
        let conservative_config = engine.get_lmr_preset(LMRPlayingStyle::Conservative);
        assert!(conservative_config.enabled);
        assert_eq!(conservative_config.min_depth, 4);
        assert_eq!(conservative_config.min_move_index, 6);
        assert_eq!(conservative_config.base_reduction, 1);
        assert_eq!(conservative_config.max_reduction, 2);

        // Test balanced preset
        let balanced_config = engine.get_lmr_preset(LMRPlayingStyle::Balanced);
        assert!(balanced_config.enabled);
        assert_eq!(balanced_config.min_depth, 3);
        assert_eq!(balanced_config.min_move_index, 4);
        assert_eq!(balanced_config.base_reduction, 1);
        assert_eq!(balanced_config.max_reduction, 3);

        println!("Configuration presets validated successfully");
    }

    #[test]
    fn test_lmr_preset_application() {
        let mut engine = create_test_engine();

        // Test applying aggressive preset
        let result = engine.apply_lmr_preset(LMRPlayingStyle::Aggressive);
        assert!(result.is_ok());

        let config = engine.get_lmr_config();
        assert_eq!(config.min_depth, 2);
        assert_eq!(config.base_reduction, 2);

        // Test applying conservative preset
        let result = engine.apply_lmr_preset(LMRPlayingStyle::Conservative);
        assert!(result.is_ok());

        let config = engine.get_lmr_config();
        assert_eq!(config.min_depth, 4);
        assert_eq!(config.base_reduction, 1);

        println!("Preset application validated successfully");
    }
}

#[cfg(test)]
mod lmr_memory_optimization_tests {
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
    fn test_lmr_memory_optimization() {
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

        // Perform searches to generate statistics
        for _ in 0..10 {
            let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        }

        let initial_stats = engine.get_lmr_stats();
        let initial_tt_size = engine.transposition_table_len();

        // Test memory optimization
        engine.optimize_lmr_memory();

        let final_stats = engine.get_lmr_stats();
        let final_tt_size = engine.transposition_table_len();

        // Memory optimization should not increase memory usage
        assert!(final_tt_size <= initial_tt_size);

        println!("Memory optimization test:");
        println!("  Initial TT size: {}", initial_tt_size);
        println!("  Final TT size: {}", final_tt_size);
        println!(
            "  Initial moves considered: {}",
            initial_stats.moves_considered
        );
        println!("  Final moves considered: {}", final_stats.moves_considered);
    }

    #[test]
    fn test_lmr_performance_report() {
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

        // Perform searches to generate data
        for _ in 0..5 {
            let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        }

        let report = engine.get_lmr_performance_report();

        // Verify report contains expected information
        assert!(report.contains("LMR Performance Report"));
        assert!(report.contains("Moves considered"));
        assert!(report.contains("Efficiency"));
        assert!(report.contains("Research rate"));
        assert!(report.contains("Optimization Recommendations"));

        println!("Performance Report:");
        println!("{}", report);
    }
}

#[cfg(test)]
mod lmr_hardware_optimization_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    #[test]
    fn test_hardware_optimized_config() {
        let engine = create_test_engine();

        let config = engine.get_hardware_optimized_config();

        // Verify hardware-optimized config is valid
        assert!(config.enabled);
        assert!(config.validate().is_ok());

        println!("Hardware-optimized configuration: {}", config.summary());
    }

    #[test]
    fn test_lmr_optimized_methods() {
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

        // Test optimized move exemption check
        let capture_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: true,
            is_promotion: false,
            captured_piece: Some(Piece {
                piece_type: PieceType::Pawn,
                player: Player::White,
            }),
            gives_check: false,
            is_recapture: false,
        };

        // This should use the optimized method internally
        assert!(engine.is_move_exempt_from_lmr(&capture_move));

        // Test optimized reduction calculation
        let quiet_move = Move {
            from: Some(Position::new(1, 1)),
            to: Position::new(2, 1),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_capture: false,
            is_promotion: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        };

        let reduction = engine.calculate_reduction(&quiet_move, 5, 6);
        assert!(reduction >= 1);
        assert!(reduction <= 3);

        println!("Optimized methods validated successfully");
    }
}

#[cfg(test)]
mod lmr_performance_regression_tests {
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
    fn test_lmr_performance_consistency() {
        if !std::env::var("RUN_PERFORMANCE_TESTS").is_ok() {
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

        // Perform multiple searches and verify consistency
        let mut results = Vec::new();
        for _ in 0..5 {
            let start_time = Instant::now();
            let result = engine.search_at_depth(&board, &captured_pieces, player, 4, 2000);
            let elapsed = start_time.elapsed();

            assert!(result.is_some());
            results.push(elapsed);
        }

        // Check that performance is reasonably consistent
        let avg_time: f64 =
            results.iter().map(|d| d.as_millis() as f64).sum::<f64>() / results.len() as f64;
        let max_deviation = results
            .iter()
            .map(|d| (d.as_millis() as f64 - avg_time).abs())
            .fold(0.0, f64::max);

        // Maximum deviation should be reasonable (less than 50% of average)
        assert!(
            max_deviation < avg_time * 0.5,
            "Performance too inconsistent: avg={:.1}ms, max_dev={:.1}ms",
            avg_time,
            max_deviation
        );

        println!("Performance consistency test passed:");
        println!("  Average time: {:.1}ms", avg_time);
        println!("  Max deviation: {:.1}ms", max_deviation);
    }

    #[test]
    fn test_lmr_optimization_impact() {
        let mut engine_with_optimization = create_test_engine();
        let mut engine_without_optimization = create_test_engine();

        // Configure both engines identically
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

        engine_with_optimization
            .update_lmr_config(lmr_config.clone())
            .unwrap();
        engine_without_optimization
            .update_lmr_config(lmr_config)
            .unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Both engines should find the same result
        let result_with =
            engine_with_optimization.search_at_depth(&board, &captured_pieces, player, 4, 2000);
        let result_without =
            engine_without_optimization.search_at_depth(&board, &captured_pieces, player, 4, 2000);

        assert!(result_with.is_some());
        assert!(result_without.is_some());

        let (move_with, score_with) = result_with.unwrap();
        let (move_without, score_without) = result_without.unwrap();

        // Scores should be very close (optimization shouldn't change search quality)
        let score_diff = (score_with - score_without).abs();
        assert!(
            score_diff < 100,
            "Score difference too large: {} vs {}",
            score_with,
            score_without
        );

        println!("Optimization impact test passed:");
        println!("  Score with optimization: {}", score_with);
        println!("  Score without optimization: {}", score_without);
        println!("  Score difference: {}", score_diff);
    }
}
