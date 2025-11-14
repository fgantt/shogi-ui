use crate::search::RepetitionState;
use crate::types::*;
use std::sync::{Arc, OnceLock};

// Include the magic bitboard module
pub mod api;
pub mod attack_patterns;
pub mod bit_iterator;
pub mod bit_utils;
pub mod bitscan;
pub mod branch_opt;
pub mod cache_opt;
pub mod debruijn;
pub mod integration;
pub mod lookup_tables;
pub mod magic;
pub mod masks;
pub mod platform_detection;
pub mod popcount;
pub mod sliding_moves;
pub mod square_utils;

// Re-export commonly used functions for convenience
pub use bit_iterator::{
    bits, bits_from, BitIterator, BitIteratorExt, ReverseBitIterator, ReverseBitIteratorExt,
};
pub use bit_utils::{
    bit_positions, complement, difference, extract_lsb, extract_msb, intersection, is_subset,
    lsb_position, msb_position, overlaps, reverse_bits, rotate_left, rotate_right,
    symmetric_difference, union,
};
pub use bitscan::{
    bit_scan_forward, bit_scan_optimized, bit_scan_reverse, clear_lsb, clear_msb,
    get_all_bit_positions, isolate_lsb, isolate_msb,
};
pub use branch_opt::{
    common_cases::{
        is_empty_optimized, is_multiple_pieces_optimized, is_not_empty_optimized,
        is_single_piece_optimized, single_piece_position_optimized,
    },
    critical_paths::{bit_scan_forward_critical, popcount_critical},
    optimized::{
        bit_scan_forward_optimized, bit_scan_reverse_optimized, is_subset_optimized,
        overlaps_optimized, popcount_optimized as popcount_branch_optimized,
    },
};
pub use cache_opt::{
    get_bit_positions_cache_optimized, popcount_cache_optimized, prefetch_bitboard,
    prefetch_bitboard_sequence, process_bitboard_sequence, CacheAlignedBitPositionTable,
    CacheAlignedFileMasks, CacheAlignedPopcountTable, CacheAlignedRankMasks, CACHE_ALIGNED_SIZE,
    CACHE_LINE_SIZE,
};
pub use integration::{BitScanningOptimizer, GeometricAnalysis, GlobalOptimizer};
pub use lookup_tables::{
    bit_positions_4bit_lookup, bit_positions_4bit_small, popcount_4bit_lookup,
    popcount_4bit_optimized, popcount_4bit_small, validate_4bit_lookup_tables,
};
pub use masks::{
    get_diagonal_mask, get_diagonal_squares, get_file_from_square, get_file_mask, get_file_squares,
    get_rank_from_square, get_rank_mask, get_rank_squares, get_square_from_rank_file,
    same_diagonal, same_file, same_rank, validate_masks,
};
pub use platform_detection::{
    get_best_bitscan_impl, get_best_popcount_impl, get_platform_capabilities,
};
pub use popcount::{is_empty, is_multiple_bits, is_single_bit, popcount, popcount_optimized};
pub use square_utils::{
    bit_to_coords, bit_to_square, bit_to_square_name, coords_to_bit, get_center_squares,
    is_center_square, is_promotion_zone, is_valid_shogi_square, promotion_zone_mask,
    square_distance, square_name_to_bit, square_to_bit,
};

/// Shared singleton for magic bitboard table (Task 2.0.2.1)
/// This allows multiple boards to share the same magic table without cloning
static SHARED_MAGIC_TABLE: OnceLock<Arc<MagicTable>> = OnceLock::new();

/// Telemetry counters for magic bitboard operations (Task 2.0.2.3)
#[derive(Debug, Default)]
struct MagicTelemetry {
    /// Count of times ray-cast fallback was used
    raycast_fallback_count: std::sync::atomic::AtomicU64,
    /// Count of times magic lookup was used
    magic_lookup_count: std::sync::atomic::AtomicU64,
    /// Count of times magic support was unavailable
    magic_unavailable_count: std::sync::atomic::AtomicU64,
}

static MAGIC_TELEMETRY: MagicTelemetry = MagicTelemetry {
    raycast_fallback_count: std::sync::atomic::AtomicU64::new(0),
    magic_lookup_count: std::sync::atomic::AtomicU64::new(0),
    magic_unavailable_count: std::sync::atomic::AtomicU64::new(0),
};

/// Get telemetry statistics for magic bitboard operations
pub fn get_magic_telemetry() -> (u64, u64, u64) {
    (
        MAGIC_TELEMETRY
            .raycast_fallback_count
            .load(std::sync::atomic::Ordering::Relaxed),
        MAGIC_TELEMETRY
            .magic_lookup_count
            .load(std::sync::atomic::Ordering::Relaxed),
        MAGIC_TELEMETRY
            .magic_unavailable_count
            .load(std::sync::atomic::Ordering::Relaxed),
    )
}

/// Get or initialize the shared magic table singleton
/// Returns None if magic table initialization fails
fn get_shared_magic_table() -> Option<Arc<MagicTable>> {
    Some(
        SHARED_MAGIC_TABLE
            .get_or_init(|| {
                // Try to create a magic table, fall back to default if it fails
                Arc::new(
                    MagicTable::new().unwrap_or_else(|_| {
                        crate::debug_utils::debug_log(
                            "[MAGIC_TABLE] Failed to initialize magic table, using default",
                        );
                        MagicTable::default()
                    }),
                )
            })
            .clone(),
    )
}

/// Initialize the shared magic table singleton explicitly
/// This should be called once at startup if magic support is desired
pub fn init_shared_magic_table() -> Result<(), MagicError> {
    let table = MagicTable::new()?;
    SHARED_MAGIC_TABLE
        .set(Arc::new(table))
        .map_err(|_| MagicError::InitializationFailed {
            reason: "Magic table already initialized".to_string(),
        })?;
    Ok(())
}

