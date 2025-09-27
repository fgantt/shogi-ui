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

/// Represents a dual-phase evaluation score for tapered evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaperedScore {
    /// Middlegame score (0-256 phase)
    pub mg: i32,
    /// Endgame score (0-256 phase)
    pub eg: i32,
}

impl TaperedScore {
    /// Create a new TaperedScore with both values equal
    pub fn new(value: i32) -> Self {
        Self { mg: value, eg: value }
    }
    
    /// Create a TaperedScore with different mg and eg values
    pub fn new_tapered(mg: i32, eg: i32) -> Self {
        Self { mg, eg }
    }
    
    /// Interpolate between mg and eg based on game phase
    /// phase: 0 = endgame, GAME_PHASE_MAX = opening
    pub fn interpolate(&self, phase: i32) -> i32 {
        (self.mg * phase + self.eg * (GAME_PHASE_MAX - phase)) / GAME_PHASE_MAX
    }
}

impl Default for TaperedScore {
    fn default() -> Self {
        Self { mg: 0, eg: 0 }
    }
}

impl std::ops::AddAssign for TaperedScore {
    fn add_assign(&mut self, other: Self) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

impl std::ops::SubAssign for TaperedScore {
    fn sub_assign(&mut self, other: Self) {
        self.mg -= other.mg;
        self.eg -= other.eg;
    }
}

impl std::ops::Add for TaperedScore {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            mg: self.mg + other.mg,
            eg: self.eg + other.eg,
        }
    }
}

impl std::ops::Sub for TaperedScore {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            mg: self.mg - other.mg,
            eg: self.eg - other.eg,
        }
    }
}

impl std::ops::Neg for TaperedScore {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            mg: -self.mg,
            eg: -self.eg,
        }
    }
}

impl std::ops::Mul<f32> for TaperedScore {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            mg: (self.mg as f32 * rhs) as i32,
            eg: (self.eg as f32 * rhs) as i32,
        }
    }
}

/// Maximum game phase value (opening)
pub const GAME_PHASE_MAX: i32 = 256;

/// Phase values for different piece types
pub const PIECE_PHASE_VALUES: [(PieceType, i32); 6] = [
    (PieceType::Knight, 1),
    (PieceType::Silver, 1),
    (PieceType::Gold, 2),
    (PieceType::Bishop, 2),
    (PieceType::Rook, 3),
    (PieceType::Lance, 1),
];

/// Configuration for advanced king safety evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KingSafetyConfig {
    /// Enable or disable advanced king safety evaluation
    pub enabled: bool,
    /// Weight for castle structure evaluation
    pub castle_weight: f32,
    /// Weight for attack analysis
    pub attack_weight: f32,
    /// Weight for threat evaluation
    pub threat_weight: f32,
    /// Phase adjustment factor for endgame
    pub phase_adjustment: f32,
    /// Enable performance mode for fast evaluation
    pub performance_mode: bool,
}

impl Default for KingSafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // Re-enabled with aggressive optimizations
            castle_weight: 0.3,  // Reduced weights for performance
            attack_weight: 0.3,
            threat_weight: 0.2,  // Lowest weight since threats are most expensive
            phase_adjustment: 0.8,
            performance_mode: true,  // Enable performance mode by default
        }
    }
}

/// Configuration options for tapered evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct TaperedEvaluationConfig {
    /// Enable or disable tapered evaluation
    pub enabled: bool,
    /// Cache game phase calculation per search node
    pub cache_game_phase: bool,
    /// Use SIMD optimizations (future feature)
    pub use_simd: bool,
    /// Memory pool size for TaperedScore objects
    pub memory_pool_size: usize,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// King safety evaluation configuration
    pub king_safety: KingSafetyConfig,
}

impl Default for TaperedEvaluationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_game_phase: true,
            use_simd: false,
            memory_pool_size: 1000,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }
}

impl TaperedEvaluationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration with tapered evaluation disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            cache_game_phase: false,
            use_simd: false,
            memory_pool_size: 0,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }
    
    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            enabled: true,
            cache_game_phase: true,
            use_simd: false,
            memory_pool_size: 2000,
            enable_performance_monitoring: true,
            king_safety: KingSafetyConfig::default(),
        }
    }
    
    /// Create a configuration optimized for memory usage
    pub fn memory_optimized() -> Self {
        Self {
            enabled: true,
            cache_game_phase: false,
            use_simd: false,
            memory_pool_size: 100,
            enable_performance_monitoring: false,
            king_safety: KingSafetyConfig::default(),
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Configuration for Late Move Reductions (LMR) parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LMRConfig {
    pub enabled: bool,                        // Enable late move reductions
    pub min_depth: u8,                        // Minimum depth to apply LMR
    pub min_move_index: u8,                   // Minimum move index to consider for reduction
    pub base_reduction: u8,                   // Base reduction amount
    pub max_reduction: u8,                    // Maximum reduction allowed
    pub enable_dynamic_reduction: bool,       // Use dynamic vs static reduction
    pub enable_adaptive_reduction: bool,      // Use position-based adaptation
    pub enable_extended_exemptions: bool,     // Extended move exemption rules
}

impl Default for LMRConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 3,
            min_move_index: 4,
            base_reduction: 1,
            max_reduction: 3,
            enable_dynamic_reduction: true,
            enable_adaptive_reduction: true,
            enable_extended_exemptions: true,
        }
    }
}

impl LMRConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.min_depth > 15 {
            return Err("min_depth should not exceed 15 for performance reasons".to_string());
        }
        if self.min_move_index == 0 {
            return Err("min_move_index must be greater than 0".to_string());
        }
        if self.min_move_index > 20 {
            return Err("min_move_index should not exceed 20".to_string());
        }
        if self.base_reduction == 0 {
            return Err("base_reduction must be greater than 0".to_string());
        }
        if self.base_reduction > 5 {
            return Err("base_reduction should not exceed 5".to_string());
        }
        if self.max_reduction < self.base_reduction {
            return Err("max_reduction must be >= base_reduction".to_string());
        }
        if self.max_reduction > 8 {
            return Err("max_reduction should not exceed 8".to_string());
        }
        Ok(())
    }

    /// Create a validated configuration, clamping values to valid ranges
    pub fn new_validated(mut self) -> Self {
        self.min_depth = self.min_depth.clamp(1, 15);
        self.min_move_index = self.min_move_index.clamp(1, 20);
        self.base_reduction = self.base_reduction.clamp(1, 5);
        self.max_reduction = self.max_reduction.clamp(self.base_reduction, 8);
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "LMRConfig: enabled={}, min_depth={}, min_move_index={}, base_reduction={}, max_reduction={}, dynamic={}, adaptive={}, extended_exemptions={}",
            self.enabled,
            self.min_depth,
            self.min_move_index,
            self.base_reduction,
            self.max_reduction,
            self.enable_dynamic_reduction,
            self.enable_adaptive_reduction,
            self.enable_extended_exemptions
        )
    }
}

