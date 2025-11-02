
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
pub mod opening_book_converter;
pub mod time_utils;
pub mod debug_utils;
pub mod tuning;
pub mod weights;
pub mod tablebase;
pub mod kif_parser;

// Advanced alpha-beta pruning tests
// Note: Comprehensive tests are implemented in the core functionality
// The advanced pruning features are tested through integration with the search engine

// Advanced evaluation modules
pub mod king_safety {
    pub use crate::evaluation::king_safety::*;
}
pub mod castles {
    pub use crate::evaluation::castles::*;
}
pub mod attacks {
    pub use crate::evaluation::attacks::*;
}
pub mod patterns {
    pub use crate::evaluation::patterns::*;
}

pub mod usi;

use moves::*;
use search::search_engine::SearchEngine;
use types::*;
use opening_book::OpeningBook;
use tablebase::MicroTablebase;

// Re-export BitboardBoard for external use
pub use bitboards::BitboardBoard;

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

#[derive(Clone)]
pub struct ShogiEngine {
    board: BitboardBoard,
    captured_pieces: CapturedPieces,
    current_player: Player,
    opening_book: OpeningBook,
    tablebase: MicroTablebase,
    stop_flag: Arc<AtomicBool>,
    search_engine: Arc<Mutex<SearchEngine>>,
    debug_mode: bool,
    pondering: bool,
    depth: u8,
    thread_count: usize,
}

impl ShogiEngine {
    pub fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_count = num_cpus::get();
        let mut engine = Self {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            current_player: Player::Black,
            opening_book: OpeningBook::new(),
            tablebase: MicroTablebase::new(),
            stop_flag: stop_flag.clone(),
            search_engine: Arc::new(Mutex::new(SearchEngine::new(Some(stop_flag), 16))),
            debug_mode: true,
            pondering: false,
            depth: 0, // Default to 0 (unlimited/adaptive), like YaneuraOu
            thread_count,
        };
        
        // Try to load persisted preferences (thread count)
        engine.load_prefs();
        // Try to load default opening book if available
        engine.load_default_opening_book();
        
