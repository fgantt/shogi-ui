//! Move Ordering Usage Examples
//!
//! This example demonstrates practical usage of the move ordering system
//! with the actual available API.

use shogi_engine::search::move_ordering::MoveOrdering;
use shogi_engine::types::{Move, PieceType, Player, Position};

fn main() {
    println!("=== Move Ordering Usage Examples ===\n");

    example_1_basic_usage();
    example_2_with_killer_moves();
    example_3_statistics();
}

/// Example 1: Basic move ordering
fn example_1_basic_usage() {
    println!("Example 1: Basic Move Ordering");
    println!("{}", "-".repeat(50));

    let mut orderer = MoveOrdering::new();

    // Create some test moves
    let moves = vec![
        Move::new_move(
            Position::new(6, 4),
            Position::new(5, 4),
            PieceType::Pawn,
            Player::Black,
            false,
        ),
        Move::new_move(
            Position::new(8, 1),
            Position::new(7, 1),
            PieceType::Lance,
            Player::Black,
            false,
        ),
        Move::new_move(
            Position::new(7, 1),
            Position::new(6, 3),
            PieceType::Knight,
            Player::Black,
            false,
        ),
    ];

    println!("Ordering {} moves...", moves.len());

    // Order the moves
    match orderer.order_moves(&moves) {
        Ok(ordered_moves) => {
            println!("Successfully ordered {} moves", ordered_moves.len());
            for (i, move_) in ordered_moves.iter().enumerate() {
                println!("  {}: {}", i + 1, move_.to_usi_string());
            }
        }
        Err(e) => {
            eprintln!("Error ordering moves: {}", e);
        }
    }

    println!();
}

/// Example 2: Using killer moves
fn example_2_with_killer_moves() {
    println!("Example 2: Killer Moves");
    println!("{}", "-".repeat(50));

    let mut orderer = MoveOrdering::new();

    let move1 = Move::new_move(
        Position::new(6, 4),
        Position::new(5, 4),
        PieceType::Pawn,
        Player::Black,
        false,
    );
    let move2 = Move::new_move(
        Position::new(8, 1),
        Position::new(7, 1),
        PieceType::Lance,
        Player::Black,
        false,
    );

    // Add killer move (simulates a beta cutoff)
    orderer.add_killer_move(move1.clone());
    println!("Added killer move: {}", move1.to_usi_string());

    // Check if moves are killer moves
    println!("Is move1 a killer? {}", orderer.is_killer_move(&move1));
    println!("Is move2 a killer? {}", orderer.is_killer_move(&move2));

    // Get killer move statistics
    let (hits, misses, hit_rate, stored) = orderer.get_killer_move_stats();
    println!("\nKiller Move Statistics:");
    println!("  Stored: {}", stored);
    println!("  Hits: {}, Misses: {}", hits, misses);
    println!("  Hit rate: {:.2}%", hit_rate);

    println!();
}

/// Example 3: Performance statistics
fn example_3_statistics() {
    println!("Example 3: Performance Statistics");
    println!("{}", "-".repeat(50));

    let mut orderer = MoveOrdering::new();

    // Create test moves
    let moves = vec![
        Move::new_move(
            Position::new(6, 4),
            Position::new(5, 4),
            PieceType::Pawn,
            Player::Black,
            false,
        ),
        Move::new_move(
            Position::new(6, 5),
            Position::new(5, 5),
            PieceType::Pawn,
            Player::Black,
            false,
        ),
    ];

    // Perform operations
    println!("Performing 100 move ordering operations...");
    for _ in 0..100 {
        let _ = orderer.order_moves(&moves);
    }

    // Get comprehensive statistics
    let stats = orderer.get_stats();
    println!("\nPerformance Statistics:");
    println!("  Total moves ordered: {}", stats.total_moves_ordered);
    println!("  Moves sorted: {}", stats.moves_sorted);
    println!(
        "  Average ordering time: {:.2}μs",
        stats.avg_ordering_time_us
    );
    println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate);
    println!(
        "  Memory usage: {} bytes ({:.2} KB)",
        stats.memory_usage_bytes,
        stats.memory_usage_bytes as f64 / 1024.0
    );

    // Get current memory usage breakdown
    let current_memory_breakdown = orderer.get_current_memory_usage();
    println!("  Current memory breakdown: {:?}", current_memory_breakdown);

    println!();
}
