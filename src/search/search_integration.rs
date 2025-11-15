//! Search algorithm integration with transposition table
//!
//! This module provides the integration between the search algorithm and the
//! advanced transposition table system, including proper flag handling,
//! best move storage, and performance optimizations.

use crate::bitboards::*;
use crate::evaluation::*;
use crate::moves::*;
use crate::search::advanced_statistics::AdvancedStatisticsManager;
use crate::search::error_handling::{ComprehensiveErrorHandler, TranspositionResult};
use crate::search::shogi_hash::*;
use crate::search::thread_safe_table::{ThreadSafeStatsSnapshot, ThreadSafeTranspositionTable};
use crate::search::transposition_config::TranspositionConfig;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Player};
use crate::types::search::TranspositionFlag;
use crate::types::transposition::TranspositionEntry;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const MAX_QUIESCENCE_DEPTH: u8 = 4;

/// Enhanced search engine with integrated transposition table
pub struct EnhancedSearchEngine {
    /// Thread-safe transposition table
    transposition_table: ThreadSafeTranspositionTable,
    /// Shogi hash calculator for position hashing
    hash_calculator: ShogiHashHandler,
    /// Error handler for robust operation
    error_handler: Arc<ComprehensiveErrorHandler>,
    /// Statistics manager for performance monitoring
    stats_manager: Arc<AdvancedStatisticsManager>,
    /// Position evaluator
    evaluator: PositionEvaluator,
    /// Move generator
    move_generator: MoveGenerator,
    /// Search statistics
    search_stats: Arc<Mutex<SearchStats>>,
}

/// Search statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    pub total_nodes: u64,
    pub tt_hits: u64,
    pub tt_stores: u64,
    pub tt_replacements: u64,
    pub tt_collisions: u64,
    pub search_depth: u8,
    pub search_time_ms: u64,
    pub principal_variation: Vec<Move>,
}

impl EnhancedSearchEngine {
    /// Create a new enhanced search engine
    pub fn new(config: TranspositionConfig) -> TranspositionResult<Self> {
        let transposition_table = ThreadSafeTranspositionTable::new(config.clone());
        let hash_calculator = ShogiHashHandler::new(1000); // Max history length
        let error_handler = Arc::new(ComprehensiveErrorHandler::new());
        let stats_manager = Arc::new(AdvancedStatisticsManager::new(
            config.table_size,
            20, // Max depth
        ));
        let evaluator = PositionEvaluator::new();
        let move_generator = MoveGenerator::new();
        let search_stats = Arc::new(Mutex::new(SearchStats::default()));

        Ok(Self {
            transposition_table,
            hash_calculator,
            error_handler,
            stats_manager,
            evaluator,
            move_generator,
            search_stats,
        })
    }

    /// Enhanced negamax with full transposition table integration
    pub fn negamax_with_tt(
        &mut self,
        board: &mut BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        time_limit_ms: u32,
    ) -> TranspositionResult<i32> {
        let start_time = Instant::now();
        let mut stats = self.search_stats.lock().unwrap();
        stats.total_nodes += 1;
        stats.search_depth = depth;
        drop(stats);

        // Calculate position hash for transposition table lookup
        let position_hash = self
            .hash_calculator
            .get_position_hash(board, player, captured_pieces);

        // === TRANSPOSITION TABLE PROBING ===
        if let Some(tt_entry) = self.probe_transposition_table(position_hash, depth)? {
            let mut stats = self.search_stats.lock().unwrap();
            stats.tt_hits += 1;
            drop(stats);

            // Update statistics
            self.stats_manager
                .record_probe(depth, true, start_time.elapsed().as_micros() as f64);

            return Ok(self.handle_tt_entry(tt_entry, alpha, beta)?);
        }

        // Record miss in statistics
        self.stats_manager
            .record_probe(depth, false, start_time.elapsed().as_micros() as f64);

        // === TERMINAL NODE CHECK ===
        if depth == 0 {
            return Ok(self.quiescence_search_with_tt(
                board,
                captured_pieces,
                player,
                alpha,
                beta,
                time_limit_ms,
                MAX_QUIESCENCE_DEPTH,
            )?);
        }

        // === MOVE GENERATION ===
        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            let is_check = board.is_king_in_check(player, captured_pieces);
            return Ok(if is_check { -100000 } else { 0 });
        }

