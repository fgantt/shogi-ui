# Endgame Tablebase System

## Overview

The Endgame Tablebase System provides perfect play capabilities for low-piece-count endgames in Shogi. This system uses embedded micro-tablebases to solve specific endgame patterns efficiently, providing optimal moves and exact distance-to-mate information.

## Features

- **Multiple Endgame Solvers**: King+Gold vs King, King+Silver vs King, King+Rook vs King
- **Intelligent Caching**: LRU/LFU/Adaptive eviction strategies with memory monitoring
- **Performance Profiling**: Detailed timing and performance metrics
- **WASM Compatibility**: Optimized for web deployment
- **Adaptive Selection**: Smart solver selection based on position complexity
- **Memory Management**: Configurable memory limits and monitoring
- **Comprehensive Testing**: Unit, integration, and performance tests

## Quick Start

### Basic Usage

```rust
use shogi_engine::{ShogiEngine, BitboardBoard, Player, CapturedPieces};

// Create engine with tablebase enabled
let mut engine = ShogiEngine::new();
engine.enable_tablebase();

// Get best move (tablebase will be consulted automatically)
let best_move = engine.get_best_move(1, 1000, None, None);

// Check tablebase statistics
let stats = engine.get_tablebase_stats();
println!("Tablebase stats: {}", stats);
```

### Configuration

```rust
use shogi_engine::tablebase::TablebaseConfig;

// Create optimized configuration
let config = TablebaseConfig::performance_optimized();

// Or create custom configuration
let mut config = TablebaseConfig::default();
config.enabled = true;
config.cache_size = 10000;
config.confidence_threshold = 0.95;

// Apply configuration to engine
let mut engine = ShogiEngine::new_with_config(config);
```

## Architecture

### Core Components

1. **MicroTablebase**: Central coordinator managing all solvers and caching
2. **EndgameSolver**: Trait defining solver interface
3. **PositionCache**: Intelligent caching system with multiple eviction strategies
4. **TablebaseConfig**: Comprehensive configuration management
5. **TablebaseStats**: Performance monitoring and statistics

### Solver Types

#### King + Gold vs King
- **Priority**: 90 (highest)
- **Use Case**: Simple endgames with king and gold
- **Patterns**: Direct mating, king approach, coordination

#### King + Silver vs King  
- **Priority**: 80
- **Use Case**: Endgames with king and silver
- **Patterns**: Silver-specific mating patterns

#### King + Rook vs King
- **Priority**: 70
- **Use Case**: Endgames with king and rook
- **Patterns**: Rook mating patterns and coordination

## Configuration Options

### Basic Configuration

```rust
let config = TablebaseConfig {
    enabled: true,
    cache_size: 10000,
    confidence_threshold: 0.95,
    max_depth: 50,
    // ... other options
};
```

### Memory Configuration

```rust
let memory_config = MemoryConfig {
    enable_monitoring: true,
    max_memory_bytes: 100 * 1024 * 1024, // 100MB
    warning_threshold: 0.8,
    critical_threshold: 0.95,
    enable_auto_eviction: true,
    check_interval_ms: 1000,
    enable_logging: true,
};
```

### WASM Configuration

```rust
let wasm_config = WasmConfig {
    enable_wasm_optimizations: true,
    max_wasm_memory: 50 * 1024 * 1024, // 50MB
    use_compact_structures: true,
    disable_heavy_features: false,
    force_synchronous: false,
    reduce_cache_sizes: true,
    disable_memory_monitoring: false,
    use_simple_logging: true,
};
```

### Performance Configuration

```rust
let perf_config = PerformanceConfig {
    eviction_strategy: EvictionStrategy::Adaptive,
    enable_adaptive_caching: true,
    max_iterations: Some(1000),
    enable_profiling: true,
    profile_interval_ms: 100,
};
```

## Usage Examples

### Basic Tablebase Probing

```rust
use shogi_engine::tablebase::MicroTablebase;
use shogi_engine::{BitboardBoard, Player, CapturedPieces};

let mut tablebase = MicroTablebase::new();
let board = BitboardBoard::new();
let captured_pieces = CapturedPieces::new();

// Probe tablebase for best move
if let Some(result) = tablebase.probe(&board, Player::Black, &captured_pieces) {
    println!("Best move: {:?}", result.best_move);
    println!("Distance to mate: {}", result.distance_to_mate);
    println!("Outcome: {:?}", result.outcome);
    println!("Confidence: {}", result.confidence);
}
```

### Custom Solver Configuration

```rust
use shogi_engine::tablebase::tablebase_config::*;

let mut config = TablebaseConfig::default();
config.solvers.king_gold_vs_king.enabled = true;
config.solvers.king_silver_vs_king.enabled = true;
config.solvers.king_rook_vs_king.enabled = false; // Disable rook solver

let tablebase = MicroTablebase::with_config(config);
```

### Performance Monitoring

```rust
// Enable profiling
tablebase.set_profiling_enabled(true);

// Perform operations
for _ in 0..1000 {
    tablebase.probe(&board, Player::Black, &captured_pieces);
}

// Get performance summary
let profiler = tablebase.get_profiler();
let summary = profiler.get_summary();
println!("Performance Summary:\n{}", summary);

// Get most expensive operations
let expensive_ops = profiler.get_most_expensive_operations(5);
for (op, metrics) in expensive_ops {
    println!("{}: {} calls, avg: {:?}", op, metrics.call_count, metrics.average_duration());
}
```

