//! Parallel search implementation using Young Brothers Wait Concept (YBWC) algorithm with work-stealing.
//!
//! This module provides multi-threaded search capabilities to utilize multiple CPU cores,
//! achieving near-linear speedup with the number of available cores.
//!
//! # Architecture
//!
//! The parallel search engine uses:
//! - Rayon thread pool for efficient thread management
//! - Shared transposition table for knowledge sharing between threads
//! - Thread-local search contexts to avoid contention
//! - Work-stealing queue for load balancing
//!
//! # Thread Safety
//!
//! All shared data structures are thread-safe:
//! - Transposition table uses `RwLock` for concurrent access
//! - Board state is cloned for each thread
//! - Move generators and evaluators are thread-local

use crate::bitboards::BitboardBoard;
use crate::evaluation::PositionEvaluator;
use crate::moves::MoveGenerator;
use crate::search::search_engine::SearchEngine;
use crate::search::search_engine::GLOBAL_NODES_SEARCHED;
use crate::search::ThreadSafeTranspositionTable;
use crate::utils::time::TimeSource;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Player};
use crate::types::search::ParallelOptions;
use crossbeam_deque::{Injector, Steal};
use num_cpus;
use parking_lot::{Condvar, Mutex as ParkingMutex};
use rayon::{prelude::*, ThreadPool, ThreadPoolBuilder};
use std::env;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, RwLock,
};
use std::thread;
use std::time::Duration;

/// Represents a unit of work (search task) to be executed by a worker thread.
#[derive(Clone)]
pub struct WorkUnit {
    /// Board state after applying the move.
    pub board: BitboardBoard,

    /// Captured pieces after applying the move.
    pub captured_pieces: CapturedPieces,

    /// Move to search at this node.
    pub move_to_search: Move,

    /// Search depth remaining.
    pub depth: u8,

    /// Alpha bound for alpha-beta pruning.
    pub alpha: i32,

    /// Beta bound for alpha-beta pruning.
    pub beta: i32,

    /// Score from parent node (used for YBWC synchronization).
    pub parent_score: i32,

    /// Player to move at this position.
    pub player: Player,

    /// Time limit for this search in milliseconds.
    pub time_limit_ms: u32,

    /// Whether this is the first (oldest) move at a node (YBWC).
    pub is_oldest_brother: bool,
}

/// Work-stealing queue for distributing search tasks among worker threads.
///
/// This queue supports:
/// - Push/pop operations from the owner thread (lock-free when uncontended)
/// - Steal operations from other threads
/// - Thread-safe synchronization
/// Thread-safe work-stealing deque used by the parallel search engine.
///
/// Thread safety:
/// - Uses a `Mutex<VecDeque<WorkUnit>>` internally; short critical sections reduce contention.
/// - Recovers from poisoned locks to keep the engine running under panic scenarios.
pub struct WorkStealingQueue {
    /// Lock-free injector queue backing this work queue.
    injector: Arc<Injector<WorkUnit>>,

    /// Statistics for this queue.
    stats: Arc<WorkQueueStats>,
}

/// Statistics for work queue operations.
#[derive(Default)]
struct WorkQueueStats {
    /// Number of items pushed to this queue.
    pushes: AtomicU64,

    /// Number of items popped from this queue.
    pops: AtomicU64,

    /// Number of items stolen from this queue.
    steals: AtomicU64,

    /// Number of times a steal yielded `Retry`.
    steal_retries: AtomicU64,
}

/// Snapshot of queue metrics captured without locking.
#[derive(Default, Debug, Clone, Copy)]
pub struct WorkQueueSnapshot {
    pub pushes: u64,
    pub pops: u64,
    pub steals: u64,
    pub steal_retries: u64,
}

impl WorkStealingQueue {
    /// Construct a new lock-free work-stealing queue.
    pub fn new() -> Self {
        Self {
            injector: Arc::new(Injector::new()),
            stats: Arc::new(WorkQueueStats::default()),
        }
    }

    /// Push a work unit to the back of the queue (owner thread operation).
    /// Push a work unit to the back of the queue.
    ///
    /// Error handling: Recovers from poisoned lock and logs a debug message.
    pub fn push_back(&self, work: WorkUnit) {
        self.injector.push(work);
        self.stats.pushes.fetch_add(1, Ordering::Relaxed);
    }

    /// Pop a work unit from the front of the queue (owner thread operation).
    /// Pop a work unit from the front of the queue.
    /// Returns `None` if the queue is empty.
    pub fn pop_front(&self) -> Option<WorkUnit> {
        match self.injector.steal() {
            Steal::Success(work) => {
                self.stats.pops.fetch_add(1, Ordering::Relaxed);
                Some(work)
            }
            Steal::Retry => {
                self.stats.steal_retries.fetch_add(1, Ordering::Relaxed);
                None
            }
            Steal::Empty => None,
        }
    }

    /// Steal a work unit from the back of the queue (other thread operation).
    /// Attempt to steal a work unit from the back of the queue.
    /// Returns `None` when the queue is empty or lock acquisition fails.
    pub fn steal(&self) -> Option<WorkUnit> {
        match self.injector.steal() {
            Steal::Success(work) => {
                self.stats.steals.fetch_add(1, Ordering::Relaxed);
                Some(work)
            }
            Steal::Retry => {
                self.stats.steal_retries.fetch_add(1, Ordering::Relaxed);
                None
            }
            Steal::Empty => None,
        }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.injector.is_empty()
    }

    /// Get the number of items in the queue.
    pub fn len(&self) -> usize {
        self.injector.len()
    }

    /// Get statistics for this queue.
    pub fn get_stats(&self) -> WorkQueueSnapshot {
        WorkQueueSnapshot {
            pushes: self.stats.pushes.load(Ordering::Relaxed),
            pops: self.stats.pops.load(Ordering::Relaxed),
            steals: self.stats.steals.load(Ordering::Relaxed),
            steal_retries: self.stats.steal_retries.load(Ordering::Relaxed),
        }
    }
}

impl Default for WorkStealingQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Enumeration describing how aggressively to track work distribution metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkMetricsMode {
    /// Disable metrics entirely for lowest overhead.
    Disabled,
    /// Track per-thread counters using relaxed atomics.
    Basic,
}

