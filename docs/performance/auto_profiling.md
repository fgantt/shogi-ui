# Automatic Profiling Integration for Hot Paths

## Overview

The automatic profiling system (Task 26.0 - Task 3.0) provides lightweight, configurable profiling for identifying performance bottlenecks in the search engine. It automatically profiles hot paths including evaluation, move ordering, and transposition table operations.

## Features

- **Automatic Profiling**: Enable profiling with a single method call
- **Sampling**: Configurable sample rate to reduce profiling overhead
- **Hot Path Identification**: Automatically identifies slowest operations
- **Overhead Tracking**: Measures and reports profiling overhead
- **JSON Export**: Export profiling data for analysis

## Configuration

### Engine Configuration

Profiling can be configured in `EngineConfig`:

```rust
let mut config = EngineConfig::default();
config.auto_profiling_enabled = true;
config.auto_profiling_sample_rate = 100; // Profile every 100th call

let mut engine = SearchEngine::new_with_engine_config(None, config);
```

### Runtime Configuration

Profiling can also be enabled/disabled at runtime:

```rust
let mut engine = SearchEngine::new(None, 16);

// Enable profiling
engine.enable_auto_profiling();

// Disable profiling
engine.disable_auto_profiling();
```

## Usage

### Basic Usage

```rust
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::types::*;

let mut engine = SearchEngine::new(None, 16);
engine.enable_auto_profiling();

// Run search operations
let board = BitboardBoard::new();
let captured_pieces = CapturedPieces::new();
let _ = engine.evaluate_position(&board, Player::Black, &captured_pieces);

// Get hot path summary
let hot_paths = engine.get_hot_path_summary(10);
for path in hot_paths {
    println!("{}: avg={:.2}ns, max={}ns, calls={}",
        path.operation,
        path.average_time_ns,
        path.max_time_ns,
        path.call_count
    );
}
```

### Exporting Profiling Data

```rust
// Export profiling data to JSON
match engine.export_profiling_data() {
    Ok(json) => {
        std::fs::write("profiling_data.json", json)?;
    }
    Err(e) => eprintln!("Failed to export profiling data: {}", e),
}
```

### Profiled Operations

The following operations are automatically profiled when enabled:

- **evaluation**: Position evaluation time
- **move_ordering**: Move ordering time
- **tt_probe**: Transposition table probe time
- **tt_store**: Transposition table store time
- **phase_calculation**: Phase calculation time (from evaluator)
- **interpolation**: Interpolation time (from evaluator)

## Sample Rate

The sample rate controls how frequently operations are profiled:

- **Sample rate = 1**: Profile every call (highest overhead, most accurate)
- **Sample rate = 100**: Profile every 100th call (default, balanced)
- **Sample rate = 1000**: Profile every 1000th call (lowest overhead, less accurate)

Higher sample rates reduce profiling overhead but may miss short-duration operations.

## Overhead Tracking

The profiler tracks its own overhead to help assess impact:

```rust
let overhead_pct = engine.performance_profiler.get_profiling_overhead_percentage();
println!("Profiling overhead: {:.2}%", overhead_pct);
```

Typical overhead is < 1% with default sample rate (100).

## Hot Path Summary

The hot path summary identifies the slowest operations:

```rust
let hot_paths = engine.get_hot_path_summary(10); // Top 10 slowest operations

for (i, path) in hot_paths.iter().enumerate() {
    println!("{}. {}: avg={:.2}ns ({} calls)",
        i + 1,
        path.operation,
        path.average_time_ns,
        path.call_count
    );
}
```

Entries are sorted by average time (descending).

## JSON Export Format

The exported JSON includes:

```json
{
  "enabled": true,
  "sample_rate": 100,
  "total_samples": 1234,
  "profiling_overhead_ns": 5000,
  "profiling_operations": 1000,
  "average_overhead_per_operation_ns": 5.0,
  "evaluation_stats": {
    "count": 1000,
    "average_ns": 1500.0,
    "max_ns": 5000,
    "min_ns": 500
  },
  "hot_paths": [
    {
      "operation": "evaluation",
      "average_time_ns": 1500.0,
      "max_time_ns": 5000,
      "min_time_ns": 500,
      "call_count": 1000
    }
  ]
}
```

## Best Practices

1. **Use sampling**: Always use a sample rate > 1 for production profiling to minimize overhead
2. **Profile selectively**: Enable profiling only when investigating performance issues
3. **Reset between runs**: Call `profiler.reset()` between profiling sessions for clean data
4. **Export for analysis**: Export profiling data to JSON for detailed analysis
5. **Monitor overhead**: Check profiling overhead percentage to ensure it's acceptable

## Limitations

- Profiling adds some overhead even with sampling
- Sample rate may miss short-duration operations
- TT operations are profiled at the search engine level, not inside the TT implementation
- Memory profiling is not included (see Task 4.0 for memory tracking)

## Future Enhancements

- Per-operation sample rates (different rates for different operations)
- Statistical sampling (sample based on operation duration)
- Real-time profiling dashboard
- Integration with external profilers (see Task 8.0)

