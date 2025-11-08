//! Performance tuning guide for the transposition table system
//!
//! This example demonstrates various performance tuning techniques and
//! configuration options for optimizing transposition table performance.

use shogi_engine::bitboards::*;
use shogi_engine::search::*;
use shogi_engine::types::*;
use std::time::Instant;

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
    println!("⚡ Transposition Table Performance Tuning Guide");
    println!("================================================");

    // 1. Configuration comparison
    println!("\n📋 Configuration Comparison");
    println!("---------------------------");

    let default_config = TranspositionConfig::default();
    let performance_config = TranspositionConfig::performance_optimized();
    let memory_config = TranspositionConfig::memory_optimized();

    println!("Default Configuration:");
    println!("  Table size: {}", default_config.table_size);
    println!(
        "  Replacement policy: {:?}",
        default_config.replacement_policy
    );
    println!("  Enable statistics: {}", default_config.enable_statistics);

    println!("\nPerformance Optimized Configuration:");
    println!("  Table size: {}", performance_config.table_size);
    println!(
        "  Replacement policy: {:?}",
        performance_config.replacement_policy
    );
    println!(
        "  Enable statistics: {}",
        performance_config.enable_statistics
    );

    println!("\nMemory Optimized Configuration:");
    println!("  Table size: {}", memory_config.table_size);
    println!(
        "  Replacement policy: {:?}",
        memory_config.replacement_policy
    );
    println!("  Enable statistics: {}", memory_config.enable_statistics);

    // 2. Performance benchmarking
    println!("\n🏃 Performance Benchmarking");
    println!("---------------------------");

    benchmark_transposition_table(&default_config, "Default");
    benchmark_transposition_table(&performance_config, "Performance Optimized");
    benchmark_transposition_table(&memory_config, "Memory Optimized");

    // 3. Hit rate analysis
    println!("\n🎯 Hit Rate Analysis");
    println!("-------------------");

    analyze_hit_rates();

    // 4. Memory usage analysis
    println!("\n💾 Memory Usage Analysis");
    println!("------------------------");

    analyze_memory_usage();

    // 5. Move ordering performance
    println!("\n🎯 Move Ordering Performance");
    println!("----------------------------");

    benchmark_move_ordering();

    // 6. Advanced statistics
    println!("\n📊 Advanced Statistics");
    println!("---------------------");

    demonstrate_advanced_statistics();

    println!("\n🎉 Performance tuning guide completed!");
    println!("\n📚 Performance Tuning Tips:");
    println!("   • Use larger table sizes for better hit rates");
    println!("   • Monitor hit rates and adjust configuration accordingly");
    println!("   • Consider memory constraints when choosing table size");
    println!("   • Use performance-optimized config for best speed");
    println!("   • Use memory-optimized config for limited memory");
    println!("   • Enable statistics for performance monitoring");
    println!("   • Consider replacement policies for your use case");
}

