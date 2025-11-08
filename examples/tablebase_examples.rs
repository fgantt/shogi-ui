//! Tablebase System Examples
//!
//! This file demonstrates various ways to use the tablebase system,
//! from basic usage to advanced configuration and performance monitoring.

use shogi_engine::{
    tablebase::{MicroTablebase, TablebaseConfig, TablebaseStats},
    BitboardBoard, CapturedPieces, PieceType, Player, Position, ShogiEngine,
};

/// Basic tablebase usage example
fn basic_usage_example() {
    println!("=== Basic Usage Example ===");

    // Create tablebase with default configuration
    let mut tablebase = MicroTablebase::new();

    // Create a simple board position
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Probe for best move
    if let Some(result) = tablebase.probe(&board, Player::Black, &captured_pieces) {
        println!("Best move: {:?}", result.best_move);
        println!("Distance to mate: {}", result.distance_to_mate);
        println!("Outcome: {:?}", result.outcome);
        println!("Confidence: {}", result.confidence);
    } else {
        println!("No tablebase solution found");
    }

    // Get statistics
    let stats = tablebase.get_stats();
    println!("Statistics: {}", stats);
}

/// Configuration example
fn configuration_example() {
    println!("\n=== Configuration Example ===");

    // Create performance-optimized configuration
    let perf_config = TablebaseConfig::performance_optimized();
    let mut perf_tablebase = MicroTablebase::with_config(perf_config);

    // Create memory-optimized configuration
    let memory_config = TablebaseConfig::memory_optimized();
    let mut memory_tablebase = MicroTablebase::with_config(memory_config);

    // Test all configurations
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    println!("Performance-optimized tablebase:");
    if let Some(_) = perf_tablebase.probe(&board, Player::Black, &captured_pieces) {
        println!("  Found solution");
    }

    println!("Memory-optimized tablebase:");
    if let Some(_) = memory_tablebase.probe(&board, Player::Black, &captured_pieces) {
        println!("  Found solution");
    }
}

/// Custom configuration example
fn custom_configuration_example() {
    println!("\n=== Custom Configuration Example ===");

    // Create custom configuration
    let mut config = TablebaseConfig::default();
    config.enabled = true;
    config.cache_size = 5000;
    config.confidence_threshold = 0.9;
    config.max_depth = 30;

    // Configure individual solvers
    config.solvers.king_gold_vs_king.enabled = true;
    config.solvers.king_silver_vs_king.enabled = true;
    config.solvers.king_rook_vs_king.enabled = false; // Disable rook solver

    // Configure memory settings
    config.memory.enable_monitoring = true;
    config.memory.max_memory_bytes = 50 * 1024 * 1024; // 50MB
    config.memory.warning_threshold = 0.8;
    config.memory.critical_threshold = 0.95;
    config.memory.enable_auto_eviction = true;

    // Configure performance settings
    config.performance.eviction_strategy =
        shogi_engine::tablebase::tablebase_config::EvictionStrategy::LRU;
    config.performance.enable_adaptive_caching = true;
    config.performance.enable_profiling = true;

    let mut tablebase = MicroTablebase::with_config(config);

    // Test the configuration
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    if let Some(result) = tablebase.probe(&board, Player::Black, &captured_pieces) {
        println!("Custom configuration working: {:?}", result.outcome);
    }

    // Check configuration
    println!("Tablebase enabled: {}", tablebase.is_enabled());
    println!(
        "Memory monitoring: {}",
        tablebase.is_memory_monitoring_enabled()
    );
    println!("Profiling enabled: {}", tablebase.is_profiling_enabled());
}

/// Performance monitoring example
fn performance_monitoring_example() {
    println!("\n=== Performance Monitoring Example ===");

    let mut tablebase = MicroTablebase::new();

    // Enable profiling
    tablebase.set_profiling_enabled(true);

    // Perform some operations
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for i in 0..100 {
        let _ = tablebase.probe(&board, Player::Black, &captured_pieces);
        if i % 20 == 0 {
            println!("  Completed {} probes", i + 1);
        }
    }

    // Get performance summary
    let profiler = tablebase.get_profiler();
    let summary = profiler.get_summary();
    println!("Performance Summary:\n{}", summary);

    // Get most expensive operations
    let expensive_ops = profiler.get_most_expensive_operations(3);
    println!("Most expensive operations:");
    for (op, metrics) in expensive_ops {
        println!(
            "  {}: {} calls, avg: {:?}",
            op,
            metrics.call_count,
            metrics.average_duration()
        );
    }

    // Get most frequent operations
    let frequent_ops = profiler.get_most_frequent_operations(3);
    println!("Most frequent operations:");
    for (op, metrics) in frequent_ops {
        println!("  {}: {} calls", op, metrics.call_count);
    }
}

