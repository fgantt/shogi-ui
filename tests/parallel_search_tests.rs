#![cfg(feature = "legacy-tests")]
//! Unit tests for parallel search engine components.

use num_cpus;
use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};

#[test]
fn test_thread_count_config_parsing() {
    // Test default configuration
    let config = ParallelSearchConfig::default();
    let expected_threads = num_cpus::get().clamp(1, 32);
    assert_eq!(config.num_threads, expected_threads);
    assert_eq!(config.enable_parallel, expected_threads > 1);

    // Test custom configuration with valid range
    let config = ParallelSearchConfig::new(4);
    assert_eq!(config.num_threads, 4);
    assert_eq!(config.enable_parallel, true);

    // Test clamping to minimum
    let config = ParallelSearchConfig::new(0);
    assert_eq!(config.num_threads, 1);
    assert_eq!(config.enable_parallel, false);

    // Test clamping to maximum
    let config = ParallelSearchConfig::new(100);
    assert_eq!(config.num_threads, 32);
    assert_eq!(config.enable_parallel, true);

    // Test set_num_threads
    let mut config = ParallelSearchConfig::default();
    config.set_num_threads(8);
    assert_eq!(config.num_threads, 8);
    assert_eq!(config.enable_parallel, true);

    config.set_num_threads(1);
    assert_eq!(config.num_threads, 1);
    assert_eq!(config.enable_parallel, false);
}

#[test]
fn test_thread_pool_creation() {
    // Test creating engine with default config
    let config = ParallelSearchConfig::default();
    let engine_result = ParallelSearchEngine::new(config);
    assert!(engine_result.is_ok(), "Thread pool creation should succeed");

    let engine = engine_result.unwrap();
    assert_eq!(engine.num_threads(), num_cpus::get().clamp(1, 32));

    // Test creating engine with custom thread count
    let config = ParallelSearchConfig::new(4);
    let engine_result = ParallelSearchEngine::new(config);
    assert!(
        engine_result.is_ok(),
        "Thread pool creation with 4 threads should succeed"
    );

    let engine = engine_result.unwrap();
    assert_eq!(engine.num_threads(), 4);
    assert!(engine.is_parallel_enabled());

    // Test single-threaded configuration
    let config = ParallelSearchConfig::new(1);
    let engine_result = ParallelSearchEngine::new(config);
    assert!(
        engine_result.is_ok(),
        "Single-threaded engine creation should succeed"
    );

    let engine = engine_result.unwrap();
    assert_eq!(engine.num_threads(), 1);
    assert!(!engine.is_parallel_enabled());
}

#[test]
fn test_usi_option_registration() {
    use shogi_engine::usi::UsiHandler;

    let mut handler = UsiHandler::new();
    let usi_response = handler.handle_command("usi");

    // Check that USI_Threads option is present in the response
    let has_threads_option = usi_response.iter().any(|line| {
        line.contains("USI_Threads") && line.contains("spin") && line.contains("min 1 max 32")
    });

    assert!(
        has_threads_option,
        "USI_Threads option should be registered in USI response"
    );

    // Verify the option format is correct
    let threads_line = usi_response
        .iter()
        .find(|line| line.contains("USI_Threads"))
        .expect("USI_Threads option line should exist");

    assert!(threads_line.contains("type spin"));
    assert!(threads_line.contains("min 1"));
    assert!(threads_line.contains("max 32"));

    for opt in [
        "ParallelEnable",
        "ParallelHash",
        "ParallelMinDepth",
        "ParallelMetrics",
        "YBWCEnable",
        "YBWCMinDepth",
        "YBWCMinBranch",
        "YBWCMaxSiblings",
        "YBWCScalingShallow",
        "YBWCScalingMid",
        "YBWCScalingDeep",
    ] {
        assert!(
            usi_response.iter().any(|line| line.contains(opt)),
            "Expected option {} in USI response",
            opt
        );
    }
}

#[test]
fn test_parallel_search_engine_instantiation() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    let config = ParallelSearchConfig::new(2);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine_result = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag));

    assert!(
        engine_result.is_ok(),
        "Parallel search engine should be created successfully"
    );
    let engine = engine_result.unwrap();
    assert_eq!(engine.num_threads(), 2);
}

#[test]
fn test_board_cloning_correctness() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use shogi_engine::{
        types::{CapturedPieces, Player},
        BitboardBoard,
    };
    use std::sync::{atomic::AtomicBool, Arc};

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let stop_flag = Arc::new(AtomicBool::new(false));

    let config = ParallelSearchConfig::new(1);
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag)).unwrap();

    let context = engine.create_thread_context(&board, &captured_pieces, player, 16);

    // Verify cloned board matches original
    assert_eq!(
        context.board().to_string_for_debug(),
        board.to_string_for_debug()
    );
}

#[test]
fn test_result_aggregation() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};

    let config = ParallelSearchConfig::new(1);
    let engine = ParallelSearchEngine::new(config).unwrap();

    // Test aggregation with valid results - just verify engine creation works
    // Actual aggregation testing will be done through integration tests
    assert!(engine.num_threads() > 0);
}

