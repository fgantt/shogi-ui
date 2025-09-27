use crate::types::*;
use crate::bitboards::*;
use crate::evaluation::castles::CastleRecognizer;
use crate::evaluation::attacks::{AttackAnalyzer, ThreatEvaluator};
use std::collections::HashMap;

/// Main king safety evaluator that combines castle recognition, attack analysis, and threat evaluation
pub struct KingSafetyEvaluator {
    config: KingSafetyConfig,
    castle_recognizer: CastleRecognizer,
    attack_analyzer: AttackAnalyzer,
    threat_evaluator: ThreatEvaluator,
    // Performance optimization: cache for expensive operations
    evaluation_cache: std::cell::RefCell<HashMap<(u64, Player), TaperedScore>>,
    // Fast mode configuration
    fast_mode_threshold: u8,
}

impl KingSafetyEvaluator {
    /// Create a new king safety evaluator with default configuration
    pub fn new() -> Self {
        Self::with_config(KingSafetyConfig::default())
    }
    
    /// Create a new king safety evaluator with custom configuration
    pub fn with_config(config: KingSafetyConfig) -> Self {
        Self {
            castle_recognizer: CastleRecognizer::new(),
            attack_analyzer: AttackAnalyzer::new(),
            threat_evaluator: ThreatEvaluator::new(),
            evaluation_cache: std::cell::RefCell::new(HashMap::new()),
            fast_mode_threshold: 6, // Use fast mode for depth >= 6
            config,
        }
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> &KingSafetyConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: KingSafetyConfig) {
        self.config = config;
    }
    
    /// Main evaluation function that combines all king safety components
    pub fn evaluate(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        self.evaluate_with_depth(board, player, 0)
    }
    
    /// Evaluate with depth information for performance optimization
    pub fn evaluate_with_depth(&self, board: &BitboardBoard, player: Player, depth: u8) -> TaperedScore {
        if !self.config.enabled {
            return TaperedScore::default();
        }
        
        // Check cache first
        let board_hash = self.get_board_hash(board);
        if let Some(cached_score) = self.evaluation_cache.borrow().get(&(board_hash, player)) {
            return *cached_score;
        }
        
        // Determine if we should use fast mode
        let use_fast_mode = self.config.performance_mode || depth >= self.fast_mode_threshold;
        
        let mut total_score = TaperedScore::default();
        
        if use_fast_mode {
            // Fast mode: simplified evaluation
            total_score = self.evaluate_fast_mode(board, player);
        } else {
            // Full evaluation
            // Castle structure evaluation
            if let Some(king_pos) = self.find_king_position(board, player) {
                let castle_score = self.castle_recognizer.evaluate_castle_structure(board, player, king_pos);
                total_score += castle_score * self.config.castle_weight;
            }
            
            // Attack analysis
            let attack_score = self.attack_analyzer.evaluate_attacks(board, player);
            total_score += attack_score * self.config.attack_weight;
            
            // Threat evaluation
            let threat_score = self.threat_evaluator.evaluate_threats(board, player);
            total_score += threat_score * self.config.threat_weight;
        }
        
        // Apply phase adjustment
        let final_score = total_score * self.config.phase_adjustment;
        
        // Cache the result (limit cache size)
        if self.evaluation_cache.borrow().len() < 10000 {
            self.evaluation_cache.borrow_mut().insert((board_hash, player), final_score);
        }
        
        final_score
    }
    
    /// Fast mode evaluation for deep search nodes
    fn evaluate_fast_mode(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = TaperedScore::default();
        
        // Simplified castle evaluation (only check for basic patterns)
        if let Some(king_pos) = self.find_king_position(board, player) {
            score += self.evaluate_basic_castle(board, player, king_pos);
        }
        
        // Simplified attack evaluation (only count major pieces near king)
        score += self.evaluate_basic_attacks(board, player);
        
        score
    }
    