fn benchmark_transposition_table(config: &TranspositionConfig, name: &str) {
    println!("\nBenchmarking {} configuration...", name);

    let mut tt = ThreadSafeTranspositionTable::new(config.clone());
    let iterations = 10000;

    // Benchmark store operations
    let start = Instant::now();
    for i in 0..iterations {
        let entry = build_entry(
            (i % 1000) as i32,
            (i % 10) as u8,
            TranspositionFlag::Exact,
            None,
            i as u64,
            0,
        );
        tt.store(entry);
    }
    let store_time = start.elapsed();

    // Benchmark probe operations
    let start = Instant::now();
    let mut hits = 0;
    for i in 0..iterations {
        if tt.probe(i as u64, (i % 10) as u8).is_some() {
            hits += 1;
        }
    }
    let probe_time = start.elapsed();

    let stats = tt.get_stats();

    println!(
        "  Store operations: {:.2}μs/op",
        store_time.as_micros() as f64 / iterations as f64
    );
    println!(
        "  Probe operations: {:.2}μs/op",
        probe_time.as_micros() as f64 / iterations as f64
    );
    println!("  Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("  Configured table size: {}", config.table_size);
    println!("  Stores recorded: {}", stats.stores);
    println!("  Replacements recorded: {}", stats.replacements);
}

fn analyze_hit_rates() {
    let config = TranspositionConfig::performance_optimized();
    let mut tt = ThreadSafeTranspositionTable::new(config);

    // Simulate realistic search patterns
    let patterns = vec![
        (1000, "Shallow search (depth 1-3)"),
        (5000, "Medium search (depth 4-6)"),
        (10000, "Deep search (depth 7-9)"),
    ];

    for (iterations, description) in patterns {
        println!("\n{}:", description);

        // Store entries with varying depths
        for i in 0..iterations {
            let depth = (i % 9 + 1) as u8;
            let entry = build_entry(
                (i % 1000) as i32,
                depth,
                TranspositionFlag::Exact,
                None,
                i as u64,
                0,
            );
            tt.store(entry);
        }

        // Probe with same pattern
        let mut hits = 0;
        for i in 0..iterations {
            let depth = (i % 9 + 1) as u8;
            if tt.probe(i as u64, depth).is_some() {
                hits += 1;
            }
        }

        let hit_rate = hits as f64 / iterations as f64;
        println!("  Hit rate: {:.2}%", hit_rate * 100.0);
        println!("  Total entries stored: {}", iterations);
        println!("  Total entries found: {}", hits);
    }
}

fn analyze_memory_usage() {
    let configs = vec![
        (TranspositionConfig::memory_optimized(), "Memory Optimized"),
        (TranspositionConfig::default(), "Default"),
        (
            TranspositionConfig::performance_optimized(),
            "Performance Optimized",
        ),
    ];

    for (config, name) in configs {
        println!("\n{}:", name);
        let table_size = config.table_size;
        let estimated_memory_kb = table_size * 16 / 1024; // Assuming 16 bytes per entry
        let statistics_enabled = config.enable_statistics;
        let tt = ThreadSafeTranspositionTable::new(config);
        let stats = tt.get_stats();

        println!("  Configured table size: {}", table_size);
        println!("  Estimated memory: ~{} KB", estimated_memory_kb);
        println!("  Statistics enabled: {}", statistics_enabled);
        println!("  Thread mode: {:?}", stats.thread_mode);
    }
}

fn benchmark_move_ordering() {
    let mut orderer = TranspositionMoveOrderer::new();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    // Generate sample moves
    let mut sample_moves = Vec::new();
    for i in 0..20 {
        sample_moves.push(Move {
            from: Some(Position { row: 7, col: i % 9 }),
            to: Position { row: 6, col: i % 9 },
            piece_type: PieceType::Pawn,
            is_capture: i % 3 == 0,
            is_promotion: false,
            gives_check: false,
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

    // Benchmark move ordering
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ordered = orderer.order_moves(
            &sample_moves,
            &board,
            &captured,
            Player::Black,
            3,
            -1000,
            1000,
            None,
        );
    }

    let ordering_time = start.elapsed();

    println!(
        "  Move ordering: {:.2}μs/operation",
        ordering_time.as_micros() as f64 / iterations as f64
    );
    println!("  Moves per operation: {}", sample_moves.len());

    // Get ordering statistics
    let hints = orderer.get_move_ordering_hints(&board, &captured, Player::Black, 3);
    println!("  TT best move: {:?}", hints.best_move);
    let stats = orderer.get_stats();
    println!("  TT hint moves: {}", stats.tt_hint_moves);
    println!("  Killer move hits: {}", stats.killer_move_hits);
    println!("  History hits: {}", stats.history_hits);
}

fn demonstrate_advanced_statistics() {
    let config = TranspositionConfig {
        enable_statistics: true,
        ..TranspositionConfig::performance_optimized()
    };

    let mut tt = ThreadSafeTranspositionTable::new(config);

    // Perform various operations to generate statistics
    for i in 0..5000 {
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
    for i in 0..1000 {
        let _ = tt.probe(i as u64, (i % 10) as u8);
    }

    let stats = tt.get_stats();

    println!("  Total probes: {}", stats.total_probes);
    println!("  Total hits: {}", stats.hits);
    println!("  Total misses: {}", stats.misses);
    println!("  Total stores: {}", stats.stores);
    println!("  Replacements: {}", stats.replacements);
    println!("  Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("  Atomic operations: {}", stats.atomic_operations);
    println!("  Poison recoveries: {}", stats.poison_recoveries);
    println!("  Thread mode: {:?}", stats.thread_mode);
    if stats.stores > 0 {
        let replacement_ratio = stats.replacements as f64 / stats.stores as f64;
        println!("  Replacement ratio: {:.2}%", replacement_ratio * 100.0);
    }
}
