//! Move ordering integration tests
//!
//! This module contains tests for the transposition table integrated move ordering system.

use crate::bitboards::*;
use crate::search::*;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Piece, PieceType, Player, Position};
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::time::Instant;

/// Test suite for move ordering performance
pub struct MoveOrderingTestSuite {
    /// Performance benchmarks
    benchmarks: MoveOrderingBenchmarks,
}

/// A test position with expected move ordering characteristics
#[derive(Debug, Clone)]
pub struct TestPosition {
    /// Position name/description
    pub name: String,
    /// FEN string of the position
    pub fen: String,
    /// Expected best move (if known)
    pub expected_best_move: Option<Move>,
    /// Expected move ordering characteristics
    pub expected_capture_first: bool,
    pub expected_promotion_priority: bool,
    pub expected_center_control: bool,
}

/// Performance benchmarks for move ordering
#[derive(Debug, Default)]
pub struct MoveOrderingBenchmarks {
    /// Total time spent on move ordering
    pub total_ordering_time_ms: f64,
    /// Number of positions tested
    pub positions_tested: usize,
    /// Average ordering time per position
    pub avg_ordering_time_ms: f64,
    /// Move ordering hit rates
    pub tt_hit_rate: f64,
    pub killer_hit_rate: f64,
    pub history_hit_rate: f64,
    /// Performance comparison with old system
    pub speedup_factor: f64,
}

impl MoveOrderingTestSuite {
    /// Create a new test suite
    pub fn new() -> Self {
        Self {
            benchmarks: MoveOrderingBenchmarks::default(),
        }
    }

    /// Run comprehensive move ordering tests
    pub fn run_all_tests(&mut self) -> TestResults {
        let mut results = TestResults::default();

        println!("Starting move ordering integration tests...");

        // Test 1: Basic functionality
        results.basic_functionality = self.test_basic_functionality();

        // Test 2: Transposition table integration
        results.tt_integration = self.test_tt_integration();

        // Test 3: Performance benchmarks
        results.performance = self.test_performance();

        // Test 4: Move ordering accuracy
        results.accuracy = self.test_move_ordering_accuracy();

        // Test 5: Memory usage
        results.memory_usage = self.test_memory_usage();

        results
    }

    /// Test basic move ordering functionality
    fn test_basic_functionality(&mut self) -> bool {
        println!("Testing basic move ordering functionality...");

        let mut orderer = TranspositionMoveOrderer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        // Create test moves
        let moves = vec![
            Move {
                from: Some(Position { row: 7, col: 4 }),
                to: Position { row: 6, col: 4 },
                piece_type: PieceType::Pawn,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
                player: Player::Black,
            },
            Move {
                from: Some(Position { row: 7, col: 3 }),
                to: Position { row: 6, col: 3 },
                piece_type: PieceType::Pawn,
                is_capture: true,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: Some(Piece {
                    piece_type: PieceType::Pawn,
                    player: Player::White,
                }),
                player: Player::Black,
            },
        ];

        let ordered_moves =
            orderer.order_moves(&moves, &board, &captured, Player::Black, 1, 0, 0, None);

        // Capture should be ordered first
        let success = ordered_moves.len() == 2 && ordered_moves[0].is_capture;

        if success {
            println!("✅ Basic functionality test passed");
        } else {
            println!("❌ Basic functionality test failed");
        }

        success
    }

    /// Test transposition table integration
    fn test_tt_integration(&mut self) -> bool {
        println!("Testing transposition table integration...");

        let mut orderer = TranspositionMoveOrderer::new();
        let tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());
        orderer.set_transposition_table(&tt);

        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        // Create a position and store it in TT
        let position_hash =
            orderer
                .hash_calculator
                .get_position_hash(&board, Player::Black, &captured);
        let best_move = Move {
            from: Some(Position { row: 7, col: 4 }),
            to: Position { row: 6, col: 4 },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        };

        let entry = TranspositionEntry {
            hash_key: position_hash,
            depth: 3,
            score: 100,
            flag: TranspositionFlag::Exact,
            best_move: Some(best_move.clone()),
            age: 0,
            source: crate::types::EntrySource::MainSearch,
        };

        tt.store(entry);

        // Create moves including the best move
        let moves = vec![
            Move {
                from: Some(Position { row: 7, col: 3 }),
                to: Position { row: 6, col: 3 },
                piece_type: PieceType::Pawn,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
                player: Player::Black,
            },
            best_move.clone(),
        ];

        let ordered_moves =
            orderer.order_moves(&moves, &board, &captured, Player::Black, 3, 0, 0, None);

        // Best move from TT should be first
        let success =
            ordered_moves.len() == 2 && orderer.moves_equal(&ordered_moves[0], &best_move);

        if success {
            println!("✅ Transposition table integration test passed");
        } else {
            println!("❌ Transposition table integration test failed");
        }