/// Information needed to unmake a move
#[derive(Debug, Clone)]
pub struct MoveInfo {
    /// The original piece type before promotion (if promotion occurred)
    pub original_piece_type: PieceType,
    /// The from position (None for drops)
    pub from: Option<Position>,
    /// The to position
    pub to: Position,
    /// The player who made the move
    pub player: Player,
    /// Whether this was a promotion move
    pub was_promotion: bool,
    /// The captured piece, if any
    pub captured_piece: Option<Piece>,
}

impl MoveInfo {
    /// Create MoveInfo from move details
    pub fn new(
        original_piece_type: PieceType,
        from: Option<Position>,
        to: Position,
        player: Player,
        was_promotion: bool,
        captured_piece: Option<Piece>,
    ) -> Self {
        Self {
            original_piece_type,
            from,
            to,
            player,
            was_promotion,
            captured_piece,
        }
    }
}

/// Bitboard-based board representation for efficient Shogi operations
pub struct BitboardBoard {
    pieces: [[Bitboard; 14]; 2],
    occupied: Bitboard,
    black_occupied: Bitboard,
    white_occupied: Bitboard,
    squares: [Option<Piece>; 81],
    attack_patterns: AttackPatterns,
    /// Precomputed attack tables for non-sliding pieces
    attack_tables: Arc<attack_patterns::AttackTables>,
    /// Magic bitboard table for sliding piece moves (shared via Arc)
    magic_table: Option<Arc<MagicTable>>,
    /// Sliding move generator for magic bitboard operations
    sliding_generator: Option<sliding_moves::SlidingMoveGenerator>,
    side_to_move: Player,
    repetition_state: RepetitionState,
}

impl BitboardBoard {
    #[inline]
    fn square_index(position: Position) -> usize {
        position.to_index() as usize
    }

    #[inline]
    fn set_square(&mut self, position: Position, piece: Option<Piece>) {
        if position.is_valid() {
            let idx = Self::square_index(position);
            self.squares[idx] = piece;
        }
    }