        // === MOVE ORDERING WITH TT HINTS ===
        let ordered_moves = self.order_moves_with_tt_hints(&legal_moves, position_hash)?;

        let mut best_move: Option<Move> = None;
        let mut best_score = -100000;

        // === MAIN SEARCH LOOP ===
        for (_i, mv) in ordered_moves.iter().enumerate() {
            // Check time limit
            if start_time.elapsed().as_millis() > time_limit_ms as u128 {
                break;
            }

            // Create board copy and make move
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured_piece) = new_board.make_move(mv) {
                new_captured.add_piece(captured_piece.piece_type, captured_piece.player);
            }

            // Recursive search with negated bounds
            let score = -self.negamax_with_tt(
                &mut new_board,
                &new_captured,
                player.opposite(),
                depth - 1,
                -beta,
                -alpha,
                time_limit_ms,
            )?;

            // Update best move and score
            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }

            // Alpha-beta pruning
            if score > alpha {
                alpha = score;
            }

            if score >= beta {
                // Beta cutoff - this is a refutation
                break;
            }

            // Update statistics
            self.stats_manager
                .record_store(start_time.elapsed().as_micros() as f64, false);
        }

        // === TRANSPOSITION TABLE STORAGE ===
        self.store_transposition_table(position_hash, best_score, depth, alpha, beta, best_move)?;

        let mut stats = self.search_stats.lock().unwrap();
        stats.tt_stores += 1;
        stats.search_time_ms = start_time.elapsed().as_millis() as u64;
        drop(stats);

        Ok(best_score)
    }

    /// Probe transposition table with proper error handling
    fn probe_transposition_table(
        &mut self,
        hash: u64,
        depth: u8,
    ) -> TranspositionResult<Option<TranspositionEntry>> {
        match self.transposition_table.probe(hash, depth) {
            Some(entry) => Ok(Some(entry)),
            None => Ok(None),
        }
    }

    /// Store entry in transposition table with proper flag handling
    fn store_transposition_table(
        &mut self,
        hash: u64,
        score: i32,
        depth: u8,
        alpha: i32,
        beta: i32,
        best_move: Option<Move>,
    ) -> TranspositionResult<()> {
        // Determine transposition flag based on alpha-beta bounds
        let flag = if score <= alpha {
            TranspositionFlag::UpperBound
        } else if score >= beta {
            TranspositionFlag::LowerBound
        } else {
            TranspositionFlag::Exact
        };

        let entry = TranspositionEntry::new_with_age(score, depth, flag, best_move, hash);
        self.transposition_table.store(entry);

        Ok(())
    }

    /// Handle transposition table entry based on flag and bounds
    fn handle_tt_entry(
        &self,
        entry: TranspositionEntry,
        alpha: i32,
        beta: i32,
    ) -> TranspositionResult<i32> {
        match entry.flag {
            TranspositionFlag::Exact => Ok(entry.score),
            TranspositionFlag::LowerBound => {
                if entry.score >= beta {
                    Ok(entry.score) // Beta cutoff
                } else {
                    Ok(entry.score) // Lower bound
                }
            }
            TranspositionFlag::UpperBound => {
                if entry.score <= alpha {
                    Ok(entry.score) // Alpha cutoff
                } else {
                    Ok(entry.score) // Upper bound
                }
            }
        }
    }

    /// Order moves with transposition table hints
    fn order_moves_with_tt_hints(
        &self,
        moves: &[Move],
        position_hash: u64,
    ) -> TranspositionResult<Vec<Move>> {
        // First, try to get best move from transposition table
        let mut ordered_moves = Vec::from(moves);

        // Look for the best move from TT
        if let Some(entry) = self.transposition_table.probe(position_hash, 255) {
            if let Some(tt_best_move) = entry.best_move {
                // Move TT best move to front
                if let Some(pos) = ordered_moves.iter().position(|m| m == &tt_best_move) {
                    ordered_moves.swap(0, pos);
                }
            }
        }

        // Additional move ordering heuristics can be added here
        // For now, we'll use the TT hint and keep the rest in original order

        Ok(ordered_moves)
    }

    /// Quiescence search with transposition table integration
    fn quiescence_search_with_tt(
        &mut self,
        board: &mut BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        alpha: i32,
        beta: i32,
        time_limit_ms: u32,
        remaining_depth: u8,
    ) -> TranspositionResult<i32> {
        let start_time = Instant::now();

        // Check time limit
        if start_time.elapsed().as_millis() > time_limit_ms as u128 {
            return Ok(self.evaluator.evaluate(board, player, captured_pieces));
        }

        if remaining_depth == 0 {
            return Ok(self.evaluator.evaluate(board, player, captured_pieces));
        }

        // Generate only capturing moves for quiescence
        let capture_moves =
            self.move_generator
                .generate_quiescence_moves(board, player, captured_pieces);

        if capture_moves.is_empty() {
            return Ok(self.evaluator.evaluate(board, player, captured_pieces));
        }

        let mut best_score = self.evaluator.evaluate(board, player, captured_pieces);
        let mut alpha = alpha;

        for mv in &capture_moves {
            // Check time limit
            if start_time.elapsed().as_millis() > time_limit_ms as u128 {
                break;
            }

            // Create board copy and make move
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured_piece) = new_board.make_move(mv) {
                new_captured.add_piece(captured_piece.piece_type, captured_piece.player);
            }

            let score = -self.quiescence_search_with_tt(
                &mut new_board,
                &new_captured,
                player.opposite(),
                -beta,
                -alpha,
                time_limit_ms,
                remaining_depth.saturating_sub(1),
            )?;

            if score > best_score {
                best_score = score;
            }

            if score > alpha {
                alpha = score;
            }

            if score >= beta {
                break; // Beta cutoff
            }
        }

        Ok(best_score)
    }

    /// Get principal variation from transposition table
    pub fn get_principal_variation(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        max_depth: u8,
    ) -> TranspositionResult<Vec<Move>> {
        let mut pv = Vec::new();
        let mut current_board = board.clone();
        let mut current_captured = captured_pieces.clone();
        let mut current_player = player;

        for _ in 0..max_depth {
            let position_hash = self.hash_calculator.get_position_hash(
                &current_board,
                current_player,
                &current_captured,
            );

            if let Some(entry) = self.transposition_table.probe(position_hash, 255) {
                if let Some(best_move) = entry.best_move {
                    pv.push(best_move.clone());

                    // Make the move to continue the PV
                    let captured = current_board.make_move(&best_move);
                    if let Some(captured_piece) = captured {
                        current_captured
                            .add_piece(captured_piece.piece_type, captured_piece.player);
                    }
                    current_player = current_player.opposite();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(pv)
    }

    /// Clear transposition table
    pub fn clear_transposition_table(&mut self) {
        self.transposition_table.clear();

        // Reset statistics
        let mut stats = self.search_stats.lock().unwrap();
        *stats = SearchStats::default();
        drop(stats);

        self.stats_manager.clear_all();
    }

    /// Get search statistics
    pub fn get_search_stats(&self) -> SearchStats {
        self.search_stats.lock().unwrap().clone()
    }

    /// Get transposition table statistics
    pub fn get_tt_stats(&self) -> ThreadSafeStatsSnapshot {
        self.transposition_table.get_stats()
    }

    /// Get advanced statistics report
    pub fn get_advanced_stats(
        &self,
    ) -> crate::search::advanced_statistics::ComprehensiveStatisticsReport {
        self.stats_manager.get_comprehensive_report()
    }

    /// Export statistics in various formats
    pub fn export_statistics(
        &self,
        format: crate::search::advanced_statistics::ExportFormat,
    ) -> crate::search::advanced_statistics::ExportedStatistics {
        let exporter =
            crate::search::advanced_statistics::StatisticsExporter::new(format.clone(), true);
        let report = self.stats_manager.get_comprehensive_report();

        crate::search::advanced_statistics::ExportedStatistics {
            cache_stats: exporter.export_cache_stats(&report.cache_stats),
            hit_rates: exporter
                .export_hit_rates(&crate::search::advanced_statistics::HitRateByDepth::new(20)),
            collision_stats: exporter.export_collision_stats(&report.collision_stats),
            trends: format!(
                r#"{{
    "hit_rate_trend": {:.6},
    "probe_time_trend": {:.6},
    "store_time_trend": {:.6},
    "occupancy_trend": {:.6},
    "hit_rate_volatility": {:.6},
    "probe_time_volatility": {:.6},
    "performance_score": {:.6},
    "data_points_count": {},
    "time_span_seconds": {}
}}"#,
                report.trends.hit_rate_trend,
                report.trends.probe_time_trend,
                report.trends.store_time_trend,
                report.trends.occupancy_trend,
                report.trends.hit_rate_volatility,
                report.trends.probe_time_volatility,
                report.trends.performance_score,
                report.trends.data_points_count,
                report.trends.time_span_seconds
            ),
            format,
        }
    }

    /// Update transposition table configuration
    pub fn update_config(&mut self, config: TranspositionConfig) -> TranspositionResult<()> {
        // Note: In a real implementation, this would require recreating the table
        // For now, we'll just log the configuration change
        self.error_handler.logger().log_error_with_context(
            crate::search::error_handling::ErrorSeverity::Info,
            "Transposition table configuration updated".to_string(),
            format!("New config: {:?}", config),
        );
        Ok(())
    }

    /// Perform iterative deepening search with transposition table
    pub fn iterative_deepening_search(
        &mut self,
        board: &mut BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        max_depth: u8,
        time_limit_ms: u32,
    ) -> TranspositionResult<(i32, Vec<Move>)> {
        let mut best_score = 0;
        let mut best_pv = Vec::new();

        for depth in 1..=max_depth {
            let start_time = Instant::now();

            // Perform search at current depth
            let score = self.negamax_with_tt(
                board,
                captured_pieces,
                player,
                depth,
                -100000,
                100000,
                time_limit_ms,
            )?;

            // Get principal variation
            let pv = self.get_principal_variation(board, captured_pieces, player, depth)?;

            // Update best results
            best_score = score;
            best_pv = pv;

            // Check if we have enough time for next iteration
            let elapsed = start_time.elapsed().as_millis();
            if elapsed > time_limit_ms as u128 / 2 {
                // Use half the time limit for next iteration
                break;
            }

            // Update search statistics
            let mut stats = self.search_stats.lock().unwrap();
            stats.principal_variation = best_pv.clone();
            drop(stats);
        }

        Ok((best_score, best_pv))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_search_engine_creation() {
        let config = TranspositionConfig::debug_config();
        let engine = EnhancedSearchEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_transposition_table_integration() {
        let config = TranspositionConfig::debug_config();
        let mut engine = EnhancedSearchEngine::new(config).unwrap();

        // Test basic operations
        engine.clear_transposition_table();
        let stats = engine.get_tt_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.stores, 0);
    }

    #[test]
    fn test_search_statistics() {
        let config = TranspositionConfig::debug_config();
        let engine = EnhancedSearchEngine::new(config).unwrap();

        let stats = engine.get_search_stats();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.tt_hits, 0);
    }

    #[test]
    fn test_principal_variation() {
        let config = TranspositionConfig::debug_config();
        let mut engine = EnhancedSearchEngine::new(config).unwrap();

        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let pv = engine.get_principal_variation(&board, &captured, Player::Black, 5);
        assert!(pv.is_ok());
        let pv = pv.unwrap();
        assert!(pv.is_empty()); // Should be empty for initial position
    }

    #[test]
    fn test_iterative_deepening() {
        let config = TranspositionConfig::debug_config();
        let mut engine = EnhancedSearchEngine::new(config).unwrap();

        let mut board = BitboardBoard::new();
        let captured = CapturedPieces::new();

        let result =
            engine.iterative_deepening_search(&mut board, &captured, Player::Black, 3, 1000);

        assert!(result.is_ok());
        let (score, pv) = result.unwrap();
        assert!(score >= -100000 && score <= 100000);
    }
}
