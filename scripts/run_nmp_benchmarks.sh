#!/bin/bash
# Benchmark execution script for Null Move Pruning performance monitoring
#
# This script runs NMP benchmarks and generates performance reports.
# Can be used for CI/CD automation or local performance monitoring.

set -e

# Configuration
METRICS_DIR="${NMP_METRICS_DIR:-target/nmp_metrics}"
BENCHMARK_RESULTS_DIR="${BENCHMARK_RESULTS_DIR:-target/criterion}"
REGRESSION_TEST="${NMP_REGRESSION_TEST:-}"

# Create metrics directory if it doesn't exist
mkdir -p "$METRICS_DIR"

echo "Running Null Move Pruning performance benchmarks..."
echo "Metrics directory: $METRICS_DIR"
echo "Benchmark results: $BENCHMARK_RESULTS_DIR"

# Run the performance monitoring benchmarks
if [ -n "$REGRESSION_TEST" ]; then
    echo "Running with regression testing enabled..."
    NMP_REGRESSION_TEST=1 cargo bench --bench null_move_performance_monitoring_benchmarks
else
    cargo bench --bench null_move_performance_monitoring_benchmarks
fi

# Also run regression tests
echo "Running regression tests..."
cargo test --test null_move_regression_tests

echo "Benchmarks completed successfully!"
echo "Metrics saved to: $METRICS_DIR/nmp_metrics.json"
echo "Criterion reports available in: $BENCHMARK_RESULTS_DIR"