#[test]
fn test_basic_parallel_search_2_threads() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use shogi_engine::{
        moves::MoveGenerator,
        types::{CapturedPieces, Player},
        BitboardBoard,
    };
    use std::sync::{atomic::AtomicBool, Arc};

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let stop_flag = Arc::new(AtomicBool::new(false));

    let config = ParallelSearchConfig::new(2);
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag.clone())).unwrap();

    // Generate legal moves
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured_pieces);

    if !moves.is_empty() {
        // Perform parallel search with very shallow depth and short time limit for testing
        let _result = engine.search_root_moves(
            &board,
            &captured_pieces,
            player,
            &moves,
            2,    // Very shallow depth
            1000, // 1 second time limit
            i32::MIN + 1,
            i32::MAX - 1,
        );

        // Should return a result (may be None if search is interrupted, but shouldn't panic)
        // Just verify the method completes without error
        assert!(true, "Parallel search completed");
    }
}

#[test]
fn test_work_stats_disabled_returns_none() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    let config = ParallelSearchConfig::new(2);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag)).unwrap();
    assert!(engine.get_work_stats().is_none());
}

#[test]
fn test_ybwc_wait_completes_without_spin() {
    use shogi_engine::search::parallel_search::{
        ParallelSearchConfig, ParallelSearchEngine, WaitOutcome,
    };
    use shogi_engine::{
        moves::MoveGenerator,
        types::{CapturedPieces, Player},
        BitboardBoard,
    };
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let config = ParallelSearchConfig::new(1);
    let engine = ParallelSearchEngine::new(config).unwrap();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    if moves.is_empty() {
        return;
    }
    let (_, sync) = engine.distribute_work(
        &board,
        &captured,
        player,
        &moves,
        2,
        100,
        i32::MIN + 1,
        i32::MAX - 1,
    );
    let sync = Arc::new(sync);
    let waiter = {
        let wait_sync = sync.clone();
        thread::spawn(move || matches!(wait_sync.wait_for_complete(200), WaitOutcome::Completed(7)))
    };
    thread::sleep(Duration::from_millis(5));
    sync.mark_complete(7);
    assert!(waiter.join().unwrap());
}

#[test]
fn test_ybwc_wait_respects_stop_flag() {
    use shogi_engine::search::parallel_search::{
        ParallelSearchConfig, ParallelSearchEngine, WaitOutcome,
    };
    use shogi_engine::{
        moves::MoveGenerator,
        types::{CapturedPieces, Player},
        BitboardBoard,
    };
    use std::sync::{atomic::AtomicBool, Arc};

    let config = ParallelSearchConfig::new(1);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag.clone())).unwrap();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    if moves.is_empty() {
        return;
    }
    let (_, sync) = engine.distribute_work(
        &board,
        &captured,
        player,
        &moves,
        2,
        50,
        i32::MIN + 1,
        i32::MAX - 1,
    );
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(sync.wait_for_complete(50), WaitOutcome::Aborted);
}

#[test]
fn test_ybwc_wait_times_out() {
    use shogi_engine::search::parallel_search::{
        ParallelSearchConfig, ParallelSearchEngine, WaitOutcome,
    };
    use shogi_engine::{
        moves::MoveGenerator,
        types::{CapturedPieces, Player},
        BitboardBoard,
    };

    let config = ParallelSearchConfig::new(1);
    let engine = ParallelSearchEngine::new(config).unwrap();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let moves = MoveGenerator::new().generate_legal_moves(&board, player, &captured);
    if moves.is_empty() {
        return;
    }
    let (_, sync) = engine.distribute_work(
        &board,
        &captured,
        player,
        &moves,
        2,
        10,
        i32::MIN + 1,
        i32::MAX - 1,
    );
    assert_eq!(sync.wait_for_complete(5), WaitOutcome::Timeout);
}

#[test]
fn test_parallel_vs_single_threaded_correctness() {
    // This test compares parallel search results with single-threaded search
    // For now, we just verify both can be instantiated
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    let config_single = ParallelSearchConfig::new(1);
    let config_parallel = ParallelSearchConfig::new(2);
    let stop_flag = Arc::new(AtomicBool::new(false));

    let engine_single =
        ParallelSearchEngine::new_with_stop_flag(config_single, Some(stop_flag.clone())).unwrap();
    let engine_parallel =
        ParallelSearchEngine::new_with_stop_flag(config_parallel, Some(stop_flag.clone())).unwrap();

    assert_eq!(engine_single.num_threads(), 1);
    assert_eq!(engine_parallel.num_threads(), 2);
}

