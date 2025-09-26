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
        // Clamp coordinates to valid range for WASM compatibility
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

    pub fn from_usi_string(usi_str: &str) -> Result<Position, &str> {
        if usi_str.len() != 2 { return Err("Invalid position string"); }
        let mut chars = usi_str.chars();
        let file_char = chars.next().ok_or("Invalid position string")?;
        let rank_char = chars.next().ok_or("Invalid position string")?;

        let col = 9 - (file_char.to_digit(10).ok_or("Invalid file")? as u8);
        let row = (rank_char as u8) - b'a';

        if col > 8 || row > 8 { return Err("Position out of bounds"); }
        Ok(Position::new(row, col))
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
    pub gives_check: bool,       // Whether this move gives check
    pub is_recapture: bool,      // Whether this is a recapture move
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

    pub fn from_usi_string(usi_str: &str, player: Player, board: &crate::bitboards::BitboardBoard) -> Result<Move, &'static str> {
        if usi_str.len() < 4 {
            return Err("Invalid USI move string length");
        }

        if usi_str.contains('*') {
            // Drop move, e.g., "P*5e"
            let parts: Vec<&str> = usi_str.split('*').collect();
            if parts.len() != 2 { return Err("Invalid drop move format"); }
            
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

            let to = Position::from_usi_string(parts[1]).map_err(|_| "Invalid position in drop move")?;
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

/// Transposition table entry specifically for quiescence search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiescenceEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
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

/// Configuration for quiescence search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiescenceConfig {
    pub max_depth: u8,                    // Maximum quiescence depth
    pub enable_delta_pruning: bool,       // Enable delta pruning
    pub enable_futility_pruning: bool,    // Enable futility pruning
    pub enable_selective_extensions: bool, // Enable selective extensions
    pub enable_tt: bool,                 // Enable transposition table
    pub futility_margin: i32,            // Futility pruning margin
    pub delta_margin: i32,               // Delta pruning margin
    pub tt_size_mb: usize,               // Quiescence TT size in MB
    pub tt_cleanup_threshold: usize,     // Threshold for TT cleanup
}

impl Default for QuiescenceConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            enable_delta_pruning: true,
            enable_futility_pruning: true,
            enable_selective_extensions: true,
            enable_tt: true,
            futility_margin: 200,
            delta_margin: 100,
            tt_size_mb: 4,                // 4MB for quiescence TT
            tt_cleanup_threshold: 10000,  // Clean up when TT has 10k entries
        }
    }
}

impl QuiescenceConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.max_depth == 0 {
            return Err("max_depth must be greater than 0".to_string());
        }
        if self.max_depth > 20 {
            return Err("max_depth should not exceed 20 for performance reasons".to_string());
        }
        if self.futility_margin < 0 {
            return Err("futility_margin must be non-negative".to_string());
        }
        if self.futility_margin > 1000 {
            return Err("futility_margin should not exceed 1000".to_string());
        }
        if self.delta_margin < 0 {
            return Err("delta_margin must be non-negative".to_string());
        }
        if self.delta_margin > 1000 {
            return Err("delta_margin should not exceed 1000".to_string());
        }
        if self.tt_size_mb == 0 {
            return Err("tt_size_mb must be greater than 0".to_string());
        }
        if self.tt_size_mb > 1024 {
            return Err("tt_size_mb should not exceed 1024MB".to_string());
        }
        if self.tt_cleanup_threshold == 0 {
            return Err("tt_cleanup_threshold must be greater than 0".to_string());
        }
        if self.tt_cleanup_threshold > 1000000 {
            return Err("tt_cleanup_threshold should not exceed 1,000,000".to_string());
        }
        Ok(())
    }

    /// Create a validated configuration, clamping values to valid ranges
    pub fn new_validated(mut self) -> Self {
        self.max_depth = self.max_depth.clamp(1, 20);
        self.futility_margin = self.futility_margin.clamp(0, 1000);
        self.delta_margin = self.delta_margin.clamp(0, 1000);
        self.tt_size_mb = self.tt_size_mb.clamp(1, 1024);
        self.tt_cleanup_threshold = self.tt_cleanup_threshold.clamp(1, 1000000);
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "QuiescenceConfig: depth={}, delta_pruning={}, futility_pruning={}, extensions={}, tt={}, tt_size={}MB, cleanup_threshold={}",
            self.max_depth,
            self.enable_delta_pruning,
            self.enable_futility_pruning,
            self.enable_selective_extensions,
            self.enable_tt,
            self.tt_size_mb,
            self.tt_cleanup_threshold
        )
    }
}