/// Performance statistics for Late Move Reductions
#[derive(Debug, Clone, Default)]
pub struct LMRStats {
    pub moves_considered: u64,                // Total moves considered for LMR
    pub reductions_applied: u64,              // Number of reductions applied
    pub researches_triggered: u64,            // Number of full-depth re-searches
    pub cutoffs_after_reduction: u64,         // Cutoffs after reduced search
    pub cutoffs_after_research: u64,          // Cutoffs after full re-search
    pub total_depth_saved: u64,               // Total depth reduction applied
    pub average_reduction: f64,               // Average reduction applied
}

impl LMRStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = LMRStats::default();
    }

    /// Get the research rate as a percentage
    pub fn research_rate(&self) -> f64 {
        if self.reductions_applied == 0 {
            return 0.0;
        }
        (self.researches_triggered as f64 / self.reductions_applied as f64) * 100.0
    }

    /// Get the efficiency of LMR as a percentage
    pub fn efficiency(&self) -> f64 {
        if self.moves_considered == 0 {
            return 0.0;
        }
        (self.reductions_applied as f64 / self.moves_considered as f64) * 100.0
    }

    /// Get the total number of cutoffs
    pub fn total_cutoffs(&self) -> u64 {
        self.cutoffs_after_reduction + self.cutoffs_after_research
    }

    /// Get the cutoff rate as a percentage
    pub fn cutoff_rate(&self) -> f64 {
        if self.moves_considered == 0 {
            return 0.0;
        }
        (self.total_cutoffs() as f64 / self.moves_considered as f64) * 100.0
    }

    /// Get the average depth saved per reduction
    pub fn average_depth_saved(&self) -> f64 {
        if self.reductions_applied == 0 {
            return 0.0;
        }
        self.total_depth_saved as f64 / self.reductions_applied as f64
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Late Move Reductions Performance Report:\n\
            - Moves considered: {}\n\
            - Reductions applied: {} ({:.2}%)\n\
            - Re-searches triggered: {} ({:.2}%)\n\
            - Total cutoffs: {} ({:.2}%)\n\
            - Average depth saved: {:.2}\n\
            - Total depth saved: {}",
            self.moves_considered,
            self.reductions_applied,
            self.efficiency(),
            self.researches_triggered,
            self.research_rate(),
            self.total_cutoffs(),
            self.cutoff_rate(),
            self.average_depth_saved(),
            self.total_depth_saved
        )
    }

    /// Get a summary of key metrics
    pub fn summary(&self) -> String {
        format!(
            "LMR: {} considered, {:.1}% reduced, {:.1}% researched, {:.1}% cutoffs, {:.1} avg saved",
            self.moves_considered,
            self.efficiency(),
            self.research_rate(),
            self.cutoff_rate(),
            self.average_depth_saved()
        )
    }
}

/// Move type classification for LMR decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Check,
    Capture,
    Promotion,
    Killer,
    TranspositionTable,
    Escape,
    Center,
    Quiet,
}

/// Position complexity levels for adaptive LMR
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionComplexity {
    Low,
    Medium,
    High,
    Unknown,
}

/// LMR playing style presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LMRPlayingStyle {
    Aggressive,
    Conservative,
    Balanced,
}

/// Performance metrics for LMR optimization
#[derive(Debug, Clone)]
pub struct LMRPerformanceMetrics {
    pub moves_considered: u64,
    pub reductions_applied: u64,
    pub researches_triggered: u64,
    pub efficiency: f64,
    pub research_rate: f64,
    pub cutoff_rate: f64,
    pub average_depth_saved: f64,
    pub total_depth_saved: u64,
    pub nodes_per_second: f64,
}

impl LMRPerformanceMetrics {
    /// Get a summary of performance metrics
    pub fn summary(&self) -> String {
        format!(
            "LMR Performance: {:.1}% efficiency, {:.1}% research rate, {:.1}% cutoffs, {:.0} NPS",
            self.efficiency,
            self.research_rate,
            self.cutoff_rate,
            self.nodes_per_second
        )
    }

    /// Check if LMR is performing well
    pub fn is_performing_well(&self) -> bool {
        self.efficiency > 20.0 && self.research_rate < 40.0 && self.cutoff_rate > 5.0
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.research_rate > 40.0 {
            recommendations.push("Consider reducing LMR aggressiveness (too many re-searches)".to_string());
        }
        
        if self.efficiency < 20.0 {
            recommendations.push("Consider increasing LMR aggressiveness (low efficiency)".to_string());
        }
        
        if self.cutoff_rate < 5.0 {
            recommendations.push("Consider improving move ordering (low cutoff rate)".to_string());
        }
        
        if self.average_depth_saved < 1.0 {
            recommendations.push("Consider increasing base reduction (low depth savings)".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("LMR performance is optimal".to_string());
        }
        
        recommendations
    }
}

/// Profile result for LMR performance analysis
#[derive(Debug, Clone)]
pub struct LMRProfileResult {
    pub total_time: std::time::Duration,
    pub average_time_per_search: std::time::Duration,
    pub total_moves_processed: u64,
    pub total_reductions_applied: u64,
    pub total_researches_triggered: u64,
    pub moves_per_second: f64,
    pub reduction_rate: f64,
    pub research_rate: f64,
}

impl LMRProfileResult {
    /// Get a summary of the profile results
    pub fn summary(&self) -> String {
        format!(
            "LMR Profile: {:.2}s total, {:.2}s avg/search, {:.0} moves/sec, {:.1}% reduced, {:.1}% researched",
            self.total_time.as_secs_f64(),
            self.average_time_per_search.as_secs_f64(),
            self.moves_per_second,
            self.reduction_rate,
            self.research_rate
        )
    }

