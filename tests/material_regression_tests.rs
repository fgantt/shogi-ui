use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::material::{MaterialEvaluationConfig, MaterialEvaluator};
use shogi_engine::types::{CapturedPieces, Piece, PieceType, Player, Position, TaperedScore};

struct RegressionCase {
    name: &'static str,
    setup: fn(&MaterialEvaluator) -> (BitboardBoard, CapturedPieces, Player, TaperedScore),
}

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

fn regression_cases() -> Vec<RegressionCase> {
    vec![
        RegressionCase {
            name: "start_position_is_balanced",
            setup: |_| {
                let board = BitboardBoard::new();
                (
                    board,
                    CapturedPieces::new(),
                    Player::Black,
                    TaperedScore::default(),
                )
            },
        },
        RegressionCase {
            name: "board_material_advantage",
            setup: |evaluator| {
                let mut board = base_board();
                board.place_piece(
                    Piece::new(PieceType::Rook, Player::Black),
                    Position::new(4, 4),
                );
                board.place_piece(
                    Piece::new(PieceType::Silver, Player::White),
                    Position::new(3, 3),
                );

                let rook = evaluator.get_piece_value(PieceType::Rook);
                let silver = evaluator.get_piece_value(PieceType::Silver);
                (
                    board,
                    CapturedPieces::new(),
                    Player::Black,
                    TaperedScore::new_tapered(rook.mg - silver.mg, rook.eg - silver.eg),
                )
            },
        },
        RegressionCase {
            name: "hand_material_advantage",
            setup: |evaluator| {
                let board = base_board();
                let mut captured = CapturedPieces::new();
                captured.add_piece(PieceType::Bishop, Player::Black);
                captured.add_piece(PieceType::Pawn, Player::Black);
                captured.add_piece(PieceType::Pawn, Player::Black);
                captured.add_piece(PieceType::Gold, Player::White);

                let bishop = evaluator.get_hand_piece_value(PieceType::Bishop);
                let pawn = evaluator.get_hand_piece_value(PieceType::Pawn);
                let gold = evaluator.get_hand_piece_value(PieceType::Gold);
                (
                    board,
                    captured,
                    Player::Black,
                    TaperedScore::new_tapered(
                        bishop.mg + 2 * pawn.mg - gold.mg,
                        bishop.eg + 2 * pawn.eg - gold.eg,
                    ),
                )
            },
        },
        RegressionCase {
            name: "hand_disabled_relies_on_board_only",
            setup: |evaluator| {
                let mut board = base_board();
                board.place_piece(
                    Piece::new(PieceType::Gold, Player::Black),
                    Position::new(5, 4),
                );
                board.place_piece(
                    Piece::new(PieceType::Pawn, Player::White),
                    Position::new(2, 4),
                );

                let gold = evaluator.get_piece_value(PieceType::Gold);
                let pawn = evaluator.get_piece_value(PieceType::Pawn);
                (
                    board,
                    CapturedPieces::new(),
                    Player::Black,
                    TaperedScore::new_tapered(gold.mg - pawn.mg, gold.eg - pawn.eg),
                )
            },
        },
    ]
}

#[test]
fn material_regression_suite_matches_expected_scores() {
    let mut evaluator = MaterialEvaluator::new();
    for case in regression_cases() {
        let (board, captured, player, expected) = (case.setup)(&evaluator);
        let score = evaluator.evaluate_material(&board, player, &captured);
        assert_eq!(score, expected, "case {}", case.name);
    }
}

#[test]
fn material_regression_respects_disabled_hand_pieces() {
    let mut config = MaterialEvaluationConfig::default();
    config.include_hand_pieces = false;
    let mut evaluator = MaterialEvaluator::with_config(config);

    let mut board = base_board();
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(5, 5),
    );
    let mut captured = CapturedPieces::new();
    captured.add_piece(PieceType::Rook, Player::Black);
    captured.add_piece(PieceType::Pawn, Player::Black);

    let score = evaluator.evaluate_material(&board, Player::Black, &captured);
    let silver = evaluator.get_piece_value(PieceType::Silver);
    assert_eq!(score.mg, silver.mg);
    assert_eq!(score.eg, silver.eg);
}
