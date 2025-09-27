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
        }
    }

    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32) -> Option<(Move, i32)> {
        crate::debug_utils::debug_log(&format!("Inside search_at_depth, depth={}", depth));
        
        self.nodes_searched = 0;
        let start_time = TimeSource::now();
        let mut alpha = -200000;
        
        let mut best_move: Option<Move> = None;
        crate::debug_utils::debug_log(&format!("Initial best_move: {:?}", best_move));
        let mut best_score = -200000;
        
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
            
            let beta = 200000;
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

    fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool) -> i32 {
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
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            return if board.is_king_in_check(player, captured_pieces) { -100000 } else { 0 };
        }
        
        let sorted_moves = self.sort_moves(&legal_moves, board);
        let mut best_score = -200000;
        let mut best_move_for_tt = None;
        
        history.push(fen_key.clone());

        for move_ in sorted_moves {
            if self.should_stop(&start_time, time_limit_ms) { break; }
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }

                let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, &start_time, time_limit_ms, history, true);

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
            return self.evaluator.evaluate(board, player, captured_pieces);
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
        
        let stand_pat = self.evaluator.evaluate(board, player, captured_pieces);
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
        let null_move_score = -self.negamax(
            board, captured_pieces, player.opposite(), 
            search_depth, -beta, -beta + 1, 
            start_time, time_limit_ms, history, 
            false  // Prevent recursive null moves
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
        let mut best_score = -200000;
        
        crate::debug_utils::debug_log("About to calculate search time limit");
        let search_time_limit = self.time_limit_ms.saturating_sub(100);

        crate::debug_utils::debug_log("Starting search loop");

        for depth in 1..=self.max_depth {
            if self.should_stop(&start_time, search_time_limit) { break; }
            let elapsed_ms = start_time.elapsed_ms();
            let remaining_time = search_time_limit.saturating_sub(elapsed_ms);

            crate::debug_utils::debug_log(&format!("Searching at depth {}", depth));

            if let Some((move_, score)) = search_engine.search_at_depth(board, captured_pieces, player, depth, remaining_time) {
                best_move = Some(move_);
                best_score = score;

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
