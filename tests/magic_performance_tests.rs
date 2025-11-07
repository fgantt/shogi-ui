#![cfg(feature = "legacy-tests")]
//! Performance tests and benchmarks for magic bitboards
//!
//! These tests measure and compare performance of magic bitboards
//! against traditional ray-casting methods.

use shogi_engine::bitboards::magic::AttackGenerator;
use shogi_engine::{
    types::{MagicTable, Piece, PieceType, Player, Position},
    BitboardBoard,
};
use std::time::Instant;

#[test]
fn test_magic_lookup_performance() {
    let table = MagicTable::new().unwrap();
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        for square in (0..81).step_by(3) {
            let _ = table.get_attacks(square, PieceType::Rook, 0);
        }
    }
    let duration = start.elapsed();

    println!(
        "Magic lookup: {} lookups in {:?}",
        iterations * 27,
        duration
    );
    println!("Average: {:?} per lookup", duration / (iterations * 27));

    // Should complete reasonably quickly
    assert!(duration.as_millis() < 5000, "Magic lookups should be fast");
}

#[test]
fn test_raycast_performance_baseline() {
    let generator = AttackGenerator::new();
    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        for square in (0..81).step_by(3) {
            let _ = generator.generate_rook_attacks(square as u8, 0);
        }
    }
    let duration = start.elapsed();

    println!("Ray-casting: {} lookups in {:?}", iterations * 27, duration);
    println!("Average: {:?} per lookup", duration / (iterations * 27));
}

#[test]
fn test_magic_vs_raycast_speedup() {
    let table = MagicTable::new().unwrap();
    let generator = AttackGenerator::new();
    let iterations = 10_000;

    // Measure magic lookup
    let magic_start = Instant::now();
    for _ in 0..iterations {
        for square in 0..81 {
            let _ = table.get_attacks(square, PieceType::Rook, 0);
        }
    }
    let magic_duration = magic_start.elapsed();

    // Measure ray-casting
    let raycast_start = Instant::now();
    for _ in 0..iterations {
        for square in 0..81 {
            let _ = generator.generate_rook_attacks(square as u8, 0);
        }
    }
    let raycast_duration = raycast_start.elapsed();

    let speedup = raycast_duration.as_nanos() as f64 / magic_duration.as_nanos() as f64;

    println!("Magic: {:?}", magic_duration);
    println!("Raycast: {:?}", raycast_duration);
    println!("Speedup: {:.2}x", speedup);

    // Magic should be at least as fast as raycast
    assert!(
        magic_duration <= raycast_duration,
        "Magic should not be slower than raycast"
    );
}

#[test]
fn test_sliding_move_generation_performance() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Place a rook in the center
    let rook_pos = Position::new(4, 4);
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::Black,
        },
        rook_pos,
    );

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = board.generate_magic_sliding_moves(rook_pos, PieceType::Rook, Player::Black);
    }

    let duration = start.elapsed();
    println!(
        "Move generation: {} iterations in {:?}",
        iterations, duration
    );
    println!("Average: {:?} per generation", duration / iterations);

    assert!(
        duration.as_millis() < 1000,
        "Move generation should be fast"
    );
}

#[test]
fn test_table_creation_performance() {
    let iterations = 10;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = MagicTable::new().unwrap();
    }

    let duration = start.elapsed();
    println!("Table creation: {} tables in {:?}", iterations, duration);
    println!("Average: {:?} per table", duration / iterations);

    // Table creation should be reasonable (not instant, but not too slow)
    assert!(
        duration.as_secs() < 60,
        "Table creation should complete in reasonable time"
    );
}

#[test]
fn test_with_blockers_performance() {
    let table = MagicTable::new().unwrap();
    let iterations = 10_000;

    // Test with various blocker configurations
    let blocker_configs = vec![0u128, 0xFFu128, 0xFF00u128, 0xFF0000u128, 0xFFFFFFFFu128];

    let start = Instant::now();
    for _ in 0..iterations {
        for square in (0..81).step_by(5) {
            for &blockers in &blocker_configs {
                let _ = table.get_attacks(square, PieceType::Rook, blockers);
            }
        }
    }
    let duration = start.elapsed();

    println!(
        "Blocker variations: {} lookups in {:?}",
        iterations * 17 * 5,
        duration
    );
    assert!(
        duration.as_millis() < 5000,
        "Lookups with blockers should be fast"
    );
}

#[test]
fn test_cache_efficiency_simulation() {
    let table = MagicTable::new().unwrap();

    // Simulate repeated lookups (as would happen in search)
    let test_squares = [20, 40, 60];
    let test_blockers = [0u128, 0xFFu128, 0xFF00u128];
    let iterations = 100_000;

    let start = Instant::now();
    for _ in 0..iterations {
        for &square in &test_squares {
            for &blockers in &test_blockers {
                let _ = table.get_attacks(square, PieceType::Rook, blockers);
            }
        }
    }
    let duration = start.elapsed();

    println!(
        "Repeated lookups: {} lookups in {:?}",
        iterations * 9,
        duration
    );
    println!("Average: {:?} per lookup", duration / (iterations * 9));
}

#[test]
fn test_worst_case_performance() {
    let table = MagicTable::new().unwrap();
    let iterations = 10_000;

    // Test all squares with complex blocker patterns
    let start = Instant::now();
    for i in 0..iterations {
        for square in 0..81 {
            let blockers = ((i * 31) % 256) as u128; // Varying blocker patterns
            let _ = table.get_attacks(square, PieceType::Rook, blockers);
            let _ = table.get_attacks(square, PieceType::Bishop, blockers);
        }
    }
    let duration = start.elapsed();

    println!(
        "Worst case: {} lookups in {:?}",
        iterations * 81 * 2,
        duration
    );
    assert!(
        duration.as_secs() < 10,
        "Worst case should still be reasonable"
    );
}