/// Performance statistics for quiescence search
#[derive(Debug, Clone, Default)]
pub struct QuiescenceStats {
    pub nodes_searched: u64,
    pub delta_prunes: u64,
    pub futility_prunes: u64,
    pub extensions: u64,
    pub tt_hits: u64,
    pub tt_misses: u64,
    pub moves_ordered: u64,
    pub check_moves_found: u64,
    pub capture_moves_found: u64,
    pub promotion_moves_found: u64,
}

impl QuiescenceStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = QuiescenceStats::default();
    }

    /// Get the total number of pruning operations
    pub fn total_prunes(&self) -> u64 {
        self.delta_prunes + self.futility_prunes
    }

    /// Get the pruning efficiency as a percentage
    pub fn pruning_efficiency(&self) -> f64 {
        if self.nodes_searched == 0 {
            return 0.0;
        }
        (self.total_prunes() as f64 / self.nodes_searched as f64) * 100.0
    }

    /// Get the transposition table hit rate as a percentage
    pub fn tt_hit_rate(&self) -> f64 {
        let total_tt_attempts = self.tt_hits + self.tt_misses;
        if total_tt_attempts == 0 {
            return 0.0;
        }
        (self.tt_hits as f64 / total_tt_attempts as f64) * 100.0
    }

    /// Get the extension rate as a percentage
    pub fn extension_rate(&self) -> f64 {
        if self.nodes_searched == 0 {
            return 0.0;
        }
        (self.extensions as f64 / self.nodes_searched as f64) * 100.0
    }

    /// Get move type distribution
    pub fn move_type_distribution(&self) -> (f64, f64, f64) {
        let total_moves = self.check_moves_found + self.capture_moves_found + self.promotion_moves_found;
        if total_moves == 0 {
            return (0.0, 0.0, 0.0);
        }
        let check_pct = (self.check_moves_found as f64 / total_moves as f64) * 100.0;
        let capture_pct = (self.capture_moves_found as f64 / total_moves as f64) * 100.0;
        let promotion_pct = (self.promotion_moves_found as f64 / total_moves as f64) * 100.0;
        (check_pct, capture_pct, promotion_pct)
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        let (check_pct, capture_pct, promotion_pct) = self.move_type_distribution();
        format!(
            "Quiescence Performance Report:\n\
            - Nodes searched: {}\n\
            - Pruning efficiency: {:.2}% ({} prunes)\n\
            - TT hit rate: {:.2}% ({} hits, {} misses)\n\
            - Extension rate: {:.2}% ({} extensions)\n\
            - Move distribution: {:.1}% checks, {:.1}% captures, {:.1}% promotions\n\
            - Moves ordered: {}",
            self.nodes_searched,
            self.pruning_efficiency(),
            self.total_prunes(),
            self.tt_hit_rate(),
            self.tt_hits,
            self.tt_misses,
            self.extension_rate(),
            self.extensions,
            check_pct,
            capture_pct,
            promotion_pct,
            self.moves_ordered
        )
    }

    /// Get a summary of key metrics
    pub fn summary(&self) -> String {
        format!(
            "QSearch: {} nodes, {:.1}% pruned, {:.1}% TT hits, {:.1}% extended",
            self.nodes_searched,
            self.pruning_efficiency(),
            self.tt_hit_rate(),
            self.extension_rate()
        )
    }
}

/// Performance sample for quiescence search profiling
#[derive(Debug, Clone)]
pub struct QuiescenceSample {
    pub iteration: usize,
    pub duration_ms: u64,
    pub nodes_searched: u64,
    pub moves_ordered: u64,
    pub delta_prunes: u64,
    pub futility_prunes: u64,
    pub extensions: u64,
    pub tt_hits: u64,
    pub tt_misses: u64,
    pub check_moves: u64,
    pub capture_moves: u64,
    pub promotion_moves: u64,
}

