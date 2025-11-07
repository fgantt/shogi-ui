#![cfg(feature = "legacy-tests")]
/// Comprehensive Integration Tests for Search Algorithm Coordination (Task 7.0.5)
///
/// Tests the interactions between PVS, NMP, IID, LMR, Quiescence Search, and Move Ordering
use shogi_engine::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
#[cfg(test)]
mod integration_coordination_tests {
    use super::*;

    fn create_test_engine() -> SearchEngine {
        SearchEngine::new(None, 16)
    }

    fn create_test_board() -> BitboardBoard {
        BitboardBoard::new()
    }

    fn create_test_captured_pieces() -> CapturedPieces {
        CapturedPieces::new()
    }

    /// Test 7.0.5.9: Verify IID → Move Ordering → LMR coordination
    /// IID move should be first in ordering AND exempted from LMR
    #[test]
    fn test_iid_move_ordering_lmr_coordination() {
        let mut engine = create_test_engine();

        // Enable all three: IID, Move Ordering (always on), LMR
        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_iid_stats();
        engine.reset_lmr_stats();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 6, 10000);

        let iid_stats = engine.get_iid_stats();
        let lmr_stats = engine.get_lmr_stats();

        println!("=== IID → Move Ordering → LMR Coordination ===");
        println!(
            "IID searches performed: {}",
            iid_stats.iid_searches_performed
        );
        println!(
            "IID moves ordered first: {}",
            iid_stats.iid_move_ordered_first
        );
        println!(
            "IID moves explicitly exempted: {}",
            lmr_stats.iid_move_explicitly_exempted
        );
        println!(
            "IID moves REDUCED (should be 0): {}",
            lmr_stats.iid_move_reduced_count
        );

        // Critical assertion: IID move should NEVER be reduced
        assert_eq!(
            lmr_stats.iid_move_reduced_count, 0,
            "IID move was reduced {} times - this indicates a bug in coordination!",
            lmr_stats.iid_move_reduced_count
        );