    #[inline]
    fn iter_pieces(&self) -> impl Iterator<Item = (Position, Piece)> + '_ {
        self.squares
            .iter()
            .enumerate()
            .filter_map(|(idx, piece)| piece.map(|p| (Position::from_index(idx as u8), p)))
    }

    pub fn new() -> Self {
        let mut board = Self::empty();
        board.setup_initial_position();
        board
    }

    pub fn set_side_to_move(&mut self, player: Player) {
        self.side_to_move = player;
    }

    pub fn side_to_move(&self) -> Player {
        self.side_to_move
    }

    pub fn set_repetition_state(&mut self, state: RepetitionState) {
        self.repetition_state = state;
    }

    pub fn repetition_state(&self) -> RepetitionState {
        self.repetition_state
    }

    pub fn empty() -> Self {
        Self {
            pieces: [[EMPTY_BITBOARD; 14]; 2],
            occupied: EMPTY_BITBOARD,
            black_occupied: EMPTY_BITBOARD,
            white_occupied: EMPTY_BITBOARD,
            squares: [None; 81],
            attack_patterns: AttackPatterns::new(),
            attack_tables: Arc::new(attack_patterns::AttackTables::new()),
            magic_table: None,
            sliding_generator: None,
            side_to_move: Player::Black,
            repetition_state: RepetitionState::None,
        }
    }

    fn setup_initial_position(&mut self) {
        let start_fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        if let Ok((board, player, _)) = BitboardBoard::from_fen(start_fen) {
            self.pieces = board.pieces;
            self.occupied = board.occupied;
            self.black_occupied = board.black_occupied;
            self.white_occupied = board.white_occupied;
            self.squares = board.squares;
            self.side_to_move = player;
            self.repetition_state = RepetitionState::None;
        }
    }

    pub fn place_piece(&mut self, piece: Piece, position: Position) {
        // Validate position
        if !position.is_valid() {
            crate::debug_utils::debug_log(&format!(
                "[PLACE_PIECE ERROR] Invalid position: row={}, col={}",
                position.row, position.col
            ));
            return;
        }

        // Debug: Log piece before conversion
        crate::debug_utils::debug_log(&format!(
            "[PLACE_PIECE] Attempting to place {:?} {:?} at row={} col={}",
            piece.player, piece.piece_type, position.row, position.col
        ));

        let player_idx = if piece.player == Player::Black { 0 } else { 1 };

        // Try to get piece_idx and catch any issue
        let piece_idx = piece.piece_type.to_u8() as usize;

        crate::debug_utils::debug_log(&format!(
            "[PLACE_PIECE] Converted piece_type to index: {}",
            piece_idx
        ));

        // Validate piece index before array access
        if piece_idx >= 14 {
            crate::debug_utils::debug_log(&format!(
                "[PLACE_PIECE ERROR] Invalid piece index: {} (piece_type: {:?}, position: row={} col={})",
                piece_idx,
                piece.piece_type,
                position.row,
                position.col
            ));
            return;
        }

        set_bit(&mut self.pieces[player_idx][piece_idx], position);
        match piece.player {
            Player::Black => set_bit(&mut self.black_occupied, position),
            Player::White => set_bit(&mut self.white_occupied, position),
        }
        set_bit(&mut self.occupied, position);
        self.set_square(position, Some(piece));
    }

    pub fn remove_piece(&mut self, position: Position) -> Option<Piece> {
        if !position.is_valid() {
            return None;
        }

        let idx = Self::square_index(position);
        if let Some(piece) = self.squares[idx] {
            crate::debug_utils::debug_log(&format!(
                "[REMOVE_PIECE] Removing {:?} {:?} from row={} col={}",
                piece.player, piece.piece_type, position.row, position.col
            ));

            let player_idx = if piece.player == Player::Black { 0 } else { 1 };
            let piece_idx = piece.piece_type.to_u8() as usize;

            // Validate piece index
            if piece_idx >= 14 {
                crate::debug_utils::debug_log(&format!(
                    "[REMOVE_PIECE ERROR] Invalid piece index: {} (piece_type: {:?}, position: row={} col={})",
                    piece_idx,
                    piece.piece_type,
                    position.row,
                    position.col
                ));
                return Some(piece);
            }

            clear_bit(&mut self.pieces[player_idx][piece_idx], position);
            match piece.player {
                Player::Black => clear_bit(&mut self.black_occupied, position),
                Player::White => clear_bit(&mut self.white_occupied, position),
            }
            clear_bit(&mut self.occupied, position);
            self.squares[idx] = None;
            Some(piece)
        } else {
            None
        }
    }

    pub fn get_piece(&self, position: Position) -> Option<Piece> {
        if !position.is_valid() {
            return None;
        }
        let idx = Self::square_index(position);
        self.squares[idx]
    }

    pub fn get_pieces(&self) -> &[[Bitboard; 14]; 2] {
        &self.pieces
    }

    pub fn is_square_occupied(&self, position: Position) -> bool {
        is_bit_set(self.occupied, position)
    }

    pub fn is_square_occupied_by(&self, position: Position, player: Player) -> bool {
        let occupied = if player == Player::Black {
            self.black_occupied
        } else {
            self.white_occupied
        };
        is_bit_set(occupied, position)
    }

    pub fn make_move(&mut self, move_: &Move) -> Option<Piece> {
        let mut captured_piece = None;
        if let Some(from) = move_.from {
            if let Some(piece_to_move) = self.get_piece(from) {
                crate::debug_utils::debug_log(&format!(
                    "[MAKE_MOVE] Moving {:?} from row={} col={} to row={} col={}",
                    piece_to_move.piece_type, from.row, from.col, move_.to.row, move_.to.col
                ));

                self.remove_piece(from);
                if move_.is_capture {
                    if let Some(cp) = self.remove_piece(move_.to) {
                        captured_piece = Some(cp.unpromoted());
                    }
                }
                let final_piece_type = if move_.is_promotion {
                    piece_to_move
                        .piece_type
                        .promoted_version()
                        .unwrap_or(piece_to_move.piece_type)
                } else {
                    piece_to_move.piece_type
                };

                crate::debug_utils::debug_log(&format!(
                    "[MAKE_MOVE] Placing {:?} at row={} col={}",
                    final_piece_type, move_.to.row, move_.to.col
                ));

                self.place_piece(Piece::new(final_piece_type, piece_to_move.player), move_.to);
            }
        } else {
            crate::debug_utils::debug_log(&format!(
                "[MAKE_MOVE] Dropping {:?} at row={} col={}",
                move_.piece_type, move_.to.row, move_.to.col
            ));

            self.place_piece(Piece::new(move_.piece_type, move_.player), move_.to);
        }
        captured_piece
    }

    /// Make a move and return MoveInfo for unmaking
    /// This is an extended version of make_move that returns the information needed to unmake
    pub fn make_move_with_info(&mut self, move_: &Move) -> MoveInfo {
        let mut captured_piece = None;
        let mut original_piece_type = move_.piece_type;

        if let Some(from) = move_.from {
            if let Some(piece_to_move) = self.get_piece(from) {
                // Capture the original piece type before any modifications
                original_piece_type = piece_to_move.piece_type;

                crate::debug_utils::debug_log(&format!(
                    "[MAKE_MOVE_WITH_INFO] Moving {:?} from row={} col={} to row={} col={}",
                    piece_to_move.piece_type, from.row, from.col, move_.to.row, move_.to.col
                ));

                self.remove_piece(from);
                if move_.is_capture {
                    if let Some(cp) = self.remove_piece(move_.to) {
                        captured_piece = Some(cp.unpromoted());
                    }
                }
                let final_piece_type = if move_.is_promotion {
                    piece_to_move
                        .piece_type
                        .promoted_version()
                        .unwrap_or(piece_to_move.piece_type)
                } else {
                    piece_to_move.piece_type
                };

                crate::debug_utils::debug_log(&format!(
                    "[MAKE_MOVE_WITH_INFO] Placing {:?} at row={} col={}",
                    final_piece_type, move_.to.row, move_.to.col
                ));

                self.place_piece(Piece::new(final_piece_type, piece_to_move.player), move_.to);
            }
        } else {
            // Drop move
            crate::debug_utils::debug_log(&format!(
                "[MAKE_MOVE_WITH_INFO] Dropping {:?} at row={} col={}",
                move_.piece_type, move_.to.row, move_.to.col
            ));

            self.place_piece(Piece::new(move_.piece_type, move_.player), move_.to);
        }

        MoveInfo::new(
            original_piece_type,
            move_.from,
            move_.to,
            move_.player,
            move_.is_promotion,
            captured_piece,
        )
    }

    /// Unmake a move, restoring the board to its previous state
    /// This reverses the operations performed by make_move()
    pub fn unmake_move(&mut self, move_info: &MoveInfo) {
        crate::debug_utils::debug_log(&format!(
            "[UNMAKE_MOVE] Unmaking move from {:?} to {:?}",
            move_info.from, move_info.to
        ));

        // Remove the piece that was placed at the destination
        self.remove_piece(move_info.to);

        // Restore the captured piece if there was one
        if let Some(ref captured_piece) = move_info.captured_piece {
            self.place_piece(captured_piece.clone(), move_info.to);
        }

        // Restore the moved piece to its original position
        if let Some(from) = move_info.from {
            // This was a normal move (not a drop)
            // Place the original piece type (before promotion) back at the from position
            self.place_piece(
                Piece::new(move_info.original_piece_type, move_info.player),
                from,
            );
        }
        // If from is None, it was a drop, so we just remove the piece (already done above)
    }

    pub fn is_king_in_check(&self, player: Player, _captured_pieces: &CapturedPieces) -> bool {
        if let Some(king_pos) = self.find_king_position(player) {
            let is_attacked = self.is_square_attacked_by(king_pos, player.opposite());
            crate::debug_utils::debug_log(&format!(
                "[IS_KING_IN_CHECK] Player: {:?}, King at: {}{}, Attacked: {}",
                player,
                (b'a' + king_pos.col) as char,
                9 - king_pos.row,
                is_attacked
            ));
            return is_attacked;
        }
        crate::debug_utils::debug_log(&format!(
            "[IS_KING_IN_CHECK] Player: {:?}, No king found!",
            player
        ));
        false
    }

    pub fn find_king_position(&self, player: Player) -> Option<Position> {
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let king_bb = self.pieces[player_idx][PieceType::King.to_u8() as usize];
        if king_bb == 0 {
            None
        } else {
            get_lsb(king_bb)
        }
    }

    pub fn is_square_attacked_by(&self, target_pos: Position, attacking_player: Player) -> bool {
        // More efficient direct attack checking without generating all moves
        for r in 0..9 {
            for c in 0..9 {
                let from_pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(from_pos) {
                    if piece.player == attacking_player {
                        // Check if this piece attacks the target_pos directly
                        if self.piece_attacks_square(&piece, from_pos, target_pos) {
                            crate::debug_utils::debug_log(&format!(
                                "[IS_SQUARE_ATTACKED_BY] Found attacker: {:?} at {}{}",
                                piece.piece_type,
                                (b'a' + from_pos.col) as char,
                                9 - from_pos.row
                            ));
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // Direct attack checking without move generation
    fn piece_attacks_square(
        &self,
        piece: &Piece,
        from_pos: Position,
        target_pos: Position,
    ) -> bool {
        let player = piece.player;

        // Early bounds check
        if from_pos.row >= 9 || from_pos.col >= 9 || target_pos.row >= 9 || target_pos.col >= 9 {
            return false;
        }

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let new_row = from_pos.row as i8 + dir;
                if new_row >= 0 && new_row < 9 {
                    let attack_pos = Position::new(new_row as u8, from_pos.col);
                    return attack_pos == target_pos;
                }
                false
            }
            PieceType::Knight => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                let move_offsets = [(2 * dir, 1), (2 * dir, -1)];
                for (dr, dc) in move_offsets.iter() {
                    let new_row = from_pos.row as i8 + dr;
                    let new_col = from_pos.col as i8 + dc;
                    if new_row >= 0 && new_col >= 0 && new_row < 9 && new_col < 9 {
                        let attack_pos = Position::new(new_row as u8, new_col as u8);
                        if attack_pos == target_pos {
                            return true;
                        }
                    }
                }
                false
            }
            PieceType::Lance => {
                let dir: i8 = if player == Player::Black { 1 } else { -1 };
                self.check_ray_attack(from_pos, target_pos, (dir, 0))
            }
            PieceType::Rook => {
                self.check_ray_attack(from_pos, target_pos, (1, 0))
                    || self.check_ray_attack(from_pos, target_pos, (-1, 0))
                    || self.check_ray_attack(from_pos, target_pos, (0, 1))
                    || self.check_ray_attack(from_pos, target_pos, (0, -1))
            }
            PieceType::Bishop => {
                self.check_ray_attack(from_pos, target_pos, (1, 1))
                    || self.check_ray_attack(from_pos, target_pos, (1, -1))
                    || self.check_ray_attack(from_pos, target_pos, (-1, 1))
                    || self.check_ray_attack(from_pos, target_pos, (-1, -1))
            }
            PieceType::PromotedBishop => {
                // Bishop + King moves
                self.check_ray_attack(from_pos, target_pos, (1, 1))
                    || self.check_ray_attack(from_pos, target_pos, (1, -1))
                    || self.check_ray_attack(from_pos, target_pos, (-1, 1))
                    || self.check_ray_attack(from_pos, target_pos, (-1, -1))
                    || self.check_king_attack(from_pos, target_pos, player)
            }
            PieceType::PromotedRook => {
                // Rook + King moves
                self.check_ray_attack(from_pos, target_pos, (1, 0))
                    || self.check_ray_attack(from_pos, target_pos, (-1, 0))
                    || self.check_ray_attack(from_pos, target_pos, (0, 1))
                    || self.check_ray_attack(from_pos, target_pos, (0, -1))
                    || self.check_king_attack(from_pos, target_pos, player)
            }
            PieceType::Silver
            | PieceType::Gold
            | PieceType::King
            | PieceType::PromotedPawn
            | PieceType::PromotedLance
            | PieceType::PromotedKnight
            | PieceType::PromotedSilver => self.check_king_attack(from_pos, target_pos, player),
        }
    }

    // Check if a ray from from_pos in direction (dr, dc) hits target_pos
    fn check_ray_attack(
        &self,
        from_pos: Position,
        target_pos: Position,
        direction: (i8, i8),
    ) -> bool {
        let (dr, dc) = direction;
        let mut current_pos = from_pos;

        loop {
            let new_row = current_pos.row as i8 + dr;
            let new_col = current_pos.col as i8 + dc;

            // Out of bounds
            if new_row < 0 || new_row >= 9 || new_col < 0 || new_col >= 9 {
                break;
            }

            current_pos = Position::new(new_row as u8, new_col as u8);

            // Found target
            if current_pos == target_pos {
                return true;
            }

            // Blocked by a piece
            if self.is_square_occupied(current_pos) {
                break;
            }
        }

        false
    }

    // Check if a king-like piece attacks target_pos
    fn check_king_attack(&self, from_pos: Position, target_pos: Position, _player: Player) -> bool {
        let row_diff = (from_pos.row as i8 - target_pos.row as i8).abs();
        let col_diff = (from_pos.col as i8 - target_pos.col as i8).abs();

        // King attacks adjacent squares (including diagonals)
        row_diff <= 1 && col_diff <= 1 && (row_diff != 0 || col_diff != 0)
    }

    pub fn is_legal_move(&self, move_: &Move, captured_pieces: &CapturedPieces) -> bool {
        let mut temp_board = self.clone();
        let mut temp_captured = captured_pieces.clone();
        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, move_.player);
        }
        !temp_board.is_king_in_check(move_.player, &temp_captured)
    }

    pub fn is_checkmate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        self.is_king_in_check(player, captured_pieces)
            && !self.has_legal_moves(player, captured_pieces)
    }

    pub fn is_stalemate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        !self.is_king_in_check(player, captured_pieces)
            && !self.has_legal_moves(player, captured_pieces)
    }

    fn has_legal_moves(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        let move_generator = crate::moves::MoveGenerator::new();
        !move_generator
            .generate_legal_moves(self, player, captured_pieces)
            .is_empty()
    }

    /// Check if both kings are in their opponent's promotion zone (impasse condition)
    /// In Shogi, this is called Jishōgi (持将棋)
    pub fn is_impasse_condition(&self) -> bool {
        let black_king_pos = self.find_king_position(Player::Black);
        let white_king_pos = self.find_king_position(Player::White);

        if let (Some(black_pos), Some(white_pos)) = (black_king_pos, white_king_pos) {
            // Black king in white's camp (ranks 0-2) AND white king in black's camp (ranks 6-8)
            return black_pos.row <= 2 && white_pos.row >= 6;
        }
        false
    }

    /// Count points for impasse resolution using the 24-point rule
    /// King = 0, Rook/Dragon = 5, Bishop/Horse = 5, all others = 1
    pub fn count_impasse_points(&self, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        let mut points = 0;

        // Count pieces on board
        for (_, piece) in self.iter_pieces() {
            if piece.player == player {
                points += match piece.piece_type {
                    PieceType::Rook | PieceType::PromotedRook => 5,
                    PieceType::Bishop | PieceType::PromotedBishop => 5,
                    PieceType::King => 0,
                    _ => 1, // Gold, Silver, Knight, Lance, Pawn, and all other promoted pieces
                };
            }
        }

        // Count captured pieces (pieces in hand)
        let hand_pieces = match player {
            Player::Black => &captured_pieces.black,
            Player::White => &captured_pieces.white,
        };

        for piece_type in hand_pieces {
            points += match piece_type {
                PieceType::Rook => 5,
                PieceType::Bishop => 5,
                _ => 1,
            };
        }

        points
    }

    /// Check impasse result and return the outcome
    /// Returns None if not an impasse condition
    /// Both players need 24+ points for a draw, otherwise the player with fewer points loses
    pub fn check_impasse_result(&self, captured_pieces: &CapturedPieces) -> Option<ImpasseResult> {
        if !self.is_impasse_condition() {
            return None;
        }

        let black_points = self.count_impasse_points(Player::Black, captured_pieces);
        let white_points = self.count_impasse_points(Player::White, captured_pieces);

        let outcome = if black_points >= 24 && white_points >= 24 {
            ImpasseOutcome::Draw
        } else if black_points < 24 {
            ImpasseOutcome::WhiteWins
        } else {
            ImpasseOutcome::BlackWins
        };

        Some(ImpasseResult {
            black_points,
            white_points,
            outcome,
        })
    }

    pub fn to_fen(&self, player: Player, captured_pieces: &CapturedPieces) -> String {
        let mut fen = String::with_capacity(128);
        for r in 0..9 {
            let mut empty_squares = 0;
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(pos) {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    fen.push_str(&piece.to_fen_char());
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if r < 8 {
                fen.push('/');
            }
        }
        fen.push(' ');
        fen.push(if player == Player::Black { 'b' } else { 'w' });
        fen.push(' ');
        let mut captured_str = String::new();
        for p in &captured_pieces.black {
            captured_str.push_str(&Piece::new(*p, Player::Black).to_fen_char());
        }
        for p in &captured_pieces.white {
            captured_str.push_str(&Piece::new(*p, Player::White).to_fen_char());
        }
        if captured_str.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&captured_str);
        }
        fen
    }

    pub fn from_fen(fen: &str) -> Result<(BitboardBoard, Player, CapturedPieces), &str> {
        let mut board = BitboardBoard::empty();
        let mut captured_pieces = CapturedPieces::new();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid FEN string: not enough parts");
        }

        // 1. Parse board state
        let board_part = parts[0];
        let ranks: Vec<&str> = board_part.split('/').collect();
        if ranks.len() != 9 {
            return Err("Invalid FEN: must have 9 ranks");
        }

        for (r, rank_str) in ranks.iter().enumerate() {
            let mut c = 0;
            let mut chars = rank_str.chars().peekable();
            while let Some(ch) = chars.next() {
                if c >= 9 {
                    return Err("Invalid FEN: rank has more than 9 files");
                }
                if let Some(digit) = ch.to_digit(10) {
                    c += digit as usize;
                } else {
                    let is_promoted = ch == '+';
                    let piece_char = if is_promoted {
                        if let Some(next_ch) = chars.next() {
                            next_ch
                        } else {
                            return Err("Invalid FEN: '+' must be followed by a piece");
                        }
                    } else {
                        ch
                    };

                    let player = if piece_char.is_uppercase() {
                        Player::Black
                    } else {
                        Player::White
                    };
                    let piece_type_char = piece_char.to_ascii_lowercase();

                    let piece_type = match piece_type_char {
                        'p' => {
                            if is_promoted {
                                PieceType::PromotedPawn
                            } else {
                                PieceType::Pawn
                            }
                        }
                        'l' => {
                            if is_promoted {
                                PieceType::PromotedLance
                            } else {
                                PieceType::Lance
                            }
                        }
                        'n' => {
                            if is_promoted {
                                PieceType::PromotedKnight
                            } else {
                                PieceType::Knight
                            }
                        }
                        's' => {
                            if is_promoted {
                                PieceType::PromotedSilver
                            } else {
                                PieceType::Silver
                            }
                        }
                        'g' => PieceType::Gold,
                        'b' => {
                            if is_promoted {
                                PieceType::PromotedBishop
                            } else {
                                PieceType::Bishop
                            }
                        }
                        'r' => {
                            if is_promoted {
                                PieceType::PromotedRook
                            } else {
                                PieceType::Rook
                            }
                        }
                        'k' => PieceType::King,
                        _ => return Err("Invalid FEN: unknown piece character"),
                    };

                    board.place_piece(
                        Piece::new(piece_type, player),
                        Position::new(r as u8, c as u8),
                    );
                    c += 1;
                }
            }
        }

        // 2. Parse side to move
        let player = match parts[1] {
            "b" => Player::Black,
            "w" => Player::White,
            _ => return Err("Invalid FEN: invalid player"),
        };
        board.side_to_move = player;
        board.repetition_state = RepetitionState::None;

        // 3. Parse pieces in hand
        if parts[2] != "-" {
            let mut count = 1;
            for ch in parts[2].chars() {
                if let Some(digit) = ch.to_digit(10) {
                    count = digit;
                } else {
                    let hand_player = if ch.is_uppercase() {
                        Player::Black
                    } else {
                        Player::White
                    };
                    let piece_type = match ch.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'l' => PieceType::Lance,
                        'n' => PieceType::Knight,
                        's' => PieceType::Silver,
                        'g' => PieceType::Gold,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        _ => return Err("Invalid FEN: unknown piece in hand"),
                    };
                    for _ in 0..count {
                        captured_pieces.add_piece(piece_type, hand_player);
                    }
                    count = 1;
                }
            }
        }

        Ok((board, player, captured_pieces))
    }

    pub fn to_string_for_debug(&self) -> String {
        let mut board_str = String::new();
        board_str.push_str("  9  8  7  6  5  4  3  2  1\n");
        board_str.push_str("+--+--+--+--+--+--+--+--+--+\n");
        for r in 0..9 {
            board_str.push('|');
            for c in 0..9 {
                let pos = Position::new(r, c);
                if let Some(piece) = self.get_piece(pos) {
                    let mut piece_char = piece.to_fen_char();
                    if piece.player == Player::White {
                        piece_char = piece_char.to_lowercase();
                    }

                    if piece_char.starts_with('+') {
                        board_str.push_str(&piece_char);
                    } else {
                        board_str.push(' ');
                        board_str.push_str(&piece_char);
                    }
                } else {
                    board_str.push_str("  ");
                }
                board_str.push('|');
            }
            board_str.push_str(&format!(" {}\n", (b'a' + r) as char));
            board_str.push_str("+--+--+--+--+--+--+--+--+--+\n");
        }
        board_str
    }

    /// Initialize with magic bitboard support
    /// Uses the shared magic table singleton (Task 2.0.2.1)
    pub fn new_with_magic_support() -> Result<Self, MagicError> {
        let magic_table = get_shared_magic_table().ok_or_else(|| MagicError::InitializationFailed {
            reason: "Failed to get shared magic table".to_string(),
        })?;
        Ok(Self {
            pieces: [[EMPTY_BITBOARD; 14]; 2],
            occupied: EMPTY_BITBOARD,
            black_occupied: EMPTY_BITBOARD,
            white_occupied: EMPTY_BITBOARD,
            squares: [None; 81],
            attack_patterns: AttackPatterns::new(),
            attack_tables: Arc::new(attack_patterns::AttackTables::new()),
            magic_table: Some(magic_table),
            sliding_generator: None,
            side_to_move: Player::Black,
            repetition_state: RepetitionState::None,
        })
    }

    /// Get attack pattern for a piece at a given square using precomputed tables
    pub fn get_attack_pattern_precomputed(
        &self,
        square: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Bitboard {
        self.attack_tables
            .get_attack_pattern(square.to_u8(), piece_type, player)
    }

    /// Check if a square is attacked by a piece using precomputed tables
    pub fn is_square_attacked_precomputed(
        &self,
        from_square: Position,
        to_square: Position,
        piece_type: PieceType,
        player: Player,
    ) -> bool {
        self.attack_tables.is_square_attacked(
            from_square.to_u8(),
            to_square.to_u8(),
            piece_type,
            player,
        )
    }

    /// Get attack table statistics and metadata
    pub fn get_attack_table_stats(&self) -> &attack_patterns::AttackTablesMetadata {
        self.attack_tables.memory_stats()
    }

    /// Get attack pattern for a square using magic bitboards
    /// Task 2.0.2.3: Added telemetry tracking for magic vs fallback usage
    pub fn get_attack_pattern(&self, square: Position, piece_type: PieceType) -> Bitboard {
        if let Some(ref magic_table) = self.magic_table {
            MAGIC_TELEMETRY
                .magic_lookup_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            magic_table.get_attacks(square.to_index(), piece_type, self.occupied)
        } else {
            // Fallback to ray-casting
            MAGIC_TELEMETRY
                .raycast_fallback_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            MAGIC_TELEMETRY
                .magic_unavailable_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            crate::debug_utils::trace_log(
                "MAGIC_FALLBACK",
                &format!(
                    "Using ray-cast fallback for {:?} at square {} (magic table unavailable)",
                    piece_type,
                    square.to_index()
                ),
            );
            
            self.generate_attack_pattern_raycast(square, piece_type)
        }
    }

    /// Generate attack pattern using ray-casting (fallback method)
    /// Task 2.0.2.2: Implemented using AttackGenerator for correct fallback behavior
    fn generate_attack_pattern_raycast(
        &self,
        square: Position,
        piece_type: PieceType,
    ) -> Bitboard {
        use crate::bitboards::magic::attack_generator::AttackGenerator;
        
        // Only support sliding pieces for ray-casting
        if !matches!(
            piece_type,
            PieceType::Rook
                | PieceType::Bishop
                | PieceType::PromotedRook
                | PieceType::PromotedBishop
        ) {
            return EMPTY_BITBOARD;
        }
        
        // Use AttackGenerator to compute ray-cast attacks
        let mut generator = AttackGenerator::new();
        let occupied = self.occupied;
        let square_idx = square.to_index();
        
        // Generate attack pattern with current occupancy
        generator.generate_attack_pattern(square_idx, piece_type, occupied)
    }

    /// Check if magic bitboards are enabled
    pub fn has_magic_support(&self) -> bool {
        self.magic_table.is_some()
    }

    /// Get magic table reference
    pub fn get_magic_table(&self) -> Option<Arc<MagicTable>> {
        self.magic_table.clone()
    }

    /// Initialize sliding move generator with magic table
    /// Uses shared magic table reference (Task 2.0.2.1 - no longer consumes table)
    pub fn init_sliding_generator(&mut self) -> Result<(), crate::types::MagicError> {
        if let Some(ref magic_table) = self.magic_table {
            // Clone the Arc to share the table instead of taking ownership
            self.sliding_generator = Some(sliding_moves::SlidingMoveGenerator::new(
                Arc::clone(magic_table),
            ));
            Ok(())
        } else {
            Err(crate::types::MagicError::InitializationFailed {
                reason: "Magic table not initialized".to_string(),
            })
        }
    }

    /// Initialize sliding move generator with custom settings
    /// Uses shared magic table reference (Task 2.0.2.1 - no longer consumes table)
    pub fn init_sliding_generator_with_settings(
        &mut self,
        magic_enabled: bool,
    ) -> Result<(), crate::types::MagicError> {
        if let Some(ref magic_table) = self.magic_table {
            // Clone the Arc to share the table instead of taking ownership
            self.sliding_generator = Some(sliding_moves::SlidingMoveGenerator::with_settings(
                Arc::clone(magic_table),
                magic_enabled,
            ));
            Ok(())
        } else {
            Err(crate::types::MagicError::InitializationFailed {
                reason: "Magic table not initialized".to_string(),
            })
        }
    }

    /// Get sliding move generator reference
    pub fn get_sliding_generator(&self) -> Option<&sliding_moves::SlidingMoveGenerator> {
        self.sliding_generator.as_ref()
    }

    /// Check if sliding generator is initialized
    pub fn is_sliding_generator_initialized(&self) -> bool {
        self.sliding_generator.is_some()
    }

    /// Generate sliding moves for a piece using magic bitboards
    /// Returns None if magic bitboards are not initialized
    pub fn generate_magic_sliding_moves(
        &self,
        from: Position,
        piece_type: PieceType,
        player: Player,
    ) -> Option<Vec<Move>> {
        self.sliding_generator
            .as_ref()
            .map(|gen| gen.generate_sliding_moves(self, from, piece_type, player))
    }

    /// Get occupied bitboard
    pub fn get_occupied_bitboard(&self) -> Bitboard {
        self.occupied
    }

    /// Check if a square is occupied by a specific player
    pub fn is_occupied_by_player(&self, pos: Position, player: Player) -> bool {
        if let Some(piece) = self.get_piece(pos) {
            piece.player == player
        } else {
            false
        }
    }
}

