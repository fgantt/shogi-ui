#![cfg(feature = "legacy-tests")]
use shogi_engine::opening_book::*;
use shogi_engine::types::*;
/// Integration tests for opening book with ShogiEngine
///
/// This module contains integration tests that test the opening book
/// functionality in the context of the full ShogiEngine.
use shogi_engine::ShogiEngine;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[cfg(test)]
mod engine_integration_tests {
    use super::*;

    #[test]
    fn test_engine_initialization_with_opening_book() {
        let engine = ShogiEngine::new();

        // Engine should be initialized
        assert!(engine.is_opening_book_loaded());

        // Should have some statistics
        let stats = engine.get_opening_book_stats();
        assert!(stats.contains("Positions:"));
        assert!(stats.contains("Moves:"));
        assert!(stats.contains("Loaded: true"));
    }

    #[test]
    fn test_opening_book_loading_from_json() {
        let mut engine = ShogiEngine::new();

        // Test loading from JSON
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"}
                    ]
                }
            }
        ]"#;

        let result = engine.load_opening_book_from_json(json_data);
        assert!(result.is_ok());
        assert!(engine.is_opening_book_loaded());
    }

    #[test]
    fn test_opening_book_loading_from_binary() {
        let mut engine = ShogiEngine::new();

        // Create a test opening book
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
        )];

        book.add_position(fen, moves);
        book = book.mark_loaded();

        // Convert to binary
        let binary_data = book.to_binary().unwrap();

        // Load into engine
        let result = engine.load_opening_book_from_binary(&binary_data);
        assert!(result.is_ok());
        assert!(engine.is_opening_book_loaded());
    }

    #[test]
    fn test_get_best_move_with_opening_book() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        // Test getting best move
        let stop_flag = Arc::new(AtomicBool::new(false));
        let best_move = engine.get_best_move(5, 1000, Some(stop_flag), None);

        // Should find an opening book move
        assert!(best_move.is_some());
        let mv = best_move.unwrap();
        assert_eq!(mv.from, Some(Position::new(2, 6))); // 27
        assert_eq!(mv.to, Position::new(2, 5)); // 26
        assert_eq!(mv.piece_type, PieceType::Rook);
    }

    #[test]
    fn test_get_random_opening_book_move() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book with multiple moves
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"},
                        {"from": "22", "to": "88"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        // Test getting random move multiple times
        let mut found_moves = std::collections::HashSet::new();
        for _ in 0..100 {
            if let Some(random_move) = engine.get_random_opening_book_move() {
                found_moves.insert((random_move.from, random_move.to));
            }
        }

        // Should find at least one move
        assert!(!found_moves.is_empty());
    }

    #[test]
    fn test_get_all_opening_book_moves() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let all_moves = engine.get_all_opening_book_moves();

        // Should have move information
        assert!(!all_moves.is_empty());
        assert!(all_moves.len() >= 2);

        // Each move should contain weight and evaluation info
        for move_info in &all_moves {
            assert!(move_info.contains("weight:"));
            assert!(move_info.contains("eval:"));
        }
    }

    #[test]
    fn test_opening_book_info() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let info = engine.get_opening_book_info();

        // Should contain basic information
        assert!(info.contains("Opening Book Info:"));
        assert!(info.contains("Positions:"));
        assert!(info.contains("Total Moves:"));
        assert!(info.contains("Version:"));
        assert!(info.contains("Current Position:"));
    }

    #[test]
    fn test_opening_book_move_info() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let move_info = engine.get_opening_book_move_info();

        // Should have move information
        assert!(move_info.is_some());
        let info = move_info.unwrap();
        assert!(info.contains("Opening book move:"));
        assert!(info.contains("weight:"));
        assert!(info.contains("eval:"));
        assert!(info.contains("opening:"));
    }

    #[test]
    fn test_fallback_to_search_when_no_opening_book_move() {
        let mut engine = ShogiEngine::new();

        // Load an empty opening book
        let json_data = r#"[]"#;
        engine.load_opening_book_from_json(json_data).unwrap();

        // Should fall back to search
        let stop_flag = Arc::new(AtomicBool::new(false));
        let best_move = engine.get_best_move(1, 1000, Some(stop_flag), None);

        // Should still find a move (from search)
        assert!(best_move.is_some());
    }

    #[test]
    fn test_opening_book_with_different_positions() {
        let mut engine = ShogiEngine::new();

        // Load opening book with multiple positions
        let json_data = r#"[
            {
                "name": "Opening 1",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"}
                    ]
                }
            },
            {
                "name": "Opening 2",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL w - 2": [
                        {"from": "83", "to": "84"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        // Test first position
        let stop_flag = Arc::new(AtomicBool::new(false));
        let best_move1 = engine.get_best_move(5, 1000, Some(stop_flag.clone()), None);
        assert!(best_move1.is_some());

        // Make a move to reach second position
        if let Some(mv) = best_move1 {
            engine.make_move(&mv.to_usi_string()).unwrap();
        }

        // Test second position
        let best_move2 = engine.get_best_move(5, 1000, Some(stop_flag), None);
        assert!(best_move2.is_some());
    }

    #[test]
    fn test_opening_book_statistics() {
        let mut engine = ShogiEngine::new();

        // Load a test opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let stats = engine.get_opening_book_stats();

        // Should contain statistics
        assert!(stats.contains("Positions:"));
        assert!(stats.contains("Moves:"));
        assert!(stats.contains("Version:"));
        assert!(stats.contains("Loaded: true"));

        // Should have at least 1 position and 2 moves
        assert!(stats.contains("Positions: 1"));
        assert!(stats.contains("Moves: 2"));
    }

    #[test]
    fn test_opening_book_with_drop_moves() {
        let mut engine = ShogiEngine::new();

        // Load opening book with drop moves
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "drop", "to": "25"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let stop_flag = Arc::new(AtomicBool::new(false));
        let best_move = engine.get_best_move(5, 1000, Some(stop_flag), None);

        // Should find the drop move
        assert!(best_move.is_some());
        let mv = best_move.unwrap();
        assert_eq!(mv.from, None); // Drop move
        assert_eq!(mv.to, Position::new(2, 4)); // 25
    }

    #[test]
    fn test_opening_book_with_promotion_moves() {
        let mut engine = ShogiEngine::new();

        // Load opening book with promotion moves
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26", "promote": true}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        let stop_flag = Arc::new(AtomicBool::new(false));
        let best_move = engine.get_best_move(5, 1000, Some(stop_flag), None);

        // Should find the promotion move
        assert!(best_move.is_some());
        let mv = best_move.unwrap();
        assert_eq!(mv.is_promotion, true);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_opening_book_lookup_performance() {
        let mut engine = ShogiEngine::new();

        // Load a large opening book
        let json_data = r#"[
            {
                "name": "Test Opening 1",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"},
                        {"from": "22", "to": "88"}
                    ]
                }
            },
            {
                "name": "Test Opening 2",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL w - 2": [
                        {"from": "83", "to": "84"},
                        {"from": "33", "to": "34"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        // Test lookup performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = engine.get_best_move(5, 1000, None, None);
        }
        let duration = start.elapsed();

        // Should be very fast (O(1) lookup)
        assert!(duration.as_millis() < 100); // Should complete in under 100ms
    }

    #[test]
    fn test_opening_book_memory_usage() {
        let mut engine = ShogiEngine::new();

        // Load opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"},
                        {"from": "77", "to": "76"}
                    ]
                }
            }
        ]"#;

        engine.load_opening_book_from_json(json_data).unwrap();

        // Test that memory usage is reasonable
        let stats = engine.get_opening_book_stats();
        assert!(stats.contains("Positions: 1"));
        assert!(stats.contains("Moves: 2"));

        // Test binary serialization size
        let binary_data = engine.opening_book.to_binary().unwrap();
        assert!(binary_data.len() > 0);
        assert!(binary_data.len() < 10000); // Should be reasonably compact
    }

    #[test]
    fn test_concurrent_opening_book_access() {
        let engine = std::sync::Arc::new(ShogiEngine::new());

        // Load opening book
        let json_data = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "27", "to": "26"}
                    ]
                }
            }
        ]"#;

        // Note: This test would need to be modified to work with Arc<ShogiEngine>
        // For now, just test that the engine can be created and cloned
        let _engine_clone = engine.clone();
        assert!(engine.is_opening_book_loaded());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_json_loading() {
        let mut engine = ShogiEngine::new();

        // Test invalid JSON
        let invalid_json = r#"{"invalid": json}"#;
        let result = engine.load_opening_book_from_json(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_json_loading() {
        let mut engine = ShogiEngine::new();

        // Test empty JSON
        let empty_json = r#"[]"#;
        let result = engine.load_opening_book_from_json(empty_json);
        assert!(result.is_ok());

        // Should have no moves
        let stats = engine.get_opening_book_stats();
        assert!(stats.contains("Positions: 0"));
        assert!(stats.contains("Moves: 0"));
    }

    #[test]
    fn test_corrupted_binary_loading() {
        let mut engine = ShogiEngine::new();

        // Test corrupted binary data
        let corrupted_data = vec![0u8; 100];
        let result = engine.load_opening_book_from_binary(&corrupted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_opening_book_with_malformed_moves() {
        let mut engine = ShogiEngine::new();

        // Test JSON with malformed moves
        let malformed_json = r#"[
            {
                "name": "Test Opening",
                "moves": {
                    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                        {"from": "invalid", "to": "26"},
                        {"from": "27", "to": "invalid"}
                    ]
                }
            }
        ]"#;

        let result = engine.load_opening_book_from_json(malformed_json);
        // Should handle gracefully (either succeed with valid moves or fail cleanly)
        // The exact behavior depends on the implementation
    }
}

#[cfg(test)]
mod streaming_mode_integration_tests {
    use super::*;

    #[test]
    fn test_streaming_mode_enable() {
        let mut book = OpeningBook::new();
        book.enable_streaming_mode(1024);

        assert!(book.get_stats().streaming_enabled);
        assert_eq!(book.get_stats().chunk_size, 1024);
    }

    #[test]
    fn test_streaming_progress_tracking() {
        let mut book = OpeningBook::new();
        book.enable_streaming_mode(1024);

        // Initially no progress
        let progress = book.get_streaming_progress();
        assert!(progress.is_some());
        let progress = progress.unwrap();
        assert_eq!(progress.chunks_loaded, 0);
        assert_eq!(progress.progress_percentage, 0.0);
    }

    #[test]
    fn test_streaming_state_save_load() {
        let mut book = OpeningBook::new();
        book.enable_streaming_mode(1024);

        // Save empty state
        let state = book.save_streaming_state();
        assert!(state.is_some());

        // Load into new book
        let mut new_book = OpeningBook::new();
        new_book.enable_streaming_mode(1024);
        let result = new_book.load_streaming_state(state.unwrap());
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod coverage_analysis_integration_tests {
    use super::*;
    use shogi_engine::opening_book::{CoverageAnalyzer, CoverageReport};

    #[test]
    fn test_coverage_report_generation() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
        )];
        book.add_position(fen, moves);
        book = book.mark_loaded();

        let report = CoverageAnalyzer::generate_coverage_report(&book);
        assert!(report.depth_stats.total_openings > 0);
    }
}
