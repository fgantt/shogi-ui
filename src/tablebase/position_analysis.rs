//! Position complexity analysis for adaptive solver selection
//! 
//! This module provides tools for analyzing the complexity of chess positions
//! to help select the most appropriate endgame solver.

use crate::{BitboardBoard, Player, Position, PieceType, Piece};
use crate::types::{CapturedPieces, Move};

/// Position complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PositionComplexity {
    /// Very simple position (e.g., basic K+Q vs K)
    VerySimple = 1,
    /// Simple position (e.g., K+R vs K)
    Simple = 2,
    /// Medium complexity (e.g., K+B vs K)
    Medium = 3,
    /// Complex position (e.g., K+N vs K)
    Complex = 4,
    /// Very complex position (e.g., multiple pieces)
    VeryComplex = 5,
}

impl PositionComplexity {
    /// Get the numeric value of the complexity
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Check if this complexity level is suitable for a given solver priority
    pub fn is_suitable_for_priority(&self, solver_priority: u8) -> bool {
        // Higher priority solvers can handle more complex positions
        // Lower complexity positions are suitable for all solvers
        self.value() <= solver_priority / 20
    }
}

/// Position analysis result
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    /// Overall complexity of the position
    pub complexity: PositionComplexity,
    /// Number of pieces on the board
    pub piece_count: u8,
    /// Number of attacking pieces
    pub attacking_pieces: u8,
    /// Number of defending pieces
    pub defending_pieces: u8,
    /// King safety score (higher = safer)
    pub king_safety: i32,
    /// Material balance (positive = advantage)
    pub material_balance: i32,
    /// Mobility score (number of legal moves)
    pub mobility: u8,
    /// Whether the position is in endgame
    pub is_endgame: bool,
    /// Whether the position is tactical (many captures/threats)
    pub is_tactical: bool,
    /// Recommended solver priority threshold
    pub recommended_solver_priority: u8,
}

impl PositionAnalysis {
    /// Create a new position analysis
    pub fn new() -> Self {
        Self {
            complexity: PositionComplexity::VerySimple,
            piece_count: 0,
            attacking_pieces: 0,
            defending_pieces: 0,
            king_safety: 0,
            material_balance: 0,
            mobility: 0,
            is_endgame: false,
            is_tactical: false,
            recommended_solver_priority: 100,
        }
    }

    /// Get a summary of the analysis
    pub fn summary(&self) -> String {
        format!(
            "Complexity: {:?}, Pieces: {}, Attacking: {}, Defending: {}, King Safety: {}, Material: {}, Mobility: {}, Endgame: {}, Tactical: {}",
            self.complexity,
            self.piece_count,
            self.attacking_pieces,
            self.defending_pieces,
            self.king_safety,
            self.material_balance,
            self.mobility,
            self.is_endgame,
            self.is_tactical
        )
    }
}

/// Position analyzer for adaptive solver selection
pub struct PositionAnalyzer {
    /// Cache for analyzed positions to avoid repeated analysis
    analysis_cache: std::collections::HashMap<u64, PositionAnalysis>,
    /// Maximum cache size
    max_cache_size: usize,
}

impl PositionAnalyzer {
    /// Create a new position analyzer
    pub fn new() -> Self {
        Self {
            analysis_cache: std::collections::HashMap::new(),
            max_cache_size: 1000,
        }
    }