// Import the BoardTrait for implementation
use crate::search::board_trait::{BoardTrait, BoardTraitExt};

impl BoardTrait for BitboardBoard {
    fn get_piece_at(&self, position: Position) -> Option<Piece> {
        self.get_piece(position)
    }

    fn get_all_pieces(&self) -> Vec<(Position, Piece)> {
        self.iter_pieces().collect()
    }

    fn count_pieces(&self, piece_type: PieceType, player: Player) -> usize {
        let player_idx = if player == Player::Black { 0 } else { 1 };
        let piece_idx = piece_type.to_u8() as usize;
        popcount(self.pieces[player_idx][piece_idx]) as usize
    }

    fn is_square_occupied(&self, position: Position) -> bool {
        self.is_square_occupied(position)
    }

    fn is_square_occupied_by_player(&self, position: Position, player: Player) -> bool {
        self.is_square_occupied_by(position, player)
    }

    fn get_occupied_bitboard_for_player(&self, player: Player) -> Bitboard {
        match player {
            Player::Black => self.black_occupied,
            Player::White => self.white_occupied,
        }
    }

    fn get_occupied_bitboard(&self) -> Bitboard {
        self.occupied
    }

    fn is_valid_position(&self) -> bool {
        // Check for exactly one king per player
        let black_kings = self.count_pieces(PieceType::King, Player::Black);
        let white_kings = self.count_pieces(PieceType::King, Player::White);
        black_kings == 1 && white_kings == 1
    }

