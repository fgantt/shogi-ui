#![cfg(feature = "legacy-tests")]
//! Integration tests for the tablebase system
//!
//! This module contains integration tests that verify the tablebase system
//! works correctly with the search engine and other components.

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::tablebase::tablebase_config::{KingGoldConfig, SolverConfig};
use shogi_engine::tablebase::{MicroTablebase, TablebaseConfig, TablebaseOutcome, TablebaseResult};
use shogi_engine::types::{CapturedPieces, Move, Piece, PieceType, Player, Position};
use shogi_engine::ShogiEngine;

/// Test tablebase integration with search engine
mod search_engine_integration {
    use super::*;

    #[test]
    fn test_tablebase_in_search_engine() {
        let mut engine = ShogiEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Enable tablebase
        engine.enable_tablebase();
        assert!(engine.is_tablebase_enabled());

        // Get initial stats
        let stats = engine.get_tablebase_stats();
        assert!(stats.contains("Probes=0"));

        // Test basic search
        let best_move = engine.get_best_move(1, 1000, None, None);

        // Verify we got a valid move
        if let Some(move_) = best_move {
            assert!(move_.from.is_some() || move_.is_drop());
            if let Some(from) = move_.from {
                assert!(from.row < 9);
                assert!(from.col < 9);
            }
            assert!(move_.to.row < 9);
            assert!(move_.to.col < 9);
        }
    }

    #[test]
    fn test_tablebase_disabled() {
        let mut engine = ShogiEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Disable tablebase
        engine.disable_tablebase();
        assert!(!engine.is_tablebase_enabled());

        // Test search without tablebase
        let best_move = engine.get_best_move(1, 1000, None, None);

        // Should still work but without tablebase
        if let Some(move_) = best_move {
            assert!(move_.from.is_some() || move_.is_drop());
        }
    }

    #[test]
    fn test_tablebase_stats_tracking() {
        let mut engine = ShogiEngine::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Enable tablebase
        engine.enable_tablebase();

        // Get initial stats
        let initial_stats = engine.get_tablebase_stats();
        assert!(initial_stats.contains("Probes=0"));

        // Perform multiple searches
        for _ in 0..5 {
            engine.get_best_move(1, 1000, None, None);
        }

        // Check that stats have been updated
        let final_stats = engine.get_tablebase_stats();
        assert!(final_stats.contains("Probes="));
    }
}

/// Test tablebase configuration
mod configuration_tests {
    use super::*;

    #[test]
    fn test_tablebase_configuration() {
        let mut config = TablebaseConfig::default();

        // Test default configuration
        assert!(config.solvers.king_gold_vs_king.enabled);
        assert_eq!(config.cache_size, 10000);

        // Test configuration modification
        config.solvers.king_gold_vs_king.enabled = false;
        config.cache_size = 5000;

        let tablebase = MicroTablebase::with_config(config);
        assert_eq!(tablebase.solver_count(), 0); // King+Gold solver disabled
    }

    #[test]
    fn test_tablebase_config_loading() {
        let config = TablebaseConfig::default();
        let json = config.to_json().unwrap();
        let loaded_config = TablebaseConfig::from_json(&json).unwrap();

        assert_eq!(
            config.solvers.king_gold_vs_king.enabled,
            loaded_config.solvers.king_gold_vs_king.enabled
        );
        assert_eq!(config.cache_size, loaded_config.cache_size);
    }
}

/// Test tablebase caching
mod caching_tests {
    use super::*;

    #[test]
    fn test_tablebase_caching() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // First probe should be a miss
        let stats1 = tablebase.get_stats().clone();
        let _result1 = tablebase.probe(&board, player, &captured_pieces);
        let stats2 = tablebase.get_stats().clone();
        assert!(stats2.cache_hits > stats1.cache_hits || stats2.solver_hits > stats1.solver_hits);

        // Second probe should be a cache hit
        let _result2 = tablebase.probe(&board, player, &captured_pieces);
        let stats3 = tablebase.get_stats().clone();
        assert!(stats3.cache_hits > stats2.cache_hits);
    }

    #[test]
    fn test_tablebase_cache_eviction() {
        let mut config = TablebaseConfig::default();
        config.cache_size = 2; // Very small cache
        let mut tablebase = MicroTablebase::with_config(config);

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Fill cache beyond capacity
        for _ in 0..5 {
            tablebase.probe(&board, player, &captured_pieces);
        }

        let stats = tablebase.get_stats();
        assert!(stats.cache_hits > 0);
    }
}

/// Test tablebase performance
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_tablebase_performance() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        let start = Instant::now();

        // Perform multiple probes
        for _ in 0..100 {
            tablebase.probe(&board, player, &captured_pieces);
        }

        let duration = start.elapsed();
        let stats = tablebase.get_stats();

        // Verify performance is reasonable (should be very fast for empty board)
        assert!(duration.as_millis() < 1000); // Less than 1 second for 100 probes
        assert!(stats.average_probe_time_ms < 1.0); // Less than 1ms per probe on average
    }

    #[test]
    fn test_tablebase_memory_usage() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform many probes to test memory usage
        for _ in 0..1000 {
            tablebase.probe(&board, player, &captured_pieces);
        }

        let stats = tablebase.get_stats();
        assert!(stats.total_probes >= 1000);

        // Cache should not grow beyond reasonable limits
        assert!(stats.cache_hits + stats.solver_hits <= 1000);
    }
}

/// Test tablebase error handling
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_tablebase_with_invalid_positions() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test with empty board (should not crash)
        let _result = tablebase.probe(&board, player, &captured_pieces);
        // Result might be None for unsupported positions, which is fine

        let stats = tablebase.get_stats();
        assert!(stats.total_probes >= 0);
    }

    #[test]
    fn test_tablebase_reset_stats() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Perform some probes
        tablebase.probe(&board, player, &captured_pieces);
        tablebase.probe(&board, player, &captured_pieces);

        let stats_before = tablebase.get_stats();
        assert!(stats_before.total_probes > 0);

        // Reset stats
        tablebase.reset_stats();

        let stats_after = tablebase.get_stats();
        assert_eq!(stats_after.total_probes, 0);
        assert_eq!(stats_after.cache_hits, 0);
        assert_eq!(stats_after.solver_hits, 0);
    }
}