        engine
    }

    fn prefs_path() -> std::path::PathBuf {
        if let Ok(dir) = std::env::var("SHOGI_PREFS_DIR") {
            let p = std::path::PathBuf::from(dir);
            let _ = std::fs::create_dir_all(&p);
            return p.join("engine_prefs.json");
        }
        let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let dir = base.join("shogi-vibe");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("engine_prefs.json")
    }

    fn load_prefs(&mut self) {
        let path = Self::prefs_path();
        if let Ok(data) = std::fs::read(&path) {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&data) {
                if let Some(tc) = json.get("thread_count").and_then(|v| v.as_u64()) {
                    self.thread_count = (tc as usize).clamp(1, 32);
                }
            }
        }
    }

    fn save_prefs(&self) {
        let path = Self::prefs_path();
        let obj = serde_json::json!({
            "thread_count": self.thread_count
        });
        let _ = std::fs::write(path, serde_json::to_vec_pretty(&obj).unwrap_or_default());
    }

    /// Load default opening book from embedded data
    fn load_default_opening_book(&mut self) {
        // Try to load from embedded JSON data first
        let json_data = include_str!("ai/openingBook.json");
        if self.load_opening_book_from_json(json_data).is_ok() {
            crate::debug_utils::debug_log("Loaded default opening book from JSON");
            return;
        }
        
        // If JSON loading fails, try to load from binary if available
        // This would be implemented when binary opening books are generated
        crate::debug_utils::debug_log("No default opening book available");
    }

    /// Load opening book from binary data
    pub fn load_opening_book_from_binary(&mut self, data: &[u8]) -> Result<(), String> {
        self.opening_book.load_from_binary(data)
            .map_err(|e| format!("Failed to load opening book: {:?}", e))
    }

    /// Load opening book from JSON data
    pub fn load_opening_book_from_json(&mut self, json_data: &str) -> Result<(), String> {
        self.opening_book.load_from_json(json_data)
            .map_err(|e| format!("Failed to load opening book: {:?}", e))
    }

    /// Check if opening book is loaded
    pub fn is_opening_book_loaded(&self) -> bool {
        self.opening_book.is_loaded()
    }

    /// Get opening book statistics
    pub fn get_opening_book_stats(&self) -> String {
        let stats = self.opening_book.get_stats();
        format!(
            "Positions: {}, Moves: {}, Version: {}, Loaded: {}",
            stats.position_count, stats.move_count, stats.version, self.opening_book.is_loaded()
        )
    }

    /// Get detailed opening book information
    pub fn get_opening_book_info(&mut self) -> String {
        if !self.opening_book.is_loaded() {
            return "Opening book not loaded".to_string();
        }
        
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        let available_moves = self.opening_book.get_moves(&fen);
        let stats = self.opening_book.get_stats();
        
        let mut info = format!(
            "Opening Book Info:\n\
            - Positions: {}\n\
            - Total Moves: {}\n\
            - Version: {}\n\
            - Current Position: {}\n",
            stats.position_count, stats.move_count, stats.version, fen
        );
        
        if let Some(moves) = available_moves {
            info.push_str(&format!("- Available Moves: {}\n", moves.len()));
            for (i, book_move) in moves.iter().enumerate().take(3) {
                info.push_str(&format!(
                    "  {}. {} (weight: {}, eval: {})\n",
                    i + 1,
                    book_move.move_notation.as_ref().unwrap_or(&"N/A".to_string()),
                    book_move.weight,
                    book_move.evaluation
                ));
            }
            if moves.len() > 3 {
                info.push_str(&format!("  ... and {} more moves\n", moves.len() - 3));
            }
        } else {
            info.push_str("- No moves available in opening book\n");
        }
        
        info
    }

    /// Get opening book move for current position with detailed info
    pub fn get_opening_book_move_info(&mut self) -> Option<String> {
        if !self.opening_book.is_loaded() {
            return None;
        }
        
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        if let Some(book_moves) = self.opening_book.get_moves(&fen) {
            if let Some(best_book_move) = book_moves.iter().max_by(|a, b| a.weight.cmp(&b.weight)) {
                Some(format!(
                    "Opening book move: {} (weight: {}, eval: {}, opening: {})",
                    best_book_move.move_notation.as_ref().unwrap_or(&"N/A".to_string()),
                    best_book_move.weight,
                    best_book_move.evaluation,
                    best_book_move.opening_name.as_ref().unwrap_or(&"Unknown".to_string())
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a random opening book move for variety
    pub fn get_random_opening_book_move(&mut self) -> Option<Move> {
        if !self.opening_book.is_loaded() {
            return None;
        }
        
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        self.opening_book.get_random_move(&fen)
    }

    /// Get all available opening book moves for current position
    pub fn get_all_opening_book_moves(&mut self) -> Vec<String> {
        if !self.opening_book.is_loaded() {
            return vec!["Opening book not loaded".to_string()];
        }
        
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        if let Some(moves) = self.opening_book.get_moves(&fen) {
            moves.iter().enumerate().map(|(i, book_move)| {
                format!(
                    "{}. {} (weight: {}, eval: {}, opening: {})",
                    i + 1,
                    book_move.move_notation.as_ref().unwrap_or(&"N/A".to_string()),
                    book_move.weight,
                    book_move.evaluation,
                    book_move.opening_name.as_ref().unwrap_or(&"Unknown".to_string())
                )
            }).collect()
        } else {
            vec!["No moves available in opening book".to_string()]
        }
    }

    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
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

    pub fn set_depth(&mut self, depth: u8) {
        // Allow depth 0 (unlimited/adaptive) - engine will decide based on time
        self.depth = depth;
        crate::debug_utils::debug_log(&format!("Set depth to: {} (0 = unlimited)", depth));
        eprintln!("DEBUG: Set depth to: {} (0 = unlimited)", depth);
    }
    
    /// Set max depth (allows 0 for unlimited)
    pub fn set_max_depth(&mut self, depth: u8) {
        self.depth = depth;
        crate::debug_utils::debug_log(&format!("Set max depth to: {} (0 = unlimited)", depth));
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

    pub fn get_fen(&self) -> String {
        self.board.to_fen(self.current_player, &self.captured_pieces)
    }

    pub fn current_player(&self) -> Player {
        self.current_player
    }
}

impl ShogiEngine {
    /// Enable or disable debug logging
    pub fn set_debug_enabled(&self, enabled: bool) {
        crate::debug_utils::set_debug_enabled(enabled);
    }
    
    /// Check if debug logging is enabled
    pub fn is_debug_enabled(&self) -> bool {
        crate::debug_utils::is_debug_enabled()
    }

    pub fn get_best_move(&mut self, depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>) -> Option<Move> {
        // CRITICAL DEBUG: Log the engine's internal state at the very beginning
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        crate::debug_utils::debug_log("========================================");
        crate::debug_utils::debug_log("[GET_BEST_MOVE] CALLED - ENGINE INTERNAL STATE:");
        crate::debug_utils::debug_log(&format!("[GET_BEST_MOVE]   Current Player: {:?}", self.current_player));
        crate::debug_utils::debug_log(&format!("[GET_BEST_MOVE]   Position FEN: {}", fen));
        crate::debug_utils::debug_log(&format!("[GET_BEST_MOVE]   Captured Pieces: black={:?}, white={:?}", 
            self.captured_pieces.black, self.captured_pieces.white));
        crate::debug_utils::debug_log("========================================");
        
        crate::debug_utils::set_search_start_time();
        crate::debug_utils::trace_log("GET_BEST_MOVE", &format!("Starting search: depth={}, time_limit={}ms", depth, time_limit_ms));
        crate::debug_utils::start_timing("tablebase_check");
        

        crate::debug_utils::trace_log("GET_BEST_MOVE", &format!("Position FEN: {}", fen));
        
        // Check tablebase first
        if let Some(tablebase_result) = self.tablebase.probe(&self.board, self.current_player, &self.captured_pieces) {
            crate::debug_utils::end_timing("tablebase_check", "GET_BEST_MOVE");
            if let Some(best_move) = tablebase_result.best_move {
                
                crate::debug_utils::log_decision("GET_BEST_MOVE", "Tablebase hit", 
                    &format!("Move: {}, outcome: {:?}, distance: {:?}", 
                        best_move.to_usi_string(), 
                        tablebase_result.outcome, 
                        tablebase_result.distance_to_mate), 
                    None);
                
                return Some(best_move);
            }
        } else {
            crate::debug_utils::end_timing("tablebase_check", "GET_BEST_MOVE");
        }
        
        // Check opening book second
        crate::debug_utils::start_timing("opening_book_check");
        if self.opening_book.is_loaded() {
            if let Some(book_move) = self.opening_book.get_best_move(&fen) {
                
                crate::debug_utils::debug_log(&format!(
                    "Found opening book move: {}",
                    book_move.to_usi_string()
                ));
                
                return Some(book_move);
            }
        }


        // Check for legal moves BEFORE starting search to avoid panics
        crate::debug_utils::debug_log("Checking for legal moves before search");
        let move_generator = MoveGenerator::new();
        let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
        
        if legal_moves.is_empty() {
            crate::debug_utils::debug_log("No legal moves available - position is checkmate or stalemate");
            return None;
        }
        
        crate::debug_utils::debug_log(&format!("Found {} legal moves, proceeding with search", legal_moves.len()));

        // Handle depth 0 (unlimited/adaptive) - use high limit, engine will adapt based on time
        // Using 100 as practical maximum (deep searches rarely exceed this)
        let actual_depth = if depth == 0 { 100 } else { depth };
        crate::debug_utils::debug_log(&format!("Creating searcher with depth: {} (requested: {}, 0 = unlimited), time_limit: {}ms", 
            actual_depth, depth, time_limit_ms));
        let mut searcher = search::search_engine::IterativeDeepening::new_with_threads(
            actual_depth,
            time_limit_ms,
            stop_flag,
            self.thread_count,
        );
        
        crate::debug_utils::debug_log("Trying to get search engine lock");
        
        
        // Try to get the search engine lock, but don't panic if it fails
        crate::debug_utils::debug_log("About to lock search engine");
        let search_result = self.search_engine.lock().map(|mut search_engine_guard| {
            crate::debug_utils::debug_log("Got search engine lock, starting search");
            searcher.search(&mut search_engine_guard, &self.board, &self.captured_pieces, self.current_player)
        });
        
        crate::debug_utils::debug_log("Search completed, checking result");
        
        
        if let Ok(Some((move_, _score))) = search_result {
            Some(move_)
        } else {
            // Fallback to random move if search fails
            let move_generator = MoveGenerator::new();
            let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
            if legal_moves.is_empty() {
                return None;
            }
            // Use a seeded RNG that's WASM-compatible
            let mut rng = StdRng::seed_from_u64(42); // Fixed seed for deterministic behavior
            legal_moves.choose(&mut rng).cloned()
        }
    }

    /// Apply a move to the engine's board
    pub fn apply_move(&mut self, move_: &Move) -> bool {
        use crate::moves::MoveGenerator;
        
        // Get current legal moves to validate
        let move_generator = MoveGenerator::new();
        let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
        
        // Check if move is legal
        if !legal_moves.contains(move_) {
            crate::debug_utils::debug_log(&format!("Move {} is not legal in current position", move_.to_usi_string()));
            return false;
        }
        
        // Apply the move
        if let Some(captured_piece) = self.board.make_move(move_) {
            self.captured_pieces.add_piece(captured_piece.piece_type, self.current_player);
        }
        
        // Switch turns
        self.current_player = self.current_player.opposite();
        
        crate::debug_utils::debug_log(&format!("Applied move: {}", move_.to_usi_string()));
        true
    }

    /// Check if the current position is a terminal state (checkmate, stalemate)
    pub fn is_game_over(&self) -> Option<GameResult> {
        use crate::moves::MoveGenerator;
        
        let move_generator = MoveGenerator::new();
        let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player, &self.captured_pieces);
        
        if legal_moves.is_empty() {
            // Check if in check (checkmate) or not (stalemate)
            let in_check = self.board.is_king_in_check(self.current_player, &self.captured_pieces);
            
            Some(if in_check {
                // Checkmate - current player loses
                if self.current_player == Player::Black {
                    GameResult::Loss  // Black is mated, Black loses
                } else {
                    GameResult::Win   // White is mated, White loses
                }
            } else {
                GameResult::Draw     // Stalemate
            })
        } else {
            None  // Game not over
        }
    }

    pub fn handle_position(&mut self, parts: &[&str]) -> Vec<String> {
        let mut output = Vec::new();
        let sfen_str: String;
        let mut moves_start_index: Option<usize> = None;

        crate::debug_utils::debug_log(&format!("handle_position called with {} parts", parts.len()));
        crate::debug_utils::debug_log(&format!("Parts: {:?}", parts));

        if parts.is_empty() {
            output.push("info string error Invalid position command".to_string());
            return output;
        }

        if parts[0] == "startpos" {
            sfen_str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string();
            crate::debug_utils::debug_log("Using startpos");
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
            crate::debug_utils::debug_log(&format!("Parsed SFEN: '{}'", sfen_str));
            if current_index < parts.len() && parts[current_index] == "moves" {
                moves_start_index = Some(current_index + 1);
            }
        } else {
            output.push("info string error Invalid position command: expected 'startpos' or 'sfen'".to_string());
            return output;
        }

        crate::debug_utils::debug_log(&format!("About to parse SFEN: '{}'", sfen_str));
        match BitboardBoard::from_fen(&sfen_str) {
            Ok((board, player, captured_pieces)) => {
                crate::debug_utils::debug_log(&format!("SFEN parsed successfully, player: {:?}", player));
                self.board = board;
                self.current_player = player;
                self.captured_pieces = captured_pieces;
                
                // CRITICAL DEBUG: Verify the state was actually set
                let verify_fen = self.board.to_fen(self.current_player, &self.captured_pieces);
                crate::debug_utils::debug_log("========================================");
                crate::debug_utils::debug_log("[HANDLE_POSITION] STATE SET - VERIFICATION:");
                crate::debug_utils::debug_log(&format!("[HANDLE_POSITION]   self.current_player = {:?}", self.current_player));
                crate::debug_utils::debug_log(&format!("[HANDLE_POSITION]   Verification FEN: {}", verify_fen));
                crate::debug_utils::debug_log(&format!("[HANDLE_POSITION]   self.captured_pieces: black={:?}, white={:?}", 
                    self.captured_pieces.black, self.captured_pieces.white));
                crate::debug_utils::debug_log("========================================");
            }
            Err(e) => {
                crate::debug_utils::debug_log(&format!("SFEN parse FAILED: {}", e));
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

        output.push("info string Board state updated.".to_string());
        output
    }


    pub fn handle_stop(&mut self) -> Vec<String> {
        self.stop_flag.store(true, Ordering::Relaxed);
        Vec::new()
    }

    pub fn handle_setoption(&mut self, parts: &[&str]) -> Vec<String> {
        let mut output = Vec::new();
        if parts.len() >= 4 && parts[0] == "name" && parts[2] == "value" {
            match parts[1] {
                "USI_Hash" => {
                    if let Ok(size) = parts[3].parse::<usize>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            *search_engine_guard = SearchEngine::new(Some(self.stop_flag.clone()), size);
                            output.push(format!("info string Set USI_Hash to {} MB", size));
                        }
                    }
                }
                "MaxDepth" | "depth" => {
                    // Support both MaxDepth (new) and depth (legacy) for backward compatibility
                    if let Ok(depth) = parts[3].parse::<u8>() {
                        if depth <= 100 {
                            self.set_max_depth(depth);
                            if depth == 0 {
                                output.push("info string Set MaxDepth to 0 (unlimited/adaptive)".to_string());
                            } else {
                                output.push(format!("info string Set MaxDepth to {}", depth));
                            }
                        } else {
                            output.push("info string error MaxDepth must be between 0 and 100".to_string());
                        }
                    }
                }
                // Quiescence search options
                "QuiescenceDepth" => {
                    if let Ok(depth) = parts[3].parse::<u8>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.quiescence.max_depth = depth;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string Set quiescence max_depth to {}", depth));
                        }
                    }
                }
                "EnableQuiescence" => {
                    if let Ok(_enabled) = parts[3].parse::<bool>() {
                        // Note: QuiescenceConfig doesn't have an 'enabled' field
                        // Quiescence search is always enabled in the engine
                        if let Ok(_search_engine_guard) = self.search_engine.lock() {
                            output.push(format!("info string Quiescence search is always enabled in the engine"));
                        }
                    }
                }
                // Null-move pruning options
                "EnableNullMove" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.null_move.enabled = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} null-move pruning", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                "NullMoveMinDepth" => {
                    if let Ok(depth) = parts[3].parse::<u8>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.null_move.min_depth = depth;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string Set null-move min_depth to {}", depth));
                        }
                    }
                }
                // Late move reduction options
                "EnableLMR" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.lmr.enabled = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} late move reduction", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                // IID options
                "EnableIID" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.iid.enabled = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} internal iterative deepening", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                // Aspiration windows options
                "EnableAspirationWindows" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.aspiration_windows.enabled = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} aspiration windows", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                "AspirationWindowSize" => {
                    if let Ok(size) = parts[3].parse::<u16>() {
                        if size >= 10 && size <= 500 {
                            if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                                let mut config = search_engine_guard.get_engine_config();
                                config.aspiration_windows.base_window_size = size as i32;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push(format!("info string Set aspiration window size to {}", size));
                            }
                        } else {
                            output.push("info string error AspirationWindowSize must be between 10 and 500".to_string());
                        }
                    }
                }
                "EnablePositionTypeTracking" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.aspiration_windows.enable_position_type_tracking = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} position type tracking", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                // Time Management Options (Task 8.0, 4.0)
                "TimeCheckFrequency" => {
                    if let Ok(frequency) = parts[3].parse::<u32>() {
                        if frequency >= 1 && frequency <= 100000 {
                            if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                                let mut config = search_engine_guard.get_engine_config();
                                config.time_management.time_check_frequency = frequency;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push(format!("info string Set time check frequency to {} nodes", frequency));
                            }
                        } else {
                            output.push("info string error TimeCheckFrequency must be between 1 and 100000".to_string());
                        }
                    }
                }
                "TimeSafetyMargin" => {
                    if let Ok(margin) = parts[3].parse::<u32>() {
                        if margin <= 10000 {
                            if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                                let mut config = search_engine_guard.get_engine_config();
                                config.time_management.absolute_safety_margin_ms = margin;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push(format!("info string Set time safety margin to {}ms", margin));
                            }
                        } else {
                            output.push("info string error TimeSafetyMargin must be between 0 and 10000".to_string());
                        }
                    }
                }
                "TimeAllocationStrategy" => {
                    if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                        let mut config = search_engine_guard.get_engine_config();
                        match parts[3] {
                            "Equal" => {
                                config.time_management.allocation_strategy = crate::types::TimeAllocationStrategy::Equal;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push("info string Set time allocation strategy to Equal".to_string());
                            }
                            "Exponential" => {
                                config.time_management.allocation_strategy = crate::types::TimeAllocationStrategy::Exponential;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push("info string Set time allocation strategy to Exponential".to_string());
                            }
                            "Adaptive" => {
                                config.time_management.allocation_strategy = crate::types::TimeAllocationStrategy::Adaptive;
                                let _ = search_engine_guard.update_engine_config(config);
                                output.push("info string Set time allocation strategy to Adaptive".to_string());
                            }
                            _ => {
                                output.push("info string error TimeAllocationStrategy must be Equal, Exponential, or Adaptive".to_string());
                            }
                        }
                    }
                }
                "EnableTimeBudget" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.time_management.enable_time_budget = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} time budget allocation", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                "EnableCheckOptimization" => {
                    if let Ok(enabled) = parts[3].parse::<bool>() {
                        if let Ok(mut search_engine_guard) = self.search_engine.lock() {
                            let mut config = search_engine_guard.get_engine_config();
                            config.time_management.enable_check_optimization = enabled;
                            let _ = search_engine_guard.update_engine_config(config);
                            output.push(format!("info string {} check position optimization", if enabled { "Enabled" } else { "Disabled" }));
                        }
                    }
                }
                // Tablebase options
                "EnableTablebase" => {
                    if parts[3] == "true" {
                        self.enable_tablebase();
                        output.push("info string Enabled tablebase".to_string());
                    } else if parts[3] == "false" {
                        self.disable_tablebase();
                        output.push("info string Disabled tablebase".to_string());
                    }
                }
                "USI_Threads" => {
                    if let Ok(threads) = parts[3].parse::<usize>() {
                        self.thread_count = threads.clamp(1, 32);
                        self.save_prefs();
                        output.push(format!("info string Set USI_Threads to {}", self.thread_count));
                    } else {
                        output.push("info string error Invalid thread count value".to_string());
                    }
                }
                _ => {
                    output.push(format!("info string Unknown option: {}", parts[1]));
                }
            }
        }
        output
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
                    self.set_debug_enabled(true);
                    output.push("info string debug mode enabled".to_string());
                },
                "off" => {
                    self.debug_mode = false;
                    self.set_debug_enabled(false);
                    output.push("info string debug mode disabled".to_string());
                },
                "trace" => {
                    self.set_debug_enabled(true);
                    output.push("info string trace logging enabled".to_string());
                },
                "notrace" => {
                    self.set_debug_enabled(false);
                    output.push("info string trace logging disabled".to_string());
                },
                _ => output.push(format!("info string unknown debug command {} (use: on/off/trace/notrace)", part)),
            }
        } else {
            output.push("info string debug command needs an argument (on/off/trace/notrace)".to_string());
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

    // Tablebase methods
    pub fn enable_tablebase(&mut self) {
        self.tablebase.enable();
    }

    pub fn disable_tablebase(&mut self) {
        self.tablebase.disable();
    }

    pub fn is_tablebase_enabled(&self) -> bool {
        self.tablebase.is_enabled()
    }

    pub fn get_tablebase_stats(&self) -> String {
        let stats = self.tablebase.get_stats();
        format!(
            "Tablebase Stats: Probes={}, Cache Hits={}, Solver Hits={}, Misses={}, Cache Hit Rate={:.2}%, Solver Hit Rate={:.2}%, Overall Hit Rate={:.2}%, Avg Probe Time={:.2}ms",
            stats.total_probes,
            stats.cache_hits,
            stats.solver_hits,
            stats.misses,
            stats.cache_hit_rate() * 100.0,
            stats.solver_hit_rate() * 100.0,
            stats.overall_hit_rate() * 100.0,
            stats.average_probe_time_ms
        )
    }

    pub fn reset_tablebase_stats(&mut self) {
        self.tablebase.reset_stats();
    }
}

// Debug control functions
pub fn is_debug_enabled() -> bool {
    debug_utils::is_debug_enabled()
}

// WASM bindings removed - application now uses Tauri for desktop functionality
// The engine is accessed via the standalone USI binary (src/bin/shogi_engine.rs)
