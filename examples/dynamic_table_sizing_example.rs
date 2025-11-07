//! Dynamic Table Sizing Example
//!
//! This example demonstrates how to use dynamic table sizing to automatically
//! adjust transposition table size based on memory usage, performance metrics,
//! and system conditions.

use shogi_engine::search::{
    AccessPatternAnalysis, DynamicSizingConfig, DynamicTableSizer, MemoryStats, PerformanceStats,
    ResizeDecision, ResizeReason,
};

fn main() {
    println!("Dynamic Table Sizing Example");
    println!("============================");

    // Demonstrate different sizing strategies
    demonstrate_sizing_strategies();

    // Demonstrate memory pressure handling
    demonstrate_memory_pressure_handling();

    // Demonstrate performance-based resizing
    demonstrate_performance_based_resizing();

    // Demonstrate access pattern analysis
    demonstrate_access_pattern_analysis();

    // Demonstrate real-time adaptation
    demonstrate_realtime_adaptation();

    println!("\nDynamic Table Sizing Example completed successfully!");
}

fn demonstrate_sizing_strategies() {
    println!("\n--- Sizing Strategies Demo ---");

    let strategies = [
        ("Conservative", DynamicSizingConfig::conservative()),
        ("Aggressive", DynamicSizingConfig::aggressive()),
        ("Memory-Based", DynamicSizingConfig::memory_based()),
        (
            "Performance-Based",
            DynamicSizingConfig::performance_based(),
        ),
    ];

    for (name, config) in strategies {
        let mut sizer = DynamicTableSizer::new(config);

        println!("{} Strategy:", name);
        println!("  Initial size: {}", sizer.get_current_size());
        println!("  Min size: {}", sizer.config.min_table_size);
        println!("  Max size: {}", sizer.config.max_table_size);
        println!("  Resize frequency: {:?}", sizer.config.resize_frequency);
        println!(
            "  Aggressive resizing: {}",
            sizer.config.aggressive_resizing
        );

        // Simulate some activity
        simulate_activity(&mut sizer, 10);

        let stats = sizer.get_stats();
        println!(
            "  After simulation: {} resizes performed",
            stats.resize_count
        );
    }
}

fn demonstrate_memory_pressure_handling() {
    println!("\n--- Memory Pressure Handling Demo ---");

    let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::memory_based());

    println!("Simulating memory pressure scenarios...");

    // Scenario 1: Gradual memory increase
    println!("Scenario 1: Gradual memory increase");
    let initial_size = sizer.get_current_size();

    for i in 0..5 {
        let memory_usage = 1000000 + (i * 2000000); // Increasing memory usage
        sizer.record_memory_usage(memory_usage);

        // Force time to pass for resize check
        sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

        if let Some(decision) = sizer.should_resize() {
            println!(
                "  Memory usage: {} -> Resize decision: {:?} (size: {})",
                memory_usage, decision.reason, decision.new_size
            );
            sizer.apply_resize(&decision);
        }
    }

    // Scenario 2: Memory pressure relief
    println!("Scenario 2: Memory pressure relief");
    sizer.record_memory_usage(500000); // Lower memory usage
    sizer.memory_stats.pressure_level = 0.3; // Low pressure

    sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

    if let Some(decision) = sizer.should_resize() {
        println!(
            "  Low memory pressure -> Resize decision: {:?} (size: {})",
            decision.reason, decision.new_size
        );
        sizer.apply_resize(&decision);
    } else {
        println!("  No resize needed under low memory pressure");
    }

    let final_size = sizer.get_current_size();
    println!(
        "Size change: {} -> {} ({:.1}% change)",
        initial_size,
        final_size,
        ((final_size as f64 / initial_size as f64) - 1.0) * 100.0
    );
}

fn demonstrate_performance_based_resizing() {
    println!("\n--- Performance-Based Resizing Demo ---");

    let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::performance_based());

    println!("Simulating performance scenarios...");

    // Scenario 1: Low hit rate
    println!("Scenario 1: Low hit rate");
    let initial_size = sizer.get_current_size();

    sizer.record_performance(0.3, 50.0); // Low hit rate
    sizer.performance_stats.avg_hit_rate = 0.35;

    sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

    if let Some(decision) = sizer.should_resize() {
        println!(
            "  Hit rate: {:.1}% -> Resize decision: {:?} (size: {})",
            sizer.performance_stats.hit_rate * 100.0,
            decision.reason,
            decision.new_size
        );
        sizer.apply_resize(&decision);
    }

    // Scenario 2: High hit rate
    println!("Scenario 2: High hit rate");
    sizer.record_performance(0.85, 120.0); // High hit rate
    sizer.performance_stats.avg_hit_rate = 0.80;

    sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

    if let Some(decision) = sizer.should_resize() {
        println!(
            "  Hit rate: {:.1}% -> Resize decision: {:?} (size: {})",
            sizer.performance_stats.hit_rate * 100.0,
            decision.reason,
            decision.new_size
        );
        sizer.apply_resize(&decision);
    }

    // Scenario 3: Performance degradation
    println!("Scenario 3: Performance degradation");
    sizer.performance_stats.trend = shogi_engine::search::PerformanceTrend::Degrading;
    sizer.record_performance(0.4, 30.0); // Degrading performance

    sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

    if let Some(decision) = sizer.should_resize() {
        println!(
            "  Performance degrading -> Resize decision: {:?} (size: {})",
            decision.reason, decision.new_size
        );
        sizer.apply_resize(&decision);
    }

    let final_size = sizer.get_current_size();
    println!(
        "Size change: {} -> {} ({:.1}% change)",
        initial_size,
        final_size,
        ((final_size as f64 / initial_size as f64) - 1.0) * 100.0
    );
}

