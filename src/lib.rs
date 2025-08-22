use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;

mod bitboards;
mod moves;
mod evaluation;
mod search;
mod types;
mod opening_book;

use bitboards::*;
use moves::*;
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
pub struct ShogiEngine {
    board: BitboardBoard,
    captured_pieces: CapturedPieces,
    current_player: Player,
    move_history: Vec<Move>,
    opening_book: OpeningBook,
}

#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        Self {
            board: BitboardBoard::new(),
            captured_pieces: CapturedPieces::new(),
            current_player: Player::Black,
            move_history: Vec::new(),
            opening_book: OpeningBook::new(),
        }
    }

    pub fn get_best_move(&mut self, difficulty: u8, time_limit_ms: u32) -> Option<Move> {
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
        if let Some(book_move) = self.opening_book.get_move(&fen) {
            return Some(book_move);
        }

        let actual_difficulty = if difficulty == 0 { 1 } else { difficulty };
        let mut searcher = search::IterativeDeepening::new(actual_difficulty, time_limit_ms);
        if let Some((move_, _score)) = searcher.search(&self.board, &self.captured_pieces, self.current_player) {
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

    pub fn set_captured_pieces(&mut self, captured_json: &str) {
        self.captured_pieces = CapturedPieces::new(); // Clear captured pieces
        let captured_pieces_json: Vec<CapturedPieceJson> = serde_json::from_str(captured_json).unwrap();
        for captured_piece_json in captured_pieces_json {
            let player = if captured_piece_json.player == "Black" { Player::Black } else { Player::White };
            let piece_type = PieceType::from_str(&captured_piece_json.piece_type).unwrap();
            self.captured_pieces.add_piece(piece_type, player);
        }
    }
}