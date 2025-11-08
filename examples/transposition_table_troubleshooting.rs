//! Troubleshooting guide for the transposition table system
//!
//! This example demonstrates common issues and their solutions when working
//! with the transposition table system.

use shogi_engine::bitboards::*;
use shogi_engine::search::*;
use shogi_engine::types::*;

fn make_entry(hash_key: u64, depth: u8, score: i32) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        TranspositionFlag::Exact,
        None,
        hash_key,
        0,
        EntrySource::MainSearch,
    )
}

fn main() {
    println!("🔧 Transposition Table Troubleshooting Guide");
    println!("==============================================");

    // 1. Low hit rate issues
    println!("\n📉 Low Hit Rate Issues");
    println!("----------------------");
    demonstrate_low_hit_rate_issues();

    // 2. Memory issues
    println!("\n💾 Memory Issues");
    println!("----------------");
    demonstrate_memory_issues();

    // 3. Performance issues
    println!("\n⚡ Performance Issues");
    println!("---------------------");
    demonstrate_performance_issues();

    // 4. Hash collision issues
    println!("\n🔀 Hash Collision Issues");
    println!("------------------------");
    demonstrate_hash_collision_issues();

    // 5. Move ordering issues
    println!("\n🎯 Move Ordering Issues");
    println!("-----------------------");
    demonstrate_move_ordering_issues();

    // 6. WASM compatibility issues
    println!("\n🌐 WASM Compatibility Issues");
    println!("-----------------------------");

    // 7. Configuration validation
    println!("\n✅ Configuration Validation");
    println!("---------------------------");
    demonstrate_configuration_validation();

    println!("\n🎉 Troubleshooting guide completed!");
    println!("\n📚 Common Solutions Summary:");
    println!("   • Low hit rates: Increase table size, check hash function");
    println!("   • Memory issues: Use memory-optimized config, reduce table size");
    println!("   • Performance issues: Use performance-optimized config, enable statistics");
    println!("   • Hash collisions: Check position representation, verify hash consistency");
    println!("   • Move ordering: Update killer moves, check TT integration");
    println!("   • WASM issues: Avoid time-based operations, use conditional compilation");
    println!("   • Configuration: Validate parameters, test different replacement policies");
}

fn demonstrate_low_hit_rate_issues() {
    println!("Problem: Transposition table has low hit rate (< 20%)");

    // Simulate low hit rate scenario
    let mut tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());

    // Store entries with random patterns (simulating poor hash distribution)
    for i in 0..1000 {
        let entry = make_entry((i * 1_000_000) as u64, 1, i as i32);
        tt.store(entry);
    }

    // Probe with different pattern
    let mut hits = 0;
    for i in 0..1000 {
        if tt.probe((i * 1000001) as u64, 1).is_some() {
            // Different pattern
            hits += 1;
        }
    }

    let stats = tt.get_stats();
    println!("  Current hit rate: {:.2}%", stats.hit_rate * 100.0);

    // Solutions
    println!("  Solutions:");
    println!("    1. Increase table size for better coverage");
    println!("    2. Check hash function consistency");
    println!("    3. Verify position representation");
    println!("    4. Use better replacement policies");

    // Demonstrate solution: larger table
    let large_config = TranspositionConfig {
        table_size: 65536, // 4x larger
        ..TranspositionConfig::default()
    };
    let mut large_tt = ThreadSafeTranspositionTable::new(large_config);

    // Store with better distribution
    for i in 0..1000 {
        let entry = make_entry(i as u64, 1, i as i32);
        large_tt.store(entry);
    }

    // Probe with same pattern
    let mut large_hits = 0;
    for i in 0..1000 {
        if large_tt.probe(i as u64, 1).is_some() {
            large_hits += 1;
        }
    }

    let large_stats = large_tt.get_stats();
    println!("  Improved hit rate: {:.2}%", large_stats.hit_rate * 100.0);
}

