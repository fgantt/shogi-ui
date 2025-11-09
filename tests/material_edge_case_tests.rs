use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::material::MaterialEvaluator;
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn place_kings_only() -> BitboardBoard {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board
}

#[test]
fn large_hand_inventory_matches_manual_sum() {
    let mut evaluator = MaterialEvaluator::new();
    let board = place_kings_only();

    let mut captured = CapturedPieces::new();
    for _ in 0..9 {
        captured.add_piece(PieceType::Pawn, Player::Black);
    }
    captured.add_piece(PieceType::Rook, Player::Black);
    captured.add_piece(PieceType::Bishop, Player::Black);
    captured.add_piece(PieceType::Gold, Player::Black);
    captured.add_piece(PieceType::Silver, Player::Black);

    captured.add_piece(PieceType::Knight, Player::White);
    captured.add_piece(PieceType::Pawn, Player::White);

    let score = evaluator.evaluate_material(&board, Player::Black, &captured);

    let mut expected = shogi_engine::types::TaperedScore::default();
    for piece in &captured.black {
        expected += evaluator.get_hand_piece_value(*piece);
    }
    for piece in &captured.white {
        expected -= evaluator.get_hand_piece_value(*piece);
    }

    assert_eq!(score, expected);
}

#[test]
fn promoted_capture_converts_to_demoted_hand_value() {
    let mut board_with_piece = place_kings_only();
    let promoted_pos = Position::new(4, 4);
    board_with_piece.place_piece(
        Piece::new(PieceType::PromotedBishop, Player::White),
        promoted_pos,
    );

    let mut evaluator = MaterialEvaluator::new();
    let baseline_score =
        evaluator.evaluate_material(&board_with_piece, Player::Black, &CapturedPieces::new());

    let board_after_capture = place_kings_only();
    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Bishop, Player::Black);
    let post_capture_score =
        evaluator.evaluate_material(&board_after_capture, Player::Black, &captured);

    let promoted_value = evaluator.get_piece_value(PieceType::PromotedBishop);
    let demoted_hand_value = evaluator.get_hand_piece_value(PieceType::Bishop);

    assert_eq!(
        post_capture_score.mg - baseline_score.mg,
        promoted_value.mg + demoted_hand_value.mg
    );
    assert_eq!(
        post_capture_score.eg - baseline_score.eg,
        promoted_value.eg + demoted_hand_value.eg
    );
}

#[test]
fn impasse_thresholds_respect_24_point_rule() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(1, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(7, 4),
    );

    // Black pieces
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(2, 2),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::Black),
        Position::new(2, 6),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(1, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(1, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Knight, Player::Black),
        Position::new(1, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Lance, Player::Black),
        Position::new(1, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4),
    );

    // White pieces
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(6, 6),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(6, 2),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(7, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(7, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Knight, Player::White),
        Position::new(7, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Lance, Player::White),
        Position::new(7, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(6, 4),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Rook, Player::Black);
    captured.add_piece(PieceType::Bishop, Player::Black);
    captured.add_piece(PieceType::Rook, Player::White);
    captured.add_piece(PieceType::Bishop, Player::White);

    let result = board
        .check_impasse_result(&captured)
        .expect("impasse expected");
    assert!(result.black_points >= 24);
    assert!(result.white_points >= 24);
    assert_eq!(result.outcome, shogi_engine::types::ImpasseOutcome::Draw);

    // Remove a bishop from White to drop below threshold.
    board.remove_piece(Position::new(6, 2));
    let downgraded = board
        .check_impasse_result(&captured)
        .expect("still impasse");
    assert!(downgraded.white_points < 24);
    assert_eq!(
        downgraded.outcome,
        shogi_engine::types::ImpasseOutcome::BlackWins
    );
}

#[test]
fn knights_and_lances_in_hand_are_valued_above_board_counterparts() {
    let evaluator = MaterialEvaluator::new();
    let knight_board = evaluator.get_piece_value(PieceType::Knight);
    let knight_hand = evaluator.get_hand_piece_value(PieceType::Knight);
    assert!(knight_hand.mg >= knight_board.mg);
    assert!(knight_hand.eg >= knight_board.eg);

    let lance_board = evaluator.get_piece_value(PieceType::Lance);
    let lance_hand = evaluator.get_hand_piece_value(PieceType::Lance);
    assert!(lance_hand.mg >= lance_board.mg);
    assert!(lance_hand.eg >= lance_board.eg);
}

#[test]
fn statistics_reset_between_repeated_positions() {
    let mut evaluator = MaterialEvaluator::new();
    let mut board = place_kings_only();
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(4, 4),
    );
    evaluator.evaluate_material(&board, Player::Black, &CapturedPieces::new());

    assert_eq!(evaluator.stats().evaluations, 1);
    let rook_contrib = evaluator.stats().board_contribution(PieceType::Rook);
    assert!(rook_contrib.mg > 0);

    evaluator.reset_stats();
    assert_eq!(evaluator.stats().evaluations, 0);

    let mut board_second = place_kings_only();
    board_second.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(4, 4),
    );
    evaluator.evaluate_material(&board_second, Player::Black, &CapturedPieces::new());

    assert_eq!(evaluator.stats().evaluations, 1);
    let pawn_contrib = evaluator.stats().board_contribution(PieceType::Pawn);
    assert_eq!(
        pawn_contrib.mg,
        evaluator.get_piece_value(PieceType::Pawn).mg as i64
    );
    assert_eq!(evaluator.stats().board_contribution(PieceType::Rook).mg, 0);
}
