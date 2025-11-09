use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::IntegratedEvaluator;
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn add_kings(board: &mut BitboardBoard) {
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
}

#[test]
fn test_integrated_phase_reflects_hand_pieces() {
    let evaluator = IntegratedEvaluator::new();
    let mut board = BitboardBoard::empty();
    add_kings(&mut board);

    evaluator.enable_statistics();

    let captured_empty = CapturedPieces::new();
    evaluator.clear_caches();
    let score_without_hand = evaluator.evaluate(&board, Player::Black, &captured_empty);
    let phase_empty = evaluator
        .get_statistics()
        .generate_report()
        .phase_stats
        .average();
    evaluator.clear_caches();

    let mut captured_with = CapturedPieces::new();
    captured_with.add_piece(PieceType::Rook, Player::Black);
    captured_with.add_piece(PieceType::Bishop, Player::Black);
    captured_with.add_piece(PieceType::Gold, Player::White);

    evaluator.reset_statistics();
    evaluator.clear_caches();
    let score_with_hand = evaluator.evaluate(&board, Player::Black, &captured_with);
    let phase_with = evaluator
        .get_statistics()
        .generate_report()
        .phase_stats
        .average();

    assert!(
        phase_with > phase_empty,
        "Adding powerful hand pieces should increase the computed game phase"
    );

    assert_ne!(
        score_without_hand, score_with_hand,
        "Integrated evaluation should respond to changes in captured piece pools"
    );

    evaluator.disable_statistics();
}

#[test]
fn test_integrated_phase_accounts_for_promoted_pieces() {
    let evaluator = IntegratedEvaluator::new();
    let mut board = BitboardBoard::empty();
    add_kings(&mut board);

    let promoted_pos = Position::new(4, 4);
    board.place_piece(
        Piece::new(PieceType::PromotedPawn, Player::Black),
        promoted_pos,
    );

    let captured = CapturedPieces::new();
    evaluator.enable_statistics();
    evaluator.clear_caches();
    let _ = evaluator.evaluate(&board, Player::Black, &captured);
    let phase_with_promoted = evaluator
        .get_statistics()
        .generate_report()
        .phase_stats
        .average();

    evaluator.clear_caches();
    evaluator.reset_statistics();
    board
        .remove_piece(promoted_pos)
        .expect("Promoted piece should exist for removal");
    board.place_piece(Piece::new(PieceType::Pawn, Player::Black), promoted_pos);

    let _ = evaluator.evaluate(&board, Player::Black, &captured);
    let phase_with_pawn = evaluator
        .get_statistics()
        .generate_report()
        .phase_stats
        .average();

    assert!(
        phase_with_promoted > phase_with_pawn,
        "Promoted piece should contribute more to the phase than its base counterpart"
    );

    evaluator.disable_statistics();
}
