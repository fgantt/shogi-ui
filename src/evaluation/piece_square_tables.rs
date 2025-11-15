//! Piece-Square Tables Module
//!
//! This module provides phase-aware piece-square tables for positional evaluation.
//! Piece-square tables assign bonuses/penalties to pieces based on their position,
//! with different values for opening/middlegame and endgame phases.
//!
//! # Overview
//!
//! The piece-square table system:
//! - Provides separate tables for all piece types (including promoted)
//! - Different values for middlegame and endgame phases
//! - Handles player symmetry automatically
//! - Optimized for fast lookups
//! - Returns TaperedScore for seamless integration
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::piece_square_tables::PieceSquareTables;
//! use crate::types::{PieceType, Position, Player};
//!
//! let tables = PieceSquareTables::new();
//! let pos = Position::new(4, 4); // Center square
//! let score = tables.get_value(PieceType::Rook, pos, Player::Black);
//! println!("Rook on center: {} (mg) → {} (eg)", score.mg, score.eg);
//! ```

use crate::types::core::{PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use thiserror::Error;

/// Piece-square tables for dual-phase positional evaluation
static BUILTIN_PST: OnceLock<Arc<PieceSquareTableStorage>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct PieceSquareTables {
    inner: Arc<PieceSquareTableStorage>,
}

#[derive(Debug)]
pub struct PieceSquareTableStorage {
    // Middlegame tables - basic pieces
    pub pawn_table_mg: [[i32; 9]; 9],
    pub lance_table_mg: [[i32; 9]; 9],
    pub knight_table_mg: [[i32; 9]; 9],
    pub silver_table_mg: [[i32; 9]; 9],
    pub gold_table_mg: [[i32; 9]; 9],
    pub bishop_table_mg: [[i32; 9]; 9],
    pub rook_table_mg: [[i32; 9]; 9],

    // Endgame tables - basic pieces
    pub pawn_table_eg: [[i32; 9]; 9],
    pub lance_table_eg: [[i32; 9]; 9],
    pub knight_table_eg: [[i32; 9]; 9],
    pub silver_table_eg: [[i32; 9]; 9],
    pub gold_table_eg: [[i32; 9]; 9],
    pub bishop_table_eg: [[i32; 9]; 9],
    pub rook_table_eg: [[i32; 9]; 9],

    // Middlegame tables - promoted pieces
    pub promoted_pawn_table_mg: [[i32; 9]; 9],
    pub promoted_lance_table_mg: [[i32; 9]; 9],
    pub promoted_knight_table_mg: [[i32; 9]; 9],
    pub promoted_silver_table_mg: [[i32; 9]; 9],
    pub promoted_bishop_table_mg: [[i32; 9]; 9],
    pub promoted_rook_table_mg: [[i32; 9]; 9],

    // Endgame tables - promoted pieces
    pub promoted_pawn_table_eg: [[i32; 9]; 9],
    pub promoted_lance_table_eg: [[i32; 9]; 9],
    pub promoted_knight_table_eg: [[i32; 9]; 9],
    pub promoted_silver_table_eg: [[i32; 9]; 9],
    pub promoted_bishop_table_eg: [[i32; 9]; 9],
    pub promoted_rook_table_eg: [[i32; 9]; 9],
}

impl Default for PieceSquareTableStorage {
    fn default() -> Self {
        Self {
            pawn_table_mg: PieceSquareTables::init_pawn_table_mg(),
            pawn_table_eg: PieceSquareTables::init_pawn_table_eg(),
            lance_table_mg: PieceSquareTables::init_lance_table_mg(),
            lance_table_eg: PieceSquareTables::init_lance_table_eg(),
            knight_table_mg: PieceSquareTables::init_knight_table_mg(),
            knight_table_eg: PieceSquareTables::init_knight_table_eg(),
            silver_table_mg: PieceSquareTables::init_silver_table_mg(),
            silver_table_eg: PieceSquareTables::init_silver_table_eg(),
            gold_table_mg: PieceSquareTables::init_gold_table_mg(),
            gold_table_eg: PieceSquareTables::init_gold_table_eg(),
            bishop_table_mg: PieceSquareTables::init_bishop_table_mg(),
            bishop_table_eg: PieceSquareTables::init_bishop_table_eg(),
            rook_table_mg: PieceSquareTables::init_rook_table_mg(),
            rook_table_eg: PieceSquareTables::init_rook_table_eg(),
            promoted_pawn_table_mg: PieceSquareTables::init_promoted_pawn_table_mg(),
            promoted_pawn_table_eg: PieceSquareTables::init_promoted_pawn_table_eg(),
            promoted_lance_table_mg: PieceSquareTables::init_promoted_lance_table_mg(),
            promoted_lance_table_eg: PieceSquareTables::init_promoted_lance_table_eg(),
            promoted_knight_table_mg: PieceSquareTables::init_promoted_knight_table_mg(),
            promoted_knight_table_eg: PieceSquareTables::init_promoted_knight_table_eg(),
            promoted_silver_table_mg: PieceSquareTables::init_promoted_silver_table_mg(),
            promoted_silver_table_eg: PieceSquareTables::init_promoted_silver_table_eg(),
            promoted_bishop_table_mg: PieceSquareTables::init_promoted_bishop_table_mg(),
            promoted_bishop_table_eg: PieceSquareTables::init_promoted_bishop_table_eg(),
            promoted_rook_table_mg: PieceSquareTables::init_promoted_rook_table_mg(),
            promoted_rook_table_eg: PieceSquareTables::init_promoted_rook_table_eg(),
        }
    }
}

impl From<PieceSquareTableRaw> for PieceSquareTableStorage {
    fn from(raw: PieceSquareTableRaw) -> Self {
        Self {
            pawn_table_mg: raw.pawn_table_mg,
            pawn_table_eg: raw.pawn_table_eg,
            lance_table_mg: raw.lance_table_mg,
            lance_table_eg: raw.lance_table_eg,
            knight_table_mg: raw.knight_table_mg,
            knight_table_eg: raw.knight_table_eg,
            silver_table_mg: raw.silver_table_mg,
            silver_table_eg: raw.silver_table_eg,
            gold_table_mg: raw.gold_table_mg,
            gold_table_eg: raw.gold_table_eg,
            bishop_table_mg: raw.bishop_table_mg,
            bishop_table_eg: raw.bishop_table_eg,
            rook_table_mg: raw.rook_table_mg,
            rook_table_eg: raw.rook_table_eg,
            promoted_pawn_table_mg: raw.promoted_pawn_table_mg,
            promoted_pawn_table_eg: raw.promoted_pawn_table_eg,
            promoted_lance_table_mg: raw.promoted_lance_table_mg,
            promoted_lance_table_eg: raw.promoted_lance_table_eg,
            promoted_knight_table_mg: raw.promoted_knight_table_mg,
            promoted_knight_table_eg: raw.promoted_knight_table_eg,
            promoted_silver_table_mg: raw.promoted_silver_table_mg,
            promoted_silver_table_eg: raw.promoted_silver_table_eg,
            promoted_bishop_table_mg: raw.promoted_bishop_table_mg,
            promoted_bishop_table_eg: raw.promoted_bishop_table_eg,
            promoted_rook_table_mg: raw.promoted_rook_table_mg,
            promoted_rook_table_eg: raw.promoted_rook_table_eg,
        }
    }
}

impl From<&PieceSquareTableStorage> for PieceSquareTableRaw {
    fn from(storage: &PieceSquareTableStorage) -> Self {
        Self {
            pawn_table_mg: storage.pawn_table_mg,
            pawn_table_eg: storage.pawn_table_eg,
            lance_table_mg: storage.lance_table_mg,
            lance_table_eg: storage.lance_table_eg,
            knight_table_mg: storage.knight_table_mg,
            knight_table_eg: storage.knight_table_eg,
            silver_table_mg: storage.silver_table_mg,
            silver_table_eg: storage.silver_table_eg,
            gold_table_mg: storage.gold_table_mg,
            gold_table_eg: storage.gold_table_eg,
            bishop_table_mg: storage.bishop_table_mg,
            bishop_table_eg: storage.bishop_table_eg,
            rook_table_mg: storage.rook_table_mg,
            rook_table_eg: storage.rook_table_eg,
            promoted_pawn_table_mg: storage.promoted_pawn_table_mg,
            promoted_pawn_table_eg: storage.promoted_pawn_table_eg,
            promoted_lance_table_mg: storage.promoted_lance_table_mg,
            promoted_lance_table_eg: storage.promoted_lance_table_eg,
            promoted_knight_table_mg: storage.promoted_knight_table_mg,
            promoted_knight_table_eg: storage.promoted_knight_table_eg,
            promoted_silver_table_mg: storage.promoted_silver_table_mg,
            promoted_silver_table_eg: storage.promoted_silver_table_eg,
            promoted_bishop_table_mg: storage.promoted_bishop_table_mg,
            promoted_bishop_table_eg: storage.promoted_bishop_table_eg,
            promoted_rook_table_mg: storage.promoted_rook_table_mg,
            promoted_rook_table_eg: storage.promoted_rook_table_eg,
        }
    }
}

impl From<PieceSquareTableStorage> for PieceSquareTableRaw {
    fn from(storage: PieceSquareTableStorage) -> Self {
        Self::from(&storage)
    }
}

impl Deref for PieceSquareTables {
    type Target = PieceSquareTableStorage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PieceSquareTableRaw {
    pub pawn_table_mg: [[i32; 9]; 9],
    pub pawn_table_eg: [[i32; 9]; 9],
    pub lance_table_mg: [[i32; 9]; 9],
    pub lance_table_eg: [[i32; 9]; 9],
    pub knight_table_mg: [[i32; 9]; 9],
    pub knight_table_eg: [[i32; 9]; 9],
    pub silver_table_mg: [[i32; 9]; 9],
    pub silver_table_eg: [[i32; 9]; 9],
    pub gold_table_mg: [[i32; 9]; 9],
    pub gold_table_eg: [[i32; 9]; 9],
    pub bishop_table_mg: [[i32; 9]; 9],
    pub bishop_table_eg: [[i32; 9]; 9],
    pub rook_table_mg: [[i32; 9]; 9],
    pub rook_table_eg: [[i32; 9]; 9],
    pub promoted_pawn_table_mg: [[i32; 9]; 9],
    pub promoted_pawn_table_eg: [[i32; 9]; 9],
    pub promoted_lance_table_mg: [[i32; 9]; 9],
    pub promoted_lance_table_eg: [[i32; 9]; 9],
    pub promoted_knight_table_mg: [[i32; 9]; 9],
    pub promoted_knight_table_eg: [[i32; 9]; 9],
    pub promoted_silver_table_mg: [[i32; 9]; 9],
    pub promoted_silver_table_eg: [[i32; 9]; 9],
    pub promoted_bishop_table_mg: [[i32; 9]; 9],
    pub promoted_bishop_table_eg: [[i32; 9]; 9],
    pub promoted_rook_table_mg: [[i32; 9]; 9],
    pub promoted_rook_table_eg: [[i32; 9]; 9],
}

impl PieceSquareTableRaw {
    pub fn default() -> Self {
        PieceSquareTableStorage::default().into()
    }
}

/// Phase-specific table pair for a single piece type.
#[derive(Debug, Clone, Copy)]
pub struct PiecePhaseTables {
    pub mg: [[i32; 9]; 9],
    pub eg: [[i32; 9]; 9],
}

#[derive(Debug, Error)]
pub enum PieceSquareTableError {
    #[error("piece-square table data missing entry for {0:?}")]
    MissingPiece(PieceType),
    #[error("king positional tables must be zero in both phases")]
    UnsupportedKingValues,
}

impl PieceSquareTables {
    /// Create a new PieceSquareTables with default values
    pub fn new() -> Self {
        let inner = BUILTIN_PST
            .get_or_init(|| Arc::new(PieceSquareTableStorage::default()))
            .clone();
        Self { inner }
    }

    pub fn from_raw(raw: PieceSquareTableRaw) -> Self {
        Self {
            inner: Arc::new(PieceSquareTableStorage::from(raw)),
        }
    }

    pub fn to_raw(&self) -> PieceSquareTableRaw {
        PieceSquareTableRaw::from(self.inner.as_ref())
    }

    /// Construct tables from externally provided phase data.
    pub fn from_phase_tables(
        tables: &HashMap<PieceType, PiecePhaseTables>,
    ) -> Result<Self, PieceSquareTableError> {
        fn get_tables<'a>(
            tables: &'a HashMap<PieceType, PiecePhaseTables>,
            piece: PieceType,
        ) -> Result<PiecePhaseTables, PieceSquareTableError> {
            tables
                .get(&piece)
                .copied()
                .ok_or(PieceSquareTableError::MissingPiece(piece))
        }

        let pawn = get_tables(tables, PieceType::Pawn)?;
        let lance = get_tables(tables, PieceType::Lance)?;
        let knight = get_tables(tables, PieceType::Knight)?;
        let silver = get_tables(tables, PieceType::Silver)?;
        let gold = get_tables(tables, PieceType::Gold)?;
        let bishop = get_tables(tables, PieceType::Bishop)?;
        let rook = get_tables(tables, PieceType::Rook)?;
        if let Some(king_tables) = tables.get(&PieceType::King) {
            if king_tables.mg != [[0; 9]; 9] || king_tables.eg != [[0; 9]; 9] {
                return Err(PieceSquareTableError::UnsupportedKingValues);
            }
        }
        let promoted_pawn = get_tables(tables, PieceType::PromotedPawn)?;
        let promoted_lance = get_tables(tables, PieceType::PromotedLance)?;
        let promoted_knight = get_tables(tables, PieceType::PromotedKnight)?;
        let promoted_silver = get_tables(tables, PieceType::PromotedSilver)?;
        let promoted_bishop = get_tables(tables, PieceType::PromotedBishop)?;
        let promoted_rook = get_tables(tables, PieceType::PromotedRook)?;

        let raw = PieceSquareTableRaw {
            pawn_table_mg: pawn.mg,
            pawn_table_eg: pawn.eg,
            lance_table_mg: lance.mg,
            lance_table_eg: lance.eg,
            knight_table_mg: knight.mg,
            knight_table_eg: knight.eg,
            silver_table_mg: silver.mg,
            silver_table_eg: silver.eg,
            gold_table_mg: gold.mg,
            gold_table_eg: gold.eg,
            bishop_table_mg: bishop.mg,
            bishop_table_eg: bishop.eg,
            rook_table_mg: rook.mg,
            rook_table_eg: rook.eg,
            promoted_pawn_table_mg: promoted_pawn.mg,
            promoted_pawn_table_eg: promoted_pawn.eg,
            promoted_lance_table_mg: promoted_lance.mg,
            promoted_lance_table_eg: promoted_lance.eg,
            promoted_knight_table_mg: promoted_knight.mg,
            promoted_knight_table_eg: promoted_knight.eg,
            promoted_silver_table_mg: promoted_silver.mg,
            promoted_silver_table_eg: promoted_silver.eg,
            promoted_bishop_table_mg: promoted_bishop.mg,
            promoted_bishop_table_eg: promoted_bishop.eg,
            promoted_rook_table_mg: promoted_rook.mg,
            promoted_rook_table_eg: promoted_rook.eg,
        };

        Ok(Self::from_raw(raw))
    }

    /// Export the tables as a map keyed by piece type.
    pub fn to_phase_tables(&self) -> HashMap<PieceType, PiecePhaseTables> {
        let mut map = HashMap::new();
        macro_rules! insert {
            ($piece:expr, $mg:expr, $eg:expr) => {
                map.insert($piece, PiecePhaseTables { mg: $mg, eg: $eg });
            };
        }

        insert!(PieceType::Pawn, self.pawn_table_mg, self.pawn_table_eg);
        insert!(PieceType::Lance, self.lance_table_mg, self.lance_table_eg);
        insert!(
            PieceType::Knight,
            self.knight_table_mg,
            self.knight_table_eg
        );
        insert!(
            PieceType::Silver,
            self.silver_table_mg,
            self.silver_table_eg
        );
        insert!(PieceType::Gold, self.gold_table_mg, self.gold_table_eg);
        insert!(
            PieceType::Bishop,
            self.bishop_table_mg,
            self.bishop_table_eg
        );
        insert!(PieceType::Rook, self.rook_table_mg, self.rook_table_eg);
        insert!(PieceType::King, [[0; 9]; 9], [[0; 9]; 9]);
        insert!(
            PieceType::PromotedPawn,
            self.promoted_pawn_table_mg,
            self.promoted_pawn_table_eg
        );
        insert!(
            PieceType::PromotedLance,
            self.promoted_lance_table_mg,
            self.promoted_lance_table_eg
        );
        insert!(
            PieceType::PromotedKnight,
            self.promoted_knight_table_mg,
            self.promoted_knight_table_eg
        );
        insert!(
            PieceType::PromotedSilver,
            self.promoted_silver_table_mg,
            self.promoted_silver_table_eg
        );
        insert!(
            PieceType::PromotedBishop,
            self.promoted_bishop_table_mg,
            self.promoted_bishop_table_eg
        );
        insert!(
            PieceType::PromotedRook,
            self.promoted_rook_table_mg,
            self.promoted_rook_table_eg
        );

        map
    }

    /// Get positional value for a piece (returns TaperedScore)
    ///
    /// This is the main entry point for piece-square table lookups.
    /// Returns a TaperedScore with separate mg/eg values.
    pub fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> TaperedScore {
        let (mg_table, eg_table) = self.get_tables(piece_type);
        let (row, col) = self.get_table_coords(pos, player);

        let mg_value = mg_table[row as usize][col as usize];
        let eg_value = eg_table[row as usize][col as usize];

        TaperedScore::new_tapered(mg_value, eg_value)
    }

    /// Get both mg and eg tables for a piece type
    ///
    /// Returns references to the middlegame and endgame tables for the specified piece.
    /// Returns zero tables for King (no positional bonus for King).
    pub fn get_tables(&self, piece_type: PieceType) -> (&[[i32; 9]; 9], &[[i32; 9]; 9]) {
        match piece_type {
            // Basic pieces
            PieceType::Pawn => (&self.pawn_table_mg, &self.pawn_table_eg),
            PieceType::Lance => (&self.lance_table_mg, &self.lance_table_eg),
            PieceType::Knight => (&self.knight_table_mg, &self.knight_table_eg),
            PieceType::Silver => (&self.silver_table_mg, &self.silver_table_eg),
            PieceType::Gold => (&self.gold_table_mg, &self.gold_table_eg),
            PieceType::Bishop => (&self.bishop_table_mg, &self.bishop_table_eg),
            PieceType::Rook => (&self.rook_table_mg, &self.rook_table_eg),

            // Promoted pieces
            PieceType::PromotedPawn => (&self.promoted_pawn_table_mg, &self.promoted_pawn_table_eg),
            PieceType::PromotedLance => {
                (&self.promoted_lance_table_mg, &self.promoted_lance_table_eg)
            }
            PieceType::PromotedKnight => (
                &self.promoted_knight_table_mg,
                &self.promoted_knight_table_eg,
            ),
            PieceType::PromotedSilver => (
                &self.promoted_silver_table_mg,
                &self.promoted_silver_table_eg,
            ),
            PieceType::PromotedBishop => (
                &self.promoted_bishop_table_mg,
                &self.promoted_bishop_table_eg,
            ),
            PieceType::PromotedRook => (&self.promoted_rook_table_mg, &self.promoted_rook_table_eg),

            // King has no positional bonus
            PieceType::King => (&[[0; 9]; 9], &[[0; 9]; 9]),
        }
    }

    /// Get table coordinates for a position and player
    ///
    /// Handles symmetry: White player's pieces use flipped coordinates
    pub fn get_table_coords(&self, pos: Position, player: Player) -> (u8, u8) {
        if player == Player::Black {
            (pos.row, pos.col)
        } else {
            // Mirror coordinates for White player
            (8 - pos.row, 8 - pos.col)
        }
    }

    // =======================================================================
    // BASIC PIECE TABLES - MIDDLEGAME
    // =======================================================================

    /// Pawn table (middlegame): Reward advancement
    fn init_pawn_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],          // Rank 1 (back row)
            [5, 5, 5, 5, 5, 5, 5, 5, 5],          // Rank 2
            [10, 10, 12, 12, 15, 12, 12, 10, 10], // Rank 3 (center files better)
            [15, 15, 18, 18, 20, 18, 18, 15, 15], // Rank 4
            [20, 20, 22, 22, 25, 22, 22, 20, 20], // Rank 5
            [25, 25, 28, 28, 30, 28, 28, 25, 25], // Rank 6
            [30, 30, 32, 32, 35, 32, 32, 30, 30], // Rank 7 (promotion zone)
            [35, 35, 38, 38, 40, 38, 38, 35, 35], // Rank 8
            [0, 0, 0, 0, 0, 0, 0, 0, 0],          // Rank 9 (promote immediately)
        ]
    }

    /// Lance table (middlegame): Center files are better
    fn init_lance_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 5, 10, 12, 10, 5, 0, 0],
            [0, 0, 0, 10, 12, 10, 5, 0, 0],
        ]
    }

    /// Knight table (middlegame): Center and advanced positions
    fn init_knight_table_mg() -> [[i32; 9]; 9] {
        [
            [-10, -10, -10, -10, -10, -10, -10, -10, -10], // Back row penalty
            [-10, 0, 0, 0, 5, 0, 0, 0, -10],
            [-10, 0, 5, 10, 15, 10, 5, 0, -10],
            [-10, 0, 10, 15, 20, 15, 10, 0, -10],
            [-10, 0, 10, 15, 20, 15, 10, 0, -10],
            [-10, 0, 5, 10, 15, 10, 5, 0, -10],
            [-10, 0, 5, 5, 10, 5, 5, 0, -10],
            [-10, 0, 0, 0, 5, 0, 0, 0, -10],
            [-10, -10, -10, -10, -10, -10, -10, -10, -10], // Can't move from row 9
        ]
    }

    /// Silver table (middlegame): Center preference, king vicinity in opening
    fn init_silver_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 5, 5, 5, 5, 0, 0],
            [0, 5, 10, 12, 15, 12, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 12, 15, 12, 10, 5, 0],
            [0, 0, 5, 5, 5, 5, 5, 0, 0],
        ]
    }

    /// Gold table (middlegame): Similar to silver, but slightly more defensive
    fn init_gold_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 8, 8, 8, 5, 0, 0],
            [0, 5, 10, 12, 15, 12, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 12, 15, 12, 10, 5, 0],
            [0, 0, 5, 8, 8, 8, 5, 0, 0],
        ]
    }

    /// Bishop table (middlegame): Center control and diagonals
    fn init_bishop_table_mg() -> [[i32; 9]; 9] {
        [
            [-10, -10, -5, -5, -5, -5, -5, -10, -10],
            [-10, 0, 5, 8, 10, 8, 5, 0, -10],
            [-5, 5, 10, 12, 15, 12, 10, 5, -5],
            [-5, 8, 12, 18, 20, 18, 12, 8, -5],
            [-5, 10, 15, 20, 22, 20, 15, 10, -5],
            [-5, 8, 12, 18, 20, 18, 12, 8, -5],
            [-5, 5, 10, 12, 15, 12, 10, 5, -5],
            [-10, 0, 5, 8, 10, 8, 5, 0, -10],
            [-10, -10, -5, -5, -5, -5, -5, -10, -10],
        ]
    }

    /// Rook table (middlegame): Center files and 7th rank
    fn init_rook_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 5, 8, 12, 15, 12, 8, 5, 0],
            [0, 5, 8, 12, 15, 12, 8, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [5, 10, 12, 15, 20, 15, 12, 10, 5], // 7th rank bonus
            [0, 5, 8, 12, 15, 12, 8, 5, 0],
        ]
    }

    // =======================================================================
    // BASIC PIECE TABLES - ENDGAME
    // =======================================================================

    /// Pawn table (endgame): Even more reward for advancement
    fn init_pawn_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [10, 10, 12, 12, 15, 12, 12, 10, 10],
            [20, 20, 25, 25, 30, 25, 25, 20, 20],
            [30, 30, 35, 35, 40, 35, 35, 30, 30],
            [40, 40, 45, 45, 50, 45, 45, 40, 40],
            [50, 50, 55, 55, 60, 55, 55, 50, 50],
            [60, 60, 65, 65, 70, 65, 65, 60, 60],
            [70, 70, 75, 75, 80, 75, 75, 70, 70],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    /// Lance table (endgame): Advanced positions more valuable
    fn init_lance_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 10, 20, 25, 20, 10, 0, 0],
            [0, 0, 10, 20, 25, 20, 10, 0, 0],
            [0, 0, 12, 22, 28, 22, 12, 0, 0],
            [0, 0, 12, 22, 28, 22, 12, 0, 0],
            [0, 0, 12, 22, 28, 22, 12, 0, 0],
            [0, 0, 12, 22, 28, 22, 12, 0, 0],
            [0, 0, 10, 20, 25, 20, 10, 0, 0],
            [0, 0, 10, 20, 25, 20, 10, 0, 0],
            [0, 0, 0, 20, 25, 20, 10, 0, 0],
        ]
    }

    /// Knight table (endgame): Less valuable in endgame
    fn init_knight_table_eg() -> [[i32; 9]; 9] {
        [
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
            [-20, -5, -5, 0, 5, 0, -5, -5, -20],
            [-20, -5, 5, 15, 25, 15, 5, -5, -20],
            [-20, 0, 15, 25, 35, 25, 15, 0, -20],
            [-20, 0, 15, 25, 35, 25, 15, 0, -20],
            [-20, -5, 10, 20, 30, 20, 10, -5, -20],
            [-20, -5, 5, 15, 20, 15, 5, -5, -20],
            [-20, -5, 0, 5, 10, 5, 0, -5, -20],
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
        ]
    }

    /// Silver table (endgame): Centralization important
    fn init_silver_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 10, 15, 18, 15, 10, 0, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 0, 10, 15, 18, 15, 10, 0, 0],
        ]
    }

    /// Gold table (endgame): King support and centralization
    fn init_gold_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 10, 15, 20, 15, 10, 0, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 10, 20, 25, 30, 25, 20, 10, 0],
            [0, 0, 10, 15, 20, 15, 10, 0, 0],
        ]
    }

    /// Bishop table (endgame): Long diagonals very powerful
    fn init_bishop_table_eg() -> [[i32; 9]; 9] {
        [
            [-20, -15, -10, -5, 0, -5, -10, -15, -20],
            [-15, 0, 10, 15, 20, 15, 10, 0, -15],
            [-10, 10, 20, 28, 35, 28, 20, 10, -10],
            [-5, 15, 28, 38, 45, 38, 28, 15, -5],
            [0, 20, 35, 45, 50, 45, 35, 20, 0],
            [-5, 15, 28, 38, 45, 38, 28, 15, -5],
            [-10, 10, 20, 28, 35, 28, 20, 10, -10],
            [-15, 0, 10, 15, 20, 15, 10, 0, -15],
            [-20, -15, -10, -5, 0, -5, -10, -15, -20],
        ]
    }

    /// Rook table (endgame): Open files and ranks critical
    fn init_rook_table_eg() -> [[i32; 9]; 9] {
        [
            [-10, -5, 0, 8, 12, 8, 0, -5, -10],
            [5, 10, 15, 20, 25, 20, 15, 10, 5],
            [10, 15, 20, 25, 30, 25, 20, 15, 10],
            [15, 20, 25, 30, 35, 30, 25, 20, 15],
            [15, 20, 25, 30, 35, 30, 25, 20, 15],
            [15, 20, 25, 30, 35, 30, 25, 20, 15],
            [10, 15, 20, 25, 30, 25, 20, 15, 10],
            [20, 25, 28, 32, 38, 32, 28, 25, 20], // 7th rank even better in endgame
            [-10, -5, 0, 8, 12, 8, 0, -5, -10],
        ]
    }

    // =======================================================================
    // PROMOTED PIECE TABLES - MIDDLEGAME
    // =======================================================================

    /// Promoted pawn table (middlegame): Gold-like movement, center control
    fn init_promoted_pawn_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 8, 10, 8, 5, 0, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 12, 18, 22, 18, 12, 5, 0],
            [0, 5, 15, 20, 25, 20, 15, 5, 0],
            [0, 5, 15, 20, 25, 20, 15, 5, 0],
            [0, 5, 12, 18, 22, 18, 12, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 5, 10, 15, 18, 15, 10, 5, 0],
            [0, 0, 5, 8, 10, 8, 5, 0, 0],
        ]
    }

    /// Promoted lance table (middlegame): Similar to promoted pawn
    fn init_promoted_lance_table_mg() -> [[i32; 9]; 9] {
        Self::init_promoted_pawn_table_mg()
    }

    /// Promoted knight table (middlegame): Gold-like, prefers center
    fn init_promoted_knight_table_mg() -> [[i32; 9]; 9] {
        Self::init_promoted_pawn_table_mg()
    }

    /// Promoted silver table (middlegame): Gold-like
    fn init_promoted_silver_table_mg() -> [[i32; 9]; 9] {
        Self::init_gold_table_mg()
    }

    /// Promoted bishop table (middlegame): Enhanced mobility
    fn init_promoted_bishop_table_mg() -> [[i32; 9]; 9] {
        [
            [-5, -5, 0, 5, 8, 5, 0, -5, -5],
            [-5, 5, 10, 15, 18, 15, 10, 5, -5],
            [0, 10, 18, 22, 28, 22, 18, 10, 0],
            [5, 15, 22, 28, 35, 28, 22, 15, 5],
            [8, 18, 28, 35, 40, 35, 28, 18, 8],
            [5, 15, 22, 28, 35, 28, 22, 15, 5],
            [0, 10, 18, 22, 28, 22, 18, 10, 0],
            [-5, 5, 10, 15, 18, 15, 10, 5, -5],
            [-5, -5, 0, 5, 8, 5, 0, -5, -5],
        ]
    }

    /// Promoted rook table (middlegame): Extremely powerful, center control
    fn init_promoted_rook_table_mg() -> [[i32; 9]; 9] {
        [
            [5, 10, 15, 20, 25, 20, 15, 10, 5],
            [5, 10, 15, 22, 28, 22, 15, 10, 5],
            [8, 12, 18, 25, 30, 25, 18, 12, 8],
            [10, 15, 22, 28, 35, 28, 22, 15, 10],
            [10, 18, 25, 32, 38, 32, 25, 18, 10],
            [10, 15, 22, 28, 35, 28, 22, 15, 10],
            [8, 12, 18, 25, 30, 25, 18, 12, 8],
            [10, 15, 20, 25, 30, 25, 20, 15, 10],
            [5, 10, 15, 20, 25, 20, 15, 10, 5],
        ]
    }

    // =======================================================================
    // PROMOTED PIECE TABLES - ENDGAME
    // =======================================================================

    /// Promoted pawn table (endgame): Very valuable, centralization
    fn init_promoted_pawn_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 10, 15, 20, 15, 10, 0, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 15, 25, 35, 42, 35, 25, 15, 0],
            [0, 15, 28, 38, 48, 38, 28, 15, 0],
            [0, 15, 28, 38, 48, 38, 28, 15, 0],
            [0, 15, 25, 35, 42, 35, 25, 15, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 10, 20, 28, 35, 28, 20, 10, 0],
            [0, 0, 10, 15, 20, 15, 10, 0, 0],
        ]
    }

    /// Promoted lance table (endgame)
    fn init_promoted_lance_table_eg() -> [[i32; 9]; 9] {
        Self::init_promoted_pawn_table_eg()
    }

    /// Promoted knight table (endgame)
    fn init_promoted_knight_table_eg() -> [[i32; 9]; 9] {
        Self::init_promoted_pawn_table_eg()
    }

    /// Promoted silver table (endgame)
    fn init_promoted_silver_table_eg() -> [[i32; 9]; 9] {
        Self::init_gold_table_eg()
    }

    /// Promoted bishop table (endgame): Dominant piece
    fn init_promoted_bishop_table_eg() -> [[i32; 9]; 9] {
        [
            [-10, -5, 0, 10, 15, 10, 0, -5, -10],
            [-5, 10, 20, 30, 38, 30, 20, 10, -5],
            [0, 20, 32, 42, 50, 42, 32, 20, 0],
            [10, 30, 42, 52, 60, 52, 42, 30, 10],
            [15, 38, 50, 60, 68, 60, 50, 38, 15],
            [10, 30, 42, 52, 60, 52, 42, 30, 10],
            [0, 20, 32, 42, 50, 42, 32, 20, 0],
            [-5, 10, 20, 30, 38, 30, 20, 10, -5],
            [-10, -5, 0, 10, 15, 10, 0, -5, -10],
        ]
    }

    /// Promoted rook table (endgame): Most powerful piece
    fn init_promoted_rook_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
            [10, 20, 30, 40, 48, 40, 30, 20, 10],
            [20, 30, 40, 50, 58, 50, 40, 30, 20],
            [30, 40, 50, 60, 68, 60, 50, 40, 30],
            [38, 48, 58, 68, 75, 68, 58, 48, 38],
            [30, 40, 50, 60, 68, 60, 50, 40, 30],
            [20, 30, 40, 50, 58, 50, 40, 30, 20],
            [30, 38, 45, 52, 60, 52, 45, 38, 30],
            [0, 10, 20, 30, 38, 30, 20, 10, 0],
        ]
    }
}

