use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};

pub mod bitboards;
pub mod moves;
pub mod evaluation;
pub mod search;
pub mod types;
pub mod opening_book;
pub mod time_utils;
pub mod debug_utils;

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
    opening_book: OpeningBook,
    stop_flag: Arc<AtomicBool>,
    search_engine: Arc<Mutex<SearchEngine>>,
    debug_mode: bool,
    pondering: bool,
    output_buffer: Arc<Mutex<Vec<String>>>,
}

#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        Self {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            current_player: Player::Black,
            opening_book: OpeningBook::new(),
            stop_flag: stop_flag.clone(),
            search_engine: Arc::new(Mutex::new(SearchEngine::new(Some(stop_flag), 16))),
            debug_mode: false,
            pondering: false,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    pub fn get_best_move_wasm(&mut self, difficulty: u8, time_limit_ms: u32) -> Option<Move> {
        self.get_best_move(difficulty, time_limit_ms, None, None)
    }

    pub fn get_board_state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.board.to_fen(self.current_player, &self.captured_pieces)).unwrap_or_else(|_| JsValue::NULL)
    }

    // Methods needed for WebAssembly integration
    pub fn set_position(&mut self, board_json: &str) {
        self.board = BitboardBoard::empty(); // Clear the board
        if let Ok(pieces) = serde_json::from_str::<Vec<PieceJson>>(board_json) {
            for piece_json in pieces {
                let player = if piece_json.player == "Black" { Player::Black } else { Player::White };
                if let Some(piece_type) = PieceType::from_str(&piece_json.piece_type) {
                    let pos = Position::new(piece_json.position.row, piece_json.position.col);
                    self.board.place_piece(Piece::new(piece_type, player), pos);
                }
            }
        }
    }

    pub fn set_current_player(&mut self, player: &str) {
        self.current_player = if player == "Black" { Player::Black } else { Player::White };
    }

    pub fn get_pending_output(&mut self) -> Vec<String> {
        if let Ok(mut buffer) = self.output_buffer.lock() {
            let output = buffer.clone();
            buffer.clear();
            output
        } else {
            Vec::new()
        }
    }

    pub fn to_string_for_debug(&self) -> String {
        let mut s = String::new();
        s.push_str("White (captured): ");
        for piece_type in &self.captured_pieces.white {
            s.push_str(&Piece::new(*piece_type, Player::White).to_fen_char());
            s.push(' ');
        }
        s.push('\n');

        s.push_str(&self.board.to_string_for_debug());

        s.push_str("Black (captured): ");
        for piece_type in &self.captured_pieces.black {
            s.push_str(&Piece::new(*piece_type, Player::Black).to_fen_char());
            s.push(' ');
        }
        s.push('\n');
        s.push_str(&format!("Current player: {:?}\n", self.current_player));
        s
    }
}

