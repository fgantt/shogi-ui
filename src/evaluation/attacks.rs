use crate::bitboards::*;
use crate::types::core::{PieceType, Player, Position};
use crate::types::evaluation::TaperedScore;
use crate::types::{Bitboard, EMPTY_BITBOARD, is_bit_set, set_bit};
use std::collections::HashMap;

/// Attack analyzer for evaluating threats to the king
pub struct AttackAnalyzer {
    config: AttackConfig,
    attack_tables: AttackTables,
}

/// Pre-computed attack tables for efficient piece attack generation
pub struct AttackTables {
    /// Attack bitboards for each piece type and position
    piece_attacks: HashMap<(PieceType, Position), Bitboard>,
    /// King attack zones for each position
    king_zones: HashMap<Position, Bitboard>,
}

impl AttackTables {
    /// Create new attack tables with pre-computed data
    pub fn new() -> Self {
        let mut tables = Self {
            piece_attacks: HashMap::new(),
            king_zones: HashMap::new(),
        };
        tables.initialize_tables();
        tables
    }

    /// Initialize all attack tables
    fn initialize_tables(&mut self) {
        // Initialize king zones for all positions
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                let zone = AttackZone::new(pos, 2);
                self.king_zones.insert(pos, zone.squares);
            }
        }

        // Initialize piece attacks for all positions and piece types
        let piece_types = vec![
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Knight,
            PieceType::Lance,
            PieceType::Pawn,
            PieceType::PromotedRook,
            PieceType::PromotedBishop,
        ];

        for piece_type in piece_types {
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    let attacks = self.generate_piece_attacks(piece_type, pos);
                    self.piece_attacks.insert((piece_type, pos), attacks);
                }
            }
        }
    }

    /// Generate attack bitboard for a piece at a given position
    fn generate_piece_attacks(&self, piece_type: PieceType, pos: Position) -> Bitboard {
        let mut attacks = EMPTY_BITBOARD;

        match piece_type {
            PieceType::Rook | PieceType::PromotedRook => {
                // Rook attacks in straight lines
                self.add_line_attacks(&mut attacks, pos, (1, 0)); // Right
                self.add_line_attacks(&mut attacks, pos, (-1, 0)); // Left
                self.add_line_attacks(&mut attacks, pos, (0, 1)); // Up
                self.add_line_attacks(&mut attacks, pos, (0, -1)); // Down
            }
            PieceType::Bishop | PieceType::PromotedBishop => {
                // Bishop attacks diagonally
                self.add_line_attacks(&mut attacks, pos, (1, 1)); // Up-right
                self.add_line_attacks(&mut attacks, pos, (-1, 1)); // Up-left
                self.add_line_attacks(&mut attacks, pos, (1, -1)); // Down-right
                self.add_line_attacks(&mut attacks, pos, (-1, -1)); // Down-left
            }
            PieceType::Gold => {
                // Gold attacks in 6 directions
                self.add_single_attacks(
                    &mut attacks,
                    pos,
                    &[(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1)],
                );
            }
            PieceType::Silver => {
                // Silver attacks in 5 directions
                self.add_single_attacks(
                    &mut attacks,
                    pos,
                    &[(1, 1), (-1, 1), (1, -1), (-1, -1), (0, 1)],
                );
            }
            PieceType::Knight => {
                // Knight attacks in L-shape
                self.add_single_attacks(&mut attacks, pos, &[(2, 1), (2, -1), (-2, 1), (-2, -1)]);
            }
            PieceType::Lance => {
                // Lance attacks forward only
                self.add_line_attacks(&mut attacks, pos, (0, 1));
            }
            PieceType::Pawn => {
                // Pawn attacks forward
                self.add_single_attacks(&mut attacks, pos, &[(0, 1)]);
            }
            _ => {}
        }

        attacks
    }

    /// Add line attacks in a given direction
    fn add_line_attacks(&self, attacks: &mut Bitboard, start_pos: Position, direction: (i8, i8)) {
        let (dr, dc) = direction;
        let mut row = start_pos.row as i8 + dr;
        let mut col = start_pos.col as i8 + dc;

        while row >= 0 && row < 9 && col >= 0 && col < 9 {
            let pos = Position::new(row as u8, col as u8);
            set_bit(attacks, pos);
            row += dr;
            col += dc;
        }
    }

    /// Add single square attacks
    fn add_single_attacks(
        &self,
        attacks: &mut Bitboard,
        start_pos: Position,
        directions: &[(i8, i8)],
    ) {
        for &(dr, dc) in directions {
            let row = start_pos.row as i8 + dr;
            let col = start_pos.col as i8 + dc;

            if row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);
                set_bit(attacks, pos);
            }
        }
    }

    /// Get attack bitboard for a piece at a position
    pub fn get_piece_attacks(&self, piece_type: PieceType, pos: Position) -> Bitboard {
        self.piece_attacks
            .get(&(piece_type, pos))
            .copied()
            .unwrap_or(EMPTY_BITBOARD)
    }

    /// Get king zone for a position
    pub fn get_king_zone(&self, pos: Position) -> Bitboard {
        self.king_zones.get(&pos).copied().unwrap_or(EMPTY_BITBOARD)
    }
}