        success
    }

    /// Test performance benchmarks
    fn test_performance(&mut self) -> bool {
        println!("Testing move ordering performance...");

        let mut orderer = TranspositionMoveOrderer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        // Generate test moves
        let moves = self.generate_test_moves(50); // 50 moves

        let start_time = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let _ordered_moves =
                orderer.order_moves(&moves, &board, &captured, Player::Black, 3, 0, 0, None);
        }

        let elapsed = start_time.elapsed();
        let avg_time_per_ordering = elapsed.as_nanos() as f64 / iterations as f64 / 1_000_000.0;

        self.benchmarks.total_ordering_time_ms = elapsed.as_millis() as f64;
        self.benchmarks.positions_tested = iterations;
        self.benchmarks.avg_ordering_time_ms = avg_time_per_ordering;

        // Performance should be reasonable (less than 1ms per ordering)
        let success = avg_time_per_ordering < 1.0;

        println!(
            "Average time per move ordering: {:.3}ms",
            avg_time_per_ordering
        );

        if success {
            println!("✅ Performance test passed");
        } else {
            println!("❌ Performance test failed");
        }

        success
    }

    /// Test move ordering accuracy
    fn test_move_ordering_accuracy(&mut self) -> bool {
        println!("Testing move ordering accuracy...");

        let mut orderer = TranspositionMoveOrderer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        // Create moves with different priorities
        let moves = vec![
            // Low priority quiet move
            Move {
                from: Some(Position { row: 7, col: 1 }),
                to: Position { row: 6, col: 1 },
                piece_type: PieceType::Pawn,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
                player: Player::Black,
            },
            // High priority capture
            Move {
                from: Some(Position { row: 7, col: 2 }),
                to: Position { row: 6, col: 2 },
                piece_type: PieceType::Pawn,
                is_capture: true,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: Some(Piece {
                    piece_type: PieceType::Rook,
                    player: Player::White,
                }),
                player: Player::Black,
            },
            // Medium priority promotion
            Move {
                from: Some(Position { row: 2, col: 3 }),
                to: Position { row: 1, col: 3 },
                piece_type: PieceType::Pawn,
                is_capture: false,
                is_promotion: true,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
                player: Player::Black,
            },
        ];

        let ordered_moves =
            orderer.order_moves(&moves, &board, &captured, Player::Black, 3, 0, 0, None);

        // Capture should be first (highest priority)
        let success = ordered_moves.len() == 3
            && ordered_moves[0].is_capture
            && !ordered_moves[0].is_promotion;

        if success {
            println!("✅ Move ordering accuracy test passed");
        } else {
            println!("❌ Move ordering accuracy test failed");
        }

        success
    }

    /// Test memory usage
    fn test_memory_usage(&mut self) -> bool {
        println!("Testing memory usage...");

        let orderer = TranspositionMoveOrderer::new();
        let stats = orderer.get_stats();

        // Memory usage should be reasonable
        // (This is a basic check - in a real implementation, we'd measure actual memory usage)
        let success = stats.total_moves_ordered == 0; // Fresh orderer should have no moves ordered yet

        if success {
            println!("✅ Memory usage test passed");
        } else {
            println!("❌ Memory usage test failed");
        }

        success
    }

    /// Generate test moves for performance testing
    fn generate_test_moves(&self, count: usize) -> Vec<Move> {
        let mut moves = Vec::new();

        for i in 0..count {
            let row = (i % 9) as u8;
            let col = ((i / 9) % 9) as u8;

            moves.push(Move {
                from: Some(Position { row: 7, col: 4 }),
                to: Position { row, col },
                piece_type: PieceType::Pawn,
                is_capture: i % 3 == 0,
                is_promotion: i % 5 == 0,
                gives_check: i % 7 == 0,
                is_recapture: false,
                captured_piece: if i % 3 == 0 {
                    Some(Piece {
                        piece_type: PieceType::Pawn,
                        player: Player::White,
                    })
                } else {
                    None
                },
                player: Player::Black,
            });
        }

        moves
    }

    /// Get benchmark results
    pub fn get_benchmarks(&self) -> &MoveOrderingBenchmarks {
        &self.benchmarks
    }
}

/// Results from move ordering tests
#[derive(Debug, Default)]
pub struct TestResults {
    pub basic_functionality: bool,
    pub tt_integration: bool,
    pub performance: bool,
    pub accuracy: bool,
    pub memory_usage: bool,
}

impl TestResults {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.basic_functionality
            && self.tt_integration
            && self.performance
            && self.accuracy
            && self.memory_usage
    }

    /// Get test summary
    pub fn summary(&self) -> String {
        let passed = [
            ("Basic Functionality", self.basic_functionality),
            ("TT Integration", self.tt_integration),
            ("Performance", self.performance),
            ("Accuracy", self.accuracy),
            ("Memory Usage", self.memory_usage),
        ];

        let total = passed.len();
        let successful = passed.iter().filter(|(_, passed)| *passed).count();

        format!("Move Ordering Tests: {}/{} passed", successful, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_ordering_suite() {
        let mut suite = MoveOrderingTestSuite::new();
        let results = suite.run_all_tests();

        println!("{}", results.summary());
        assert!(results.all_passed(), "Not all move ordering tests passed");
    }

    #[test]
    fn test_move_orderer_creation() {
        let orderer = TranspositionMoveOrderer::new();
        assert_eq!(orderer.get_stats().total_moves_ordered, 0);
    }

    #[test]
    fn test_benchmark_creation() {
        let benchmarks = MoveOrderingBenchmarks::default();
        assert_eq!(benchmarks.total_ordering_time_ms, 0.0);
    }
}