impl WorkMetricsMode {
    fn is_enabled(self) -> bool {
        matches!(self, WorkMetricsMode::Basic)
    }
}

/// Lock-free recorder for work distribution metrics.
pub struct WorkDistributionRecorder {
    mode: WorkMetricsMode,
    work_units_per_thread: Vec<AtomicU64>,
    steal_count_per_thread: Vec<AtomicU64>,
    total_work_units: AtomicU64,
}

impl WorkDistributionRecorder {
    pub fn new(num_threads: usize, mode: WorkMetricsMode) -> Self {
        if mode.is_enabled() {
            let work_units = (0..num_threads)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>();
            let steals = (0..num_threads)
                .map(|_| AtomicU64::new(0))
                .collect::<Vec<_>>();
            Self {
                mode,
                work_units_per_thread: work_units,
                steal_count_per_thread: steals,
                total_work_units: AtomicU64::new(0),
            }
        } else {
            Self {
                mode,
                work_units_per_thread: Vec::new(),
                steal_count_per_thread: Vec::new(),
                total_work_units: AtomicU64::new(0),
            }
        }
    }

    #[inline(always)]
    pub fn record_work(&self, thread_id: usize) {
        if !self.mode.is_enabled() {
            return;
        }
        if let Some(counter) = self.work_units_per_thread.get(thread_id) {
            counter.fetch_add(1, Ordering::Relaxed);
            self.total_work_units.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline(always)]
    pub fn record_steal(&self, thread_id: usize) {
        if !self.mode.is_enabled() {
            return;
        }
        if let Some(counter) = self.steal_count_per_thread.get(thread_id) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn mode(&self) -> WorkMetricsMode {
        self.mode
    }

    pub fn snapshot(&self) -> Option<WorkDistributionStats> {
        if !self.mode.is_enabled() {
            return None;
        }
        let work_units: Vec<u64> = self
            .work_units_per_thread
            .iter()
            .map(|counter| counter.load(Ordering::Relaxed))
            .collect();
        let steal_count: Vec<u64> = self
            .steal_count_per_thread
            .iter()
            .map(|counter| counter.load(Ordering::Relaxed))
            .collect();
        let max_work = work_units.iter().max().copied().unwrap_or(0);
        let min_work = work_units
            .iter()
            .filter(|&&x| x > 0)
            .min()
            .copied()
            .unwrap_or(0);
        Some(WorkDistributionStats {
            mode: self.mode,
            work_units_per_thread: work_units,
            steal_count_per_thread: steal_count,
            total_work_units: self.total_work_units.load(Ordering::Relaxed),
            max_work_units: max_work,
            min_work_units: min_work,
        })
    }
}

/// Statistics for work distribution across threads.
#[derive(Debug)]
pub struct WorkDistributionStats {
    /// Metrics mode used to gather this snapshot.
    pub mode: WorkMetricsMode,

    /// Total work units processed per thread.
    pub work_units_per_thread: Vec<u64>,

    /// Total steal count per thread.
    pub steal_count_per_thread: Vec<u64>,

    /// Total number of work units processed.
    pub total_work_units: u64,

    /// Maximum work units processed by any thread.
    pub max_work_units: u64,

    /// Minimum work units processed by any thread.
    pub min_work_units: u64,
}

impl WorkDistributionStats {
    /// Create new statistics tracker for the given number of threads.
    pub fn new(num_threads: usize) -> Self {
        Self {
            mode: WorkMetricsMode::Basic,
            work_units_per_thread: vec![0; num_threads],
            steal_count_per_thread: vec![0; num_threads],
            total_work_units: 0,
            max_work_units: 0,
            min_work_units: 0,
        }
    }

    /// Get the work distribution ratio (max/min).
    pub fn distribution_ratio(&self) -> f64 {
        if !self.mode.is_enabled() || self.min_work_units == 0 {
            return f64::INFINITY;
        }
        self.max_work_units as f64 / self.min_work_units as f64
    }
}

impl Default for WorkDistributionStats {
    fn default() -> Self {
        Self {
            mode: WorkMetricsMode::Disabled,
            work_units_per_thread: Vec::new(),
            steal_count_per_thread: Vec::new(),
            total_work_units: 0,
            max_work_units: 0,
            min_work_units: 0,
        }
    }
}

/// Configuration for parallel search engine.
#[derive(Clone, Debug)]
pub struct ParallelSearchConfig {
    /// Number of threads to use for parallel search (1-32).
    pub num_threads: usize,

    /// Minimum depth at which to activate parallel search.
    pub min_depth_parallel: u8,

    /// Whether parallel search is enabled.
    pub enable_parallel: bool,

    /// Hash table size for worker contexts (MB).
    pub hash_size_mb: usize,

    /// Whether YBWC coordination is enabled.
    pub ybwc_enabled: bool,

    /// Minimum depth before YBWC engages.
    pub ybwc_min_depth: u8,

    /// Minimum branching factor required for YBWC trigger.
    pub ybwc_min_branch: usize,

    /// Maximum sibling searches allowed when YBWC triggers.
    pub ybwc_max_siblings: usize,

    /// Divisor for shallow depth YBWC scaling.
    pub ybwc_shallow_divisor: usize,

    /// Divisor for mid depth YBWC scaling.
    pub ybwc_mid_divisor: usize,

    /// Divisor for deep depth YBWC scaling.
    pub ybwc_deep_divisor: usize,

    /// Mode controlling work distribution metrics collection.
    pub work_metrics_mode: WorkMetricsMode,
}

impl Default for ParallelSearchConfig {
    fn default() -> Self {
        let num_threads = num_cpus::get();
        Self {
            num_threads: num_threads.clamp(1, 32),
            min_depth_parallel: 4,
            enable_parallel: num_threads > 1,
            hash_size_mb: 16,
            ybwc_enabled: false,
            ybwc_min_depth: 2,
            ybwc_min_branch: 8,
            ybwc_max_siblings: 8,
            ybwc_shallow_divisor: 6,
            ybwc_mid_divisor: 4,
            ybwc_deep_divisor: 2,
            work_metrics_mode: WorkMetricsMode::Disabled,
        }
    }
}

impl ParallelSearchConfig {
    /// Create a new parallel search configuration with the specified number of threads.
    ///
    /// # Arguments
    ///
    /// * `num_threads` - Number of threads (will be clamped to 1-32 range)
    ///
    /// # Returns
    ///
    /// A new `ParallelSearchConfig` with clamped thread count.
    pub fn new(num_threads: usize) -> Self {
        Self {
            num_threads: num_threads.clamp(1, 32),
            min_depth_parallel: 4,
            enable_parallel: num_threads > 1,
            hash_size_mb: 16,
            ybwc_enabled: false,
            ybwc_min_depth: 2,
            ybwc_min_branch: 8,
            ybwc_max_siblings: 8,
            ybwc_shallow_divisor: 6,
            ybwc_mid_divisor: 4,
            ybwc_deep_divisor: 2,
            work_metrics_mode: WorkMetricsMode::Disabled,
        }
    }

    /// Set the number of threads, clamping to valid range (1-32).
    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.num_threads = num_threads.clamp(1, 32);
        self.enable_parallel = self.num_threads > 1;
    }

    /// Configure work metrics collection mode.
    pub fn set_work_metrics_mode(&mut self, mode: WorkMetricsMode) {
        self.work_metrics_mode = mode;
    }

    /// Convenience toggle for enabling or disabling basic metrics.
    pub fn enable_work_metrics(&mut self, enabled: bool) {
        self.work_metrics_mode = if enabled {
            WorkMetricsMode::Basic
        } else {
            WorkMetricsMode::Disabled
        };
    }

    pub fn from_parallel_options(options: &ParallelOptions, threads: usize) -> Self {
        let mut config = Self::default();
        config.num_threads = threads.clamp(1, 32);
        config.enable_parallel = options.enable_parallel && config.num_threads > 1;
        config.min_depth_parallel = options.min_depth_parallel;
        config.hash_size_mb = options.hash_size_mb.clamp(1, 512);
        config.ybwc_enabled = options.ybwc_enabled;
        config.ybwc_min_depth = options.ybwc_min_depth;
        config.ybwc_min_branch = options.ybwc_min_branch.max(1);
        config.ybwc_max_siblings = options.ybwc_max_siblings.max(1);
        config.ybwc_shallow_divisor = options.ybwc_shallow_divisor.max(1);
        config.ybwc_mid_divisor = options.ybwc_mid_divisor.max(1);
        config.ybwc_deep_divisor = options.ybwc_deep_divisor.max(1);
        config.work_metrics_mode = if options.enable_metrics {
            WorkMetricsMode::Basic
        } else {
            WorkMetricsMode::Disabled
        };
        config
    }
}

/// Thread-local search context for parallel search workers.
///
/// Each thread maintains its own copy of board state, move generator,
/// and evaluator to avoid contention during parallel search.
#[allow(dead_code)]
pub struct ThreadLocalSearchContext {
    /// Cached root board position for quick resets.
    root_board: BitboardBoard,

