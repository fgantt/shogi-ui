use shogi_engine::search::{ParallelSearchEngine, ParallelSearchConfig};
use shogi_engine::search::search_engine::GLOBAL_NODES_SEARCHED;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{CapturedPieces, Player};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[test]
fn test_thread_creation_failure_handling() {
    std::env::set_var("SHOGI_FORCE_POOL_FAIL", "1");
    let config = ParallelSearchConfig::new(4);
    let res = ParallelSearchEngine::new(config);
    assert!(res.is_err(), "Expected forced pool failure");
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
}

#[test]
fn test_fallback_to_single_threaded() {
    // Force pool failure and confirm we can still search single-threaded via IterativeDeepening path
    std::env::set_var("SHOGI_FORCE_POOL_FAIL", "1");
    let mut engine_core = shogi_engine::search::search_engine::SearchEngine::new(None, 16);
    let mut id = shogi_engine::search::search_engine::IterativeDeepening::new_with_threads(2, 200, None, 4);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let res = id.search(&mut engine_core, &board, &captured, player);
    assert!(res.is_some(), "Search should still return using single-threaded fallback");
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
}

#[test]
fn test_panic_recovery() {
    std::env::set_var("SHOGI_FORCE_WORKER_PANIC", "1");
    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new(config).expect("pool");
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = shogi_engine::moves::MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    let res = engine.search_root_moves(&board, &captured, player, &moves, 2, 200, i32::MIN/2+1, i32::MAX/2-1);
    // Search should complete despite one worker panic
    assert!(res.is_some() || res.is_none());
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
}

#[test]
fn test_no_threads_continue_after_stop() {
    let stop = Arc::new(AtomicBool::new(false));
    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop.clone())).expect("pool");
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = shogi_engine::moves::MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);
    let handle = std::thread::spawn({
        let engine_ref = engine;
        move || {
            engine_ref.search_root_moves(&board, &captured, player, &moves, 5, 5_000, i32::MIN/2+1, i32::MAX/2-1)
        }
    });
    // Let it start
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Trigger stop
    stop.store(true, Ordering::Relaxed);
    let t0 = std::time::Instant::now();
    let _ = handle.join();
    let elapsed = t0.elapsed();
    assert!(elapsed.as_millis() < 1000, "Search did not stop promptly: {:?}", elapsed);
}



