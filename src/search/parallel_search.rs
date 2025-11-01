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

use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::collections::VecDeque;
use std::time::Duration;
use rayon::{ThreadPool, ThreadPoolBuilder, prelude::*};
use std::env;
use crate::search::ThreadSafeTranspositionTable;
use crate::bitboards::BitboardBoard;
use crate::types::{Player, CapturedPieces, Move};
use crate::moves::MoveGenerator;
use crate::evaluation::PositionEvaluator;
use crate::search::search_engine::SearchEngine;
use crate::search::search_engine::GLOBAL_NODES_SEARCHED;
use crate::time_utils::TimeSource;
use std::thread;
use num_cpus;

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
pub struct WorkStealingQueue {
    /// The underlying deque protected by a mutex.
    queue: Arc<Mutex<VecDeque<WorkUnit>>>,
    
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
    /// Total nanoseconds spent waiting on the queue lock
    lock_wait_ns: AtomicU64,
    /// Number of poison recoveries performed
    poison_recoveries: AtomicU64,
}

impl WorkStealingQueue {
    /// Create a new work-stealing queue.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(WorkQueueStats::default()),
        }
    }
    
    /// Push a work unit to the back of the queue (owner thread operation).
    pub fn push_back(&self, work: WorkUnit) {
        let t0 = std::time::Instant::now();
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.push_back(work);
                self.stats.pushes.fetch_add(1, Ordering::Relaxed);
                let dt = t0.elapsed().as_nanos() as u64;
                self.stats.lock_wait_ns.fetch_add(dt, Ordering::Relaxed);
            }
            Err(poison) => {
                // Recover from poisoned lock to avoid stalling the system
                let mut queue = poison.into_inner();
                queue.push_back(work);
                self.stats.pushes.fetch_add(1, Ordering::Relaxed);
                self.stats.poison_recoveries.fetch_add(1, Ordering::Relaxed);
                let dt = t0.elapsed().as_nanos() as u64;
                self.stats.lock_wait_ns.fetch_add(dt, Ordering::Relaxed);
                crate::debug_utils::debug_log("Recovered from poisoned work queue in push_back");
            }
        }
    }
    
    /// Pop a work unit from the front of the queue (owner thread operation).
    pub fn pop_front(&self) -> Option<WorkUnit> {
        let t0 = std::time::Instant::now();
        match self.queue.lock() {
            Ok(mut queue) => {
                if let Some(work) = queue.pop_front() {
                    self.stats.pops.fetch_add(1, Ordering::Relaxed);
                    let dt = t0.elapsed().as_nanos() as u64;
                    self.stats.lock_wait_ns.fetch_add(dt, Ordering::Relaxed);
                    Some(work)
                } else {
                    let dt = t0.elapsed().as_nanos() as u64;
                    self.stats.lock_wait_ns.fetch_add(dt, Ordering::Relaxed);
                    None
                }
            }
            Err(poison) => {
                let mut queue = poison.into_inner();
                let result = queue.pop_front();
                if result.is_some() {
                    self.stats.pops.fetch_add(1, Ordering::Relaxed);
                }
                self.stats.poison_recoveries.fetch_add(1, Ordering::Relaxed);
                let dt = t0.elapsed().as_nanos() as u64;
                self.stats.lock_wait_ns.fetch_add(dt, Ordering::Relaxed);
                crate::debug_utils::debug_log("Recovered from poisoned work queue in pop_front");
                result
            }
        }
    }
    
    /// Steal a work unit from the back of the queue (other thread operation).
    pub fn steal(&self) -> Option<WorkUnit> {
        if let Ok(mut queue) = self.queue.lock() {
            if let Some(work) = queue.pop_back() {
                self.stats.steals.fetch_add(1, Ordering::Relaxed);
                return Some(work);
            }
        }
        None
    }
    
    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        if let Ok(queue) = self.queue.lock() {
            return queue.is_empty();
        }
        true
    }
    
    /// Get the number of items in the queue.
    pub fn len(&self) -> usize {
        if let Ok(queue) = self.queue.lock() {
            return queue.len();
        }
        0
    }
    
    /// Get statistics for this queue.
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.stats.pushes.load(Ordering::Relaxed),
            self.stats.pops.load(Ordering::Relaxed),
            self.stats.steals.load(Ordering::Relaxed),
            self.stats.lock_wait_ns.load(Ordering::Relaxed),
            self.stats.poison_recoveries.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
