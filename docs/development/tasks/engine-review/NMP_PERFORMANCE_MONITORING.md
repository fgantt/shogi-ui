# Null Move Pruning Performance Monitoring Guide

This document describes how to use the automated performance monitoring and benchmarking suite for Null Move Pruning (NMP).

## Overview

The performance monitoring suite provides:
- Automated benchmark execution for CI/CD
- Performance regression detection
- Historical statistics tracking
- Position-type specific metrics (opening, middlegame, endgame)
- Comparison benchmarks (NMP enabled vs disabled)
- Automated performance reports

## Running Benchmarks

### Basic Execution

```bash
# Run the performance monitoring benchmarks
cargo bench --bench null_move_performance_monitoring_benchmarks

# Run regression tests
cargo test --test null_move_regression_tests

# Run using the benchmark script
./scripts/run_nmp_benchmarks.sh
```

### CI/CD Execution

For automated CI/CD runs with regression testing:

```bash
# Set regression test environment variable
export NMP_REGRESSION_TEST=1
export NMP_METRICS_DIR=target/nmp_metrics

# Run benchmarks with regression checks
cargo bench --bench null_move_performance_monitoring_benchmarks

# Run regression tests
cargo test --test null_move_regression_tests
```

### Configuration Options

Environment variables for benchmark configuration:

- `NMP_REGRESSION_TEST`: Enable regression test checks (benchmarks will fail if thresholds not met)
- `NMP_METRICS_DIR`: Directory to save performance metrics JSON files
- `BENCHMARK_RESULTS_DIR`: Directory for Criterion benchmark results (default: `target/criterion`)

## Benchmark Suites

### 1. NMP Comparison Benchmarks

Compares NMP performance with different configurations:
- Disabled
- Default
- With verification search
- With mate threat detection
- With endgame type detection
- Full features enabled

**Usage:**
```bash
cargo bench --bench null_move_performance_monitoring_benchmarks -- nmp_comparison
```

### 2. Position Type Benchmarks

Measures NMP effectiveness across different position types:
- Initial position
- Opening positions
- Middlegame positions
- Endgame positions

**Usage:**
```bash
cargo bench --bench null_move_performance_monitoring_benchmarks -- nmp_by_position_type
```

### 3. Regression Testing Benchmarks

Tests performance at different depths to ensure consistent performance:
- Depth 3
- Depth 4
- Depth 5

**Usage:**
```bash
NMP_REGRESSION_TEST=1 cargo bench --bench null_move_performance_monitoring_benchmarks -- nmp_regression_testing
```

### 4. Comprehensive Monitoring

Full performance monitoring with metrics collection and historical tracking.

**Usage:**
```bash
NMP_METRICS_DIR=target/nmp_metrics cargo bench --bench null_move_performance_monitoring_benchmarks -- comprehensive_nmp_monitoring
```

## Performance Regression Tests

The regression test suite (`tests/null_move_regression_tests.rs`) includes:

1. **Default Configuration Test**: Verifies cutoff rate >= 20%, efficiency >= 15%
2. **Disabled Test**: Verifies search completes without NMP
3. **Verification Test**: Verifies cutoff rate >= 15% with verification enabled
4. **Effectiveness Test**: Compares NMP enabled vs disabled
5. **Different Depths Test**: Tests performance at depths 3, 4, 5

### Running Regression Tests

```bash
# Run all regression tests
cargo test --test null_move_regression_tests

# Run specific test
cargo test --test null_move_regression_tests test_nmp_performance_regression_default_config
```

## Performance Baselines

Default performance thresholds (can be customized):

```rust
PerformanceBaseline {
    min_cutoff_rate: 30.0,      // At least 30% cutoff rate
    max_search_time_ms: 5000.0, // Max 5 seconds per search
    min_efficiency: 20.0,       // At least 20% efficiency
    max_overhead_percent: 20.0, // Max 20% overhead from NMP features
}
```

## Metrics Tracking

### Metrics Structure

Each benchmark run generates a `PerformanceMetrics` record:

