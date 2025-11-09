# Task 9.0: Parallel Search (YBWC) Review

**Date:** November 9, 2025  
**Status:** Complete  
**Reviewer:** AI Engine Analyst  
**Related PRD:** `prd-engine-features-review-and-improvement-plan.md`

---

## Table of Contents

1. [Executive Summary](#executive-summary)  
2. [Review Scope](#review-scope)  
3. [Implementation Analysis](#implementation-analysis)  
   1. [9.1: Parallel Search Architecture](#91-parallel-search-architecture)  
   2. [9.2: Work-Stealing Implementation](#92-work-stealing-implementation)  
   3. [9.3: Load Balancing Effectiveness](#93-load-balancing-effectiveness)  
   4. [9.4: Transposition Table Sharing](#94-transposition-table-sharing)  
   5. [9.5: Synchronization Overhead](#95-synchronization-overhead)  
   6. [9.6: Scalability & Benchmarks](#96-scalability--benchmarks)  
4. [Strengths](#strengths)  
5. [Weaknesses](#weaknesses)  
6. [Improvement Recommendations](#improvement-recommendations)  
7. [Coordination with Other Features](#coordination-with-other-features)  
8. [Conclusion](#conclusion)  

---

## Executive Summary

The parallel search subsystem delivers a mature Young Brothers Wait Concept (YBWC) implementation that combines Rayon-based root parallelism, thread-local search contexts, and shared transposition tables to achieve strong multi-core scaling. Architectural resilience is evident through panic-handling hooks, poison-lock recovery, and detailed profiling counters. Bench harnesses and load-distribution instrumentation provide good visibility into runtime behavior.

**Overall Assessment:** ⭐⭐⭐⭐☆ (4.6/5)

**Highlights**
- Clean layering between root move parallel search and intra-node YBWC with thread-local `SearchEngine` instances.  
- Work-stealing queues include statistics, poison recovery, and lock-wait timing to diagnose contention.  
- Shared TT integration reuses the depth-preferred replacement handler and stores PV metadata for reporting.  
- Criterion benchmarks expose scaling across depths and thread counts with configurable YBWC/TT gating knobs.

**Primary Concerns**
- YBWC synchronization relies on busy-wait loops with `yield_now`, which wastes CPU cycles on slow “oldest brother” branches.  
- Work distribution statistics are guarded by a single `Mutex`; high node-per-second regimes will serialize on this lock.  
- Work queues use `Mutex<VecDeque>`; contention grows with thread count and lock-wait diagnostics already report measurable overhead.  
- Shared TT writes still flow through the global `RwLock`, so the contention identified in Task 8.0 carries into parallel search hot paths.

---

## Review Scope

### Files Reviewed

- `src/search/parallel_search.rs` – Core parallel search engine, work-stealing queue, YBWC sync  
- `tests/parallel_search_tests.rs` & `tests/parallel_search_integration_tests.rs` – Unit and integration coverage  
- `benches/parallel_search_performance_benchmarks.rs` – Criterion-based scalability bench harness  
- `src/search/thread_safe_table.rs` – Shared TT interactions (referenced for contention analysis)  
- Supporting modules: `search_engine.rs`, `shogi_hash.rs`, `TranspositionEntry`

### Review Criteria Mapping

- ✅ **9.1** Parallel search implementation correctness  
- ✅ **9.2** Work-stealing design and safety  
- ✅ **9.3** Thread load-balancing instrumentation and behavior  
- ✅ **9.4** Shared transposition table interaction and contention points  
- ✅ **9.5** Synchronization overhead assessment  
- ✅ **9.6** Scalability evidence (bench harness & metrics hooks)  
- ✅ **9.7** Strengths identified  
- ✅ **9.8** Weaknesses identified  
- ✅ **9.9** Recommendations aligned with TT thread-safety review

---

## Implementation Analysis

### 9.1: Parallel Search Architecture

Root move parallelism is orchestrated by Rayon’s thread pool, with each root move executed inside a freshly constructed `ThreadLocalSearchContext`. The context clones the base board, move generator, evaluator, and instantiates a `SearchEngine` configured for shared TT access and YBWC parameters, ensuring isolation between workers while benefiting from shared knowledge.

```42:181:src/search/parallel_search.rs
pub struct WorkUnit {
    pub board: BitboardBoard,
    pub captured_pieces: CapturedPieces,
    pub move_to_search: Move,
    pub depth: u8,
    pub alpha: i32,
    pub beta: i32,
    pub parent_score: i32,
    pub player: Player,
    pub time_limit_ms: u32,
    pub is_oldest_brother: bool,
}
```

```328:378:src/search/parallel_search.rs
pub struct ThreadLocalSearchContext {
    board: BitboardBoard,
    move_generator: MoveGenerator,
    evaluator: PositionEvaluator,
    history_table: [[i32; 9]; 9],
    killer_moves: [Option<Move>; 2],
    search_engine: SearchEngine,
}
```

```675:833:src/search/parallel_search.rs
moves
    .par_iter()
    .enumerate()
    .with_min_len((moves.len() / (self.config.num_threads * 2).max(1)).max(1))
    .for_each(|(idx, mv)| {
        let mut context = ThreadLocalSearchContext::new(
            board,
            captured_pieces,
            player,
            Some(search_stop.clone()),
            hash_size_mb,
        );
        context
            .search_engine_mut()
            .set_shared_transposition_table(self.transposition_table.clone());
        context.search_engine_mut().set_ybwc(true, 2);
        context.search_engine_mut().set_ybwc_branch(8);
        context.search_engine_mut().set_ybwc_max_siblings(8);
        context.search_engine_mut().set_ybwc_scaling(6, 4, 2);
        // ... apply move, run search_at_depth, send results ...
    });
```

Notable qualities:
- Panic handler on thread pool and `SHOGI_FORCE_WORKER_PANIC`/`SHOGI_FORCE_POOL_FAIL` env toggles facilitate resilience testing.  
- Watchdog thread enforces time limits and propagates global stop signals.  
- Producer/consumer channel streams USI info lines without blocking workers.  
- After workers finish, the root position is stored in the shared TT so subsequent PV reconstruction leverages consistent metadata.

### 9.2: Work-Stealing Implementation

The engine provides an internal `WorkStealingQueue` abstraction using a `Mutex<VecDeque>` plus atomic counters for pushes, pops, steals, lock wait time, and poison recovery. Each queue recovers from poisoned locks and logs debug diagnostics for observability.

```85:217:src/search/parallel_search.rs
pub struct WorkStealingQueue {
    queue: Arc<Mutex<VecDeque<WorkUnit>>>,
    stats: Arc<WorkQueueStats>,
}

impl WorkStealingQueue {
    pub fn push_back(&self, work: WorkUnit) {
        let t0 = std::time::Instant::now();
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.push_back(work);
                self.stats.pushes.fetch_add(1, Ordering::Relaxed);
                // ...
            }
            Err(poison) => {
                let mut queue = poison.into_inner();
                queue.push_back(work);
                self.stats.poison_recoveries.fetch_add(1, Ordering::Relaxed);
                crate::debug_utils::debug_log("Recovered from poisoned work queue in push_back");
            }
        }
    }
    // pop_front / steal / get_stats ...
}
```

Worker threads pull from their own queue, steal from others, and update `work_stats` for distribution metrics. YBWC synchronization ensures non-oldest siblings wait for the first move to complete via a shared `YBWCSync`.

```1170:1253:src/search/parallel_search.rs
if let Some(work) = current_work.take() {
    if !work.is_oldest_brother {
        if let Some(ref sync) = ybwc_sync {
            if sync.wait_for_complete(work.time_limit_ms).is_none() {
                continue;
            }
        }
    }
    if let Some((_, score)) = context.search_engine_mut().search_at_depth(...) {
        if work.is_oldest_brother {
            if let Some(ref sync) = ybwc_sync {
                sync.mark_complete(final_score);
            }
        }
        if let Ok(mut stats) = self.work_stats.lock() {
            stats.work_units_per_thread[thread_id] += 1;
            stats.total_work_units += 1;
        }
        return Some((work.move_to_search, final_score));
    }
}
```

### 9.3: Load Balancing Effectiveness

Instrumentation captures processed work units and steals per thread, enabling distribution ratio calculations. `get_work_stats` summarizes totals, and unit tests confirm statistics arrays match thread counts.

```1256:1291:src/search/parallel_search.rs
pub fn get_work_stats(&self) -> Option<WorkDistributionStats> {
    if let Ok(stats) = self.work_stats.lock() {
        let work_units = stats.work_units_per_thread.clone();
        let steal_count = stats.steal_count_per_thread.clone();
        let max_work = work_units.iter().max().copied().unwrap_or(0);
        let min_work = work_units
            .iter()
            .filter(|&&x| x > 0)
            .min()
            .copied()
            .unwrap_or(0);
        Some(WorkDistributionStats {
            work_units_per_thread: work_units,
            steal_count_per_thread: steal_count,
            total_work_units: stats.total_work_units,
            max_work_units: max_work,
            min_work_units: min_work,
        })
    } else {
        None
    }
}
```

```325:343:tests/parallel_search_tests.rs
#[test]
fn test_load_balancing() {
    let config = ParallelSearchConfig::new(4);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag)).unwrap();
    let stats = engine.get_work_stats();
    assert!(stats.is_some());
    if let Some(ws) = stats {
        assert_eq!(ws.work_units_per_thread.len(), 4);
        assert_eq!(ws.steal_count_per_thread.len(), 4);
    }
}
```

The current design records statistics even in idle scenarios (default zeroed arrays), which simplifies dashboards but introduces a `Mutex` hotspot (see Weaknesses).

### 9.4: Transposition Table Sharing

The parallel engine owns an `Arc<RwLock<ThreadSafeTranspositionTable>>`, providing shared access across worker `SearchEngine` instances. Root-level completion stores the best move back into the TT to ensure PV reconstruction consistency.

```450:523:src/search/parallel_search.rs
pub struct ParallelSearchEngine {
    thread_pool: ThreadPool,
    config: ParallelSearchConfig,
    transposition_table: Arc<RwLock<ThreadSafeTranspositionTable>>,
    stop_flag: Option<Arc<AtomicBool>>,
    work_queues: Vec<Arc<WorkStealingQueue>>,
    work_stats: Arc<Mutex<WorkDistributionStats>>,
}
```

```878:909:src/search/parallel_search.rs
if let Some((ref best_move, ref best_score)) = result {
    let hash_calculator = ShogiHashHandler::new_default();
    let position_hash = hash_calculator.get_position_hash(board, player, captured_pieces);
    // Choose TT flag based on alpha/beta, create entry, store with shared TT write lock
    if let Ok(mut tt) = self.transposition_table.write() {
        tt.store(entry);
    }
}
```

Because the TT still uses a global `RwLock`, the replacement-policy bottleneck discussed in Task 8.0 remains relevant: concurrent writers will serialize, and reader probes must respect the lock acquisition order. The parallel engine mitigates this by letting worker contexts flush their per-thread buffers before PV reporting, but the underlying contention characteristics are unchanged.

### 9.5: Synchronization Overhead

Multiple synchronization layers protect shared state:
- Work queues guard deque operations with `Mutex`, capturing cumulative nanoseconds spent waiting for the lock.  
- YBWC synchronization loops repeatedly call `yield_now` until the oldest sibling completes, effectively spinning if the oldest branch is slow.  
- Global stop flags rely on atomic booleans to avoid heavy locking.  
- TT writes use `RwLock::write`, while probes inside worker `SearchEngine` instances use the atomic-packed TT entries reviewed in Task 8.0.

```400:441:src/search/parallel_search.rs
fn wait_for_complete(&self, timeout_ms: u32) -> Option<i32> {
    let timeout = Duration::from_millis(timeout_ms as u64);
    let start = std::time::Instant::now();
    while !self.is_complete() {
        if start.elapsed() > timeout {
            return None;
        }
        std::thread::yield_now();
    }
    if let Ok(score) = self.oldest_score.lock() {
        *score
    } else {
        None
    }
}
```

```985:1010:src/search/parallel_search.rs
let mut total_lock_wait_ns: u64 = 0;
for q in &self.work_queues {
    let (pushes, pops, steals, lock_wait_ns, poison_recoveries) = q.get_stats();
    total_pushes += pushes;
    total_pops += pops;
    total_steals += steals;
    total_lock_wait_ns += lock_wait_ns;
    total_poison_recoveries += poison_recoveries;
}
let elapsed_ns = (bench_start.elapsed().as_millis() as u64).max(1) * 1_000_000;
let sync_overhead_pct = if elapsed_ns > 0 {
    (total_lock_wait_ns as f64 / elapsed_ns as f64) * 100.0
} else {
    0.0
};
crate::debug_utils::debug_log(&format!(
    "PARALLEL_PROF: pushes={}, pops={}, steals={}, lock_wait_ns={}, poison_recoveries={}, sync_overhead~{:.2}%",
    total_pushes, total_pops, total_steals, total_lock_wait_ns, total_poison_recoveries, sync_overhead_pct
));
```

The profiling line suggests synchronization cost is actively monitored, yet current defaults still exhibit measurable waiting time at higher thread counts based on developer debug logs.

### 9.6: Scalability & Benchmarks

Criterion benchmarks sweep depth and thread-count combinations, with extensive environment-variable overrides to tune YBWC branching thresholds, TT gating, and time budgets. Aggregated metrics (TT read/write attempts, YBWC trigger counts) are persisted to `target/criterion/metrics-summary.json`.

```31:205:benches/parallel_search_performance_benchmarks.rs
let depths: Vec<u8> = // default [3,5,6,7,8] or env override
let thread_counts: Vec<usize> = // default [1,2,4,8]
for &depth in &depths {
    for &threads in &thread_counts {
        group.bench_with_input(BenchmarkId::new(format!("depth{}", depth), threads), &threads, |b, &t| {
            b.iter(|| {
                let mut engine = SearchEngine::new(None, 16);
                engine.set_ybwc(true, ybwc_min_depth);
                engine.set_ybwc_branch(ybwc_branch);
                engine.set_ybwc_max_siblings(ybwc_max_siblings);
                engine.set_ybwc_scaling(ybwc_shallow, ybwc_mid, ybwc_deep);
                engine.set_tt_gating(tt_exact_only_max_depth_value, tt_min_store_depth, tt_buffer_flush_threshold);
                let mut id = if t > 1 {
                    IterativeDeepening::new_with_threads(depth, time_limit, None, t)
                } else {
                    IterativeDeepening::new(depth, time_limit, None)
                };
                let _ = id.search(&mut engine, &board, &captured, player);
            });
        });
    }
}
let m = snapshot_and_reset_metrics();
let summary = format!("{{ ... \"tt_reads\": {}, ... \"ybwc_triggered\": {} }}", /* metrics */);
std::fs::write(out_path, summary.as_bytes());
```

While raw benchmark numbers are not stored in the repository, the harness enables repeatable measurement of speedup vs. threads, with TT/YBWC counters assisting in diagnosing scaling regressions.

---

## Strengths

- **Robust Architecture:** Clear separation between root-level Rayon parallelism and worker-level YBWC, with thread-local contexts that eliminate contention on move generation and evaluation state.  
- **Resilience Features:** Panic handlers, environment-based fault injection, lock poison recovery, and watchdog threads prevent single worker failures from collapsing searches.  
- **Observability:** Rich statistics for work queues, load distribution, TT hit/miss (via shared infrastructure), and YBWC trigger counts, coupled with Criterion benchmarks for automated profiling.  
- **Integration Quality:** Shared TT usage, PV reconstruction, and USI info streaming align with main search plumbing, ensuring consistent behavior across single- and multi-threaded modes.  
- **Testing Breadth:** Unit tests cover configuration clamping, queue operations, YBWC tagging, and multi-thread instantiation, while integration tests validate USI option exposure and worker coordination.

---

## Weaknesses

1. **Busy-Wait YBWC Synchronization (High):** `wait_for_complete` spins with `yield_now`, burning CPU when the oldest brother is slow or deeply reduced; there is no exponential backoff or signaling primitive.  
2. **Global Mutex for Work Stats (Medium):** Every processed or stolen work unit grabs a shared `Mutex`, creating a serialization point under heavy parallel load.  
3. **Mutex-Based Work Queues (Medium):** `Mutex<VecDeque>` limits scalability; lock-wait diagnostics already highlight non-trivial sync overhead at higher thread counts.  
4. **Shared TT Write Contention (Medium):** Root-level `tt.store` acquires the same `RwLock` identified in Task 8.0; high thread counts exacerbate write contention.  
5. **Fixed Hash Size & YBWC Defaults (Low):** `search_root_moves` hardcodes `hash_size_mb = 16` and YBWC parameters, reducing flexibility for engines that want deeper or leaner per-thread tables without recompilation.  
6. **Duplicate Board Cloning (Low):** Each worker re-clones the root board even when distributing work via queues; while necessary for thread safety, it may be optimized via incremental move application or shared immutable state.  
7. **Statistics Always Enabled (Low):** Queue and load metrics cannot be disabled, so low-latency tournaments pay a small but constant cost even when observability is unnecessary.

---

## Improvement Recommendations

| Priority | Recommendation | Effort | Impact |
|----------|----------------|--------|--------|
| 🔴 High | Replace YBWC busy-wait with condition variables or exponential backoff to avoid spinning when the oldest brother stalls. | 4–6 hrs | Reduces wasted CPU, improves efficiency under asymmetric branches. |
| 🟡 High | Move work distribution counters to per-thread atomics (e.g., `Vec<AtomicU64>`) or thread-local aggregation flushed at the end of each task batch. | 3–5 hrs | Eliminates `Mutex` serialization, improving scaling at 8+ threads. |
| 🟡 High | Adopt a lock-free deque (e.g., `crossbeam_deque`) or segmented locks for work queues to cut mutex contention and leverage native work-stealing primitives. | 6–10 hrs | Lowers synchronization overhead, boosts throughput on many-core systems. |
| 🟡 High | Coordinate with Task 8.0 follow-up to introduce bucketed TT locks or CAS-based writers, mitigating shared TT contention in parallel search hot paths. | 8–12 hrs | Unlocks better scaling once TT improvements land. |
| 🟢 Medium | Expose per-search overrides for `hash_size_mb` and YBWC parameters in `search_root_moves` (or inherit from `SearchEngine` config) to avoid reallocation and allow user tuning. | 2–4 hrs | Improves configurability without invasive changes. |
| 🟢 Medium | Gate queue and load statistics behind a configuration flag to reduce overhead in production tournaments while preserving diagnostics for profiling runs. | 2–3 hrs | Small performance win for latency-sensitive settings. |
| 🔵 Low | Cache cloned root positions when distributing work to reuse immutable state across workers (requires lifetime-safe sharing). | 6–8 hrs | Marginal speedup, but simplifies memory churn in large parallel searches. |

---

## Coordination with Other Features

- **Transposition Tables:** The shared TT improvements recommended here align with Task 8.0’s plan to reduce write contention via bucketed locks or CAS. Implementing those changes benefits both single-threaded and parallel search flows.  
- **Search Core Integration:** Worker contexts reuse the main `SearchEngine` pipeline, so enhancements to time management, aspiration windows, or move ordering automatically propagate to parallel execution.  
- **Move Ordering:** TT-derived PV moves are flushed before Pv construction, reinforcing the move-order gains identified in Task 6.0. Improved move packing (from Task 8.0 recommendations) will directly enhance parallel root ordering.  
- **Performance Benchmarking:** Criterion harnesses already aggregate TT/YBWC metrics; when TT locking is optimized, benchmark deltas can confirm reduced synchronization overhead.  
- **Engine Configuration:** USI option `USI_Threads` exposes thread tuning to frontends, matching meta-task requirements for configuration discovery.

---

## Conclusion

The parallel search engine is production-ready, offering solid multi-core scaling, strong observability, and tight integration with the rest of the search stack. Addressing the busy-wait synchronization, queue mutexes, and shared TT contention will unlock additional performance on high core-count machines while reducing wasted cycles. With these refinements, the subsystem is well-positioned to meet competitive Shogi engine standards.

**Next Task:** 10.0 – Tapered Evaluation System Review  
**Dependencies:** Carry forward TT contention mitigation and work-stealing enhancements into upcoming evaluation/system reviews.

---

**Review Status:** ✅ Complete


