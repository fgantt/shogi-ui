use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::{Move, Position, Player, PieceType};

#[derive(Serialize, Deserialize, Debug)]
pub struct OpeningBook {
    openings: Vec<Opening>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Opening {
    name: String,
    moves: HashMap<String, Vec<BookMove>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookMove {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub promote: bool,
    #[serde(rename = "pieceType")]
    #[serde(default)]
    pub piece_type: String,
}

impl OpeningBook {
    pub fn new() -> Self {
        let book_str = include_str!("ai/openingBook.json");
        let openings: Vec<Opening> = serde_json::from_str(book_str).unwrap();
        Self { openings }
    }

    pub fn get_move(&self, fen: &str) -> Option<Move> {
        for opening in &self.openings {
            if let Some(book_moves) = opening.moves.get(fen) {
                if let Some(book_move) = book_moves.first() {
                    return Some(self.convert_book_move_to_move(book_move));
                }
            }
        }
        None
    }

    fn convert_book_move_to_move(&self, book_move: &BookMove) -> Move {
        let from = if book_move.from == "drop" {
            None
        } else {
            let from_row = book_move.from.chars().nth(1).unwrap().to_digit(10).unwrap() as u8 - 1;
            let from_col = book_move.from.chars().nth(0).unwrap().to_digit(10).unwrap() as u8 - 1;
            Some(Position::new(from_row, from_col))
        };

        let to_row = book_move.to.chars().nth(1).unwrap().to_digit(10).unwrap() as u8 - 1;
        let to_col = book_move.to.chars().nth(0).unwrap().to_digit(10).unwrap() as u8 - 1;
        let to = Position::new(to_row, to_col);

        Move {
            from,
            to,
            is_promotion: book_move.promote,
            piece_type: PieceType::from_str(&book_move.piece_type).unwrap(),
            captured_piece: None,
            is_capture: false,
            player: Player::Black, // This is not correct, but it will be fixed later
        }
    }
}
