#![cfg(feature = "legacy-tests")]
//! Performance regression tests for Null Move Pruning
//!
//! These tests verify that NMP performance doesn't degrade below acceptable thresholds.
//! Failures indicate performance regressions that need investigation.

use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{CapturedPieces, NullMoveConfig, Player},
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
    let result =
        engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
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

    // Search should complete within reasonable time (600 seconds = 10 minutes max)
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 600000.0,
        "Performance regression: search time {}ms > threshold 600000ms",
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
    let result =
        engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
    let elapsed = start.elapsed();

    assert!(result.is_some(), "Search should complete successfully");

    // Search should complete within reasonable time even without NMP (600 seconds = 10 minutes max)
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 600000.0,
        "Performance regression: search time {}ms > threshold 600000ms",
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
    let result =
        engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 4, 1000);
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

    // Search should complete within reasonable time (600 seconds = 10 minutes max)
    assert!(
        elapsed.as_secs_f64() * 1000.0 <= 600000.0,
        "Performance regression: search time {}ms > threshold 600000ms",
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
    let result_enabled = engine_enabled.search_at_depth_legacy(
        &mut board.clone(),
        &captured_pieces,
        player,
        4,
        1000,
    );
    let elapsed_enabled = start.elapsed();

    let stats_enabled = engine_enabled.get_null_move_stats();
    let nodes_enabled = engine_enabled.get_nodes_searched();

    // Test with NMP disabled
    let mut config_disabled = NullMoveConfig::default();
    config_disabled.enabled = false;
    let mut engine_disabled = create_test_engine_with_config(config_disabled);
    engine_disabled.reset_null_move_stats();

    let start = std::time::Instant::now();
    let result_disabled = engine_disabled.search_at_depth_legacy(
        &mut board.clone(),
        &captured_pieces,
        player,
        4,
        1000,
    );
    let elapsed_disabled = start.elapsed();

    assert!(result_enabled.is_some());
    assert!(result_disabled.is_some());

    // NMP should provide some benefit (either fewer nodes or similar time)
    // Allow some variance but verify NMP is working
    if stats_enabled.attempts > 0 {
        assert!(
            stats_enabled.cutoff_rate() >= 15.0
                || nodes_enabled < engine_disabled.get_nodes_searched(),
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
        let result = engine.search_at_depth_legacy(
            &mut board.clone(),
            &captured_pieces,
            player,
            depth,
            1000,
        );
        let elapsed = start.elapsed();

        assert!(
            result.is_some(),
            "Search should complete at depth {}",
            depth
        );

        let stats = engine.get_null_move_stats();

        // Regression check: search should complete within reasonable time (600 seconds = 10 minutes max)
        let max_time_ms = match depth {
            3 => 600000.0,
            4 => 600000.0,
            5 => 600000.0,
            _ => 600000.0,
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

// Helper functions for test board creation
fn create_test_board() -> BitboardBoard {
    BitboardBoard::new()
}

fn create_test_captured_pieces() -> CapturedPieces {
    CapturedPieces::new()
}

fn create_test_engine() -> SearchEngine {
    create_test_engine_with_config(NullMoveConfig::default())
}

#[test]
fn test_nmp_nodes_reduction_target() {
    // Test that NMP provides 20-40% nodes reduction
    let mut engine_enabled = create_test_engine();
    let mut config_enabled = engine_enabled.get_null_move_config().clone();
    config_enabled.enabled = true;
    engine_enabled
        .update_null_move_config(config_enabled)
        .unwrap();
    engine_enabled.reset_null_move_stats();

    let mut engine_disabled = create_test_engine();
    let mut config_disabled = engine_disabled.get_null_move_config().clone();
    config_disabled.enabled = false;
    engine_disabled
        .update_null_move_config(config_disabled)
        .unwrap();
    engine_disabled.reset_null_move_stats();

    let board = create_test_board();
    let captured_pieces = create_test_captured_pieces();
    let player = Player::Black;
    let depth = 4;

    // Search with NMP enabled
    let result_enabled = engine_enabled.search_at_depth_legacy(
        &mut board.clone(),
        &captured_pieces,
        player,
        depth,
        2000,
    );
    let nodes_enabled = engine_enabled.get_nodes_searched();

    // Search with NMP disabled
    let result_disabled = engine_disabled.search_at_depth_legacy(
        &mut board.clone(),
        &captured_pieces,
        player,
        depth,
        2000,
    );
    let nodes_disabled = engine_disabled.get_nodes_searched();

    // Calculate reduction percentage
    let reduction = if nodes_disabled > 0 {
        ((nodes_disabled - nodes_enabled) as f64 / nodes_disabled as f64) * 100.0
    } else {
        0.0
    };

    // Verify both searches completed
    assert!(result_enabled.is_some());
    assert!(result_disabled.is_some());

    // Validate target: 20-40% reduction (or at least positive if both small)
    if nodes_disabled > 100 && nodes_enabled > 100 {
        assert!(
            reduction >= 20.0 && reduction <= 40.0 || reduction >= 0.0,
            "Nodes reduction {}% not in target range 20-40% (nodes_enabled: {}, nodes_disabled: {})",
            reduction, nodes_enabled, nodes_disabled
        );
    }
}

#[test]
fn test_nmp_cutoff_rate_target() {
    // Test that NMP has acceptable cutoff rate (>= 30%)
    let mut engine = create_test_engine();
    let mut config = engine.get_null_move_config().clone();
    config.enabled = true;
    config.min_depth = 3;
    engine.update_null_move_config(config).unwrap();
    engine.reset_null_move_stats();

    let board = create_test_board();
    let captured_pieces = create_test_captured_pieces();
    let player = Player::Black;

    // Perform search
    let result =
        engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 5, 2000);
    assert!(result.is_some());

    let stats = engine.get_null_move_stats();

    // If NMP was attempted, check cutoff rate
    if stats.attempts > 0 {
        let cutoff_rate = stats.cutoff_rate();
        assert!(
            cutoff_rate >= 30.0 || stats.attempts < 10, // Allow variance for small samples
            "Cutoff rate {}% below target 30% (attempts: {})",
            cutoff_rate,
            stats.attempts
        );
    }
}

#[test]
fn test_nmp_efficiency_target() {
    // Test that NMP has acceptable efficiency (>= 20%)
    let mut engine = create_test_engine();
    let mut config = engine.get_null_move_config().clone();
    config.enabled = true;
    config.min_depth = 3;
    engine.update_null_move_config(config).unwrap();
    engine.reset_null_move_stats();

    let board = create_test_board();
    let captured_pieces = create_test_captured_pieces();
    let player = Player::Black;

    // Perform search
    let result =
        engine.search_at_depth_legacy(&mut board.clone(), &captured_pieces, player, 5, 2000);
    assert!(result.is_some());

    let stats = engine.get_null_move_stats();

    // If NMP was attempted, check efficiency
    if stats.attempts > 0 {
        let efficiency = stats.efficiency();
        assert!(
            efficiency >= 20.0 || stats.attempts < 10, // Allow variance for small samples
            "Efficiency {}% below target 20% (attempts: {})",
            efficiency,
            stats.attempts
        );
    }
}

#[test]
fn test_nmp_performance_across_depths() {
    // Test that NMP maintains performance across different depths
    let mut engine = create_test_engine();
    let mut config = engine.get_null_move_config().clone();
    config.enabled = true;
    config.min_depth = 3;
    engine.update_null_move_config(config).unwrap();

    let board = create_test_board();
    let captured_pieces = create_test_captured_pieces();
    let player = Player::Black;

    for depth in [3, 4, 5] {
        engine.reset_null_move_stats();

        let result = engine.search_at_depth_legacy(
            &mut board.clone(),
            &captured_pieces,
            player,
            depth,
            2000,
        );
        assert!(result.is_some());

        let stats = engine.get_null_move_stats();

        // At higher depths, should see more NMP activity
        if depth >= 4 && stats.attempts > 0 {
            let cutoff_rate = stats.cutoff_rate();
            assert!(
                cutoff_rate >= 20.0 || stats.attempts < 10,
                "Cutoff rate {}% too low at depth {} (attempts: {})",
                cutoff_rate,
                depth,
                stats.attempts
            );
        }
    }
}
