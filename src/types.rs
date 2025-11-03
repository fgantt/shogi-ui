use serde::{Serialize, Deserialize};

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
            crate::debug_utils::debug_log(&format!(
                "[PIECE::NEW ERROR] Invalid piece_type with to_u8() = {}. Defaulting to Pawn.",
                piece_idx
            ));
            return Self { piece_type: PieceType::Pawn, player };
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
        }.to_string();

        if self.player == Player::Black {
            fen_char = fen_char.to_uppercase();
        }

        fen_char
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Impasse (Jishōgi / 持将棋) detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpasseResult {
    pub black_points: i32,
    pub white_points: i32,
    pub outcome: ImpasseOutcome,
}

/// Possible outcomes of an impasse situation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpasseOutcome {
    Draw,         // Both players have 24+ points
    BlackWins,    // White has < 24 points
    WhiteWins,    // Black has < 24 points
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspositionEntry {
    pub score: i32,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub best_move: Option<Move>,
    /// Hash key for this entry (used for collision detection)
    pub hash_key: u64,
    /// Age counter for replacement policies
    pub age: u32,
}

impl TranspositionEntry {
    /// Create a new transposition table entry
    pub fn new(score: i32, depth: u8, flag: TranspositionFlag, best_move: Option<Move>, hash_key: u64, age: u32) -> Self {
        Self {
            score,
            depth,
            flag,
            best_move,
            hash_key,
            age,
        }
    }
    
    /// Create a new entry with default age (0)
    pub fn new_with_age(score: i32, depth: u8, flag: TranspositionFlag, best_move: Option<Move>, hash_key: u64) -> Self {
        Self::new(score, depth, flag, best_move, hash_key, 0)
    }
    
    /// Check if this entry is valid for the given search depth
    pub fn is_valid_for_depth(&self, required_depth: u8) -> bool {
        self.depth >= required_depth
    }
    
    /// Check if this entry matches the given hash key
    pub fn matches_hash(&self, hash_key: u64) -> bool {
        self.hash_key == hash_key
    }
    
    /// Check if this entry is exact (not a bound)
    pub fn is_exact(&self) -> bool {
        matches!(self.flag, TranspositionFlag::Exact)
    }
    
    /// Check if this entry is a lower bound
    pub fn is_lower_bound(&self) -> bool {
        matches!(self.flag, TranspositionFlag::LowerBound)
    }
    
    /// Check if this entry is an upper bound
    pub fn is_upper_bound(&self) -> bool {
        matches!(self.flag, TranspositionFlag::UpperBound)
    }
    
    /// Update the age of this entry
    pub fn update_age(&mut self, new_age: u32) {
        self.age = new_age;
    }
    
    /// Get the memory size of this entry in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    
    /// Create a debug string representation
    pub fn debug_string(&self) -> String {
        let move_str = match &self.best_move {
            Some(m) => format!("{}", m.to_usi_string()),
            None => "None".to_string(),
        };
        
        format!(
            "TranspositionEntry {{ score: {}, depth: {}, flag: {:?}, best_move: {}, hash_key: 0x{:016x}, age: {} }}",
            self.score, self.depth, self.flag, move_str, self.hash_key, self.age
        )
    }
    
    /// Check if this entry should be replaced by another entry
    pub fn should_replace_with(&self, other: &TranspositionEntry) -> bool {
        // Replace if hash keys don't match (collision)
        if !self.matches_hash(other.hash_key) {
            return true;
        }
        
        // Replace if the new entry has greater depth
        if other.depth > self.depth {
            return true;
        }
        
        // Replace if depths are equal but new entry is exact and current is not
        if other.depth == self.depth && other.is_exact() && !self.is_exact() {
            return true;
        }
        
        // Replace if the new entry is newer (higher age)
        if other.age > self.age {
            return true;
        }
        
        false
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

impl TranspositionFlag {
    /// Get a string representation of the flag
    pub fn to_string(&self) -> &'static str {
        match self {
            TranspositionFlag::Exact => "Exact",
            TranspositionFlag::LowerBound => "LowerBound",
            TranspositionFlag::UpperBound => "UpperBound",
        }
    }
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

// ============================================================================
// FEATURE EXTRACTION CONSTANTS FOR AUTOMATED TUNING
// ============================================================================

/// Total number of evaluation features for tuning
pub const NUM_EVAL_FEATURES: usize = 2000;

/// Number of middlegame features (first half of feature vector)
pub const NUM_MG_FEATURES: usize = NUM_EVAL_FEATURES / 2;

/// Number of endgame features (second half of feature vector)
pub const NUM_EG_FEATURES: usize = NUM_EVAL_FEATURES / 2;

// Material feature indices (14 piece types × 2 players = 28 features)
pub const MATERIAL_PAWN_INDEX: usize = 0;
pub const MATERIAL_LANCE_INDEX: usize = 1;
pub const MATERIAL_KNIGHT_INDEX: usize = 2;
pub const MATERIAL_SILVER_INDEX: usize = 3;
pub const MATERIAL_GOLD_INDEX: usize = 4;
pub const MATERIAL_BISHOP_INDEX: usize = 5;
pub const MATERIAL_ROOK_INDEX: usize = 6;
pub const MATERIAL_KING_INDEX: usize = 7;
pub const MATERIAL_PROMOTED_PAWN_INDEX: usize = 8;
pub const MATERIAL_PROMOTED_LANCE_INDEX: usize = 9;
pub const MATERIAL_PROMOTED_KNIGHT_INDEX: usize = 10;
pub const MATERIAL_PROMOTED_SILVER_INDEX: usize = 11;
pub const MATERIAL_PROMOTED_BISHOP_INDEX: usize = 12;
pub const MATERIAL_PROMOTED_ROOK_INDEX: usize = 13;
pub const MATERIAL_WHITE_PAWN_INDEX: usize = 14;
pub const MATERIAL_WHITE_LANCE_INDEX: usize = 15;
pub const MATERIAL_WHITE_KNIGHT_INDEX: usize = 16;
pub const MATERIAL_WHITE_SILVER_INDEX: usize = 17;
pub const MATERIAL_WHITE_GOLD_INDEX: usize = 18;
pub const MATERIAL_WHITE_BISHOP_INDEX: usize = 19;
pub const MATERIAL_WHITE_ROOK_INDEX: usize = 20;
pub const MATERIAL_WHITE_KING_INDEX: usize = 21;
pub const MATERIAL_WHITE_PROMOTED_PAWN_INDEX: usize = 22;
pub const MATERIAL_WHITE_PROMOTED_LANCE_INDEX: usize = 23;
pub const MATERIAL_WHITE_PROMOTED_KNIGHT_INDEX: usize = 24;
pub const MATERIAL_WHITE_PROMOTED_SILVER_INDEX: usize = 25;
pub const MATERIAL_WHITE_PROMOTED_BISHOP_INDEX: usize = 26;
pub const MATERIAL_WHITE_PROMOTED_ROOK_INDEX: usize = 27;

// Positional features (piece-square tables)
pub const PST_PAWN_MG_START: usize = 28;
pub const PST_PAWN_EG_START: usize = PST_PAWN_MG_START + 81;
pub const PST_LANCE_MG_START: usize = PST_PAWN_EG_START + 81;
pub const PST_LANCE_EG_START: usize = PST_LANCE_MG_START + 81;
pub const PST_KNIGHT_MG_START: usize = PST_LANCE_EG_START + 81;
pub const PST_KNIGHT_EG_START: usize = PST_KNIGHT_MG_START + 81;
pub const PST_SILVER_MG_START: usize = PST_KNIGHT_EG_START + 81;
pub const PST_SILVER_EG_START: usize = PST_SILVER_MG_START + 81;
pub const PST_GOLD_MG_START: usize = PST_SILVER_EG_START + 81;
pub const PST_GOLD_EG_START: usize = PST_GOLD_MG_START + 81;
pub const PST_BISHOP_MG_START: usize = PST_GOLD_EG_START + 81;
pub const PST_BISHOP_EG_START: usize = PST_BISHOP_MG_START + 81;
pub const PST_ROOK_MG_START: usize = PST_BISHOP_EG_START + 81;
pub const PST_ROOK_EG_START: usize = PST_ROOK_MG_START + 81;

// King safety features
pub const KING_SAFETY_CASTLE_INDEX: usize = 500;
pub const KING_SAFETY_ATTACK_INDEX: usize = 501;
pub const KING_SAFETY_THREAT_INDEX: usize = 502;
pub const KING_SAFETY_SHIELD_INDEX: usize = 503;
pub const KING_SAFETY_EXPOSURE_INDEX: usize = 504;

// Pawn structure features
pub const PAWN_STRUCTURE_CHAINS_INDEX: usize = 600;
pub const PAWN_STRUCTURE_ADVANCEMENT_INDEX: usize = 601;
pub const PAWN_STRUCTURE_ISOLATION_INDEX: usize = 602;
pub const PAWN_STRUCTURE_PASSED_INDEX: usize = 603;
pub const PAWN_STRUCTURE_BACKWARD_INDEX: usize = 604;

// Mobility features
pub const MOBILITY_TOTAL_MOVES_INDEX: usize = 700;
pub const MOBILITY_PIECE_MOVES_INDEX: usize = 701;
pub const MOBILITY_ATTACK_MOVES_INDEX: usize = 702;
pub const MOBILITY_DEFENSE_MOVES_INDEX: usize = 703;

// Coordination features
pub const COORDINATION_CONNECTED_ROOKS_INDEX: usize = 800;
pub const COORDINATION_BISHOP_PAIR_INDEX: usize = 801;
pub const COORDINATION_ATTACK_PATTERNS_INDEX: usize = 802;
pub const COORDINATION_PIECE_SUPPORT_INDEX: usize = 803;

// Center control features
pub const CENTER_CONTROL_CENTER_SQUARES_INDEX: usize = 900;
pub const CENTER_CONTROL_OUTPOST_INDEX: usize = 901;
pub const CENTER_CONTROL_SPACE_INDEX: usize = 902;

// Development features
pub const DEVELOPMENT_MAJOR_PIECES_INDEX: usize = 1000;
pub const DEVELOPMENT_MINOR_PIECES_INDEX: usize = 1001;
pub const DEVELOPMENT_CASTLING_INDEX: usize = 1002;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Endgame type classification for intelligent endgame detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EndgameType {
    /// Not in endgame (too many pieces)
    NotEndgame,
    /// Material endgame: low piece count, mostly pawns and minor pieces
    MaterialEndgame,
    /// King activity endgame: kings are active and centralized
    KingActivityEndgame,
    /// Zugzwang-prone endgame: positions where any move may worsen the position
    ZugzwangEndgame,
}

impl EndgameType {
    /// Get a string representation of the endgame type
    pub fn to_string(&self) -> &'static str {
        match self {
            EndgameType::NotEndgame => "NotEndgame",
            EndgameType::MaterialEndgame => "MaterialEndgame",
            EndgameType::KingActivityEndgame => "KingActivityEndgame",
            EndgameType::ZugzwangEndgame => "ZugzwangEndgame",
        }
    }
}

/// Dynamic reduction formula options for null move pruning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DynamicReductionFormula {
    /// Static reduction: always use reduction_factor
    Static,
    /// Linear reduction: R = 2 + depth / 6 (integer division, creates steps)
    Linear,
    /// Smooth reduction: R = 2 + (depth / 6.0).round() (floating-point with rounding for smoother scaling)
    Smooth,
}

impl Default for DynamicReductionFormula {
    fn default() -> Self {
        DynamicReductionFormula::Linear
    }
}

impl DynamicReductionFormula {
    /// Calculate reduction value for given depth and base reduction
    /// 
    /// # Formula Selection Guidelines
    /// 
    /// **Static**: Always returns base_reduction. Most conservative approach.
    /// - Use when: You want consistent, predictable reduction regardless of depth
    /// - Best for: Positions requiring maximum safety
    /// 
    /// **Linear**: R = base_reduction + depth / 6 (integer division)
    /// - Use when: You want step-wise scaling that increases at multiples of 6
    /// - Behavior: Creates steps - depth 3-5 -> R=base, 6-11 -> R=base+1, 12-17 -> R=base+2
    /// - Best for: General play with predictable reduction scaling
    /// 
    /// **Smooth**: R = base_reduction + (depth / 6.0).round()
    /// - Use when: You want smoother, more gradual scaling without large steps
    /// - Behavior: Increases reduction earlier than Linear (e.g., depth 3-5 -> R=base+1)
    /// - Best for: Positions where smoother scaling improves NMP effectiveness
    /// 
    /// # Examples (with base_reduction = 2)
    /// ```
    /// // At depth 3
    /// Static: 2
    /// Linear: 2 + 3/6 = 2
    /// Smooth: 2 + (3/6.0).round() = 3
    /// 
    /// // At depth 9
    /// Static: 2
    /// Linear: 2 + 9/6 = 3
    /// Smooth: 2 + (9/6.0).round() = 4
    /// 
    /// // At depth 12
    /// Static: 2
    /// Linear: 2 + 12/6 = 4
    /// Smooth: 2 + (12/6.0).round() = 4
    /// ```
    pub fn calculate_reduction(&self, depth: u8, base_reduction: u8) -> u8 {
        match self {
            DynamicReductionFormula::Static => base_reduction,
            DynamicReductionFormula::Linear => {
                // Linear: R = base_reduction + depth / 6
                // This creates steps: depth 3-5 -> R=base+0, depth 6-11 -> R=base+1, etc.
                base_reduction + (depth / 6)
            }
            DynamicReductionFormula::Smooth => {
                // Smooth: R = base_reduction + (depth / 6.0).round()
                // Uses floating-point division with rounding for smoother scaling
                let reduction_add = (depth as f32 / 6.0).round() as u8;
                base_reduction + reduction_add
            }
        }
    }
}

/// Preset configurations for Null Move Pruning
/// 
/// These presets provide pre-configured settings optimized for different playing styles:
/// - **Conservative**: Higher safety margins, lower reduction, stricter endgame detection (safer but slower)
/// - **Aggressive**: Lower safety margins, higher reduction, relaxed endgame detection (faster but riskier)
/// - **Balanced**: Default values optimized for general play (good balance of speed and safety)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NullMovePreset {
    /// Conservative preset: Higher verification_margin, lower reduction_factor, stricter endgame detection
    /// Best for: Critical positions, endgame analysis, when safety is more important than speed
    Conservative,
    /// Aggressive preset: Lower verification_margin, higher reduction_factor, relaxed endgame detection
    /// Best for: Fast time controls, opening/middlegame, when speed is more important than safety
    Aggressive,
    /// Balanced preset: Default values optimized for general play
    /// Best for: Standard time controls, general use cases
    Balanced,
}

impl NullMovePreset {
    /// Get a string representation of the preset
    pub fn to_string(&self) -> &'static str {
        match self {
            NullMovePreset::Conservative => "Conservative",
            NullMovePreset::Aggressive => "Aggressive",
            NullMovePreset::Balanced => "Balanced",
        }
    }
    
    /// Parse a preset from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "conservative" => Some(NullMovePreset::Conservative),
            "aggressive" => Some(NullMovePreset::Aggressive),
            "balanced" => Some(NullMovePreset::Balanced),
            _ => None,
        }
    }
}

/// Configuration for null move pruning parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NullMoveConfig {
    pub enabled: bool,                      // Enable null move pruning
    pub min_depth: u8,                      // Minimum depth to use NMP
    pub reduction_factor: u8,               // Static reduction factor (R)
    pub max_pieces_threshold: u8,           // Disable NMP when pieces < threshold
    pub enable_dynamic_reduction: bool,     // Use dynamic reduction (deprecated, use dynamic_reduction_formula instead)
    pub enable_endgame_detection: bool,     // Disable NMP in endgame
    pub verification_margin: i32,           // Safety margin for verification search (centipawns)
    pub dynamic_reduction_formula: DynamicReductionFormula,  // Formula for dynamic reduction calculation
    pub enable_mate_threat_detection: bool,  // Enable mate threat detection (default: false, opt-in feature)
    pub mate_threat_margin: i32,            // Threshold for mate threat detection (default: 500 centipawns)
    pub enable_endgame_type_detection: bool,  // Enable endgame type detection (default: false, opt-in feature)
    pub material_endgame_threshold: u8,       // Threshold for material endgame detection (default: 12 pieces)
    pub king_activity_threshold: u8,          // Threshold for king activity endgame detection (default: 8 pieces)
    pub zugzwang_threshold: u8,              // Threshold for zugzwang-prone endgame detection (default: 6 pieces)
    pub preset: Option<NullMovePreset>,      // Optional: Track which preset was used to create this config
}

impl Default for NullMoveConfig {
    fn default() -> Self {
        // Default uses Balanced preset values
        NullMoveConfig::from_preset(NullMovePreset::Balanced)
    }
}