impl WorkStealingQueue {
    pub fn test_poison(&self) {
        let _ = std::panic::catch_unwind({
            let queue_arc = self.queue.clone();
            move || {
                let _guard = queue_arc.lock().unwrap();
                panic!("intentional poison");
            }
        });
    }
}

impl Default for WorkStealingQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for work distribution across threads.
#[derive(Default, Debug)]
pub struct WorkDistributionStats {
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
            work_units_per_thread: vec![0; num_threads],
            steal_count_per_thread: vec![0; num_threads],
            total_work_units: 0,
            max_work_units: 0,
            min_work_units: 0,
        }
    }
    
    /// Get the work distribution ratio (max/min).
    pub fn distribution_ratio(&self) -> f64 {
        if self.min_work_units == 0 {
            return f64::INFINITY;
        }
        self.max_work_units as f64 / self.min_work_units as f64
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
}

impl Default for ParallelSearchConfig {
    fn default() -> Self {
        let num_threads = num_cpus::get();
        Self {
            num_threads: num_threads.clamp(1, 32),
            min_depth_parallel: 4,
            enable_parallel: num_threads > 1,
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
        }
    }
    
    /// Set the number of threads, clamping to valid range (1-32).
    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.num_threads = num_threads.clamp(1, 32);
        self.enable_parallel = self.num_threads > 1;
    }
}

/// Thread-local search context for parallel search workers.
///
/// Each thread maintains its own copy of board state, move generator,
/// and evaluator to avoid contention during parallel search.
pub struct ThreadLocalSearchContext {
    /// Thread-local board state (cloned from root position).
    board: BitboardBoard,
    
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
        _captured_pieces: &CapturedPieces,
        _player: Player,
        stop_flag: Option<Arc<AtomicBool>>,
        hash_size_mb: usize,
    ) -> Self {
        Self {
            board: board.clone(),
            move_generator: MoveGenerator::new(),
            evaluator: PositionEvaluator::new(),
            history_table: [[0; 9]; 9],
            killer_moves: [None, None],
            search_engine: SearchEngine::new(stop_flag, hash_size_mb),
        }
    }
    
    /// Get mutable reference to the thread-local board.
    pub fn board_mut(&mut self) -> &mut BitboardBoard {
        &mut self.board
    }
    
    /// Get reference to the thread-local board.
    pub fn board(&self) -> &BitboardBoard {
        &self.board
    }
    
    /// Get mutable reference to the thread-local search engine.
    pub fn search_engine_mut(&mut self) -> &mut SearchEngine {
        &mut self.search_engine
    }
}

/// Synchronization data for YBWC "oldest brother wait" concept.
pub struct YBWCSync {
    /// Whether the oldest brother (first move) has completed.
    oldest_complete: Arc<AtomicBool>,
    
    /// Result from the oldest brother search (score).
    oldest_score: Arc<Mutex<Option<i32>>>,
}

