use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::{
    config::EvaluationWeights,
    integration::{ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator},
    position_features::{PositionFeatureConfig, PositionFeatureEvaluator},
};
use shogi_engine::types::{CapturedPieces, Player, TaperedScore};

#[test]
fn position_feature_toggles_skip_computation_and_statistics() {
    let config = PositionFeatureConfig {
        enable_king_safety: false,
        enable_pawn_structure: false,
        enable_mobility: false,
        enable_center_control: false,
        enable_development: false,
    };

    let mut evaluator = PositionFeatureEvaluator::with_config(config);
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    assert_eq!(
        evaluator.evaluate_king_safety(&board, Player::Black),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().king_safety_evals, 0);

    assert_eq!(
        evaluator.evaluate_pawn_structure(&board, Player::Black),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().pawn_structure_evals, 0);

    assert_eq!(
        evaluator.evaluate_mobility(&board, Player::Black, &captured),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().mobility_evals, 0);

    assert_eq!(
        evaluator.evaluate_center_control(&board, Player::Black),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().center_control_evals, 0);

    assert_eq!(
        evaluator.evaluate_development(&board, Player::Black),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().development_evals, 0);
}

#[test]
fn integrated_evaluator_respects_position_feature_weights() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::all_disabled();
    config.components.position_features = true;
    config.enable_phase_cache = false;
    config.enable_eval_cache = false;
    config.use_optimized_path = false;
    config.position_features = PositionFeatureConfig {
        enable_king_safety: true,
        enable_pawn_structure: false,
        enable_mobility: false,
        enable_center_control: false,
        enable_development: false,
    };
    config.weights = EvaluationWeights {
        material_weight: 0.0,
        position_weight: 0.0,
        king_safety_weight: 1.0,
        pawn_structure_weight: 0.0,
        mobility_weight: 0.0,
        center_control_weight: 0.0,
        development_weight: 0.0,
    };

    let evaluator = IntegratedEvaluator::with_config(config.clone());
    let weighted_score = evaluator.evaluate(&board, Player::Black, &captured);
    assert!(
        weighted_score != 0,
        "Expected non-zero score when king safety is enabled with a weight of 1.0"
    );

    let mut zero_weight_config = config;
    zero_weight_config.weights.king_safety_weight = 0.0;
    let zero_weight_evaluator = IntegratedEvaluator::with_config(zero_weight_config);
    let zero_score = zero_weight_evaluator.evaluate(&board, Player::Black, &captured);

    assert_eq!(
        zero_score, 0,
        "Score should be zero when all position feature weights are disabled"
    );
    assert_ne!(
        weighted_score, zero_score,
        "Changing the king safety weight should impact the final evaluation"
    );
}
