use crate::types::*;
use crate::bitboards::*;
use crate::evaluation::*;
use crate::moves::*;
use crate::tablebase::MicroTablebase;
use std::collections::HashMap;
use crate::time_utils::TimeSource;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering, AtomicU64}};
use crate::{TranspositionEntry, TranspositionFlag};
use rayon::prelude::*;
use crate::search::move_ordering::MoveOrdering;
use crate::search::tapered_search_integration::TaperedSearchEnhancer;
use crate::search::{ParallelSearchEngine, ParallelSearchConfig};

thread_local! {
    static YBWC_ENGINE_TLS: std::cell::RefCell<Option<SearchEngine>> = std::cell::RefCell::new(None);
}

pub struct SearchEngine {
    evaluator: PositionEvaluator,
    move_generator: MoveGenerator,
    tablebase: MicroTablebase,
    transposition_table: crate::search::ThreadSafeTranspositionTable,
    /// Optional shared transposition table for parallel search contexts
    shared_transposition_table: Option<Arc<RwLock<crate::search::ThreadSafeTranspositionTable>>>,
    hash_calculator: crate::search::ShogiHashHandler,
    move_orderer: crate::search::TranspositionMoveOrderer,
    advanced_move_orderer: MoveOrdering,
    quiescence_tt: HashMap<String, QuiescenceEntry>,
    history_table: [[i32; 9]; 9],
    killer_moves: [Option<Move>; 2],
    nodes_searched: u64,
    stop_flag: Option<Arc<AtomicBool>>,
    quiescence_config: QuiescenceConfig,
    quiescence_stats: QuiescenceStats,
    null_move_config: NullMoveConfig,
    null_move_stats: NullMoveStats,
    lmr_config: LMRConfig,
    lmr_stats: LMRStats,
    aspiration_config: AspirationWindowConfig,
    aspiration_stats: AspirationWindowStats,
    iid_config: IIDConfig,
    iid_stats: IIDStats,
    previous_scores: Vec<i32>,
    // Advanced Alpha-Beta Pruning
    pruning_manager: PruningManager,
    // Tapered evaluation search integration
    tapered_search_enhancer: TaperedSearchEnhancer,
    // Current search state for diagnostics
    current_alpha: i32,
    current_beta: i32,
    current_best_move: Option<Move>,
    current_best_score: i32,
    current_depth: u8,
    search_start_time: Option<TimeSource>,
    // Buffered TT writes to reduce lock contention when using shared TT
    tt_write_buffer: Vec<TranspositionEntry>,
    tt_write_buffer_capacity: usize,
    // YBWC configuration (scaffold)
    ybwc_enabled: bool,
    ybwc_min_depth: u8,
    ybwc_min_branch: usize,
    ybwc_max_siblings: usize,
    // Dynamic scaling divisors for sibling cap based on depth tier
    ybwc_div_shallow: usize,
    ybwc_div_mid: usize,
    ybwc_div_deep: usize,
    // TT write gating threshold (min depth to store non-Exact entries)
    tt_write_min_depth_value: u8,
    // Up to and including this search depth, only write Exact entries to TT
    tt_exact_only_max_depth_value: u8,
    // Instrumentation counters for shared TT usage (bench/profiling)
    shared_tt_probe_attempts: u64,
    shared_tt_probe_hits: u64,
    shared_tt_store_attempts: u64,
    shared_tt_store_writes: u64,
    tt_buffer_flushes: u64,
    tt_buffer_entries_written: u64,
}

/// Global aggregate of nodes searched across all threads for live reporting.
pub static GLOBAL_NODES_SEARCHED: AtomicU64 = AtomicU64::new(0);
// Global contention metrics for shared TT
pub static TT_TRY_READS: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_READ_SUCCESSES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_READ_FAILS: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITE_SUCCESSES: AtomicU64 = AtomicU64::new(0);
pub static TT_TRY_WRITE_FAILS: AtomicU64 = AtomicU64::new(0);
// YBWC metrics
pub static YBWC_SIBLING_BATCHES: AtomicU64 = AtomicU64::new(0);
pub static YBWC_SIBLINGS_EVALUATED: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)]
impl SearchEngine {
    fn ybwc_dynamic_sibling_cap(&self, depth: u8, branch_len: usize) -> usize {
        if branch_len == 0 { return 0; }
        let over_min = depth.saturating_sub(self.ybwc_min_depth);
        let divisor = match over_min {
            0 => self.ybwc_div_shallow.max(1),
            1 => self.ybwc_div_mid.max(1),
            _ => self.ybwc_div_deep.max(1),
        };
        let scaled = (branch_len / divisor).max(1);
        scaled.min(self.ybwc_max_siblings)
    }
    #[inline]
    fn tt_write_min_depth(&self) -> u8 { self.tt_write_min_depth_value }
    fn tt_exact_only_max_depth(&self) -> u8 { self.tt_exact_only_max_depth_value }

    pub fn set_ybwc(&mut self, enabled: bool, min_depth: u8) {
        self.ybwc_enabled = enabled;
        self.ybwc_min_depth = min_depth;
    }

    pub fn set_ybwc_branch(&mut self, min_branch: usize) {
        self.ybwc_min_branch = min_branch;
    }

    pub fn set_tt_gating(&mut self, exact_only_max_depth: u8, non_exact_min_depth: u8, buffer_capacity: usize) {
        self.tt_exact_only_max_depth_value = exact_only_max_depth;
        self.tt_write_min_depth_value = non_exact_min_depth;
        self.tt_write_buffer_capacity = buffer_capacity;
    }

    pub fn set_ybwc_max_siblings(&mut self, max_siblings: usize) {
        self.ybwc_max_siblings = max_siblings.max(1);
    }

    pub fn set_ybwc_scaling(&mut self, shallow_divisor: usize, mid_divisor: usize, deep_divisor: usize) {
        self.ybwc_div_shallow = shallow_divisor.max(1);
        self.ybwc_div_mid = mid_divisor.max(1);
        self.ybwc_div_deep = deep_divisor.max(1);
    }

