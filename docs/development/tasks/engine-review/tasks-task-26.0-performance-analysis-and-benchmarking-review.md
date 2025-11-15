# Tasks: Performance Analysis and Benchmarking Improvements

**Parent PRD:** `task-26.0-performance-analysis-and-benchmarking-review.md`  
**Date:** December 2024  
**Status:** In Progress

---

## Overview

This task list implements the performance analysis and benchmarking improvements identified in the Performance Analysis and Benchmarking Review (Task 26.0). The improvements enhance observability, automate baseline management, integrate profiling into hot paths, and enable CI-based regression detection.

## Relevant Files

- `src/search/search_engine.rs` - Main search implementation with `get_performance_metrics()`, `calculate_nodes_per_second()`
- `src/evaluation/performance.rs` - `PerformanceProfiler` for evaluation timing and hot path analysis
- `src/search/move_ordering.rs` - `MemoryTracker` for memory usage breakdown
- `src/search/thread_safe_table.rs` - Transposition table statistics and hit rate tracking
- `src/search/parallel_search.rs` - Parallel search metrics and work distribution tracking
- `src/types.rs` - Performance metric structs (`PerformanceMetrics`, `IIDPerformanceMetrics`, `MoveOrderingEffectivenessStats`)
- `benches/` - 78 benchmark files covering all performance aspects
- `src/bin/profiler.rs` - Standalone profiling tool
- `src/search/performance_tuning.rs` - Performance optimization utilities
- `docs/performance/baselines/` - Performance baseline storage directory (to be created)
- `resources/benchmark_positions/` - Standard benchmark position set (to be created)
- `scripts/run_performance_baseline.sh` - Baseline generation script (to be created)
- `.github/workflows/performance-regression.yml` - CI regression detection workflow (to be created)

### Notes

- Unit tests should be placed alongside the code files they are testing
- Integration tests go in the `tests/` directory
- Benchmarks go in the `benches/` directory
- Baseline files should be JSON format and versioned by git commit
- Use `cargo test` to run tests, `cargo bench` to run benchmarks

---

## Tasks

- [x] 1.0 Performance Baseline Persistence and Comparison Framework (Medium Priority - Est: 4-6 hours)
  - [x] 1.1 Create `PerformanceBaseline` struct in `src/types.rs` matching PRD Section 12.1 JSON format (timestamp, hardware, search_metrics, evaluation_metrics, tt_metrics, move_ordering_metrics, parallel_search_metrics, memory_metrics)
  - [x] 1.2 Create `BaselineManager` struct in `src/search/performance_tuning.rs` with methods: `new()`, `load_baseline()`, `save_baseline()`, `compare_with_baseline()`
  - [x] 1.3 Implement `collect_baseline_metrics()` method in `SearchEngine` to gather all metrics from `get_performance_metrics()`, transposition table stats, move ordering stats, parallel search stats
  - [x] 1.4 Implement hardware detection: CPU model, core count, RAM size (use `std::env` and system info where available, fallback to "Unknown")
  - [x] 1.5 Implement `save_baseline_to_file()` method that exports baseline to JSON in `docs/performance/baselines/` directory
  - [x] 1.6 Implement `load_baseline_from_file()` method that reads JSON baseline file
  - [x] 1.7 Implement `compare_baselines()` method that calculates percentage differences for each metric category
  - [x] 1.8 Implement `detect_regression()` method that flags metrics degrading >5% (configurable threshold)
  - [x] 1.9 Add git commit hash to baseline metadata (use `git rev-parse HEAD` or environment variable)
  - [x] 1.10 Create `scripts/run_performance_baseline.sh` script that runs benchmark suite and saves baseline
  - [x] 1.11 Write unit test `test_baseline_serialization` to verify JSON round-trip (save/load)
  - [x] 1.12 Write unit test `test_baseline_comparison` to verify comparison logic calculates differences correctly
  - [x] 1.13 Write integration test `test_baseline_regression_detection` to verify regression detection flags >5% degradation
  - [x] 1.14 Add documentation for baseline format and usage in `docs/performance/baselines/README.md`