#[test]
fn test_serialization_performance() {
    let table = MagicTable::new().unwrap();
    let iterations = 100;

    // Test serialization performance
    let serialize_start = Instant::now();
    let mut serialized = Vec::new();
    for _ in 0..iterations {
        serialized = table.serialize().unwrap();
    }
    let serialize_duration = serialize_start.elapsed();

    println!(
        "Serialization: {} iterations in {:?}",
        iterations, serialize_duration
    );
    println!("Data size: {} bytes", serialized.len());

    // Test deserialization performance
    let deserialize_start = Instant::now();
    for _ in 0..iterations {
        let _ = MagicTable::deserialize(&serialized).unwrap();
    }
    let deserialize_duration = deserialize_start.elapsed();

    println!(
        "Deserialization: {} iterations in {:?}",
        iterations, deserialize_duration
    );

    assert!(
        serialize_duration.as_millis() < 1000,
        "Serialization should be fast"
    );
    assert!(
        deserialize_duration.as_millis() < 1000,
        "Deserialization should be fast"
    );
}

#[test]
fn test_memory_usage_estimation() {
    let table = MagicTable::new().unwrap();
    let stats = table.performance_stats();

    let estimated_memory = stats.total_attack_patterns * 16; // u128 = 16 bytes

    println!(
        "Estimated memory usage: {} bytes ({} KB)",
        estimated_memory,
        estimated_memory / 1024
    );
    println!("Attack patterns: {}", stats.total_attack_patterns);
    println!("Memory efficiency: {:.2}%", stats.memory_efficiency * 100.0);

    // Memory usage should be reasonable (< 10MB for Shogi)
    assert!(
        estimated_memory < 10_000_000,
        "Memory usage should be under 10MB"
    );
}

#[test]
fn test_concurrent_access_simulation() {
    let table = MagicTable::new().unwrap();
    let iterations = 10_000;

    // Simulate multiple "threads" accessing the table
    // (Actually sequential, but tests immutability)
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate 4 concurrent lookups
        let _ = table.get_attacks(20, PieceType::Rook, 0);
        let _ = table.get_attacks(40, PieceType::Bishop, 0);
        let _ = table.get_attacks(60, PieceType::Rook, 0xFFu128);
        let _ = table.get_attacks(80, PieceType::Bishop, 0xFF00u128);
    }

    let duration = start.elapsed();
    println!(
        "Concurrent access simulation: {} lookups in {:?}",
        iterations * 4,
        duration
    );

    assert!(
        duration.as_millis() < 1000,
        "Concurrent access should be fast"
    );
}

#[test]
fn test_bishop_vs_rook_performance() {
    let table = MagicTable::new().unwrap();
    let iterations = 10_000;

    // Measure rook performance
    let rook_start = Instant::now();
    for _ in 0..iterations {
        for square in 0..81 {
            let _ = table.get_attacks(square, PieceType::Rook, 0);
        }
    }
    let rook_duration = rook_start.elapsed();

    // Measure bishop performance
    let bishop_start = Instant::now();
    for _ in 0..iterations {
        for square in 0..81 {
            let _ = table.get_attacks(square, PieceType::Bishop, 0);
        }
    }
    let bishop_duration = bishop_start.elapsed();

    println!("Rook lookups: {:?}", rook_duration);
    println!("Bishop lookups: {:?}", bishop_duration);

    // Both should be fast and similar
    assert!(
        rook_duration.as_millis() < 5000,
        "Rook lookups should be fast"
    );
    assert!(
        bishop_duration.as_millis() < 5000,
        "Bishop lookups should be fast"
    );
}

#[test]
fn test_full_game_simulation_performance() {
    let mut board = BitboardBoard::new_with_magic_support().unwrap();
    board.init_sliding_generator().ok();

    // Set up a typical mid-game position with several sliding pieces
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::Black,
        },
        Position::new(0, 0),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Bishop,
            player: Player::Black,
        },
        Position::new(1, 1),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Rook,
            player: Player::White,
        },
        Position::new(7, 7),
    );
    board.place_piece(
        Piece {
            piece_type: PieceType::Bishop,
            player: Player::White,
        },
        Position::new(6, 6),
    );

    // Add some blockers
    for i in 0..4 {
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::Black,
            },
            Position::new(2, i * 2),
        );
        board.place_piece(
            Piece {
                piece_type: PieceType::Pawn,
                player: Player::White,
            },
            Position::new(6, i * 2 + 1),
        );
    }

    let iterations = 1_000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Generate moves for all sliding pieces
        let _ =
            board.generate_magic_sliding_moves(Position::new(0, 0), PieceType::Rook, Player::Black);
        let _ = board.generate_magic_sliding_moves(
            Position::new(1, 1),
            PieceType::Bishop,
            Player::Black,
        );
        let _ =
            board.generate_magic_sliding_moves(Position::new(7, 7), PieceType::Rook, Player::White);
        let _ = board.generate_magic_sliding_moves(
            Position::new(6, 6),
            PieceType::Bishop,
            Player::White,
        );
    }

    let duration = start.elapsed();
    println!(
        "Full game simulation: {} move generations in {:?}",
        iterations * 4,
        duration
    );
    println!("Average: {:?} per position", duration / iterations);

    assert!(
        duration.as_millis() < 2000,
        "Full game simulation should be fast"
    );
}
