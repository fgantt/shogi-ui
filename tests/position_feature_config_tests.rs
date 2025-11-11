use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::{
    config::EvaluationWeights,
    integration::{ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator},
    position_features::{PositionFeatureConfig, PositionFeatureEvaluator},
};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position, TaperedScore};

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
        evaluator.evaluate_king_safety(&board, Player::Black, &captured),
        TaperedScore::default()
    );
    assert_eq!(evaluator.stats().king_safety_evals, 0);

    assert_eq!(
        evaluator.evaluate_pawn_structure(&board, Player::Black, &captured),
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
        tactical_weight: 0.0,
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

#[test]
fn telemetry_includes_position_feature_stats_when_enabled() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::all_disabled();
    config.components.position_features = true;
    config.collect_position_feature_stats = true;
    config.enable_phase_cache = false;
    config.enable_eval_cache = false;

    let evaluator = IntegratedEvaluator::with_config(config);
    evaluator.enable_statistics();
    evaluator.evaluate(&board, Player::Black, &captured);

    let telemetry = evaluator
        .telemetry_snapshot()
        .expect("Expected telemetry snapshot");
    let stats = telemetry
        .position_features
        .expect("Expected position feature statistics");

    assert!(
        stats.king_safety_evals > 0 || stats.pawn_structure_evals > 0,
        "Expected at least one position feature evaluation to be recorded"
    );
}

#[test]
fn telemetry_omits_position_feature_stats_when_disabled() {
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags::all_disabled();
    config.components.position_features = true;
    config.collect_position_feature_stats = false;
    config.enable_phase_cache = false;
    config.enable_eval_cache = false;

    let evaluator = IntegratedEvaluator::with_config(config);
    evaluator.enable_statistics();
    evaluator.evaluate(&board, Player::Black, &captured);

    let telemetry = evaluator
        .telemetry_snapshot()
        .expect("Expected telemetry snapshot");
    assert!(
        telemetry.position_features.is_none(),
        "Position feature statistics should be omitted when collection is disabled"
    );
}

fn midgame_example_board() -> BitboardBoard {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(7, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(1, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(4, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(3, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(5, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(2, 4),
    );
    board
}

#[test]
fn combined_position_features_midgame_snapshot_responds_to_toggles() {
    let board = midgame_example_board();
    let captured = CapturedPieces::new();

    let mut base_config = IntegratedEvaluationConfig::default();
    base_config.components = ComponentFlags::all_disabled();
    base_config.components.position_features = true;
    base_config.position_features = PositionFeatureConfig {
        enable_king_safety: true,
        enable_pawn_structure: true,
        enable_mobility: true,
        enable_center_control: true,
        enable_development: true,
    };
    base_config.enable_phase_cache = false;
    base_config.enable_eval_cache = false;

    let baseline_eval = IntegratedEvaluator::with_config(base_config.clone());
    let baseline_score = baseline_eval.evaluate(&board, Player::Black, &captured);

    let mut mobility_disabled = base_config.clone();
    mobility_disabled.position_features.enable_mobility = false;
    let mobility_eval = IntegratedEvaluator::with_config(mobility_disabled);
    let mobility_score = mobility_eval.evaluate(&board, Player::Black, &captured);

    assert_ne!(
        baseline_score, mobility_score,
        "Disabling mobility should impact the combined position feature evaluation"
    );

    let mut pawn_disabled = base_config;
    pawn_disabled.position_features.enable_pawn_structure = false;
    let pawn_eval = IntegratedEvaluator::with_config(pawn_disabled);
    let pawn_score = pawn_eval.evaluate(&board, Player::Black, &captured);

    assert_ne!(
        baseline_score, pawn_score,
        "Disabling pawn structure should impact the combined position feature evaluation"
    );
}