### Memory Management

```rust
// Check memory usage
let memory_usage = tablebase.get_current_memory_usage();
let peak_usage = tablebase.get_peak_memory_usage();
println!("Current memory: {} bytes", memory_usage);
println!("Peak memory: {} bytes", peak_usage);

// Get memory summary
let memory_summary = tablebase.get_memory_summary();
println!("Memory Summary:\n{}", memory_summary);

// Force memory cleanup if needed
if tablebase.get_current_memory_usage() > 50 * 1024 * 1024 {
    tablebase.perform_emergency_eviction();
}
```

### Adaptive Solver Selection

```rust
// Analyze position complexity
let analysis = tablebase.analyze_position(&board, Player::Black, &captured_pieces);
println!("Position complexity: {:?}", analysis.complexity);
println!("Recommended solver priority: {}", analysis.recommended_solver_priority);

// Check if position is suitable for a specific solver
let is_suitable = tablebase.is_position_suitable_for_solver(&board, Player::Black, &captured_pieces, 90);
println!("Suitable for high-priority solvers: {}", is_suitable);
```

## Integration with Search Engine

The tablebase system is automatically integrated with the search engine:

1. **Pre-search Probing**: Tablebase is consulted before normal search
2. **Move Prioritization**: Tablebase moves are prioritized in move ordering
3. **Score Conversion**: Tablebase scores are converted to search scores
4. **Fallback**: Normal search continues if no tablebase solution

### Search Integration Example

```rust
use shogi_engine::search::SearchEngine;

let mut search_engine = SearchEngine::new();
search_engine.enable_tablebase();

// Search will automatically use tablebase
let (best_move, score) = search_engine.get_best_move(&board, Player::Black, &captured_pieces, 1, 1000);

// Check if move came from tablebase
if search_engine.is_tablebase_move(&best_move) {
    println!("Move from tablebase!");
}
```

## Performance Optimization

### Cache Optimization

```rust
// Use adaptive eviction for better performance
let mut config = TablebaseConfig::default();
config.performance.eviction_strategy = EvictionStrategy::Adaptive;
config.performance.enable_adaptive_caching = true;

let tablebase = MicroTablebase::with_config(config);
```

### Memory Optimization

```rust
// Optimize for memory-constrained environments
let config = TablebaseConfig::memory_optimized();
let tablebase = MicroTablebase::with_config(config);
```

### WASM Optimization

```rust
// Optimize for web deployment
let config = TablebaseConfig::wasm_optimized();
let tablebase = MicroTablebase::with_config(config);
```

## Testing

### Running Tests

```bash
# Run all tablebase tests
cargo test tablebase

# Run specific test categories
cargo test tablebase_tests
cargo test tablebase_integration_tests
cargo test tablebase_endgame_tests
cargo test tablebase_engine_integration_tests

# Run performance benchmarks
cargo bench tablebase_performance_benchmarks
```

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Component interaction testing
3. **Endgame Tests**: Specific endgame scenario testing
4. **Engine Tests**: Full engine integration testing
5. **Performance Tests**: Benchmarking and performance validation

## Troubleshooting

### Common Issues

1. **No Tablebase Hits**: Check if position matches solver patterns
2. **Memory Issues**: Adjust memory limits or enable auto-eviction
3. **Performance Issues**: Enable profiling and check operation timings
4. **WASM Issues**: Use WASM-optimized configuration

### Debug Information

```rust
// Enable debug logging
let mut config = TablebaseConfig::default();
config.wasm.use_simple_logging = false; // Enable detailed logging

// Get detailed statistics
let stats = tablebase.get_stats();
println!("Cache hits: {}", stats.cache_hits);
println!("Solver hits: {}", stats.solver_hits);
println!("Average probe time: {}ms", stats.average_probe_time);
```

## Future Extensions

### Planned Features

1. **Additional Solvers**: More endgame patterns
2. **Machine Learning**: ML-based position evaluation
3. **Distributed Caching**: Multi-node cache sharing
4. **Advanced Analytics**: Detailed performance analysis

### Contributing

To add new endgame solvers:

1. Implement the `EndgameSolver` trait
2. Add solver to `MicroTablebase`
3. Create comprehensive tests
4. Update documentation

## API Reference

### MicroTablebase

- `new()` - Create with default configuration
- `with_config(config)` - Create with custom configuration
- `probe(board, player, captured_pieces)` - Probe for best move
- `enable()` / `disable()` - Enable/disable tablebase
- `get_stats()` - Get performance statistics
- `reset_stats()` - Reset statistics

### TablebaseConfig

- `default()` - Default configuration
- `performance_optimized()` - Performance-optimized configuration
- `memory_optimized()` - Memory-optimized configuration
- `wasm_optimized()` - WASM-optimized configuration

### TablebaseStats

- `total_probes` - Total number of probes
- `cache_hits` - Number of cache hits
- `solver_hits` - Number of solver hits
- `average_probe_time` - Average probe time in milliseconds
- `memory_usage_percentage` - Current memory usage percentage

## License

This tablebase system is part of the Shogi Engine project and follows the same license terms.
