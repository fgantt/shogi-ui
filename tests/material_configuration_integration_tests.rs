use shogi_engine::evaluation::integration::{
    ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator,
};
use shogi_engine::evaluation::material::MaterialEvaluationConfig;
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};
use shogi_engine::BitboardBoard;

#[test]
fn integrated_evaluator_material_toggle_updates_scores_and_clears_caches() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(4, 4),
    );

    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::minimal();
    config.enable_eval_cache = true;
    config.enable_phase_cache = true;
    config.use_optimized_path = false;
    config.material = MaterialEvaluationConfig {
        include_hand_pieces: true,
        use_research_values: true,
        values_path: None,
    };

    let mut evaluator = IntegratedEvaluator::with_config(config.clone());

    let research_score = evaluator.evaluate(&board, Player::Black, &captured);
    assert_ne!(research_score, 0);
    assert!(evaluator.cache_stats().eval_cache_size > 0);
    assert!(evaluator
        .telemetry_snapshot()
        .and_then(|t| t.material)
        .is_some());

    let mut updated_config = config;
    updated_config.material.use_research_values = false;

    evaluator.set_config(updated_config);

    let cache_stats = evaluator.cache_stats();
    assert_eq!(cache_stats.eval_cache_size, 0);
    assert_eq!(cache_stats.phase_cache_size, 0);

    let classic_score = evaluator.evaluate(&board, Player::Black, &captured);
    assert_ne!(classic_score, research_score);
    assert!(evaluator
        .telemetry_snapshot()
        .and_then(|t| t.material)
        .is_some());
}

#[test]
fn integrated_evaluator_resets_material_stats_on_config_update() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(3, 3),
    );

    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::minimal();
    config.enable_eval_cache = false;
    config.enable_phase_cache = false;
    config.use_optimized_path = false;
    config.material = MaterialEvaluationConfig {
        include_hand_pieces: true,
        use_research_values: true,
        values_path: None,
    };

    let mut evaluator = IntegratedEvaluator::with_config(config.clone());
    evaluator.evaluate(&board, Player::Black, &captured);

    assert_eq!(evaluator.material_statistics().evaluations, 1);

    config.material.use_research_values = false;
    evaluator.set_config(config);

    assert_eq!(evaluator.material_statistics().evaluations, 0);
}
