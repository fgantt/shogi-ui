# Task List: Parallel Search Implementation

This task list is derived from the PRD for adding parallel search to the Shogi engine using YBWC algorithm with work-stealing.

## Relevant Files

- `Cargo.toml` - Add `rayon` dependency for thread pool management
- `src/search/parallel_search.rs` - New module containing parallel search engine implementation
- `src/search/mod.rs` - Add parallel_search module declaration and exports
- `src/search/search_engine.rs` - Modify SearchEngine to support parallel context creation
- `src/search/search_engine.rs` (IterativeDeepening) - Integrate parallel search into iterative deepening
- `src/usi.rs` - Add USI_Threads option registration and handling
- `src/lib.rs` - Update handle_setoption to support USI_Threads configuration
- `src/types.rs` - Add ParallelSearchConfig to EngineConfig if needed
- `tests/parallel_search_tests.rs` - Unit tests for parallel search components
- `tests/parallel_search_integration_tests.rs` - Integration tests for parallel search with all features
- `tests/parallel_search_correctness_tests.rs` - Correctness tests comparing with single-threaded
- `tests/parallel_search_thread_safety_tests.rs` - Thread safety and race condition tests
- `tests/parallel_search_stress_tests.rs` - Stress tests (1000-game suite, long-running searches)
- `benches/parallel_search_performance_benchmarks.rs` - Performance benchmarks for speedup measurement

### Notes

- All test files should be placed in the `tests/` directory at the project root
- Use `cargo test` to run all tests, `cargo test parallel_search` to run parallel search tests specifically
- Use `cargo bench` to run performance benchmarks
- The parallel search should be transparent to existing code - primarily integrated through `IterativeDeepening::search()`

## Tasks

- [x] 1.0 Foundation and Infrastructure Setup
  - [x] 1.1 Add `rayon = "1.8"` dependency to `Cargo.toml` under `[dependencies]` section
  - [x] 1.2 Create new file `src/search/parallel_search.rs` with module structure and basic doc comments
  - [x] 1.3 Add `pub mod parallel_search;` declaration to `src/search/mod.rs`
  - [x] 1.4 Create `ParallelSearchConfig` struct with fields: `num_threads: usize`, `min_depth_parallel: u8`, `enable_parallel: bool`
  - [x] 1.5 Implement `ParallelSearchConfig::default()` that auto-detects CPU cores using `num_cpus::get()`
  - [x] 1.6 Create `ParallelSearchEngine` struct skeleton with fields for thread pool, config, and shared TT reference
  - [x] 1.7 Implement basic `ParallelSearchEngine::new(config: ParallelSearchConfig)` constructor that creates rayon thread pool
  - [x] 1.8 Add thread count validation (1-32 range) with clamping in `ParallelSearchConfig`
  - [x] 1.9 Add `USI_Threads` option to USI handler in `src/usi.rs` `handle_usi()` method: `"option name USI_Threads type spin default {} min 1 max 32".to_string()`
  - [x] 1.10 Update `ShogiEngine::handle_setoption()` in `src/lib.rs` to parse `USI_Threads` option and store thread count
  - [x] 1.11 Add thread count storage field to `ShogiEngine` struct (e.g., `thread_count: usize`)
  - [x] 1.12 Initialize thread count in `ShogiEngine::new()` to `num_cpus::get()`
  - [x] 1.13 Create unit test file `tests/parallel_search_tests.rs` with basic structure
  - [x] 1.14 Write test `test_thread_count_config_parsing()` to verify USI option parsing
  - [x] 1.15 Write test `test_thread_pool_creation()` to verify rayon thread pool can be created
  - [x] 1.16 Write test `test_usi_option_registration()` to verify `USI_Threads` appears in `handle_usi()` output
  - [x] 1.17 Verify code compiles without warnings: `cargo build --release`
  - [x] 1.18 Run all new tests: `cargo test parallel_search`