fn demonstrate_memory_issues() {
    println!("Problem: High memory usage or memory allocation failures");

    // Simulate memory pressure
    let large_config = TranspositionConfig {
        table_size: 1048576, // 1M entries
        ..TranspositionConfig::default()
    };

    println!("  Large table configuration:");
    println!("    Table size: {}", large_config.table_size);
    println!(
        "    Estimated memory: ~{} MB",
        large_config.table_size * 16 / (1024 * 1024)
    );

    // Solutions
    println!("  Solutions:");
    println!("    1. Use memory-optimized configuration");
    println!("    2. Reduce table size based on available memory");
    println!("    3. Monitor memory usage with statistics");
    println!("    4. Consider memory constraints in WASM");

    // Demonstrate memory-optimized solution
    let memory_config = TranspositionConfig::memory_optimized();
    println!("  Memory-optimized configuration:");
    println!("    Table size: {}", memory_config.table_size);
    println!(
        "    Estimated memory: ~{} MB",
        memory_config.table_size * 16 / (1024 * 1024)
    );
}

fn demonstrate_performance_issues() {
    println!("Problem: Slow transposition table operations");

    // Simulate performance measurement
    let config = TranspositionConfig::default();
    let mut tt = ThreadSafeTranspositionTable::new(config);

    let iterations = 1000;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let entry = make_entry(i as u64, 1, i as i32);
        tt.store(entry);
    }

    let duration = start.elapsed();
    let avg_time_us = duration.as_micros() as f64 / iterations as f64;

    println!("  Current performance: {:.2}μs/operation", avg_time_us);

    // Solutions
    println!("  Solutions:");
    println!("    1. Use performance-optimized configuration");
    println!("    2. Enable cache line alignment");
    println!("    3. Use appropriate replacement policies");
    println!("    4. Monitor statistics for bottlenecks");

    // Demonstrate performance-optimized solution
    let perf_config = TranspositionConfig::performance_optimized();
    let mut perf_tt = ThreadSafeTranspositionTable::new(perf_config);

    let perf_start = std::time::Instant::now();

    for i in 0..iterations {
        let entry = make_entry(i as u64, 1, i as i32);
        perf_tt.store(entry);
    }

    let perf_duration = perf_start.elapsed();
    let perf_avg_time_us = perf_duration.as_micros() as f64 / iterations as f64;

    println!(
        "  Improved performance: {:.2}μs/operation",
        perf_avg_time_us
    );
    println!(
        "  Performance improvement: {:.1}%",
        ((avg_time_us - perf_avg_time_us) / avg_time_us) * 100.0
    );
}

fn demonstrate_hash_collision_issues() {
    println!("Problem: High collision rate (> 10%)");

    // Simulate hash collisions
    let mut tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());

    // Force collisions by using same hash keys
    for i in 0..100 {
        let entry = make_entry(i as u64 % 10, 1, i as i32);
        tt.store(entry);
    }

    let stats = tt.get_stats();
    println!("  Stores: {}", stats.stores);
    println!("  Replacements (collisions): {}", stats.replacements);
    if stats.stores > 0 {
        let collision_ratio = stats.replacements as f64 / stats.stores as f64;
        println!("  Collision ratio: {:.2}%", collision_ratio * 100.0);
    }

    // Solutions
    println!("  Solutions:");
    println!("    1. Check hash function implementation");
    println!("    2. Verify position representation consistency");
    println!("    3. Increase table size to reduce collision probability");
    println!("    4. Use better hash mixing techniques");

    // Demonstrate hash function validation
    let hash_calc = ShogiHashHandler::new(1000);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    let hash1 = hash_calc.get_position_hash(&board, Player::Black, &captured);
    let hash2 = hash_calc.get_position_hash(&board, Player::Black, &captured);

    println!("  Hash consistency check:");
    println!("    Hash 1: 0x{:X}", hash1);
    println!("    Hash 2: 0x{:X}", hash2);
    println!("    Consistent: {}", hash1 == hash2);
}