impl NullMoveConfig {
    /// Create a configuration from a preset
    /// 
    /// This creates a new `NullMoveConfig` with settings optimized for the specified preset:
    /// - **Conservative**: Higher verification_margin (400), lower reduction_factor (2), stricter endgame detection
    /// - **Aggressive**: Lower verification_margin (100), higher reduction_factor (3), relaxed endgame detection
    /// - **Balanced**: Default values optimized for general play (verification_margin: 200, reduction_factor: 2)
    pub fn from_preset(preset: NullMovePreset) -> Self {
        match preset {
            NullMovePreset::Conservative => Self {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,           // Lower reduction for safety
                max_pieces_threshold: 14,      // Stricter endgame detection (disable when < 14 pieces)
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 400,       // Higher safety margin (400 centipawns)
                dynamic_reduction_formula: DynamicReductionFormula::Linear,
                enable_mate_threat_detection: true,  // Enable mate threat detection for safety
                mate_threat_margin: 600,       // Higher mate threat margin (600 centipawns)
                enable_endgame_type_detection: true,  // Enable endgame type detection
                material_endgame_threshold: 14,  // Higher threshold (14 pieces)
                king_activity_threshold: 10,    // Higher threshold (10 pieces)
                zugzwang_threshold: 8,         // Higher threshold (8 pieces)
                preset: Some(NullMovePreset::Conservative),
            },
            NullMovePreset::Aggressive => Self {
                enabled: true,
                min_depth: 2,                  // Lower min_depth for more aggressiveness
                reduction_factor: 3,            // Higher reduction for speed
                max_pieces_threshold: 10,       // Relaxed endgame detection (disable when < 10 pieces)
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 100,       // Lower safety margin (100 centipawns)
                dynamic_reduction_formula: DynamicReductionFormula::Smooth,  // Use smooth formula for better scaling
                enable_mate_threat_detection: false,  // Disable mate threat detection for speed
                mate_threat_margin: 400,        // Lower mate threat margin (400 centipawns)
                enable_endgame_type_detection: false,  // Disable endgame type detection for speed
                material_endgame_threshold: 10,  // Lower threshold (10 pieces)
                king_activity_threshold: 6,     // Lower threshold (6 pieces)
                zugzwang_threshold: 4,          // Lower threshold (4 pieces)
                preset: Some(NullMovePreset::Aggressive),
            },
            NullMovePreset::Balanced => Self {
                enabled: true,
                min_depth: 3,
                reduction_factor: 2,
                max_pieces_threshold: 12,       // Standard endgame detection
                enable_dynamic_reduction: true,
                enable_endgame_detection: true,
                verification_margin: 200,       // Default safety margin (200 centipawns)
                dynamic_reduction_formula: DynamicReductionFormula::Linear,
                enable_mate_threat_detection: false,  // Disabled by default (opt-in)
                mate_threat_margin: 500,        // Default mate threat margin (500 centipawns)
                enable_endgame_type_detection: false,  // Disabled by default (opt-in)
                material_endgame_threshold: 12,  // Default threshold (12 pieces)
                king_activity_threshold: 8,      // Default threshold (8 pieces)
                zugzwang_threshold: 6,          // Default threshold (6 pieces)
                preset: Some(NullMovePreset::Balanced),
            },
        }
    }
    
    /// Apply a preset to this configuration
    /// 
    /// This updates the configuration with settings from the specified preset,
    /// preserving the preset reference for tracking.
    pub fn apply_preset(&mut self, preset: NullMovePreset) {
        let preset_config = Self::from_preset(preset);
        *self = preset_config;
    }
    
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
        if self.verification_margin < 0 {
            return Err("verification_margin must be non-negative".to_string());
        }
        if self.verification_margin > 1000 {
            return Err("verification_margin should not exceed 1000 centipawns".to_string());
        }
        if self.mate_threat_margin < 0 {
            return Err("mate_threat_margin must be non-negative".to_string());
        }
        if self.mate_threat_margin > 2000 {
            return Err("mate_threat_margin should not exceed 2000 centipawns".to_string());
        }
        if self.material_endgame_threshold == 0 {
            return Err("material_endgame_threshold must be greater than 0".to_string());
        }
        if self.material_endgame_threshold > 40 {
            return Err("material_endgame_threshold should not exceed 40".to_string());
        }
        if self.king_activity_threshold == 0 {
            return Err("king_activity_threshold must be greater than 0".to_string());
        }
        if self.king_activity_threshold > 40 {
            return Err("king_activity_threshold should not exceed 40".to_string());
        }
        if self.zugzwang_threshold == 0 {
            return Err("zugzwang_threshold must be greater than 0".to_string());
        }
        if self.zugzwang_threshold > 40 {
            return Err("zugzwang_threshold should not exceed 40".to_string());
        }
        Ok(())
    }

    /// Create a validated configuration, clamping values to valid ranges
    pub fn new_validated(mut self) -> Self {
        self.min_depth = self.min_depth.clamp(1, 10);
        self.reduction_factor = self.reduction_factor.clamp(1, 5);
        self.max_pieces_threshold = self.max_pieces_threshold.clamp(1, 40);
        self.verification_margin = self.verification_margin.clamp(0, 1000);
        self.mate_threat_margin = self.mate_threat_margin.clamp(0, 2000);
        self.material_endgame_threshold = self.material_endgame_threshold.clamp(1, 40);
        self.king_activity_threshold = self.king_activity_threshold.clamp(1, 40);
        self.zugzwang_threshold = self.zugzwang_threshold.clamp(1, 40);
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        let preset_str = if let Some(preset) = &self.preset {
            format!(", preset={}", preset.to_string())
        } else {
            String::new()
        };
        format!(
            "NullMoveConfig: enabled={}, min_depth={}, reduction_factor={}, max_pieces_threshold={}, dynamic_reduction={}, endgame_detection={}, verification_margin={}, reduction_formula={:?}, mate_threat_detection={}, mate_threat_margin={}, endgame_type_detection={}, material_endgame_threshold={}, king_activity_threshold={}, zugzwang_threshold={}{}",
            self.enabled,
            self.min_depth,
            self.reduction_factor,
            self.max_pieces_threshold,
            self.enable_dynamic_reduction,
            self.enable_endgame_detection,
            self.verification_margin,
            self.dynamic_reduction_formula,
            self.enable_mate_threat_detection,
            self.mate_threat_margin,
            self.enable_endgame_type_detection,
            self.material_endgame_threshold,
            self.king_activity_threshold,
            self.zugzwang_threshold,
            preset_str
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
    pub verification_attempts: u64,         // Number of verification searches attempted
    pub verification_cutoffs: u64,           // Number of verification searches that resulted in cutoffs
    pub mate_threat_attempts: u64,           // Number of mate threat detection attempts
    pub mate_threat_detected: u64,            // Number of mate threats detected and verified
    pub disabled_material_endgame: u64,      // Times disabled due to material endgame detection
    pub disabled_king_activity_endgame: u64,  // Times disabled due to king activity endgame detection
    pub disabled_zugzwang: u64,              // Times disabled due to zugzwang-prone endgame detection
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

    /// Get the verification search cutoff rate as a percentage
    pub fn verification_cutoff_rate(&self) -> f64 {
        if self.verification_attempts == 0 {
            return 0.0;
        }
        (self.verification_cutoffs as f64 / self.verification_attempts as f64) * 100.0
    }

    /// Get the mate threat detection rate as a percentage
    pub fn mate_threat_detection_rate(&self) -> f64 {
        if self.mate_threat_attempts == 0 {
            return 0.0;
        }
        (self.mate_threat_detected as f64 / self.mate_threat_attempts as f64) * 100.0
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Null Move Pruning Performance Report:\n\
            - Attempts: {}\n\
            - Cutoffs: {} ({:.2}%)\n\
            - Total disabled: {} ({} in check, {} endgame)\n\
            - Average reduction: {:.2}\n\
            - Efficiency: {:.2}%\n\
            - Verification attempts: {}\n\
            - Verification cutoffs: {} ({:.2}%)\n\
            - Mate threat attempts: {}\n\
            - Mate threats detected: {} ({:.2}%)\n\
            - Disabled material endgame: {}\n\
            - Disabled king activity endgame: {}\n\
            - Disabled zugzwang: {}",
            self.attempts,
            self.cutoffs,
            self.cutoff_rate(),
            self.total_disabled(),
            self.disabled_in_check,
            self.disabled_endgame,
            self.average_reduction_factor(),
            self.efficiency(),
            self.verification_attempts,
            self.verification_cutoffs,
            self.verification_cutoff_rate(),
            self.mate_threat_attempts,
            self.mate_threat_detected,
            self.mate_threat_detection_rate(),
            self.disabled_material_endgame,
            self.disabled_king_activity_endgame,
            self.disabled_zugzwang
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionComplexity {
    Low,
    Medium,
    High,
    Unknown,
}

/// Efficient board state representation for IID search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDBoardState {
    /// Compact position key for quick comparison
    pub key: u64,
    /// Material balance (Black - White)
    pub material_balance: i32,
    /// Total number of pieces on board
    pub piece_count: u8,
    /// King positions (Black, White)
    pub king_positions: (Option<Position>, Option<Position>),
    /// Cached move generation results
    pub move_cache: Option<Vec<Move>>,
}

/// Statistics for IID overhead monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDOverheadStats {
    /// Total number of IID searches performed
    pub total_searches: u64,
    /// Number of searches skipped due to time pressure
    pub time_pressure_skips: u64,
    /// Current overhead threshold percentage
    pub current_threshold: f64,
    /// Average overhead percentage
    pub average_overhead: f64,
    /// Number of threshold adjustments made
    pub threshold_adjustments: u32,
}

/// Result of a multi-PV IID search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDPVResult {
    /// The best move for this PV
    pub move_: Move,
    /// Evaluation score for this PV
    pub score: i32,
    /// Search depth used
    pub depth: u8,
    /// Complete principal variation
    pub principal_variation: Vec<Move>,
    /// Index of this PV (0 = best, 1 = second best, etc.)
    pub pv_index: usize,
    /// Time taken for this PV search in milliseconds
    pub search_time_ms: u32,
}

/// Analysis of multiple principal variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPVAnalysis {
    /// Total number of PVs found
    pub total_pvs: usize,
    /// Spread between best and worst PV scores
    pub score_spread: f64,
    /// Tactical themes identified across PVs
    pub tactical_themes: Vec<TacticalTheme>,
    /// Diversity of moves across PVs (0.0 to 1.0)
    pub move_diversity: f64,
    /// Overall complexity assessment
    pub complexity_assessment: PositionComplexity,
}

/// Tactical themes in chess positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TacticalTheme {
    /// Capture moves
    Capture,
    /// Check moves
    Check,
    /// Promotion moves
    Promotion,
    /// Piece development
    Development,
    /// Positional moves
    Positional,
}

/// A move identified as promising in shallow IID search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromisingMove {
    /// The promising move
    pub move_: Move,
    /// Score from shallow search
    pub shallow_score: i32,
    /// Improvement over current alpha
    pub improvement_over_alpha: i32,
    /// Tactical indicators for this move
    pub tactical_indicators: TacticalIndicators,
}

/// Result of IID probing with deeper verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDProbeResult {
    /// The verified move
    pub move_: Move,
    /// Score from shallow search
    pub shallow_score: i32,
    /// Score from deeper search
    pub deep_score: i32,
    /// Difference between shallow and deep scores
    pub score_difference: i32,
    /// Confidence in the verification (0.0 to 1.0)
    pub verification_confidence: f64,
    /// Tactical indicators for this move
    pub tactical_indicators: TacticalIndicators,
    /// Depth used for probing
    pub probe_depth: u8,
    /// Time taken for probing in milliseconds
    pub search_time_ms: u32,
}

/// Tactical indicators for move assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalIndicators {
    /// Whether the move is a capture
    pub is_capture: bool,
    /// Whether the move is a promotion
    pub is_promotion: bool,
    /// Whether the move gives check
    pub gives_check: bool,
    /// Whether the move is a recapture
    pub is_recapture: bool,
    /// Piece value involved in the move
    pub piece_value: i32,
    /// Estimated mobility impact
    pub mobility_impact: i32,
    /// Estimated king safety impact
    pub king_safety_impact: i32,
}

/// Performance benchmark results for IID vs non-IID search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDPerformanceBenchmark {
    /// Number of benchmark iterations
    pub iterations: usize,
    /// Search depth used
    pub depth: u8,
    /// Time limit per search in milliseconds
    pub time_limit_ms: u32,
    /// IID search times for each iteration
    pub iid_times: Vec<u32>,
    /// Non-IID search times for each iteration
    pub non_iid_times: Vec<u32>,
    /// IID nodes searched for each iteration
    pub iid_nodes: Vec<u64>,
    /// Score differences between IID and non-IID results
    pub score_differences: Vec<i32>,
    /// Average IID search time
    pub avg_iid_time: f64,
    /// Average non-IID search time
    pub avg_non_iid_time: f64,
    /// Average IID nodes searched
    pub avg_iid_nodes: f64,
    /// Average score difference
    pub avg_score_difference: f64,
    /// Time efficiency percentage (positive = IID faster)
    pub time_efficiency: f64,
    /// Node efficiency (nodes per millisecond)
    pub node_efficiency: f64,
    /// Accuracy assessment
    pub accuracy: String,
}

impl Default for IIDPerformanceBenchmark {
    fn default() -> Self {
        Self {
            iterations: 0,
            depth: 0,
            time_limit_ms: 0,
            iid_times: Vec::new(),
            non_iid_times: Vec::new(),
            iid_nodes: Vec::new(),
            score_differences: Vec::new(),
            avg_iid_time: 0.0,
            avg_non_iid_time: 0.0,
            avg_iid_nodes: 0.0,
            avg_score_difference: 0.0,
            time_efficiency: 0.0,
            node_efficiency: 0.0,
            accuracy: "Unknown".to_string(),
        }
    }
}

/// Detailed performance analysis for IID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDPerformanceAnalysis {
    /// Overall efficiency metric
    pub overall_efficiency: f64,
    /// Cutoff rate achieved
    pub cutoff_rate: f64,
    /// Overhead percentage
    pub overhead_percentage: f64,
    /// Success rate of IID moves
    pub success_rate: f64,
    /// Performance recommendations
    pub recommendations: Vec<String>,
    /// Identified bottlenecks
    pub bottleneck_analysis: Vec<String>,
    /// Optimization potential assessment
    pub optimization_potential: String,
}

/// Game result for strength testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    /// Win for the player
    Win,
    /// Loss for the player
    Loss,
    /// Draw
    Draw,
}

/// Position difficulty for strength testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionDifficulty {
    /// Easy position
    Easy,
    /// Medium difficulty
    Medium,
    /// Hard position
    Hard,
}

/// Confidence level for strength test analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Low confidence
    Low,
    /// Medium confidence
    Medium,
    /// High confidence
    High,
}

/// Test position for strength testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthTestPosition {
    /// FEN string of the position
    pub fen: String,
    /// Description of the position
    pub description: String,
    /// Expected game result
    pub expected_result: GameResult,
    /// Difficulty level
    pub difficulty: PositionDifficulty,
}

/// Result for a single position in strength testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionStrengthResult {
    /// Index of the position
    pub position_index: usize,
    /// FEN string of the position
    pub position_fen: String,
    /// Expected result
    pub expected_result: GameResult,
    /// Number of wins with IID enabled
    pub iid_wins: usize,
    /// Number of wins with IID disabled
    pub non_iid_wins: usize,
    /// Win rate with IID enabled
    pub iid_win_rate: f64,
    /// Win rate with IID disabled
    pub non_iid_win_rate: f64,
    /// Improvement (IID win rate - non-IID win rate)
    pub improvement: f64,
}

impl Default for PositionStrengthResult {
    fn default() -> Self {
        Self {
            position_index: 0,
            position_fen: String::new(),
            expected_result: GameResult::Draw,
            iid_wins: 0,
            non_iid_wins: 0,
            iid_win_rate: 0.0,
            non_iid_win_rate: 0.0,
            improvement: 0.0,
        }
    }
}

/// Overall strength test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IIDStrengthTestResult {
    /// Total number of test positions
    pub total_positions: usize,
    /// Number of games per position
    pub games_per_position: usize,
    /// Time per move in milliseconds
    pub time_per_move_ms: u32,
    /// Results for each position
    pub position_results: Vec<PositionStrengthResult>,
    /// Overall improvement across all positions
    pub overall_improvement: f64,
    /// Average IID win rate
    pub average_iid_win_rate: f64,
    /// Average non-IID win rate
    pub average_non_iid_win_rate: f64,
    /// Statistical significance
    pub statistical_significance: f64,
}

impl Default for IIDStrengthTestResult {
    fn default() -> Self {
        Self {
            total_positions: 0,
            games_per_position: 0,
            time_per_move_ms: 0,
            position_results: Vec::new(),
            overall_improvement: 0.0,
            average_iid_win_rate: 0.0,
            average_non_iid_win_rate: 0.0,
            statistical_significance: 0.0,
        }
    }
}

impl IIDStrengthTestResult {
    /// Calculate overall statistics
    pub fn calculate_overall_statistics(&mut self) {
        if self.position_results.is_empty() {
            return;
        }

        let total_iid_wins: usize = self.position_results.iter().map(|r| r.iid_wins).sum();
        let total_non_iid_wins: usize = self.position_results.iter().map(|r| r.non_iid_wins).sum();
        let total_games = self.position_results.len() * self.games_per_position;

        self.average_iid_win_rate = total_iid_wins as f64 / total_games as f64;
        self.average_non_iid_win_rate = total_non_iid_wins as f64 / total_games as f64;
        self.overall_improvement = self.average_iid_win_rate - self.average_non_iid_win_rate;

        // Calculate statistical significance (simplified)
        let variance = self.position_results.iter()
            .map(|r| (r.improvement - self.overall_improvement).powi(2))
            .sum::<f64>() / self.position_results.len() as f64;
        let standard_error = (variance / self.position_results.len() as f64).sqrt();
        self.statistical_significance = if standard_error > 0.0 {
            self.overall_improvement / standard_error
        } else {
            0.0
        };
    }
}

/// Analysis of strength test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthTestAnalysis {
    /// Overall improvement observed
    pub overall_improvement: f64,
    /// Positions with significant improvement/regression
    pub significant_positions: Vec<usize>,
    /// Recommendations based on results
    pub recommendations: Vec<String>,
    /// Confidence level in the analysis
    pub confidence_level: ConfidenceLevel,
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
            verification_margin: 200,
            dynamic_reduction_formula: DynamicReductionFormula::Linear,
            enable_mate_threat_detection: false,
            mate_threat_margin: 500,
            enable_endgame_type_detection: false,
            material_endgame_threshold: 12,
            king_activity_threshold: 8,
            zugzwang_threshold: 6,
            preset: None,
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