fn demonstrate_access_pattern_analysis() {
    println!("\n--- Access Pattern Analysis Demo ---");

    let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::performance_based());

    println!("Simulating different access patterns...");

    // Pattern 1: High locality (repeated accesses)
    println!("Pattern 1: High locality");
    for _ in 0..20 {
        sizer.record_access(0x1000, 5); // Same hash, same depth
        sizer.record_access(0x2000, 3);
        sizer.record_access(0x1000, 5); // Repeat
        sizer.record_access(0x3000, 4);
        sizer.record_access(0x2000, 3); // Repeat
    }

    sizer.update_access_analysis();
    println!(
        "  Access locality: {:.2}",
        sizer.access_analysis.access_locality
    );
    println!(
        "  Hot spot concentration: {:.2}",
        sizer.access_analysis.hot_spot_concentration
    );
    println!("  Entropy: {:.2}", sizer.access_analysis.entropy);

    // Pattern 2: Low locality (random accesses)
    println!("Pattern 2: Low locality");
    sizer.access_history.clear();

    for i in 0..100 {
        sizer.record_access(0x1000 + (i * 0x100), 3 + (i % 5)); // Different hashes
    }

    sizer.update_access_analysis();
    println!(
        "  Access locality: {:.2}",
        sizer.access_analysis.access_locality
    );
    println!(
        "  Hot spot concentration: {:.2}",
        sizer.access_analysis.hot_spot_concentration
    );
    println!("  Entropy: {:.2}", sizer.access_analysis.entropy);

    // Test resize decision based on access patterns
    sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

    if let Some(decision) = sizer.should_resize() {
        println!(
            "  Access pattern -> Resize decision: {:?} (size: {})",
            decision.reason, decision.new_size
        );
    }
}

fn demonstrate_realtime_adaptation() {
    println!("\n--- Real-time Adaptation Demo ---");

    let mut sizer = DynamicTableSizer::new(DynamicSizingConfig::aggressive());

    println!("Simulating real-time adaptation over time...");

    let mut resize_count = 0;
    let mut total_size_changes = 0;

    // Simulate 10 time periods with varying conditions
    for period in 0..10 {
        println!("Period {}:", period + 1);

        // Vary conditions over time
        let memory_usage = 1000000 + (period * 500000);
        let hit_rate = 0.3 + (period as f64 * 0.05); // Gradually improving
        let access_frequency = 50.0 + (period as f64 * 10.0);

        sizer.record_memory_usage(memory_usage);
        sizer.record_performance(hit_rate, access_frequency);

        // Record some access patterns
        for i in 0..20 {
            sizer.record_access(0x1000 + (i % 10) * 0x100, 3 + (i % 3));
        }

        // Force time to pass
        sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

        let size_before = sizer.get_current_size();

        if let Some(decision) = sizer.should_resize() {
            sizer.apply_resize(&decision);
            resize_count += 1;

            let size_change = (decision.new_size as f64 / size_before as f64 - 1.0) * 100.0;
            total_size_changes += size_change.abs() as i32;

            println!(
                "  Memory: {}, Hit rate: {:.1}%, Size: {} -> {} ({:+.1}%)",
                memory_usage,
                hit_rate * 100.0,
                size_before,
                decision.new_size,
                size_change
            );
            println!(
                "  Reason: {:?}, Confidence: {:.2}",
                decision.reason, decision.confidence
            );
        } else {
            println!(
                "  Memory: {}, Hit rate: {:.1}%, Size: {} (no change)",
                memory_usage,
                hit_rate * 100.0,
                size_before
            );
        }
    }

    let stats = sizer.get_stats();
    println!("Adaptation summary:");
    println!("  Total resizes: {}", resize_count);
    println!(
        "  Average size change: {:.1}%",
        total_size_changes as f64 / resize_count.max(1) as f64
    );
    println!("  Final size: {}", stats.current_size);
    println!(
        "  Memory pressure: {:.2}",
        stats.memory_stats.pressure_level
    );
    println!("  Performance trend: {:?}", stats.performance_stats.trend);
}

fn simulate_activity(sizer: &mut DynamicTableSizer, iterations: usize) {
    for i in 0..iterations {
        // Simulate memory usage
        let memory_usage = 1000000 + (i * 100000);
        sizer.record_memory_usage(memory_usage);

        // Simulate performance
        let hit_rate = 0.5 + (i as f64 * 0.02);
        let access_frequency = 50.0 + (i as f64 * 5.0);
        sizer.record_performance(hit_rate, access_frequency);

        // Simulate access patterns
        for j in 0..10 {
            sizer.record_access(0x1000 + (j * 0x100), 3 + (j % 3));
        }

        // Force time to pass for resize checks
        sizer.last_resize = std::time::Instant::now() - std::time::Duration::from_secs(70);

        // Check for resize
        if let Some(decision) = sizer.should_resize() {
            sizer.apply_resize(&decision);
        }
    }
}
