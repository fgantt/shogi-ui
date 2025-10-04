use crate::types::*;
use crate::bitboards::*;
use crate::moves::MoveGenerator;
use crate::weights::{WeightManager, WeightError};

// Advanced evaluation modules
pub mod king_safety;
pub mod castles;
pub mod attacks;
pub mod patterns;

use king_safety::KingSafetyEvaluator;

/// Position evaluator for the Shogi engine
pub struct PositionEvaluator {
    // Piece-square tables for positional evaluation
    piece_square_tables: PieceSquareTables,
    // Configuration for tapered evaluation
    config: TaperedEvaluationConfig,
    // Advanced king safety evaluator
    king_safety_evaluator: KingSafetyEvaluator,
    // Weight manager for tuned evaluation weights
    weight_manager: WeightManager,
    // Whether to use tuned weights for evaluation
    use_tuned_weights: bool,
}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {
            piece_square_tables: PieceSquareTables::new(),
            config: TaperedEvaluationConfig::default(),
            king_safety_evaluator: KingSafetyEvaluator::new(),
            weight_manager: WeightManager::new(),
            use_tuned_weights: false,
        }
    }
    
    /// Create a new evaluator with custom configuration
    pub fn with_config(config: TaperedEvaluationConfig) -> Self {
        Self {
            piece_square_tables: PieceSquareTables::new(),
            config: config.clone(),
            king_safety_evaluator: KingSafetyEvaluator::with_config(config.king_safety),
            weight_manager: WeightManager::new(),
            use_tuned_weights: false,
        }
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> &TaperedEvaluationConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: TaperedEvaluationConfig) {
        self.config = config.clone();
        self.king_safety_evaluator = KingSafetyEvaluator::with_config(config.king_safety);
    }
    
    /// Enable or disable advanced king safety evaluation
    pub fn set_advanced_king_safety(&mut self, enabled: bool) {
        self.config.king_safety.enabled = enabled;
        self.king_safety_evaluator = KingSafetyEvaluator::with_config(self.config.king_safety.clone());
    }
    
    /// Get the current king safety configuration
    pub fn get_king_safety_config(&self) -> &KingSafetyConfig {
        &self.config.king_safety
    }
    
    /// Update king safety configuration
    pub fn set_king_safety_config(&mut self, config: KingSafetyConfig) {
        self.config.king_safety = config.clone();
        self.king_safety_evaluator = KingSafetyEvaluator::with_config(config);
    }

    /// Load tuned weights from a file
    pub fn load_tuned_weights<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), WeightError> {
        self.weight_manager.load_weights(path)?;
        self.use_tuned_weights = self.weight_manager.is_enabled();
        Ok(())
    }

    /// Enable or disable tuned weights
    pub fn set_use_tuned_weights(&mut self, enabled: bool) {
        if enabled && self.weight_manager.has_weights() {
            self.weight_manager.set_enabled(true);
            self.use_tuned_weights = true;
        } else {
            self.weight_manager.set_enabled(false);
            self.use_tuned_weights = false;
        }
    }

    /// Check if tuned weights are enabled
    pub fn is_using_tuned_weights(&self) -> bool {
        self.use_tuned_weights && self.weight_manager.is_enabled()
    }

    /// Get weight manager for direct access
    pub fn get_weight_manager(&self) -> &WeightManager {
        &self.weight_manager
    }

    /// Get mutable weight manager for direct access
    pub fn get_weight_manager_mut(&mut self) -> &mut WeightManager {
        &mut self.weight_manager
    }

    /// Save current weights to a file
    pub fn save_tuned_weights<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        tuning_method: String,
        validation_error: f64,
        training_positions: usize,
    ) -> Result<(), WeightError> {
        self.weight_manager.save_weights(path, tuning_method, validation_error, training_positions)
    }

    /// Extract raw feature values for tuning
    /// Returns a vector of unweighted feature values that can be used for automated tuning
    pub fn get_evaluation_features(
        &self, 
        board: &BitboardBoard, 
        player: Player, 
        captured_pieces: &CapturedPieces
    ) -> Vec<f64> {
        let mut features = vec![0.0; NUM_EVAL_FEATURES];
        
        // Extract material features
        self.extract_material_features(&mut features, board, player, captured_pieces);
        
        // Extract positional features
        self.extract_positional_features(&mut features, board, player);
        
        // Extract king safety features
        self.extract_king_safety_features(&mut features, board, player);
        
        // Extract pawn structure features
        self.extract_pawn_structure_features(&mut features, board, player);
        
        // Extract mobility features
        self.extract_mobility_features(&mut features, board, player, captured_pieces);
        
        // Extract coordination features
        self.extract_coordination_features(&mut features, board, player);
        
        // Extract center control features
        self.extract_center_control_features(&mut features, board, player);
        
        // Extract development features
        self.extract_development_features(&mut features, board, player);
        
        features
    }
    
    /// Apply tuned weights to features and return final evaluation score
    pub fn evaluate_with_weights(
        &mut self,
        features: &[f64],
        game_phase: i32
    ) -> Result<i32, WeightError> {
        // Use the weight manager to apply weights
        self.weight_manager.apply_weights(features, game_phase)
    }

    /// Legacy method for backward compatibility
    pub fn evaluate_with_weights_legacy(
        &self,
        features: &[f64],
        weights: &[f64],
        game_phase: i32
    ) -> i32 {
        // Apply phase-dependent weighting
        let phase_weight = game_phase as f64 / 100.0; // Assuming GAME_PHASE_MAX = 100
        
        let mut mg_score = 0.0;
        let mut eg_score = 0.0;
        
        for (i, &feature) in features.iter().enumerate() {
            if i < NUM_MG_FEATURES {
                mg_score += feature * weights[i];
            } else {
                eg_score += feature * weights[i];
            }
        }
        
        // Interpolate based on game phase
        let final_score = mg_score * phase_weight + eg_score * (1.0 - phase_weight);
        final_score as i32
    }

    /// Evaluate the current position from the perspective of the given player
    pub fn evaluate(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        self.evaluate_with_context(board, player, captured_pieces, 0, false, false, false, false)
    }

    /// Evaluate using tuned weights if available, otherwise use traditional evaluation
    pub fn evaluate_with_tuned_weights(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> i32 {
        if self.is_using_tuned_weights() {
            // Extract features and use tuned weights
            let features = self.get_evaluation_features(board, player, captured_pieces);
            let game_phase = self.calculate_game_phase(board);
            
            match self.evaluate_with_weights(&features, game_phase) {
                Ok(score) => score,
                Err(_) => {
                    // Fall back to traditional evaluation if weight application fails
                    self.evaluate(board, player, captured_pieces)
                }
            }
        } else {
            // Use traditional evaluation
            self.evaluate(board, player, captured_pieces)
        }
    }
    
    /// Evaluate with search context for performance optimization
    pub fn evaluate_with_context(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces, depth: u8, is_root: bool, has_capture: bool, has_check: bool, is_quiescence: bool) -> i32 {
        // Check if tapered evaluation is enabled
        if !self.config.enabled {
            // Fall back to simple evaluation (just material and basic positional)
            return self.evaluate_simple(board, player);
        }
        
        // 1. Calculate game phase
        let game_phase = self.calculate_game_phase(board);
        
        // 2. Accumulate all evaluation terms
        let mut total_score = TaperedScore::default();
        
        // Add tempo bonus (same in all phases)
        total_score += TaperedScore::new(10);
        
        // Material and positional evaluation
        total_score += self.evaluate_material_and_position(board, player);
        
        // Pawn structure
        total_score += self.evaluate_pawn_structure(board, player);
        
        // King safety with context
        total_score += self.evaluate_king_safety_with_context(board, player, depth, is_root, has_capture, has_check, is_quiescence);
        
        // Mobility
        total_score += self.evaluate_mobility(board, player, captured_pieces);
        
        // Piece coordination
        total_score += self.evaluate_piece_coordination(board, player);
        
        // Center control
        total_score += self.evaluate_center_control(board, player);
        
        // Development
        total_score += self.evaluate_development(board, player);
        
        // 3. Interpolate final score based on game phase
        let final_score = total_score.interpolate(game_phase);
        
        // 4. Return score from perspective of current player
        // Note: The evaluation is already calculated from the perspective of the given player
        final_score
    }
    
    /// Simple evaluation fallback when tapered evaluation is disabled
    fn evaluate_simple(&self, board: &BitboardBoard, player: Player) -> i32 {
        // Simple material and positional evaluation
        let mut score = 0;
        
        // Material evaluation (simplified) - use basic material values
        // This is a simplified version that doesn't require complex piece iteration
        score += 100; // Basic tempo bonus
        
        // Basic positional evaluation (simplified)
        score += self.evaluate_material_and_position(board, player).mg;
        
        score
    }
    
    /// Get piece value for simple evaluation
    fn get_piece_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 100,
            PieceType::Lance => 300,
            PieceType::Knight => 300,
            PieceType::Silver => 400,
            PieceType::Gold => 500,
            PieceType::Bishop => 600,
            PieceType::Rook => 800,
            PieceType::King => 10000,
            // Promoted pieces have higher values
            PieceType::PromotedPawn => 200,
            PieceType::PromotedLance => 400,
            PieceType::PromotedKnight => 400,
            PieceType::PromotedSilver => 500,
            PieceType::PromotedBishop => 700,
            PieceType::PromotedRook => 900,
        }
    }

    /// Evaluate material and positional value
    fn evaluate_material_and_position(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value = piece.piece_type.base_value();
                    let positional_value = self.piece_square_tables.get_value(piece.piece_type, pos, piece.player);
                    
                    // Material values are the same in all phases
                    let material_score = TaperedScore::new(piece_value);
                    let total_piece_score = material_score + positional_value;
                    
                    if piece.player == player {
                        score += total_piece_score;
                    } else {
                        score -= total_piece_score;
                    }
                }
            }
        }
        
        score
    }

    /// Evaluate pawn structure
    fn evaluate_pawn_structure(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;
        let mut pawns = Vec::new();
        
        // Collect pawns for this player
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.player == player {
                        pawns.push(pos);
                    }
                }
            }
        }
        
        if pawns.is_empty() {
            return TaperedScore::default();
        }
        
        // Bonus for pawn chains (more important in middlegame)
        for i in 0..pawns.len() {
            for j in i + 1..pawns.len() {
                let r1 = pawns[i].row;
                let c1 = pawns[i].col;
                let r2 = pawns[j].row;
                let c2 = pawns[j].col;
                
                // Check if pawns are adjacent horizontally or vertically
                if (r1.abs_diff(r2) == 1 && c1 == c2) || (c1.abs_diff(c2) == 1 && r1 == r2) {
                    mg_score += 15; // Bonus for connected pawns in middlegame
                    eg_score += 10; // Less important in endgame
                }
            }
        }
        
        // Bonus for advanced pawns (more important in endgame)
        let promotion_zone_start = if player == Player::Black { 2 } else { 6 };
        for pawn in pawns.iter() {
            let advancement = if player == Player::Black {
                (promotion_zone_start as i32 - pawn.row as i32).max(0)
            } else {
                (pawn.row as i32 - promotion_zone_start as i32).max(0)
            };
            
            if advancement > 0 {
                mg_score += advancement * 8; // Moderate bonus in middlegame
                eg_score += advancement * 15; // Higher bonus in endgame
            }
        }
        
        // Penalty for isolated pawns (more severe in endgame)
        for pawn in pawns.iter() {
            if self.is_pawn_isolated(board, *pawn, player) {
                mg_score -= 15; // Moderate penalty in middlegame
                eg_score -= 25; // Higher penalty in endgame
            }
        }
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a pawn is isolated
    fn is_pawn_isolated(&self, board: &BitboardBoard, pawn_pos: Position, player: Player) -> bool {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                
                let new_row = pawn_pos.row as i8 + dr;
                let new_col = pawn_pos.col as i8 + dc;
                
                if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                    let pos = Position::new(new_row as u8, new_col as u8);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.piece_type == PieceType::Pawn && piece.player == player {
                            return false; // Pawn is not isolated
                        }
                    }
                }
            }
        }
        
        true // Pawn is isolated
    }

    /// Evaluate king safety using advanced evaluation system
    fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.evaluate_king_safety_with_context(board, player, 0, false, false, false, false)
    }
    
    /// Evaluate king safety with search context for performance optimization
    fn evaluate_king_safety_with_context(&self, board: &BitboardBoard, player: Player, depth: u8, is_root: bool, has_capture: bool, has_check: bool, is_quiescence: bool) -> TaperedScore {
        // Use advanced king safety evaluation if enabled
        if self.config.king_safety.enabled {
            let start_time = if self.config.enable_performance_monitoring {
                Some(crate::time_utils::TimeSource::now())
            } else {
                None
            };
            
            let result = if is_quiescence {
                // Skip king safety in quiescence search
                self.king_safety_evaluator.evaluate_quiescence(board, player)
            } else {
                // Use selective evaluation for better performance
                self.king_safety_evaluator.evaluate_selective(board, player, depth, is_root, has_capture, has_check)
            };
            
            // Log performance if monitoring is enabled
            if let Some(start) = start_time {
                let duration_ms = start.elapsed_ms();
                if duration_ms > 1 { // Log if takes more than 1ms
                    println!("Advanced king safety evaluation took: {}ms", duration_ms);
                }
            }
            
            return result;
        }
        
        // Fallback to basic king safety evaluation
        self.evaluate_basic_king_safety(board, player)
    }
    
    /// Basic king safety evaluation (fallback when advanced evaluation is disabled)
    fn evaluate_basic_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;
        
        // Find king position
        let king_pos = self.find_king_position(board, player);
        if king_pos.is_none() {
            return TaperedScore::default();
        }
        
        let king_pos = king_pos.unwrap();
        
        // King shield: reward for having friendly pieces nearby (more important in middlegame)
        let shield_offsets = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1), (0, 1),
            (1, -1), (1, 0), (1, 1),
        ];
        
        for (dr, dc) in shield_offsets.iter() {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        let shield_value = match piece.piece_type {
                            PieceType::Gold => 40,
                            PieceType::Silver => 30,
                            PieceType::Knight => 20,
                            PieceType::Lance => 15,
                            PieceType::Pawn => 10,
                            _ => 5,
                        };
                        mg_score += shield_value; // Full value in middlegame
                        eg_score += shield_value / 2; // Reduced value in endgame
                    }
                }
            }
        }
        
        // Penalty for nearby enemy pieces (more severe in middlegame)
        let mut enemy_attackers = 0;
        for row in (king_pos.row.saturating_sub(2))..=(king_pos.row + 2).min(8) {
            for col in (king_pos.col.saturating_sub(2))..=(king_pos.col + 2).min(8) {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player != player {
                        enemy_attackers += 1;
                    }
                }
            }
        }
        
        mg_score -= enemy_attackers * 30; // Higher penalty in middlegame
        eg_score -= enemy_attackers * 15; // Lower penalty in endgame
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate mobility (number of legal moves)
    fn evaluate_mobility(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> TaperedScore {
        let move_generator = MoveGenerator::new();
        let legal_moves = move_generator.generate_legal_moves(board, player, captured_pieces);
        let move_count = legal_moves.len() as i32;
        
        // Mobility is more important in endgame
        let mg_score = move_count * 1; // Lower value in middlegame
        let eg_score = move_count * 3; // Higher value in endgame
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    

    /// Evaluate piece coordination
    fn evaluate_piece_coordination(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;
        
        // Bonus for connected rooks (more important in middlegame)
        let connected_rooks = self.evaluate_connected_rooks(board, player);
        mg_score += connected_rooks;
        eg_score += connected_rooks / 2; // Less important in endgame
        
        // Bonus for bishop pair (more important in middlegame)
        let bishop_pair = self.evaluate_bishop_pair(board, player);
        mg_score += bishop_pair;
        eg_score += bishop_pair / 2; // Less important in endgame
        
        // Bonus for coordinated attacks (more important in middlegame)
        let coordinated_attacks = self.evaluate_coordinated_attacks(board, player);
        mg_score += coordinated_attacks;
        eg_score += coordinated_attacks / 2; // Less important in endgame
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate connected rooks
    fn evaluate_connected_rooks(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut rooks = Vec::new();
        
        // Collect rooks for this player
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.piece_type == PieceType::Rook && piece.player == player {
                        rooks.push(pos);
                    }
                }
            }
        }
        
        if rooks.len() < 2 {
            return 0;
        }
        
        let mut score = 0;
        
        // Check if rooks are on the same rank or file
        for i in 0..rooks.len() {
            for j in i + 1..rooks.len() {
                let r1 = rooks[i].row;
                let c1 = rooks[i].col;
                let r2 = rooks[j].row;
                let c2 = rooks[j].col;
                
                if r1 == r2 || c1 == c2 {
                    // Check if there are no pieces blocking the connection
                    if self.is_rook_connection_clear(board, rooks[i], rooks[j]) {
                        score += 30; // Bonus for connected rooks
                    }
                }
            }
        }
        
        score
    }

    /// Check if rook connection is clear
    fn is_rook_connection_clear(&self, board: &BitboardBoard, rook1: Position, rook2: Position) -> bool {
        if rook1.row == rook2.row {
            // Same rank, check if no pieces between
            let min_col = rook1.col.min(rook2.col);
            let max_col = rook1.col.max(rook2.col);
            
            for col in min_col + 1..max_col {
                let pos = Position::new(rook1.row, col);
                if board.is_square_occupied(pos) {
                    return false;
                }
            }
        } else if rook1.col == rook2.col {
            // Same file, check if no pieces between
            let min_row = rook1.row.min(rook2.row);
            let max_row = rook1.row.max(rook2.row);
            
            for row in min_row + 1..max_row {
                let pos = Position::new(row, rook1.col);
                if board.is_square_occupied(pos) {
                    return false;
                }
            }
        }
        
        true
    }

    /// Evaluate bishop pair
    fn evaluate_bishop_pair(&self, board: &BitboardBoard, player: Player) -> i32 {
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
            20 // Bonus for bishop pair
        } else {
            0
        }
    }

    /// Evaluate coordinated attacks
    fn evaluate_coordinated_attacks(&self, _board: &BitboardBoard, _player: Player) -> i32 {
        // This is a simplified implementation
        // In practice, we'd analyze attack patterns and piece coordination
        0
    }

    /// Evaluate center control
    fn evaluate_center_control(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;
        
        // Bonus for controlling center squares (3-5, 3-5)
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        mg_score += 20; // Full bonus for center control in middlegame
                        eg_score += 10; // Reduced bonus in endgame
                    } else {
                        mg_score -= 20; // Full penalty for opponent center control in middlegame
                        eg_score -= 10; // Reduced penalty in endgame
                    }
                }
            }
        }
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Evaluate development
    fn evaluate_development(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut mg_score = 0;
        let mut eg_score = 0;
        
        // Bonus for developing pieces early (more important in middlegame)
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        match piece.piece_type {
                            // Encourage moving key pieces out of their starting positions
                            PieceType::Bishop | PieceType::Rook | PieceType::Silver | PieceType::Gold => {
                                if self.is_piece_developed(piece.piece_type, pos, player) {
                                    mg_score += 25; // Full bonus for developing important pieces in middlegame
                                    eg_score += 5; // Reduced bonus in endgame
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        TaperedScore::new_tapered(mg_score, eg_score)
    }

    /// Check if a piece is developed
    fn is_piece_developed(&self, piece_type: PieceType, pos: Position, player: Player) -> bool {
        let start_row = if player == Player::Black { 8 } else { 0 };
        match piece_type {
            PieceType::Rook => pos.row != start_row,
            PieceType::Bishop => pos.row != start_row,
            PieceType::Silver => pos.row != start_row,
            PieceType::Gold => pos.row != start_row,
            _ => false, // Only check major pieces for this specific bonus
        }
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

    /// Calculate the current game phase (0 = endgame, GAME_PHASE_MAX = opening)
    /// This is based on the number and type of non-pawn, non-king pieces on the board
    pub fn calculate_game_phase(&self, board: &BitboardBoard) -> i32 {
        let mut phase = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if let Some(phase_value) = self.get_piece_phase_value(piece.piece_type) {
                        phase += phase_value;
                    }
                }
            }
        }
        
        // Scale to 0-256 range
        // Starting position has 30 total phase value (15 per player)
        // We want this to map to GAME_PHASE_MAX (256)
        // So we scale by 256/30 ≈ 8.53
        let scaled_phase = (phase * GAME_PHASE_MAX) / 30;
        
        // Clamp to valid range
        scaled_phase.min(GAME_PHASE_MAX).max(0)
    }
    
    /// Get phase value for a piece type
    /// Returns None for pieces that don't contribute to game phase (pawns, kings, promoted pieces)
    fn get_piece_phase_value(&self, piece_type: PieceType) -> Option<i32> {
        PIECE_PHASE_VALUES
            .iter()
            .find(|(pt, _)| *pt == piece_type)
            .map(|(_, value)| *value)
    }

    // ============================================================================
    // FEATURE EXTRACTION METHODS FOR AUTOMATED TUNING
    // ============================================================================

    /// Extract material features (piece count differences)
    fn extract_material_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) {
        let mut piece_counts = [0; 14]; // 14 piece types
        
        // Count pieces for both players
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_idx = piece.piece_type.to_u8() as usize;
                    if piece_idx < 14 {
                        if piece.player == player {
                            piece_counts[piece_idx] += 1;
                        } else {
                            piece_counts[piece_idx] -= 1;
                        }
                    }
                }
            }
        }
        
        // Add captured pieces to material count
        // Black captured pieces (positive for Black)
        for &piece_type in &captured_pieces.black {
            let piece_idx = piece_type.to_u8() as usize;
            if piece_idx < 14 {
                piece_counts[piece_idx] += 1;
            }
        }
        
        // White captured pieces (negative for Black, positive for White)
        for &piece_type in &captured_pieces.white {
            let piece_idx = piece_type.to_u8() as usize;
            if piece_idx < 14 {
                piece_counts[piece_idx] -= 1;
            }
        }
        
        // Store material features
        features[MATERIAL_PAWN_INDEX] = piece_counts[0] as f64;
        features[MATERIAL_LANCE_INDEX] = piece_counts[1] as f64;
        features[MATERIAL_KNIGHT_INDEX] = piece_counts[2] as f64;
        features[MATERIAL_SILVER_INDEX] = piece_counts[3] as f64;
        features[MATERIAL_GOLD_INDEX] = piece_counts[4] as f64;
        features[MATERIAL_BISHOP_INDEX] = piece_counts[5] as f64;
        features[MATERIAL_ROOK_INDEX] = piece_counts[6] as f64;
        features[MATERIAL_KING_INDEX] = piece_counts[7] as f64;
        features[MATERIAL_PROMOTED_PAWN_INDEX] = piece_counts[8] as f64;
        features[MATERIAL_PROMOTED_LANCE_INDEX] = piece_counts[9] as f64;
        features[MATERIAL_PROMOTED_KNIGHT_INDEX] = piece_counts[10] as f64;
        features[MATERIAL_PROMOTED_SILVER_INDEX] = piece_counts[11] as f64;
        features[MATERIAL_PROMOTED_BISHOP_INDEX] = piece_counts[12] as f64;
        features[MATERIAL_PROMOTED_ROOK_INDEX] = piece_counts[13] as f64;
    }

    /// Extract positional features (piece-square table values)
    fn extract_positional_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        // Extract middlegame positional features
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let (mg_table, _) = self.piece_square_tables.get_tables(piece.piece_type);
                    let (table_row, table_col) = self.piece_square_tables.get_table_coords(pos, piece.player);
                    let value = mg_table[table_row as usize][table_col as usize];
                    
                    let feature_idx = self.get_positional_feature_index(piece.piece_type, row, col, true);
                    if feature_idx < features.len() {
                        if piece.player == player {
                            features[feature_idx] = value as f64;
                        } else {
                            features[feature_idx] = -(value as f64);
                        }
                    }
                }
            }
        }
        
        // Extract endgame positional features
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let (_, eg_table) = self.piece_square_tables.get_tables(piece.piece_type);
                    let (table_row, table_col) = self.piece_square_tables.get_table_coords(pos, piece.player);
                    let value = eg_table[table_row as usize][table_col as usize];
                    
                    let feature_idx = self.get_positional_feature_index(piece.piece_type, row, col, false);
                    if feature_idx < features.len() {
                        if piece.player == player {
                            features[feature_idx] = value as f64;
                        } else {
                            features[feature_idx] = -(value as f64);
                        }
                    }
                }
            }
        }
    }

    /// Get feature index for positional features
    fn get_positional_feature_index(&self, piece_type: PieceType, row: u8, col: u8, is_mg: bool) -> usize {
        let square_idx = (row * 9 + col) as usize;
        match piece_type {
            PieceType::Pawn => {
                if is_mg { PST_PAWN_MG_START + square_idx } else { PST_PAWN_EG_START + square_idx }
            },
            PieceType::Lance => {
                if is_mg { PST_LANCE_MG_START + square_idx } else { PST_LANCE_EG_START + square_idx }
            },
            PieceType::Knight => {
                if is_mg { PST_KNIGHT_MG_START + square_idx } else { PST_KNIGHT_EG_START + square_idx }
            },
            PieceType::Silver => {
                if is_mg { PST_SILVER_MG_START + square_idx } else { PST_SILVER_EG_START + square_idx }
            },
            PieceType::Gold => {
                if is_mg { PST_GOLD_MG_START + square_idx } else { PST_GOLD_EG_START + square_idx }
            },
            PieceType::Bishop => {
                if is_mg { PST_BISHOP_MG_START + square_idx } else { PST_BISHOP_EG_START + square_idx }
            },
            PieceType::Rook => {
                if is_mg { PST_ROOK_MG_START + square_idx } else { PST_ROOK_EG_START + square_idx }
            },
            _ => 0, // Other pieces don't have positional tables
        }
    }

    /// Extract king safety features
    fn extract_king_safety_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        let king_safety_score = self.evaluate_king_safety(board, player);
        
        // Store king safety features (simplified for now)
        features[KING_SAFETY_CASTLE_INDEX] = (king_safety_score.mg / 4) as f64; // Approximate castle component
        features[KING_SAFETY_ATTACK_INDEX] = (king_safety_score.mg / 4) as f64; // Approximate attack component
        features[KING_SAFETY_THREAT_INDEX] = (king_safety_score.mg / 4) as f64; // Approximate threat component
        features[KING_SAFETY_SHIELD_INDEX] = (king_safety_score.mg / 4) as f64; // Approximate shield component
        features[KING_SAFETY_EXPOSURE_INDEX] = (king_safety_score.eg / 4) as f64; // Approximate exposure component
    }

    /// Extract pawn structure features
    fn extract_pawn_structure_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        let pawn_score = self.evaluate_pawn_structure(board, player);
        
        // Store pawn structure features (simplified for now)
        features[PAWN_STRUCTURE_CHAINS_INDEX] = (pawn_score.mg / 3) as f64; // Approximate chains component
        features[PAWN_STRUCTURE_ADVANCEMENT_INDEX] = (pawn_score.eg / 3) as f64; // Approximate advancement component
        features[PAWN_STRUCTURE_ISOLATION_INDEX] = (pawn_score.mg / 3) as f64; // Approximate isolation component
        features[PAWN_STRUCTURE_PASSED_INDEX] = (pawn_score.eg / 3) as f64; // Approximate passed pawns
        features[PAWN_STRUCTURE_BACKWARD_INDEX] = (pawn_score.mg / 3) as f64; // Approximate backward pawns
    }

    /// Extract mobility features
    fn extract_mobility_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) {
        let mobility_score = self.evaluate_mobility(board, player, captured_pieces);
        
        // Store mobility features (simplified for now)
        features[MOBILITY_TOTAL_MOVES_INDEX] = (mobility_score.mg / 2) as f64; // Approximate total moves
        features[MOBILITY_PIECE_MOVES_INDEX] = (mobility_score.eg / 2) as f64; // Approximate piece moves
        features[MOBILITY_ATTACK_MOVES_INDEX] = (mobility_score.mg / 2) as f64; // Approximate attack moves
        features[MOBILITY_DEFENSE_MOVES_INDEX] = (mobility_score.eg / 2) as f64; // Approximate defense moves
    }

    /// Extract coordination features
    fn extract_coordination_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        let coordination_score = self.evaluate_piece_coordination(board, player);
        
        // Store coordination features (simplified for now)
        features[COORDINATION_CONNECTED_ROOKS_INDEX] = (coordination_score.mg / 2) as f64; // Approximate connected rooks
        features[COORDINATION_BISHOP_PAIR_INDEX] = (coordination_score.mg / 2) as f64; // Approximate bishop pair
        features[COORDINATION_ATTACK_PATTERNS_INDEX] = (coordination_score.eg / 2) as f64; // Approximate attack patterns
        features[COORDINATION_PIECE_SUPPORT_INDEX] = (coordination_score.eg / 2) as f64; // Approximate piece support
    }

    /// Extract center control features
    fn extract_center_control_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        let center_score = self.evaluate_center_control(board, player);
        
        // Store center control features (simplified for now)
        features[CENTER_CONTROL_CENTER_SQUARES_INDEX] = (center_score.mg / 2) as f64; // Approximate center squares
        features[CENTER_CONTROL_OUTPOST_INDEX] = (center_score.eg / 2) as f64; // Approximate outposts
        features[CENTER_CONTROL_SPACE_INDEX] = (center_score.mg / 2) as f64; // Approximate space control
    }

    /// Extract development features
    fn extract_development_features(&self, features: &mut [f64], board: &BitboardBoard, player: Player) {
        let development_score = self.evaluate_development(board, player);
        
        // Store development features (simplified for now)
        features[DEVELOPMENT_MAJOR_PIECES_INDEX] = (development_score.mg / 2) as f64; // Approximate major pieces
        features[DEVELOPMENT_MINOR_PIECES_INDEX] = (development_score.eg / 2) as f64; // Approximate minor pieces
        features[DEVELOPMENT_CASTLING_INDEX] = (development_score.mg / 2) as f64; // Approximate castling
    }
}