impl Default for PieceSquareTables {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_square_tables_creation() {
        let tables = PieceSquareTables::new();
        // Should compile and create without panic
        assert_eq!(tables.pawn_table_mg[0][0], 0);
    }

    #[test]
    fn test_get_value_pawn() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4); // Center square
        let score = tables.get_value(PieceType::Pawn, pos, Player::Black);

        // Pawns should have positive value in center
        assert!(score.mg > 0);
        assert!(score.eg > score.mg); // Pawns more valuable in endgame
    }

    #[test]
    fn test_get_value_rook() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4); // Center square
        let score = tables.get_value(PieceType::Rook, pos, Player::Black);

        // Rooks should have positive value in center
        assert!(score.mg > 0);
        assert!(score.eg > score.mg); // Rooks more valuable in endgame
    }

    #[test]
    fn test_get_value_promoted_pieces() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4);

        // Promoted pawn
        let promoted_pawn = tables.get_value(PieceType::PromotedPawn, pos, Player::Black);
        assert!(promoted_pawn.mg > 0);
        assert!(promoted_pawn.eg > 0);

        // Promoted rook
        let promoted_rook = tables.get_value(PieceType::PromotedRook, pos, Player::Black);
        assert!(promoted_rook.mg > 0);
        assert!(promoted_rook.eg > 0);
        assert!(promoted_rook.eg > promoted_pawn.eg); // Promoted rook better than promoted pawn
    }

    #[test]
    fn test_get_value_king() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4);
        let score = tables.get_value(PieceType::King, pos, Player::Black);

        // King should have no positional bonus
        assert_eq!(score.mg, 0);
        assert_eq!(score.eg, 0);
    }

    #[test]
    fn test_symmetry() {
        let tables = PieceSquareTables::new();

        // Check symmetry: Black (0,0) == White (8,8)
        let black_pos = Position::new(0, 0);
        let white_pos = Position::new(8, 8);

        let black_score = tables.get_value(PieceType::Pawn, black_pos, Player::Black);
        let white_score = tables.get_value(PieceType::Pawn, white_pos, Player::White);

        // Due to symmetry, these should be equal
        assert_eq!(black_score.mg, white_score.mg);
        assert_eq!(black_score.eg, white_score.eg);
    }

    #[test]
    fn test_table_coords() {
        let tables = PieceSquareTables::new();

        // Black player: no transformation
        let pos = Position::new(4, 4);
        let (row, col) = tables.get_table_coords(pos, Player::Black);
        assert_eq!(row, 4);
        assert_eq!(col, 4);

        // White player: flip coordinates
        let (row, col) = tables.get_table_coords(pos, Player::White);
        assert_eq!(row, 4); // 8 - 4 = 4
        assert_eq!(col, 4); // 8 - 4 = 4
    }

    #[test]
    fn test_pawn_advancement_bonus() {
        let tables = PieceSquareTables::new();

        // Pawns should get more value as they advance
        let back_row = Position::new(0, 4);
        let mid_row = Position::new(4, 4);
        let front_row = Position::new(7, 4);

        let back_score = tables.get_value(PieceType::Pawn, back_row, Player::Black);
        let mid_score = tables.get_value(PieceType::Pawn, mid_row, Player::Black);
        let front_score = tables.get_value(PieceType::Pawn, front_row, Player::Black);

        assert!(mid_score.mg > back_score.mg);
        assert!(front_score.mg > mid_score.mg);

        // Even more pronounced in endgame
        assert!(mid_score.eg > back_score.eg);
        assert!(front_score.eg > mid_score.eg);
    }

    #[test]
    fn test_center_bonus() {
        let tables = PieceSquareTables::new();

        // Center should be better than edge for most pieces
        let center = Position::new(4, 4);
        let edge = Position::new(4, 0);

        let center_rook = tables.get_value(PieceType::Rook, center, Player::Black);
        let edge_rook = tables.get_value(PieceType::Rook, edge, Player::Black);

        assert!(center_rook.mg > edge_rook.mg);
        assert!(center_rook.eg > edge_rook.eg);
    }

    #[test]
    fn test_knight_back_rank_penalty() {
        let tables = PieceSquareTables::new();

        // Knights should have penalty on back ranks
        let back_rank = Position::new(0, 4);
        let center = Position::new(4, 4);

        let back_knight = tables.get_value(PieceType::Knight, back_rank, Player::Black);
        let center_knight = tables.get_value(PieceType::Knight, center, Player::Black);

        assert!(back_knight.mg < center_knight.mg);
    }

    #[test]
    fn test_promoted_vs_unpromoted() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4);

        // Test pawn vs promoted pawn
        let pawn = tables.get_value(PieceType::Pawn, pos, Player::Black);
        let promoted_pawn = tables.get_value(PieceType::PromotedPawn, pos, Player::Black);

        // Promoted pieces should have different positional characteristics from their base forms.
        // Note: This is just positional bonus, not including material value.
        assert_ne!(
            (pawn.mg, pawn.eg),
            (promoted_pawn.mg, promoted_pawn.eg),
            "Promoted pawn should not share the exact mg/eg pair with an unpromoted pawn"
        );

        // Test rook vs promoted rook
        let rook = tables.get_value(PieceType::Rook, pos, Player::Black);
        let promoted_rook = tables.get_value(PieceType::PromotedRook, pos, Player::Black);

        assert_ne!(rook.mg, promoted_rook.mg);
    }

    #[test]
    fn test_all_pieces_have_tables() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4);

        // Test that all piece types return valid values
        let piece_types = [
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

        for piece_type in piece_types {
            let score = tables.get_value(piece_type, pos, Player::Black);
            // Should not panic
            let _ = score.mg;
            let _ = score.eg;
        }
    }

    #[test]
    fn test_table_bounds() {
        let tables = PieceSquareTables::new();

        // Test all corners and edges
        let positions = [
            Position::new(0, 0),
            Position::new(0, 8),
            Position::new(8, 0),
            Position::new(8, 8),
            Position::new(0, 4),
            Position::new(8, 4),
            Position::new(4, 0),
            Position::new(4, 8),
        ];

        for pos in positions {
            let score = tables.get_value(PieceType::Rook, pos, Player::Black);
            // Should not panic on boundary positions
            let _ = score.mg;
            let _ = score.eg;
        }
    }

    #[test]
    fn test_promoted_piece_symmetry() {
        let tables = PieceSquareTables::new();
        let promoted_pieces = [
            PieceType::PromotedPawn,
            PieceType::PromotedLance,
            PieceType::PromotedKnight,
            PieceType::PromotedSilver,
            PieceType::PromotedBishop,
            PieceType::PromotedRook,
        ];

        for piece_type in promoted_pieces {
            for row in 0..9 {
                for col in 0..9 {
                    let black_pos = Position::new(row, col);
                    let white_pos = Position::new(8 - row, 8 - col);

                    let black_score = tables.get_value(piece_type, black_pos, Player::Black);
                    let white_score = tables.get_value(piece_type, white_pos, Player::White);

                    assert_eq!(
                        black_score.mg, white_score.mg,
                        "Promoted {:?} mg mismatch at ({}, {}) vs mirrored position",
                        piece_type, row, col
                    );
                    assert_eq!(
                        black_score.eg, white_score.eg,
                        "Promoted {:?} eg mismatch at ({}, {}) vs mirrored position",
                        piece_type, row, col
                    );
                }
            }
        }
    }

    #[test]
    fn test_taper_interpolation_endpoints_match_tables() {
        let tables = PieceSquareTables::new();
        let sample_positions = [
            (PieceType::Pawn, Position::new(6, 4)),
            (PieceType::Rook, Position::new(4, 4)),
            (PieceType::Bishop, Position::new(3, 3)),
            (PieceType::PromotedRook, Position::new(4, 4)),
        ];

        for (piece_type, pos) in sample_positions {
            let score_black = tables.get_value(piece_type, pos, Player::Black);
            assert_eq!(
                score_black.interpolate(256),
                score_black.mg,
                "mg endpoint mismatch for {:?}",
                piece_type
            );
            assert_eq!(
                score_black.interpolate(0),
                score_black.eg,
                "eg endpoint mismatch for {:?}",
                piece_type
            );
        }
    }
}