impl ShogiEngine {
    pub fn get_best_move(&mut self, difficulty: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>, output_buffer: Option<Arc<Mutex<Vec<String>>>>) -> Option<Move> {
        crate::debug_utils::debug_log("Starting get_best_move");
        
        if let Some(buffer) = &output_buffer {
            if let Ok(mut buf) = buffer.lock() {
                buf.push("info string DEBUG: Starting get_best_move".to_string());
            }
        }

        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        if let Some(book_move) = self.opening_book.get_move(&fen) {
            if let Some(buffer) = &output_buffer {
                if let Ok(mut buf) = buffer.lock() {
                    buf.push("info string DEBUG: Found opening book move".to_string());
                }
            }
            return Some(book_move);
        }

        if let Some(buffer) = &output_buffer {
            if let Ok(mut buf) = buffer.lock() {
                buf.push("info string DEBUG: No opening book move, starting search".to_string());
            }
        }

        let actual_difficulty = if difficulty == 0 { 1 } else { difficulty };
        crate::debug_utils::debug_log("Creating searcher");
        let mut searcher = search::IterativeDeepening::new(actual_difficulty, time_limit_ms, stop_flag, None);
        
        crate::debug_utils::debug_log("Trying to get search engine lock");
        
        if let Some(buffer) = &output_buffer {
            if let Ok(mut buf) = buffer.lock() {
                buf.push("info string DEBUG: Created searcher, trying to get search engine lock".to_string());
            }
        }
        
        // Try to get the search engine lock, but don't panic if it fails
        crate::debug_utils::debug_log("About to lock search engine");
        let search_result = self.search_engine.lock().map(|mut search_engine_guard| {
            crate::debug_utils::debug_log("Got search engine lock, starting search");
            if let Some(buffer) = &output_buffer {
                if let Ok(mut buf) = buffer.lock() {
                    buf.push("info string DEBUG: Got search engine lock, starting search".to_string());
                }
            }
            searcher.search(&mut search_engine_guard, &self.board, &self.captured_pieces, self.current_player)
        });
        
        crate::debug_utils::debug_log("Search completed, checking result");
        
        if let Some(buffer) = &output_buffer {
            if let Ok(mut buf) = buffer.lock() {
                buf.push("info string DEBUG: Search completed, checking result".to_string());
            }
        }
        
        if let Ok(Some((move_, _score))) = search_result {
            if let Some(buffer) = &output_buffer {
                if let Ok(mut buf) = buffer.lock() {
                    buf.push("info string DEBUG: Search found best move".to_string());
                }
            }
            Some(move_)
        } else {
            if let Some(buffer) = &output_buffer {
                if let Ok(mut buf) = buffer.lock() {
                    buf.push("info string DEBUG: Search failed, trying fallback random move".to_string());
                }
            }
            // Fallback to random move if search fails
            let move_generator = MoveGenerator::new();
            let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
            if legal_moves.is_empty() {
                if let Some(buffer) = &output_buffer {
                    if let Ok(mut buf) = buffer.lock() {
                        buf.push("info string DEBUG: No legal moves available".to_string());
                    }
                }
                return None;
            }
            // Use a seeded RNG that's WASM-compatible
            let mut rng = StdRng::seed_from_u64(42); // Fixed seed for deterministic behavior
            if let Some(buffer) = &output_buffer {
                if let Ok(mut buf) = buffer.lock() {
                    buf.push(format!("info string DEBUG: Found {} legal moves, choosing random", legal_moves.len()));
                }
            }
            legal_moves.choose(&mut rng).cloned()
        }
    }