    /// Check if LMR is performing efficiently
    pub fn is_efficient(&self) -> bool {
        self.reduction_rate > 20.0 && self.research_rate < 30.0 && self.moves_per_second > 1000.0
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

    #[test]
    fn test_tapered_score_new() {
        let score = TaperedScore::new(100);
        assert_eq!(score.mg, 100);
        assert_eq!(score.eg, 100);
    }

    #[test]
    fn test_tapered_score_new_tapered() {
        let score = TaperedScore::new_tapered(200, 150);
        assert_eq!(score.mg, 200);
        assert_eq!(score.eg, 150);
    }

    #[test]
    fn test_tapered_score_default() {
        let score = TaperedScore::default();
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_tapered_score_interpolation() {
        let score = TaperedScore::new_tapered(100, 200);
        
        // At phase 0 (endgame), should return eg value
        assert_eq!(score.interpolate(0), 200);
        
        // At phase 256 (opening), should return mg value
        assert_eq!(score.interpolate(GAME_PHASE_MAX), 100);
        
        // At phase 128 (middlegame), should return average
        assert_eq!(score.interpolate(128), 150);
        
        // Test edge cases
        assert_eq!(score.interpolate(64), 175);  // 100 * 64 + 200 * 192 / 256
        assert_eq!(score.interpolate(192), 125); // 100 * 192 + 200 * 64 / 256
    }

    #[test]
    fn test_tapered_score_add() {
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(50, 75);
        let result = score1 + score2;
        
        assert_eq!(result.mg, 150);
        assert_eq!(result.eg, 275);
    }

    #[test]
    fn test_tapered_score_sub() {
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(30, 50);
        let result = score1 - score2;
        
        assert_eq!(result.mg, 70);
        assert_eq!(result.eg, 150);
    }

    #[test]
    fn test_tapered_score_neg() {
        let score = TaperedScore::new_tapered(100, -200);
        let neg_score = -score;
        
        assert_eq!(neg_score.mg, -100);
        assert_eq!(neg_score.eg, 200);
    }

    #[test]
    fn test_tapered_score_add_assign() {
        let mut score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(50, 75);
        score1 += score2;
        
        assert_eq!(score1.mg, 150);
        assert_eq!(score1.eg, 275);
    }

    #[test]
    fn test_tapered_score_equality() {
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(100, 200);
        let score3 = TaperedScore::new_tapered(100, 201);
        
        assert_eq!(score1, score2);
        assert_ne!(score1, score3);
    }

    #[test]
    fn test_tapered_score_clone_copy() {
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = score1; // Copy
        let score3 = score1.clone(); // Clone
        
        assert_eq!(score1, score2);
        assert_eq!(score1, score3);
        assert_eq!(score2, score3);
    }

    #[test]
    fn test_tapered_score_hash() {
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(100, 200);
        
        map.insert(score1, "first");
        assert_eq!(map.get(&score2), Some(&"first"));
    }

    #[test]
    fn test_tapered_score_serialization() {
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test JSON serialization
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: TaperedScore = serde_json::from_str(&json).unwrap();
        assert_eq!(score, deserialized);
    }

    #[test]
    fn test_game_phase_constants() {
        assert_eq!(GAME_PHASE_MAX, 256);
        assert_eq!(PIECE_PHASE_VALUES.len(), 6);
        
        // Test that all piece types have phase values
        let piece_types = [
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Lance,
        ];
        
        for piece_type in &piece_types {
            assert!(PIECE_PHASE_VALUES.iter().any(|(pt, _)| *pt == *piece_type));
        }
    }

    #[test]
    fn test_tapered_score_interpolation_edge_cases() {
        let score = TaperedScore::new_tapered(100, 200);
        
        // Test with negative phase (should still work)
        // 100 * (-1) + 200 * (256 - (-1)) / 256 = -100 + 200 * 257 / 256 = -100 + 51400 / 256 = 51300 / 256 = 200
        assert_eq!(score.interpolate(-1), 200);
        
        // Test with phase > GAME_PHASE_MAX
        // 100 * 300 + 200 * (256 - 300) / 256 = 30000 + 200 * (-44) / 256 = (30000 - 8800) / 256 = 21200 / 256 = 82
        assert_eq!(score.interpolate(300), 82);
        
        // Test with zero values
        let zero_score = TaperedScore::new_tapered(0, 0);
        assert_eq!(zero_score.interpolate(128), 0);
    }

    #[test]
    fn test_tapered_score_arithmetic_consistency() {
        let score1 = TaperedScore::new_tapered(100, 200);
        let score2 = TaperedScore::new_tapered(50, 75);
        
        // Test that (a + b) - b = a
        let sum = score1 + score2;
        let diff = sum - score2;
        assert_eq!(diff, score1);
        
        // Test that a + (-a) = 0
        let neg_score1 = -score1;
        let zero = score1 + neg_score1;
        assert_eq!(zero, TaperedScore::default());
    }
}

/// Configuration for Aspiration Windows parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AspirationWindowConfig {
    /// Enable aspiration windows
    pub enabled: bool,
    /// Base window size in centipawns
    pub base_window_size: i32,
    /// Dynamic window scaling factor
    pub dynamic_scaling: bool,
    /// Maximum window size (safety limit)
    pub max_window_size: i32,
    /// Minimum depth to apply aspiration windows
    pub min_depth: u8,
    /// Enable adaptive window sizing
    pub enable_adaptive_sizing: bool,
    /// Maximum number of re-searches per depth
    pub max_researches: u8,
    /// Enable fail-high/fail-low statistics
    pub enable_statistics: bool,
}

impl Default for AspirationWindowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_window_size: 50,        // 50 centipawns
            dynamic_scaling: true,
            max_window_size: 200,        // 200 centipawns
            min_depth: 2,                // Start at depth 2
            enable_adaptive_sizing: true,
            max_researches: 2,           // Allow up to 2 re-searches
            enable_statistics: true,
        }
    }
}

impl AspirationWindowConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.base_window_size <= 0 {
            return Err("base_window_size must be greater than 0".to_string());
        }
        if self.base_window_size > 1000 {
            return Err("base_window_size should not exceed 1000 centipawns".to_string());
        }
        if self.max_window_size < self.base_window_size {
            return Err("max_window_size must be >= base_window_size".to_string());
        }
        if self.max_window_size > 2000 {
            return Err("max_window_size should not exceed 2000 centipawns".to_string());
        }
        if self.min_depth == 0 {
            return Err("min_depth must be greater than 0".to_string());
        }
        if self.min_depth > 10 {
            return Err("min_depth should not exceed 10 for performance reasons".to_string());
        }
        if self.max_researches == 0 {
            return Err("max_researches must be greater than 0".to_string());
        }
        if self.max_researches > 5 {
            return Err("max_researches should not exceed 5".to_string());
        }
        Ok(())
    }

    /// Create a validated configuration, clamping values to valid ranges
    pub fn new_validated(mut self) -> Self {
        self.base_window_size = self.base_window_size.clamp(1, 1000);
        self.max_window_size = self.max_window_size.clamp(self.base_window_size, 2000);
        self.min_depth = self.min_depth.clamp(1, 10);
        self.max_researches = self.max_researches.clamp(1, 5);
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "AspirationWindowConfig: enabled={}, base_window_size={}, max_window_size={}, min_depth={}, dynamic_scaling={}, adaptive_sizing={}, max_researches={}, statistics={}",
            self.enabled,
            self.base_window_size,
            self.max_window_size,
            self.min_depth,
            self.dynamic_scaling,
            self.enable_adaptive_sizing,
            self.max_researches,
            self.enable_statistics
        )
    }
}

/// Performance statistics for Aspiration Windows
#[derive(Debug, Clone, Default)]
pub struct AspirationWindowStats {
    /// Total searches performed
    pub total_searches: u64,
    /// Successful searches (no re-search needed)
    pub successful_searches: u64,
    /// Fail-low occurrences
    pub fail_lows: u64,
    /// Fail-high occurrences  
    pub fail_highs: u64,
    /// Total re-searches performed
    pub total_researches: u64,
    /// Average window size used
    pub average_window_size: f64,
    /// Time saved (estimated)
    pub estimated_time_saved_ms: u64,
    /// Nodes saved (estimated)
    pub estimated_nodes_saved: u64,
    /// Maximum window size used
    pub max_window_size_used: i32,
    /// Minimum window size used
    pub min_window_size_used: i32,
    /// Total time spent in aspiration window searches (ms)
    pub total_search_time_ms: u64,
    /// Total time spent in re-searches (ms)
    pub total_research_time_ms: u64,
    /// Average search time per depth (ms)
    pub average_search_time_ms: f64,
    /// Average re-search time per depth (ms)
    pub average_research_time_ms: f64,
    /// Window size variance (for tuning analysis)
    pub window_size_variance: f64,
    /// Success rate by depth (for depth analysis)
    pub success_rate_by_depth: Vec<f64>,
    /// Re-search rate by depth (for depth analysis)
    pub research_rate_by_depth: Vec<f64>,
    /// Average window size by depth (for depth analysis)
    pub window_size_by_depth: Vec<f64>,
    /// Performance trend over time (last 100 searches)
    pub recent_performance: Vec<f64>,
    /// Configuration effectiveness score (0.0 to 1.0)
    pub configuration_effectiveness: f64,
    /// Memory usage statistics
    pub memory_usage_bytes: u64,
    /// Peak memory usage bytes
    pub peak_memory_usage_bytes: u64,
    /// Cache hit rate for window size calculations
    pub cache_hit_rate: f64,
    /// Adaptive tuning success rate
    pub adaptive_tuning_success_rate: f64,
}