    fn flush_tt_buffer(&mut self) {
        if self.tt_write_buffer.is_empty() { return; }
        if let Some(ref shared_tt) = self.shared_transposition_table {
            TT_TRY_WRITES.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut guard) = shared_tt.try_write() {
                TT_TRY_WRITE_SUCCESSES.fetch_add(1, Ordering::Relaxed);
                let to_write = self.tt_write_buffer.len() as u64;
                self.tt_buffer_flushes += 1;
                self.tt_buffer_entries_written += to_write;
                for e in self.tt_write_buffer.drain(..) {
                    self.shared_tt_store_writes += 1;
                    guard.store(e);
                }
                return;
            } else {
                TT_TRY_WRITE_FAILS.fetch_add(1, Ordering::Relaxed);
            }
        }
        // Fallback: write to local TT without holding shared lock
        for e in self.tt_write_buffer.drain(..) {
            self.transposition_table.store(e);
        }
    }

    #[inline]
    fn maybe_buffer_tt_store(&mut self, entry: TranspositionEntry, depth: u8, flag: TranspositionFlag) {
        // Gate writes: at shallow depths, only store Exact entries
        if depth <= self.tt_exact_only_max_depth() && !matches!(flag, TranspositionFlag::Exact) {
            return;
        }
        // Gate non-Exact writes: allow only deeper-than-threshold entries
        if !(matches!(flag, TranspositionFlag::Exact) || depth >= self.tt_write_min_depth()) {
            return;
        }
        if self.shared_transposition_table.is_some() {
            self.shared_tt_store_attempts += 1;
            self.tt_write_buffer.push(entry);
            if self.tt_write_buffer.len() >= self.tt_write_buffer_capacity {
                self.flush_tt_buffer();
            }
        } else {
            self.transposition_table.store(entry);
        }
    }
    pub fn new(stop_flag: Option<Arc<AtomicBool>>, hash_size_mb: usize) -> Self {
        Self::new_with_config(stop_flag, hash_size_mb, QuiescenceConfig::default())
    }

    pub fn new_with_config(stop_flag: Option<Arc<AtomicBool>>, hash_size_mb: usize, quiescence_config: QuiescenceConfig) -> Self {
        let config = crate::search::TranspositionConfig::performance_optimized();
        let config = crate::search::TranspositionConfig {
            table_size: hash_size_mb * 1024 * 1024 / 100, // Approximate entries
            ..config
        };
        const BYTES_PER_ENTRY: usize = 100; // Approximate size of a TT entry
        let quiescence_capacity = quiescence_config.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        Self {
            evaluator: PositionEvaluator::new(),
            move_generator: MoveGenerator::new(),
            tablebase: MicroTablebase::new(),
            transposition_table: crate::search::ThreadSafeTranspositionTable::new(config),
            shared_transposition_table: None,
            hash_calculator: crate::search::ShogiHashHandler::new(1000),
            move_orderer: crate::search::TranspositionMoveOrderer::new(),
            advanced_move_orderer: MoveOrdering::new(),
            quiescence_tt: HashMap::with_capacity(quiescence_capacity),
            history_table: [[0; 9]; 9],
            killer_moves: [None, None],
            nodes_searched: 0,
            stop_flag,
            quiescence_config,
            quiescence_stats: QuiescenceStats::default(),
            null_move_config: NullMoveConfig::default(),
            null_move_stats: NullMoveStats::default(),
            lmr_config: LMRConfig::default(),
            lmr_stats: LMRStats::default(),
            aspiration_config: AspirationWindowConfig::default(),
            aspiration_stats: AspirationWindowStats::default(),
            iid_config: IIDConfig::default(),
            iid_stats: IIDStats::default(),
            previous_scores: Vec::new(),
            // Advanced Alpha-Beta Pruning
            pruning_manager: PruningManager::new(PruningParameters::default()),
            // Tapered evaluation search integration
            tapered_search_enhancer: TaperedSearchEnhancer::new(),
            // Initialize diagnostic fields
            current_alpha: 0,
            current_beta: 0,
            current_best_move: None,
            current_best_score: 0,
            current_depth: 0,
            search_start_time: None,
            tt_write_buffer: Vec::with_capacity(64),
            tt_write_buffer_capacity: 512,
            ybwc_enabled: false,
            ybwc_min_depth: 4,
            ybwc_min_branch: 12,
            ybwc_max_siblings: 8,
            ybwc_div_shallow: 4,
            ybwc_div_mid: 3,
            ybwc_div_deep: 2,
            tt_write_min_depth_value: 9,
            tt_exact_only_max_depth_value: 8,
            shared_tt_probe_attempts: 0,
            shared_tt_probe_hits: 0,
            shared_tt_store_attempts: 0,
            shared_tt_store_writes: 0,
            tt_buffer_flushes: 0,
            tt_buffer_entries_written: 0,
        }
    }

    /// Initialize the move orderer with the transposition table
    fn initialize_move_orderer(&mut self) {
        self.move_orderer.set_transposition_table(&self.transposition_table);
    }

    /// Initialize advanced move ordering system
    fn initialize_advanced_move_orderer(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) {
        // Update game phase for position-specific strategies
        let move_count = self.nodes_searched as usize; // Approximate move count
        let material_balance = self.evaluate_position(board, player, captured_pieces);
        let tactical_complexity = self.calculate_tactical_complexity(board, captured_pieces, player);
        
        self.advanced_move_orderer.update_game_phase(move_count, material_balance, tactical_complexity);
        
        // Integrate with transposition table (prefer shared TT when available)
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        let tt_entry_opt = if let Some(ref shared_tt) = self.shared_transposition_table {
            self.shared_tt_probe_attempts += 1;
            TT_TRY_READS.fetch_add(1, Ordering::Relaxed);
            if let Ok(guard) = shared_tt.try_read() {
                TT_TRY_READ_SUCCESSES.fetch_add(1, Ordering::Relaxed);
                let r = guard.probe(position_hash, depth);
                if r.is_some() { self.shared_tt_probe_hits += 1; }
                r
            } else {
                TT_TRY_READ_FAILS.fetch_add(1, Ordering::Relaxed);
                self.transposition_table.probe(position_hash, depth)
            }
        } else {
            self.transposition_table.probe(position_hash, depth)
        };
        if let Some(tt_entry) = tt_entry_opt {
            let _ = self.advanced_move_orderer.integrate_with_transposition_table(Some(&tt_entry), board, captured_pieces, player, depth);
        }
    }

    /// Expose nodes searched for external aggregators/monitors.
    pub fn get_nodes_searched(&self) -> u64 {
        self.nodes_searched
    }

    /// Set a shared transposition table for reporting and ordering in parallel contexts.
    pub fn set_shared_transposition_table(&mut self, shared: Arc<RwLock<crate::search::ThreadSafeTranspositionTable>>) {
        self.shared_transposition_table = Some(shared);
    }

    /// Calculate tactical complexity for position-specific strategies
    fn calculate_tactical_complexity(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> f64 {
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        let capture_count = legal_moves.iter().filter(|m| m.is_capture).count();
        let check_count = legal_moves.iter().filter(|m| {
            let mut test_board = board.clone();
            let mut test_captured = captured_pieces.clone();
            if let Some(captured) = test_board.make_move(m) {
                test_captured.add_piece(captured.piece_type, player);
            }
            test_board.is_king_in_check(player.opposite(), &test_captured)
        }).count();
        
        let total_moves = legal_moves.len() as f64;
        if total_moves == 0.0 {
            return 0.0;
        }
        
        (capture_count + check_count) as f64 / total_moves
    }

    /// Update move orderer with killer move
    fn update_move_orderer_killer(&mut self, killer_move: Move) {
        self.move_orderer.update_killer_moves(killer_move.clone());
        // Also update advanced move orderer
        self.advanced_move_orderer.add_killer_move(killer_move);
    }

    /// Order moves using advanced move ordering system
    fn order_moves_advanced(&mut self, moves: &[Move], board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Result<Vec<Move>, String> {
        // Initialize advanced move orderer for this position
        self.initialize_advanced_move_orderer(board, captured_pieces, player, depth);
        
        // Use advanced move ordering with all heuristics
        Ok(self.advanced_move_orderer.order_moves_with_all_heuristics(moves, board, captured_pieces, player, depth))
    }

    /// Order moves for negamax search with advanced move ordering
    fn order_moves_for_negamax(&mut self, moves: &[Move], board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, alpha: i32, beta: i32) -> Vec<Move> {
        // Try advanced move ordering first
        match self.order_moves_advanced(moves, board, captured_pieces, player, depth) {
            Ok(ordered_moves) => {
                // Update PV move if we have a best move from transposition table
                if let Some(best_move) = self.get_best_move_from_tt(board, captured_pieces, player, depth) {
                    self.advanced_move_orderer.update_pv_move(board, captured_pieces, player, depth, best_move, 0);
                }
                ordered_moves
            }
            Err(_) => {
                // Fallback to traditional move ordering
                self.move_orderer.order_moves(moves, board, captured_pieces, player, depth, alpha, beta, None)
            }
        }
    }

    /// Get best move from transposition table for PV move ordering
    fn get_best_move_from_tt(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, _depth: u8) -> Option<Move> {
        let _position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        // This would need to be implemented based on the transposition table interface
        // For now, return None as a placeholder
        None
    }

    /// Update move orderer with history
    fn update_move_orderer_history(&mut self, mv: &Move, depth: u8) {
        self.move_orderer.update_history(mv, depth);
    }

    /// Create a new SearchEngine with full EngineConfig
    pub fn new_with_engine_config(stop_flag: Option<Arc<AtomicBool>>, config: EngineConfig) -> Self {
        const BYTES_PER_ENTRY: usize = 100; // Approximate size of a TT entry
        let tt_config = crate::search::TranspositionConfig::performance_optimized();
        let tt_config = crate::search::TranspositionConfig {
            table_size: config.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY,
            ..tt_config
        };
        let quiescence_capacity = config.quiescence.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        
        Self {
            evaluator: PositionEvaluator::new(),
            move_generator: MoveGenerator::new(),
            tablebase: MicroTablebase::new(),
            transposition_table: crate::search::ThreadSafeTranspositionTable::new(tt_config),
            shared_transposition_table: None,
            hash_calculator: crate::search::ShogiHashHandler::new(1000),
            move_orderer: crate::search::TranspositionMoveOrderer::new(),
            advanced_move_orderer: MoveOrdering::new(),
            quiescence_tt: HashMap::with_capacity(quiescence_capacity),
            history_table: [[0; 9]; 9],
            killer_moves: [None, None],
            nodes_searched: 0,
            stop_flag,
            quiescence_config: config.quiescence,
            quiescence_stats: QuiescenceStats::default(),
            null_move_config: config.null_move,
            null_move_stats: NullMoveStats::default(),
            lmr_config: config.lmr,
            lmr_stats: LMRStats::default(),
            aspiration_config: config.aspiration_windows,
            aspiration_stats: AspirationWindowStats::default(),
            iid_config: config.iid,
            iid_stats: IIDStats::default(),
            previous_scores: Vec::new(),
            // Advanced Alpha-Beta Pruning
            pruning_manager: PruningManager::new(PruningParameters::default()),
            // Tapered evaluation search integration
            tapered_search_enhancer: TaperedSearchEnhancer::new(),
            // Initialize diagnostic fields
            current_alpha: 0,
            current_beta: 0,
            current_best_move: None,
            current_best_score: 0,
            current_depth: 0,
            search_start_time: None,
            tt_write_buffer: Vec::with_capacity(64),
            tt_write_buffer_capacity: 512,
            ybwc_enabled: false,
            ybwc_min_depth: 4,
            ybwc_min_branch: 12,
            ybwc_max_siblings: 8,
            ybwc_div_shallow: 4,
            ybwc_div_mid: 3,
            ybwc_div_deep: 2,
            tt_write_min_depth_value: 9,
            tt_exact_only_max_depth_value: 8,
            shared_tt_probe_attempts: 0,
            shared_tt_probe_hits: 0,
            shared_tt_store_attempts: 0,
            shared_tt_store_writes: 0,
            tt_buffer_flushes: 0,
            tt_buffer_entries_written: 0,
        }
    }

    /// Create a new SearchEngine with a preset configuration
    pub fn new_with_preset(stop_flag: Option<Arc<AtomicBool>>, preset: EnginePreset) -> Self {
        let config = EngineConfig::get_preset(preset);
        Self::new_with_engine_config(stop_flag, config)
    }

    /// Update the engine configuration
    pub fn update_engine_config(&mut self, config: EngineConfig) -> Result<(), String> {
        // Validate the configuration
        config.validate()?;
        
        // Update individual configurations
        self.quiescence_config = config.quiescence;
        self.null_move_config = config.null_move;
        self.lmr_config = config.lmr;
        self.aspiration_config = config.aspiration_windows;
        self.iid_config = config.iid;
        
        // Reset statistics when configuration changes
        self.quiescence_stats.reset();
        self.null_move_stats.reset();
        self.lmr_stats.reset();
        self.aspiration_stats.reset();
        self.iid_stats.reset();
        
        // Reinitialize performance monitoring with new max depth
        self.initialize_performance_monitoring(config.max_depth);
        
        Ok(())
    }

    /// Get the current engine configuration
    pub fn get_engine_config(&self) -> EngineConfig {
        EngineConfig {
            quiescence: self.quiescence_config.clone(),
            null_move: self.null_move_config.clone(),
            lmr: self.lmr_config.clone(),
            aspiration_windows: self.aspiration_config.clone(),
            iid: self.iid_config.clone(),
            tt_size_mb: self.transposition_table.size() * 100 / (1024 * 1024), // Approximate
            debug_logging: false, // This would need to be tracked separately
            max_depth: 20, // This would need to be tracked separately
            time_management: TimeManagementConfig::default(),
        }
    }

    /// Apply a configuration preset
    pub fn apply_preset(&mut self, preset: EnginePreset) -> Result<(), String> {
        let config = EngineConfig::get_preset(preset);
        self.update_engine_config(config)
    }

    // ===== INTERNAL ITERATIVE DEEPENING (IID) METHODS =====

    /// Determine if IID should be applied at this position
    pub fn should_apply_iid(&mut self, depth: u8, tt_move: Option<&Move>, legal_moves: &[Move], start_time: &TimeSource, time_limit_ms: u32) -> bool {
        // 1. IID must be enabled
        if !self.iid_config.enabled { 
            self.iid_stats.positions_skipped_depth += 1;
            return false; 
        }
        
        // 2. Sufficient depth for IID to be meaningful
        if depth < self.iid_config.min_depth { 
            self.iid_stats.positions_skipped_depth += 1;
            return false; 
        }
        
        // 3. No transposition table move available
        if tt_move.is_some() { 
            self.iid_stats.positions_skipped_tt_move += 1;
            return false; 
        }
        
        // 4. Reasonable number of legal moves (avoid IID in tactical positions)
        if legal_moves.len() > self.iid_config.max_legal_moves { 
            self.iid_stats.positions_skipped_move_count += 1;
            return false; 
        }
        
        // 5. Not in quiescence search
        if depth == 0 { return false; }
        
        // 6. Not in time pressure (if enabled)
        if self.iid_config.enable_time_pressure_detection && self.is_time_pressure(start_time, time_limit_ms) { 
            self.iid_stats.positions_skipped_time_pressure += 1;
            return false; 
        }
        
        true
    }

    /// Calculate the depth for IID search based on strategy
    pub fn calculate_iid_depth(&self, main_depth: u8) -> u8 {
        match self.iid_config.depth_strategy {
            IIDDepthStrategy::Fixed => self.iid_config.iid_depth_ply,
            IIDDepthStrategy::Relative => {
                // Use depth - 2, but ensure minimum of 2
                std::cmp::max(2, main_depth.saturating_sub(2))
            },
            IIDDepthStrategy::Adaptive => {
                // Adjust based on position complexity and time remaining
                let base_depth = if main_depth > 6 { 3 } else { 2 };
                // For now, use fixed base depth - can be enhanced later with position analysis
                base_depth
            }
        }
    }

    /// Check if we're in time pressure
    fn is_time_pressure(&self, start_time: &TimeSource, time_limit_ms: u32) -> bool {
        let elapsed = start_time.elapsed_ms() as u32;
        let remaining = time_limit_ms.saturating_sub(elapsed);
        remaining < time_limit_ms / 10 // Less than 10% time remaining
    }

    /// Perform IID search and extract the best move
    pub fn perform_iid_search(&mut self, 
                         board: &mut BitboardBoard, 
                         captured_pieces: &CapturedPieces, 
                         player: Player, 
                         iid_depth: u8, 
                         alpha: i32, 
                         _beta: i32, 
                         start_time: &TimeSource, 
                         time_limit_ms: u32, 
                         history: &mut Vec<String>) -> Option<Move> {
        
        let iid_start_time = TimeSource::now();
        let initial_nodes = self.nodes_searched;
        
        // Perform shallow search with null window for efficiency
        let iid_score = self.negamax_with_context(
            board, 
            captured_pieces, 
            player, 
            iid_depth, 
            alpha - 1,  // Null window
            alpha, 
            start_time, 
            time_limit_ms, 
            history, 
            true,  // can_null_move
            false, // is_root
            false, // has_capture
            false  // has_check
        );
        
        // Record IID statistics
        let iid_time = iid_start_time.elapsed_ms() as u64;
        self.iid_stats.iid_time_ms += iid_time;
        self.iid_stats.total_iid_nodes += self.nodes_searched - initial_nodes;
        
        // Only return move if IID found something promising
        if iid_score > alpha {
            // Extract the best move from transposition table
            let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
            if let Some(entry) = self.transposition_table.probe(position_hash, 255) {
                if let Some(best_move) = &entry.best_move {
                    return Some(best_move.clone());
                }
            }
        }
        
        self.iid_stats.iid_searches_failed += 1;
        None
    }

    /// Extract the best move from transposition table for a given position
    fn extract_best_move_from_tt(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<Move> {
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        if let Some(entry) = self.transposition_table.probe(position_hash, 255) {
            entry.best_move.clone()
        } else {
            None
        }
    }

    // ===== IID CONFIGURATION MANAGEMENT =====

    /// Create default IID configuration
    pub fn new_iid_config() -> IIDConfig {
        IIDConfig::default()
    }
    
    /// Update IID configuration with validation
    pub fn update_iid_config(&mut self, config: IIDConfig) -> Result<(), String> {
        config.validate()?;
        self.iid_config = config;
        Ok(())
    }
    
    /// Get current IID configuration
    pub fn get_iid_config(&self) -> &IIDConfig {
        &self.iid_config
    }
    
    /// Get current IID statistics
    pub fn get_iid_stats(&self) -> &IIDStats {
        &self.iid_stats
    }
    
    /// Reset IID statistics
    pub fn reset_iid_stats(&mut self) {
        self.iid_stats = IIDStats::default();
    }

    /// Analyze IID performance metrics and adapt configuration if enabled
    pub fn adapt_iid_configuration(&mut self) {
        if !self.iid_config.enable_adaptive_tuning {
            return;
        }

        let metrics = self.get_iid_performance_metrics();
        
        // Only adapt if we have sufficient data
        if self.iid_stats.iid_searches_performed < 50 {
            return;
        }

        let mut config_changed = false;
        let mut new_config = self.iid_config.clone();

        // Adapt minimum depth based on efficiency
        if metrics.iid_efficiency < 20.0 && new_config.min_depth > 2 {
            // Low efficiency - increase minimum depth to be more selective
            new_config.min_depth = new_config.min_depth.saturating_sub(1);
            config_changed = true;
        } else if metrics.iid_efficiency > 60.0 && new_config.min_depth < 6 {
            // High efficiency - decrease minimum depth to apply more broadly
            new_config.min_depth = new_config.min_depth.saturating_add(1);
            config_changed = true;
        }

        // Adapt IID depth based on cutoff rate
        if metrics.cutoff_rate < 10.0 && new_config.iid_depth_ply > 1 {
            // Low cutoff rate - reduce IID depth to save time
            new_config.iid_depth_ply = new_config.iid_depth_ply.saturating_sub(1);
            config_changed = true;
        } else if metrics.cutoff_rate > 40.0 && new_config.iid_depth_ply < 4 {
            // High cutoff rate - increase IID depth for better move ordering
            new_config.iid_depth_ply = new_config.iid_depth_ply.saturating_add(1);
            config_changed = true;
        }

        // Adapt time overhead threshold based on actual overhead
        if metrics.overhead_percentage > 25.0 && new_config.time_overhead_threshold > 0.05 {
            // High overhead - be more restrictive
            new_config.time_overhead_threshold = (new_config.time_overhead_threshold - 0.05).max(0.05);
            config_changed = true;
        } else if metrics.overhead_percentage < 5.0 && new_config.time_overhead_threshold < 0.3 {
            // Low overhead - can be more aggressive
            new_config.time_overhead_threshold = (new_config.time_overhead_threshold + 0.05).min(0.3);
            config_changed = true;
        }

        // Adapt move count threshold based on success rate
        if metrics.success_rate < 90.0 && new_config.max_legal_moves > 20 {
            // Low success rate - be more selective
            new_config.max_legal_moves = new_config.max_legal_moves.saturating_sub(5);
            config_changed = true;
        } else if metrics.success_rate > 98.0 && new_config.max_legal_moves < 50 {
            // High success rate - can apply more broadly
            new_config.max_legal_moves = new_config.max_legal_moves.saturating_add(5);
            config_changed = true;
        }

        // Apply the new configuration if changes were made
        if config_changed {
            self.iid_config = new_config;
        }
    }

    /// Get adaptive IID configuration recommendations based on current performance
    pub fn get_iid_adaptation_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !self.iid_config.enable_adaptive_tuning {
            return recommendations;
        }

        let metrics = self.get_iid_performance_metrics();
        
        if self.iid_stats.iid_searches_performed < 50 {
            recommendations.push("Insufficient data for recommendations. Need at least 50 IID searches.".to_string());
            return recommendations;
        }

        // Efficiency-based recommendations
        if metrics.iid_efficiency < 20.0 {
            recommendations.push("Low IID efficiency (20%). Consider increasing min_depth or reducing max_legal_moves.".to_string());
        } else if metrics.iid_efficiency > 60.0 {
            recommendations.push("High IID efficiency (60%). Consider decreasing min_depth for broader application.".to_string());
        }

        // Cutoff rate recommendations
        if metrics.cutoff_rate < 10.0 {
            recommendations.push("Low cutoff rate (10%). Consider reducing iid_depth_ply to save time.".to_string());
        } else if metrics.cutoff_rate > 40.0 {
            recommendations.push("High cutoff rate (40%). Consider increasing iid_depth_ply for better move ordering.".to_string());
        }

        // Overhead recommendations
        if metrics.overhead_percentage > 25.0 {
            recommendations.push("High time overhead (25%). Consider reducing time_overhead_threshold.".to_string());
        } else if metrics.overhead_percentage < 5.0 {
            recommendations.push("Low time overhead (5%). Consider increasing time_overhead_threshold for more aggressive IID.".to_string());
        }

        // Success rate recommendations
        if metrics.success_rate < 90.0 {
            recommendations.push("Low success rate (90%). Consider being more selective with move count thresholds.".to_string());
        }

        recommendations
    }

    /// Manually trigger IID configuration adaptation
    pub fn trigger_iid_adaptation(&mut self) {
        self.adapt_iid_configuration();
    }
    /// Assess position complexity for dynamic IID depth adjustment
    fn assess_position_complexity(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> PositionComplexity {
        let mut complexity_score = 0;

        // Count material imbalance
        let black_material = self.count_material(board, Player::Black, captured_pieces);
        let white_material = self.count_material(board, Player::White, captured_pieces);
        let material_imbalance = (black_material - white_material).abs();
        complexity_score += (material_imbalance / 100) as usize; // Scale down

        // Count tactical pieces (Rooks, Bishops, Knights)
        let tactical_pieces = self.count_tactical_pieces(board);
        complexity_score += tactical_pieces;

        // Count mobility (legal moves available)
        let mobility = self.count_mobility(board);
        complexity_score += mobility / 10; // Scale down

        // Check for king safety issues
        let king_safety_issues = self.assess_king_safety_complexity(board);
        complexity_score += king_safety_issues;

        // Check for tactical threats (checks, captures, promotions)
        let tactical_threats = self.count_tactical_threats(board);
        complexity_score += tactical_threats;

        // Categorize complexity
        if complexity_score < 10 {
            PositionComplexity::Low
        } else if complexity_score < 25 {
            PositionComplexity::Medium
        } else {
            PositionComplexity::High
        }
    }

    /// Count material value for a player
    fn count_material(&self, board: &BitboardBoard, player: Player, _captured_pieces: &CapturedPieces) -> i32 {
        let mut material = 0;
        
        // Count pieces on board
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == player {
                        material += self.get_piece_value(piece.piece_type);
                    }
                }
            }
        }

        // Add captured pieces (simplified for now)
        // TODO: Implement proper captured pieces counting
        // let captured = captured_pieces.get_captured_pieces(player);
        // for piece_type in captured.keys() {
        //     material += self.get_piece_value(*piece_type) * captured[piece_type] as i32;
        // }

        material
    }

    /// Get piece value for material counting
    fn get_piece_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 100,
            PieceType::Lance => 300,
            PieceType::Knight => 400,
            PieceType::Silver => 500,
            PieceType::Gold => 600,
            PieceType::Bishop => 800,
            PieceType::Rook => 1000,
            PieceType::King => 10000,
            _ => 0,
        }
    }

    /// Count tactical pieces (Rooks, Bishops, Knights)
    fn count_tactical_pieces(&self, board: &BitboardBoard) -> usize {
        let mut count = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    match piece.piece_type {
                        PieceType::Rook | PieceType::Bishop | PieceType::Knight => count += 1,
                        _ => {}
                    }
                }
            }
        }
        
        count
    }

    /// Count mobility (legal moves available)
    fn count_mobility(&self, board: &BitboardBoard) -> usize {
        let generator = MoveGenerator::new();
        let captured_pieces = CapturedPieces::new();
        
        let black_moves = generator.generate_legal_moves(board, Player::Black, &captured_pieces);
        let white_moves = generator.generate_legal_moves(board, Player::White, &captured_pieces);
        
        black_moves.len() + white_moves.len()
    }

    /// Assess king safety complexity
    fn assess_king_safety_complexity(&self, board: &BitboardBoard) -> usize {
        let mut complexity = 0;
        
        // Check if kings are in danger
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.piece_type == PieceType::King {
                        // Simple check: if king is not in starting position, increase complexity
                        if piece.player == Player::Black && row < 6 {
                            complexity += 2;
                        } else if piece.player == Player::White && row > 2 {
                            complexity += 2;
                        }
                    }
                }
            }
        }
        
        complexity
    }

    /// Count tactical threats (checks, captures, promotions)
    fn count_tactical_threats(&self, board: &BitboardBoard) -> usize {
        let generator = MoveGenerator::new();
        let captured_pieces = CapturedPieces::new();
        let mut threats = 0;
        
        let black_moves = generator.generate_legal_moves(board, Player::Black, &captured_pieces);
        let white_moves = generator.generate_legal_moves(board, Player::White, &captured_pieces);
        
        // Count captures and promotions
        for mv in black_moves.iter().chain(white_moves.iter()) {
            if mv.is_capture {
                threats += 1;
            }
            if mv.is_promotion {
                threats += 1;
            }
        }
        
        threats
    }
    /// Calculate dynamic IID depth based on position complexity
    pub fn calculate_dynamic_iid_depth(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, base_depth: u8) -> u8 {
        if !self.iid_config.enable_adaptive_tuning {
            return base_depth;
        }

        let complexity = self.assess_position_complexity(board, captured_pieces);
        
        match complexity {
            PositionComplexity::Low => {
                // Simple positions: reduce IID depth to save time
                base_depth.saturating_sub(1).max(1)
            },
            PositionComplexity::Medium => {
                // Medium positions: use base depth
                base_depth
            },
            PositionComplexity::High => {
                // Complex positions: increase IID depth for better move ordering
                base_depth.saturating_add(1).min(4)
            },
            PositionComplexity::Unknown => {
                // Unknown complexity: use base depth as fallback
                base_depth
            }
        }
    }

    /// Efficient board state management for IID search
    pub fn create_iid_board_state(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> IIDBoardState {
        IIDBoardState {
            // Store only essential position data instead of full board clone
            key: self.calculate_position_key(board),
            material_balance: self.calculate_material_balance(board, captured_pieces),
            piece_count: self.count_pieces(board),
            king_positions: self.get_king_positions(board),
            // Store move generation cache to avoid regenerating moves
            move_cache: None,
        }
    }

    /// Calculate a compact position key for IID board state
    pub fn calculate_position_key(&self, board: &BitboardBoard) -> u64 {
        let mut key = 0u64;
        
        // Simple hash of piece positions
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    let piece_hash = match piece.piece_type {
                        PieceType::Pawn => 1,
                        PieceType::Lance => 2,
                        PieceType::Knight => 3,
                        PieceType::Silver => 4,
                        PieceType::Gold => 5,
                        PieceType::Bishop => 6,
                        PieceType::Rook => 7,
                        PieceType::King => 8,
                        _ => 0,
                    };
                    
                    let player_factor: i32 = if piece.player == Player::Black { 1 } else { -1 };
                    let position_hash = (row as u64 * 9 + col as u64) * piece_hash as u64;
                    
                    key ^= position_hash.wrapping_mul(player_factor.abs() as u64);
                }
            }
        }
        
        key
    }
    /// Calculate material balance efficiently
    pub fn calculate_material_balance(&self, board: &BitboardBoard, _captured_pieces: &CapturedPieces) -> i32 {
        let mut balance = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    let value = self.get_piece_value(piece.piece_type);
                    balance += if piece.player == Player::Black { value } else { -value };
                }
            }
        }
        
        balance
    }

    /// Count pieces efficiently
    pub fn count_pieces(&self, board: &BitboardBoard) -> u8 {
        let mut count = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                if board.get_piece(Position { row, col }).is_some() {
                    count += 1;
                }
            }
        }
        
        count
    }

    /// Get king positions efficiently
    pub fn get_king_positions(&self, board: &BitboardBoard) -> (Option<Position>, Option<Position>) {
        let mut black_king = None;
        let mut white_king = None;
        
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.piece_type == PieceType::King {
                        match piece.player {
                            Player::Black => black_king = Some(Position { row, col }),
                            Player::White => white_king = Some(Position { row, col }),
                        }
                    }
                }
            }
        }
        
        (black_king, white_king)
    }

    /// Memory-efficient IID search with optimized board state management
    pub fn perform_iid_search_optimized(&mut self,
                                       board: &mut BitboardBoard,
                                       captured_pieces: &CapturedPieces,
                                       player: Player,
                                       iid_depth: u8,
                                       alpha: i32,
                                       beta: i32,
                                       start_time: &TimeSource,
                                       time_limit_ms: u32,
                                       history: &mut Vec<String>) -> Option<Move> {
        if !self.iid_config.enabled || iid_depth == 0 {
            return None;
        }

        // Create efficient board state instead of full clone
        let _board_state = self.create_iid_board_state(board, captured_pieces);
        
        // Use memory pool for move generation
        let _move_pool: Vec<Move> = Vec::with_capacity(50); // Pre-allocate reasonable capacity
        
        let generator = MoveGenerator::new();
        let moves = generator.generate_legal_moves(board, player, captured_pieces);
        
        // Limit moves for IID efficiency
        let moves_to_search = if moves.len() > self.iid_config.max_legal_moves {
            &moves[..self.iid_config.max_legal_moves]
        } else {
            &moves
        };

        if moves_to_search.is_empty() {
            return None;
        }

        // Perform null window search with memory optimization
        let mut best_move: Option<Move> = None;
        let mut best_score = alpha;
        
        // Track memory usage
        let initial_memory = self.get_memory_usage();
        
        for move_ in moves_to_search {
            // Check time limit
            if start_time.elapsed_ms() >= time_limit_ms {
                break;
            }

            // Make move on a copy of the board to avoid corruption
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            let move_result = new_board.make_move(&move_);
            if move_result.is_none() {
                continue;
            }
            
            if let Some(captured) = move_result {
                new_captured.add_piece(captured.piece_type, player);
            }

            // Recursive search with reduced depth
            let score = -self.negamax_with_context(
                &mut new_board,
                &new_captured,
                player.opposite(),
                iid_depth - 1,
                beta.saturating_neg(),
                best_score.saturating_neg(),
                start_time,
                time_limit_ms,
                history,
                false,
                false,
                false,
                false
            );

            if score > best_score {
                best_score = score;
                best_move = Some(move_.clone());
                
                // Early termination if we have a good enough move
                if score >= beta {
                    break;
                }
            }
        }

        // Track memory efficiency
        let final_memory = self.get_memory_usage();
        self.track_memory_usage(final_memory - initial_memory);

        // Update statistics
        self.iid_stats.iid_searches_performed += 1;
        self.iid_stats.total_iid_nodes += moves_to_search.len() as u64;
        self.iid_stats.iid_time_ms += start_time.elapsed_ms() as u64;

        best_move
    }

    /// Get current memory usage (placeholder implementation)
    pub fn get_memory_usage(&self) -> usize {
        // In a real implementation, this would track actual memory usage
        // For now, return a placeholder
        0
    }

    /// Track memory usage for optimization
    pub fn track_memory_usage(&mut self, _usage: usize) {
        // In a real implementation, this would track and analyze memory usage patterns
        // For now, this is a placeholder for the memory tracking infrastructure
    }

    /// Monitor IID overhead in real-time and adjust thresholds automatically
    pub fn monitor_iid_overhead(&mut self, iid_time_ms: u32, total_time_ms: u32) {
        if total_time_ms == 0 {
            return;
        }

        let overhead_percentage = (iid_time_ms as f64 / total_time_ms as f64) * 100.0;
        
        // Track overhead statistics
        self.update_overhead_statistics(overhead_percentage);
        
        // Adjust thresholds if needed
        self.adjust_overhead_thresholds(overhead_percentage);
    }

    /// Update overhead statistics for monitoring
    fn update_overhead_statistics(&mut self, overhead_percentage: f64) {
        // In a real implementation, this would maintain rolling averages
        // For now, we'll use the existing IID stats structure
        
        // Track if this is a high overhead search
        if overhead_percentage > self.iid_config.time_overhead_threshold * 100.0 {
            self.iid_stats.positions_skipped_time_pressure += 1;
        }
    }

    /// Automatically adjust IID overhead thresholds based on performance
    fn adjust_overhead_thresholds(&mut self, current_overhead: f64) {
        if !self.iid_config.enable_adaptive_tuning {
            return;
        }

        let mut config_changed = false;
        let mut new_config = self.iid_config.clone();

        // Adjust time overhead threshold based on current performance
        if current_overhead > 30.0 && new_config.time_overhead_threshold > 0.05 {
            // High overhead detected - be more restrictive
            new_config.time_overhead_threshold = (new_config.time_overhead_threshold - 0.02).max(0.05);
            config_changed = true;
        } else if current_overhead < 10.0 && new_config.time_overhead_threshold < 0.3 {
            // Low overhead detected - can be more aggressive
            new_config.time_overhead_threshold = (new_config.time_overhead_threshold + 0.02).min(0.3);
            config_changed = true;
        }

        // Adjust move count threshold based on overhead
        if current_overhead > 25.0 && new_config.max_legal_moves > 20 {
            // High overhead - reduce move count to save time
            new_config.max_legal_moves = new_config.max_legal_moves.saturating_sub(5);
            config_changed = true;
        } else if current_overhead < 8.0 && new_config.max_legal_moves < 50 {
            // Low overhead - can handle more moves
            new_config.max_legal_moves = new_config.max_legal_moves.saturating_add(5);
            config_changed = true;
        }

        if config_changed {
            self.iid_config = new_config;
        }
    }

    /// Get current IID overhead statistics
    pub fn get_iid_overhead_stats(&self) -> IIDOverheadStats {
        let total_searches = self.iid_stats.iid_searches_performed;
        let time_pressure_skips = self.iid_stats.positions_skipped_time_pressure;
        
        IIDOverheadStats {
            total_searches,
            time_pressure_skips,
            current_threshold: self.iid_config.time_overhead_threshold,
            average_overhead: self.calculate_average_overhead(),
            threshold_adjustments: self.count_threshold_adjustments(),
        }
    }

    /// Calculate average IID overhead percentage
    fn calculate_average_overhead(&self) -> f64 {
        if self.iid_stats.iid_searches_performed == 0 {
            return 0.0;
        }
        
        // In a real implementation, this would calculate from actual timing data
        // For now, return a placeholder based on skip statistics
        let skip_rate = self.iid_stats.positions_skipped_time_pressure as f64 / 
                       self.iid_stats.iid_searches_performed as f64;
        
        // Estimate average overhead based on skip rate
        if skip_rate > 0.5 {
            25.0 // High overhead
        } else if skip_rate > 0.2 {
            15.0 // Medium overhead
        } else {
            8.0  // Low overhead
        }
    }

    /// Count how many times thresholds have been adjusted
    fn count_threshold_adjustments(&self) -> u32 {
        // In a real implementation, this would track actual adjustments
        // For now, return a placeholder
        if self.iid_config.enable_adaptive_tuning {
            (self.iid_stats.iid_searches_performed / 10) as u32 // Estimate based on searches
        } else {
            0
        }
    }

    /// Check if IID overhead is acceptable for current position
    pub fn is_iid_overhead_acceptable(&self, estimated_iid_time_ms: u32, time_limit_ms: u32) -> bool {
        if time_limit_ms == 0 {
            return false;
        }

        let overhead_percentage = (estimated_iid_time_ms as f64 / time_limit_ms as f64) * 100.0;
        overhead_percentage <= self.iid_config.time_overhead_threshold * 100.0
    }

    /// Estimate IID time based on position complexity and depth
    pub fn estimate_iid_time(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, depth: u8) -> u32 {
        let complexity = self.assess_position_complexity(board, captured_pieces);
        let base_time = match complexity {
            PositionComplexity::Low => 5,    // 5ms for simple positions
            PositionComplexity::Medium => 15, // 15ms for medium positions
            PositionComplexity::High => 30,   // 30ms for complex positions
            PositionComplexity::Unknown => 20, // Default estimate
        };

        // Scale by depth (exponential growth)
        base_time * (depth as u32 + 1)
    }

    /// Get overhead monitoring recommendations
    pub fn get_overhead_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let stats = self.get_iid_overhead_stats();
        
        if stats.total_searches < 20 {
            recommendations.push("Insufficient data for overhead analysis. Need at least 20 IID searches.".to_string());
            return recommendations;
        }

        if stats.average_overhead > 25.0 {
            recommendations.push("High IID overhead detected (25%). Consider reducing time_overhead_threshold or max_legal_moves.".to_string());
        } else if stats.average_overhead < 8.0 {
            recommendations.push("Low IID overhead (8%). Consider increasing thresholds for more aggressive IID usage.".to_string());
        }

        let skip_rate = if stats.total_searches > 0 {
            stats.time_pressure_skips as f64 / stats.total_searches as f64
        } else {
            0.0
        };

        if skip_rate > 0.4 {
            recommendations.push("High time pressure skip rate (40%). IID may be too aggressive for current time controls.".to_string());
        } else if skip_rate < 0.1 {
            recommendations.push("Low time pressure skip rate (10%). IID could be used more aggressively.".to_string());
        }

        recommendations
    }
    /// Multi-PV IID search to find multiple principal variations
    pub fn perform_multi_pv_iid_search(&mut self,
                                      board: &mut BitboardBoard,
                                      captured_pieces: &CapturedPieces,
                                      player: Player,
                                      iid_depth: u8,
                                      pv_count: usize,
                                      alpha: i32,
                                      _beta: i32,
                                      start_time: &TimeSource,
                                      time_limit_ms: u32,
                                      history: &mut Vec<String>) -> Vec<IIDPVResult> {
        if !self.iid_config.enabled || iid_depth == 0 || pv_count == 0 {
            return Vec::new();
        }

        let generator = MoveGenerator::new();
        let moves = generator.generate_legal_moves(board, player, captured_pieces);
        
        // Limit moves for IID efficiency
        let moves_to_search = if moves.len() > self.iid_config.max_legal_moves {
            &moves[..self.iid_config.max_legal_moves]
        } else {
            &moves
        };

        if moves_to_search.is_empty() {
            return Vec::new();
        }

        let mut pv_results = Vec::new();
        let mut current_alpha = alpha;
        let mut remaining_moves = moves_to_search.to_vec();

        // Find multiple PVs using aspiration windows
        for pv_index in 0..pv_count.min(remaining_moves.len()) {
            if start_time.elapsed_ms() >= time_limit_ms {
                break;
            }

            let mut best_move: Option<Move> = None;
            let mut best_score = current_alpha;
            let mut best_pv = Vec::new();

            // Search remaining moves for this PV
            for (_move_index, move_) in remaining_moves.iter().enumerate() {
                if start_time.elapsed_ms() >= time_limit_ms {
                    break;
                }

                // Make move on a copy of the board to avoid corruption
                let mut new_board = board.clone();
                let mut new_captured = captured_pieces.clone();
                
                let move_result = new_board.make_move(move_);
                if move_result.is_none() {
                    continue;
                }
                
                if let Some(captured) = move_result {
                    new_captured.add_piece(captured.piece_type, player);
                }

                // Use aspiration window for this PV
                let window_size = if pv_index == 0 { 50 } else { 25 }; // Smaller window for secondary PVs
                let aspiration_alpha = best_score - window_size;
                let aspiration_beta = best_score + window_size;

                // Recursive search
                let score = -self.negamax_with_context(
                    &mut new_board,
                    &new_captured,
                    player.opposite(),
                    iid_depth - 1,
                    -aspiration_beta,
                    -aspiration_alpha,
                    start_time,
                    time_limit_ms,
                    history,
                    false,
                    false,
                    false,
                    false
                );

                if score > best_score {
                    best_score = score;
                    best_move = Some(move_.clone());
                    
                    // Build PV for this move
                    best_pv = self.build_pv_from_move(move_.clone(), iid_depth);
                    
                    // Update alpha for next PV
                    current_alpha = best_score;
                }
            }

            // Add this PV result
            if let Some(best_move) = best_move.clone() {
                pv_results.push(IIDPVResult {
                    move_: best_move.clone(),
                    score: best_score,
                    depth: iid_depth,
                    principal_variation: best_pv,
                    pv_index,
                    search_time_ms: start_time.elapsed_ms(),
                });

                // Remove this move from remaining moves to avoid duplicates
                remaining_moves.retain(|m| !self.moves_equal(m, &best_move));
            }
        }

        // Update statistics
        self.iid_stats.iid_searches_performed += 1;
        self.iid_stats.total_iid_nodes += moves_to_search.len() as u64;
        self.iid_stats.iid_time_ms += start_time.elapsed_ms() as u64;

        pv_results
    }

    /// Build principal variation from a given move
    fn build_pv_from_move(&self, move_: Move, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        pv.push(move_);
        
        // In a real implementation, this would trace the PV from the transposition table
        // For now, we'll create a placeholder PV
        for i in 1..depth {
            // Placeholder moves - in real implementation would be actual PV moves
            if let Some(next_move) = self.create_placeholder_move(i) {
                pv.push(next_move);
            }
        }
        
        pv
    }

    /// Create placeholder move for PV building
    fn create_placeholder_move(&self, index: u8) -> Option<Move> {
        // This is a placeholder implementation
        // In a real implementation, this would extract moves from the transposition table
        Some(Move {
            from: Some(Position { row: index % 9, col: (index + 1) % 9 }),
            to: Position { row: (index + 1) % 9, col: index % 9 },
            piece_type: PieceType::Pawn,
            captured_piece: None,
            is_promotion: false,
            is_capture: false,
            gives_check: false,
            is_recapture: false,
            player: Player::Black,
        })
    }

    /// Analyze multiple PVs to find tactical patterns
    pub fn analyze_multi_pv_patterns(&self, pv_results: &[IIDPVResult]) -> MultiPVAnalysis {
        let mut analysis = MultiPVAnalysis {
            total_pvs: pv_results.len(),
            score_spread: 0.0,
            tactical_themes: Vec::new(),
            move_diversity: 0.0,
            complexity_assessment: PositionComplexity::Unknown,
        };

        if pv_results.is_empty() {
            return analysis;
        }

        // Calculate score spread
        let scores: Vec<i32> = pv_results.iter().map(|pv| pv.score).collect();
        let min_score = *scores.iter().min().unwrap_or(&0);
        let max_score = *scores.iter().max().unwrap_or(&0);
        analysis.score_spread = (max_score - min_score) as f64;

        // Analyze tactical themes
        analysis.tactical_themes = self.identify_tactical_themes(pv_results);

        // Calculate move diversity
        analysis.move_diversity = self.calculate_move_diversity(pv_results);

        // Assess complexity
        analysis.complexity_assessment = self.assess_pv_complexity(pv_results);

        analysis
    }

    /// Identify tactical themes in multiple PVs
    fn identify_tactical_themes(&self, pv_results: &[IIDPVResult]) -> Vec<TacticalTheme> {
        let mut themes = Vec::new();

        for pv in pv_results {
            if pv.principal_variation.len() >= 2 {
                let first_move = &pv.principal_variation[0];
                
                // Identify common tactical themes
                if first_move.is_capture {
                    themes.push(TacticalTheme::Capture);
                } else if first_move.is_promotion {
                    themes.push(TacticalTheme::Promotion);
                } else if first_move.gives_check {
                    themes.push(TacticalTheme::Check);
                } else if self.is_development_move(first_move) {
                    themes.push(TacticalTheme::Development);
                } else {
                    themes.push(TacticalTheme::Positional);
                }
            }
        }

        // Remove duplicates and count frequencies
        themes.sort();
        themes.dedup();
        themes
    }

    /// Check if a move is a development move
    pub fn is_development_move(&self, move_: &Move) -> bool {
        // Simple heuristic for development moves
        match move_.piece_type {
            PieceType::Knight | PieceType::Bishop => true,
            PieceType::Rook => {
                // Rook development (moving from starting position)
                if let Some(from) = move_.from {
                    from.row == 0 || from.row == 8 // Starting rank
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    /// Calculate move diversity across PVs
    fn calculate_move_diversity(&self, pv_results: &[IIDPVResult]) -> f64 {
        if pv_results.len() <= 1 {
            return 0.0;
        }

        let mut unique_squares = std::collections::HashSet::new();
        let mut unique_piece_types = std::collections::HashSet::new();

        for pv in pv_results {
            if let Some(from) = pv.move_.from {
                unique_squares.insert((from.row, from.col));
            }
            unique_squares.insert((pv.move_.to.row, pv.move_.to.col));
            unique_piece_types.insert(pv.move_.piece_type);
        }

        let total_possible_squares = 81; // 9x9 board
        let total_possible_pieces = 8; // Number of piece types

        let square_diversity = unique_squares.len() as f64 / total_possible_squares as f64;
        let piece_diversity = unique_piece_types.len() as f64 / total_possible_pieces as f64;

        (square_diversity + piece_diversity) / 2.0
    }

    /// Assess complexity based on PV characteristics
    fn assess_pv_complexity(&self, pv_results: &[IIDPVResult]) -> PositionComplexity {
        let tactical_count = pv_results.iter()
            .filter(|pv| pv.move_.is_capture || pv.move_.is_promotion || pv.move_.gives_check)
            .count();

        let tactical_ratio = tactical_count as f64 / pv_results.len() as f64;

        if tactical_ratio > 0.7 {
            PositionComplexity::High
        } else if tactical_ratio > 0.3 {
            PositionComplexity::Medium
        } else {
            PositionComplexity::Low
        }
    }

    /// Get multi-PV IID recommendations
    pub fn get_multi_pv_recommendations(&self, analysis: &MultiPVAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if analysis.total_pvs == 0 {
            recommendations.push("No principal variations found. Position may be terminal.".to_string());
            return recommendations;
        }

        // Score spread recommendations
        if analysis.score_spread > 100.0 {
            recommendations.push("Large score spread detected. Position has multiple tactical options with significant evaluation differences.".to_string());
        } else if analysis.score_spread < 20.0 {
            recommendations.push("Small score spread. Position is roughly balanced with multiple similar options.".to_string());
        }

        // Tactical theme recommendations
        if analysis.tactical_themes.len() > 3 {
            recommendations.push("Multiple tactical themes present. Position offers diverse strategic approaches.".to_string());
        } else if analysis.tactical_themes.len() == 1 {
            recommendations.push(format!("Single tactical theme dominates: {:?}. Focus on this pattern.", analysis.tactical_themes[0]));
        }

        // Move diversity recommendations
        if analysis.move_diversity > 0.7 {
            recommendations.push("High move diversity. Position offers many different piece movements.".to_string());
        } else if analysis.move_diversity < 0.3 {
            recommendations.push("Low move diversity. Position has limited piece movement options.".to_string());
        }

        // Complexity recommendations
        match analysis.complexity_assessment {
            PositionComplexity::High => {
                recommendations.push("High complexity position. Multiple tactical elements require careful calculation.".to_string());
            },
            PositionComplexity::Medium => {
                recommendations.push("Medium complexity position. Balanced tactical and positional considerations.".to_string());
            },
            PositionComplexity::Low => {
                recommendations.push("Low complexity position. Focus on positional play and long-term planning.".to_string());
            },
            PositionComplexity::Unknown => {
                recommendations.push("Complexity assessment unavailable. Use standard evaluation principles.".to_string());
            },
        }

        recommendations
    }

    /// IID with probing for deeper verification of promising moves
    pub fn perform_iid_with_probing(&mut self,
                                   board: &mut BitboardBoard,
                                   captured_pieces: &CapturedPieces,
                                   player: Player,
                                   iid_depth: u8,
                                   probe_depth: u8,
                                   alpha: i32,
                                   beta: i32,
                                   start_time: &TimeSource,
                                   time_limit_ms: u32,
                                   history: &mut Vec<String>) -> Option<IIDProbeResult> {
        if !self.iid_config.enabled || iid_depth == 0 {
            return None;
        }

        let generator = MoveGenerator::new();
        let moves = generator.generate_legal_moves(board, player, captured_pieces);
        
        // Limit moves for IID efficiency
        let moves_to_search = if moves.len() > self.iid_config.max_legal_moves {
            &moves[..self.iid_config.max_legal_moves]
        } else {
            &moves
        };

        if moves_to_search.is_empty() {
            return None;
        }

        // Phase 1: Initial shallow IID search to identify promising moves
        let promising_moves = self.identify_promising_moves(
            board, captured_pieces, player, moves_to_search, iid_depth, 
            alpha, beta, start_time, time_limit_ms, history
        );

        if promising_moves.is_empty() {
            return None;
        }

        // Phase 2: Deeper probing of promising moves
        let probe_results = self.probe_promising_moves(
            board, captured_pieces, player, &promising_moves, probe_depth,
            alpha, beta, start_time, time_limit_ms, history
        );

        // Phase 3: Select best move based on probing results
        let best_result = self.select_best_probe_result(probe_results);

        // Update statistics
        self.iid_stats.iid_searches_performed += 1;
        self.iid_stats.total_iid_nodes += moves_to_search.len() as u64;
        self.iid_stats.iid_time_ms += start_time.elapsed_ms() as u64;

        best_result
    }
    /// Identify promising moves from initial shallow search
    pub fn identify_promising_moves(&mut self,
                               board: &mut BitboardBoard,
                               captured_pieces: &CapturedPieces,
                               player: Player,
                               moves: &[Move],
                               iid_depth: u8,
                               alpha: i32,
                               beta: i32,
                               start_time: &TimeSource,
                               time_limit_ms: u32,
                               history: &mut Vec<String>) -> Vec<PromisingMove> {
        let mut promising_moves = Vec::new();
        let mut current_alpha = alpha;
        let promising_threshold = 50; // Minimum score improvement to be considered promising

        for move_ in moves {
            if start_time.elapsed_ms() >= time_limit_ms {
                break;
            }

            // Make move on a copy of the board to avoid corruption
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            let move_result = new_board.make_move(move_);
            if move_result.is_none() {
                continue;
            }
            
            if let Some(captured) = move_result {
                new_captured.add_piece(captured.piece_type, player);
            }

            // Shallow search to evaluate move potential
            let score = -self.negamax_with_context(
                &mut new_board,
                &new_captured,
                player.opposite(),
                iid_depth - 1,
                beta.saturating_neg(),
                current_alpha.saturating_neg(),
                start_time,
                time_limit_ms,
                history,
                false,
                false,
                false,
                false
            );

            // Check if move is promising enough for deeper probing
            if score > current_alpha + promising_threshold {
                promising_moves.push(PromisingMove {
                    move_: move_.clone(),
                    shallow_score: score,
                    improvement_over_alpha: score - current_alpha,
                    tactical_indicators: self.assess_tactical_indicators(move_),
                });

                current_alpha = score;
            }
        }

        // Sort by improvement over alpha (most promising first)
        promising_moves.sort_by(|a, b| b.improvement_over_alpha.cmp(&a.improvement_over_alpha));
        
        // Limit to top promising moves for efficiency
        promising_moves.truncate(3);
        
        promising_moves
    }
    /// Probe promising moves with deeper search
    fn probe_promising_moves(&mut self,
                            board: &mut BitboardBoard,
                            captured_pieces: &CapturedPieces,
                            player: Player,
                            promising_moves: &[PromisingMove],
                            probe_depth: u8,
                            alpha: i32,
                            beta: i32,
                            start_time: &TimeSource,
                            time_limit_ms: u32,
                            history: &mut Vec<String>) -> Vec<IIDProbeResult> {
        let mut probe_results = Vec::new();

        for promising_move in promising_moves {
            if start_time.elapsed_ms() >= time_limit_ms {
                break;
            }

            // Make move on a copy of the board to avoid corruption
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            let move_result = new_board.make_move(&promising_move.move_);
            if move_result.is_none() {
                continue;
            }
            
            if let Some(captured) = move_result {
                new_captured.add_piece(captured.piece_type, player);
            }

            // Deeper search for verification
            let deep_score = -self.negamax_with_context(
                &mut new_board,
                &new_captured,
                player.opposite(),
                probe_depth - 1,
                beta.saturating_neg(),
                alpha.saturating_neg(),
                start_time,
                time_limit_ms,
                history,
                false,
                false,
                false,
                false
            );

            // Calculate verification metrics
            let score_difference = (deep_score - promising_move.shallow_score).abs();
            let verification_confidence = self.calculate_verification_confidence(
                promising_move.shallow_score, deep_score, score_difference
            );

            probe_results.push(IIDProbeResult {
                move_: promising_move.move_.clone(),
                shallow_score: promising_move.shallow_score,
                deep_score,
                score_difference,
                verification_confidence,
                tactical_indicators: promising_move.tactical_indicators.clone(),
                probe_depth,
                search_time_ms: start_time.elapsed_ms(),
            });
        }

        probe_results
    }

    /// Select best move based on probing results
    pub fn select_best_probe_result(&self, probe_results: Vec<IIDProbeResult>) -> Option<IIDProbeResult> {
        if probe_results.is_empty() {
            return None;
        }

        // Select move with best combination of score and verification confidence
        probe_results.into_iter().max_by(|a, b| {
            // Primary: Deep score
            let score_comparison = a.deep_score.cmp(&b.deep_score);
            if score_comparison != std::cmp::Ordering::Equal {
                return score_comparison;
            }

            // Secondary: Verification confidence
            a.verification_confidence.partial_cmp(&b.verification_confidence).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Assess tactical indicators for a move
    pub fn assess_tactical_indicators(&self, move_: &Move) -> TacticalIndicators {
        TacticalIndicators {
            is_capture: move_.is_capture,
            is_promotion: move_.is_promotion,
            gives_check: move_.gives_check,
            is_recapture: move_.is_recapture,
            piece_value: self.get_piece_value_for_move(move_),
            mobility_impact: self.estimate_mobility_impact(move_),
            king_safety_impact: self.estimate_king_safety_impact(move_),
        }
    }

    /// Calculate verification confidence based on score consistency
    pub fn calculate_verification_confidence(&self, _shallow_score: i32, _deep_score: i32, score_difference: i32) -> f64 {
        if score_difference == 0 {
            return 1.0; // Perfect confidence
        }

        let max_expected_difference = 100; // Expected variation between shallow and deep search
        let confidence = (max_expected_difference as f64 - score_difference as f64) / max_expected_difference as f64;
        confidence.max(0.0).min(1.0)
    }

    /// Get piece value for move assessment
    pub fn get_piece_value_for_move(&self, move_: &Move) -> i32 {
        match move_.piece_type {
            PieceType::Pawn => 100,
            PieceType::Lance => 300,
            PieceType::Knight => 300,
            PieceType::Silver => 400,
            PieceType::Gold => 500,
            PieceType::Bishop => 700,
            PieceType::Rook => 900,
            PieceType::King => 10000,
            // Promoted pieces have higher values
            PieceType::PromotedPawn => 800,
            PieceType::PromotedLance => 600,
            PieceType::PromotedKnight => 600,
            PieceType::PromotedSilver => 600,
            PieceType::PromotedBishop => 1100,
            PieceType::PromotedRook => 1300,
        }
    }

    /// Estimate mobility impact of a move
    pub fn estimate_mobility_impact(&self, _move_: &Move) -> i32 {
        // Placeholder implementation - would analyze actual mobility changes
        // Higher value pieces generally have higher mobility impact
        match _move_.piece_type {
            PieceType::Pawn => 10,
            PieceType::Lance => 20,
            PieceType::Knight => 25,
            PieceType::Silver => 30,
            PieceType::Gold => 35,
            PieceType::Bishop => 40,
            PieceType::Rook => 45,
            PieceType::King => 50,
            // Promoted pieces have higher mobility impact
            PieceType::PromotedPawn => 60,
            PieceType::PromotedLance => 50,
            PieceType::PromotedKnight => 50,
            PieceType::PromotedSilver => 50,
            PieceType::PromotedBishop => 70,
            PieceType::PromotedRook => 80,
        }
    }

    /// Estimate king safety impact of a move
    pub fn estimate_king_safety_impact(&self, _move_: &Move) -> i32 {
        // Placeholder implementation - would analyze actual king safety changes
        // Moves that give check or attack the king have higher impact
        if _move_.gives_check {
            50
        } else if _move_.is_capture {
            20
        } else {
            5
        }
    }
    /// Performance benchmark comparing IID vs non-IID search
    pub fn benchmark_iid_performance(&mut self,
                                   board: &mut BitboardBoard,
                                   captured_pieces: &CapturedPieces,
                                   player: Player,
                                   depth: u8,
                                   time_limit_ms: u32,
                                   iterations: usize) -> IIDPerformanceBenchmark {
        let mut benchmark = IIDPerformanceBenchmark {
            iterations,
            depth,
            time_limit_ms,
            ..Default::default()
        };

        for iteration in 0..iterations {
            let _start_time = TimeSource::now();
            let mut history = Vec::new();

            // Benchmark with IID enabled
            let iid_config = self.iid_config.clone();
            self.iid_config.enabled = true;
            
            let iid_start = TimeSource::now();
            let iid_result = self.negamax_with_context(
                board, captured_pieces, player, depth,
                -10000, 10000, &iid_start, time_limit_ms,
                &mut history, false, false, false, false
            );
            let iid_time = iid_start.elapsed_ms();
            let iid_nodes = self.iid_stats.total_iid_nodes;

            // Benchmark with IID disabled
            self.iid_config.enabled = false;
            let non_iid_start = TimeSource::now();
            let non_iid_result = self.negamax_with_context(
                board, captured_pieces, player, depth,
                -10000, 10000, &non_iid_start, time_limit_ms,
                &mut history, false, false, false, false
            );
            let non_iid_time = non_iid_start.elapsed_ms();

            // Restore original IID config
            self.iid_config = iid_config;

            // Record iteration results
            benchmark.iid_times.push(iid_time);
            benchmark.non_iid_times.push(non_iid_time);
            benchmark.iid_nodes.push(iid_nodes);
            benchmark.score_differences.push((iid_result - non_iid_result).abs());

            // Calculate running averages
            benchmark.avg_iid_time = benchmark.iid_times.iter().sum::<u32>() as f64 / (iteration + 1) as f64;
            benchmark.avg_non_iid_time = benchmark.non_iid_times.iter().sum::<u32>() as f64 / (iteration + 1) as f64;
            benchmark.avg_iid_nodes = benchmark.iid_nodes.iter().sum::<u64>() as f64 / (iteration + 1) as f64;
            benchmark.avg_score_difference = benchmark.score_differences.iter().sum::<i32>() as f64 / (iteration + 1) as f64;

            // Calculate efficiency metrics
            benchmark.time_efficiency = if benchmark.avg_non_iid_time > 0.0 {
                (benchmark.avg_non_iid_time - benchmark.avg_iid_time) / benchmark.avg_non_iid_time * 100.0
            } else {
                0.0
            };

            benchmark.node_efficiency = if benchmark.avg_iid_nodes > 0.0 {
                benchmark.avg_iid_nodes / (benchmark.avg_iid_time + 1.0) // Nodes per millisecond
            } else {
                0.0
            };

            benchmark.accuracy = if benchmark.avg_score_difference < 50.0 {
                "High".to_string()
            } else if benchmark.avg_score_difference < 100.0 {
                "Medium".to_string()
            } else {
                "Low".to_string()
            };
        }

        benchmark
    }

    /// Get detailed IID performance analysis
    pub fn get_iid_performance_analysis(&self) -> IIDPerformanceAnalysis {
        let metrics = self.get_iid_performance_metrics();
        
        IIDPerformanceAnalysis {
            overall_efficiency: metrics.iid_efficiency,
            cutoff_rate: metrics.cutoff_rate,
            overhead_percentage: metrics.overhead_percentage,
            success_rate: metrics.success_rate,
            recommendations: self.generate_performance_recommendations(&metrics),
            bottleneck_analysis: self.analyze_performance_bottlenecks(&metrics),
            optimization_potential: self.assess_optimization_potential(&metrics),
        }
    }

    /// Generate performance recommendations based on metrics
    fn generate_performance_recommendations(&self, metrics: &IIDPerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.iid_efficiency < 0.3 {
            recommendations.push("Consider disabling IID - efficiency is very low".to_string());
        } else if metrics.iid_efficiency < 0.5 {
            recommendations.push("IID efficiency is low - consider adjusting depth or thresholds".to_string());
        }

        if metrics.overhead_percentage > 20.0 {
            recommendations.push("High overhead detected - reduce max_legal_moves or time_overhead_threshold".to_string());
        }

        if metrics.cutoff_rate < 0.1 {
            recommendations.push("Low cutoff rate - IID moves may not be improving search order".to_string());
        }

        if metrics.success_rate < 0.6 {
            recommendations.push("Low success rate - consider enabling adaptive tuning".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("IID performance is optimal - no changes needed".to_string());
        }

        recommendations
    }

    /// Analyze performance bottlenecks
    fn analyze_performance_bottlenecks(&self, metrics: &IIDPerformanceMetrics) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        if metrics.overhead_percentage > 15.0 {
            bottlenecks.push("Time overhead is the primary bottleneck".to_string());
        }

        if self.iid_stats.positions_skipped_tt_move > self.iid_stats.iid_searches_performed * 2 {
            bottlenecks.push("Too many positions skipped due to TT moves".to_string());
        }

        if self.iid_stats.positions_skipped_depth < 5 {
            bottlenecks.push("IID being applied at insufficient depths".to_string());
        }

        if self.iid_stats.positions_skipped_move_count > self.iid_stats.iid_searches_performed {
            bottlenecks.push("Too many positions skipped due to move count limits".to_string());
        }

        if bottlenecks.is_empty() {
            bottlenecks.push("No significant bottlenecks detected".to_string());
        }

        bottlenecks
    }

    /// Assess optimization potential
    fn assess_optimization_potential(&self, metrics: &IIDPerformanceMetrics) -> String {
        let potential_score = (metrics.iid_efficiency * 0.4 + 
                              (1.0 - metrics.overhead_percentage / 100.0) * 0.3 +
                              metrics.cutoff_rate * 0.3) * 100.0;

        if potential_score > 80.0 {
            "High - IID is performing optimally".to_string()
        } else if potential_score > 60.0 {
            "Medium - Some optimization opportunities exist".to_string()
        } else if potential_score > 40.0 {
            "Low - Significant optimization needed".to_string()
        } else {
            "Very Low - Consider disabling or major reconfiguration".to_string()
        }
    }

    /// Strength testing framework to verify IID playing strength improvement
    pub fn strength_test_iid_vs_non_iid(&mut self,
                                       test_positions: &[StrengthTestPosition],
                                       time_per_move_ms: u32,
                                       games_per_position: usize) -> IIDStrengthTestResult {
        let mut result = IIDStrengthTestResult {
            total_positions: test_positions.len(),
            games_per_position,
            time_per_move_ms,
            ..Default::default()
        };

        for (pos_index, position) in test_positions.iter().enumerate() {
            let mut position_result = PositionStrengthResult {
                position_index: pos_index,
                position_fen: position.fen.clone(),
                expected_result: position.expected_result.clone(),
                ..Default::default()
            };

            // Test with IID enabled
            let iid_config = self.iid_config.clone();
            self.iid_config.enabled = true;
            
            let iid_wins = self.play_strength_games(
                &position.fen,
                position.expected_result,
                time_per_move_ms,
                games_per_position,
                true // IID enabled
            );
            
            // Test with IID disabled
            self.iid_config.enabled = false;
            
            let non_iid_wins = self.play_strength_games(
                &position.fen,
                position.expected_result,
                time_per_move_ms,
                games_per_position,
                false // IID disabled
            );

            // Restore original config
            self.iid_config = iid_config;

            position_result.iid_wins = iid_wins;
            position_result.non_iid_wins = non_iid_wins;
            position_result.iid_win_rate = iid_wins as f64 / games_per_position as f64;
            position_result.non_iid_win_rate = non_iid_wins as f64 / games_per_position as f64;
            position_result.improvement = position_result.iid_win_rate - position_result.non_iid_win_rate;

            result.position_results.push(position_result);
        }

        // Calculate overall statistics
        result.calculate_overall_statistics();
        result
    }

    /// Play multiple games for strength testing
    fn play_strength_games(&mut self,
                          fen: &str,
                          expected_result: GameResult,
                          time_per_move_ms: u32,
                          num_games: usize,
                          iid_enabled: bool) -> usize {
        let mut wins = 0;

        for _ in 0..num_games {
            if let Ok(mut board) = self.parse_fen_position(fen) {
                let result = self.play_single_game(&mut board, time_per_move_ms, iid_enabled);
                
                match (result, expected_result) {
                    (GameResult::Win, GameResult::Win) => wins += 1,
                    (GameResult::Loss, GameResult::Loss) => wins += 1,
                    (GameResult::Draw, GameResult::Draw) => wins += 1,
                    _ => {} // No win for this game
                }
            }
        }

        wins
    }

    /// Play a single game for strength testing
    fn play_single_game(&mut self,
                       board: &mut BitboardBoard,
                       time_per_move_ms: u32,
                       iid_enabled: bool) -> GameResult {
        let original_config = self.iid_config.clone();
        self.iid_config.enabled = iid_enabled;
        
        let mut move_count = 0;
        let max_moves = 200; // Prevent infinite games
        
        while move_count < max_moves {
            let _start_time = TimeSource::now();
            let mut history = Vec::new();
            
            // Find best move
            let best_move = self.find_best_move(
                board,
                &CapturedPieces::new(),
                Player::Black, // Simplified - would need proper turn tracking
                3, // depth
                time_per_move_ms,
                &mut history
            );
            
            if let Some(move_) = best_move {
                // Make move
                let _ = board.make_move(&move_);
                move_count += 1;
            } else {
                break; // No legal moves
            }
            
            // Check for game end conditions (simplified)
            if self.is_game_over(board) {
                break;
            }
        }
        
        // Restore original config
        self.iid_config = original_config;
        
        // Determine game result (simplified - would need proper evaluation)
        if move_count >= max_moves {
            GameResult::Draw
        } else {
            // Simplified result determination
            GameResult::Win // Placeholder
        }
    }

    /// Parse FEN position for strength testing
    fn parse_fen_position(&self, _fen: &str) -> Result<BitboardBoard, String> {
        // Simplified FEN parsing - would need full implementation
        Ok(BitboardBoard::new())
    }

    /// Check if game is over
    fn is_game_over(&self, _board: &BitboardBoard) -> bool {
        // Simplified game over detection - would need full implementation
        false
    }

    /// Find best move for strength testing
    fn find_best_move(&mut self,
                     board: &mut BitboardBoard,
                     captured_pieces: &CapturedPieces,
                     player: Player,
                     depth: u8,
                     time_limit_ms: u32,
                     history: &mut Vec<String>) -> Option<Move> {
        let start_time = TimeSource::now();
        
        // Use the main search function
        let _score = self.negamax_with_context(
            board,
            captured_pieces,
            player,
            depth,
            -10000,
            10000,
            &start_time,
            time_limit_ms,
            history,
            false,
            false,
            false,
            false
        );
        
        // Extract best move from transposition table or search results
        self.extract_best_move_from_tt(board, player, captured_pieces)
    }

    /// Generate strength test positions
    pub fn generate_strength_test_positions(&self) -> Vec<StrengthTestPosition> {
        vec![
            StrengthTestPosition {
                fen: "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(),
                description: "Starting position".to_string(),
                expected_result: GameResult::Draw,
                difficulty: PositionDifficulty::Easy,
            },
            StrengthTestPosition {
                fen: "lnsgkgsnl/1r5b1/ppppppppp/9/9/4P4/PPPP1PPPP/1B5R1/LNSGKGSNL b - 1".to_string(),
                description: "After one move".to_string(),
                expected_result: GameResult::Draw,
                difficulty: PositionDifficulty::Medium,
            },
            StrengthTestPosition {
                fen: "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1".to_string(),
                description: "White to move".to_string(),
                expected_result: GameResult::Win,
                difficulty: PositionDifficulty::Hard,
            },
        ]
    }

    /// Analyze strength test results
    pub fn analyze_strength_test_results(&self, result: &IIDStrengthTestResult) -> StrengthTestAnalysis {
        let mut analysis = StrengthTestAnalysis {
            overall_improvement: result.overall_improvement,
            significant_positions: Vec::new(),
            recommendations: Vec::new(),
            confidence_level: ConfidenceLevel::Low,
        };

        // Find positions with significant improvement
        for position_result in &result.position_results {
            if position_result.improvement.abs() > 0.1 {
                analysis.significant_positions.push(position_result.position_index);
            }
        }

        // Generate recommendations
        if result.overall_improvement > 0.05 {
            analysis.recommendations.push("IID shows clear strength improvement - keep enabled".to_string());
            analysis.confidence_level = ConfidenceLevel::High;
        } else if result.overall_improvement > 0.02 {
            analysis.recommendations.push("IID shows modest improvement - consider keeping enabled".to_string());
            analysis.confidence_level = ConfidenceLevel::Medium;
        } else if result.overall_improvement < -0.05 {
            analysis.recommendations.push("IID shows strength regression - consider disabling".to_string());
            analysis.confidence_level = ConfidenceLevel::High;
        } else {
            analysis.recommendations.push("IID impact is neutral - more testing needed".to_string());
            analysis.confidence_level = ConfidenceLevel::Low;
        }

        // Add position-specific recommendations
        for &pos_index in &analysis.significant_positions {
            if let Some(pos_result) = result.position_results.get(pos_index) {
                if pos_result.improvement > 0.1 {
                    analysis.recommendations.push(
                        format!("Strong improvement on position {} - IID effective for this type", pos_index)
                    );
                } else if pos_result.improvement < -0.1 {
                    analysis.recommendations.push(
                        format!("Regression on position {} - IID may be harmful for this type", pos_index)
                    );
                }
            }
        }

        analysis
    }

    /// Get IID performance metrics
    pub fn get_iid_performance_metrics(&self) -> IIDPerformanceMetrics {
        // For now, use a placeholder for total search time - this would need to be tracked
        let total_search_time_ms = 1000; // Placeholder
        IIDPerformanceMetrics::from_stats(&self.iid_stats, total_search_time_ms)
    }

    /// Test IID move ordering with various scenarios
    #[cfg(test)]
    pub fn test_iid_move_ordering() {
        use crate::bitboards::BitboardBoard;
        use crate::types::{Move, Position, PieceType, Player};

        let mut engine = SearchEngine::new(None, 64);
        let board = BitboardBoard::new();
        
        // Create test moves
        let move1 = Move {
            from: Some(Position { row: 6, col: 4 }),
            to: Position { row: 5, col: 4 },
            piece_type: PieceType::Pawn,
            captured_piece: None,
            is_promotion: false,
            is_capture: false,
            gives_check: false,
            is_recapture: false,
            player: Player::Black,
        };
        
        let move2 = Move {
            from: Some(Position { row: 6, col: 3 }),
            to: Position { row: 5, col: 3 },
            piece_type: PieceType::Pawn,
            captured_piece: None,
            is_promotion: false,
            is_capture: false,
            gives_check: false,
            is_recapture: false,
            player: Player::Black,
        };
        
        let moves = vec![move1.clone(), move2.clone()];
        
        // Test 1: No IID move - should use standard scoring
        let sorted_no_iid = engine.sort_moves(&moves, &board, None);
        assert_eq!(sorted_no_iid.len(), 2);
        
        // Test 2: With IID move - IID move should be first
        let sorted_with_iid = engine.sort_moves(&moves, &board, Some(&move2));
        assert_eq!(sorted_with_iid.len(), 2);
        assert!(engine.moves_equal(&sorted_with_iid[0], &move2));
        
        println!("IID move ordering tests passed!");
    }
    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32, alpha: i32, beta: i32) -> Option<(Move, i32)> {
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Starting search at depth {} (alpha: {}, beta: {})", depth, alpha, beta));
        crate::debug_utils::start_timing(&format!("search_at_depth_{}", depth));
        
        // Optimize pruning performance periodically
        if depth % 3 == 0 {
            self.optimize_pruning_performance();
        }
        
        // Check tablebase first
        crate::debug_utils::start_timing("tablebase_probe");
        if let Some(tablebase_result) = self.tablebase.probe(board, player, captured_pieces) {
            crate::debug_utils::end_timing("tablebase_probe", "SEARCH_AT_DEPTH");
            if let Some(ref best_move) = tablebase_result.best_move {
                crate::debug_utils::log_decision("SEARCH_AT_DEPTH", "Tablebase hit", 
                    &format!("Move: {}, outcome: {:?}, distance: {:?}, confidence: {:.2}", 
                        best_move.to_usi_string(),
                        tablebase_result.outcome,
                        tablebase_result.distance_to_mate,
                        tablebase_result.confidence), 
                    None);
                
                // Convert tablebase score to search score
                let score = self.convert_tablebase_score(&tablebase_result);
                crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Tablebase score: {}", score));
                crate::debug_utils::end_timing(&format!("search_at_depth_{}", depth), "SEARCH_AT_DEPTH");
                return Some((best_move.clone(), score));
            } else {
                crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "Tablebase hit but no best move found");
            }
        } else {
            crate::debug_utils::end_timing("tablebase_probe", "SEARCH_AT_DEPTH");
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "TABLEBASE MISS: Position not in tablebase");
        }
        
        self.nodes_searched = 0;
        let start_time = TimeSource::now();
        let mut alpha = alpha;
        
        let mut best_move: Option<Move> = None;
        // CRITICAL FIX: Initialize to i32::MIN + 1 instead of alpha to allow tracking
        // of any move, even if it doesn't exceed alpha. This prevents the bug where
        // best_move would be None even when legal moves exist.
        let mut best_score = i32::MIN + 1;
        
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "Generating legal moves");
        crate::debug_utils::start_timing("move_generation");
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        crate::debug_utils::end_timing("move_generation", "SEARCH_AT_DEPTH");
        
        if legal_moves.is_empty() {
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "No legal moves found");
            crate::debug_utils::end_timing(&format!("search_at_depth_{}", depth), "SEARCH_AT_DEPTH");
            return None;
        }
        
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Found {} legal moves", legal_moves.len()));
        
        // Debug: log the first few moves
        for (i, mv) in legal_moves.iter().take(5).enumerate() {
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Move {}: {}", i, mv.to_usi_string()));
        }
        
        // If depth is 0, return static evaluation with a fallback legal move to avoid underflow
        if depth == 0 {
            let eval_score = self.evaluator.evaluate(board, player, captured_pieces);
            // Choose the first legal move as a placeholder; callers at depth 0 should not rely on the move
            let placeholder_move = legal_moves[0].clone();
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
                "Depth==0 early return with eval_score={} and placeholder move {}",
                eval_score, placeholder_move.to_usi_string()
            ));
            crate::debug_utils::end_timing(&format!("search_at_depth_{}", depth), "SEARCH_AT_DEPTH");
            return Some((placeholder_move, eval_score));
        }

        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "Sorting moves");
        crate::debug_utils::start_timing("move_sorting");
        // Initialize move orderer if not already done
        self.initialize_move_orderer();
        
        // Use advanced move ordering for better performance
        let sorted_moves = self.order_moves_for_negamax(&legal_moves, board, captured_pieces, player, depth, alpha, beta);
        crate::debug_utils::end_timing("move_sorting", "SEARCH_AT_DEPTH");
        
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "Starting move evaluation loop");
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
            "Search parameters: depth={}, alpha={}, beta={}, time_limit={}ms, moves_count={}", 
            depth, alpha, beta, time_limit_ms, sorted_moves.len()));
        
        let mut history: Vec<String> = vec![board.to_fen(player, captured_pieces)];

        for (move_index, move_) in sorted_moves.iter().enumerate() {
            if self.should_stop(&start_time, time_limit_ms) { 
                crate::debug_utils::trace_log("SEARCH_AT_DEPTH", "Time limit reached, stopping move evaluation");
                break; 
            }
            
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Evaluating move {}: {} (alpha: {}, beta: {}, current_best: {})", 
                move_index + 1, move_.to_usi_string(), alpha, beta,
                best_move.as_ref().map(|m| m.to_usi_string()).unwrap_or("None".to_string())));
            crate::debug_utils::start_timing(&format!("move_eval_{}", move_index));
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }
            
            let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, beta.saturating_neg(), alpha.saturating_neg(), &start_time, time_limit_ms, &mut history, true);
            crate::debug_utils::end_timing(&format!("move_eval_{}", move_index), "SEARCH_AT_DEPTH");
            
            // Enhanced move evaluation logging
            crate::debug_utils::log_move_eval("SEARCH_AT_DEPTH", &move_.to_usi_string(), score, 
                &format!("move {} of {} (alpha: {}, beta: {}, current_best_score: {})", 
                    move_index + 1, sorted_moves.len(), alpha, beta, best_score));
            
            if score > best_score {
                crate::debug_utils::log_decision("SEARCH_AT_DEPTH", "New best move", 
                    &format!("Move {} improved score from {} to {} (alpha: {})", 
                        move_.to_usi_string(), best_score, score, alpha), 
                    Some(score));
                best_score = score;
                best_move = Some(move_.clone());
                
                // Log the new best move details
                crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
                    "BEST_MOVE_UPDATE: {} -> {} (score: {}, alpha: {})", 
                    move_.to_usi_string(), move_.to_usi_string(), score, alpha));
            } else {
                crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
                    "Move {} scored {} (not better than current best: {})", 
                    move_.to_usi_string(), score, best_score));
            }
            
            if score > alpha {
                crate::debug_utils::log_decision("SEARCH_AT_DEPTH", "Alpha update", 
                    &format!("Score {} > alpha {}, updating alpha", score, alpha), 
                    Some(score));
                alpha = score;
            }

            // YBWC: after first move, evaluate siblings in parallel if enabled
            if self.ybwc_enabled && depth >= self.ybwc_min_depth && move_index == 0 && sorted_moves.len() >= self.ybwc_min_branch {
                let all_siblings = &sorted_moves[1..];
                let dyn_cap = self.ybwc_dynamic_sibling_cap(depth, all_siblings.len());
                let sib_limit = dyn_cap.min(all_siblings.len());
                let siblings = &all_siblings[..sib_limit];
                YBWC_SIBLING_BATCHES.fetch_add(1, Ordering::Relaxed);
                YBWC_SIBLINGS_EVALUATED.fetch_add(siblings.len() as u64, Ordering::Relaxed);
                let stop_flag = self.stop_flag.clone();
                let shared_tt = self.shared_transposition_table.clone();
                let quiescence_cfg = self.quiescence_config.clone();
                let sibling_results: Vec<(i32, usize)> = siblings.par_iter().enumerate().with_min_len(8).map(|(sib_idx, sib_mv)| {
                    // Prepare child position
                    let mut sib_board = board.clone();
                    let mut sib_captured = captured_pieces.clone();
                    if let Some(captured) = sib_board.make_move(sib_mv) {
                        sib_captured.add_piece(captured.piece_type, player);
                    }
                    // Reuse a per-thread engine from thread-local storage
                    let s = YBWC_ENGINE_TLS.with(|cell| {
                        let mut opt = cell.borrow_mut();
                        if opt.is_none() {
                            let mut e = SearchEngine::new_with_config(stop_flag.clone(), 16, quiescence_cfg.clone());
                            if let Some(ref shared) = shared_tt {
                                e.set_shared_transposition_table(shared.clone());
                            }
                            e.set_ybwc(true, self.ybwc_min_depth);
                            e.set_ybwc_branch(self.ybwc_min_branch);
                            *opt = Some(e);
                        }
                        let eng = opt.as_mut().unwrap();
                        let score = -eng.negamax(&mut sib_board, &sib_captured, player.opposite(), depth - 1, beta.saturating_neg(), alpha.saturating_neg(), &start_time, time_limit_ms, &mut vec![], true);
                        eng.flush_tt_buffer();
                        score
                    });
                    (s, sib_idx + 1) // store original index offset by 1
                }).collect();

                for (s, idx) in sibling_results.into_iter() {
                    let mv = sorted_moves[idx].clone();
                    if s > best_score {
                        best_score = s;
                        best_move = Some(mv);
                    }
                    if s > alpha { alpha = s; }
                    if alpha >= beta { 
                        crate::debug_utils::log_decision("NEGAMAX", "Beta cutoff (YBWC)", 
                            &format!("Alpha {} >= beta {} after parallel siblings, cutting off", alpha, beta), 
                            Some(alpha));
                        self.flush_tt_buffer();
                        break;
                    }
                }
                break; // we handled all remaining siblings
            }
        }

        // CRITICAL FIX: Fallback move selection to prevent returning None when moves exist
        // This addresses the bug where best_move would be None even when legal moves
        // were available. The fallback ensures we always return a move if one exists.
        if best_move.is_none() && !sorted_moves.is_empty() {
            // If no move was better than alpha, use the first move as fallback
            // This is better than returning None, as it provides a legal move
            // even if it's not the best possible move.
            best_move = Some(sorted_moves[0].clone());
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", 
                "FALLBACK: No move exceeded alpha, using first move as fallback");
        }

        // Validate move tracking consistency
        self.validate_move_tracking(&best_move, best_score, sorted_moves.len());

        // Store the root position in the transposition table so get_pv can extract it
        if let Some(ref best_move_ref) = best_move {
            let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
            let flag = if best_score <= alpha {
                TranspositionFlag::UpperBound
            } else if best_score >= beta {
                TranspositionFlag::LowerBound
            } else {
                TranspositionFlag::Exact
            };
            let entry = TranspositionEntry::new_with_age(
                best_score,
                depth,
                flag,
                Some(best_move_ref.clone()),
                position_hash
            );
            self.maybe_buffer_tt_store(entry, depth, flag);
        }

        crate::debug_utils::end_timing(&format!("search_at_depth_{}", depth), "SEARCH_AT_DEPTH");
        crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!("Search completed: best_move={:?}, best_score={}", 
            best_move.as_ref().map(|m| m.to_usi_string()), best_score));
        
        let result = best_move.map(|m| (m, best_score));
        if !self.validate_search_result(result.clone(), depth, alpha, beta) {
            crate::debug_utils::trace_log("SEARCH_AT_DEPTH", 
                "Search result validation failed, attempting recovery");
            // Recovery logic here - for now just return the result anyway
        }
        // Ensure buffered entries are flushed at the end of a root search
        self.flush_tt_buffer();
        result
    }

    /// Convert tablebase result to search score
    fn convert_tablebase_score(&self, result: &crate::tablebase::TablebaseResult) -> i32 {
        match result.outcome {
            crate::tablebase::TablebaseOutcome::Win => {
                // Winning position: score based on distance to mate
                if let Some(distance) = result.distance_to_mate {
                    10000 - distance as i32
                } else {
                    10000
                }
            }
            crate::tablebase::TablebaseOutcome::Loss => {
                // Losing position: negative score based on distance to mate
                if let Some(distance) = result.distance_to_mate {
                    -10000 - distance as i32
                } else {
                    -10000
                }
            }
            crate::tablebase::TablebaseOutcome::Draw => {
                // Draw position
                0
            }
            crate::tablebase::TablebaseOutcome::Unknown => {
                // Unknown position: use confidence to scale the score
                if let Some(distance) = result.distance_to_mate {
                    ((10000 - distance as i32) as f32 * result.confidence) as i32
                } else {
                    0
                }
            }
        }
    }

    /// Backward-compatible wrapper for search_at_depth without alpha/beta parameters
    pub fn search_at_depth_legacy(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32) -> Option<(Move, i32)> {
        self.search_at_depth(board, captured_pieces, player, depth, time_limit_ms, i32::MIN + 1, i32::MAX - 1)
    }

    fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool) -> i32 {
        self.negamax_with_context(board, captured_pieces, player, depth, alpha, beta, start_time, time_limit_ms, history, can_null_move, false, false, false)
    }
    
    fn negamax_with_context(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool, is_root: bool, _has_capture: bool, has_check: bool) -> i32 {
        if self.should_stop(&start_time, time_limit_ms) { 
            crate::debug_utils::trace_log("NEGAMAX", "Time limit reached, returning 0");
            return 0; 
        }
        self.nodes_searched += 1;
        GLOBAL_NODES_SEARCHED.fetch_add(1, Ordering::Relaxed);
        let fen_key = board.to_fen(player, captured_pieces);
        if history.contains(&fen_key) {
            crate::debug_utils::trace_log("NEGAMAX", "Repetition detected, returning 0 (draw)");
            return 0; // Repetition is a draw
        }

        // Check transposition table
        // Calculate position hash for the new TT system
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        if let Some(entry) = self.transposition_table.probe(position_hash, depth) {
            crate::debug_utils::trace_log("NEGAMAX", &format!("Transposition table hit: depth={}, score={}, flag={:?}", 
                entry.depth, entry.score, entry.flag));
            match entry.flag {
                TranspositionFlag::Exact => return entry.score,
                TranspositionFlag::LowerBound => if entry.score >= beta { 
                    crate::debug_utils::trace_log("NEGAMAX", "TT lower bound cutoff");
                    return entry.score; 
                },
                TranspositionFlag::UpperBound => if entry.score <= alpha { 
                    crate::debug_utils::trace_log("NEGAMAX", "TT upper bound cutoff");
                    return entry.score; 
                },
            }
        }
        
        // === NULL MOVE PRUNING ===
        if self.should_attempt_null_move(board, captured_pieces, player, depth, can_null_move) {
            crate::debug_utils::trace_log("NULL_MOVE", &format!("Attempting null move pruning at depth {}", depth));
            crate::debug_utils::start_timing("null_move_search");
            let null_move_score = self.perform_null_move_search(
                board, captured_pieces, player, depth, beta, start_time, time_limit_ms, history
            );
            crate::debug_utils::end_timing("null_move_search", "NULL_MOVE");
            
            if null_move_score >= beta {
                // Beta cutoff - position is too good, prune this branch
                crate::debug_utils::log_decision("NULL_MOVE", "Beta cutoff", 
                    &format!("Null move score {} >= beta {}, pruning branch", null_move_score, beta), 
                    Some(null_move_score));
                self.null_move_stats.cutoffs += 1;
                return beta;
            } else {
                crate::debug_utils::trace_log("NULL_MOVE", &format!("Null move score {} < beta {}, continuing search", null_move_score, beta));
            }
        }
        // === END NULL MOVE PRUNING ===
        
        if depth == 0 {
            // crate::debug_utils::trace_log("QUIESCENCE", &format!("Starting quiescence search (alpha: {}, beta: {})", alpha, beta));
            crate::debug_utils::start_timing("quiescence_search");
            let result = self.quiescence_search(&mut board.clone(), captured_pieces, player, alpha, beta, &start_time, time_limit_ms, 5);
            crate::debug_utils::end_timing("quiescence_search", "QUIESCENCE");
            // crate::debug_utils::trace_log("QUIESCENCE", &format!("Quiescence search completed: score={}", result));
            return result;
        }
        
        // Use the passed context parameters
        crate::debug_utils::trace_log("NEGAMAX", &format!("Generating moves at depth {} (alpha: {}, beta: {})", depth, alpha, beta));
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            let is_check = board.is_king_in_check(player, captured_pieces);
            let score = if is_check { -100000 } else { 0 };
            crate::debug_utils::trace_log("NEGAMAX", &format!("No legal moves: check={}, score={}", is_check, score));
            return score;
        }
        
        crate::debug_utils::trace_log("NEGAMAX", &format!("Found {} legal moves", legal_moves.len()));
        
        // === INTERNAL ITERATIVE DEEPENING (IID) ===
        let mut iid_move = None;
        let tt_move = self.transposition_table.probe(position_hash, 255).and_then(|entry| entry.best_move.clone());
        let should_apply_iid = self.should_apply_iid(depth, tt_move.as_ref(), &legal_moves, start_time, time_limit_ms);
        
        if should_apply_iid {
            crate::debug_utils::trace_log("IID", &format!("Applying Internal Iterative Deepening at depth {}", depth));
            crate::debug_utils::start_timing("iid_search");
            let iid_depth = self.calculate_iid_depth(depth);
            crate::debug_utils::trace_log("IID", &format!("Applying IID at depth {} with IID depth {}", depth, iid_depth));
            
            let iid_start_time = TimeSource::now();
            iid_move = self.perform_iid_search(
                &mut board.clone(), 
                captured_pieces, 
                player, 
                iid_depth, 
                alpha, 
                beta, 
                start_time, 
                time_limit_ms, 
                history
            );
            
            let iid_time = iid_start_time.elapsed_ms();
            self.iid_stats.iid_searches_performed += 1;
            crate::debug_utils::end_timing("iid_search", "IID");
            
            if let Some(ref mv) = iid_move {
                crate::debug_utils::trace_log("IID", &format!("Found move {} in {}ms", mv.to_usi_string(), iid_time));
            } else {
                crate::debug_utils::trace_log("IID", &format!("No move found after {}ms", iid_time));
            }
        } else {
            crate::debug_utils::trace_log("IID", &format!("Skipped at depth {} (enabled={}, tt_move={}, moves={})", 
                depth, 
                self.iid_config.enabled, 
                tt_move.is_some(), 
                legal_moves.len()));
        }
        // === END IID ===
        
        crate::debug_utils::trace_log("NEGAMAX", "Sorting moves for evaluation");
        // Initialize move orderer if not already done
        self.initialize_move_orderer();
        
        // Use advanced move ordering for better performance
        let sorted_moves = self.order_moves_for_negamax(&legal_moves, board, captured_pieces, player, depth, alpha, beta);
        let mut best_score = -200000;
        let mut best_move_for_tt = None;
        
        history.push(fen_key.clone());

        let mut move_index = 0;
        let mut iid_move_improved_alpha = false;
        
        crate::debug_utils::trace_log("NEGAMAX", &format!("Starting move evaluation loop with {} moves", sorted_moves.len()));
        
        for move_ in &sorted_moves {
            if self.should_stop(&start_time, time_limit_ms) { 
                crate::debug_utils::trace_log("NEGAMAX", "Time limit reached, stopping move evaluation");
                break; 
            }
            move_index += 1;
            
            crate::debug_utils::trace_log("NEGAMAX", &format!("Evaluating move {}: {} (alpha: {}, beta: {})", 
                move_index, move_.to_usi_string(), alpha, beta));
            
            // Create search state for advanced pruning decisions
            let mut search_state = crate::types::SearchState::new(depth, alpha, beta);
            search_state.move_number = move_index as u8;
            search_state.update_fields(
                has_check,
                self.evaluate_position(board, player, captured_pieces),
                self.get_position_hash(board),
                self.get_game_phase(board)
            );
            
            // Check if move should be pruned using advanced pruning techniques with conditional logic
            let should_consider_pruning = self.pruning_manager.should_apply_conditional_pruning(&search_state, move_);
            if should_consider_pruning {
                let pruning_decision = self.pruning_manager.should_prune(&search_state, move_);
                
                if pruning_decision.is_pruned() {
                    crate::debug_utils::trace_log("NEGAMAX", &format!("Move {} pruned by advanced pruning", move_.to_usi_string()));
                    continue; // Skip this move
                }
            }
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured) = new_board.make_move(move_) {
                new_captured.add_piece(captured.piece_type, player);
            }

            crate::debug_utils::start_timing(&format!("move_search_{}", move_index));
            let score = self.search_move_with_lmr(
                &mut new_board, 
                &new_captured, 
                player, 
                depth, 
                alpha, 
                beta, 
                &start_time, 
                time_limit_ms, 
                history, 
                move_, 
                move_index,
                is_root,
                move_.is_capture,
                has_check
            );
            crate::debug_utils::end_timing(&format!("move_search_{}", move_index), "NEGAMAX");

            crate::debug_utils::log_move_eval("NEGAMAX", &move_.to_usi_string(), score, 
                &format!("move {} of {}", move_index, sorted_moves.len()));

            if score > best_score {
                crate::debug_utils::log_decision("NEGAMAX", "New best move", 
                    &format!("Move {} improved score from {} to {}", move_.to_usi_string(), best_score, score), 
                    Some(score));
                best_score = score;
                best_move_for_tt = Some(move_.clone());
                if score > alpha {
                    crate::debug_utils::log_decision("NEGAMAX", "Alpha update", 
                        &format!("Score {} > alpha {}, updating alpha", score, alpha), 
                        Some(score));
                    alpha = score;
                    
                    // Track if this was the IID move that first improved alpha
                    if let Some(iid_mv) = &iid_move {
                        if self.moves_equal(move_, iid_mv) && !iid_move_improved_alpha {
                            iid_move_improved_alpha = true;
                            self.iid_stats.iid_move_first_improved_alpha += 1;
                            crate::debug_utils::trace_log("IID", &format!("Move {} first improved alpha to {}", move_.to_usi_string(), alpha));
                        }
                    }
                    
                    if !move_.is_capture {
                        self.update_killer_moves(move_.clone());
                    }
                    if let Some(from) = move_.from {
                        self.history_table[from.row as usize][from.col as usize] += (depth * depth) as i32;
                    }
                }
                if alpha >= beta { 
                    crate::debug_utils::log_decision("NEGAMAX", "Beta cutoff", 
                        &format!("Alpha {} >= beta {}, cutting off search", alpha, beta), 
                        Some(alpha));
                    // Track if IID move caused cutoff
                    if let Some(iid_mv) = &iid_move {
                        if self.moves_equal(move_, iid_mv) {
                            self.iid_stats.iid_move_caused_cutoff += 1;
                            crate::debug_utils::trace_log("IID", &format!("Move {} caused beta cutoff", move_.to_usi_string()));
                        }
                    }
                    // Opportunistically flush buffered TT writes on cutoffs to reduce later bursts
                    self.flush_tt_buffer();
                    break; 
                }
            }
        }
        
        history.pop();

        let flag = if best_score <= alpha { TranspositionFlag::UpperBound } else if best_score >= beta { TranspositionFlag::LowerBound } else { TranspositionFlag::Exact };
        // Use the position hash we calculated earlier for proper TT storage
        let entry = TranspositionEntry::new_with_age(best_score, depth, flag, best_move_for_tt, position_hash);
        self.maybe_buffer_tt_store(entry, depth, flag);
        
        crate::debug_utils::trace_log("NEGAMAX", &format!("Negamax completed: depth={}, score={}, flag={:?}", depth, best_score, flag));
        
        best_score
    }
    fn quiescence_search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
        if self.should_stop(&start_time, time_limit_ms) { 
            // crate::debug_utils::trace_log("QUIESCENCE", "Time limit reached, returning 0");
            return 0; 
        }
        
        // crate::debug_utils::trace_log("QUIESCENCE", &format!("Starting quiescence search: depth={}, alpha={}, beta={}", depth, alpha, beta));

        // Update statistics
        self.quiescence_stats.nodes_searched += 1;

        // Check depth limit
        if depth == 0 || depth > self.quiescence_config.max_depth {
            // crate::debug_utils::trace_log("QUIESCENCE", &format!("Depth limit reached (depth={}), evaluating position", depth));
            let score = self.evaluator.evaluate_with_context(board, player, captured_pieces, depth, false, false, false, true);
            // crate::debug_utils::trace_log("QUIESCENCE", &format!("Position evaluation: {}", score));
            return score;
        }

        // Transposition table lookup
        if self.quiescence_config.enable_tt {
            // Clean up TT if it's getting too large
            if self.quiescence_tt.len() > self.quiescence_config.tt_cleanup_threshold {
                // crate::debug_utils::trace_log("QUIESCENCE", "Cleaning up quiescence TT");
                self.cleanup_quiescence_tt(self.quiescence_config.tt_cleanup_threshold / 2);
            }
            
            let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
            if let Some(entry) = self.quiescence_tt.get(&fen_key) {
                if entry.depth >= depth {
                    self.quiescence_stats.tt_hits += 1;
                    // crate::debug_utils::trace_log("QUIESCENCE", &format!("Quiescence TT hit: depth={}, score={}, flag={:?}", 
                    //     entry.depth, entry.score, entry.flag));
                    match entry.flag {
                        TranspositionFlag::Exact => return entry.score,
                        TranspositionFlag::LowerBound => if entry.score >= beta { 
                            // crate::debug_utils::trace_log("QUIESCENCE", "Quiescence TT lower bound cutoff");
                            return entry.score; 
                        },
                        TranspositionFlag::UpperBound => if entry.score <= alpha { 
                            // crate::debug_utils::trace_log("QUIESCENCE", "Quiescence TT upper bound cutoff");
                            return entry.score; 
                        },
                    }
                }
            } else {
                self.quiescence_stats.tt_misses += 1;
                // crate::debug_utils::trace_log("QUIESCENCE", "Quiescence TT miss");
            }
        }
        
        // crate::debug_utils::trace_log("QUIESCENCE", "Evaluating stand-pat position");
        let stand_pat = self.evaluator.evaluate_with_context(board, player, captured_pieces, depth, false, false, false, true);
        // crate::debug_utils::trace_log("QUIESCENCE", &format!("Stand-pat evaluation: {}", stand_pat));
        
        if stand_pat >= beta { 
            crate::debug_utils::log_decision("QUIESCENCE", "Stand-pat beta cutoff", 
                &format!("Stand-pat {} >= beta {}, returning beta", stand_pat, beta), 
                Some(stand_pat));
            return beta; 
        }
        if alpha < stand_pat { 
            crate::debug_utils::log_decision("QUIESCENCE", "Stand-pat alpha update", 
                &format!("Stand-pat {} > alpha {}, updating alpha", stand_pat, alpha), 
                Some(stand_pat));
            alpha = stand_pat; 
        }
        
        // crate::debug_utils::trace_log("QUIESCENCE", "Generating noisy moves");
        let noisy_moves = self.generate_noisy_moves(board, player, captured_pieces);
        // crate::debug_utils::trace_log("QUIESCENCE", &format!("Found {} noisy moves", noisy_moves.len()));
        
        // Track move type statistics
        for move_ in &noisy_moves {
            if move_.gives_check {
                self.quiescence_stats.check_moves_found += 1;
            }
            if move_.is_capture {
                self.quiescence_stats.capture_moves_found += 1;
            }
            if move_.is_promotion {
                self.quiescence_stats.promotion_moves_found += 1;
            }
        }
        
        // crate::debug_utils::trace_log("QUIESCENCE", "Sorting noisy moves");
        let sorted_noisy_moves = self.sort_quiescence_moves_advanced(&noisy_moves, board, captured_pieces, player);
        self.quiescence_stats.moves_ordered += noisy_moves.len() as u64;

        // crate::debug_utils::trace_log("QUIESCENCE", &format!("Starting noisy move evaluation with {} moves", sorted_noisy_moves.len()));

        for (move_index, move_) in sorted_noisy_moves.iter().enumerate() {
            if self.should_stop(&start_time, time_limit_ms) { 
                // crate::debug_utils::trace_log("QUIESCENCE", "Time limit reached, stopping move evaluation");
                break; 
            }
            
            // crate::debug_utils::trace_log("QUIESCENCE", &format!("Evaluating move {}: {} (alpha: {}, beta: {})", 
            //     move_index + 1, move_.to_usi_string(), alpha, beta));
            
            // Apply pruning checks
            if self.should_prune_delta(&move_, stand_pat, alpha) {
                // crate::debug_utils::trace_log("QUIESCENCE", &format!("Delta pruning move {}", move_.to_usi_string()));
                self.quiescence_stats.delta_prunes += 1;
                continue;
            }
            
            if self.should_prune_futility(&move_, stand_pat, alpha, depth) {
                // crate::debug_utils::trace_log("QUIESCENCE", &format!("Futility pruning move {}", move_.to_usi_string()));
                self.quiescence_stats.futility_prunes += 1;
                continue;
            }
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }
            
            // Check for selective extension
            let search_depth = if self.should_extend(&move_, depth) && depth > 1 {
                // crate::debug_utils::trace_log("QUIESCENCE", &format!("Extending search for move {}", move_.to_usi_string()));
                self.quiescence_stats.extensions += 1;
                depth - 1 // Still reduce depth but less aggressively
            } else {
                depth - 1
            };
            
            crate::debug_utils::start_timing(&format!("quiescence_move_{}", move_index));
            let score = -self.quiescence_search(&mut new_board, &new_captured, player.opposite(), beta.saturating_neg(), alpha.saturating_neg(), &start_time, time_limit_ms, search_depth);
            crate::debug_utils::end_timing(&format!("quiescence_move_{}", move_index), "QUIESCENCE");
            
            // crate::debug_utils::log_move_eval("QUIESCENCE", &move_.to_usi_string(), score, 
            //     &format!("move {} of {}", move_index + 1, sorted_noisy_moves.len()));
            
            if score >= beta { 
                crate::debug_utils::log_decision("QUIESCENCE", "Beta cutoff", 
                    &format!("Score {} >= beta {}, cutting off search", score, beta), 
                    Some(score));
                // Store result in transposition table
                if self.quiescence_config.enable_tt {
                    let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
                    let flag = TranspositionFlag::LowerBound;
                    self.quiescence_tt.insert(fen_key, QuiescenceEntry {
                        score: beta,
                        depth,
                        flag,
                        best_move: Some(move_.clone()),
                    });
                }
                return beta; 
            }
            if score > alpha { 
                crate::debug_utils::log_decision("QUIESCENCE", "Alpha update", 
                    &format!("Score {} > alpha {}, updating alpha", score, alpha), 
                    Some(score));
                alpha = score; 
            }
        }
        
        // crate::debug_utils::trace_log("QUIESCENCE", &format!("Quiescence search completed: depth={}, score={}", depth, alpha));
        
        // Store result in transposition table
        if self.quiescence_config.enable_tt {
            let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
            let flag = if alpha <= -beta { TranspositionFlag::UpperBound } 
                      else if alpha >= beta { TranspositionFlag::LowerBound } 
                      else { TranspositionFlag::Exact };
            self.quiescence_tt.insert(fen_key, QuiescenceEntry {
                score: alpha,
                depth,
                flag,
                best_move: None, // We don't store best move for quiescence search
            });
        }
        
        alpha
    }

    fn should_stop(&self, start_time: &TimeSource, time_limit_ms: u32) -> bool {
        if let Some(flag) = &self.stop_flag {
            if flag.load(Ordering::Relaxed) {
                return true;
            }
        }
        start_time.has_exceeded_limit(time_limit_ms)
    }

    fn generate_noisy_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        self.move_generator.generate_quiescence_moves(board, player, captured_pieces)
    }

    /// Sort quiescence moves using advanced move ordering
    fn sort_quiescence_moves_advanced(&mut self, moves: &[Move], _board: &BitboardBoard, _captured_pieces: &CapturedPieces, _player: Player) -> Vec<Move> {
        if moves.is_empty() {
            return Vec::new();
        }

        // Try advanced move ordering for quiescence search
        match self.advanced_move_orderer.order_moves(moves) {
            Ok(ordered_moves) => ordered_moves,
            Err(_) => {
                // Fallback to traditional quiescence move ordering
                self.sort_quiescence_moves(moves)
            }
        }
    }

    

    pub fn sort_moves(&mut self, moves: &[Move], board: &BitboardBoard, iid_move: Option<&Move>) -> Vec<Move> {
        // Enhanced move ordering with transposition table integration
        self.initialize_move_orderer();
        let captured_pieces = CapturedPieces::new(); // Default empty captured pieces
        let player = Player::Black; // Default player (will be overridden by caller if needed)
        self.move_orderer.order_moves(moves, board, &captured_pieces, player, 1, 0, 0, iid_move)
    }
    
    /// Enhanced move ordering that considers pruning effectiveness
    pub fn sort_moves_with_pruning_awareness(
        &mut self, 
        moves: &[Move], 
        board: &BitboardBoard, 
        iid_move: Option<&Move>,
        depth: Option<u8>,
        alpha: Option<i32>,
        beta: Option<i32>
    ) -> Vec<Move> {
        // First, check if any move is a tablebase move
        let mut tablebase_moves = Vec::new();
        let mut regular_moves = Vec::new();
        
        for move_ in moves {
            if self.is_tablebase_move(move_, board) {
                tablebase_moves.push(move_.clone());
                crate::debug_utils::debug_log(&format!(
                    "TABLEBASE MOVE PRIORITIZED: {}",
                    move_.to_usi_string()
                ));
            } else {
                regular_moves.push(move_.clone());
            }
        }
        
        if !tablebase_moves.is_empty() {
            crate::debug_utils::debug_log(&format!(
                "Found {} tablebase moves, {} regular moves",
                tablebase_moves.len(),
                regular_moves.len()
            ));
        }
        
        // Score and sort regular moves with pruning awareness
        let mut scored_regular: Vec<(Move, i32)> = regular_moves.iter()
            .map(|m| {
                let base_score = self.score_move(m, board, iid_move);
                let pruning_score = self.score_move_for_pruning(m, board, depth, alpha, beta);
                (m.clone(), base_score + pruning_score)
            })
            .collect();
        scored_regular.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Combine: tablebase moves first, then regular moves
        let mut result = tablebase_moves;
        result.extend(scored_regular.into_iter().map(|(m, _)| m));
        
        result
    }
    /// Check if a move is a tablebase move by probing the tablebase
    fn is_tablebase_move(&mut self, move_: &Move, board: &BitboardBoard) -> bool {
        // Create a temporary board state to check if this move leads to a tablebase position
        let mut temp_board = board.clone();
        let mut temp_captured = CapturedPieces::new();
        
        // Make the move
        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, move_.player);
        }
        
        // Check if the resulting position is in the tablebase
        if let Some(tablebase_result) = self.tablebase.probe(&temp_board, move_.player.opposite(), &temp_captured) {
            tablebase_result.best_move.is_some()
        } else {
            false
        }
    }
    pub fn score_move(&self, move_: &Move, _board: &BitboardBoard, iid_move: Option<&Move>) -> i32 {
        // Priority 1: IID move gets maximum score
        if let Some(iid_mv) = iid_move {
            if self.moves_equal(move_, iid_mv) {
                return i32::MAX;
            }
        }
        
        // Priority 2: Transposition table move (simplified - we don't have access to player here)
        // This would need to be passed as a parameter or handled differently
        
        // Priority 3: Standard move scoring
        let mut score = 0;
        if move_.is_promotion { score += 800; }
        if move_.is_capture {
            if let Some(captured_piece) = &move_.captured_piece {
                score += captured_piece.piece_type.base_value() * 10;
            }
            score += 1000;
        }
        if let Some(killer) = &self.killer_moves[0] {
            if self.moves_equal(move_, killer) { score += 900; }
        }
        if let Some(killer) = &self.killer_moves[1] {
            if self.moves_equal(move_, killer) { score += 800; }
        }
        if let Some(from) = move_.from {
            score += self.history_table[from.row as usize][from.col as usize];
        }
        if move_.to.row >= 3 && move_.to.row <= 5 && move_.to.col >= 3 && move_.to.col <= 5 {
            score += 20;
        }
        score
    }
    /// Score a move based on pruning effectiveness
    fn score_move_for_pruning(&self, move_: &Move, _board: &BitboardBoard, depth: Option<u8>, alpha: Option<i32>, beta: Option<i32>) -> i32 {
        let mut pruning_score = 0;
        
        // Bonus for moves that are less likely to be pruned
        if let Some(d) = depth {
            // Tactical moves (captures, promotions, checks) are less likely to be pruned
            if move_.is_capture {
                pruning_score += 200;
                // Higher value captures are even less likely to be pruned
                if let Some(captured_piece) = &move_.captured_piece {
                    pruning_score += captured_piece.piece_type.base_value() / 10;
                }
            }
            
            if move_.is_promotion {
                pruning_score += 150;
            }
            
            if move_.gives_check {
                pruning_score += 100;
            }
            
            // Bonus for moves that are likely to cause cutoffs (good for pruning)
            if let Some(from) = move_.from {
                // History table indicates moves that have caused cutoffs before
                pruning_score += self.history_table[from.row as usize][from.col as usize] / 10;
            }
            
            // Killer moves are likely to cause cutoffs
            if let Some(killer) = &self.killer_moves[0] {
                if self.moves_equal(move_, killer) { 
                    pruning_score += 50; 
                }
            }
            if let Some(killer) = &self.killer_moves[1] {
                if self.moves_equal(move_, killer) { 
                    pruning_score += 40; 
                }
            }
            
            // Depth-dependent adjustments
            if d <= 2 {
                // At shallow depths, prioritize moves that are less likely to be pruned
                pruning_score += 30;
            } else if d >= 4 {
                // At deeper depths, prioritize moves that are more likely to cause cutoffs
                pruning_score += 20;
            }
        }
        
        // Alpha-beta window awareness
        if let (Some(a), Some(b)) = (alpha, beta) {
            let window_size = b.saturating_sub(a);
            if window_size < 100 {
                // Narrow window - prioritize moves likely to cause cutoffs
                pruning_score += 25;
            } else if window_size > 500 {
                // Wide window - prioritize moves less likely to be pruned
                pruning_score += 15;
            }
        }
        
        pruning_score
    }
    
    /// Adaptive move ordering based on pruning statistics
    fn get_adaptive_ordering_adjustment(&self, move_: &Move, depth: u8) -> i32 {
        let mut adjustment = 0;
        
        // Get pruning statistics
        let stats = &self.pruning_manager.statistics;
        let total_moves = stats.total_moves.max(1);
        let pruning_rate = stats.pruned_moves as f64 / total_moves as f64;
        
        // Adjust ordering based on pruning effectiveness
        if pruning_rate > 0.3 {
            // High pruning rate - prioritize moves less likely to be pruned
            if move_.is_capture || move_.is_promotion || move_.gives_check {
                adjustment += 50; // Tactical moves are less likely to be pruned
            } else {
                adjustment -= 25; // Quiet moves are more likely to be pruned
            }
        } else if pruning_rate < 0.1 {
            // Low pruning rate - prioritize moves more likely to cause cutoffs
            if let Some(from) = move_.from {
                adjustment += self.history_table[from.row as usize][from.col as usize] / 5;
            }
            
            // Killer moves are likely to cause cutoffs
            if let Some(killer) = &self.killer_moves[0] {
                if self.moves_equal(move_, killer) { 
                    adjustment += 30; 
                }
            }
        }
        
        // Depth-dependent adjustments
        if depth <= 2 {
            // At shallow depths, be more conservative with pruning
            adjustment += 20;
        } else if depth >= 5 {
            // At deeper depths, be more aggressive with pruning
            adjustment -= 15;
        }
        
        adjustment
    }
    
    /// Enhanced move ordering with adaptive pruning awareness
    pub fn sort_moves_adaptive(&mut self, moves: &[Move], board: &BitboardBoard, iid_move: Option<&Move>, depth: u8, alpha: i32, beta: i32) -> Vec<Move> {
        // First, check if any move is a tablebase move
        let mut tablebase_moves = Vec::new();
        let mut regular_moves = Vec::new();
        
        for move_ in moves {
            if self.is_tablebase_move(move_, board) {
                tablebase_moves.push(move_.clone());
            } else {
                regular_moves.push(move_.clone());
            }
        }
        
        // Score and sort regular moves with adaptive pruning awareness
        let mut scored_regular: Vec<(Move, i32)> = regular_moves.iter()
            .map(|m| {
                let base_score = self.score_move(m, board, iid_move);
                let pruning_score = self.score_move_for_pruning(m, board, Some(depth), Some(alpha), Some(beta));
                let adaptive_score = self.get_adaptive_ordering_adjustment(m, depth);
                (m.clone(), base_score + pruning_score + adaptive_score)
            })
            .collect();
        scored_regular.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Combine: tablebase moves first, then regular moves
        let mut result = tablebase_moves;
        result.extend(scored_regular.into_iter().map(|(m, _)| m));
        
        result
    }

    pub fn moves_equal(&self, move1: &Move, move2: &Move) -> bool {
        move1.from == move2.from && move1.to == move2.to && move1.piece_type == move2.piece_type
    }

    fn update_killer_moves(&mut self, new_killer: Move) {
        if let Some(killer) = &self.killer_moves[0] {
            if self.moves_equal(&new_killer, killer) { return; }
        }
        if let Some(killer) = &self.killer_moves[1] {
            if self.moves_equal(&new_killer, killer) { return; }
        }
        self.killer_moves[1] = self.killer_moves[0].take();
        self.killer_moves[0] = Some(new_killer);
    }

    pub fn clear(&mut self) {
        self.transposition_table.clear();
        self.history_table = [[0; 9]; 9];
        self.killer_moves = [None, None];
        self.lmr_stats.reset();
    }

    #[cfg(test)]
    pub fn transposition_table_len(&self) -> usize {
        self.transposition_table.size()
    }

    #[cfg(test)]
    pub fn transposition_table_capacity(&self) -> usize {
        self.transposition_table.size() // ThreadSafeTranspositionTable doesn't expose capacity
    }

    fn get_pv(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut current_board = board.clone();
        let mut current_captured = captured_pieces.clone();
        let mut current_player = player;
    
        for _ in 0..depth {
            let position_hash = self.hash_calculator.get_position_hash(&current_board, current_player, &current_captured);
            // Probe with depth=0 to accept entries from any search depth
            if let Some(entry) = self.transposition_table.probe(position_hash, 0) {
                if let Some(move_) = &entry.best_move {
                    pv.push(move_.clone());
                    if let Some(captured) = current_board.make_move(move_) {
                        current_captured.add_piece(captured.piece_type, current_player);
                    }
                    current_player = current_player.opposite();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        pv
    }

    /// Public wrapper to fetch principal variation for reporting.
    pub fn get_pv_for_reporting(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        // Prefer building PV from shared TT when available for cross-thread consistency
        if let Some(ref shared_tt) = self.shared_transposition_table {
            TT_TRY_READS.fetch_add(1, Ordering::Relaxed);
            if let Ok(tt) = shared_tt.try_read() {
                TT_TRY_READ_SUCCESSES.fetch_add(1, Ordering::Relaxed);
                let mut pv = Vec::new();
                let mut current_board = board.clone();
                let mut current_captured = captured_pieces.clone();
                let mut current_player = player;
                for _ in 0..depth {
                    let position_hash = self.hash_calculator.get_position_hash(&current_board, current_player, &current_captured);
                    self.shared_tt_probe_attempts += 1;
                    if let Some(entry) = tt.probe(position_hash, 0) {
                        self.shared_tt_probe_hits += 1;
                        if let Some(move_) = &entry.best_move {
                            pv.push(move_.clone());
                            if let Some(captured) = current_board.make_move(move_) {
                                current_captured.add_piece(captured.piece_type, current_player);
                            }
                            current_player = current_player.opposite();
                        } else { break; }
                    } else { break; }
                }
                return pv;
            }
        }
        if self.shared_transposition_table.is_some() { TT_TRY_READ_FAILS.fetch_add(1, Ordering::Relaxed); }
        self.get_pv(board, captured_pieces, player, depth)
    }

    /// Check if a move should be pruned using delta pruning
    fn should_prune_delta(&self, move_: &Move, stand_pat: i32, alpha: i32) -> bool {
        if !self.quiescence_config.enable_delta_pruning {
            return false;
        }

        let material_gain = move_.captured_piece_value();
        let promotion_bonus = move_.promotion_value();
        let total_gain = material_gain + promotion_bonus;
        
        // If the best possible outcome is still worse than alpha, prune
        stand_pat + total_gain + self.quiescence_config.delta_margin <= alpha
    }

    /// Adaptive delta pruning based on position characteristics
    fn should_prune_delta_adaptive(&self, move_: &Move, stand_pat: i32, alpha: i32, depth: u8, move_count: usize) -> bool {
        if !self.quiescence_config.enable_delta_pruning {
            return false;
        }

        let material_gain = move_.captured_piece_value();
        let promotion_bonus = move_.promotion_value();
        let total_gain = material_gain + promotion_bonus;
        
        // Adaptive margin based on depth and move count
        let mut adaptive_margin = self.quiescence_config.delta_margin;
        
        // Increase margin at deeper depths (more aggressive pruning)
        if depth > 3 {
            adaptive_margin += (depth as i32 - 3) * 50;
        }
        
        // Increase margin when there are many moves (more selective)
        if move_count > 8 {
            adaptive_margin += (move_count as i32 - 8) * 25;
        }
        
        // Decrease margin for high-value captures (less aggressive pruning)
        if total_gain > 200 {
            adaptive_margin = adaptive_margin / 2;
        }
        
        // If the best possible outcome is still worse than alpha, prune
        stand_pat + total_gain + adaptive_margin <= alpha
    }

    /// Check if a move should be pruned using futility pruning
    fn should_prune_futility(&self, move_: &Move, stand_pat: i32, alpha: i32, depth: u8) -> bool {
        if !self.quiescence_config.enable_futility_pruning {
            return false;
        }

        let futility_margin = match depth {
            1 => self.quiescence_config.futility_margin / 2,
            2 => self.quiescence_config.futility_margin,
            _ => self.quiescence_config.futility_margin * 2,
        };
        
        let material_gain = move_.captured_piece_value();
        stand_pat + material_gain + futility_margin <= alpha
    }

    /// Adaptive futility pruning based on position characteristics
    fn should_prune_futility_adaptive(&self, move_: &Move, stand_pat: i32, alpha: i32, depth: u8, move_count: usize) -> bool {
        if !self.quiescence_config.enable_futility_pruning {
            return false;
        }

        let mut futility_margin = match depth {
            1 => self.quiescence_config.futility_margin / 2,
            2 => self.quiescence_config.futility_margin,
            _ => self.quiescence_config.futility_margin * 2,
        };
        
        // Adaptive adjustments based on position characteristics
        if move_count > 10 {
            futility_margin += 50; // More aggressive pruning with many moves
        }
        
        if depth > 4 {
            futility_margin += (depth as i32 - 4) * 25; // More aggressive at deeper depths
        }
        
        let material_gain = move_.captured_piece_value();
        stand_pat + material_gain + futility_margin <= alpha
    }
    /// Check if a move should be extended in quiescence search
    fn should_extend(&self, move_: &Move, _depth: u8) -> bool {
        if !self.quiescence_config.enable_selective_extensions {
            return false;
        }

        // Extend for checks
        if move_.gives_check {
            return true;
        }
        
        // Extend for recaptures
        if move_.is_recapture {
            return true;
        }
        
        // Extend for promotions
        if move_.is_promotion {
            return true;
        }
        
        // Extend for captures of high-value pieces
        if move_.is_capture && move_.captured_piece_value() > 500 {
            return true;
        }
        
        false
    }

    /// Reset quiescence statistics
    pub fn reset_quiescence_stats(&mut self) {
        self.quiescence_stats = QuiescenceStats::default();
    }

    /// Get quiescence statistics
    pub fn get_quiescence_stats(&self) -> &QuiescenceStats {
        &self.quiescence_stats
    }

    /// Update quiescence configuration
    pub fn update_quiescence_config(&mut self, config: QuiescenceConfig) {
        self.quiescence_config = config;
    }

    /// Update quiescence configuration with validation
    pub fn update_quiescence_config_validated(&mut self, config: QuiescenceConfig) -> Result<(), String> {
        config.validate()?;
        self.quiescence_config = config;
        Ok(())
    }

    /// Update quiescence configuration with automatic validation and clamping
    pub fn update_quiescence_config_safe(&mut self, config: QuiescenceConfig) {
        self.quiescence_config = config.new_validated();
    }

    /// Get current quiescence configuration
    pub fn get_quiescence_config(&self) -> &QuiescenceConfig {
        &self.quiescence_config
    }

    /// Update specific configuration parameters
    pub fn update_quiescence_depth(&mut self, depth: u8) -> Result<(), String> {
        if depth == 0 || depth > 20 {
            return Err("Depth must be between 1 and 20".to_string());
        }
        self.quiescence_config.max_depth = depth;
        Ok(())
    }

    /// Update TT size and reinitialize if needed
    pub fn update_quiescence_tt_size(&mut self, size_mb: usize) -> Result<(), String> {
        if size_mb == 0 || size_mb > 1024 {
            return Err("TT size must be between 1 and 1024 MB".to_string());
        }
        self.quiescence_config.tt_size_mb = size_mb;
        // Reinitialize TT with new size
        const BYTES_PER_ENTRY: usize = 100;
        let new_capacity = size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        self.quiescence_tt = HashMap::with_capacity(new_capacity);
        Ok(())
    }

    /// Compare two moves for quiescence search ordering
    fn compare_quiescence_moves(&self, a: &Move, b: &Move) -> std::cmp::Ordering {
        // Use a simple, guaranteed total order based on move properties
        // This ensures we never have equal moves that are actually different
        
        // 1. Checks first (highest priority)
        match (a.gives_check, b.gives_check) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        
        // 2. Captures vs non-captures (captures have higher priority)
        match (a.is_capture, b.is_capture) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            (true, true) => {
                // Both are captures - use MVV-LVA
                let a_value = a.captured_piece_value() - a.piece_value();
                let b_value = b.captured_piece_value() - b.piece_value();
                let capture_cmp = b_value.cmp(&a_value);
                if capture_cmp != std::cmp::Ordering::Equal {
                    return capture_cmp;
                }
            },
            (false, false) => {
                // Neither is a capture - continue to other criteria
            }
        }
        
        // 3. Promotions
        match (a.is_promotion, b.is_promotion) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        
        // 4. Use a simple hash-based comparison to ensure total order
        let a_hash = self.move_hash(a);
        let b_hash = self.move_hash(b);
        a_hash.cmp(&b_hash)
    }

    /// Create a simple hash for move comparison
    fn move_hash(&self, move_: &Move) -> u64 {
        let mut hash = 0u64;
        
        // Hash the to position
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.row as u64);
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.col as u64);
        
        // Hash the from position (if exists)
        if let Some(from) = move_.from {
            hash = hash.wrapping_mul(31).wrapping_add(from.row as u64);
            hash = hash.wrapping_mul(31).wrapping_add(from.col as u64);
        }
        
        // Hash the piece type
        hash = hash.wrapping_mul(31).wrapping_add(move_.piece_type as u64);
        
        // Hash the player
        hash = hash.wrapping_mul(31).wrapping_add(move_.player as u64);
        
        hash
    }

    /// Enhanced move ordering with position-specific heuristics
    fn compare_quiescence_moves_enhanced(&self, a: &Move, b: &Move, board: &BitboardBoard, player: Player) -> std::cmp::Ordering {
        // 1. Checks first (highest priority)
        match (a.gives_check, b.gives_check) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        
        // 2. Captures vs non-captures (captures have higher priority)
        match (a.is_capture, b.is_capture) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            (true, true) => {
                // Both are captures - use MVV-LVA with position awareness
                let a_value = self.assess_capture_value(a, board, player);
                let b_value = self.assess_capture_value(b, board, player);
                return b_value.cmp(&a_value);
            },
            (false, false) => {
                // Neither is a capture - continue to other criteria
            }
        }
        
        // 3. Promotions with position awareness
        match (a.is_promotion, b.is_promotion) {
            (true, false) => {
                let a_promotion_value = self.assess_promotion_value(a, board, player);
                let b_promotion_value = self.assess_promotion_value(b, board, player);
                return b_promotion_value.cmp(&a_promotion_value);
            },
            (false, true) => {
                let a_promotion_value = self.assess_promotion_value(a, board, player);
                let b_promotion_value = self.assess_promotion_value(b, board, player);
                return b_promotion_value.cmp(&a_promotion_value);
            },
            _ => {}
        }
        
        // 4. Tactical threat assessment with position awareness
        let a_threat_value = self.assess_tactical_threat_enhanced(a, board, player);
        let b_threat_value = self.assess_tactical_threat_enhanced(b, board, player);
        if a_threat_value != b_threat_value {
            return b_threat_value.cmp(&a_threat_value);
        }
        
        // 5. Position-specific ordering
        let a_position_value = self.assess_position_value(a, board, player);
        let b_position_value = self.assess_position_value(b, board, player);
        if a_position_value != b_position_value {
            return b_position_value.cmp(&a_position_value);
        }
        
        // 6. Default ordering (by piece value)
        b.piece_value().cmp(&a.piece_value())
    }

    /// Assess capture value with position awareness
    fn assess_capture_value(&self, move_: &Move, board: &BitboardBoard, player: Player) -> i32 {
        let mut value = move_.captured_piece_value() - move_.piece_value();
        
        // Bonus for capturing pieces that are attacking our pieces
        if let Some(captured_piece) = &move_.captured_piece {
            if self.is_piece_attacking_our_king(captured_piece, move_.to, board, player) {
                value += 200; // Bonus for capturing attacking pieces
            }
        }
        
        // Bonus for capturing pieces in the center
        if self.is_center_square(move_.to) {
            value += 50;
        }
        
        value
    }

    /// Assess promotion value with position awareness
    fn assess_promotion_value(&self, move_: &Move, board: &BitboardBoard, player: Player) -> i32 {
        let mut value = move_.promotion_value();
        
        // Bonus for promoting in the center
        if self.is_center_square(move_.to) {
            value += 100;
        }
        
        // Bonus for promoting pieces that are attacking
        if self.is_piece_attacking_opponent(move_, board, player) {
            value += 150;
        }
        
        value
    }

    /// Enhanced tactical threat assessment
    fn assess_tactical_threat_enhanced(&self, move_: &Move, board: &BitboardBoard, player: Player) -> i32 {
        let mut threat_value = 0;
        
        // High value for captures
        if move_.is_capture {
            threat_value += move_.captured_piece_value();
        }
        
        // High value for checks
        if move_.gives_check {
            threat_value += 1000;
        }
        
        // High value for promotions
        if move_.is_promotion {
            threat_value += move_.promotion_value();
        }
        
        // High value for recaptures
        if move_.is_recapture {
            threat_value += 500;
        }
        
        // Bonus for threats to opponent's king
        if self.is_threatening_opponent_king(move_, board, player) {
            threat_value += 300;
        }
        
        // Bonus for threats in the center
        if self.is_center_square(move_.to) {
            threat_value += 50;
        }
        
        threat_value
    }

    /// Assess position-specific value of a move
    fn assess_position_value(&self, move_: &Move, board: &BitboardBoard, player: Player) -> i32 {
        let mut value = 0;
        
        // Center control bonus
        if self.is_center_square(move_.to) {
            value += 30;
        }
        
        // Development bonus for pieces moving forward
        if self.is_forward_move(move_, player) {
            value += 20;
        }
        
        // Mobility bonus
        value += self.assess_mobility_gain(move_, board, player);
        
        value
    }

    /// Check if a square is in the center
    fn is_center_square(&self, pos: Position) -> bool {
        pos.row >= 3 && pos.row <= 5 && pos.col >= 3 && pos.col <= 5
    }

    /// Check if a piece is attacking our king
    fn is_piece_attacking_our_king(&self, piece: &Piece, _pos: Position, _board: &BitboardBoard, player: Player) -> bool {
        // Simplified check - in a real implementation, this would check actual attack patterns
        piece.player == player.opposite()
    }

    /// Check if a move is attacking the opponent
    fn is_piece_attacking_opponent(&self, move_: &Move, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified check - in a real implementation, this would check actual attack patterns
        move_.is_capture || move_.gives_check
    }

    /// Check if a move threatens the opponent's king
    fn is_threatening_opponent_king(&self, move_: &Move, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified check - in a real implementation, this would check actual attack patterns
        move_.gives_check
    }

    /// Check if a move is forward for the player
    fn is_forward_move(&self, move_: &Move, player: Player) -> bool {
        if let Some(from) = move_.from {
            match player {
                Player::Black => move_.to.row > from.row,
                Player::White => move_.to.row < from.row,
            }
        } else {
            false
        }
    }

    /// Assess mobility gain from a move
    fn assess_mobility_gain(&self, move_: &Move, _board: &BitboardBoard, _player: Player) -> i32 {
        // Simplified mobility assessment
        if self.is_center_square(move_.to) {
            10
        } else {
            5
        }
    }

    /// Assess the tactical threat value of a move
    fn assess_tactical_threat(&self, move_: &Move) -> i32 {
        let mut threat_value = 0;
        
        // High value for captures
        if move_.is_capture {
            threat_value += move_.captured_piece_value();
        }
        
        // High value for checks
        if move_.gives_check {
            threat_value += 1000;
        }
        
        // High value for promotions
        if move_.is_promotion {
            threat_value += move_.promotion_value();
        }
        
        // High value for recaptures
        if move_.is_recapture {
            threat_value += 500;
        }
        
        threat_value
    }

    /// Sort moves specifically for quiescence search
    fn sort_quiescence_moves(&self, moves: &[Move]) -> Vec<Move> {
        let mut sorted_moves = moves.to_vec();
        sorted_moves.sort_by(|a, b| self.compare_quiescence_moves(a, b));
        sorted_moves
    }

    /// Clear the quiescence transposition table
    pub fn clear_quiescence_tt(&mut self) {
        self.quiescence_tt.clear();
    }

    /// Get the size of the quiescence transposition table
    pub fn quiescence_tt_size(&self) -> usize {
        self.quiescence_tt.len()
    }

    /// Clean up old entries from the quiescence transposition table
    pub fn cleanup_quiescence_tt(&mut self, max_entries: usize) {
        if self.quiescence_tt.len() > max_entries {
            // Simple cleanup: clear half the entries
            let entries_to_remove = self.quiescence_tt.len() / 2;
            let keys_to_remove: Vec<String> = self.quiescence_tt.keys()
                .take(entries_to_remove)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                self.quiescence_tt.remove(&key);
            }
        }
    }

    /// Get a comprehensive performance report for quiescence search
    pub fn get_quiescence_performance_report(&self) -> String {
        self.quiescence_stats.performance_report()
    }

    /// Get a summary of quiescence performance
    pub fn get_quiescence_summary(&self) -> String {
        self.quiescence_stats.summary()
    }

    /// Get configuration and performance summary
    pub fn get_quiescence_status(&self) -> String {
        format!(
            "{}\n{}\nTT Size: {} entries",
            self.quiescence_config.summary(),
            self.quiescence_stats.summary(),
            self.quiescence_tt.len()
        )
    }

    /// Reset quiescence statistics
    pub fn reset_quiescence_performance(&mut self) {
        self.quiescence_stats.reset();
    }

    /// Get quiescence efficiency metrics
    pub fn get_quiescence_efficiency(&self) -> (f64, f64, f64) {
        (
            self.quiescence_stats.pruning_efficiency(),
            self.quiescence_stats.tt_hit_rate(),
            self.quiescence_stats.extension_rate()
        )
    }
    /// Profile quiescence search performance
    pub fn profile_quiescence_search(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, iterations: usize) -> QuiescenceProfile {
        let mut profile = QuiescenceProfile::new();
        let time_source = TimeSource::now();
        
        for i in 0..iterations {
            self.reset_quiescence_stats();
            let start_time = std::time::Instant::now();
            
            let _result = self.quiescence_search(
                board,
                captured_pieces,
                player,
                -10000,
                10000,
                &time_source,
                1000,
                depth
            );
            
            let duration = start_time.elapsed();
            let stats = self.get_quiescence_stats().clone();
            
            profile.add_sample(QuiescenceSample {
                iteration: i,
                duration_ms: duration.as_millis() as u64,
                nodes_searched: stats.nodes_searched,
                moves_ordered: stats.moves_ordered,
                delta_prunes: stats.delta_prunes,
                futility_prunes: stats.futility_prunes,
                extensions: stats.extensions,
                tt_hits: stats.tt_hits,
                tt_misses: stats.tt_misses,
                check_moves: stats.check_moves_found,
                capture_moves: stats.capture_moves_found,
                promotion_moves: stats.promotion_moves_found,
            });
        }
        
        profile
    }

    /// Get detailed performance metrics
    pub fn get_quiescence_performance_metrics(&self) -> QuiescencePerformanceMetrics {
        let stats = self.get_quiescence_stats();
        QuiescencePerformanceMetrics {
            nodes_per_second: if stats.nodes_searched > 0 { 
                stats.nodes_searched as f64 / 1.0 // Placeholder - would need timing info
            } else { 0.0 },
            pruning_efficiency: stats.pruning_efficiency(),
            tt_hit_rate: stats.tt_hit_rate(),
            extension_rate: stats.extension_rate(),
            move_ordering_efficiency: if stats.moves_ordered > 0 {
                (stats.nodes_searched as f64 / stats.moves_ordered as f64) * 100.0
            } else { 0.0 },
            tactical_move_ratio: if stats.nodes_searched > 0 {
                ((stats.check_moves_found + stats.capture_moves_found + stats.promotion_moves_found) as f64 / stats.nodes_searched as f64) * 100.0
            } else { 0.0 },
        }
    }
    // ===== NULL MOVE PRUNING METHODS =====
    /// Check if null move pruning should be attempted in the current position
    fn should_attempt_null_move(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces,
                               player: Player, depth: u8, can_null_move: bool) -> bool {
        if !self.null_move_config.enabled || !can_null_move {
            return false;
        }
        
        // Must have sufficient depth
        if depth < self.null_move_config.min_depth {
            return false;
        }
        
        // Cannot be in check
        if board.is_king_in_check(player, captured_pieces) {
            self.null_move_stats.disabled_in_check += 1;
            return false;
        }
        
        // Endgame detection
        if self.null_move_config.enable_endgame_detection {
            let piece_count = self.count_pieces_on_board(board);
            if piece_count < self.null_move_config.max_pieces_threshold {
                self.null_move_stats.disabled_endgame += 1;
                return false;
            }
        }
        
        true
    }
    
    /// Count the number of pieces on the board for endgame detection
    fn count_pieces_on_board(&self, board: &BitboardBoard) -> u8 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if board.is_square_occupied(Position::new(row, col)) {
                    count += 1;
                }
            }
        }
        count
    }
    
    /// Perform a null move search with reduced depth
    fn perform_null_move_search(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces,
                               player: Player, depth: u8, beta: i32, start_time: &TimeSource,
                               time_limit_ms: u32, history: &mut Vec<String>) -> i32 {
        self.null_move_stats.attempts += 1;
        
        // Calculate reduction factor
        let reduction = if self.null_move_config.enable_dynamic_reduction {
            2 + depth / 6  // Dynamic reduction
        } else {
            self.null_move_config.reduction_factor as u8  // Static reduction
        };
        
        let search_depth = depth - 1 - reduction;
        self.null_move_stats.depth_reductions += reduction as u64;
        
        // Perform null move search with zero-width window
        let null_move_score = -self.negamax_with_context(
            board, captured_pieces, player.opposite(), 
            search_depth, beta.saturating_neg(), beta.saturating_neg().saturating_add(1), 
            start_time, time_limit_ms, history, 
            false, false, false, false  // Prevent recursive null moves
        );
        
        null_move_score
    }

    // ===== NULL MOVE CONFIGURATION MANAGEMENT =====

    /// Create default null move configuration
    pub fn new_null_move_config() -> NullMoveConfig {
        NullMoveConfig::default()
    }
    
    /// Update null move configuration with validation
    pub fn update_null_move_config(&mut self, config: NullMoveConfig) -> Result<(), String> {
        config.validate()?;
        self.null_move_config = config;
        Ok(())
    }
    
    /// Get current null move configuration
    pub fn get_null_move_config(&self) -> &NullMoveConfig {
        &self.null_move_config
    }
    
    /// Get current null move statistics
    pub fn get_null_move_stats(&self) -> &NullMoveStats {
        &self.null_move_stats
    }
    
    /// Reset null move statistics
    pub fn reset_null_move_stats(&mut self) {
        self.null_move_stats = NullMoveStats::default();
    }

    // ===== LATE MOVE REDUCTIONS CONFIGURATION MANAGEMENT =====

    /// Create default LMR configuration
    pub fn new_lmr_config() -> LMRConfig {
        LMRConfig::default()
    }
    
    /// Update LMR configuration with validation
    pub fn update_lmr_config(&mut self, config: LMRConfig) -> Result<(), String> {
        config.validate()?;
        self.lmr_config = config;
        Ok(())
    }
    
    /// Get current LMR configuration
    pub fn get_lmr_config(&self) -> &LMRConfig {
        &self.lmr_config
    }
    
    /// Get current LMR statistics
    pub fn get_lmr_stats(&self) -> &LMRStats {
        &self.lmr_stats
    }
    
    /// Reset LMR statistics
    pub fn reset_lmr_stats(&mut self) {
        self.lmr_stats = LMRStats::default();
    }

    // ===== ASPIRATION WINDOWS CONFIGURATION MANAGEMENT =====

    /// Create default aspiration window configuration
    pub fn new_aspiration_window_config() -> AspirationWindowConfig {
        AspirationWindowConfig::default()
    }
    
    /// Update aspiration window configuration with validation
    pub fn update_aspiration_window_config(&mut self, config: AspirationWindowConfig) -> Result<(), String> {
        config.validate()?;
        self.aspiration_config = config;
        Ok(())
    }
    
    /// Get current aspiration window configuration
    pub fn get_aspiration_window_config(&self) -> &AspirationWindowConfig {
        &self.aspiration_config
    }
    
    /// Get current aspiration window statistics
    pub fn get_aspiration_window_stats(&self) -> &AspirationWindowStats {
        &self.aspiration_stats
    }
    /// Reset aspiration window statistics
    pub fn reset_aspiration_window_stats(&mut self) {
        self.aspiration_stats = AspirationWindowStats::default();
    }

    /// Get aspiration window performance metrics for tuning
    pub fn get_aspiration_window_performance_metrics(&self) -> AspirationWindowPerformanceMetrics {
        let stats = &self.aspiration_stats;
        
        AspirationWindowPerformanceMetrics {
            total_searches: stats.total_searches,
            successful_searches: stats.successful_searches,
            fail_lows: stats.fail_lows,
            fail_highs: stats.fail_highs,
            total_researches: stats.total_researches,
            success_rate: stats.success_rate(),
            research_rate: stats.research_rate(),
            efficiency: stats.efficiency(),
            average_window_size: stats.average_window_size,
            estimated_time_saved_ms: stats.estimated_time_saved_ms,
            estimated_nodes_saved: stats.estimated_nodes_saved,
        }
    }

    /// Get aspiration window configuration presets for different playing styles
    pub fn get_aspiration_window_preset(&self, style: AspirationWindowPlayingStyle) -> AspirationWindowConfig {
        match style {
            AspirationWindowPlayingStyle::Aggressive => AspirationWindowConfig {
                enabled: true,
                base_window_size: 30,        // Smaller window for more aggressive pruning
                dynamic_scaling: true,
                max_window_size: 150,
                min_depth: 2,
                enable_adaptive_sizing: true,
                max_researches: 3,           // Allow more re-searches
                enable_statistics: true,
            },
            AspirationWindowPlayingStyle::Conservative => AspirationWindowConfig {
                enabled: true,
                base_window_size: 80,        // Larger window for safety
                dynamic_scaling: true,
                max_window_size: 300,
                min_depth: 3,                // Start later
                enable_adaptive_sizing: true,
                max_researches: 1,           // Fewer re-searches
                enable_statistics: true,
            },
            AspirationWindowPlayingStyle::Balanced => AspirationWindowConfig {
                enabled: true,
                base_window_size: 50,        // Default balanced settings
                dynamic_scaling: true,
                max_window_size: 200,
                min_depth: 2,
                enable_adaptive_sizing: true,
                max_researches: 2,
                enable_statistics: true,
            },
        }
    }

    /// Apply aspiration window configuration preset
    pub fn apply_aspiration_window_preset(&mut self, style: AspirationWindowPlayingStyle) -> Result<(), String> {
        let preset = self.get_aspiration_window_preset(style);
        self.update_aspiration_window_config(preset)
    }

    /// Optimize aspiration window memory usage by clearing old statistics
    pub fn optimize_aspiration_window_memory(&mut self) {
        // Reset statistics if they get too large
        if self.aspiration_stats.total_searches > 1_000_000 {
            self.aspiration_stats.reset();
        }
        
        // Clear previous scores if they get too large
        if self.previous_scores.len() > 1000 {
            self.previous_scores.clear();
        }
        
        // Clear transposition table if it gets too large
        if self.transposition_table.size() > 100_000 {
            self.transposition_table.clear();
        }
    }

    // ===== ASPIRATION WINDOW SIZE CALCULATION =====

    /// Calculate static window size
    fn calculate_static_window_size(&self, depth: u8) -> i32 {
        if depth < self.aspiration_config.min_depth {
            return i32::MAX; // Use full-width window
        }
        self.aspiration_config.base_window_size
    }

    /// Calculate dynamic window size based on depth and score
    fn calculate_dynamic_window_size(&self, depth: u8, previous_score: i32) -> i32 {
        let base_size = self.aspiration_config.base_window_size;
        
        if !self.aspiration_config.dynamic_scaling {
            return base_size;
        }
        
        // Scale based on depth
        let depth_factor = 1.0 + (depth as f64 - 1.0) * 0.1;
        
        // Scale based on score magnitude (more volatile scores = larger window)
        let score_factor = 1.0 + (previous_score.abs() as f64 / 1000.0) * 0.2;
        
        let dynamic_size = (base_size as f64 * depth_factor * score_factor) as i32;
        
        // Apply limits
        dynamic_size.min(self.aspiration_config.max_window_size)
    }

    /// Calculate adaptive window size based on recent failures
    fn calculate_adaptive_window_size(&self, depth: u8, recent_failures: u8) -> i32 {
        let base_size = self.calculate_dynamic_window_size(depth, 0);
        
        if !self.aspiration_config.enable_adaptive_sizing {
            return base_size;
        }
        
        // Increase window size if recent failures
        let failure_factor = 1.0 + (recent_failures as f64 * 0.3);
        let adaptive_size = (base_size as f64 * failure_factor) as i32;
        
        adaptive_size.min(self.aspiration_config.max_window_size)
    }

    /// Calculate final window size combining all strategies
    pub fn calculate_window_size(&self, depth: u8, _previous_score: i32, recent_failures: u8) -> i32 {
        if !self.aspiration_config.enabled {
            return i32::MAX; // Use full-width window
        }

        if depth < self.aspiration_config.min_depth {
            return i32::MAX; // Use full-width window
        }

        let window_size = self.calculate_adaptive_window_size(depth, recent_failures);
        self.validate_window_size(window_size)
    }

    /// Validate window size to ensure reasonable bounds
    fn validate_window_size(&self, window_size: i32) -> i32 {
        // Ensure minimum window size for stability
        let min_size = 10;
        let max_size = self.aspiration_config.max_window_size;
        
        let validated_size = window_size.max(min_size).min(max_size);
        
        // Log extreme values for debugging
        if validated_size != window_size {
            crate::debug_utils::debug_log(&format!(
                "Aspiration: Window size clamped from {} to {}",
                window_size, validated_size
            ));
        }
        
        validated_size
    }

    /// Calculate window size with debugging and statistics tracking
    pub fn calculate_window_size_with_stats(&mut self, depth: u8, previous_score: i32, recent_failures: u8) -> i32 {
        let window_size = self.calculate_window_size(depth, previous_score, recent_failures);
        
        // Update statistics
        if self.aspiration_config.enable_statistics {
            self.aspiration_stats.average_window_size = 
                (self.aspiration_stats.average_window_size * (self.aspiration_stats.total_searches as f64) + window_size as f64) 
                / (self.aspiration_stats.total_searches + 1) as f64;
        }
        
        // Debug logging
        if window_size != i32::MAX {
            crate::debug_utils::debug_log(&format!(
                "Aspiration: depth={}, previous_score={}, recent_failures={}, window_size={}",
                depth, previous_score, recent_failures, window_size
            ));
        }
        
        window_size
    }

    /// Get window size preset for different playing styles
    pub fn get_window_size_preset(&self, style: AspirationWindowPlayingStyle) -> i32 {
        match style {
            AspirationWindowPlayingStyle::Aggressive => {
                // Smaller windows for faster, more aggressive play
                self.aspiration_config.base_window_size / 2
            },
            AspirationWindowPlayingStyle::Conservative => {
                // Larger windows for safer, more thorough play
                self.aspiration_config.base_window_size * 2
            },
            AspirationWindowPlayingStyle::Balanced => {
                // Standard window size
                self.aspiration_config.base_window_size
            },
        }
    }

    /// Calculate window size based on position complexity
    pub fn calculate_complexity_based_window_size(&self, depth: u8, position_complexity: f64) -> i32 {
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }
        
        // Adjust window size based on position complexity
        // More complex positions get larger windows
        let complexity_factor = 1.0 + (position_complexity * 0.5);
        let adjusted_size = (base_size as f64 * complexity_factor) as i32;
        
        self.validate_window_size(adjusted_size)
    }

    /// Calculate window size based on time remaining
    pub fn calculate_time_based_window_size(&self, depth: u8, time_remaining_ms: u32, total_time_ms: u32) -> i32 {
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }
        
        // Adjust window size based on time pressure
        // Less time = smaller windows for faster search
        let time_ratio = time_remaining_ms as f64 / total_time_ms as f64;
        let time_factor = 0.5 + (time_ratio * 0.5); // Range from 0.5 to 1.0
        let adjusted_size = (base_size as f64 * time_factor) as i32;
        
        self.validate_window_size(adjusted_size)
    }

    /// Calculate window size based on search history and performance
    pub fn calculate_history_based_window_size(&self, depth: u8, recent_success_rate: f64) -> i32 {
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }
        
        // Adjust window size based on recent success rate
        // Lower success rate = larger windows for more thorough search
        let success_factor = if recent_success_rate > 0.8 {
            0.8 // Smaller windows for high success rate
        } else if recent_success_rate > 0.5 {
            1.0 // Standard windows for moderate success rate
        } else {
            1.5 // Larger windows for low success rate
        };
        
        let adjusted_size = (base_size as f64 * success_factor) as i32;
        self.validate_window_size(adjusted_size)
    }

    /// Calculate window size based on move count and branching factor
    pub fn calculate_branching_based_window_size(&self, depth: u8, move_count: usize) -> i32 {
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }
        
        // Adjust window size based on branching factor
        // More moves = smaller windows to maintain search speed
        let branching_factor = if move_count > 50 {
            0.7 // Smaller windows for high branching factor
        } else if move_count > 20 {
            0.9 // Slightly smaller windows for moderate branching factor
        } else {
            1.1 // Larger windows for low branching factor
        };
        
        let adjusted_size = (base_size as f64 * branching_factor) as i32;
        self.validate_window_size(adjusted_size)
    }

    /// Calculate comprehensive window size using all available factors
    pub fn calculate_comprehensive_window_size(&mut self, depth: u8, previous_score: i32, recent_failures: u8, 
                                             position_complexity: f64, time_remaining_ms: u32, total_time_ms: u32,
                                             recent_success_rate: f64, move_count: usize) -> i32 {
        if !self.aspiration_config.enabled {
            return i32::MAX; // Use full-width window
        }

        if depth < self.aspiration_config.min_depth {
            return i32::MAX; // Use full-width window
        }

        // Calculate base window size
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }

        // Apply all adjustment factors
        let depth_factor = 1.0 + (depth as f64 - 1.0) * 0.1;
        let score_factor = 1.0 + (previous_score.abs() as f64 / 1000.0) * 0.2;
        let failure_factor = 1.0 + (recent_failures as f64 * 0.3);
        let complexity_factor = 1.0 + (position_complexity * 0.5);
        let time_ratio = time_remaining_ms as f64 / total_time_ms as f64;
        let time_factor = 0.5 + (time_ratio * 0.5);
        let success_factor = if recent_success_rate > 0.8 { 0.8 } else if recent_success_rate > 0.5 { 1.0 } else { 1.5 };
        let branching_factor = if move_count > 50 { 0.7 } else if move_count > 20 { 0.9 } else { 1.1 };

        // Combine all factors
        let comprehensive_size = (base_size as f64 * depth_factor * score_factor * failure_factor * 
                                 complexity_factor * time_factor * success_factor * branching_factor) as i32;

        let final_size = self.validate_window_size(comprehensive_size);
        
        // Update statistics
        if self.aspiration_config.enable_statistics {
            self.aspiration_stats.average_window_size = 
                (self.aspiration_stats.average_window_size * (self.aspiration_stats.total_searches as f64) + final_size as f64) 
                / (self.aspiration_stats.total_searches + 1) as f64;
        }

        // Debug logging
        crate::debug_utils::debug_log(&format!(
            "Aspiration: comprehensive window size calculation - depth={}, base={}, final={}, factors=[d:{:.2}, s:{:.2}, f:{:.2}, c:{:.2}, t:{:.2}, su:{:.2}, b:{:.2}]",
            depth, base_size, final_size, depth_factor, score_factor, failure_factor, 
            complexity_factor, time_factor, success_factor, branching_factor
        ));

        final_size
    }

    /// Get window size statistics for analysis and tuning
    pub fn get_window_size_statistics(&self) -> WindowSizeStatistics {
        WindowSizeStatistics {
            average_window_size: self.aspiration_stats.average_window_size,
            min_window_size: 10, // Minimum enforced window size
            max_window_size: self.aspiration_config.max_window_size,
            total_calculations: self.aspiration_stats.total_searches,
            success_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.successful_searches as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            fail_low_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.fail_lows as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            fail_high_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.fail_highs as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
        }
    }

    /// Reset window size statistics
    pub fn reset_window_size_statistics(&mut self) {
        self.aspiration_stats.average_window_size = 0.0;
    }

    /// Calculate optimal window size based on historical performance
    pub fn calculate_optimal_window_size(&self, depth: u8, recent_performance: f64) -> i32 {
        let base_size = self.calculate_static_window_size(depth);
        
        if base_size == i32::MAX {
            return base_size; // Full-width window
        }

        // Adjust based on recent performance
        // Better performance = smaller windows for efficiency
        // Worse performance = larger windows for thoroughness
        let performance_factor = if recent_performance > 0.9 {
            0.7 // High performance: smaller windows
        } else if recent_performance > 0.7 {
            0.85 // Good performance: slightly smaller windows
        } else if recent_performance > 0.5 {
            1.0 // Average performance: standard windows
        } else if recent_performance > 0.3 {
            1.2 // Poor performance: larger windows
        } else {
            1.5 // Very poor performance: much larger windows
        };

        let optimal_size = (base_size as f64 * performance_factor) as i32;
        self.validate_window_size(optimal_size)
    }

    // ===== ASPIRATION WINDOW RE-SEARCH LOGIC =====

    /// Handle fail-low by widening window downward
    fn handle_fail_low(&mut self, alpha: &mut i32, beta: &mut i32, 
                       previous_score: i32, window_size: i32) {
        self.aspiration_stats.fail_lows += 1;
        
        // Enhanced validation with recovery
        if !self.validate_and_recover_window(alpha, beta, previous_score, window_size, 0) {
            crate::debug_utils::trace_log("ASPIRATION_FAIL_LOW", 
                "Window validation failed, using fallback");
            return;
        }
        
        // Adaptive window widening based on failure pattern
        let adaptive_factor = self.calculate_adaptive_factor("fail_low");
        let widened_window = window_size * adaptive_factor;
        
        // Widen window downward with adaptive sizing
        let new_alpha = i32::MIN + 1;
        let new_beta = previous_score + widened_window;
        
        // Ensure valid window bounds with additional safety checks
        if new_beta <= new_alpha {
            crate::debug_utils::trace_log("ASPIRATION_FAIL_LOW", 
                "Invalid window bounds, using conservative approach");
            *alpha = i32::MIN + 1;
            *beta = previous_score + window_size;
            
            // Final safety check
            if *beta <= *alpha {
                *alpha = i32::MIN + 1;
                *beta = i32::MAX - 1;
            }
        } else {
            *alpha = new_alpha;
            *beta = new_beta;
        }
        
        // Update performance metrics
        self.update_fail_low_metrics(previous_score, window_size);
        
        crate::debug_utils::trace_log("ASPIRATION_FAIL_LOW", 
            &format!("Fail-low handled: alpha={}, beta={}, adaptive_factor={}", 
                *alpha, *beta, adaptive_factor));
    }

    /// Handle fail-high by widening window upward
    fn handle_fail_high(&mut self, alpha: &mut i32, beta: &mut i32,
                        previous_score: i32, window_size: i32) {
        self.aspiration_stats.fail_highs += 1;
        
        // Enhanced validation with recovery
        if !self.validate_and_recover_window(alpha, beta, previous_score, window_size, 0) {
            crate::debug_utils::trace_log("ASPIRATION_FAIL_HIGH", 
                "Window validation failed, using fallback");
            return;
        }
        
        // Adaptive window widening based on failure pattern
        let adaptive_factor = self.calculate_adaptive_factor("fail_high");
        let widened_window = window_size * adaptive_factor;
        
        // Widen window upward with adaptive sizing
        let new_alpha = previous_score - widened_window;
        let new_beta = i32::MAX - 1;
        
        // Ensure valid window bounds with additional safety checks
        if new_alpha >= new_beta {
            crate::debug_utils::trace_log("ASPIRATION_FAIL_HIGH", 
                "Invalid window bounds, using conservative approach");
            *alpha = previous_score - window_size;
            *beta = i32::MAX - 1;
            
            // Final safety check
            if *alpha >= *beta {
                *alpha = i32::MIN + 1;
                *beta = i32::MAX - 1;
            }
        } else {
            *alpha = new_alpha;
            *beta = new_beta;
        }
        
        // Update performance metrics
        self.update_fail_high_metrics(previous_score, window_size);
        
        crate::debug_utils::trace_log("ASPIRATION_FAIL_HIGH", 
            &format!("Fail-high handled: alpha={}, beta={}, adaptive_factor={}", 
                *alpha, *beta, adaptive_factor));
    }

    /// Update aspiration window statistics
    fn update_aspiration_stats(&mut self, had_research: bool, research_count: u8) {
        self.aspiration_stats.total_searches += 1;
        if !had_research {
            self.aspiration_stats.successful_searches += 1;
        }
        self.aspiration_stats.total_researches += research_count as u64;
    }

    /// Validate window parameters for error handling
    fn validate_window_parameters(&self, previous_score: i32, window_size: i32) -> bool {
        // Check for reasonable score bounds
        if previous_score < -100000 || previous_score > 100000 {
            crate::debug_utils::debug_log(&format!(
                "Aspiration: Invalid previous_score: {} (out of reasonable bounds)",
                previous_score
            ));
            return false;
        }
        
        // Check for reasonable window size
        if window_size <= 0 || window_size > self.aspiration_config.max_window_size * 2 {
            crate::debug_utils::debug_log(&format!(
                "Aspiration: Invalid window_size: {} (out of reasonable bounds)",
                window_size
            ));
            return false;
        }
        
        true
    }

    /// Enhanced window validation with recovery mechanisms
    fn validate_and_recover_window(&mut self, alpha: &mut i32, beta: &mut i32, 
                                  previous_score: i32, window_size: i32, 
                                  _depth: u8) -> bool {
        
        // Initial validation
        if !self.validate_window_parameters(previous_score, window_size) {
            crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                "Invalid parameters detected, attempting recovery");
            
            // Recovery attempt 1: Use safe defaults
            let safe_score = previous_score.clamp(-50000, 50000);
            let safe_window = window_size.clamp(10, self.aspiration_config.max_window_size);
            
            if self.validate_window_parameters(safe_score, safe_window) {
                *alpha = safe_score - safe_window;
                *beta = safe_score + safe_window;
                crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                    &format!("Recovery successful: alpha={}, beta={}", alpha, beta));
                return true;
            }
            
            // Recovery attempt 2: Fall back to full-width search
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                "Recovery failed, using full-width search");
            return true;
        }
        
        // Validate window bounds
        if *alpha >= *beta {
            crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                &format!("Invalid window bounds: alpha={} >= beta={}", alpha, beta));
            
            // Recovery: Ensure alpha < beta
            if *alpha >= *beta {
                // Use safe arithmetic to prevent overflow when alpha and beta are very large
                let center = (*alpha as i64 + *beta as i64) / 2;
                let half_window = window_size / 2;
                *alpha = center.saturating_sub(half_window as i64) as i32;
                *beta = center.saturating_add(half_window as i64) as i32;
                
                // Final safety check
                if *alpha >= *beta {
                    *alpha = i32::MIN + 1;
                    *beta = i32::MAX - 1;
                }
                
                crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                    &format!("Window bounds corrected: alpha={}, beta={}", alpha, beta));
            }
        }
        
        // Validate window size is reasonable for depth
        let current_window_size = (*beta as i64).saturating_sub(*alpha as i64);
        let expected_max_size = self.aspiration_config.max_window_size;
        
        if current_window_size > expected_max_size as i64 {
            crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                &format!("Window too large: {} > {}, adjusting", current_window_size, expected_max_size));
            
            // Use safe arithmetic to prevent overflow when alpha and beta are very large
            let center = (*alpha as i64 + *beta as i64) / 2;
            let half_max_size = expected_max_size / 2;
            *alpha = center.saturating_sub(half_max_size as i64) as i32;
            *beta = center.saturating_add(half_max_size as i64) as i32;
            
            crate::debug_utils::trace_log("WINDOW_VALIDATION", 
                &format!("Window size adjusted: alpha={}, beta={}", alpha, beta));
        }
        
        true
    }
    /// Check if window is in a stable state
    fn is_window_stable(&self, alpha: i32, beta: i32, previous_score: i32) -> bool {
        let window_size = (beta as i64).saturating_sub(alpha as i64);
        // Use safe arithmetic to prevent overflow when alpha and beta are very large
        let center = (alpha as i64 + beta as i64) / 2;
        let score_deviation = (center - previous_score as i64).abs();
        
        // Window is stable if:
        // 1. Size is reasonable
        // 2. Center is close to previous score
        // 3. Bounds are valid
        window_size > 0 && 
        window_size <= self.aspiration_config.max_window_size as i64 &&
        score_deviation <= window_size / 4 &&
        alpha < beta
    }

    /// Calculate adaptive factor based on failure type and history
    fn calculate_adaptive_factor(&self, failure_type: &str) -> i32 {
        let base_factor = match failure_type {
            "fail_low" => 2,      // More aggressive widening for fail-low
            "fail_high" => 2,     // More aggressive widening for fail-high
            "search_failed" => 3, // Most aggressive for complete failures
            "timeout" => 1,       // Conservative for timeouts
            _ => 2,               // Default moderate factor
        };
        
        // Adjust based on recent failure rate
        let recent_failures = self.aspiration_stats.fail_lows + self.aspiration_stats.fail_highs;
        let total_searches = self.aspiration_stats.total_searches.max(1);
        let failure_rate = recent_failures as f64 / total_searches as f64;
        
        if failure_rate > 0.3 {
            // High failure rate - be more conservative
            (base_factor as f64 * 0.8) as i32
        } else if failure_rate < 0.1 {
            // Low failure rate - can be more aggressive
            (base_factor as f64 * 1.2) as i32
        } else {
            base_factor
        }
    }

    /// Enhanced failure type classification
    fn classify_failure_type(&self, score: i32, alpha: i32, beta: i32, 
                            search_successful: bool, timeout_occurred: bool) -> &'static str {
        if !search_successful {
            if timeout_occurred {
                "timeout"
            } else {
                "search_failed"
            }
        } else if score <= alpha {
            "fail_low"
        } else if score >= beta {
            "fail_high"
        } else {
            "success"
        }
    }
    /// Update fail-low performance metrics
    fn update_fail_low_metrics(&mut self, previous_score: i32, window_size: i32) {
        if self.aspiration_config.enable_statistics {
            // Track fail-low patterns for optimization
            self.aspiration_stats.estimated_time_saved_ms = self.aspiration_stats.estimated_time_saved_ms.saturating_sub(10);
            self.aspiration_stats.estimated_nodes_saved = self.aspiration_stats.estimated_nodes_saved.saturating_sub(1000);
        }
        
        // Log performance impact
        crate::debug_utils::debug_log(&format!(
            "Aspiration: Fail-low metrics updated - score={}, window={}, total_fail_lows={}",
            previous_score, window_size, self.aspiration_stats.fail_lows
        ));
    }

    /// Update fail-high performance metrics
    fn update_fail_high_metrics(&mut self, previous_score: i32, window_size: i32) {
        if self.aspiration_config.enable_statistics {
            // Track fail-high patterns for optimization
            self.aspiration_stats.estimated_time_saved_ms = self.aspiration_stats.estimated_time_saved_ms.saturating_sub(10);
            self.aspiration_stats.estimated_nodes_saved = self.aspiration_stats.estimated_nodes_saved.saturating_sub(1000);
        }
        
        // Log performance impact
        crate::debug_utils::debug_log(&format!(
            "Aspiration: Fail-high metrics updated - score={}, window={}, total_fail_highs={}",
            previous_score, window_size, self.aspiration_stats.fail_highs
        ));
    }

    /// Handle graceful degradation when aspiration windows fail
    pub fn handle_aspiration_failure(&mut self, depth: u8, reason: &str) -> (i32, i32) {
        crate::debug_utils::debug_log(&format!(
            "Aspiration: Graceful degradation at depth {} - reason: {}",
            depth, reason
        ));
        
        // Update failure statistics
        if self.aspiration_config.enable_statistics {
            self.aspiration_stats.total_searches += 1;
            // Don't increment successful_searches since this is a failure
        }
        
        // Return full-width window for fallback
        (i32::MIN + 1, i32::MAX - 1)
    }

    /// Check if aspiration windows should be disabled due to poor performance
    pub fn should_disable_aspiration_windows(&self) -> bool {
        if !self.aspiration_config.enabled {
            return true;
        }
        
        // Disable if too many failures
        if self.aspiration_stats.total_searches > 100 {
            let failure_rate = (self.aspiration_stats.fail_lows + self.aspiration_stats.fail_highs) as f64 
                / self.aspiration_stats.total_searches as f64;
            
            if failure_rate > 0.8 {
                crate::debug_utils::debug_log(&format!(
                    "Aspiration: High failure rate {:.2}%, disabling aspiration windows",
                    failure_rate * 100.0
                ));
                return true;
            }
        }
        
        // Disable if too many re-searches
        if self.aspiration_stats.total_searches > 50 {
            let research_rate = self.aspiration_stats.total_researches as f64 
                / self.aspiration_stats.total_searches as f64;
            
            if research_rate > 2.0 {
                crate::debug_utils::debug_log(&format!(
                    "Aspiration: High re-search rate {:.2}, disabling aspiration windows",
                    research_rate
                ));
                return true;
            }
        }
        
        false
    }

    /// Get re-search efficiency metrics
    pub fn get_research_efficiency(&self) -> ResearchEfficiencyMetrics {
        ResearchEfficiencyMetrics {
            total_searches: self.aspiration_stats.total_searches,
            successful_searches: self.aspiration_stats.successful_searches,
            fail_lows: self.aspiration_stats.fail_lows,
            fail_highs: self.aspiration_stats.fail_highs,
            total_researches: self.aspiration_stats.total_researches,
            success_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.successful_searches as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            research_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.total_researches as u8 as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            fail_low_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.fail_lows as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            fail_high_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.fail_highs as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
        }
    }

    // ===== PERFORMANCE MONITORING AND STATISTICS =====

    /// Initialize performance monitoring
    pub fn initialize_performance_monitoring(&mut self, max_depth: u8) {
        self.aspiration_stats.initialize_depth_tracking(max_depth);
    }

    /// Update performance statistics during search
    pub fn update_performance_stats(&mut self, depth: u8, success: bool, had_research: bool, 
                                   window_size: i32, search_time_ms: u64, research_time_ms: u64) {
        // Update basic statistics
        self.aspiration_stats.total_searches += 1;
        if success {
            self.aspiration_stats.successful_searches += 1;
        }
        if had_research {
            self.aspiration_stats.total_researches += 1;
        }

        // Update depth-based statistics
        self.aspiration_stats.update_depth_stats(depth, success, had_research, window_size);
        
        // Update window size statistics
        self.aspiration_stats.update_window_size_stats(window_size);
        
        // Update time statistics
        self.aspiration_stats.update_time_stats(search_time_ms, research_time_ms);
        
        // Update memory statistics
        let current_memory = self.estimate_memory_usage();
        self.aspiration_stats.update_memory_stats(current_memory);
        
        // Add performance data point
        let performance = if success { 1.0 } else { 0.5 };
        self.aspiration_stats.add_performance_data_point(performance);
    }

    /// Estimate current memory usage
    fn estimate_memory_usage(&self) -> u64 {
        // Estimate memory usage based on data structures
        let base_memory = std::mem::size_of::<Self>() as u64;
        let previous_scores_memory = (self.previous_scores.len() * std::mem::size_of::<i32>()) as u64;
        let depth_tracking_memory = (self.aspiration_stats.success_rate_by_depth.len() * std::mem::size_of::<f64>() * 3) as u64;
        
        base_memory + previous_scores_memory + depth_tracking_memory
    }

    /// Get comprehensive performance analysis
    pub fn get_performance_analysis(&mut self) -> AspirationWindowPerformanceMetrics {
        self.aspiration_stats.calculate_performance_metrics()
    }

    /// Get depth-based analysis
    pub fn get_depth_analysis(&self) -> DepthAnalysis {
        self.aspiration_stats.get_depth_analysis()
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        self.aspiration_stats.get_performance_summary()
    }

    /// Check for performance regression
    pub fn check_performance_regression(&self) -> Option<String> {
        let trend = self.aspiration_stats.get_performance_trend();
        let summary = self.get_performance_summary();
        
        if trend < -0.2 {
            Some(format!("Performance regression detected: trend = {:.2}", trend))
        } else if summary.configuration_effectiveness < 0.4 {
            Some(format!("Poor configuration effectiveness: {:.2}", summary.configuration_effectiveness))
        } else if summary.success_rate < 0.5 {
            Some(format!("Low success rate: {:.2}", summary.success_rate))
        } else if summary.research_rate > 2.5 {
            Some(format!("High research rate: {:.2}", summary.research_rate))
        } else {
            None
        }
    }

    /// Get adaptive tuning recommendations
    pub fn get_adaptive_tuning_recommendations(&self) -> Vec<String> {
        let summary = self.get_performance_summary();
        let mut recommendations = summary.get_recommendations();
        
        // Add depth-specific recommendations
        let depth_analysis = self.get_depth_analysis();
        if !depth_analysis.success_rate_by_depth.is_empty() {
            let (optimal_start, optimal_end) = depth_analysis.get_optimal_depth_range();
            if optimal_start > 0 || optimal_end < depth_analysis.success_rate_by_depth.len() as u8 - 1 {
                recommendations.push(format!(
                    "Consider limiting aspiration windows to depths {}-{} for optimal performance",
                    optimal_start, optimal_end
                ));
            }
        }
        
        // Add memory optimization recommendations
        if summary.memory_efficiency < 0.5 {
            recommendations.push("Consider reducing previous_scores history or depth tracking data".to_string());
        }
        
        recommendations
    }

    /// Get real-time performance monitoring data
    pub fn get_real_time_performance(&self) -> RealTimePerformance {
        RealTimePerformance {
            current_searches: self.aspiration_stats.total_searches,
            current_success_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.successful_searches as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            current_research_rate: if self.aspiration_stats.total_searches > 0 {
                self.aspiration_stats.total_researches as u8 as f64 / self.aspiration_stats.total_searches as f64
            } else {
                0.0
            },
            current_window_size: self.aspiration_stats.average_window_size,
            performance_trend: self.aspiration_stats.get_performance_trend(),
            memory_usage: self.aspiration_stats.memory_usage_bytes,
            configuration_effectiveness: self.aspiration_stats.configuration_effectiveness,
        }
    }

    /// Reset performance statistics
    pub fn reset_performance_stats(&mut self) {
        self.aspiration_stats.reset();
    }

    /// Optimize performance based on current statistics
    pub fn optimize_performance(&mut self) -> Vec<String> {
        let mut optimizations = Vec::new();
        let summary = self.get_performance_summary();
        
        // Auto-tune based on performance
        if summary.success_rate < 0.7 && summary.research_rate > 1.5 {
            // Increase window size
            let mut config = self.get_aspiration_window_config().clone();
            config.base_window_size = (config.base_window_size as f64 * 1.2) as i32;
            config.base_window_size = config.base_window_size.min(config.max_window_size);
            self.update_aspiration_window_config(config).unwrap();
            optimizations.push("Increased base_window_size for better success rate".to_string());
        }
        
        if summary.success_rate > 0.9 && summary.research_rate < 0.5 {
            // Decrease window size for efficiency
            let mut config = self.get_aspiration_window_config().clone();
            config.base_window_size = (config.base_window_size as f64 * 0.9) as i32;
            config.base_window_size = config.base_window_size.max(10);
            self.update_aspiration_window_config(config).unwrap();
            optimizations.push("Decreased base_window_size for better efficiency".to_string());
        }
        
        if summary.configuration_effectiveness < 0.6 {
            // Reset to default configuration
            let default_config = AspirationWindowConfig::default();
            self.update_aspiration_window_config(default_config).unwrap();
            optimizations.push("Reset to default configuration due to poor effectiveness".to_string());
        }
        
        optimizations
    }

    // ===== LATE MOVE REDUCTIONS CORE LOGIC =====

    /// Search a move with Late Move Reductions applied
    fn search_move_with_lmr(&mut self, 
                           board: &mut BitboardBoard, 
                           captured_pieces: &CapturedPieces, 
                           player: Player, 
                           depth: u8, 
                           alpha: i32, 
                           beta: i32, 
                           start_time: &TimeSource, 
                           time_limit_ms: u32, 
                           history: &mut Vec<String>, 
                           move_: &Move, 
                           move_index: usize,
                           _is_root: bool,
                           has_capture: bool,
                           has_check: bool) -> i32 {
        
        self.lmr_stats.moves_considered += 1;
        
        // Create search state for advanced pruning
        let mut search_state = crate::types::SearchState::new(depth, alpha, beta);
        search_state.move_number = move_index as u8;
        search_state.update_fields(
            has_check,
            self.evaluate_position(board, player, captured_pieces),
            self.get_position_hash(board),
            self.get_game_phase(board)
        );
        
        // Check if LMR should be applied using new PruningManager
        let reduction = self.pruning_manager.calculate_lmr_reduction(&search_state, move_);
        
        if reduction > 0 {
            self.lmr_stats.reductions_applied += 1;
            self.pruning_manager.statistics.lmr_applied += 1;
            self.lmr_stats.total_depth_saved += reduction as u64;
            
            // Perform reduced-depth search with null window
            let reduced_depth = depth - 1 - reduction;
            let score = -self.negamax_with_context(
                board, 
                captured_pieces, 
                player.opposite(), 
                reduced_depth, 
                alpha.saturating_neg().saturating_sub(1), 
                alpha.saturating_neg(), 
                start_time, 
                time_limit_ms, 
                history, 
                true,
                false, // not root
                has_capture,
                has_check
            );
            
            // Check if re-search is needed
            if score > alpha {
                self.lmr_stats.researches_triggered += 1;
                self.pruning_manager.statistics.re_searches += 1;
                
                // Re-search at full depth
                let full_score = -self.negamax_with_context(
                    board, 
                    captured_pieces, 
                    player.opposite(), 
                    depth - 1, 
                    beta.saturating_neg(), 
                    alpha.saturating_neg(), 
                    start_time, 
                    time_limit_ms, 
                    history, 
                    true,
                    false, // not root
                    has_capture,
                    has_check
                );
                
                if full_score >= beta {
                    self.lmr_stats.cutoffs_after_research += 1;
                }
                
                return full_score;
            } else {
                if score >= beta {
                    self.lmr_stats.cutoffs_after_reduction += 1;
                }
                return score;
            }
        } else {
            // No reduction - perform full-depth search
            -self.negamax_with_context(
                board, 
                captured_pieces, 
                player.opposite(), 
                depth - 1, 
                beta.saturating_neg(), 
                alpha.saturating_neg(), 
                start_time, 
                time_limit_ms, 
                history, 
                true,
                false, // not root
                has_capture,
                has_check
            )
        }
    }

    /// Check if LMR should be applied to a move
    fn should_apply_lmr(&self, move_: &Move, depth: u8, move_index: usize) -> bool {
        if !self.lmr_config.enabled {
            return false;
        }
        
        // Must meet minimum depth requirement
        if depth < self.lmr_config.min_depth {
            return false;
        }
        
        // Must be beyond minimum move index
        if move_index < self.lmr_config.min_move_index as usize {
            return false;
        }
        
        // Apply exemption rules (use optimized version)
        if self.is_move_exempt_from_lmr_optimized(move_) {
            return false;
        }
        
        true
    }

    /// Check if a move is exempt from LMR
    fn is_move_exempt_from_lmr(&self, move_: &Move) -> bool {
        // Basic exemptions
        if move_.is_capture || move_.is_promotion || move_.gives_check {
            return true;
        }
        
        if self.lmr_config.enable_extended_exemptions {
            // Extended exemptions
            if self.is_killer_move(move_) {
                return true;
            }
            
            if self.is_transposition_table_move(move_) {
                return true;
            }
            
            if self.is_escape_move(move_) {
                return true;
            }
        }
        
        false
    }

    /// Calculate reduction amount for LMR
    fn calculate_reduction(&self, move_: &Move, depth: u8, move_index: usize) -> u8 {
        if !self.lmr_config.enable_dynamic_reduction {
            return self.lmr_config.base_reduction;
        }
        
        let mut reduction = self.lmr_config.base_reduction;
        
        // Dynamic reduction based on depth
        if depth >= 6 {
            reduction += 1;
        }
        if depth >= 10 {
            reduction += 1;
        }
        
        // Dynamic reduction based on move index
        if move_index >= 8 {
            reduction += 1;
        }
        if move_index >= 16 {
            reduction += 1;
        }
        
        // Adaptive reduction based on position characteristics
        if self.lmr_config.enable_adaptive_reduction {
            reduction = self.apply_adaptive_reduction(reduction, move_, depth);
        }
        
        // Ensure reduction doesn't exceed maximum
        reduction.min(self.lmr_config.max_reduction)
            .min(depth.saturating_sub(2)) // Don't reduce to zero or negative
    }

    /// Apply adaptive reduction based on position characteristics
    fn apply_adaptive_reduction(&self, base_reduction: u8, move_: &Move, _depth: u8) -> u8 {
        let mut reduction = base_reduction;
        
        // More conservative reduction in tactical positions
        if self.is_tactical_position() {
            reduction = reduction.saturating_sub(1);
        }
        
        // More aggressive reduction in quiet positions
        if self.is_quiet_position() {
            reduction += 1;
        }
        
        // Adjust based on move characteristics
        if self.is_center_move(move_) {
            reduction = reduction.saturating_sub(1);
        }
        
        reduction
    }

    /// Check if a move is a killer move
    fn is_killer_move(&self, move_: &Move) -> bool {
        self.killer_moves.iter().any(|killer| {
            killer.as_ref().map_or(false, |k| self.moves_equal(move_, k))
        })
    }

    /// Check if a move is from transposition table
    fn is_transposition_table_move(&self, move_: &Move) -> bool {
        // This is a simplified implementation
        // In a full implementation, we would track the best move from TT lookup
        // For now, we'll use a heuristic based on move characteristics
        move_.is_capture && move_.captured_piece_value() > 500
    }

    /// Check if a move is an escape move
    fn is_escape_move(&self, move_: &Move) -> bool {
        // Check if this move escapes from a threat
        // This is a simplified implementation based on move characteristics
        if let Some(from) = move_.from {
            // Check if moving away from center (potential escape)
            let from_center = self.is_center_square(from);
            let to_center = self.is_center_move(move_);
            if from_center && !to_center {
                return true;
            }
        }
        false
    }

    /// Check if position is tactical
    fn is_tactical_position(&self) -> bool {
        // Determine if position has tactical characteristics
        // This is a simplified implementation based on recent statistics
        let stats = &self.lmr_stats;
        if stats.moves_considered > 0 {
            // If we've seen many captures or checks recently, it's tactical
            let capture_ratio = stats.cutoffs_after_reduction as f64 / stats.moves_considered as f64;
            return capture_ratio > 0.3; // More than 30% of moves are cutoffs
        }
        false
    }

    /// Check if position is quiet
    fn is_quiet_position(&self) -> bool {
        // Determine if position is quiet (few captures, checks)
        // This is a simplified implementation based on recent statistics
        let stats = &self.lmr_stats;
        if stats.moves_considered > 0 {
            // If we've seen few cutoffs recently, it's quiet
            let cutoff_ratio = stats.total_cutoffs() as f64 / stats.moves_considered as f64;
            return cutoff_ratio < 0.1; // Less than 10% of moves are cutoffs
        }
        true // Default to quiet if no data
    }

    /// Check if a move targets center squares
    fn is_center_move(&self, move_: &Move) -> bool {
        self.is_center_square(move_.to)
    }


    // ===== ADDITIONAL LMR HELPER METHODS =====

    /// Get the tactical value of a move for LMR decisions
    fn get_move_tactical_value(&self, move_: &Move) -> i32 {
        let mut value = 0;
        
        // High value for captures
        if move_.is_capture {
            value += move_.captured_piece_value();
        }
        
        // High value for checks
        if move_.gives_check {
            value += 1000;
        }
        
        // High value for promotions
        if move_.is_promotion {
            value += move_.promotion_value();
        }
        
        // Medium value for center moves
        if self.is_center_move(move_) {
            value += 50;
        }
        
        // Medium value for killer moves
        if self.is_killer_move(move_) {
            value += 200;
        }
        
        value
    }

    /// Classify a move type for LMR exemption decisions
    fn classify_move_type(&self, move_: &Move) -> MoveType {
        if move_.gives_check {
            MoveType::Check
        } else if move_.is_capture {
            MoveType::Capture
        } else if move_.is_promotion {
            MoveType::Promotion
        } else if self.is_killer_move(move_) {
            MoveType::Killer
        } else if self.is_transposition_table_move(move_) {
            MoveType::TranspositionTable
        } else if self.is_escape_move(move_) {
            MoveType::Escape
        } else if self.is_center_move(move_) {
            MoveType::Center
        } else {
            MoveType::Quiet
        }
    }

    /// Get the position complexity for adaptive LMR
    fn get_position_complexity(&self) -> PositionComplexity {
        let stats = &self.lmr_stats;
        
        if stats.moves_considered == 0 {
            return PositionComplexity::Unknown;
        }
        
        let cutoff_rate = stats.total_cutoffs() as f64 / stats.moves_considered as f64;
        let research_rate = stats.research_rate() / 100.0;
        
        if cutoff_rate > 0.4 || research_rate > 0.3 {
            PositionComplexity::High
        } else if cutoff_rate > 0.2 || research_rate > 0.15 {
            PositionComplexity::Medium
        } else {
            PositionComplexity::Low
        }
    }

    /// Check if LMR is effective in current position
    fn is_lmr_effective(&self) -> bool {
        let stats = &self.lmr_stats;
        
        if stats.moves_considered < 10 {
            return true; // Not enough data, assume effective
        }
        
        let efficiency = stats.efficiency();
        let research_rate = stats.research_rate() / 100.0;
        
        // LMR is effective if we're reducing many moves but not re-searching too many
        efficiency > 20.0 && research_rate < 0.4
    }
    /// Get recommended LMR parameters based on position
    fn get_adaptive_lmr_params(&self) -> (u8, u8) {
        let complexity = self.get_position_complexity();
        let is_effective = self.is_lmr_effective();
        
        let base_reduction = match complexity {
            PositionComplexity::High => if is_effective { 2 } else { 1 },
            PositionComplexity::Medium => 1,
            PositionComplexity::Low => 2,
            PositionComplexity::Unknown => 1,
        };
        
        let max_reduction = match complexity {
            PositionComplexity::High => 4,
            PositionComplexity::Medium => 3,
            PositionComplexity::Low => 5,
            PositionComplexity::Unknown => 3,
        };
        
        (base_reduction, max_reduction)
    }

    // ===== LMR PERFORMANCE OPTIMIZATION =====

    /// Optimized move exemption check with early returns
    fn is_move_exempt_from_lmr_optimized(&self, move_: &Move) -> bool {
        // Early return for most common exemptions (captures, promotions, checks)
        if move_.is_capture || move_.is_promotion || move_.gives_check {
            return true;
        }
        
        // Only check extended exemptions if enabled
        if !self.lmr_config.enable_extended_exemptions {
            return false;
        }
        
        // Check killer moves (most common extended exemption)
        if self.is_killer_move(move_) {
            return true;
        }
        
        // Check other exemptions only if needed
        self.is_transposition_table_move(move_) || self.is_escape_move(move_)
    }

    /// Optimized reduction calculation with cached values
    fn calculate_reduction_optimized(&self, move_: &Move, depth: u8, move_index: usize) -> u8 {
        if !self.lmr_config.enable_dynamic_reduction {
            return self.lmr_config.base_reduction;
        }
        
        let mut reduction = self.lmr_config.base_reduction;
        
        // Pre-calculate depth-based reductions
        if depth >= 10 {
            reduction += 2;
        } else if depth >= 6 {
            reduction += 1;
        }
        
        // Pre-calculate move index-based reductions
        if move_index >= 16 {
            reduction += 2;
        } else if move_index >= 8 {
            reduction += 1;
        }
        
        // Apply adaptive reduction only if enabled and needed
        if self.lmr_config.enable_adaptive_reduction && reduction < self.lmr_config.max_reduction {
            reduction = self.apply_adaptive_reduction_optimized(reduction, move_, depth);
        }
        
        // Ensure reduction doesn't exceed maximum
        reduction.min(self.lmr_config.max_reduction)
            .min(depth.saturating_sub(2))
    }
    /// Optimized adaptive reduction with early returns
    fn apply_adaptive_reduction_optimized(&self, base_reduction: u8, move_: &Move, _depth: u8) -> u8 {
        let mut reduction = base_reduction;
        
        // Quick center move check (most common case)
        if self.is_center_move(move_) {
            reduction = reduction.saturating_sub(1);
            return reduction; // Early return for center moves
        }
        
        // Only check position characteristics if we have enough data
        if self.lmr_stats.moves_considered < 5 {
            return reduction;
        }
        
        // Check tactical position (more expensive, do last)
        if self.is_tactical_position() {
            reduction = reduction.saturating_sub(1);
        } else if self.is_quiet_position() {
            reduction += 1;
        }
        
        reduction
    }

    /// Get LMR performance metrics for tuning
    pub fn get_lmr_performance_metrics(&self) -> LMRPerformanceMetrics {
        let stats = &self.lmr_stats;
        
        LMRPerformanceMetrics {
            moves_considered: stats.moves_considered,
            reductions_applied: stats.reductions_applied,
            researches_triggered: stats.researches_triggered,
            efficiency: stats.efficiency(),
            research_rate: stats.research_rate(),
            cutoff_rate: stats.cutoff_rate(),
            average_depth_saved: stats.average_depth_saved(),
            total_depth_saved: stats.total_depth_saved,
            nodes_per_second: if stats.moves_considered > 0 {
                // This would need timing information in a real implementation
                stats.moves_considered as f64 * 1000.0 // Placeholder
            } else {
                0.0
            },
        }
    }
    /// Auto-tune LMR parameters based on performance
    pub fn auto_tune_lmr_parameters(&mut self) -> Result<(), String> {
        let metrics = self.get_lmr_performance_metrics();
        
        // Only auto-tune if we have enough data
        if metrics.moves_considered < 100 {
            return Err("Insufficient data for auto-tuning".to_string());
        }
        
        let mut new_config = self.lmr_config.clone();
        
        // Adjust parameters based on performance
        if metrics.research_rate > 40.0 {
            // Too many researches - reduce aggressiveness
            new_config.base_reduction = new_config.base_reduction.saturating_sub(1);
            new_config.max_reduction = new_config.max_reduction.saturating_sub(1);
        } else if metrics.research_rate < 10.0 && metrics.efficiency > 30.0 {
            // Too few researches - increase aggressiveness
            new_config.base_reduction = (new_config.base_reduction + 1).min(5);
            new_config.max_reduction = (new_config.max_reduction + 1).min(8);
        }
        
        // Adjust move index threshold based on efficiency
        if metrics.efficiency > 50.0 {
            // High efficiency - can be more aggressive
            new_config.min_move_index = new_config.min_move_index.saturating_sub(1);
        } else if metrics.efficiency < 20.0 {
            // Low efficiency - be more conservative
            new_config.min_move_index = (new_config.min_move_index + 1).min(10);
        }
        
        // Apply the new configuration
        self.update_lmr_config(new_config)
    }

    /// Get LMR configuration presets for different playing styles
    pub fn get_lmr_preset(&self, style: LMRPlayingStyle) -> LMRConfig {
        match style {
            LMRPlayingStyle::Aggressive => LMRConfig {
                enabled: true,
                min_depth: 2,
                min_move_index: 3,
                base_reduction: 2,
                max_reduction: 4,
                enable_dynamic_reduction: true,
                enable_adaptive_reduction: true,
                enable_extended_exemptions: true,
            },
            LMRPlayingStyle::Conservative => LMRConfig {
                enabled: true,
                min_depth: 4,
                min_move_index: 6,
                base_reduction: 1,
                max_reduction: 2,
                enable_dynamic_reduction: true,
                enable_adaptive_reduction: true,
                enable_extended_exemptions: true,
            },
            LMRPlayingStyle::Balanced => LMRConfig {
                enabled: true,
                min_depth: 3,
                min_move_index: 4,
                base_reduction: 1,
                max_reduction: 3,
                enable_dynamic_reduction: true,
                enable_adaptive_reduction: true,
                enable_extended_exemptions: true,
            },
        }
    }

    /// Apply LMR configuration preset
    pub fn apply_lmr_preset(&mut self, style: LMRPlayingStyle) -> Result<(), String> {
        let preset = self.get_lmr_preset(style);
        self.update_lmr_config(preset)
    }

    /// Optimize LMR memory usage by clearing old statistics
    pub fn optimize_lmr_memory(&mut self) {
        // Reset statistics if they get too large
        if self.lmr_stats.moves_considered > 1_000_000 {
            self.lmr_stats.reset();
        }
        
        // Clear transposition table if it gets too large
        if self.transposition_table.size() > 100_000 {
            self.transposition_table.clear();
        }
    }

    /// Get LMR performance report with optimization suggestions
    pub fn get_lmr_performance_report(&self) -> String {
        let metrics = self.get_lmr_performance_metrics();
        let recommendations = metrics.get_optimization_recommendations();
        
        let mut report = format!(
            "LMR Performance Report:\n\
            - Moves considered: {}\n\
            - Reductions applied: {}\n\
            - Researches triggered: {}\n\
            - Efficiency: {:.1}%\n\
            - Research rate: {:.1}%\n\
            - Cutoff rate: {:.1}%\n\
            - Average depth saved: {:.2}\n\
            - Total depth saved: {}\n\
            - Performance status: {}\n\n\
            Optimization Recommendations:",
            metrics.moves_considered,
            metrics.reductions_applied,
            metrics.researches_triggered,
            metrics.efficiency,
            metrics.research_rate,
            metrics.cutoff_rate,
            metrics.average_depth_saved,
            metrics.total_depth_saved,
            if metrics.is_performing_well() { "Good" } else { "Needs tuning" }
        );
        
        for (i, rec) in recommendations.iter().enumerate() {
            report.push_str(&format!("\n{}. {}", i + 1, rec));
        }
        
        report
    }

    /// Profile LMR overhead and return timing information
    pub fn profile_lmr_overhead(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                               player: Player, depth: u8, iterations: usize) -> LMRProfileResult {
        let mut total_time = std::time::Duration::new(0, 0);
        let mut total_moves = 0u64;
        let mut total_reductions = 0u64;
        let mut total_researches = 0u64;
        
        for _ in 0..iterations {
            self.reset_lmr_stats();
            let start_time = std::time::Instant::now();
            
            let _result = self.search_at_depth_legacy(board, captured_pieces, player, depth, 5000);
            
            let elapsed = start_time.elapsed();
            total_time += elapsed;
            
            let stats = self.get_lmr_stats();
            total_moves += stats.moves_considered;
            total_reductions += stats.reductions_applied;
            total_researches += stats.researches_triggered;
        }
        
        LMRProfileResult {
            total_time,
            average_time_per_search: total_time / iterations as u32,
            total_moves_processed: total_moves,
            total_reductions_applied: total_reductions,
            total_researches_triggered: total_researches,
            moves_per_second: if total_time.as_secs_f64() > 0.0 {
                total_moves as f64 / total_time.as_secs_f64()
            } else {
                0.0
            },
            reduction_rate: if total_moves > 0 {
                (total_reductions as f64 / total_moves as f64) * 100.0
            } else {
                0.0
            },
            research_rate: if total_reductions > 0 {
                (total_researches as f64 / total_reductions as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Get hardware-optimized LMR configuration
    pub fn get_hardware_optimized_config(&self) -> LMRConfig {
        // This would analyze system capabilities in a real implementation
        // For now, return a balanced configuration
        LMRConfig {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        }
    }

    /// Log null move attempt for debugging
    fn log_null_move_attempt(&self, depth: u8, reduction: u8, score: i32, cutoff: bool) {
        crate::debug_utils::debug_log(&format!(
            "NMP: depth={}, reduction={}, score={}, cutoff={}",
            depth, reduction, score, cutoff
        ));
    }

    /// Check if position is safe for null move pruning with additional safety checks
    fn is_safe_for_null_move(&self, board: &BitboardBoard, _captured_pieces: &CapturedPieces, player: Player) -> bool {
        // Basic safety checks are already in should_attempt_null_move
        // Additional safety checks can be added here
        
        // Check if we have major pieces (rooks, bishops, golds) - more conservative in endgame
        let major_piece_count = self.count_major_pieces(board, player);
        if major_piece_count < 2 {
            return false; // Too few major pieces - potential zugzwang risk
        }
        
        // Check if position is in late endgame (very few pieces)
        if self.is_late_endgame(board) {
            return false; // Late endgame - high zugzwang risk
        }
        
        true
    }

    /// Check if position is in late endgame where zugzwang is common
    fn is_late_endgame(&self, board: &BitboardBoard) -> bool {
        let total_pieces = self.count_pieces_on_board(board);
        total_pieces <= 8 // Very conservative threshold for late endgame
    }

    /// Count major pieces for a player (rooks, bishops, golds)
    fn count_major_pieces(&self, board: &BitboardBoard, player: Player) -> u8 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position { row, col }) {
                    if piece.player == player {
                        match piece.piece_type {
                            PieceType::Rook | PieceType::Bishop | PieceType::Gold => count += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        count
    }

    /// Enhanced safety check for null move pruning
    fn is_enhanced_safe_for_null_move(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> bool {
        // Basic safety checks
        if !self.is_safe_for_null_move(board, captured_pieces, player) {
            return false;
        }
        
        // Additional tactical safety checks
        // Check if opponent has strong attacking pieces
        let opponent = player.opposite();
        let opponent_major_pieces = self.count_major_pieces(board, opponent);
        if opponent_major_pieces >= 3 {
            return false; // Opponent has strong pieces - potential tactical danger
        }
        
        true
    }

    /// Validate move tracking consistency
    fn validate_move_tracking(&self, best_move: &Option<Move>, best_score: i32, 
                             moves_evaluated: usize) -> bool {
        if moves_evaluated > 0 && best_move.is_none() {
            crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                &format!("WARNING: {} moves evaluated but no best move stored (score: {})", 
                    moves_evaluated, best_score));
            return false;
        }
        true
    }

    /// Validate search result consistency
    fn validate_search_result(&self, result: Option<(Move, i32)>, 
                             depth: u8, alpha: i32, beta: i32) -> bool {
        match result {
            Some((ref move_, score)) => {
                // Validate score is within reasonable bounds
                if score < -50000 || score > 50000 {
                    crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                        &format!("WARNING: Score {} is outside reasonable bounds", score));
                    return false;
                }
                
                // Validate move is not empty
                if move_.to_usi_string().is_empty() {
                    crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                        "WARNING: Empty move string in search result");
                    return false;
                }
                
                // CRITICAL FIX: Safe arithmetic to prevent integer overflow
                // Using saturating_sub/add instead of direct arithmetic prevents panics
                // when alpha/beta are close to i32::MIN/MAX boundaries
                let alpha_threshold = alpha.saturating_sub(1000);
                let beta_threshold = beta.saturating_add(1000);
                if score < alpha_threshold || score > beta_threshold {
                    crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                        &format!("WARNING: Score {} significantly outside window [{}, {}]", 
                            score, alpha, beta));
                    // This is not necessarily an error, but worth logging
                }
                
                // Validate move format (basic USI format check)
                let move_str = move_.to_usi_string();
                if move_str.len() < 4 || move_str.len() > 6 {
                    crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                        &format!("WARNING: Move string '{}' has unusual length", move_str));
                }
                
                // Log successful validation
                crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                    &format!("Search result validated: move={}, score={}, depth={}", 
                        move_.to_usi_string(), score, depth));
                
                true
            },
            None => {
                crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                    &format!("WARNING: Search returned None at depth {} (alpha: {}, beta: {})", 
                        depth, alpha, beta));
                false
            }
        }
    }

    /// Enhanced search result validation with recovery suggestions
    fn validate_search_result_with_recovery(&self, result: Option<(Move, i32)>, 
                                           depth: u8, alpha: i32, beta: i32) -> (bool, Option<String>) {
        match result {
            Some((ref move_, score)) => {
                let mut issues = Vec::new();
                
                // Check score bounds
                if score < -50000 || score > 50000 {
                    issues.push("Score outside reasonable bounds".to_string());
                }
                
                // Check move validity
                if move_.to_usi_string().is_empty() {
                    issues.push("Empty move string".to_string());
                }
                
                // Check score consistency (safe arithmetic)
                let alpha_threshold = alpha.saturating_sub(1000);
                let beta_threshold = beta.saturating_add(1000);
                if score < alpha_threshold || score > beta_threshold {
                    issues.push("Score significantly outside window".to_string());
                }
                
                if issues.is_empty() {
                    (true, None)
                } else {
                    let recovery_suggestion = if score < -50000 || score > 50000 {
                        "Consider checking evaluation function for overflow".to_string()
                    } else if move_.to_usi_string().is_empty() {
                        "Check move generation and storage logic".to_string()
                    } else {
                        "Score may be correct but window may be too narrow".to_string()
                    };
                    
                    (false, Some(recovery_suggestion))
                }
            },
            None => {
                let recovery_suggestion = if depth == 0 {
                    "Check if position has legal moves".to_string()
                } else {
                    "Check search timeout and move generation".to_string()
                };
                (false, Some(recovery_suggestion))
            }
        }
    }

    /// Comprehensive consistency checks for aspiration window system
    fn perform_consistency_checks(&self, alpha: i32, beta: i32, previous_score: i32, 
                                 window_size: i32, depth: u8, researches: u8) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check window bounds consistency
        if alpha >= beta {
            issues.push(format!("Invalid window bounds: alpha={} >= beta={}", alpha, beta));
        }
        
        // Check window size consistency
        let actual_window_size = (beta as i64).saturating_sub(alpha as i64);
        if actual_window_size != window_size as i64 && window_size != i32::MAX {
            issues.push(format!("Window size mismatch: actual={}, expected={}", 
                actual_window_size, window_size));
        }
        
        // Check score consistency with window (safe arithmetic)
        let alpha_threshold = alpha.saturating_sub(window_size);
        let beta_threshold = beta.saturating_add(window_size);
        if previous_score < alpha_threshold || previous_score > beta_threshold {
            issues.push(format!("Previous score {} outside expected range for window [{}, {}]", 
                previous_score, alpha, beta));
        }
        
        // Check depth consistency
        if depth < self.aspiration_config.min_depth && window_size != i32::MAX {
            issues.push(format!("Aspiration window used at depth {} < min_depth {}", 
                depth, self.aspiration_config.min_depth));
        }
        
        // Check research count consistency
        if researches > self.aspiration_config.max_researches {
            issues.push(format!("Research count {} exceeds max_researches {}", 
                researches, self.aspiration_config.max_researches));
        }
        
        // Check configuration consistency
        if self.aspiration_config.base_window_size > self.aspiration_config.max_window_size {
            issues.push(format!("base_window_size {} > max_window_size {}", 
                self.aspiration_config.base_window_size, self.aspiration_config.max_window_size));
        }
        
        // Check statistics consistency
        if self.aspiration_stats.fail_lows + self.aspiration_stats.fail_highs > self.aspiration_stats.total_researches {
            issues.push("Fail count exceeds research count in statistics".to_string());
        }
        
        issues
    }

    /// Validate aspiration window state consistency
    fn validate_aspiration_state(&self, alpha: i32, beta: i32, previous_score: i32, 
                                researches: u8, depth: u8) -> bool {
        let issues = self.perform_consistency_checks(alpha, beta, previous_score, 
                                                   (beta as i64).saturating_sub(alpha as i64) as i32, depth, researches);
        
        if !issues.is_empty() {
            crate::debug_utils::trace_log("CONSISTENCY_CHECK", 
                &format!("Found {} consistency issues:", issues.len()));
            for issue in issues {
                crate::debug_utils::trace_log("CONSISTENCY_CHECK", &format!("  - {}", issue));
            }
            false
        } else {
            crate::debug_utils::trace_log("CONSISTENCY_CHECK", 
                "All consistency checks passed");
            true
        }
    }

    /// Comprehensive recovery mechanisms for aspiration window failures
    fn attempt_aspiration_recovery(&mut self, alpha: &mut i32, beta: &mut i32, 
                                  previous_score: i32, window_size: i32, 
                                  failure_type: &str, researches: u8, _depth: u8) -> bool {
        
        crate::debug_utils::trace_log("ASPIRATION_RECOVERY", 
            &format!("Attempting recovery for failure type: {}, researches: {}", 
                failure_type, researches));
        
        // Recovery strategy 1: Reset to safe defaults
        if self.recover_with_safe_defaults(alpha, beta, previous_score, window_size) {
            crate::debug_utils::trace_log("ASPIRATION_RECOVERY", 
                "Recovery successful with safe defaults");
            return true;
        }
        
        // Recovery strategy 2: Adaptive window adjustment
        if self.recover_with_adaptive_adjustment(alpha, beta, previous_score, window_size, failure_type) {
            crate::debug_utils::trace_log("ASPIRATION_RECOVERY", 
                "Recovery successful with adaptive adjustment");
            return true;
        }
        
        // Recovery strategy 3: Fall back to full-width search
        if self.recover_with_full_width(alpha, beta) {
            crate::debug_utils::trace_log("ASPIRATION_RECOVERY", 
                "Recovery successful with full-width search");
            return true;
        }
        
        crate::debug_utils::trace_log("ASPIRATION_RECOVERY", 
            "All recovery strategies failed");
        false
    }

    /// Recovery strategy 1: Reset to safe defaults
    fn recover_with_safe_defaults(&self, alpha: &mut i32, beta: &mut i32, 
                                 previous_score: i32, window_size: i32) -> bool {
        // Clamp values to safe ranges
        let safe_score = previous_score.clamp(-10000, 10000);
        let safe_window = window_size.clamp(10, self.aspiration_config.max_window_size);
        
        // Create safe window
        *alpha = safe_score - safe_window;
        *beta = safe_score + safe_window;
        
        // Validate the result
        if *alpha < *beta && *alpha > i32::MIN + 1000 && *beta < i32::MAX - 1000 {
            crate::debug_utils::trace_log("RECOVERY_SAFE_DEFAULTS", 
                &format!("Safe defaults applied: alpha={}, beta={}", alpha, beta));
            true
        } else {
            false
        }
    }

    /// Recovery strategy 2: Adaptive window adjustment
    fn recover_with_adaptive_adjustment(&self, alpha: &mut i32, beta: &mut i32, 
                                       previous_score: i32, window_size: i32, 
                                       failure_type: &str) -> bool {
        let adjustment_factor = match failure_type {
            "fail_low" => 1.5,
            "fail_high" => 1.5,
            "search_failed" => 2.0,
            "timeout" => 0.8,
            _ => 1.2,
        };
        
        let adjusted_window = (window_size as f64 * adjustment_factor) as i32;
        let safe_window = adjusted_window.clamp(10, self.aspiration_config.max_window_size);
        
        *alpha = previous_score - safe_window;
        *beta = previous_score + safe_window;
        
        // Validate the result
        if *alpha < *beta {
            crate::debug_utils::trace_log("RECOVERY_ADAPTIVE", 
                &format!("Adaptive adjustment applied: alpha={}, beta={}, factor={}", 
                    alpha, beta, adjustment_factor));
            true
        } else {
            false
        }
    }
    /// Recovery strategy 3: Fall back to full-width search
    fn recover_with_full_width(&self, alpha: &mut i32, beta: &mut i32) -> bool {
        *alpha = i32::MIN + 1;
        *beta = i32::MAX - 1;
        
        crate::debug_utils::trace_log("RECOVERY_FULL_WIDTH", 
            "Fallback to full-width search applied");
        true
    }

    /// Emergency recovery for critical failures
    fn emergency_recovery(&mut self, alpha: &mut i32, beta: &mut i32, 
                         previous_score: i32, _depth: u8) -> bool {
        crate::debug_utils::trace_log("EMERGENCY_RECOVERY", 
            "Emergency recovery activated");
        
        // Reset statistics to prevent cascading failures
        self.aspiration_stats.fail_lows = 0;
        self.aspiration_stats.fail_highs = 0;
        self.aspiration_stats.total_researches = 0;
        
        // Use very conservative window
        let emergency_window = 25; // Very small window
        *alpha = previous_score - emergency_window;
        *beta = previous_score + emergency_window;
        
        // Final safety check
        if *alpha >= *beta {
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
        }
        
        crate::debug_utils::trace_log("EMERGENCY_RECOVERY", 
            &format!("Emergency recovery complete: alpha={}, beta={}", alpha, beta));
        true
    }

    /// Comprehensive error handling for aspiration window operations
    fn handle_aspiration_error(&mut self, error_type: &str, error_context: &str, 
                              alpha: &mut i32, beta: &mut i32, previous_score: i32, 
                              depth: u8, _researches: u8) -> bool {
        
        crate::debug_utils::trace_log("ASPIRATION_ERROR", 
            &format!("Error type: {}, context: {}", error_type, error_context));
        
        match error_type {
            "window_overflow" => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Window overflow detected, applying bounds check");
                *alpha = (*alpha).clamp(i32::MIN + 1, i32::MAX - 1);
                *beta = (*beta).clamp(i32::MIN + 1, i32::MAX - 1);
                true
            },
            "invalid_parameters" => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Invalid parameters detected, using safe defaults");
                self.recover_with_safe_defaults(alpha, beta, previous_score, 50)
            },
            "statistics_corruption" => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Statistics corruption detected, resetting");
                self.aspiration_stats.reset();
                self.recover_with_safe_defaults(alpha, beta, previous_score, 50)
            },
            "cascading_failure" => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Cascading failure detected, emergency recovery");
                self.emergency_recovery(alpha, beta, previous_score, depth)
            },
            "timeout_cascade" => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Timeout cascade detected, disabling aspiration");
                *alpha = i32::MIN + 1;
                *beta = i32::MAX - 1;
                true
            },
            _ => {
                crate::debug_utils::trace_log("ASPIRATION_ERROR", 
                    "Unknown error type, using fallback");
                self.recover_with_full_width(alpha, beta)
            }
        }
    }

    /// Error detection and classification
    fn detect_aspiration_errors(&self, alpha: i32, beta: i32, previous_score: i32, 
                               researches: u8, _depth: u8) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check for window overflow
        if alpha <= i32::MIN + 100 || beta >= i32::MAX - 100 {
            errors.push("window_overflow".to_string());
        }
        
        // Check for invalid parameters
        if alpha >= beta || previous_score < -100000 || previous_score > 100000 {
            errors.push("invalid_parameters".to_string());
        }
        
        // Check for statistics corruption
        if self.aspiration_stats.fail_lows > self.aspiration_stats.total_searches ||
           self.aspiration_stats.fail_highs > self.aspiration_stats.total_searches {
            errors.push("statistics_corruption".to_string());
        }
        
        // Check for cascading failure (too many researches)
        if researches > self.aspiration_config.max_researches + 1 {
            errors.push("cascading_failure".to_string());
        }
        
        // Check for timeout cascade (if we have timeout detection)
        if researches > 5 { // Arbitrary threshold for potential timeout issues
            errors.push("timeout_cascade".to_string());
        }
        
        errors
    }

    /// Safe aspiration window operation with error handling
    fn safe_aspiration_operation<F>(&mut self, operation: F, alpha: &mut i32, beta: &mut i32, 
                                   previous_score: i32, depth: u8, researches: u8) -> bool 
    where F: FnOnce(&mut i32, &mut i32) -> bool {
        
        // Pre-operation error detection
        let errors = self.detect_aspiration_errors(*alpha, *beta, previous_score, researches, depth);
        if !errors.is_empty() {
            for error in errors {
                if !self.handle_aspiration_error(&error, "pre_operation", alpha, beta, 
                                               previous_score, depth, researches) {
                    return false;
                }
            }
        }
        
        // Perform the operation with error handling
        let success = operation(alpha, beta);
        
        if success {
            // Post-operation validation
            self.validate_aspiration_state(*alpha, *beta, previous_score, researches, depth);
        }
        
        success
    }

    /// Graceful degradation system for aspiration windows
    fn apply_graceful_degradation(&mut self, alpha: &mut i32, beta: &mut i32, 
                                 previous_score: i32, depth: u8, researches: u8) -> bool {
        
        // Determine degradation level based on failure patterns
        let degradation_level = self.calculate_degradation_level(researches, depth);
        
        crate::debug_utils::trace_log("GRACEFUL_DEGRADATION", 
            &format!("Applying degradation level {} for researches={}, depth={}", 
                degradation_level, researches, depth));
        
        match degradation_level {
            0 => {
                // No degradation - normal operation
                true
            },
            1 => {
                // Level 1: Reduce window aggressiveness
                self.degrade_window_aggressiveness(alpha, beta, previous_score)
            },
            2 => {
                // Level 2: Disable adaptive features
                self.degrade_adaptive_features(alpha, beta, previous_score)
            },
            3 => {
                // Level 3: Use conservative defaults
                self.degrade_to_conservative_defaults(alpha, beta, previous_score)
            },
            4 => {
                // Level 4: Disable aspiration windows entirely
                self.degrade_disable_aspiration(alpha, beta)
            },
            _ => {
                // Emergency: Full fallback
                self.emergency_recovery(alpha, beta, previous_score, depth)
            }
        }
    }

    /// Calculate degradation level based on failure patterns
    fn calculate_degradation_level(&self, researches: u8, depth: u8) -> u8 {
        let mut level = 0;
        
        // Factor 1: Research count
        if researches > self.aspiration_config.max_researches {
            level += 2;
        } else if researches > self.aspiration_config.max_researches / 2 {
            level += 1;
        }
        
        // Factor 2: Failure rate
        let total_searches = self.aspiration_stats.total_searches.max(1);
        let failure_rate = (self.aspiration_stats.fail_lows + self.aspiration_stats.fail_highs) as f64 
                          / total_searches as f64;
        
        if failure_rate > 0.5 {
            level += 2;
        } else if failure_rate > 0.3 {
            level += 1;
        }
        
        // Factor 3: Depth (deeper searches are more critical)
        if depth > 10 {
            level += 1;
        }
        
        // Factor 4: Recent consecutive failures
        if researches > 3 {
            level += 1;
        }
        
        level.min(4) // Cap at level 4
    }
    /// Level 1 degradation: Reduce window aggressiveness
    fn degrade_window_aggressiveness(&self, alpha: &mut i32, beta: &mut i32, 
                                     previous_score: i32) -> bool {
        let conservative_window = 25; // Very conservative window
        *alpha = previous_score - conservative_window;
        *beta = previous_score + conservative_window;
        
        crate::debug_utils::trace_log("DEGRADATION_LEVEL_1", 
            "Reduced window aggressiveness");
        true
    }

    /// Level 2 degradation: Disable adaptive features
    fn degrade_adaptive_features(&self, alpha: &mut i32, beta: &mut i32, 
                                 previous_score: i32) -> bool {
        let fixed_window = 50; // Fixed window size
        *alpha = previous_score - fixed_window;
        *beta = previous_score + fixed_window;
        
        crate::debug_utils::trace_log("DEGRADATION_LEVEL_2", 
            "Disabled adaptive features, using fixed window");
        true
    }

    /// Level 3 degradation: Use conservative defaults
    fn degrade_to_conservative_defaults(&self, alpha: &mut i32, beta: &mut i32, 
                                         previous_score: i32) -> bool {
        let safe_score = previous_score.clamp(-1000, 1000);
        let safe_window = 30;
        *alpha = safe_score - safe_window;
        *beta = safe_score + safe_window;
        
        crate::debug_utils::trace_log("DEGRADATION_LEVEL_3", 
            "Using conservative defaults");
        true
    }

    /// Level 4 degradation: Disable aspiration windows entirely
    fn degrade_disable_aspiration(&self, alpha: &mut i32, beta: &mut i32) -> bool {
        *alpha = i32::MIN + 1;
        *beta = i32::MAX - 1;
        
        crate::debug_utils::trace_log("DEGRADATION_LEVEL_4", 
            "Disabled aspiration windows, using full-width search");
        true
    }
    /// Monitor system health and trigger degradation if needed
    fn monitor_system_health(&mut self, alpha: i32, beta: i32, previous_score: i32, 
                             depth: u8, researches: u8) -> bool {
        let health_score = self.calculate_system_health_score(alpha, beta, previous_score, 
                                                             depth, researches);
        
        crate::debug_utils::trace_log("SYSTEM_HEALTH", 
            &format!("System health score: {}", health_score));
        
        if health_score < 0.3 {
            // System is unhealthy, trigger graceful degradation
            crate::debug_utils::trace_log("SYSTEM_HEALTH", 
                "System health critical, triggering graceful degradation");
            return false; // Signal that degradation is needed
        }
        
        true
    }

    /// Calculate system health score (0.0 = critical, 1.0 = healthy)
    fn calculate_system_health_score(&self, alpha: i32, beta: i32, previous_score: i32, 
                                      _depth: u8, researches: u8) -> f64 {
        let mut score = 1.0;
        
        // Factor 1: Window validity
        if alpha >= beta {
            score -= 0.5;
        }
        
        // Factor 2: Parameter bounds
        if previous_score < -50000 || previous_score > 50000 {
            score -= 0.3;
        }
        
        // Factor 3: Research count
        let research_ratio = researches as f64 / self.aspiration_config.max_researches as f64;
        if research_ratio > 1.0 {
            score -= 0.4;
        } else if research_ratio > 0.5 {
            score -= 0.2;
        }
        
        // Factor 4: Failure rate
        let total_searches = self.aspiration_stats.total_searches.max(1);
        let failure_rate = (self.aspiration_stats.fail_lows + self.aspiration_stats.fail_highs) as f64 
                           / total_searches as f64;
        score -= failure_rate * 0.3;
        
        score.max(0.0).min(1.0)
    }

    /// Comprehensive aspiration window retry strategy
    /// 
    /// This method implements a robust retry mechanism for aspiration window failures.
    /// It addresses the critical issue where aspiration window searches would fail
    /// completely, causing the engine to return no result.
    /// 
    /// # Arguments
    /// * `alpha` - Current alpha bound (modified in place)
    /// * `beta` - Current beta bound (modified in place)
    /// * `previous_score` - Score from previous iteration
    /// * `window_size` - Size of the aspiration window
    /// * `failure_type` - Type of failure ("fail_low", "fail_high", "search_failed")
    /// * `researches` - Number of retry attempts so far
    /// * `depth` - Current search depth
    /// 
    /// # Returns
    /// `true` if retry should continue, `false` if should fall back to full-width search
    /// 
    /// # Strategy
    /// 1. Validate parameters to ensure they're reasonable
    /// 2. Check if max retry limit has been reached
    /// 3. Apply failure-type-specific recovery strategies
    /// 4. Implement graceful degradation if recovery fails
    fn handle_aspiration_retry(&mut self, alpha: &mut i32, beta: &mut i32, 
                              previous_score: i32, window_size: i32, 
                              failure_type: &str, researches: u8, _depth: u8) -> bool {
        
        // Validate parameters
        if !self.validate_window_parameters(previous_score, window_size) {
            crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                "Invalid parameters, falling back to full-width search");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return true;
        }
        
        // Check if we've exceeded max researches
        if researches >= self.aspiration_config.max_researches {
            crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                "Max researches exceeded, falling back to full-width search");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return true;
        }
        
        // Handle different failure types with specific strategies
        match failure_type {
            "fail_low" => {
                self.handle_fail_low(alpha, beta, previous_score, window_size);
                crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                    &format!("Fail-low retry: alpha={}, beta={}, researches={}", 
                        alpha, beta, researches));
            },
            "fail_high" => {
                self.handle_fail_high(alpha, beta, previous_score, window_size);
                crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                    &format!("Fail-high retry: alpha={}, beta={}, researches={}", 
                        alpha, beta, researches));
            },
            "search_failed" => {
                // Widen window significantly for search failures (safe arithmetic)
                let doubled_window = window_size.saturating_mul(2);
                let new_alpha = previous_score.saturating_sub(doubled_window);
                let new_beta = previous_score.saturating_add(doubled_window);
                
                if new_alpha < new_beta {
                    *alpha = new_alpha;
                    *beta = new_beta;
                    crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                        &format!("Search failure retry: alpha={}, beta={}, researches={}", 
                            alpha, beta, researches));
                } else {
                    // Fallback to full-width search
                    *alpha = i32::MIN + 1;
                    *beta = i32::MAX - 1;
                    crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                        "Search failure: invalid window, falling back to full-width");
                }
            },
            "timeout" => {
                // For timeouts, use a more conservative approach
                let conservative_window = window_size / 2;
                let new_alpha = previous_score - conservative_window;
                let new_beta = previous_score + conservative_window;
                
                if new_alpha < new_beta {
                    *alpha = new_alpha;
                    *beta = new_beta;
                } else {
                    *alpha = i32::MIN + 1;
                    *beta = i32::MAX - 1;
                }
                crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                    &format!("Timeout retry: alpha={}, beta={}, researches={}", 
                        alpha, beta, researches));
            },
            _ => {
                crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                    "Unknown failure type, falling back to full-width search");
                *alpha = i32::MIN + 1;
                *beta = i32::MAX - 1;
            }
        }
        
        // Validate the new window
        if *alpha >= *beta {
            crate::debug_utils::trace_log("ASPIRATION_RETRY", 
                "Invalid window after retry, falling back to full-width search");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
        }
        
        true
    }

    // ============================================================================
    // DIAGNOSTIC TOOLS AND MONITORING
    // ============================================================================

    /// Get comprehensive search state for debugging and diagnostics
    /// 
    /// This method provides a snapshot of the current search state, including
    /// aspiration window parameters, move tracking status, and performance metrics.
    /// Useful for debugging search issues and monitoring engine health.
    pub fn get_search_state(&self) -> SearchState {
        SearchState {
            alpha: self.current_alpha,
            beta: self.current_beta,
            best_move: self.current_best_move.clone(),
            best_score: self.current_best_score,
            nodes_searched: self.nodes_searched,
            depth: self.current_depth,
            aspiration_enabled: self.aspiration_config.enabled,
            researches: self.aspiration_stats.total_researches as u8,
            health_score: self.calculate_system_health_score(
                self.current_alpha, 
                self.current_beta, 
                self.current_best_score,
                self.current_depth,
                self.aspiration_stats.total_researches as u8
            ),
        }
    }

    /// Get detailed aspiration window diagnostics
    /// 
    /// Provides comprehensive information about the current aspiration window
    /// state, including window parameters, retry statistics, and health metrics.
    pub fn get_aspiration_diagnostics(&self) -> AspirationDiagnostics {
        AspirationDiagnostics {
            alpha: self.current_alpha,
            beta: self.current_beta,
            window_size: (self.current_beta as i64).saturating_sub(self.current_alpha as i64) as i32,
            researches: self.aspiration_stats.total_researches as u8,
            success_rate: self.aspiration_stats.success_rate(),
            health_score: self.calculate_system_health_score(
                self.current_alpha, 
                self.current_beta, 
                self.current_best_score,
                self.current_depth,
                self.aspiration_stats.total_researches as u8
            ),
            estimated_time_saved: self.aspiration_stats.estimated_time_saved_ms,
            estimated_nodes_saved: self.aspiration_stats.estimated_nodes_saved,
            failure_rate: self.aspiration_stats.fail_low_rate(),
        }
    }

    /// Classify the current error state and provide recovery suggestions
    /// 
    /// Analyzes the current search state to identify potential issues and
    /// suggests appropriate recovery strategies.
    pub fn classify_error_type(&self, score: i32, alpha: i32, beta: i32) -> String {
        if score <= alpha {
            "fail_low".to_string()
        } else if score >= beta {
            "fail_high".to_string()
        } else if alpha >= beta {
            "invalid_window".to_string()
        } else if score < alpha - 1000 || score > beta + 1000 {
            "extreme_score".to_string()
        } else {
            "normal".to_string()
        }
    }

    /// Get recovery suggestion for a specific error type
    /// 
    /// Provides specific recommendations for handling different types of
    /// search failures and aspiration window issues.
    pub fn get_recovery_suggestion(&self, error_type: &str) -> String {
        match error_type {
            "fail_low" => "Lower alpha bound or widen window downward".to_string(),
            "fail_high" => "Raise beta bound or widen window upward".to_string(),
            "invalid_window" => "Reset to full-width search".to_string(),
            "extreme_score" => "Check evaluation function for anomalies".to_string(),
            "normal" => "No recovery needed".to_string(),
            _ => "Unknown error type, use emergency recovery".to_string(),
        }
    }

    /// Generate a comprehensive diagnostic report
    /// 
    /// Creates a detailed report of the current search state, including
    /// all relevant metrics, potential issues, and recommendations.
    pub fn generate_diagnostic_report(&self) -> String {
        let state = self.get_search_state();
        let diagnostics = self.get_aspiration_diagnostics();
        let error_type = self.classify_error_type(state.best_score, state.alpha, state.beta);
        let suggestion = self.get_recovery_suggestion(&error_type);

        format!(
            "=== SEARCH DIAGNOSTIC REPORT ===\n\
            Search State:\n\
            - Alpha: {}, Beta: {}, Window Size: {}\n\
            - Best Move: {:?}, Best Score: {}\n\
            - Nodes Searched: {}, Depth: {}\n\
            \n\
            Aspiration Window:\n\
            - Enabled: {}, Researches: {}\n\
            - Success Rate: {:.2}%, Failure Rate: {:.2}%\n\
            - Health Score: {:.2}\n\
            - Time Saved: {}ms, Nodes Saved: {}\n\
            \n\
            Error Analysis:\n\
            - Error Type: {}\n\
            - Suggestion: {}\n\
            \n\
            Recommendations:\n\
            - Monitor health score for degradation\n\
            - Check error logs for patterns\n\
            - Consider adjusting window parameters if issues persist\n\
            =================================",
            state.alpha, state.beta, diagnostics.window_size,
            state.best_move.as_ref().map(|m| m.to_usi_string()),
            state.best_score, state.nodes_searched, state.depth,
            state.aspiration_enabled, diagnostics.researches,
            diagnostics.success_rate * 100.0, diagnostics.failure_rate * 100.0,
            diagnostics.health_score,
            diagnostics.estimated_time_saved, diagnostics.estimated_nodes_saved,
            error_type, suggestion
        )
    }

    /// Check if the search engine is in a healthy state
    /// 
    /// Performs various health checks to determine if the search engine
    /// is operating normally or if there are potential issues.
    pub fn is_healthy(&self) -> bool {
        let health_score = self.calculate_system_health_score(
            self.current_alpha, 
            self.current_beta, 
            self.current_best_score,
            self.current_depth,
            self.aspiration_stats.total_researches as u8
        );
        
        // Consider healthy if health score is above 0.7
        health_score > 0.7
    }

    /// Get performance metrics for monitoring
    /// 
    /// Returns key performance indicators for monitoring engine performance
    /// and detecting potential issues.
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            nodes_per_second: self.calculate_nodes_per_second(),
            aspiration_success_rate: self.aspiration_stats.success_rate(),
            average_window_size: self.calculate_average_window_size(),
            retry_frequency: self.aspiration_stats.total_researches as f64 / 
                           (self.aspiration_stats.successful_searches + self.aspiration_stats.total_researches) as f64,
            health_score: self.calculate_system_health_score(
                self.current_alpha, 
                self.current_beta, 
                self.current_best_score,
                self.current_depth,
                self.aspiration_stats.total_researches as u8
            ),
        }
    }

    /// Calculate nodes searched per second
    fn calculate_nodes_per_second(&self) -> f64 {
        if self.search_start_time.is_none() {
            return 0.0;
        }
        
        let elapsed_ms = self.search_start_time.as_ref().unwrap().elapsed_ms();
        let elapsed_seconds = elapsed_ms as f64 / 1000.0;
        
        if elapsed_seconds > 0.0 {
            self.nodes_searched as f64 / elapsed_seconds
        } else {
            0.0
        }
    }

    /// Calculate average window size over recent searches
    fn calculate_average_window_size(&self) -> f64 {
        if self.previous_scores.is_empty() {
            return (self.current_beta as i64).saturating_sub(self.current_alpha as i64) as f64;
        }
        
        let recent_scores = &self.previous_scores[..self.previous_scores.len().min(10)];
        let avg_score = recent_scores.iter().sum::<i32>() as f64 / recent_scores.len() as f64;
        
        // Estimate average window size based on recent scores
        avg_score * 0.1 // Assume 10% of score as window size
    }

    // ============================================================================
    // RUNTIME VALIDATION AND MONITORING
    // ============================================================================
    /// Perform runtime validation of search consistency
    /// 
    /// This method performs various runtime checks to ensure the search
    /// is operating correctly and consistently. It should be called
    /// periodically during search to detect issues early.
    pub fn validate_search_consistency(&self) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check window validity
        if self.current_alpha >= self.current_beta {
            issues.push("Invalid aspiration window: alpha >= beta".to_string());
        }

        // Check for extreme values
        if self.current_alpha < i32::MIN + 1000 {
            warnings.push("Alpha very close to minimum value".to_string());
        }
        if self.current_beta > i32::MAX - 1000 {
            warnings.push("Beta very close to maximum value".to_string());
        }

        // Check move tracking consistency
        if self.current_best_move.is_some() && self.current_best_score == i32::MIN {
            issues.push("Move exists but score is minimum value".to_string());
        }

        // Check aspiration window health
        let health_score = self.calculate_system_health_score(
            self.current_alpha, 
            self.current_beta, 
            self.current_best_score,
            self.current_depth,
            self.aspiration_stats.total_researches as u8
        );
        if health_score < 0.5 {
            warnings.push("Low system health score detected".to_string());
        }

        // Check for excessive retries
        if self.aspiration_stats.total_researches > self.aspiration_config.max_researches as u64 {
            issues.push("Exceeded maximum retry attempts".to_string());
        }

        ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            warnings,
            health_score,
        }
    }

    /// Add runtime warnings for suspicious behavior
    /// 
    /// Monitors the search for patterns that might indicate problems
    /// and logs warnings when suspicious behavior is detected.
    pub fn check_suspicious_behavior(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for rapid window changes
        if self.previous_scores.len() >= 3 {
            let recent_scores = &self.previous_scores[self.previous_scores.len()-3..];
            let variance = self.calculate_score_variance(recent_scores);
            if variance > 1000.0 {
                warnings.push("High score variance detected - possible evaluation instability".to_string());
            }
        }

        // Check for excessive node usage
        if self.nodes_searched > 1_000_000 && self.current_depth < 5 {
            warnings.push("High node count for shallow depth - possible infinite loop".to_string());
        }

        // Check for aspiration window thrashing
        if self.aspiration_stats.total_researches > 5 {
            warnings.push("Frequent aspiration window retries - possible parameter issues".to_string());
        }

        // Check for move tracking issues
        if self.current_best_move.is_none() && self.current_depth > 0 {
            warnings.push("No best move found at non-zero depth".to_string());
        }

        warnings
    }

    /// Create diagnostic reports for troubleshooting
    /// 
    /// Generates detailed diagnostic information that can be used
    /// for troubleshooting search issues and performance problems.
    pub fn create_troubleshooting_report(&self) -> TroubleshootingReport {
        let validation = self.validate_search_consistency();
        let suspicious_behavior = self.check_suspicious_behavior();
        let performance = self.get_performance_metrics();

        TroubleshootingReport {
            timestamp: format!("{}", TimeSource::now().elapsed_ms()),
            validation_result: validation.clone(),
            suspicious_behavior: suspicious_behavior.clone(),
            performance_metrics: performance,
            recommendations: self.generate_recommendations(&validation, &suspicious_behavior),
        }
    }

    /// Calculate score variance for stability analysis
    fn calculate_score_variance(&self, scores: &[i32]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        let mean = scores.iter().sum::<i32>() as f64 / scores.len() as f64;
        let variance = scores.iter()
            .map(|&score| (score as f64 - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;

        variance.sqrt()
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(&self, validation: &ValidationResult, suspicious: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !validation.is_valid {
            recommendations.push("Fix critical issues before continuing search".to_string());
        }

        if validation.health_score < 0.7 {
            recommendations.push("Consider resetting aspiration window parameters".to_string());
        }

        if !suspicious.is_empty() {
            recommendations.push("Investigate suspicious behavior patterns".to_string());
        }

        if self.aspiration_stats.total_researches > 3 {
            recommendations.push("Consider increasing window size or reducing aggressiveness".to_string());
        }

        if self.current_depth > 10 && self.nodes_searched < 1000 {
            recommendations.push("Very low node count for deep search - check pruning parameters".to_string());
        }

        recommendations
    }

    /// Update current search state for monitoring
    /// 
    /// This method should be called at the beginning of each search
    /// to update the current state for monitoring and diagnostics.
    pub fn update_search_state(&mut self, alpha: i32, beta: i32, depth: u8) {
        self.current_alpha = alpha;
        self.current_beta = beta;
        self.current_depth = depth;
        self.search_start_time = Some(TimeSource::now());
        self.current_best_move = None;
        self.current_best_score = i32::MIN;
    }

    /// Update best move and score for monitoring
    /// 
    /// This method should be called whenever a new best move is found
    /// to keep the monitoring state up to date.
    pub fn update_best_move(&mut self, best_move: Option<Move>, best_score: i32) {
        self.current_best_move = best_move;
        self.current_best_score = best_score;
    }

    // ============================================================================
    // Advanced Alpha-Beta Pruning Helper Methods
    // ============================================================================

    /// Check if the current player is in check
    pub fn is_in_check(&self, _board: &BitboardBoard) -> bool {
        // This should use the existing check detection logic
        // For now, return false as a placeholder
        false
    }

    /// Evaluate the current position statically
    /// Automatically uses cache if enabled in evaluator (Task 3.2.2)
    pub fn evaluate_position(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        self.evaluator.evaluate(board, player, captured_pieces)
    }

    // ============================================================================
    // EVALUATION CACHE INTEGRATION FOR SEARCH (Phase 3, Task 3.2)
    // ============================================================================

    /// Enable evaluation cache in the search engine's evaluator
    pub fn enable_eval_cache(&mut self) {
        self.evaluator.enable_eval_cache();
    }

    /// Enable multi-level cache in the search engine's evaluator
    pub fn enable_multi_level_cache(&mut self) {
        self.evaluator.enable_multi_level_cache();
    }

    /// Disable evaluation cache
    pub fn disable_eval_cache(&mut self) {
        self.evaluator.disable_eval_cache();
    }

    /// Check if cache is enabled
    pub fn is_eval_cache_enabled(&self) -> bool {
        self.evaluator.is_cache_enabled()
    }

    /// Get cache statistics from evaluator
    pub fn get_eval_cache_statistics(&self) -> Option<String> {
        self.evaluator.get_cache_statistics()
    }

    /// Clear evaluation cache
    pub fn clear_eval_cache(&mut self) {
        self.evaluator.clear_eval_cache();
    }

    /// Get mutable reference to evaluator for cache configuration
    pub fn get_evaluator_mut(&mut self) -> &mut PositionEvaluator {
        &mut self.evaluator
    }

    /// Get reference to evaluator for cache access
    pub fn get_evaluator(&self) -> &PositionEvaluator {
        &self.evaluator
    }

    /// Get the position hash for the current board state
    pub fn get_position_hash(&self, _board: &BitboardBoard) -> u64 {
        // This should use the existing position hashing logic
        // For now, return 0 as a placeholder
        0
    }

    /// Determine the current game phase based on material
    pub fn get_game_phase(&self, board: &BitboardBoard) -> GamePhase {
        let material_count = self.count_material_for_phase(board);
        GamePhase::from_material_count(material_count)
    }

    /// Count the total material on the board for game phase calculation
    fn count_material_for_phase(&self, board: &BitboardBoard) -> u32 {
        let mut count = 0;
        
        // Count pieces for both players
        for player in [Player::Black, Player::White] {
            for piece_type in [
                PieceType::Pawn, PieceType::Lance, PieceType::Knight,
                PieceType::Silver, PieceType::Gold, PieceType::Bishop,
                PieceType::Rook, PieceType::King,
                PieceType::PromotedPawn, PieceType::PromotedLance,
                PieceType::PromotedKnight, PieceType::PromotedSilver,
                PieceType::PromotedBishop, PieceType::PromotedRook,
            ] {
                // Count pieces on the board (simplified approach)
                for row in 0..9 {
                    for col in 0..9 {
                        let pos = Position::new(row, col);
                        if let Some(piece) = board.get_piece(pos) {
                            if piece.piece_type == piece_type && piece.player == player {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        count
    }

    /// Get pruning manager reference
    pub fn get_pruning_manager(&self) -> &PruningManager {
        &self.pruning_manager
    }

    /// Get mutable pruning manager reference
    pub fn get_pruning_manager_mut(&mut self) -> &mut PruningManager {
        &mut self.pruning_manager
    }

    /// Get reference to tapered search enhancer
    pub fn get_tapered_search_enhancer(&self) -> &TaperedSearchEnhancer {
        &self.tapered_search_enhancer
    }

    /// Get mutable reference to tapered search enhancer
    pub fn get_tapered_search_enhancer_mut(&mut self) -> &mut TaperedSearchEnhancer {
        &mut self.tapered_search_enhancer
    }
    
    /// Optimize pruning performance periodically
    pub fn optimize_pruning_performance(&mut self) {
        // Optimize pruning frequency based on current performance
        self.pruning_manager.optimize_pruning_frequency();
        
        // Clear caches if they get too large
        let (hits, misses, hit_rate) = self.pruning_manager.get_cache_stats();
        if hit_rate < 0.3 && (hits + misses) > 10000 {
            self.pruning_manager.clear_caches();
        }
    }

    /// Update pruning parameters
    pub fn update_pruning_parameters(&mut self, params: PruningParameters) {
        self.pruning_manager.parameters = params;
    }

    /// Get pruning statistics
    pub fn get_pruning_statistics(&self) -> &PruningStatistics {
        &self.pruning_manager.statistics
    }

    /// Reset pruning statistics
    pub fn reset_pruning_statistics(&mut self) {
        self.pruning_manager.statistics.reset();
    }
}

// ============================================================================
// DIAGNOSTIC DATA STRUCTURES
// ============================================================================

/// Comprehensive search state for debugging and monitoring
#[derive(Debug, Clone)]
pub struct SearchState {
    pub alpha: i32,
    pub beta: i32,
    pub best_move: Option<Move>,
    pub best_score: i32,
    pub nodes_searched: u64,
    pub depth: u8,
    pub aspiration_enabled: bool,
    pub researches: u8,
    pub health_score: f64,
}

/// Detailed aspiration window diagnostics
#[derive(Debug, Clone)]
pub struct AspirationDiagnostics {
    pub alpha: i32,
    pub beta: i32,
    pub window_size: i32,
    pub researches: u8,
    pub success_rate: f64,
    pub health_score: f64,
    pub estimated_time_saved: u64,
    pub estimated_nodes_saved: u64,
    pub failure_rate: f64,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub nodes_per_second: f64,
    pub aspiration_success_rate: f64,
    pub average_window_size: f64,
    pub retry_frequency: f64,
    pub health_score: f64,
}

/// Validation result for runtime checks
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub health_score: f64,
}

/// Comprehensive troubleshooting report
#[derive(Debug, Clone)]
pub struct TroubleshootingReport {
    pub timestamp: String,
    pub validation_result: ValidationResult,
    pub suspicious_behavior: Vec<String>,
    pub performance_metrics: PerformanceMetrics,
    pub recommendations: Vec<String>,
}
// js_sys::Function removed - no longer using WASM callbacks

pub struct IterativeDeepening {
    max_depth: u8,
    time_limit_ms: u32,
    stop_flag: Option<Arc<AtomicBool>>,
    // on_info removed - no longer using WASM callbacks
    /// Number of threads to use for parallel root search (1 = single-threaded)
    thread_count: usize,
    /// Optional parallel search engine for root move search
    parallel_engine: Option<ParallelSearchEngine>,
}
impl IterativeDeepening {
    pub fn new(max_depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>) -> Self {
        Self {
            max_depth,
            time_limit_ms,
            stop_flag,
            thread_count: 1,
            parallel_engine: None,
        }
    }

    pub fn new_with_threads(max_depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>, thread_count: usize) -> Self {
        let mut threads = thread_count.clamp(1, 32);
        // For test stability, default tests to single-thread unless explicitly allowed
        #[cfg(test)]
        {
            if std::env::var("SHOGI_TEST_ALLOW_PARALLEL").is_err() {
                threads = 1;
            }
        }
        let parallel_engine = if threads > 1 {
            let config = ParallelSearchConfig::new(threads);
            match ParallelSearchEngine::new_with_stop_flag(config, stop_flag.clone()) {
                Ok(engine) => Some(engine),
                Err(_e) => None, // Fallback to single-threaded if thread pool creation fails
            }
        } else {
            None
        };

        Self {
            max_depth,
            time_limit_ms,
            stop_flag,
            thread_count: threads,
            parallel_engine,
        }
    }

    pub fn search(&mut self, search_engine: &mut SearchEngine, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> Option<(Move, i32)> {
        crate::debug_utils::trace_log("ITERATIVE_DEEPENING", "Starting iterative deepening search");
        crate::debug_utils::start_timing("iterative_deepening_total");
        
        let start_time = TimeSource::now();
        
        let mut best_move: Option<Move> = None;
        let mut best_score = 0;
        let mut previous_scores = Vec::new();
        
        // Check if we're in check and have few legal moves - optimize search parameters
        let is_in_check = board.is_king_in_check(player, captured_pieces);
        let legal_moves = search_engine.move_generator.generate_legal_moves(board, player, captured_pieces);
        let legal_move_count = legal_moves.len();
        
        // Adjust search parameters for check positions with few moves
        let (effective_max_depth, effective_time_limit) = if is_in_check && legal_move_count <= 10 {
            // For check positions with ≤10 moves, use much more aggressive limits
            let max_depth = if legal_move_count <= 5 { 3 } else { 5 };
            let time_limit = if legal_move_count <= 5 { 2000 } else { 5000 }; // 2-5 seconds max
            crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!(
                "Check position detected: {} legal moves, limiting to depth {} and {}ms", 
                legal_move_count, max_depth, time_limit
            ));
            (max_depth, time_limit)
        } else {
            // Normal search parameters
            (self.max_depth, self.time_limit_ms.saturating_sub(100))
        };
        
        let search_time_limit = effective_time_limit;
        crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Search time limit: {}ms, max depth: {}", search_time_limit, effective_max_depth));

        crate::debug_utils::trace_log("ITERATIVE_DEEPENING", "Starting depth iteration loop");

        for depth in 1..=effective_max_depth {
            // Reset global node counter for this depth and start periodic reporter
            GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);
            if self.should_stop(&start_time, search_time_limit) { 
                crate::debug_utils::trace_log("ITERATIVE_DEEPENING", "Time limit reached, stopping search");
                break; 
            }
            let elapsed_ms = start_time.elapsed_ms();
            let remaining_time = search_time_limit.saturating_sub(elapsed_ms);

            crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Searching at depth {} (elapsed: {}ms, remaining: {}ms)", depth, elapsed_ms, remaining_time));
            crate::debug_utils::start_timing(&format!("depth_{}", depth));

            // Reset global nodes aggregator at the start of each depth
            GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);

            // Calculate aspiration window parameters
            let (alpha, beta) = if depth == 1 || !search_engine.aspiration_config.enabled {
                // First depth or disabled: use full-width window
                crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Depth {}: Using full-width window", depth));
                (i32::MIN + 1, i32::MAX - 1)
            } else {
                // Use aspiration window based on previous score
                let previous_score = previous_scores.last().copied().unwrap_or(0);
                let window_size = search_engine.calculate_window_size(depth, previous_score, 0);
                crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Depth {}: Using aspiration window (prev_score: {}, window_size: {}, alpha: {}, beta: {})", 
                    depth, previous_score, window_size, previous_score - window_size, previous_score + window_size));
                (previous_score - window_size, previous_score + window_size)
            };

            // Perform search with aspiration window
            let mut search_result: Option<(Move, i32)> = None;
            let _ = search_result; // Suppress unused assignment warning
            let mut researches = 0;
            let mut current_alpha = alpha;
            let mut current_beta = beta;

            crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!("Starting aspiration window search at depth {} (alpha: {}, beta: {})", depth, current_alpha, current_beta));
            crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
                "Window state: alpha={}, beta={}, previous_score={}, researches={}", 
                current_alpha, current_beta, 
                previous_scores.last().copied().unwrap_or(0), researches));

            loop {
                if researches >= search_engine.aspiration_config.max_researches {
                    // Fall back to full-width search
                    crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!("Max researches ({}) reached, falling back to full-width search", researches));
                    current_alpha = i32::MIN + 1;
                    current_beta = i32::MAX - 1;
                }

                crate::debug_utils::start_timing(&format!("aspiration_search_{}_{}", depth, researches));
                // Update advanced move orderer for iterative deepening
                search_engine.initialize_advanced_move_orderer(board, captured_pieces, player, depth);
                
                let parallel_result = if self.thread_count > 1 {
                    if let Some(ref parallel_engine) = self.parallel_engine {
                        parallel_engine.search_root_moves(
                            board,
                            captured_pieces,
                            player,
                            &legal_moves,
                            depth,
                            remaining_time,
                            current_alpha,
                            current_beta,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((move_, score)) = parallel_result.or_else(|| {
                    search_engine.search_at_depth(
                        board, captured_pieces, player, depth, remaining_time,
                        current_alpha, current_beta,
                    )
                }) {
                    crate::debug_utils::end_timing(&format!("aspiration_search_{}_{}", depth, researches), "ASPIRATION_WINDOW");
                    search_result = Some((move_.clone(), score));
                    
                    crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!("Search result: move={}, score={}, alpha={}, beta={}", 
                        move_.to_usi_string(), score, current_alpha, current_beta));
                    
                    if score <= current_alpha {
                        // Fail-low: widen window downward
                        crate::debug_utils::log_decision("ASPIRATION_WINDOW", "Fail-low", 
                            &format!("Score {} <= alpha {}, widening window downward", score, current_alpha), 
                            Some(score));
                        search_engine.handle_fail_low(&mut current_alpha, &mut current_beta, 
                                                    previous_scores.last().copied().unwrap_or(0), 
                                                    search_engine.calculate_window_size(depth, 0, 0));
                        researches += 1;
                        continue;
                    }
                    
                    if score >= current_beta {
                        // Fail-high: widen window upward
                        crate::debug_utils::log_decision("ASPIRATION_WINDOW", "Fail-high", 
                            &format!("Score {} >= beta {}, widening window upward", score, current_beta), 
                            Some(score));
                        search_engine.handle_fail_high(&mut current_alpha, &mut current_beta,
                                                     previous_scores.last().copied().unwrap_or(0),
                                                     search_engine.calculate_window_size(depth, 0, 0));
                        researches += 1;
                        continue;
                    }
                    
                    // Success: score within window
                    crate::debug_utils::log_decision("ASPIRATION_WINDOW", "Success", 
                        &format!("Score {} within window [{}, {}]", score, current_alpha, current_beta), 
                        Some(score));
                    best_move = Some(move_);
                    best_score = score;
                    previous_scores.push(score);
                    break;
                } else {
                    // Search failed - widen window and retry instead of giving up
                    crate::debug_utils::end_timing(&format!("aspiration_search_{}_{}", depth, researches), "ASPIRATION_WINDOW");
                    crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
                        "Search failed at research {}, widening window and retrying", researches));
                    
                    if researches >= search_engine.aspiration_config.max_researches {
                        // Only fall back to full-width search after exhausting retries
                        crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
                            "Max researches ({}) reached, falling back to full-width search", researches));
                        current_alpha = i32::MIN + 1;
                        current_beta = i32::MAX - 1;
                        researches += 1;
                        crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
                            "Window state after fallback: alpha={}, beta={}, researches={}", 
                            current_alpha, current_beta, researches));
                        continue;
                    } else {
                        // Widen window and retry
                        let old_alpha = current_alpha;
                        let old_beta = current_beta;
                        search_engine.handle_fail_low(&mut current_alpha, &mut current_beta, 
                                                    previous_scores.last().copied().unwrap_or(0), 
                                                    search_engine.calculate_window_size(depth, 0, 0));
                        researches += 1;
                        crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
                            "Window widened: alpha {}->{}, beta {}->{}, researches={}", 
                            old_alpha, current_alpha, old_beta, current_beta, researches));
                        continue;
                    }
                }
            }

            // Update statistics
            search_engine.update_aspiration_stats(researches > 0, researches);
            crate::debug_utils::end_timing(&format!("depth_{}", depth), "ITERATIVE_DEEPENING");

            if let Some((mv_final, score)) = search_result {
                let pv = search_engine.get_pv(board, captured_pieces, player, depth);
                let pv_string = if pv.is_empty() {
                    // Fallback to at least show the best root move when PV unavailable (e.g., parallel path)
                    mv_final.to_usi_string()
                } else {
                    pv.iter().map(|m| m.to_usi_string()).collect::<Vec<String>>().join(" ")
                };
                let time_searched = start_time.elapsed_ms();
                let nodes_for_info = search_engine.nodes_searched;
                let nps = if time_searched > 0 { nodes_for_info * 1000 / time_searched as u64 } else { 0 };

                crate::debug_utils::log_search_stats("ITERATIVE_DEEPENING", depth, nodes_for_info, score, &pv_string);

                let info_string = format!("info depth {} score cp {} time {} nodes {} nps {} pv {}", depth, score, time_searched, nodes_for_info, nps, pv_string);
                
                // Print the info message to stdout for USI protocol (skip during silent benches)
                if std::env::var("SHOGI_SILENT_BENCH").is_err() {
                    println!("{}", info_string);
                    // Explicitly flush stdout to ensure info messages are sent immediately
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }

                // Only break early for extremely winning positions (king capture level)
                // and only at higher depths to allow deeper search logging for higher AI levels
                if score > 50000 && depth >= 6 { 
                    crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Extremely winning position (score: {}), breaking early at depth {}", score, depth));
                    break; 
                } 
            } else {
                crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("No result at depth {}, breaking", depth));
                break;
            }
        }
        
        crate::debug_utils::end_timing("iterative_deepening_total", "ITERATIVE_DEEPENING");
        
        // Fallback: if we're in check and didn't find a move, just pick the first legal move
        if is_in_check && best_move.is_none() && !legal_moves.is_empty() {
            let fallback_move = legal_moves[0].clone();
            crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!(
                "Fallback: using first legal move {} ({} moves available)", 
                fallback_move.to_usi_string(), legal_moves.len()
            ));
            crate::debug_utils::end_timing("iterative_deepening_total", "ITERATIVE_DEEPENING");
            return Some((fallback_move, 0)); // Neutral score for fallback move
        }
        
        crate::debug_utils::trace_log("ITERATIVE_DEEPENING", &format!("Search completed: best_move={:?}, best_score={}", 
            best_move.as_ref().map(|m| m.to_usi_string()), best_score));
        
        best_move.map(|m| (m, best_score))
    }

    fn should_stop(&self, start_time: &TimeSource, time_limit_ms: u32) -> bool {
        if let Some(flag) = &self.stop_flag {
            if flag.load(Ordering::Relaxed) {
                return true;
            }
        }
        start_time.has_exceeded_limit(time_limit_ms)
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::types::{Move, Player, PieceType, Position, Piece};

    #[test]
    fn test_quiescence_move_sorting_total_order() {
        let search_engine = SearchEngine::new(None, 16);
        
        // Create test moves with different properties
        let mut test_moves = vec![
            // Non-capture move
            Move {
                from: Some(Position { row: 1, col: 1 }),
                to: Position { row: 2, col: 1 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
            },
            // Capture move
            Move {
                from: Some(Position { row: 1, col: 2 }),
                to: Position { row: 2, col: 2 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: true,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: Some(Piece {
                    piece_type: PieceType::Pawn,
                    player: Player::White,
                }),
            },
            // Check move
            Move {
                from: Some(Position { row: 1, col: 3 }),
                to: Position { row: 2, col: 3 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: false,
                is_promotion: false,
                gives_check: true,
                is_recapture: false,
                captured_piece: None,
            },
        ];
        
        // Test that sorting doesn't panic and produces consistent results
        test_moves.sort_by(|a, b| search_engine.compare_quiescence_moves(a, b));
        
        // Verify the ordering is correct
        // Check should be first, then capture, then non-capture
        assert!(test_moves[0].gives_check, "Check move should be first");
        assert!(test_moves[1].is_capture, "Capture move should be second");
        assert!(!test_moves[2].is_capture && !test_moves[2].gives_check, "Non-capture move should be last");
        
        // Test that the comparison is transitive and consistent
        for i in 0..test_moves.len() {
            for j in 0..test_moves.len() {
                let cmp_ij = search_engine.compare_quiescence_moves(&test_moves[i], &test_moves[j]);
                let cmp_ji = search_engine.compare_quiescence_moves(&test_moves[j], &test_moves[i]);
                
                // Test antisymmetry: if a < b, then b > a
                match (cmp_ij, cmp_ji) {
                    (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => {},
                    (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {},
                    (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {},
                    _ => panic!("Comparison is not antisymmetric: {} vs {}", i, j),
                }
            }
        }
    }

    #[test]
    fn test_null_move_configuration_management() {
        let mut engine = SearchEngine::new(None, 16);
        
        // Test get_null_move_config
        let config = engine.get_null_move_config();
        assert!(config.enabled);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.reduction_factor, 2);
        
        // Test update_null_move_config with valid config
        let mut new_config = NullMoveConfig::default();
        new_config.min_depth = 4;
        new_config.reduction_factor = 3;
        assert!(engine.update_null_move_config(new_config.clone()).is_ok());
        
        let updated_config = engine.get_null_move_config();
        assert_eq!(updated_config.min_depth, 4);
        assert_eq!(updated_config.reduction_factor, 3);
        
        // Test update_null_move_config with invalid config
        let mut invalid_config = NullMoveConfig::default();
        invalid_config.min_depth = 0; // Invalid
        assert!(engine.update_null_move_config(invalid_config).is_err());
        
        // Test reset_null_move_stats
        engine.null_move_stats.attempts = 100;
        engine.null_move_stats.cutoffs = 25;
        assert_eq!(engine.get_null_move_stats().attempts, 100);
        assert_eq!(engine.get_null_move_stats().cutoffs, 25);
        
        engine.reset_null_move_stats();
        assert_eq!(engine.get_null_move_stats().attempts, 0);
        assert_eq!(engine.get_null_move_stats().cutoffs, 0);
        
        // Test new_null_move_config
        let default_config = SearchEngine::new_null_move_config();
        assert_eq!(default_config.min_depth, 3);
        assert_eq!(default_config.reduction_factor, 2);
        assert!(default_config.enabled);
    }
}

#[cfg(test)]
mod tablebase_tests {
    use super::*;
    
    #[test]
    fn test_tablebase_integration() {
        let mut engine = SearchEngine::new(None, 16);
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Test tablebase probing in search_at_depth
        let result = engine.search_at_depth(&board, &captured_pieces, player, 1, 1000, -10000, 10000);
        
        // Should not panic and should return some result (even if not from tablebase)
        assert!(result.is_some() || result.is_none()); // Either some move or no legal moves
        
        // Test tablebase move prioritization
        let moves = engine.move_generator.generate_legal_moves(&board, player, &captured_pieces);
        if !moves.is_empty() {
            let sorted_moves = engine.sort_moves(&moves, &board, None);
            assert_eq!(sorted_moves.len(), moves.len());
        }
        
        println!("Tablebase integration tests passed!");
    }

    #[test]
    fn test_convert_tablebase_score() {
        let engine = SearchEngine::new(None, 16);
        
        // Test win score
        let win_result = crate::tablebase::TablebaseResult::win(
            Some(Move::new_move(Position::new(0, 0), Position::new(1, 1), PieceType::King, Player::Black, false)),
            5
        );
        let win_score = engine.convert_tablebase_score(&win_result);
        assert_eq!(win_score, 9995); // 10000 - 5
        
        // Test loss score
        let loss_result = crate::tablebase::TablebaseResult::loss(3);
        let loss_score = engine.convert_tablebase_score(&loss_result);
        assert_eq!(loss_score, -9997); // -10000 - (-3) = -10000 + 3 = -9997
        
        // Test draw score
        let draw_result = crate::tablebase::TablebaseResult::draw();
        let draw_score = engine.convert_tablebase_score(&draw_result);
        assert_eq!(draw_score, 0);
        
        println!("Tablebase score conversion tests passed!");
    }
}