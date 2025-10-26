//! Move ordering integration with transposition table
//! 
//! This module provides enhanced move ordering that integrates with the
//! transposition table system to prioritize moves based on stored best moves,
//! hash table hints, and search history.
//! 
//! # Features
//! 
//! - **Transposition Table Integration**: Uses stored best moves from previous searches
//! - **MVV-LVA Ordering**: Captures ordered by Most Valuable Victim - Least Valuable Attacker
//! - **Killer Move Heuristic**: Prioritizes moves that caused beta cutoffs
//! - **History Heuristic**: Tracks successful quiet moves
//! - **Performance Statistics**: Detailed metrics on move ordering effectiveness
//! 
//! # Usage
//! 
//! ```rust
//! use shogi_engine::search::{TranspositionMoveOrderer, ThreadSafeTranspositionTable};
//! use shogi_engine::bitboards::BitboardBoard;
//! use shogi_engine::types::{Move, Player, CapturedPieces};
//! 
//! // Create move orderer
//! let mut orderer = TranspositionMoveOrderer::new();
//! 
//! // Set transposition table reference
//! let tt = ThreadSafeTranspositionTable::new(Default::default());
//! orderer.set_transposition_table(&tt);
//! 
//! // Order moves for a position
//! let board = BitboardBoard::new();
//! let captured = CapturedPieces::new();
//! let moves = vec![/* your moves */];
//! 
//! let ordered_moves = orderer.order_moves(
//!     &moves, &board, &captured, Player::Black, 
//!     3, // depth
//!     -1000, // alpha
//!     1000,  // beta
//!     None   // iid_move
//! );
//! 
//! // Get ordering statistics
//! let stats = orderer.get_move_ordering_hints(&moves, &board, &captured, Player::Black);
//! println!("TT hint moves: {}", stats.tt_hint_moves);
//! ```
//! 
//! # Move Ordering Heuristics
//! 
//! 1. **Transposition Table Best Moves**: Highest priority for stored best moves
//! 2. **Captures**: Ordered by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
//! 3. **Killer Moves**: Moves that caused beta cutoffs at the same depth
//! 4. **History Heuristic**: Quiet moves that have been successful in the past
//! 5. **Default Ordering**: Remaining moves in original order
//! 
//! # Performance Tips
//! 
//! - Update killer moves after beta cutoffs
//! - Update history scores for quiet moves
//! - Monitor TT hit rates for best move prioritization
//! - Consider move ordering statistics for tuning

use crate::types::*;
use crate::bitboards::*;
use crate::search::thread_safe_table::ThreadSafeTranspositionTable;
use crate::search::shogi_hash::*;
use std::collections::HashMap;

/// Enhanced move ordering system with transposition table integration
pub struct TranspositionMoveOrderer {
    /// Transposition table for accessing stored best moves
    transposition_table: *const ThreadSafeTranspositionTable,
    /// Hash calculator for position hashing
    pub hash_calculator: ShogiHashHandler,
    /// Move ordering statistics
    stats: MoveOrderingStats,
    /// History table for move ordering (from/to squares)
    history_table: [[i32; 81]; 81], // 9x9 board positions
    /// Killer moves for move ordering
    killer_moves: [Option<Move>; 2],
    /// Counter moves (move that refutes the previous move)
    counter_moves: HashMap<Move, Move>,
}

/// Statistics for move ordering performance
#[derive(Debug, Clone, Default)]
pub struct MoveOrderingStats {
    pub total_moves_ordered: u64,
    pub tt_hint_moves: u64,
    pub best_move_hits: u64,
    pub killer_move_hits: u64,
    pub counter_move_hits: u64,
    pub history_hits: u64,
    pub ordering_time_ms: f64,
}

/// Move ordering hints from transposition table
#[derive(Debug, Clone)]
pub struct MoveOrderingHints {
    /// Best move from transposition table
    pub best_move: Option<Move>,
    /// Hash key for the position
    pub position_hash: u64,
    /// Depth of the transposition table entry
    pub tt_depth: u8,
    /// Score from transposition table
    pub tt_score: Option<i32>,
    /// Flag from transposition table
    pub tt_flag: Option<TranspositionFlag>,
}