- [x] 2.0 Core Parallel Search Engine Implementation
  - [x] 2.1 Add thread-local search context struct `ThreadLocalSearchContext` with fields: `board: BitboardBoard`, `move_generator: MoveGenerator`, `evaluator: PositionEvaluator`, `history_table: [[i32; 9]; 9]`, `killer_moves: [Option<Move>; 2]`
  - [x] 2.2 Implement `ThreadLocalSearchContext::new(board: &BitboardBoard)` that clones board and creates new generators/evaluators
  - [x] 2.3 Add method `ParallelSearchEngine::create_thread_context(board: &BitboardBoard) -> ThreadLocalSearchContext`
  - [x] 2.4 Add `ParallelSearchEngine::search_root_moves()` method that takes board, moves list, and returns parallelized search results
  - [x] 2.5 Implement basic root-level move parallelization using `rayon::prelude::*` parallel iterator over moves
  - [x] 2.6 Add result aggregation logic to combine scores from all threads and find best move
  - [x] 2.7 Implement `ParallelSearchEngine::search_single_move()` that performs search on one move using thread-local context
  - [x] 2.8 Integrate with existing `SearchEngine` by calling `search_at_depth()` from thread-local context
  - [x] 2.9 Ensure board cloning is efficient - verify `BitboardBoard::clone()` implementation
  - [x] 2.10 Add method to `ParallelSearchEngine` that accepts shared `Arc<ThreadSafeTranspositionTable>` reference
  - [x] 2.11 Implement thread-safe access to shared TT from worker threads using `RwLock` read locks for probes
  - [x] 2.12 Add stop flag checking in parallel search workers (check `Arc<AtomicBool>` before each search)
  - [x] 2.13 Write test `test_parallel_search_engine_instantiation()` to verify engine can be created
  - [x] 2.14 Write test `test_board_cloning_correctness()` to verify cloned boards match original
  - [x] 2.15 Write test `test_result_aggregation()` with mock search results
  - [x] 2.16 Write test `test_basic_parallel_search_2_threads()` that searches simple position with 2 threads
  - [x] 2.17 Write test `test_parallel_vs_single_threaded_correctness()` comparing results on same positions
  - [x] 2.18 Run tests with `--test-threads=1` flag to check for race conditions: `cargo test -- --test-threads=1`
  - [x] 2.19 Verify no deadlocks by running parallel search tests multiple times
  - [x] 2.20 Ensure all tests pass and results match single-threaded for simple positions

- [x] 3.0 Work-Stealing and YBWC Algorithm
  - [x] 3.1 Create `WorkUnit` struct to represent a search task with fields: `board: BitboardBoard`, `move_to_search: Move`, `depth: u8`, `alpha: i32`, `beta: i32`, `parent_score: i32`
  - [x] 3.2 Implement work-stealing queue using custom deque with `Arc<Mutex<VecDeque<WorkUnit>>>`
  - [x] 3.3 Create `WorkStealingQueue` struct with methods: `push_back()`, `pop_front()`, `steal()`
  - [x] 3.4 Implement YBWC "oldest brother wait" synchronization - first move at node waits for completion before siblings start
  - [x] 3.5 Add work distribution logic that assigns work units to threads based on YBWC principles
  - [x] 3.6 Implement work-stealing mechanism that allows idle threads to steal work from busy threads
  - [x] 3.7 Add `ParallelSearchEngine::distribute_work()` method that creates work units and distributes them
  - [x] 3.8 Implement `ParallelSearchEngine::worker_thread_loop()` that processes work units and steals when idle
  - [x] 3.9 Add synchronization primitives (e.g., `Arc<Mutex<WorkStealingQueue>>`) for thread-safe work queue access
  - [x] 3.10 Implement proper load balancing by tracking thread workload and redistributing work
  - [x] 3.11 Add statistics tracking for work distribution (work units per thread, steal count, etc.)
  - [x] 3.12 Write test `test_work_stealing_queue_operations()` for basic queue operations
  - [x] 3.13 Write test `test_ybwc_synchronization_correctness()` verifying oldest brother wait behavior
  - [x] 3.14 Write test `test_load_balancing()` that verifies all threads receive work (no starvation)
  - [x] 3.15 Write test `test_work_stealing_triggers()` verifying idle threads steal work correctly
  - [x] 3.16 Write stress test `test_many_threads_work_stealing()` with 8-16 threads
  - [x] 3.17 Verify no race conditions in work queue operations using multiple iterations
  - [x] 3.18 Ensure all threads remain active during search (check thread activity statistics)
  - [x] 3.19 Verify work distribution is relatively even (max/min work units per thread ratio < 2.0)

