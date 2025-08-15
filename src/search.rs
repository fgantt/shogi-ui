use crate::types::*;
use crate::bitboards::*;
use crate::evaluation::*;
use crate::moves::*;
use std::collections::HashMap;
use web_sys::console;

fn now() -> f64 {
    let window = web_sys::window().expect("should have a window in this context");
    let performance = window.performance().expect("performance should be available");
    let now  = performance.now();
    now
}

/// Search engine for the Shogi AI
pub struct SearchEngine {
    evaluator: PositionEvaluator,
    move_generator: MoveGenerator,
    transposition_table: HashMap<u64, TranspositionEntry>,
    history_table: [[i32; 9]; 9],
    killer_moves: [Option<Move>; 2],
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            move_generator: MoveGenerator::new(),
            transposition_table: HashMap::new(),
            history_table: [[0; 9]; 9],
            killer_moves: [None, None],
        }
    }

    /// Search for the best move at a given depth
    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32) -> Option<(Move, i32)> {
        console::log_1(&format!("search_at_depth: depth={}, player={:?}", depth, player).into());
        console::log_1(&format!("search_at_depth: entered function").into());
        let start_time = now();
        let mut alpha = i32::MIN + 1;
        let mut beta = i32::MAX - 1;
        
        let mut best_move = None;
        let mut best_score = i32::MIN;
        
        console::log_1(&format!("search_at_depth: before generate_legal_moves").into());
        // Generate all legal moves
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        console::log_1(&format!("search_at_depth: after generate_legal_moves, count={}", legal_moves.len()).into());
        
        if legal_moves.is_empty() {
            console::log_1(&format!("search_at_depth: no legal moves found").into());
            return None;
        }
        
        console::log_1(&format!("search_at_depth: before sort_moves").into());
        // Sort moves for better pruning
        let sorted_moves = self.sort_moves(&legal_moves, board);
        console::log_1(&format!("search_at_depth: after sort_moves").into());
        
        console::log_1(&format!("search_at_depth: entering move loop").into());
        for move_ in sorted_moves {
            console::log_1(&format!("search_at_depth: inside move loop, current move={:?}", move_).into());
            // Check time limit
            if (now() - start_time) > time_limit_ms as f64 {
                console::log_1(&format!("search_at_depth: time limit exceeded").into());
                break;
            }
            
            console::log_1(&format!("search_at_depth: before make_move").into());
            // Make move
            let mut new_board = board.clone();
            let captured_piece = new_board.make_move(&move_);
            console::log_1(&format!("search_at_depth: after make_move").into());

            let mut new_captured = captured_pieces.clone();
            if let Some(piece) = captured_piece {
                new_captured.add_piece(piece.piece_type, player);
            }
            
            console::log_1(&format!("search_at_depth: before negamax").into());
            // Search
            let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms);
            console::log_1(&format!("search_at_depth: after negamax, score={}", score).into());
            
            // Update best move and score
            if score > best_score {
                best_score = score;
                best_move = Some(move_);
            }
            
            // Alpha-beta pruning
            if score >= beta {
                console::log_1(&format!("search_at_depth: beta cutoff, score={}", score).into());
                break;
            }
            if score > alpha {
                alpha = score;
            }
        }

        console::log_1(&format!("search_at_depth: best_move={:?}, best_score={}", best_move, best_score).into());
        best_move.map(|m| (m, best_score))
    }

    /// Negamax search with alpha-beta pruning
    fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, alpha: i32, beta: i32, start_time: f64, time_limit_ms: u32) -> i32 {
        console::log_1(&format!("negamax: depth={}, player={:?}, alpha={}, beta={}", depth, player, alpha, beta).into());
        // Check time limit
        if (now() - start_time) > time_limit_ms as f64 {
            console::log_1(&format!("negamax: time limit exceeded").into());
            return 0;
        }
        
        // Check transposition table
        let hash = board.get_zobrist_hash();
        if let Some(entry) = self.transposition_table.get(&hash) {
            if entry.depth >= depth {
                match entry.flag {
                    TranspositionFlag::Exact => {
                        console::log_1(&format!("negamax: TT hit (Exact), score={}", entry.score).into());
                        return entry.score;
                    }
                    TranspositionFlag::LowerBound => {
                        if entry.score >= beta {
                            console::log_1(&format!("negamax: TT hit (LowerBound), score={}", entry.score).into());
                            return entry.score;
                        }
                    }
                    TranspositionFlag::UpperBound => {
                        if entry.score <= alpha {
                            console::log_1(&format!("negamax: TT hit (UpperBound), score={}", entry.score).into());
                            return entry.score;
                        }
                    }
                }
            }
        }
        
        // Leaf node evaluation
        if depth == 0 {
            let score = self.quiescence_search(board, captured_pieces, player, alpha, beta, start_time, time_limit_ms);
            console::log_1(&format!("negamax: depth=0, score={}", score).into());
            return score;
        }
        
        // Null move pruning
        if depth >= 3 && !board.is_king_in_check(player) {
            console::log_1(&format!("negamax: null move pruning").into());
            let null_move_score = -self.negamax(board, captured_pieces, player.opposite(), depth - 3, -beta, -beta + 1, start_time, time_limit_ms);
            if null_move_score >= beta {
                console::log_1(&format!("negamax: null move pruning cutoff, score={}", null_move_score).into());
                return beta;
            }
        }
        
        // Futility pruning
        let stand_pat = self.evaluator.evaluate(board, player);
        if depth <= 2 && stand_pat - 300 >= beta {
            console::log_1(&format!("negamax: futility pruning, stand_pat={}", stand_pat).into());
            return stand_pat;
        }
        
        // Generate moves
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            if board.is_king_in_check(player) {
                console::log_1(&format!("negamax: checkmate").into());
                return i32::MIN + 1; // Checkmate
            } else {
                console::log_1(&format!("negamax: stalemate").into());
                return 0; // Stalemate
            }
        }
        
        // Sort moves
        let sorted_moves = self.sort_moves(&legal_moves, board);
        
        let mut best_score = i32::MIN;
        let mut alpha = alpha;
        
        for (i, move_) in sorted_moves.iter().enumerate() {
            console::log_1(&format!("negamax: trying move {:?}", move_).into());
            // Make move
            let mut new_board = board.clone();
            let captured_piece = new_board.make_move(move_);

            let mut new_captured = captured_pieces.clone();
            if let Some(piece) = captured_piece {
                new_captured.add_piece(piece.piece_type, player);
            }
            
            let score = if i == 0 {
                // Principal variation search
                -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms)
            } else {
                // Null window search
                let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -alpha - 1, -alpha, start_time, time_limit_ms);
                if score > alpha && score < beta {
                    // Re-search with full window
                    -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms)
                } else {
                    score
                }
            };
            
            if score > best_score {
                best_score = score;
                
                if score > alpha {
                    alpha = score;
                    
                    // Update killer moves
                    if !move_.is_capture {
                        self.update_killer_moves(move_.clone());
                    }
                    
                    // Update history table
                    if let Some(from) = move_.from {
                        self.history_table[from.row as usize][from.col as usize] += (depth * depth) as i32;
                    }
                }
                
                if alpha >= beta {
                    console::log_1(&format!("negamax: beta cutoff, score={}", score).into());
                    break; // Beta cutoff
                }
            }
        }
        
        // Store in transposition table
        let flag = if best_score <= alpha {
            TranspositionFlag::UpperBound
        } else if best_score >= beta {
            TranspositionFlag::LowerBound
        } else {
            TranspositionFlag::Exact
        };
        
        let entry = TranspositionEntry {
            score: best_score,
            depth,
            flag,
            best_move: None,
        };
        
        self.transposition_table.insert(hash, entry);
        
        console::log_1(&format!("negamax: returning score={}", best_score).into());
        best_score
    }

    /// Quiescence search for tactical positions
    fn quiescence_search(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: f64, time_limit_ms: u32) -> i32 {
        console::log_1(&format!("quiescence_search: player={:?}, alpha={}, beta={}", player, alpha, beta).into());
        // Check time limit
        if (now() - start_time) > time_limit_ms as f64 {
            console::log_1(&format!("quiescence_search: time limit exceeded").into());
            return 0;
        }
        
        let stand_pat = self.evaluator.evaluate(board, player);
        console::log_1(&format!("quiescence_search: stand_pat={}", stand_pat).into());
        
        if stand_pat >= beta {
            console::log_1(&format!("quiescence_search: stand_pat >= beta, returning beta").into());
            return beta;
        }
        
        if alpha < stand_pat {
            alpha = stand_pat;
            console::log_1(&format!("quiescence_search: alpha updated to stand_pat={}", alpha).into());
        }
        
        // Generate only captures and checks
        let noisy_moves = self.generate_noisy_moves(board, player, captured_pieces);
        console::log_1(&format!("quiescence_search: generated {} noisy moves", noisy_moves.len()).into());
        
        for move_ in noisy_moves {
            console::log_1(&format!("quiescence_search: trying noisy move {:?}", move_).into());
            // Check time limit
            if (now() - start_time) > time_limit_ms as f64 {
                console::log_1(&format!("quiescence_search: time limit exceeded during noisy move").into());
                break;
            }
            
            // Make move
            let mut new_board = board.clone();
            let captured_piece = new_board.make_move(&move_);

            let mut new_captured = captured_pieces.clone();
            if let Some(piece) = captured_piece {
                new_captured.add_piece(piece.piece_type, player);
            }
            
            // Search
            let score = -self.quiescence_search(&new_board, &new_captured, player.opposite(), -beta, -alpha, start_time, time_limit_ms);
            
            if score >= beta {
                console::log_1(&format!("quiescence_search: score >= beta, returning beta").into());
                return beta;
            }
            
            if score > alpha {
                alpha = score;
                console::log_1(&format!("quiescence_search: alpha updated to score={}", alpha).into());
            }
        }
        
        console::log_1(&format!("quiescence_search: returning alpha={}", alpha).into());
        alpha
    }

    /// Generate noisy moves (captures, checks, promotions)
    fn generate_noisy_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
        let mut noisy_moves = Vec::new();
        
        // Generate all legal moves
        let all_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        
        for move_ in all_moves {
            if move_.is_capture || move_.is_promotion || self.is_checking_move(board, &move_) {
                noisy_moves.push(move_);
            }
        }
        console::log_1(&format!("generate_noisy_moves: generated {} noisy moves", noisy_moves.len()).into());
        noisy_moves
    }

    /// Check if a move gives check
    fn is_checking_move(&self, board: &BitboardBoard, move_: &Move) -> bool {
        console::log_1(&format!("is_checking_move: checking move {:?}", move_).into());
        let mut new_board = board.clone();
        new_board.make_move(move_);
        let is_check = new_board.is_king_in_check(move_.player.opposite());
        console::log_1(&format!("is_checking_move: move {:?} results in check: {}", move_, is_check).into());
        is_check
    }

    /// Sort moves for better alpha-beta pruning
    fn sort_moves(&self, moves: &[Move], board: &BitboardBoard) -> Vec<Move> {
        let mut scored_moves: Vec<(Move, i32)> = moves.iter().map(|m| (m.clone(), self.score_move(m, board))).collect();
        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
        scored_moves.into_iter().map(|(m, _)| m).collect()
    }

    /// Score a move for move ordering
    fn score_move(&self, move_: &Move, board: &BitboardBoard) -> i32 {
        let mut score = 0;
        
        // Promotion bonus
        if move_.is_promotion {
            score += 800;
        }
        
        // Capture bonus (MVV-LVA)
        if move_.is_capture {
            if let Some(captured_piece) = &move_.captured_piece {
                score += captured_piece.piece_type.base_value() * 10;
            }
            score += 1000; // Base capture bonus
        }
        
        // Killer move bonus
        if let Some(killer) = &self.killer_moves[0] {
            if self.moves_equal(move_, killer) {
                score += 900;
            }
        }
        if let Some(killer) = &self.killer_moves[1] {
            if self.moves_equal(move_, killer) {
                score += 800;
            }
        }
        
        // History bonus
        if let Some(from) = move_.from {
            score += self.history_table[from.row as usize][from.col as usize];
        }
        
        // Center control bonus
        if move_.to.row >= 3 && move_.to.row <= 5 && move_.to.col >= 3 && move_.to.col <= 5 {
            score += 20;
        }
        
        score
    }

    /// Check if two moves are equal
    fn moves_equal(&self, move1: &Move, move2: &Move) -> bool {
        move1.from == move2.from && move1.to == move2.to && move1.piece_type == move2.piece_type
    }

    /// Update killer moves
    fn update_killer_moves(&mut self, new_killer: Move) {
        // Check if it's already a killer move
        if let Some(killer) = &self.killer_moves[0] {
            if self.moves_equal(&new_killer, killer) {
                return;
            }
        }
        if let Some(killer) = &self.killer_moves[1] {
            if self.moves_equal(&new_killer, killer) {
                return;
            }
        }
        
        // Shift killer moves and add new one
        self.killer_moves[1] = self.killer_moves[0].take();
        self.killer_moves[0] = Some(new_killer);
    }

    /// Clear search state
    pub fn clear(&mut self) {
        self.transposition_table.clear();
        self.history_table = [[0; 9]; 9];
        self.killer_moves = [None, None];
    }
}

