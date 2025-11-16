//! Material Evaluation Module
//!
//! This module provides phase-aware material evaluation for the Shogi engine.
//! Material values differ between opening/middlegame and endgame phases, providing
//! more accurate position assessment throughout the game.
//!
//! # Overview
//!
//! The material evaluation system:
//! - Assigns different values to pieces in opening vs endgame
//! - Handles promoted pieces appropriately
//! - Evaluates captured pieces (pieces in hand)
//! - Calculates material balance for both players
//! - Integrates seamlessly with tapered evaluation
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::material::MaterialEvaluator;
//! use crate::types::{BitboardBoard, Player, CapturedPieces};
//!
//! let evaluator = MaterialEvaluator::new();
//! let board = BitboardBoard::new();
//! let captured_pieces = CapturedPieces::new();
//!
//! let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
//! ```

use crate::bitboards::BitboardBoard;
use crate::utils::telemetry::debug_log;
use crate::evaluation::material_value_loader::MaterialValueLoader;
use crate::types::board::CapturedPieces;
use crate::types::core::{PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;
use std::path::{Path, PathBuf};
use toml;

macro_rules! ts {
    ($mg:expr, $eg:expr) => {
        TaperedScore { mg: $mg, eg: $eg }
    };
}

#[cfg(feature = "material_fast_loop")]
const ALL_PIECE_TYPES: [PieceType; PieceType::COUNT] = [
    PieceType::Pawn,
    PieceType::Lance,
    PieceType::Knight,
    PieceType::Silver,
    PieceType::Gold,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::King,
    PieceType::PromotedPawn,
    PieceType::PromotedLance,
    PieceType::PromotedKnight,
    PieceType::PromotedSilver,
    PieceType::PromotedBishop,
    PieceType::PromotedRook,
];

#[cfg(feature = "material_fast_loop")]
const HAND_PIECE_TYPES: [PieceType; 7] = [
    PieceType::Pawn,
    PieceType::Lance,
    PieceType::Knight,
    PieceType::Silver,
    PieceType::Gold,
    PieceType::Bishop,
    PieceType::Rook,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialValueSet {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub board_values: [TaperedScore; PieceType::COUNT],
    #[serde(default)]
    pub hand_values: [Option<TaperedScore>; PieceType::COUNT],
}

impl MaterialValueSet {
    pub fn research() -> Self {
        Self::from_legacy(
            "research",
            "Research Value Set",
            Some("Internal tuning study"),
            Some("2024.10"),
            Some("2025-08-10"),
            Self::legacy_research_board(),
            Self::legacy_research_hand(),
        )
    }

    pub fn classic() -> Self {
        Self::from_legacy(
            "classic",
            "Classic Value Set",
            Some("Legacy engine defaults"),
            Some("2023.04"),
            Some("2024-07-15"),
            Self::legacy_classic_board(),
            Self::legacy_classic_hand(),
        )
    }

    fn from_legacy(
        id: &str,
        display_name: &str,
        source: Option<&str>,
        version: Option<&str>,
        last_updated: Option<&str>,
        board_values: [TaperedScore; PieceType::COUNT],
        hand_values: [Option<TaperedScore>; PieceType::COUNT],
    ) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            description: None,
            source: source.map(|s| s.to_string()),
            version: version.map(|s| s.to_string()),
            last_updated: last_updated.map(|s| s.to_string()),
            board_values,
            hand_values,
        }
    }

    #[inline]
    pub fn board_value(&self, piece_type: PieceType) -> TaperedScore {
        self.board_values[piece_type.as_index()]
    }

    #[inline]
    pub fn hand_value(&self, piece_type: PieceType) -> TaperedScore {
        self.hand_values[piece_type.as_index()].unwrap_or_else(|| self.board_value(piece_type))
    }

    pub fn validate(&self) -> Result<(), String> {
        for idx in 0..PieceType::COUNT {
            let piece = PieceType::from_u8(idx as u8);
            let board = self.board_values[idx];
            if board.mg == 0 && board.eg == 0 && piece != PieceType::King {
                return Err(format!(
                    "Board value missing for piece {:?} in value set {}",
                    piece, self.id
                ));
            }
        }
        Ok(())
    }

    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, MaterialValueSetError> {
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .map_err(|err| MaterialValueSetError::Io {
                path: PathBuf::from("<reader>"),
                message: err.to_string(),
            })?;
        Self::from_json_bytes(&data)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, MaterialValueSetError> {
        let path_ref = path.as_ref();
        let data = std::fs::read(path_ref).map_err(|err| MaterialValueSetError::Io {
            path: path_ref.to_path_buf(),
            message: err.to_string(),
        })?;
        let extension = path_ref
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match extension.as_str() {
            "json" => Self::from_json_bytes(&data),
            "toml" => Self::from_toml_bytes(&data),
            "" => Err(MaterialValueSetError::UnsupportedFormat {
                extension: "<missing>".to_string(),
            }),
            other => Err(MaterialValueSetError::UnsupportedFormat {
                extension: other.to_string(),
            }),
        }
    }

    pub fn to_writer<W: std::io::Write>(&self, writer: W) -> Result<(), MaterialValueSetError> {
        serde_json::to_writer_pretty(writer, self)
            .map_err(|err| MaterialValueSetError::Serialize(err.to_string()))
    }

    fn from_json_bytes(bytes: &[u8]) -> Result<Self, MaterialValueSetError> {
        let value_set: MaterialValueSet = serde_json::from_slice(bytes)
            .map_err(|err| MaterialValueSetError::Parse(err.to_string()))?;
        value_set
            .validate()
            .map_err(MaterialValueSetError::Validation)?;
        Ok(value_set)
    }

    fn from_toml_bytes(bytes: &[u8]) -> Result<Self, MaterialValueSetError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|err| MaterialValueSetError::Parse(err.to_string()))?;
        let value_set: MaterialValueSet =
            toml::from_str(text).map_err(|err| MaterialValueSetError::Parse(err.to_string()))?;
        value_set
            .validate()
            .map_err(MaterialValueSetError::Validation)?;
        Ok(value_set)
    }

    pub fn legacy_research_board() -> [TaperedScore; PieceType::COUNT] {
        [
            ts!(100, 120),
            ts!(300, 280),
            ts!(350, 320),
            ts!(450, 460),
            ts!(500, 520),
            ts!(800, 850),
            ts!(1000, 1100),
            ts!(20000, 20000),
            ts!(500, 550),
            ts!(500, 540),
            ts!(520, 550),
            ts!(520, 550),
            ts!(1200, 1300),
            ts!(1400, 1550),
        ]
    }

    pub fn legacy_research_hand() -> [Option<TaperedScore>; PieceType::COUNT] {
        [
            Some(ts!(110, 130)),
            Some(ts!(320, 300)),
            Some(ts!(370, 350)),
            Some(ts!(480, 490)),
            Some(ts!(530, 550)),
            Some(ts!(850, 920)),
            Some(ts!(1050, 1180)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    }

    pub fn legacy_classic_board() -> [TaperedScore; PieceType::COUNT] {
        [
            ts!(100, 110),
            ts!(280, 300),
            ts!(320, 330),
            ts!(430, 440),
            ts!(500, 500),
            ts!(780, 820),
            ts!(950, 1020),
            ts!(20000, 20000),
            ts!(480, 520),
            ts!(480, 520),
            ts!(500, 530),
            ts!(500, 530),
            ts!(1150, 1220),
            ts!(1320, 1450),
        ]
    }

    pub fn legacy_classic_hand() -> [Option<TaperedScore>; PieceType::COUNT] {
        [
            Some(ts!(105, 115)),
            Some(ts!(300, 310)),
            Some(ts!(340, 350)),
            Some(ts!(450, 460)),
            Some(ts!(520, 520)),
            Some(ts!(820, 860)),
            Some(ts!(990, 1080)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MaterialValueSetError {
    #[error("Unsupported material value file format: `{extension}`")]
    UnsupportedFormat { extension: String },
    #[error("Unable to read material value file `{path:?}`: {message}")]
    Io {
        path: std::path::PathBuf,
        message: String,
    },
    #[error("Failed to parse material value set: {0}")]
    Parse(String),
    #[error("Invalid material value set: {0}")]
    Validation(String),
    #[error("Failed to serialize material value set: {0}")]
    Serialize(String),
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AggregateScore {
    pub mg: i64,
    pub eg: i64,
}

impl AggregateScore {
    #[inline]
    fn add_tapered(&mut self, value: TaperedScore, sign: i64) {
        self.mg += value.mg as i64 * sign;
        self.eg += value.eg as i64 * sign;
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MaterialPresetUsage {
    pub research: u64,
    pub classic: u64,
    pub custom: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialTelemetry {
    pub evaluations: u64,
    pub board_contributions: Vec<AggregateScore>,
    pub hand_contributions: Vec<AggregateScore>,
    pub hand_balance: AggregateScore,
    pub phase_weighted_total: i64,
    pub preset_usage: MaterialPresetUsage,
}

/// Represents incremental changes to material state for optimized updates.
#[derive(Debug, Clone, Default)]
pub struct MaterialDelta {
    board_deltas: [i32; PieceType::COUNT],
    hand_deltas: [i32; PieceType::COUNT],
}

impl MaterialDelta {
    pub fn add_board(&mut self, piece_type: PieceType, delta: i32) {
        let idx = piece_type.as_index();
        self.board_deltas[idx] += delta;
    }

    pub fn add_hand(&mut self, piece_type: PieceType, delta: i32) {
        let idx = piece_type.as_index();
        self.hand_deltas[idx] += delta;
    }

    pub fn board_delta(&self, piece_type: PieceType) -> i32 {
        self.board_deltas[piece_type.as_index()]
    }

    pub fn hand_delta(&self, piece_type: PieceType) -> i32 {
        self.hand_deltas[piece_type.as_index()]
    }

    pub fn clear(&mut self) {
        self.board_deltas = [0; PieceType::COUNT];
        self.hand_deltas = [0; PieceType::COUNT];
    }
}

#[derive(Debug, Clone)]
struct MaterialContribution {
    board: [AggregateScore; PieceType::COUNT],
    hand: [AggregateScore; PieceType::COUNT],
    hand_balance: AggregateScore,
}

impl Default for MaterialContribution {
    fn default() -> Self {
        Self {
            board: [AggregateScore::default(); PieceType::COUNT],
            hand: [AggregateScore::default(); PieceType::COUNT],
            hand_balance: AggregateScore::default(),
        }
    }
}

impl MaterialContribution {
    fn add_board(&mut self, piece_type: PieceType, value: TaperedScore, is_player: bool) {
        let sign = if is_player { 1 } else { -1 };
        self.board[piece_type.as_index()].add_tapered(value, sign);
    }

    fn add_hand(&mut self, piece_type: PieceType, value: TaperedScore, is_player: bool) {
        let sign = if is_player { 1 } else { -1 };
        self.hand[piece_type.as_index()].add_tapered(value, sign);
        self.hand_balance.add_tapered(value, sign);
    }
}

/// Material evaluator with phase-aware piece values
pub struct MaterialEvaluator {
    /// Configuration for material evaluation
    config: MaterialEvaluationConfig,
    /// Statistics for monitoring
    stats: MaterialEvaluationStats,
    /// Active material value set
    value_set: MaterialValueSet,
}

impl MaterialEvaluator {
    fn select_value_set(config: &MaterialEvaluationConfig) -> MaterialValueSet {
        MaterialValueLoader::load(config).unwrap_or_else(|err| {
            debug_log(&format!(
                "[MaterialEvaluator] Failed to load external value set: {}. Falling back to {} preset.",
                err,
                if config.use_research_values {
                    "research"
                } else {
                    "classic"
                }
            ));
        if config.use_research_values {
                MaterialValueSet::research()
            } else {
                MaterialValueSet::classic()
            }
        })
    }

    /// Create a new MaterialEvaluator with default configuration
    pub fn new() -> Self {
        Self::with_config(MaterialEvaluationConfig::default())
    }

    /// Create a new MaterialEvaluator with custom configuration
    pub fn with_config(config: MaterialEvaluationConfig) -> Self {
        let value_set = Self::select_value_set(&config);
        Self {
            config,
            stats: MaterialEvaluationStats::default(),
            value_set,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MaterialEvaluationConfig {
        &self.config
    }

    /// Apply a new configuration, rebuilding value tables and resetting statistics.
    pub fn apply_config(&mut self, config: MaterialEvaluationConfig) {
        self.config = config;
        self.value_set = Self::select_value_set(&self.config);
        self.reset_stats();
    }

    /// Get the currently loaded material value set
    pub fn value_set(&self) -> &MaterialValueSet {
        &self.value_set
    }

    /// Evaluate material for a player
    ///
    /// Returns a TaperedScore with middlegame and endgame material values
    pub fn evaluate_material(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> TaperedScore {
        self.stats.register_value_set(&self.value_set);
        let mut contribution = MaterialContribution::default();
        let mut score = TaperedScore::default();

        // Evaluate pieces on board
        score += self.evaluate_board_material(board, player, &mut contribution);

        // Evaluate captured pieces (pieces in hand)
        if self.config.include_hand_pieces {
            score += self.evaluate_hand_material(captured_pieces, player, &mut contribution);
        }

        self.stats.record_contribution(&contribution);
        score
    }

    /// Compute a tapered score delta for incremental updates.
    pub fn evaluate_delta(&self, delta: &MaterialDelta) -> TaperedScore {
        let mut score = TaperedScore::default();
        for idx in 0..PieceType::COUNT {
            let board_delta = delta.board_deltas[idx];
            if board_delta != 0 {
                let piece_type = PieceType::from_u8(idx as u8);
                let value = self.get_piece_value(piece_type);
                score.mg += value.mg * board_delta;
                score.eg += value.eg * board_delta;
            }

            let hand_delta = delta.hand_deltas[idx];
            if hand_delta != 0 {
                let piece_type = PieceType::from_u8(idx as u8);
                let value = self.get_hand_piece_value(piece_type);
                score.mg += value.mg * hand_delta;
                score.eg += value.eg * hand_delta;
            }
        }
        score
    }

    /// Apply an incremental delta to an existing score, updating statistics.
    pub fn apply_delta(
        &mut self,
        current_score: TaperedScore,
        delta: &MaterialDelta,
    ) -> TaperedScore {
        let delta_score = self.evaluate_delta(delta);
        let mut updated = current_score;
        updated += delta_score;

        self.stats.register_value_set(&self.value_set);
        self.stats.apply_delta(delta, &self.value_set);
        self.stats.record_phase_weighted(updated.mg);

        updated
    }

    /// Evaluate material for pieces on the board
    fn evaluate_board_material(
        &self,
        board: &BitboardBoard,
        player: Player,
        contribution: &mut MaterialContribution,
    ) -> TaperedScore {
        #[cfg(feature = "material_fast_loop")]
        if self.config.enable_fast_loop {
            return self.evaluate_board_material_fast(board, player, contribution);
        }

        let mut score = TaperedScore::default();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value = self.get_piece_value(piece.piece_type);

                    if piece.player == player {
                        contribution.add_board(piece.piece_type, piece_value, true);
                        score += piece_value;
                    } else {
                        contribution.add_board(piece.piece_type, piece_value, false);
                        score -= piece_value;
                    }
                }
            }
        }

        score
    }

    #[cfg(feature = "material_fast_loop")]
    fn evaluate_board_material_fast(
        &self,
        board: &BitboardBoard,
        player: Player,
        contribution: &mut MaterialContribution,
    ) -> TaperedScore {
        let mut score = TaperedScore::default();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let opponent_idx = 1 - player_idx;
        let pieces = board.get_pieces();

        for piece_type in ALL_PIECE_TYPES.iter().copied() {
            let idx = piece_type.as_index();
            let bitboard_self = pieces[player_idx][idx];
            let bitboard_opponent = pieces[opponent_idx][idx];
            if bitboard_self == 0 && bitboard_opponent == 0 {
                continue;
            }
            let value = self.get_piece_value(piece_type);
            let player_count = bitboard_self.count_ones() as i64;
            let opponent_count = bitboard_opponent.count_ones() as i64;
            if player_count > 0 {
                contribution.board[idx].add_tapered(value, player_count);
                score += TaperedScore::new_tapered(
                    value.mg * player_count as i32,
                    value.eg * player_count as i32,
                );
            }
            if opponent_count > 0 {
                contribution.board[idx].add_tapered(value, -opponent_count);
                score -= TaperedScore::new_tapered(
                    value.mg * opponent_count as i32,
                    value.eg * opponent_count as i32,
                );
            }
        }

        score
    }

    /// Evaluate material for captured pieces (pieces in hand)
    fn evaluate_hand_material(
        &self,
        captured_pieces: &CapturedPieces,
        player: Player,
        contribution: &mut MaterialContribution,
    ) -> TaperedScore {
        #[cfg(feature = "material_fast_loop")]
        if self.config.enable_fast_loop {
            return self.evaluate_hand_material_fast(captured_pieces, player, contribution);
        }

        let mut score = TaperedScore::default();

        // Get captured pieces for this player
        let player_captures = match player {
            Player::Black => &captured_pieces.black,
            Player::White => &captured_pieces.white,
        };

        // Get opponent's captured pieces
        let opponent_captures = match player {
            Player::Black => &captured_pieces.white,
            Player::White => &captured_pieces.black,
        };

        // Add value for pieces we can drop
        for &piece_type in player_captures {
            let value = self.get_hand_piece_value(piece_type);
            contribution.add_hand(piece_type, value, true);
            score += value;
        }

        // Subtract value for pieces opponent can drop
        for &piece_type in opponent_captures {
            let value = self.get_hand_piece_value(piece_type);
            contribution.add_hand(piece_type, value, false);
            score -= value;
        }

        score
    }

    #[cfg(feature = "material_fast_loop")]
    fn evaluate_hand_material_fast(
        &self,
        captured_pieces: &CapturedPieces,
        player: Player,
        contribution: &mut MaterialContribution,
    ) -> TaperedScore {
        let mut score = TaperedScore::default();
        for piece_type in HAND_PIECE_TYPES.iter().copied() {
            let value = self.get_hand_piece_value(piece_type);
            let player_count = captured_pieces.count(piece_type, player) as i64;
            let opponent_count = captured_pieces.count(piece_type, player.opposite()) as i64;

            if player_count > 0 {
                contribution.add_hand(piece_type, value, true);
                score += TaperedScore::new_tapered(
                    value.mg * player_count as i32,
                    value.eg * player_count as i32,
                );
            }
            if opponent_count > 0 {
                contribution.add_hand(piece_type, value, false);
                score -= TaperedScore::new_tapered(
                    value.mg * opponent_count as i32,
                    value.eg * opponent_count as i32,
                );
            }
        }
        score
    }

    /// Get tapered value for a piece on the board
    ///
    /// Returns a TaperedScore with separate mg/eg values
    pub fn get_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        self.value_set.board_value(piece_type)
    }

    /// Get tapered value for a piece in hand
    ///
    /// Hand pieces are generally more valuable than board pieces
    /// because they can be dropped anywhere (with restrictions)
    pub fn get_hand_piece_value(&self, piece_type: PieceType) -> TaperedScore {
        self.value_set.hand_value(piece_type)
    }

    /// Calculate material balance for a player
    ///
    /// Positive value means the player has more material
    /// Negative value means the opponent has more material
    pub fn calculate_material_balance(
        &mut self,
        board: &BitboardBoard,
        captured_pieces: &CapturedPieces,
        player: Player,
    ) -> TaperedScore {
        self.evaluate_material(board, player, captured_pieces)
    }

    /// Count total material on board (both players)
    pub fn count_total_material(&self, board: &BitboardBoard) -> i32 {
        let mut total = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type != PieceType::King {
                        // Use middlegame value as base
                        total += self.get_piece_value(piece.piece_type).mg;
                    }
                }
            }
        }

        total
    }

    /// Count material by piece type
    pub fn count_material_by_type(
        &self,
        board: &BitboardBoard,
        piece_type: PieceType,
        player: Player,
    ) -> i32 {
        let mut count = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == piece_type && piece.player == player {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Get evaluation statistics
    pub fn stats(&self) -> &MaterialEvaluationStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut MaterialEvaluationStats {
        &mut self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = MaterialEvaluationStats::default();
    }
}

impl Default for MaterialEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for material evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialEvaluationConfig {
    /// Include hand pieces (captured pieces) in evaluation
    pub include_hand_pieces: bool,
    /// Use research-based values vs classic values
    pub use_research_values: bool,
    /// Optional path to a custom material value set (JSON/TOML)
    pub values_path: Option<String>,
    /// Enable optimized fast-loop traversal for board/hand evaluation
    #[serde(default)]
    pub enable_fast_loop: bool,
}

impl Default for MaterialEvaluationConfig {
    fn default() -> Self {
        Self {
            include_hand_pieces: true,
            use_research_values: true,
            values_path: None,
            enable_fast_loop: false,
        }
    }
}

/// Statistics for monitoring material evaluation
#[derive(Debug, Clone)]
pub struct MaterialEvaluationStats {
    pub evaluations: u64,
    board_contributions: [AggregateScore; PieceType::COUNT],
    hand_contributions: [AggregateScore; PieceType::COUNT],
    hand_balance: AggregateScore,
    phase_weighted_total: i64,
    preset_usage: MaterialPresetUsage,
}

impl MaterialEvaluationStats {
    pub fn new() -> Self {
        Self {
            evaluations: 0,
            board_contributions: [AggregateScore::default(); PieceType::COUNT],
            hand_contributions: [AggregateScore::default(); PieceType::COUNT],
            hand_balance: AggregateScore::default(),
            phase_weighted_total: 0,
            preset_usage: MaterialPresetUsage::default(),
        }
    }

    fn register_value_set(&mut self, value_set: &MaterialValueSet) {
        match value_set.id.as_str() {
            "research" => self.preset_usage.research += 1,
            "classic" => self.preset_usage.classic += 1,
            _ => self.preset_usage.custom += 1,
        }
    }

    fn record_contribution(&mut self, contribution: &MaterialContribution) {
        self.evaluations += 1;
        for idx in 0..PieceType::COUNT {
            self.board_contributions[idx].mg += contribution.board[idx].mg;
            self.board_contributions[idx].eg += contribution.board[idx].eg;
            self.hand_contributions[idx].mg += contribution.hand[idx].mg;
            self.hand_contributions[idx].eg += contribution.hand[idx].eg;
        }
        self.hand_balance.mg += contribution.hand_balance.mg;
        self.hand_balance.eg += contribution.hand_balance.eg;
    }

    pub(crate) fn record_phase_weighted(&mut self, score: i32) {
        self.phase_weighted_total += score as i64;
    }

    fn apply_delta(&mut self, delta: &MaterialDelta, value_set: &MaterialValueSet) {
        self.evaluations += 1;
        for idx in 0..PieceType::COUNT {
            let board_delta = delta.board_deltas[idx];
            if board_delta != 0 {
                let piece_type = PieceType::from_u8(idx as u8);
                let value = value_set.board_value(piece_type);
                self.board_contributions[idx].add_tapered(value, board_delta as i64);
            }

            let hand_delta = delta.hand_deltas[idx];
            if hand_delta != 0 {
                let piece_type = PieceType::from_u8(idx as u8);
                let value = value_set.hand_value(piece_type);
                self.hand_contributions[idx].add_tapered(value, hand_delta as i64);
                self.hand_balance.add_tapered(value, hand_delta as i64);
            }
        }
    }

    pub fn board_contribution(&self, piece_type: PieceType) -> AggregateScore {
        self.board_contributions[piece_type.as_index()]
    }

    pub fn hand_contribution(&self, piece_type: PieceType) -> AggregateScore {
        self.hand_contributions[piece_type.as_index()]
    }

    pub fn hand_balance(&self) -> AggregateScore {
        self.hand_balance
    }

    pub fn phase_weighted_total(&self) -> i64 {
        self.phase_weighted_total
    }

    pub fn preset_usage(&self) -> MaterialPresetUsage {
        self.preset_usage
    }

    pub fn snapshot(&self) -> MaterialTelemetry {
        MaterialTelemetry {
            evaluations: self.evaluations,
            board_contributions: self.board_contributions.to_vec(),
            hand_contributions: self.hand_contributions.to_vec(),
            hand_balance: self.hand_balance,
            phase_weighted_total: self.phase_weighted_total,
            preset_usage: self.preset_usage,
        }
    }
}

impl Default for MaterialEvaluationStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Piece, Position};
    use tempfile::NamedTempFile;

    #[test]
    fn test_material_evaluator_creation() {
        let evaluator = MaterialEvaluator::new();
        assert!(evaluator.config().include_hand_pieces);
    }

    #[test]
    fn test_material_evaluator_with_config() {
        let config = MaterialEvaluationConfig {
            include_hand_pieces: false,
            use_research_values: false,
            ..MaterialEvaluationConfig::default()
        };
        let evaluator = MaterialEvaluator::with_config(config);
        assert!(!evaluator.config().include_hand_pieces);
        assert_eq!(evaluator.value_set().id, "classic");
    }

    #[test]
    fn test_piece_values_basic() {
        let evaluator = MaterialEvaluator::new();

        // Test basic pieces
        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        assert_eq!(pawn.mg, 100);
        assert_eq!(pawn.eg, 120);

        let rook = evaluator.get_piece_value(PieceType::Rook);
        assert_eq!(rook.mg, 1000);
        assert_eq!(rook.eg, 1100);

        let king = evaluator.get_piece_value(PieceType::King);
        assert_eq!(king.mg, 20000);
        assert_eq!(king.eg, 20000);
    }

    #[test]
    fn test_piece_values_promoted() {
        let evaluator = MaterialEvaluator::new();

        let promoted_pawn = evaluator.get_piece_value(PieceType::PromotedPawn);
        assert_eq!(promoted_pawn.mg, 500);
        assert_eq!(promoted_pawn.eg, 550);

        let promoted_rook = evaluator.get_piece_value(PieceType::PromotedRook);
        assert_eq!(promoted_rook.mg, 1400);
        assert_eq!(promoted_rook.eg, 1550);
    }

    #[test]
    fn test_hand_piece_values() {
        let evaluator = MaterialEvaluator::new();

        // Hand pieces should be slightly more valuable
        let board_pawn = evaluator.get_piece_value(PieceType::Pawn);
        let hand_pawn = evaluator.get_hand_piece_value(PieceType::Pawn);
        assert!(hand_pawn.mg > board_pawn.mg);
        assert!(hand_pawn.eg > board_pawn.eg);

        // Promoted pieces fall back to board values when not explicitly provided
        let hand_promoted = evaluator.get_hand_piece_value(PieceType::PromotedPawn);
        let board_promoted = evaluator.get_piece_value(PieceType::PromotedPawn);
        assert_eq!(hand_promoted, board_promoted);
    }

    #[test]
    fn test_evaluate_starting_position() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Starting position should be balanced (both players have equal material)
        let black_score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(black_score.mg, 0);
        assert_eq!(black_score.eg, 0);

        let white_score = evaluator.evaluate_material(&board, Player::White, &captured_pieces);
        assert_eq!(white_score.mg, 0);
        assert_eq!(white_score.eg, 0);
    }

    #[test]
    fn test_evaluate_with_captures() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a captured pawn for Black
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);

        let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        // Black should have extra value from the captured pawn
        let hand_pawn_value = evaluator.get_hand_piece_value(PieceType::Pawn);
        assert_eq!(score.mg, hand_pawn_value.mg);
        assert_eq!(score.eg, hand_pawn_value.eg);
    }

    #[test]
    fn test_evaluate_without_hand_pieces() {
        let config = MaterialEvaluationConfig {
            include_hand_pieces: false,
            ..MaterialEvaluationConfig::default()
        };
        let mut evaluator = MaterialEvaluator::with_config(config);
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a captured pawn
        captured_pieces.add_piece(PieceType::Pawn, Player::Black);

        let score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        // Hand pieces should not be counted
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_material_balance() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let balance = evaluator.calculate_material_balance(&board, &captured_pieces, Player::Black);

        // Starting position should have zero balance
        assert_eq!(balance.mg, 0);
        assert_eq!(balance.eg, 0);
    }

    #[test]
    fn test_value_sets_change_piece_values() {
        let research_eval = MaterialEvaluator::new();
        let classic_eval = MaterialEvaluator::with_config(MaterialEvaluationConfig {
            use_research_values: false,
            ..MaterialEvaluationConfig::default()
        });

        let research_rook = research_eval.get_piece_value(PieceType::Rook);
        let classic_rook = classic_eval.get_piece_value(PieceType::Rook);

        assert_ne!(
            (research_rook.mg, research_rook.eg),
            (classic_rook.mg, classic_rook.eg)
        );
    }

    #[test]
    fn test_value_set_toggle_affects_evaluation() {
        let mut board = BitboardBoard::empty();
        let position = Position::new(4, 4);
        board.place_piece(Piece::new(PieceType::Rook, Player::Black), position);

        let config_research = MaterialEvaluationConfig::default();
        let config_classic = MaterialEvaluationConfig {
            use_research_values: false,
            ..MaterialEvaluationConfig::default()
        };

        let mut research_eval = MaterialEvaluator::with_config(config_research);
        let mut classic_eval = MaterialEvaluator::with_config(config_classic);
        let captured = CapturedPieces::new();

        let research_score = research_eval.evaluate_material(&board, Player::Black, &captured);
        let classic_score = classic_eval.evaluate_material(&board, Player::Black, &captured);

        assert_ne!(
            (research_score.mg, research_score.eg),
            (classic_score.mg, classic_score.eg)
        );
    }

    #[test]
    fn test_material_stats_track_contributions() {
        let mut evaluator = MaterialEvaluator::new();
        let mut board = BitboardBoard::empty();
        board.place_piece(
            Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 4),
        );
        board.place_piece(
            Piece::new(PieceType::Bishop, Player::White),
            Position::new(4, 5),
        );

        let mut captured = CapturedPieces::new();
        captured.add_piece(PieceType::Pawn, Player::Black);
        captured.add_piece(PieceType::Silver, Player::White);

        evaluator.evaluate_material(&board, Player::Black, &captured);

        let stats = evaluator.stats();
        assert_eq!(stats.evaluations, 1);

        let rook_value = evaluator.get_piece_value(PieceType::Rook);
        let rook_contrib = stats.board_contribution(PieceType::Rook);
        assert_eq!(rook_contrib.mg, rook_value.mg as i64);
        assert_eq!(rook_contrib.eg, rook_value.eg as i64);

        let bishop_value = evaluator.get_piece_value(PieceType::Bishop);
        let bishop_contrib = stats.board_contribution(PieceType::Bishop);
        assert_eq!(bishop_contrib.mg, -(bishop_value.mg as i64));
        assert_eq!(bishop_contrib.eg, -(bishop_value.eg as i64));

        let pawn_value = evaluator.get_hand_piece_value(PieceType::Pawn);
        let pawn_contrib = stats.hand_contribution(PieceType::Pawn);
        assert_eq!(pawn_contrib.mg, pawn_value.mg as i64);
        assert_eq!(pawn_contrib.eg, pawn_value.eg as i64);

        let silver_value = evaluator.get_hand_piece_value(PieceType::Silver);
        let silver_contrib = stats.hand_contribution(PieceType::Silver);
        assert_eq!(silver_contrib.mg, -(silver_value.mg as i64));
        assert_eq!(silver_contrib.eg, -(silver_value.eg as i64));

        let hand_balance = stats.hand_balance();
        assert_eq!(
            hand_balance.mg,
            pawn_value.mg as i64 - silver_value.mg as i64
        );
        assert_eq!(
            hand_balance.eg,
            pawn_value.eg as i64 - silver_value.eg as i64
        );
    }

    #[test]
    fn test_material_preset_usage_tracking() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured = CapturedPieces::new();
        evaluator.evaluate_material(&board, Player::Black, &captured);
        let preset_usage = evaluator.stats().preset_usage();
        assert_eq!(preset_usage.research, 1);
        assert_eq!(preset_usage.classic, 0);
        assert_eq!(preset_usage.custom, 0);

        let mut custom_set = MaterialValueSet::classic();
        custom_set.id = "custom".into();
        custom_set.board_values[PieceType::Pawn.as_index()].mg += 10;
        let mut temp_file = NamedTempFile::new().expect("temp material file");
        custom_set
            .to_writer(&mut temp_file)
            .expect("write custom material file");

        let mut config = MaterialEvaluationConfig::default();
        config.use_research_values = true;
        config.values_path = Some(temp_file.path().to_string_lossy().to_string());
        let mut custom_eval = MaterialEvaluator::with_config(config);
        custom_eval.evaluate_material(&board, Player::Black, &captured);
        let usage = custom_eval.stats().preset_usage();
        assert_eq!(usage.custom, 1);
    }

    #[test]
    fn test_material_delta_matches_full_evaluation() {
        let mut base_board = BitboardBoard::empty();
        base_board.place_piece(
            Piece::new(PieceType::King, Player::Black),
            Position::new(8, 4),
        );
        base_board.place_piece(
            Piece::new(PieceType::King, Player::White),
            Position::new(0, 4),
        );
        let base_captured = CapturedPieces::new();

        let mut full_evaluator = MaterialEvaluator::new();
        let _ = full_evaluator.evaluate_material(&base_board, Player::Black, &base_captured);

        let mut updated_board = base_board.clone();
        updated_board.place_piece(
            Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 4),
        );
        updated_board.place_piece(
            Piece::new(PieceType::Silver, Player::White),
            Position::new(3, 3),
        );

        let mut updated_captured = CapturedPieces::new();
        updated_captured.add_piece(PieceType::Pawn, Player::Black);
        updated_captured.add_piece(PieceType::Bishop, Player::White);

        let updated_full =
            full_evaluator.evaluate_material(&updated_board, Player::Black, &updated_captured);

        let mut delta = MaterialDelta::default();
        delta.add_board(PieceType::Rook, 1);
        delta.add_board(PieceType::Silver, -1);
        delta.add_hand(PieceType::Pawn, 1);
        delta.add_hand(PieceType::Bishop, -1);

        let mut incremental_evaluator = MaterialEvaluator::new();
        let base_score =
            incremental_evaluator.evaluate_material(&base_board, Player::Black, &base_captured);
        let via_delta = incremental_evaluator.apply_delta(base_score, &delta);

        assert_eq!(updated_full, via_delta);
    }

    #[cfg(feature = "material_fast_loop")]
    #[test]
    fn test_fast_loop_matches_legacy_path() {
        let mut board = BitboardBoard::empty();
        for (piece_type, player, row, col) in [
            (PieceType::Rook, Player::Black, 4, 4),
            (PieceType::Bishop, Player::Black, 5, 5),
            (PieceType::Silver, Player::White, 2, 2),
            (PieceType::Gold, Player::White, 3, 6),
        ] {
            board.place_piece(Piece::new(piece_type, player), Position::new(row, col));
        }

        let mut captured = CapturedPieces::new();
        captured.add_piece(PieceType::Pawn, Player::Black);
        captured.add_piece(PieceType::Knight, Player::White);

        let slow_score =
            MaterialEvaluator::new().evaluate_material(&board, Player::Black, &captured);

        let mut fast_config = MaterialEvaluationConfig::default();
        fast_config.enable_fast_loop = true;
        let fast_score = MaterialEvaluator::with_config(fast_config).evaluate_material(
            &board,
            Player::Black,
            &captured,
        );

        assert_eq!(slow_score, fast_score);
    }

    #[test]
    fn test_count_total_material() {
        let evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();

        let total = evaluator.count_total_material(&board);

        // Starting position should have significant material (excluding kings)
        assert!(total > 10000); // Both players have material
    }

    #[test]
    fn test_count_material_by_type() {
        let evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();

        // Starting position has 9 pawns per player
        let pawn_count = evaluator.count_material_by_type(&board, PieceType::Pawn, Player::Black);
        assert_eq!(pawn_count, 9);

        // Starting position has 1 king per player
        let king_count = evaluator.count_material_by_type(&board, PieceType::King, Player::Black);
        assert_eq!(king_count, 1);

        // Starting position has 2 rooks per player
        let rook_count = evaluator.count_material_by_type(&board, PieceType::Rook, Player::Black);
        assert_eq!(rook_count, 1);
    }

    #[test]
    fn test_endgame_values_higher() {
        let evaluator = MaterialEvaluator::new();

        // Most pieces should be more valuable in endgame
        let rook = evaluator.get_piece_value(PieceType::Rook);
        assert!(rook.eg > rook.mg, "Rook should be more valuable in endgame");

        let bishop = evaluator.get_piece_value(PieceType::Bishop);
        assert!(
            bishop.eg > bishop.mg,
            "Bishop should be more valuable in endgame"
        );

        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        assert!(pawn.eg > pawn.mg, "Pawn should be more valuable in endgame");
    }

    #[test]
    fn test_promoted_pieces_more_valuable() {
        let evaluator = MaterialEvaluator::new();

        // Promoted pieces should be more valuable than unpromoted
        let pawn = evaluator.get_piece_value(PieceType::Pawn);
        let promoted_pawn = evaluator.get_piece_value(PieceType::PromotedPawn);
        assert!(promoted_pawn.mg > pawn.mg);
        assert!(promoted_pawn.eg > pawn.eg);

        let rook = evaluator.get_piece_value(PieceType::Rook);
        let promoted_rook = evaluator.get_piece_value(PieceType::PromotedRook);
        assert!(promoted_rook.mg > rook.mg);
        assert!(promoted_rook.eg > rook.eg);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        assert_eq!(evaluator.stats().evaluations, 0);

        evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);

        evaluator.evaluate_material(&board, Player::White, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 2);
    }

    #[test]
    fn test_reset_statistics() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);

        evaluator.reset_stats();
        assert_eq!(evaluator.stats().evaluations, 0);
    }

    #[test]
    fn test_evaluation_consistency() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Multiple evaluations should return the same result
        let score1 = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);

        assert_eq!(score1.mg, score2.mg);
        assert_eq!(score1.eg, score2.eg);
    }

    #[test]
    fn test_apply_config_resets_stats_and_updates_values() {
        let mut evaluator = MaterialEvaluator::with_config(MaterialEvaluationConfig::default());

        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        assert_eq!(evaluator.stats().evaluations, 1);

        let research_rook = evaluator.get_piece_value(PieceType::Rook);

        evaluator.apply_config(MaterialEvaluationConfig {
            include_hand_pieces: false,
            use_research_values: false,
            ..MaterialEvaluationConfig::default()
        });

        assert_eq!(evaluator.stats().evaluations, 0);

        let classic_rook = evaluator.get_piece_value(PieceType::Rook);
        assert_ne!(research_rook, classic_rook);
        assert!(!evaluator.config().include_hand_pieces);
    }

    #[test]
    fn test_symmetry() {
        let mut evaluator = MaterialEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Black and White should have opposite scores in starting position
        let black_score = evaluator.evaluate_material(&board, Player::Black, &captured_pieces);
        let white_score = evaluator.evaluate_material(&board, Player::White, &captured_pieces);

        assert_eq!(black_score.mg, -white_score.mg);
        assert_eq!(black_score.eg, -white_score.eg);
    }
}
