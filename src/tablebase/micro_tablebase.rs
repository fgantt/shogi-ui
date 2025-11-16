//! Core tablebase implementation
//!
//! This module contains the main MicroTablebase struct that coordinates
//! all endgame solvers and provides the primary interface for tablebase
//! functionality.

use super::endgame_solvers::{KingGoldVsKingSolver, KingRookVsKingSolver, KingSilverVsKingSolver};
use super::{
    EndgameSolver, PositionAnalyzer, PositionCache, TablebaseConfig, TablebaseProfiler,
    TablebaseResult, TablebaseStats,
};
use crate::utils::time::TimeSource;
use crate::types::core::{Player, Position};
use crate::BitboardBoard;
use crate::CapturedPieces;

/// Main tablebase implementation
///
/// This struct coordinates all endgame solvers and provides caching
/// and statistics for tablebase operations.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use shogi_engine::tablebase::MicroTablebase;
/// use shogi_engine::{BitboardBoard, Player, CapturedPieces};
///
/// let mut tablebase = MicroTablebase::new();
/// let board = BitboardBoard::new();
/// let captured_pieces = CapturedPieces::new();
///
/// // Probe for best move
/// if let Some(result) = tablebase.probe(&board, Player::Black, &captured_pieces) {
///     println!("Best move: {:?}", result.best_move);
///     println!("Distance to mate: {}", result.distance_to_mate);
/// }
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use shogi_engine::tablebase::{MicroTablebase, TablebaseConfig};
///
/// let config = TablebaseConfig::performance_optimized();
/// let tablebase = MicroTablebase::with_config(config);
/// ```
///
/// ## Performance Monitoring
///
/// ```rust
/// // Enable profiling
/// tablebase.set_profiling_enabled(true);
///
/// // Perform operations...
///
/// // Get performance summary
/// let profiler = tablebase.get_profiler();
/// let summary = profiler.get_summary();
/// println!("Performance: {}", summary);
/// ```
///
/// ## Performance Targets
///
/// - Cache probe latency (warm cache): **< 1ms**
/// - Solver computation latency (supported K+G/K+S/K+R endgames): **< 10ms**
/// - Move ordering cache should prevent repeated tablebase probes during a search iteration
///
/// ## Memory Management
///
/// ```rust
/// // Check memory usage
/// let memory_usage = tablebase.get_current_memory_usage();
/// println!("Current memory: {} bytes", memory_usage);
///
/// // Get memory summary
/// let memory_summary = tablebase.get_memory_summary();
/// println!("Memory: {}", memory_summary);
/// ```
pub struct MicroTablebase {
    /// List of endgame solvers, sorted by priority
    solvers: Vec<Box<dyn EndgameSolver>>,
    /// Position cache for storing results
    position_cache: PositionCache,
    /// Configuration for the tablebase system
    config: TablebaseConfig,
    /// Statistics for monitoring performance
    stats: TablebaseStats,
    /// Last memory check time
    last_memory_check: TimeSource,
    /// Position analyzer for adaptive solver selection
    position_analyzer: PositionAnalyzer,
    /// Performance profiler for detailed timing analysis
    #[allow(dead_code)]
    profiler: TablebaseProfiler,
}

impl MicroTablebase {
    /// Create a new tablebase with default configuration
    pub fn new() -> Self {
        let config = TablebaseConfig::default();
        Self::with_config(config)
    }

    /// Create a new tablebase with specified configuration
    pub fn with_config(config: TablebaseConfig) -> Self {
        let mut solvers: Vec<Box<dyn EndgameSolver>> = Vec::new();

        // Add actual solvers
        solvers.push(Box::new(KingGoldVsKingSolver::new()));
        solvers.push(Box::new(KingSilverVsKingSolver::new()));
        solvers.push(Box::new(KingRookVsKingSolver::new()));

        // Sort by priority (highest first)
        solvers.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        // Create cache with configuration
        let cache_size = config.cache_size;

        let cache_config = super::position_cache::CacheConfig {
            max_size: cache_size,
            eviction_strategy: config.performance.eviction_strategy,
            enable_adaptive_eviction: config.performance.enable_adaptive_caching,
        };

        Self {
            solvers,
            position_cache: PositionCache::with_config(cache_config),
            config,
            stats: TablebaseStats::new(),
            last_memory_check: TimeSource::now(),
            position_analyzer: PositionAnalyzer::new(),
            profiler: TablebaseProfiler::new(),
        }
    }

