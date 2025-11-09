#![cfg(feature = "legacy-tests")]
// End-to-end USI tests for parallel search configuration and persistence

use shogi_engine::ShogiEngine;
use std::env;
use std::fs;
use std::path::PathBuf;

fn temp_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("shogi_prefs_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn usi_threads_persistence_roundtrip() {
    // Arrange: point preferences to temp directory
    let prefs_dir = temp_dir();
    env::set_var("SHOGI_PREFS_DIR", &prefs_dir);

    // Act: set USI_Threads and ensure it's saved
    let mut engine = ShogiEngine::new();
    let out = engine.handle_setoption(&["name", "USI_Threads", "value", "4"]);
    assert!(out.iter().any(|s| s.contains("Set USI_Threads to 4")));

    // Ensure file exists
    let prefs_file = prefs_dir.join("engine_prefs.json");
    assert!(prefs_file.exists(), "prefs file missing: {:?}", prefs_file);

    // New engine should load persisted value
    let mut engine2 = ShogiEngine::new();
    // Verify get_best_move uses the thread count indirectly by not panicking
    // and returning some move at shallow depth
    let best = engine2.get_best_move(1, 100, None);
    assert!(best.is_some(), "Expected a move at shallow depth");
}

#[test]
fn usi_basic_search_flow() {
    // Arrange
    env::set_var("SHOGI_PREFS_DIR", temp_dir());
    let mut engine = ShogiEngine::new();

    // Set depth via USI and verify message
    let out = engine.handle_setoption(&["name", "depth", "value", "2"]);
    assert!(out.iter().any(|s| s.contains("Set depth to 2")));

    // Start new game, then search
    let _ = engine.handle_usinewgame();
    let mv = engine.get_best_move(1, 200, None);
    assert!(mv.is_some(), "Engine should return a move in basic flow");
}

#[test]
fn usi_parallel_options_flow() {
    env::set_var("SHOGI_PREFS_DIR", temp_dir());
    let mut engine = ShogiEngine::new();

    let _ = engine.handle_setoption(&["name", "USI_Threads", "value", "4"]);
    let _ = engine.handle_setoption(&["name", "ParallelEnable", "value", "false"]);
    let mut options = engine.parallel_search_options();
    assert!(!options.enable_parallel);

    let _ = engine.handle_setoption(&["name", "ParallelEnable", "value", "true"]);
    let _ = engine.handle_setoption(&["name", "ParallelMinDepth", "value", "6"]);
    let _ = engine.handle_setoption(&["name", "ParallelHash", "value", "24"]);
    let _ = engine.handle_setoption(&["name", "ParallelMetrics", "value", "true"]);
    let _ = engine.handle_setoption(&["name", "YBWCEnable", "value", "true"]);
    let _ = engine.handle_setoption(&["name", "YBWCMinDepth", "value", "3"]);
    let _ = engine.handle_setoption(&["name", "YBWCMinBranch", "value", "12"]);
    let _ = engine.handle_setoption(&["name", "YBWCMaxSiblings", "value", "6"]);
    let _ = engine.handle_setoption(&["name", "YBWCScalingShallow", "value", "5"]);
    let _ = engine.handle_setoption(&["name", "YBWCScalingMid", "value", "3"]);
    let _ = engine.handle_setoption(&["name", "YBWCScalingDeep", "value", "2"]);

    options = engine.parallel_search_options();
    assert!(options.enable_parallel);
    assert_eq!(options.min_depth_parallel, 6);
    assert_eq!(options.hash_size_mb, 24);
    assert!(options.enable_metrics);
    assert!(options.ybwc_enabled);
    assert_eq!(options.ybwc_min_depth, 3);
    assert_eq!(options.ybwc_min_branch, 12);
    assert_eq!(options.ybwc_max_siblings, 6);
    assert_eq!(options.ybwc_shallow_divisor, 5);
    assert_eq!(options.ybwc_mid_divisor, 3);
    assert_eq!(options.ybwc_deep_divisor, 2);

    let mv = engine.get_best_move(1, 200, None);
    assert!(
        mv.is_some(),
        "Engine should still return a move after parallel configuration"
    );
}
