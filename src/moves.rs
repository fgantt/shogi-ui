use crate::bitboards::*;
use crate::types::*;
use std::collections::HashSet;

pub struct MoveGenerator {
    // Cache for move generation to avoid redundant work
    move_cache: std::collections::HashMap<String, Vec<Move>>,
    cache_hits: u64,
    cache_misses: u64,
    // Feature flags for magic bitboard integration
    magic_bitboard_enabled: bool,
    batch_processing_enabled: bool,
    // Performance metrics
    magic_move_count: u64,
    raycast_move_count: u64,
    magic_generation_time: std::time::Duration,
    raycast_generation_time: std::time::Duration,
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            move_cache: std::collections::HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            magic_bitboard_enabled: true,
            batch_processing_enabled: true,
            magic_move_count: 0,
            raycast_move_count: 0,
            magic_generation_time: std::time::Duration::ZERO,
            raycast_generation_time: std::time::Duration::ZERO,
        }
    }

    /// Create a new move generator with custom settings
    pub fn with_settings(magic_enabled: bool, batch_processing: bool) -> Self {
        Self {
            move_cache: std::collections::HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            magic_bitboard_enabled: magic_enabled,
            batch_processing_enabled: batch_processing,
            magic_move_count: 0,
            raycast_move_count: 0,
            magic_generation_time: std::time::Duration::ZERO,
            raycast_generation_time: std::time::Duration::ZERO,
        }
    }

    /// Enable or disable magic bitboards
    pub fn set_magic_bitboard_enabled(&mut self, enabled: bool) {
        self.magic_bitboard_enabled = enabled;
    }

    /// Check if magic bitboards are enabled
    pub fn is_magic_bitboard_enabled(&self) -> bool {
        self.magic_bitboard_enabled
    }

    /// Enable or disable batch processing
    pub fn set_batch_processing_enabled(&mut self, enabled: bool) {
        self.batch_processing_enabled = enabled;
    }

    /// Check if batch processing is enabled
    pub fn is_batch_processing_enabled(&self) -> bool {
        self.batch_processing_enabled
    }

    /// Get performance comparison metrics
    pub fn get_performance_metrics(&self) -> MoveGenerationMetrics {
        MoveGenerationMetrics {
            magic_move_count: self.magic_move_count,
            raycast_move_count: self.raycast_move_count,
            magic_generation_time: self.magic_generation_time,
            raycast_generation_time: self.raycast_generation_time,
            magic_bitboard_enabled: self.magic_bitboard_enabled,
            batch_processing_enabled: self.batch_processing_enabled,
        }
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.magic_move_count = 0;
        self.raycast_move_count = 0;
        self.magic_generation_time = std::time::Duration::ZERO;
        self.raycast_generation_time = std::time::Duration::ZERO;
    }

    pub fn generate_legal_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let is_in_check = board.is_king_in_check(player, captured_pieces);

        crate::debug_utils::debug_log(&format!(
            "[GENERATE_LEGAL_MOVES] Player: {:?}, In check: {}",
            player, is_in_check
        ));

        let pseudo_legal_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);
        crate::debug_utils::debug_log(&format!(
            "[GENERATE_LEGAL_MOVES] Generated {} pseudo-legal moves",
            pseudo_legal_moves.len()
        ));

        let legal_moves: Vec<Move> = pseudo_legal_moves
            .into_iter()
            .filter(|m| {
                let mut temp_board = board.clone();
                let mut temp_captured = captured_pieces.clone();

                if let Some(captured) = temp_board.make_move(m) {
                    temp_captured.add_piece(captured.piece_type, player);
                }

                !temp_board.is_king_in_check(player, &temp_captured)
            })
            .collect();

        crate::debug_utils::debug_log(&format!(
            "[GENERATE_LEGAL_MOVES] Final legal moves: {}",
            legal_moves.len()
        ));
        if is_in_check {
            crate::debug_utils::debug_log(&format!(
                "[GENERATE_LEGAL_MOVES] Player was in check, found {} legal moves",
                legal_moves.len()
            ));
        }

        legal_moves
    }

    pub fn generate_legal_captures(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let pseudo_legal_moves =
            self.generate_pseudo_legal_captures(board, player, captured_pieces);

        // Filter out moves that leave the king in check
        pseudo_legal_moves
            .into_iter()
            .filter(|m| {
                let mut temp_board = board.clone();
                let mut temp_captured = captured_pieces.clone();
                if let Some(captured) = temp_board.make_move(m) {
                    temp_captured.add_piece(captured.piece_type, m.player);
                }
                !temp_board.is_king_in_check(player, &temp_captured)
            })
            .collect()
    }

    fn generate_pseudo_legal_captures(
        &self,
        board: &BitboardBoard,
        player: Player,
        _captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        self.generate_capture_piece_moves(board, player)
    }

    fn generate_capture_piece_moves(&self, board: &BitboardBoard, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        moves.extend(self.generate_capture_moves_for_piece(board, piece, pos));
                    }
                }
            }
        }
        moves
    }

    fn generate_capture_moves_for_piece(
        &self,
        board: &BitboardBoard,
        piece: &Piece,
        pos: Position,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = piece.player;

        let handle_capture_move = |moves: &mut Vec<Move>, to_pos: Position| {
            if !board.is_square_occupied_by(to_pos, player) {
                if board.is_square_occupied(to_pos) {
                    // Is a capture
                    let from_in_opponent_promo = pos.is_in_promotion_zone(player.opposite());
                    let to_in_opponent_promo = to_pos.is_in_promotion_zone(player.opposite());

                    // Non-promoted move
                    let mut move_ = Move::new_move(pos, to_pos, piece.piece_type, player, false);
                    move_.is_capture = true;
                    move_.captured_piece = board.get_piece(to_pos).cloned();
                    moves.push(move_);

                    // Promoted move
                    if piece.piece_type.can_promote()
                        && (from_in_opponent_promo || to_in_opponent_promo)
                    {
                        let mut promoted_move =
                            Move::new_move(pos, to_pos, piece.piece_type, player, true);
                        promoted_move.is_capture = true;
                        promoted_move.captured_piece = board.get_piece(to_pos).cloned();
                        moves.push(promoted_move);
                    }
                }
            }
        };

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: i8 = if player == Player::Black { -1 } else { 1 };
                let new_row = pos.row as i8 + dir;
                if new_row >= 0 && new_row < 9 {
                    handle_capture_move(&mut moves, Position::new(new_row as u8, pos.col));
                }
            }
            PieceType::Knight => {
                let dir: i8 = if player == Player::Black { -1 } else { 1 };
                let move_offsets = [(2 * dir, 1), (2 * dir, -1)];
                for (dr, dc) in move_offsets.iter() {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_capture_move(
                            &mut moves,
                            Position::new(new_row as u8, new_col as u8),
                        );
                    }
                }
            }
            PieceType::Lance | PieceType::Rook | PieceType::Bishop => {
                let directions = match piece.piece_type {
                    PieceType::Lance => {
                        if player == Player::Black {
                            vec![(-1, 0)]
                        } else {
                            vec![(1, 0)]
                        }
                    }
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => vec![],
                };

                for (dr, dc) in directions {
                    let mut current_pos = pos;
                    loop {
                        let new_row = current_pos.row as i8 + dr;
                        let new_col = current_pos.col as i8 + dc;
                        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                            break;
                        }

                        current_pos = Position::new(new_row as u8, new_col as u8);
                        handle_capture_move(&mut moves, current_pos);

                        if board.is_square_occupied(current_pos) {
                            break;
                        }
                    }
                }
            }
            PieceType::Silver
            | PieceType::Gold
            | PieceType::King
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver
            | PieceType::PromotedBishop
            | PieceType::PromotedRook => {
                let dir: i8 = if player == Player::Black { -1 } else { 1 };
                let offsets = piece.piece_type.get_move_offsets(dir);
                for (dr, dc) in offsets {
                    let new_row = pos.row as i8 + dr;
                    let new_col = pos.col as i8 + dc;
                    if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                        handle_capture_move(
                            &mut moves,
                            Position::new(new_row as u8, new_col as u8),
                        );
                    }
                }
            }
        }
        moves
    }

    fn generate_pseudo_legal_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        moves.extend(self.generate_all_piece_moves(board, player));
        moves.extend(self.generate_drop_moves(board, player, captured_pieces));
        moves
    }

    pub fn generate_all_piece_moves(&self, board: &BitboardBoard, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = board.get_piece(pos) {
                    if piece.player == player {
                        moves.extend(self.generate_moves_for_single_piece(board, piece, pos));
                    }
                }
            }
        }
        moves
    }

    fn generate_moves_for_single_piece(
        &self,
        board: &BitboardBoard,
        piece: &Piece,
        pos: Position,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let player = piece.player;

        let handle_move = |moves: &mut Vec<Move>, to_pos: Position| {
            if !board.is_square_occupied_by(to_pos, player) {
                let is_capture = board.is_square_occupied(to_pos);
                let from_in_opponent_promo = pos.is_in_promotion_zone(player.opposite());
                let to_in_opponent_promo = to_pos.is_in_promotion_zone(player.opposite());

                // Non-promoted move
                let mut move_ = Move::new_move(pos, to_pos, piece.piece_type, player, false);
                if is_capture {
                    move_.is_capture = true;
                    move_.captured_piece = board.get_piece(to_pos).cloned();
                }
                moves.push(move_);

                // Promoted move
                if piece.piece_type.can_promote()
                    && (from_in_opponent_promo || to_in_opponent_promo)
                {
                    let mut promoted_move =
                        Move::new_move(pos, to_pos, piece.piece_type, player, true);
                    if is_capture {
                        promoted_move.is_capture = true;
                        promoted_move.captured_piece = board.get_piece(to_pos).cloned();
                    }
                    moves.push(promoted_move);
                }
            }
        };

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: i8 = if player == Player::Black { -1 } else { 1 };
                let new_row = pos.row as i8 + dir;
                if new_row >= 0 && new_row < 9 {
                    handle_move(&mut moves, Position::new(new_row as u8, pos.col));
                }
            }
            PieceType::Knight => {
                // Use precomputed attack patterns for better performance
                let attacks = board.get_attack_pattern_precomputed(pos, piece.piece_type, player);

                // Convert attack bitboard to individual moves
                for target_square in 0..81 {
                    if attacks & (1u128 << target_square) != 0 {
                        let target_pos = Position::from_index(target_square as u8);
                        handle_move(&mut moves, target_pos);
                    }
                }
            }
            PieceType::Lance | PieceType::Rook | PieceType::Bishop => {
                let directions = match piece.piece_type {
                    PieceType::Lance => {
                        if player == Player::Black {
                            vec![(-1, 0)]
                        } else {
                            vec![(1, 0)]
                        }
                    }
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    _ => vec![],
                };

                for (dr, dc) in directions {
                    let mut current_pos = pos;
                    loop {
                        let new_row = current_pos.row as i8 + dr;
                        let new_col = current_pos.col as i8 + dc;
                        if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                            break;
                        }

                        current_pos = Position::new(new_row as u8, new_col as u8);
                        handle_move(&mut moves, current_pos);

                        if board.is_square_occupied(current_pos) {
                            break;
                        }
                    }
                }
            }
            PieceType::Silver
            | PieceType::Gold
            | PieceType::King
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver
            | PieceType::PromotedBishop
            | PieceType::PromotedRook => {
                // Use precomputed attack patterns for better performance
                let attacks = board.get_attack_pattern_precomputed(pos, piece.piece_type, player);

                // Convert attack bitboard to individual moves
                for target_square in 0..81 {
                    if attacks & (1u128 << target_square) != 0 {
                        let target_pos = Position::from_index(target_square as u8);
                        handle_move(&mut moves, target_pos);
                    }
                }
            }
        }
        moves
    }

    fn generate_drop_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut processed_pieces = HashSet::new();
        let captured = if player == Player::Black {
            &captured_pieces.black
        } else {
            &captured_pieces.white
        };

        for &piece_type in captured {
            if !processed_pieces.insert(piece_type) {
                continue;
            }

            for r in 0..9 {
                for c in 0..9 {
                    let pos = Position::new(r, c);
                    if !board.is_square_occupied(pos) {
                        // Basic legality check for drops (e.g., pawn drops)
                        if is_legal_drop_location(board, piece_type, pos, player) {
                            moves.push(Move::new_drop(piece_type, pos, player));
                        }
                    }
                }
            }
        }
        moves
    }

    /// Generate all moves that give check to the opponent
    pub fn generate_checks(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut check_moves = Vec::new();
        let opponent = player.opposite();

        // Generate all pseudo-legal moves
        let all_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);

        for mut move_ in all_moves {
            // Make the move on a temporary board
            let mut temp_board = board.clone();
            let mut temp_captured = captured_pieces.clone();

            if let Some(captured) = temp_board.make_move(&move_) {
                temp_captured.add_piece(captured.piece_type, player);
            }

            // Check if this move gives check to the opponent
            if temp_board.is_king_in_check(opponent, &temp_captured) {
                move_.gives_check = true;
                check_moves.push(move_);
            }
        }

        check_moves
    }

    /// Generate all promotion moves
    pub fn generate_promotions(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut promotion_moves = Vec::new();

        // Generate all pseudo-legal moves
        let all_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);

        for move_ in all_moves {
            if move_.is_promotion {
                promotion_moves.push(move_);
            }
        }

        promotion_moves
    }

    /// Generate moves that create tactical threats
    pub fn generate_tactical_threats(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        let mut threat_moves = Vec::new();
        let _opponent = player.opposite();

        // Generate all pseudo-legal moves
        let all_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);

        for move_ in all_moves {
            // Check if this move creates a threat (attacks opponent pieces or creates tactical patterns)
            if self.is_tactical_threat(&move_, board, player) {
                threat_moves.push(move_);
            }
        }

        threat_moves
    }

    /// Check if a move creates a tactical threat
    fn is_tactical_threat(&self, move_: &Move, board: &BitboardBoard, player: Player) -> bool {
        // For now, we'll consider moves that attack opponent pieces as threats
        // This can be expanded to include more sophisticated threat detection
        if move_.is_capture {
            return true;
        }

        // Check if the move attacks opponent pieces
        if let Some(from) = move_.from {
            if let Some(piece) = board.get_piece(from) {
                // Check if this piece can attack opponent pieces from the new position
                let opponent = player.opposite();
                let mut temp_board = board.clone();
                temp_board.remove_piece(from);
                temp_board.place_piece(*piece, move_.to);

                // Check if the piece can attack any opponent pieces from this position
                for r in 0..9 {
                    for c in 0..9 {
                        let pos = Position::new(r, c);
                        if let Some(target_piece) = board.get_piece(pos) {
                            if target_piece.player == opponent {
                                // This is a simplified threat detection
                                // In a more sophisticated implementation, we would check
                                // if the piece can actually attack the target
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Generate all quiescence moves (captures, checks, promotions, threats)
    pub fn generate_quiescence_moves(
        &self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        // Pre-allocate with estimated capacity to reduce allocations
        let mut moves = Vec::with_capacity(32);

        // 1. Generate captures (highest priority) - most important for quiescence
        let captures = self.generate_legal_captures(board, player, captured_pieces);
        moves.extend(captures);

        // 2. Generate checks - high priority for tactical positions
        let checks = self.generate_checks(board, player, captured_pieces);
        moves.extend(checks);

        // 3. Generate promotions - important for endgame tactics
        let promotions = self.generate_promotions(board, player, captured_pieces);
        moves.extend(promotions);

        // 4. Generate tactical threats - only if we have few moves so far
        if moves.len() < 16 {
            // Only generate threats if we don't have many tactical moves
            let threats = self.generate_tactical_threats(board, player, captured_pieces);
            moves.extend(threats);
        }

        // Remove duplicates efficiently and sort by priority
        self.deduplicate_and_sort_quiescence_moves(&mut moves);
        moves
    }

    /// Optimized deduplication and sorting for quiescence moves
    fn deduplicate_and_sort_quiescence_moves(&self, moves: &mut Vec<Move>) {
        if moves.is_empty() {
            return;
        }

        // Sort first to group duplicates together
        moves.sort_by(|a, b| self.compare_quiescence_moves_simple(a, b));

        // Remove duplicates (moves with same from, to, and piece_type)
        let mut write_index = 0;
        for read_index in 1..moves.len() {
            if !self.moves_equal(&moves[write_index], &moves[read_index]) {
                write_index += 1;
                if write_index != read_index {
                    moves[write_index] = moves[read_index].clone();
                }
            }
        }
        moves.truncate(write_index + 1);
    }

    /// Check if two moves are equal (for deduplication)
    fn moves_equal(&self, a: &Move, b: &Move) -> bool {
        a.from == b.from && a.to == b.to && a.piece_type == b.piece_type && a.player == b.player
    }

    /// Generate quiescence moves with caching
    pub fn generate_quiescence_moves_cached(
        &mut self,
        board: &BitboardBoard,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> Vec<Move> {
        // Create cache key from board state
        let cache_key = format!(
            "q_{}_{}",
            board.to_fen(player, captured_pieces),
            player as u8
        );

        // Check cache first
        if let Some(cached_moves) = self.move_cache.get(&cache_key) {
            self.cache_hits += 1;
            return cached_moves.clone();
        }

        self.cache_misses += 1;

        // Generate moves if not in cache
        let moves = self.generate_quiescence_moves(board, player, captured_pieces);

        // Cache the result (limit cache size)
        if self.move_cache.len() < 1000 {
            self.move_cache.insert(cache_key, moves.clone());
        }

        moves
    }

    /// Clear the move cache
    pub fn clear_cache(&mut self) {
        self.move_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        let total_attempts = self.cache_hits + self.cache_misses;
        let hit_rate = if total_attempts > 0 {
            (self.cache_hits as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }

    /// Simple comparison function for quiescence moves (used in MoveGenerator)
    pub fn compare_quiescence_moves_simple(&self, a: &Move, b: &Move) -> std::cmp::Ordering {
        // Create a simple, guaranteed total order by using a hash-based comparison
        // This ensures we never have equal moves that are actually different

        // 1. Checks first
        match (a.gives_check, b.gives_check) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // 2. Captures vs non-captures (captures have higher priority)
        match (a.is_capture, b.is_capture) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            (true, true) => {
                // Both are captures - use MVV-LVA
                let a_value = a.captured_piece_value() - a.piece_value();
                let b_value = b.captured_piece_value() - b.piece_value();
                let capture_cmp = b_value.cmp(&a_value);
                if capture_cmp != std::cmp::Ordering::Equal {
                    return capture_cmp;
                }
            }
            (false, false) => {
                // Neither is a capture - continue to other criteria
            }
        }

        // 3. Promotions
        match (a.is_promotion, b.is_promotion) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // 4. Tactical threat assessment
        let a_threat_value = self.assess_tactical_threat_value(a);
        let b_threat_value = self.assess_tactical_threat_value(b);
        let threat_cmp = b_threat_value.cmp(&a_threat_value);
        if threat_cmp != std::cmp::Ordering::Equal {
            return threat_cmp;
        }

        // 5. Piece value ordering
        let piece_cmp = b.piece_value().cmp(&a.piece_value());
        if piece_cmp != std::cmp::Ordering::Equal {
            return piece_cmp;
        }

        // 6. Use a simple hash-based comparison to ensure total order
        // This guarantees that different moves will always have different orderings
        let a_hash = self.move_hash(a);
        let b_hash = self.move_hash(b);
        a_hash.cmp(&b_hash)
    }

    /// Create a simple hash for move comparison
    fn move_hash(&self, move_: &Move) -> u64 {
        let mut hash = 0u64;

        // Hash the to position
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.row as u64);
        hash = hash.wrapping_mul(31).wrapping_add(move_.to.col as u64);

        // Hash the from position (if exists)
        if let Some(from) = move_.from {
            hash = hash.wrapping_mul(31).wrapping_add(from.row as u64);
            hash = hash.wrapping_mul(31).wrapping_add(from.col as u64);
        }

        // Hash the piece type
        hash = hash.wrapping_mul(31).wrapping_add(move_.piece_type as u64);

        // Hash the player
        hash = hash.wrapping_mul(31).wrapping_add(move_.player as u64);

        hash
    }

    /// Compare two moves for quiescence search ordering
    #[allow(dead_code)]
    fn compare_quiescence_moves(&self, a: &Move, b: &Move) -> std::cmp::Ordering {
        // 1. Checks first
        match (a.gives_check, b.gives_check) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // 2. MVV-LVA for captures
        if a.is_capture && b.is_capture {
            let a_value = a.captured_piece_value() - a.piece_value();
            let b_value = b.captured_piece_value() - b.piece_value();
            let capture_cmp = b_value.cmp(&a_value);
            if capture_cmp != std::cmp::Ordering::Equal {
                return capture_cmp;
            }
        }

        // 3. Promotions
        match (a.is_promotion, b.is_promotion) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // 4. Tactical threat assessment
        let a_threat_value = self.assess_tactical_threat_value(a);
        let b_threat_value = self.assess_tactical_threat_value(b);
        let threat_cmp = b_threat_value.cmp(&a_threat_value);
        if threat_cmp != std::cmp::Ordering::Equal {
            return threat_cmp;
        }

        // 5. Piece value ordering
        let piece_cmp = b.piece_value().cmp(&a.piece_value());
        if piece_cmp != std::cmp::Ordering::Equal {
            return piece_cmp;
        }

        // 6. Position-based ordering (to ensure total order)
        let a_pos_value = (a.to.row as i32 * 9 + a.to.col as i32) as i32;
        let b_pos_value = (b.to.row as i32 * 9 + b.to.col as i32) as i32;
        let pos_cmp = a_pos_value.cmp(&b_pos_value);
        if pos_cmp != std::cmp::Ordering::Equal {
            return pos_cmp;
        }

        // 7. From position ordering
        match (a.from, b.from) {
            (Some(a_from), Some(b_from)) => {
                let a_from_value = (a_from.row as i32 * 9 + a_from.col as i32) as i32;
                let b_from_value = (b_from.row as i32 * 9 + b_from.col as i32) as i32;
                a_from_value.cmp(&b_from_value)
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }

    /// Assess tactical threat value for move ordering
    fn assess_tactical_threat_value(&self, move_: &Move) -> i32 {
        let mut threat_value = 0;

        // High value for captures
        if move_.is_capture {
            threat_value += move_.captured_piece_value();
        }

        // High value for checks
        if move_.gives_check {
            threat_value += 1000;
        }

        // High value for promotions
        if move_.is_promotion {
            threat_value += move_.promotion_value();
        }

        // High value for recaptures
        if move_.is_recapture {
            threat_value += 500;
        }

        threat_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboards::BitboardBoard;
    use crate::types::{CapturedPieces, Piece, PieceType, Player, Position};

    #[test]
    fn test_white_pawn_promotion() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPP1PP/1B5R1/LNSGKGSNL w - 1";
        let (board, player, captured_pieces) = BitboardBoard::from_fen(fen).unwrap();
        let move_generator = MoveGenerator::new();
        let moves = move_generator.generate_legal_moves(&board, player, &captured_pieces);

        for m in &moves {
            if m.to_usi_string().contains("+") {
                assert!(
                    m.to.is_in_promotion_zone(player)
                        || m.from.unwrap().is_in_promotion_zone(player)
                );
            }
        }
    }

    #[test]
    fn test_quiescence_move_sorting_total_order() {
        let move_generator = MoveGenerator::new();

        // Create test moves with different properties
        let mut test_moves = vec![
            // Non-capture move
            Move {
                from: Some(Position { row: 1, col: 1 }),
                to: Position { row: 2, col: 1 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: false,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: None,
            },
            // Capture move
            Move {
                from: Some(Position { row: 1, col: 2 }),
                to: Position { row: 2, col: 2 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: true,
                is_promotion: false,
                gives_check: false,
                is_recapture: false,
                captured_piece: Some(Piece {
                    piece_type: PieceType::Pawn,
                    player: Player::White,
                }),
            },
            // Check move
            Move {
                from: Some(Position { row: 1, col: 3 }),
                to: Position { row: 2, col: 3 },
                piece_type: PieceType::Pawn,
                player: Player::Black,
                is_capture: false,
                is_promotion: false,
                gives_check: true,
                is_recapture: false,
                captured_piece: None,
            },
        ];

        // Test that sorting doesn't panic and produces consistent results
        test_moves.sort_by(|a, b| move_generator.compare_quiescence_moves_simple(a, b));

        // Verify the ordering is correct
        // Check should be first, then capture, then non-capture
        assert!(test_moves[0].gives_check, "Check move should be first");
        assert!(test_moves[1].is_capture, "Capture move should be second");
        assert!(
            !test_moves[2].is_capture && !test_moves[2].gives_check,
            "Non-capture move should be last"
        );

        // Test that the comparison is transitive and consistent
        for i in 0..test_moves.len() {
            for j in 0..test_moves.len() {
                let cmp_ij =
                    move_generator.compare_quiescence_moves_simple(&test_moves[i], &test_moves[j]);
                let cmp_ji =
                    move_generator.compare_quiescence_moves_simple(&test_moves[j], &test_moves[i]);

                // Test antisymmetry: if a < b, then b > a
                match (cmp_ij, cmp_ji) {
                    (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => {}
                    (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {}
                    (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {}
                    _ => panic!("Comparison is not antisymmetric: {} vs {}", i, j),
                }
            }
        }
    }
}
fn is_legal_drop_location(
    board: &BitboardBoard,
    piece_type: PieceType,
    pos: Position,
    player: Player,
) -> bool {
    if piece_type == PieceType::Pawn {
        // Rule 1: Cannot drop on a file that already contains an unpromoted pawn of the same color (Nifu / 二歩)
        for r in 0..9 {
            if let Some(p) = board.get_piece(Position::new(r, pos.col)) {
                if p.piece_type == PieceType::Pawn && p.player == player {
                    crate::debug_utils::debug_log(&format!(
                        "[NIFU] Illegal pawn drop at {}{}. Already have pawn on file {}",
                        (b'a' + pos.col) as char,
                        9 - pos.row,
                        (b'a' + pos.col) as char
                    ));
                    return false;
                }
            }
        }

        // Rule 2: Cannot drop pawn to give immediate checkmate (Uchifuzume / 打ち歩詰め)
        // This rule only applies to drops that give checkmate, not just check
        if is_pawn_drop_mate(board, pos, player) {
            crate::debug_utils::debug_log(&format!(
                "[UCHIFUZUME] Illegal pawn drop mate at {}{}",
                (b'a' + pos.col) as char,
                9 - pos.row
            ));
            return false;
        }
    }

    // Cannot drop a piece where it has no legal moves
    let last_rank = if player == Player::Black { 0 } else { 8 };
    let second_last_rank = if player == Player::Black { 1 } else { 7 };
    match piece_type {
        PieceType::Pawn | PieceType::Lance if pos.row == last_rank => return false,
        PieceType::Knight if pos.row == last_rank || pos.row == second_last_rank => return false,
        _ => true,
    }
}

/// Check if dropping a pawn at the given position gives immediate checkmate (Uchifuzume)
/// This is illegal in Shogi - you cannot drop a pawn to deliver checkmate
fn is_pawn_drop_mate(board: &BitboardBoard, drop_pos: Position, player: Player) -> bool {
    // Find opponent's king
    let opponent = player.opposite();
    let Some(king_pos) = board.find_king_position(opponent) else {
        return false; // No king, can't be checkmate
    };

    // Check if the pawn would give check
    let pawn_gives_check = match player {
        Player::Black => {
            // Black pawn attacks one square forward (decreasing row)
            king_pos.row == drop_pos.row.wrapping_sub(1) && king_pos.col == drop_pos.col
        }
        Player::White => {
            // White pawn attacks one square forward (increasing row)
            king_pos.row == drop_pos.row + 1 && king_pos.col == drop_pos.col
        }
    };

    if !pawn_gives_check {
        return false; // Not even giving check, so not checkmate
    }

    // Now check if it's actually checkmate (king has no escape)
    // This requires simulating the pawn drop and checking if the king has any legal moves
    let mut temp_board = board.clone();
    temp_board.place_piece(Piece::new(PieceType::Pawn, player), drop_pos);

    // Check if opponent king has any legal moves or if check can be blocked
    // For simplicity, check if king can escape to any adjacent square
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }

            let new_row = king_pos.row as i8 + dr;
            let new_col = king_pos.col as i8 + dc;

            if new_row >= 0 && new_row < 9 && new_col >= 0 && new_col < 9 {
                let escape_pos = Position::new(new_row as u8, new_col as u8);

                // Check if this square is a valid escape
                if let Some(piece_at_escape) = temp_board.get_piece(escape_pos) {
                    // Can only escape by capturing opponent's piece
                    if piece_at_escape.player == player {
                        continue; // Can't capture own piece
                    }
                }

                // Temporarily move king to escape square and see if it's still in check
                temp_board.remove_piece(king_pos);
                temp_board.place_piece(Piece::new(PieceType::King, opponent), escape_pos);

                // Create a temporary CapturedPieces (empty for this check)
                let temp_captured = CapturedPieces::new();
                if !temp_board.is_king_in_check(opponent, &temp_captured) {
                    return false; // King can escape, so not checkmate
                }

                // Restore the king position for next check
                temp_board.remove_piece(escape_pos);
                temp_board.place_piece(Piece::new(PieceType::King, opponent), king_pos);
            }
        }
    }

    // TODO: Also check if the pawn can be captured or if check can be blocked
    // For now, this is a reasonable approximation

    true // Looks like checkmate
}

/// Performance metrics for move generation
#[derive(Debug, Clone)]
pub struct MoveGenerationMetrics {
    pub magic_move_count: u64,
    pub raycast_move_count: u64,
    pub magic_generation_time: std::time::Duration,
    pub raycast_generation_time: std::time::Duration,
    pub magic_bitboard_enabled: bool,
    pub batch_processing_enabled: bool,
}

impl MoveGenerationMetrics {
    /// Calculate the speedup ratio of magic bitboards over ray-casting
    pub fn magic_speedup_ratio(&self) -> f64 {
        if self.raycast_generation_time.as_nanos() > 0 {
            self.raycast_generation_time.as_nanos() as f64
                / self.magic_generation_time.as_nanos() as f64
        } else {
            1.0
        }
    }

    /// Calculate the efficiency of magic bitboards
    pub fn magic_efficiency(&self) -> f64 {
        if self.magic_move_count + self.raycast_move_count > 0 {
            self.magic_move_count as f64 / (self.magic_move_count + self.raycast_move_count) as f64
        } else {
            0.0
        }
    }

    /// Get average time per move for magic bitboards
    pub fn magic_avg_time_per_move(&self) -> std::time::Duration {
        if self.magic_move_count > 0 {
            self.magic_generation_time / self.magic_move_count as u32
        } else {
            std::time::Duration::ZERO
        }
    }

    /// Get average time per move for ray-casting
    pub fn raycast_avg_time_per_move(&self) -> std::time::Duration {
        if self.raycast_move_count > 0 {
            self.raycast_generation_time / self.raycast_move_count as u32
        } else {
            std::time::Duration::ZERO
        }
    }
}