    fn get_side_to_move(&self) -> Player {
        self.side_to_move
    }

    fn get_repetition_state(&self) -> RepetitionState {
        self.repetition_state
    }

    fn get_captured_pieces(&self, _player: Player) -> Vec<PieceType> {
        // This method requires access to CapturedPieces, which is not stored in BitboardBoard
        // We'll return an empty vector for now - this should be managed by the game state
        Vec::new()
    }

    fn get_captured_pieces_count(&self, _player: Player) -> usize {
        // This method requires access to CapturedPieces, which is not stored in BitboardBoard
        // We'll return 0 for now - this should be managed by the game state
        0
    }

    fn get_captured_piece_count(&self, _piece_type: PieceType, _player: Player) -> usize {
        // This method requires access to CapturedPieces, which is not stored in BitboardBoard
        // We'll return 0 for now - this should be managed by the game state
        0
    }

    fn has_captured_piece(&self, _piece_type: PieceType, _player: Player) -> bool {
        // This method requires access to CapturedPieces, which is not stored in BitboardBoard
        // We'll return false for now - this should be managed by the game state
        false
    }

    fn get_position_id(
        &self,
        player: Player,
        captured_pieces: &CapturedPieces,
        repetition_state: RepetitionState,
    ) -> u64 {
        use crate::search::zobrist::ZobristHasher;
        let hasher = ZobristHasher::new();
        hasher.hash_position(self, player, captured_pieces, repetition_state)
    }

