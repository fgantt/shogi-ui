use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::position_features::PositionFeatureEvaluator;
use shogi_engine::types::{Piece, PieceType, Player, Position};

#[test]
fn center_control_prefers_attack_map_over_empty_center() {
    let mut attack_board = BitboardBoard::empty();
    attack_board.place_piece(
        Piece::new(PieceType::Bishop, Player::Black),
        Position::new(6, 2),
    );

    let mut passive_board = BitboardBoard::empty();
    passive_board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(4, 4),
    );

    let mut attack_eval = PositionFeatureEvaluator::new();
    let attack_score = attack_eval.evaluate_center_control(&attack_board, Player::Black, false);

    let mut passive_eval = PositionFeatureEvaluator::new();
    let passive_score = passive_eval.evaluate_center_control(&passive_board, Player::Black, false);

    assert!(
        attack_score.mg > passive_score.mg,
        "Squares controlled by active pieces should outweigh passive occupation"
    );
}

#[test]
fn center_control_rewards_edge_anchor_pressure() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(6, 0),
    );

    let mut evaluator = PositionFeatureEvaluator::new();
    let score = evaluator.evaluate_center_control(&board, Player::Black, false);

    assert!(
        score.mg > 0,
        "Edge pressure should provide a positive contribution"
    );
}

#[test]
fn development_penalises_stuck_knights() {
    let mut stuck_board = BitboardBoard::empty();
    stuck_board.place_piece(
        Piece::new(PieceType::Knight, Player::Black),
        Position::new(8, 1),
    );

    let mut developed_board = BitboardBoard::empty();
    developed_board.place_piece(
        Piece::new(PieceType::Knight, Player::Black),
        Position::new(6, 2),
    );

    let mut evaluator = PositionFeatureEvaluator::new();
    let stuck_score = evaluator.evaluate_development(&stuck_board, Player::Black, false);

    let mut evaluator_developed = PositionFeatureEvaluator::new();
    let developed_score =
        evaluator_developed.evaluate_development(&developed_board, Player::Black, false);

    assert!(
        developed_score.mg > stuck_score.mg,
        "Developed knights should score higher than those on their home rank"
    );
}

#[test]
fn development_penalises_retreating_promoted_pieces() {
    let mut advanced_board = BitboardBoard::empty();
    advanced_board.place_piece(
        Piece::new(PieceType::PromotedSilver, Player::Black),
        Position::new(4, 3),
    );

    let mut retreat_board = BitboardBoard::empty();
    retreat_board.place_piece(
        Piece::new(PieceType::PromotedSilver, Player::Black),
        Position::new(7, 3),
    );

    let mut advanced_eval = PositionFeatureEvaluator::new();
    let advanced_score = advanced_eval.evaluate_development(&advanced_board, Player::Black, false);

    let mut retreat_eval = PositionFeatureEvaluator::new();
    let retreat_score = retreat_eval.evaluate_development(&retreat_board, Player::Black, false);

    assert!(
        advanced_score.mg > retreat_score.mg,
        "Promoted defenders that retreat should lose development value"
    );
}