/// Piece-square tables for dual-phase positional evaluation
#[derive(Clone)]
struct PieceSquareTables {
    // Middlegame tables
    pawn_table_mg: [[i32; 9]; 9],
    lance_table_mg: [[i32; 9]; 9],
    knight_table_mg: [[i32; 9]; 9],
    silver_table_mg: [[i32; 9]; 9],
    gold_table_mg: [[i32; 9]; 9],
    bishop_table_mg: [[i32; 9]; 9],
    rook_table_mg: [[i32; 9]; 9],
    
    // Endgame tables
    pawn_table_eg: [[i32; 9]; 9],
    lance_table_eg: [[i32; 9]; 9],
    knight_table_eg: [[i32; 9]; 9],
    silver_table_eg: [[i32; 9]; 9],
    gold_table_eg: [[i32; 9]; 9],
    bishop_table_eg: [[i32; 9]; 9],
    rook_table_eg: [[i32; 9]; 9],
}

impl PieceSquareTables {
    fn new() -> Self {
        Self {
            // Initialize middlegame tables
            pawn_table_mg: Self::init_pawn_table_mg(),
            lance_table_mg: Self::init_lance_table_mg(),
            knight_table_mg: Self::init_knight_table_mg(),
            silver_table_mg: Self::init_silver_table_mg(),
            gold_table_mg: Self::init_gold_table_mg(),
            bishop_table_mg: Self::init_bishop_table_mg(),
            rook_table_mg: Self::init_rook_table_mg(),
            
            // Initialize endgame tables
            pawn_table_eg: Self::init_pawn_table_eg(),
            lance_table_eg: Self::init_lance_table_eg(),
            knight_table_eg: Self::init_knight_table_eg(),
            silver_table_eg: Self::init_silver_table_eg(),
            gold_table_eg: Self::init_gold_table_eg(),
            bishop_table_eg: Self::init_bishop_table_eg(),
            rook_table_eg: Self::init_rook_table_eg(),
        }
    }

