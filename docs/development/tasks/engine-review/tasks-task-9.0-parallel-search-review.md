## Relevant Files

- `src/search/parallel_search.rs` - Core parallel search engine, YBWC coordination, and work-stealing queues.
- `src/search/thread_safe_table.rs` - Shared table wrapper used by parallel workers; needs contention improvements.
- `src/search/hierarchical_transposition_table.rs` - Optional hierarchical TT layer to ensure compatibility post-optimization.
- `src/search/search_engine.rs` - Entry points configuring TT access and YBWC parameters.
- `src/search/runtime_configuration.rs` - Engine configuration surface for exposing parallel search options.
- `tests/parallel_search_tests.rs` - Unit coverage for queue behavior, synchronization, and stats reporting.
- `tests/parallel_search_integration_tests.rs` - Integration coverage for multi-thread coordination and USI options.
- `benches/parallel_search_performance_benchmarks.rs` - Criterion harness for scaling and synchronization regression tracking.

### Notes

- Reuse bucketed-lock groundwork from Task 8.0; avoid regressions in thread-safe TT behavior.
- Validate with `cargo test --test parallel_search_tests -- --test-threads=16` and targeted integration runs.
- Capture before/after synchronization metrics via existing `PARALLEL_PROF` debug logs and Criterion benches.
- Ensure new configuration knobs remain compatible with USI option exposure and default engine presets.

## Tasks

- [ ] 1.0 Replace YBWC busy-wait synchronization with event-driven signaling
  - [ ] 1.1 Audit `YBWCSync`/`wait_for_complete` to document current spin/yield behavior and contention scenarios.
  - [ ] 1.2 Select signaling primitive (e.g., `Condvar`, `parking_lot::Condvar`, or async channel) that preserves timeout semantics.
  - [ ] 1.3 Refactor oldest-brother completion path to notify waiting siblings without spinning; ensure poisoned-lock recovery remains intact.
  - [ ] 1.4 Update waiting logic to respect global stop flags and propagate timeout/abort conditions deterministically.
  - [ ] 1.5 Extend unit tests to cover concurrent wait/notify flows and regression-test starvation scenarios.
  - [ ] 1.6 Stress-test under asymmetric move trees (deep PV vs. reduced siblings) to confirm CPU utilization drops and no deadlocks occur.
- [ ] 2.0 Modernize work queue implementation to reduce locking contention
  - [ ] 2.1 Evaluate replacing `Mutex<VecDeque>` queues with lock-free alternatives (`crossbeam_deque`, segmented locks) while preserving statistics hooks.
  - [ ] 2.2 Implement new queue abstraction and migrate push/pop/steal paths, including poison recovery and debug logging.
  - [ ] 2.3 Update worker scheduling to prefer local queues and minimize cross-thread stealing latency under high load.
  - [ ] 2.4 Adapt instrumentation to new queue internals, ensuring lock-wait timing and recovery counters remain meaningful.
  - [ ] 2.5 Refresh unit tests validating queue ordering, stealing fairness, and poison recovery across multiple threads.
  - [ ] 2.6 Benchmark parallel search at 4/8/16 threads to verify reduced lock wait times and document improvements.
- [ ] 3.0 Make work distribution metrics contention-free and optional
  - [ ] 3.1 Identify all call sites mutating `work_stats` mutex and map required metrics (per-thread work, steals, totals).
  - [ ] 3.2 Introduce per-thread atomics or thread-local buffers aggregated post-search to remove the global mutex hotspot.
  - [ ] 3.3 Add configuration flag to enable/disable metrics collection, defaulting to off for latency-sensitive builds.
  - [ ] 3.4 Provide aggregation/reporting utilities that operate without locks when metrics are disabled.
  - [ ] 3.5 Update tests to cover enabled/disabled metric paths and ensure vector lengths match thread counts.
  - [ ] 3.6 Document the runtime cost of metrics and recommended settings in developer docs.
- [ ] 4.0 Streamline shared transposition table access for parallel workers
  - [ ] 4.1 Profile current shared TT usage (read/write ratios, bucket contention) in the parallel engine using existing debug hooks.
  - [ ] 4.2 Integrate bucketed-lock API from Task 8.0 by reusing per-hash bucket selection in parallel store paths.
  - [ ] 4.3 Add batched or deferred write paths to minimize lock acquisitions when flushing worker-local TT updates.
  - [ ] 4.4 Ensure compatibility with hierarchical TT feature (`hierarchical-tt`) by adapting promotion/demotion paths for parallel callers.
  - [ ] 4.5 Extend unit/integration tests to cover concurrent TT writes and verify no regressions in PV reconstruction.
  - [ ] 4.6 Update benches to capture TT lock wait metrics before/after changes and summarize in completion notes.
- [ ] 5.0 Expose parallel search configuration knobs and defaults
  - [ ] 5.1 Extend `ParallelSearchConfig` to surface YBWC thresholds, hash size, and statistics toggles with sensible defaults.
  - [ ] 5.2 Wire new config fields through engine builders (`SearchEngine`, `ShogiEngine`) and USI option exposure.
  - [ ] 5.3 Ensure configuration overrides propagate into worker contexts without redundant allocations or cloning.
  - [ ] 5.4 Update documentation (developer guides and USI option reference) to describe new tunables and recommended presets.
  - [ ] 5.5 Add integration tests asserting configuration changes affect runtime behavior (e.g., disabling metrics, adjusting hash size).
- [ ] 6.0 Optimize root position cloning overhead in parallel workers
  - [ ] 6.1 Analyze current root board cloning flow and quantify per-thread allocation costs under typical search workloads.
  - [ ] 6.2 Prototype shared immutable board state or incremental move application that keeps worker contexts consistent without redundant cloning.
  - [ ] 6.3 Validate thread-safety (no interior mutability) when sharing immutable state across workers; fall back to cloning where needed.
  - [ ] 6.4 Measure performance impact in benchmarks to confirm reduction in clone overhead and document trade-offs.
  - [ ] 6.5 Update tests to ensure shared state does not leak mutations between workers and that fallback cloning still works.