/// Depth selection strategy for Internal Iterative Deepening
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IIDDepthStrategy {
    /// Use a fixed depth for IID search
    Fixed,
    /// Use a depth relative to the main search depth (depth - 2)
    Relative,
    /// Adapt depth based on position complexity and time remaining
    Adaptive,
}

impl Default for IIDDepthStrategy {
    fn default() -> Self {
        IIDDepthStrategy::Fixed
    }
}


/// Configuration for Internal Iterative Deepening (IID) parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IIDConfig {
    /// Enable or disable Internal Iterative Deepening
    pub enabled: bool,
    /// Minimum depth to apply IID
    pub min_depth: u8,
    /// Fixed depth for IID search (when using Fixed strategy)
    pub iid_depth_ply: u8,
    /// Maximum number of legal moves to apply IID (avoid in tactical positions)
    pub max_legal_moves: usize,
    /// Maximum time overhead threshold (0.0 to 1.0)
    pub time_overhead_threshold: f64,
    /// Depth selection strategy
    pub depth_strategy: IIDDepthStrategy,
    /// Enable time pressure detection to skip IID
    pub enable_time_pressure_detection: bool,
    /// Enable adaptive tuning based on performance metrics
    pub enable_adaptive_tuning: bool,
}

impl Default for IIDConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_depth: 4,                    // Apply IID at depth 4+
            iid_depth_ply: 2,               // 2-ply IID search
            max_legal_moves: 35,            // Skip IID in tactical positions
            time_overhead_threshold: 0.15,  // Max 15% time overhead
            depth_strategy: IIDDepthStrategy::Fixed,
            enable_time_pressure_detection: true,
            enable_adaptive_tuning: false,   // Disabled by default
        }
    }
}

impl IIDConfig {
    /// Validate the configuration parameters and return any errors
    pub fn validate(&self) -> Result<(), String> {
        if self.min_depth < 2 {
            return Err("min_depth must be at least 2".to_string());
        }
        if self.min_depth > 15 {
            return Err("min_depth should not exceed 15 for performance reasons".to_string());
        }
        if self.iid_depth_ply == 0 {
            return Err("iid_depth_ply must be greater than 0".to_string());
        }
        if self.iid_depth_ply > 5 {
            return Err("iid_depth_ply should not exceed 5 for performance reasons".to_string());
        }
        if self.max_legal_moves == 0 {
            return Err("max_legal_moves must be greater than 0".to_string());
        }
        if self.max_legal_moves > 100 {
            return Err("max_legal_moves should not exceed 100".to_string());
        }
        if self.time_overhead_threshold < 0.0 || self.time_overhead_threshold > 1.0 {
            return Err("time_overhead_threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "IIDConfig: enabled={}, min_depth={}, iid_depth_ply={}, max_moves={}, overhead_threshold={:.2}, strategy={:?}",
            self.enabled,
            self.min_depth,
            self.iid_depth_ply,
            self.max_legal_moves,
            self.time_overhead_threshold,
            self.depth_strategy
        )
    }
}

/// Performance statistics for Internal Iterative Deepening
#[derive(Debug, Clone, Default)]
pub struct IIDStats {
    /// Number of IID searches performed
    pub iid_searches_performed: u64,
    /// Number of times IID move was first to improve alpha
    pub iid_move_first_improved_alpha: u64,
    /// Number of times IID move caused a cutoff
    pub iid_move_caused_cutoff: u64,
    /// Total nodes searched in IID searches
    pub total_iid_nodes: u64,
    /// Total time spent in IID searches (milliseconds)
    pub iid_time_ms: u64,
    /// Positions skipped due to transposition table move
    pub positions_skipped_tt_move: u64,
    /// Positions skipped due to insufficient depth
    pub positions_skipped_depth: u64,
    /// Positions skipped due to too many legal moves
    pub positions_skipped_move_count: u64,
    /// Positions skipped due to time pressure
    pub positions_skipped_time_pressure: u64,
    /// IID searches that failed to find a move
    pub iid_searches_failed: u64,
    /// IID searches that found a move but it didn't improve alpha
    pub iid_moves_ineffective: u64,
}

impl IIDStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = IIDStats::default();
    }

    /// Get the IID efficiency rate as a percentage
    pub fn efficiency_rate(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        (self.iid_move_first_improved_alpha as f64 / self.iid_searches_performed as f64) * 100.0
    }

    /// Get the IID cutoff rate as a percentage
    pub fn cutoff_rate(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        (self.iid_move_caused_cutoff as f64 / self.iid_searches_performed as f64) * 100.0
    }

    /// Get the skip rate for each condition
    pub fn skip_rate_tt_move(&self) -> f64 {
        let total_skips = self.positions_skipped_tt_move + self.positions_skipped_depth + 
                         self.positions_skipped_move_count + self.positions_skipped_time_pressure;
        if total_skips == 0 {
            return 0.0;
        }
        (self.positions_skipped_tt_move as f64 / total_skips as f64) * 100.0
    }

    /// Get the average nodes per IID search
    pub fn average_nodes_per_iid(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        self.total_iid_nodes as f64 / self.iid_searches_performed as f64
    }

    /// Get the average time per IID search
    pub fn average_time_per_iid(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        self.iid_time_ms as f64 / self.iid_searches_performed as f64
    }

    /// Get the success rate of IID searches
    pub fn success_rate(&self) -> f64 {
        if self.iid_searches_performed == 0 {
            return 0.0;
        }
        let successful = self.iid_searches_performed - self.iid_searches_failed;
        (successful as f64 / self.iid_searches_performed as f64) * 100.0
    }

    /// Get a comprehensive performance report
    pub fn performance_report(&self) -> String {
        format!(
            "Internal Iterative Deepening Performance Report:\n\
            - IID searches performed: {}\n\
            - Success rate: {:.2}%\n\
            - Efficiency rate: {:.2}%\n\
            - Cutoff rate: {:.2}%\n\
            - Average nodes per IID: {:.1}\n\
            - Average time per IID: {:.1}ms\n\
            - Positions skipped (TT): {} ({:.1}%)\n\
            - Positions skipped (depth): {} ({:.1}%)\n\
            - Positions skipped (moves): {} ({:.1}%)\n\
            - Positions skipped (time): {} ({:.1}%)",
            self.iid_searches_performed,
            self.success_rate(),
            self.efficiency_rate(),
            self.cutoff_rate(),
            self.average_nodes_per_iid(),
            self.average_time_per_iid(),
            self.positions_skipped_tt_move,
            self.skip_rate_tt_move(),
            self.positions_skipped_depth,
            (self.positions_skipped_depth as f64 / (self.positions_skipped_tt_move + self.positions_skipped_depth + self.positions_skipped_move_count + self.positions_skipped_time_pressure) as f64) * 100.0,
            self.positions_skipped_move_count,
            (self.positions_skipped_move_count as f64 / (self.positions_skipped_tt_move + self.positions_skipped_depth + self.positions_skipped_move_count + self.positions_skipped_time_pressure) as f64) * 100.0,
            self.positions_skipped_time_pressure,
            (self.positions_skipped_time_pressure as f64 / (self.positions_skipped_tt_move + self.positions_skipped_depth + self.positions_skipped_move_count + self.positions_skipped_time_pressure) as f64) * 100.0
        )
    }

    /// Get a summary of key metrics
    pub fn summary(&self) -> String {
        format!(
            "IID: {} searches, {:.1}% efficient, {:.1}% cutoffs, {:.1} avg nodes, {:.1}ms avg time",
            self.iid_searches_performed,
            self.efficiency_rate(),
            self.cutoff_rate(),
            self.average_nodes_per_iid(),
            self.average_time_per_iid()
        )
    }
}

/// Performance metrics for Internal Iterative Deepening
#[derive(Debug, Clone)]
pub struct IIDPerformanceMetrics {
    /// Alpha improvements per IID search
    pub iid_efficiency: f64,
    /// Percentage of IID moves causing cutoffs
    pub cutoff_rate: f64,
    /// Time overhead vs total search time
    pub overhead_percentage: f64,
    /// Average nodes saved per IID search
    pub nodes_saved_per_iid: f64,
    /// Success rate of IID searches
    pub success_rate: f64,
    /// Average time per IID search in milliseconds
    pub average_iid_time: f64,
    /// Skip rate for various conditions
    pub tt_skip_rate: f64,
    pub depth_skip_rate: f64,
    pub move_count_skip_rate: f64,
    pub time_pressure_skip_rate: f64,
}

impl IIDPerformanceMetrics {
    /// Create performance metrics from IID statistics
    pub fn from_stats(stats: &IIDStats, total_search_time_ms: u64) -> Self {
        let total_skips = stats.positions_skipped_tt_move + stats.positions_skipped_depth + 
                         stats.positions_skipped_move_count + stats.positions_skipped_time_pressure;
        
        Self {
            iid_efficiency: stats.efficiency_rate(),
            cutoff_rate: stats.cutoff_rate(),
            overhead_percentage: if total_search_time_ms > 0 {
                (stats.iid_time_ms as f64 / total_search_time_ms as f64) * 100.0
            } else { 0.0 },
            nodes_saved_per_iid: stats.average_nodes_per_iid(),
            success_rate: stats.success_rate(),
            average_iid_time: stats.average_time_per_iid(),
            tt_skip_rate: if total_skips > 0 {
                (stats.positions_skipped_tt_move as f64 / total_skips as f64) * 100.0
            } else { 0.0 },
            depth_skip_rate: if total_skips > 0 {
                (stats.positions_skipped_depth as f64 / total_skips as f64) * 100.0
            } else { 0.0 },
            move_count_skip_rate: if total_skips > 0 {
                (stats.positions_skipped_move_count as f64 / total_skips as f64) * 100.0
            } else { 0.0 },
            time_pressure_skip_rate: if total_skips > 0 {
                (stats.positions_skipped_time_pressure as f64 / total_skips as f64) * 100.0
            } else { 0.0 },
        }
    }

    /// Get a summary of the performance metrics
    pub fn summary(&self) -> String {
        format!(
            "IID Performance: {:.1}% efficient, {:.1}% cutoffs, {:.1}% overhead, {:.1} avg nodes saved",
            self.iid_efficiency,
            self.cutoff_rate,
            self.overhead_percentage,
            self.nodes_saved_per_iid
        )
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
    /// Use static evaluation for aspiration window initialization (Task 4.1)
    pub use_static_eval_for_init: bool,
    /// Enable position type tracking for window optimization (Task 7.1)
    pub enable_position_type_tracking: bool,
    /// Disable statistics tracking in production builds (Task 7.2)
    /// When true, statistics tracking is disabled regardless of enable_statistics
    pub disable_statistics_in_production: bool,
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
            use_static_eval_for_init: true, // Use static eval for first window (Task 4.1)
            enable_position_type_tracking: true, // Task 7.1: Enable position type tracking by default
            disable_statistics_in_production: false, // Task 7.2: Allow statistics in production by default
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

/// Window size statistics by position type (Task 7.1)
#[derive(Debug, Clone, Default)]
pub struct WindowSizeByPositionType {
    /// Average window size in opening
    pub opening_avg_window_size: f64,
    /// Average window size in middlegame
    pub middlegame_avg_window_size: f64,
    /// Average window size in endgame
    pub endgame_avg_window_size: f64,
    /// Number of searches in opening
    pub opening_searches: u64,
    /// Number of searches in middlegame
    pub middlegame_searches: u64,
    /// Number of searches in endgame
    pub endgame_searches: u64,
}

/// Success rate statistics by position type (Task 7.1)
#[derive(Debug, Clone, Default)]
pub struct SuccessRateByPositionType {
    /// Success rate in opening
    pub opening_success_rate: f64,
    /// Success rate in middlegame
    pub middlegame_success_rate: f64,
    /// Success rate in endgame
    pub endgame_success_rate: f64,
    /// Successful searches in opening
    pub opening_successful: u64,
    /// Successful searches in middlegame
    pub middlegame_successful: u64,
    /// Successful searches in endgame
    pub endgame_successful: u64,
    /// Total searches in opening
    pub opening_total: u64,
    /// Total searches in middlegame
    pub middlegame_total: u64,
    /// Total searches in endgame
    pub endgame_total: u64,
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
    /// Window size statistics by position type (Task 7.1)
    pub window_size_by_position_type: WindowSizeByPositionType,
    /// Success rate by position type (Task 7.1)
    pub success_rate_by_position_type: SuccessRateByPositionType,
}

/// Time allocation strategy for iterative deepening (Task 4.8)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAllocationStrategy {
    /// Equal time allocation per depth
    Equal,
    /// Exponential time allocation (later depths get more time)
    Exponential,
    /// Adaptive allocation based on previous depth completion times
    Adaptive,
}

impl Default for TimeAllocationStrategy {
    fn default() -> Self {
        TimeAllocationStrategy::Adaptive
    }
}


/// Time budget allocation tracking (Task 4.10)
#[derive(Debug, Clone, Default)]
pub struct TimeBudgetStats {
    /// Depth completion times in milliseconds
    pub depth_completion_times_ms: Vec<u32>,
    /// Estimated time per depth based on history
    pub estimated_time_per_depth_ms: Vec<u32>,
    /// Actual time used per depth
    pub actual_time_per_depth_ms: Vec<u32>,
    /// Time budget allocated per depth
    pub budget_per_depth_ms: Vec<u32>,
    /// Number of depths completed
    pub depths_completed: u8,
    /// Number of depths that exceeded budget
    pub depths_exceeded_budget: u8,
    /// Average time estimation accuracy (0.0 to 1.0)
    pub estimation_accuracy: f64,
}

impl AspirationWindowStats {
    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = AspirationWindowStats::default();
    }
    
    /// Update window size statistics by position type (Task 7.1)
    pub fn update_window_size_by_position_type(&mut self, phase: GamePhase, window_size: i32) {
        let stats = &mut self.window_size_by_position_type;
        match phase {
            GamePhase::Opening => {
                let old_avg = stats.opening_avg_window_size;
                let count = stats.opening_searches;
                stats.opening_searches += 1;
                stats.opening_avg_window_size = 
                    (old_avg * count as f64 + window_size as f64) / (count + 1) as f64;
            },
            GamePhase::Middlegame => {
                let old_avg = stats.middlegame_avg_window_size;
                let count = stats.middlegame_searches;
                stats.middlegame_searches += 1;
                stats.middlegame_avg_window_size = 
                    (old_avg * count as f64 + window_size as f64) / (count + 1) as f64;
            },
            GamePhase::Endgame => {
                let old_avg = stats.endgame_avg_window_size;
                let count = stats.endgame_searches;
                stats.endgame_searches += 1;
                stats.endgame_avg_window_size = 
                    (old_avg * count as f64 + window_size as f64) / (count + 1) as f64;
            },
        }
    }
    