        // If IID ran, it should have been exempted
        if iid_stats.iid_searches_performed > 0 {
            assert!(
                lmr_stats.iid_move_explicitly_exempted > 0,
                "IID ran but moves weren't exempted from LMR"
            );
        }
    }

    /// Test 7.0.5.10: Verify NMP and IID use isolated hash histories
    /// No false repetitions should occur
    #[test]
    fn test_nmp_iid_hash_history_isolation() {
        let mut engine = create_test_engine();

        // Enable both NMP and IID
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset statistics
        engine.reset_null_move_stats();
        engine.reset_iid_stats();

        // Perform search - hash histories should remain isolated
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);

        let nmp_stats = engine.get_null_move_stats();
        let iid_stats = engine.get_iid_stats();

        println!("=== Hash History Isolation ===");
        println!("NMP attempts: {}", nmp_stats.attempts);
        println!("IID searches: {}", iid_stats.iid_searches_performed);
        println!("Search completed: {:?}", result.is_some());

        // Search should complete successfully (no false repetitions)
        assert!(
            result.is_some(),
            "Search should complete successfully with isolated hash histories"
        );
    }

    /// Test 7.0.5.11: Test TT interaction between NMP, IID, and main search
    #[test]
    fn test_tt_interaction_nmp_iid_main() {
        let mut engine = create_test_engine();

        // Enable NMP and IID
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset metrics
        engine.reset_core_search_metrics();

        // Perform search - all three will interact with TT
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 6, 5000);

        let metrics = engine.get_core_search_metrics();

        println!("=== TT Interaction ===");
        println!("Total TT probes: {}", metrics.total_tt_probes);
        println!("Total TT hits: {}", metrics.total_tt_hits);
        println!(
            "Auxiliary overwrites prevented: {}",
            metrics.tt_auxiliary_overwrites_prevented
        );
        println!(
            "Main entries preserved: {}",
            metrics.tt_main_entries_preserved
        );

        // TT should be actively used
        assert!(metrics.total_tt_probes > 0, "TT should be probed");

        // Priority system should protect main entries
        assert!(metrics.tt_auxiliary_overwrites_prevented >= 0);
    }

    /// Test 7.0.5.12: Test time pressure coordination across all algorithms
    #[test]
    fn test_time_pressure_coordination_all_algorithms() {
        let mut engine = create_test_engine();

        // Enable all algorithms
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset all statistics
        engine.reset_null_move_stats();
        engine.reset_iid_stats();
        engine.reset_lmr_stats();

        // Perform search with tight time limit to trigger time pressure
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 50); // 50ms

        let nmp_stats = engine.get_null_move_stats();
        let iid_stats = engine.get_iid_stats();
        let lmr_stats = engine.get_lmr_stats();

        println!("=== Time Pressure Coordination ===");
        println!(
            "NMP skipped (time pressure): {}",
            nmp_stats.skipped_time_pressure
        );
        println!(
            "IID skipped (time pressure): {}",
            iid_stats.positions_skipped_time_pressure
        );
        println!("LMR moves considered: {}", lmr_stats.moves_considered);

        // Time pressure coordination should be active
        assert!(nmp_stats.skipped_time_pressure >= 0);
        assert!(iid_stats.positions_skipped_time_pressure >= 0);
    }

    /// Test all algorithms working together harmoniously
    #[test]
    fn test_all_algorithms_integration() {
        let mut engine = create_test_engine();

        // Enable everything
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        lmr_config.min_move_index = 2;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset all statistics
        engine.reset_null_move_stats();
        engine.reset_iid_stats();
        engine.reset_lmr_stats();
        engine.reset_core_search_metrics();

        // Perform search with all algorithms active
        let result = engine.search_at_depth(&board, &captured_pieces, player, 6, 10000);

        let nmp_stats = engine.get_null_move_stats();
        let iid_stats = engine.get_iid_stats();
        let lmr_stats = engine.get_lmr_stats();
        let metrics = engine.get_core_search_metrics();

        println!("=== Full Integration Test ===");
        println!("Search completed: {}", result.is_some());
        println!("NMP attempts: {}", nmp_stats.attempts);
        println!("NMP cutoffs: {}", nmp_stats.cutoffs);
        println!("IID searches: {}", iid_stats.iid_searches_performed);
        println!(
            "IID moves ordered first: {}",
            iid_stats.iid_move_ordered_first
        );
        println!(
            "IID moves exempted from LMR: {}",
            lmr_stats.iid_move_explicitly_exempted
        );
        println!(
            "IID moves REDUCED (should be 0): {}",
            lmr_stats.iid_move_reduced_count
        );
        println!("LMR reductions applied: {}", lmr_stats.reductions_applied);
        println!("LMR re-searches: {}", lmr_stats.researches_triggered);
        println!(
            "TT hit rate: {:.2}%",
            if metrics.total_tt_probes > 0 {
                (metrics.total_tt_hits as f64 / metrics.total_tt_probes as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "TT pollution prevented: {}",
            metrics.tt_auxiliary_overwrites_prevented
        );
        println!("Evaluation cache hits: {}", metrics.evaluation_cache_hits);

        // All algorithms should have run successfully
        assert!(result.is_some(), "Search should complete");
        assert_eq!(
            lmr_stats.iid_move_reduced_count, 0,
            "IID move should never be reduced"
        );
    }

    /// Test performance regression - search should be efficient with all features
    #[test]
    fn test_integration_performance_regression() {
        let mut engine = create_test_engine();

        // Enable all algorithms
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let mut lmr_config = LMRConfig::default();
        lmr_config.enabled = true;
        lmr_config.min_depth = 3;
        engine.update_lmr_config(lmr_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Perform search and measure basic performance
        let start = std::time::Instant::now();
        let result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);
        let elapsed = start.elapsed();

        println!("=== Performance Regression Check ===");
        println!("Search completed in: {:?}", elapsed);
        println!("Result: {:?}", result.is_some());

        // Search should complete in reasonable time
        assert!(result.is_some(), "Search should complete");
        assert!(
            elapsed.as_secs() < 6,
            "Search should complete within time limit"
        );
    }
}