    pub fn handle_position(&mut self, parts: &[&str]) -> Vec<String> {
        let mut output = Vec::new();
        let sfen_str: String;
        let mut moves_start_index: Option<usize> = None;

        if parts.is_empty() {
            output.push("info string error Invalid position command".to_string());
            return output;
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
            output.push("info string error Invalid position command: expected 'startpos' or 'sfen'".to_string());
            return output;
        }

        match BitboardBoard::from_fen(&sfen_str) {
            Ok((board, player, captured_pieces)) => {
                self.board = board;
                self.current_player = player;
                self.captured_pieces = captured_pieces;
            }
            Err(e) => {
                output.push(format!("info string error Failed to parse FEN: {}", e));
                return output;
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
                        output.push(format!("info string error Failed to parse move '{}': {}", move_str, e));
                        return output;
                    }
                }
            }
        }

        if self.debug_mode {
            // Send debug output to output buffer and immediate output for WASM compatibility
            let debug_output = format!("info string DEBUG: {}", self.to_string_for_debug());
            output.push("info string DEBUG: Board state:".to_string());
            output.push(debug_output.clone());
            
            if let Ok(mut buffer) = self.output_buffer.lock() {
                buffer.push("info string DEBUG: Board state:".to_string());
                buffer.push(debug_output);
            }
        }
        output.push("info string Board state updated.".to_string());
        output
    }

    pub fn handle_go(&mut self, parts: &[&str]) -> Vec<String> {
        // Immediate console logging for debugging
        crate::debug_utils::debug_log("Starting handle_go");
        
        let mut btime = 0;
        let mut wtime = 0;
        let mut byoyomi = 0;
        let mut is_ponder = false;

        // Add debug logging
        if let Ok(mut buffer) = self.output_buffer.lock() {
            buffer.push("info string DEBUG: Starting handle_go".to_string());
        }

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
                "ponder" => {
                    is_ponder = true;
                    i += 1;
                },
                _ => i += 1,
            }
        }

        self.pondering = is_ponder;

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

        if let Ok(mut buffer) = self.output_buffer.lock() {
            buffer.push(format!("info string DEBUG: Time to use: {}ms", time_to_use));
        }

        crate::debug_utils::debug_log(&format!("Time to use: {}ms", time_to_use));

        self.stop_flag.store(false, Ordering::Relaxed);
        
        // Clear the output buffer
        if let Ok(mut buffer) = self.output_buffer.lock() {
            buffer.clear();
            buffer.push("info string DEBUG: Buffer cleared, starting search".to_string());
        }

        crate::debug_utils::debug_log("About to start asynchronous search");

        if let Ok(mut buffer) = self.output_buffer.lock() {
            buffer.push("info string DEBUG: Starting asynchronous search".to_string());
        }

        // Start the search in a separate thread for standalone, or synchronously for WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            let stop_flag = self.stop_flag.clone();
            let output_buffer = self.output_buffer.clone();
            let board = self.board.clone();
            let captured_pieces = self.captured_pieces.clone();
            let current_player = self.current_player;
            
            std::thread::spawn(move || {
                let mut engine = ShogiEngine::new();
                engine.board = board;
                engine.captured_pieces = captured_pieces;
                engine.current_player = current_player;
                
                if let Some(mv) = engine.get_best_move(5, time_to_use, Some(stop_flag), Some(output_buffer.clone())) {
                    if let Ok(mut buffer) = output_buffer.lock() {
                        buffer.push(format!("bestmove {}", mv.to_usi_string()));
                        buffer.push("info string DEBUG: Best move found and sent".to_string());
                    }
                } else {
                    if let Ok(mut buffer) = output_buffer.lock() {
                        buffer.push("bestmove resign".to_string());
                        buffer.push("info string DEBUG: No move found, resigning".to_string());
                    }
                }
            });
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, we need to do the search synchronously since threads aren't supported
            let best_move = self.get_best_move(5, time_to_use, Some(self.stop_flag.clone()), Some(self.output_buffer.clone()));
            
            if let Ok(mut buffer) = self.output_buffer.lock() {
                if let Some(mv) = best_move {
                    buffer.push(format!("bestmove {}", mv.to_usi_string()));
                    buffer.push("info string DEBUG: Best move found and sent".to_string());
                } else {
                    buffer.push("bestmove resign".to_string());
                    buffer.push("info string DEBUG: No move found, resigning".to_string());
                }
            }
        }

        Vec::new()
    }

    pub fn handle_stop(&mut self) -> Vec<String> {
        self.stop_flag.store(true, Ordering::Relaxed);
        Vec::new()
    }

    pub fn handle_setoption(&mut self, parts: &[&str]) -> Vec<String> {
        if parts.len() >= 4 && parts[0] == "name" && parts[2] == "value" {
            if parts[1] == "USI_Hash" {
                if let Ok(size) = parts[3].parse::<usize>() {
                    if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                        *search_engine_guard = SearchEngine::new(Some(self.stop_flag.clone()), size);
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn handle_usinewgame(&mut self) -> Vec<String> {
        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
            search_engine_guard.clear();
        }
        Vec::new()
    }

    pub fn handle_debug(&mut self, parts: &[&str]) -> Vec<String> {
        let mut output = Vec::new();
        if let Some(part) = parts.get(0) {
            match *part {
                "on" => {
                    self.debug_mode = true;
                    output.push("info string debug mode enabled".to_string());
                },
                "off" => {
                    self.debug_mode = false;
                    output.push("info string debug mode disabled".to_string());
                },
                _ => output.push(format!("info string unknown debug command {}", part)),
            }
        } else {
            output.push("info string debug command needs an argument (on/off)".to_string());
        }
        output
    }

    pub fn handle_ponderhit(&mut self) -> Vec<String> {
        self.pondering = false;
        // The engine should switch from pondering to normal search.
        // For now, we just print an info string.
        vec!["info string ponderhit received".to_string()]
    }

    pub fn handle_gameover(&self, parts: &[&str]) -> Vec<String> {
        if let Some(result) = parts.get(0) {
            vec![format!("info string game over: {}", result)]
        } else {
            vec!["info string game over command received without a result".to_string()]
        }
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
            let mut searcher = search::IterativeDeepening::new(1, 1000, None, None);
            searcher.search(&mut search_engine_guard, &board, &captured_pieces, Player::Black);
        }

        engine.handle_usinewgame();

        let search_engine_guard = engine.search_engine.lock().unwrap();
        assert_eq!(search_engine_guard.transposition_table_len(), 0);
    }

    #[test]
    fn test_handle_debug() {
        let mut engine = ShogiEngine::new();
        assert!(!engine.debug_mode);
        engine.handle_debug(&["on"]);
        assert!(engine.debug_mode);
        engine.handle_debug(&["off"]);
        assert!(!engine.debug_mode);
    }

    #[test]
    fn test_handle_ponder() {
        let mut engine = ShogiEngine::new();
        assert!(!engine.pondering);
        engine.handle_go(&["ponder"]);
        assert!(engine.pondering);
        engine.handle_ponderhit();
        assert!(!engine.pondering);
    }
}

pub struct UsiHandler {
    engine: ShogiEngine,
}

impl UsiHandler {
    pub fn new() -> Self {
        Self {
            engine: ShogiEngine::new(),
        }
    }

    pub fn handle_command(&mut self, command_str: &str) -> Vec<String> {
        let parts: Vec<&str> = command_str.trim().split_whitespace().collect();

        if parts.is_empty() {
            return Vec::new();
        }

        if self.engine.is_debug_mode() {
            // TODO: Add proper logging instead of returning debug messages.
        }

        match parts[0] {
            "usi" => self.handle_usi(),
            "isready" => self.handle_isready(),
            "debug" => self.engine.handle_debug(&parts[1..]),
            "position" => self.engine.handle_position(&parts[1..]),
            "go" => self.engine.handle_go(&parts[1..]),
            "stop" => self.engine.handle_stop(),
            "ponderhit" => self.engine.handle_ponderhit(),
            "setoption" => self.engine.handle_setoption(&parts[1..]),
            "usinewgame" => self.engine.handle_usinewgame(),
            "gameover" => self.engine.handle_gameover(&parts[1..]),
            "quit" => Vec::new(), // quit is handled by the caller
            _ => vec![format!("info string Unknown command: {}", parts.join(" "))],
        }
    }

    pub fn get_pending_output(&mut self) -> Vec<String> {
        self.engine.get_pending_output()
    }

    fn handle_usi(&self) -> Vec<String> {
        vec![
            "id name Shogi Engine".to_string(),
            "id author Gemini".to_string(),
            "option name USI_Hash type spin default 16 min 1 max 1024".to_string(),
            "usiok".to_string(),
        ]
    }

    fn handle_isready(&self) -> Vec<String> {
        vec!["readyok".to_string()]
    }
}

#[wasm_bindgen]
pub struct WasmUsiHandler {
    handler: UsiHandler,
}

#[wasm_bindgen]
impl WasmUsiHandler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            handler: UsiHandler::new(),
        }
    }

    pub fn process_command(&mut self, command: &str) -> JsValue {
        let output = self.handler.handle_command(command);
        serde_wasm_bindgen::to_value(&output).unwrap_or_else(|_| JsValue::NULL)
    }

    pub fn get_pending_output(&mut self) -> JsValue {
        let output = self.handler.engine.get_pending_output();
        serde_wasm_bindgen::to_value(&output).unwrap_or_else(|_| JsValue::NULL)
    }
}

// Debug control functions
#[wasm_bindgen]
pub fn set_debug_enabled(enabled: bool) {
    debug_utils::set_debug_enabled(enabled);
}

#[wasm_bindgen]
pub fn is_debug_enabled() -> bool {
    debug_utils::is_debug_enabled()
}
