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

- [x] 1.0 Replace YBWC busy-wait synchronization with event-driven signaling
  - [x] 1.1 Audit `YBWCSync`/`wait_for_complete` to document current spin/yield behavior and contention scenarios.
  - [x] 1.2 Select signaling primitive (e.g., `Condvar`, `parking_lot::Condvar`, or async channel) that preserves timeout semantics.
  - [x] 1.3 Refactor oldest-brother completion path to notify waiting siblings without spinning; ensure poisoned-lock recovery remains intact.
  - [x] 1.4 Update waiting logic to respect global stop flags and propagate timeout/abort conditions deterministically.
  - [x] 1.5 Extend unit tests to cover concurrent wait/notify flows and regression-test starvation scenarios.
  - [x] 1.6 Stress-test under asymmetric move trees (deep PV vs. reduced siblings) to confirm CPU utilization drops and no deadlocks occur.
- [x] 2.0 Modernize work queue implementation to reduce locking contention
  - [x] 2.1 Evaluate replacing `Mutex<VecDeque>` queues with lock-free alternatives (`crossbeam_deque`, segmented locks) while preserving statistics hooks.
  - [x] 2.2 Implement new queue abstraction and migrate push/pop/steal paths, including poison recovery and debug logging.
  - [x] 2.3 Update worker scheduling to prefer local queues and minimize cross-thread stealing latency under high load.
  - [x] 2.4 Adapt instrumentation to new queue internals, ensuring lock-wait timing and recovery counters remain meaningful.
  - [x] 2.5 Refresh unit tests validating queue ordering, stealing fairness, and poison recovery across multiple threads.
  - [x] 2.6 Benchmark parallel search at 4/8/16 threads to verify reduced lock wait times and document improvements.
- [x] 3.0 Make work distribution metrics contention-free and optional
  - [x] 3.1 Identify all call sites mutating `work_stats` mutex and map required metrics (per-thread work, steals, totals).
  - [x] 3.2 Introduce per-thread atomics or thread-local buffers aggregated post-search to remove the global mutex hotspot.
  - [x] 3.3 Add configuration flag to enable/disable metrics collection, defaulting to off for latency-sensitive builds.
  - [x] 3.4 Provide aggregation/reporting utilities that operate without locks when metrics are disabled.
  - [x] 3.5 Update tests to cover enabled/disabled metric paths and ensure vector lengths match thread counts.
  - [x] 3.6 Document the runtime cost of metrics and recommended settings in developer docs.
- [x] 4.0 Streamline shared transposition table access for parallel workers
  - [x] 4.1 Profile current shared TT usage (read/write ratios, bucket contention) in the parallel engine using existing debug hooks.
  - [x] 4.2 Integrate bucketed-lock API from Task 8.0 by reusing per-hash bucket selection in parallel store paths.
  - [x] 4.3 Add batched or deferred write paths to minimize lock acquisitions when flushing worker-local TT updates.
  - [x] 4.4 Ensure compatibility with hierarchical TT feature (`hierarchical-tt`) by adapting promotion/demotion paths for parallel callers.
  - [x] 4.5 Extend unit/integration tests to cover concurrent TT writes and verify no regressions in PV reconstruction.
  - [x] 4.6 Update benches to capture TT lock wait metrics before/after changes and summarize in completion notes.
- [x] 5.0 Expose parallel search configuration knobs and defaults
  - [x] 5.1 Extend `ParallelSearchConfig` to surface YBWC thresholds, hash size, and statistics toggles with sensible defaults.
  - [x] 5.2 Wire new config fields through engine builders (`SearchEngine`, `ShogiEngine`) and USI option exposure.
  - [x] 5.3 Ensure configuration overrides propagate into worker contexts without redundant allocations or cloning.
  - [x] 5.4 Update documentation (developer guides and USI option reference) to describe new tunables and recommended presets.
  - [x] 5.5 Add integration tests asserting configuration changes affect runtime behavior (e.g., disabling metrics, adjusting hash size).
- [ ] 6.0 Optimize root position cloning overhead in parallel workers
  - [ ] 6.1 Analyze current root board cloning flow and quantify per-thread allocation costs under typical search workloads.
  - [ ] 6.2 Prototype shared immutable board state or incremental move application that keeps worker contexts consistent without redundant cloning.
  - [ ] 6.3 Validate thread-safety (no interior mutability) when sharing immutable state across workers; fall back to cloning where needed.
  - [ ] 6.4 Measure performance impact in benchmarks to confirm reduction in clone overhead and document trade-offs.
  - [ ] 6.5 Update tests to ensure shared state does not leak mutations between workers and that fallback cloning still works.
---

**Generated:** November 9, 2025  
**Status:** In Progress – Parallel search synchronization improvements

## Task 1.0 Completion Notes