    /// Basic castle evaluation for fast mode
    fn evaluate_basic_castle(&self, board: &BitboardBoard, player: Player, king_pos: Position) -> TaperedScore {
        let mut score = 0;
        
        // Check for basic defensive pieces around king
        let defensive_offsets = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
        
        for (dr, dc) in defensive_offsets.iter() {
            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;
            
            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let pos = Position::new(new_row as u8, new_col as u8);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        match piece.piece_type {
                            PieceType::Gold | PieceType::Silver => score += 20,
                            PieceType::Pawn => score += 10,
                            _ => score += 5,
                        }
                    }
                }
            }
        }
        
        TaperedScore::new_tapered(score, score / 2)
    }
    
    /// Basic attack evaluation for fast mode
    fn evaluate_basic_attacks(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        let mut score = 0;
        
        if let Some(king_pos) = self.find_king_position(board, player) {
            let opponent = player.opposite();
            
            // Check for major pieces attacking the king zone
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.player == opponent {
                            let distance = ((row as i8 - king_pos.row as i8).abs() + (col as i8 - king_pos.col as i8).abs()) as u8;
                            
                            if distance <= 3 {
                                match piece.piece_type {
                                    PieceType::Rook | PieceType::PromotedRook => score -= 30,
                                    PieceType::Bishop | PieceType::PromotedBishop => score -= 25,
                                    PieceType::Silver | PieceType::Gold => score -= 15,
                                    _ => score -= 5,
                                }
                            }
                        }
                    }
                }
            }
        }
        
        TaperedScore::new_tapered(score, score / 2)
    }
    
    /// Get a simple hash for the board position
    fn get_board_hash(&self, board: &BitboardBoard) -> u64 {
        // Simple hash based on piece positions
        let mut hash = 0u64;
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_hash = (piece.piece_type as u8 as u64) << (piece.player as u8 * 4);
                    hash ^= piece_hash.wrapping_mul(pos.row as u64 + 1).wrapping_mul(pos.col as u64 + 1);
                }
            }
        }
        hash
    }
    
    /// Clear the evaluation cache
    pub fn clear_cache(&self) {
        self.evaluation_cache.borrow_mut().clear();
    }
    
    /// Set the fast mode threshold
    pub fn set_fast_mode_threshold(&mut self, threshold: u8) {
        self.fast_mode_threshold = threshold;
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.evaluation_cache.borrow();
        (cache.len(), 10000) // current size, max size
    }
    
    /// Evaluate castle structure for the given player
    pub fn evaluate_castle_structure(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // TODO: Implement castle pattern recognition
        // This is a placeholder implementation
        TaperedScore::default()
    }
    
    /// Evaluate attacks on the king for the given player
    pub fn evaluate_attacks(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // TODO: Implement attack analysis
        // This is a placeholder implementation
        TaperedScore::default()
    }
    
    /// Evaluate tactical threats to the king for the given player
    pub fn evaluate_threats(&self, _board: &BitboardBoard, _player: Player) -> TaperedScore {
        // TODO: Implement threat evaluation
        // This is a placeholder implementation
        TaperedScore::default()
    }
    
    /// Fast evaluation for nodes deep in search tree
    pub fn evaluate_fast(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
        if !self.config.enabled || !self.config.performance_mode {
            return self.evaluate(board, player);
        }
        
        // Use fast mode evaluation
        self.evaluate_fast_mode(board, player)
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

impl Default for KingSafetyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_safety_evaluator_creation() {
        let evaluator = KingSafetyEvaluator::new();
        assert!(evaluator.get_config().enabled);
        assert_eq!(evaluator.get_config().castle_weight, 1.0);
        assert_eq!(evaluator.get_config().attack_weight, 1.0);
        assert_eq!(evaluator.get_config().threat_weight, 1.0);
    }

    #[test]
    fn test_king_safety_evaluator_with_config() {
        let config = KingSafetyConfig {
            enabled: false,
            castle_weight: 2.0,
            attack_weight: 1.5,
            threat_weight: 0.5,
            phase_adjustment: 0.6,
            performance_mode: true,
        };
        
        let evaluator = KingSafetyEvaluator::with_config(config.clone());
        assert_eq!(evaluator.get_config().enabled, false);
        assert_eq!(evaluator.get_config().castle_weight, 2.0);
        assert_eq!(evaluator.get_config().attack_weight, 1.5);
        assert_eq!(evaluator.get_config().threat_weight, 0.5);
    }

    #[test]
    fn test_king_safety_evaluation_disabled() {
        let mut evaluator = KingSafetyEvaluator::new();
        let mut config = evaluator.get_config().clone();
        config.enabled = false;
        evaluator.set_config(config);
        
        let board = BitboardBoard::new();
        let score = evaluator.evaluate(&board, Player::Black);
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_king_safety_evaluation_enabled() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::new();
        let score = evaluator.evaluate(&board, Player::Black);
        
        // Should return a score (even if placeholder)
        assert_eq!(score, TaperedScore::default());
    }

    #[test]
    fn test_find_king_position() {
        let evaluator = KingSafetyEvaluator::new();
        let board = BitboardBoard::new();
        
        let black_king = evaluator.find_king_position(&board, Player::Black);
        let white_king = evaluator.find_king_position(&board, Player::White);
        
        assert!(black_king.is_some());
        assert!(white_king.is_some());
        
        // Verify king positions are different
        assert_ne!(black_king.unwrap(), white_king.unwrap());
    }
}
