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
//! - Mobility (move counts and piece activity)
//! - Piece coordination (connected pieces, attacks)
//! - Center control (occupation patterns)
//! - Development (piece positioning and activity)

use crate::evaluation::PositionEvaluator;
use crate::evaluation::king_safety::KingSafetyEvaluator;
use crate::{BitboardBoard, types::{Player, CapturedPieces, NUM_EVAL_FEATURES, PieceType, Position, KingSafetyConfig}};
use super::types::TrainingPosition;

/// Feature extractor for automated tuning
pub struct FeatureExtractor {
    evaluator: PositionEvaluator,
    king_safety_evaluator: KingSafetyEvaluator,
}

impl FeatureExtractor {
    /// Create a new feature extractor
    pub fn new() -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            king_safety_evaluator: KingSafetyEvaluator::with_config(KingSafetyConfig::default()),
        }
    }

    /// Create a new feature extractor with custom king safety configuration
    pub fn with_king_safety_config(config: KingSafetyConfig) -> Self {
        Self {
            evaluator: PositionEvaluator::new(),
            king_safety_evaluator: KingSafetyEvaluator::with_config(config),
        }
    }

    /// Extract all features from a position
    pub fn extract_features(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<f64> {
        self.evaluator.get_evaluation_features(board, player, captured_pieces)
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
    pub fn extract_positional_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
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
    pub fn extract_king_safety_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
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

    /// Extract mobility features
    pub fn extract_mobility_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 20]; // Various mobility components
        
        // Calculate mobility for each piece type
        let piece_types = [
            PieceType::Pawn, PieceType::Lance, PieceType::Knight,
            PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook
        ];
        
        for (i, piece_type) in piece_types.iter().enumerate() {
            if i < features.len() {
                features[i] = self.calculate_piece_mobility(board, player, *piece_type);
            }
        }
        
        // Overall mobility metrics
        features[7] = self.calculate_total_mobility(board, player);
        features[8] = self.calculate_center_mobility(board, player);
        
        features
    }

    /// Extract piece coordination features
    pub fn extract_coordination_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 25]; // Various coordination components
        
        // Bishop pair
        features[0] = self.count_bishop_pair(board, player);
        
        // Connected rooks
        features[1] = self.count_connected_rooks(board, player);
        
        // Piece coordination patterns
        features[2] = self.calculate_piece_coordination(board, player);
        features[3] = self.calculate_attack_coordination(board, player);
        
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
            Position::new(3, 3), Position::new(3, 4), Position::new(3, 5),
            Position::new(4, 3), Position::new(4, 4), Position::new(4, 5),
            Position::new(5, 3), Position::new(5, 4), Position::new(5, 5),
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
    pub fn extract_development_features(
        &self,
        board: &BitboardBoard,
        player: Player,
    ) -> Vec<f64> {
        let mut features = vec![0.0; 20]; // Development patterns
        
        // Count pieces in starting ranks vs advanced ranks
        let starting_ranks = if player == Player::White { [0, 1] } else { [7, 8] };
        let advanced_ranks = if player == Player::White { [2, 3, 4, 5, 6] } else { [3, 4, 5, 6, 7] };
        
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
    fn evaluate_castle_structure(&self, board: &BitboardBoard, king_pos: Position, player: Player) -> f64 {
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
                                    if adj_piece.piece_type == PieceType::Pawn && adj_piece.player == player {
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
                                        if check_piece.piece_type == PieceType::Pawn && check_piece.player == player {
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

    /// Calculate piece mobility
    fn calculate_piece_mobility(&self, board: &BitboardBoard, player: Player, piece_type: PieceType) -> f64 {
        // Simplified mobility calculation
        // In a real implementation, this would generate actual moves
        let mut mobility = 0.0;
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == piece_type && piece.player == player {
                        // Estimate mobility based on piece type and position
                        mobility += match piece_type {
                            PieceType::Pawn => 1.0,
                            PieceType::Lance => 2.0,
                            PieceType::Knight => 2.0,
                            PieceType::Silver => 3.0,
                            PieceType::Gold => 3.0,
                            PieceType::Bishop => 4.0,
                            PieceType::Rook => 4.0,
                            _ => 0.0,
                        };
                    }
                }
            }
        }
        
        mobility
    }

    /// Calculate total mobility
    fn calculate_total_mobility(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut total = 0.0;
        
        for piece_type in [PieceType::Pawn, PieceType::Lance, PieceType::Knight, 
                          PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook] {
            total += self.calculate_piece_mobility(board, player, piece_type);
        }
        
        total
    }

    /// Calculate center mobility
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
        
        if bishop_count >= 2 { 1.0 } else { 0.0 }
    }

    /// Count connected rooks
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

    /// Calculate piece coordination
    fn calculate_piece_coordination(&self, board: &BitboardBoard, player: Player) -> f64 {
        // Simplified coordination calculation
        let mut coordination = 0.0;
        
        // Count pieces that can support each other
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player && piece.piece_type != PieceType::King {
                        // Check for supporting pieces in adjacent squares
                        for row_offset in -1..=1 {
                            for col_offset in -1..=1 {
                                if row_offset == 0 && col_offset == 0 { continue; }
                                
                                let new_row = row as i32 + row_offset;
                                let new_col = col as i32 + col_offset;
                                
                                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                                    let support_pos = Position::new(new_row as u8, new_col as u8);
                                    if let Some(support_piece) = board.get_piece(support_pos) {
                                        if support_piece.player == player {
                                            coordination += 0.1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        coordination
    }

    /// Calculate attack coordination
    fn calculate_attack_coordination(&self, board: &BitboardBoard, player: Player) -> f64 {
        // Simplified attack coordination
        let mut attack_coordination = 0.0;
        
        // Count pieces that can attack the same squares
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player != player {
                        // Count how many of our pieces can attack this square
                        let mut attackers = 0;
                        
                        for attack_row in 0..9 {
                            for attack_col in 0..9 {
                                let attack_pos = Position::new(attack_row, attack_col);
                                if let Some(attack_piece) = board.get_piece(attack_pos) {
                                    if attack_piece.player == player {
                                        // Simplified: check if piece can attack (distance-based)
                                        let distance = ((row as i32 - attack_row as i32).abs() + 
                                                      (col as i32 - attack_col as i32).abs()) as u8;
                                        
                                        if distance <= 2 { // Within attack range
                                            attackers += 1;
                                        }
                                    }
                                }
                            }
                        }
                        
                        if attackers >= 2 {
                            attack_coordination += 1.0;
                        }
                    }
                }
            }
        }
        
        attack_coordination
    }

    /// Calculate center control
    fn calculate_center_control(&self, board: &BitboardBoard, player: Player) -> f64 {
        let mut center_control = 0.0;
        
        // Check center squares
        let center_squares = [
            Position::new(3, 3), Position::new(3, 4), Position::new(3, 5),
            Position::new(4, 3), Position::new(4, 4), Position::new(4, 5),
            Position::new(5, 3), Position::new(5, 4), Position::new(5, 5),
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
        let starting_ranks = if player == Player::White { [0, 1] } else { [7, 8] };
        
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
    use crate::{BitboardBoard, types::{Player, CapturedPieces, PieceType}};

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
        
        let features = extractor.extract_mobility_features(&board, Player::White);
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
        
        let features = extractor.extract_coordination_features(&board, Player::White);
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
        
        // Normal values should remain unchanged
        assert_eq!(features[2], 5.0);
        
        // Infinite and NaN values should be handled
        assert!(features[3].is_finite());
        assert!(features[4].is_finite());
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
        
        let features = extractor.extract_coordination_features(&board, Player::White);
        
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