```rust
struct PerformanceMetrics {
    timestamp: String,
    configuration: String,
    position_type: String,
    depth: u8,
    search_time_ms: f64,
    nodes_searched: u64,
    nmp_attempts: u64,
    nmp_cutoffs: u64,
    cutoff_rate: f64,
    average_reduction: f64,
    efficiency: f64,
    verification_attempts: u64,
    verification_cutoffs: u64,
    mate_threat_attempts: u64,
    mate_threat_detected: u64,
    disabled_endgame: u64,
    disabled_material_endgame: u64,
    disabled_king_activity_endgame: u64,
    disabled_zugzwang: u64,
}
```

### Historical Tracking

Metrics are saved to JSON files when `NMP_METRICS_DIR` is set:

```bash
# Enable metrics saving
export NMP_METRICS_DIR=target/nmp_metrics

# Run benchmarks (metrics automatically saved)
cargo bench --bench null_move_performance_monitoring_benchmarks

# Metrics saved to: target/nmp_metrics/nmp_metrics.json
```

The metrics file keeps the last 100 entries to prevent file growth.

## Interpreting Results

### Key Metrics

1. **Cutoff Rate**: Percentage of null move attempts that resulted in cutoffs
   - Target: >= 30% (or >= 20% in regression tests)
   - Higher is better - indicates NMP is effective

2. **Efficiency**: Overall efficiency of NMP
   - Target: >= 20%
   - Higher is better - indicates good NMP usage

3. **Average Reduction**: Average depth reduction per null move attempt
   - Typically 2-5 depending on depth and formula
   - Higher reductions = more aggressive pruning

4. **Search Time**: Time to complete search
   - Should be reasonable (depends on depth)
   - Regression tests verify it doesn't exceed thresholds

### Regression Detection

A regression is detected when:
- Cutoff rate drops below threshold
- Efficiency drops below threshold
- Search time exceeds maximum threshold

When regression is detected:
- Benchmarks fail with descriptive error messages
- CI/CD pipeline should fail to alert developers

## CI/CD Integration

### GitHub Actions Example

```yaml
name: NMP Performance Benchmarks

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run NMP regression tests
        run: |
          export NMP_REGRESSION_TEST=1
          export NMP_METRICS_DIR=target/nmp_metrics
          cargo bench --bench null_move_performance_monitoring_benchmarks
          cargo test --test null_move_regression_tests
      
      - name: Upload metrics
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: nmp-metrics
          path: target/nmp_metrics/
```

## Viewing Results

### Criterion HTML Reports

Criterion generates HTML reports in `target/criterion/`:

- **Comparison benchmarks**: `target/criterion/nmp_comparison/`
- **Position type benchmarks**: `target/criterion/nmp_by_position_type/`
- **Regression benchmarks**: `target/criterion/nmp_regression_testing/`
- **Comprehensive monitoring**: `target/criterion/comprehensive_nmp_monitoring/`

Open `report/index.html` in each directory to view detailed results.

### Metrics JSON

Historical metrics are saved in JSON format:

```bash
# View metrics
cat target/nmp_metrics/nmp_metrics.json | jq '.[] | select(.configuration == "default") | {cutoff_rate, efficiency, search_time_ms}'
```

## Troubleshooting

### Benchmarks Fail with Regression Errors

If regression tests fail:
1. Check if NMP configuration was changed
2. Verify search behavior hasn't changed
3. Review recent commits for NMP-related changes
4. Adjust baseline thresholds if needed (in `PerformanceBaseline::default()`)

### Metrics Not Saving

If metrics aren't being saved:
1. Verify `NMP_METRICS_DIR` environment variable is set
2. Check directory permissions
3. Ensure directory exists or can be created

### Slow Benchmarks

If benchmarks are too slow:
1. Reduce sample size: Set `sample_size` in benchmark groups
2. Reduce measurement time: Set `measurement_time` to shorter duration
3. Skip optional benchmarks: Run only essential regression tests

## Best Practices

1. **Run regression tests in CI/CD** to catch performance regressions early
2. **Save metrics regularly** to track performance trends over time
3. **Review metrics after significant changes** to NMP implementation
4. **Adjust baselines** as needed when NMP behavior intentionally changes
5. **Document threshold changes** when updating performance baselines

## Related Documentation

- `tasks-task-2.0-null-move-pruning-review.md`: Task list and implementation notes
- Criterion documentation: https://docs.rs/criterion/
- Performance optimization guide: `docs/design/implementation/performance-analysis/`

