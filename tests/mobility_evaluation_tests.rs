use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::position_features::PositionFeatureEvaluator;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn base_board() -> BitboardBoard {
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
fn mobility_includes_drop_opportunities() {
    let board = base_board();

    let mut drop_captured = CapturedPieces::new();
    drop_captured.add_piece(PieceType::Rook, Player::Black);
    drop_captured.add_piece(PieceType::Pawn, Player::Black);

    let empty_captured = CapturedPieces::new();

    let mut baseline_evaluator = PositionFeatureEvaluator::new();
    let baseline = baseline_evaluator.evaluate_mobility(&board, Player::Black, &empty_captured);

    let mut drop_evaluator = PositionFeatureEvaluator::new();
    let with_drops = drop_evaluator.evaluate_mobility(&board, Player::Black, &drop_captured);

    assert!(
        with_drops.mg > baseline.mg,
        "Expected drop opportunities to increase middlegame mobility (baseline {}, with drops {})",
        baseline.mg,
        with_drops.mg
    );
    assert!(
        with_drops.eg > baseline.eg,
        "Expected drop opportunities to increase endgame mobility (baseline {}, with drops {})",
        baseline.eg,
        with_drops.eg
    );
}

#[test]
fn mobility_attack_moves_award_bonus() {
    let rook_pos = Position::new(4, 4);
    let capture_pos = Position::new(4, 7);

    let mut board_without_attack = base_board();
    board_without_attack.place_piece(Piece::new(PieceType::Rook, Player::Black), rook_pos);
    let mut board_with_attack = board_without_attack.clone();
    board_with_attack.place_piece(Piece::new(PieceType::Silver, Player::White), capture_pos);

    let captured = CapturedPieces::new();
    let mut attack_evaluator = PositionFeatureEvaluator::new();
    let attack_score =
        attack_evaluator.evaluate_mobility(&board_with_attack, Player::Black, &captured);

    let mut non_attack_evaluator = PositionFeatureEvaluator::new();
    let non_attack_score =
        non_attack_evaluator.evaluate_mobility(&board_without_attack, Player::Black, &captured);

    let (attack_moves, attack_central, attack_captures) =
        collect_piece_mobility_stats(&board_with_attack, Player::Black, &captured, rook_pos);
    let (non_attack_moves, non_attack_central, non_attack_captures) =
        collect_piece_mobility_stats(&board_without_attack, Player::Black, &captured, rook_pos);

    let expected_mg_delta = evaluate_rook_mobility(attack_moves, attack_central, attack_captures)
        - evaluate_rook_mobility(non_attack_moves, non_attack_central, non_attack_captures);
    let actual_mg_delta = attack_score.mg - non_attack_score.mg;

    assert!(
        attack_captures > non_attack_captures,
        "Expected capture count to increase when an enemy piece blocks the rook (non attack {}, attack {})",
        non_attack_captures,
        attack_captures
    );

    assert_eq!(
        actual_mg_delta, expected_mg_delta,
        "Mobility delta should match expected rook contribution (expected {}, actual {})",
        expected_mg_delta, actual_mg_delta
    );
}

fn collect_piece_mobility_stats(
    board: &BitboardBoard,
    player: Player,
    captured: &CapturedPieces,
    pos: Position,
) -> (i32, i32, i32) {
    let generator = MoveGenerator::new();
    let moves = generator.generate_legal_moves(board, player, captured);
    let mut total = 0;
    let mut central = 0;
    let mut captures = 0;

    for mv in moves.iter().filter(|m| m.from == Some(pos)) {
        total += 1;
        if is_central_square(mv.to) {
            central += 1;
        }
        if mv.is_capture {
            captures += 1;
        }
    }

    (total, central, captures)
}

fn evaluate_rook_mobility(moves: i32, central: i32, captures: i32) -> i32 {
    // Mirror PositionFeatureEvaluator weights for rooks (mg side only).
    let mut mg = moves * 5;
    if moves <= 2 {
        mg -= 18;
    }
    if central > 0 {
        mg += central * 4;
    }
    if captures > 0 {
        mg += captures * 4;
    }
    mg
}

fn is_central_square(pos: Position) -> bool {
    pos.row >= 3 && pos.row <= 5 && pos.col >= 3 && pos.col <= 5
}

#[test]
fn mobility_handles_promoted_pieces() {
    let mut promoted_board = base_board();
    let promoted_pos = Position::new(4, 4);
    promoted_board.place_piece(
        Piece::new(PieceType::PromotedPawn, Player::Black),
        promoted_pos,
    );

    let captured = CapturedPieces::new();
    let mut promoted_evaluator = PositionFeatureEvaluator::new();
    let promoted_score =
        promoted_evaluator.evaluate_mobility(&promoted_board, Player::Black, &captured);

    let mut pawn_board = promoted_board.clone();
    pawn_board.remove_piece(promoted_pos);
    pawn_board.place_piece(Piece::new(PieceType::Pawn, Player::Black), promoted_pos);

    let mut pawn_evaluator = PositionFeatureEvaluator::new();
    let pawn_score = pawn_evaluator.evaluate_mobility(&pawn_board, Player::Black, &captured);

    assert!(
        promoted_score.mg >= pawn_score.mg,
        "Promoted pawn mobility should be at least as high as pawn mobility (pawn {}, promoted {})",
        pawn_score.mg,
        promoted_score.mg
    );
    assert!(
        promoted_score.eg >= pawn_score.eg,
        "Promoted pawn endgame mobility should be at least as high as pawn mobility (pawn {}, promoted {})",
        pawn_score.eg,
        promoted_score.eg
    );
}
