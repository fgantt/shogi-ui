use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::castle_fixtures::castle_fixtures;
use shogi_engine::evaluation::castles::CastleRecognizer;
use shogi_engine::evaluation::king_safety::KingSafetyEvaluator;
use shogi_engine::types::{Piece, PieceType, Player, Position};

/// Test that attacked castles properly integrate attack penalties
#[test]
fn test_attacked_castle_penalties() {
    let evaluator = KingSafetyEvaluator::new();
    let fixtures = castle_fixtures();

    // Find attacked and canonical castles
    let attacked = fixtures
        .iter()
        .find(|f| f.name == "mino_attacked_rook_file")
        .unwrap();
    let canonical = fixtures
        .iter()
        .find(|f| f.name == "mino_canonical_black")
        .unwrap();

    let (attacked_board, _) = (attacked.builder)(attacked.player);
    let (canonical_board, _) = (canonical.builder)(canonical.player);

    let attacked_score = evaluator.evaluate(&attacked_board, attacked.player);
    let canonical_score = evaluator.evaluate(&canonical_board, canonical.player);

    // Attacked castle should have lower (more negative) score due to attack penalties
    assert!(
        attacked_score.mg < canonical_score.mg || attacked_score.eg < canonical_score.eg,
        "Attacked castle should have lower score than canonical castle"
    );
}

/// Test that open files are detected and penalize castle evaluation
#[test]
fn test_open_file_penalty() {
    let evaluator = KingSafetyEvaluator::new();
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);

    // Set up Mino castle
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );

    // Evaluate without attack
    let score_without_attack = evaluator.evaluate(&board, Player::Black);

    // Add opponent rook on the same file as the king (open file attack)
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );

    // Evaluate with attack
    let score_with_attack = evaluator.evaluate(&board, Player::Black);

    // Score should be lower with the attack
    assert!(
        score_with_attack.mg < score_without_attack.mg
            || score_with_attack.eg < score_without_attack.eg,
        "Open file attack should reduce castle score"
    );
}

/// Test that infiltration is detected and penalized
#[test]
fn test_infiltration_penalty() {
    let recognizer = CastleRecognizer::new();
    let fixtures = castle_fixtures();

    let attacked = fixtures
        .iter()
        .find(|f| f.name == "anaguma_attacked_infiltration")
        .unwrap();
    let canonical = fixtures
        .iter()
        .find(|f| f.name == "anaguma_canonical_black")
        .unwrap();

    let (attacked_board, attacked_king) = (attacked.builder)(attacked.player);
    let (canonical_board, canonical_king) = (canonical.builder)(canonical.player);

    let attacked_eval = recognizer.evaluate_castle(&attacked_board, attacked.player, attacked_king);
    let canonical_eval =
        recognizer.evaluate_castle(&canonical_board, canonical.player, canonical_king);

    // Attacked castle should show infiltration
    assert!(
        attacked_eval.infiltration_ratio > canonical_eval.infiltration_ratio,
        "Attacked castle should have higher infiltration ratio"
    );
}

/// Test that mating nets are detected and penalized
#[test]
fn test_mating_net_detection() {
    let evaluator = KingSafetyEvaluator::new();
    let fixtures = castle_fixtures();

    let attacked = fixtures
        .iter()
        .find(|f| f.name == "yagura_attacked_mating_net")
        .unwrap();

    let (board, _) = (attacked.builder)(attacked.player);
    let score = evaluator.evaluate(&board, attacked.player);

    // Mating net should result in a very negative score
    assert!(
        score.mg < -50 || score.eg < -50,
        "Mating net should result in significant penalty"
    );
}

/// Test that castle quality affects attack evaluation
#[test]
fn test_castle_quality_affects_attack_evaluation() {
    let evaluator = KingSafetyEvaluator::new();
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);

    // Set up partial castle (low quality)
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 4),
    );
    // Missing silver and pawns

    // Add opponent attack
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );

    let partial_score = evaluator.evaluate(&board, Player::Black);

    // Set up full castle (high quality)
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );

    let full_score = evaluator.evaluate(&board, Player::Black);

    // Full castle should score better than partial even with same attack
    assert!(
        full_score.mg > partial_score.mg || full_score.eg > partial_score.eg,
        "Full castle should score better than partial castle with same attack"
    );
}

/// Test that threat evaluation integrates with castle evaluation
#[test]
fn test_threat_evaluation_integration() {
    let evaluator = KingSafetyEvaluator::new();
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);

    // Set up castle
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );

    // Evaluate without threat
    let score_without_threat = evaluator.evaluate(&board, Player::Black);

    // Add pin threat (rook pinning gold to king)
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(5, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(4, 3),
    );

    // Evaluate with threat
    let score_with_threat = evaluator.evaluate(&board, Player::Black);

    // Threat should reduce score
    assert!(
        score_with_threat.mg < score_without_threat.mg
            || score_with_threat.eg < score_without_threat.eg,
        "Threat should reduce castle evaluation score"
    );
}

/// Test that broken castles are more vulnerable to attacks
#[test]
fn test_broken_castle_vulnerability() {
    let evaluator = KingSafetyEvaluator::new();
    let fixtures = castle_fixtures();

    let broken = fixtures
        .iter()
        .find(|f| f.name == "mino_broken_breached_wall")
        .unwrap();
    let canonical = fixtures
        .iter()
        .find(|f| f.name == "mino_canonical_black")
        .unwrap();

    // Add same attack to both
    let (mut broken_board, _) = (broken.builder)(broken.player);
    let (mut canonical_board, _) = (canonical.builder)(canonical.player);

    broken_board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );
    canonical_board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );

    let broken_score = evaluator.evaluate(&broken_board, broken.player);
    let canonical_score = evaluator.evaluate(&canonical_board, canonical.player);

    // Broken castle should score worse with same attack
    assert!(
        broken_score.mg < canonical_score.mg || broken_score.eg < canonical_score.eg,
        "Broken castle should be more vulnerable to attacks"
    );
}

/// Test that castle evaluation and attack evaluation combine correctly
#[test]
fn test_castle_attack_combination() {
    let evaluator = KingSafetyEvaluator::new();
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);

    // Set up strong castle
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(7, 2),
    );

    let strong_castle_score = evaluator.evaluate(&board, Player::Black);

    // Add coordinated attack
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(4, 2),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(5, 3),
    );

    let attacked_score = evaluator.evaluate(&board, Player::Black);

    // Attack should reduce score significantly
    let score_delta = strong_castle_score.mg - attacked_score.mg;
    assert!(
        score_delta > 20,
        "Coordinated attack should significantly reduce castle score, delta: {}",
        score_delta
    );
}

/// Test that telemetry tracks castle/attack interactions
#[test]
fn test_castle_attack_telemetry() {
    let evaluator = KingSafetyEvaluator::new();
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(8, 4);

    // Set up castle with attack
    board.place_piece(Piece::new(PieceType::King, Player::Black), king_pos);
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(7, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(6, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(3, 4),
    );

    evaluator.evaluate(&board, Player::Black);
    let stats = evaluator.stats();

    // Should have tracked evaluation
    assert!(stats.evaluations > 0, "Should track evaluations");
    // Should have tracked castle or attack evaluation
    assert!(
        stats.castle_matches > 0 || stats.partial_castles > 0 || stats.bare_kings > 0,
        "Should track castle state"
    );
}
