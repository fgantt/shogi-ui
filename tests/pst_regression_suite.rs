use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::piece_square_tables::PieceSquareTables;
use shogi_engine::evaluation::pst_loader::{PieceSquareTableConfig, PieceSquareTableLoader};
use shogi_engine::types::{Piece, PieceType, Player, Position, TaperedScore};

#[test]
fn default_loader_matches_builtin_tables() {
    let builtin = PieceSquareTables::new();
    let loader = PieceSquareTableLoader::load(&PieceSquareTableConfig::default())
        .expect("default preset should load");

    for (idx, board) in sample_boards().into_iter().enumerate() {
        let builtin_score_black = aggregate_pst(&builtin, &board, Player::Black);
        let loader_score_black = aggregate_pst(&loader, &board, Player::Black);
        assert_eq!(
            builtin_score_black, loader_score_black,
            "Black perspective mismatch on board {}",
            idx
        );

        let builtin_score_white = aggregate_pst(&builtin, &board, Player::White);
        let loader_score_white = aggregate_pst(&loader, &board, Player::White);
        assert_eq!(
            builtin_score_white, loader_score_white,
            "White perspective mismatch on board {}",
            idx
        );
    }
}

fn aggregate_pst(
    tables: &PieceSquareTables,
    board: &BitboardBoard,
    player: Player,
) -> TaperedScore {
    let mut score = TaperedScore::default();

    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                let value = tables.get_value(piece.piece_type, pos, piece.player);
                if piece.player == player {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
    }

    score
}

fn sample_boards() -> Vec<BitboardBoard> {
    vec![
        opening_structure(),
        contested_middlegame(),
        simplified_endgame(),
    ]
}

fn opening_structure() -> BitboardBoard {
    let mut board = BitboardBoard::empty();
    // Black side pieces
    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(7, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::Black),
        Position::new(7, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::Black),
        Position::new(8, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::Black),
        Position::new(8, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(6, 4),
    );

    // White side pieces
    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(0, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(1, 1),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::White),
        Position::new(1, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(0, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(0, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(2, 4),
    );

    board
}

fn contested_middlegame() -> BitboardBoard {
    let mut board = BitboardBoard::empty();

    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(8, 2),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::Black),
        Position::new(4, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Bishop, Player::Black),
        Position::new(5, 3),
    );
    board.place_piece(
        Piece::new(PieceType::PromotedPawn, Player::Black),
        Position::new(3, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Knight, Player::Black),
        Position::new(6, 2),
    );

    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(1, 6),
    );
    board.place_piece(
        Piece::new(PieceType::Rook, Player::White),
        Position::new(5, 4),
    );
    board.place_piece(
        Piece::new(PieceType::PromotedSilver, Player::White),
        Position::new(2, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(2, 7),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(3, 6),
    );

    board
}

fn simplified_endgame() -> BitboardBoard {
    let mut board = BitboardBoard::empty();

    board.place_piece(
        Piece::new(PieceType::King, Player::Black),
        Position::new(7, 4),
    );
    board.place_piece(
        Piece::new(PieceType::PromotedRook, Player::Black),
        Position::new(4, 3),
    );
    board.place_piece(
        Piece::new(PieceType::PromotedBishop, Player::Black),
        Position::new(5, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::Black),
        Position::new(2, 4),
    );

    board.place_piece(
        Piece::new(PieceType::King, Player::White),
        Position::new(1, 4),
    );
    board.place_piece(
        Piece::new(PieceType::Gold, Player::White),
        Position::new(2, 3),
    );
    board.place_piece(
        Piece::new(PieceType::Silver, Player::White),
        Position::new(3, 5),
    );
    board.place_piece(
        Piece::new(PieceType::Pawn, Player::White),
        Position::new(6, 4),
    );

    board
}
