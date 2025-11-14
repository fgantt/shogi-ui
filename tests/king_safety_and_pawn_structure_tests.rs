use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::position_features::{
    PositionFeatureConfig, PositionFeatureEvaluator,
};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn king_safety_only() -> PositionFeatureEvaluator {
    let config = PositionFeatureConfig {
        enable_king_safety: true,
        enable_pawn_structure: false,
        enable_mobility: false,
        enable_center_control: false,
        enable_development: false,
    };
    PositionFeatureEvaluator::with_config(config)
}

fn pawn_structure_only() -> PositionFeatureEvaluator {
    let config = PositionFeatureConfig {
        enable_king_safety: false,
        enable_pawn_structure: true,
        enable_mobility: false,
        enable_center_control: false,
        enable_development: false,
    };
    PositionFeatureEvaluator::with_config(config)
}

#[test]
fn promoted_defender_counts_as_gold_in_shield() {
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);

    let defender_pos = Position::new(7, 4);
    board.place_piece(
        Piece::new(PieceType::PromotedPawn, Player::Black),
        defender_pos,
    );

    let mut evaluator = king_safety_only();
    let base_score = evaluator.evaluate_king_safety(&board, Player::Black, &CapturedPieces::new());

    board.remove_piece(defender_pos);
    board.place_piece(Piece::new(PieceType::Gold, Player::Black), defender_pos);
    let mut gold_eval = king_safety_only();
    let gold_score = gold_eval.evaluate_king_safety(&board, Player::Black, &CapturedPieces::new());

    assert_eq!(
        base_score.mg, gold_score.mg,
        "Promoted defenders should contribute the same shield value as a gold"
    );
}

#[test]
fn defensive_gold_drop_from_hand_increases_king_safety() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Gold, Player::Black);

    let mut evaluator = king_safety_only();
    let with_gold = evaluator
        .evaluate_king_safety(&board, Player::Black, &captured)
        .mg;

    let mut evaluator_no_hand = king_safety_only();
    let without_gold = evaluator_no_hand
        .evaluate_king_safety(&board, Player::Black, &CapturedPieces::new())
        .mg;

    assert!(
        with_gold > without_gold,
        "Gold in hand should improve king safety via drop coverage"
    );
}

#[test]
fn enemy_pawn_in_hand_penalises_king_safety() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Pawn, Player::White);

    let mut evaluator = king_safety_only();
    let penalty_score = evaluator
        .evaluate_king_safety(&board, Player::Black, &captured)
        .mg;

    let mut baseline = king_safety_only();
    let baseline_score = baseline
        .evaluate_king_safety(&board, Player::Black, &CapturedPieces::new())
        .mg;

    assert!(
        penalty_score < baseline_score,
        "Enemy pawn in hand should decrease king safety due to drop threats"
    );
}

#[test]
fn enemy_tokin_adjacent_is_more_dangerous_than_pawn() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );

    let mut pawn_board = board.clone();
    pawn_board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(7, 4),
    );

    let mut tokin_board = board.clone();
    tokin_board.place_piece(
        Piece::new(PieceType::PromotedPawn, Player::White),
        Position::new(7, 4),
    );

    let mut evaluator_pawn = king_safety_only();
    let pawn_score = evaluator_pawn
        .evaluate_king_safety(&pawn_board, Player::Black, &CapturedPieces::new())
        .mg;

    let mut evaluator_tokin = king_safety_only();
    let tokin_score = evaluator_tokin
        .evaluate_king_safety(&tokin_board, Player::Black, &CapturedPieces::new())
        .mg;

    assert!(
        tokin_score < pawn_score,
        "Promoted pawn adjacent to the king should penalise king safety more than an unpromoted pawn"
    );
}