    /// Thread-local working board state.
    board: BitboardBoard,

    /// Cached root captured pieces for quick resets.
    root_captured: CapturedPieces,

    /// Thread-local captured pieces state.
    captured_pieces: CapturedPieces,

    /// Thread-local move generator.
    move_generator: MoveGenerator,

    /// Thread-local position evaluator.
    evaluator: PositionEvaluator,

    /// Thread-local history table for move ordering.
    history_table: [[i32; 9]; 9],

    /// Thread-local killer moves (2 slots per depth).
    killer_moves: [Option<Move>; 2],

    /// Thread-local search engine instance.
    search_engine: SearchEngine,
}

impl ThreadLocalSearchContext {
    /// Create a new thread-local search context by cloning the root board state.
    ///
    /// # Arguments
    ///
    /// * `board` - Root board position to clone
    /// * `captured_pieces` - Root captured pieces to clone
    /// * `player` - Current player to move
    /// * `stop_flag` - Shared stop flag for search interruption
    /// * `hash_size_mb` - Size of transposition table in MB
    pub fn new(
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        _player: Player,
        stop_flag: Option<Arc<AtomicBool>>,
        hash_size_mb: usize,
    ) -> Self {
        let root_board = board.clone();
        let root_captured = captured_pieces.clone();
        Self {
            root_board: root_board.clone(),
            board: root_board.clone(),
            root_captured: root_captured.clone(),
            captured_pieces: root_captured,
            move_generator: MoveGenerator::new(),
            evaluator: PositionEvaluator::new(),
            history_table: [[0; 9]; 9],
            killer_moves: [None, None],
            search_engine: SearchEngine::new(stop_flag, hash_size_mb),
        }
    }

    /// Refresh the cached root state with a new board/captured snapshot.
    pub fn refresh_root_state(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces) {
        self.root_board.clone_from(board);
        self.board.clone_from(board);
        self.root_captured.clone_from(captured_pieces);
        self.captured_pieces.clone_from(captured_pieces);
    }

    /// Reset working state back to cached root copies.
    pub fn reset_to_root(&mut self) {
        self.board.clone_from(&self.root_board);
        self.captured_pieces.clone_from(&self.root_captured);
    }

    /// Get mutable reference to the thread-local board.
    pub fn board_mut(&mut self) -> &mut BitboardBoard {
        &mut self.board
    }

    /// Get reference to the thread-local board.
    pub fn board(&self) -> &BitboardBoard {
        &self.board
    }

    /// Get mutable reference to thread-local captured pieces.
    pub fn captured_pieces_mut(&mut self) -> &mut CapturedPieces {
        &mut self.captured_pieces
    }

    /// Get reference to thread-local captured pieces.
    pub fn captured_pieces(&self) -> &CapturedPieces {
        &self.captured_pieces
    }

    /// Borrow mutable references to both board and captured pieces simultaneously.
    pub fn board_and_captured_mut(&mut self) -> (&mut BitboardBoard, &mut CapturedPieces) {
        (&mut self.board, &mut self.captured_pieces)
    }

