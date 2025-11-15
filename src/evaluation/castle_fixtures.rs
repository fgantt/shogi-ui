#![allow(dead_code)]

use crate::bitboards::BitboardBoard;
use crate::evaluation::castle_geometry::RelativeOffset;
use crate::types::core::{Piece, PieceType, Player, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleFixtureTheme {
    Canonical,
    Mirrored,
    Partial,
    Broken,
    Attacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleType {
    Mino,
    Anaguma,
    Yagura,
    Snowroof,
}

pub struct CastleFixture {
    pub name: &'static str,
    pub theme: CastleFixtureTheme,
    pub castle_type: CastleType,
    pub player: Player,
    pub builder: fn(Player) -> (BitboardBoard, Position),
}

pub fn castle_fixtures() -> Vec<CastleFixture> {
    vec![
        // Canonical castles
        CastleFixture {
            name: "mino_canonical_black",
            theme: CastleFixtureTheme::Canonical,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_canonical,
        },
        CastleFixture {
            name: "mino_canonical_white",
            theme: CastleFixtureTheme::Canonical,
            castle_type: CastleType::Mino,
            player: Player::White,
            builder: mino_canonical,
        },
        CastleFixture {
            name: "anaguma_canonical_black",
            theme: CastleFixtureTheme::Canonical,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_canonical,
        },
        CastleFixture {
            name: "yagura_canonical_black",
            theme: CastleFixtureTheme::Canonical,
            castle_type: CastleType::Yagura,
            player: Player::Black,
            builder: yagura_canonical,
        },
        // Mirrored castles
        CastleFixture {
            name: "mino_mirrored_left_black",
            theme: CastleFixtureTheme::Mirrored,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_mirrored_left,
        },
        CastleFixture {
            name: "mino_mirrored_right_black",
            theme: CastleFixtureTheme::Mirrored,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_mirrored_right,
        },
        CastleFixture {
            name: "anaguma_mirrored_left_black",
            theme: CastleFixtureTheme::Mirrored,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_mirrored_left,
        },
        CastleFixture {
            name: "anaguma_mirrored_right_black",
            theme: CastleFixtureTheme::Mirrored,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_mirrored_right,
        },
        // Partial castles
        CastleFixture {
            name: "mino_partial_missing_silver",
            theme: CastleFixtureTheme::Partial,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_partial_missing_silver,
        },
        CastleFixture {
            name: "mino_partial_missing_pawns",
            theme: CastleFixtureTheme::Partial,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_partial_missing_pawns,
        },
        CastleFixture {
            name: "anaguma_partial_missing_gold",
            theme: CastleFixtureTheme::Partial,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_partial_missing_gold,
        },
        // Broken castles
        CastleFixture {
            name: "mino_broken_breached_wall",
            theme: CastleFixtureTheme::Broken,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_broken_breached_wall,
        },
        CastleFixture {
            name: "anaguma_broken_missing_defenders",
            theme: CastleFixtureTheme::Broken,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_broken_missing_defenders,
        },
        // Attacked castles
        CastleFixture {
            name: "mino_attacked_rook_file",
            theme: CastleFixtureTheme::Attacked,
            castle_type: CastleType::Mino,
            player: Player::Black,
            builder: mino_attacked_rook_file,
        },
        CastleFixture {
            name: "anaguma_attacked_infiltration",
            theme: CastleFixtureTheme::Attacked,
            castle_type: CastleType::Anaguma,
            player: Player::Black,
            builder: anaguma_attacked_infiltration,
        },
        CastleFixture {
            name: "yagura_attacked_mating_net",
            theme: CastleFixtureTheme::Attacked,
            castle_type: CastleType::Yagura,
            player: Player::Black,
            builder: yagura_attacked_mating_net,
        },
    ]
}

fn place_piece(board: &mut BitboardBoard, player: Player, piece_type: PieceType, row: u8, col: u8) {
    board.place_piece(Piece::new(piece_type, player), Position::new(row, col));
}

fn place_relative(
    board: &mut BitboardBoard,
    player: Player,
    king_pos: Position,
    offset: RelativeOffset,
    piece_type: PieceType,
) {
    if let Some(pos) = offset.to_absolute(king_pos, player) {
        board.place_piece(Piece::new(piece_type, player), pos);
    }
}

// Canonical Mino castle (center file)
fn mino_canonical(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Canonical Anaguma castle
fn anaguma_canonical(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 2);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Canonical Yagura castle
fn yagura_canonical(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(0, 3),
        PieceType::Lance,
    );

    (board, king_pos)
}

// Mirrored Mino (left side)
fn mino_mirrored_left(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 2);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Mirrored Mino (right side)
fn mino_mirrored_right(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 6);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Mirrored Anaguma (left side)
fn anaguma_mirrored_left(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 1);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Mirrored Anaguma (right side)
fn anaguma_mirrored_right(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 7);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 2),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Partial Mino - missing silver
fn mino_partial_missing_silver(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    // Missing silver
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Partial Mino - missing pawns
fn mino_partial_missing_pawns(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    // Missing pawns

    (board, king_pos)
}

// Partial Anaguma - missing gold
fn anaguma_partial_missing_gold(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 2);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    // Missing one gold
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Broken Mino - breached pawn wall
fn mino_broken_breached_wall(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    // Only one pawn (breached wall)
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Broken Anaguma - missing defenders
fn anaguma_broken_missing_defenders(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 2);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    // Missing gold and silver
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );

    (board, king_pos)
}

// Attacked Mino - rook on file
fn mino_attacked_rook_file(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);
    let opponent = player.opposite();

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 1),
        PieceType::Pawn,
    );

    // Opponent's rook attacking the file
    let attack_row = if player == Player::Black { 3 } else { 5 };
    place_piece(&mut board, opponent, PieceType::Rook, attack_row, 4);

    (board, king_pos)
}

// Attacked Anaguma - infiltration
fn anaguma_attacked_infiltration(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 2);
    let opponent = player.opposite();

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, 0),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );

    // Opponent piece infiltrating the king zone
    let infil_row = if player == Player::Black { 7 } else { 1 };
    place_piece(&mut board, opponent, PieceType::Knight, infil_row, 3);

    (board, king_pos)
}

// Attacked Yagura - mating net
fn yagura_attacked_mating_net(player: Player) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_row = if player == Player::Black { 8 } else { 0 };
    let king_pos = Position::new(king_row, 4);
    let opponent = player.opposite();

    board.place_piece(Piece::new(PieceType::King, player), king_pos);
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-1, -1),
        PieceType::Gold,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -1),
        PieceType::Silver,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, -2),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(-2, 0),
        PieceType::Pawn,
    );
    place_relative(
        &mut board,
        player,
        king_pos,
        RelativeOffset::new(0, 3),
        PieceType::Lance,
    );

    // Opponent pieces creating mating net
    let attack_row = if player == Player::Black { 4 } else { 4 };
    place_piece(&mut board, opponent, PieceType::Rook, attack_row, 4);
    place_piece(&mut board, opponent, PieceType::Bishop, attack_row, 2);
    place_piece(&mut board, opponent, PieceType::Gold, attack_row + 1, 3);

    (board, king_pos)
}