- [ ] 2.0 Benchmark Result Aggregation and Reporting (Medium Priority - Est: 4-6 hours)
  - [ ] 2.1 Create `BenchmarkAggregator` struct in `src/search/performance_tuning.rs` to collect results from multiple benchmark runs
  - [ ] 2.2 Implement `aggregate_criterion_results()` method that parses Criterion.rs JSON output from `target/criterion/`
  - [ ] 2.3 Implement `generate_benchmark_report()` method that creates summary report with: benchmark name, mean time, std deviation, throughput, comparison vs baseline
  - [ ] 2.4 Create `BenchmarkReport` struct with fields: benchmark_name, mean_time_ns, std_dev_ns, throughput_ops_per_sec, samples, baseline_comparison
  - [ ] 2.5 Implement `export_report_to_json()` method that saves aggregated report to `docs/performance/reports/`
  - [ ] 2.6 Implement `export_report_to_markdown()` method that generates human-readable markdown report
  - [ ] 2.7 Add `compare_with_baseline()` method to `BenchmarkReport` that loads baseline and calculates percentage change
  - [ ] 2.8 Create `scripts/aggregate_benchmark_results.sh` script that runs all benchmarks and generates aggregated report
  - [ ] 2.9 Add environment variable `BENCHMARK_BASELINE_PATH` to specify baseline file for comparison
  - [ ] 2.10 Write unit test `test_benchmark_aggregation` to verify aggregator collects results correctly
  - [ ] 2.11 Write unit test `test_report_generation` to verify report format is correct
  - [ ] 2.12 Write integration test `test_full_benchmark_pipeline` that runs sample benchmark and verifies aggregation works
  - [ ] 2.13 Add documentation for benchmark aggregation workflow in `docs/performance/reports/README.md`

- [ ] 3.0 Automatic Profiling Integration for Hot Paths (Medium Priority - Est: 6-8 hours)
  - [ ] 3.1 Add `auto_profiling_enabled: bool` field to `SearchEngine` configuration (default: false)
  - [ ] 3.2 Modify `PerformanceProfiler` in `src/evaluation/performance.rs` to support automatic enable/disable based on configuration
  - [ ] 3.3 Add `enable_auto_profiling()` method to `SearchEngine` that enables profiling for hot paths (evaluation, move ordering, TT operations)
  - [ ] 3.4 Integrate automatic profiling into `evaluate_position()` method: enable profiler if `auto_profiling_enabled` is true
  - [ ] 3.5 Integrate automatic profiling into `order_moves_for_negamax()` method: record ordering time if enabled
  - [ ] 3.6 Integrate automatic profiling into transposition table probe/store operations: record TT operation times
  - [ ] 3.7 Add `get_hot_path_summary()` method to `PerformanceProfiler` that returns top N slowest operations
  - [ ] 3.8 Implement automatic profiler sampling: only profile every Nth call to reduce overhead (configurable sample rate)
  - [ ] 3.9 Add `export_profiling_data()` method that saves profiling results to JSON for analysis
  - [ ] 3.10 Add configuration option `auto_profiling_sample_rate: u32` (default: 100, profile every 100th call)
  - [ ] 3.11 Add `profiling_overhead_tracking` to measure impact of profiling on performance
  - [ ] 3.12 Write unit test `test_auto_profiling_enable` to verify profiling activates when enabled
  - [ ] 3.13 Write unit test `test_profiling_sample_rate` to verify sampling reduces overhead
  - [ ] 3.14 Write integration test `test_hot_path_identification` that runs search and verifies hot paths are identified
  - [ ] 3.15 Add documentation for automatic profiling configuration and usage

- [ ] 4.0 Actual Memory Usage Tracking (RSS) (Low Priority - Est: 4-6 hours)
  - [ ] 4.1 Add `sysinfo` crate dependency to `Cargo.toml` for cross-platform memory tracking
  - [ ] 4.2 Create `MemoryTracker` struct in `src/search/memory_tracking.rs` with methods: `new()`, `get_current_rss()`, `get_peak_rss()`, `get_memory_breakdown()`
  - [ ] 4.3 Implement `get_current_rss()` using `sysinfo::System` to get actual process memory (RSS) on Linux/macOS/Windows
  - [ ] 4.4 Implement `get_peak_rss()` that tracks maximum RSS during search
  - [ ] 4.5 Replace placeholder `get_memory_usage()` in `SearchEngine` (line 2616) to call `MemoryTracker::get_current_rss()`
  - [ ] 4.6 Update `track_memory_usage()` method in `SearchEngine` (line 2623) to actually track RSS snapshots
  - [ ] 4.7 Add `memory_tracker: MemoryTracker` field to `SearchEngine` struct
  - [ ] 4.8 Integrate memory tracking into search: take snapshot at search start, end, and periodically during long searches
  - [ ] 4.9 Add `get_memory_breakdown()` method that combines RSS tracking with component-level estimates (TT, caches, etc.)
  - [ ] 4.10 Add memory tracking statistics to `PerformanceMetrics` struct: `current_rss_bytes`, `peak_rss_bytes`, `memory_growth_bytes`
  - [ ] 4.11 Add memory leak detection: alert if memory grows >50% during single search (configurable threshold)
  - [ ] 4.12 Write unit test `test_memory_tracking` to verify RSS is retrieved correctly (may need platform-specific mocks)
  - [ ] 4.13 Write integration test `test_memory_growth_tracking` that runs search and verifies memory tracking works
  - [ ] 4.14 Add documentation for memory tracking capabilities and limitations