    /// Check memory usage and perform eviction if necessary
    pub fn check_memory_usage(&mut self) {
        if !self.config.memory.enable_monitoring {
            return;
        }

        // Check if enough time has passed since last check
        let now = TimeSource::now();
        let time_since_last_check = now
            .elapsed_ms()
            .saturating_sub(self.last_memory_check.elapsed_ms());
        if u64::from(time_since_last_check) < self.config.memory.check_interval_ms {
            return;
        }

        self.last_memory_check = now;

        // Estimate current memory usage
        let current_memory = self.estimate_memory_usage();
        self.stats.update_memory_usage(current_memory);

        // Check for memory warnings and critical alerts
        if self.stats.is_memory_warning(
            self.config.memory.max_memory_bytes,
            self.config.memory.warning_threshold,
        ) {
            self.stats.record_memory_warning();
            if self.config.memory.enable_logging {
                println!(
                    "Tablebase memory warning: {} bytes used ({:.1}% of limit)",
                    current_memory,
                    self.stats
                        .memory_usage_percentage(self.config.memory.max_memory_bytes)
                );
            }
        }

        if self.stats.is_memory_critical(
            self.config.memory.max_memory_bytes,
            self.config.memory.critical_threshold,
        ) {
            self.stats.record_memory_critical_alert();
            if self.config.memory.enable_logging {
                println!(
                    "Tablebase memory critical: {} bytes used ({:.1}% of limit)",
                    current_memory,
                    self.stats
                        .memory_usage_percentage(self.config.memory.max_memory_bytes)
                );
            }

            // Perform automatic eviction if enabled
            if self.config.memory.enable_auto_eviction {
                self.perform_emergency_eviction();
            }
        }
    }

    /// Estimate current memory usage in bytes
    fn estimate_memory_usage(&self) -> usize {
        // Estimate memory usage based on cache size and configuration
        let cache_memory = self.position_cache.estimate_memory_usage();
        let config_memory = std::mem::size_of::<TablebaseConfig>();
        let stats_memory = std::mem::size_of::<TablebaseStats>();
        let solvers_memory = self.solvers.len() * std::mem::size_of::<Box<dyn EndgameSolver>>();

        cache_memory + config_memory + stats_memory + solvers_memory
    }

    /// Perform emergency eviction to reduce memory usage
    fn perform_emergency_eviction(&mut self) {
        // Clear half of the cache
        self.position_cache.clear_half();
        self.stats.record_auto_eviction();

        if self.config.memory.enable_logging {
            println!("Tablebase emergency eviction performed");
        }
    }

