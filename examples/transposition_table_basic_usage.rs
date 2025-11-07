//! Basic usage example for the transposition table system
//!
//! This example demonstrates the fundamental usage of the transposition table
//! components including creation, storage, retrieval, and statistics.

use shogi_engine::bitboards::*;
use shogi_engine::search::*;
use shogi_engine::types::*;

fn main() {
    println!("🎯 Basic Transposition Table Usage Example");
    println!("===========================================");

    // 1. Create a transposition table with default configuration
    println!("\n📋 Creating transposition table...");
    let config = TranspositionConfig::default();
    let mut tt = ThreadSafeTranspositionTable::new(config);
    println!("✅ Transposition table created with default configuration");

    // 2. Create a sample transposition entry
    println!("\n📝 Creating sample transposition entry...");
    let entry = TranspositionEntry::new(
        150,
        5,
        TranspositionFlag::Exact,
        Some(Move {
            from: Some(Position { row: 7, col: 4 }),
            to: Position { row: 6, col: 4 },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        }),
        0x123456789ABCDEF0,
        0,
        EntrySource::MainSearch,
    );
    println!(
        "✅ Entry created with hash: 0x{:X}, depth: {}, score: {}",
        entry.hash_key, entry.depth, entry.score
    );

    // 3. Store the entry in the transposition table
    println!("\n💾 Storing entry in transposition table...");
    tt.store(entry.clone());
    println!("✅ Entry stored successfully");

    // 4. Retrieve the entry from the transposition table
    println!("\n🔍 Retrieving entry from transposition table...");
    match tt.probe(entry.hash_key, entry.depth) {
        Some(retrieved_entry) => {
            println!("✅ Entry found!");
            println!("   Hash: 0x{:X}", retrieved_entry.hash_key);
            println!("   Depth: {}", retrieved_entry.depth);
            println!("   Score: {}", retrieved_entry.score);
            println!("   Flag: {:?}", retrieved_entry.flag);
            println!("   Best Move: {:?}", retrieved_entry.best_move);
            println!("   Age: {}", retrieved_entry.age);
        }
        None => {
            println!("❌ Entry not found");
        }
    }

    // 5. Get statistics from the transposition table
    println!("\n📊 Getting transposition table statistics...");
    let stats = tt.get_stats();
    println!("✅ Statistics retrieved:");
    println!("   Total probes: {}", stats.total_probes);
    println!("   Total stores: {}", stats.stores);
    println!("   Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("   Stores: {}", stats.stores);
    println!("   Replacements: {}", stats.replacements);

    // 6. Demonstrate multiple entries and replacement
    println!("\n🔄 Demonstrating multiple entries and replacement...");
    for i in 0..10 {
        let test_entry = TranspositionEntry::new(
            i as i32 * 10,
            1,
            TranspositionFlag::Exact,
            None,
            i as u64,
            0,
            EntrySource::MainSearch,
        );
        tt.store(test_entry);
    }
    println!("✅ Stored 10 test entries");

    // Check how many we can retrieve
    let mut found_count = 0;
    for i in 0..10 {
        if tt.probe(i as u64, 1).is_some() {
            found_count += 1;
        }
    }
    println!("✅ Found {}/10 entries after storage", found_count);

    // 7. Demonstrate hash calculation
    println!("\n🧮 Demonstrating hash calculation...");
    let hash_calc = ShogiHashHandler::new(1000);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    let position_hash = hash_calc.get_position_hash(&board, Player::Black, &captured);
    println!("✅ Position hash calculated: 0x{:X}", position_hash);

    // 8. Demonstrate move ordering integration
    println!("\n🎯 Demonstrating move ordering integration...");
    let mut move_orderer = TranspositionMoveOrderer::new();
    move_orderer.set_transposition_table(&tt);

    // Create sample moves
    let sample_moves = vec![
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
            from: Some(Position { row: 7, col: 2 }),
            to: Position { row: 6, col: 2 },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        },
    ];

    let ordered_moves = move_orderer.order_moves(
        &sample_moves,
        &board,
        &captured,
        Player::Black,
        3,
        -1000,
        1000,
        None,
    );

    println!("✅ Move ordering completed:");
    println!("   Original moves: {}", sample_moves.len());
    println!("   Ordered moves: {}", ordered_moves.len());

    // Get move ordering statistics
    let hints = move_orderer.get_move_ordering_hints(&board, &captured, Player::Black, 3);
    println!("   TT best move: {:?}", hints.best_move);
    println!("   TT depth: {}", hints.tt_depth);
    if let Some(score) = hints.tt_score {
        println!("   TT score: {}", score);
    }

    let ordering_stats = move_orderer.get_stats();
    println!("   TT hint moves: {}", ordering_stats.tt_hint_moves);
    println!("   Killer move hits: {}", ordering_stats.killer_move_hits);
    println!("   History hits: {}", ordering_stats.history_hits);

    println!("\n🎉 Basic usage example completed successfully!");
    println!("\n📚 Key Takeaways:");
    println!("   • Transposition tables store and retrieve search results efficiently");
    println!("   • Hash keys uniquely identify positions");
    println!("   • Move ordering improves search performance");
    println!("   • Statistics help monitor and tune performance");
    println!("   • The system is designed for both native and WASM environments");
}
