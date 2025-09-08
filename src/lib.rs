use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::thread;

pub mod bitboards;
pub mod moves;
pub mod evaluation;
pub mod search;
pub mod types;
pub mod opening_book;

use bitboards::*;
use moves::*;
use search::SearchEngine;
use types::*;
use opening_book::OpeningBook;

#[derive(Serialize, Deserialize)]
struct PieceJson {
    position: PositionJson,
    piece_type: String,
    player: String,
}

#[derive(Serialize, Deserialize)]
struct PositionJson {
    row: u8,
    col: u8,
}

#[derive(Serialize, Deserialize)]
struct CapturedPieceJson {
    piece_type: String,
    player: String,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ShogiEngine {
    board: BitboardBoard,
    captured_pieces: CapturedPieces,
    current_player: Player,
    move_history: Vec<Move>,
    opening_book: OpeningBook,
    stop_flag: Arc<AtomicBool>,
    search_engine: Arc<Mutex<SearchEngine>>,
}

#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        Self {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            current_player: Player::Black,
            move_history: Vec::new(),
            opening_book: OpeningBook::new(),
            stop_flag: stop_flag.clone(),
            search_engine: Arc::new(Mutex::new(SearchEngine::new(Some(stop_flag), 16))),
        }
    }

    pub fn get_best_move_wasm(&mut self, difficulty: u8, time_limit_ms: u32) -> Option<Move> {
        self.get_best_move(difficulty, time_limit_ms, None)
    }

    pub fn get_board_state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.board.to_fen(self.current_player, &self.captured_pieces)).unwrap()
    }

    // Methods needed for WebAssembly integration
    pub fn set_position(&mut self, board_json: &str) {
        self.board = BitboardBoard::empty(); // Clear the board
        let pieces: Vec<PieceJson> = serde_json::from_str(board_json).unwrap();
        for piece_json in pieces {
            let player = if piece_json.player == "Black" { Player::Black } else { Player::White };
            let piece_type = PieceType::from_str(&piece_json.piece_type).unwrap();
            let pos = Position::new(piece_json.position.row, piece_json.position.col);
            self.board.place_piece(Piece::new(piece_type, player), pos);
        }
    }

    pub fn set_current_player(&mut self, player: &str) {
        self.current_player = if player == "Black" { Player::Black } else { Player::White };
    }
}

impl ShogiEngine {
    pub fn get_best_move(&mut self, difficulty: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>) -> Option<Move> {
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        if let Some(book_move) = self.opening_book.get_move(&fen) {
            return Some(book_move);
        }

        let actual_difficulty = if difficulty == 0 { 1 } else { difficulty };
        let mut searcher = search::IterativeDeepening::new(actual_difficulty, time_limit_ms, stop_flag);
        let mut search_engine_guard = self.search_engine.lock().unwrap();
        if let Some((move_, _score)) = searcher.search(&mut search_engine_guard, &self.board, &self.captured_pieces, self.current_player) {
            Some(move_)
        } else {
            // Fallback to random move if search fails
            let move_generator = MoveGenerator::new();
            let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
            if legal_moves.is_empty() {
                return None;
            }
            let mut rng = rand::thread_rng();
            legal_moves.choose(&mut rng).cloned()
        }
    }