/// Configuration for attack analysis
pub struct AttackConfig {
    pub enabled: bool,
    pub attack_zone_radius: u8,
    pub coordination_bonus: i32,
    pub double_attack_bonus: i32,
}

impl Default for AttackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            attack_zone_radius: 2,
            coordination_bonus: 50,
            double_attack_bonus: 30,
        }
    }
}

/// Attack zone around a position
pub struct AttackZone {
    pub center: Position,
    pub radius: u8,
    pub squares: Bitboard,
}

impl AttackZone {
    /// Create a new attack zone around the given center position
    pub fn new(center: Position, radius: u8) -> Self {
        let mut squares = EMPTY_BITBOARD;

        for dr in -(radius as i8)..=(radius as i8) {
            for dc in -(radius as i8)..=(radius as i8) {
                let new_row = center.row as i8 + dr;
                let new_col = center.col as i8 + dc;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    set_bit(&mut squares, pos);
                }
            }
        }

        Self {
            center,
            radius,
            squares,
        }
    }
}

/// Results of attack evaluation
pub struct AttackEvaluation {
    pub num_attackers: u8,
    pub attack_weight: i32,
    pub coordination_bonus: i32,
    pub tactical_threats: i32,
}

impl Default for AttackEvaluation {
    fn default() -> Self {
        Self {
            num_attackers: 0,
            attack_weight: 0,
            coordination_bonus: 0,
            tactical_threats: 0,
        }
    }
}