    /// Probe the tablebase for a position
    ///
    /// This method checks the cache first, then tries each solver
    /// in priority order until one can solve the position.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `player` - The player to move
    /// * `captured_pieces` - The captured pieces for both players
    ///
    /// # Returns
    /// `Some(TablebaseResult)` if the position can be solved, `None` otherwise
    pub fn probe(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TablebaseResult> {
        if !self.config.enabled {
            return None;
        }

        // Check memory usage before probing
        self.check_memory_usage();

        // Start timing the probe
        let start_time = TimeSource::now();

        // Check cache first
        let cached_result = self.position_cache.get(board, player, captured_pieces);

        if let Some(result) = cached_result {
            let probe_time = start_time.elapsed_ms() as u64;
            self.stats.record_probe(true, false, None, probe_time);
            return Some(result);
        }

        // Analyze position complexity for adaptive solver selection (skip trivial cases)
        let mut position_analysis = None;
        if !self.is_simple_endgame(board, captured_pieces) {
            let analysis_start = TimeSource::now();
            let analysis = self
                .position_analyzer
                .analyze_position(board, player, captured_pieces);
            self.stats
                .record_position_analysis_time(analysis_start.elapsed_ms() as u64);
            position_analysis = Some(analysis);
        }

        // Try each solver in priority order, but skip solvers that can't handle the complexity
        let solver_timer = TimeSource::now();
        let mut solver_result = None;
        for solver in &self.solvers {
            if !solver.is_enabled() {
                continue;
            }

            if let Some(ref analysis) = position_analysis {
                // Skip solver if it can't handle the position complexity
                if !analysis
                    .complexity
                    .is_suitable_for_priority(solver.priority())
                {
                    continue;
                }
            }

            if solver.can_solve(board, player, captured_pieces) {
                if let Some(result) = solver.solve(board, player, captured_pieces) {
                    // Check if result meets confidence threshold
                    if result.confidence >= self.config.confidence_threshold {
                        solver_result = Some((result, solver.name()));
                        break;
                    }
                }
            }
        }
        self.stats
            .record_solver_selection_time(solver_timer.elapsed_ms() as u64);

        if let Some((result, solver_name)) = solver_result {
            let probe_time = start_time.elapsed_ms() as u64;
            self.stats
                .record_probe(false, true, Some(solver_name), probe_time);
            return Some(result);
        }

        let probe_time = start_time.elapsed_ms() as u64;
        self.stats.record_probe(false, false, None, probe_time);
        None
    }

    /// Probe the tablebase and update statistics
    ///
    /// This method is similar to `probe()` but also updates the
    /// internal statistics for monitoring purposes.
    pub fn probe_with_stats(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Option<TablebaseResult> {
        let start_time = TimeSource::now();

        if !self.config.enabled {
            return None;
        }

        // Check cache first
        if let Some(cached_result) = self.position_cache.get(board, player, captured_pieces) {
            let probe_time = start_time.elapsed_ms() as u64;
            self.stats.record_probe(true, false, None, probe_time);
            return Some(cached_result);
        }

        // Try each solver in priority order
        for solver in &self.solvers {
            if !solver.is_enabled() {
                continue;
            }

            if solver.can_solve(board, player, captured_pieces) {
                if let Some(result) = solver.solve(board, player, captured_pieces) {
                    // Check if result meets confidence threshold
                    if result.confidence >= self.config.confidence_threshold {
                        let probe_time = start_time.elapsed_ms() as u64;
                        self.stats
                            .record_probe(false, true, Some(solver.name()), probe_time);

                        // Cache the result
                        self.position_cache
                            .put(board, player, captured_pieces, result.clone());
                        return Some(result);
                    }
                }
            }
        }

        let probe_time = start_time.elapsed_ms() as u64;
        self.stats.record_probe(false, false, None, probe_time);
        None
    }

    /// Get the current statistics
    pub fn get_stats(&self) -> &TablebaseStats {
        &self.stats
    }

    /// Get a mutable reference to the statistics
    pub fn get_stats_mut(&mut self) -> &mut TablebaseStats {
        &mut self.stats
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &TablebaseConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: TablebaseConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        self.position_cache = PositionCache::with_size(self.config.cache_size);
        Ok(())
    }

    /// Clear the position cache
    pub fn clear_cache(&mut self) {
        self.position_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        self.position_cache.stats()
    }

    /// Get the number of solvers
    pub fn solver_count(&self) -> usize {
        self.solvers.len()
    }

    /// Enable the tablebase
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable the tablebase
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Check if the tablebase is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get solver information
    pub fn get_solver_info(&self) -> Vec<String> {
        self.solvers
            .iter()
            .map(|solver| solver.get_config_info())
            .collect()
    }

    /// Reset tablebase statistics
    pub fn reset_stats(&mut self) {
        self.stats = TablebaseStats::new();
    }

    /// Enable or disable the tablebase
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    fn is_simple_endgame(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> bool {
        if !captured_pieces.black.is_empty() || !captured_pieces.white.is_empty() {
            return false;
        }
        self.count_pieces(board) <= 4
    }

    fn count_pieces(&self, board: &BitboardBoard) -> u8 {
        let mut count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if board.get_piece(Position::new(row, col)).is_some() {
                    count += 1;
                }
            }
        }
        count
    }
}

impl Default for MicroTablebase {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MicroTablebase {
    fn clone(&self) -> Self {
        // Create a new tablebase with the same configuration
        let mut new_tablebase = MicroTablebase::with_config(self.config.clone());

        // Copy the statistics
        new_tablebase.stats = self.stats.clone();

        // Note: We can't clone the solvers because they're trait objects,
        // but that's okay since they're stateless and will be recreated
        // with the same configuration

        new_tablebase
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, PieceType, Player, Position};
    use crate::BitboardBoard;
    use crate::CapturedPieces;

    // Mock solver for testing
    struct MockSolver {
        name: &'static str,
        priority: u8,
        enabled: bool,
        can_solve_result: bool,
        solve_result: Option<TablebaseResult>,
    }