impl AspirationWindowStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = AspirationWindowStats::default();
    }

    /// Initialize depth-based tracking vectors
    pub fn initialize_depth_tracking(&mut self, max_depth: u8) {
        self.success_rate_by_depth = vec![0.0; max_depth as usize + 1];
        self.research_rate_by_depth = vec![0.0; max_depth as usize + 1];
        self.window_size_by_depth = vec![0.0; max_depth as usize + 1];
    }

    /// Update depth-based statistics
    pub fn update_depth_stats(&mut self, depth: u8, success: bool, had_research: bool, window_size: i32) {
        if depth < self.success_rate_by_depth.len() as u8 {
            let depth_idx = depth as usize;
            
            // Update success rate
            if success {
                self.success_rate_by_depth[depth_idx] += 1.0;
            }
            
            // Update research rate
            if had_research {
                self.research_rate_by_depth[depth_idx] += 1.0;
            }
            
            // Update window size
            self.window_size_by_depth[depth_idx] = window_size as f64;
        }
    }

    /// Calculate comprehensive performance metrics
    pub fn calculate_performance_metrics(&mut self) -> AspirationWindowPerformanceMetrics {
        let success_rate = if self.total_searches > 0 {
            self.successful_searches as f64 / self.total_searches as f64
        } else {
            0.0
        };

        let research_rate = if self.total_searches > 0 {
            self.total_researches as f64 / self.total_searches as f64
        } else {
            0.0
        };

        let _fail_low_rate = if self.total_searches > 0 {
            self.fail_lows as f64 / self.total_searches as f64
        } else {
            0.0
        };

        let _fail_high_rate = if self.total_searches > 0 {
            self.fail_highs as f64 / self.total_searches as f64
        } else {
            0.0
        };

        // Calculate efficiency based on success rate and research rate
        let efficiency = if research_rate > 0.0 {
            success_rate / (1.0 + research_rate)
        } else {
            success_rate
        };

        // Update average times
        if self.total_searches > 0 {
            self.average_search_time_ms = self.total_search_time_ms as f64 / self.total_searches as f64;
        }
        if self.total_researches > 0 {
            self.average_research_time_ms = self.total_research_time_ms as f64 / self.total_researches as f64;
        }

        // Calculate configuration effectiveness
        self.configuration_effectiveness = self.calculate_configuration_effectiveness();

        AspirationWindowPerformanceMetrics {
            total_searches: self.total_searches,
            successful_searches: self.successful_searches,
            fail_lows: self.fail_lows,
            fail_highs: self.fail_highs,
            total_researches: self.total_researches,
            success_rate,
            research_rate,
            efficiency,
            average_window_size: self.average_window_size,
            estimated_time_saved_ms: self.estimated_time_saved_ms,
            estimated_nodes_saved: self.estimated_nodes_saved,
        }
    }

    /// Calculate configuration effectiveness score
    fn calculate_configuration_effectiveness(&self) -> f64 {
        if self.total_searches < 10 {
            return 0.5; // Neutral score for insufficient data
        }

        let success_rate = self.successful_searches as f64 / self.total_searches as f64;
        let research_rate = self.total_researches as f64 / self.total_searches as f64;
        let fail_rate = (self.fail_lows + self.fail_highs) as f64 / self.total_searches as f64;

        // Effectiveness based on high success rate, low research rate, and low fail rate
        let effectiveness = success_rate * (1.0 - research_rate * 0.5) * (1.0 - fail_rate * 0.3);
        effectiveness.max(0.0).min(1.0)
    }

    /// Update window size statistics
    pub fn update_window_size_stats(&mut self, window_size: i32) {
        // Update min/max
        if window_size > self.max_window_size_used {
            self.max_window_size_used = window_size;
        }
        if window_size < self.min_window_size_used || self.min_window_size_used == 0 {
            self.min_window_size_used = window_size;
        }

        // Update average (exponential moving average)
        if self.total_searches == 0 {
            self.average_window_size = window_size as f64;
        } else {
            let alpha = 0.1; // Smoothing factor
            self.average_window_size = alpha * window_size as f64 + (1.0 - alpha) * self.average_window_size;
        }
    }

    /// Update time statistics
    pub fn update_time_stats(&mut self, search_time_ms: u64, research_time_ms: u64) {
        self.total_search_time_ms += search_time_ms;
        self.total_research_time_ms += research_time_ms;
    }

    /// Update memory usage statistics
    pub fn update_memory_stats(&mut self, current_usage: u64) {
        self.memory_usage_bytes = current_usage;
        if current_usage > self.peak_memory_usage_bytes {
            self.peak_memory_usage_bytes = current_usage;
        }
    }

    /// Add performance data point for trend analysis
    pub fn add_performance_data_point(&mut self, performance: f64) {
        self.recent_performance.push(performance);
        
        // Keep only last 100 data points
        if self.recent_performance.len() > 100 {
            self.recent_performance.remove(0);
        }
    }

    /// Calculate performance trend
    pub fn get_performance_trend(&self) -> f64 {
        if self.recent_performance.len() < 10 {
            return 0.0; // Not enough data
        }

        let mid = self.recent_performance.len() / 2;
        let recent_avg = self.recent_performance[mid..].iter().sum::<f64>() / (self.recent_performance.len() - mid) as f64;
        let early_avg = self.recent_performance[..mid].iter().sum::<f64>() / mid as f64;
        
        recent_avg - early_avg
    }

    /// Get depth-based analysis
    pub fn get_depth_analysis(&self) -> DepthAnalysis {
        DepthAnalysis {
            success_rate_by_depth: self.success_rate_by_depth.clone(),
            research_rate_by_depth: self.research_rate_by_depth.clone(),
            window_size_by_depth: self.window_size_by_depth.clone(),
        }
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_searches: self.total_searches,
            success_rate: if self.total_searches > 0 {
                self.successful_searches as f64 / self.total_searches as f64
            } else {
                0.0
            },
            research_rate: if self.total_searches > 0 {
                self.total_researches as f64 / self.total_searches as f64
            } else {
                0.0
            },
            average_window_size: self.average_window_size,
            configuration_effectiveness: self.configuration_effectiveness,
            performance_trend: self.get_performance_trend(),
            memory_efficiency: if self.peak_memory_usage_bytes > 0 {
                self.memory_usage_bytes as f64 / self.peak_memory_usage_bytes as f64
            } else {
                1.0
            },
        }
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.successful_searches as f64 / self.total_searches as f64) * 100.0
    }

    /// Get the re-search rate as a percentage
    pub fn research_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.total_researches as f64 / self.total_searches as f64) * 100.0
    }

    /// Get the efficiency of aspiration windows
    pub fn efficiency(&self) -> f64 {
        // Higher is better: more time saved, fewer re-searches
        let time_savings = self.estimated_time_saved_ms as f64;
        let research_penalty = self.total_researches as f64 * 10.0; // Penalty for re-searches
        time_savings - research_penalty
    }

    /// Get the fail-low rate as a percentage
    pub fn fail_low_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.fail_lows as f64 / self.total_searches as f64) * 100.0
    }

    /// Get the fail-high rate as a percentage
    pub fn fail_high_rate(&self) -> f64 {
        if self.total_searches == 0 {
            return 0.0;
        }
        (self.fail_highs as f64 / self.total_searches as f64) * 100.0
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Aspiration Windows Performance Report:\n\
            - Total searches: {}\n\
            - Successful searches: {} ({:.2}%)\n\
            - Fail-lows: {} ({:.2}%)\n\
            - Fail-highs: {} ({:.2}%)\n\
            - Total re-searches: {} ({:.2}%)\n\
            - Average window size: {:.2}\n\
            - Estimated time saved: {} ms\n\
            - Estimated nodes saved: {}",
            self.total_searches,
            self.successful_searches,
            self.success_rate(),
            self.fail_lows,
            self.fail_low_rate(),
            self.fail_highs,
            self.fail_high_rate(),
            self.total_researches,
            self.research_rate(),
            self.average_window_size,
            self.estimated_time_saved_ms,
            self.estimated_nodes_saved
        )
    }

    /// Get a summary of key metrics
    pub fn summary(&self) -> String {
        format!(
            "Aspiration: {} searches, {:.1}% success, {:.1}% re-search, {:.1}% fail-low, {:.1}% fail-high, {:.1} avg window",
            self.total_searches,
            self.success_rate(),
            self.research_rate(),
            self.fail_low_rate(),
            self.fail_high_rate(),
            self.average_window_size
        )
    }
}