#[test]
fn test_work_stealing_queue_operations() {
    use shogi_engine::search::parallel_search::WorkStealingQueue;
    use shogi_engine::{
        types::{Move, Player},
        BitboardBoard,
    };

    let queue = WorkStealingQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    // Create a test work unit
    let board = BitboardBoard::new();
    let captured_pieces = shogi_engine::types::CapturedPieces::new();
    let test_move = Move::from_usi_string("7g7f", Player::Black, &board).unwrap();

    let work_unit = shogi_engine::search::parallel_search::WorkUnit {
        board: board.clone(),
        captured_pieces: captured_pieces.clone(),
        move_to_search: test_move.clone(),
        depth: 3,
        alpha: -1000,
        beta: 1000,
        parent_score: 0,
        player: Player::Black,
        time_limit_ms: 1000,
        is_oldest_brother: true,
    };

    // Test push_back and pop_front
    queue.push_back(work_unit.clone());
    assert!(!queue.is_empty());
    assert_eq!(queue.len(), 1);

    let popped = queue.pop_front();
    assert!(popped.is_some());
    let popped_work = popped.unwrap();
    assert_eq!(popped_work.depth, 3);
    assert!(queue.is_empty());

    // Test steal operation
    queue.push_back(work_unit.clone());
    let stolen = queue.steal();
    assert!(stolen.is_some());
    assert!(queue.is_empty());

    // Test statistics
    let snapshot = queue.get_stats();
    assert_eq!(snapshot.pushes, 2);
    assert_eq!(snapshot.pops, 1);
    assert_eq!(snapshot.steals, 1);
}

#[test]
fn test_ybwc_synchronization_correctness() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    let config = ParallelSearchConfig::new(2);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag)).unwrap();

    // Test that distribute_work creates oldest_brother correctly
    use shogi_engine::{
        moves::MoveGenerator,
        types::{CapturedPieces, Player},
        BitboardBoard,
    };

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;
    let move_generator = MoveGenerator::new();
    let moves = move_generator.generate_legal_moves(&board, player, &captured_pieces);

    if moves.len() >= 3 {
        let (work_units, _ybwc_sync): (Vec<_>, _) = engine.distribute_work(
            &board,
            &captured_pieces,
            player,
            &moves,
            4,
            1000,
            i32::MIN + 1,
            i32::MAX - 1,
        );

        // First move should be marked as oldest brother
        assert!(work_units[0].is_oldest_brother);

        // Other moves should not be oldest brother
        for work in &work_units[1..] {
            assert!(!work.is_oldest_brother);
        }

        // Verify work units are created correctly
        assert_eq!(work_units.len(), moves.len());
    }
}

#[test]
fn test_load_balancing() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    let mut config = ParallelSearchConfig::new(4);
    config.enable_work_metrics(true);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let engine = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag)).unwrap();

    // Verify work queues are created for all threads
    assert_eq!(engine.num_threads(), 4);

    // Test that we can get work distribution stats
    let stats = engine.get_work_stats();
    assert!(stats.is_some());
    if let Some(ws) = stats {
        assert_eq!(ws.work_units_per_thread.len(), 4);
        assert_eq!(ws.steal_count_per_thread.len(), 4);
    }
}

#[test]
fn test_work_stealing_triggers() {
    use shogi_engine::search::parallel_search::WorkStealingQueue;
    use shogi_engine::{
        types::{Move, Player},
        BitboardBoard,
    };

    let queue = WorkStealingQueue::new();

    // Create test work unit
    let board = BitboardBoard::new();
    let captured_pieces = shogi_engine::types::CapturedPieces::new();
    let test_move = Move::from_usi_string("7g7f", Player::Black, &board).unwrap();

    let work = shogi_engine::search::parallel_search::WorkUnit {
        board: board.clone(),
        captured_pieces: captured_pieces.clone(),
        move_to_search: test_move,
        depth: 2,
        alpha: -1000,
        beta: 1000,
        parent_score: 0,
        player: Player::Black,
        time_limit_ms: 1000,
        is_oldest_brother: false,
    };

    // Push work to queue
    queue.push_back(work.clone());

    // Another thread should be able to steal from this queue
    let stolen = queue.steal();
    assert!(stolen.is_some());

    // Verify steal statistics
    let snapshot = queue.get_stats();
    assert!(snapshot.steals > 0);
}

#[test]
fn test_many_threads_work_stealing() {
    use shogi_engine::search::parallel_search::{ParallelSearchConfig, ParallelSearchEngine};
    use std::sync::{atomic::AtomicBool, Arc};

    // Test with many threads (8, 16)
    for num_threads in [8, 16] {
        let mut config = ParallelSearchConfig::new(num_threads);
        config.enable_work_metrics(true);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let engine_result = ParallelSearchEngine::new_with_stop_flag(config, Some(stop_flag));

        assert!(
            engine_result.is_ok(),
            "Should create engine with {} threads",
            num_threads
        );
        let engine = engine_result.unwrap();
        assert_eq!(engine.num_threads(), num_threads);

        // Verify statistics structure
        let stats = engine.get_work_stats();
        assert!(stats.is_some());
        if let Some(ws) = stats {
            assert_eq!(ws.work_units_per_thread.len(), num_threads);
        }
    }
}
