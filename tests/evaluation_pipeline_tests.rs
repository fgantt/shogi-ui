use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::{
    integration::{ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator},
    pst_loader::PieceSquareTableConfig,
};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn pst_only_config() -> IntegratedEvaluationConfig {
    let mut config = IntegratedEvaluationConfig::default();
    config.components = ComponentFlags {
        material: false,
        piece_square_tables: true,
        position_features: false,
        opening_principles: false,
        endgame_patterns: false,
        tactical_patterns: false,
        positional_patterns: false,
    };
    config.enable_phase_cache = false;
    config.enable_eval_cache = false;
    config.use_optimized_path = false;
    config.pst = PieceSquareTableConfig::default();
    config
}

fn base_board_with_advanced_pawn() -> BitboardBoard {
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
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 4),
    );
    board
}

fn add_phase_weighting_support(board: &mut BitboardBoard) {
    // Add mirrored major and minor pieces for both players so the position
    // remains balanced while the game phase stays in the middlegame range.
    let mirrored_pairs = [
        (PieceType::Rook, Position::new(8, 0), Position::new(0, 8)),
        (PieceType::Bishop, Position::new(7, 1), Position::new(1, 7)),
        (PieceType::Gold, Position::new(8, 3), Position::new(0, 5)),
        (PieceType::Silver, Position::new(7, 3), Position::new(1, 5)),
        (PieceType::Knight, Position::new(7, 2), Position::new(1, 6)),
        (PieceType::Lance, Position::new(8, 1), Position::new(0, 7)),
        (PieceType::Pawn, Position::new(6, 3), Position::new(2, 5)),
        (PieceType::Pawn, Position::new(6, 5), Position::new(2, 3)),
    ];

    for (piece_type, black_pos, white_pos) in mirrored_pairs {
        board.place_piece(Piece::new(piece_type, Player::Black), black_pos);
        board.place_piece(Piece::new(piece_type, Player::White), white_pos);
    }
}

#[test]
fn pst_contribution_increases_as_position_reaches_endgame() {
    let config = pst_only_config();
    let evaluator = IntegratedEvaluator::with_config(config);
    let captured = CapturedPieces::new();

    let mut middlegame_board = base_board_with_advanced_pawn();
    add_phase_weighting_support(&mut middlegame_board);
    let endgame_board = base_board_with_advanced_pawn();

    let middlegame_score = evaluator.evaluate(&middlegame_board, Player::Black, &captured);
    let endgame_score = evaluator.evaluate(&endgame_board, Player::Black, &captured);

    assert!(
        middlegame_score > 0,
        "Expected positive PST contribution in middlegame position"
    );
    assert!(
        endgame_score > middlegame_score,
        "Endgame PST contribution ({endgame_score}) should exceed middlegame contribution ({middlegame_score}) due to higher endgame weights"
    );
}

#[test]
fn pst_telemetry_reports_breakdown() {
    let config = pst_only_config();
    let evaluator = IntegratedEvaluator::with_config(config);
    evaluator.enable_statistics();
    let captured = CapturedPieces::new();

    let mut board = base_board_with_advanced_pawn();
    add_phase_weighting_support(&mut board);

    let score = evaluator.evaluate(&board, Player::Black, &captured);
    assert!(
        score > 0,
        "Expected positive PST contribution for the prepared position"
    );

    let telemetry = evaluator
        .telemetry_snapshot()
        .expect("Telemetry snapshot should be available after evaluation");
    let pst = telemetry
        .pst
        .expect("PST telemetry should be recorded when PST component is enabled");

    assert!(
        pst.total_mg != 0 || pst.total_eg != 0,
        "Expected non-zero PST totals, got mg {} eg {}",
        pst.total_mg,
        pst.total_eg
    );

    let pawn_entry = pst
        .per_piece
        .iter()
        .find(|entry| entry.piece == PieceType::Pawn)
        .expect("Pawn contribution should be present in per-piece telemetry");
    assert!(
        pawn_entry.mg != 0 || pawn_entry.eg != 0,
        "Expected pawn entry to reflect contribution, got mg {} eg {}",
        pawn_entry.mg,
        pawn_entry.eg
    );

    let stats = evaluator.get_statistics();
    let pst_stats = stats.pst_statistics();
    assert_eq!(
        pst_stats.sample_count(),
        1,
        "Expected aggregated PST statistics to record a single evaluation sample"
    );
    let (avg_mg, avg_eg) = (pst_stats.average_total_mg(), pst_stats.average_total_eg());
    assert!(
        avg_mg != 0.0 || avg_eg != 0.0,
        "Expected aggregated averages to be non-zero"
    );
    let (pawn_avg_mg, pawn_avg_eg) = pst_stats.average_for_piece(PieceType::Pawn);
    assert!(
        pawn_avg_mg != 0.0 || pawn_avg_eg != 0.0,
        "Expected pawn aggregate averages to be non-zero"
    );
}
