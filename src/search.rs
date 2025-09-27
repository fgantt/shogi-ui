use crate::types::*;
use crate::bitboards::*;
use crate::evaluation::*;
use crate::moves::*;
use std::collections::HashMap;
use crate::time_utils::TimeSource;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct SearchEngine {
    evaluator: PositionEvaluator,
    move_generator: MoveGenerator,
    transposition_table: HashMap<String, TranspositionEntry>,
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
    previous_scores: Vec<i32>,
}

impl SearchEngine {
    pub fn new(stop_flag: Option<Arc<AtomicBool>>, hash_size_mb: usize) -> Self {
        Self::new_with_config(stop_flag, hash_size_mb, QuiescenceConfig::default())
    }

    pub fn new_with_config(stop_flag: Option<Arc<AtomicBool>>, hash_size_mb: usize, quiescence_config: QuiescenceConfig) -> Self {
        const BYTES_PER_ENTRY: usize = 100; // Approximate size of a TT entry
        let capacity = hash_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        let quiescence_capacity = quiescence_config.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        Self {
            evaluator: PositionEvaluator::new(),
            move_generator: MoveGenerator::new(),
            transposition_table: HashMap::with_capacity(capacity),
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
            previous_scores: Vec::new(),
        }
    }

    /// Create a new SearchEngine with full EngineConfig
    pub fn new_with_engine_config(stop_flag: Option<Arc<AtomicBool>>, config: EngineConfig) -> Self {
        const BYTES_PER_ENTRY: usize = 100; // Approximate size of a TT entry
        let capacity = config.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        let quiescence_capacity = config.quiescence.tt_size_mb * 1024 * 1024 / BYTES_PER_ENTRY;
        
        Self {
            evaluator: PositionEvaluator::new(),
            move_generator: MoveGenerator::new(),
            transposition_table: HashMap::with_capacity(capacity),
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
            previous_scores: Vec::new(),
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
        
        // Reset statistics when configuration changes
        self.quiescence_stats.reset();
        self.null_move_stats.reset();
        self.lmr_stats.reset();
        self.aspiration_stats.reset();
        
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
            tt_size_mb: self.transposition_table.capacity() * 100 / (1024 * 1024), // Approximate
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

    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32, alpha: i32, beta: i32) -> Option<(Move, i32)> {
        crate::debug_utils::debug_log(&format!("Inside search_at_depth, depth={}", depth));
        
        self.nodes_searched = 0;
        let start_time = TimeSource::now();
        let mut alpha = alpha;
        
        let mut best_move: Option<Move> = None;
        crate::debug_utils::debug_log(&format!("Initial best_move: {:?}", best_move));
        let mut best_score = alpha;
        
        crate::debug_utils::debug_log("About to generate legal moves");
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            crate::debug_utils::debug_log("No legal moves found");
            return None;
        }
        
        crate::debug_utils::debug_log(&format!("Found {} legal moves", legal_moves.len()));
        
        // Debug: log the first few moves
        for (i, mv) in legal_moves.iter().take(5).enumerate() {
            crate::debug_utils::debug_log(&format!("Move {}: {}", i, mv.to_usi_string()));
        }
        
        crate::debug_utils::debug_log("About to sort moves");
        
        let sorted_moves = self.sort_moves(&legal_moves, board);
        
        crate::debug_utils::debug_log("About to start move evaluation loop");
        
        let mut history: Vec<String> = vec![board.to_fen(player, captured_pieces)];

        for move_ in sorted_moves {
            if self.should_stop(&start_time, time_limit_ms) { break; }
            
            crate::debug_utils::debug_log("About to make move");
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }
            
            let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, &start_time, time_limit_ms, &mut history, true);
            
            if score > best_score {
                best_score = score;
                best_move = Some(move_);
            }
            