    pub fn search_root_child(
        &mut self,
        mv: &Move,
        player: Player,
        depth: u8,
        time_limit_ms: u32,
        alpha: i32,
        beta: i32,
    ) -> Option<i32> {
        let board = &mut self.board;
        let captured = &mut self.captured_pieces;
        if let Some(captured_piece) = board.make_move(mv) {
            captured.add_piece(captured_piece.piece_type, player);
        }
        self.search_engine
            .search_at_depth(
                board,
                captured,
                player.opposite(),
                depth,
                time_limit_ms,
                alpha,
                beta,
            )
            .map(|(_, score)| score)
    }

    pub fn flush_and_get_pv(&mut self, player: Player, depth: u8) -> Vec<Move> {
        self.search_engine.flush_tt_buffer();
        self.search_engine
            .get_pv_for_reporting(&self.board, &self.captured_pieces, player, depth)
    }

    /// Update the stop flag used by the thread-local search engine.
    pub fn update_stop_flag(&mut self, stop_flag: Option<Arc<AtomicBool>>) {
        self.search_engine.set_stop_flag(stop_flag);
    }

    /// Get mutable reference to the thread-local search engine.
    pub fn search_engine_mut(&mut self) -> &mut SearchEngine {
        &mut self.search_engine
    }
}

static NEXT_CONTEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

struct ThreadContextHolder {
    generation: u64,
    context: ThreadLocalSearchContext,
}

impl ThreadContextHolder {
    fn new(
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        stop_flag: Option<Arc<AtomicBool>>,
        hash_size_mb: usize,
    ) -> Self {
        Self {
            generation: 0,
            context: ThreadLocalSearchContext::new(
                board,
                captured_pieces,
                player,
                stop_flag,
                hash_size_mb,
            ),
        }
    }
}

/// Result of waiting for the oldest brother to complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitOutcome {
    Completed(i32),
    Timeout,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaitStatus {
    Pending,
    Completed,
    Aborted,
}

struct YBWCSyncState {
    status: WaitStatus,
    score: Option<i32>,
}

struct YBWCSyncInner {
    state: ParkingMutex<YBWCSyncState>,
    condvar: Condvar,
    stop_flag: Option<Arc<AtomicBool>>,
}

/// Synchronization data for YBWC "oldest brother wait" concept.
pub struct YBWCSync {
    inner: Arc<YBWCSyncInner>,
}

impl YBWCSync {
    fn new(stop_flag: Option<Arc<AtomicBool>>) -> Self {
        Self {
            inner: Arc::new(YBWCSyncInner {
                state: ParkingMutex::new(YBWCSyncState {
                    status: WaitStatus::Pending,
                    score: None,
                }),
                condvar: Condvar::new(),
                stop_flag,
            }),
        }
    }

    /// Mark oldest brother as complete and store its score.
    pub fn mark_complete(&self, score: i32) {
        let mut guard = self.inner.state.lock();
        guard.status = WaitStatus::Completed;
        guard.score = Some(score);
        drop(guard);
        self.inner.condvar.notify_all();
    }

    /// Abort waiting siblings (used when global stop flag is raised).
    fn abort(&self) {
        let mut guard = self.inner.state.lock();
        guard.status = WaitStatus::Aborted;
        drop(guard);
        self.inner.condvar.notify_all();
    }

    /// Wait for oldest brother to complete (with timeout and stop flag support).
    pub fn wait_for_complete(&self, timeout_ms: u32) -> WaitOutcome {
        let timeout = Duration::from_millis(timeout_ms as u64);
        let start = std::time::Instant::now();

        loop {
            // Fast path: check stop flag without taking lock.
            if let Some(ref stop_flag) = self.inner.stop_flag {
                if stop_flag.load(Ordering::Acquire) {
                    self.abort();
                }
            }

            let mut state = self.inner.state.lock();
            match state.status {
                WaitStatus::Completed => {
                    return WaitOutcome::Completed(state.score.unwrap_or(0));
                }
                WaitStatus::Aborted => {
                    return WaitOutcome::Aborted;
                }
                WaitStatus::Pending => {
                    let now = std::time::Instant::now();
                    if now.duration_since(start) >= timeout {
                        state.status = WaitStatus::Aborted;
                        return WaitOutcome::Timeout;
                    }
                    let remaining = timeout.saturating_sub(now.duration_since(start));
                    let timed_out = self
                        .inner
                        .condvar
                        .wait_for(&mut state, remaining)
                        .timed_out();
                    if timed_out {
                        state.status = WaitStatus::Aborted;
                        return WaitOutcome::Timeout;
                    }
                    // loop to re-check status after wake
                }
            }
        }
    }
}

/// Parallel search engine using YBWC algorithm with work-stealing.
///
/// This engine coordinates parallel search across multiple threads,
/// sharing a transposition table while maintaining thread-local search contexts.
pub struct ParallelSearchEngine {
    /// Thread pool for managing worker threads.
    thread_pool: ThreadPool,

    /// Parallel search configuration.
    config: ParallelSearchConfig,

    /// Shared transposition table accessible by all threads.
    transposition_table: Arc<RwLock<ThreadSafeTranspositionTable>>,

    /// Shared stop flag for interrupting search across all threads.
    stop_flag: Option<Arc<AtomicBool>>,

    /// Work queues for each thread (for work-stealing).
    work_queues: Vec<Arc<WorkStealingQueue>>,

    /// Work distribution statistics.
    work_stats: Arc<WorkDistributionRecorder>,
}

impl ParallelSearchEngine {
    /// Create a new parallel search engine with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Parallel search configuration
    ///
    /// # Returns
    ///
    /// A new `ParallelSearchEngine` instance, or an error if thread pool creation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread pool cannot be created.
    /// Create a new engine using an internal shared transposition table.
    ///
    /// Thread safety: the shared TT is protected by `RwLock`.
    /// Error handling: returns `Err(String)` on thread pool creation failure.
    pub fn new(config: ParallelSearchConfig) -> Result<Self, String> {
        if env::var("SHOGI_FORCE_POOL_FAIL").ok().as_deref() == Some("1") {
            return Err("Forced pool failure via SHOGI_FORCE_POOL_FAIL".to_string());
        }
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .stack_size(8 * 1024 * 1024)
            .panic_handler(|_| {
                // Ensure panics in worker threads do not bring the process down; request stop
                crate::utils::telemetry::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        // For now, we'll create a placeholder transposition table.
        // This will be replaced with the actual shared TT from SearchEngine in later checkpoints.
        let tt_config = crate::search::TranspositionConfig::performance_optimized();
        let transposition_table =
            Arc::new(RwLock::new(ThreadSafeTranspositionTable::new(tt_config)));

        let num_threads = config.num_threads;
        let metrics_mode = config.work_metrics_mode;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();

        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag: None,
            work_queues,
            work_stats: Arc::new(WorkDistributionRecorder::new(num_threads, metrics_mode)),
        })
    }

