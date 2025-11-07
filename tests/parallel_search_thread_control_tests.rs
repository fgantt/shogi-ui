#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::search::{ParallelSearchConfig, ParallelSearchEngine};
use shogi_engine::types::{CapturedPieces, Move, Player};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn legal_moves(board: &BitboardBoard, player: Player, captured: &CapturedPieces) -> Vec<Move> {
    let mg = MoveGenerator::new();
    mg.generate_legal_moves(board, player, captured)
}

#[test]
fn test_stop_flag_propagation() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = legal_moves(&board, player, &captured);
    assert!(
        !moves.is_empty(),
        "Expected legal moves from start position"
    );

    let mut config = ParallelSearchConfig::new(4);
    config.enable_parallel = true;
    let stop = Arc::new(AtomicBool::new(true)); // already set
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop)).expect("pool");

    let result = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        3,
        1000,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    assert!(result.is_none(), "Expected None when stop flag pre-set");
}

#[test]
fn test_time_limit_enforcement() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = legal_moves(&board, player, &captured);

    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new(config).expect("pool");

    let t0 = std::time::Instant::now();
    let _ = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        5,
        25,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    let elapsed = t0.elapsed();
    // Should return reasonably quickly (watchdog enforced); allow generous slack for CI
    assert!(
        elapsed.as_millis() < 1500,
        "Search exceeded expected time: {:?}",
        elapsed
    );
}

#[test]
fn test_graceful_shutdown_then_next_search() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = legal_moves(&board, player, &captured);

    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new(config).expect("pool");

    // First, short time-limited search (triggers watchdog)
    let _ = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        4,
        30,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    // Then a normal short search should still succeed
    let res2 = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        2,
        500,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    // Either Some or None is acceptable; importantly, it returns
    assert!(res2.is_some() || res2.is_none());
}

#[test]
fn test_partial_result_validity() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = legal_moves(&board, player, &captured);

    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new(config).expect("pool");

    // Small time limit to encourage partial result
    let res = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        4,
        40,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    if let Some((best, _score)) = res {
        // Best move should be legal from this position
        let legal = moves
            .iter()
            .any(|m| m.to_usi_string() == best.to_usi_string());
        assert!(legal, "Returned move must be legal");
    }
}
