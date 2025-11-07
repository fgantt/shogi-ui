//! Multi-Level Transposition Table Example
//!
//! This example demonstrates how to use the multi-level transposition table
//! system for improved cache efficiency and reduced hash collisions.

use shogi_engine::search::{
    MemoryAllocationStrategy, MultiLevelConfig, MultiLevelTranspositionTable, TranspositionEntry,
    TranspositionFlag,
};

fn main() {
    println!("Multi-Level Transposition Table Example");
    println!("======================================");

    // Create a multi-level table with 4 levels
    let mut table = MultiLevelTranspositionTable::new(4, 1024);

    println!(
        "Created multi-level table with {} levels",
        table.config.levels
    );

    // Demonstrate basic operations
    demonstrate_basic_operations(&mut table);

    // Demonstrate level selection
    demonstrate_level_selection(&table);

    // Demonstrate cross-level search
    demonstrate_cross_level_search(&mut table);

    // Demonstrate statistics
    demonstrate_statistics(&table);

    // Demonstrate custom configuration
    demonstrate_custom_configuration();

    // Demonstrate memory allocation strategies
    demonstrate_memory_allocation_strategies();

    println!("\nMulti-Level Transposition Table Example completed successfully!");
}

fn demonstrate_basic_operations(table: &mut MultiLevelTranspositionTable) {
    println!("\n--- Basic Operations Demo ---");

    // Store entries at different depths
    for i in 0..10 {
        let entry = TranspositionEntry {
            hash_key: i as u64,
            depth: i as u8,
            score: (i as i32 % 100) - 50,
            flag: match i % 3 {
                0 => TranspositionFlag::Exact,
                1 => TranspositionFlag::LowerBound,
                _ => TranspositionFlag::UpperBound,
            },
            best_move: None,
            age: 0,
        };

        table.store(entry);
        println!("Stored entry {} at depth {}", i, i);
    }

    // Probe for entries
    for i in 0..10 {
        if let Some(found) = table.probe(i as u64, i as u8) {
            println!(
                "Found entry {}: score={}, depth={}, flag={:?}",
                i, found.score, found.depth, found.flag
            );
        } else {
            println!("Entry {} not found", i);
        }
    }
}

fn demonstrate_level_selection(table: &MultiLevelTranspositionTable) {
    println!("\n--- Level Selection Demo ---");

    println!("Depth thresholds: {:?}", table.config.depth_thresholds);

    for depth in 0..=10 {
        let level = table.get_level_for_depth(depth);
        let level_config = &table.level_configs[level];
        println!(
            "Depth {} -> Level {} (depth range: {}-{}, size: {})",
            depth, level, level_config.min_depth, level_config.max_depth, level_config.size
        );
    }
}

