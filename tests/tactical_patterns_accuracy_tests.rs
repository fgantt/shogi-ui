use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::integration::{
    ComponentFlags, IntegratedEvaluationConfig, IntegratedEvaluator,
};
use shogi_engine::evaluation::tactical_patterns::{TacticalConfig, TacticalPatternRecognizer};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position};

fn forks_only_config() -> TacticalConfig {
    TacticalConfig {
        enable_forks: true,
        enable_pins: false,
        enable_skewers: false,
        enable_discovered_attacks: false,
        enable_knight_forks: false,
        enable_back_rank_threats: false,
        ..TacticalConfig::default()
    }
}

fn back_rank_only_config() -> TacticalConfig {
    TacticalConfig {
        enable_forks: false,
        enable_pins: false,
        enable_skewers: false,
        enable_discovered_attacks: false,
        enable_knight_forks: false,
        enable_back_rank_threats: true,
        ..TacticalConfig::default()
    }
}

fn pins_only_config() -> TacticalConfig {
    TacticalConfig {
        enable_forks: false,
        enable_pins: true,
        enable_skewers: false,
        enable_discovered_attacks: false,
        enable_knight_forks: false,
        enable_back_rank_threats: false,
        ..TacticalConfig::default()
    }
}

#[test]
fn forks_respect_blockers_and_line_of_sight() {
    let mut board = BitboardBoard::empty();
    let rook_pos = Position::new(4, 4);
    board.place_piece(Piece::new(PieceType::Rook, Player::Black), rook_pos);

    // Vertical target with no blockers
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(2, 4),
    );

    // Horizontal target shielded by friendly piece
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(4, 7),
    );
    let blocker_pos = Position::new(4, 6);
    board.place_piece(Piece::new(PieceType::Silver, Player::Black), blocker_pos);

    let captured = CapturedPieces::new();
    let mut recognizer = TacticalPatternRecognizer::with_config(forks_only_config());
    let blocked_score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
    assert_eq!(
        blocked_score.mg, 0,
        "Blocked rook fork should not award a bonus"
    );

    board.remove_piece(blocker_pos);
    let mut recognizer_unblocked = TacticalPatternRecognizer::with_config(forks_only_config());
    let unblocked_score = recognizer_unblocked.evaluate_tactics(&board, Player::Black, &captured);
    assert!(
        unblocked_score.mg > 0,
        "Removing the blocker should allow the fork to be scored"
    );
}

#[test]
fn back_rank_threats_require_clear_files() {
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(0, 4);
    board.place_piece(Piece::new(PieceType::King, Player::White), king_pos);
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(0, 8),
    );

    // Friendly pieces limiting the king's mobility
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(0, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(1, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(1, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(1, 5),
    );

    // Friendly blocker shielding the king along the back rank
    let blocker = Position::new(0, 6);
    board.place_piece(Piece::new(PieceType::Gold, Player::White), blocker);

    let captured = CapturedPieces::new();
    let mut recognizer = TacticalPatternRecognizer::with_config(back_rank_only_config());
    let blocked_score = recognizer.evaluate_tactics(&board, Player::White, &captured);
    assert_eq!(
        blocked_score.mg, 0,
        "Friendly blockers should prevent back-rank threat penalties"
    );

    board.remove_piece(blocker);
    let mut recognizer_unblocked = TacticalPatternRecognizer::with_config(back_rank_only_config());
    let threatened_score = recognizer_unblocked.evaluate_tactics(&board, Player::White, &captured);
    assert!(
        threatened_score.mg < 0,
        "Clearing the file should introduce a back-rank threat penalty"
    );
}

#[test]
fn pins_apply_negative_penalty() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(1, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(3, 4),
    );

    let captured = CapturedPieces::new();
    let mut recognizer = TacticalPatternRecognizer::with_config(pins_only_config());
    let score = recognizer.evaluate_tactics(&board, Player::White, &captured);
    assert!(
        score.mg < 0,
        "Pinned piece should produce a negative tactical score"
    );
}

#[test]
fn tactical_weight_scales_contribution() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(4, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(2, 4),
    );
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(4, 7),
    );

    let captured = CapturedPieces::new();

    let mut config = IntegratedEvaluationConfig::default();
    config.use_optimized_path = false;
    config.enable_eval_cache = false;
    config.enable_phase_cache = false;
    config.components = ComponentFlags {
        material: false,
        piece_square_tables: false,
        position_features: false,
        opening_principles: false,
        endgame_patterns: false,
        tactical_patterns: true,
        positional_patterns: false,
    };
    config.weights.tactical_weight = 1.0;

    let evaluator = IntegratedEvaluator::with_config(config.clone());
    let base_score = evaluator.evaluate(&board, Player::Black, &captured);
    assert!(
        base_score.abs() > 0,
        "Baseline tactical evaluation should be non-zero"
    );

    let mut scaled_config = config;
    scaled_config.weights.tactical_weight = 0.5;
    let scaled_evaluator = IntegratedEvaluator::with_config(scaled_config);
    let scaled_score = scaled_evaluator.evaluate(&board, Player::Black, &captured);

    let expected = (base_score as f32 * 0.5).round() as i32;
    assert!(
        (scaled_score - expected).abs() <= 2,
        "Scaled tactical weight should roughly halve the contribution (expected {}, got {})",
        expected,
        scaled_score
    );
}

#[test]
fn drop_rook_creates_fork_threat() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(4, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(4, 7),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Rook, Player::Black);

    let mut recognizer = TacticalPatternRecognizer::with_config(forks_only_config());
    let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
    assert!(
        score.mg > 0,
        "Dropping a rook to fork two valuable pieces should produce a positive score"
    );
}

#[test]
fn drop_rook_applies_pin_bonus() {
    let mut board = BitboardBoard::empty();
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(1, 4),
    );

    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Rook, Player::Black);

    let mut recognizer = TacticalPatternRecognizer::with_config(pins_only_config());
    let score = recognizer.evaluate_tactics(&board, Player::Black, &captured);
    assert!(
        score.mg > 0,
        "Dropping a rook to pin an opponent piece should yield a positive tactical bonus"
    );
}
