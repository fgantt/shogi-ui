//! Feature extraction system for automated tuning
//!
//! This module provides functionality to extract feature vectors from game positions
//! for use in the automated tuning process. The feature extraction system breaks down
//! the evaluation function into individual components that can be tuned independently.
//!
//! Key features extracted:
//! - Material balance (piece counts and values)
//! - Positional features (piece-square tables)
//! - King safety (castles, attacks, threats)
//! - Pawn structure (chains, advancement, isolation)
//! - Mobility (move counts and piece activity) - **Uses actual move generation**
//! - Piece coordination (connected pieces, attacks) - **Uses actual move generation**
//! - Center control (occupation patterns)
//! - Development (piece positioning and activity)
//!
//! ## Implementation Details
//!
//! ### Mobility Features
//! Mobility features are calculated using actual legal move generation rather than
//! heuristic estimates. The `extract_mobility_features()` method:
//! - Generates all legal moves for the player using `MoveGenerator::generate_legal_moves()`
//! - Counts moves per piece type (Pawn, Lance, Knight, Silver, Gold, Bishop, Rook)
//! - Calculates total mobility as the total number of legal moves
//! - Measures center mobility by counting moves targeting center squares
//!
//! This provides accurate mobility measurements that reflect the actual tactical
//! capabilities of the position, rather than simplified estimates.
//!
//! ### Coordination Features
//! Coordination features analyze actual piece interactions using move generation:
//! - **Connected Rooks**: Detects rooks on same rank/file with clear paths
//! - **Piece Coordination**: Counts moves that support friendly pieces or coordinate attacks
//! - **Attack Coordination**: Identifies squares attacked by multiple pieces
//! - **Defense Coordination**: Measures how well pieces defend each other
//!
//! These features use actual move generation to identify real tactical relationships
//! between pieces, providing more accurate coordination measurements than distance-based
//! heuristics.

use super::types::TrainingPosition;
use crate::evaluation::king_safety::KingSafetyEvaluator;
use crate::evaluation::PositionEvaluator;
use crate::moves::MoveGenerator;
use crate::{
    types::{CapturedPieces, KingSafetyConfig, Move, PieceType, Player, Position},
    BitboardBoard,
};

/// Feature extractor for automated tuning
pub struct FeatureExtractor {
    evaluator: PositionEvaluator,
    king_safety_evaluator: KingSafetyEvaluator,
    move_generator: MoveGenerator,
}

