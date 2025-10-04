use shogi_engine::bitboards::attack_patterns::AttackTables;
use shogi_engine::types::{PieceType, Player};
use std::time::Instant;

fn main() {
    println!("=== Attack Pattern Precomputation Demo ===\n");
    
    // Create attack tables
    println!("Creating attack tables...");
    let start_time = Instant::now();
    let tables = AttackTables::new();
    let creation_time = start_time.elapsed();
    
    println!("✓ Attack tables created in {:.2}ms", creation_time.as_secs_f64() * 1000.0);
    println!("✓ Memory usage: ~{} bytes", std::mem::size_of_val(&tables));
    
    // Display metadata
    let metadata = tables.memory_stats();
    println!("✓ Generation time: {:.2}ms", metadata.initialization_time.as_secs_f64() * 1000.0);
    println!("✓ Validation passed: {}\n", metadata.validation_passed);
    
    // Demonstrate O(1) lookup performance
    println!("Demonstrating O(1) lookup performance...");
    
    let test_squares = vec![0, 4, 40, 76, 80]; // Corner, edge, center, edge, corner
    let piece_types = vec![
        PieceType::King,
        PieceType::Knight,
        PieceType::Gold,
        PieceType::Silver,
        PieceType::PromotedPawn,
    ];
    
    for &square in &test_squares {
        let row = square / 9;
        let col = square % 9;
        println!("Square {} ({},{})", square, row, col);
        
        for piece_type in &piece_types {
            let black_pattern = tables.get_attack_pattern(square, *piece_type, Player::Black);
            let white_pattern = tables.get_attack_pattern(square, *piece_type, Player::White);
            
            let black_count = black_pattern.count_ones();
            let white_count = white_pattern.count_ones();
            
            println!("  {:?}: Black={} attacks, White={} attacks", 
                    piece_type, black_count, white_count);
        }
        println!();
    }
    
    // Performance test
    println!("Performance test - 1,000,000 lookups...");
    let start_time = Instant::now();
    
    let mut total_attacks = 0u32;
    for _ in 0..1000000 {
        let square = (total_attacks % 81) as u8;
        let piece_type = match total_attacks % 4 {
            0 => PieceType::King,
            1 => PieceType::Knight,
            2 => PieceType::Gold,
            _ => PieceType::Silver,
        };
        let player = if total_attacks % 2 == 0 { Player::Black } else { Player::White };
        
        let pattern = tables.get_attack_pattern(square, piece_type, player);
        total_attacks += pattern.count_ones();
    }
    
    let lookup_time = start_time.elapsed();
    println!("✓ 1,000,000 lookups completed in {:.2}ms", lookup_time.as_secs_f64() * 1000.0);
    println!("✓ Average lookup time: {:.2}ns", lookup_time.as_nanos() / 1000000);
    println!("✓ Total attacks found: {}", total_attacks);
    
    // Demonstrate memory efficiency
    println!("\nMemory efficiency demonstration:");
    println!("✓ AttackTables size: {} bytes", std::mem::size_of::<AttackTables>());
    println!("✓ Total patterns stored: {}", 81 * 10); // 81 squares × 10 piece types
    println!("✓ Bytes per pattern: {} bytes", std::mem::size_of::<u128>());
    println!("✓ Cache line friendly: {} (64-byte aligned)", std::mem::align_of::<AttackTables>() >= 64);
    
    // Show some specific patterns
    println!("\nSample attack patterns:");
    let center_square = 40; // (4,4)
    
    println!("King attacks from center square {}:", center_square);
    let king_pattern = tables.get_attack_pattern(center_square, PieceType::King, Player::Black);
    print_bitboard_pattern(king_pattern);
    
    println!("\nKnight attacks from center square {}:", center_square);
    let knight_pattern = tables.get_attack_pattern(center_square, PieceType::Knight, Player::Black);
    print_bitboard_pattern(knight_pattern);
    
    println!("\nGold attacks from center square {}:", center_square);
    let gold_pattern = tables.get_attack_pattern(center_square, PieceType::Gold, Player::Black);
    print_bitboard_pattern(gold_pattern);
}

fn print_bitboard_pattern(pattern: u128) {
    println!("   a b c d e f g h i");
    for row in 0..9 {
        print!("{} ", 9 - row);
        for col in 0..9 {
            let square = row * 9 + col;
            let bit = (pattern >> square) & 1;
            if bit != 0 {
                print!("X ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
}
