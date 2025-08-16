use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn opposite(self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum PieceType {
    Pawn,
    Lance,
    Knight,
    Silver,
    Gold,
    Bishop,
    Rook,
    King,
    PromotedPawn,
    PromotedLance,
    PromotedKnight,
    PromotedSilver,
    PromotedBishop,
    PromotedRook,
}

impl PieceType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pawn" => Some(PieceType::Pawn),
            "Lance" => Some(PieceType::Lance),
            "Knight" => Some(PieceType::Knight),
            "Silver" => Some(PieceType::Silver),
            "Gold" => Some(PieceType::Gold),
            "Bishop" => Some(PieceType::Bishop),
            "Rook" => Some(PieceType::Rook),
            "King" => Some(PieceType::King),
            "PromotedPawn" => Some(PieceType::PromotedPawn),
            "PromotedLance" => Some(PieceType::PromotedLance),
            "PromotedKnight" => Some(PieceType::PromotedKnight),
            "PromotedSilver" => Some(PieceType::PromotedSilver),
            "PromotedBishop" => Some(PieceType::PromotedBishop),
            "PromotedRook" => Some(PieceType::PromotedRook),
            _ => None,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => PieceType::Pawn,
            1 => PieceType::Lance,
            2 => PieceType::Knight,
            3 => PieceType::Silver,
            4 => PieceType::Gold,
            5 => PieceType::Bishop,
            6 => PieceType::Rook,
            7 => PieceType::King,
            8 => PieceType::PromotedPawn,
            9 => PieceType::PromotedLance,
            10 => PieceType::PromotedKnight,
            11 => PieceType::PromotedSilver,
            12 => PieceType::PromotedBishop,
            13 => PieceType::PromotedRook,
            _ => PieceType::Pawn,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            PieceType::Pawn => 0,
            PieceType::Lance => 1,
            PieceType::Knight => 2,
            PieceType::Silver => 3,
            PieceType::Gold => 4,
            PieceType::Bishop => 5,
            PieceType::Rook => 6,
            PieceType::King => 7,
            PieceType::PromotedPawn => 8,
            PieceType::PromotedLance => 9,
            PieceType::PromotedKnight => 10,
            PieceType::PromotedSilver => 11,
            PieceType::PromotedBishop => 12,
            PieceType::PromotedRook => 13,
        }
    }

    pub fn base_value(self) -> i32 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Lance => 300,
            PieceType::Knight => 320,
            PieceType::Silver => 450,
            PieceType::Gold => 500,
            PieceType::Bishop => 800,
            PieceType::Rook => 1000,
            PieceType::King => 20000,
            PieceType::PromotedPawn => 500,
            PieceType::PromotedLance => 500,
            PieceType::PromotedKnight => 500,
            PieceType::PromotedSilver => 500,
            PieceType::PromotedBishop => 1200,
            PieceType::PromotedRook => 1300,
        }
    }

    pub fn can_promote(self) -> bool {
        matches!(self, 
            PieceType::Pawn | 
            PieceType::Lance | 
            PieceType::Knight | 
            PieceType::Silver | 
            PieceType::Bishop | 
            PieceType::Rook
        )
    }

    pub fn promoted_version(self) -> Option<Self> {
        match self {
            PieceType::Pawn => Some(PieceType::PromotedPawn),
            PieceType::Lance => Some(PieceType::PromotedLance),
            PieceType::Knight => Some(PieceType::PromotedKnight),
            PieceType::Silver => Some(PieceType::PromotedSilver),
            PieceType::Bishop => Some(PieceType::PromotedBishop),
            PieceType::Rook => Some(PieceType::PromotedRook),
            _ => None,
        }
    }

    pub fn unpromoted_version(self) -> Option<Self> {
        match self {
            PieceType::PromotedPawn => Some(PieceType::Pawn),
            PieceType::PromotedLance => Some(PieceType::Lance),
            PieceType::PromotedKnight => Some(PieceType::Knight),
            PieceType::PromotedSilver => Some(PieceType::Silver),
            PieceType::PromotedBishop => Some(PieceType::Bishop),
            PieceType::PromotedRook => Some(PieceType::Rook),
            _ => None,
        }
    }

    pub fn get_move_offsets(&self, direction: i8) -> Vec<(i8, i8)> {
        match self {
            PieceType::Silver => vec![(direction, 0), (direction, -1), (direction, 1), (-direction, -1), (-direction, 1)],
            PieceType::Gold | PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight | PieceType::PromotedSilver => 
                vec![(direction, 0), (direction, -1), (direction, 1), (0, -1), (0, 1), (-direction, 0)],
            PieceType::King => vec![(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)],
            PieceType::PromotedBishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1), (1, 0), (-1, 0), (0, 1), (0, -1)],
            PieceType::PromotedRook => vec![(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)],
            _ => vec![], // Pawn, Lance, Knight, Rook, Bishop are handled by sliding logic
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn new(row: u8, col: u8) -> Self {
        debug_assert!(row < 9 && col < 9);
        Self { row, col }
    }

    pub fn from_u8(value: u8) -> Self {
        let row = value / 9;
        let col = value % 9;
        Self::new(row, col)
    }

    pub fn to_u8(self) -> u8 {
        self.row * 9 + self.col
    }

    pub fn is_valid(self) -> bool {
        self.row < 9 && self.col < 9
    }

    pub fn distance_to(self, other: Position) -> u8 {
        let dr = if self.row > other.row { self.row - other.row } else { other.row - self.row };
        let dc = if self.col > other.col { self.col - other.col } else { other.col - self.col };
        dr + dc
    }

    pub fn is_in_promotion_zone(self, player: Player) -> bool {
        match player {
            Player::Black => self.row >= 6,
            Player::White => self.row <= 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}

impl Piece {
    pub fn new(piece_type: PieceType, player: Player) -> Self {
        Self { piece_type, player }
    }

    pub fn value(self) -> i32 {
        self.piece_type.base_value()
    }

    pub fn unpromoted(self) -> Self {
        if let Some(unpromoted_type) = self.piece_type.unpromoted_version() {
            Piece::new(unpromoted_type, self.player)
        } else {
            self
        }
    }

    pub fn to_fen_char(&self) -> String {
        let mut fen_char = match self.piece_type {
            PieceType::Pawn => "p",
            PieceType::Lance => "l",
            PieceType::Knight => "n",
            PieceType::Silver => "s",
            PieceType::Gold => "g",
            PieceType::Bishop => "b",
            PieceType::Rook => "r",
            PieceType::King => "k",
            PieceType::PromotedPawn => "+p",
            PieceType::PromotedLance => "+l",
            PieceType::PromotedKnight => "+n",
            PieceType::PromotedSilver => "+s",
            PieceType::PromotedBishop => "+b",
            PieceType::PromotedRook => "+r",
        }.to_string();

        if self.player == Player::Black {
            fen_char = fen_char.to_uppercase();
        }

        fen_char
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Move {
    pub from: Option<Position>,  // None for drops
    pub to: Position,
    pub piece_type: PieceType,
    pub player: Player,
    pub is_promotion: bool,
    pub is_capture: bool,
    pub captured_piece: Option<Piece>,
}

impl Move {
    pub fn new_move(from: Position, to: Position, piece_type: PieceType, player: Player, promote: bool) -> Self {
        Self {
            from: Some(from),
            to,
            piece_type,
            player,
            is_promotion: promote,
            is_capture: false,
            captured_piece: None,
        }
    }

    pub fn new_drop(piece_type: PieceType, to: Position, player: Player) -> Self {
        Self {
            from: None,
            to,
            piece_type,
            player,
            is_promotion: false,
            is_capture: false,
            captured_piece: None,
        }
    }

    pub fn is_drop(&self) -> bool {
        self.from.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedPieces {
    pub black: Vec<PieceType>,
    pub white: Vec<PieceType>,
}

impl CapturedPieces {
    pub fn new() -> Self {
        Self {
            black: Vec::new(),
            white: Vec::new(),
        }
    }

    pub fn add_piece(&mut self, piece_type: PieceType, player: Player) {
        match player {
            Player::Black => self.black.push(piece_type),
            Player::White => self.white.push(piece_type),
        }
    }

    pub fn remove_piece(&mut self, piece_type: PieceType, player: Player) -> bool {
        let pieces = match player {
            Player::Black => &mut self.black,
            Player::White => &mut self.white,
        };
        
        if let Some(index) = pieces.iter().position(|&p| p == piece_type) {
            pieces.remove(index);
            true
        } else {
            false
        }
    }

    pub fn count(&self, piece_type: PieceType, player: Player) -> usize {
        let pieces = match player {
            Player::Black => &self.black,
            Player::White => &self.white,
        };
        pieces.iter().filter(|&&p| p == piece_type).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspositionEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

// Bitboard representation for efficient operations
pub type Bitboard = u128;  // 81 squares need 81 bits, u128 gives us 128 bits

pub const EMPTY_BITBOARD: Bitboard = 0;
pub const ALL_SQUARES: Bitboard = 0x1FFFFFFFFFFFFFFFFFFFFFFFF;  // 81 bits set

// Bitboard utilities
pub fn set_bit(bitboard: &mut Bitboard, position: Position) {
    *bitboard |= 1 << position.to_u8();
}

pub fn clear_bit(bitboard: &mut Bitboard, position: Position) {
    *bitboard &= !(1 << position.to_u8());
}

pub fn is_bit_set(bitboard: Bitboard, position: Position) -> bool {
    (bitboard & (1 << position.to_u8())) != 0
}

pub fn count_bits(bitboard: Bitboard) -> u32 {
    bitboard.count_ones()
}

pub fn get_lsb(bitboard: Bitboard) -> Option<Position> {
    if bitboard == 0 {
        None
    } else {
        let lsb = bitboard.trailing_zeros() as u8;
        Some(Position::from_u8(lsb))
    }
}

pub fn pop_lsb(bitboard: &mut Bitboard) -> Option<Position> {
    if let Some(pos) = get_lsb(*bitboard) {
        *bitboard &= *bitboard - 1;
        Some(pos)
    } else {
        None
    }
}
