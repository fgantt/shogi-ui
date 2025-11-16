//! Core Domain Types
//!
//! This module contains the fundamental domain types for shogi: Player, PieceType, Position, Piece, and Move.
//! Extracted from `types.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub const COUNT: usize = 14;

    pub const fn as_index(self) -> usize {
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
        matches!(
            self,
            PieceType::Pawn
                | PieceType::Lance
                | PieceType::Knight
                | PieceType::Silver
                | PieceType::Bishop
                | PieceType::Rook
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
            PieceType::Silver => vec![
                (direction, 0),
                (direction, -1),
                (direction, 1),
                (-direction, -1),
                (-direction, 1),
            ],
            PieceType::Gold
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => vec![
                (direction, 0),
                (direction, -1),
                (direction, 1),
                (0, -1),
                (0, 1),
                (-direction, 0),
            ],
            PieceType::King => vec![
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            PieceType::PromotedBishop => vec![
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
            ],
            PieceType::PromotedRook => vec![
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            _ => vec![], // Pawn, Lance, Knight, Rook, Bishop are handled by sliding logic
        }
    }
}

/// Board coordinate (0-based row/col). `Display` renders USI-like `"7f"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn new(row: u8, col: u8) -> Self {
        // Clamp coordinates to valid range for platform compatibility
        let row = if row >= 9 { 8 } else { row };
        let col = if col >= 9 { 8 } else { col };
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

    pub fn to_index(self) -> u8 {
        self.to_u8()
    }

    /// Create a Position from a 0-based index (0-80)
    pub fn from_index(index: u8) -> Self {
        Self {
            row: index / 9,
            col: index % 9,
        }
    }

    pub fn is_valid(self) -> bool {
        self.row < 9 && self.col < 9
    }

    pub fn distance_to(self, other: Position) -> u8 {
        let dr = if self.row > other.row {
            self.row - other.row
        } else {
            other.row - self.row
        };
        let dc = if self.col > other.col {
            self.col - other.col
        } else {
            other.col - self.col
        };
        dr + dc
    }

    pub fn is_in_promotion_zone(self, player: Player) -> bool {
        match player {
            Player::Black => self.row >= 6,
            Player::White => self.row <= 2,
        }
    }

    pub fn from_usi_string(usi_str: &str) -> Result<Position, &str> {
        if usi_str.len() != 2 {
            return Err("Invalid position string");
        }
        let mut chars = usi_str.chars();
        let file_char = chars.next().ok_or("Invalid position string")?;
        let rank_char = chars.next().ok_or("Invalid position string")?;

        let col = 9 - (file_char.to_digit(10).ok_or("Invalid file")? as u8);
        let row = (rank_char as u8) - b'a';

        if col > 8 || row > 8 {
            return Err("Position out of bounds");
        }
        Ok(Position::new(row, col))
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // USI-like: file is 9-col, rank is 'a'+row
        let file = 9 - self.col;
        let rank = (b'a' + self.row) as char;
        write!(f, "{}{}", file, rank)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}

impl Piece {
    pub fn new(piece_type: PieceType, player: Player) -> Self {
        // Validate that piece_type is valid by checking its to_u8() value
        let piece_idx = piece_type.to_u8();
        if piece_idx >= 14 {
            // This should never happen with valid PieceType, but protect against corruption
            crate::utils::telemetry::debug_log(&format!(
                "[PIECE::NEW ERROR] Invalid piece_type with to_u8() = {}. Defaulting to Pawn.",
                piece_idx
            ));
            return Self {
                piece_type: PieceType::Pawn,
                player,
            };
        }
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
        }
        .to_string();

        if self.player == Player::Black {
            fen_char = fen_char.to_uppercase();
        }

        fen_char
    }
}

/// A move in USI terms. `Display` delegates to `to_usi_string()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    pub from: Option<Position>, // None for drops
    pub to: Position,
    pub piece_type: PieceType,
    pub player: Player,
    pub is_promotion: bool,
    pub is_capture: bool,
    pub captured_piece: Option<Piece>,
    pub gives_check: bool,  // Whether this move gives check
    pub is_recapture: bool, // Whether this is a recapture move
}

impl Move {
    #[allow(dead_code)]
    pub fn new(
        from: Option<Position>,
        to: Position,
        piece_type: PieceType,
        is_capture: bool,
        is_promotion: bool,
        gives_check: bool,
        is_recapture: bool,
    ) -> Self {
        Self {
            from,
            to,
            piece_type,
            player: Player::Black,
            is_promotion,
            is_capture,
            captured_piece: None,
            gives_check,
            is_recapture,
        }
    }

    pub fn new_move(
        from: Position,
        to: Position,
        piece_type: PieceType,
        player: Player,
        promote: bool,
    ) -> Self {
        Self {
            from: Some(from),
            to,
            piece_type,
            player,
            is_promotion: promote,
            is_capture: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
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
            gives_check: false,
            is_recapture: false,
        }
    }

    pub fn is_drop(&self) -> bool {
        self.from.is_none()
    }

