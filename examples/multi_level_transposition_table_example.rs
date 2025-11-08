//! Multi-Level Transposition Table Example
//!
//! This example demonstrates how to use the multi-level transposition table
//! system for improved cache efficiency and reduced hash collisions.

use shogi_engine::search::{
    MemoryAllocationStrategy, MultiLevelConfig, MultiLevelTranspositionTable,
};
use shogi_engine::types::{EntrySource, TranspositionEntry, TranspositionFlag};

fn build_entry(
    hash_key: u64,
    depth: u8,
    score: i32,
    flag: TranspositionFlag,
) -> TranspositionEntry {
    TranspositionEntry::new(
        score,
        depth,
        flag,
        None,
        hash_key,
        0,
        EntrySource::MainSearch,
    )
}

fn select_level(config: &MultiLevelConfig, depth: u8) -> usize {
    for (level, &threshold) in config.depth_thresholds.iter().enumerate() {
        if depth <= threshold {
            return level;
        }
    }
    config.levels.saturating_sub(1)
}

fn compute_level_info(config: &MultiLevelConfig, level: usize) -> (usize, u8, u8) {
    let size = match config.allocation_strategy {
        MemoryAllocationStrategy::Equal => config.base_size,
        MemoryAllocationStrategy::Proportional => {
            let multiplier = config.size_multiplier.powi(level as i32);
            ((config.base_size as f64) * multiplier) as usize
        }
        MemoryAllocationStrategy::Custom => {
            let priority_multiplier = (level + 1) as f64;
            ((config.base_size as f64) * priority_multiplier) as usize
        }
    }
    .max(config.min_level_size)
    .min(config.max_level_size);

    let min_depth = if level == 0 {
        0
    } else {
        config.depth_thresholds[level - 1] + 1
    };

    let max_depth = if level < config.depth_thresholds.len() {
        config.depth_thresholds[level]
    } else {
        u8::MAX
    };

    (size, min_depth, max_depth)
}

fn main() {
    println!("Multi-Level Transposition Table Example");
    println!("======================================");

    // Create a multi-level table with 4 levels
    let mut base_config = MultiLevelConfig::default();
    base_config.levels = 4;
    base_config.base_size = 1024;
    let mut table = MultiLevelTranspositionTable::with_config(base_config.clone());

    println!(
        "Created multi-level table with {} levels",
        base_config.levels
    );

    // Demonstrate basic operations
    demonstrate_basic_operations(&mut table);

    // Demonstrate level selection
    demonstrate_level_selection(&base_config);

    // Demonstrate cross-level search
    demonstrate_cross_level_search(&mut table, &base_config);

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
        let flag = match i % 3 {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            _ => TranspositionFlag::UpperBound,
        };
        let entry = build_entry(i as u64, i as u8, (i as i32 % 100) - 50, flag);

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

fn demonstrate_level_selection(config: &MultiLevelConfig) {
    println!("\n--- Level Selection Demo ---");

    println!("Depth thresholds: {:?}", config.depth_thresholds);

    for depth in 0..=10 {
        let level = select_level(config, depth);
        let (size, min_depth, max_depth) = compute_level_info(config, level);
        println!(
            "Depth {} -> Level {} (depth range: {}-{}, size: {})",
            depth, level, min_depth, max_depth, size
        );
    }
}

fn demonstrate_cross_level_search(
    table: &mut MultiLevelTranspositionTable,
    config: &MultiLevelConfig,
) {
    println!("\n--- Cross-Level Search Demo ---");

    // Store entries in different levels
    let entries = vec![
        (1000, 1, 100), // Level 0
        (2000, 4, 200), // Level 1
        (3000, 8, 300), // Level 2
    ];

    for (hash, depth, score) in entries {
        let entry = build_entry(hash, depth, score, TranspositionFlag::Exact);
        table.store(entry);
        println!(
            "Stored entry {} at depth {} in level {}",
            hash,
            depth,
            select_level(config, depth)
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

    let table = MultiLevelTranspositionTable::with_config(config.clone());

    println!("Custom configuration:");
    println!("  Levels: {}", config.levels);
    println!("  Base size: {}", config.base_size);
    println!("  Size multiplier: {}", config.size_multiplier);
    println!("  Depth thresholds: {:?}", config.depth_thresholds);
    println!("  Level policies enabled: {}", config.enable_level_policies);

    println!("\nLevel configurations:");
    for level in 0..config.levels {
        let (size, min_depth, max_depth) = compute_level_info(&config, level);
        println!(
            "  Level {}: size={}, depth_range={}-{}",
            level, size, min_depth, max_depth
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
    let equal_table = MultiLevelTranspositionTable::with_config(config.clone());

    println!("Equal Allocation Strategy:");
    for level in 0..config.levels {
        let (size, _, _) = compute_level_info(&config, level);
        println!("  Level {}: size={}", level, size);
    }

    // Proportional allocation
    let mut config = base_config.clone();
    config.allocation_strategy = MemoryAllocationStrategy::Proportional;
    let proportional_table = MultiLevelTranspositionTable::with_config(config.clone());

    println!("\nProportional Allocation Strategy:");
    for level in 0..config.levels {
        let (size, _, _) = compute_level_info(&config, level);
        println!("  Level {}: size={}", level, size);
    }

    // Custom allocation
    let mut config = base_config;
    config.allocation_strategy = MemoryAllocationStrategy::Custom;
    let custom_table = MultiLevelTranspositionTable::with_config(config.clone());

    println!("\nCustom Allocation Strategy:");
    for level in 0..config.levels {
        let (size, _, _) = compute_level_info(&config, level);
        println!("  Level {}: size={}", level, size);
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
