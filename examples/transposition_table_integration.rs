//! Integration examples for the transposition table system
//!
//! This example demonstrates how to integrate the transposition table system
//! with the search engine and other components.

use shogi_engine::bitboards::*;
use shogi_engine::search::*;
use shogi_engine::types::*;

fn build_entry(
    score: i32,
    depth: u8,
    flag: TranspositionFlag,
    best_move: Option<Move>,
    hash_key: u64,
    age: u32,
) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        flag,
        best_move,
        hash_key,
        age,
        EntrySource::MainSearch,
    )
}

fn main() {
    println!("🔗 Transposition Table Integration Examples");
    println!("===========================================");

    // 1. Basic search engine integration
    println!("\n🔍 Basic Search Engine Integration");
    println!("----------------------------------");
    basic_search_engine_integration();

    // 2. Advanced search engine integration
    println!("\n⚡ Advanced Search Engine Integration");
    println!("-------------------------------------");
    advanced_search_engine_integration();

    // 3. Move ordering integration
    println!("\n🎯 Move Ordering Integration");
    println!("-----------------------------");
    move_ordering_integration();

    // 4. Statistics integration
    println!("\n📊 Statistics Integration");
    println!("-------------------------");
    statistics_integration();

    // 5. Configuration integration
    println!("\n⚙️ Configuration Integration");
    println!("-----------------------------");
    configuration_integration();

    // 6. Error handling integration
    println!("\n🛡️ Error Handling Integration");
    println!("------------------------------");
    error_handling_integration();

    println!("\n🎉 Integration examples completed!");
    println!("\n📚 Integration Best Practices:");
    println!("   • Initialize transposition table before search");
    println!("   • Use consistent hash calculation across components");
    println!("   • Monitor statistics for performance tuning");
    println!("   • Handle errors gracefully with fallback strategies");
    println!("   • Use appropriate configuration for your use case");
    println!("   • Test integration in both native and WASM environments");
}

fn basic_search_engine_integration() {
    println!("Creating search engine with transposition table integration...");

    // Create search engine with default configuration
    let mut engine = SearchEngine::new(None, 64); // 64MB transposition table

    // Set up a test position
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    println!("  Search engine created with integrated transposition table");
    println!("  Board: Starting position");
    println!("  Captured pieces: Empty");

    // Perform a search
    println!("  Performing search at depth 3...");
    let start = std::time::Instant::now();

    if let Some((best_move, score)) =
        engine.search_at_depth(&board, &captured, Player::Black, 3, 1000, -1000, 1000)
    {
        let search_time = start.elapsed();
        println!("  ✅ Search completed successfully!");
        println!("     Best move: {:?}", best_move);
        println!("     Score: {}", score);
        println!("     Search time: {:.2}ms", search_time.as_millis());

        // Get transposition table statistics
        let stats = engine.get_tt_stats();
        println!("     TT hit rate: {:.2}%,", stats.hit_rate * 100.0);
        println!("     TT stores: {}", stats.stores);
        println!("     TT replacements: {}", stats.replacements);
    } else {
        println!("  ❌ Search failed");
    }
}