/// Performance profile for quiescence search
#[derive(Debug, Clone)]
pub struct QuiescenceProfile {
    pub samples: Vec<QuiescenceSample>,
    pub average_duration_ms: f64,
    pub average_nodes_searched: f64,
    pub average_pruning_efficiency: f64,
    pub average_tt_hit_rate: f64,
    pub average_extension_rate: f64,
}

impl QuiescenceProfile {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            average_duration_ms: 0.0,
            average_nodes_searched: 0.0,
            average_pruning_efficiency: 0.0,
            average_tt_hit_rate: 0.0,
            average_extension_rate: 0.0,
        }
    }

    pub fn add_sample(&mut self, sample: QuiescenceSample) {
        self.samples.push(sample);
        self.update_averages();
    }

    fn update_averages(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        let total_duration: u64 = self.samples.iter().map(|s| s.duration_ms).sum();
        let total_nodes: u64 = self.samples.iter().map(|s| s.nodes_searched).sum();
        let total_prunes: u64 = self.samples.iter().map(|s| s.delta_prunes + s.futility_prunes).sum();
        let total_tt_attempts: u64 = self.samples.iter().map(|s| s.tt_hits + s.tt_misses).sum();
        let total_extensions: u64 = self.samples.iter().map(|s| s.extensions).sum();

        self.average_duration_ms = total_duration as f64 / self.samples.len() as f64;
        self.average_nodes_searched = total_nodes as f64 / self.samples.len() as f64;
        self.average_pruning_efficiency = if total_nodes > 0 {
            (total_prunes as f64 / total_nodes as f64) * 100.0
        } else { 0.0 };
        self.average_tt_hit_rate = if total_tt_attempts > 0 {
            (self.samples.iter().map(|s| s.tt_hits).sum::<u64>() as f64 / total_tt_attempts as f64) * 100.0
        } else { 0.0 };
        self.average_extension_rate = if total_nodes > 0 {
            (total_extensions as f64 / total_nodes as f64) * 100.0
        } else { 0.0 };
    }

    pub fn get_performance_report(&self) -> String {
        format!(
            "Quiescence Performance Profile:\n\
            - Samples: {}\n\
            - Average Duration: {:.2}ms\n\
            - Average Nodes: {:.0}\n\
            - Average Pruning Efficiency: {:.2}%\n\
            - Average TT Hit Rate: {:.2}%\n\
            - Average Extension Rate: {:.2}%",
            self.samples.len(),
            self.average_duration_ms,
            self.average_nodes_searched,
            self.average_pruning_efficiency,
            self.average_tt_hit_rate,
            self.average_extension_rate
        )
    }
}

/// Detailed performance metrics for quiescence search
#[derive(Debug, Clone)]
pub struct QuiescencePerformanceMetrics {
    pub nodes_per_second: f64,
    pub pruning_efficiency: f64,
    pub tt_hit_rate: f64,
    pub extension_rate: f64,
    pub move_ordering_efficiency: f64,
    pub tactical_move_ratio: f64,
}

impl QuiescencePerformanceMetrics {
    pub fn summary(&self) -> String {
        format!(
            "Performance Metrics: {:.0} nodes/s, {:.1}% pruned, {:.1}% TT hits, {:.1}% extended, {:.1}% tactical",
            self.nodes_per_second,
            self.pruning_efficiency,
            self.tt_hit_rate,
            self.extension_rate,
            self.tactical_move_ratio
        )
    }
}

/// Configuration for null move pruning parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullMoveConfig {
    pub enabled: bool,                      // Enable null move pruning
    pub min_depth: u8,                      // Minimum depth to use NMP
    pub reduction_factor: u8,               // Static reduction factor (R)
    pub max_pieces_threshold: u8,           // Disable NMP when pieces < threshold
    pub enable_dynamic_reduction: bool,     // Use dynamic R = 2 + depth/6
    pub enable_endgame_detection: bool,     // Disable NMP in endgame
}

impl Default for NullMoveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 3,
            reduction_factor: 2,
            max_pieces_threshold: 12,       // Disable when < 12 pieces
            enable_dynamic_reduction: true,
            enable_endgame_detection: true,
        }
    }
}