    impl MockSolver {
        fn new(name: &'static str, priority: u8) -> Self {
            Self {
                name,
                priority,
                enabled: true,
                can_solve_result: false,
                solve_result: None,
            }
        }

        fn with_can_solve(mut self, can_solve: bool) -> Self {
            self.can_solve_result = can_solve;
            self
        }

        fn with_solve_result(mut self, result: Option<TablebaseResult>) -> Self {
            self.solve_result = result;
            self
        }

        fn with_enabled(mut self, enabled: bool) -> Self {
            self.enabled = enabled;
            self
        }
    }

    impl EndgameSolver for MockSolver {
        fn can_solve(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> bool {
            self.can_solve_result
        }

        fn solve(
            &self,
            _board: &BitboardBoard,
            _player: Player,
            _captured_pieces: &CapturedPieces,
        ) -> Option<TablebaseResult> {
            self.solve_result.clone()
        }

        fn priority(&self) -> u8 {
            self.priority
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[test]
    fn test_micro_tablebase_creation() {
        let tablebase = MicroTablebase::new();
        assert!(tablebase.is_enabled());
        assert_eq!(tablebase.solver_count(), 3); // Now we have all three solvers
        assert_eq!(tablebase.get_cache_stats(), (0, 0, 0.0));
    }

    #[test]
    fn test_micro_tablebase_with_config() {
        let config = TablebaseConfig::memory_optimized();
        let tablebase = MicroTablebase::with_config(config);
        assert!(tablebase.is_enabled());
        assert_eq!(tablebase.get_config().cache_size, 1000);
    }

    #[test]
    fn test_micro_tablebase_probe_disabled() {
        let mut config = TablebaseConfig::default();
        config.enabled = false;
        let mut tablebase = MicroTablebase::with_config(config);

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        assert!(tablebase.probe(&board, player, &captured_pieces).is_none());
    }

    #[test]
    fn test_micro_tablebase_probe_with_stats() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Initially no stats
        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 0);

        // Probe should return None (no solvers)
        let result = tablebase.probe_with_stats(&board, player, &captured_pieces);
        assert!(result.is_none());

        // Should have recorded a probe
        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_micro_tablebase_config_management() {
        let mut tablebase = MicroTablebase::new();
        let _original_config = tablebase.get_config().clone();

        // Test setting new config
        let new_config = TablebaseConfig::performance_optimized();
        assert!(tablebase.set_config(new_config).is_ok());

        let updated_config = tablebase.get_config();
        assert_eq!(updated_config.cache_size, 50000);

        // Test invalid config
        let mut invalid_config = TablebaseConfig::default();
        invalid_config.cache_size = 0;
        assert!(tablebase.set_config(invalid_config).is_err());

        // Config should remain unchanged
        assert_eq!(tablebase.get_config().cache_size, 50000);
    }

    #[test]
    fn test_micro_tablebase_cache_management() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Clear cache
        tablebase.clear_cache();
        assert_eq!(tablebase.get_cache_stats(), (0, 0, 0.0));

        // Test cache operations
        let move_ = Move::new_move(
            Position::new(0, 0),
            Position::new(1, 1),
            PieceType::King,
            Player::Black,
            false,
        );

        let result = TablebaseResult::win(Some(move_), 5);
        tablebase
            .position_cache
            .put(&board, player, &captured_pieces, result);

        // Should be able to get from cache
        let cached_result = tablebase
            .position_cache
            .get(&board, player, &captured_pieces);
        assert!(cached_result.is_some());
    }

    #[test]
    fn test_micro_tablebase_enable_disable() {
        let mut tablebase = MicroTablebase::new();
        assert!(tablebase.is_enabled());

        tablebase.set_enabled(false);
        assert!(!tablebase.is_enabled());

        tablebase.set_enabled(true);
        assert!(tablebase.is_enabled());
    }

    #[test]
    fn test_micro_tablebase_stats_reset() {
        let mut tablebase = MicroTablebase::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        let player = Player::Black;

        // Generate some stats
        tablebase.probe_with_stats(&board, player, &captured_pieces);
        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 1);

        // Reset stats
        tablebase.reset_stats();
        let stats = tablebase.get_stats();
        assert_eq!(stats.total_probes, 0);
    }
}
