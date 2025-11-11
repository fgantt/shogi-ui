use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::tactical_patterns::{TacticalConfig, TacticalPatternRecognizer};
use shogi_engine::types::{Piece, PieceType, Player, Position};

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
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        blocker_pos,
    );

    let mut recognizer = TacticalPatternRecognizer::with_config(forks_only_config());
    let blocked_score = recognizer.evaluate_tactics(&board, Player::Black);
    assert_eq!(
        blocked_score.mg, 0,
        "Blocked rook fork should not award a bonus"
    );

    board.remove_piece(blocker_pos);
    let mut recognizer_unblocked = TacticalPatternRecognizer::with_config(forks_only_config());
    let unblocked_score = recognizer_unblocked.evaluate_tactics(&board, Player::Black);
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
    board.place_piece(Piece::new(PieceType::Rook, Player::Black), Position::new(0, 8));

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
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        blocker,
    );

    let mut recognizer = TacticalPatternRecognizer::with_config(back_rank_only_config());
    let blocked_score = recognizer.evaluate_tactics(&board, Player::White);
    assert_eq!(
        blocked_score.mg, 0,
        "Friendly blockers should prevent back-rank threat penalties"
    );

    board.remove_piece(blocker);
    let mut recognizer_unblocked = TacticalPatternRecognizer::with_config(back_rank_only_config());
    let threatened_score = recognizer_unblocked.evaluate_tactics(&board, Player::White);
    assert!(
        threatened_score.mg < 0,
        "Clearing the file should introduce a back-rank threat penalty"
    );
}