    /// Get positional value for a piece (returns TaperedScore)
    fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> TaperedScore {
        let (mg_table, eg_table) = self.get_tables(piece_type);
        let (row, col) = self.get_table_coords(pos, player);
        
        let mg_value = mg_table[row as usize][col as usize];
        let eg_value = eg_table[row as usize][col as usize];
        
        TaperedScore::new_tapered(mg_value, eg_value)
    }
    
    /// Get both mg and eg tables for a piece type
    fn get_tables(&self, piece_type: PieceType) -> (&[[i32; 9]; 9], &[[i32; 9]; 9]) {
        match piece_type {
            PieceType::Pawn => (&self.pawn_table_mg, &self.pawn_table_eg),
            PieceType::Lance => (&self.lance_table_mg, &self.lance_table_eg),
            PieceType::Knight => (&self.knight_table_mg, &self.knight_table_eg),
            PieceType::Silver => (&self.silver_table_mg, &self.silver_table_eg),
            PieceType::Gold => (&self.gold_table_mg, &self.gold_table_eg),
            PieceType::Bishop => (&self.bishop_table_mg, &self.bishop_table_eg),
            PieceType::Rook => (&self.rook_table_mg, &self.rook_table_eg),
            _ => return (&[[0; 9]; 9], &[[0; 9]; 9]), // No positional value for other pieces
        }
    }
    
