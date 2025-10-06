//! WASM Transposition Table Example
//! 
//! This example demonstrates how to use the WASM-compatible transposition table
//! system in both native and WebAssembly environments. It shows the key features
//! and performance characteristics of the WASM-optimized implementation.

use shogi_engine::search::{
    WasmTranspositionTable, WasmTranspositionConfig, WasmTime, WasmDuration,
    WasmBenchmarkSuite, WasmPerformanceProfiler, TranspositionEntry, TranspositionFlag
};

fn main() {
    println!("WASM Transposition Table Example");
    println!("=================================");
    
    // Create WASM-optimized configuration
    let config = WasmTranspositionConfig {
        max_memory_mb: 32, // 32MB limit for WASM
        enable_memory_monitoring: true,
        memory_pressure_threshold: 0.8,
        enable_gc_hints: true,
        ..Default::default()
    };
    
    // Create WASM transposition table
    let mut table = WasmTranspositionTable::new(config.clone());
    
    println!("Created WASM transposition table with {} entries", table.size);
    
    // Demonstrate basic operations
    demonstrate_basic_operations(&mut table);
    
    // Demonstrate memory management
    demonstrate_memory_management(&mut table);
    
    // Demonstrate performance profiling
    demonstrate_performance_profiling(&mut table);
    
    // Run benchmarks
    run_benchmarks(config);
    
    // Demonstrate time handling
    demonstrate_time_handling();
    
    println!("\nWASM Transposition Table Example completed successfully!");
}

fn demonstrate_basic_operations(table: &mut WasmTranspositionTable) {
    println!("\n--- Basic Operations Demo ---");
    
    // Store some entries
    for i in 0..10 {
        let entry = TranspositionEntry {
            hash_key: i as u64,
            depth: (i % 5) as u8 + 1,
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
    }
    
    // Probe for entries
    for i in 0..10 {
        if let Some(found) = table.probe(i as u64, 1) {
            println!("Found entry {}: score={}, depth={}, flag={:?}", 
                i, found.score, found.depth, found.flag);
        } else {
            println!("Entry {} not found", i);
        }
    }
    
    // Show statistics
    let stats = table.get_stats();
    println!("Statistics: hits={}, misses={}, stores={}, hit_rate={:.2}%",
        stats.hits, stats.misses, stats.stores, stats.hit_rate * 100.0);
}

fn demonstrate_memory_management(table: &mut WasmTranspositionTable) {
    println!("\n--- Memory Management Demo ---");
    
    let initial_memory = table.get_memory_usage();
    println!("Initial memory usage: {} bytes", initial_memory);
    
    // Fill table to test memory pressure
    for i in 0..1000 {
        let entry = TranspositionEntry {
            hash_key: i as u64,
            depth: 1,
            score: 0,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        table.store(entry);
    }
    
    let after_fill_memory = table.get_memory_usage();
    println!("After filling: {} bytes", after_fill_memory);
    println!("Memory pressure: {}", table.is_memory_pressure());
    
    // Clear table
    table.clear();
    let after_clear_memory = table.get_memory_usage();
    println!("After clearing: {} bytes", after_clear_memory);
    
    // Hint garbage collection
    table.hint_gc();
    println!("Garbage collection hint sent");
}

fn demonstrate_performance_profiling(table: &mut WasmTranspositionTable) {
    println!("\n--- Performance Profiling Demo ---");
    
    let mut profiler = WasmPerformanceProfiler::new();
    
    // Profile store operations
    for i in 0..100 {
        let start = profiler.start_operation("store");
        let entry = TranspositionEntry {
            hash_key: i as u64,
            depth: 1,
            score: 0,
            flag: TranspositionFlag::Exact,
            best_move: None,
            age: 0,
        };
        table.store(entry);
        profiler.end_operation("store", start);
    }
    
    // Profile probe operations
    for i in 0..100 {
        let start = profiler.start_operation("probe");
        table.probe(i as u64, 1);
        profiler.end_operation("probe", start);
    }
    
    // Show profiling results
    let stats = profiler.get_all_stats();
    for stat in stats {
        println!("Operation '{}': {} calls, {:.2} ops/sec, {:.2}μs avg",
            stat.name, stat.count, stat.ops_per_second, 
            stat.avg_time_per_op.as_millis() as f64 * 1000.0);
    }
    
    println!("Total profiling time: {}ms", profiler.total_time().as_millis());
}

fn run_benchmarks(config: WasmTranspositionConfig) {
    println!("\n--- Benchmarking Demo ---");
    
    let mut benchmark_suite = WasmBenchmarkSuite::new(config);
    let results = benchmark_suite.run_all();
    
    let summary = benchmark_suite.get_summary();
    println!("Benchmark Summary:");
    println!("  Total benchmarks: {}", summary.total_benchmarks);
    println!("  Total operations: {}", summary.total_operations);
    println!("  Total time: {}ms", summary.total_time_ms);
    println!("  Average ops/sec: {:.2}", summary.avg_ops_per_second);
    println!("  Total memory usage: {} bytes", summary.total_memory_usage);
    println!("  Total binary size impact: {}KB", summary.total_binary_size_kb);
    
    println!("\nIndividual Results:");
    for result in &results {
        println!("  {}: {} ops/sec, {:.2}μs avg, {} bytes memory",
            result.name, result.ops_per_second, result.avg_time_per_op_us,
            result.memory_usage);
    }
}

fn demonstrate_time_handling() {
    println!("\n--- Time Handling Demo ---");
    
    let start = WasmTime::now();
    println!("Start time: {}ms", start.as_millis());
    
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let end = WasmTime::now();
    let elapsed = start.elapsed();
    
    println!("End time: {}ms", end.as_millis());
    println!("Elapsed time: {}ms", elapsed.as_millis());
    
    // Test duration operations
    let duration1 = WasmDuration::from_millis(100);
    let duration2 = WasmDuration::from_millis(200);
    
    println!("Duration 1: {}ms", duration1.as_millis());
    println!("Duration 2: {}ms", duration2.as_millis());
    println!("Duration 1 + Duration 2: {}ms", 
        WasmDuration::from_millis(duration1.as_millis() + duration2.as_millis()).as_millis());
    
    // Test time conversion
    let time_from_millis = WasmTime::from_millis(12345);
    println!("Time from 12345ms: {}ms", time_from_millis.as_millis());
}
