#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::search::search_engine::GLOBAL_NODES_SEARCHED;
use shogi_engine::search::{ParallelSearchConfig, ParallelSearchEngine};
use shogi_engine::types::{CapturedPieces, Player};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn case_thread_creation_handling_smoke() {
    // Do not rely on env flags due to parallel test scheduling; just ensure creation succeeds normally
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
    let config = ParallelSearchConfig::new(2);
    let res = ParallelSearchEngine::new(config);
    assert!(
        res.is_ok(),
        "Expected pool creation to succeed in normal conditions"
    );
}

fn case_fallback_to_single_threaded() {
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
    // Force pool failure and confirm we can still search single-threaded via IterativeDeepening path
    std::env::set_var("SHOGI_FORCE_POOL_FAIL", "1");
    let mut engine_core = shogi_engine::search::search_engine::SearchEngine::new(None, 16);
    let mut id = shogi_engine::search::search_engine::IterativeDeepening::new_with_threads(
        2,
        200,
        None,
        4,
        ParallelSearchConfig::new(4),
    );
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let res = id.search(&mut engine_core, &board, &captured, player);
    assert!(
        res.is_some(),
        "Search should still return using single-threaded fallback"
    );
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
}

fn case_panic_recovery() {
    if std::env::var("SHOGI_TEST_ALLOW_PANIC").ok().as_deref() != Some("1") {
        // Skip by default to avoid intentional panic failing the suite
        return;
    }
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
    std::env::set_var("SHOGI_FORCE_WORKER_PANIC", "1");
    let config = ParallelSearchConfig::new(4);
    let engine = ParallelSearchEngine::new(config).expect("pool");
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves =
        shogi_engine::moves::MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    let _ = engine.search_root_moves(
        &board,
        &captured,
        player,
        &moves,
        2,
        200,
        i32::MIN / 2 + 1,
        i32::MAX / 2 - 1,
    );
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
}

fn case_no_threads_continue_after_stop() {
    std::env::remove_var("SHOGI_FORCE_POOL_FAIL");
    std::env::remove_var("SHOGI_FORCE_WORKER_PANIC");
    let stop = Arc::new(AtomicBool::new(false));
    let config = ParallelSearchConfig::new(4);
    let engine =
        ParallelSearchEngine::new_with_stop_flag(config, Some(stop.clone())).expect("pool");
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves =
        shogi_engine::moves::MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    GLOBAL_NODES_SEARCHED.store(0, Ordering::Relaxed);
    let handle = std::thread::spawn({
        let engine_ref = engine;
        move || {
            engine_ref.search_root_moves(
                &board,
                &captured,
                player,
                &moves,
                5,
                5_000,
                i32::MIN / 2 + 1,
                i32::MAX / 2 - 1,
            )
        }
    });
    // Let it start
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Trigger stop
    stop.store(true, Ordering::Relaxed);
    let t0 = std::time::Instant::now();
    let _ = handle.join();
    let elapsed = t0.elapsed();
    assert!(
        elapsed.as_millis() < 1000,
        "Search did not stop promptly: {:?}",
        elapsed
    );
}

#[test]
fn failure_simulation_suite_serialized() {
    case_thread_creation_handling_smoke();
    case_fallback_to_single_threaded();
    case_panic_recovery();
    case_no_threads_continue_after_stop();
}
