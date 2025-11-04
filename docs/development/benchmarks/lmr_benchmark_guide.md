# LMR Benchmark Guide

This guide explains how to run and interpret LMR (Late Move Reduction) benchmarks.

## Overview

LMR benchmarks are designed to:
- Measure LMR performance and effectiveness
- Track performance over time to detect regressions
- Compare different LMR configurations
- Validate performance thresholds

## Running Benchmarks

### All LMR Benchmarks

```bash
cargo bench --bench lmr_consolidation_performance_benchmarks
cargo bench --bench lmr_re_search_margin_benchmarks
cargo bench --bench lmr_tt_move_detection_benchmarks
cargo bench --bench lmr_performance_monitoring_benchmarks
```

### Specific Benchmark Groups

```bash
# Consolidation benchmarks
cargo bench --bench lmr_consolidation_performance_benchmarks -- --bench

# Re-search margin benchmarks
cargo bench --bench lmr_re_search_margin_benchmarks -- --bench

# TT move detection benchmarks
cargo bench --bench lmr_tt_move_detection_benchmarks -- --bench

# Performance monitoring benchmarks
cargo bench --bench lmr_performance_monitoring_benchmarks -- --bench
```

### CI/CD Integration

Benchmarks can be run in CI/CD pipelines to track performance over time:

```yaml
# Example GitHub Actions workflow
- name: Run LMR Benchmarks
  run: |
    cargo bench --bench lmr_performance_monitoring_benchmarks -- --test
```

## Benchmark Suites

### 1. Consolidation Performance Benchmarks

**Purpose:** Measure the performance of the consolidated LMR implementation in PruningManager.

**Metrics:**
- Search time
- Nodes searched
- LMR reduction rate (efficiency)
- Re-search rate
- Cutoff rate
- Average reduction and depth saved

**Thresholds:**
- Efficiency >= 25%
- Re-search rate <= 30%
- Cutoff rate >= 10%

### 2. Re-search Margin Benchmarks

**Purpose:** Measure the impact of re-search margin on LMR effectiveness.

**Metrics:**
- Search time with different margin values (0, 25, 50, 75, 100 centipawns)
- Re-search rate
- LMR effectiveness
- Re-search margin effectiveness

**Expected Results:**
- Re-search margin should reduce re-search rate
- Optimal margin value balances efficiency with accuracy

### 3. TT Move Detection Benchmarks

**Purpose:** Measure the performance impact of actual TT move detection vs heuristic.

**Metrics:**
- Search time with TT detection enabled vs disabled
- TT move exemption rate
- LMR effectiveness
- TT move tracking overhead

**Expected Results:**
- TT move tracking overhead < 1%
- Improved LMR accuracy with actual TT move detection

### 4. Performance Monitoring Benchmarks

**Purpose:** Comprehensive monitoring and comparison of LMR performance.

**Metrics:**
- LMR enabled vs disabled comparison
- Different LMR configurations (default, aggressive, conservative)
- Performance regression validation
- Phase-specific performance tracking
- Performance alerts and thresholds

## Interpreting Results

### Performance Thresholds

LMR performance is considered healthy if it meets all thresholds:

1. **Efficiency (>= 25%)**: Percentage of moves that have LMR applied
   - Too low: LMR not being applied enough
   - Too high: LMR being applied too aggressively

2. **Re-search Rate (5-30%)**: Percentage of reduced searches that trigger re-search
   - Too high (>30%): LMR too aggressive, many re-searches needed
   - Too low (<5%): LMR too conservative, missing opportunities

3. **Cutoff Rate (>= 10%)**: Percentage of moves that cause cutoffs
   - Too low: Poor move ordering or LMR applied incorrectly
   - Higher is generally better (indicates good move ordering)

### Performance Alerts

The benchmark suite includes alert mechanisms that detect:

- **Low efficiency (<25%)**: LMR not being applied enough
- **High re-search rate (>30%)**: LMR too aggressive
- **Low re-search rate (<5%)**: LMR too conservative
- **Low cutoff rate (<10%)**: Poor move ordering correlation

### Regression Detection

Performance regression tests validate that:

1. Efficiency >= 25%
2. Re-search rate <= 30%
3. Cutoff rate >= 10%
4. Search time doesn't increase significantly (<5% acceptable)

If thresholds are not met, the benchmark will report alerts and fail in CI/CD.

## Phase-Specific Performance

LMR performance can vary by game phase:

- **Opening**: Typically lower reduction rates, more tactical
- **Middlegame**: Balanced reduction rates
- **Endgame**: May have different reduction patterns

Phase statistics are tracked separately and included in performance reports.

## Configuration Comparison

Benchmarks compare different LMR configurations:

1. **Default**: Balanced configuration
2. **Aggressive**: Higher reduction, lower re-search margin
3. **Conservative**: Lower reduction, higher re-search margin

Use benchmarks to find the optimal configuration for your use case.

## Benchmark Configuration

Benchmark execution can be configured via environment variables:

```bash
# Increase measurement time
Criterion.time = Duration::from_secs(30)

# Increase sample size
Criterion.sample_size = 20
```

## Performance Reports

After running benchmarks, performance reports include:

- Overall LMR statistics (efficiency, re-search rate, cutoff rate)
- Performance by game phase
- Performance alerts (if any)
- Comparison with baseline metrics

Reports can be exported for analysis:

```rust
let metrics = engine.export_lmr_metrics();
let report = engine.get_lmr_performance_report();
```

## CI/CD Integration

### Setting Up Automated Benchmarks

1. Add benchmark jobs to CI/CD pipeline
2. Run benchmarks on every commit (or PR)
3. Compare results with baseline
4. Fail pipeline if thresholds not met
5. Store results for historical tracking

### Example GitHub Actions Workflow

```yaml
name: LMR Performance Benchmarks

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run LMR Benchmarks
        run: |
          cargo bench --bench lmr_performance_monitoring_benchmarks -- --test
```

## Troubleshooting

### Benchmarks Taking Too Long

- Reduce measurement time
- Reduce sample size
- Run specific benchmark groups only

### High Re-search Rate

- Check if re-search margin is too low
- Verify LMR is not too aggressive
- Review move ordering quality

### Low Efficiency

- Check if LMR thresholds are too strict
- Verify extended exemptions are not too broad
- Review position classification accuracy

### Low Cutoff Rate

- Check move ordering quality
- Verify LMR is not applied to good moves
- Review TT move detection accuracy

## Additional Resources

- [LMR Design Documentation](../design/implementation/late-move-reductions/DESIGN_LATE_MOVE_REDUCTIONS.md)
- [Task List: LMR Improvements](../../tasks/engine-review/tasks-task-3.0-late-move-reduction-review.md)
- [Performance Monitoring API Reference](../../../src/types.rs)