    fn clone_board(&self) -> Self {
        self.clone()
    }

    fn is_terminal_position(&self, captured_pieces: &CapturedPieces) -> bool {
        // Check for checkmate or stalemate
        self.is_checkmate(Player::Black, captured_pieces)
            || self.is_checkmate(Player::White, captured_pieces)
            || self.is_stalemate(Player::Black, captured_pieces)
            || self.is_stalemate(Player::White, captured_pieces)
    }

    fn get_game_phase(&self) -> GamePhase {
        let _total_pieces = BoardTraitExt::get_total_piece_count(self);
        let material_count = self.get_total_material_count();
        GamePhase::from_material_count(material_count)
    }

    fn is_legal_move(&self, move_: &Move, captured_pieces: &CapturedPieces) -> bool {
        self.is_legal_move(move_, captured_pieces)
    }

    fn get_king_position(&self, player: Player) -> Option<Position> {
        self.find_king_position(player)
    }

    fn is_king_in_check(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
        self.is_king_in_check(player, captured_pieces)
    }

    fn get_material_balance(&self, player: Player) -> i32 {
        let mut balance = 0;

        // Calculate material balance for the current player
        for (_, piece) in self.iter_pieces() {
            let value = piece.piece_type.base_value();
            if piece.player == player {
                balance += value;
            } else {
                balance -= value;
            }
        }

        balance
    }