fn demonstrate_move_ordering_issues() {
    println!("Problem: Move ordering not improving search performance");

    let mut orderer = TranspositionMoveOrderer::new();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    // Create moves without proper ordering
    let mut moves = Vec::new();
    for i in 0..10 {
        moves.push(Move {
            from: Some(Position { row: 7, col: i }),
            to: Position { row: 6, col: i },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        });
    }

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

    println!("  Move ordering results:");
    println!("    Original moves: {}", moves.len());
    println!("    Ordered moves: {}", ordered_moves.len());

    let hints = orderer.get_move_ordering_hints(&board, &captured, Player::Black, 3);
    println!("    TT best move: {:?}", hints.best_move);
    println!("    TT depth: {}", hints.tt_depth);
    if let Some(score) = hints.tt_score {
        println!("    TT score: {}", score);
    }

    let stats = orderer.get_stats();
    println!("    TT hint moves: {}", stats.tt_hint_moves);
    println!("    Killer move hits: {}", stats.killer_move_hits);
    println!("    History hits: {}", stats.history_hits);

    // Solutions
    println!("  Solutions:");
    println!("    1. Set transposition table reference in move orderer");
    println!("    2. Update killer moves after beta cutoffs");
    println!("    3. Update history scores for quiet moves");
    println!("    4. Monitor move ordering statistics");
    println!("    5. Ensure proper TT integration");

    // Demonstrate proper setup
    let mut tt = ThreadSafeTranspositionTable::new(TranspositionConfig::default());
    let hash_calc = ShogiHashHandler::new(1000);
    let position_hash = hash_calc.get_position_hash(&board, Player::Black, &captured);
    let tt_entry = TranspositionEntry::new(
        320,
        4,
        TranspositionFlag::Exact,
        moves.first().cloned(),
        position_hash,
        0,
        EntrySource::MainSearch,
    );
    tt.store(tt_entry);

    orderer.set_transposition_table(&tt);

    println!("  After proper setup:");
    orderer.reset_stats();
    let _ordered_with_tt = orderer.order_moves(
        &moves,
        &board,
        &captured,
        Player::Black,
        3,
        -1000,
        1000,
        None,
    );
    let hints_after = orderer.get_move_ordering_hints(&board, &captured, Player::Black, 3);
    println!("    TT best move: {:?}", hints_after.best_move);
    println!("    TT depth: {}", hints_after.tt_depth);
    if let Some(score) = hints_after.tt_score {
        println!("    TT score: {}", score);
    }
    let stats_after = orderer.get_stats();
    println!("    TT hint moves: {}", stats_after.tt_hint_moves);
    println!("    Killer move hits: {}", stats_after.killer_move_hits);
    println!("    History hits: {}", stats_after.history_hits);
}

fn demonstrate_configuration_validation() {
    println!("Problem: Invalid or suboptimal configuration");

    // Test various configurations
    let configs = vec![
        ("Default", TranspositionConfig::default()),
        ("Performance", TranspositionConfig::performance_optimized()),
        ("Memory", TranspositionConfig::memory_optimized()),
    ];

    for (name, config) in configs {
        println!("  {} Configuration:", name);
        println!("    Table size: {}", config.table_size);
        println!("    Replacement policy: {:?}", config.replacement_policy);
        println!("    Enable statistics: {}", config.enable_statistics);

        // Validate configuration
        let mut issues = Vec::new();

        if config.table_size == 0 {
            issues.push("Table size cannot be zero");
        }

        if config.table_size > 1048576 {
            issues.push("Table size may be too large for memory constraints");
        }

        if !config.enable_statistics && config.table_size > 65536 {
            issues.push("Consider enabling statistics for large tables");
        }

        if issues.is_empty() {
            println!("    ✅ Configuration is valid");
        } else {
            println!("    ⚠️  Configuration issues:");
            for issue in issues {
                println!("      • {}", issue);
            }
        }
    }

    // Solutions
    println!("  Solutions:");
    println!("    1. Validate all configuration parameters");
    println!("    2. Test configurations with your specific use case");
    println!("    3. Monitor performance with different settings");
    println!("    4. Use predefined configurations as starting points");
    println!("    5. Consider memory and performance trade-offs");
}