    pub fn handle_position(&mut self, parts: &[&str]) {
        let sfen_str: String;
        let mut moves_start_index: Option<usize> = None;

        if parts.is_empty() {
            println!("info string error Invalid position command");
            return;
        }

        if parts[0] == "startpos" {
            sfen_str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
            if parts.len() > 1 && parts[1] == "moves" {
                moves_start_index = Some(2);
            }
        } else if parts[0] == "sfen" {
            // sfen can be up to 4 parts, plus "moves"
            let mut sfen_parts = Vec::new();
            let mut current_index = 1;
            while current_index < parts.len() && parts[current_index] != "moves" {
                sfen_parts.push(parts[current_index]);
                current_index += 1;
            }
            sfen_str = sfen_parts.join(" ");
            if current_index < parts.len() && parts[current_index] == "moves" {
                moves_start_index = Some(current_index + 1);
            }
        } else {
            println!("info string error Invalid position command: expected 'startpos' or 'sfen'");
            return;
        }

        match BitboardBoard::from_fen(&sfen_str) {
            Ok((board, player, captured_pieces)) => {
                self.board = board;
                self.current_player = player;
                self.captured_pieces = captured_pieces;
            }
            Err(e) => {
                println!("info string error Failed to parse FEN: {}", e);
                return;
            }
        }

        if let Some(start_index) = moves_start_index {
            for move_str in &parts[start_index..] {
                match Move::from_usi_string(move_str, self.current_player, &self.board) {
                    Ok(mv) => {
                        if let Some(captured) = self.board.make_move(&mv) {
                            self.captured_pieces.add_piece(captured.piece_type, self.current_player);
                        }
                        self.current_player = self.current_player.opposite();
                    }
                    Err(e) => {
                        println!("info string error Failed to parse move '{}': {}", move_str, e);
                        return;
                    }
                }
            }
        }
        println!("info string Board state updated.");
    }

    pub fn handle_go(&mut self, parts: &[&str]) {
        let mut btime = 0;
        let mut wtime = 0;
        let mut byoyomi = 0;

        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "btime" => {
                    if i + 1 < parts.len() {
                        btime = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    } else { i += 1; }
                },
                "wtime" => {
                    if i + 1 < parts.len() {
                        wtime = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    } else { i += 1; }
                },
                "byoyomi" => {
                    if i + 1 < parts.len() {
                        byoyomi = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    } else { i += 1; }
                },
                _ => i += 1,
            }
        }

        let time_to_use = if byoyomi > 0 {
            byoyomi
        } else {
            let time_for_player = if self.current_player == Player::Black { btime } else { wtime };
            if time_for_player > 0 {
                time_for_player / 40
            } else {
                5000 // Default to 5 seconds if no time control is given
            }
        };

        self.stop_flag.store(false, Ordering::Relaxed);
        let mut engine_clone = self.clone();

        thread::spawn(move || {
            let best_move = engine_clone.get_best_move(5, time_to_use, Some(engine_clone.stop_flag.clone()));
            if let Some(mv) = best_move {
                println!("bestmove {}", mv.to_usi_string());
            } else {
                println!("bestmove resign");
            }
        });
    }

    pub fn handle_stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn handle_setoption(&mut self, parts: &[&str]) {
        if parts.len() >= 4 && parts[0] == "name" && parts[2] == "value" {
            if parts[1] == "USI_Hash" {
                if let Ok(size) = parts[3].parse::<usize>() {
                    let mut search_engine_guard = self.search_engine.lock().unwrap();
                    *search_engine_guard = SearchEngine::new(Some(self.stop_flag.clone()), size);
                }
            }
        }
    }

    pub fn handle_usinewgame(&mut self) {
        let mut search_engine_guard = self.search_engine.lock().unwrap();
        search_engine_guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_setoption_hash() {
        let mut engine = ShogiEngine::new();
        let parts = vec!["name", "USI_Hash", "value", "32"];
        engine.handle_setoption(&parts);
        let search_engine = engine.search_engine.lock().unwrap();
        const BYTES_PER_ENTRY: usize = 100;
        let expected_capacity = 32 * 1024 * 1024 / BYTES_PER_ENTRY;
        assert!(search_engine.transposition_table_capacity() >= expected_capacity);
    }

    #[test]
    fn test_handle_usinewgame() {
        let mut engine = ShogiEngine::new();
        
        // Simulate a search to populate the transposition table
        {
            let mut search_engine_guard = engine.search_engine.lock().unwrap();
            let board = BitboardBoard::new();
            let captured_pieces = CapturedPieces::new();
            let mut searcher = search::IterativeDeepening::new(1, 1000, None);
            searcher.search(&mut search_engine_guard, &board, &captured_pieces, Player::Black);
        }

        engine.handle_usinewgame();

        let search_engine_guard = engine.search_engine.lock().unwrap();
        assert_eq!(search_engine_guard.transposition_table_len(), 0);
    }
}