    /// Update success rate statistics by position type (Task 7.1)
    pub fn update_success_rate_by_position_type(&mut self, phase: GamePhase, successful: bool) {
        let stats = &mut self.success_rate_by_position_type;
        match phase {
            GamePhase::Opening => {
                stats.opening_total += 1;
                if successful {
                    stats.opening_successful += 1;
                }
                stats.opening_success_rate = 
                    stats.opening_successful as f64 / stats.opening_total as f64;
            },
            GamePhase::Middlegame => {
                stats.middlegame_total += 1;
                if successful {
                    stats.middlegame_successful += 1;
                }
                stats.middlegame_success_rate = 
                    stats.middlegame_successful as f64 / stats.middlegame_total as f64;
            },
            GamePhase::Endgame => {
                stats.endgame_total += 1;
                if successful {
                    stats.endgame_successful += 1;
                }
                stats.endgame_success_rate = 
                    stats.endgame_successful as f64 / stats.endgame_total as f64;
            },
        }
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
    /// Internal Iterative Deepening configuration
    pub iid: IIDConfig,
    /// Transposition table size in MB
    pub tt_size_mb: usize,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Maximum search depth
    pub max_depth: u8,
    /// Time management settings
    pub time_management: TimeManagementConfig,
    /// Number of threads for parallel search (USI_Threads)
    pub thread_count: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            quiescence: QuiescenceConfig::default(),
            null_move: NullMoveConfig::default(),
            lmr: LMRConfig::default(),
            aspiration_windows: AspirationWindowConfig::default(),
            iid: IIDConfig::default(),
            tt_size_mb: 64,
            debug_logging: false,
            max_depth: 20,
            time_management: TimeManagementConfig::default(),
            thread_count: num_cpus::get(),
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
        iid: IIDConfig,
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
            iid,
            tt_size_mb,
            debug_logging,
            max_depth,
            time_management,
            thread_count: num_cpus::get(),
        }
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate individual components
        self.quiescence.validate()?;
        self.null_move.validate()?;
        self.lmr.validate()?;
        self.aspiration_windows.validate()?;
        self.iid.validate()?;
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
                    verification_margin: 200,
                    dynamic_reduction_formula: DynamicReductionFormula::Linear,
                    enable_mate_threat_detection: false,
                    mate_threat_margin: 500,
                    enable_endgame_type_detection: false,
                    material_endgame_threshold: 12,
                    king_activity_threshold: 8,
                    zugzwang_threshold: 6,
                    preset: None,
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
                    use_static_eval_for_init: true,
                    enable_position_type_tracking: true,
                    disable_statistics_in_production: false,
                },
                iid: IIDConfig {
                    enabled: true,
                    min_depth: 3,
                    iid_depth_ply: 2,
                    max_legal_moves: 40,
                    time_overhead_threshold: 0.20,
                    depth_strategy: IIDDepthStrategy::Fixed,
                    enable_time_pressure_detection: true,
                    enable_adaptive_tuning: false,
                },
                tt_size_mb: 128,
                debug_logging: false,
                max_depth: 25,
                time_management: TimeManagementConfig::default(),
                thread_count: num_cpus::get(),
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
                    verification_margin: 200,
                    dynamic_reduction_formula: DynamicReductionFormula::Static,
                    enable_mate_threat_detection: false,
                    mate_threat_margin: 500,
                    enable_endgame_type_detection: false,
                    material_endgame_threshold: 12,
                    king_activity_threshold: 8,
                    zugzwang_threshold: 6,
                    preset: None,
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
                    use_static_eval_for_init: true,
                    enable_position_type_tracking: true,
                    disable_statistics_in_production: false,
                },
                iid: IIDConfig {
                    enabled: true,
                    min_depth: 5,
                    iid_depth_ply: 3,
                    max_legal_moves: 30,
                    time_overhead_threshold: 0.10,
                    depth_strategy: IIDDepthStrategy::Fixed,
                    enable_time_pressure_detection: true,
                    enable_adaptive_tuning: false,
                },
                tt_size_mb: 256,
                debug_logging: false,
                max_depth: 30,
                time_management: TimeManagementConfig::default(),
                thread_count: num_cpus::get(),
            },
            EnginePreset::Balanced => Self {
                quiescence: QuiescenceConfig::default(),
                null_move: NullMoveConfig::default(),
                lmr: LMRConfig::default(),
                aspiration_windows: AspirationWindowConfig::default(),
                iid: IIDConfig::default(),
                tt_size_mb: 128,
                debug_logging: false,
                max_depth: 25,
                time_management: TimeManagementConfig::default(),
                thread_count: num_cpus::get(),
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
            "Engine Config: TT={}MB, MaxDepth={}, Quiescence={}, NMP={}, LMR={}, Aspiration={}, IID={}",
            self.tt_size_mb,
            self.max_depth,
            if self.quiescence.enable_tt { "ON" } else { "OFF" },
            if self.null_move.enabled { "ON" } else { "OFF" },
            if self.lmr.enabled { "ON" } else { "OFF" },
            if self.aspiration_windows.enabled { "ON" } else { "OFF" },
            if self.iid.enabled { "ON" } else { "OFF" }
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
    /// Time allocation strategy for iterative deepening (Task 4.8)
    pub allocation_strategy: TimeAllocationStrategy,
    /// Safety margin as percentage of total time (0.0 to 1.0) (Task 4.7)
    pub safety_margin: f64,
    /// Minimum time per depth in milliseconds (Task 4.7)
    pub min_time_per_depth_ms: u32,
    /// Maximum time per depth in milliseconds (0 = no limit) (Task 4.7)
    pub max_time_per_depth_ms: u32,
    /// Enable check position optimization (Task 4.3)
    pub enable_check_optimization: bool,
    /// Check position max depth threshold (Task 4.4)
    pub check_max_depth: u8,
    /// Check position time limit in milliseconds (Task 4.4)
    pub check_time_limit_ms: u32,
    /// Enable time budget allocation (Task 4.5)
    pub enable_time_budget: bool,
    /// Time check frequency: check every N nodes instead of every node (Task 8.4)
    /// Set to 1 to check every node, higher values reduce overhead
    pub time_check_frequency: u32,
    /// Absolute safety margin in milliseconds (Task 8.2, 8.3)
    /// Used in addition to percentage-based safety_margin
    /// This represents the minimum overhead buffer needed for time checks and search completion
    pub absolute_safety_margin_ms: u32,
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
            allocation_strategy: TimeAllocationStrategy::Adaptive,
            safety_margin: 0.1, // 10% safety margin
            min_time_per_depth_ms: 50,
            max_time_per_depth_ms: 0, // No limit by default
            enable_check_optimization: true,
            check_max_depth: 5,
            check_time_limit_ms: 5000,
            enable_time_budget: true,
            time_check_frequency: 1024, // Task 8.4: Check every 1024 nodes (reduce overhead)
            absolute_safety_margin_ms: 100, // Task 8.2, 8.3: 100ms absolute safety margin
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
        
        // Validate iterative deepening time allocation settings (Task 4.7)
        if self.safety_margin < 0.0 || self.safety_margin > 0.5 {
            return Err("safety_margin must be between 0.0 and 0.5".to_string());
        }
        if self.min_time_per_depth_ms == 0 {
            return Err("min_time_per_depth_ms must be greater than 0".to_string());
        }
        if self.max_time_per_depth_ms > 0 && self.max_time_per_depth_ms < self.min_time_per_depth_ms {
            return Err("max_time_per_depth_ms must be >= min_time_per_depth_ms".to_string());
        }
        if self.check_max_depth == 0 || self.check_max_depth > 10 {
            return Err("check_max_depth must be between 1 and 10".to_string());
        }
        
        // Task 8.4: Validate time check frequency
        if self.time_check_frequency == 0 {
            return Err("time_check_frequency must be greater than 0".to_string());
        }
        if self.time_check_frequency > 100000 {
            return Err("time_check_frequency should not exceed 100000 for performance reasons".to_string());
        }
        
        // Task 8.3: Validate absolute safety margin
        if self.absolute_safety_margin_ms > 10000 {
            return Err("absolute_safety_margin_ms should not exceed 10000ms (10 seconds)".to_string());
        }

        Ok(())
    }
    
    /// Get a summary of the time management configuration including iterative deepening settings
    pub fn summary_full(&self) -> String {
        format!(
            "TimeManagement: enabled={}, buffer={:.1}%, strategy={:?}, safety_margin={:.1}%, min_time_depth={}ms, max_time_depth={}ms, check_opt={}, time_budget={}",
            self.enabled,
            self.buffer_percentage * 100.0,
            self.allocation_strategy,
            self.safety_margin * 100.0,
            self.min_time_per_depth_ms,
            self.max_time_per_depth_ms,
            self.enable_check_optimization,
            self.enable_time_budget
        )
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
            iid: IIDConfig::default(),
            tt_size_mb,
            debug_logging: false,
            max_depth: 20,
            time_management: TimeManagementConfig::default(),
            thread_count: num_cpus::get(),
        }
    }

    /// Create a configuration from individual components
    pub fn create_from_components(
        quiescence: QuiescenceConfig,
        null_move: NullMoveConfig,
        lmr: LMRConfig,
        aspiration_windows: AspirationWindowConfig,
        iid: IIDConfig,
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
            iid,
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

// ============================================================================
// Magic Bitboard Types
// ============================================================================

/// Magic bitboard specific errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum MagicError {
    #[error("Failed to generate magic number for square {square} piece {piece_type:?}")]
    GenerationFailed { square: u8, piece_type: PieceType },
    
    #[error("Magic number validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Insufficient memory for magic table: required {required}, available {available}")]
    InsufficientMemory { required: usize, available: usize },
    
    #[error("Magic table initialization failed: {reason}")]
    InitializationFailed { reason: String },
    
    #[error("Invalid square index: {square}")]
    InvalidSquare { square: u8 },
    
    #[error("Invalid piece type for magic bitboards: {piece_type:?}")]
    InvalidPieceType { piece_type: PieceType },
    
    #[error("IO error: {0}")]
    IoError(String),
}

/// Represents a magic bitboard entry for a single square
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagicBitboard {
    /// The magic number used for hashing
    pub magic_number: u64,
    /// Bitmask of relevant occupied squares
    pub mask: Bitboard,
    /// Number of bits to shift the hash result
    pub shift: u8,
    /// Base address for attack table
    pub attack_base: usize,
    /// Number of attack patterns for this square
    pub table_size: usize,
}

impl Default for MagicBitboard {
    fn default() -> Self {
        Self {
            magic_number: 0,
            mask: EMPTY_BITBOARD,
            shift: 0,
            attack_base: 0,
            table_size: 0,
        }
    }
}

/// Complete magic bitboard table for all squares
#[derive(Clone, Debug)]
pub struct MagicTable {
    /// Magic bitboards for rook attacks (81 squares)
    pub rook_magics: [MagicBitboard; 81],
    /// Magic bitboards for bishop attacks (81 squares)
    pub bishop_magics: [MagicBitboard; 81],
    /// Precomputed attack patterns storage
    pub attack_storage: Vec<Bitboard>,
    /// Memory pool for attack tables
    pub memory_pool: MemoryPool,
}

impl Default for MagicTable {
    fn default() -> Self {
        Self {
            rook_magics: [MagicBitboard::default(); 81],
            bishop_magics: [MagicBitboard::default(); 81],
            attack_storage: Vec::new(),
            memory_pool: MemoryPool::default(),
        }
    }
}

/// Memory pool for efficient allocation of attack tables
#[derive(Clone, Debug)]
pub struct MemoryPool {
    /// Pre-allocated memory blocks
    pub blocks: Vec<Vec<Bitboard>>,
    /// Current allocation index
    pub current_block: usize,
    /// Current position in current block
    pub current_offset: usize,
    /// Block size for allocation
    pub block_size: usize,
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            current_block: 0,
            current_offset: 0,
            block_size: 4096, // Default block size
        }
    }
}

/// Magic number generation result
#[derive(Debug, Clone, Copy)]
pub struct MagicGenerationResult {
    pub magic_number: u64,
    pub mask: Bitboard,
    pub shift: u8,
    pub table_size: usize,
    pub generation_time: std::time::Duration,
}

/// Attack pattern generation configuration
#[derive(Debug, Clone)]
pub struct AttackConfig {
    pub piece_type: PieceType,
    pub square: u8,
    pub include_promoted: bool,
    pub max_distance: Option<u8>,
}

/// Performance metrics for magic bitboard operations
#[derive(Debug, Default, Clone)]
#[derive(PartialEq)]
pub struct PerformanceMetrics {
    pub lookup_count: u64,
    pub total_lookup_time: std::time::Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub memory_usage: usize,
    pub fallback_lookups: u64,
}

// ============================================================================
// Advanced Alpha-Beta Pruning Structures
// ============================================================================

// Import BitboardBoard for adaptive parameter methods
use crate::bitboards::BitboardBoard;

/// Game phase for position-dependent pruning decisions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
}

impl GamePhase {
    /// Determine game phase based on material count
    pub fn from_material_count(material: u32) -> Self {
        match material {
            0..=20 => GamePhase::Endgame,
            21..=35 => GamePhase::Middlegame,
            _ => GamePhase::Opening,
        }
    }
}

/// Search state for advanced alpha-beta pruning
#[derive(Debug, Clone)]
pub struct SearchState {
    pub depth: u8,
    pub move_number: u8,
    pub alpha: i32,
    pub beta: i32,
    pub is_in_check: bool,
    pub static_eval: i32,
    pub best_move: Option<Move>,
    pub position_hash: u64,
    pub game_phase: GamePhase,
}

impl SearchState {
    pub fn new(depth: u8, alpha: i32, beta: i32) -> Self {
        Self {
            depth,
            move_number: 0,
            alpha,
            beta,
            is_in_check: false,
            static_eval: 0,
            best_move: None,
            position_hash: 0,
            game_phase: GamePhase::Middlegame,
        }
    }
    
    /// Update search state with current position information
    /// Note: This method should be called from SearchEngine with the appropriate values
    pub fn update_fields(&mut self, is_in_check: bool, static_eval: i32, position_hash: u64, game_phase: GamePhase) {
        self.is_in_check = is_in_check;
        self.static_eval = static_eval;
        self.position_hash = position_hash;
        self.game_phase = game_phase;
    }
}

/// Pruning decision result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PruningDecision {
    Search,           // Search normally
    ReducedSearch,    // Search with reduced depth
    Skip,             // Skip this move
    Razor,            // Use razoring
}

impl PruningDecision {
    pub fn is_pruned(&self) -> bool {
        matches!(self, PruningDecision::Skip)
    }
    
    pub fn needs_reduction(&self) -> bool {
        matches!(self, PruningDecision::ReducedSearch)
    }
}

/// Parameters for advanced alpha-beta pruning techniques
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct PruningParameters {
    // Futility pruning parameters
    pub futility_margin: [i32; 8],
    pub futility_depth_limit: u8,
    pub extended_futility_depth: u8,
    
    // Late move reduction parameters
    pub lmr_base_reduction: u8,
    pub lmr_move_threshold: u8,
    pub lmr_depth_threshold: u8,
    pub lmr_max_reduction: u8,
    
    // Delta pruning parameters
    pub delta_margin: i32,
    pub delta_depth_limit: u8,
    
    // Razoring parameters
    pub razoring_depth_limit: u8,
    pub razoring_margin: i32,
    pub razoring_margin_endgame: i32,
    
    // Multi-cut pruning parameters
    pub multi_cut_threshold: u8,
    pub multi_cut_depth_limit: u8,
    
    // Adaptive parameters
    pub adaptive_enabled: bool,
    pub position_dependent_margins: bool,
}

impl Default for PruningParameters {
    fn default() -> Self {
        Self {
            futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
            futility_depth_limit: 3,
            extended_futility_depth: 5,
            lmr_base_reduction: 1,
            lmr_move_threshold: 3,
            lmr_depth_threshold: 2,
            lmr_max_reduction: 3,
            delta_margin: 200,
            delta_depth_limit: 4,
            razoring_depth_limit: 3,
            razoring_margin: 300,
            razoring_margin_endgame: 200,
            multi_cut_threshold: 3,
            multi_cut_depth_limit: 4,
            adaptive_enabled: false,
            position_dependent_margins: false,
        }
    }
}

/// Statistics for pruning effectiveness monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningStatistics {
    pub total_moves: u64,
    pub pruned_moves: u64,
    pub futility_pruned: u64,
    pub delta_pruned: u64,
    pub razored: u64,
    pub lmr_applied: u64,
    pub re_searches: u64,
    pub multi_cuts: u64,
}

impl PruningStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_decision(&mut self, decision: PruningDecision) {
        self.total_moves += 1;
        