impl YBWCSync {
    fn new() -> Self {
        Self {
            oldest_complete: Arc::new(AtomicBool::new(false)),
            oldest_score: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Mark oldest brother as complete and store its score.
    fn mark_complete(&self, score: i32) {
        if let Ok(mut s) = self.oldest_score.lock() {
            *s = Some(score);
        }
        self.oldest_complete.store(true, Ordering::Release);
    }
    
    /// Check if oldest brother is complete.
    fn is_complete(&self) -> bool {
        self.oldest_complete.load(Ordering::Acquire)
    }
    
    /// Wait for oldest brother to complete (with timeout).
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
    work_stats: Arc<Mutex<WorkDistributionStats>>,
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
    pub fn new(config: ParallelSearchConfig) -> Result<Self, String> {
        if env::var("SHOGI_FORCE_POOL_FAIL").ok().as_deref() == Some("1") {
            return Err("Forced pool failure via SHOGI_FORCE_POOL_FAIL".to_string());
        }
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .stack_size(8 * 1024 * 1024)
            .panic_handler(|_| {
                // Ensure panics in worker threads do not bring the process down; request stop
                crate::debug_utils::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;
        
        // For now, we'll create a placeholder transposition table.
        // This will be replaced with the actual shared TT from SearchEngine in later checkpoints.
        let tt_config = crate::search::TranspositionConfig::performance_optimized();
        let transposition_table = Arc::new(RwLock::new(
            ThreadSafeTranspositionTable::new(tt_config)
        ));
        
        let num_threads = config.num_threads;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();
        
        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag: None,
            work_queues,
            work_stats: Arc::new(Mutex::new(WorkDistributionStats::new(num_threads))),
        })
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
    pub fn new_with_stop_flag(config: ParallelSearchConfig, stop_flag: Option<Arc<AtomicBool>>) -> Result<Self, String> {
        if env::var("SHOGI_FORCE_POOL_FAIL").ok().as_deref() == Some("1") {
            return Err("Forced pool failure via SHOGI_FORCE_POOL_FAIL".to_string());
        }
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .stack_size(8 * 1024 * 1024)
            .panic_handler(|_| {
                crate::debug_utils::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;
        
        let tt_config = crate::search::TranspositionConfig::performance_optimized();
        let transposition_table = Arc::new(RwLock::new(
            ThreadSafeTranspositionTable::new(tt_config)
        ));
        
        let num_threads = config.num_threads;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();
        
        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag,
            work_queues,
            work_stats: Arc::new(Mutex::new(WorkDistributionStats::new(num_threads))),
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
        ThreadLocalSearchContext::new(board, captured_pieces, player, self.stop_flag.clone(), hash_size_mb)
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
                crate::debug_utils::debug_log("Parallel worker thread panicked; requesting stop and continuing on remaining threads");
            })
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;
        
        let num_threads = config.num_threads;
        let work_queues: Vec<Arc<WorkStealingQueue>> = (0..num_threads)
            .map(|_| Arc::new(WorkStealingQueue::new()))
            .collect();
        
        Ok(Self {
            thread_pool,
            config,
            transposition_table,
            stop_flag,
            work_queues,
            work_stats: Arc::new(Mutex::new(WorkDistributionStats::new(num_threads))),
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
        let hash_size_mb = 16; // Default hash size, will be configurable later
        let (tx, rx) = std::sync::mpsc::channel::<(Move, i32, String)>();
        // Reset global nodes counter for this depth
        GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);
        let start_time = TimeSource::now();
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
                let nps = if elapsed > 0 { nodes.saturating_mul(1000) / (elapsed as u64) } else { 0 };
                // Emit real USI info line with score and PV (skip during silent benches)
                if std::env::var("SHOGI_SILENT_BENCH").is_err() {
                    if !best_pv.is_empty() {
                        println!(
                            "info depth {} seldepth {} multipv 1 score cp {} time {} nodes {} nps {} pv {}",
                            depth, depth, // seldepth approximated as depth for now
                            if let Ok(g) = best_for_consumer.lock() { g.1 } else { score },
                            elapsed, nodes, nps, best_pv
                        );
                    }
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
            }
        });

        self.thread_pool.install(|| {
            // Use min_len to encourage chunking and reduce scheduling overhead
            moves
                .par_iter()
                .enumerate()
                .with_min_len((moves.len() / (self.config.num_threads * 2).max(1)).max(1))
                .for_each(|(idx, mv)| {
                // Check stop flag before searching this move
                if search_stop.load(Ordering::Relaxed) {
                    crate::debug_utils::debug_log("Stop flag set before worker started move; skipping");
                    return;
                }

                // Create thread-local context per task (simpler Send bounds for rayon)
                let mut context = ThreadLocalSearchContext::new(
                        board,
                        captured_pieces,
                        player,
                        Some(search_stop.clone()),
                        hash_size_mb,
                    );
                // Provide shared TT to improve ordering/PV consistency across workers
                context.search_engine_mut().set_shared_transposition_table(self.transposition_table.clone());

                // Clone board and apply move
                let mut test_board = board.clone();
                let mut test_captured = captured_pieces.clone();
                if let Some(captured) = test_board.make_move(mv) {
                    test_captured.add_piece(captured.piece_type, player);
                }

                // Search at reduced depth
                let search_depth = if depth > 0 { depth - 1 } else { 0 };

                // Optional test injection: simulate panic in a worker for the first task
                if env::var("SHOGI_FORCE_WORKER_PANIC").ok().as_deref() == Some("1") && idx == 0 {
                    panic!("Forced worker panic for testing");
                }
                if let Some((_, score_child)) = context.search_engine_mut().search_at_depth(
                    &test_board,
                    &test_captured,
                    player.opposite(),
                    search_depth,
                    time_limit_ms,
                    -beta,
                    -alpha,
                ) {
                    // Build PV from child and prefix root move
                    let pv_moves = context
                        .search_engine_mut()
                        .get_pv_for_reporting(&test_board, &test_captured, player.opposite(), search_depth);
                    let mv_root = mv.to_usi_string();
                    let mut pv_string = String::with_capacity(mv_root.len() + pv_moves.len() * 4);
                    pv_string.push_str(&mv_root);
                    for child in pv_moves.iter() {
                        pv_string.push(' ');
                        pv_string.push_str(&child.to_usi_string());
                    }
                    // Check stop flag again in case stop was requested during search
                    if search_stop.load(Ordering::Relaxed) {
                        crate::debug_utils::debug_log("Stop flag observed after move search; reporting partial and returning");
                    }
                    let score = -score_child;
                    let _ = tx.send((mv.clone(), score, pv_string));
                } else {
                    // Still report move completion with no PV
                    crate::debug_utils::debug_log("Search_at_depth returned None; reporting move with no PV");
                    let _ = tx.send((mv.clone(), i32::MIN / 2, mv.to_usi_string()));
                }
            });
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

        // Aggregate queue stats to estimate contention and synchronization overhead
        let mut total_pushes: u64 = 0;
        let mut total_pops: u64 = 0;
        let mut total_steals: u64 = 0;
        let mut total_lock_wait_ns: u64 = 0;
        let mut total_poison_recoveries: u64 = 0;
        for q in &self.work_queues {
            let (pushes, pops, steals, lock_wait_ns, poison_recoveries) = q.get_stats();
            total_pushes += pushes;
            total_pops += pops;
            total_steals += steals;
            total_lock_wait_ns += lock_wait_ns;
            total_poison_recoveries += poison_recoveries;
        }
        let elapsed_ms = bench_start.elapsed().as_millis() as u64;
        let elapsed_ms = elapsed_ms.max(1); // avoid div by zero
        let elapsed_ns = (elapsed_ms as u64) * 1_000_000u64;
        let sync_overhead_pct = if elapsed_ns > 0 {
            (total_lock_wait_ns as f64 / elapsed_ns as f64) * 100.0
        } else { 0.0 };
        crate::debug_utils::debug_log(&format!(
            "PARALLEL_PROF: pushes={}, pops={}, steals={}, lock_wait_ns={}, poison_recoveries={}, sync_overhead~{:.2}%",
            total_pushes, total_pops, total_steals, total_lock_wait_ns, total_poison_recoveries, sync_overhead_pct
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
        if let Some((_, score)) = context.search_engine_mut().search_at_depth(
            board,
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
        let ybwc_sync = YBWCSync::new();
        
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
                        // Wait for oldest brother to complete (with timeout)
                        if sync.wait_for_complete(work.time_limit_ms).is_none() {
                            // Timeout or interrupted, skip this work
                            continue;
                        }
                    }
                }
                
                // Perform search
                if let Some((_, score)) = context.search_engine_mut().search_at_depth(
                    &work.board,
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
                    if let Ok(mut stats) = self.work_stats.lock() {
                        if thread_id < stats.work_units_per_thread.len() {
                            stats.work_units_per_thread[thread_id] += 1;
                            stats.total_work_units += 1;
                        }
                    }
                    
                    return Some((work.move_to_search, final_score));
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
                            if let Ok(mut stats) = self.work_stats.lock() {
                                if thread_id < stats.steal_count_per_thread.len() {
                                    stats.steal_count_per_thread[thread_id] += 1;
                                }
                            }
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
        if let Ok(stats) = self.work_stats.lock() {
            let work_units = stats.work_units_per_thread.clone();
            let steal_count = stats.steal_count_per_thread.clone();
            let max_work = work_units.iter().max().copied().unwrap_or(0);
            let min_work = work_units.iter().filter(|&&x| x > 0).min().copied().unwrap_or(0);
            
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
}

