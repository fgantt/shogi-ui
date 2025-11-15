#![allow(dead_code)]

use crate::bitboards::BitboardBoard;
use crate::types::board::CapturedPieces;
use crate::types::core::{Piece, PieceType, Player, Position};

#[derive(Debug, Clone, Copy)]
pub enum FixtureTheme {
    CentralFight,
    CastleWeakness,
    SpaceClamp,
}

#[derive(Debug, Clone, Copy)]
pub enum FixtureAdvantage {
    Black { min_cp: i32 },
    White { min_cp: i32 },
    Balanced { max_cp: i32 },
}

pub struct PositionalFixture {
    pub name: &'static str,
    pub theme: FixtureTheme,
    pub advantage: FixtureAdvantage,
    pub builder: fn() -> (BitboardBoard, CapturedPieces),
}

pub fn positional_fixtures() -> Vec<PositionalFixture> {
    vec![
        PositionalFixture {
            name: "central_file_bind",
            theme: FixtureTheme::CentralFight,
            advantage: FixtureAdvantage::Black { min_cp: 40 },
            builder: central_file_bind,
        },
        PositionalFixture {
            name: "castle_gap_exposed_king",
            theme: FixtureTheme::CastleWeakness,
            advantage: FixtureAdvantage::Black { min_cp: 30 },
            builder: castle_gap_exposed_king,
        },
        PositionalFixture {
            name: "space_clamp_double_lance",
            theme: FixtureTheme::SpaceClamp,
            advantage: FixtureAdvantage::Black { min_cp: 35 },
            builder: space_clamp_double_lance,
        },
    ]
}

fn place_piece(board: &mut BitboardBoard, player: Player, piece_type: PieceType, row: u8, col: u8) {
    board.place_piece(Piece::new(piece_type, player), Position::new(row, col));
}

fn central_file_bind() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let mut captured = CapturedPieces::new();

    // Kings
    place_piece(&mut board, Player::Black, PieceType::King, 8, 4);
    place_piece(&mut board, Player::White, PieceType::King, 0, 4);

    // Black central pressure
    place_piece(&mut board, Player::Black, PieceType::Rook, 4, 4);
    place_piece(&mut board, Player::Black, PieceType::Bishop, 5, 3);
    place_piece(&mut board, Player::Black, PieceType::Silver, 5, 5);
    place_piece(&mut board, Player::Black, PieceType::Gold, 4, 3);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 6, 4);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 5, 4);

    // White defenders
    place_piece(&mut board, Player::White, PieceType::Silver, 2, 4);
    place_piece(&mut board, Player::White, PieceType::Pawn, 3, 4);
    place_piece(&mut board, Player::White, PieceType::Pawn, 3, 3);
    place_piece(&mut board, Player::White, PieceType::Bishop, 1, 6);

    // Black retains an extra pawn in hand for potential drops.
    captured.add_piece(PieceType::Pawn, Player::Black);

    (board, captured)
}

fn castle_gap_exposed_king() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let captured = CapturedPieces::new();

    // Kings
    place_piece(&mut board, Player::Black, PieceType::King, 8, 4);
    place_piece(&mut board, Player::White, PieceType::King, 0, 4);

    // White castle pieces misplaced, creating gaps.
    place_piece(&mut board, Player::White, PieceType::Gold, 1, 5);
    place_piece(&mut board, Player::White, PieceType::Silver, 1, 3);

    // Black siege pieces
    place_piece(&mut board, Player::Black, PieceType::Rook, 3, 4);
    place_piece(&mut board, Player::Black, PieceType::Bishop, 2, 2);
    place_piece(&mut board, Player::Black, PieceType::Gold, 4, 4);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 4, 5);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 2, 4);

    (board, captured)
}

fn space_clamp_double_lance() -> (BitboardBoard, CapturedPieces) {
    let mut board = BitboardBoard::empty();
    let mut captured = CapturedPieces::new();

    // Kings
    place_piece(&mut board, Player::Black, PieceType::King, 8, 4);
    place_piece(&mut board, Player::White, PieceType::King, 0, 4);

    // Black advanced territory
    place_piece(&mut board, Player::Black, PieceType::Pawn, 3, 2);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 3, 4);
    place_piece(&mut board, Player::Black, PieceType::Pawn, 3, 6);
    place_piece(&mut board, Player::Black, PieceType::Lance, 2, 0);
    place_piece(&mut board, Player::Black, PieceType::Lance, 2, 8);
    place_piece(&mut board, Player::Black, PieceType::Rook, 4, 6);
    place_piece(&mut board, Player::Black, PieceType::Silver, 4, 3);
    place_piece(&mut board, Player::Black, PieceType::Gold, 4, 5);
    place_piece(&mut board, Player::Black, PieceType::Silver, 5, 5);

    // White cramped defenders
    place_piece(&mut board, Player::White, PieceType::Gold, 7, 4);
    place_piece(&mut board, Player::White, PieceType::Silver, 7, 5);
    place_piece(&mut board, Player::White, PieceType::Pawn, 6, 3);
    place_piece(&mut board, Player::White, PieceType::Pawn, 6, 5);

    // Black has extra pawns ready to maintain the clamp.
    captured.add_piece(PieceType::Pawn, Player::Black);
    captured.add_piece(PieceType::Pawn, Player::Black);

    (board, captured)
}
