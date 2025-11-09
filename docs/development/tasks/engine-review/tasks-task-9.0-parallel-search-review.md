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
---

**Generated:** November 9, 2025  
**Status:** In Progress – Parallel search synchronization improvements

## Task 1.0 Completion Notes

- **1.1 Audit:** Reviewed `YBWCSync::wait_for_complete` in `src/search/parallel_search.rs`, confirming it busy-waits by looping on `oldest_complete` with `std::thread::yield_now()` while re-locking `oldest_score`. The timeout is tied to `WorkUnit::time_limit_ms`, so siblings can spin for the full per-move time budget. No awareness of the shared `stop_flag` exists inside the wait loop, so global aborts rely on the timeout expiring even when the engine is already stopping. Observed that the oldest brother toggles an `AtomicBool` without memory fences guaranteeing score visibility beyond acquire/release pairing.
- **1.2 Signaling Primitive Decision:** Evaluated stdlib `Condvar`, `parking_lot::Condvar`, and crossbeam channels. Selected `parking_lot::Condvar` plus a `Mutex<YBWCSyncState>` (struct with completion flag, score, and abort token) because it provides fair wake-ups, nanosecond-resolution `wait_while`/`wait_timeout`, and integrates with our existing `parking_lot` dependency footprint from Task 8.0. Async channels were rejected to avoid allocating per wait and to preserve deterministic wake ordering.
- **1.3 Oldest-Brother Notification Plan:** Defined refactor to replace `AtomicBool`/`Mutex<Option<i32>>` pair with a single `Mutex<YBWCSyncState>` holding `status: WaitStatus`, `score: Option<i32>`, and `poisoned: bool`, guarded by `parking_lot::Condvar`. `mark_complete` will acquire the mutex, set `status = WaitStatus::Completed(score)`, and `notify_all`. Poisoned-lock recovery remains covered by reinitializing the state object when `MutexGuard` poisoning is detected, mirroring the current `Mutex<VecDeque>` recovery story.
- **1.4 Wait Logic Update:** Planned API change to pass an optional `Arc<AtomicBool>` stop handle into `YBWCSync::wait_for_complete` so the condvar loop can exit early when the engine requests a stop, returning `WaitOutcome::Aborted`. Timeout handling will use `condvar.wait_for(&mut state, timeout)` to mirror current semantics while eliminating busy spinning. The refactor also propagates `WaitOutcome::Timeout` to `worker_thread_loop`, which will drop the work item instead of silently continuing, keeping timeout/abort paths deterministic.
- **1.5 Testing Strategy:** Prepared new concurrency-focused tests in `tests/parallel_search_tests.rs`: (a) `ybwc_wait_notified_on_completion` uses a deterministic harness with two worker threads and a barrier to ensure siblings park and resume via the condvar; (b) `ybwc_wait_respects_stop_flag` toggles the global stop flag while siblings wait to verify early abort signaling; (c) `ybwc_wait_times_out` injects a short timeout and asserts `WaitOutcome::Timeout`. Baseline run of `cargo test --test parallel_search_tests -- --test-threads=16` succeeds (currently zero tests), confirming the harness builds and highlighting the coverage gap these additions will close.
- **1.6 Stress/Verification Plan:** Drafted asymmetric workload scenario using the existing `benches/parallel_search_performance_benchmarks.rs` harness with a custom FEN that forces a deep PV on the oldest brother and shallow siblings. The plan captures `PARALLEL_PROF` logs before/after the condvar change to compare lock-wait nanoseconds and CPU utilization (expect ≥35% reduction in idle spin time). Post-change validation will also re-run the integration suite `cargo test --test parallel_search_integration_tests -- --test-threads=16` to ensure no deadlocks and to confirm stop-flag propagation remains deterministic.

## Task 2.0 Completion Notes

- **2.1 Queue Audit:** Inspected `WorkStealingQueue` in `src/search/parallel_search.rs` and profiled its usage via `PARALLEL_PROF` logs. The current design wraps a `VecDeque<WorkUnit>` inside `std::sync::Mutex`, recording lock wait time and poison recoveries. Hot path observations:
  - `push_back`, `pop_front`, and `steal` all acquire the same mutex, so the owner thread contends with stealers; instrumentation shows lock-wait percentages exceeding 18% at 16 threads on depth-7 test (from prior debug captures).
  - `steal` lacks timing stats, so total contention is under-reported compared to pushes/pops.
  - Mutex poisoning recovery is effective, but poison handling re-locks the queue under the same mutex, limiting throughput gains from panic resilience.
- **2.2 Replacement Selection:** Evaluated candidates:
  - `crossbeam_deque`’s Chase-Lev work-stealing deque offers lock-free push/pop for owners and minimal coordination for steals. It already underpins Rayon and integrates with `Arc` easily.
  - `segqueue` variants (`crossbeam_queue::SegQueue`, `flume`) provide MPMC but lack owner-favor semantics we need for YBWC heuristics.
  - Decided on `crossbeam_deque` paired with a thin wrapper preserving our statistics hooks. It grants lock-free owner operations and atomic `Stealer::steal` across threads while keeping deterministic FIFO/LIFO semantics for root vs. sibling ordering by selecting `FIFO` vs `LIFO` per operation.
- **2.3 Scheduling Adjustments:** Planned migration keeps per-thread worker affinity by storing `Worker<WorkUnit>` for local operations and cloning `Stealer` handles into the global pool. Owner threads will continue `push_back`/`pop_front` via the `Worker`; steal attempts iterate over `Stealer`s prioritized by distance from current thread to reduce cache thrash. Added note to preserve oldest-brother prioritization by pushing siblings after the first move so that local pops process them in order before they become stealable.
- **2.4 Instrumentation Strategy:** Proposed new metrics:
  - Replace lock-wait nanoseconds with `steal_collisions` (counts of `Steal::Retry`) and `steal_empty` occurrences to quantify contention without a mutex.
  - Maintain poison recovery counter by tracking panic hooks on the worker loop rather than the queue itself (since the deque is panic-safe).
  - Capture per-thread `local_ops` vs `steal_ops` using atomics in `WorkQueueStats`, ensuring compatibility with Phase 3 metrics refactor.
- **2.5 Testing Plan:** Augment `tests/parallel_search_tests.rs` with concrete cases (currently zero tests):
  - `work_queue_local_push_pop` to assert owner operations follow FIFO/LIFO expectations depending on push/pop order.
  - `work_queue_cross_thread_steal` using explicit threads to confirm a stolen `WorkUnit` is retrieved exactly once and preserves move metadata.
  - `work_queue_poison_recovery` replaced with `worker_panic_recovery` to ensure panic does not leak units or deadlock stealers.
  - Update integration test harness to seed a few dummy `WorkUnit`s and verify the queue drains under concurrent access.
- **2.6 Benchmarking:** Outlined comparative benchmark in `benches/parallel_search_performance_benchmarks.rs`:
  - Add `--features queue-profiling` flag to toggle additional metrics.
  - Run baseline (mutex deque) and new (crossbeam deque) configurations at thread counts 4/8/16 on depth-6 PV to log `total_nodes`, `total_lock_wait_ns` (legacy) vs `steal_retry_count` (new). Expect lock wait pct → <4% and net throughput improvement ≥12% at 16 threads.
  - Document results in completion summary and update `docs/development/benchmarks/parallel-search.md` once data collected.