            if score > alpha {
                alpha = score;
            }
        }

        best_move.map(|m| (m, best_score))
    }

    /// Backward-compatible wrapper for search_at_depth without alpha/beta parameters
    pub fn search_at_depth_legacy(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32) -> Option<(Move, i32)> {
        self.search_at_depth(board, captured_pieces, player, depth, time_limit_ms, i32::MIN + 1, i32::MAX - 1)
    }

    fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool) -> i32 {
        self.negamax_with_context(board, captured_pieces, player, depth, alpha, beta, start_time, time_limit_ms, history, can_null_move, false, false, false)
    }
    
    fn negamax_with_context(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool, is_root: bool, has_capture: bool, has_check: bool) -> i32 {
        if self.should_stop(&start_time, time_limit_ms) { return 0; }
        self.nodes_searched += 1;
        let fen_key = board.to_fen(player, captured_pieces);
        if history.contains(&fen_key) {
            return 0; // Repetition is a draw
        }

        if let Some(entry) = self.transposition_table.get(&fen_key) {
            if entry.depth >= depth {
                match entry.flag {
                    TranspositionFlag::Exact => return entry.score,
                    TranspositionFlag::LowerBound => if entry.score >= beta { return entry.score; },
                    TranspositionFlag::UpperBound => if entry.score <= alpha { return entry.score; },
                }
            }
        }
        
        // === NULL MOVE PRUNING ===
        if self.should_attempt_null_move(board, captured_pieces, player, depth, can_null_move) {
            let null_move_score = self.perform_null_move_search(
                board, captured_pieces, player, depth, beta, start_time, time_limit_ms, history
            );
            
            if null_move_score >= beta {
                // Beta cutoff - position is too good, prune this branch
                self.null_move_stats.cutoffs += 1;
                return beta;
            }
        }
        // === END NULL MOVE PRUNING ===
        
        if depth == 0 {
            return self.quiescence_search(&mut board.clone(), captured_pieces, player, alpha, beta, &start_time, time_limit_ms, 5);
        }
        
        // Use the passed context parameters
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            return if board.is_king_in_check(player, captured_pieces) { -100000 } else { 0 };
        }
        
        let sorted_moves = self.sort_moves(&legal_moves, board);
        let mut best_score = -200000;
        let mut best_move_for_tt = None;
        
        history.push(fen_key.clone());

        let mut move_index = 0;
        for move_ in sorted_moves {
            if self.should_stop(&start_time, time_limit_ms) { break; }
            move_index += 1;
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }

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
                &move_, 
                move_index,
                is_root,
                move_.is_capture,
                has_check
            );

            if score > best_score {
                best_score = score;
                best_move_for_tt = Some(move_.clone());
                if score > alpha {
                    alpha = score;
                    if !move_.is_capture {
                        self.update_killer_moves(move_.clone());
                    }
                    if let Some(from) = move_.from {
                        self.history_table[from.row as usize][from.col as usize] += (depth * depth) as i32;
                    }
                }
                if alpha >= beta { break; }
            }
        }
        
        history.pop();

        let flag = if best_score <= alpha { TranspositionFlag::UpperBound } else if best_score >= beta { TranspositionFlag::LowerBound } else { TranspositionFlag::Exact };
        self.transposition_table.insert(fen_key, TranspositionEntry { score: best_score, depth, flag, best_move: best_move_for_tt });
        
        best_score
    }

    fn quiescence_search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
        if self.should_stop(&start_time, time_limit_ms) { return 0; }

        // Update statistics
        self.quiescence_stats.nodes_searched += 1;

        // Check depth limit
        if depth == 0 || depth > self.quiescence_config.max_depth {
            return self.evaluator.evaluate_with_context(board, player, captured_pieces, depth, false, false, false, true);
        }

        // Transposition table lookup
        if self.quiescence_config.enable_tt {
            // Clean up TT if it's getting too large
            if self.quiescence_tt.len() > self.quiescence_config.tt_cleanup_threshold {
                self.cleanup_quiescence_tt(self.quiescence_config.tt_cleanup_threshold / 2);
            }
            
            let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
            if let Some(entry) = self.quiescence_tt.get(&fen_key) {
                if entry.depth >= depth {
                    self.quiescence_stats.tt_hits += 1;
                    match entry.flag {
                        TranspositionFlag::Exact => return entry.score,
                        TranspositionFlag::LowerBound => if entry.score >= beta { return entry.score; },
                        TranspositionFlag::UpperBound => if entry.score <= alpha { return entry.score; },
                    }
                }
            } else {
                self.quiescence_stats.tt_misses += 1;
            }
        }
        
        let stand_pat = self.evaluator.evaluate_with_context(board, player, captured_pieces, depth, false, false, false, true);
        if stand_pat >= beta { return beta; }
        if alpha < stand_pat { alpha = stand_pat; }
        
        let noisy_moves = self.generate_noisy_moves(board, player, captured_pieces);
        
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
        
        let sorted_noisy_moves = self.sort_quiescence_moves(&noisy_moves);
        self.quiescence_stats.moves_ordered += noisy_moves.len() as u64;

        for move_ in sorted_noisy_moves {
            if self.should_stop(&start_time, time_limit_ms) { break; }
            
            // Apply pruning checks
            if self.should_prune_delta(&move_, stand_pat, alpha) {
                self.quiescence_stats.delta_prunes += 1;
                continue;
            }
            
            if self.should_prune_futility(&move_, stand_pat, alpha, depth) {
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
                self.quiescence_stats.extensions += 1;
                depth - 1 // Still reduce depth but less aggressively
            } else {
                depth - 1
            };
            
            let score = -self.quiescence_search(&mut new_board, &new_captured, player.opposite(), -beta, -alpha, &start_time, time_limit_ms, search_depth);
            
            if score >= beta { 
                // Store result in transposition table
                if self.quiescence_config.enable_tt {
                    let fen_key = format!("q_{}", board.to_fen(player, captured_pieces));
                    let flag = TranspositionFlag::LowerBound;
                    self.quiescence_tt.insert(fen_key, QuiescenceEntry {
                        score: beta,
                        depth,
                        flag,
                        best_move: Some(move_),
                    });
                }
                return beta; 
            }
            if score > alpha { alpha = score; }
        }
        
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

    

    fn sort_moves(&self, moves: &[Move], board: &BitboardBoard) -> Vec<Move> {
        let mut scored_moves: Vec<(Move, i32)> = moves.iter().map(|m| (m.clone(), self.score_move(m, board))).collect();
        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
        scored_moves.into_iter().map(|(m, _)| m).collect()
    }

    fn score_move(&self, move_: &Move, _board: &BitboardBoard) -> i32 {
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

    fn moves_equal(&self, move1: &Move, move2: &Move) -> bool {
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
        self.transposition_table.len()
    }

    #[cfg(test)]
    pub fn transposition_table_capacity(&self) -> usize {
        self.transposition_table.capacity()
    }

    fn get_pv(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut current_board = board.clone();
        let mut current_captured = captured_pieces.clone();
        let mut current_player = player;
    
        for _ in 0..depth {
            let fen_key = current_board.to_fen(current_player, &current_captured);
            if let Some(entry) = self.transposition_table.get(&fen_key) {
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
            search_depth, -beta, -beta + 1, 
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
        if self.transposition_table.len() > 100_000 {
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
        
        // Validate inputs
        if !self.validate_window_parameters(previous_score, window_size) {
            crate::debug_utils::debug_log("Aspiration: Invalid parameters in handle_fail_low, using fallback");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return;
        }
        
        // Widen window downward with adaptive sizing
        let new_alpha = i32::MIN + 1;
        let new_beta = previous_score + window_size;
        
        // Ensure valid window bounds
        if new_beta <= new_alpha {
            crate::debug_utils::debug_log("Aspiration: Invalid window bounds in fail-low, using fallback");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return;
        }
        
        *alpha = new_alpha;
        *beta = new_beta;
        
        // Log for debugging with performance metrics
        crate::debug_utils::debug_log(&format!(
            "Aspiration: Fail-low, widening window to [{}, {}] (prev_score={}, window_size={})",
            *alpha, *beta, previous_score, window_size
        ));
        
        // Update performance metrics
        self.update_fail_low_metrics(previous_score, window_size);
    }

    /// Handle fail-high by widening window upward
    fn handle_fail_high(&mut self, alpha: &mut i32, beta: &mut i32,
                        previous_score: i32, window_size: i32) {
        self.aspiration_stats.fail_highs += 1;
        
        // Validate inputs
        if !self.validate_window_parameters(previous_score, window_size) {
            crate::debug_utils::debug_log("Aspiration: Invalid parameters in handle_fail_high, using fallback");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return;
        }
        
        // Widen window upward with adaptive sizing
        let new_alpha = previous_score - window_size;
        let new_beta = i32::MAX - 1;
        
        // Ensure valid window bounds
        if new_alpha >= new_beta {
            crate::debug_utils::debug_log("Aspiration: Invalid window bounds in fail-high, using fallback");
            *alpha = i32::MIN + 1;
            *beta = i32::MAX - 1;
            return;
        }
        
        *alpha = new_alpha;
        *beta = new_beta;
        
        // Log for debugging with performance metrics
        crate::debug_utils::debug_log(&format!(
            "Aspiration: Fail-high, widening window to [{}, {}] (prev_score={}, window_size={})",
            *alpha, *beta, previous_score, window_size
        ));
        
        // Update performance metrics
        self.update_fail_high_metrics(previous_score, window_size);
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
                self.aspiration_stats.total_researches as f64 / self.aspiration_stats.total_searches as f64
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
                self.aspiration_stats.total_researches as f64 / self.aspiration_stats.total_searches as f64
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
                           is_root: bool,
                           has_capture: bool,
                           has_check: bool) -> i32 {
        
        self.lmr_stats.moves_considered += 1;
        
        // Check if LMR should be applied
        if self.should_apply_lmr(move_, depth, move_index) {
            self.lmr_stats.reductions_applied += 1;
            
            // Calculate reduction amount (use optimized version)
            let reduction = self.calculate_reduction_optimized(move_, depth, move_index);
            self.lmr_stats.total_depth_saved += reduction as u64;
            
            // Perform reduced-depth search with null window
            let reduced_depth = depth - 1 - reduction;
            let score = -self.negamax_with_context(
                board, 
                captured_pieces, 
                player.opposite(), 
                reduced_depth, 
                -alpha - 1, 
                -alpha, 
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
                
                // Re-search at full depth
                let full_score = -self.negamax_with_context(
                    board, 
                    captured_pieces, 
                    player.opposite(), 
                    depth - 1, 
                    -beta, 
                    -alpha, 
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
                -beta, 
                -alpha, 
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
        if self.transposition_table.len() > 100_000 {
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
}


use js_sys::Function;

pub struct IterativeDeepening {
    max_depth: u8,
    time_limit_ms: u32,
    stop_flag: Option<Arc<AtomicBool>>,
    on_info: Option<Function>,
}

impl IterativeDeepening {
    pub fn new(max_depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>, on_info: Option<Function>) -> Self {
        Self {
            max_depth,
            time_limit_ms,
            stop_flag,
            on_info,
        }
    }

    pub fn search(&mut self, search_engine: &mut SearchEngine, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> Option<(Move, i32)> {
        crate::debug_utils::debug_log("Inside search method");
        
        crate::debug_utils::debug_log("About to get start time");
        let start_time = TimeSource::now();
        
        let mut best_move: Option<Move> = None;
        let mut best_score = 0;
        let mut previous_scores = Vec::new();
        
        crate::debug_utils::debug_log("About to calculate search time limit");
        let search_time_limit = self.time_limit_ms.saturating_sub(100);

        crate::debug_utils::debug_log("Starting search loop");

        for depth in 1..=self.max_depth {
            if self.should_stop(&start_time, search_time_limit) { break; }
            let elapsed_ms = start_time.elapsed_ms();
            let remaining_time = search_time_limit.saturating_sub(elapsed_ms);

            crate::debug_utils::debug_log(&format!("Searching at depth {}", depth));

            // Calculate aspiration window parameters
            let (alpha, beta) = if depth == 1 || !search_engine.aspiration_config.enabled {
                // First depth or disabled: use full-width window
                (i32::MIN + 1, i32::MAX - 1)
            } else {
                // Use aspiration window based on previous score
                let previous_score = previous_scores.last().copied().unwrap_or(0);
                let window_size = search_engine.calculate_window_size(depth, previous_score, 0);
                (previous_score - window_size, previous_score + window_size)
            };

            // Perform search with aspiration window
            let mut search_result = None;
            let mut researches = 0;
            let mut current_alpha = alpha;
            let mut current_beta = beta;

            loop {
                if researches >= search_engine.aspiration_config.max_researches {
                    // Fall back to full-width search
                    current_alpha = i32::MIN + 1;
                    current_beta = i32::MAX - 1;
                }

                if let Some((move_, score)) = search_engine.search_at_depth(
                    board, captured_pieces, player, depth, remaining_time,
                    current_alpha, current_beta
                ) {
                    search_result = Some((move_.clone(), score));
                    
                    if score <= current_alpha {
                        // Fail-low: widen window downward
                        search_engine.handle_fail_low(&mut current_alpha, &mut current_beta, 
                                                    previous_scores.last().copied().unwrap_or(0), 
                                                    search_engine.calculate_window_size(depth, 0, 0));
                        researches += 1;
                        continue;
                    }
                    
                    if score >= current_beta {
                        // Fail-high: widen window upward
                        search_engine.handle_fail_high(&mut current_alpha, &mut current_beta,
                                                     previous_scores.last().copied().unwrap_or(0),
                                                     search_engine.calculate_window_size(depth, 0, 0));
                        researches += 1;
                        continue;
                    }
                    
                    // Success: score within window
                    best_move = Some(move_);
                    best_score = score;
                    previous_scores.push(score);
                    break;
                } else {
                    // Search failed completely
                    break;
                }
            }

            // Update statistics
            search_engine.update_aspiration_stats(researches > 0, researches);

            if let Some((_move_, score)) = search_result {
                let pv = search_engine.get_pv(board, captured_pieces, player, depth);
                let pv_string = pv.iter().map(|m| m.to_usi_string()).collect::<Vec<String>>().join(" ");
                let time_searched = start_time.elapsed_ms();
                let nps = if time_searched > 0 { search_engine.nodes_searched * 1000 / time_searched as u64 } else { 0 };

                let info_string = format!("info depth {} score cp {} time {} nodes {} nps {} pv {}", depth, score, time_searched, search_engine.nodes_searched, nps, pv_string);
                if let Some(on_info) = &self.on_info {
                    let this = wasm_bindgen::JsValue::NULL;
                    let s = wasm_bindgen::JsValue::from_str(&info_string);
                    if let Err(e) = on_info.call1(&this, &s) {
                        crate::debug_utils::debug_log(&format!("Error calling on_info callback: {:?}", e));
                    }
                }

                // Only break early for extremely winning positions (king capture level)
                // and only at higher depths to allow deeper search logging for higher AI levels
                if score > 50000 && depth >= 6 { break; } 
            } else {
                break;
            }
        }
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