impl AttackAnalyzer {
    /// Create a new attack analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: AttackConfig::default(),
            attack_tables: AttackTables::new(),
        }
    }

    /// Create a new attack analyzer with custom configuration
    pub fn with_config(config: AttackConfig) -> Self {
        Self {
            config,
            attack_tables: AttackTables::new(),
        }
    }

    /// Evaluate attacks on the king for the given player
    pub fn evaluate_attacks(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        if !self.config.enabled {
            return TaperedScore::default();
        }

        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        let king_zone = self.attack_tables.get_king_zone(king_pos);
        let opponent = player.opposite();

        let mut evaluation = AttackEvaluation::default();

        // Analyze attacks in the king zone
        self.analyze_king_zone_attacks(board, king_pos, king_zone, opponent, &mut evaluation);

        // Analyze attack coordination
        self.analyze_attack_coordination(board, king_pos, opponent, &mut evaluation);

        self.convert_to_tapered_score(evaluation)
    }

    /// Analyze attacks in the king zone
    fn analyze_king_zone_attacks(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        king_zone: Bitboard,
        opponent: Player,
        evaluation: &mut AttackEvaluation,
    ) {
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if !is_bit_set(king_zone, pos) {
                    continue;
                }

                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        // Check if this piece attacks the king
                        let piece_attacks =
                            self.attack_tables.get_piece_attacks(piece.piece_type, pos);
                        if is_bit_set(piece_attacks, king_pos) {
                            evaluation.num_attackers += 1;
                            evaluation.attack_weight +=
                                self.get_piece_attack_value(piece.piece_type);
                        }
                    }
                }
            }
        }
    }

    /// Analyze attack coordination (rook-bishop, double attacks)
    fn analyze_attack_coordination(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
        evaluation: &mut AttackEvaluation,
    ) {
        let mut rook_attacks = EMPTY_BITBOARD;
        let mut bishop_attacks = EMPTY_BITBOARD;
        let mut double_attacks = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        let piece_attacks =
                            self.attack_tables.get_piece_attacks(piece.piece_type, pos);

                        if is_bit_set(piece_attacks, king_pos) {
                            match piece.piece_type {
                                PieceType::Rook | PieceType::PromotedRook => {
                                    rook_attacks |= piece_attacks;
                                }
                                PieceType::Bishop | PieceType::PromotedBishop => {
                                    bishop_attacks |= piece_attacks;
                                }
                                _ => {}
                            }

                            // Count double attacks (multiple pieces attacking same square)
                            for target_row in 0..9 {
                                for target_col in 0..9 {
                                    let target_pos = Position::new(target_row, target_col);
                                    if is_bit_set(piece_attacks, target_pos) {
                                        // Check if other pieces also attack this square
                                        let mut attackers = 0;
                                        for other_row in 0..9 {
                                            for other_col in 0..9 {
                                                let other_pos = Position::new(other_row, other_col);
                                                if let Some(other_piece) =
                                                    board.get_piece(other_pos)
                                                {
                                                    if other_piece.player == opponent
                                                        && other_pos != pos
                                                    {
                                                        let other_attacks =
                                                            self.attack_tables.get_piece_attacks(
                                                                other_piece.piece_type,
                                                                other_pos,
                                                            );
                                                        if is_bit_set(other_attacks, target_pos) {
                                                            attackers += 1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if attackers > 0 {
                                            double_attacks += attackers;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Rook-Bishop coordination bonus
        let coordination_squares = rook_attacks & bishop_attacks;
        if coordination_squares != EMPTY_BITBOARD {
            evaluation.coordination_bonus += self.config.coordination_bonus;
        }

        // Double attack bonus
        if double_attacks > 0 {
            evaluation.coordination_bonus +=
                self.config.double_attack_bonus * double_attacks as i32;
        }
    }

    /// Get attack value for a piece type
    fn get_piece_attack_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Rook => 100,
            PieceType::Bishop => 80,
            PieceType::PromotedRook => 120,
            PieceType::PromotedBishop => 100,
            PieceType::Silver => 60,
            PieceType::Gold => 70,
            PieceType::Knight => 50,
            PieceType::Lance => 40,
            PieceType::Pawn => 30,
            _ => 20,
        }
    }

    /// Convert attack evaluation to tapered score
    fn convert_to_tapered_score(&self, evaluation: AttackEvaluation) -> TaperedScore {
        let total_score =
            evaluation.attack_weight + evaluation.coordination_bonus + evaluation.tactical_threats;
        TaperedScore::new_tapered(-total_score, -total_score / 3)
    }

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
}

/// Threat evaluator for detecting tactical patterns
pub struct ThreatEvaluator {
    config: ThreatConfig,
    attack_tables: AttackTables,
}

/// Configuration for threat evaluation
pub struct ThreatConfig {
    pub enabled: bool,
    pub pin_penalty: i32,
    pub skewer_penalty: i32,
    pub fork_penalty: i32,
    pub discovered_attack_bonus: i32,
}

impl Default for ThreatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pin_penalty: 40,
            skewer_penalty: 35,
            fork_penalty: 50,
            discovered_attack_bonus: 30,
        }
    }
}

/// Types of tactical patterns
#[derive(Debug, Clone, PartialEq)]
pub enum TacticalType {
    Pin,
    Skewer,
    Fork,
    DiscoveredAttack,
    DoubleAttack,
}

/// Tactical pattern detection result
pub struct TacticalPattern {
    pub pattern_type: TacticalType,
    pub danger_level: u8, // 1-10 scale
    pub attacker_pos: Position,
    pub target_pos: Position,
    pub value: i32,
}

impl ThreatEvaluator {
    /// Create a new threat evaluator with default configuration
    pub fn new() -> Self {
        Self {
            config: ThreatConfig::default(),
            attack_tables: AttackTables::new(),
        }
    }

    /// Create a new threat evaluator with custom configuration
    pub fn with_config(config: ThreatConfig) -> Self {
        Self {
            config,
            attack_tables: AttackTables::new(),
        }
    }

    /// Evaluate tactical threats to the king for the given player
    pub fn evaluate_threats(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.evaluate_threats_with_mode(board, player, false)
    }

    /// Evaluate threats with fast mode support
    pub fn evaluate_threats_with_mode(
        &self,
        board: &BitboardBoard,
        player: Player,
        fast_mode: bool,
    ) -> TaperedScore {
        if !self.config.enabled {
            return TaperedScore::default();
        }

        let king_pos = match self.find_king_position(board, player) {
            Some(pos) => pos,
            None => return TaperedScore::default(),
        };

        // In fast mode, only check basic pins for performance
        if fast_mode {
            let opponent = player.opposite();
            let pin_threats = self.detect_pins(board, king_pos, opponent);
            return TaperedScore::new_tapered(-pin_threats, -pin_threats / 3);
        }

        let opponent = player.opposite();
        let mut total_threats = 0;

        // Detect pins
        total_threats += self.detect_pins(board, king_pos, opponent);

        // Detect skewers
        total_threats += self.detect_skewers(board, king_pos, opponent);

        // Detect forks
        total_threats += self.detect_forks(board, king_pos, opponent);

        // Detect discovered attacks
        total_threats += self.detect_discovered_attacks(board, king_pos, opponent);

        TaperedScore::new_tapered(-total_threats, -total_threats / 3)
    }

    /// Detect pins (pieces that cannot move because they protect the king)
    fn detect_pins(&self, board: &BitboardBoard, king_pos: Position, opponent: Player) -> i32 {
        let mut pin_value = 0;

        // Check for rook/queen pins
        pin_value += self.detect_line_pins(board, king_pos, opponent, PieceType::Rook);
        pin_value += self.detect_line_pins(board, king_pos, opponent, PieceType::PromotedRook);

        // Check for bishop/queen pins
        pin_value += self.detect_line_pins(board, king_pos, opponent, PieceType::Bishop);
        pin_value += self.detect_line_pins(board, king_pos, opponent, PieceType::PromotedBishop);

        pin_value * self.config.pin_penalty / 100
    }

    /// Detect line-based pins (rook, bishop)
    fn detect_line_pins(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
        piece_type: PieceType,
    ) -> i32 {
        let mut pins = 0;

        // Check all directions for this piece type
        let directions = match piece_type {
            PieceType::Rook | PieceType::PromotedRook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
            PieceType::Bishop | PieceType::PromotedBishop => {
                vec![(1, 1), (-1, 1), (1, -1), (-1, -1)]
            }
            _ => return 0,
        };

        for (dr, dc) in directions {
            let mut found_attacker = false;
            let mut pinned_pieces = 0;

            let mut row = king_pos.row as i8 + dr;
            let mut col = king_pos.col as i8 + dc;

            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);

                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent && piece.piece_type == piece_type {
                        found_attacker = true;
                        break;
                    } else if piece.player != opponent {
                        // Friendly piece in the line
                        if found_attacker {
                            pinned_pieces += 1;
                        }
                    }
                }

                row += dr;
                col += dc;
            }

            if found_attacker && pinned_pieces > 0 {
                pins += pinned_pieces;
            }
        }

        pins
    }

    /// Detect skewers (attacking through a piece to hit a more valuable target)
    fn detect_skewers(&self, board: &BitboardBoard, king_pos: Position, opponent: Player) -> i32 {
        let mut skewers = 0;

        // Similar to pins but looking for attacks through pieces
        skewers += self.detect_line_skewers(board, king_pos, opponent, PieceType::Rook);
        skewers += self.detect_line_skewers(board, king_pos, opponent, PieceType::Bishop);

        skewers * self.config.skewer_penalty / 100
    }

    /// Detect line-based skewers
    fn detect_line_skewers(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
        piece_type: PieceType,
    ) -> i32 {
        let mut skewers = 0;

        let directions = match piece_type {
            PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
            PieceType::Bishop => vec![(1, 1), (-1, 1), (1, -1), (-1, -1)],
            _ => return 0,
        };

        for (dr, dc) in directions {
            let mut found_attacker = false;
            let mut pieces_in_line = 0;

            let mut row = king_pos.row as i8 + dr;
            let mut col = king_pos.col as i8 + dc;

            while row >= 0 && row < 9 && col >= 0 && col < 9 {
                let pos = Position::new(row as u8, col as u8);

                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent && piece.piece_type == piece_type {
                        found_attacker = true;
                        break;
                    } else if piece.player != opponent {
                        pieces_in_line += 1;
                    }
                }

                row += dr;
                col += dc;
            }

            if found_attacker && pieces_in_line > 1 {
                skewers += pieces_in_line - 1;
            }
        }

        skewers
    }

    /// Detect forks (one piece attacking multiple targets)
    fn detect_forks(&self, board: &BitboardBoard, king_pos: Position, opponent: Player) -> i32 {
        let mut forks = 0;

        // Check for knight forks
        forks += self.detect_knight_forks(board, king_pos, opponent);

        // Check for other piece forks
        forks += self.detect_piece_forks(board, king_pos, opponent);

        forks * self.config.fork_penalty / 100
    }

    /// Detect knight forks - optimized version
    fn detect_knight_forks(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
    ) -> i32 {
        let mut forks = 0;

        // Only check knights in a reasonable area around the king
        let min_row = king_pos.row.saturating_sub(3);
        let max_row = (king_pos.row + 3).min(8);
        let min_col = king_pos.col.saturating_sub(3);
        let max_col = (king_pos.col + 3).min(8);

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent && piece.piece_type == PieceType::Knight {
                        let knight_attacks =
                            self.attack_tables.get_piece_attacks(PieceType::Knight, pos);
                        let mut targets = 0;

                        // Check if knight attacks the king
                        if is_bit_set(knight_attacks, king_pos) {
                            targets += 1;
                        }

                        // Check only pieces near the king for other targets
                        for target_row in min_row..=max_row {
                            for target_col in min_col..=max_col {
                                let target_pos = Position::new(target_row, target_col);
                                if target_pos != king_pos && is_bit_set(knight_attacks, target_pos)
                                {
                                    if let Some(target_piece) = board.get_piece(target_pos) {
                                        if target_piece.player != opponent {
                                            targets += 1;
                                        }
                                    }
                                }
                            }
                        }

                        if targets >= 2 {
                            forks += targets - 1;
                        }
                    }
                }
            }
        }

        forks
    }

    /// Detect forks with other pieces - optimized version
    fn detect_piece_forks(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
    ) -> i32 {
        let mut forks = 0;

        // Only check pieces in a reasonable area around the king
        let min_row = king_pos.row.saturating_sub(3);
        let max_row = (king_pos.row + 3).min(8);
        let min_col = king_pos.col.saturating_sub(3);
        let max_col = (king_pos.col + 3).min(8);

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        // Only check pieces that can create meaningful forks
                        match piece.piece_type {
                            PieceType::Rook
                            | PieceType::PromotedRook
                            | PieceType::Bishop
                            | PieceType::PromotedBishop
                            | PieceType::Silver
                            | PieceType::Gold => {
                                let piece_attacks =
                                    self.attack_tables.get_piece_attacks(piece.piece_type, pos);
                                let mut targets = 0;

                                // Check if piece attacks the king
                                if is_bit_set(piece_attacks, king_pos) {
                                    targets += 1;
                                }

                                // Check only pieces near the king for other targets
                                for target_row in min_row..=max_row {
                                    for target_col in min_col..=max_col {
                                        let target_pos = Position::new(target_row, target_col);
                                        if target_pos != king_pos
                                            && is_bit_set(piece_attacks, target_pos)
                                        {
                                            if let Some(target_piece) = board.get_piece(target_pos)
                                            {
                                                if target_piece.player != opponent {
                                                    targets += 1;
                                                }
                                            }
                                        }
                                    }
                                }

                                if targets >= 2 {
                                    forks += targets - 1;
                                }
                            }
                            _ => {} // Skip other pieces for performance
                        }
                    }
                }
            }
        }

        forks
    }

    /// Detect discovered attacks - optimized version
    fn detect_discovered_attacks(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        opponent: Player,
    ) -> i32 {
        let mut discovered_attacks = 0;

        // Only check pieces in a 5x5 area around the king for performance
        let min_row = king_pos.row.saturating_sub(2);
        let max_row = (king_pos.row + 2).min(8);
        let min_col = king_pos.col.saturating_sub(2);
        let max_col = (king_pos.col + 2).min(8);

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == opponent {
                        // Only check long-range pieces that could create discovered attacks
                        match piece.piece_type {
                            PieceType::Rook
                            | PieceType::PromotedRook
                            | PieceType::Bishop
                            | PieceType::PromotedBishop => {
                                let piece_attacks =
                                    self.attack_tables.get_piece_attacks(piece.piece_type, pos);
                                if is_bit_set(piece_attacks, king_pos) {
                                    discovered_attacks += 1;
                                }
                            }
                            _ => {} // Skip other pieces for performance
                        }
                    }
                }
            }
        }

        discovered_attacks * self.config.discovered_attack_bonus / 100
    }

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
}