impl TranspositionMoveOrderer {
    /// Get current time
    fn get_current_time() -> std::time::Instant {
        std::time::Instant::now()
    }

    /// Create a new move orderer
    pub fn new() -> Self {
        Self {
            transposition_table: std::ptr::null(),
            hash_calculator: ShogiHashHandler::new(1000),
            stats: MoveOrderingStats::default(),
            history_table: [[0; 81]; 81],
            killer_moves: [None, None],
            counter_moves: HashMap::new(),
        }
    }

    /// Set the transposition table reference
    pub fn set_transposition_table(&mut self, tt: &ThreadSafeTranspositionTable) {
        self.transposition_table = tt as *const ThreadSafeTranspositionTable;
    }

    /// Order moves with transposition table integration
    pub fn order_moves(
        &mut self,
        moves: &[Move],
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
        _alpha: i32,
        _beta: i32,
        iid_move: Option<&Move>,
    ) -> Vec<Move> {
        let start_time = Self::get_current_time();
        
        // Get move ordering hints from transposition table
        let hints = self.get_move_ordering_hints(board, captured_pieces, player, depth);
        
        // Categorize moves based on type and priority
        let mut categorized_moves = self.categorize_moves(moves, board, &hints, iid_move);
        
        // Order each category
        self.order_captures(&mut categorized_moves.captures, board, &hints);
        self.order_killers(&mut categorized_moves.killers, board, &hints);
        self.order_quiet_moves(&mut categorized_moves.quiet_moves, board, &hints);
        self.order_other_moves(&mut categorized_moves.other_moves, board, &hints);
        
        // Combine ordered moves
        let mut ordered_moves = Vec::new();
        
        // 1. Best move from transposition table (highest priority)
        if let Some(best_move) = hints.best_move {
            if let Some(pos) = moves.iter().position(|m| self.moves_equal(m, &best_move)) {
                ordered_moves.push(moves[pos].clone());
                self.stats.best_move_hits += 1;
            }
        }
        
        // 2. Captures (MVV-LVA order)
        ordered_moves.extend(categorized_moves.captures);
        
        // 3. Killer moves
        ordered_moves.extend(categorized_moves.killers);
        
        // 4. Quiet moves (history heuristic)
        ordered_moves.extend(categorized_moves.quiet_moves);
        
        // 5. Other moves
        ordered_moves.extend(categorized_moves.other_moves);
        
        // Update statistics
        self.stats.total_moves_ordered += moves.len() as u64;
        
        // Calculate timing (WASM-compatible)
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.stats.ordering_time_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;
        }
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, we can't measure time precisely, so we use a dummy value
            self.stats.ordering_time_ms = 0.0;
        }
        
        ordered_moves
    }

    /// Get move ordering hints from transposition table
    pub fn get_move_ordering_hints(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
        depth: u8,
    ) -> MoveOrderingHints {
        let position_hash = self.hash_calculator.get_position_hash(board, player, captured_pieces);
        
        // Probe transposition table for best move and other hints
        let (best_move, tt_depth, tt_score, tt_flag) = if !self.transposition_table.is_null() {
            unsafe {
                let tt = &*self.transposition_table;
                if let Some(entry) = tt.probe(position_hash, depth) {
                    self.stats.tt_hint_moves += 1;
                    (entry.best_move, entry.depth, Some(entry.score), Some(entry.flag))
                } else {
                    (None, 0, None, None)
                }
            }
        } else {
            (None, 0, None, None)
        };

        MoveOrderingHints {
            best_move,
            position_hash,
            tt_depth,
            tt_score,
            tt_flag,
        }
    }

    /// Categorize moves based on type and priority
    fn categorize_moves(
        &self,
        moves: &[Move],
        _board: &BitboardBoard,
        hints: &MoveOrderingHints,
        iid_move: Option<&Move>,
    ) -> CategorizedMoves {
        let mut categorized = CategorizedMoves::default();
        
        for mv in moves {
            // Skip best move from TT (will be added first)
            if let Some(ref best_move) = hints.best_move {
                if self.moves_equal(mv, best_move) {
                    continue;
                }
            }
            
            // Skip IID move (will be added first)
            if let Some(iid_mv) = iid_move {
                if self.moves_equal(mv, iid_mv) {
                    continue;
                }
            }
            
            // Categorize based on move type
            if mv.is_capture {
                categorized.captures.push(mv.clone());
            } else if self.is_killer_move(mv) {
                categorized.killers.push(mv.clone());
            } else if self.is_quiet_move(mv) {
                categorized.quiet_moves.push(mv.clone());
            } else {
                categorized.other_moves.push(mv.clone());
            }
        }
        
        categorized
    }

    /// Order captures using MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    fn order_captures(&self, captures: &mut Vec<Move>, _board: &BitboardBoard, _hints: &MoveOrderingHints) {
        captures.sort_by(|a, b| {
            let a_mvv_lva = self.calculate_mvv_lva(a);
            let b_mvv_lva = self.calculate_mvv_lva(b);
            b_mvv_lva.cmp(&a_mvv_lva)
        });
    }

    /// Order killer moves
    fn order_killers(&mut self, killers: &mut Vec<Move>, _board: &BitboardBoard, _hints: &MoveOrderingHints) {
        killers.sort_by(|a, b| {
            let a_killer_score = self.get_killer_score(a);
            let b_killer_score = self.get_killer_score(b);
            b_killer_score.cmp(&a_killer_score)
        });
    }

    /// Order quiet moves using history heuristic
    fn order_quiet_moves(&mut self, quiet_moves: &mut Vec<Move>, _board: &BitboardBoard, _hints: &MoveOrderingHints) {
        quiet_moves.sort_by(|a, b| {
            let a_history = self.get_history_score(a);
            let b_history = self.get_history_score(b);
            b_history.cmp(&a_history)
        });
    }

    /// Order other moves
    fn order_other_moves(&mut self, other_moves: &mut Vec<Move>, _board: &BitboardBoard, _hints: &MoveOrderingHints) {
        // For other moves, use a combination of factors
        other_moves.sort_by(|a, b| {
            let a_score = self.score_move_general(a);
            let b_score = self.score_move_general(b);
            b_score.cmp(&a_score)
        });
    }

    /// Calculate MVV-LVA score for a capture move
    fn calculate_mvv_lva(&self, mv: &Move) -> i32 {
        if !mv.is_capture {
            return 0;
        }
        
        let victim_value = if let Some(captured) = &mv.captured_piece {
            captured.piece_type.base_value()
        } else {
            0
        };
        
        let attacker_value = mv.piece_type.base_value();
        
        // MVV-LVA: Most Valuable Victim - Least Valuable Attacker
        // Higher values are better (more valuable captures first)
        victim_value * 10 - attacker_value
    }

    /// Check if a move is a killer move
    fn is_killer_move(&self, mv: &Move) -> bool {
        self.killer_moves.iter().any(|killer| {
            if let Some(k) = killer {
                self.moves_equal(mv, k)
            } else {
                false
            }
        })
    }

    /// Get killer move score
    fn get_killer_score(&mut self, mv: &Move) -> i32 {
        for (i, killer) in self.killer_moves.iter().enumerate() {
            if let Some(k) = killer {
                if self.moves_equal(mv, k) {
                    self.stats.killer_move_hits += 1;
                    return 1000 - (i as i32 * 100); // First killer gets higher score
                }
            }
        }
        0
    }

    /// Check if a move is quiet (not capture, not promotion, not check)
    fn is_quiet_move(&self, mv: &Move) -> bool {
        !mv.is_capture && !mv.is_promotion && !mv.gives_check
    }

    /// Get history score for a move
    fn get_history_score(&mut self, mv: &Move) -> i32 {
        if let (Some(from), Some(to)) = (mv.from, Some(mv.to)) {
            let from_idx = (from.row * 9 + from.col) as usize;
            let to_idx = (to.row * 9 + to.col) as usize;
            if from_idx < 81 && to_idx < 81 {
                self.stats.history_hits += 1;
                return self.history_table[from_idx][to_idx];
            }
        }
        0
    }

    /// General move scoring for other moves
    fn score_move_general(&mut self, mv: &Move) -> i32 {
        let mut score = 0;
        
        // Promotion bonus
        if mv.is_promotion {
            score += 800;
        }
        
        // Center control bonus
        if mv.to.row >= 3 && mv.to.row <= 5 && mv.to.col >= 3 && mv.to.col <= 5 {
            score += 20;
        }
        
        // Piece value bonus
        score += mv.piece_type.base_value() / 10;
        
        // Counter move bonus
        if let Some(counter) = self.counter_moves.get(mv) {
            if self.moves_equal(mv, counter) {
                self.stats.counter_move_hits += 1;
                score += 300;
            }
        }
        
        score
    }

    /// Check if two moves are equal
    pub fn moves_equal(&self, move1: &Move, move2: &Move) -> bool {
        move1.from == move2.from 
            && move1.to == move2.to 
            && move1.piece_type == move2.piece_type
            && move1.is_promotion == move2.is_promotion
    }

    /// Update killer moves
    pub fn update_killer_moves(&mut self, new_killer: Move) {
        // Don't update if it's already a killer move
        if self.is_killer_move(&new_killer) {
            return;
        }
        
        // Shift killer moves and add new one at position 0
        self.killer_moves[1] = self.killer_moves[0].take();
        self.killer_moves[0] = Some(new_killer);
    }

    /// Update history table
    pub fn update_history(&mut self, mv: &Move, depth: u8) {
        if let (Some(from), Some(to)) = (mv.from, Some(mv.to)) {
            let from_idx = (from.row * 9 + from.col) as usize;
            let to_idx = (to.row * 9 + to.col) as usize;
            if from_idx < 81 && to_idx < 81 {
                // Increase history score based on depth
                self.history_table[from_idx][to_idx] += depth as i32 * depth as i32;
                
                // Prevent overflow
                if self.history_table[from_idx][to_idx] > 10000 {
                    self.history_table[from_idx][to_idx] = 10000;
                }
            }
        }
    }

    /// Update counter move
    pub fn update_counter_move(&mut self, move_: &Move, counter: Move) {
        self.counter_moves.insert(move_.clone(), counter);
    }

    /// Clear all move ordering data
    pub fn clear(&mut self) {
        self.history_table = [[0; 81]; 81];
        self.killer_moves = [None, None];
        self.counter_moves.clear();
        self.stats = MoveOrderingStats::default();
    }

    /// Get move ordering statistics
    pub fn get_stats(&self) -> &MoveOrderingStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = MoveOrderingStats::default();
    }
}

