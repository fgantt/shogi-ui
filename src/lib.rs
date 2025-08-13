use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;

mod bitboards;
mod moves;
mod evaluation;
mod search;
mod types;

use bitboards::*;
use moves::*;
use evaluation::*;
use search::*;
use types::*;

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
pub struct ShogiEngine {
    board: BitboardBoard,
    captured_pieces: CapturedPieces,
    current_player: Player,
    move_history: Vec<Move>,
    transposition_table: HashMap<u64, TranspositionEntry>,
}

#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        Self {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            current_player: Player::Black,
            move_history: Vec::new(),
            transposition_table: HashMap::new(),
        }
    }

    pub fn get_best_move(&mut self, difficulty: u8, time_limit_ms: u32) -> Option<Move> {
        // Create a move generator
        let move_generator = MoveGenerator::new();
        
        // Generate all legal moves for the current player
        let legal_moves = move_generator.generate_legal_moves(&self.board, self.current_player);
        
        if legal_moves.is_empty() {
            return None;
        }

        // Choose a random move
        let mut rng = rand::thread_rng();
        legal_moves.choose(&mut rng).cloned()
    }

    fn search_at_depth(&self, _depth: u8, _time_limit_ms: u32) -> Option<(Move, i32)> {
        // Placeholder for search implementation
        None
    }

    pub fn make_move(&mut self, from: u8, to: u8, promote: bool) -> bool {
        let from_pos = Position::from_u8(from);
        let to_pos = Position::from_u8(to);
        
        if let Some(move_) = self.board.generate_move(from_pos, to_pos, promote) {
            if self.board.is_legal_move(&move_) {
                self.board.make_move(&move_);
                self.move_history.push(move_.clone());
                self.current_player = self.current_player.opposite();
                return true;
            }
        }
        false
    }

    pub fn drop_piece(&mut self, piece_type: u8, to: u8) -> bool {
        let piece = PieceType::from_u8(piece_type);
        let to_pos = Position::from_u8(to);
        
        if let Some(move_) = self.board.generate_drop(piece, to_pos) {
            if self.board.is_legal_move(&move_) {
                self.board.make_move(&move_);
                self.move_history.push(move_);
                self.current_player = self.current_player.opposite();
                return true;
            }
        }
        false
    }

    pub fn get_board_state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.board.to_fen()).unwrap()
    }

    pub fn is_checkmate(&self) -> bool {
        self.board.is_checkmate(self.current_player)
    }

    pub fn is_stalemate(&self) -> bool {
        self.board.is_stalemate(self.current_player)
    }

    pub fn get_legal_moves(&self, from: u8) -> Vec<u8> {
        let from_pos = Position::from_u8(from);
        self.board
            .get_legal_moves(from_pos)
            .into_iter()
            .map(|pos| pos.to_u8())
            .collect()
    }

    pub fn get_legal_drops(&self, piece_type: u8) -> Vec<u8> {
        let piece = PieceType::from_u8(piece_type);
        self.board
            .get_legal_drops(piece)
            .into_iter()
            .map(|pos| pos.to_u8())
            .collect()
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

    pub fn set_captured_pieces(&mut self, captured_json: &str) {
        self.captured_pieces = CapturedPieces::new(); // Clear captured pieces
        let captured_pieces: Vec<CapturedPieceJson> = serde_json::from_str(captured_json).unwrap();
        for captured_piece_json in captured_pieces {
            let player = if captured_piece_json.player == "Black" { Player::Black } else { Player::White };
            let piece_type = PieceType::from_str(&captured_piece_json.piece_type).unwrap();
            self.captured_pieces.add_piece(piece_type, player);
        }
    }
}

// Helper function to convert between Rust and JavaScript types
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