impl NullMoveConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.min_depth > 10 {
            return Err("min_depth should not exceed 10 for performance reasons".to_string());
        }
        if self.reduction_factor == 0 {
            return Err("reduction_factor must be greater than 0".to_string());
        }
        if self.reduction_factor > 5 {
            return Err("reduction_factor should not exceed 5".to_string());
        }
        if self.max_pieces_threshold == 0 {
            return Err("max_pieces_threshold must be greater than 0".to_string());
        }
        if self.max_pieces_threshold > 40 {
            return Err("max_pieces_threshold should not exceed 40".to_string());
        }
        Ok(())
    }

    /// Create a validated configuration, clamping values to valid ranges
    pub fn new_validated(mut self) -> Self {
        self.min_depth = self.min_depth.clamp(1, 10);
        self.reduction_factor = self.reduction_factor.clamp(1, 5);
        self.max_pieces_threshold = self.max_pieces_threshold.clamp(1, 40);
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "NullMoveConfig: enabled={}, min_depth={}, reduction_factor={}, max_pieces_threshold={}, dynamic_reduction={}, endgame_detection={}",
            self.enabled,
            self.min_depth,
            self.reduction_factor,
            self.max_pieces_threshold,
            self.enable_dynamic_reduction,
            self.enable_endgame_detection
        )
    }
}

/// Performance statistics for null move pruning
#[derive(Debug, Clone, Default)]
pub struct NullMoveStats {
    pub attempts: u64,                      // Number of null move attempts
    pub cutoffs: u64,                       // Number of successful cutoffs
    pub depth_reductions: u64,              // Total depth reductions applied
    pub disabled_in_check: u64,             // Times disabled due to check
    pub disabled_endgame: u64,              // Times disabled due to endgame
}