/// Iterative deepening search
pub struct IterativeDeepening {
    search_engine: SearchEngine,
    max_depth: u8,
    time_limit_ms: u32,
}

impl IterativeDeepening {
    pub fn new(max_depth: u8, time_limit_ms: u32) -> Self {
        console::log_1(&format!("IterativeDeepening::new: max_depth={}, time_limit_ms={}", max_depth, time_limit_ms).into());
        Self {
            search_engine: SearchEngine::new(),
            max_depth,
            time_limit_ms,
        }
    }

    /// Perform iterative deepening search
    pub fn search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> Option<(Move, i32)> {
        console::log_1(&format!("IterativeDeepening::search: starting search for player {:?}", player).into());
        // Add log before start_time
        console::log_1(&format!("IterativeDeepening::search: before start_time").into());
        let start_time = now();
        // Add log before best_move
        console::log_1(&format!("IterativeDeepening::search: before best_move").into());
        let mut best_move = None;
        console::log_1(&format!("IterativeDeepening::search: best_move initialized").into());
        // Add log before best_score
        console::log_1(&format!("IterativeDeepening::search: before best_score").into());
        let mut best_score = i32::MIN;
        console::log_1(&format!("IterativeDeepening::search: best_score initialized").into());
        
        // Reserve some time for the final iteration
        // Add log before search_time_limit
        console::log_1(&format!("IterativeDeepening::search: before search_time_limit").into());
        let search_time_limit = self.time_limit_ms - 100; // Reserve 100ms
        console::log_1(&format!("IterativeDeepening::search: search_time_limit initialized").into());
        
        for depth in 1..=self.max_depth {
            console::log_1(&format!("IterativeDeepening::search: searching at depth {}", depth).into());
            // Check if we have enough time
                        console::log_1(&"before elapsed".into());
            let elapsed = now() - start_time;
            console::log_1(&"after elapsed".into());
            if elapsed >= search_time_limit as f64 {
                console::log_1(&format!("IterativeDeepening::search: time limit reached, breaking").into());
                break;
            }
            
            let remaining_time = (search_time_limit as f64 - elapsed) as u32;
            web_sys::console::log_1(&format!("IterativeDeepening::search: calling search_at_depth with board_hash={}, captured_black={}, captured_white={}, player={:?}, depth={}, remaining_time={}", board.get_zobrist_hash(), captured_pieces.black.len(), captured_pieces.white.len(), player, depth, remaining_time).into());
            if let Some((move_, score)) = self.search_engine.search_at_depth(board, captured_pieces, player, depth, remaining_time) {
                best_move = Some(move_);
                best_score = score;
                
                // Early exit if we're clearly winning
                if score > 1000 && depth >= 3 {
                    console::log_1(&format!("IterativeDeepening::search: early exit due to high score").into());
                    break;
                }
            } else {
                // Search failed, use previous result
                console::log_1(&format!("IterativeDeepening::search: search_at_depth returned None, breaking").into());
                break;
            }
        }
        
        console::log_1(&format!("IterativeDeepening::search: finished search, best_move={:?}, best_score={}", best_move, best_score).into());
        best_move.map(|m| (m, best_score))
    }

    /// Get search statistics
    pub fn get_stats(&self) -> SearchStats {
        SearchStats {
            transposition_table_size: self.search_engine.transposition_table.len(),
            max_depth: self.max_depth,
        }
    }
}

/// Search statistics
pub struct SearchStats {
    pub transposition_table_size: usize,
    pub max_depth: u8,
}

/// Opening book integration
pub struct OpeningBook {
    openings: Vec<Opening>,
}

impl OpeningBook {
    pub fn new() -> Self {
        Self {
            openings: Vec::new(),
        }
    }

    /// Load opening book from file
    pub fn load_from_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This would load the opening book from a file
        // For now, we'll create a simple example
        self.openings.push(Opening {
            name: "Yagura".to_string(),
            moves: vec![
                "77-76".to_string(),
                "33-34".to_string(),
                "69-78".to_string(),
            ],
        });
        
        Ok(())
    }

    /// Find opening move for current position
    pub fn find_move(&self, board: &BitboardBoard, move_history: &[Move]) -> Option<Move> {
        // This would implement opening book lookup
        // For now, return None
        None
    }
}

/// Opening structure
struct Opening {
    name: String,
    moves: Vec<String>,
}
