use std::collections::HashMap;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn opposite(&self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'l' => Some(PieceType::Lance),
            'n' => Some(PieceType::Knight),
            's' => Some(PieceType::Silver),
            'g' => Some(PieceType::Gold),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }

    pub fn to_char(&self, player: &Player) -> char {
        let c = match self {
            PieceType::Pawn => 'p',
            PieceType::Lance => 'l',
            PieceType::Knight => 'n',
            PieceType::Silver => 's',
            PieceType::Gold => 'g',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::King => 'k',
            // Promoted pieces should be handled by the caller, e.g. by adding a '+'
            promoted_piece if promoted_piece.is_promoted() => {
                return promoted_piece.unpromoted_version().to_char(player);
            }
            _ => panic!("to_char called on an unexpected piece type"),
        };
        if player == &Player::Black { c.to_ascii_uppercase() } else { c }
    }

    pub fn is_promoted(&self) -> bool {
        matches!(self, PieceType::PromotedPawn | PieceType::PromotedLance | PieceType::PromotedKnight | PieceType::PromotedSilver | PieceType::PromotedBishop | PieceType::PromotedRook)
    }

    pub fn promoted_version(&self) -> Option<Self> {
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

    pub fn unpromoted_version(&self) -> Self {
        match self {
            PieceType::PromotedPawn => PieceType::Pawn,
            PieceType::PromotedLance => PieceType::Lance,
            PieceType::PromotedKnight => PieceType::Knight,
            PieceType::PromotedSilver => PieceType::Silver,
            PieceType::PromotedBishop => PieceType::Bishop,
            PieceType::PromotedRook => PieceType::Rook,
            _ => *self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: u8,
    pub col: u8,
}

impl Position {
    pub fn new(row: u8, col: u8) -> Self {
        Position { row, col }
    }

    pub fn from_usi(s: &str) -> Result<Self> {
        if s.len() != 2 { return Err(anyhow!("Invalid USI position string length: {}", s)); }
        let mut chars = s.chars();
        let col_char = chars.next().unwrap();
        let row_char = chars.next().unwrap();

        let col = match col_char {
            '1' => 8,
            '2' => 7,
            '3' => 6,
            '4' => 5,
            '5' => 4,
            '6' => 3,
            '7' => 2,
            '8' => 1,
            '9' => 0,
            _ => return Err(anyhow!("Invalid USI column character: {}", col_char)),
        };
        let row = match row_char {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            'i' => 8,
            _ => return Err(anyhow!("Invalid USI row character: {}", row_char)),
        };
        Ok(Position::new(row, col))
    }

    pub fn to_usi(&self) -> String {
        let col_char = match self.col {
            0 => '9',
            1 => '8',
            2 => '7',
            3 => '6',
            4 => '5',
            5 => '4',
            6 => '3',
            7 => '2',
            8 => '1',
            _ => unreachable!(),
        };
        let row_char = match self.row {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            8 => 'i',
            _ => unreachable!(),
        };
        format!("{}{}", col_char, row_char)
    }
}

pub struct BoardState {
    pub board: [[Option<Piece>; 9]; 9],
    pub current_player: Player,
    pub black_captured: HashMap<PieceType, u8>,
    pub white_captured: HashMap<PieceType, u8>,
}

impl BoardState {
    pub fn new() -> Self {
        BoardState {
            board: [[None; 9]; 9],
            current_player: Player::Black,
            black_captured: HashMap::new(),
            white_captured: HashMap::new(),
        }
    }

    pub fn parse_fen(fen: &str) -> Result<Self> {
        let mut board_state = BoardState::new();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 3 { return Err(anyhow!("Invalid FEN string: not enough parts")); }

        // 1. Parse board state
        let board_part = parts[0];
        for (r, rank_str) in board_part.split('/').enumerate() {
            let mut c = 0;
            let mut chars = rank_str.chars().peekable();
            while let Some(ch) = chars.next() {
                if let Some(digit) = ch.to_digit(10) {
                    c += digit as usize;
                } else {
                    let is_promoted = ch == '+';
                    let piece_char = if is_promoted { chars.next().unwrap_or(' ') } else { ch };
                    let player = if piece_char.is_uppercase() { Player::Black } else { Player::White };
                    let mut piece_type = PieceType::from_char(piece_char).unwrap();
                    if is_promoted { piece_type = piece_type.promoted_version().unwrap(); }
                    board_state.board[r][c] = Some(Piece { piece_type, player });
                    c += 1;
                }
            }
        }

        // 2. Parse side to move
        board_state.current_player = match parts[1] {
            "b" => Player::Black,
            "w" => Player::White,
            _ => return Err(anyhow!("Invalid FEN: invalid player")),
        };

        // 3. Parse pieces in hand
        if parts[2] != "-" {
            let mut count = 1;
            for ch in parts[2].chars() {
                if let Some(digit) = ch.to_digit(10) {
                    count = digit as u8;
                } else {
                    let player = if ch.is_uppercase() { Player::Black } else { Player::White };
                    let piece_type = PieceType::from_char(ch).unwrap();
                    let captured_map = if player == Player::Black { &mut board_state.black_captured } else { &mut board_state.white_captured };
                    for _ in 0..count {
                        *captured_map.entry(piece_type).or_insert(0) += 1;
                    }
                    count = 1;
                }
            }
        }
        Ok(board_state)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(128);
        for r in 0..9 {
            let mut empty_squares = 0;
            for c in 0..9 {
                if let Some(piece) = &self.board[r][c] {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    if piece.piece_type.is_promoted() {
                        fen.push('+');
                    }
                    fen.push(piece.piece_type.to_char(&piece.player));
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if r < 8 { fen.push('/'); }
        }
        fen.push(' ');
        fen.push(if self.current_player == Player::Black { 'b' } else { 'w' });
        fen.push(' ');

        let mut captured_str = String::new();
        for (piece_type, &count) in &self.black_captured {
            if count > 0 {
                if count > 1 { captured_str.push_str(&count.to_string()); }
                captured_str.push(piece_type.to_char(&Player::Black));
            }
        }
        for (piece_type, &count) in &self.white_captured {
            if count > 0 {
                if count > 1 { captured_str.push_str(&count.to_string()); }
                captured_str.push(piece_type.to_char(&Player::White));
            }
        }

        if captured_str.is_empty() { fen.push('-'); } else { fen.push_str(&captured_str); }
        fen.push_str(" 1"); // Move clock, always 1 for simplicity in this harness
        fen
    }

    pub fn apply_move(&mut self, usi_move_str: &str) -> Result<()> {
        if usi_move_str == "resign" { return Ok(()); }

        let is_drop = usi_move_str.contains("*");
        let is_promotion = usi_move_str.ends_with("+");

        if is_drop {
            // Example: P*2c
            let parts: Vec<&str> = usi_move_str.split('*').collect();
            if parts.len() != 2 { return Err(anyhow!("Invalid drop move format: {}", usi_move_str)); }
            let piece_char = parts[0].chars().next().ok_or_else(|| anyhow!("Missing piece for drop"))?;
            let piece_type = PieceType::from_char(piece_char).ok_or_else(|| anyhow!("Invalid piece type for drop: {}", piece_char))?;
            let to_pos = Position::from_usi(parts[1])?;

            let captured_map = match self.current_player {
                Player::Black => &mut self.black_captured,
                Player::White => &mut self.white_captured,
            };

            let count = captured_map.entry(piece_type).or_insert(0);
            if *count == 0 { 
                eprintln!("Error: Cannot drop piece {:?}: not in hand. Captured map: {:?}", piece_type, captured_map);
                return Err(anyhow!("Cannot drop piece {:?}: not in hand", piece_type)); 
            }
            *count -= 1;

            if self.board[to_pos.row as usize][to_pos.col as usize].is_some() {
                return Err(anyhow!("Cannot drop piece on occupied square: {}", to_pos.to_usi()));
            }

            self.board[to_pos.row as usize][to_pos.col as usize] = Some(Piece { piece_type, player: self.current_player });
        } else {
            // Example: 7g7f, 2c2d+
            let from_usi = &usi_move_str[0..2];
            let to_usi = &usi_move_str[2..4];
            let from_pos = Position::from_usi(from_usi)?;
            let to_pos = Position::from_usi(to_usi)?;

            let mut piece = self.board[from_pos.row as usize][from_pos.col as usize].take()
                .ok_or_else(|| anyhow!("No piece at from position: {}", from_pos.to_usi()))?;

            if piece.player != self.current_player {
                return Err(anyhow!("Moving opponent's piece: {:?}", piece));
            }

            // Handle capture
            if let Some(captured_piece) = self.board[to_pos.row as usize][to_pos.col as usize].take() {
                let captured_map = match self.current_player {
                    Player::Black => &mut self.black_captured,
                    Player::White => &mut self.white_captured,
                };
                *captured_map.entry(captured_piece.piece_type.unpromoted_version()).or_insert(0) += 1;
            }

            // Handle promotion
            if is_promotion {
                piece.piece_type = piece.piece_type.promoted_version()
                    .ok_or_else(|| anyhow!("Cannot promote piece type: {:?}", piece.piece_type))?;
            }

            self.board[to_pos.row as usize][to_pos.col as usize] = Some(piece);
        }

        self.current_player = self.current_player.opposite();
        Ok(())
    }
}