- [ ] 5.0 Standard Benchmark Position Set and Automated Regression Suite (Medium Priority - Est: 6-8 hours)
  - [ ] 5.1 Create `resources/benchmark_positions/` directory
  - [ ] 5.2 Create `BenchmarkPosition` struct in `src/types.rs` with fields: `name: String`, `fen: String`, `position_type: PositionType`, `expected_depth: u8`, `description: String`
  - [ ] 5.3 Create `PositionType` enum: `Opening`, `MiddlegameTactical`, `MiddlegamePositional`, `EndgameKingActivity`, `EndgameZugzwang`
  - [ ] 5.4 Create `standard_positions.json` file in `resources/benchmark_positions/` with 5 standard positions from PRD Section 12.3
  - [ ] 5.5 Implement `load_standard_positions()` function that reads `standard_positions.json` and returns `Vec<BenchmarkPosition>`
  - [ ] 5.6 Create `BenchmarkRunner` struct in `src/search/performance_tuning.rs` that runs benchmarks on standard positions
  - [ ] 5.7 Implement `run_position_benchmark()` method that searches each standard position and collects metrics
  - [ ] 5.8 Implement `run_regression_suite()` method that runs all standard positions and compares against baseline
  - [ ] 5.9 Create `RegressionTestResult` struct with fields: `position_name`, `baseline_time_ms`, `current_time_ms`, `regression_detected: bool`, `regression_percentage`
  - [ ] 5.10 Implement `detect_regressions()` method that flags positions with >5% time increase (configurable threshold)
  - [ ] 5.11 Create `scripts/run_regression_suite.sh` script that runs standard positions and generates regression report
  - [ ] 5.12 Add `--regression-test` flag to benchmark runner that fails if any regression detected
  - [ ] 5.13 Write unit test `test_standard_positions_loading` to verify positions load correctly from JSON
  - [ ] 5.14 Write integration test `test_regression_suite_execution` that runs regression suite and verifies detection works
  - [ ] 5.15 Add documentation for standard positions and regression suite usage

- [ ] 6.0 CI Integration for Performance Regression Detection (Medium Priority - Est: 4-6 hours)
  - [ ] 6.1 Create `.github/workflows/performance-regression.yml` workflow file
  - [ ] 6.2 Configure workflow to run on: pull requests, pushes to master, scheduled daily runs
  - [ ] 6.3 Add workflow step to run benchmark suite: `cargo bench --all -- --output-format json`
  - [ ] 6.4 Add workflow step to load baseline from `docs/performance/baselines/latest.json` (or create if missing)
  - [ ] 6.5 Add workflow step to compare current results with baseline using `BaselineManager::compare_baselines()`
  - [ ] 6.6 Add workflow step to run regression suite: `scripts/run_regression_suite.sh --regression-test`
  - [ ] 6.7 Configure workflow to fail if regression detected (>5% degradation in any metric)
  - [ ] 6.8 Add workflow step to upload benchmark results as artifact for analysis
  - [ ] 6.9 Add workflow step to comment on PR with performance comparison if regression detected
  - [ ] 6.10 Add environment variable `PERFORMANCE_REGRESSION_THRESHOLD` (default: 5.0) for configurable threshold
  - [ ] 6.11 Add workflow step to update baseline if on master branch and no regressions detected
  - [ ] 6.12 Create `scripts/ci_performance_check.sh` helper script that CI workflow calls
  - [ ] 6.13 Write test to verify CI workflow logic (may need to mock GitHub Actions environment)
  - [ ] 6.14 Add documentation for CI performance regression workflow in `.github/workflows/README.md`
  - [ ] 6.15 Test workflow locally using `act` or similar tool to verify it works correctly