        match decision {
            PruningDecision::Skip => self.pruned_moves += 1,
            PruningDecision::Razor => self.razored += 1,
            _ => {}
        }
    }
    
    pub fn get_pruning_rate(&self) -> f64 {
        if self.total_moves == 0 {
            0.0
        } else {
            self.pruned_moves as f64 / self.total_moves as f64
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Pruning effectiveness metrics for analysis
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningEffectiveness {
    pub futility_rate: f64,
    pub delta_rate: f64,
    pub razoring_rate: f64,
    pub multi_cut_rate: f64,
    pub lmr_rate: f64,
    pub overall_effectiveness: f64,
}

/// Pruning frequency statistics for detailed analysis
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningFrequencyStats {
    pub total_moves: u64,
    pub pruned_moves: u64,
    pub futility_pruned: u64,
    pub delta_pruned: u64,
    pub razored: u64,
    pub lmr_applied: u64,
    pub multi_cuts: u64,
    pub re_searches: u64,
    pub pruning_rate: f64,
    pub cache_hit_rate: f64,
}

/// Search performance metrics for monitoring
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchPerformanceMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: usize,
    pub position_cache_size: usize,
    pub check_cache_size: usize,
    pub total_cache_operations: u64,
    pub cache_hit_rate: f64,
}

/// Comprehensive search metrics for performance monitoring (Task 5.7-5.9)
/// Tracks key performance indicators: cutoff rate, TT hit rate, aspiration window success
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CoreSearchMetrics {
    /// Total number of nodes searched
    pub total_nodes: u64,
    /// Total number of alpha-beta cutoffs
    pub total_cutoffs: u64,
    /// Total number of transposition table probes
    pub total_tt_probes: u64,
    /// Total number of transposition table hits
    pub total_tt_hits: u64,
    /// Total number of aspiration window searches
    pub total_aspiration_searches: u64,
    /// Number of successful aspiration window searches (no re-search needed)
    pub successful_aspiration_searches: u64,
    /// Number of beta cutoffs (move ordering effectiveness indicator)
    pub beta_cutoffs: u64,
    /// Number of exact score entries in TT
    pub tt_exact_hits: u64,
    /// Number of lower bound entries in TT
    pub tt_lower_bound_hits: u64,
    /// Number of upper bound entries in TT
    pub tt_upper_bound_hits: u64,
}

impl CoreSearchMetrics {
    /// Create a new empty metrics structure
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Calculate cutoff rate as a percentage (Task 5.7)
    pub fn cutoff_rate(&self) -> f64 {
        if self.total_nodes == 0 {
            return 0.0;
        }
        (self.total_cutoffs as f64 / self.total_nodes as f64) * 100.0
    }
    
    /// Calculate transposition table hit rate as a percentage (Task 5.7)
    pub fn tt_hit_rate(&self) -> f64 {
        if self.total_tt_probes == 0 {
            return 0.0;
        }
        (self.total_tt_hits as f64 / self.total_tt_probes as f64) * 100.0
    }
    
    /// Calculate aspiration window success rate as a percentage (Task 5.7)
    pub fn aspiration_success_rate(&self) -> f64 {
        if self.total_aspiration_searches == 0 {
            return 0.0;
        }
        (self.successful_aspiration_searches as f64 / self.total_aspiration_searches as f64) * 100.0
    }
    
    /// Get breakdown of TT hit types
    pub fn tt_hit_breakdown(&self) -> (f64, f64, f64) {
        if self.total_tt_hits == 0 {
            return (0.0, 0.0, 0.0);
        }
        let exact_pct = (self.tt_exact_hits as f64 / self.total_tt_hits as f64) * 100.0;
        let lower_pct = (self.tt_lower_bound_hits as f64 / self.total_tt_hits as f64) * 100.0;
        let upper_pct = (self.tt_upper_bound_hits as f64 / self.total_tt_hits as f64) * 100.0;
        (exact_pct, lower_pct, upper_pct)
    }
    
    /// Generate a comprehensive metrics report (Task 5.9)
    pub fn generate_report(&self) -> String {
        let cutoff_rate = self.cutoff_rate();
        let tt_hit_rate = self.tt_hit_rate();
        let aspiration_success = self.aspiration_success_rate();
        let (exact_pct, lower_pct, upper_pct) = self.tt_hit_breakdown();
        
        format!(
            "Core Search Metrics Report:\n\
            =========================\n\
            Total Nodes Searched: {}\n\
            Total Cutoffs: {} ({:.2}% cutoff rate)\n\
            Beta Cutoffs: {}\n\
            \n\
            Transposition Table:\n\
            - Total Probes: {}\n\
            - Total Hits: {} ({:.2}% hit rate)\n\
            - Exact Entries: {} ({:.2}%)\n\
            - Lower Bound: {} ({:.2}%)\n\
            - Upper Bound: {} ({:.2}%)\n\
            \n\
            Aspiration Windows:\n\
            - Total Searches: {}\n\
            - Successful: {} ({:.2}% success rate)\n\
            - Re-searches: {}\n\
            ",
            self.total_nodes,
            self.total_cutoffs, cutoff_rate,
            self.beta_cutoffs,
            self.total_tt_probes,
            self.total_tt_hits, tt_hit_rate,
            self.tt_exact_hits, exact_pct,
            self.tt_lower_bound_hits, lower_pct,
            self.tt_upper_bound_hits, upper_pct,
            self.total_aspiration_searches,
            self.successful_aspiration_searches, aspiration_success,
            self.total_aspiration_searches - self.successful_aspiration_searches
        )
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceReport {
    pub pruning_effectiveness: PruningEffectiveness,
    pub frequency_stats: PruningFrequencyStats,
    pub search_metrics: SearchPerformanceMetrics,
    pub timestamp: std::time::SystemTime,
    pub report_id: String,
}

/// Performance comparison with baseline
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceComparison {
    pub current_report: PerformanceReport,
    pub baseline_report: PerformanceReport,
    pub pruning_improvement: PruningImprovement,
    pub cache_improvement: CacheImprovement,
    pub overall_improvement: f64,
}

/// Pruning improvement metrics
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PruningImprovement {
    pub futility_improvement: f64,
    pub delta_improvement: f64,
    pub razoring_improvement: f64,
    pub multi_cut_improvement: f64,
    pub overall_effectiveness_improvement: f64,
}

/// Cache improvement metrics
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CacheImprovement {
    pub hit_rate_improvement: f64,
    pub size_efficiency: f64,
    pub operation_efficiency: f64,
}

/// Cache statistics for detailed analysis
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub pruning_cache_size: usize,
    pub position_cache_size: usize,
    pub check_cache_size: usize,
}

/// Performance data export for analysis
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceDataExport {
    pub report: PerformanceReport,
    pub raw_statistics: PruningStatistics,
    pub cache_stats: CacheStats,
}

/// Manager for coordinating advanced alpha-beta pruning techniques
pub struct PruningManager {
    pub parameters: PruningParameters,
    pub statistics: PruningStatistics,
    pub adaptive_params: Option<AdaptiveParameters>,
    // Performance optimization caches
    check_cache: std::collections::HashMap<u64, bool>,
    position_cache: std::collections::HashMap<u64, PositionAnalysis>,
    pruning_cache: std::collections::HashMap<u64, PruningDecision>,
    // Performance counters
    cache_hits: u64,
    cache_misses: u64,
}

#[allow(dead_code)]
impl PruningManager {
    pub fn new(parameters: PruningParameters) -> Self {
        Self {
            parameters,
            statistics: PruningStatistics::new(),
            adaptive_params: None,
            check_cache: std::collections::HashMap::new(),
            position_cache: std::collections::HashMap::new(),
            pruning_cache: std::collections::HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Determine if a move should be pruned and how (optimized version)
    pub fn should_prune(&mut self, state: &SearchState, mv: &Move) -> PruningDecision {
        // Early exit for obvious cases
        if self.should_skip_pruning(state, mv) {
            return PruningDecision::Search;
        }
        
        // Check cache first for performance
        let cache_key = self.compute_cache_key(state, mv);
        if let Some(cached_decision) = self.pruning_cache.get(&cache_key) {
            self.cache_hits += 1;
            return *cached_decision;
        }
        self.cache_misses += 1;
        
        let mut decision = PruningDecision::Search;
        
        // Apply pruning techniques in order of safety
        decision = self.check_advanced_futility_pruning(state, mv, decision);
        decision = self.check_advanced_delta_pruning(state, mv, decision);
        decision = self.check_advanced_razoring(state, decision);
        
        // Cache the result (with size limit to prevent memory growth)
        if self.pruning_cache.len() < 10000 {
            self.pruning_cache.insert(cache_key, decision);
        }
        
        self.statistics.record_decision(decision);
        decision
    }
    
    /// Fast check to skip pruning for obvious cases
    fn should_skip_pruning(&self, state: &SearchState, mv: &Move) -> bool {
        // Skip if depth is too shallow
        if state.depth < 2 {
            return true;
        }
        
        // Skip if in check (pruning is dangerous)
        if state.is_in_check {
            return true;
        }
        
        // Skip if move is tactical (capture, promotion, check)
        if mv.is_capture || mv.is_promotion || mv.gives_check {
            return true;
        }
        
        false
    }
    
    /// Compute cache key for pruning decisions
    fn compute_cache_key(&self, state: &SearchState, mv: &Move) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        
        state.position_hash.hash(&mut hasher);
        state.depth.hash(&mut hasher);
        state.move_number.hash(&mut hasher);
        state.alpha.hash(&mut hasher);
        state.beta.hash(&mut hasher);
        state.static_eval.hash(&mut hasher);
        // Hash game phase as u8
        (state.game_phase as u8).hash(&mut hasher);
        
        // Hash move properties
        if let Some(from) = mv.from {
            from.row.hash(&mut hasher);
            from.col.hash(&mut hasher);
        }
        mv.to.row.hash(&mut hasher);
        mv.to.col.hash(&mut hasher);
        mv.piece_type.hash(&mut hasher);
        mv.is_capture.hash(&mut hasher);
        mv.is_promotion.hash(&mut hasher);
        mv.gives_check.hash(&mut hasher);
        
        hasher.finish()
    }
    
    /// Calculate late move reduction for a move
    pub fn calculate_lmr_reduction(&self, state: &SearchState, mv: &Move) -> u8 {
        if !self.should_apply_lmr(state, mv) {
            return 0;
        }
        
        let reduction = self.parameters.lmr_base_reduction +
                      (state.move_number / 8) as u8 +
                      (state.depth / 4) as u8;
        
        reduction.min(self.parameters.lmr_max_reduction).min(state.depth - 1)
    }
    
    /// Check if futility pruning should be applied
    fn check_futility_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.futility_depth_limit {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Enhanced futility pruning with move-specific analysis
        let margin = self.get_futility_margin(state);
        let move_potential = self.calculate_move_potential(mv, state);
        
        // Apply futility pruning if static eval + margin + move potential < alpha
        if state.static_eval + margin + move_potential < state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Check if delta pruning should be applied
    fn check_delta_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.delta_depth_limit {
            return current;
        }
        
        if !self.is_capture_move(mv) {
            return current;
        }
        
        // Enhanced delta pruning with advanced analysis
        let material_gain = self.calculate_material_gain(mv);
        let margin = self.get_delta_margin(state);
        let capture_bonus = self.calculate_capture_bonus(mv, state);
        
        // Apply delta pruning if static eval + material gain + margin + bonus < alpha
        if state.static_eval + material_gain + margin + capture_bonus < state.alpha {
            self.statistics.delta_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Check if razoring should be applied
    fn check_razoring(&mut self, state: &SearchState, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.razoring_depth_limit {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Enhanced razoring with advanced analysis
        let margin = self.get_razoring_margin(state);
        let position_bonus = self.calculate_razoring_bonus(state);
        
        // Apply razoring if static eval + margin + bonus < alpha
        if state.static_eval + margin + position_bonus < state.alpha {
            self.statistics.razored += 1;
            return PruningDecision::Razor;
        }
        
        current
    }
    
    /// Check if late move reduction should be applied
    fn should_apply_lmr(&self, state: &SearchState, mv: &Move) -> bool {
        state.move_number > self.parameters.lmr_move_threshold &&
        state.depth > self.parameters.lmr_depth_threshold &&
        !self.is_capture_move(mv) &&
        !self.is_promotion_move(mv) &&
        !state.is_in_check
    }
    
    /// Get futility margin based on position characteristics
    fn get_futility_margin(&self, state: &SearchState) -> i32 {
        let base_margin = self.parameters.futility_margin[state.depth as usize];
        
        if self.parameters.position_dependent_margins {
            match state.game_phase {
                GamePhase::Endgame => base_margin / 2,
                GamePhase::Opening => base_margin.saturating_mul(3) / 2,
                GamePhase::Middlegame => base_margin,
            }
        } else {
            base_margin
        }
    }
    
    /// Calculate the potential value of a move for futility pruning
    fn calculate_move_potential(&self, mv: &Move, state: &SearchState) -> i32 {
        let mut potential = 0;
        
        // Base move value
        if mv.is_capture {
            potential += self.calculate_material_gain(mv);
        }
        
        if mv.is_promotion {
            potential += 100; // Promotion bonus
        }
        
        // Position-dependent adjustments
        match state.game_phase {
            GamePhase::Opening => {
                // Opening moves have higher potential for positional gains
                potential += 50;
            },
            GamePhase::Endgame => {
                // Endgame moves focus on material and king safety
                potential += 25;
            },
            GamePhase::Middlegame => {
                // Middlegame moves have moderate potential
                potential += 35;
            },
        }
        
        // Depth-dependent potential (deeper moves have less potential)
        let depth_factor = (10 - state.depth as i32).max(1);
        potential = potential.saturating_mul(depth_factor as i32) / 10;
        
        potential.max(0)
    }
    
    /// Check if extended futility pruning should be applied (for deeper positions)
    fn check_extended_futility_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        // Only apply extended futility at deeper depths
        if state.depth <= self.parameters.futility_depth_limit || 
           state.depth > self.parameters.extended_futility_depth {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Extended futility pruning with larger margins
        let extended_margin = self.get_futility_margin(state).saturating_mul(2);
        let move_potential = self.calculate_move_potential(mv, state);
        
        // More conservative pruning at deeper depths
        if state.static_eval + extended_margin + move_potential < state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Advanced futility pruning with multiple conditions
    fn check_advanced_futility_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.extended_futility_depth {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Multiple futility conditions
        let margin = self.get_futility_margin(state);
        let move_potential = self.calculate_move_potential(mv, state);
        
        // Condition 1: Standard futility pruning
        if state.static_eval + margin + move_potential < state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        // Condition 2: Aggressive futility for very bad positions
        if state.static_eval < state.alpha.saturating_sub(500) && 
           state.static_eval + margin / 2 + move_potential < state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        // Condition 3: Late move futility (for moves beyond a certain threshold)
        if state.move_number > 4 && 
           state.static_eval + margin + move_potential / 2 < state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Get delta margin based on position characteristics
    fn get_delta_margin(&self, state: &SearchState) -> i32 {
        let base_margin = self.parameters.delta_margin;
        
        if self.parameters.position_dependent_margins {
            match state.game_phase {
                GamePhase::Endgame => base_margin / 2,  // More aggressive in endgame
                GamePhase::Opening => base_margin * 3 / 2,  // More conservative in opening
                GamePhase::Middlegame => base_margin,
            }
        } else {
            base_margin
        }
    }
    
    /// Calculate capture bonus for delta pruning
    fn calculate_capture_bonus(&self, mv: &Move, state: &SearchState) -> i32 {
        let mut bonus = 0;
        
        // Bonus for capturing higher-value pieces
        if let Some(captured_piece) = mv.captured_piece {
            match captured_piece.piece_type {
                PieceType::King => bonus += 1000,  // Should never be pruned
                PieceType::Rook | PieceType::Bishop => bonus += 100,
                PieceType::Gold | PieceType::Silver => bonus += 50,
                PieceType::Knight | PieceType::Lance => bonus += 25,
                PieceType::Pawn => bonus += 10,
                _ => bonus += 5,
            }
        }
        
        // Bonus for capturing in endgame (more tactical)
        if state.game_phase == GamePhase::Endgame {
            bonus += 25;
        }
        
        // Bonus for capturing at deeper depths (more tactical)
        if state.depth > 2 {
            bonus += state.depth as i32 * 10;
        }
        
        // Bonus for capturing when ahead (more tactical)
        if state.static_eval > 100 {
            bonus += 20;
        }
        
        bonus
    }
    
    /// Check if extended delta pruning should be applied (for deeper positions)
    fn check_extended_delta_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        // Only apply extended delta pruning at deeper depths
        if state.depth <= self.parameters.delta_depth_limit || 
           state.depth > self.parameters.delta_depth_limit + 2 {
            return current;
        }
        
        if !self.is_capture_move(mv) {
            return current;
        }
        
        // Extended delta pruning with larger margins
        let material_gain = self.calculate_material_gain(mv);
        let extended_margin = self.get_delta_margin(state).saturating_mul(2);
        let capture_bonus = self.calculate_capture_bonus(mv, state);
        
        // More conservative pruning at deeper depths
        if state.static_eval + material_gain + extended_margin + capture_bonus < state.alpha {
            self.statistics.delta_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Advanced delta pruning with multiple conditions
    fn check_advanced_delta_pruning(&mut self, state: &SearchState, mv: &Move, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.delta_depth_limit + 2 {
            return current;
        }
        
        if !self.is_capture_move(mv) {
            return current;
        }
        
        // Multiple delta pruning conditions
        let material_gain = self.calculate_material_gain(mv);
        let margin = self.get_delta_margin(state);
        let capture_bonus = self.calculate_capture_bonus(mv, state);
        
        // Condition 1: Standard delta pruning
        if state.static_eval + material_gain + margin + capture_bonus < state.alpha {
            self.statistics.delta_pruned += 1;
            return PruningDecision::Skip;
        }
        
        // Condition 2: Aggressive delta pruning for very bad positions
        if state.static_eval < state.alpha.saturating_sub(300) && 
           state.static_eval + material_gain + margin / 2 + capture_bonus < state.alpha {
            self.statistics.delta_pruned += 1;
            return PruningDecision::Skip;
        }
        
        // Condition 3: Late move delta pruning (for moves beyond a certain threshold)
        if state.move_number > 3 && 
           state.static_eval + material_gain + margin + capture_bonus / 2 < state.alpha {
            self.statistics.delta_pruned += 1;
            return PruningDecision::Skip;
        }
        
        current
    }
    
    /// Get razoring margin based on game phase
    fn get_razoring_margin(&self, state: &SearchState) -> i32 {
        match state.game_phase {
            GamePhase::Endgame => self.parameters.razoring_margin_endgame,
            _ => self.parameters.razoring_margin,
        }
    }
    
    /// Calculate razoring bonus based on position characteristics
    fn calculate_razoring_bonus(&self, state: &SearchState) -> i32 {
        let mut bonus = 0;
        
        // Bonus for tactical positions (more likely to have tactical shots)
        if state.depth <= 2 {
            bonus += 50; // More conservative at shallow depths
        }
        
        // Bonus for endgame positions (more tactical)
        if state.game_phase == GamePhase::Endgame {
            bonus += 75;
        }
        
        // Bonus for positions with material imbalance (more tactical)
        if state.static_eval.abs() > 200 {
            bonus += 25;
        }
        
        // Bonus for deeper positions (more tactical)
        if state.depth > 1 {
            bonus += state.depth as i32 * 15;
        }
        
        // Penalty for very bad positions (less likely to have tactical shots)
        if state.static_eval < -500 {
            bonus -= 50;
        }
        
        bonus
    }
    
    /// Check if extended razoring should be applied (for deeper positions)
    fn check_extended_razoring(&mut self, state: &SearchState, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        // Only apply extended razoring at deeper depths
        if state.depth <= self.parameters.razoring_depth_limit || 
           state.depth > self.parameters.razoring_depth_limit + 2 {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Extended razoring with larger margins
        let extended_margin = self.get_razoring_margin(state).saturating_mul(2);
        let position_bonus = self.calculate_razoring_bonus(state);
        
        // More conservative razoring at deeper depths
        if state.static_eval + extended_margin + position_bonus < state.alpha {
            self.statistics.razored += 1;
            return PruningDecision::Razor;
        }
        
        current
    }
    
    /// Advanced razoring with multiple conditions
    fn check_advanced_razoring(&mut self, state: &SearchState, current: PruningDecision) -> PruningDecision {
        if current != PruningDecision::Search {
            return current;
        }
        
        if state.depth > self.parameters.razoring_depth_limit + 2 {
            return current;
        }
        
        if state.is_in_check {
            return current;
        }
        
        // Multiple razoring conditions
        let margin = self.get_razoring_margin(state);
        let position_bonus = self.calculate_razoring_bonus(state);
        
        // Condition 1: Standard razoring
        if state.static_eval + margin + position_bonus < state.alpha {
            self.statistics.razored += 1;
            return PruningDecision::Razor;
        }
        
        // Condition 2: Aggressive razoring for very bad positions
        if state.static_eval < state.alpha.saturating_sub(400) && 
           state.static_eval + margin / 2 + position_bonus < state.alpha {
            self.statistics.razored += 1;
            return PruningDecision::Razor;
        }
        
        // Condition 3: Late move razoring (for moves beyond a certain threshold)
        if state.move_number > 2 && 
           state.static_eval + margin + position_bonus / 2 < state.alpha {
            self.statistics.razored += 1;
            return PruningDecision::Razor;
        }
        
        current
    }
    
    /// Check if a move is tactical (capture, promotion, or check)
    fn is_tactical_move(&self, mv: &Move) -> bool {
        mv.is_capture || mv.is_promotion || mv.gives_check
    }
    
    /// Optimized check detection with caching
    pub fn is_in_check_cached(&mut self, position_hash: u64, is_in_check: bool) -> bool {
        if let Some(&cached_result) = self.check_cache.get(&position_hash) {
            self.cache_hits += 1;
            return cached_result;
        }
        
        self.cache_misses += 1;
        
        // Cache the result (with size limit)
        if self.check_cache.len() < 5000 {
            self.check_cache.insert(position_hash, is_in_check);
        }
        
        is_in_check
    }
    
    /// Analyze position characteristics for adaptive pruning
    pub fn analyze_position(&mut self, state: &SearchState) -> PositionAnalysis {
        let cache_key = state.position_hash;
        
        if let Some(cached_analysis) = self.position_cache.get(&cache_key) {
            self.cache_hits += 1;
            return cached_analysis.clone();
        }
        
        self.cache_misses += 1;
        
        let analysis = PositionAnalysis {
            position_type: self.classify_position_type(state),
            tactical_potential: self.calculate_tactical_potential(state),
            material_balance: state.static_eval,
            king_safety: self.calculate_king_safety(state),
            is_quiet: self.is_quiet_position(state),
            is_tactical: self.is_tactical_position(state),
            complexity: self.calculate_position_complexity(state),
        };
        
        // Cache the result (with size limit)
        if self.position_cache.len() < 3000 {
            self.position_cache.insert(cache_key, analysis.clone());
        }
        
        analysis
    }
    
    /// Classify position type for adaptive pruning
    fn classify_position_type(&self, state: &SearchState) -> PositionType {
        match state.game_phase {
            GamePhase::Endgame => PositionType::Endgame,
            GamePhase::Opening => {
                if state.static_eval.abs() > 200 {
                    PositionType::Tactical
                } else {
                    PositionType::Positional
                }
            },
            GamePhase::Middlegame => {
                if state.static_eval.abs() > 300 {
                    PositionType::Tactical
                } else {
                    PositionType::Positional
                }
            },
        }
    }
    
    /// Calculate tactical potential of position
    fn calculate_tactical_potential(&self, state: &SearchState) -> u8 {
        let mut potential = 0;
        
        // Material imbalance increases tactical potential
        if state.static_eval.abs() > 200 {
            potential += 30;
        }
        
        // Endgame positions are more tactical
        if state.game_phase == GamePhase::Endgame {
            potential += 40;
        }
        
        // Deeper positions have higher tactical potential
        potential += state.depth as u8 * 5;
        
        potential.min(100)
    }
    
    /// Calculate king safety
    fn calculate_king_safety(&self, state: &SearchState) -> u8 {
        // Simplified king safety calculation
        // In a real implementation, this would analyze king position, pawn structure, etc.
        if state.static_eval < -300 {
            20 // King in danger
        } else if state.static_eval > 300 {
            80 // King safe
        } else {
            50 // Neutral
        }
    }
    
    /// Check if position is quiet
    fn is_quiet_position(&self, state: &SearchState) -> bool {
        state.static_eval.abs() < 100 && state.game_phase != GamePhase::Endgame
    }
    
    /// Check if position is tactical
    fn is_tactical_position(&self, state: &SearchState) -> bool {
        state.static_eval.abs() > 200 || state.game_phase == GamePhase::Endgame
    }
    
    /// Calculate position complexity
    fn calculate_position_complexity(&self, state: &SearchState) -> u8 {
        let mut complexity = 0;
        
        // Material imbalance increases complexity
        complexity += (state.static_eval.abs() / 50) as u8;
        
        // Endgame positions are more complex
        if state.game_phase == GamePhase::Endgame {
            complexity += 30;
        }
        
        // Deeper positions are more complex
        complexity += state.depth as u8 * 3;
        
        complexity.min(100)
    }
    
    /// Get cache performance statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        let total_requests = self.cache_hits + self.cache_misses;
        let hit_rate = if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }
    
    /// Clear all caches to free memory
    pub fn clear_caches(&mut self) {
        self.check_cache.clear();
        self.position_cache.clear();
        self.pruning_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
    
    /// Smart conditional pruning based on position characteristics
    pub fn should_apply_conditional_pruning(&mut self, state: &SearchState, _mv: &Move) -> bool {
        let analysis = self.analyze_position(state);
        
        // Don't prune in very tactical positions
        if analysis.is_tactical && analysis.tactical_potential > 70 {
            return false;
        }
        
        // Don't prune when king is in danger
        if analysis.king_safety < 30 {
            return false;
        }
        
        // Don't prune in very complex positions
        if analysis.complexity > 80 {
            return false;
        }
        
        // Don't prune the first few moves at each depth
        if state.move_number < 3 {
            return false;
        }
        
        // Apply conditional pruning based on position type
        match analysis.position_type {
            PositionType::Tactical => {
                // Be more conservative in tactical positions
                state.move_number > 4 && analysis.tactical_potential < 50
            },
            PositionType::Positional => {
                // Can be more aggressive in positional positions
                state.move_number > 2 && analysis.complexity < 60
            },
            PositionType::Endgame => {
                // Endgame pruning depends on material balance
                state.move_number > 1 && analysis.material_balance.abs() < 200
            },
            PositionType::Normal => {
                // Standard pruning conditions
                state.move_number > 2
            },
        }
    }
    
    /// Optimize pruning frequency based on current performance
    pub fn optimize_pruning_frequency(&mut self) {
        let stats = &self.statistics;
        let total_moves = stats.total_moves.max(1);
        let pruning_rate = stats.pruned_moves as f64 / total_moves as f64;
        
        // Adjust parameters based on pruning effectiveness
        if pruning_rate > 0.4 {
            // High pruning rate - be more conservative
            self.parameters.futility_depth_limit = self.parameters.futility_depth_limit.max(3);
            self.parameters.delta_depth_limit = self.parameters.delta_depth_limit.max(4);
            self.parameters.razoring_depth_limit = self.parameters.razoring_depth_limit.max(4);
        } else if pruning_rate < 0.1 {
            // Low pruning rate - be more aggressive
            self.parameters.futility_depth_limit = self.parameters.futility_depth_limit.min(5);
            self.parameters.delta_depth_limit = self.parameters.delta_depth_limit.min(6);
            self.parameters.razoring_depth_limit = self.parameters.razoring_depth_limit.min(5);
        }
        
        // Adjust margins based on success rate
        let success_rate = if stats.total_moves > 0 {
            (stats.total_moves - stats.pruned_moves) as f64 / stats.total_moves as f64
        } else {
            0.0
        };
        
        if success_rate > 0.8 {
            // High success rate - can be more aggressive
            for i in 0..8 {
                self.parameters.futility_margin[i] = (self.parameters.futility_margin[i] as f32 * 0.9) as i32;
            }
            self.parameters.delta_margin = (self.parameters.delta_margin as f32 * 0.9) as i32;
            self.parameters.razoring_margin = (self.parameters.razoring_margin as f32 * 0.9) as i32;
        } else if success_rate < 0.6 {
            // Low success rate - be more conservative
            for i in 0..8 {
                self.parameters.futility_margin[i] = (self.parameters.futility_margin[i] as f32 * 1.1) as i32;
            }
            self.parameters.delta_margin = (self.parameters.delta_margin as f32 * 1.1) as i32;
            self.parameters.razoring_margin = (self.parameters.razoring_margin as f32 * 1.1) as i32;
        }
    }
    
    /// Check if a move is a capture
    fn is_capture_move(&self, mv: &Move) -> bool {
        mv.is_capture
    }
    
    /// Check if a move is a promotion
    fn is_promotion_move(&self, mv: &Move) -> bool {
        mv.is_promotion
    }
    
    /// Calculate material gain from a capture move
    fn calculate_material_gain(&self, mv: &Move) -> i32 {
        if let Some(captured_piece) = mv.captured_piece {
            captured_piece.piece_type.base_value() - mv.piece_type.base_value()
        } else {
            0
        }
    }
    
    // ============================================================================
    // Advanced Pruning Techniques (Phase 4.2)
    // ============================================================================
    
    /// Extended futility pruning with more aggressive margins
    pub fn check_extended_futility(&mut self, state: &SearchState, mv: &Move) -> PruningDecision {
        // Only apply at appropriate depths
        if state.depth > self.parameters.extended_futility_depth {
            return PruningDecision::Search;
        }
        
        // Skip for important moves
        if self.is_capture_move(mv) || self.is_promotion_move(mv) {
            return PruningDecision::Search;
        }
        
        // Extended futility margin calculation
        let extended_margin = self.get_extended_futility_margin(state);
        let futility_value = state.static_eval + extended_margin;
        
        // Check if the move is unlikely to raise alpha
        if futility_value <= state.alpha {
            self.statistics.futility_pruned += 1;
            return PruningDecision::Skip;
        }
        
        PruningDecision::Search
    }
    
    /// Get extended futility margin based on depth and position
    fn get_extended_futility_margin(&self, state: &SearchState) -> i32 {
        let base_margin = if state.depth < self.parameters.futility_margin.len() as u8 {
            self.parameters.futility_margin[state.depth as usize]
        } else {
            self.parameters.futility_margin[self.parameters.futility_margin.len() - 1]
        };
        
        // Extended margins are more aggressive
        let extended_multiplier = match state.game_phase {
            GamePhase::Opening => 1.5,
            GamePhase::Middlegame => 1.3,
            GamePhase::Endgame => 1.2,
        };
        
        (base_margin as f32 * extended_multiplier) as i32
    }
    
    /// Multi-cut pruning: prune if multiple moves fail to raise alpha
    pub fn check_multi_cut(&mut self, state: &SearchState, moves_tried: usize, 
                          consecutive_fails: usize) -> PruningDecision {
        // Only apply at appropriate depths
        if state.depth > self.parameters.multi_cut_depth_limit {
            return PruningDecision::Search;
        }
        
        // Need to have tried at least a few moves
        if moves_tried < self.parameters.multi_cut_threshold as usize {
            return PruningDecision::Search;
        }
        
        // Check if we have enough consecutive failures
        if consecutive_fails >= self.parameters.multi_cut_threshold as usize {
            self.statistics.multi_cuts += 1;
            return PruningDecision::Skip;
        }
        
        PruningDecision::Search
    }
    
    /// Probabilistic pruning: prune based on move success probability
    pub fn check_probabilistic_pruning(&mut self, state: &SearchState, mv: &Move, 
                                      move_index: usize) -> PruningDecision {
        // Only apply in late moves at appropriate depths
        if state.depth > 4 || move_index < 8 {
            return PruningDecision::Search;
        }
        
        // Calculate move success probability
        let probability = self.calculate_move_probability(state, mv, move_index);
        
        // Probabilistic threshold based on depth
        let threshold = match state.depth {
            0..=1 => 0.05,  // Very aggressive at low depth
            2..=3 => 0.10,  // Moderate at medium depth
            _ => 0.15,      // Conservative at higher depth
        };
        
        // Prune if probability of success is too low
        if probability < threshold {
            self.statistics.pruned_moves += 1;
            return PruningDecision::Skip;
        }
        
        PruningDecision::Search
    }
    
    /// Calculate the probability that a move will improve the score
    fn calculate_move_probability(&self, state: &SearchState, mv: &Move, move_index: usize) -> f64 {
        let mut probability = 1.0;
        
        // Reduce probability for late moves
        probability *= 1.0 / (1.0 + move_index as f64 * 0.1);
        
        // Increase probability for captures
        if self.is_capture_move(mv) {
            probability *= 1.5;
        }
        
        // Increase probability for promotions
        if self.is_promotion_move(mv) {
            probability *= 1.3;
        }
        
        // Adjust based on game phase
        let phase_factor = match state.game_phase {
            GamePhase::Opening => 0.8,   // Less reliable in opening
            GamePhase::Middlegame => 1.0, // Normal in middlegame
            GamePhase::Endgame => 1.2,   // More reliable in endgame
        };
        probability *= phase_factor;
        
        // Adjust based on depth
        let depth_factor = 1.0 - (state.depth as f64 * 0.05);
        probability *= depth_factor.max(0.5);
        
        // Clamp probability to [0, 1]
        probability.min(1.0).max(0.0)
    }
    
    /// Enhanced multi-cut with position-dependent thresholds
    pub fn check_enhanced_multi_cut(&mut self, state: &SearchState, moves_tried: usize,
                                   consecutive_fails: usize, best_score: i32) -> PruningDecision {
        // Basic multi-cut first
        let basic_decision = self.check_multi_cut(state, moves_tried, consecutive_fails);
        if matches!(basic_decision, PruningDecision::Skip) {
            return basic_decision;
        }
        
        // Enhanced check: if best score is far below alpha, be more aggressive
        let score_gap = state.alpha.saturating_sub(best_score);
        let gap_threshold = match state.game_phase {
            GamePhase::Opening => 300,
            GamePhase::Middlegame => 250,
            GamePhase::Endgame => 200,
        };
        
        if score_gap > gap_threshold && consecutive_fails >= 2 {
            self.statistics.multi_cuts += 1;
            return PruningDecision::Skip;
        }
        
        PruningDecision::Search
    }
    
    /// Validate extended futility pruning effectiveness
    pub fn validate_extended_futility(&self, state: &SearchState) -> bool {
        // Check if conditions are appropriate for extended futility
        state.depth <= self.parameters.extended_futility_depth &&
        state.static_eval < state.beta &&
        !state.is_in_check
    }
    
    /// Validate multi-cut pruning effectiveness
    pub fn validate_multi_cut(&self, moves_tried: usize, consecutive_fails: usize) -> bool {
        moves_tried >= self.parameters.multi_cut_threshold as usize &&
        consecutive_fails >= self.parameters.multi_cut_threshold as usize
    }
    
    /// Get pruning effectiveness statistics
    pub fn get_pruning_effectiveness(&self) -> PruningEffectiveness {
        let total_opportunities = self.statistics.total_moves;
        let total_pruned = self.statistics.pruned_moves;
        
        let effectiveness_ratio = if total_opportunities > 0 {
            total_pruned as f64 / total_opportunities as f64
        } else {
            0.0
        };
        
        PruningEffectiveness {
            futility_rate: if total_opportunities > 0 {
                self.statistics.futility_pruned as f64 / total_opportunities as f64
            } else {
                0.0
            },
            delta_rate: if total_opportunities > 0 {
                self.statistics.delta_pruned as f64 / total_opportunities as f64
            } else {
                0.0
            },
            razoring_rate: if total_opportunities > 0 {
                self.statistics.razored as f64 / total_opportunities as f64
            } else {
                0.0
            },
            multi_cut_rate: if total_opportunities > 0 {
                self.statistics.multi_cuts as f64 / total_opportunities as f64
            } else {
                0.0
            },
            overall_effectiveness: effectiveness_ratio,
            lmr_rate: if total_opportunities > 0 {
                self.statistics.lmr_applied as f64 / total_opportunities as f64
            } else {
                0.0
            },
        }
    }
    
    // ============================================================================
    // Performance Monitoring (Phase 4.3)
    // ============================================================================
    
    /// Record pruning decision with detailed statistics
    pub fn record_pruning_decision(&mut self, decision: PruningDecision, move_type: MoveType, depth: u8) {
        self.statistics.total_moves += 1;
        
        match decision {
            PruningDecision::Skip => {
                self.statistics.pruned_moves += 1;
                self.record_pruning_by_type(move_type, depth);
            },
            PruningDecision::Razor => {
                self.statistics.razored += 1;
            },
            PruningDecision::ReducedSearch => {
                self.statistics.lmr_applied += 1;
            },
            _ => {}
        }
    }
    
    /// Record pruning by move type and depth for detailed analysis
    fn record_pruning_by_type(&mut self, move_type: MoveType, _depth: u8) {
        // This would be enhanced with more detailed tracking
        // For now, we use the existing statistics
        match move_type {
            MoveType::Capture => {
                // Captures are rarely pruned, so this is significant
            },
            MoveType::Quiet => {
                // Quiet moves are commonly pruned
            },
            MoveType::Check => {
                // Check moves should be pruned carefully
            },
            MoveType::Promotion => {
                // Promotion moves should be pruned carefully
            },
            _ => {}
        }
    }
    
    /// Get detailed pruning frequency statistics
    pub fn get_pruning_frequency_stats(&self) -> PruningFrequencyStats {
        let total_moves = self.statistics.total_moves;
        
        PruningFrequencyStats {
            total_moves,
            pruned_moves: self.statistics.pruned_moves,
            futility_pruned: self.statistics.futility_pruned,
            delta_pruned: self.statistics.delta_pruned,
            razored: self.statistics.razored,
            lmr_applied: self.statistics.lmr_applied,
            multi_cuts: self.statistics.multi_cuts,
            re_searches: self.statistics.re_searches,
            pruning_rate: if total_moves > 0 {
                self.statistics.pruned_moves as f64 / total_moves as f64
            } else {
                0.0
            },
            cache_hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
            } else {
                0.0
            },
        }
    }
    
    /// Get search performance metrics
    pub fn get_search_performance_metrics(&self) -> SearchPerformanceMetrics {
        SearchPerformanceMetrics {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_size: self.pruning_cache.len(),
            position_cache_size: self.position_cache.len(),
            check_cache_size: self.check_cache.len(),
            total_cache_operations: self.cache_hits + self.cache_misses,
            cache_hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
            } else {
                0.0
            },
        }
    }
    
    /// Generate comprehensive performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let pruning_effectiveness = self.get_pruning_effectiveness();
        let frequency_stats = self.get_pruning_frequency_stats();
        let search_metrics = self.get_search_performance_metrics();
        
        PerformanceReport {
            pruning_effectiveness,
            frequency_stats,
            search_metrics,
            timestamp: std::time::SystemTime::now(),
            report_id: self.generate_report_id(),
        }
    }
    
    /// Generate unique report ID
    fn generate_report_id(&self) -> String {
        format!("pruning_report_{}", self.statistics.total_moves)
    }
    
    /// Compare performance with baseline
    pub fn compare_with_baseline(&self, baseline: &PerformanceReport) -> PerformanceComparison {
        let current = self.generate_performance_report();
        let current_clone = current.clone();
        
        PerformanceComparison {
            current_report: current,
            baseline_report: baseline.clone(),
            pruning_improvement: self.calculate_pruning_improvement(&current_clone, baseline),
            cache_improvement: self.calculate_cache_improvement(&current_clone, baseline),
            overall_improvement: self.calculate_overall_improvement(&current_clone, baseline),
        }
    }
    
    /// Calculate pruning improvement metrics
    fn calculate_pruning_improvement(&self, current: &PerformanceReport, baseline: &PerformanceReport) -> PruningImprovement {
        let current_effectiveness = &current.pruning_effectiveness;
        let baseline_effectiveness = &baseline.pruning_effectiveness;
        
        PruningImprovement {
            futility_improvement: current_effectiveness.futility_rate - baseline_effectiveness.futility_rate,
            delta_improvement: current_effectiveness.delta_rate - baseline_effectiveness.delta_rate,
            razoring_improvement: current_effectiveness.razoring_rate - baseline_effectiveness.razoring_rate,
            multi_cut_improvement: current_effectiveness.multi_cut_rate - baseline_effectiveness.multi_cut_rate,
            overall_effectiveness_improvement: current_effectiveness.overall_effectiveness - baseline_effectiveness.overall_effectiveness,
        }
    }
    
    /// Calculate cache improvement metrics
    fn calculate_cache_improvement(&self, current: &PerformanceReport, baseline: &PerformanceReport) -> CacheImprovement {
        let current_metrics = &current.search_metrics;
        let baseline_metrics = &baseline.search_metrics;
        
        CacheImprovement {
            hit_rate_improvement: current_metrics.cache_hit_rate - baseline_metrics.cache_hit_rate,
            size_efficiency: if baseline_metrics.cache_size > 0 {
                current_metrics.cache_size as f64 / baseline_metrics.cache_size as f64
            } else {
                1.0
            },
            operation_efficiency: if baseline_metrics.total_cache_operations > 0 {
                current_metrics.total_cache_operations as f64 / baseline_metrics.total_cache_operations as f64
            } else {
                1.0
            },
        }
    }
    
    /// Calculate overall improvement score
    fn calculate_overall_improvement(&self, current: &PerformanceReport, baseline: &PerformanceReport) -> f64 {
        let pruning_improvement = self.calculate_pruning_improvement(current, baseline);
        let cache_improvement = self.calculate_cache_improvement(current, baseline);
        
        // Weighted combination of improvements
        let pruning_score = pruning_improvement.overall_effectiveness_improvement * 0.6;
        let cache_score = cache_improvement.hit_rate_improvement * 0.4;
        
        pruning_score + cache_score
    }
    
    /// Reset all performance statistics
    pub fn reset_performance_stats(&mut self) {
        self.statistics.reset();
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.pruning_cache.clear();
        self.position_cache.clear();
        self.check_cache.clear();
    }
    
    /// Export performance data for analysis
    pub fn export_performance_data(&self) -> PerformanceDataExport {
        PerformanceDataExport {
            report: self.generate_performance_report(),
            raw_statistics: self.statistics.clone(),
            cache_stats: CacheStats {
                hits: self.cache_hits,
                misses: self.cache_misses,
                pruning_cache_size: self.pruning_cache.len(),
                position_cache_size: self.position_cache.len(),
                check_cache_size: self.check_cache.len(),
            },
        }
    }
}

/// Adaptive parameters for position-dependent pruning
#[derive(Debug, PartialEq)]
pub struct AdaptiveParameters {
    pub position_analysis: PositionAnalyzer,
    pub parameter_history: Vec<ParameterSnapshot>,
    pub learning_rate: f64,
}

impl AdaptiveParameters {
    pub fn new() -> Self {
        Self {
            position_analysis: PositionAnalyzer::new(),
            parameter_history: Vec::new(),
            learning_rate: 0.1,
        }
    }
    
    /// Adjust parameters based on performance metrics and position analysis
    pub fn adjust_parameters(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                           player: Player, performance: &PerformanceMetrics, 
                           current_params: &PruningParameters) -> PruningParameters {
        // Analyze current position
        let position_analysis = self.position_analysis.analyze_position(board, captured_pieces, player);
        
        // Calculate parameter adjustments based on position type and performance
        let adjustment = self.calculate_adjustment(position_analysis.position_type, performance);
        
        // Apply adjustments to current parameters
        let new_params = self.apply_adjustment(current_params, adjustment);
        
        // Record parameter change in history
        self.record_parameter_change(new_params.clone(), performance.clone());
        
        new_params
    }
    
    /// Calculate parameter adjustment based on position type and performance
    fn calculate_adjustment(&self, position_type: PositionType, performance: &PerformanceMetrics) -> ParameterAdjustment {
        let mut adjustment = ParameterAdjustment::default();
        
        // Calculate cache hit ratio for performance assessment
        let total_cache_ops = performance.cache_hits + performance.cache_misses;
        let cache_hit_ratio = if total_cache_ops > 0 {
            performance.cache_hits as f64 / total_cache_ops as f64
        } else {
            0.5 // Default neutral ratio
        };
        
        // Adjust parameters based on position type and performance
        match position_type {
            PositionType::Tactical => {
                // In tactical positions, be more conservative with pruning
                adjustment.futility_adjustment = -100;
                adjustment.lmr_adjustment = 1;
                adjustment.delta_adjustment = -50;
                adjustment.razoring_adjustment = -200;
            },
            PositionType::Positional => {
                // In positional positions, can be more aggressive with pruning
                adjustment.futility_adjustment = 50;
                adjustment.lmr_adjustment = 0;
                adjustment.delta_adjustment = 25;
                adjustment.razoring_adjustment = 100;
            },
            PositionType::Endgame => {
                // In endgame, be very conservative to avoid tactical errors
                adjustment.futility_adjustment = -200;
                adjustment.lmr_adjustment = 2;
                adjustment.delta_adjustment = -100;
                adjustment.razoring_adjustment = -300;
            },
            PositionType::Normal => {
                // Normal adjustments based on performance
                if cache_hit_ratio < 0.3 {
                    // Poor cache performance, reduce pruning aggressiveness
                    adjustment.futility_adjustment = -50;
                    adjustment.lmr_adjustment = 1;
                    adjustment.delta_adjustment = -25;
                    adjustment.razoring_adjustment = -100;
                } else if cache_hit_ratio > 0.7 {
                    // Good cache performance, can be more aggressive
                    adjustment.futility_adjustment = 25;
                    adjustment.lmr_adjustment = 0;
                    adjustment.delta_adjustment = 15;
                    adjustment.razoring_adjustment = 50;
                }
            }
        }
        
        // Apply learning rate to adjustments
        adjustment.futility_adjustment = (adjustment.futility_adjustment as f64 * self.learning_rate) as i32;
        adjustment.lmr_adjustment = (adjustment.lmr_adjustment as f64 * self.learning_rate) as u8;
        adjustment.delta_adjustment = (adjustment.delta_adjustment as f64 * self.learning_rate) as i32;
        adjustment.razoring_adjustment = (adjustment.razoring_adjustment as f64 * self.learning_rate) as i32;
        
        adjustment
    }
    
    /// Apply parameter adjustments to current parameters
    fn apply_adjustment(&self, current_params: &PruningParameters, adjustment: ParameterAdjustment) -> PruningParameters {
        let mut new_params = current_params.clone();
        
        // Apply futility pruning adjustments
        for i in 0..new_params.futility_margin.len() {
            new_params.futility_margin[i] = (new_params.futility_margin[i] as i32 + adjustment.futility_adjustment)
                .max(50) // Minimum margin
                .min(1000) as i32; // Maximum margin
        }
        
        // Apply LMR adjustments
        new_params.lmr_base_reduction = (new_params.lmr_base_reduction as i32 + adjustment.lmr_adjustment as i32)
            .max(1) // Minimum reduction
            .min(4) as u8; // Maximum reduction
        
        // Apply delta pruning adjustments
        new_params.delta_margin = (new_params.delta_margin + adjustment.delta_adjustment)
            .max(25) // Minimum margin
            .min(500); // Maximum margin
        
        // Apply razoring adjustments
        new_params.razoring_margin = (new_params.razoring_margin as i32 + adjustment.razoring_adjustment)
            .max(100) // Minimum margin
            .min(2000) as i32; // Maximum margin
        
        new_params
    }
    
    /// Record parameter change in history for learning
    fn record_parameter_change(&mut self, parameters: PruningParameters, performance: PerformanceMetrics) {
        let snapshot = ParameterSnapshot {
            timestamp: std::time::SystemTime::now(),
            parameters,
            performance,
        };
        
        self.parameter_history.push(snapshot);
        
        // Limit history size to prevent memory growth
        if self.parameter_history.len() > 1000 {
            self.parameter_history.remove(0);
        }
    }
    
    /// Optimize learning rate based on recent performance
    pub fn optimize_learning_rate(&mut self) {
        if self.parameter_history.len() < 10 {
            return;
        }
        
        let recent_snapshots = &self.parameter_history[self.parameter_history.len() - 10..];
        let avg_cache_hit_ratio = recent_snapshots.iter()
            .map(|s| {
                let total = s.performance.cache_hits + s.performance.cache_misses;
                if total > 0 { s.performance.cache_hits as f64 / total as f64 } else { 0.5 }
            })
            .sum::<f64>() / recent_snapshots.len() as f64;
        
        // Adjust learning rate based on performance
        if avg_cache_hit_ratio > 0.8 {
            // Good performance, can be more aggressive
            self.learning_rate = (self.learning_rate * 1.05).min(0.2);
        } else if avg_cache_hit_ratio < 0.3 {
            // Poor performance, be more conservative
            self.learning_rate = (self.learning_rate * 0.95).max(0.01);
        }
    }
    
    /// Get parameter recommendations based on position analysis
    pub fn get_position_recommendations(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                                      player: Player) -> PruningParameters {
        let position_analysis = self.position_analysis.analyze_position(board, captured_pieces, player);
        
        // Start with default parameters
        let mut params = PruningParameters::default();
        
        // Adjust based on position characteristics
        match position_analysis.position_type {
            PositionType::Tactical => {
                // Conservative pruning for tactical positions
                params.futility_margin = [300, 350, 400, 450, 500, 550, 600, 650];
                params.lmr_base_reduction = 1;
                params.delta_margin = 150;
                params.razoring_margin = 800;
            },
            PositionType::Positional => {
                // More aggressive pruning for positional positions
                params.futility_margin = [150, 175, 200, 225, 250, 275, 300, 325];
                params.lmr_base_reduction = 2;
                params.delta_margin = 75;
                params.razoring_margin = 400;
            },
            PositionType::Endgame => {
                // Very conservative pruning for endgame
                params.futility_margin = [400, 450, 500, 550, 600, 650, 700, 750];
                params.lmr_base_reduction = 1;
                params.delta_margin = 200;
                params.razoring_margin = 1000;
            },
            PositionType::Normal => {
                // Standard parameters
                params.futility_margin = [200, 225, 250, 275, 300, 325, 350, 375];
                params.lmr_base_reduction = 2;
                params.delta_margin = 100;
                params.razoring_margin = 600;
            }
        }
        
        params
    }
    
    /// Implement parameter learning system based on performance tracking
    pub fn learn_from_performance(&mut self, performance_history: &[PerformanceMetrics]) {
        if performance_history.len() < 5 {
            return;
        }
        
        // Calculate performance trends
        let recent_performance = &performance_history[performance_history.len() - 5..];
        let avg_cache_hit_ratio = self.calculate_average_cache_hit_ratio(recent_performance);
        let performance_trend = self.calculate_performance_trend(performance_history);
        
        // Adjust learning rate based on performance
        if performance_trend > 0.1 {
            // Improving performance, increase learning rate
            self.learning_rate = (self.learning_rate * 1.1).min(0.2);
        } else if performance_trend < -0.1 {
            // Declining performance, decrease learning rate
            self.learning_rate = (self.learning_rate * 0.9).max(0.01);
        }
        
        // Update parameter recommendations based on performance patterns
        self.update_parameter_recommendations(avg_cache_hit_ratio, performance_trend);
    }
    
    /// Calculate average cache hit ratio from performance metrics
    fn calculate_average_cache_hit_ratio(&self, performance_metrics: &[PerformanceMetrics]) -> f64 {
        let total_ratio: f64 = performance_metrics.iter()
            .map(|pm| {
                let total = pm.cache_hits + pm.cache_misses;
                if total > 0 { pm.cache_hits as f64 / total as f64 } else { 0.5 }
            })
            .sum();
        
        total_ratio / performance_metrics.len() as f64
    }
    
    /// Calculate performance trend over time
    fn calculate_performance_trend(&self, performance_history: &[PerformanceMetrics]) -> f64 {
        if performance_history.len() < 10 {
            return 0.0;
        }
        
        let recent = &performance_history[performance_history.len() - 5..];
        let older = &performance_history[performance_history.len() - 10..performance_history.len() - 5];
        
        let recent_avg = self.calculate_average_cache_hit_ratio(recent);
        let older_avg = self.calculate_average_cache_hit_ratio(older);
        
        recent_avg - older_avg
    }
    
    /// Update parameter recommendations based on performance patterns
    fn update_parameter_recommendations(&mut self, cache_hit_ratio: f64, performance_trend: f64) {
        // This would update the internal parameter recommendations
        // based on observed performance patterns
        // For now, we'll just adjust the learning rate
        if cache_hit_ratio > 0.8 && performance_trend > 0.0 {
            self.learning_rate = (self.learning_rate * 1.05).min(0.2);
        } else if cache_hit_ratio < 0.3 || performance_trend < -0.1 {
            self.learning_rate = (self.learning_rate * 0.95).max(0.01);
        }
    }
    
    /// Get optimized parameters for specific game phase
    pub fn get_phase_optimized_parameters(&self, game_phase: GamePhase) -> PruningParameters {
        let mut params = PruningParameters::default();
        
        match game_phase {
            GamePhase::Opening => {
                // Opening: moderate pruning, focus on development
                params.futility_margin = [180, 200, 220, 240, 260, 280, 300, 320];
                params.lmr_base_reduction = 2;
                params.delta_margin = 90;
                params.razoring_margin = 500;
            },
            GamePhase::Middlegame => {
                // Middlegame: standard pruning
                params.futility_margin = [200, 225, 250, 275, 300, 325, 350, 375];
                params.lmr_base_reduction = 2;
                params.delta_margin = 100;
                params.razoring_margin = 600;
            },
            GamePhase::Endgame => {
                // Endgame: conservative pruning to avoid tactical errors
                params.futility_margin = [300, 350, 400, 450, 500, 550, 600, 650];
                params.lmr_base_reduction = 1;
                params.delta_margin = 150;
                params.razoring_margin = 800;
            }
        }
        
        params
    }
    
    /// Validate parameter ranges and constraints
    pub fn validate_parameters(&self, params: &PruningParameters) -> bool {
        // Check futility margin ranges
        for margin in &params.futility_margin {
            if *margin < 50 || *margin > 1000 {
                return false;
            }
        }
        
        // Check LMR reduction range
        if params.lmr_base_reduction < 1 || params.lmr_base_reduction > 4 {
            return false;
        }
        
        // Check delta margin range
        if params.delta_margin < 25 || params.delta_margin > 500 {
            return false;
        }
        
        // Check razoring margin range
        if params.razoring_margin < 100 || params.razoring_margin > 2000 {
            return false;
        }
        
        true
    }
    
    /// Get parameter statistics for analysis
    pub fn get_parameter_statistics(&self) -> ParameterStatistics {
        if self.parameter_history.is_empty() {
            return ParameterStatistics::default();
        }
        
        let recent_snapshots = &self.parameter_history[self.parameter_history.len() - 10..];
        
        let avg_cache_hit_ratio = recent_snapshots.iter()
            .map(|s| {
                let total = s.performance.cache_hits + s.performance.cache_misses;
                if total > 0 { s.performance.cache_hits as f64 / total as f64 } else { 0.5 }
            })
            .sum::<f64>() / recent_snapshots.len() as f64;
        
        let avg_futility_margin = recent_snapshots.iter()
            .map(|s| s.parameters.futility_margin[0] as f64)
            .sum::<f64>() / recent_snapshots.len() as f64;
        
        let avg_lmr_reduction = recent_snapshots.iter()
            .map(|s| s.parameters.lmr_base_reduction as f64)
            .sum::<f64>() / recent_snapshots.len() as f64;
        
        ParameterStatistics {
            total_adjustments: self.parameter_history.len(),
            avg_cache_hit_ratio,
            avg_futility_margin,
            avg_lmr_reduction,
            learning_rate: self.learning_rate,
        }
    }
}

/// Position analyzer for adaptive parameters
#[derive(Debug, PartialEq)]
pub struct PositionAnalyzer;

impl PositionAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Analyze position and return detailed analysis
    pub fn analyze_position(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                           player: Player) -> PositionAnalysis {
        let material_balance = self.calculate_material_balance(board, captured_pieces, player);
        let tactical_potential = self.calculate_tactical_potential(board, player);
        let king_safety = self.calculate_king_safety(board, player);
        let is_quiet = self.is_quiet_position(board, player);
        let is_tactical = self.is_tactical_position(board, player);
        let complexity = self.calculate_position_complexity(board, captured_pieces);
        
        let position_type = self.classify_position_type(material_balance, tactical_potential, 
                                                       king_safety, is_quiet, is_tactical);
        
        PositionAnalysis {
            position_type,
            tactical_potential,
            material_balance,
            king_safety,
            is_quiet,
            is_tactical,
            complexity,
        }
    }
    
    /// Calculate material balance for the current player
    fn calculate_material_balance(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                                 player: Player) -> i32 {
        let mut balance = 0;
        
        // Calculate piece values on board
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    let value = match piece.piece_type {
                        PieceType::Pawn => 100,
                        PieceType::Lance => 300,
                        PieceType::Knight => 300,
                        PieceType::Silver => 400,
                        PieceType::Gold => 500,
                        PieceType::Bishop => 550,
                        PieceType::Rook => 650,
                        PieceType::King => 1000,
                        PieceType::PromotedPawn => 600,
                        PieceType::PromotedLance => 600,
                        PieceType::PromotedKnight => 600,
                        PieceType::PromotedSilver => 600,
                        PieceType::PromotedBishop => 650,
                        PieceType::PromotedRook => 750,
                    };
                    
                    if piece.player == player {
                        balance += value;
                    } else {
                        balance -= value;
                    }
                }
            }
        }
        
        // Add captured pieces value (simplified - could be enhanced with actual piece values)
        let player_captures = match player {
            Player::Black => &captured_pieces.black,
            Player::White => &captured_pieces.white,
        };
        let opponent_captures = match player {
            Player::Black => &captured_pieces.white,
            Player::White => &captured_pieces.black,
        };
        
        for piece_type in player_captures {
            let value = match piece_type {
                PieceType::Pawn => 100,
                PieceType::Lance => 300,
                PieceType::Knight => 300,
                PieceType::Silver => 400,
                PieceType::Gold => 500,
                PieceType::Bishop => 550,
                PieceType::Rook => 650,
                _ => 0,
            };
            balance += value;
        }
        
        for piece_type in opponent_captures {
            let value = match piece_type {
                PieceType::Pawn => 100,
                PieceType::Lance => 300,
                PieceType::Knight => 300,
                PieceType::Silver => 400,
                PieceType::Gold => 500,
                PieceType::Bishop => 550,
                PieceType::Rook => 650,
                _ => 0,
            };
            balance -= value;
        }
        
        balance
    }
    
    /// Calculate tactical potential (0-255)
    fn calculate_tactical_potential(&self, board: &BitboardBoard, player: Player) -> u8 {
        let mut potential = 0;
        
        // Check for pieces that can create tactical threats
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    if piece.player == player {
                        match piece.piece_type {
                            PieceType::Bishop | PieceType::PromotedBishop => potential += 30,
                            PieceType::Rook | PieceType::PromotedRook => potential += 35,
                            PieceType::Knight => potential += 20,
                            PieceType::Lance => potential += 15,
                            _ => potential += 5,
                        }
                    }
                }
            }
        }
        
        potential.min(255)
    }
    
    /// Calculate king safety (0-255, higher = safer)
    fn calculate_king_safety(&self, board: &BitboardBoard, player: Player) -> u8 {
        let mut safety = 100; // Base safety
        
        // Find king position
        for row in 0..9 {
            for col in 0..9 {
                if let Some(piece) = board.get_piece(Position::new(row, col)) {
                    if piece.player == player && piece.piece_type == PieceType::King {
                        // Check surrounding pieces for protection
                        for dr in -1..=1 {
                            for dc in -1..=1 {
                                if dr == 0 && dc == 0 { continue; }
                                let check_row = row as i32 + dr;
                                let check_col = col as i32 + dc;
                                if check_row >= 0 && check_row < 9 && check_col >= 0 && check_col < 9 {
                                    if let Some(protector) = board.get_piece(Position::new(check_row as u8, check_col as u8)) {
                                        if protector.player == player {
                                            safety += 10;
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        
        safety.min(255)
    }
    
    /// Check if position is quiet (no immediate tactical threats)
    fn is_quiet_position(&self, _board: &BitboardBoard, _player: Player) -> bool {
        // Simplified quiet position detection
        // In a real implementation, this would check for immediate captures, checks, etc.
        true // Placeholder
    }
    
    /// Check if position has tactical characteristics
    fn is_tactical_position(&self, board: &BitboardBoard, player: Player) -> bool {
        // Check for pieces in attacking positions
        let tactical_potential = self.calculate_tactical_potential(board, player);
        tactical_potential > 100
    }
    
    /// Calculate position complexity (0-255)
    fn calculate_position_complexity(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> u8 {
        let mut complexity = 0;
        
        // Count total pieces
        let mut piece_count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if board.get_piece(Position::new(row, col)).is_some() {
                    piece_count += 1;
                }
            }
        }
        
        // More pieces = more complexity
        complexity += (piece_count * 3).min(100);
        
        // Add captured pieces complexity
        let total_captures = captured_pieces.black.len() + captured_pieces.white.len();
        complexity += ((total_captures * 2) as u8).min(50);
        
        complexity.min(255)
    }
    
    /// Classify position type based on analysis
    fn classify_position_type(&self, material_balance: i32, tactical_potential: u8, 
                             king_safety: u8, is_quiet: bool, is_tactical: bool) -> PositionType {
        // Determine if this is an endgame
        if material_balance.abs() > 800 || tactical_potential < 50 {
            return PositionType::Endgame;
        }
        
        // Determine if this is tactical
        if is_tactical || tactical_potential > 150 || king_safety < 80 {
            return PositionType::Tactical;
        }
        
        // Determine if this is positional
        if is_quiet && tactical_potential < 100 && king_safety > 120 {
            return PositionType::Positional;
        }
        
        PositionType::Normal
    }
    
    /// Get parameter recommendations for specific position type
    pub fn get_parameter_recommendations(&self, position_type: PositionType) -> ParameterAdjustment {
        match position_type {
            PositionType::Tactical => ParameterAdjustment {
                futility_adjustment: -100,
                lmr_adjustment: 1,
                delta_adjustment: -50,
                razoring_adjustment: -200,
            },
            PositionType::Positional => ParameterAdjustment {
                futility_adjustment: 50,
                lmr_adjustment: 0,
                delta_adjustment: 25,
                razoring_adjustment: 100,
            },
            PositionType::Endgame => ParameterAdjustment {
                futility_adjustment: -200,
                lmr_adjustment: 2,
                delta_adjustment: -100,
                razoring_adjustment: -300,
            },
            PositionType::Normal => ParameterAdjustment {
                futility_adjustment: 0,
                lmr_adjustment: 0,
                delta_adjustment: 0,
                razoring_adjustment: 0,
            },
        }
    }
}

/// Position type for adaptive pruning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionType {
    Tactical,
    Positional,
    Endgame,
    Normal,
}

/// Position analysis for adaptive pruning
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    pub position_type: PositionType,
    pub tactical_potential: u8,
    pub material_balance: i32,
    pub king_safety: u8,
    pub is_quiet: bool,
    pub is_tactical: bool,
    pub complexity: u8,
}

/// Parameter adjustment for adaptive pruning
#[derive(Debug, Default, PartialEq)]
pub struct ParameterAdjustment {
    pub futility_adjustment: i32,
    pub lmr_adjustment: u8,
    pub delta_adjustment: i32,
    pub razoring_adjustment: i32,
}

/// Parameter snapshot for tracking changes
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct ParameterSnapshot {
    pub timestamp: std::time::SystemTime,
    pub parameters: PruningParameters,
    pub performance: PerformanceMetrics,
}

/// Parameter statistics for analysis and learning
#[derive(Debug, Default, PartialEq)]
pub struct ParameterStatistics {
    pub total_adjustments: usize,
    pub avg_cache_hit_ratio: f64,
    pub avg_futility_margin: f64,
    pub avg_lmr_reduction: f64,
    pub learning_rate: f64,
}

#[cfg(test)]
mod transposition_entry_tests {
    use super::*;
    
    #[test]
    fn test_transposition_entry_creation() {
        let entry = TranspositionEntry::new(
            100, 
            5, 
            TranspositionFlag::Exact, 
            None, 
            0x1234567890ABCDEF, 
            42
        );
        
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.flag, TranspositionFlag::Exact);
        assert!(entry.best_move.is_none());
        assert_eq!(entry.hash_key, 0x1234567890ABCDEF);
        assert_eq!(entry.age, 42);
    }
    
    #[test]
    fn test_transposition_entry_with_age() {
        let entry = TranspositionEntry::new_with_age(
            -50, 
            3, 
            TranspositionFlag::LowerBound, 
            None, 
            0xFEDCBA0987654321
        );
        
        assert_eq!(entry.score, -50);
        assert_eq!(entry.depth, 3);
        assert_eq!(entry.flag, TranspositionFlag::LowerBound);
        assert_eq!(entry.hash_key, 0xFEDCBA0987654321);
        assert_eq!(entry.age, 0); // Default age
    }
    
    #[test]
    fn test_is_valid_for_depth() {
        let entry = TranspositionEntry::new_with_age(
            0, 
            5, 
            TranspositionFlag::Exact, 
            None, 
            0x1234
        );
        
        // Entry depth 5 should be valid for required depth 5
        assert!(entry.is_valid_for_depth(5));
        // Entry depth 5 should be valid for required depth 4
        assert!(entry.is_valid_for_depth(4));
        // Entry depth 5 should NOT be valid for required depth 6
        assert!(!entry.is_valid_for_depth(6));
    }
    
    #[test]
    fn test_matches_hash() {
        let hash_key = 0x1234567890ABCDEF;
        let entry = TranspositionEntry::new_with_age(
            0, 
            0, 
            TranspositionFlag::Exact, 
            None, 
            hash_key
        );
        
        assert!(entry.matches_hash(hash_key));
        assert!(!entry.matches_hash(0xFEDCBA0987654321));
        assert!(!entry.matches_hash(0));
    }
    
    #[test]
    fn test_flag_checks() {
        let exact_entry = TranspositionEntry::new_with_age(
            0, 0, TranspositionFlag::Exact, None, 0x1234
        );
        let lower_entry = TranspositionEntry::new_with_age(
            0, 0, TranspositionFlag::LowerBound, None, 0x1234
        );
        let upper_entry = TranspositionEntry::new_with_age(
            0, 0, TranspositionFlag::UpperBound, None, 0x1234
        );
        
        assert!(exact_entry.is_exact());
        assert!(!exact_entry.is_lower_bound());
        assert!(!exact_entry.is_upper_bound());
        
        assert!(!lower_entry.is_exact());
        assert!(lower_entry.is_lower_bound());
        assert!(!lower_entry.is_upper_bound());
        
        assert!(!upper_entry.is_exact());
        assert!(!upper_entry.is_lower_bound());
        assert!(upper_entry.is_upper_bound());
    }
    
    #[test]
    fn test_age_management() {
        let mut entry = TranspositionEntry::new_with_age(
            0, 0, TranspositionFlag::Exact, None, 0x1234
        );
        
        assert_eq!(entry.age, 0);
        entry.update_age(100);
        assert_eq!(entry.age, 100);
        entry.update_age(0);
        assert_eq!(entry.age, 0);
    }
    
    #[test]
    fn test_memory_size() {
        let entry = TranspositionEntry::new_with_age(
            0, 0, TranspositionFlag::Exact, None, 0x1234
        );
        
        let size = entry.memory_size();
        assert!(size > 0);
        // Should be reasonable size (not too large)
        assert!(size < 1000);
    }
    
    #[test]
    fn test_debug_string() {
        let entry = TranspositionEntry::new(
            42, 
            3, 
            TranspositionFlag::Exact, 
            None, 
            0x1234567890ABCDEF, 
            10
        );
        
        let debug_str = entry.debug_string();
        assert!(debug_str.contains("score: 42"));
        assert!(debug_str.contains("depth: 3"));
        assert!(debug_str.contains("Exact"));
        assert!(debug_str.contains("best_move: None"));
        assert!(debug_str.contains("0x1234567890abcdef"));
        assert!(debug_str.contains("age: 10"));
    }
    
    #[test]
    fn test_debug_string_with_move() {
        let move_ = Move::new_move(
            Position::new(1, 1), 
            Position::new(2, 1), 
            PieceType::Pawn, 
            Player::Black, 
            false
        );
        
        let entry = TranspositionEntry::new(
            100, 
            5, 
            TranspositionFlag::LowerBound, 
            Some(move_), 
            0xFEDCBA0987654321, 
            20
        );
        
        let debug_str = entry.debug_string();
        assert!(debug_str.contains("score: 100"));
        assert!(debug_str.contains("depth: 5"));
        assert!(debug_str.contains("LowerBound"));
        assert!(debug_str.contains("best_move:"));
        assert!(debug_str.contains("0xfedcba0987654321"));
        assert!(debug_str.contains("age: 20"));
    }
    
    #[test]
    fn test_should_replace_with_hash_mismatch() {
        let entry1 = TranspositionEntry::new_with_age(
            0, 5, TranspositionFlag::Exact, None, 0x1111
        );
        let entry2 = TranspositionEntry::new_with_age(
            0, 3, TranspositionFlag::LowerBound, None, 0x2222
        );
        
        // Should replace due to hash mismatch (collision)
        assert!(entry1.should_replace_with(&entry2));
    }
    
    #[test]
    fn test_should_replace_with_depth() {
        let entry1 = TranspositionEntry::new_with_age(
            0, 3, TranspositionFlag::Exact, None, 0x1111
        );
        let entry2 = TranspositionEntry::new_with_age(
            0, 5, TranspositionFlag::LowerBound, None, 0x1111
        );
        
        // Should replace due to greater depth
        assert!(entry1.should_replace_with(&entry2));
    }
    
    #[test]
    fn test_should_replace_with_exact_flag() {
        let entry1 = TranspositionEntry::new_with_age(
            0, 5, TranspositionFlag::LowerBound, None, 0x1111
        );
        let entry2 = TranspositionEntry::new_with_age(
            0, 5, TranspositionFlag::Exact, None, 0x1111
        );
        
        // Should replace due to exact flag when depths are equal
        assert!(entry1.should_replace_with(&entry2));
    }
    
    #[test]
    fn test_should_replace_with_age() {
        let entry1 = TranspositionEntry::new(
            0, 5, TranspositionFlag::Exact, None, 0x1111, 10
        );
        let entry2 = TranspositionEntry::new(
            0, 5, TranspositionFlag::Exact, None, 0x1111, 20
        );
        
        // Should replace due to newer age when everything else is equal
        assert!(entry1.should_replace_with(&entry2));
    }
    
    #[test]
    fn test_should_not_replace() {
        let entry1 = TranspositionEntry::new(
            0, 5, TranspositionFlag::Exact, None, 0x1111, 20
        );
        let entry2 = TranspositionEntry::new(
            0, 3, TranspositionFlag::LowerBound, None, 0x1111, 10
        );
        
        // Should NOT replace - current entry is better in all aspects
        assert!(!entry1.should_replace_with(&entry2));
    }
    
    #[test]
    fn test_transposition_flag_to_string() {
        assert_eq!(TranspositionFlag::Exact.to_string(), "Exact");
        assert_eq!(TranspositionFlag::LowerBound.to_string(), "LowerBound");
        assert_eq!(TranspositionFlag::UpperBound.to_string(), "UpperBound");
    }
    
    #[test]
    fn test_transposition_entry_clone() {
        let original = TranspositionEntry::new(
            100, 
            5, 
            TranspositionFlag::Exact, 
            None, 
            0x1234567890ABCDEF, 
            42
        );
        
        let cloned = original.clone();
        
        assert_eq!(original.score, cloned.score);
        assert_eq!(original.depth, cloned.depth);
        assert_eq!(original.flag, cloned.flag);
        assert_eq!(original.best_move, cloned.best_move);
        assert_eq!(original.hash_key, cloned.hash_key);
        assert_eq!(original.age, cloned.age);
    }
    
    #[test]
    fn test_transposition_entry_with_best_move() {
        let move_ = Move::new_move(
            Position::new(0, 0), 
            Position::new(1, 1), 
            PieceType::King, 
            Player::White, 
            false
        );
        
        let entry = TranspositionEntry::new_with_age(
            150, 
            7, 
            TranspositionFlag::Exact, 
            Some(move_), 
            0xABCDEF1234567890
        );
        
        assert_eq!(entry.score, 150);
        assert_eq!(entry.depth, 7);
        assert!(entry.best_move.is_some());
        
        let best_move = entry.best_move.unwrap();
        assert_eq!(best_move.piece_type, PieceType::King);
        assert_eq!(best_move.player, Player::White);
        assert!(!best_move.is_promotion);
    }
    
    #[test]
    fn test_edge_cases() {
        // Test with maximum values
        let max_entry = TranspositionEntry::new(
            i32::MAX, 
            u8::MAX, 
            TranspositionFlag::Exact, 
            None, 
            u64::MAX, 
            u32::MAX
        );
        
        assert_eq!(max_entry.score, i32::MAX);
        assert_eq!(max_entry.depth, u8::MAX);
        assert_eq!(max_entry.hash_key, u64::MAX);
        assert_eq!(max_entry.age, u32::MAX);
        
        // Test with minimum values
        let min_entry = TranspositionEntry::new(
            i32::MIN, 
            0, 
            TranspositionFlag::UpperBound, 
            None, 
            0, 
            0
        );
        
        assert_eq!(min_entry.score, i32::MIN);
        assert_eq!(min_entry.depth, 0);
        assert_eq!(min_entry.hash_key, 0);
        assert_eq!(min_entry.age, 0);
    }
}
