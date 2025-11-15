# Performance Baseline Documentation

This directory contains performance baselines for regression detection and trend analysis (Task 26.0 - Task 1.0).

## Overview

Performance baselines capture a snapshot of engine performance metrics at a specific point in time. They are used to:

- Detect performance regressions in new code changes
- Track performance trends over time
- Compare performance across different hardware configurations
- Validate optimization effectiveness

## Baseline Format

Baselines are stored as JSON files matching the structure defined in `src/types.rs::PerformanceBaseline`. Each baseline includes:

- **Timestamp**: ISO 8601 timestamp when baseline was created
- **Git Commit**: Git commit hash for version tracking
- **Hardware Info**: CPU model, core count, RAM size
- **Search Metrics**: Nodes per second, cutoff rates, cutoff indices
- **Evaluation Metrics**: Evaluation time, cache hit rates
- **TT Metrics**: Transposition table hit rates, entry quality, occupancy
- **Move Ordering Metrics**: Cutoff indices, PV/killer/cache hit rates
- **Parallel Search Metrics**: Speedup and efficiency on multiple cores
- **Memory Metrics**: Memory usage by component

## Usage

### Creating a Baseline

```bash
# Run the baseline script
./scripts/run_performance_baseline.sh

# Or programmatically in Rust:
use shogi_engine::search::performance_tuning::BaselineManager;
use shogi_engine::search::search_engine::SearchEngine;

let engine = SearchEngine::new(None, 16);
// ... run representative searches ...
let baseline = engine.collect_baseline_metrics();
let manager = BaselineManager::new();
manager.save_baseline(&baseline, "baseline.json")?;
```

### Loading a Baseline

```rust
use shogi_engine::search::performance_tuning::BaselineManager;

let manager = BaselineManager::new();
let baseline = manager.load_baseline("baseline.json")?;
```

### Comparing Baselines

```rust
let manager = BaselineManager::new();
let current = engine.collect_baseline_metrics();
let reference = manager.load_baseline("reference.json")?;

let comparison = manager.compare_baselines(&current, &reference);
println!("Nodes/sec change: {:.2}%", 
    comparison.search_metrics_diff.nodes_per_second_change);
```

### Detecting Regressions

```rust
let result = manager.detect_regression(&current, &reference);
if result.has_regression {
    println!("Regressions detected:");
    for regression in &result.regressions {
        println!("  {}: {:.2}% change", regression.metric, regression.change_percent);
    }
}
```

## Regression Threshold

The default regression threshold is **5.0%**. This means any metric that degrades by more than 5% compared to the baseline is flagged as a regression.

You can customize the threshold:

```rust
let mut manager = BaselineManager::new();
manager.set_regression_threshold(10.0); // 10% threshold
```

## File Naming Convention

Baseline files are typically named:
- `baseline-{commit_hash}-{timestamp}.json` - Timestamped baselines
- `latest.json` - Symlink or copy of most recent baseline
- `reference.json` - Reference baseline for comparisons

## CI Integration

Baselines can be used in CI workflows to automatically detect performance regressions:

1. Load the reference baseline from `docs/performance/baselines/latest.json`
2. Run benchmarks to collect current metrics
3. Compare current metrics with baseline
4. Fail the build if regressions are detected

See Task 6.0 for CI integration details.

## Best Practices

1. **Create baselines after major optimizations** - Capture performance after significant changes
2. **Version control baselines** - Commit baseline files to track performance history
3. **Hardware-specific baselines** - Create separate baselines for different hardware configurations
4. **Regular updates** - Update reference baselines when performance improves
5. **Document context** - Include notes about what changed between baselines

## Limitations

- Evaluation metrics (average time, phase calc time) are currently placeholders and need evaluator interface enhancements
- Parallel search metrics default to 0 if parallel search is not used
- Memory metrics are estimates based on data structure sizes, not actual RSS

## Future Enhancements

- Automatic baseline generation in CI
- Baseline comparison visualization
- Performance trend analysis over time
- Hardware-specific baseline recommendations

