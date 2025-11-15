# Benchmark Result Aggregation and Reporting

This directory contains aggregated benchmark reports generated from Criterion.rs benchmark results (Task 26.0 - Task 2.0).

## Overview

The benchmark aggregation system collects results from Criterion.rs benchmark runs, aggregates them into comprehensive reports, and compares them against performance baselines to detect regressions.

## Features

- **Automatic Aggregation**: Parses Criterion.rs JSON output from `target/criterion/`
- **Baseline Comparison**: Compares current results against performance baselines
- **Multiple Export Formats**: JSON and Markdown report formats
- **Regression Detection**: Flags benchmarks with performance regressions (>5% threshold)
- **Summary Statistics**: Aggregated metrics across all benchmarks

## Usage

### Basic Aggregation

```rust
use shogi_engine::search::performance_tuning::BenchmarkAggregator;

let aggregator = BenchmarkAggregator::new();
let reports = aggregator.aggregate_criterion_results("target/criterion")?;
let aggregated = aggregator.generate_benchmark_report(&reports);
aggregator.export_report_to_json(&aggregated, "benchmark_report.json")?;
aggregator.export_report_to_markdown(&aggregated, "benchmark_report.md")?;
```

### With Baseline Comparison

```rust
use shogi_engine::search::performance_tuning::BenchmarkAggregator;

let mut aggregator = BenchmarkAggregator::new();
aggregator.set_baseline_path("docs/performance/baselines/latest.json");

let reports = aggregator.aggregate_criterion_results("target/criterion")?;
let aggregated = aggregator.generate_benchmark_report(&reports);

// Reports will include baseline comparisons
aggregator.export_report_to_json(&aggregated, "benchmark_report.json")?;
```

### Using Environment Variables

```bash
# Set baseline path via environment variable
export BENCHMARK_BASELINE_PATH=docs/performance/baselines/latest.json

# Run aggregation script
./scripts/aggregate_benchmark_results.sh
```

## Report Format

### JSON Report Structure

```json
{
  "timestamp": "2024-12-01T00:00:00Z",
  "git_commit": "abc123...",
  "hardware": {
    "cpu": "Apple M1",
    "cores": 8,
    "ram_gb": 16
  },
  "benchmarks": [
    {
      "benchmark_name": "search_benchmark",
      "mean_time_ns": 1000000.0,
      "std_dev_ns": 50000.0,
      "throughput_ops_per_sec": 1000.0,
      "samples": 100,
      "baseline_comparison": {
        "has_regression": false,
        "change_percent": 2.5,
        "baseline_value": 975000.0,
        "current_value": 1000000.0
      }
    }
  ],
  "summary": {
    "total_benchmarks": 1,
    "average_mean_time_ns": 1000000.0,
    "total_throughput_ops_per_sec": 1000.0,
    "regressions_detected": 0
  }
}
```

### Markdown Report

The Markdown report includes:
- Header with timestamp, git commit, and hardware info
- Summary statistics
- Detailed benchmark table with regression indicators

## Criterion.rs Integration

The aggregator parses Criterion.rs output from the standard directory structure:

```
target/criterion/
  {benchmark_name}/
    {id}/
      base/
        estimates.json  # Parsed for metrics
```

The `estimates.json` file contains:
- `mean.point_estimate`: Mean time in nanoseconds
- `mean.standard_error`: Standard error
- `throughput.per_second.point_estimate`: Throughput in ops/sec

## Regression Detection

Regressions are detected when:
- Current mean time is >5% higher than baseline (configurable threshold)
- Comparison is performed if baseline path is provided

To customize the threshold:

```rust
let mut aggregator = BenchmarkAggregator::new();
aggregator.set_regression_threshold(10.0); // 10% threshold
```

## Script Usage

The `scripts/aggregate_benchmark_results.sh` script provides a convenient way to:
1. Run benchmarks (if needed)
2. Aggregate results
3. Generate reports

```bash
# Basic usage
./scripts/aggregate_benchmark_results.sh

# With baseline comparison
BENCHMARK_BASELINE_PATH=docs/performance/baselines/latest.json \
  ./scripts/aggregate_benchmark_results.sh
```

## Best Practices

1. **Regular Aggregation**: Run aggregation after major changes to track performance trends
2. **Baseline Updates**: Update baseline when performance improves significantly
3. **CI Integration**: Use in CI to detect performance regressions automatically
4. **Version Control**: Commit reports to track performance history
5. **Hardware Consistency**: Compare results on similar hardware for meaningful comparisons

## Limitations

- **Simplified Baseline Comparison**: Currently compares all benchmarks against a single baseline metric (nodes_per_second). Future enhancements could map specific benchmarks to specific baseline metrics.
- **Sample Count**: Sample count is estimated (default: 100) as it's not directly available in Criterion.rs estimates.json
- **Benchmark Name Extraction**: Relies on directory structure; may need adjustment for complex benchmark naming

## Future Enhancements

- Benchmark-to-baseline metric mapping
- Historical trend analysis
- Performance visualization
- Automated regression alerts
- Integration with CI/CD pipelines