/// Aspiration window playing style presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspirationWindowPlayingStyle {
    Aggressive,
    Conservative,
    Balanced,
}

/// Performance metrics for Aspiration Windows optimization
#[derive(Debug, Clone)]
pub struct AspirationWindowPerformanceMetrics {
    pub total_searches: u64,
    pub successful_searches: u64,
    pub fail_lows: u64,
    pub fail_highs: u64,
    pub total_researches: u64,
    pub success_rate: f64,
    pub research_rate: f64,
    pub efficiency: f64,
    pub average_window_size: f64,
    pub estimated_time_saved_ms: u64,
    pub estimated_nodes_saved: u64,
}

impl AspirationWindowPerformanceMetrics {
    /// Get a summary of performance metrics
    pub fn summary(&self) -> String {
        format!(
            "Aspiration Windows Performance: {:.1}% success, {:.1}% re-search, {:.1}% fail-low, {:.1}% fail-high, {:.0} ms saved",
            self.success_rate,
            self.research_rate,
            self.fail_lows as f64 / self.total_searches as f64 * 100.0,
            self.fail_highs as f64 / self.total_searches as f64 * 100.0,
            self.estimated_time_saved_ms
        )
    }

    /// Check if aspiration windows are performing well
    pub fn is_performing_well(&self) -> bool {
        self.success_rate > 70.0 && self.research_rate < 30.0 && self.efficiency > 0.0
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.research_rate > 30.0 {
            recommendations.push("Consider increasing window size (too many re-searches)".to_string());
        }
        
        if self.success_rate < 70.0 {
            recommendations.push("Consider decreasing window size (too many failures)".to_string());
        }
        
        if self.fail_lows > self.fail_highs * 2 {
            recommendations.push("Consider asymmetric window sizing (more fail-lows than fail-highs)".to_string());
        }
        
        if self.fail_highs > self.fail_lows * 2 {
            recommendations.push("Consider asymmetric window sizing (more fail-highs than fail-lows)".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Aspiration windows are performing well".to_string());
        }
        
        recommendations
    }
}

/// Statistics for window size analysis and tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSizeStatistics {
    /// Average window size used
    pub average_window_size: f64,
    /// Minimum window size enforced
    pub min_window_size: i32,
    /// Maximum window size allowed
    pub max_window_size: i32,
    /// Total number of window size calculations
    pub total_calculations: u64,
    /// Success rate of aspiration windows
    pub success_rate: f64,
    /// Fail-low rate
    pub fail_low_rate: f64,
    /// Fail-high rate
    pub fail_high_rate: f64,
}

impl Default for WindowSizeStatistics {
    fn default() -> Self {
        Self {
            average_window_size: 0.0,
            min_window_size: 10,
            max_window_size: 200,
            total_calculations: 0,
            success_rate: 0.0,
            fail_low_rate: 0.0,
            fail_high_rate: 0.0,
        }
    }
}

impl WindowSizeStatistics {
    /// Get a summary of window size statistics
    pub fn summary(&self) -> String {
        format!(
            "Window Size Stats: avg={:.1}, min={}, max={}, calculations={}, success={:.1}%, fail_low={:.1}%, fail_high={:.1}%",
            self.average_window_size,
            self.min_window_size,
            self.max_window_size,
            self.total_calculations,
            self.success_rate * 100.0,
            self.fail_low_rate * 100.0,
            self.fail_high_rate * 100.0
        )
    }

    /// Check if window size is well-tuned
    pub fn is_well_tuned(&self) -> bool {
        self.success_rate > 0.7 && self.fail_low_rate < 0.2 && self.fail_high_rate < 0.2
    }

    /// Get tuning recommendations
    pub fn get_tuning_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.success_rate < 0.6 {
            recommendations.push("Low success rate: consider increasing base_window_size".to_string());
        }
        if self.fail_low_rate > 0.3 {
            recommendations.push("High fail-low rate: consider larger base_window_size".to_string());
        }
        if self.fail_high_rate > 0.3 {
            recommendations.push("High fail-high rate: consider larger base_window_size".to_string());
        }
        if self.average_window_size < (self.min_window_size as f64) * 1.5 {
            recommendations.push("Very small average window: consider increasing base_window_size".to_string());
        }
        if self.average_window_size > (self.max_window_size as f64) * 0.8 {
            recommendations.push("Very large average window: consider decreasing base_window_size".to_string());
        }
        
        recommendations
    }
}

/// Metrics for re-search efficiency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEfficiencyMetrics {
    /// Total searches performed
    pub total_searches: u64,
    /// Successful searches (no re-search needed)
    pub successful_searches: u64,
    /// Fail-low occurrences
    pub fail_lows: u64,
    /// Fail-high occurrences
    pub fail_highs: u64,
    /// Total re-searches performed
    pub total_researches: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Re-search rate (average re-searches per search)
    pub research_rate: f64,
    /// Fail-low rate (0.0 to 1.0)
    pub fail_low_rate: f64,
    /// Fail-high rate (0.0 to 1.0)
    pub fail_high_rate: f64,
}

impl Default for ResearchEfficiencyMetrics {
    fn default() -> Self {
        Self {
            total_searches: 0,
            successful_searches: 0,
            fail_lows: 0,
            fail_highs: 0,
            total_researches: 0,
            success_rate: 0.0,
            research_rate: 0.0,
            fail_low_rate: 0.0,
            fail_high_rate: 0.0,
        }
    }
}