    fn get_total_material_count(&self) -> u32 {
        self.squares.iter().filter(|slot| slot.is_some()).count() as u32
    }

    fn is_in_promotion_zone(&self, position: Position, player: Player) -> bool {
        match player {
            Player::Black => position.row >= 6, // Black promotes in ranks 6-8
            Player::White => position.row <= 2, // White promotes in ranks 0-2
        }
    }

    fn get_drop_moves(&self, _piece_type: PieceType, _player: Player) -> Vec<Move> {
        // This method requires access to CapturedPieces, which is not stored in BitboardBoard
        // We'll return an empty vector for now - this should be managed by the game state
        Vec::new()
    }

    fn is_legal_drop(
        &self,
        piece_type: PieceType,
        position: Position,
        player: Player,
        captured_pieces: &CapturedPieces,
    ) -> bool {
        // Check if the square is empty
        if self.is_square_occupied(position) {
            return false;
        }

        // Check if the player has the piece in hand
        if captured_pieces.count(piece_type, player) == 0 {
            return false;
        }

        // Check piece-specific drop rules
        match piece_type {
            PieceType::Pawn => {
                // Pawn cannot be dropped in the last rank
                match player {
                    Player::Black => position.row < 8,
                    Player::White => position.row > 0,
                }
            }
            PieceType::Lance => {
                // Lance cannot be dropped in the last rank
                match player {
                    Player::Black => position.row < 8,
                    Player::White => position.row > 0,
                }
            }
            PieceType::Knight => {
                // Knight cannot be dropped in the last two ranks
                match player {
                    Player::Black => position.row < 7,
                    Player::White => position.row > 1,
                }
            }
            _ => true, // Other pieces can be dropped anywhere
        }
    }

