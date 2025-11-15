//! Board Representation Types
//!
//! This module contains types related to board representation: CapturedPieces and GamePhase.
//! Extracted from `types.rs` as part of Task 1.0: File Modularization and Structure Improvements.

use serde::{Deserialize, Serialize};
use super::core::{PieceType, Player};

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

impl Default for CapturedPieces {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
}

impl GamePhase {
    pub fn from_piece_count(piece_count: u8) -> Self {
        if piece_count >= 30 {
            GamePhase::Opening
        } else if piece_count >= 15 {
            GamePhase::Middlegame
        } else {
            GamePhase::Endgame
        }
    }

    /// Create GamePhase from material count (for backward compatibility)
    pub fn from_material_count(material_count: u8) -> Self {
        Self::from_piece_count(material_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captured_pieces() {
        let mut captured = CapturedPieces::new();
        captured.add_piece(PieceType::Pawn, Player::Black);
        assert_eq!(captured.count(PieceType::Pawn, Player::Black), 1);
    }

    #[test]
    fn test_game_phase() {
        assert_eq!(GamePhase::from_piece_count(35), GamePhase::Opening);
        assert_eq!(GamePhase::from_piece_count(20), GamePhase::Middlegame);
        assert_eq!(GamePhase::from_piece_count(10), GamePhase::Endgame);
    }
}

