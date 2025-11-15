# Memory Usage Tracking (RSS)

## Overview

The memory tracking system (Task 26.0 - Task 4.0) provides actual RSS (Resident Set Size) memory tracking using the `sysinfo` crate for cross-platform support. It tracks current and peak memory usage, detects memory leaks, and provides component-level memory breakdowns.

## Features

- **Actual RSS Tracking**: Uses operating system APIs to get real memory usage
- **Peak Memory Tracking**: Tracks maximum memory usage during search
- **Memory Growth Tracking**: Monitors memory growth since initialization
- **Leak Detection**: Alerts when memory growth exceeds threshold (>50% by default)
- **Component Breakdown**: Combines RSS with component-level estimates (TT, caches, etc.)

## Usage

### Basic Usage

```rust
use shogi_engine::search::search_engine::SearchEngine;
use shogi_engine::types::*;

let mut engine = SearchEngine::new(None, 16);

// Get current RSS
let current_rss = engine.get_memory_usage();
println!("Current RSS: {} bytes", current_rss);

// Get memory breakdown
let breakdown = engine.get_memory_breakdown();
println!("Current RSS: {} bytes", breakdown.current_rss_bytes);
println!("Peak RSS: {} bytes", breakdown.peak_rss_bytes);
println!("Memory growth: {} bytes ({:.2}%)",
    breakdown.memory_growth_bytes,
    breakdown.memory_growth_percentage
);
```

### Memory Tracking During Search

Memory tracking is automatically integrated into search operations:

- **Search Start**: Peak tracking is reset
- **Search End**: Peak RSS is updated
- **Periodic Updates**: Memory is tracked during long searches

```rust
// Memory tracking happens automatically during search
let result = engine.search_at_depth(&board, &captured_pieces, Player::Black, 5);

// After search, check memory
let breakdown = engine.get_memory_breakdown();
if breakdown.leak_detected {
    println!("Warning: Potential memory leak detected!");
}
```

### Direct MemoryTracker Usage

```rust
use shogi_engine::search::memory_tracking::MemoryTracker;

let tracker = MemoryTracker::new();

// Get current RSS
let rss = tracker.get_current_rss();

// Get peak RSS
let peak = tracker.get_peak_rss();

// Get memory growth
let growth = tracker.get_memory_growth();
let growth_pct = tracker.get_memory_growth_percentage();

// Check for leak
if tracker.check_for_leak() {
    println!("Memory leak detected!");
}
```

## Memory Breakdown

The memory breakdown combines actual RSS with component-level estimates:

```rust
let breakdown = engine.get_memory_breakdown();

// RSS data
breakdown.current_rss_bytes      // Current RSS
breakdown.peak_rss_bytes         // Peak RSS
breakdown.memory_growth_bytes    // Growth since start
breakdown.memory_growth_percentage // Growth percentage

// Component estimates
breakdown.component_breakdown.tt_memory_bytes           // TT memory
breakdown.component_breakdown.cache_memory_bytes        // Cache memory
breakdown.component_breakdown.move_ordering_memory_bytes // Move ordering memory
breakdown.component_breakdown.other_memory_bytes        // Other memory
breakdown.component_breakdown.total_component_bytes     // Total estimated
```

## Leak Detection

Memory leak detection alerts when memory growth exceeds a threshold:

- **Default Threshold**: 50% growth
- **Configurable**: Can be set via `MemoryTracker::with_leak_threshold()`
- **Automatic**: Checked during `track_memory_usage()`

```rust
// Create tracker with custom threshold
let tracker = MemoryTracker::with_leak_threshold(25.0); // 25% threshold

// Check for leak
if tracker.check_for_leak() {
    println!("Memory leak detected: {:.2}% growth",
        tracker.get_memory_growth_percentage()
    );
}
```

## Integration with Performance Baselines

Memory metrics are automatically included in performance baselines:

```rust
let baseline = engine.collect_baseline_metrics();

// Memory metrics use actual RSS
baseline.memory_metrics.peak_memory_mb  // Peak RSS in MB
baseline.memory_metrics.tt_memory_mb    // TT memory estimate
baseline.memory_metrics.cache_memory_mb // Cache memory estimate
```

## Performance Metrics

Memory tracking is integrated into `PerformanceMetrics`:

```rust
let metrics = PerformanceMetrics::default();

metrics.current_rss_bytes    // Current RSS
metrics.peak_rss_bytes       // Peak RSS
metrics.memory_growth_bytes  // Memory growth
```

## Platform Support

The `sysinfo` crate provides cross-platform support:

- **Linux**: Uses `/proc/self/status` or `/proc/self/statm`
- **macOS**: Uses `libproc` or `sysctl`
- **Windows**: Uses `GetProcessMemoryInfo`

## Limitations

1. **RSS vs Allocated**: RSS reflects actual physical memory, not allocated memory
2. **Component Estimates**: Component breakdowns are estimates, not exact measurements
3. **Platform Differences**: Memory reporting may vary slightly between platforms
4. **Overhead**: Memory tracking adds minimal overhead (< 0.1%)

## Best Practices

1. **Reset at Search Start**: Always reset peak tracking at search start
2. **Monitor Growth**: Check memory growth percentage regularly
3. **Leak Detection**: Use leak detection to identify memory issues early
4. **Component Analysis**: Use component breakdown to identify memory hotspots
5. **Baseline Comparison**: Compare memory usage across different code versions

## Troubleshooting

### High Memory Usage

If memory usage is high:

1. Check component breakdown to identify largest components
2. Verify TT size is appropriate
3. Check for memory leaks using leak detection
4. Review cache sizes and eviction policies

### Memory Leak Detection

If leak detection triggers:

1. Verify it's not a false positive (normal growth during search)
2. Check for unbounded data structures
3. Review cache eviction policies
4. Use profiling tools to identify leaks

## Future Enhancements

- Per-component RSS tracking (if OS supports it)
- Memory allocation tracking (using custom allocators)
- Historical memory trend analysis
- Memory pressure detection and automatic cleanup

