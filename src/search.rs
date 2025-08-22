use crate::types::*;
use crate::bitboards::*;
use crate::evaluation::*;
use crate::moves::*;
use std::collections::HashMap;

fn now() -> f64 {
    web_sys::window().expect("should have a window in this context").performance().expect("performance should be available").now()
}



pub struct SearchEngine {
    evaluator: PositionEvaluator,
    move_generator: MoveGenerator,
    transposition_table: HashMap<String, TranspositionEntry>,
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

    pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32) -> Option<(Move, i32)> {
        let start_time = now();
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        
        let mut best_move = None;
        let mut best_score = i32::MIN;
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            return None;
        }
        
        let sorted_moves = self.sort_moves(&legal_moves, board);
        
        let mut history: Vec<String> = vec![board.to_fen(player, captured_pieces)];

        for move_ in sorted_moves {
            if (now() - start_time) > time_limit_ms as f64 { break; }
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            
            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }
            
            let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms, &mut history);
            
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

    fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: f64, time_limit_ms: u32, history: &mut Vec<String>) -> i32 {
        let fen_key = board.to_fen(player, captured_pieces);
        if history.contains(&fen_key) {
            return 0; // Repetition is a draw
        }

        if (now() - start_time) > time_limit_ms as f64 { return 0; }

        if let Some(entry) = self.transposition_table.get(&fen_key) {
            if entry.depth >= depth {
                match entry.flag {
                    TranspositionFlag::Exact => return entry.score,
                    TranspositionFlag::LowerBound => if entry.score >= beta { return entry.score; },
                    TranspositionFlag::UpperBound => if entry.score <= alpha { return entry.score; },
                }
            }
        }
        
        if depth == 0 {
            return self.quiescence_search(board, captured_pieces, player, alpha, beta, start_time, time_limit_ms, 5);
        }
        
        let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
        if legal_moves.is_empty() {
            return if board.is_king_in_check(player, captured_pieces) { i32::MIN + 1 } else { 0 };
        }
        
        let sorted_moves = self.sort_moves(&legal_moves, board);
        let mut best_score = i32::MIN;
        
        history.push(fen_key.clone());

        for move_ in sorted_moves {
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();

            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }

            let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms, history);

            if score > best_score {
                best_score = score;
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
        self.transposition_table.insert(fen_key, TranspositionEntry { score: best_score, depth, flag, best_move: None });
        
        best_score
    }

    fn quiescence_search(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: f64, time_limit_ms: u32, depth: u8) -> i32 {
        if (now() - start_time) > time_limit_ms as f64 { return 0; }

        if depth == 0 {
            return self.evaluator.evaluate(board, player, captured_pieces);
        }
        
        let stand_pat = self.evaluator.evaluate(board, player, captured_pieces);
        if stand_pat >= beta { return beta; }
        if alpha < stand_pat { alpha = stand_pat; }
        
        let noisy_moves = self.generate_noisy_moves(board, player, captured_pieces);
        let sorted_noisy_moves = self.sort_moves(&noisy_moves, board);

        for move_ in sorted_noisy_moves {
            if (now() - start_time) > time_limit_ms as f64 { break; }
            
            let mut new_board = board.clone();
            let mut new_captured = captured_pieces.clone();
            if let Some(captured) = new_board.make_move(&move_) {
                new_captured.add_piece(captured.piece_type, player);
            }
            
            let score = -self.quiescence_search(&new_board, &new_captured, player.opposite(), -beta, -alpha, start_time, time_limit_ms, depth - 1);
            
            if score >= beta { return beta; }
            if score > alpha { alpha = score; }
        }
        
        alpha
    }

    fn generate_noisy_moves(&self, board: &BitboardBoard, player: Player, _captured_pieces: &CapturedPieces) -> Vec<Move> {
        self.move_generator.generate_legal_captures(board, player, _captured_pieces)
    }

    fn is_checking_move(&self, board: &BitboardBoard, move_: &Move, captured_pieces: &CapturedPieces) -> bool {
        let mut new_board = board.clone();
        new_board.make_move(move_);
        new_board.is_king_in_check(move_.player.opposite(), captured_pieces)
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
}


pub struct IterativeDeepening {
    search_engine: SearchEngine,
    max_depth: u8,
    time_limit_ms: u32,
}

impl IterativeDeepening {
    pub fn new(max_depth: u8, time_limit_ms: u32) -> Self {
        Self {
            search_engine: SearchEngine::new(),
            max_depth,
            time_limit_ms,
        }
    }

    pub fn search(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> Option<(Move, i32)> {
        let start_time = now();
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let search_time_limit = self.time_limit_ms.saturating_sub(100);

        for depth in 1..=self.max_depth {
            if (now() - start_time) >= search_time_limit as f64 { break; }
            let remaining_time = (search_time_limit as f64 - (now() - start_time)) as u32;

            if let Some((move_, score)) = self.search_engine.search_at_depth(board, captured_pieces, player, depth, remaining_time) {
                best_move = Some(move_);
                best_score = score;
                if score > 10000 && depth >= 3 { break; } 
            } else {
                break;
            }
        }
        best_move.map(|m| (m, best_score))
    }
}