fn demonstrate_cross_level_search(table: &mut MultiLevelTranspositionTable) {
    println!("\n--- Cross-Level Search Demo ---");

    // Store entries in different levels
    let entries = vec![
        (1000, 1, 100), // Level 0
        (2000, 4, 200), // Level 1
        (3000, 8, 300), // Level 2
    ];

    for (hash, depth, score) in entries {
        let entry = TranspositionEntry {
            hash_key: hash,
            depth,
            score,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        table.store(entry);
        println!(
            "Stored entry {} at depth {} in level {}",
            hash,
            depth,
            table.get_level_for_depth(depth)
        );
    }

    // Search for entries from different starting depths
    for (hash, original_depth, expected_score) in entries {
        // Search from a different depth than where it was stored
        let search_depth = original_depth + 2;
        if let Some(found) = table.probe(hash, search_depth) {
            println!("Cross-level search: found entry {} (stored at depth {}, searched at depth {}): score={}", 
                hash, original_depth, search_depth, found.score);
            assert_eq!(found.score, expected_score);
        } else {
            println!("Cross-level search failed for entry {}", hash);
        }
    }
}

fn demonstrate_statistics(table: &MultiLevelTranspositionTable) {
    println!("\n--- Statistics Demo ---");

    let stats = table.get_stats();
    println!("Overall Statistics:");
    println!("  Total hits: {}", stats.total_hits);
    println!("  Total misses: {}", stats.total_misses);
    println!("  Total stores: {}", stats.total_stores);
    println!("  Total replacements: {}", stats.total_replacements);
    println!("  Cross-level hits: {}", stats.cross_level_hits);
    println!("  Total memory usage: {} bytes", stats.total_memory_usage);

    println!("\nPer-Level Statistics:");
    for (level, level_stats) in stats.level_stats.iter().enumerate() {
        println!(
            "  Level {}: hits={}, misses={}, stores={}, hit_rate={:.2}%, memory={} bytes",
            level,
            level_stats.hits,
            level_stats.misses,
            level_stats.stores,
            level_stats.hit_rate * 100.0,
            level_stats.memory_usage
        );
    }

    println!("\nMemory Usage per Level:");
    for (level, memory) in stats.level_memory_usage.iter().enumerate() {
        println!("  Level {}: {} bytes", level, memory);
    }
}

fn demonstrate_custom_configuration() {
    println!("\n--- Custom Configuration Demo ---");

    let config = MultiLevelConfig {
        levels: 5,
        base_size: 2048,
        size_multiplier: 2.0,
        min_level_size: 512,
        max_level_size: 16384,
        depth_thresholds: vec![1, 3, 7, 15], // More granular depth separation
        enable_level_policies: true,
        allocation_strategy: MemoryAllocationStrategy::Custom,
    };

    let table = MultiLevelTranspositionTable::with_config(config);

    println!("Custom configuration:");
    println!("  Levels: {}", table.config.levels);
    println!("  Base size: {}", table.config.base_size);
    println!("  Size multiplier: {}", table.config.size_multiplier);
    println!("  Depth thresholds: {:?}", table.config.depth_thresholds);

    println!("\nLevel configurations:");
    for (level, level_config) in table.level_configs.iter().enumerate() {
        println!(
            "  Level {}: size={}, depth_range={}-{}, policy={:?}",
            level,
            level_config.size,
            level_config.min_depth,
            level_config.max_depth,
            level_config.replacement_policy
        );
    }
}

fn demonstrate_memory_allocation_strategies() {
    println!("\n--- Memory Allocation Strategies Demo ---");

    let base_config = MultiLevelConfig {
        levels: 3,
        base_size: 1000,
        size_multiplier: 1.5,
        ..Default::default()
    };

    // Equal allocation
    let mut config = base_config.clone();
    config.allocation_strategy = MemoryAllocationStrategy::Equal;
    let equal_table = MultiLevelTranspositionTable::with_config(config);

    println!("Equal Allocation Strategy:");
    for (level, level_config) in equal_table.level_configs.iter().enumerate() {
        println!("  Level {}: size={}", level, level_config.size);
    }

    // Proportional allocation
    let mut config = base_config.clone();
    config.allocation_strategy = MemoryAllocationStrategy::Proportional;
    let proportional_table = MultiLevelTranspositionTable::with_config(config);

    println!("\nProportional Allocation Strategy:");
    for (level, level_config) in proportional_table.level_configs.iter().enumerate() {
        println!("  Level {}: size={}", level, level_config.size);
    }

    // Custom allocation
    let mut config = base_config;
    config.allocation_strategy = MemoryAllocationStrategy::Custom;
    let custom_table = MultiLevelTranspositionTable::with_config(config);

    println!("\nCustom Allocation Strategy:");
    for (level, level_config) in custom_table.level_configs.iter().enumerate() {
        println!("  Level {}: size={}", level, level_config.size);
    }

    // Compare total memory usage
    println!("\nMemory Usage Comparison:");
    println!("  Equal: {} bytes", equal_table.get_total_memory_usage());
    println!(
        "  Proportional: {} bytes",
        proportional_table.get_total_memory_usage()
    );
    println!("  Custom: {} bytes", custom_table.get_total_memory_usage());
}