impl NullMoveStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = NullMoveStats::default();
    }

    /// Get the cutoff rate as a percentage
    pub fn cutoff_rate(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        (self.cutoffs as f64 / self.attempts as f64) * 100.0
    }

    /// Get the average reduction factor
    pub fn average_reduction_factor(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        self.depth_reductions as f64 / self.attempts as f64
    }

    /// Get the total number of disabled attempts
    pub fn total_disabled(&self) -> u64 {
        self.disabled_in_check + self.disabled_endgame
    }

    /// Get the efficiency of null move pruning as a percentage
    pub fn efficiency(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        (self.cutoffs as f64 / (self.attempts + self.total_disabled()) as f64) * 100.0
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Null Move Pruning Performance Report:\n\
            - Attempts: {}\n\
            - Cutoffs: {} ({:.2}%)\n\
            - Total disabled: {} ({} in check, {} endgame)\n\
            - Average reduction: {:.2}\n\
            - Efficiency: {:.2}%",
            self.attempts,
            self.cutoffs,
            self.cutoff_rate(),
            self.total_disabled(),
            self.disabled_in_check,
            self.disabled_endgame,
            self.average_reduction_factor(),
            self.efficiency()
        )
    }

    /// Get a summary of key metrics
    pub fn summary(&self) -> String {
        format!(
            "NMP: {} attempts, {:.1}% cutoffs, {:.1}% efficiency, {:.1} avg reduction",
            self.attempts,
            self.cutoff_rate(),
            self.efficiency(),
            self.average_reduction_factor()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;

    #[test]
    fn test_position_from_usi() {
        assert_eq!(Position::from_usi_string("1a").unwrap(), Position::new(0, 8));
        assert_eq!(Position::from_usi_string("5e").unwrap(), Position::new(4, 4));
        assert_eq!(Position::from_usi_string("9i").unwrap(), Position::new(8, 0));
        assert!(Position::from_usi_string("0a").is_err());
        assert!(Position::from_usi_string("1j").is_err());
        assert!(Position::from_usi_string("1a1").is_err());
    }

    #[test]
    fn test_move_to_usi() {
        // Normal move
        let mv1 = Move::new_move(Position::new(6, 2), Position::new(5, 2), PieceType::Pawn, Player::Black, false);
        assert_eq!(mv1.to_usi_string(), "7g7f");

        // Promotion
        let mv2 = Move::new_move(Position::new(1, 1), Position::new(7, 7), PieceType::Bishop, Player::Black, true);
        assert_eq!(mv2.to_usi_string(), "8b2h+");

        // Drop
        let mv3 = Move::new_drop(PieceType::Pawn, Position::new(3, 3), Player::Black);
        assert_eq!(mv3.to_usi_string(), "P*6d");
    }

    #[test]
    fn test_move_from_usi() {
        let board = BitboardBoard::new(); // Initial position

        // Normal move
        let mv1 = Move::from_usi_string("7g7f", Player::Black, &board).unwrap();
        assert_eq!(mv1.from, Some(Position::new(6, 2)));
        assert_eq!(mv1.to, Position::new(5, 2));
        assert_eq!(mv1.is_promotion, false);
        assert_eq!(mv1.is_drop(), false);

        // Drop
        let mv2 = Move::from_usi_string("P*5e", Player::White, &board).unwrap();
        assert_eq!(mv2.piece_type, PieceType::Pawn);
        assert_eq!(mv2.to, Position::new(4, 4));
        assert_eq!(mv2.is_drop(), true);
    }

    #[test]
    fn test_null_move_config_default() {
        let config = NullMoveConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_depth, 3);
        assert_eq!(config.reduction_factor, 2);
        assert_eq!(config.max_pieces_threshold, 12);
        assert!(config.enable_dynamic_reduction);
        assert!(config.enable_endgame_detection);
    }

    #[test]
    fn test_null_move_config_validation() {
        let mut config = NullMoveConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Test invalid configurations
        config.min_depth = 0;
        assert!(config.validate().is_err());
        
        config.min_depth = 3;
        config.reduction_factor = 0;
        assert!(config.validate().is_err());
        
        config.reduction_factor = 2;
        config.max_pieces_threshold = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_null_move_config_new_validated() {
        let config = NullMoveConfig {
            enabled: true,
            min_depth: 0,  // Invalid
            reduction_factor: 10,  // Invalid
            max_pieces_threshold: 50,  // Invalid
            enable_dynamic_reduction: true,
            enable_endgame_detection: true,
        };
        
        let validated = config.new_validated();
        assert_eq!(validated.min_depth, 1);
        assert_eq!(validated.reduction_factor, 5);
        assert_eq!(validated.max_pieces_threshold, 40);
    }

    #[test]
    fn test_null_move_stats_default() {
        let stats = NullMoveStats::default();
        assert_eq!(stats.attempts, 0);
        assert_eq!(stats.cutoffs, 0);
        assert_eq!(stats.depth_reductions, 0);
        assert_eq!(stats.disabled_in_check, 0);
        assert_eq!(stats.disabled_endgame, 0);
    }

    #[test]
    fn test_null_move_stats_calculations() {
        let mut stats = NullMoveStats {
            attempts: 100,
            cutoffs: 25,
            depth_reductions: 200,
            disabled_in_check: 10,
            disabled_endgame: 5,
        };
        
        assert_eq!(stats.cutoff_rate(), 25.0);
        assert_eq!(stats.average_reduction_factor(), 2.0);
        assert_eq!(stats.total_disabled(), 15);
        assert!((stats.efficiency() - 21.74).abs() < 0.01); // 25 / (100 + 15) * 100
        
        stats.reset();
        assert_eq!(stats.attempts, 0);
        assert_eq!(stats.cutoff_rate(), 0.0);
    }

    #[test]
    fn test_null_move_config_summary() {
        let config = NullMoveConfig::default();
        let summary = config.summary();
        assert!(summary.contains("NullMoveConfig"));
        assert!(summary.contains("enabled=true"));
        assert!(summary.contains("min_depth=3"));
    }

    #[test]
    fn test_null_move_stats_summary() {
        let stats = NullMoveStats {
            attempts: 50,
            cutoffs: 10,
            depth_reductions: 100,
            disabled_in_check: 5,
            disabled_endgame: 2,
        };
        let summary = stats.summary();
        assert!(summary.contains("NMP"));
        assert!(summary.contains("50 attempts"));
        assert!(summary.contains("20.0% cutoffs"));
    }
}