    /// Create a position analyzer with specified cache size
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            analysis_cache: std::collections::HashMap::new(),
            max_cache_size: cache_size,
        }
    }

    /// Analyze a position and return complexity information
    pub fn analyze_position(&mut self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> PositionAnalysis {
        // Generate cache key
        let cache_key = self.generate_cache_key(board, player, captured_pieces);
        
        // Check cache first
        if let Some(cached_analysis) = self.analysis_cache.get(&cache_key) {
            return cached_analysis.clone();
        }

        // Perform analysis
        let analysis = self.perform_analysis(board, player, captured_pieces);
        
        // Cache the result
        if self.analysis_cache.len() >= self.max_cache_size {
            self.evict_oldest_analysis();
        }
        self.analysis_cache.insert(cache_key, analysis.clone());
        
        analysis
    }

    /// Perform the actual position analysis
    fn perform_analysis(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> PositionAnalysis {
        let mut analysis = PositionAnalysis::new();
        
        // Count pieces
        analysis.piece_count = self.count_pieces(board);
        analysis.attacking_pieces = self.count_attacking_pieces(board, player);
        analysis.defending_pieces = self.count_defending_pieces(board, player);
        
        // Analyze king safety
        analysis.king_safety = self.analyze_king_safety(board, player);
        
        // Analyze material balance
        analysis.material_balance = self.analyze_material_balance(board, captured_pieces);
        
        // Analyze mobility
        analysis.mobility = self.analyze_mobility(board, player);
        
        // Determine if endgame
        analysis.is_endgame = self.is_endgame_position(board, captured_pieces);
        
        // Determine if tactical
        analysis.is_tactical = self.is_tactical_position(board, player);
        
        // Calculate overall complexity
        analysis.complexity = self.calculate_complexity(&analysis);
        
        // Recommend solver priority
        analysis.recommended_solver_priority = self.recommend_solver_priority(&analysis);
        
        analysis
    }

    /// Count total pieces on the board
    fn count_pieces(&self, board: &BitboardBoard) -> u8 {
        let pieces = board.get_pieces();
        let mut count = 0;
        for player_pieces in pieces.iter() {
            for piece_bitboard in player_pieces.iter() {
                count += crate::types::count_bits(*piece_bitboard);
            }
        }
        count as u8
    }

    /// Count attacking pieces for the given player
    fn count_attacking_pieces(&self, board: &BitboardBoard, player: Player) -> u8 {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let mut count = 0;
        
        // Count non-king pieces (kings are defensive)
        for (piece_idx, piece_bitboard) in pieces[player_idx].iter().enumerate() {
            if piece_idx != PieceType::King.to_u8() as usize {
                count += crate::types::count_bits(*piece_bitboard);
            }
        }
        
        count as u8
    }

    /// Count defending pieces for the given player
    fn count_defending_pieces(&self, board: &BitboardBoard, player: Player) -> u8 {
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        
        // Count king and defensive pieces
        let king_bitboard = pieces[player_idx][PieceType::King.to_u8() as usize];
        let king_count = crate::types::count_bits(king_bitboard);
        
        king_count as u8
    }

    /// Analyze king safety
    fn analyze_king_safety(&self, board: &BitboardBoard, player: Player) -> i32 {
        // Simplified king safety analysis
        // In a real implementation, this would analyze king position, pawn structure, etc.
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let king_bitboard = pieces[player_idx][PieceType::King.to_u8() as usize];
        
        if king_bitboard == 0 {
            return -1000; // No king = very unsafe
        }
        
        // Basic safety based on king position
        let king_pos = crate::types::get_lsb(king_bitboard);
        if let Some(pos) = king_pos {
            // Kings in center are generally safer in endgames
            let center_distance = self.distance_from_center(pos);
            100 - center_distance as i32
        } else {
            0
        }
    }

    /// Calculate distance from center of board
    fn distance_from_center(&self, pos: Position) -> u8 {
        let center_row = 4; // 9x9 board center
        let center_col = 4;
        let row_dist = (pos.row as i32 - center_row).abs() as u8;
        let col_dist = (pos.col as i32 - center_col).abs() as u8;
        row_dist + col_dist
    }

    /// Analyze material balance
    fn analyze_material_balance(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> i32 {
        let pieces = board.get_pieces();
        let mut balance = 0;
        
        // Count material on board
        for (player_idx, player_pieces) in pieces.iter().enumerate() {
            let multiplier = if player_idx == 0 { 1 } else { -1 }; // Black positive, White negative
            for (piece_idx, piece_bitboard) in player_pieces.iter().enumerate() {
                let count = crate::types::count_bits(*piece_bitboard);
                let value = self.get_piece_value(piece_idx as u8);
                balance += (count as i32 * value) * multiplier;
            }
        }
        
        // Add captured pieces
        for piece_type in [
            PieceType::Pawn, PieceType::Lance, PieceType::Knight,
            PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook
        ] {
            let black_captured = captured_pieces.count(piece_type, Player::Black);
            let white_captured = captured_pieces.count(piece_type, Player::White);
            let value = self.get_piece_value(piece_type.to_u8());
            balance += (black_captured as i32 - white_captured as i32) * value;
        }
        
        balance
    }

    /// Get piece value for material calculation
    fn get_piece_value(&self, piece_type: u8) -> i32 {
        match piece_type {
            0 => 1,   // Pawn
            1 => 3,   // Lance
            2 => 3,   // Knight
            3 => 5,   // Silver
            4 => 6,   // Gold
            5 => 8,   // Bishop
            6 => 10,  // Rook
            7 => 0,   // King (not counted in material)
            _ => 0,
        }
    }

    /// Analyze mobility (number of legal moves)
    fn analyze_mobility(&self, board: &BitboardBoard, player: Player) -> u8 {
        // Simplified mobility analysis
        // In a real implementation, this would generate all legal moves
        let pieces = board.get_pieces();
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let mut mobility = 0;
        
        for piece_bitboard in pieces[player_idx].iter() {
            let count = crate::types::count_bits(*piece_bitboard);
            mobility += count * 8; // Estimate 8 moves per piece
        }
        
        mobility as u8
    }

    /// Check if position is in endgame
    fn is_endgame_position(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces) -> bool {
        let piece_count = self.count_pieces(board);
        let total_captured = self.count_captured_pieces(captured_pieces);
        
        // Endgame if few pieces on board or many pieces captured
        piece_count <= 6 || total_captured >= 20
    }

    /// Count total captured pieces
    fn count_captured_pieces(&self, captured_pieces: &CapturedPieces) -> u8 {
        let mut total = 0;
        for piece_type in [
            PieceType::Pawn, PieceType::Lance, PieceType::Knight,
            PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook
        ] {
            total += captured_pieces.count(piece_type, Player::Black);
            total += captured_pieces.count(piece_type, Player::White);
        }
        total as u8
    }

    /// Check if position is tactical
    fn is_tactical_position(&self, board: &BitboardBoard, player: Player) -> bool {
        // Simplified tactical analysis
        // In a real implementation, this would look for threats, captures, etc.
        let attacking_pieces = self.count_attacking_pieces(board, player);
        attacking_pieces >= 2
    }

    /// Calculate overall position complexity
    fn calculate_complexity(&self, analysis: &PositionAnalysis) -> PositionComplexity {
        let mut complexity_score = 0;
        
        // Piece count factor
        complexity_score += analysis.piece_count as i32 * 2;
        
        // Attacking pieces factor
        complexity_score += analysis.attacking_pieces as i32 * 3;
        
        // Tactical factor
        if analysis.is_tactical {
            complexity_score += 10;
        }
        
        // Endgame factor (endgames are generally simpler)
        if analysis.is_endgame {
            complexity_score -= 5;
        }
        
        // King safety factor
        if analysis.king_safety < 50 {
            complexity_score += 5;
        }
        
        // Determine complexity level
        match complexity_score {
            0..=5 => PositionComplexity::VerySimple,
            6..=10 => PositionComplexity::Simple,
            11..=20 => PositionComplexity::Medium,
            21..=30 => PositionComplexity::Complex,
            _ => PositionComplexity::VeryComplex,
        }
    }

    /// Recommend solver priority based on analysis
    fn recommend_solver_priority(&self, analysis: &PositionAnalysis) -> u8 {
        match analysis.complexity {
            PositionComplexity::VerySimple => 100,
            PositionComplexity::Simple => 90,
            PositionComplexity::Medium => 80,
            PositionComplexity::Complex => 70,
            PositionComplexity::VeryComplex => 60,
        }
    }

    /// Generate cache key for position
    fn generate_cache_key(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash board state
        let pieces = board.get_pieces();
        for player_pieces in pieces.iter() {
            for piece_bitboard in player_pieces.iter() {
                piece_bitboard.hash(&mut hasher);
            }
        }
        
        // Hash player
        player.hash(&mut hasher);
        
        // Hash captured pieces
        for piece_type in [
            PieceType::Pawn, PieceType::Lance, PieceType::Knight,
            PieceType::Silver, PieceType::Gold, PieceType::Bishop, PieceType::Rook
        ] {
            captured_pieces.count(piece_type, Player::Black).hash(&mut hasher);
            captured_pieces.count(piece_type, Player::White).hash(&mut hasher);
        }
        
        hasher.finish()
    }

    /// Evict oldest analysis from cache
    fn evict_oldest_analysis(&mut self) {
        if let Some(&key) = self.analysis_cache.keys().next() {
            self.analysis_cache.remove(&key);
        }
    }

    /// Clear analysis cache
    pub fn clear_cache(&mut self) {
        self.analysis_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.analysis_cache.len(), self.max_cache_size)
    }
}

impl Default for PositionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
