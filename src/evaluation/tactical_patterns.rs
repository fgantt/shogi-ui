//! Tactical Pattern Recognition Module
//!
//! This module implements detection of tactical patterns in Shogi including:
//! - Forks (double attacks)
//! - Pins (pieces that cannot move without exposing king/valuable piece)
//! - Skewers (attacks through less valuable piece to more valuable)
//! - Discovered attacks
//! - Knight forks (special case with unique movement)
//! - Back rank threats
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::evaluation::tactical_patterns::TacticalPatternRecognizer;
//!
//! let recognizer = TacticalPatternRecognizer::new();
//! let tactical_score = recognizer.evaluate_tactics(&board, Player::Black);
//! ```

use crate::types::*;
use crate::bitboards::BitboardBoard;
use serde::{Deserialize, Serialize};

/// Tactical pattern recognizer
pub struct TacticalPatternRecognizer {
    config: TacticalConfig,
    stats: TacticalStats,
}

impl TacticalPatternRecognizer {
    /// Create a new tactical pattern recognizer
    pub fn new() -> Self {
        Self {
            config: TacticalConfig::default(),
            stats: TacticalStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TacticalConfig) -> Self {
        Self {
            config,
            stats: TacticalStats::default(),
        }
    }

    /// Evaluate all tactical patterns for a player
    pub fn evaluate_tactics(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.evaluations += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Detect forks (double attacks)
        if self.config.enable_forks {
            let forks = self.detect_forks(board, player);
            mg_score += forks.mg;
            eg_score += forks.eg;
        }

        // Detect pins
        if self.config.enable_pins {
            let pins = self.detect_pins(board, player);
            mg_score += pins.mg;
            eg_score += pins.eg;
        }

        // Detect skewers
        if self.config.enable_skewers {
            let skewers = self.detect_skewers(board, player);
            mg_score += skewers.mg;
            eg_score += skewers.eg;
        }

        // Detect discovered attacks
        if self.config.enable_discovered_attacks {
            let discovered = self.detect_discovered_attacks(board, player);
            mg_score += discovered.mg;
            eg_score += discovered.eg;
        }

        // Detect knight forks (special handling)
        if self.config.enable_knight_forks {
            let knight_forks = self.detect_knight_forks(board, player);
            mg_score += knight_forks.mg;
            eg_score += knight_forks.eg;
        }

        // Detect back rank threats
        if self.config.enable_back_rank_threats {
            let back_rank = self.detect_back_rank_threats(board, player);
            mg_score += back_rank.mg;
            eg_score += back_rank.eg;
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    // ===================================================================
    // FORK DETECTION (Double Attacks)
    // ===================================================================

    /// Detect forks (pieces attacking 2+ valuable targets simultaneously)
    fn detect_forks(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.fork_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check each piece for fork potential
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let fork_value = self.check_piece_for_forks(board, pos, piece.piece_type, player);
                        mg_score += fork_value.0;
                        eg_score += fork_value.1;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a piece is forking multiple targets
    fn check_piece_for_forks(&self, board: &BitboardBoard, pos: Position, piece_type: PieceType, player: Player) -> (i32, i32) {
        let targets = self.get_attacked_pieces(board, pos, piece_type, player);
        
        if targets.len() >= 2 {
            // Fork detected - calculate value
            let total_value: i32 = targets.iter().map(|(_, value)| value).sum();
            let fork_bonus = (total_value * self.config.fork_bonus_factor) / 100;
            
            // Forking king is especially valuable
            let has_king_fork = targets.iter().any(|(pt, _)| *pt == PieceType::King);
            let king_bonus = if has_king_fork { self.config.king_fork_bonus } else { 0 };
            
            self.stats.forks_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            (fork_bonus + king_bonus, (fork_bonus + king_bonus) / 2)
        } else {
            (0, 0)
        }
    }

    /// Get list of enemy pieces attacked by a piece at given position
    fn get_attacked_pieces(&self, board: &BitboardBoard, pos: Position, piece_type: PieceType, player: Player) -> Vec<(PieceType, i32)> {
        let mut attacked = Vec::new();
        let opponent = player.opposite();

        // Get attack squares for this piece
        let attack_squares = self.get_piece_attacks(pos, piece_type, player);

        for target_pos in attack_squares {
            if let Some(target_piece) = board.get_piece(target_pos) {
                if target_piece.player == opponent {
                    let value = target_piece.piece_type.base_value() / 100;
                    attacked.push((target_piece.piece_type, value));
                }
            }
        }

        attacked
    }

    /// Get squares attacked by a piece
    fn get_piece_attacks(&self, pos: Position, piece_type: PieceType, player: Player) -> Vec<Position> {
        let mut attacks = Vec::new();

        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                // Rook attacks (ranks and files)
                for dir in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    self.add_sliding_attacks(&mut attacks, pos, dir);
                }
            },
            PieceType::Bishop | PieceType::PromotedBishop => {
                // Bishop attacks (diagonals)
                for dir in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                    self.add_sliding_attacks(&mut attacks, pos, dir);
                }
            },
            PieceType::Knight => {
                // Knight moves (L-shape)
                let knight_moves = if player == Player::Black {
                    vec![(-2, -1), (-2, 1)]
                } else {
                    vec![(2, -1), (2, 1)]
                };
                for (dr, dc) in knight_moves {
                    self.add_single_attack(&mut attacks, pos, dr, dc);
                }
            },
            PieceType::Gold | PieceType::PromotedPawn | PieceType::PromotedLance | 
            PieceType::PromotedKnight | PieceType::PromotedSilver => {
                // Gold-like movement
                let gold_moves = if player == Player::Black {
                    vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0)]
                } else {
                    vec![(1, -1), (1, 0), (1, 1), (0, -1), (0, 1), (-1, 0)]
                };
                for (dr, dc) in gold_moves {
                    self.add_single_attack(&mut attacks, pos, dr, dc);
                }
            },
            PieceType::Silver => {
                // Silver movement
                let silver_moves = if player == Player::Black {
                    vec![(-1, -1), (-1, 0), (-1, 1), (1, -1), (1, 1)]
                } else {
                    vec![(1, -1), (1, 0), (1, 1), (-1, -1), (-1, 1)]
                };
                for (dr, dc) in silver_moves {
                    self.add_single_attack(&mut attacks, pos, dr, dc);
                }
            },
            PieceType::Pawn => {
                // Pawn captures
                let dr = if player == Player::Black { -1 } else { 1 };
                self.add_single_attack(&mut attacks, pos, dr, 0);
            },
            PieceType::King => {
                // King moves (all 8 directions)
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr != 0 || dc != 0 {
                            self.add_single_attack(&mut attacks, pos, dr, dc);
                        }
                    }
                }
            },
            PieceType::Lance => {
                // Lance attacks (forward only)
                let dir = if player == Player::Black { (-1, 0) } else { (1, 0) };
                self.add_sliding_attacks(&mut attacks, pos, dir);
            },
        }

        attacks
    }

    /// Add sliding attacks in a direction
    fn add_sliding_attacks(&self, attacks: &mut Vec<Position>, pos: Position, dir: (i8, i8)) {
        let mut row = pos.row as i8 + dir.0;
        let mut col = pos.col as i8 + dir.1;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            attacks.push(Position::new(row as u8, col as u8));
            row += dir.0;
            col += dir.1;
        }
    }

    /// Add single attack move
    fn add_single_attack(&self, attacks: &mut Vec<Position>, pos: Position, dr: i8, dc: i8) {
        let new_row = pos.row as i8 + dr;
        let new_col = pos.col as i8 + dc;

        if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
            attacks.push(Position::new(new_row as u8, new_col as u8));
        }
    }

    // ===================================================================
    // PIN DETECTION
    // ===================================================================

    /// Detect pins (pieces that cannot move without exposing valuable piece)
    fn detect_pins(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.pin_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Find king position
        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check for pins along ranks, files, and diagonals
        mg_score += self.check_pins_in_directions(board, king_pos, player, &[(1, 0), (-1, 0), (0, 1), (0, -1)]);
        mg_score += self.check_pins_in_directions(board, king_pos, player, &[(1, 1), (-1, 1), (1, -1), (-1, -1)]);

        eg_score = mg_score / 2; // Pins slightly less critical in endgame

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check for pins in given directions
    fn check_pins_in_directions(&self, board: &BitboardBoard, king_pos: Position, player: Player, directions: &[(i8, i8)]) -> i32 {
        let mut pin_value = 0;
        let opponent = player.opposite();

        for &(dr, dc) in directions {
            let mut pieces_in_line = Vec::new();
            let mut row = king_pos.row as i8 + dr;
            let mut col = king_pos.col as i8 + dc;

            // Scan outward from king
            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);
                
                if let Some(piece) = board.get_piece(pos) {
                    pieces_in_line.push((pos, piece));
                    
                    // If we hit an enemy piece, check if it creates a pin
                    if piece.player == opponent {
                        if self.can_pin_along_line(piece.piece_type, dr, dc) {
                            // Check if exactly one friendly piece between king and attacker
                            if pieces_in_line.len() == 2 && pieces_in_line[0].1.player == player {
                                let pinned_value = pieces_in_line[0].1.piece_type.base_value() / 100;
                                pin_value += pinned_value * self.config.pin_penalty_factor / 100;
                                self.stats.pins_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                        break;
                    }
                }
                
                row += dr;
                col += dc;
            }
        }

        pin_value
    }

    /// Check if piece type can create pins along given direction
    fn can_pin_along_line(&self, piece_type: PieceType, dr: i8, dc: i8) -> bool {
        match piece_type {
            PieceType::Rook | PieceType::PromotedRook | PieceType::Lance => {
                // Can pin along ranks and files
                dr == 0 || dc == 0
            },
            PieceType::Bishop | PieceType::PromotedBishop => {
                // Can pin along diagonals
                dr.abs() == dc.abs()
            },
            _ => false,
        }
    }

    // ===================================================================
    // SKEWER DETECTION
    // ===================================================================

    /// Detect skewers (attacking through piece to hit more valuable target)
    fn detect_skewers(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.skewer_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Check each enemy sliding piece for skewer potential
        let opponent = player.opposite();
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        match piece.piece_type {
                            PieceType::Rook | PieceType::PromotedRook => {
                                mg_score += self.check_skewers_from_piece(board, pos, player, &[(1, 0), (-1, 0), (0, 1), (0, -1)]);
                            },
                            PieceType::Bishop | PieceType::PromotedBishop => {
                                mg_score += self.check_skewers_from_piece(board, pos, player, &[(1, 1), (-1, 1), (1, -1), (-1, -1)]);
                            },
                            _ => {},
                        }
                    }
                }
            }
        }

        eg_score = mg_score / 2;
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check for skewers from a specific piece position
    fn check_skewers_from_piece(&self, board: &BitboardBoard, pos: Position, player: Player, directions: &[(i8, i8)]) -> i32 {
        let mut skewer_value = 0;

        for &(dr, dc) in directions {
            let mut pieces_in_line = Vec::new();
            let mut row = pos.row as i8 + dr;
            let mut col = pos.col as i8 + dc;

            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let check_pos = Position::new(row as u8, col as u8);
                
                if let Some(piece) = board.get_piece(check_pos) {
                    if piece.player == player {
                        pieces_in_line.push(piece);
                        
                        // Check if we have a skewer (2 pieces, second more valuable)
                        if pieces_in_line.len() == 2 {
                            let val1 = pieces_in_line[0].piece_type.base_value();
                            let val2 = pieces_in_line[1].piece_type.base_value();
                            
                            if val2 > val1 {
                                skewer_value += (val2 - val1) * self.config.skewer_bonus_factor / 10000;
                                self.stats.skewers_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            break;
                        }
                    } else {
                        // Hit opponent piece, stop
                        break;
                    }
                }
                
                row += dr;
                col += dc;
            }
        }

        skewer_value
    }

    // ===================================================================
    // DISCOVERED ATTACK DETECTION
    // ===================================================================

    /// Detect discovered attack potential
    fn detect_discovered_attacks(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.discovered_checks += 1;

        let mut mg_score = 0;
        let opponent = player.opposite();

        // Find opponent king
        let opp_king_pos = match self.find_king_position(board, opponent) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check if any of our pieces can create discovered attacks by moving
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        if self.can_create_discovered_attack(board, pos, opp_king_pos, player) {
                            mg_score += self.config.discovered_attack_bonus;
                            self.stats.discovered_attacks_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, mg_score / 2)
    }

    /// Check if moving a piece can create a discovered attack
    fn can_create_discovered_attack(&self, board: &BitboardBoard, piece_pos: Position, target_pos: Position, player: Player) -> bool {
        // Check if there's a friendly sliding piece behind this piece that would attack target
        let dr_sign = (target_pos.row as i8 - piece_pos.row as i8).signum();
        let dc_sign = (target_pos.col as i8 - piece_pos.col as i8).signum();

        if dr_sign == 0 && dc_sign == 0 {
            return false;
        }

        // Check if there's an attacking piece behind
        let mut row = piece_pos.row as i8 - dr_sign;
        let mut col = piece_pos.col as i8 - dc_sign;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let check_pos = Position::new(row as u8, col as u8);
            
            if let Some(piece) = board.get_piece(check_pos) {
                if piece.player == player {
                    // Check if this piece can attack along this line
                    return self.can_pin_along_line(piece.piece_type, dr_sign, dc_sign);
                } else {
                    return false;
                }
            }
            
            row -= dr_sign;
            col -= dc_sign;
        }

        false
    }

    // ===================================================================
    // KNIGHT FORK DETECTION
    // ===================================================================

    /// Detect knight fork patterns (special handling for knight's unique movement)
    fn detect_knight_forks(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.knight_fork_checks += 1;

        let mut mg_score = 0;
        let mut eg_score = 0;

        // Find all knights
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type == PieceType::Knight {
                        let fork_value = self.check_knight_for_forks(board, pos, player);
                        mg_score += fork_value;
                        eg_score += fork_value / 2;
                    }
                }
            }
        }

        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a knight is creating a fork
    fn check_knight_for_forks(&self, board: &BitboardBoard, pos: Position, player: Player) -> i32 {
        let targets = self.get_attacked_pieces(board, pos, PieceType::Knight, player);
        
        if targets.len() >= 2 {
            let total_value: i32 = targets.iter().map(|(_, value)| value).sum();
            let has_king = targets.iter().any(|(pt, _)| *pt == PieceType::King);
            
            let base_bonus = (total_value * self.config.knight_fork_bonus_factor) / 100;
            let king_bonus = if has_king { self.config.king_fork_bonus * 2 } else { 0 };
            
            self.stats.knight_forks_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            base_bonus + king_bonus
        } else {
            0
        }
    }

    // ===================================================================
    // BACK RANK THREAT DETECTION
    // ===================================================================

    /// Detect back rank threats (king trapped on back rank)
    fn detect_back_rank_threats(&mut self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.stats.back_rank_checks += 1;

        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // Check if king is on back rank
        let back_rank = if player == Player::Black { 8 } else { 0 };
        
        if king_pos.row != back_rank {
            return TaperedScore::default();
        }

        // Check if king is trapped (no escape squares)
        let escape_count = self.count_king_escape_squares(board, king_pos, player);
        
        if escape_count == 0 {
            // King is trapped - check for enemy threats on back rank
            let threats = self.count_back_rank_threats(board, king_pos, player);
            
            if threats > 0 {
                let penalty = threats * self.config.back_rank_threat_penalty;
                self.stats.back_rank_threats_found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return TaperedScore::new_tapered(-penalty, -penalty / 2);
            }
        }

        TaperedScore::default()
    }

    /// Count escape squares for king
    fn count_king_escape_squares(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> i32 {
        let mut count = 0;
        
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                
                let new_row = king_pos.row as i8 + dr;
                let new_col = king_pos.col as i8 + dc;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    
                    // Check if square is empty or has enemy piece
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player != player {
                            count += 1;
                        }
                    } else {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }

    /// Count enemy threats on back rank
    fn count_back_rank_threats(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> i32 {
        let mut threats = 0;
        let opponent = player.opposite();

        // Check for enemy rooks/promoted rooks on same rank
        for col in 0..9 {
            if col == king_pos.col {
                continue;
            }
            
            let pos = Position::new(king_pos.row, col);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == opponent {
                    match piece.piece_type {
                        PieceType::Rook | PieceType::PromotedRook => threats += 1,
                        _ => {},
                    }
                }
            }
        }

        threats
    }

    // ===================================================================
    // HELPER METHODS
    // ===================================================================

    /// Find king position for a player
    fn find_king_position(&self, board: &BitboardBoard, player: Player) -> Option<Position> {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::King && piece.player == player {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// Get statistics
    pub fn stats(&self) -> &TacticalStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = TacticalStats::default();
    }
}

impl Default for TacticalPatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for tactical pattern recognition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TacticalConfig {
    pub enable_forks: bool,
    pub enable_pins: bool,
    pub enable_skewers: bool,
    pub enable_discovered_attacks: bool,
    pub enable_knight_forks: bool,
    pub enable_back_rank_threats: bool,
    
    // Bonus/penalty factors (percentage)
    pub fork_bonus_factor: i32,
    pub knight_fork_bonus_factor: i32,
    pub king_fork_bonus: i32,
    pub pin_penalty_factor: i32,
    pub skewer_bonus_factor: i32,
    pub discovered_attack_bonus: i32,
    pub back_rank_threat_penalty: i32,
}

impl Default for TacticalConfig {
    fn default() -> Self {
        Self {
            enable_forks: true,
            enable_pins: true,
            enable_skewers: true,
            enable_discovered_attacks: true,
            enable_knight_forks: true,
            enable_back_rank_threats: true,
            
            fork_bonus_factor: 50,
            knight_fork_bonus_factor: 60,
            king_fork_bonus: 100,
            pin_penalty_factor: 40,
            skewer_bonus_factor: 30,
            discovered_attack_bonus: 80,
            back_rank_threat_penalty: 150,
        }
    }
}

/// Statistics for tactical pattern recognition
#[derive(Debug, Default)]
pub struct TacticalStats {
    pub evaluations: u64,
    pub fork_checks: u64,
    pub pin_checks: u64,
    pub skewer_checks: u64,
    pub discovered_checks: u64,
    pub knight_fork_checks: u64,
    pub back_rank_checks: u64,
    
    pub forks_found: std::sync::atomic::AtomicU64,
    pub pins_found: std::sync::atomic::AtomicU64,
    pub skewers_found: std::sync::atomic::AtomicU64,
    pub discovered_attacks_found: std::sync::atomic::AtomicU64,
    pub knight_forks_found: std::sync::atomic::AtomicU64,
    pub back_rank_threats_found: std::sync::atomic::AtomicU64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tactical_recognizer_creation() {
        let recognizer = TacticalPatternRecognizer::new();
        assert!(recognizer.config.enable_forks);
        assert!(recognizer.config.enable_pins);
    }

    #[test]
    fn test_tactical_config_default() {
        let config = TacticalConfig::default();
        assert_eq!(config.fork_bonus_factor, 50);
        assert_eq!(config.king_fork_bonus, 100);
    }

    #[test]
    fn test_fork_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        let score = recognizer.detect_forks(&board, Player::Black);
        assert!(score.mg >= 0);
        assert!(score.eg >= 0);
    }

    #[test]
    fn test_pin_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        let score = recognizer.detect_pins(&board, Player::Black);
        assert!(score.mg >= 0);
        assert!(score.eg >= 0);
    }

    #[test]
    fn test_knight_fork_detection() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        let score = recognizer.detect_knight_forks(&board, Player::Black);
        assert!(score.mg >= 0);
    }

    #[test]
    fn test_evaluate_tactics() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        let score = recognizer.evaluate_tactics(&board, Player::Black);
        assert_eq!(recognizer.stats().evaluations, 1);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        recognizer.evaluate_tactics(&board, Player::Black);
        
        let stats = recognizer.stats();
        assert!(stats.fork_checks >= 1);
        assert!(stats.pin_checks >= 1);
    }

    #[test]
    fn test_reset_statistics() {
        let mut recognizer = TacticalPatternRecognizer::new();
        let board = BitboardBoard::new();
        
        recognizer.evaluate_tactics(&board, Player::Black);
        recognizer.reset_stats();
        
        assert_eq!(recognizer.stats().evaluations, 0);
    }
}