- [ ] 7.0 Telemetry Export and Advanced Metrics Analysis (Low Priority - Est: 4-6 hours)
  - [ ] 7.1 Create `TelemetryExporter` struct in `src/search/performance_tuning.rs` with methods: `export_to_json()`, `export_to_csv()`, `export_to_markdown()`
  - [ ] 7.2 Implement `export_performance_metrics_to_json()` that exports all `PerformanceMetrics` to JSON file
  - [ ] 7.3 Implement `export_performance_metrics_to_csv()` that exports metrics to CSV format for spreadsheet analysis
  - [ ] 7.4 Add `export_efficiency_metrics()` method that exports IID and LMR efficiency metrics (PRD Section 3.4 gap)
  - [ ] 7.5 Add `export_tt_entry_quality_distribution()` method that exports entry quality distribution (Exact/Beta/Alpha percentages) (PRD Section 5.2 gap)
  - [ ] 7.6 Add `export_hit_rate_by_depth()` method that exports transposition table hit rates stratified by depth (PRD Section 5.3 gap)
  - [ ] 7.7 Add `export_scalability_metrics()` method that exports parallel search scalability metrics for regression analysis (PRD Section 7.3 gap)
  - [ ] 7.8 Add `export_cache_effectiveness()` method that exports cache hit rates and size monitoring (PRD Section 4.2 gap)
  - [ ] 7.9 Create `scripts/export_telemetry.sh` script that runs search and exports all telemetry data
  - [ ] 7.10 Add configuration option `telemetry_export_enabled: bool` (default: false) to enable automatic export
  - [ ] 7.11 Add `telemetry_export_path: String` configuration option to specify export directory
  - [ ] 7.12 Write unit test `test_telemetry_json_export` to verify JSON export format is correct
  - [ ] 7.13 Write unit test `test_telemetry_csv_export` to verify CSV export format is correct
  - [ ] 7.14 Write integration test `test_telemetry_export_pipeline` that runs search and verifies all exports work
  - [ ] 7.15 Add documentation for telemetry export formats and usage

- [ ] 8.0 External Profiler Integration and Hot Path Analysis (Low Priority - Est: 6-8 hours)
  - [ ] 8.1 Create `ExternalProfiler` trait in `src/search/performance_tuning.rs` for integration with perf/Instruments (PRD Section 10.3 gap)
  - [ ] 8.2 Implement `PerfProfiler` struct that generates perf-compatible output (Linux)
  - [ ] 8.3 Implement `InstrumentsProfiler` struct that generates Instruments-compatible output (macOS)
  - [ ] 8.4 Add `enable_external_profiling()` method to `SearchEngine` that sets up external profiler hooks
  - [ ] 8.5 Add profiler markers/annotations to hot paths: evaluation, move ordering, TT operations, parallel search
  - [ ] 8.6 Create `scripts/run_with_perf.sh` script that runs search with perf profiling (Linux)
  - [ ] 8.7 Create `scripts/run_with_instruments.sh` script that runs search with Instruments profiling (macOS)
  - [ ] 8.8 Add `export_profiling_markers()` method that exports profiler markers to JSON for analysis
  - [ ] 8.9 Add documentation for external profiler integration and usage
  - [ ] 8.10 Write integration test `test_external_profiler_markers` that verifies markers are placed correctly

---

**Phase 2 Complete - Detailed Sub-Tasks Generated**

All parent tasks have been broken down into **105 actionable sub-tasks** (updated from 75). Each sub-task is specific, testable, and includes:
- Implementation details based on the PRD analysis
- Testing requirements (unit tests, integration tests)
- Configuration options where applicable
- Documentation updates
- Cross-references to specific sections in the PRD document

**Coverage Verification:**

✅ **Section 12 (Performance Baseline Metrics):**
- 12.1 Baseline Metrics Structure → Task 1.0 (all sub-tasks)
- 12.2 Baseline Persistence → Task 1.0 (sub-tasks 1.5-1.6, 1.9)
- 12.3 Standard Benchmark Positions → Task 5.0 (all sub-tasks)

✅ **Section 14 (Improvement Recommendations):**
- Performance baseline persistence → Task 1.0 (Medium Priority)
- Benchmark result aggregation → Task 2.0 (Medium Priority)
- Automatic profiling integration → Task 3.0 (Medium Priority)
- Actual memory usage tracking → Task 4.0 (Low Priority)
- CI regression detection → Task 6.0 (Medium Priority)
- Telemetry export (JSON/CSV) → Task 7.0 (Low Priority)
- External profiler integration → Task 8.0 (Low Priority)