#[test]
fn pawn_in_hand_supports_new_chain() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Pawn, Player::Black);

    let mut evaluator = pawn_structure_only();
    let with_drop = evaluator
        .evaluate_pawn_structure(&board, Player::Black, &captured, false)
        .mg;

    let mut evaluator_no_hand = pawn_structure_only();
    let without_drop = evaluator_no_hand
        .evaluate_pawn_structure(&board, Player::Black, &CapturedPieces::new(), false)
        .mg;

    assert!(
        with_drop > without_drop,
        "Pawn in hand should create potential chains and raise the score"
    );
}

#[test]
fn gold_in_hand_can_complete_chain_gap() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(5, 5),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Gold, Player::Black);

    let mut evaluator_with_gold = pawn_structure_only();
    let with_gold = evaluator_with_gold
        .evaluate_pawn_structure(&board, Player::Black, &captured, false)
        .mg;

    let mut evaluator_no_gold = pawn_structure_only();
    let without_gold = evaluator_no_gold
        .evaluate_pawn_structure(&board, Player::Black, &CapturedPieces::new(), false)
        .mg;

    assert!(
        with_gold > without_gold,
        "Gold in hand should be counted as potential chain support around existing pawns"
    );
}

#[test]
fn illegal_double_pawns_heavily_penalised() {
    let mut board_normal = BitboardBoard::empty();
    board_normal.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 2),
    );
    board_normal.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );

    let mut board_illegal = BitboardBoard::empty();
    board_illegal.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 4),
    );
    board_illegal.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(5, 4),
    );

    let mut evaluator = pawn_structure_only();
    let normal = evaluator
        .evaluate_pawn_structure(&board_normal, Player::Black, &CapturedPieces::new(), false)
        .mg;

    let mut evaluator_illegal = pawn_structure_only();
    let illegal = evaluator_illegal
        .evaluate_pawn_structure(&board_illegal, Player::Black, &CapturedPieces::new(), false)
        .mg;

    assert!(
        illegal + 60 < normal,
        "Illegal doubled pawns should incur a substantial penalty"
    );
}

#[test]
fn advanced_pawns_score_higher_than_home_rank() {
    let mut board_far = BitboardBoard::empty();
    board_far.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(5, 4),
    );

    let mut board_advanced = BitboardBoard::empty();
    board_advanced.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4),
    );

    let mut evaluator_far = pawn_structure_only();
    let far_score = evaluator_far
        .evaluate_pawn_structure(&board_far, Player::Black, &CapturedPieces::new(), false)
        .mg;

    let mut evaluator_advanced = pawn_structure_only();
    let advanced_score = evaluator_advanced
        .evaluate_pawn_structure(
            &board_advanced,
            Player::Black,
            &CapturedPieces::new(),
            false,
        )
        .mg;

    assert!(
        advanced_score > far_score,
        "Pawns deep in enemy territory should score higher than ones near home"
    );
}

#[test]
fn passed_pawn_score_reduced_by_enemy_gold_drop() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4),
    );

    let mut evaluator = pawn_structure_only();
    let no_enemy_hand = evaluator
        .evaluate_pawn_structure(&board, Player::Black, &CapturedPieces::new(), false)
        .mg;

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Gold, Player::White);

    let mut evaluator_blocked = pawn_structure_only();
    let with_enemy_gold = evaluator_blocked
        .evaluate_pawn_structure(&board, Player::Black, &captured, false)
        .mg;

    assert!(
        with_enemy_gold < no_enemy_hand,
        "Enemy gold in hand should reduce the value of a passed pawn"
    );
}

#[test]
fn passed_pawn_score_reduced_by_enemy_knight_drop() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(3, 4),
    );

    let mut evaluator = pawn_structure_only();
    let baseline = evaluator
        .evaluate_pawn_structure(&board, Player::Black, &CapturedPieces::new(), false)
        .mg;

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Knight, Player::White);

    let mut evaluator_knight = pawn_structure_only();
    let with_knight = evaluator_knight
        .evaluate_pawn_structure(&board, Player::Black, &captured, false)
        .mg;

    assert!(
        with_knight < baseline,
        "Enemy knight in hand should threaten to drop and reduce passed pawn value"
    );
}