impl Default for AttackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThreatEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_analyzer_creation() {
        let analyzer = AttackAnalyzer::new();
        assert!(analyzer.config.enabled);
        assert_eq!(analyzer.config.attack_zone_radius, 2);
        assert_eq!(analyzer.config.coordination_bonus, 50);
        assert_eq!(analyzer.config.double_attack_bonus, 30);
    }

    #[test]
    fn test_attack_zone_creation() {
        let center = Position::new(4, 4);
        let zone = AttackZone::new(center, 2);

        assert_eq!(zone.center, center);
        assert_eq!(zone.radius, 2);
        // The squares bitboard should have bits set for the 5x5 area around the center
        assert!(zone.squares != EMPTY_BITBOARD);
    }

    #[test]
    fn test_attack_evaluation_creation() {
        let evaluation = AttackEvaluation::default();
        assert_eq!(evaluation.num_attackers, 0);
        assert_eq!(evaluation.attack_weight, 0);
        assert_eq!(evaluation.coordination_bonus, 0);
        assert_eq!(evaluation.tactical_threats, 0);
    }

    #[test]
    fn test_piece_attack_values() {
        let analyzer = AttackAnalyzer::new();

        assert_eq!(analyzer.get_piece_attack_value(PieceType::Rook), 100);
        assert_eq!(analyzer.get_piece_attack_value(PieceType::Bishop), 80);
        assert_eq!(
            analyzer.get_piece_attack_value(PieceType::PromotedRook),
            120
        );
        assert_eq!(analyzer.get_piece_attack_value(PieceType::Pawn), 30);
    }

    #[test]
    fn test_attack_tables_creation() {
        let tables = AttackTables::new();

        // Test that tables are initialized
        let center = Position::new(4, 4);
        let rook_attacks = tables.get_piece_attacks(PieceType::Rook, center);
        let king_zone = tables.get_king_zone(center);

        assert!(rook_attacks != EMPTY_BITBOARD);
        assert!(king_zone != EMPTY_BITBOARD);
    }

    #[test]
    fn test_threat_evaluator_creation() {
        let evaluator = ThreatEvaluator::new();
        assert!(evaluator.config.enabled);
        assert_eq!(evaluator.config.pin_penalty, 40);
        assert_eq!(evaluator.config.skewer_penalty, 35);
        assert_eq!(evaluator.config.fork_penalty, 50);
        assert_eq!(evaluator.config.discovered_attack_bonus, 30);
    }

    #[test]
    fn test_tactical_type_enum() {
        let pin_type = TacticalType::Pin;
        let skewer_type = TacticalType::Skewer;
        let fork_type = TacticalType::Fork;

        assert_eq!(pin_type, TacticalType::Pin);
        assert_eq!(skewer_type, TacticalType::Skewer);
        assert_eq!(fork_type, TacticalType::Fork);
    }

    #[test]
    fn test_threat_evaluation() {
        let evaluator = ThreatEvaluator::new();
        let board = BitboardBoard::new();
        let score = evaluator.evaluate_threats(&board, Player::Black);

        // Should return a score (may be non-zero due to discovered attack detection)
        assert!(score.mg <= 0); // Should be negative (threats are bad)
        assert!(score.eg <= 0);
    }
}