/// Memory management example
fn memory_management_example() {
    println!("\n=== Memory Management Example ===");

    let mut tablebase = MicroTablebase::new();

    // Check initial memory usage
    let initial_memory = tablebase.get_current_memory_usage();
    println!("Initial memory usage: {} bytes", initial_memory);

    // Perform operations to use memory
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for i in 0..50 {
        let _ = tablebase.probe(&board, Player::Black, &captured_pieces);
        if i % 10 == 0 {
            let current_memory = tablebase.get_current_memory_usage();
            let peak_memory = tablebase.get_peak_memory_usage();
            println!(
                "  After {} probes: current={} bytes, peak={} bytes",
                i + 1,
                current_memory,
                peak_memory
            );
        }
    }

    // Get memory summary
    let memory_summary = tablebase.get_memory_summary();
    println!("Memory Summary:\n{}", memory_summary);

    // Check if memory monitoring is enabled
    println!(
        "Memory monitoring enabled: {}",
        tablebase.is_memory_monitoring_enabled()
    );
}

/// Adaptive solver selection example
fn adaptive_solver_selection_example() {
    println!("\n=== Adaptive Solver Selection Example ===");

    let mut tablebase = MicroTablebase::new();

    // Test with different board positions
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    // Analyze position complexity
    let analysis = tablebase.analyze_position(&board, Player::Black, &captured_pieces);
    println!("Position complexity: {:?}", analysis.complexity);
    println!("Piece count: {}", analysis.piece_count);
    println!("Captured pieces: {}", analysis.captured_piece_count);
    println!("Is check: {}", analysis.is_check);
    println!("Is tactical: {}", analysis.is_tactical);
    println!(
        "Recommended solver priority: {}",
        analysis.recommended_solver_priority
    );

    // Check if position is suitable for different solver priorities
    for priority in [90, 80, 70, 50] {
        let suitable = tablebase.is_position_suitable_for_solver(
            &board,
            Player::Black,
            &captured_pieces,
            priority,
        );
        println!("Suitable for priority {}: {}", priority, suitable);
    }

    // Get analysis cache stats
    let cache_stats = tablebase.get_analysis_cache_stats();
    println!("Analysis cache stats: {}", cache_stats);
}

/// Engine integration example
fn engine_integration_example() {
    println!("\n=== Engine Integration Example ===");

    // Create engine with tablebase enabled
    let mut engine = ShogiEngine::new();
    engine.enable_tablebase();

    // Check if tablebase is enabled
    println!("Tablebase enabled: {}", engine.is_tablebase_enabled());

    // Get best move (tablebase will be consulted automatically)
    let best_move = engine.get_best_move(1, 1000, None, None);

    if let Some(mv) = best_move {
        println!("Best move found: {:?}", mv);
    } else {
        println!("No move found");
    }

    // Get tablebase statistics
    let stats = engine.get_tablebase_stats();
    println!("Tablebase stats: {}", stats);

    // Reset statistics
    engine.reset_tablebase_stats();
    println!("Statistics reset");

    // Disable tablebase
    engine.disable_tablebase();
    println!("Tablebase disabled: {}", !engine.is_tablebase_enabled());
}

/// Error handling example
fn error_handling_example() {
    println!("\n=== Error Handling Example ===");

    let mut tablebase = MicroTablebase::new();

    // Test with empty board (should not find solution)
    let empty_board = BitboardBoard::empty();
    let captured_pieces = CapturedPieces::new();

    match tablebase.probe(&empty_board, Player::Black, &captured_pieces) {
        Some(result) => println!("Unexpected solution found: {:?}", result),
        None => println!("No solution found (expected for empty board)"),
    }

    // Test with disabled tablebase
    tablebase.disable();
    match tablebase.probe(&empty_board, Player::Black, &captured_pieces) {
        Some(_) => println!("Unexpected solution found with disabled tablebase"),
        None => println!("No solution found with disabled tablebase (expected)"),
    }

    // Re-enable tablebase
    tablebase.enable();
    println!("Tablebase re-enabled: {}", tablebase.is_enabled());
}

/// Statistics and monitoring example
fn statistics_monitoring_example() {
    println!("\n=== Statistics and Monitoring Example ===");

    let mut tablebase = MicroTablebase::new();

    // Perform some operations
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    for i in 0..20 {
        let _ = tablebase.probe(&board, Player::Black, &captured_pieces);
        if i % 5 == 0 {
            let stats = tablebase.get_stats();
            println!("After {} probes: {}", i + 1, stats);
        }
    }

    // Get detailed statistics
    let stats = tablebase.get_stats();
    println!("Final statistics: {}", stats);

    // Reset statistics
    tablebase.reset_stats();
    let reset_stats = tablebase.get_stats();
    println!("After reset: {}", reset_stats);
}

/// Main function to run all examples
fn main() {
    println!("Tablebase System Examples");
    println!("========================");

    basic_usage_example();
    configuration_example();
    custom_configuration_example();
    performance_monitoring_example();
    memory_management_example();
    adaptive_solver_selection_example();
    engine_integration_example();
    error_handling_example();
    statistics_monitoring_example();

    println!("\nAll examples completed successfully!");
}
