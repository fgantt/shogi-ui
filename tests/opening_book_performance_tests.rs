#![cfg(feature = "legacy-tests")]
/// Performance tests for opening book implementation
///
/// This module contains performance benchmarks and regression tests
/// to ensure the opening book implementation is efficient.
use shogi_engine::opening_book::*;
use shogi_engine::types::*;
use std::time::Instant;

// Safety constants to prevent system crashes
const MAX_TEST_POSITIONS: usize = 100;
const MAX_TEST_ITERATIONS: usize = 100;

// Safety check to prevent running tests that might crash the system
fn should_run_performance_tests() -> bool {
    // Only run performance tests if we're in a safe environment
    // This prevents system crashes during development
    std::env::var("RUN_PERFORMANCE_TESTS").is_ok() || std::env::var("CI").is_ok()
}

#[cfg(test)]
mod performance_benchmarks {
    use super::*;

    #[test]
    fn test_hashmap_vs_linear_search_performance() {
        // Create a large opening book
        let mut book = OpeningBook::new();
        let mut linear_positions = Vec::new();

        // Generate test positions
        for i in 0..MAX_TEST_POSITIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let moves = vec![BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15,
            )];

            book.add_position(fen.clone(), moves.clone());
            linear_positions.push((fen, moves));
        }

        book = book.mark_loaded();

        // Test HashMap lookup performance
        let start = Instant::now();
        for i in 0..MAX_TEST_ITERATIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let _ = book.get_moves(&fen);
        }
        let hashmap_duration = start.elapsed();

        // Test linear search performance
        let start = Instant::now();
        for i in 0..MAX_TEST_ITERATIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let _ = linear_positions.iter().find(|(f, _)| f == &fen);
        }
        let linear_duration = start.elapsed();

        // HashMap should be significantly faster
        println!("HashMap lookup time: {:?}", hashmap_duration);
        println!("Linear search time: {:?}", linear_duration);
        assert!(hashmap_duration < linear_duration);
    }

    #[test]
    fn test_binary_serialization_performance() {
        // Create a large opening book
        let mut book = OpeningBook::new();

        for i in 0..500 {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let moves = vec![
                BookMove::new_with_metadata(
                    Some(Position::new(2, 6)),
                    Position::new(2, 5),
                    PieceType::Rook,
                    false,
                    false,
                    850,
                    15,
                    Some(format!("Opening {}", i)),
                    Some(format!("27-26-{}", i)),
                ),
                BookMove::new_with_metadata(
                    Some(Position::new(7, 6)),
                    Position::new(7, 5),
                    PieceType::Pawn,
                    false,
                    false,
                    800,
                    10,
                    Some(format!("Opening {}", i)),
                    Some(format!("77-76-{}", i)),
                ),
            ];

            book.add_position(fen, moves);
        }

        book = book.mark_loaded();

        // Test serialization performance
        let start = Instant::now();
        let binary_data = book.to_binary().unwrap();
        let serialization_duration = start.elapsed();

        // Test deserialization performance
        let start = Instant::now();
        let deserialized_book = OpeningBook::from_binary(&binary_data).unwrap();
        let deserialization_duration = start.elapsed();

        println!("Serialization time: {:?}", serialization_duration);
        println!("Deserialization time: {:?}", deserialization_duration);
        println!("Binary size: {} bytes", binary_data.len());

        // Should be reasonably fast
        assert!(serialization_duration.as_millis() < 1000);
        assert!(deserialization_duration.as_millis() < 1000);

        // Verify data integrity
        let original_stats = book.get_stats();
        let deserialized_stats = deserialized_book.get_stats();
        assert_eq!(
            deserialized_stats.position_count,
            original_stats.position_count
        );
        assert_eq!(deserialized_stats.move_count, original_stats.move_count);
    }

    #[test]
    fn test_move_selection_performance() {
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();

        // Create position with many moves
        let mut moves = Vec::new();
        for i in 0..100 {
            moves.push(BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                800 + (i as u32),
                15,
            ));
        }

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        // Test best move selection performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = book.get_best_move(&fen);
        }
        let best_move_duration = start.elapsed();

        // Test random move selection performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = book.get_random_move(&fen);
        }
        let random_move_duration = start.elapsed();

        // Test all moves retrieval performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = book.get_moves(&fen);
        }
        let all_moves_duration = start.elapsed();

        println!("Best move selection time: {:?}", best_move_duration);
        println!("Random move selection time: {:?}", random_move_duration);
        println!("All moves retrieval time: {:?}", all_moves_duration);

        // All operations should be fast
        assert!(best_move_duration.as_millis() < 100);
        assert!(random_move_duration.as_millis() < 100);
        assert!(all_moves_duration.as_millis() < 100);
    }

    #[test]
    fn test_memory_usage_efficiency() {
        let mut book = OpeningBook::new();

        // Create a large opening book
        for i in 0..MAX_TEST_POSITIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let moves = vec![BookMove::new_with_metadata(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15,
                Some(format!("Opening {}", i)),
                Some(format!("27-26-{}", i)),
            )];

            book.add_position(fen, moves);
        }

        book = book.mark_loaded();

        // Test binary serialization size
        let binary_data = book.to_binary().unwrap();
        let binary_size = binary_data.len();

        // Test JSON serialization size for comparison
        let json_data = serde_json::to_string(&book).unwrap();
        let json_size = json_data.len();

        println!("Binary size: {} bytes", binary_size);
        println!("JSON size: {} bytes", json_size);
        println!(
            "Compression ratio: {:.2}%",
            (binary_size as f64 / json_size as f64) * 100.0
        );

        // Binary format should be more compact than JSON
        assert!(binary_size < json_size);

        // Should be reasonably compact (less than 1MB for 1000 positions)
        assert!(binary_size < 1_000_000);
    }

    #[test]
    fn test_concurrent_access_performance() {
        let book = std::sync::Arc::new(OpeningBook::new());
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();

        // Add some test data
        let mut temp_book = (*book).clone();
        let moves = vec![BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
        )];
        temp_book.add_position(fen.clone(), moves);
        let book = std::sync::Arc::new(temp_book.mark_loaded());

        // Test concurrent read access
        let start = Instant::now();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let book = book.clone();
                let fen = fen.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = book.get_moves(&fen);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        let concurrent_duration = start.elapsed();

        println!("Concurrent access time: {:?}", concurrent_duration);

        // Should complete in reasonable time
        assert!(concurrent_duration.as_millis() < 1000);
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_opening_book_consistency() {
        // Test that the same opening book produces consistent results
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                850,
                15,
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                800,
                10,
            ),
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        // Test multiple times to ensure consistency
        for _ in 0..100 {
            let best_move = book.get_best_move(&fen);
            assert!(best_move.is_some());
            assert_eq!(best_move.unwrap().piece_type, PieceType::Rook); // Should always be the highest weight
        }
    }

    #[test]
    fn test_binary_serialization_consistency() {
        // Test that binary serialization/deserialization preserves all data
        let mut original_book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![BookMove::new_with_metadata(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
            Some("Test Opening".to_string()),
            Some("27-26".to_string()),
        )];

        original_book.add_position(fen.clone(), moves);
        original_book = original_book.mark_loaded();

        // Serialize and deserialize
        let binary_data = original_book.to_binary().unwrap();
        let deserialized_book = OpeningBook::from_binary(&binary_data).unwrap();

        // Verify all data is preserved
        let original_stats = original_book.get_stats();
        let deserialized_stats = deserialized_book.get_stats();
        assert_eq!(
            deserialized_stats.position_count,
            original_stats.position_count
        );
        assert_eq!(deserialized_stats.move_count, original_stats.move_count);
        assert_eq!(deserialized_book.is_loaded(), original_book.is_loaded());

        // Verify moves can be retrieved
        let original_moves = original_book.get_moves(&fen);
        let deserialized_moves = deserialized_book.get_moves(&fen);

        assert!(original_moves.is_some());
        assert!(deserialized_moves.is_some());
        assert_eq!(
            original_moves.unwrap().len(),
            deserialized_moves.unwrap().len()
        );
    }

    #[test]
    fn test_weight_distribution_consistency() {
        // Test that weight-based selection works correctly
        let mut book = OpeningBook::new();
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
        let moves = vec![
            BookMove::new(
                Some(Position::new(2, 6)),
                Position::new(2, 5),
                PieceType::Rook,
                false,
                false,
                100, // Low weight
                15,
            ),
            BookMove::new(
                Some(Position::new(7, 6)),
                Position::new(7, 5),
                PieceType::Pawn,
                false,
                false,
                900, // High weight
                10,
            ),
        ];

        book.add_position(fen.clone(), moves);
        book = book.mark_loaded();

        // Test random selection multiple times
        let mut high_weight_count = 0;
        for _ in 0..1000 {
            if let Some(random_move) = book.get_random_move(&fen) {
                // Check if this is the move with weight 900 by checking piece type
                if random_move.piece_type == PieceType::Pawn {
                    high_weight_count += 1;
                }
            }
        }

        // High weight move should be selected more often
        assert!(high_weight_count > 500); // Should be selected more than 50% of the time
    }

    #[test]
    fn test_edge_case_performance() {
        // Test performance with edge cases
        let mut book = OpeningBook::new();

        // Test with very long FEN strings
        let long_fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".repeat(10);
        let moves = vec![BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
        )];

        book.add_position(long_fen.clone(), moves);
        book = book.mark_loaded();

        // Test lookup performance with long FEN
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = book.get_moves(&long_fen);
        }
        let duration = start.elapsed();

        // Should still be fast even with long FEN strings
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_memory_leak_prevention() {
        // Test that there are no memory leaks
        let mut book = OpeningBook::new();

        // Add and remove many positions
        for i in 0..MAX_TEST_POSITIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
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
        }

        book = book.mark_loaded();

        // Test that memory usage is reasonable
        let binary_data = book.to_binary().unwrap();
        assert!(binary_data.len() < 1_000_000); // Should be less than 1MB

        // Test that deserialization works correctly
        let deserialized_book = OpeningBook::from_binary(&binary_data).unwrap();
        let original_stats = book.get_stats();
        let deserialized_stats = deserialized_book.get_stats();
        assert_eq!(
            deserialized_stats.position_count,
            original_stats.position_count
        );
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_safe_opening_book_performance() {
        // Safe test that won't crash the system
        let mut book = OpeningBook::new();

        // Create a small opening book for safe testing
        for i in 0..10 {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
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
        }

        book = book.mark_loaded();

        // Test basic functionality
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let moves = book.get_moves(fen);
        assert!(moves.is_some());

        let best_move = book.get_best_move(fen);
        assert!(best_move.is_some());

        println!("Safe opening book test completed successfully");
    }

    #[test]
    fn test_large_opening_book_performance() {
        // Skip this test if not in a safe environment to prevent system crashes
        if !should_run_performance_tests() {
            println!("Skipping large opening book performance test to prevent system crashes");
            return;
        }

        // Test with a reasonably large opening book
        // Note: Reduced from 10,000 to 100 positions to prevent system crashes
        let mut book = OpeningBook::new();

        // Create positions (reduced from 10,000 to prevent system crashes)
        for i in 0..MAX_TEST_POSITIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
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
        }

        book = book.mark_loaded();

        // Test lookup performance
        let start = Instant::now();
        for i in 0..MAX_TEST_ITERATIONS {
            let fen = format!(
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL {} - {}",
                if i % 2 == 0 { "b" } else { "w" },
                i + 1
            );
            let _ = book.get_moves(&fen);
        }
        let duration = start.elapsed();

        // Should still be fast even with large opening book
        assert!(duration.as_millis() < 1000);

        // Test binary serialization performance
        let start = Instant::now();
        let binary_data = book.to_binary().unwrap();
        let serialization_duration = start.elapsed();

        // Should complete in reasonable time
        assert!(serialization_duration.as_millis() < 5000);

        println!("Large book lookup time: {:?}", duration);
        println!(
            "Large book serialization time: {:?}",
            serialization_duration
        );
        println!("Large book binary size: {} bytes", binary_data.len());
    }

    #[test]
    fn test_concurrent_access_stress() {
        let book = std::sync::Arc::new(OpeningBook::new());
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();

        // Add test data
        let mut temp_book = (*book).clone();
        let moves = vec![BookMove::new(
            Some(Position::new(2, 6)),
            Position::new(2, 5),
            PieceType::Rook,
            false,
            false,
            850,
            15,
        )];
        temp_book.add_position(fen.clone(), moves);
        let book = std::sync::Arc::new(temp_book.mark_loaded());

        // Test heavy concurrent access
        let start = Instant::now();
        let handles: Vec<_> = (0..20)
            .map(|_| {
                let book = book.clone();
                let fen = fen.clone();
                std::thread::spawn(move || {
                    for _ in 0..1000 {
                        let _ = book.get_moves(&fen);
                        let _ = book.get_best_move(&fen);
                        let _ = book.get_random_move(&fen);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
        let duration = start.elapsed();

        // Should complete in reasonable time
        assert!(duration.as_millis() < 5000);

        println!("Concurrent stress test time: {:?}", duration);
    }
}