    fn get_position_hash(&self, captured_pieces: &CapturedPieces) -> u64 {
        // Use the Zobrist hasher to compute the hash
        use crate::search::zobrist::ZobristHasher;
        let hasher = ZobristHasher::new();
        hasher.hash_position(
            self,
            self.get_side_to_move(),
            captured_pieces,
            self.get_repetition_state(),
        )
    }

    fn update_hash_for_move(
        &self,
        current_hash: u64,
        move_: &Move,
        captured_pieces_before: &CapturedPieces,
        captured_pieces_after: &CapturedPieces,
    ) -> u64 {
        // Use the Zobrist hasher to update the hash
        use crate::search::zobrist::ZobristHasher;
        let hasher = ZobristHasher::new();
        hasher.update_hash_for_move(
            current_hash,
            move_,
            self,
            self,
            captured_pieces_before,
            captured_pieces_after,
        )
    }
}

impl Clone for BitboardBoard {
    fn clone(&self) -> Self {
        Self {
            pieces: self.pieces,
            occupied: self.occupied,
            black_occupied: self.black_occupied,
            white_occupied: self.white_occupied,
            squares: self.squares,
            attack_patterns: self.attack_patterns.clone(),
            attack_tables: Arc::clone(&self.attack_tables),
            magic_table: self.magic_table.clone(),
            sliding_generator: self.sliding_generator.clone(),
            side_to_move: self.side_to_move,
            repetition_state: self.repetition_state,
        }
    }
}

#[derive(Clone)]
struct AttackPatterns {
    // Simplified for brevity
}

impl AttackPatterns {
    fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CapturedPieces, PieceType, Player, Position};

    #[test]
    fn test_from_fen_startpos() {
        let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
        let (board, player, captured) = BitboardBoard::from_fen(fen).unwrap();

        assert_eq!(player, Player::Black);
        assert!(captured.black.is_empty());
        assert!(captured.white.is_empty());

        // Spot check a few pieces
        let black_lance = board.get_piece(Position::new(8, 0)).unwrap();
        assert_eq!(black_lance.piece_type, PieceType::Lance);
        assert_eq!(black_lance.player, Player::Black);

        let white_king = board.get_piece(Position::new(0, 4)).unwrap();
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.player, Player::White);

        let black_pawn = board.get_piece(Position::new(6, 4)).unwrap();
        assert_eq!(black_pawn.piece_type, PieceType::Pawn);
        assert_eq!(black_pawn.player, Player::Black);
    }

    #[test]
    fn test_from_fen_with_drops_and_promotions() {
        let fen = "8l/1l+R2P3/p2pBG1pp/kps1p4/Nn1P2G2/P1P1P2PP/1PS6/1KSG3+r1/LN2+p3L w Sbgn3p 124";
        let (board, player, captured) = BitboardBoard::from_fen(fen).unwrap();

        assert_eq!(player, Player::White);

        // Check captured pieces
        assert_eq!(
            captured
                .black
                .iter()
                .filter(|&&p| p == PieceType::Silver)
                .count(),
            1
        );
        assert_eq!(
            captured
                .white
                .iter()
                .filter(|&&p| p == PieceType::Pawn)
                .count(),
            3
        );
        assert_eq!(
            captured
                .white
                .iter()
                .filter(|&&p| p == PieceType::Knight)
                .count(),
            1
        );
        assert_eq!(
            captured
                .white
                .iter()
                .filter(|&&p| p == PieceType::Gold)
                .count(),
            1
        );

        // Spot check a few pieces on board
        let promoted_rook = board.get_piece(Position::new(1, 2)).unwrap();
        assert_eq!(promoted_rook.piece_type, PieceType::PromotedRook);
        assert_eq!(promoted_rook.player, Player::Black);

        let promoted_pawn = board.get_piece(Position::new(8, 4)).unwrap();
        assert_eq!(promoted_pawn.piece_type, PieceType::PromotedPawn);
        assert_eq!(promoted_pawn.player, Player::White);
    }
}