impl ResearchEfficiencyMetrics {
    /// Get a summary of re-search efficiency
    pub fn summary(&self) -> String {
        format!(
            "Re-search Efficiency: {} searches, {:.1}% success, {:.2} re-search rate, {:.1}% fail-low, {:.1}% fail-high",
            self.total_searches,
            self.success_rate * 100.0,
            self.research_rate,
            self.fail_low_rate * 100.0,
            self.fail_high_rate * 100.0
        )
    }

    /// Check if re-search efficiency is good
    pub fn is_efficient(&self) -> bool {
        self.success_rate > 0.7 && self.research_rate < 1.5 && self.fail_low_rate < 0.3 && self.fail_high_rate < 0.3
    }

    /// Get efficiency recommendations
    pub fn get_efficiency_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.success_rate < 0.6 {
            recommendations.push("Low success rate: consider increasing base_window_size".to_string());
        }
        if self.research_rate > 2.0 {
            recommendations.push("High re-search rate: consider increasing base_window_size or max_researches".to_string());
        }
        if self.fail_low_rate > 0.4 {
            recommendations.push("High fail-low rate: consider larger base_window_size".to_string());
        }
        if self.fail_high_rate > 0.4 {
            recommendations.push("High fail-high rate: consider larger base_window_size".to_string());
        }
        if self.fail_lows > self.fail_highs * 2 {
            recommendations.push("Asymmetric failures: consider asymmetric window sizing".to_string());
        }
        if self.fail_highs > self.fail_lows * 2 {
            recommendations.push("Asymmetric failures: consider asymmetric window sizing".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Re-search efficiency is good".to_string());
        }
        
        recommendations
    }
}

/// Depth-based analysis for aspiration windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthAnalysis {
    /// Success rate by depth
    pub success_rate_by_depth: Vec<f64>,
    /// Re-search rate by depth
    pub research_rate_by_depth: Vec<f64>,
    /// Average window size by depth
    pub window_size_by_depth: Vec<f64>,
}

impl DepthAnalysis {
    /// Get analysis summary
    pub fn summary(&self) -> String {
        format!(
            "Depth Analysis: {} depths analyzed, avg success rate: {:.1}%, avg research rate: {:.1}%",
            self.success_rate_by_depth.len(),
            self.get_average_success_rate() * 100.0,
            self.get_average_research_rate() * 100.0
        )
    }

    /// Get average success rate across all depths
    pub fn get_average_success_rate(&self) -> f64 {
        if self.success_rate_by_depth.is_empty() {
            return 0.0;
        }
        self.success_rate_by_depth.iter().sum::<f64>() / self.success_rate_by_depth.len() as f64
    }

    /// Get average research rate across all depths
    pub fn get_average_research_rate(&self) -> f64 {
        if self.research_rate_by_depth.is_empty() {
            return 0.0;
        }
        self.research_rate_by_depth.iter().sum::<f64>() / self.research_rate_by_depth.len() as f64
    }

    /// Get optimal depth range for aspiration windows
    pub fn get_optimal_depth_range(&self) -> (u8, u8) {
        let mut best_start = 0;
        let mut best_end = 0;
        let mut best_score = 0.0;

        for start in 0..self.success_rate_by_depth.len() {
            for end in start..self.success_rate_by_depth.len() {
                let range_success = self.success_rate_by_depth[start..=end].iter().sum::<f64>() / (end - start + 1) as f64;
                let range_research = self.research_rate_by_depth[start..=end].iter().sum::<f64>() / (end - start + 1) as f64;
                let score = range_success * (1.0 - range_research * 0.5);
                
                if score > best_score {
                    best_score = score;
                    best_start = start;
                    best_end = end;
                }
            }
        }

        (best_start as u8, best_end as u8)
    }
}

/// Performance summary for aspiration windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total searches performed
    pub total_searches: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Re-search rate (0.0 to 1.0)
    pub research_rate: f64,
    /// Average window size used
    pub average_window_size: f64,
    /// Configuration effectiveness (0.0 to 1.0)
    pub configuration_effectiveness: f64,
    /// Performance trend (positive = improving, negative = declining)
    pub performance_trend: f64,
    /// Memory efficiency (0.0 to 1.0)
    pub memory_efficiency: f64,
}

impl PerformanceSummary {
    /// Get performance grade (A+ to F)
    pub fn get_performance_grade(&self) -> String {
        let score = (self.success_rate * 0.4 + 
                    (1.0 - self.research_rate) * 0.3 + 
                    self.configuration_effectiveness * 0.2 + 
                    self.memory_efficiency * 0.1) * 100.0;

        match score as u8 {
            95..=100 => "A+".to_string(),
            90..=94 => "A".to_string(),
            85..=89 => "A-".to_string(),
            80..=84 => "B+".to_string(),
            75..=79 => "B".to_string(),
            70..=74 => "B-".to_string(),
            65..=69 => "C+".to_string(),
            60..=64 => "C".to_string(),
            55..=59 => "C-".to_string(),
            50..=54 => "D".to_string(),
            _ => "F".to_string(),
        }
    }

    /// Get performance recommendations
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.success_rate < 0.7 {
            recommendations.push("Low success rate: consider increasing base_window_size".to_string());
        }
        if self.research_rate > 1.5 {
            recommendations.push("High research rate: consider increasing base_window_size or max_researches".to_string());
        }
        if self.configuration_effectiveness < 0.6 {
            recommendations.push("Poor configuration effectiveness: consider tuning parameters".to_string());
        }
        if self.performance_trend < -0.1 {
            recommendations.push("Declining performance: consider resetting or retuning".to_string());
        }
        if self.memory_efficiency < 0.5 {
            recommendations.push("Low memory efficiency: consider optimizing memory usage".to_string());
        }
        if self.average_window_size < 20.0 {
            recommendations.push("Very small average window: consider increasing base_window_size".to_string());
        }
        if self.average_window_size > 150.0 {
            recommendations.push("Very large average window: consider decreasing base_window_size".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is good, no recommendations needed".to_string());
        }

        recommendations
    }

    /// Check if performance is acceptable
    pub fn is_acceptable(&self) -> bool {
        self.success_rate > 0.6 && 
        self.research_rate < 2.0 && 
        self.configuration_effectiveness > 0.5 &&
        self.memory_efficiency > 0.3
    }
}

/// Real-time performance monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimePerformance {
    /// Current number of searches performed
    pub current_searches: u64,
    /// Current success rate (0.0 to 1.0)
    pub current_success_rate: f64,
    /// Current research rate (0.0 to 1.0)
    pub current_research_rate: f64,
    /// Current average window size
    pub current_window_size: f64,
    /// Performance trend (positive = improving, negative = declining)
    pub performance_trend: f64,
    /// Current memory usage in bytes
    pub memory_usage: u64,
    /// Current configuration effectiveness (0.0 to 1.0)
    pub configuration_effectiveness: f64,
}