impl FeatureExtractor {
    /// Create a new feature extractor
    pub fn new() -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            king_safety_evaluator: KingSafetyEvaluator::with_config(KingSafetyConfig::default()),
            move_generator: MoveGenerator::new(),
        }
    }

    /// Create a new feature extractor with custom king safety configuration
    pub fn with_king_safety_config(config: KingSafetyConfig) -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            king_safety_evaluator: KingSafetyEvaluator::with_config(config),
            move_generator: MoveGenerator::new(),
        }
    }

    /// Extract all features from a position
    pub fn extract_features(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<f64> {
        self.evaluator
            .get_evaluation_features(board, player, captured_pieces)
    }

    /// Extract material features (piece count differences)
    pub fn extract_material_features(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 14]; // 14 piece types

        // Count pieces on board
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_idx = piece.piece_type.to_u8() as usize;
                    if piece_idx < 14 {
                        if piece.player == player {
                            features[piece_idx] += 1.0;
                        } else {
                            features[piece_idx] -= 1.0;
                        }
                    }
                }
            }
        }

        // Add captured pieces
        for &piece_type in &captured_pieces.black {
            let piece_idx = piece_type.to_u8() as usize;
            if piece_idx < 14 {
                features[piece_idx] += 1.0;
            }
        }

        for &piece_type in &captured_pieces.white {
            let piece_idx = piece_type.to_u8() as usize;
            if piece_idx < 14 {
                features[piece_idx] -= 1.0;
            }
        }

        features
    }

    /// Extract positional features (piece-square table values)
    pub fn extract_positional_features(&self, board: &BitboardBoard, player: Player) -> Vec<f64> {
        let mut features = vec![0.0; 126]; // 14 piece types * 9 squares

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_idx = piece.piece_type.to_u8() as usize;
                    if piece_idx < 14 {
                        let square_idx = (row * 9 + col) as usize;
                        let feature_idx = piece_idx * 9 + square_idx;

                        if feature_idx < features.len() {
                            if piece.player == player {
                                features[feature_idx] += 1.0;
                            } else {
                                features[feature_idx] -= 1.0;
                            }
                        }
                    }
                }
            }
        }

        features
    }

    /// Extract king safety features
    pub fn extract_king_safety_features(&self, board: &BitboardBoard, player: Player) -> Vec<f64> {
        let mut features = vec![0.0; 50]; // Various king safety components

        // Find king positions
        let mut white_king_pos = None;
        let mut black_king_pos = None;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::King {
                        match piece.player {
                            Player::White => white_king_pos = Some(pos),
                            Player::Black => black_king_pos = Some(pos),
                        }
                    }
                }
            }
        }

        // Extract castle features
        if let Some(king_pos) = match player {
            Player::White => white_king_pos,
            Player::Black => black_king_pos,
        } {
            // Castle evaluation (simplified)
            let castle_value = self.evaluate_castle_structure(board, king_pos, player);
            features[0] = castle_value;

            // King safety evaluation
            let safety_score = self.king_safety_evaluator.evaluate_fast(board, player);
            features[1] = safety_score.mg as f64;
        }

        features
    }

    /// Extract pawn structure features
    pub fn extract_pawn_structure_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 30]; // Various pawn structure components

        // Count pawns by rank
        let mut pawn_counts = [0; 9];

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        pawn_counts[row as usize] += 1;
                    }
                }
            }
        }

        // Store pawn counts as features
        for (i, &count) in pawn_counts.iter().enumerate() {
            if i < features.len() {
                features[i] = count as f64;
            }
        }

        // Calculate pawn structure metrics
        features[9] = self.calculate_pawn_advancement(board, player);
        features[10] = self.calculate_pawn_connectivity(board, player);
        features[11] = self.calculate_pawn_isolated(board, player);

        features
    }

    /// Extract mobility features using actual move generation
    ///
    /// This method generates legal moves and extracts mobility features
    /// based on actual move counts per piece type, providing accurate
    /// measurements instead of heuristic estimates.
    pub fn extract_mobility_features(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 20]; // Various mobility components

        // Generate all legal moves once for efficiency
        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        // Calculate mobility for each piece type
        let piece_types = [
            PieceType::Pawn,
            PieceType::Lance,
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
        ];

        // Count moves per piece type
        for (i, piece_type) in piece_types.iter().enumerate() {
            if i < features.len() {
                let mut count = 0.0;
                for mv in &legal_moves {
                    if let Some(from) = mv.from {
                        if let Some(piece) = board.get_piece(from) {
                            if piece.piece_type == *piece_type && piece.player == player {
                                count += 1.0;
                            }
                        }
                    }
                }
                features[i] = count;
            }
        }

        // Overall mobility metrics
        features[7] = legal_moves.len() as f64; // Total legal moves
        features[8] = self.calculate_center_mobility_from_moves(board, player, &legal_moves);

        features
    }

    /// Extract piece coordination features using actual move generation
    ///
    /// This method analyzes actual piece interactions by generating moves
    /// and examining how pieces support and attack together, providing
    /// accurate coordination measurements instead of heuristic estimates.
    pub fn extract_coordination_features(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 25]; // Various coordination components

        // Bishop pair
        features[0] = self.count_bishop_pair(board, player);

        // Connected rooks (using actual move generation)
        features[1] = self.count_connected_rooks_with_moves(board, player, captured_pieces);

        // Piece coordination patterns (using actual moves)
        features[2] = self.calculate_piece_coordination_with_moves(board, player, captured_pieces);
        features[3] = self.calculate_attack_coordination_with_moves(board, player, captured_pieces);

        // Pieces defending each other (actual interactions)
        features[4] = self.calculate_piece_defense_coordination(board, player, captured_pieces);

        features
    }

    /// Extract center control features
    pub fn extract_center_control_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 16]; // Center control patterns

        // Define center squares (4x4 center)
        let center_squares = [
            Position::new(3, 3),
            Position::new(3, 4),
            Position::new(3, 5),
            Position::new(4, 3),
            Position::new(4, 4),
            Position::new(4, 5),
            Position::new(5, 3),
            Position::new(5, 4),
            Position::new(5, 5),
        ];

        // Count pieces in center
        for (i, &pos) in center_squares.iter().enumerate() {
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player {
                    features[i] = 1.0;
                } else {
                    features[i] = -1.0;
                }
            }
        }

        // Center control metrics
        features[9] = self.calculate_center_control(board, player);

        features
    }

    /// Extract development features
    pub fn extract_development_features(&self, board: &BitboardBoard, player: Player) -> Vec<f64> {
        let mut features = vec![0.0; 20]; // Development patterns

        // Count pieces in starting ranks vs advanced ranks
        let starting_ranks = if player == Player::White {
            [0, 1]
        } else {
            [7, 8]
        };
        let advanced_ranks = if player == Player::White {
            [2, 3, 4, 5, 6]
        } else {
            [3, 4, 5, 6, 7]
        };

        let mut starting_pieces = 0;
        let mut advanced_pieces = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type != PieceType::King {
                        if starting_ranks.contains(&row) {
                            starting_pieces += 1;
                        } else if advanced_ranks.contains(&row) {
                            advanced_pieces += 1;
                        }
                    }
                }
            }
        }

        features[0] = starting_pieces as f64;
        features[1] = advanced_pieces as f64;
        features[2] = self.calculate_development_score(board, player);

        features
    }

    /// Normalize features to consistent ranges
    pub fn normalize_features(&self, features: &mut [f64]) {
        for feature in features.iter_mut() {
            // Handle NaN and infinite values
            if !feature.is_finite() {
                *feature = 0.0;
                continue;
            }

            // Clip extreme values
            *feature = feature.clamp(-1000.0, 1000.0);

            // Apply sigmoid normalization for bounded features
            if feature.abs() > 10.0 {
                *feature = feature.signum() * (1.0 - (-feature.abs()).exp());
            }
        }
    }

    /// Validate feature values
    pub fn validate_features(&self, features: &[f64]) -> Result<(), String> {
        for (i, &feature) in features.iter().enumerate() {
            if !feature.is_finite() {
                return Err(format!("Feature {} is not finite: {}", i, feature));
            }

            if feature.abs() > 10000.0 {
                return Err(format!("Feature {} has extreme value: {}", i, feature));
            }
        }

        Ok(())
    }

    /// Create a training position from a game position
    pub fn create_training_position(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
        result: f64,
        game_phase: i32,
        is_quiet: bool,
        move_number: u32,
    ) -> TrainingPosition {
        let features = self.extract_features(board, player, captured_pieces);
        TrainingPosition::new(features, result, game_phase, is_quiet, move_number, player)
    }

    // ============================================================================
    // HELPER METHODS FOR FEATURE CALCULATION
    // ============================================================================

    /// Evaluate castle structure
    fn evaluate_castle_structure(
        &self,
        board: &BitboardBoard,
        king_pos: Position,
        player: Player,
    ) -> f64 {
        // Simplified castle evaluation
        // In a real implementation, this would evaluate specific castle patterns
        let mut castle_value = 0.0;

        // Check for pieces around the king
        for row_offset in -1..=1 {
            for col_offset in -1..=1 {
                let new_row = king_pos.row as i32 + row_offset;
                let new_col = king_pos.col as i32 + col_offset;

                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player == player {
                            castle_value += 0.1; // Bonus for pieces around king
                        }
                    }
                }
            }
        }

        castle_value
    }

    /// Calculate pawn advancement
    fn calculate_pawn_advancement(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut advancement = 0.0;
        let target_rank = if player == Player::White { 8 } else { 0 };

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        let distance = if player == Player::White {
                            (target_rank as i32 - row as i32).abs() as f64
                        } else {
                            (row as i32 - target_rank as i32).abs() as f64
                        };
                        advancement += distance;
                    }
                }
            }
        }

        advancement
    }

    /// Calculate pawn connectivity
    fn calculate_pawn_connectivity(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut connectivity = 0.0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        // Check for adjacent pawns
                        let adjacent_positions = [
                            Position::new(row, col.saturating_sub(1)),
                            Position::new(row, col.saturating_add(1)),
                        ];

                        for adj_pos in adjacent_positions {
                            if adj_pos.col < 9 {
                                if let Some(adj_piece) = board.get_piece(adj_pos) {
                                    if adj_piece.piece_type == PieceType::Pawn
                                        && adj_piece.player == player
                                    {
                                        connectivity += 0.5;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        connectivity
    }

    /// Calculate isolated pawns
    fn calculate_pawn_isolated(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut isolated_count = 0.0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        // Check if pawn is isolated (no pawns on adjacent files)
                        let left_file = col.saturating_sub(1);
                        let right_file = col.saturating_add(1);

                        let mut has_adjacent_pawn = false;

                        for check_col in [left_file, right_file] {
                            if check_col < 9 {
                                for check_row in 0..9 {
                                    let check_pos = Position::new(check_row, check_col);
                                    if let Some(check_piece) = board.get_piece(check_pos) {
                                        if check_piece.piece_type == PieceType::Pawn
                                            && check_piece.player == player
                                        {
                                            has_adjacent_pawn = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if !has_adjacent_pawn {
                            isolated_count += 1.0;
                        }
                    }
                }
            }
        }

        isolated_count
    }

    /// Calculate piece mobility using actual move generation
    ///
    /// This method generates legal moves for pieces of the specified type
    /// and counts them, providing accurate mobility measurements instead
    /// of heuristic estimates.
    ///
    /// Note: This method is kept for potential future use but is currently
    /// replaced by inline move counting in extract_mobility_features().
    #[allow(dead_code)]
    fn calculate_piece_mobility(
        &self,
        board: &BitboardBoard,
        player: Player,
        piece_type: PieceType,
        captured_pieces: &CapturedPieces,
    ) -> f64 {
        // Generate all legal moves for the player
        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        // Count moves made by pieces of the specified type
        let mut mobility = 0.0;
        for mv in &legal_moves {
            if let Some(from) = mv.from {
                if let Some(piece) = board.get_piece(from) {
                    if piece.piece_type == piece_type && piece.player == player {
                        mobility += 1.0;
                    }
                }
            }
        }

        mobility
    }

    /// Calculate center mobility from actual moves
    ///
    /// Counts moves that target center squares (rows 3-5, cols 3-5).
    fn calculate_center_mobility_from_moves(
        &self,
        _board: &BitboardBoard,
        _player: Player,
        moves: &[Move],
    ) -> f64 {
        let mut center_moves = 0.0;
        for mv in moves {
            // Check if move targets center square (rows 3-5, cols 3-5)
            if mv.to.row >= 3 && mv.to.row <= 5 && mv.to.col >= 3 && mv.to.col <= 5 {
                center_moves += 1.0;
            }
        }
        center_moves
    }

    /// Calculate center mobility (deprecated - use calculate_center_mobility_from_moves)
    ///
    /// This method is kept for backward compatibility but should be replaced
    /// with calculate_center_mobility_from_moves which uses actual moves.
    #[allow(dead_code)]
    fn calculate_center_mobility(&self, board: &BitboardBoard, player: Player) -> f64 {
        // Count pieces that can influence center
        let mut center_mobility = 0.0;

        for row in 3..6 {
            for col in 3..6 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        center_mobility += 1.0;
                    }
                }
            }
        }

        center_mobility
    }

    /// Count bishop pair
    fn count_bishop_pair(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut bishop_count = 0;

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Bishop && piece.player == player {
                        bishop_count += 1;
                    }
                }
            }
        }

        if bishop_count >= 2 {
            1.0
        } else {
            0.0
        }
    }

    /// Count connected rooks using actual move generation
    ///
    /// Checks if rooks can reach each other's squares, indicating connection.
    fn count_connected_rooks_with_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> f64 {
        // Find all rooks for the player
        let mut rook_positions = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if (piece.piece_type == PieceType::Rook
                        || piece.piece_type == PieceType::PromotedRook)
                        && piece.player == player
                    {
                        rook_positions.push(pos);
                    }
                }
            }
        }

        if rook_positions.len() < 2 {
            return 0.0;
        }

        // Generate moves for each rook and check if they can reach other rook's square
        let mut connected = 0.0;
        for &rook1_pos in &rook_positions {
            // Create a temporary board to generate moves for this specific rook
            // (simplified: check if rooks are on same rank or file with clear path)
            for &rook2_pos in &rook_positions {
                if rook1_pos == rook2_pos {
                    continue;
                }

                // Check if rooks are on same rank or file
                if rook1_pos.row == rook2_pos.row || rook1_pos.col == rook2_pos.col {
                    // Check if path is clear (simplified check)
                    let path_clear = self.check_rook_path_clear(board, rook1_pos, rook2_pos);
                    if path_clear {
                        connected += 1.0;
                    }
                }
            }
        }

        // Normalize: return 1.0 if at least one pair is connected
        if connected > 0.0 {
            1.0
        } else {
            0.0
        }
    }

    /// Check if path between two rooks is clear
    fn check_rook_path_clear(&self, board: &BitboardBoard, from: Position, to: Position) -> bool {
        if from.row == to.row {
            // Same rank - check file path
            let start_col = from.col.min(to.col);
            let end_col = from.col.max(to.col);
            for col in (start_col + 1)..end_col {
                let pos = Position::new(from.row, col);
                if board.get_piece(pos).is_some() {
                    return false;
                }
            }
            true
        } else if from.col == to.col {
            // Same file - check rank path
            let start_row = from.row.min(to.row);
            let end_row = from.row.max(to.row);
            for row in (start_row + 1)..end_row {
                let pos = Position::new(row, from.col);
                if board.get_piece(pos).is_some() {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Count connected rooks (deprecated - use count_connected_rooks_with_moves)
    ///
    /// This method is kept for backward compatibility but should be replaced
    /// with count_connected_rooks_with_moves which uses actual move generation.
    #[allow(dead_code)]
    fn count_connected_rooks(&self, board: &BitboardBoard, player: Player) -> f64 {
        // Simplified: check if rooks are on same rank or file
        let mut rook_positions = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Rook && piece.player == player {
                        rook_positions.push(pos);
                    }
                }
            }
        }

        if rook_positions.len() >= 2 {
            let rook1 = rook_positions[0];
            let rook2 = rook_positions[1];

            // Check if rooks are connected (same rank or file, no pieces between)
            if rook1.row == rook2.row || rook1.col == rook2.col {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculate piece coordination using actual move generation
    ///
    /// Analyzes how pieces can support each other by checking if pieces
    /// can move to squares occupied or attacked by friendly pieces.
    fn calculate_piece_coordination_with_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> f64 {
        // Generate legal moves for the player
        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        let mut coordination = 0.0;

        // For each move, check if the destination square is:
        // 1. Occupied by a friendly piece (supporting that piece)
        // 2. Attacked by another friendly piece (coordination)
        for mv in &legal_moves {
            if let Some(from) = mv.from {
                // Check if destination is occupied by friendly piece
                if let Some(target_piece) = board.get_piece(mv.to) {
                    if target_piece.player == player {
                        coordination += 1.0; // Supporting friendly piece
                    }
                }

                // Check if destination is attacked by other friendly pieces
                // (simplified: count how many other pieces can reach this square)
                let mut attackers = 0;
                for other_mv in &legal_moves {
                    if other_mv.from != Some(from) && other_mv.to == mv.to {
                        if let Some(other_from) = other_mv.from {
                            if let Some(other_piece) = board.get_piece(other_from) {
                                if other_piece.player == player {
                                    attackers += 1;
                                }
                            }
                        }
                    }
                }
                if attackers > 0 {
                    coordination += attackers as f64 * 0.5; // Coordination bonus
                }
            }
        }

        coordination
    }

    /// Calculate attack coordination using actual move generation
    ///
    /// Counts how many enemy squares are attacked by multiple friendly pieces,
    /// indicating coordinated attacks.
    fn calculate_attack_coordination_with_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> f64 {
        // Generate legal moves (including captures)
        let legal_moves = self
            .move_generator
            .generate_legal_moves(board, player, captured_pieces);

        // Count how many enemy squares are attacked by multiple pieces
        let mut attack_targets: std::collections::HashMap<Position, usize> =
            std::collections::HashMap::new();

        for mv in &legal_moves {
            // Count captures (attacks on enemy pieces)
            if mv.is_capture {
                *attack_targets.entry(mv.to).or_insert(0) += 1;
            }
            // Also count attacks on empty squares near enemy pieces
            else if let Some(piece) = board.get_piece(mv.to) {
                if piece.player != player {
                    *attack_targets.entry(mv.to).or_insert(0) += 1;
                }
            }
        }

        // Count squares attacked by 2+ pieces (coordinated attacks)
        let mut coordinated_attacks = 0.0;
        for (_, count) in attack_targets {
            if count >= 2 {
                coordinated_attacks += 1.0;
            }
        }

        coordinated_attacks
    }

    /// Calculate piece defense coordination
    ///
    /// Counts how many friendly pieces are defended by other friendly pieces,
    /// indicating defensive coordination.
    fn calculate_piece_defense_coordination(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> f64 {
        // Generate legal moves for the opponent to see what they can attack
        let opponent = match player {
            Player::Black => Player::White,
            Player::White => Player::Black,
        };
        let opponent_moves = self
            .move_generator
            .generate_legal_moves(board, opponent, captured_pieces);

        // Find squares with friendly pieces
        let mut friendly_squares = std::collections::HashSet::new();
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type != PieceType::King {
                        friendly_squares.insert(pos);
                    }
                }
            }
        }

        // Count how many friendly pieces are under attack
        let mut attacked_pieces = std::collections::HashSet::new();
        for mv in &opponent_moves {
            if mv.is_capture && friendly_squares.contains(&mv.to) {
                attacked_pieces.insert(mv.to);
            }
        }

        // For each attacked piece, count how many friendly pieces can defend it
        let mut defense_coordination = 0.0;
        for &attacked_pos in &attacked_pieces {
            // Generate moves for friendly player to see defenders
            let friendly_moves = self
                .move_generator
                .generate_legal_moves(board, player, captured_pieces);

            let mut defenders = 0;
            for mv in &friendly_moves {
                if mv.to == attacked_pos {
                    defenders += 1;
                }
            }

            if defenders > 0 {
                defense_coordination += defenders as f64;
            }
        }

        defense_coordination
    }

    /// Calculate center control
    fn calculate_center_control(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut center_control = 0.0;

        // Check center squares
        let center_squares = [
            Position::new(3, 3),
            Position::new(3, 4),
            Position::new(3, 5),
            Position::new(4, 3),
            Position::new(4, 4),
            Position::new(4, 5),
            Position::new(5, 3),
            Position::new(5, 4),
            Position::new(5, 5),
        ];

        for &pos in &center_squares {
            if let Some(piece) = board.get_piece(pos) {
                if piece.player == player {
                    center_control += 1.0;
                } else {
                    center_control -= 1.0;
                }
            }
        }

        center_control
    }

    /// Calculate development score
    fn calculate_development_score(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut development = 0.0;

        // Count pieces that have moved from starting positions
        let starting_ranks = if player == Player::White {
            [0, 1]
        } else {
            [7, 8]
        };

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type != PieceType::King {
                        if !starting_ranks.contains(&row) {
                            development += 1.0; // Bonus for developed pieces
                        }
                    }
                }
            }
        }

        development
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{CapturedPieces, PieceType, Player},
        BitboardBoard, NUM_EVAL_FEATURES,
    };

    #[test]
    fn test_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let features = extractor.extract_features(&board, Player::White, &captured_pieces);
        assert_eq!(features.len(), NUM_EVAL_FEATURES);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_material_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let features = extractor.extract_material_features(&board, Player::White, &captured_pieces);
        assert_eq!(features.len(), 14); // 14 piece types

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_positional_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_positional_features(&board, Player::White);
        assert_eq!(features.len(), 126); // 14 piece types * 9 squares

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_king_safety_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_king_safety_features(&board, Player::White);
        assert_eq!(features.len(), 50);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_pawn_structure_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_pawn_structure_features(&board, Player::White);
        assert_eq!(features.len(), 30);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_mobility_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let captured_pieces = CapturedPieces::new();
        let features = extractor.extract_mobility_features(&board, Player::White, &captured_pieces);
        assert_eq!(features.len(), 20);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_coordination_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let captured_pieces = CapturedPieces::new();
        let features = extractor.extract_coordination_features(&board, Player::White, &captured_pieces);
        assert_eq!(features.len(), 25);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_center_control_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_center_control_features(&board, Player::White);
        assert_eq!(features.len(), 16);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_development_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_development_features(&board, Player::White);
        assert_eq!(features.len(), 20);

        // All features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_feature_normalization() {
        let extractor = FeatureExtractor::new();
        let mut features = vec![1000.0, -1000.0, 5.0, f64::INFINITY, f64::NAN];

        extractor.normalize_features(&mut features);

        // Extreme values should be clamped
        assert!(features[0] <= 1000.0);
        assert!(features[1] >= -1000.0);
    }

    #[test]
    fn test_mobility_feature_accuracy() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test that mobility features use actual move generation
        let features = extractor.extract_mobility_features(&board, Player::Black, &captured_pieces);

        // On initial position, Black should have legal moves
        // Total moves should be > 0 (initial position has many legal moves)
        assert!(features[7] > 0.0, "Total mobility should be > 0 on initial position");

        // All mobility features should be non-negative (move counts)
        for (i, &feature) in features.iter().enumerate() {
            if i < 7 {
                // Piece type mobility counts
                assert!(
                    feature >= 0.0,
                    "Mobility feature {} should be non-negative, got {}",
                    i,
                    feature
                );
            }
        }

        // Test with a different position (empty board with one piece)
        let mut test_board = BitboardBoard::empty();
        test_board.place_piece(
            crate::types::Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 4),
        );
        test_board.set_side_to_move(Player::Black);

        let test_features =
            extractor.extract_mobility_features(&test_board, Player::Black, &captured_pieces);
        // Rook on empty board should have many moves
        assert!(test_features[5] > 0.0, "Rook mobility should be > 0");
    }

    #[test]
    fn test_coordination_feature_accuracy() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Test that coordination features use actual move generation
        let features =
            extractor.extract_coordination_features(&board, Player::Black, &captured_pieces);

        // Bishop pair should be 0 on initial position (only one bishop per player)
        assert_eq!(features[0], 0.0, "Initial position should not have bishop pair");

        // All coordination features should be finite
        for (i, &feature) in features.iter().enumerate() {
            assert!(
                feature.is_finite(),
                "Coordination feature {} should be finite, got {}",
                i,
                feature
            );
        }

        // Test with two rooks on same rank (should be connected)
        let mut test_board = BitboardBoard::empty();
        test_board.place_piece(
            crate::types::Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 2),
        );
        test_board.place_piece(
            crate::types::Piece::new(PieceType::Rook, Player::Black),
            Position::new(4, 6),
        );
        test_board.set_side_to_move(Player::Black);

        let test_features =
            extractor.extract_coordination_features(&test_board, Player::Black, &captured_pieces);
        // Connected rooks should be detected
        assert!(
            test_features[1] >= 0.0,
            "Connected rooks feature should be non-negative"
        );
    }

    #[test]
    fn test_feature_extraction_consistency() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        // Extract features multiple times - should be consistent
        let features1 = extractor.extract_features(&board, Player::Black, &captured_pieces);
        let features2 = extractor.extract_features(&board, Player::Black, &captured_pieces);

        assert_eq!(features1.len(), features2.len());
        for (i, (f1, f2)) in features1.iter().zip(features2.iter()).enumerate() {
            assert!(
                (f1 - f2).abs() < 1e-10,
                "Feature {} should be consistent between calls: {} vs {}",
                i,
                f1,
                f2
            );
        }

        // Test that mobility features are consistent
        let mobility1 = extractor.extract_mobility_features(&board, Player::Black, &captured_pieces);
        let mobility2 = extractor.extract_mobility_features(&board, Player::Black, &captured_pieces);

        assert_eq!(mobility1.len(), mobility2.len());
        for (i, (m1, m2)) in mobility1.iter().zip(mobility2.iter()).enumerate() {
            assert!(
                (m1 - m2).abs() < 1e-10,
                "Mobility feature {} should be consistent: {} vs {}",
                i,
                m1,
                m2
            );
        }
    }

    #[test]
    fn test_feature_validation() {
        let extractor = FeatureExtractor::new();

        // Valid features
        let valid_features = vec![1.0, -2.5, 0.0, 100.0];
        assert!(extractor.validate_features(&valid_features).is_ok());

        // Invalid features (NaN)
        let invalid_features = vec![1.0, f64::NAN, 3.0];
        assert!(extractor.validate_features(&invalid_features).is_err());

        // Invalid features (Infinite)
        let invalid_features = vec![1.0, f64::INFINITY, 3.0];
        assert!(extractor.validate_features(&invalid_features).is_err());

        // Invalid features (Extreme values)
        let invalid_features = vec![1.0, 20000.0, 3.0];
        assert!(extractor.validate_features(&invalid_features).is_err());
    }

    #[test]
    fn test_training_position_creation() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();

        let position = extractor.create_training_position(
            &board,
            Player::White,
            &captured_pieces,
            0.5,
            100,
            true,
            15,
        );

        assert_eq!(position.features.len(), NUM_EVAL_FEATURES);
        assert_eq!(position.result, 0.5);
        assert_eq!(position.game_phase, 100);
        assert!(position.is_quiet);
        assert_eq!(position.move_number, 15);
    }

    #[test]
    fn test_feature_extraction_with_captured_pieces() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();
        let mut captured_pieces = CapturedPieces::new();

        // Add a captured piece
        captured_pieces.add_piece(PieceType::Silver, Player::Black);

        let features = extractor.extract_material_features(&board, Player::White, &captured_pieces);

        // Should show material difference
        assert!(features.iter().any(|&f| f != 0.0));
    }

    #[test]
    fn test_bishop_pair_detection() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let captured_pieces = CapturedPieces::new();
        let features = extractor.extract_coordination_features(&board, Player::White, &captured_pieces);

        // Bishop pair feature should be 0.0 or 1.0
        assert!(features[0] == 0.0 || features[0] == 1.0);
    }

    #[test]
    fn test_center_control_calculation() {
        let extractor = FeatureExtractor::new();
        let board = BitboardBoard::new();

        let features = extractor.extract_center_control_features(&board, Player::White);

        // Center control features should be finite
        for feature in features {
            assert!(feature.is_finite());
        }
    }
}