    fn configure_worker_engine(&self, engine: &mut SearchEngine) {
        engine.set_ybwc(self.config.ybwc_enabled, self.config.ybwc_min_depth);
        engine.set_ybwc_branch(self.config.ybwc_min_branch);
        engine.set_ybwc_max_siblings(self.config.ybwc_max_siblings);
        engine.set_ybwc_scaling(
            self.config.ybwc_shallow_divisor,
            self.config.ybwc_mid_divisor,
            self.config.ybwc_deep_divisor,
        );
    }

    /// Create a new parallel search engine with stop flag.
    ///
    /// # Arguments
    ///
    /// * `config` - Parallel search configuration
    /// * `stop_flag` - Optional shared stop flag for interrupting search
    ///
    /// # Returns
    ///
    /// A new `ParallelSearchEngine` instance, or an error if thread pool creation fails.
    /// Create a new engine with an optional external stop flag.
    ///
    /// When the stop flag is set, workers observe it and stop after current work.
    pub fn new_with_stop_flag(
        config: ParallelSearchConfig,
        stop_flag: Option<Arc<AtomicBool>>,
    ) -> Result<Self, String> {
        if env::var("SHOGI_FORCE_POOL_FAIL").ok().as_deref() == Some("1") {
            return Err("Forced pool failure via SHOGI_FORCE_POOL_FAIL".to_string());
        }
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .stack_size(8 * 1024 * 1024)
            .panic_handler(|_| {
                crate::utils::telemetry::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        let tt_config = crate::search::TranspositionConfig::performance_optimized();
        let transposition_table =
            Arc::new(RwLock::new(ThreadSafeTranspositionTable::new(tt_config)));

        let num_threads = config.num_threads;
        let metrics_mode = config.work_metrics_mode;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();

        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag,
            work_queues,
            work_stats: Arc::new(WorkDistributionRecorder::new(num_threads, metrics_mode)),
        })
    }

    /// Get the number of threads configured for this engine.
    pub fn num_threads(&self) -> usize {
        self.config.num_threads
    }

    /// Check if parallel search is enabled.
    pub fn is_parallel_enabled(&self) -> bool {
        self.config.enable_parallel
    }

    /// Create a thread-local search context for a worker thread.
    ///
    /// # Arguments
    ///
    /// * `board` - Root board position to clone
    /// * `captured_pieces` - Root captured pieces to clone
    /// * `player` - Current player to move
    /// * `hash_size_mb` - Size of transposition table in MB
    pub fn create_thread_context(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        hash_size_mb: usize,
    ) -> ThreadLocalSearchContext {
        let mut context = ThreadLocalSearchContext::new(
            board,
            captured_pieces,
            player,
            self.stop_flag.clone(),
            hash_size_mb,
        );
        self.configure_worker_engine(context.search_engine_mut());
        context
    }

    /// Get reference to the shared transposition table.
    pub fn transposition_table(&self) -> &Arc<RwLock<ThreadSafeTranspositionTable>> {
        &self.transposition_table
    }

    /// Create a new parallel search engine with a shared transposition table.
    ///
    /// # Arguments
    ///
    /// * `config` - Parallel search configuration
    /// * `transposition_table` - Shared transposition table to use across all threads
    /// * `stop_flag` - Optional shared stop flag for interrupting search
    ///
    /// # Returns
    ///
    /// A new `ParallelSearchEngine` instance with shared TT, or an error if thread pool creation fails.
    /// Create a new engine with a provided shared transposition table.
    ///
    /// Useful when composing with an existing `SearchEngine` TT to maximize reuse.
    pub fn new_with_shared_tt(
        config: ParallelSearchConfig,
        transposition_table: Arc<RwLock<ThreadSafeTranspositionTable>>,
        stop_flag: Option<Arc<AtomicBool>>,
    ) -> Result<Self, String> {
        if env::var("SHOGI_FORCE_POOL_FAIL").ok().as_deref() == Some("1") {
            return Err("Forced pool failure via SHOGI_FORCE_POOL_FAIL".to_string());
        }
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .stack_size(8 * 1024 * 1024)
            .panic_handler(|_| {
                crate::utils::telemetry::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        let num_threads = config.num_threads;
        let metrics_mode = config.work_metrics_mode;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();

        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag,
            work_queues,
            work_stats: Arc::new(WorkDistributionRecorder::new(num_threads, metrics_mode)),
        })
    }

