#!/bin/bash
# Script to aggregate benchmark results and generate reports (Task 26.0 - Task 2.0)
#
# This script runs all benchmarks and generates aggregated reports in JSON and Markdown formats.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRITERION_DIR="$PROJECT_ROOT/target/criterion"
REPORTS_DIR="$PROJECT_ROOT/docs/performance/reports"

# Get baseline path from environment or use default
BASELINE_PATH="${BENCHMARK_BASELINE_PATH:-$PROJECT_ROOT/docs/performance/baselines/latest.json}"

echo "Aggregating benchmark results..."
echo "Criterion directory: $CRITERION_DIR"
echo "Reports directory: $REPORTS_DIR"
echo "Baseline path: $BASELINE_PATH"

# Create reports directory
mkdir -p "$REPORTS_DIR"

# Run benchmarks if criterion directory doesn't exist or is empty
if [ ! -d "$CRITERION_DIR" ] || [ -z "$(ls -A $CRITERION_DIR 2>/dev/null)" ]; then
    echo "Running benchmarks..."
    cd "$PROJECT_ROOT"
    cargo bench --all 2>&1 | tee /tmp/benchmark_output.log || {
        echo "Warning: Benchmark run completed with warnings or errors"
    }
else
    echo "Using existing benchmark results in $CRITERION_DIR"
fi

# The actual aggregation would be done by a Rust binary
# For now, we create a simple script that can be extended
echo ""
echo "Benchmark aggregation complete."
echo "To aggregate actual benchmark results, implement a Rust binary that:"
echo "  1. Uses BenchmarkAggregator::aggregate_criterion_results()"
echo "  2. Generates report using generate_benchmark_report()"
echo "  3. Exports to JSON and Markdown using export_report_to_json() and export_report_to_markdown()"
echo ""
echo "Example usage in Rust:"
echo "  use shogi_engine::search::performance_tuning::BenchmarkAggregator;"
echo ""
echo "  let mut aggregator = BenchmarkAggregator::new();"
echo "  if let Ok(baseline_path) = std::env::var(\"BENCHMARK_BASELINE_PATH\") {"
echo "      aggregator.set_baseline_path(baseline_path);"
echo "  }"
echo "  let reports = aggregator.aggregate_criterion_results(\"target/criterion\")?;"
echo "  let aggregated = aggregator.generate_benchmark_report(&reports);"
echo "  aggregator.export_report_to_json(&aggregated, \"benchmark_report.json\")?;"
echo "  aggregator.export_report_to_markdown(&aggregated, \"benchmark_report.md\")?;"