impl RealTimePerformance {
    /// Get performance status
    pub fn get_status(&self) -> String {
        if self.current_searches < 10 {
            "Insufficient data".to_string()
        } else if self.current_success_rate > 0.8 && self.current_research_rate < 1.0 {
            "Excellent".to_string()
        } else if self.current_success_rate > 0.7 && self.current_research_rate < 1.5 {
            "Good".to_string()
        } else if self.current_success_rate > 0.6 && self.current_research_rate < 2.0 {
            "Fair".to_string()
        } else {
            "Poor".to_string()
        }
    }

    /// Get performance alerts
    pub fn get_alerts(&self) -> Vec<String> {
        let mut alerts = Vec::new();
        
        if self.current_searches > 50 {
            if self.current_success_rate < 0.5 {
                alerts.push("Low success rate detected".to_string());
            }
            if self.current_research_rate > 2.0 {
                alerts.push("High research rate detected".to_string());
            }
            if self.performance_trend < -0.1 {
                alerts.push("Performance declining".to_string());
            }
            if self.configuration_effectiveness < 0.4 {
                alerts.push("Poor configuration effectiveness".to_string());
            }
        }
        
        alerts
    }

    /// Get performance summary
    pub fn summary(&self) -> String {
        format!(
            "Real-time Performance: {} searches, {:.1}% success, {:.2} research rate, {} status",
            self.current_searches,
            self.current_success_rate * 100.0,
            self.current_research_rate,
            self.get_status()
        )
    }
}

/// Main engine configuration containing all search optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Quiescence search configuration
    pub quiescence: QuiescenceConfig,
    /// Null move pruning configuration
    pub null_move: NullMoveConfig,
    /// Late move reductions configuration
    pub lmr: LMRConfig,
    /// Aspiration windows configuration
    pub aspiration_windows: AspirationWindowConfig,
    /// Transposition table size in MB
    pub tt_size_mb: usize,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Maximum search depth
    pub max_depth: u8,
    /// Time management settings
    pub time_management: TimeManagementConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            quiescence: QuiescenceConfig::default(),
            null_move: NullMoveConfig::default(),
            lmr: LMRConfig::default(),
            aspiration_windows: AspirationWindowConfig::default(),
            tt_size_mb: 64,
            debug_logging: false,
            max_depth: 20,
            time_management: TimeManagementConfig::default(),
        }
    }
}

impl EngineConfig {
    /// Create a new engine configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new engine configuration with custom settings
    pub fn new_custom(
        quiescence: QuiescenceConfig,
        null_move: NullMoveConfig,
        lmr: LMRConfig,
        aspiration_windows: AspirationWindowConfig,
        tt_size_mb: usize,
        debug_logging: bool,
        max_depth: u8,
        time_management: TimeManagementConfig,
    ) -> Self {
        Self {
            quiescence,
            null_move,
            lmr,
            aspiration_windows,
            tt_size_mb,
            debug_logging,
            max_depth,
            time_management,
        }
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate individual components
        self.quiescence.validate()?;
        self.null_move.validate()?;
        self.lmr.validate()?;
        self.aspiration_windows.validate()?;
        self.time_management.validate()?;

        // Validate global settings
        if self.tt_size_mb == 0 || self.tt_size_mb > 1024 {
            return Err("TT size must be between 1 and 1024 MB".to_string());
        }

        if self.max_depth == 0 || self.max_depth > 50 {
            return Err("Max depth must be between 1 and 50".to_string());
        }

        Ok(())
    }

    /// Get a configuration preset
    pub fn get_preset(preset: EnginePreset) -> Self {
        match preset {
            EnginePreset::Default => Self::default(),
            EnginePreset::Aggressive => Self {
                quiescence: QuiescenceConfig {
                    max_depth: 6,
                    enable_delta_pruning: true,
                    enable_futility_pruning: true,
                    enable_selective_extensions: true,
                    enable_tt: true,
                    futility_margin: 200,
                    delta_margin: 200,
                    tt_size_mb: 32,
                    tt_cleanup_threshold: 100000,
                },
                null_move: NullMoveConfig {
                    enabled: true,
                    min_depth: 3,
                    reduction_factor: 2,
                    max_pieces_threshold: 8,
                    enable_dynamic_reduction: true,
                    enable_endgame_detection: true,
                },
                lmr: LMRConfig {
                    enabled: true,
                    min_depth: 3,
                    min_move_index: 4,
                    base_reduction: 1,
                    max_reduction: 3,
                    enable_dynamic_reduction: true,
                    enable_adaptive_reduction: true,
                    enable_extended_exemptions: true,
                },
                aspiration_windows: AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 25,
                    dynamic_scaling: true,
                    max_window_size: 150,
                    min_depth: 2,
                    enable_adaptive_sizing: true,
                    max_researches: 2,
                    enable_statistics: true,
                },
                tt_size_mb: 128,
                debug_logging: false,
                max_depth: 25,
                time_management: TimeManagementConfig::default(),
            },
            EnginePreset::Conservative => Self {
                quiescence: QuiescenceConfig {
                    max_depth: 8,
                    enable_delta_pruning: true,
                    enable_futility_pruning: true,
                    enable_selective_extensions: true,
                    enable_tt: true,
                    futility_margin: 100,
                    delta_margin: 100,
                    tt_size_mb: 64,
                    tt_cleanup_threshold: 200000,
                },
                null_move: NullMoveConfig {
                    enabled: true,
                    min_depth: 4,
                    reduction_factor: 1,
                    max_pieces_threshold: 6,
                    enable_dynamic_reduction: false,
                    enable_endgame_detection: true,
                },
                lmr: LMRConfig {
                    enabled: true,
                    min_depth: 4,
                    min_move_index: 6,
                    base_reduction: 1,
                    max_reduction: 2,
                    enable_dynamic_reduction: false,
                    enable_adaptive_reduction: false,
                    enable_extended_exemptions: true,
                },
                aspiration_windows: AspirationWindowConfig {
                    enabled: true,
                    base_window_size: 100,
                    dynamic_scaling: true,
                    max_window_size: 300,
                    min_depth: 3,
                    enable_adaptive_sizing: true,
                    max_researches: 3,
                    enable_statistics: true,
                },
                tt_size_mb: 256,
                debug_logging: false,
                max_depth: 30,
                time_management: TimeManagementConfig::default(),
            },
            EnginePreset::Balanced => Self {
                quiescence: QuiescenceConfig::default(),
                null_move: NullMoveConfig::default(),
                lmr: LMRConfig::default(),
                aspiration_windows: AspirationWindowConfig::default(),
                tt_size_mb: 128,
                debug_logging: false,
                max_depth: 25,
                time_management: TimeManagementConfig::default(),
            },
        }
    }

    /// Apply a configuration preset
    pub fn apply_preset(&mut self, preset: EnginePreset) {
        *self = Self::get_preset(preset);
    }

    /// Get configuration summary
    pub fn summary(&self) -> String {
        format!(
            "Engine Config: TT={}MB, MaxDepth={}, Quiescence={}, NMP={}, LMR={}, Aspiration={}",
            self.tt_size_mb,
            self.max_depth,
            if self.quiescence.enable_tt { "ON" } else { "OFF" },
            if self.null_move.enabled { "ON" } else { "OFF" },
            if self.lmr.enabled { "ON" } else { "OFF" },
            if self.aspiration_windows.enabled { "ON" } else { "OFF" }
        )
    }
}

