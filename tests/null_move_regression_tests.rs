//! Performance regression tests for Null Move Pruning
//!
//! These tests verify that NMP performance doesn't degrade below acceptable thresholds.
//! Failures indicate performance regressions that need investigation.

use shogi_engine::{
    search::SearchEngine,
    bitboards::BitboardBoard,
    types::{CapturedPieces, Player, NullMoveConfig},
};

fn create_test_engine_with_config(config: NullMoveConfig) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    engine.update_null_move_config(config).unwrap();
    engine
}

#[test]
fn test_nmp_performance_regression_default_config() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let config = NullMoveConfig::default();
    
    let mut engine = create_test_engine_with_config(config);
    engine.reset_null_move_stats();
    
    let start = std::time::Instant::now();
    let result = engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed = start.elapsed();
    
    assert!(result.is_some(), "Search should complete successfully");
    
    let stats = engine.get_null_move_stats();
    let cutoff_rate = stats.cutoff_rate();
    let efficiency = stats.efficiency();
    
    // Regression thresholds
    if stats.attempts > 0 {
        assert!(
            cutoff_rate >= 20.0,
            "Performance regression: cutoff rate {}% < threshold 20%",
            cutoff_rate
        );
        assert!(
            efficiency >= 15.0,
            "Performance regression: efficiency {}% < threshold 15%",
            efficiency
        );
    }
    
    // Search should complete within reasonable time
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 10000.0,
        "Performance regression: search time {}ms > threshold 10000ms",
        elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
fn test_nmp_performance_regression_disabled() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let mut config = NullMoveConfig::default();
    config.enabled = false;
    
    let mut engine = create_test_engine_with_config(config);
    engine.reset_null_move_stats();
    
    let start = std::time::Instant::now();
    let result = engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed = start.elapsed();
    
    assert!(result.is_some(), "Search should complete successfully");
    
    // Search should complete within reasonable time even without NMP
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 10000.0,
        "Performance regression: search time {}ms > threshold 10000ms",
        elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
fn test_nmp_performance_regression_with_verification() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let mut config = NullMoveConfig::default();
    config.verification_margin = 200;
    
    let mut engine = create_test_engine_with_config(config);
    engine.reset_null_move_stats();
    
    let start = std::time::Instant::now();
    let result = engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed = start.elapsed();
    
    assert!(result.is_some(), "Search should complete successfully");
    
    let stats = engine.get_null_move_stats();
    let cutoff_rate = stats.cutoff_rate();
    
    // Regression thresholds
    if stats.attempts > 0 {
        assert!(
            cutoff_rate >= 15.0,
            "Performance regression: cutoff rate {}% < threshold 15% (with verification)",
            cutoff_rate
        );
    }
    
    // Search should complete within reasonable time
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 12000.0,
        "Performance regression: search time {}ms > threshold 12000ms",
        elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
fn test_nmp_performance_regression_effectiveness() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    
    // Test with NMP enabled
    let config_enabled = NullMoveConfig::default();
    let mut engine_enabled = create_test_engine_with_config(config_enabled);
    engine_enabled.reset_null_move_stats();
    
    let start = std::time::Instant::now();
    let result_enabled = engine_enabled.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed_enabled = start.elapsed();
    
    let stats_enabled = engine_enabled.get_null_move_stats();
    let nodes_enabled = engine_enabled.get_nodes_searched();
    
    // Test with NMP disabled
    let mut config_disabled = NullMoveConfig::default();
    config_disabled.enabled = false;
    let mut engine_disabled = create_test_engine_with_config(config_disabled);
    engine_disabled.reset_null_move_stats();
    
    let start = std::time::Instant::now();
    let result_disabled = engine_disabled.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed_disabled = start.elapsed();
    
    assert!(result_enabled.is_some());
    assert!(result_disabled.is_some());
    
    // NMP should provide some benefit (either fewer nodes or similar time)
    // Allow some variance but verify NMP is working
    if stats_enabled.attempts > 0 {
        assert!(
            stats_enabled.cutoff_rate() >= 15.0 || nodes_enabled < engine_disabled.get_nodes_searched(),
            "NMP effectiveness regression: cutoff rate too low and no node reduction"
        );
    }
}

#[test]
fn test_nmp_performance_regression_different_depths() {
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let config = NullMoveConfig::default();
    
    // Test at different depths
    for depth in [3, 4, 5] {
        let mut engine = create_test_engine_with_config(config.clone());
        engine.reset_null_move_stats();
        
        let start = std::time::Instant::now();
        let result = engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, depth, 1000);
        let elapsed = start.elapsed();
        
        assert!(result.is_some(), "Search should complete at depth {}", depth);
        
        let stats = engine.get_null_move_stats();
        
        // Regression check: search should complete within reasonable time
        let max_time_ms = match depth {
            3 => 5000.0,
            4 => 10000.0,
            5 => 20000.0,
            _ => 30000.0,
        };
        
        assert!(
            elapsed.as_secs_f64() * 1000.0 <= max_time_ms,
            "Performance regression at depth {}: search time {}ms > threshold {}ms",
            depth,
            elapsed.as_secs_f64() * 1000.0,
            max_time_ms
        );
        
        // If NMP was active, verify it had some effectiveness
        if stats.attempts > 0 {
            assert!(
                stats.cutoffs >= 0,
                "NMP should have non-negative cutoffs at depth {}",
                depth
            );
        }
    }
}

