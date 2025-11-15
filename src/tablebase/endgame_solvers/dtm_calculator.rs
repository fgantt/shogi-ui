//! Distance To Mate (DTM) Calculator
//!
//! This module provides utilities for calculating the actual distance to mate
//! in endgame positions using iterative deepening search.

use crate::bitboards::BitboardBoard;
use crate::moves::MoveGenerator;
use crate::types::board::CapturedPieces;
use crate::types::core::{Move, Player};

/// Calculate the distance to mate using iterative deepening search
///
/// This function performs a lightweight mate-finding search to determine
/// the actual number of moves needed to force checkmate.
///
/// # Arguments
/// * `board` - The current board position
/// * `player` - The player to move (attacking player)
/// * `captured_pieces` - The captured pieces (must be provided for correct board state)
/// * `max_depth` - Maximum search depth (default 30 moves)
///
/// # Returns
/// * `Some(distance)` - Distance to mate in moves if mate is found
/// * `None` - If mate cannot be found within max_depth
const NODE_LIMIT: usize = 25_000;

pub fn calculate_dtm(
    board: &BitboardBoard,
    player: Player,
    captured_pieces: &CapturedPieces,
    max_depth: u8,
) -> Option<u8> {
    // Check if already in checkmate (mate in 0 moves)
    let opponent = player.opposite();
    if board.is_checkmate(opponent, captured_pieces) {
        return Some(0);
    }

    // Use iterative deepening to find shortest path to mate
    let move_generator = MoveGenerator::new();
    let mut nodes_explored = 0usize;

    for depth in 1..=max_depth {
        if nodes_explored >= NODE_LIMIT {
            break;
        }
        if let Some(_mate) = find_mate_at_depth(
            board,
            player,
            captured_pieces,
            depth,
            &move_generator,
            &mut nodes_explored,
            NODE_LIMIT,
        ) {
            return Some(depth);
        }
    }

    None
}

/// Find mate at a specific depth using depth-first search
fn find_mate_at_depth(
    board: &BitboardBoard,
    player: Player,
    captured_pieces: &CapturedPieces,
    depth: u8,
    move_generator: &MoveGenerator,
    nodes_explored: &mut usize,
    node_limit: usize,
) -> Option<Move> {
    if *nodes_explored >= node_limit {
        return None;
    }

    *nodes_explored += 1;

    if depth == 0 {
        // At depth 0, check if opponent is in checkmate
        let opponent = player.opposite();
        if board.is_checkmate(opponent, captured_pieces) {
            // Return a dummy move - we only care about finding mate, not the exact move
            return Some(Move::new_move(
                crate::types::Position::new(0, 0),
                crate::types::Position::new(0, 0),
                crate::types::PieceType::King,
                player,
                false,
            ));
        }
        return None;
    }

    // Generate all legal moves for current player
    let moves = move_generator.generate_legal_moves(board, player, captured_pieces);

    // If no legal moves, this is stalemate or checkmate (opponent's perspective)
    if moves.is_empty() {
        return None;
    }

    // Try each move and see if it leads to mate at the target depth
    for move_ in &moves {
        if *nodes_explored >= node_limit {
            break;
        }
        let mut temp_board = board.clone();
        let mut temp_captured = captured_pieces.clone();

        // Make the move
        if let Some(captured) = temp_board.make_move(move_) {
            temp_captured.add_piece(captured.piece_type, player);
        }

        // Check if this move leads to mate at depth-1 for opponent
        let opponent = player.opposite();
        if find_mate_at_depth(
            &temp_board,
            opponent,
            &temp_captured,
            depth - 1,
            move_generator,
            nodes_explored,
            node_limit,
        )
        .is_some()
        {
            return Some(move_.clone());
        }
    }

    None
}

/// Calculate DTM with caching to avoid redundant searches
pub fn calculate_dtm_with_cache(
    board: &BitboardBoard,
    player: Player,
    captured_pieces: &CapturedPieces,
    max_depth: u8,
    cache: &mut std::collections::HashMap<u64, Option<u8>>,
) -> Option<u8> {
    // Create a composite key from position hash and player
    use crate::search::BoardTrait;
    let position_hash = board.get_position_hash(captured_pieces);
    let player_key = if player == Player::Black { 0u64 } else { 1u64 };
    let cache_key = position_hash ^ player_key;

    // Check cache first
    if let Some(cached_dtm) = cache.get(&cache_key) {
        return *cached_dtm;
    }

    // Calculate DTM
    let dtm = calculate_dtm(board, player, captured_pieces, max_depth);

    // Cache result
    cache.insert(cache_key, dtm);

    dtm
}