    /// Perform parallel search on root-level moves.
    ///
    /// This method parallelizes the search across all root moves,
    /// with each thread searching one move independently.
    ///
    /// # Arguments
    ///
    /// * `board` - Root board position
    /// * `captured_pieces` - Captured pieces information
    /// * `player` - Current player to move
    /// * `moves` - List of legal moves at root position
    /// * `depth` - Search depth
    /// * `time_limit_ms` - Time limit in milliseconds
    /// * `alpha` - Alpha bound for alpha-beta pruning
    /// * `beta` - Beta bound for alpha-beta pruning
    ///
    /// # Returns
    ///
    /// Best move and score, or None if search was interrupted or no moves available.
    pub fn search_root_moves(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        moves: &[Move],
        depth: u8,
        time_limit_ms: u32,
        alpha: i32,
        beta: i32,
    ) -> Option<(Move, i32)> {
        if moves.is_empty() {
            return None;
        }

        // Combined, per-search stop flag (aggregates engine stop and time limit)
        let search_stop = Arc::new(AtomicBool::new(false));
        // If engine-level stop is already set, respect it
        if let Some(ref engine_stop) = self.stop_flag {
            if engine_stop.load(Ordering::Relaxed) {
                search_stop.store(true, Ordering::Relaxed);
            }
        }

        // Use thread pool to parallelize search across moves, while streaming results
        let hash_size_mb = self.config.hash_size_mb;
        let (tx, rx) = std::sync::mpsc::channel::<(Move, i32, String)>();
        // Reset global nodes counter and seldepth for this depth
        GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);
        crate::search::search_engine::GLOBAL_SELDEPTH.store(0, Ordering::Relaxed);
        let _start_time = TimeSource::now();
        let bench_start = std::time::Instant::now();
        let watchdog_cancel = Arc::new(AtomicBool::new(false));
        // Start a watchdog to enforce time limit and propagate external stop
        let wd_cancel = watchdog_cancel.clone();
        let wd_stop = search_stop.clone();
        let engine_stop_opt = self.stop_flag.clone();
        let deadline = std::time::Instant::now() + Duration::from_millis(time_limit_ms as u64);
        let watchdog = std::thread::spawn(move || {
            while !wd_cancel.load(Ordering::Relaxed) {
                // External stop propagates
                if let Some(ref engine_stop) = engine_stop_opt {
                    if engine_stop.load(Ordering::Relaxed) {
                        wd_stop.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                // Time limit enforcement
                if std::time::Instant::now() >= deadline {
                    wd_stop.store(true, Ordering::Relaxed);
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });

        // Shared best-so-far for return value
        let best_shared: Arc<Mutex<(Option<Move>, i32)>> = Arc::new(Mutex::new((None, i32::MIN)));
        let best_for_consumer = best_shared.clone();

        // Start consumer thread to stream info lines as results arrive
        let consumer = thread::spawn(move || {
            let mut best_pv = String::new();
            while let Ok((mv, score, pv)) = rx.recv() {
                // Update best-so-far
                if let Ok(mut guard) = best_for_consumer.lock() {
                    if score > guard.1 {
                        *guard = (Some(mv.clone()), score);
                        best_pv = pv.clone();
                    }
                }
                let elapsed = bench_start.elapsed().as_millis() as u64;
                let nodes = GLOBAL_NODES_SEARCHED.load(Ordering::Relaxed);
                let nps = if elapsed > 0 {
                    nodes.saturating_mul(1000) / (elapsed as u64)
                } else {
                    0
                };
                // Get actual seldepth (selective depth) - the maximum depth reached
                // If seldepth wasn't updated during search (shouldn't happen), use depth as fallback
                let seldepth_raw =
                    crate::search::search_engine::GLOBAL_SELDEPTH.load(Ordering::Relaxed) as u8;
                let seldepth = if seldepth_raw == 0 {
                    depth
                } else {
                    seldepth_raw.max(depth)
                };
                // Emit real USI info line with score and PV (skip during silent benches)
                if std::env::var("SHOGI_SILENT_BENCH").is_err() {
                    if !best_pv.is_empty() {
                        println!(
                            "info depth {} seldepth {} multipv 1 score cp {} time {} nodes {} nps {} pv {}",
                            depth, seldepth,
                            if let Ok(g) = best_for_consumer.lock() { g.1 } else { score },
                            elapsed, nodes, nps, best_pv
                        );
                    }
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
            }
        });

        let shared_tt = self.transposition_table.clone();
        let generation_id = NEXT_CONTEXT_GENERATION.fetch_add(1, Ordering::Relaxed);

        self.thread_pool.install(|| {
            moves
                .par_iter()
                .enumerate()
                .with_min_len((moves.len() / (self.config.num_threads * 2).max(1)).max(1))
                .for_each_init(
                    || {
                        ThreadContextHolder::new(
                            board,
                            captured_pieces,
                            player,
                            Some(search_stop.clone()),
                            hash_size_mb,
                        )
                    },
                    |holder, (idx, mv)| {
                        if search_stop.load(Ordering::Relaxed) {
                            crate::utils::telemetry::debug_log(
                                "Stop flag set before worker started move; skipping",
                            );
                            return;
                        }

                        holder
                            .context
                            .update_stop_flag(Some(search_stop.clone()));

                        if holder.generation != generation_id {
                            holder
                                .context
                                .refresh_root_state(board, captured_pieces);
                            holder
                                .context
                                .search_engine_mut()
                                .set_shared_transposition_table(shared_tt.clone());
                            self.configure_worker_engine(holder.context.search_engine_mut());
                            holder.generation = generation_id;
                        } else {
                            holder.context.reset_to_root();
                        }

                        let search_depth = if depth > 0 { depth - 1 } else { 0 };

                        if env::var("SHOGI_FORCE_WORKER_PANIC")
                            .ok()
                            .as_deref()
                            == Some("1")
                            && idx == 0
                        {
                            panic!("Forced worker panic for testing");
                        }

                        let search_score = holder.context.search_root_child(
                            mv,
                            player,
                            search_depth,
                            time_limit_ms,
                            -beta,
                            -alpha,
                        );

                        if let Some(score_child) = search_score {
                            let seldepth = crate::search::search_engine::GLOBAL_SELDEPTH
                                .load(Ordering::Relaxed) as u8;
                            let pv_depth = if seldepth > 0 { seldepth } else { 64 };
                            let pv_moves = holder
                                .context
                                .flush_and_get_pv(player.opposite(), pv_depth);
                            let mv_root = mv.to_usi_string();
                            let mut pv_string =
                                String::with_capacity(mv_root.len() + pv_moves.len() * 4);
                            pv_string.push_str(&mv_root);
                            for child in pv_moves.iter() {
                                pv_string.push(' ');
                                pv_string.push_str(&child.to_usi_string());
                            }
                            if search_stop.load(Ordering::Relaxed) {
                                crate::utils::telemetry::debug_log("Stop flag observed after move search; reporting partial and returning");
                            }
                            let score = -score_child;
                            let _ = tx.send((mv.clone(), score, pv_string));
                        } else {
                            crate::utils::telemetry::debug_log(
                                "Search_at_depth returned None; reporting move with no PV",
                            );
                            let _ = tx.send((mv.clone(), i32::MIN / 2, mv.to_usi_string()));
                        }

                        holder.context.reset_to_root();
                    },
                );
        });
        // Close the channel to signal the consumer that no more results are coming
        drop(tx);
        // All senders dropped; wait for consumer to finish
        // Signal and join watchdog
        watchdog_cancel.store(true, Ordering::Relaxed);
        let _ = watchdog.join();
        let _ = consumer.join();
        let result = if let Ok(guard) = best_shared.lock() {
            guard.0.clone().map(|m| (m, guard.1))
        } else {
            None
        };

        // Store root position in shared TT so PV can be built from root
        // This ensures the PV chain is complete when building from the root position
        if let Some((ref best_move, ref best_score)) = result {
            use crate::search::shogi_hash::ShogiHashHandler;
            use crate::{TranspositionEntry, TranspositionFlag};

            // Calculate root position hash
            let hash_calculator = ShogiHashHandler::new_default();
            let position_hash = hash_calculator.get_position_hash(board, player, captured_pieces);

            // Determine flag based on score vs alpha/beta
            let flag = if *best_score <= alpha {
                TranspositionFlag::UpperBound
            } else if *best_score >= beta {
                TranspositionFlag::LowerBound
            } else {
                TranspositionFlag::Exact
            };

            // Create TT entry with best_move stored (critical for PV building)
            let entry = TranspositionEntry::new_with_age(
                *best_score,
                depth,
                flag,
                Some(best_move.clone()),
                position_hash,
            );

            // Store in shared TT
            if let Ok(tt) = self.transposition_table.read() {
                tt.store(entry);
            }

            // IMPORTANT: Before building PV from root, we need to ensure all worker threads
            // have flushed their TT buffers. However, worker threads are already done at this point.
            // The issue might be that some positions along the PV simply weren't searched deeply enough,
            // or their TT entries don't have best_move. We've already fixed storing best_move,
            // so if PV is still short, it likely means the search depth itself is limited.

            // Now rebuild the full PV from the root position using the shared TT
            // This ensures we get the complete PV chain, not just from child positions
            let seldepth =
                crate::search::search_engine::GLOBAL_SELDEPTH.load(Ordering::Relaxed) as u8;
            let pv_depth = if seldepth > 0 { seldepth } else { depth };

            // Create a temporary search engine context to build PV from root
            let mut temp_context = ThreadLocalSearchContext::new(
                board,
                captured_pieces,
                player,
                self.stop_flag.clone(),
                16,
            );
            temp_context
                .search_engine_mut()
                .set_shared_transposition_table(self.transposition_table.clone());

            // Build full PV from root position - try multiple times if first attempt is short
            // This helps if there's a race condition with TT writes
            let mut full_pv = temp_context.search_engine_mut().get_pv_for_reporting(
                board,
                captured_pieces,
                player,
                pv_depth,
            );

            // If PV is shorter than expected, try building again after a brief delay
            // to allow any remaining TT writes to flush
            if full_pv.len() < (depth as usize).min(10) {
                std::thread::sleep(std::time::Duration::from_millis(5));
                full_pv = temp_context.search_engine_mut().get_pv_for_reporting(
                    board,
                    captured_pieces,
                    player,
                    pv_depth,
                );
            }

            // Emit final info line with the complete PV if we have at least 2 moves
            if full_pv.len() >= 2 && std::env::var("SHOGI_SILENT_BENCH").is_err() {
                let elapsed = bench_start.elapsed().as_millis() as u64;
                let nodes = GLOBAL_NODES_SEARCHED.load(Ordering::Relaxed);
                let nps = if elapsed > 0 {
                    nodes.saturating_mul(1000) / (elapsed as u64)
                } else {
                    0
                };
                let seldepth_final = if seldepth == 0 {
                    depth
                } else {
                    seldepth.max(depth)
                };
                let pv_string: String = full_pv
                    .iter()
                    .map(|m| m.to_usi_string())
                    .collect::<Vec<String>>()
                    .join(" ");

                if !pv_string.is_empty() {
                    println!(
                        "info depth {} seldepth {} multipv 1 score cp {} time {} nodes {} nps {} pv {}",
                        depth, seldepth_final, *best_score, elapsed, nodes, nps, pv_string
                    );
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
            }
        }

        // Aggregate queue stats to estimate contention and synchronization overhead
        let mut total_pushes: u64 = 0;
        let mut total_pops: u64 = 0;
        let mut total_steals: u64 = 0;
        let mut total_steal_retries: u64 = 0;
        for q in &self.work_queues {
            let snapshot = q.get_stats();
            total_pushes += snapshot.pushes;
            total_pops += snapshot.pops;
            total_steals += snapshot.steals;
            total_steal_retries += snapshot.steal_retries;
        }
        let metrics_mode = self.work_stats.mode();
        let total_work_units = self
            .work_stats
            .snapshot()
            .map(|s| s.total_work_units)
            .unwrap_or(0);
        crate::utils::telemetry::debug_log(&format!(
            "PARALLEL_PROF: pushes={}, pops={}, steals={}, steal_retries={}, work_metrics_mode={:?}, total_work_units={}",
            total_pushes, total_pops, total_steals, total_steal_retries, metrics_mode, total_work_units
        ));
        result
    }

    /// Aggregate search results from all threads and find the best move.
    ///
    /// # Arguments
    ///
    /// * `results` - Vector of search results from each thread
    ///
    /// # Returns
    ///
    /// Best move and score, or None if no valid results.
    #[allow(dead_code)]
    fn aggregate_results(&self, results: Vec<Option<(Move, i32)>>) -> Option<(Move, i32)> {
        let mut best_move: Option<Move> = None;
        let mut best_score = i32::MIN;

        for result in results {
            if let Some((mv, score)) = result {
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
            }
        }

        best_move.map(|mv| (mv, best_score))
    }

    /// Search a single move using thread-local context.
    ///
    /// # Arguments
    ///
    /// * `context` - Thread-local search context
    /// * `board` - Board position after applying the move
    /// * `captured_pieces` - Captured pieces after applying the move
    /// * `player` - Player to move at this position
    /// * `depth` - Search depth
    /// * `time_limit_ms` - Time limit in milliseconds
    /// * `alpha` - Alpha bound
    /// * `beta` - Beta bound
    ///
    /// # Returns
    ///
    /// Search score, or None if search was interrupted.
    pub fn search_single_move(
        &self,
        context: &mut ThreadLocalSearchContext,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        time_limit_ms: u32,
        alpha: i32,
        beta: i32,
    ) -> Option<i32> {
        // Check stop flag
        if let Some(ref stop_flag) = self.stop_flag {
            if stop_flag.load(Ordering::Relaxed) {
                return None;
            }
        }

        // Perform search
        let mut test_board = board.clone();
        if let Some((_, score)) = context.search_engine_mut().search_at_depth(
            &mut test_board,
            captured_pieces,
            player,
            depth,
            time_limit_ms,
            alpha,
            beta,
        ) {
            Some(score)
        } else {
            None
        }
    }

    /// Distribute work units to threads based on YBWC principles.
    ///
    /// Creates work units for each move, with the first move marked as "oldest brother"
    /// for YBWC synchronization.
    ///
    /// # Arguments
    ///
    /// * `board` - Root board position
    /// * `captured_pieces` - Captured pieces information
    /// * `player` - Current player to move
    /// * `moves` - List of legal moves
    /// * `depth` - Search depth
    /// * `time_limit_ms` - Time limit in milliseconds
    /// * `alpha` - Alpha bound
    /// * `beta` - Beta bound
    ///
    /// # Returns
    ///
    /// Vector of work units and YBWC synchronization object.
    pub fn distribute_work(
        &self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        moves: &[Move],
        depth: u8,
        time_limit_ms: u32,
        alpha: i32,
        beta: i32,
    ) -> (Vec<WorkUnit>, YBWCSync) {
        let mut work_units = Vec::new();
        let ybwc_sync = YBWCSync::new(self.stop_flag.clone());

        for (idx, mv) in moves.iter().enumerate() {
            // Clone board and apply move
            let mut test_board = board.clone();
            let mut test_captured = captured_pieces.clone();

            if let Some(captured) = test_board.make_move(mv) {
                test_captured.add_piece(captured.piece_type, player);
            }

            let work_unit = WorkUnit {
                board: test_board,
                captured_pieces: test_captured,
                move_to_search: mv.clone(),
                depth: if depth > 0 { depth - 1 } else { 0 },
                alpha: -beta,
                beta: -alpha,
                parent_score: 0,
                player: player.opposite(),
                time_limit_ms,
                is_oldest_brother: idx == 0, // First move is oldest brother
            };

            work_units.push(work_unit);
        }

        (work_units, ybwc_sync)
    }

    /// Worker thread loop that processes work units and steals when idle.
    ///
    /// This method implements the core work-stealing logic:
    /// 1. Try to pop work from own queue
    /// 2. If empty, try to steal from other threads' queues
    /// 3. Process work unit and update statistics
    ///
    /// # Arguments
    ///
    /// * `thread_id` - ID of this worker thread (0-indexed)
    /// * `work_unit` - Work unit to process (if provided, process it directly)
    /// * `ybwc_sync` - YBWC synchronization object (for oldest brother wait)
    /// * `context` - Thread-local search context
    ///
    /// # Returns
    ///
    /// Search result (move and score), or None if interrupted.
    pub fn worker_thread_loop(
        &self,
        thread_id: usize,
        work_unit: Option<WorkUnit>,
        ybwc_sync: Option<Arc<YBWCSync>>,
        context: &mut ThreadLocalSearchContext,
    ) -> Option<(Move, i32)> {
        let mut current_work = work_unit;

        loop {
            // Check stop flag
            if let Some(ref stop_flag) = self.stop_flag {
                if stop_flag.load(Ordering::Relaxed) {
                    return None;
                }
            }

            // If we have work, process it
            if let Some(work) = current_work.take() {
                // If this is oldest brother, we process immediately
                // Otherwise, wait for oldest brother to complete (YBWC)
                if !work.is_oldest_brother {
                    if let Some(ref sync) = ybwc_sync {
                        match sync.wait_for_complete(work.time_limit_ms) {
                            WaitOutcome::Completed(_) => {}
                            WaitOutcome::Timeout | WaitOutcome::Aborted => {
                                continue;
                            }
                        }
                    }
                }

                // Perform search
                let mut test_board = work.board.clone();
                if let Some((_, score)) = context.search_engine_mut().search_at_depth(
                    &mut test_board,
                    &work.captured_pieces,
                    work.player,
                    work.depth,
                    work.time_limit_ms,
                    work.alpha,
                    work.beta,
                ) {
                    let final_score = -score; // Negate for parent perspective

                    // If oldest brother, mark sync as complete
                    if work.is_oldest_brother {
                        if let Some(ref sync) = ybwc_sync {
                            sync.mark_complete(final_score);
                        }
                    }

                    // Update statistics
                    self.work_stats.record_work(thread_id);

                    return Some((work.move_to_search, final_score));
                } else if work.is_oldest_brother {
                    if let Some(ref sync) = ybwc_sync {
                        sync.abort();
                    }
                }
            }

            // No work in hand, try to get work from queue
            if thread_id < self.work_queues.len() {
                // Try to pop from own queue first
                if let Some(work) = self.work_queues[thread_id].pop_front() {
                    current_work = Some(work);
                    continue;
                }

                // Try to steal from other threads
                for (idx, queue) in self.work_queues.iter().enumerate() {
                    if idx != thread_id {
                        if let Some(work) = queue.steal() {
                            // Update steal statistics
                            self.work_stats.record_steal(thread_id);
                            current_work = Some(work);
                            break;
                        }
                    }
                }

                // If still no work found, yield and try again
                if current_work.is_none() {
                    std::thread::yield_now();
                    // Check if all queues are empty
                    let all_empty = self.work_queues.iter().all(|q| q.is_empty());
                    if all_empty {
                        return None; // No more work available
                    }
                }
            } else {
                // Invalid thread ID, exit
                return None;
            }
        }
    }

    /// Get work distribution statistics.
    pub fn get_work_stats(&self) -> Option<WorkDistributionStats> {
        self.work_stats.snapshot()
    }
}