✅ **Section 15 (Testing & Validation Plan):**
- Baseline establishment → Task 1.0, 5.0
- Regression detection → Task 1.0, 5.0, 6.0
- Hot path profiling → Task 3.0, 8.0
- Memory profiling → Task 4.0
- Scalability testing → Task 7.0 (scalability metrics export)

✅ **Additional Gaps Addressed:**
- Efficiency metrics export (Section 3.4) → Task 7.0 (sub-task 7.4)
- Entry quality distribution export (Section 5.2) → Task 7.0 (sub-task 7.5)
- Hit rate by depth analysis (Section 5.3) → Task 7.0 (sub-task 7.6)
- Scalability metrics persistence (Section 7.3) → Task 7.0 (sub-task 7.7)
- Cache effectiveness monitoring (Section 4.2) → Task 7.0 (sub-task 7.8)
- External profiler integration (Section 10.3) → Task 8.0 (all sub-tasks)

**Task Priorities:**
- **Phase 1 (Immediate, 1-2 weeks):** Tasks 1.0, 2.0, 5.0 - Baseline and automation infrastructure
- **Phase 2 (Short-term, 2-3 weeks):** Tasks 3.0, 6.0 - Profiling integration and CI
- **Phase 3 (Long-term, 1 month):** Tasks 4.0, 7.0, 8.0 - Memory tracking, telemetry export, external profiler integration

**Expected Cumulative Benefits:**
- **Observability:** Automated baseline comparison, regression detection, hot path identification, comprehensive telemetry export
- **Automation:** CI integration prevents performance regressions from reaching production
- **Reproducibility:** Standard position set enables consistent performance comparisons
- **Monitoring:** Actual RSS tracking replaces placeholder, enables memory leak detection
- **Developer Experience:** Automatic profiling identifies bottlenecks without manual instrumentation
- **Analysis:** Telemetry export enables post-processing, visualization, and trend analysis
- **Integration:** External profiler support enables deep performance analysis with industry-standard tools

---

### Task 1.0 Completion Notes

- **Implementation**: Created `PerformanceBaseline` struct in `src/types.rs` matching PRD Section 12.1 JSON format with all required metric categories (search, evaluation, TT, move ordering, parallel search, memory). Implemented `BaselineManager` in `src/search/performance_tuning.rs` with save/load, comparison, and regression detection methods. Added `collect_baseline_metrics()` method to `SearchEngine` that gathers metrics from all subsystems. Implemented hardware detection using environment variables and platform-specific commands (Linux `/proc/cpuinfo`, macOS `sysctl`). Added git commit hash tracking via `get_git_commit_hash()` function that checks environment variable or git command.

- **Configuration**: Baseline directory defaults to `docs/performance/baselines/` (configurable via `BaselineManager::with_directory()`). Regression threshold defaults to 5.0% (configurable via `set_regression_threshold()`). Baseline files are JSON format with ISO 8601 timestamps and git commit hashes for version tracking.

- **Testing**: Added comprehensive test suite in `tests/performance_baseline_tests.rs` covering: baseline serialization round-trip (`test_baseline_serialization`), comparison logic verification (`test_baseline_comparison`), regression detection with >5% threshold (`test_baseline_regression_detection`), no false positives for improvements (`test_baseline_no_regression`), directory creation (`test_baseline_directory_creation`), git commit hash inclusion (`test_baseline_git_commit_hash`), hardware info detection (`test_baseline_hardware_info`). All tests pass.

- **Scripts**: Created `scripts/run_performance_baseline.sh` script that runs benchmarks and provides instructions for baseline collection. Script handles directory creation, git commit hash extraction, and timestamp generation.

- **Documentation**: Added comprehensive documentation in `docs/performance/baselines/README.md` covering: baseline format specification, usage examples (create, load, compare, detect regressions), regression threshold configuration, file naming conventions, CI integration guidance, best practices, limitations, and future enhancements.

- **Known Limitations**: Evaluation metrics (average_evaluation_time_ns, phase_calc_time_ns) are currently placeholders (TODO comments) and require evaluator interface enhancements. Parallel search metrics default to 0.0 if parallel search is not used. Memory metrics are estimates based on data structure sizes, not actual RSS (will be addressed in Task 4.0).

- **Follow-ups**: Consider enhancing evaluator interface to expose evaluation timing and cache statistics. Consider adding parallel search metrics collection when parallel search is enabled. Task 4.0 will replace memory estimates with actual RSS tracking.