/// Categorized moves for ordering
#[derive(Debug, Default)]
struct CategorizedMoves {
    captures: Vec<Move>,
    killers: Vec<Move>,
    quiet_moves: Vec<Move>,
    other_moves: Vec<Move>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_orderer_creation() {
        let orderer = TranspositionMoveOrderer::new();
        assert_eq!(orderer.stats.total_moves_ordered, 0);
    }

    #[test]
    fn test_move_categorization() {
        let mut orderer = TranspositionMoveOrderer::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();
        
        let moves = vec![
            Move {
                from: Some(Position { row: 7, col: 4 }),
                to: Position { row: 6, col: 4 },
                piece_type: PieceType::Pawn,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
                player: Player::Black,
            },
        ];
        
        let hints = MoveOrderingHints {
            best_move: None,
            position_hash: 0,
            tt_depth: 0,
            tt_score: None,
            tt_flag: None,
        };
        
        let categorized = orderer.categorize_moves(&moves, &board, &hints, None);
        assert_eq!(categorized.quiet_moves.len(), 1);
    }

    #[test]
    fn test_killer_move_management() {
        let mut orderer = TranspositionMoveOrderer::new();
        
        let killer_move = Move {
            from: Some(Position { row: 7, col: 4 }),
            to: Position { row: 6, col: 4 },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        };
        
        orderer.update_killer_moves(killer_move.clone());
        assert!(orderer.is_killer_move(&killer_move));
    }

    #[test]
    fn test_history_updates() {
        let mut orderer = TranspositionMoveOrderer::new();
        
        let mv = Move {
            from: Some(Position { row: 7, col: 4 }),
            to: Position { row: 6, col: 4 },
            piece_type: PieceType::Pawn,
            is_capture: false,
            is_promotion: false,
            gives_check: false,
            is_recapture: false,
            captured_piece: None,
            player: Player::Black,
        };
        
        orderer.update_history(&mv, 3);
        let score = orderer.get_history_score(&mv);
        assert!(score > 0);
    }
}
