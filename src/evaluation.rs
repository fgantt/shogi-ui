use crate::types::*;
use crate::bitboards::*;

/// Position evaluator for the Shogi engine
pub struct PositionEvaluator {
    // Piece-square tables for positional evaluation
    piece_square_tables: PieceSquareTables,
    
    // Evaluation weights
    weights: EvaluationWeights,
}

impl PositionEvaluator {
    pub fn new() -> Self {
        Self {
            piece_square_tables: PieceSquareTables::new(),
            weights: EvaluationWeights::new(),
        }
    }

    /// Evaluate the current position from the perspective of the given player
    pub fn evaluate(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        // Add a small tempo bonus for the current player
        score += 10;

        // Material and positional score
        score += self.evaluate_material_and_position(board, player);
        
        // Pawn structure
        score += self.evaluate_pawn_structure(board, player);
        
        // King safety
        score += self.evaluate_king_safety(board, player);
        
        // Mobility
        score += self.evaluate_mobility(board, player);
        
        // Piece coordination
        score += self.evaluate_piece_coordination(board, player);
        
        // Center control
        score += self.evaluate_center_control(board, player);
        
        // Development
        score += self.evaluate_development(board, player);
        
        score
    }

    /// Evaluate material and positional value
    fn evaluate_material_and_position(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    let piece_value = piece.piece_type.base_value();
                    let positional_value = self.piece_square_tables.get_value(piece.piece_type, pos, piece.player);
                    
                    if piece.player == player {
                        score += piece_value + positional_value;
                    } else {
                        score -= piece_value + positional_value;
                    }
                }
            }
        }
        
        score
    }

    /// Evaluate pawn structure
    fn evaluate_pawn_structure(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
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
            return 0;
        }
        
        // Bonus for pawn chains
        for i in 0..pawns.len() {
            for j in i + 1..pawns.len() {
                let r1 = pawns[i].row;
                let c1 = pawns[i].col;
                let r2 = pawns[j].row;
                let c2 = pawns[j].col;
                
                // Check if pawns are adjacent horizontally or vertically
                if (r1.abs_diff(r2) == 1 && c1 == c2) || (c1.abs_diff(c2) == 1 && r1 == r2) {
                    score += 15; // Bonus for connected pawns
                }
            }
        }
        
        // Bonus for advanced pawns
        let promotion_zone_start = if player == Player::Black { 2 } else { 6 };
        for pawn in pawns.iter() {
            if player == Player::Black && pawn.row <= promotion_zone_start {
                score += (promotion_zone_start - pawn.row) as i32 * 10;
            } else if player == Player::White && pawn.row >= promotion_zone_start {
                score += (pawn.row - promotion_zone_start) as i32 * 10;
            }
        }
        
        // Penalty for isolated pawns
        for pawn in pawns.iter() {
            if self.is_pawn_isolated(board, *pawn, player) {
                score -= 20;
            }
        }
        
        score
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

    /// Evaluate king safety
    fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        // Find king position
        let king_pos = self.find_king_position(board, player);
        if king_pos.is_none() {
            return 0;
        }
        
        let king_pos = king_pos.unwrap();
        
        // King shield: reward for having friendly pieces nearby
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
                        score += match piece.piece_type {
                            PieceType::Gold => 40,
                            PieceType::Silver => 30,
                            PieceType::Knight => 20,
                            PieceType::Lance => 15,
                            PieceType::Pawn => 10,
                            _ => 5,
                        };
                    }
                }
            }
        }
        
        // Penalty for nearby enemy pieces
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
        
        score -= enemy_attackers * 30;
        
        score
    }

    /// Evaluate mobility (number of legal moves)
    fn evaluate_mobility(&self, board: &BitboardBoard, player: Player) -> i32 {
        // This is a simplified implementation
        // In practice, we'd generate all legal moves and count them
        let mut mobility = 0;
        
        // Count pieces and their potential moves
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        mobility += self.estimate_piece_mobility(piece.piece_type, pos, board);
                    }
                }
            }
        }
        
        mobility * 10
    }

    /// Estimate mobility for a piece type
    fn estimate_piece_mobility(&self, piece_type: PieceType, pos: Position, board: &BitboardBoard) -> i32 {
        match piece_type {
            PieceType::Pawn => 1,
            PieceType::Lance => 3,
            PieceType::Knight => 2,
            PieceType::Silver => 5,
            PieceType::Gold => 6,
            PieceType::Bishop => 8,
            PieceType::Rook => 8,
            PieceType::King => 8,
            _ => 6, // Promoted pieces
        }
    }

    /// Evaluate piece coordination
    fn evaluate_piece_coordination(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        // Bonus for connected rooks
        score += self.evaluate_connected_rooks(board, player);
        
        // Bonus for bishop pair
        score += self.evaluate_bishop_pair(board, player);
        
        // Bonus for coordinated attacks
        score += self.evaluate_coordinated_attacks(board, player);
        
        score
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
    fn evaluate_coordinated_attacks(&self, board: &BitboardBoard, player: Player) -> i32 {
        // This is a simplified implementation
        // In practice, we'd analyze attack patterns and piece coordination
        0
    }

    /// Evaluate center control
    fn evaluate_center_control(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        // Bonus for controlling center squares (3-5, 3-5)
        for row in 3..=5 {
            for col in 3..=5 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        score += 20; // Bonus for center control
                    } else {
                        score -= 20; // Penalty for opponent center control
                    }
                }
            }
        }
        
        score
    }

    /// Evaluate development
    fn evaluate_development(&self, board: &BitboardBoard, player: Player) -> i32 {
        let mut score = 0;
        
        // Bonus for developing pieces early
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        match piece.piece_type {
                            // Encourage moving key pieces out of their starting positions
                            PieceType::Bishop | PieceType::Rook | PieceType::Silver | PieceType::Gold => {
                                if self.is_piece_developed(piece.piece_type, pos, player) {
                                    score += 25; // Increased bonus for developing important pieces
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        score
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
}

/// Piece-square tables for positional evaluation
#[derive(Clone)]
struct PieceSquareTables {
    pawn_table: [[i32; 9]; 9],
    lance_table: [[i32; 9]; 9],
    knight_table: [[i32; 9]; 9],
    silver_table: [[i32; 9]; 9],
    gold_table: [[i32; 9]; 9],
    bishop_table: [[i32; 9]; 9],
    rook_table: [[i32; 9]; 9],
}

impl PieceSquareTables {
    fn new() -> Self {
        Self {
            pawn_table: Self::init_pawn_table(),
            lance_table: Self::init_lance_table(),
            knight_table: Self::init_knight_table(),
            silver_table: Self::init_silver_table(),
            gold_table: Self::init_gold_table(),
            bishop_table: Self::init_bishop_table(),
            rook_table: Self::init_rook_table(),
        }
    }

    /// Get positional value for a piece
    fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> i32 {
        let table = match piece_type {
            PieceType::Pawn => &self.pawn_table,
            PieceType::Lance => &self.lance_table,
            PieceType::Knight => &self.knight_table,
            PieceType::Silver => &self.silver_table,
            PieceType::Gold => &self.gold_table,
            PieceType::Bishop => &self.bishop_table,
            PieceType::Rook => &self.rook_table,
            _ => return 0, // No positional value for other pieces
        };
        
        let (row, col) = if player == Player::Black {
            (pos.row, pos.col)
        } else {
            (8 - pos.row, 8 - pos.col)
        };
        
        table[row as usize][col as usize]
    }

    fn init_pawn_table() -> [[i32; 9]; 9] {
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

    fn init_lance_table() -> [[i32; 9]; 9] {
        [
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
            [0, 0, 5, 10, 10, 10, 5, 0, 0],
        ]
    }

    fn init_knight_table() -> [[i32; 9]; 9] {
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

    fn init_silver_table() -> [[i32; 9]; 9] {
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

    fn init_gold_table() -> [[i32; 9]; 9] {
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

    fn init_bishop_table() -> [[i32; 9]; 9] {
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

    fn init_rook_table() -> [[i32; 9]; 9] {
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
}

/// Evaluation weights for different components
#[derive(Clone)]
struct EvaluationWeights {
    material_weight: i32,
    positional_weight: i32,
    pawn_structure_weight: i32,
    king_safety_weight: i32,
    mobility_weight: i32,
    coordination_weight: i32,
    center_control_weight: i32,
    development_weight: i32,
}

impl EvaluationWeights {
    fn new() -> Self {
        Self {
            material_weight: 100,
            positional_weight: 1,
            pawn_structure_weight: 1,
            king_safety_weight: 1,
            mobility_weight: 1,
            coordination_weight: 1,
            center_control_weight: 1,
            development_weight: 1,
        }
    }
}