    pub fn from_usi_string(
        usi_str: &str,
        player: Player,
        board: &crate::bitboards::BitboardBoard,
    ) -> Result<Move, &'static str> {
        if usi_str.len() < 4 {
            return Err("Invalid USI move string length");
        }

        if usi_str.contains('*') {
            // Drop move, e.g., "P*5e"
            let parts: Vec<&str> = usi_str.split('*').collect();
            if parts.len() != 2 {
                return Err("Invalid drop move format");
            }

            let piece_type = match parts[0] {
                "P" => PieceType::Pawn,
                "L" => PieceType::Lance,
                "N" => PieceType::Knight,
                "S" => PieceType::Silver,
                "G" => PieceType::Gold,
                "B" => PieceType::Bishop,
                "R" => PieceType::Rook,
                _ => return Err("Invalid piece type for drop"),
            };

            let to =
                Position::from_usi_string(parts[1]).map_err(|_| "Invalid position in drop move")?;
            Ok(Move::new_drop(piece_type, to, player))
        } else {
            // Normal move, e.g., "7g7f" or "2b8h+"
            let from_str = &usi_str[0..2];
            let to_str = &usi_str[2..4];
            let is_promotion = usi_str.ends_with('+');

            let from = Position::from_usi_string(from_str).map_err(|_| "Invalid from position")?;
            let to = Position::from_usi_string(to_str).map_err(|_| "Invalid to position")?;

            let piece_to_move = board.get_piece(from).ok_or("No piece at source square")?;
            if piece_to_move.player != player {
                return Err("Attempting to move opponent's piece");
            }

            let mut mv = Move::new_move(from, to, piece_to_move.piece_type, player, is_promotion);

            if board.is_square_occupied(to) {
                mv.is_capture = true;
            }

            Ok(mv)
        }
    }

    pub fn to_usi_string(&self) -> String {
        if let Some(from_pos) = self.from {
            // Standard move or promotion
            let from_str = format!("{}{}", 9 - from_pos.col, (b'a' + from_pos.row) as char);
            let to_str = format!("{}{}", 9 - self.to.col, (b'a' + self.to.row) as char);
            let promotion_str = if self.is_promotion { "+" } else { "" };
            format!("{}{}{}", from_str, to_str, promotion_str)
        } else {
            // Drop
            let piece_char = match self.piece_type {
                PieceType::Pawn => "P",
                PieceType::Lance => "L",
                PieceType::Knight => "N",
                PieceType::Silver => "S",
                PieceType::Gold => "G",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                _ => "", // Should not happen for a drop
            };
            let to_str = format!("{}{}", 9 - self.to.col, (b'a' + self.to.row) as char);
            format!("{}*{}", piece_char, to_str)
        }
    }

    /// Get the value of the captured piece in this move
    pub fn captured_piece_value(&self) -> i32 {
        if let Some(ref captured) = self.captured_piece {
            captured.piece_type.base_value()
        } else {
            0
        }
    }

    /// Get the value of the piece being moved
    pub fn piece_value(&self) -> i32 {
        self.piece_type.base_value()
    }

    /// Get the promotion value bonus for this move
    pub fn promotion_value(&self) -> i32 {
        if self.is_promotion {
            // Calculate the difference between promoted and unpromoted piece values
            let promoted_value = self.piece_type.base_value();
            if let Some(unpromoted_type) = self.piece_type.unpromoted_version() {
                let unpromoted_value = unpromoted_type.base_value();
                promoted_value - unpromoted_value
            } else {
                0
            }
        } else {
            0
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_usi_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;

    #[test]
    fn test_player_opposite() {
        assert_eq!(Player::Black.opposite(), Player::White);
        assert_eq!(Player::White.opposite(), Player::Black);
    }

    #[test]
    fn test_piece_type_conversion() {
        assert_eq!(PieceType::Pawn.to_u8(), 0);
        assert_eq!(PieceType::from_u8(0), PieceType::Pawn);
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 3);
        assert_eq!(pos.row, 5);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn test_move_creation() {
        let from = Position::new(6, 6);
        let to = Position::new(5, 6);
        let mv = Move::new_move(from, to, PieceType::Pawn, Player::Black, false);
        assert_eq!(mv.from, Some(from));
        assert_eq!(mv.to, to);
    }

    #[test]
    fn test_position_display_usi_like() {
        // (row=5,col=2) => file = 9-2 = 7, rank = 'a'+5 = 'f' => "7f"
        let pos = Position::new(5, 2);
        assert_eq!(pos.to_string(), "7f");
    }

    #[test]
    fn test_move_display_usi_string() {
        let board = BitboardBoard::new();
        let from = Position::new(6, 6);
        let to = Position::new(5, 6);
        let mv = Move::new_move(from, to, PieceType::Pawn, Player::Black, false);
        // 6,6 => file 9-6=3; rank 'a'+6='g' so "3g3f"
        assert!(mv.to_string().ends_with("3f"));
        let parsed = Move::from_usi_string(&mv.to_string(), Player::Black, &board);
        assert!(parsed.is_ok());
    }
}