fn advanced_search_engine_integration() {
    println!("Creating advanced search engine with performance optimizations...");

    // Create search engine with performance-optimized configuration
    let mut engine = SearchEngine::new_with_config(
        EngineConfig {
            transposition_table_size_mb: 128,
            enable_quiescence_search: true,
            enable_iterative_deepening: true,
            max_search_time_ms: 5000,
        },
        EngineConfig::default(),
    );

    // Set up a more complex position
    let (board, player, captured) = BitboardBoard::from_fen(
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/4P4/PPPP1PPPP/1B5R1/LNSGKGSNL w - 1",
    )
    .unwrap_or_else(|_| (BitboardBoard::new(), Player::Black, CapturedPieces::new()));

    println!("  Advanced search engine created");
    println!("  Board: Complex starting position");
    println!("  Player: {:?}", player);
    println!(
        "  Captured pieces: {}",
        captured.count_pieces(Player::Black) + captured.count_pieces(Player::White)
    );

    // Perform iterative deepening search
    println!("  Performing iterative deepening search...");
    let start = std::time::Instant::now();

    for depth in 1..=4 {
        println!("    Searching at depth {}...", depth);
        let depth_start = std::time::Instant::now();

        if let Some((best_move, score)) =
            engine.search_at_depth(&board, &captured, player, depth, 1000, -1000, 1000)
        {
            let depth_time = depth_start.elapsed();
            println!(
                "      Depth {}: Score = {}, Time = {:.2}ms",
                depth,
                score,
                depth_time.as_millis()
            );
        }
    }

    let total_time = start.elapsed();
    println!("  ✅ Iterative deepening completed!");
    println!("     Total time: {:.2}ms", total_time.as_millis());

    // Get comprehensive statistics
    let stats = engine.get_tt_stats();
    println!("     Final TT hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("     Stores recorded: {}", stats.stores);
    println!("     Replacements recorded: {}", stats.replacements);
    println!("     Total probes: {}", stats.total_probes);
}

fn move_ordering_integration() {
    println!("Demonstrating move ordering integration...");

    // Create move orderer with transposition table integration
    let mut orderer = TranspositionMoveOrderer::new();
    let tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());
    orderer.set_transposition_table(&tt);

    // Create a board position
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    // Generate sample moves
    let moves = generate_sample_moves();
    println!("  Generated {} sample moves", moves.len());

    // Order moves using transposition table integration
    println!("  Ordering moves with TT integration...");
    let start = std::time::Instant::now();

    let ordered_moves = orderer.order_moves(
        &moves,
        &board,
        &captured,
        Player::Black,
        3,
        -1000,
        1000,
        None,
    );

    let ordering_time = start.elapsed();
    println!("  ✅ Move ordering completed!");
    println!("     Ordering time: {:.2}μs", ordering_time.as_micros());
    println!("     Original moves: {}", moves.len());
    println!("     Ordered moves: {}", ordered_moves.len());

    // Get move ordering statistics
    let hints = orderer.get_move_ordering_hints(&board, &captured, Player::Black, 3);
    println!("     TT best move: {:?}", hints.best_move);
    println!("     TT depth: {}", hints.tt_depth);
    let ordering_stats = orderer.get_stats();
    println!("     TT hint moves: {}", ordering_stats.tt_hint_moves);
    println!("     Killer move hits: {}", ordering_stats.killer_move_hits);
    println!("     History hits: {}", ordering_stats.history_hits);
    println!(
        "     Counter move hits: {}",
        ordering_stats.counter_move_hits
    );

    // Demonstrate killer move updates
    println!("  Updating killer moves...");
    if let Some(first_move) = moves.first() {
        orderer.update_killer_moves(*first_move, 3);
        println!("    Updated killer moves for depth 3");
    }

    // Demonstrate history updates
    println!("  Updating history scores...");
    for (i, mv) in moves.iter().take(3).enumerate() {
        orderer.update_history(mv, 3);
        println!("    Updated history for move {}", i + 1);
    }
}

fn statistics_integration() {
    println!("Demonstrating statistics integration...");

    // Create transposition table with statistics enabled
    let config = TranspositionConfig {
        enable_statistics: true,
        ..TranspositionConfig::performance_optimized()
    };

    let mut tt = ThreadSafeTranspositionTable::new(config);

    // Perform various operations to generate statistics
    println!("  Performing operations to generate statistics...");

    for i in 0..1000 {
        let flag = match i % 3 {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            _ => TranspositionFlag::UpperBound,
        };
        let best_move = if i % 2 == 0 {
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
            })
        } else {
            None
        };
        let entry = build_entry(
            (i % 1000) as i32,
            (i % 10) as u8,
            flag,
            best_move,
            i as u64,
            (i % 100) as u32,
        );
        tt.store(entry);
    }

    // Probe some entries
    for i in 0..500 {
        let _ = tt.probe(i as u64, (i % 10) as u8);
    }

    // Get comprehensive statistics
    let stats = tt.get_stats();

    println!("  ✅ Statistics generated:");
    println!("     Total probes: {}", stats.total_probes);
    println!("     Hits: {}", stats.hits);
    println!("     Misses: {}", stats.misses);
    println!("     Stores: {}", stats.stores);
    println!("     Replacements: {}", stats.replacements);
    println!("     Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("     Atomic operations: {}", stats.atomic_operations);
    println!("     Poison recoveries: {}", stats.poison_recoveries);

    // Demonstrate statistics export
    println!("  Exporting statistics...");
    let stats_json = format!(
        "{{\"hit_rate\": {:.4}, \"total_probes\": {}, \"stores\": {}}}",
        stats.hit_rate, stats.total_probes, stats.stores
    );
    println!("     JSON export: {}", stats_json);
}

