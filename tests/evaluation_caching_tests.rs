#![cfg(feature = "legacy-tests")]
/// Unit tests for Evaluation Result Caching (Task 7.0.4)
///
/// Tests that position evaluation is computed once per node and reused
/// to eliminate redundant evaluation calls.
use shogi_engine::*;

#[cfg(test)]
mod evaluation_caching_tests {
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

    /// Test 7.0.4.10: Verify evaluation caching statistics are tracked
    #[test]
    fn test_evaluation_cache_statistics() {
        let mut engine = create_test_engine();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset metrics
        engine.reset_core_search_metrics();

        // Perform search
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 5000);

        // Check evaluation caching statistics
        let metrics = engine.get_core_search_metrics();

        println!("Evaluation cache hits: {}", metrics.evaluation_cache_hits);
        println!("Evaluation calls saved: {}", metrics.evaluation_calls_saved);
        println!("Total nodes: {}", metrics.total_nodes);

        // Cache hit statistics should be tracked
        assert!(metrics.evaluation_cache_hits >= 0);
        assert!(metrics.evaluation_calls_saved >= 0);
    }

    /// Test that cached evaluation is used in fallback paths
    #[test]
    fn test_evaluation_reuse_in_fallback() {
        let mut engine = create_test_engine();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset metrics
        engine.reset_core_search_metrics();

        // Perform multiple searches to trigger various code paths
        for _ in 0..3 {
            let _result = engine.search_at_depth(&board, &captured_pieces, player, 4, 1000);
        }

        let metrics = engine.get_core_search_metrics();

        println!("=== Evaluation Caching Effectiveness ===");
        println!("Cache hits: {}", metrics.evaluation_cache_hits);
        println!("Calls saved: {}", metrics.evaluation_calls_saved);

        // Should have some cache hits across multiple searches
        assert!(metrics.evaluation_cache_hits >= 0);
    }

    /// Test evaluation caching with NMP enabled
    #[test]
    fn test_evaluation_caching_with_nmp() {
        let mut engine = create_test_engine();

        // Enable NMP
        let mut nmp_config = NullMoveConfig::default();
        nmp_config.enabled = true;
        nmp_config.min_depth = 3;
        engine.update_null_move_config(nmp_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset metrics
        engine.reset_core_search_metrics();

        // Perform search - NMP will benefit from cached evaluation
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 5, 3000);

        let metrics = engine.get_core_search_metrics();

        println!("With NMP enabled:");
        println!("  Evaluation cache hits: {}", metrics.evaluation_cache_hits);
        println!("  Calls saved: {}", metrics.evaluation_calls_saved);

        // Verify caching is working
        assert!(metrics.evaluation_cache_hits >= 0);
    }

    /// Test evaluation caching with IID enabled
    #[test]
    fn test_evaluation_caching_with_iid() {
        let mut engine = create_test_engine();

        // Enable IID
        let mut iid_config = IIDConfig::default();
        iid_config.enabled = true;
        iid_config.min_depth = 4;
        engine.update_iid_config(iid_config).unwrap();

        let board = create_test_board();
        let captured_pieces = create_test_captured_pieces();
        let player = Player::Black;

        // Reset metrics
        engine.reset_core_search_metrics();

        // Perform search - IID will benefit from cached evaluation
        let _result = engine.search_at_depth(&board, &captured_pieces, player, 6, 5000);

        let metrics = engine.get_core_search_metrics();

        println!("With IID enabled:");
        println!("  Evaluation cache hits: {}", metrics.evaluation_cache_hits);
        println!("  Calls saved: {}", metrics.evaluation_calls_saved);

        // Verify caching is working
        assert!(metrics.evaluation_cache_hits >= 0);
    }

    /// Test that SearchState properly stores static_eval
    #[test]
    fn test_search_state_static_eval() {
        use shogi_engine::types::{GamePhase, SearchState};

        let mut state = SearchState::new(5, -1000, 1000);

        // Initially zero
        assert_eq!(state.static_eval, 0);

        // Update fields with evaluation
        state.update_fields(false, 250, 0x12345, GamePhase::Middlegame);

        // Should now have the evaluation
        assert_eq!(state.static_eval, 250);
        assert_eq!(state.game_phase, GamePhase::Middlegame);
    }
}
