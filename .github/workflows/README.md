# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated testing, benchmarking, and CI/CD.

## Workflows

### Performance Regression Detection (`performance-regression.yml`)

**Purpose**: Automatically detect performance regressions in the engine by comparing current performance metrics against a baseline.

**Triggers**:
- Push to `master` or `main` branch (when performance-related files change)
- Pull requests to `master` or `main` branch
- Scheduled daily runs (3 AM UTC)
- Manual dispatch

**What it does**:
1. Runs the full benchmark suite using `cargo bench --all`
2. Collects performance baseline metrics from the current build
3. Loads the baseline from `docs/performance/baselines/latest.json` (or creates if missing)
4. Compares current results with baseline using `BaselineManager::compare_baselines()`
5. Runs regression suite on standard positions using `scripts/run_regression_suite.sh`
6. Detects regressions (>5% degradation in any metric, configurable threshold)
7. Uploads benchmark results as artifacts
8. Comments on PR with performance comparison if regression detected
9. Updates baseline on master branch if no regressions detected

**Configuration**:
- `PERFORMANCE_REGRESSION_THRESHOLD`: Regression threshold percentage (default: 5.0)
  - Can be set as a GitHub repository variable or environment variable
- `BASELINE_PATH`: Path to baseline file (default: `docs/performance/baselines/latest.json`)

**Helper Scripts**:
- `scripts/ci_performance_check.sh`: Helper script for baseline collection, comparison, and updates
- `scripts/run_regression_suite.sh`: Runs regression suite on standard positions

**Artifacts**:
- Benchmark results (JSON files from Criterion)
- Performance comparison results
- Regression suite results

**Failure Conditions**:
- Regression detected in baseline comparison (>5% degradation)
- Regression detected in regression suite (>5% time increase on any position)

**Baseline Updates**:
- Baseline is automatically updated on master/main branch when:
  - No regressions are detected
  - All benchmarks complete successfully
  - Comparison shows performance is stable or improved

## Other Workflows

### Castle Pattern Tests (`castle-pattern-tests.yml`)
Runs castle pattern recognition regression tests and benchmarks.

### NMP Performance Benchmarks (`nmp-performance-benchmarks.yml`)
Runs Null Move Pruning performance benchmarks and regression tests.

## Local Testing

To test workflows locally, you can use [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run performance regression workflow
act -W .github/workflows/performance-regression.yml

# Run with specific event
act pull_request -W .github/workflows/performance-regression.yml
```

## Troubleshooting

### Baseline file not found
- First run: The baseline will be created automatically after benchmarks complete
- Subsequent runs: Ensure `docs/performance/baselines/latest.json` exists in the repository

### Regression detected unexpectedly
- Check system load and resource availability
- Verify baseline was created on similar hardware
- Review threshold setting (may need adjustment for your hardware)

### Workflow fails on baseline update
- Ensure workflow has write permissions
- Check that baseline directory exists and is writable
- Verify no regressions were detected (baseline won't update if regressions exist)

## Best Practices

1. **Establish Baseline**: Run workflow on a known good commit to establish initial baseline
2. **Monitor Trends**: Review benchmark artifacts regularly to track performance trends
3. **Adjust Threshold**: Tune `PERFORMANCE_REGRESSION_THRESHOLD` based on normal variance
4. **Review PR Comments**: Check PR comments for detailed regression information
5. **Investigate Regressions**: Don't ignore regressions - investigate and fix performance issues

## Related Documentation

- [Performance Baselines](../docs/performance/baselines/README.md)
- [Benchmark Positions](../docs/performance/benchmark_positions.md)
- [Performance Analysis PRD](../docs/development/tasks/engine-review/task-26.0-performance-analysis-and-benchmarking-review.md)