/// Engine configuration presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnginePreset {
    /// Default balanced configuration
    Default,
    /// Aggressive configuration for fast play
    Aggressive,
    /// Conservative configuration for careful analysis
    Conservative,
    /// Balanced configuration
    Balanced,
}

/// Time management configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeManagementConfig {
    /// Enable time management
    pub enabled: bool,
    /// Time buffer percentage (0.0 to 1.0)
    pub buffer_percentage: f64,
    /// Minimum time per move in milliseconds
    pub min_time_ms: u32,
    /// Maximum time per move in milliseconds
    pub max_time_ms: u32,
    /// Time increment per move in milliseconds
    pub increment_ms: u32,
    /// Enable time pressure detection
    pub enable_pressure_detection: bool,
    /// Time pressure threshold (0.0 to 1.0)
    pub pressure_threshold: f64,
}

impl Default for TimeManagementConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_percentage: 0.1,
            min_time_ms: 100,
            max_time_ms: 30000,
            increment_ms: 0,
            enable_pressure_detection: true,
            pressure_threshold: 0.2,
        }
    }
}

impl TimeManagementConfig {
    /// Validate time management configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.buffer_percentage < 0.0 || self.buffer_percentage > 1.0 {
            return Err("Buffer percentage must be between 0.0 and 1.0".to_string());
        }

        if self.min_time_ms > self.max_time_ms {
            return Err("Min time cannot be greater than max time".to_string());
        }

        if self.pressure_threshold < 0.0 || self.pressure_threshold > 1.0 {
            return Err("Pressure threshold must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }

    /// Calculate time allocation for a move
    pub fn calculate_time_allocation(&self, total_time_ms: u32, moves_remaining: u32) -> u32 {
        if !self.enabled || moves_remaining == 0 {
            return self.min_time_ms;
        }

        let base_time = total_time_ms / moves_remaining;
        let buffered_time = (base_time as f64 * (1.0 - self.buffer_percentage)) as u32;
        
        buffered_time.max(self.min_time_ms).min(self.max_time_ms)
    }

    /// Check if in time pressure
    pub fn is_time_pressure(&self, time_remaining_ms: u32, total_time_ms: u32) -> bool {
        if !self.enable_pressure_detection || total_time_ms == 0 {
            return false;
        }

        let time_ratio = time_remaining_ms as f64 / total_time_ms as f64;
        time_ratio < self.pressure_threshold
    }
}

/// Configuration migration utilities
pub struct ConfigMigration;

impl ConfigMigration {
    /// Migrate from old configuration format to new EngineConfig
    pub fn migrate_from_legacy(
        quiescence_config: QuiescenceConfig,
        null_move_config: NullMoveConfig,
        lmr_config: LMRConfig,
        aspiration_config: AspirationWindowConfig,
        tt_size_mb: usize,
    ) -> EngineConfig {
        EngineConfig {
            quiescence: quiescence_config,
            null_move: null_move_config,
            lmr: lmr_config,
            aspiration_windows: aspiration_config,
            tt_size_mb,
            debug_logging: false,
            max_depth: 20,
            time_management: TimeManagementConfig::default(),
        }
    }

    /// Create a configuration from individual components
    pub fn create_from_components(
        quiescence: QuiescenceConfig,
        null_move: NullMoveConfig,
        lmr: LMRConfig,
        aspiration_windows: AspirationWindowConfig,
        tt_size_mb: usize,
        debug_logging: bool,
        max_depth: u8,
        time_management: TimeManagementConfig,
    ) -> EngineConfig {
        EngineConfig::new_custom(
            quiescence,
            null_move,
            lmr,
            aspiration_windows,
            tt_size_mb,
            debug_logging,
            max_depth,
            time_management,
        )
    }

    /// Validate and fix configuration issues
    pub fn validate_and_fix(mut config: EngineConfig) -> Result<EngineConfig, String> {
        // Fix common issues
        if config.tt_size_mb == 0 {
            config.tt_size_mb = 64;
        }
        if config.max_depth == 0 {
            config.max_depth = 20;
        }
        if config.max_depth > 50 {
            config.max_depth = 50;
        }

        // Validate the fixed configuration
        config.validate()?;
        Ok(config)
    }

    /// Get configuration recommendations based on system resources
    pub fn get_recommendations_for_system(available_memory_mb: usize) -> EngineConfig {
        let mut config = EngineConfig::default();
        
        // Adjust TT size based on available memory
        if available_memory_mb >= 1024 {
            config.tt_size_mb = 256;
            config.quiescence.tt_size_mb = 64;
        } else if available_memory_mb >= 512 {
            config.tt_size_mb = 128;
            config.quiescence.tt_size_mb = 32;
        } else {
            config.tt_size_mb = 64;
            config.quiescence.tt_size_mb = 16;
        }

        // Adjust max depth based on available memory
        if available_memory_mb >= 2048 {
            config.max_depth = 30;
        } else if available_memory_mb >= 1024 {
            config.max_depth = 25;
        } else {
            config.max_depth = 20;
        }

        config
    }

    /// Export configuration to JSON
    pub fn export_to_json(config: &EngineConfig) -> Result<String, String> {
        serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))
    }

    /// Import configuration from JSON
    pub fn import_from_json(json: &str) -> Result<EngineConfig, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize configuration: {}", e))
    }

    /// Compare two configurations
    pub fn compare_configs(config1: &EngineConfig, config2: &EngineConfig) -> ConfigComparison {
        ConfigComparison {
            quiescence_different: config1.quiescence != config2.quiescence,
            null_move_different: config1.null_move != config2.null_move,
            lmr_different: config1.lmr != config2.lmr,
            aspiration_different: config1.aspiration_windows != config2.aspiration_windows,
            tt_size_different: config1.tt_size_mb != config2.tt_size_mb,
            max_depth_different: config1.max_depth != config2.max_depth,
            time_management_different: config1.time_management != config2.time_management,
        }
    }
}

/// Configuration comparison result
#[derive(Debug, Clone)]
pub struct ConfigComparison {
    pub quiescence_different: bool,
    pub null_move_different: bool,
    pub lmr_different: bool,
    pub aspiration_different: bool,
    pub tt_size_different: bool,
    pub max_depth_different: bool,
    pub time_management_different: bool,
}

impl ConfigComparison {
    /// Check if any configuration is different
    pub fn has_differences(&self) -> bool {
        self.quiescence_different ||
        self.null_move_different ||
        self.lmr_different ||
        self.aspiration_different ||
        self.tt_size_different ||
        self.max_depth_different ||
        self.time_management_different
    }

    /// Get summary of differences
    pub fn get_differences_summary(&self) -> Vec<String> {
        let mut differences = Vec::new();
        
        if self.quiescence_different {
            differences.push("Quiescence configuration".to_string());
        }
        if self.null_move_different {
            differences.push("Null move configuration".to_string());
        }
        if self.lmr_different {
            differences.push("LMR configuration".to_string());
        }
        if self.aspiration_different {
            differences.push("Aspiration windows configuration".to_string());
        }
        if self.tt_size_different {
            differences.push("Transposition table size".to_string());
        }
        if self.max_depth_different {
            differences.push("Maximum depth".to_string());
        }
        if self.time_management_different {
            differences.push("Time management configuration".to_string());
        }
        
        differences
    }
}