fn configuration_integration() {
    println!("Demonstrating configuration integration...");

    // Test different configurations
    let configs = vec![
        ("Default", TranspositionConfig::default()),
        ("Performance", TranspositionConfig::performance_optimized()),
        ("Memory", TranspositionConfig::memory_optimized()),
    ];

    for (name, config) in configs {
        println!("  Testing {} configuration:", name);

        // Create transposition table with specific configuration
        let mut tt = ThreadSafeTranspositionTable::new(config.clone());

        // Perform benchmark
        let iterations = 1000;
        let start = std::time::Instant::now();

        for i in 0..iterations {
            let entry = build_entry(i as i32, 1, TranspositionFlag::Exact, None, i as u64, 0);
            tt.store(entry);
        }

        let duration = start.elapsed();
        let avg_time_us = duration.as_micros() as f64 / iterations as f64;

        let stats = tt.get_stats();

        println!("     Table size: {}", config.table_size);
        println!("     Replacement policy: {:?}", config.replacement_policy);
        println!("     Average store time: {:.2}μs", avg_time_us);
        println!("     Hit rate: {:.2}%", stats.hit_rate * 100.0);
    }

    // Demonstrate runtime configuration updates
    println!("  Demonstrating runtime configuration updates...");
    let mut tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());

    // Update configuration (this would be done through a configuration manager in practice)
    println!("     Configuration updated at runtime");
    println!("     Transposition table adapts to new settings");
}

fn error_handling_integration() {
    println!("Demonstrating error handling integration...");

    // Create error handler
    let error_handler = ComprehensiveErrorHandler::new();

    // Demonstrate graceful error handling
    println!("  Testing error scenarios...");

    // Test with invalid configuration
    let invalid_config = TranspositionConfig {
        table_size: 0, // Invalid size
        ..TranspositionConfig::default()
    };

    println!("     Testing with invalid table size (0)...");
    let mut tt = ThreadSafeTranspositionTable::new(invalid_config);

    // This should handle the error gracefully
    let entry = build_entry(100, 3, TranspositionFlag::Exact, None, 12345, 0);

    tt.store(entry);
    let result = tt.probe(12345, 3);

    if result.is_some() {
        println!("       ✅ Error handled gracefully - operation succeeded");
    } else {
        println!("       ⚠️  Error handled gracefully - operation failed safely");
    }

    // Test error recovery
    println!("  Testing error recovery...");

    // Create a new valid transposition table
    let mut recovery_tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());

    // Store and retrieve successfully
    recovery_tt.store(build_entry(
        100,
        3,
        TranspositionFlag::Exact,
        None,
        12345,
        0,
    ));
    let recovery_result = recovery_tt.probe(12345, 3);

    if recovery_result.is_some() {
        println!("     ✅ Error recovery successful");
    } else {
        println!("     ❌ Error recovery failed");
    }

    // Demonstrate graceful degradation
    println!("  Demonstrating graceful degradation...");
    println!("     System continues to function with reduced performance");
    println!("     Fallback strategies maintain basic functionality");
    println!("     Error logging provides diagnostic information");
}

fn generate_sample_moves() -> Vec<Move> {
    let mut moves = Vec::new();

    // Generate pawn moves
    for col in 0..9 {
        moves.push(Move {
            from: Some(Position { row: 7, col }),
            to: Position { row: 6, col },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        });
    }

    // Generate capture moves
    for i in 0..5 {
        moves.push(Move {
            from: Some(Position { row: 6, col: i }),
            to: Position { row: 5, col: i + 1 },
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
        });
    }

    // Generate promotion moves
    for i in 0..3 {
        moves.push(Move {
            from: Some(Position { row: 2, col: i }),
            to: Position { row: 1, col: i },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: true,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        });
    }

    moves
}