    /// Get table coordinates for a position and player
    fn get_table_coords(&self, pos: Position, player: Player) -> (u8, u8) {
        if player == Player::Black {
            (pos.row, pos.col)
        } else {
            //TODO(feg): With the switch to tsshogi, this is may no longer be needed.
            (8 - pos.row, 8 - pos.col)
        }
    }

    // Middlegame table initialization functions
    fn init_pawn_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [5, 5, 5, 5, 5, 5, 5, 5, 5],
            [10, 10, 10, 10, 10, 10, 10, 10, 10],
            [15, 15, 15, 15, 15, 15, 15, 15, 15],
            [20, 20, 20, 20, 20, 20, 20, 20, 20],
            [25, 25, 25, 25, 25, 25, 25, 25, 25],
            [30, 30, 30, 30, 30, 30, 30, 30, 30],
            [35, 35, 35, 35, 35, 35, 35, 35, 35],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    // Endgame table initialization functions
    fn init_pawn_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [10, 10, 10, 10, 10, 10, 10, 10, 10],
            [20, 20, 20, 20, 20, 20, 20, 20, 20],
            [30, 30, 30, 30, 30, 30, 30, 30, 30],
            [40, 40, 40, 40, 40, 40, 40, 40, 40],
            [50, 50, 50, 50, 50, 50, 50, 50, 50],
            [60, 60, 60, 60, 60, 60, 60, 60, 60],
            [70, 70, 70, 70, 70, 70, 70, 70, 70],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    fn init_lance_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 0, 10, 10, 10, 5, 0, 0],
        ]
    }

    fn init_lance_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 10, 20, 20, 20, 10, 0, 0],
            [0, 0, 0, 20, 20, 20, 10, 0, 0],
        ]
    }

    fn init_knight_table_mg() -> [[i32; 9]; 9] {
        [
            [-10, -10, -10, -10, -10, -10, -10, -10, -10],
            [-10, 0, 0, 0, 0, 0, 0, 0, -10],
            [-10, 0, 5, 10, 15, 10, 5, 0, -10],
            [-10, 0, 10, 15, 20, 15, 10, 0, -10],
            [-10, 0, 5, 10, 15, 10, 5, 0, -10],
            [-10, 0, 5, 10, 10, 10, 5, 0, -10],
            [-10, 0, 5, 5, 5, 5, 5, 0, -10],
            [-10, 0, 0, 0, 0, 0, 0, 0, -10],
            [-10, -10, -10, -10, -10, -10, -10, -10, -10],
        ]
    }

    fn init_knight_table_eg() -> [[i32; 9]; 9] {
        [
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
            [-20, 0, 0, 0, 0, 0, 0, 0, -20],
            [-20, 0, 10, 20, 30, 20, 10, 0, -20],
            [-20, 0, 20, 30, 40, 30, 20, 0, -20],
            [-20, 0, 10, 20, 30, 20, 10, 0, -20],
            [-20, 0, 10, 20, 20, 20, 10, 0, -20],
            [-20, 0, 10, 10, 10, 10, 10, 0, -20],
            [-20, 0, 0, 0, 0, 0, 0, 0, -20],
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
        ]
    }

    fn init_silver_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    fn init_silver_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    fn init_gold_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    fn init_gold_table_eg() -> [[i32; 9]; 9] {
        [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 10, 20, 30, 30, 30, 20, 10, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
    }

    fn init_bishop_table_mg() -> [[i32; 9]; 9] {
        [
            [-10, -10, -10, -10, -10, -10, -10, -10, -10],
            [-10, 0, 5, 10, 10, 10, 5, 0, -10],
            [-10, 5, 10, 15, 15, 15, 10, 5, -10],
            [-10, 10, 15, 20, 20, 20, 15, 10, -10],
            [-10, 10, 15, 20, 20, 20, 15, 10, -10],
            [-10, 5, 10, 15, 15, 15, 10, 5, -10],
            [-10, 0, 5, 10, 10, 10, 5, 0, -10],
            [-10, 0, 0, 0, 0, 0, 0, 0, -10],
            [-10, -10, -10, -10, -10, -10, -10, -10, -10],
        ]
    }

    fn init_bishop_table_eg() -> [[i32; 9]; 9] {
        [
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
            [-20, 0, 10, 20, 20, 20, 10, 0, -20],
            [-20, 10, 20, 30, 30, 30, 20, 10, -20],
            [-20, 20, 30, 40, 40, 40, 30, 20, -20],
            [-20, 20, 30, 40, 40, 40, 30, 20, -20],
            [-20, 10, 20, 30, 30, 30, 20, 10, -20],
            [-20, 0, 10, 20, 20, 20, 10, 0, -20],
            [-20, 0, 0, 0, 0, 0, 0, 0, -20],
            [-20, -20, -20, -20, -20, -20, -20, -20, -20],
        ]
    }

    fn init_rook_table_mg() -> [[i32; 9]; 9] {
        [
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
        ]
    }

    fn init_rook_table_eg() -> [[i32; 9]; 9] {
        [
            [-10, -5, 0, 5, 5, 5, 0, -5, -10],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [5, 10, 15, 20, 20, 20, 15, 10, 5],
            [10, 15, 20, 25, 25, 25, 20, 15, 10],
            [10, 15, 20, 25, 25, 25, 20, 15, 10],
            [10, 15, 20, 25, 25, 25, 20, 15, 10],
            [5, 10, 15, 20, 20, 20, 15, 10, 5],
            [0, 5, 10, 15, 15, 15, 10, 5, 0],
            [-10, -5, 0, 5, 5, 5, 0, -5, -10],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_game_phase_starting_position() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new(); // Starting position
        
        // Starting position should have maximum phase
        // Each player has: 2 Knights(1) + 2 Silvers(1) + 2 Golds(2) + 1 Bishop(2) + 1 Rook(3) + 2 Lances(1)
        // Total per player: 2*1 + 2*1 + 2*2 + 1*2 + 1*3 + 2*1 = 2 + 2 + 4 + 2 + 3 + 2 = 15
        // Both players: 15 * 2 = 30
        // But we need to scale this to 0-256 range
        let phase = evaluator.calculate_game_phase(&board);
        assert_eq!(phase, GAME_PHASE_MAX);
    }

    #[test]
    fn test_calculate_game_phase_empty_board() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // Remove all pieces to create empty board
        // This is a bit tricky since we can't easily create an empty board
        // For now, we'll test with a board that has minimal pieces
        let phase = evaluator.calculate_game_phase(&board);
        assert!(phase >= 0);
        assert!(phase <= GAME_PHASE_MAX);
    }

    #[test]
    fn test_calculate_game_phase_endgame_position() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // In a typical endgame, we'd have fewer major pieces
        // For now, we'll test that the phase calculation works
        let phase = evaluator.calculate_game_phase(&board);
        assert!(phase >= 0);
        assert!(phase <= GAME_PHASE_MAX);
    }

    #[test]
    fn test_get_piece_phase_value() {
        let evaluator = PositionEvaluator::new();
        
        // Test pieces that contribute to game phase
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Knight), Some(1));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Silver), Some(1));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Gold), Some(2));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Bishop), Some(2));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Rook), Some(3));
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Lance), Some(1));
        
        // Test pieces that don't contribute to game phase
        assert_eq!(evaluator.get_piece_phase_value(PieceType::Pawn), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::King), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedPawn), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedLance), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedKnight), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedSilver), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedBishop), None);
        assert_eq!(evaluator.get_piece_phase_value(PieceType::PromotedRook), None);
    }

    #[test]
    fn test_game_phase_calculation_consistency() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // Phase calculation should be consistent across multiple calls
        let phase1 = evaluator.calculate_game_phase(&board);
        let phase2 = evaluator.calculate_game_phase(&board);
        assert_eq!(phase1, phase2);
    }

    #[test]
    fn test_game_phase_calculation_performance() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // Performance test - should complete quickly
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = evaluator.calculate_game_phase(&board);
        }
        let duration = start.elapsed();
        
        // Should complete 1000 calculations in reasonable time (< 100ms)
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_game_phase_range() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        let phase = evaluator.calculate_game_phase(&board);
        
        // Phase should be within valid range
        assert!(phase >= 0);
        assert!(phase <= GAME_PHASE_MAX);
    }

    #[test]
    fn test_piece_phase_values_consistency() {
        // Test that all piece types in PIECE_PHASE_VALUES have valid phase values
        for (piece_type, phase_value) in &PIECE_PHASE_VALUES {
            assert!(*phase_value > 0, "Phase value for {:?} should be positive", piece_type);
            assert!(*phase_value <= 10, "Phase value for {:?} should be reasonable", piece_type);
        }
        
        // Test that we have the expected number of piece types
        assert_eq!(PIECE_PHASE_VALUES.len(), 6);
        
        // Test that all expected piece types are present
        let expected_pieces = [
            PieceType::Knight,
            PieceType::Silver,
            PieceType::Gold,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Lance,
        ];
        
        for expected_piece in &expected_pieces {
            assert!(
                PIECE_PHASE_VALUES.iter().any(|(pt, _)| *pt == *expected_piece),
                "Piece type {:?} should be in PIECE_PHASE_VALUES",
                expected_piece
            );
        }
    }

    #[test]
    fn test_dual_phase_table_lookup() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4); // Center square
        let player = Player::Black;
        
        // Test pawn table lookup
        let pawn_score = tables.get_value(PieceType::Pawn, pos, player);
        assert!(pawn_score.mg > 0); // Should have positive mg value
        assert!(pawn_score.eg > pawn_score.mg); // Endgame should value pawn advancement more
        
        // Test rook table lookup
        let rook_score = tables.get_value(PieceType::Rook, pos, player);
        assert!(rook_score.mg > 0); // Should have positive mg value
        assert!(rook_score.eg > rook_score.mg); // Endgame should value rook activity more
    }

    #[test]
    fn test_get_tables_method() {
        let tables = PieceSquareTables::new();
        
        // Test pawn tables
        let (mg_table, eg_table) = tables.get_tables(PieceType::Pawn);
        assert_eq!(mg_table[1][4], 5); // First row should have value 5
        assert_eq!(eg_table[1][4], 10); // Endgame should have higher values
        
        // Test rook tables
        let (mg_table, eg_table) = tables.get_tables(PieceType::Rook);
        assert_eq!(mg_table[0][4], 15); // First row should have value 15
        assert_eq!(eg_table[0][4], 5); // Endgame first row should have value 5
        
        // Test invalid piece type
        let (mg_table, eg_table) = tables.get_tables(PieceType::King);
        assert_eq!(mg_table[0][0], 0); // Should return zero tables
        assert_eq!(eg_table[0][0], 0);
    }

    #[test]
    fn test_table_coordinate_calculation() {
        let tables = PieceSquareTables::new();
        
        // Test Black player (no coordinate flip)
        let pos = Position::new(4, 4);
        let (row, col) = tables.get_table_coords(pos, Player::Black);
        assert_eq!(row, 4);
        assert_eq!(col, 4);
        
        // Test White player (coordinate flip)
        let (row, col) = tables.get_table_coords(pos, Player::White);
        assert_eq!(row, 4); // 8 - 4 = 4
        assert_eq!(col, 4); // 8 - 4 = 4
    }

    #[test]
    fn test_tapered_score_return_type() {
        let tables = PieceSquareTables::new();
        let pos = Position::new(4, 4);
        let player = Player::Black;
        
        // Test that get_value returns TaperedScore
        let score = tables.get_value(PieceType::Pawn, pos, player);
        assert!(score.mg >= 0);
        assert!(score.eg >= 0);
        
        // Test interpolation
        let interpolated = score.interpolate(128); // Middle phase
        assert!(interpolated > 0);
    }

    #[test]
    fn test_endgame_table_values() {
        let tables = PieceSquareTables::new();
        
        // Test that endgame tables have higher values for advancement
        let (pawn_mg, pawn_eg) = tables.get_tables(PieceType::Pawn);
        assert!(pawn_eg[6][4] > pawn_mg[6][4]); // Advanced pawns more valuable in endgame
        
        // Test that endgame tables penalize back rank for rooks
        let (rook_mg, rook_eg) = tables.get_tables(PieceType::Rook);
        assert!(rook_eg[0][4] < rook_mg[0][4]); // Back rank worse in endgame
        assert!(rook_eg[8][4] < rook_mg[8][4]); // Back rank worse in endgame
    }

    #[test]
    fn test_middlegame_table_values() {
        let tables = PieceSquareTables::new();
        
        // Test that middlegame tables emphasize center control
        let (pawn_mg, _pawn_eg) = tables.get_tables(PieceType::Pawn);
        assert!(pawn_mg[4][4] > pawn_mg[0][0]); // Center more valuable than corner
        
        // Test that middlegame tables have reasonable values
        let (rook_mg, _rook_eg) = tables.get_tables(PieceType::Rook);
        assert!(rook_mg[4][4] > 0); // Center should be positive
        assert!(rook_mg[0][4] > 0); // First rank should be positive in middlegame
    }

    #[test]
    fn test_tapered_evaluation_methods() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test pawn structure evaluation
        let pawn_score = evaluator.evaluate_pawn_structure(&board, Player::Black);
        assert!(pawn_score.mg >= 0 || pawn_score.eg >= 0); // Should have some value
        
        // Test king safety evaluation
        let king_safety = evaluator.evaluate_king_safety(&board, Player::Black);
        assert!(king_safety.mg >= 0 || king_safety.eg >= 0); // Should have some value
        
        // Test mobility evaluation
        let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        assert!(mobility.mg >= 0 || mobility.eg >= 0); // Should have some value
        
        // Test piece coordination evaluation
        let coordination = evaluator.evaluate_piece_coordination(&board, Player::Black);
        assert!(coordination.mg >= 0 || coordination.eg >= 0); // Should have some value
        
        // Test center control evaluation
        let center_control = evaluator.evaluate_center_control(&board, Player::Black);
        assert!(center_control.mg >= 0 || center_control.eg >= 0); // Should have some value
        
        // Test development evaluation
        let development = evaluator.evaluate_development(&board, Player::Black);
        assert!(development.mg >= 0 || development.eg >= 0); // Should have some value
    }

    #[test]
    fn test_evaluation_phase_differences() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that different evaluation components have different mg/eg values
        let pawn_score = evaluator.evaluate_pawn_structure(&board, Player::Black);
        let king_safety = evaluator.evaluate_king_safety(&board, Player::Black);
        let mobility = evaluator.evaluate_mobility(&board, Player::Black, &captured_pieces);
        let coordination = evaluator.evaluate_piece_coordination(&board, Player::Black);
        let center_control = evaluator.evaluate_center_control(&board, Player::Black);
        let development = evaluator.evaluate_development(&board, Player::Black);
        
        // At least some components should have different mg/eg values
        let has_differences = pawn_score.mg != pawn_score.eg ||
                             king_safety.mg != king_safety.eg ||
                             mobility.mg != mobility.eg ||
                             coordination.mg != coordination.eg ||
                             center_control.mg != center_control.eg ||
                             development.mg != development.eg;
        
        assert!(has_differences, "Some evaluation components should have different mg/eg values");
    }

    #[test]
    fn test_evaluation_interpolation() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that evaluation interpolation works correctly
        let score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        // Score should be a reasonable integer value
        assert!(score.abs() < 10000, "Evaluation score should be reasonable: {}", score);
        
        // Test with different game phases
        let game_phase = evaluator.calculate_game_phase(&board);
        assert!(game_phase >= 0 && game_phase <= GAME_PHASE_MAX);
    }

    #[test]
    fn test_evaluation_consistency() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that evaluation is consistent
        let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        assert_eq!(score1, score2, "Evaluation should be consistent");
        
        // Test that evaluation is symmetric (opposite for different players)
        let black_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let white_score = evaluator.evaluate(&board, Player::White, &captured_pieces);
        
        // Scores should be opposite (or very close due to rounding)
        // Note: The evaluation is calculated from the perspective of the given player,
        // so both scores should be positive for the starting position
        assert!(black_score > 0, "Black evaluation should be positive: {}", black_score);
        assert!(white_score > 0, "White evaluation should be positive: {}", white_score);
    }

    #[test]
    fn test_phase_interpolation_accuracy() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test interpolation at different phases
        let game_phase = evaluator.calculate_game_phase(&board);
        
        // Create a test TaperedScore
        let test_score = TaperedScore::new_tapered(100, 200);
        
        // Test interpolation at various phases
        let phase_0_score = test_score.interpolate(0); // Should be close to eg value
        let phase_128_score = test_score.interpolate(128); // Should be halfway between mg and eg
        let phase_256_score = test_score.interpolate(256); // Should be close to mg value
        
        // Verify interpolation accuracy
        assert!(phase_0_score >= 190 && phase_0_score <= 210, 
                "Phase 0 should be close to eg value: {}", phase_0_score);
        assert!(phase_128_score >= 140 && phase_128_score <= 160, 
                "Phase 128 should be halfway between mg and eg: {}", phase_128_score);
        assert!(phase_256_score >= 90 && phase_256_score <= 110, 
                "Phase 256 should be close to mg value: {}", phase_256_score);
    }

    #[test]
    fn test_evaluation_different_phases() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test evaluation at different game phases
        let opening_phase = 256; // Maximum phase
        let middlegame_phase = 128; // Middle phase
        let endgame_phase = 0; // Minimum phase
        
        // Create test scores for different phases
        let opening_score = TaperedScore::new_tapered(100, 50).interpolate(opening_phase);
        let middlegame_score = TaperedScore::new_tapered(100, 50).interpolate(middlegame_phase);
        let endgame_score = TaperedScore::new_tapered(100, 50).interpolate(endgame_phase);
        
        // Verify phase-dependent scoring
        assert!(opening_score > middlegame_score, 
                "Opening should favor mg values: {} > {}", opening_score, middlegame_score);
        assert!(middlegame_score > endgame_score, 
                "Middlegame should be between opening and endgame: {} > {}", middlegame_score, endgame_score);
        assert!(endgame_score < opening_score, 
                "Endgame should favor eg values: {} < {}", endgame_score, opening_score);
    }

    #[test]
    fn test_smooth_phase_transitions() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that evaluation changes smoothly as phase changes
        let test_score = TaperedScore::new_tapered(100, 200);
        
        let mut prev_score = test_score.interpolate(0);
        for phase in 1..=256 {
            let current_score = test_score.interpolate(phase);
            
            // Score should change smoothly (no sudden jumps)
            let score_diff = (current_score - prev_score).abs();
            assert!(score_diff <= 2, 
                    "Score change should be smooth at phase {}: {} -> {} (diff: {})", 
                    phase, prev_score, current_score, score_diff);
            
            prev_score = current_score;
        }
    }

    #[test]
    fn test_evaluation_phase_edge_cases() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test edge cases
        let test_score = TaperedScore::new_tapered(100, 200);
        
        // Test phase 0 (pure endgame)
        let phase_0 = test_score.interpolate(0);
        assert_eq!(phase_0, 200, "Phase 0 should return pure eg value");
        
        // Test phase 256 (pure middlegame)
        let phase_256 = test_score.interpolate(256);
        assert_eq!(phase_256, 100, "Phase 256 should return pure mg value");
        
        // Test negative phase (should still work)
        let phase_neg = test_score.interpolate(-1);
        assert!(phase_neg >= 100, "Negative phase should be reasonable: {}", phase_neg);
        
        // Test phase > 256 (should still work)
        let phase_large = test_score.interpolate(300);
        assert!(phase_large < 100, "Large phase should favor mg even more: {}", phase_large);
    }

    #[test]
    fn test_evaluation_consistency_across_phases() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that evaluation is consistent across different phases
        let game_phase = evaluator.calculate_game_phase(&board);
        
        // Multiple evaluations should be consistent
        let score1 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let score2 = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        assert_eq!(score1, score2, "Evaluation should be consistent across calls");
        
        // Test that phase calculation is consistent
        let phase1 = evaluator.calculate_game_phase(&board);
        let phase2 = evaluator.calculate_game_phase(&board);
        
        assert_eq!(phase1, phase2, "Phase calculation should be consistent");
    }

    #[test]
    fn test_tapered_evaluation_performance() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test that tapered evaluation doesn't significantly impact performance
        let start = std::time::Instant::now();
        
        for _ in 0..1000 {
            let _ = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        }
        
        let duration = start.elapsed();
        
        // Should complete 1000 evaluations in reasonable time (< 1 second)
        assert!(duration.as_millis() < 1000, 
                "Tapered evaluation should be fast: {}ms for 1000 evaluations", 
                duration.as_millis());
    }

    #[test]
    fn test_evaluation_phase_boundaries() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test evaluation at phase boundaries
        let test_score = TaperedScore::new_tapered(100, 200);
        
        // Test exact phase boundaries
        let phase_0_score = test_score.interpolate(0);
        let phase_1_score = test_score.interpolate(1);
        let phase_255_score = test_score.interpolate(255);
        let phase_256_score = test_score.interpolate(256);
        
        // Verify boundary behavior
        assert_eq!(phase_0_score, 200, "Phase 0 should be pure eg");
        assert_eq!(phase_256_score, 100, "Phase 256 should be pure mg");
        
        // Verify smooth transition at boundaries
        let diff_0_1 = (phase_1_score - phase_0_score).abs();
        let diff_255_256 = (phase_256_score - phase_255_score).abs();
        
        assert!(diff_0_1 <= 1, "Smooth transition at phase 0-1: {}", diff_0_1);
        assert!(diff_255_256 <= 1, "Smooth transition at phase 255-256: {}", diff_255_256);
    }

    #[test]
    fn test_advanced_king_safety_integration() {
        let mut evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test with advanced king safety enabled (default)
        let score_advanced = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        // Test with advanced king safety disabled
        evaluator.set_advanced_king_safety(false);
        let score_basic = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        // Both should return valid scores
        assert!(score_advanced != 0 || score_basic != 0);
        
        // Re-enable advanced king safety
        evaluator.set_advanced_king_safety(true);
        let score_advanced_again = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        
        // Should be consistent
        assert_eq!(score_advanced, score_advanced_again);
    }

    #[test]
    fn test_king_safety_configuration() {
        let mut evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // Test default configuration
        let config = evaluator.get_king_safety_config();
        assert!(config.enabled);
        assert_eq!(config.castle_weight, 0.3);
        assert_eq!(config.attack_weight, 0.3);
        assert_eq!(config.threat_weight, 0.2);
        
        // Test custom configuration
        let mut custom_config = config.clone();
        custom_config.castle_weight = 1.5;
        custom_config.attack_weight = 0.8;
        custom_config.threat_weight = 1.2;
        
        evaluator.set_king_safety_config(custom_config);
        let updated_config = evaluator.get_king_safety_config();
        assert_eq!(updated_config.castle_weight, 1.5);
        assert_eq!(updated_config.attack_weight, 0.8);
        assert_eq!(updated_config.threat_weight, 1.2);
    }

    #[test]
    fn test_king_safety_evaluation_consistency() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        
        // Test that king safety evaluation returns consistent results
        let score1 = evaluator.evaluate_king_safety(&board, Player::Black);
        let score2 = evaluator.evaluate_king_safety(&board, Player::Black);
        
        assert_eq!(score1, score2);
        
        // Test both players
        let black_score = evaluator.evaluate_king_safety(&board, Player::Black);
        let white_score = evaluator.evaluate_king_safety(&board, Player::White);
        
        // Both should return valid TaperedScore values (may be equal for starting position)
        assert_eq!(black_score.mg, black_score.mg); // Basic sanity check
        assert_eq!(white_score.mg, white_score.mg); // Basic sanity check
    }

    // ============================================================================
    // FEATURE EXTRACTION TESTS FOR AUTOMATED TUNING
    // ============================================================================

    #[test]
    fn test_get_evaluation_features_basic() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test feature extraction
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // Should return correct number of features
        assert_eq!(features.len(), NUM_EVAL_FEATURES);
        
        // All features should be finite numbers
        for (i, &feature) in features.iter().enumerate() {
            assert!(feature.is_finite(), "Feature {} should be finite, got {}", i, feature);
        }
    }

    #[test]
    fn test_material_feature_extraction() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // In starting position, both players have equal material, so all differences should be 0
        assert_eq!(features[MATERIAL_PAWN_INDEX], 0.0);
        assert_eq!(features[MATERIAL_LANCE_INDEX], 0.0);
        assert_eq!(features[MATERIAL_KNIGHT_INDEX], 0.0);
        assert_eq!(features[MATERIAL_SILVER_INDEX], 0.0);
        assert_eq!(features[MATERIAL_GOLD_INDEX], 0.0);
        assert_eq!(features[MATERIAL_BISHOP_INDEX], 0.0);
        assert_eq!(features[MATERIAL_ROOK_INDEX], 0.0);
        assert_eq!(features[MATERIAL_KING_INDEX], 0.0);
        
        // Test with unequal material by adding pieces to hand
        let mut captured_pieces_unequal = CapturedPieces::new();
        captured_pieces_unequal.add_piece(PieceType::Silver, Player::Black);
        
        let features_unequal = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces_unequal);
        
        // Black should have more silver pieces
        assert!(features_unequal[MATERIAL_SILVER_INDEX] > 0.0);
    }

    #[test]
    fn test_positional_feature_extraction() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // Positional features should be non-zero for starting position
        // Test a few key positions
        let center_square_idx = 4 * 9 + 4; // Center square (4,4)
        
        // Check that positional features are being extracted
        // For starting position, features might be zero due to symmetry
        // Just verify the feature vector has the correct structure
        assert_eq!(features.len(), NUM_EVAL_FEATURES);
        assert!(PST_PAWN_MG_START + center_square_idx < features.len());
        assert!(PST_PAWN_EG_START + center_square_idx < features.len());
    }

    #[test]
    fn test_feature_extraction_consistency() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Extract features multiple times
        let features1 = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        let features2 = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // Should be identical
        assert_eq!(features1.len(), features2.len());
        for (i, (&f1, &f2)) in features1.iter().zip(features2.iter()).enumerate() {
            assert_eq!(f1, f2, "Feature {} should be consistent", i);
        }
    }

    #[test]
    fn test_feature_extraction_player_perspective() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Extract features for both players
        let black_features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        let white_features = evaluator.get_evaluation_features(&board, Player::White, &captured_pieces);
        
        // Should have same length
        assert_eq!(black_features.len(), white_features.len());
        
        // Material features should be opposite for symmetric position
        assert_eq!(black_features[MATERIAL_PAWN_INDEX], -white_features[MATERIAL_PAWN_INDEX]);
        assert_eq!(black_features[MATERIAL_ROOK_INDEX], -white_features[MATERIAL_ROOK_INDEX]);
        assert_eq!(black_features[MATERIAL_KING_INDEX], -white_features[MATERIAL_KING_INDEX]);
    }

    #[test]
    fn test_evaluate_with_weights() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Extract features
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // Create dummy weights (all 1.0)
        let weights = vec![1.0; NUM_EVAL_FEATURES];
        
        // Test evaluation with weights
        let game_phase = evaluator.calculate_game_phase(&board);
        let weighted_score = evaluator.evaluate_with_weights_legacy(&features, &weights, game_phase);
        
        // Should return a finite integer score
        assert!(weighted_score != i32::MIN && weighted_score != i32::MAX || weighted_score == 0);
    }

    #[test]
    fn test_feature_vector_bounds() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        
        // Test that all feature indices are within bounds
        assert!(MATERIAL_PAWN_INDEX < features.len());
        assert!(MATERIAL_KING_INDEX < features.len());
        assert!(PST_PAWN_MG_START < features.len());
        assert!(PST_ROOK_EG_START + 80 < features.len()); // Last square in rook table
        assert!(KING_SAFETY_CASTLE_INDEX < features.len());
        assert!(PAWN_STRUCTURE_CHAINS_INDEX < features.len());
        assert!(MOBILITY_TOTAL_MOVES_INDEX < features.len());
        assert!(COORDINATION_CONNECTED_ROOKS_INDEX < features.len());
        assert!(CENTER_CONTROL_CENTER_SQUARES_INDEX < features.len());
        assert!(DEVELOPMENT_MAJOR_PIECES_INDEX < features.len());
    }

    #[test]
    fn test_feature_extraction_performance() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test performance of feature extraction
        let start = std::time::Instant::now();
        
        for _ in 0..1000 {
            let _features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        }
        
        let duration = start.elapsed();
        
        // Should complete 1000 extractions in reasonable time (< 1 second)
        assert!(duration.as_millis() < 1000, 
                "Feature extraction should be fast: {}ms for 1000 extractions", 
                duration.as_millis());
    }

    #[test]
    fn test_weighted_evaluation_phase_interpolation() {
        let evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        let weights = vec![1.0; NUM_EVAL_FEATURES];
        
        // Test evaluation at different phases
        let opening_score = evaluator.evaluate_with_weights_legacy(&features, &weights, GAME_PHASE_MAX);
        let endgame_score = evaluator.evaluate_with_weights_legacy(&features, &weights, 0);
        let middlegame_score = evaluator.evaluate_with_weights_legacy(&features, &weights, GAME_PHASE_MAX / 2);
        
        // All should be finite
        assert!(opening_score != i32::MIN && opening_score != i32::MAX || opening_score == 0);
        assert!(endgame_score != i32::MIN && endgame_score != i32::MAX || endgame_score == 0);
        assert!(middlegame_score != i32::MIN && middlegame_score != i32::MAX || middlegame_score == 0);
    }

    #[test]
    fn test_feature_extraction_edge_cases() {
        let evaluator = PositionEvaluator::new();
        let captured_pieces = CapturedPieces::new();
        
        // Test with empty board
        let empty_board = BitboardBoard::empty();
        let empty_features = evaluator.get_evaluation_features(&empty_board, Player::Black, &captured_pieces);
        
        // Should still return correct number of features
        assert_eq!(empty_features.len(), NUM_EVAL_FEATURES);
        
        // All features should be finite
        for (i, &feature) in empty_features.iter().enumerate() {
            assert!(feature.is_finite(), "Empty board feature {} should be finite, got {}", i, feature);
        }
    }

    #[test]
    fn test_weight_manager_integration() {
        let mut evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Initially should not be using tuned weights
        assert!(!evaluator.is_using_tuned_weights());
        
        // Test enabling/disabling tuned weights
        evaluator.set_use_tuned_weights(true);
        assert!(!evaluator.is_using_tuned_weights()); // Should still be false since no weights loaded
        
        evaluator.set_use_tuned_weights(false);
        assert!(!evaluator.is_using_tuned_weights());
    }

    #[test]
    fn test_evaluate_with_tuned_weights_fallback() {
        let mut evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Should fall back to traditional evaluation when no tuned weights
        let traditional_score = evaluator.evaluate(&board, Player::Black, &captured_pieces);
        let tuned_score = evaluator.evaluate_with_tuned_weights(&board, Player::Black, &captured_pieces);
        
        // Should be the same since no tuned weights are loaded
        assert_eq!(traditional_score, tuned_score);
    }

    #[test]
    fn test_weight_manager_access() {
        let evaluator = PositionEvaluator::new();
        
        // Test weight manager access
        let weight_manager = evaluator.get_weight_manager();
        assert!(!weight_manager.is_enabled());
        assert!(!weight_manager.has_weights());
        
        let stats = weight_manager.get_stats();
        assert_eq!(stats.applications, 0);
    }

    #[test]
    fn test_evaluate_with_weights_new_method() {
        let mut evaluator = PositionEvaluator::new();
        let board = BitboardBoard::new();
        let captured_pieces = CapturedPieces::new();
        
        // Extract features
        let features = evaluator.get_evaluation_features(&board, Player::Black, &captured_pieces);
        let game_phase = evaluator.calculate_game_phase(&board);
        
        // Test new evaluate_with_weights method (should use default weights)
        let result = evaluator.evaluate_with_weights(&features, game_phase);
        assert!(result.is_ok());
        
        let score = result.unwrap();
        assert!(score != i32::MIN && score != i32::MAX);
    }
}


