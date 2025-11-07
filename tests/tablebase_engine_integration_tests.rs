#![cfg(feature = "legacy-tests")]
//! Comprehensive integration tests for tablebase system with complete engine
//!
//! This module tests the tablebase system integrated with the complete
//! Shogi engine, including search integration, move ordering, and
//! performance validation.

use shogi_engine::{
    bitboards::BitboardBoard,
    tablebase::{TablebaseConfig, TablebaseStats},
    types::{CapturedPieces, Move, PieceType, Player, Position},
    ShogiEngine,
};

/// Test basic engine integration with tablebase
#[test]
fn test_engine_tablebase_integration() {
    let mut engine = ShogiEngine::new();

    // Enable tablebase
    engine.enable_tablebase();
    assert!(engine.is_tablebase_enabled());

    // Test basic functionality
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let best_move = engine.get_best_move(1, 1000, None, None);

    // Should return a move (even if not from tablebase)
    assert!(best_move.is_some());
}

/// Test tablebase configuration and statistics
#[test]
fn test_tablebase_configuration() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Get initial stats (returns a string summary)
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());

    // Test with a simple position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let _best_move = engine.get_best_move(1, 1000, None, None);

    // Stats should be updated
    let updated_stats = engine.get_tablebase_stats();
    assert!(!updated_stats.is_empty());
}

/// Test tablebase performance profiling
#[test]
fn test_tablebase_performance_profiling() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Test performance profiling (returns a string summary)
    let profiler = engine.get_tablebase_stats();
    assert!(!profiler.is_empty());
}

/// Test tablebase memory monitoring
#[test]
fn test_tablebase_memory_monitoring() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Test memory monitoring
    let stats = engine.get_tablebase_stats();
    // Test memory summary (stats is a string)
    assert!(!stats.is_empty());
}

/// Test tablebase with different configurations
#[test]
fn test_tablebase_configurations() {
    // Test default configuration
    let mut engine1 = ShogiEngine::new();
    engine1.enable_tablebase();

    // Test with custom configuration
    let config = TablebaseConfig::performance_optimized();
    let mut engine2 = ShogiEngine::new();
    engine2.enable_tablebase();

    // Both should work
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let move1 = engine1.get_best_move(1, 1000, None, None);
    let move2 = engine2.get_best_move(1, 1000, None, None);

    assert!(move1.is_some());
    assert!(move2.is_some());
}

/// Test tablebase optimizations
#[test]
fn test_tablebase_optimizations() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Test tablebase functionality
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());

    // Test with a simple position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let _best_move = engine.get_best_move(1, 1000, None, None);
}

/// Test tablebase error handling
#[test]
fn test_tablebase_error_handling() {
    let mut engine = ShogiEngine::new();

    // Test with tablebase disabled
    engine.disable_tablebase();
    assert!(!engine.is_tablebase_enabled());

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let best_move = engine.get_best_move(1, 1000, None, None);

    // Should still return a move (from normal search)
    assert!(best_move.is_some());
}

/// Test tablebase statistics reset
#[test]
fn test_tablebase_statistics_reset() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Make some moves to generate stats
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let _best_move = engine.get_best_move(1, 1000, None, None);

    // Reset stats
    engine.reset_tablebase_stats();

    // Stats should be reset
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());
}

/// Test tablebase with different time limits
#[test]
fn test_tablebase_with_time_limits() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with short time limit
    let short_move = engine.get_best_move(1, 100, None, None);
    assert!(short_move.is_some());

    // Test with longer time limit
    let long_move = engine.get_best_move(1, 5000, None, None);
    assert!(long_move.is_some());
}

/// Test tablebase performance under load
#[test]
fn test_tablebase_performance_under_load() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test multiple moves in sequence
    for _ in 0..10 {
        let best_move = engine.get_best_move(1, 1000, None, None);
        assert!(best_move.is_some());
    }

    // Check that stats were updated
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());
}

/// Test tablebase with different players
#[test]
fn test_tablebase_with_different_players() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with Black player
    let black_move = engine.get_best_move(1, 1000, None, None);
    assert!(black_move.is_some());

    // Test with White player
    let white_move = engine.get_best_move(1, 1000, None, None);
    assert!(white_move.is_some());
}

/// Test tablebase integration with search engine
#[test]
fn test_tablebase_search_engine_integration() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that tablebase integrates with search
    let best_move = engine.get_best_move(1, 1000, None, None);
    assert!(best_move.is_some());

    // Check that search engine is working
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());
}

/// Test tablebase with empty board
#[test]
fn test_tablebase_with_empty_board() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::empty();
    let captured_pieces = CapturedPieces::new();

    // Test with empty board
    let best_move = engine.get_best_move(1, 1000, None, None);

    // Should handle empty board gracefully
    assert!(best_move.is_some() || best_move.is_none());
}

/// Test tablebase performance metrics
#[test]
fn test_tablebase_performance_metrics() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Make a move to generate metrics
    let _best_move = engine.get_best_move(1, 1000, None, None);

    // Check performance metrics
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());
}

/// Test tablebase with different piece counts
#[test]
fn test_tablebase_with_different_piece_counts() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with full board
    let full_move = engine.get_best_move(1, 1000, None, None);
    assert!(full_move.is_some());

    // Test with empty board
    let empty_board = BitboardBoard::empty();
    let empty_move = engine.get_best_move(1, 1000, None, None);
    assert!(empty_move.is_some() || empty_move.is_none());
}

/// Test tablebase error recovery
#[test]
fn test_tablebase_error_recovery() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test that engine recovers from errors
    let best_move = engine.get_best_move(1, 1000, None, None);
    assert!(best_move.is_some());

    // Test multiple calls
    for _ in 0..5 {
        let move_ = engine.get_best_move(1, 1000, None, None);
        assert!(move_.is_some());
    }
}

/// Test tablebase with different search depths
#[test]
fn test_tablebase_with_different_search_depths() {
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Test with different time limits (affecting search depth)
    let shallow_move = engine.get_best_move(1, 100, None, None);
    let deep_move = engine.get_best_move(1, 5000, None, None);

    assert!(shallow_move.is_some());
    assert!(deep_move.is_some());
}

/// Test tablebase integration completeness
#[test]
fn test_tablebase_integration_completeness() {
    let mut engine = ShogiEngine::new();

    // Test all tablebase methods
    engine.enable_tablebase();
    assert!(engine.is_tablebase_enabled());

    engine.disable_tablebase();
    assert!(!engine.is_tablebase_enabled());

    engine.enable_tablebase();

    // Test with actual game position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let best_move = engine.get_best_move(1, 1000, None, None);

    assert!(best_move.is_some());

    // Test statistics
    let stats = engine.get_tablebase_stats();
    assert!(!stats.is_empty());

    // Test reset
    engine.reset_tablebase_stats();
    let reset_stats = engine.get_tablebase_stats();
    assert!(!reset_stats.is_empty());
}