- [x] 4.0 Feature Integration and Compatibility
  - [x] 4.1 Verify `ThreadSafeTranspositionTable` works correctly in parallel context - test concurrent read/write access
  - [x] 4.2 Implement TT entry sharing by ensuring all threads use same `Arc<RwLock<ThreadSafeTranspositionTable>>`
  - [x] 4.3 Add TT statistics aggregation - combine hit/miss counts from all threads into total statistics
  - [x] 4.4 Test TT collision handling in parallel context - verify no lost entries with concurrent writes
  - [x] 4.5 Verify TT hit rate improves with parallel search compared to single-threaded baseline
  - [x] 4.6 Test LMR behavior in parallel search - verify LMR reductions match single-threaded effectiveness
  - [x] 4.7 Test null move pruning in parallel context - ensure pruning decisions are correct across threads
  - [x] 4.8 Verify thread-local move ordering (history tables, killer moves) doesn't interfere with pruning
  - [x] 4.9 Implement statistics collection for LMR/null move across threads and aggregate correctly
  - [x] 4.10 Add tests for pruning correctness comparing parallel vs single-threaded on pruning-heavy positions
  - [x] 4.11 Test IID behavior in parallel search - verify IID finds good moves for ordering
  - [x] 4.12 Test aspiration window re-searches in parallel context - verify re-search triggers correctly
  - [x] 4.13 Verify iterative deepening coordination with parallel search - ensure depth progression works correctly
  - [x] 4.14 Test IID move promotion works with shared TT - verify TT entries from IID are accessible to all threads
  - [x] 4.15 Test tablebase lookup in parallel search - ensure lookups work correctly when multiple threads query
  - [x] 4.16 Verify tablebase results are shared appropriately (one thread can terminate early if tablebase hit found)
  - [x] 4.17 Test opening book lookup with parallel search - ensure lookups work correctly
  - [x] 4.18 Verify no race conditions in tablebase/opening book access - use synchronization if needed
  - [x] 4.19 Implement early termination propagation - when tablebase/opening book provides result, stop all threads
  - [x] 4.20 Write integration test `test_parallel_search_with_lmr()` verifying LMR integration
  - [x] 4.21 Write integration test `test_parallel_search_with_null_move()` verifying null move integration
  - [x] 4.22 Write integration test `test_parallel_search_with_iid()` verifying IID integration
  - [x] 4.23 Write integration test `test_parallel_search_with_aspiration_windows()` verifying aspiration window integration
  - [x] 4.24 Write integration test `test_parallel_search_with_tablebase()` verifying tablebase integration
  - [x] 4.25 Write integration test `test_parallel_search_with_opening_book()` verifying opening book integration
  - [x] 4.26 Verify all existing feature tests still pass with parallel search enabled
  - [x] 4.27 Test edge cases: very shallow depth, all moves pruned, no legal moves scenarios

- [ ] 5.0 Error Handling, Fallback, and Quality Assurance
 - [x] 5.1 Implement thread creation error handling - catch `rayon::ThreadPoolBuilder` errors and log warning
 - [x] 5.2 Implement automatic fallback to single-threaded mode when thread creation fails
 - [x] 5.3 Implement panic handling using `rayon`'s panic handling to catch worker thread panics
 - [x] 5.4 Implement panic recovery - when worker thread panics, continue with remaining threads or fallback
 - [x] 5.5 Implement mutex poison error handling - handle `PoisonError` from locks gracefully
 - [x] 5.6 Add comprehensive error logging using existing logging infrastructure (log crate or debug_utils)
 - [x] 5.7 Verify stop flag propagation - ensure all threads check `Arc<AtomicBool>` stop flag regularly
 - [x] 5.8 Test time limit enforcement across threads - verify all threads respect time limits
 - [x] 5.9 Implement graceful thread shutdown on stop - threads should finish current work unit then exit