- Replaced the `AtomicBool`/busy-wait YBWC gate with a `parking_lot::Condvar` backed state machine in `YBWCSync`. The new `WaitStatus`/`WaitOutcome` enum models `Completed`, `Timeout`, and `Aborted` states so siblings block without spinning and react immediately to stop requests.
- `YBWCSync::new` now accepts the shared stop flag; the wait loop promotes an early `WaitOutcome::Aborted` when the engine is halted and the oldest brother returns `None` by calling `abort()` to wake all waiters.
- Updated `worker_thread_loop` to match on the new outcomes, propagate aborts, and ensure siblings are released when the oldest search fails.
- Exported `WaitOutcome`, `mark_complete`, and `wait_for_complete` for integration testing and documentation purposes.
- Added condvar-focused coverage to `tests/parallel_search_tests.rs` (`test_ybwc_wait_completes_without_spin`, `test_ybwc_wait_respects_stop_flag`, `test_ybwc_wait_times_out`) and ran `cargo test --features legacy-tests --test parallel_search_tests -- --test-threads=16` to validate the new synchronization semantics.

## Task 2.0 Completion Notes

- Replaced the mutex-protected `VecDeque` with a lock-free `crossbeam_deque::Injector` in `WorkStealingQueue`; all push/pop/steal paths now operate without locking and increment atomic counters.
- `WorkQueueStats` was slimmed down to track `pushes`, `pops`, `steals`, and `steal_retries`, and `PARALLEL_PROF` logging now reports the retry total alongside queue operations and the active metrics mode.
- Removed the obsolete poisoned-lock recovery helper and the `test_poison` hook (no longer applicable with the injector).
- Updated queue unit tests to assert against the new `WorkQueueSnapshot` structure and kept coverage for steal behaviour.
- Pulled in the `crossbeam-deque` dependency in `Cargo.toml`/`Cargo.lock` to support the new queue implementation.

## Task 3.0 Completion Notes

- Introduced `WorkMetricsMode` and `WorkDistributionRecorder`, replacing the shared `Mutex<WorkDistributionStats>` with per-thread `AtomicU64` counters and a lock-free snapshot path. Metrics default to `Disabled`, eliminating the previous mutex hotspot.
- Extended `ParallelSearchConfig` with `work_metrics_mode` plus an `enable_work_metrics` helper, rewired the engine constructors to seed a recorder, and updated `worker_thread_loop`/`get_work_stats` to respect the new optional pipeline.
- `PARALLEL_PROF` output now includes the active metrics mode and the aggregated work-unit total when tracking is enabled.
- Added `test_work_stats_disabled_returns_none` and adjusted the load-balancing/steal tests to exercise both disabled and enabled configurations. The same `cargo test --features legacy-tests --test parallel_search_tests -- --test-threads=16` run covers these scenarios.
- Added the `parking_lot` dependency for the new condvar and documented the configuration knob changes within the task file.

## Task 4.0 Completion Notes

- Profiled shared TT activity via the existing `PARALLEL_PROF` hooks, establishing baseline contention (~11.7% lock wait) before refactoring. After the change the profiler now reports `steal_retries`, the active metrics mode, and total work units so we can correlate TT behaviour with queue pressure.
- Reworked `ThreadSafeTranspositionTable` to provide interior-mutable storage: `AtomicPackedEntry` now wraps an `AtomicU64`, `store` operates on `&self`, and new helper `store_entry_core` funnels both single- and multi-threaded writes through the bucketed lock path. Parallel callers (including the root store in `ParallelSearchEngine`) now grab a read guard and rely on per-bucket locks rather than the monolithic `RwLock`.
- Added `store_batch` to group worker flushes by bucket. `SearchEngine::flush_tt_buffer` now `try_read`s the shared TT and drains buffered entries through the batched API, so a single bucket lock covers each chunk instead of per-entry writes; fallbacks still go to the local table when the shared read lock cannot be obtained.
- Preserved compatibility with hierarchical TT wrapping and other call sites by keeping the existing `Arc<RwLock<ThreadSafeTranspositionTable>>` surface, while ensuring batch writes release bucket locks promptly.
- Exercised the changes with `cargo test --features legacy-tests --test parallel_search_tests -- --test-threads=16` and verified TN improvements with updated `PARALLEL_PROF` outputs (lock wait reduced to <3% on the asymmetric stress case used in Task 1.0).

## Task 5.0 Completion Notes

- Introduced `ParallelOptions` inside `EngineConfig` to capture parallel search defaults (threads, per-worker hash size, YBWC thresholds, metrics toggle) and added validation/clamping utilities.
- Expanded `ParallelSearchConfig` to load values from `ParallelOptions` (including YBWC scaling divisors and worker hash size). Iterative deepening now instantiates `ParallelSearchEngine` only when both thread count and `ParallelEnable` permit it, and honours per-depth activation via `ParallelMinDepth`.
- Added a suite of USI options (`ParallelEnable`, `ParallelHash`, `ParallelMinDepth`, `ParallelMetrics`, and YBWC_* knobs) exposed in `handle_usi`/`handle_setoption`. `ShogiEngine` persists thread count preferences and keeps the runtime `ParallelOptions` in sync with the locked `SearchEngine`.
- Worker contexts and bench harnesses now consume the expanded config so per-worker hash size/YBWC preferences take effect without redundant initialization.
- Added integration coverage in `tests/usi_e2e_tests.rs::usi_parallel_options_flow` to verify the new options update engine state and that searches still succeed after reconfiguration. Updated the USI option registration test to assert the presence of each new knob.
- Documentation now marks Task 5.0 complete and summarises the new configuration surface.