- [x] 5.10 Implement partial result handling - when search stopped, return best result found so far
 - [x] 5.11 Add verification that no threads continue after stop flag is set
 - [x] 5.12 Write test `test_thread_creation_failure_handling()` simulating thread pool creation failure
 - [x] 5.13 Write test `test_panic_recovery()` that simulates panic in worker thread and verifies recovery
 - [x] 5.14 Write test `test_mutex_poison_handling()` simulating mutex poison scenarios
 - [x] 5.15 Write test `test_fallback_to_single_threaded()` verifying fallback triggers and works
 - [x] 5.16 Write test `test_stop_flag_propagation()` verifying all threads stop within 100ms of flag set
 - [x] 5.17 Write test `test_time_limit_enforcement()` verifying time limits respected across threads
 - [x] 5.18 Write test `test_graceful_shutdown()` with many stop/start cycles
 - [x] 5.19 Write test `test_partial_result_validity()` verifying partial results are valid moves
 - [x] 5.20 Profile parallel search performance using `cargo bench` and criterion benchmarks
  - [x] 5.21 Optimize lock contention - minimize TT read lock duration, batch writes if possible
  - [x] 5.22 Optimize board cloning efficiency - verified clone cost is low (~0.44µs; clone+make_move ~0.66µs), no change needed
  - [x] 5.23 Tune work-stealing parameters (queue sizes, steal frequency, etc.) for optimal performance
  - [x] 5.24 Optimize memory allocation patterns - reuse buffers, minimize allocations in hot path
 - [x] 5.25 Create benchmark suite `benches/parallel_search_performance_benchmarks.rs` using criterion
 - [x] 5.26 Benchmark single-threaded vs parallel (2, 4, 8 cores) on standard test positions
  - [x] 5.27 Measure lock contention overhead using profiling tools
  - [x] 5.28 Measure synchronization overhead (should be < 10%)
  - [ ] 5.29 Verify speedup targets met: ≥3x on 4 cores, test on multiple hardware configurations
    - Current run (depth 5/6) peaks at ≈1.18× on 8 threads; target not met.
    - Actions: gate TT writes, deepen YBWC parallelism beyond root, buffer per-thread writes, tune granularity.
    - New: Added silent bench mode and aggregated lock/YBWC metrics to quantify contention and parallelism usage during benches.
      - Metrics (aggregate example): `tt_reads=292,199`, `tt_writes=137` (1 write failure), `ybwc_batches=0`, `ybwc_siblings=0` for the current benchmark mix → YBWC not triggering often at benchmark positions/settings; consider deeper-trigger conditions or alternative bench positions to exercise YBWC.
  - [x] 5.30 Create correctness test suite with 100+ tactical positions comparing parallel vs single-threaded (dataset added; param suite implemented)
  - [x] 5.31 Create thread safety tests - run multiple searches concurrently, verify consistency
  - [x] 5.32 Create stress test suite: 1000-game test, long-running searches (5+ minutes), high thread count (16+) (added #[ignore] stress test)
  - [x] 5.33 Update existing tests for parallel compatibility - modify assertions that assume single-threaded behavior (default tests to single-thread unless SHOGI_TEST_ALLOW_PARALLEL=1)
  - [x] 5.34 Create tactical puzzle solving tests - verify parallel search solves puzzles correctly (sanity test added)
  - [~] 5.35 Create endgame test suite with tablebase positions (added #[ignore] smoke test; expand with TB coverage)
  - [x] 5.36 Integrate parallel search into `IterativeDeepening::search()` - add parallel path when threads > 1
  - [x] 5.37 Make parallel search the default in `IterativeDeepening` when thread count > 1
  - [ ] 5.38 Update `EngineConfig` struct to include thread count setting (add `thread_count: usize` field)
  - [ ] 5.39 Ensure thread count persists across search sessions - store in `ShogiEngine`
- [x] 5.38 Update `EngineConfig` struct to include thread count setting (add `thread_count: usize` field)
    - Implemented in `types::EngineConfig`; presets and migration updated.
- [x] 5.39 Ensure thread count persists across search sessions - store in `ShogiEngine`
    - `USI_Threads` saved/loaded from `~/.config/shogi-vibe/engine_prefs.json`.
 - [x] 5.40 Address all compiler warnings: run `cargo build --release` and fix all warnings
  - [ ] 5.41 Verify no stubbed code exists - search for `unimplemented!()`, `todo!()`, `panic!("not implemented")`
  - [ ] 5.42 Add comprehensive doc comments to all public APIs in `parallel_search.rs`
  - [ ] 5.43 Document thread safety guarantees in doc comments
  - [ ] 5.44 Document error handling and fallback behavior in doc comments
  - [ ] 5.45 Run full test suite: `cargo test` - ensure all tests pass
 - [x] 5.46 Run release build and verify zero warnings: `cargo build --release 2>&1 | grep warning`
  - [ ] 5.47 Run end-to-end integration tests with USI protocol
  - [ ] 5.48 Test configuration persistence - verify thread count setting survives engine restart
 - [x] 5.47 Run end-to-end integration tests with USI protocol
    - Added `tests/usi_e2e_tests.rs`: sets options, starts new game, performs a shallow search.
 - [x] 5.48 Test configuration persistence - verify thread count setting survives engine restart
    - Implemented `SHOGI_PREFS_DIR` override; tests write/read `engine_prefs.json` and validate round-trip.
  - [ ] 5.49 Perform final code review - verify code quality, naming conventions, error handling
- [x] 5.50 Create summary document of performance results, speedup achieved, and any known limitations
    - Done: `docs/release/PERFORMANCE_SUMMARY.md` created and updated with depth 5/6/7/8 results, notes, and next steps.


## Group B Performance Results (Benchmarks)

- Benchmark: `parallel_root_search` (depth 3/5/6), time limit 1000ms for depth 5/6, threads {1,2,4,8}
- Environment: same machine as development runs; Criterion defaults; values below are mean times (lower is better) with approximate speedups vs 1-thread baselines.

- Depth 5 (mean):
  - 1 thread: 1.427 s
  - 2 threads: 0.919 s (≈1.55×)
  - 4 threads: 0.946 s (≈1.51×)
  - 8 threads: 0.917 s (≈1.56×)

- Depth 6 (mean / notes):
  - 1 thread: 1.405 s
  - 2 threads: 0.918 s (≈1.53×)
  - 4 threads: 0.946 s (≈1.49×)
  - 8 threads: 1.213 s mean (median ≈0.922 s → ≈1.52×); mean impacted by outliers, median stable

- Changes that improved scaling:
  - Shared transposition table across all workers (reads + writes) for better reuse and PV consistency
  - Batching (`with_min_len`) to reduce scheduling overhead; per-task search contexts for Send safety
  - Minor contention profiling added to work-stealing queue

- Takeaway: At deeper depths (5/6), parallel speedups stabilize around ~1.5× on 2–8 threads with current configuration; further gains likely require additional parallelization within deeper nodes and contention reductions.

Additional runs (depth 7/8, stricter TT gating + YBWC thresholds):

- Depth 7 (mean): 1t 1.947s; 2t 2.560s (0.76×); 4t 2.222s (0.88×); 8t 1.728s (1.13×)
- Depth 8 (mean): 1t 2.287s; 2t 3.608s (0.63×); 4t 2.302s (0.99×); 8t 2.696s (0.85×)

Notes:
- Results indicate regression at higher depths with current thresholds; likely overhead-bound (including verbose USI info during benches) and insufficient deep-node parallelization.
- Next steps: add a silent bench mode to suppress USI info lines, increase YBWC granularity deeper in the tree, and continue TT write gating refinements.
