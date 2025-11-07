use crate::evaluation::castles::*;
use crate::types::*;

/// Mino castle pattern definition
pub fn get_mino_castle() -> CastlePattern {
    CastlePattern {
        name: "Mino",
        pieces: vec![
            CastlePiece {
                piece_type: PieceType::Gold,
                relative_pos: (-1, -1),
                required: true,
                weight: 10,
            },
            CastlePiece {
                piece_type: PieceType::Silver,
                relative_pos: (-2, -1),
                required: true,
                weight: 9,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-2, -2),
                required: false,
                weight: 6,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-1, -2),
                required: false,
                weight: 6,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (0, -2),
                required: false,
                weight: 6,
            },
        ],
        score: TaperedScore::new_tapered(180, 60),
        flexibility: 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mino_castle_pattern() {
        let mino_castle = get_mino_castle();
        assert_eq!(mino_castle.name, "Mino");
        assert_eq!(mino_castle.pieces.len(), 5);
        assert_eq!(mino_castle.flexibility, 2);
        assert_eq!(mino_castle.score.mg, 180);
        assert_eq!(mino_castle.score.eg, 60);
    }

    #[test]
    fn test_mino_castle_required_pieces() {
        let mino_castle = get_mino_castle();
        let required_pieces: Vec<&CastlePiece> =
            mino_castle.pieces.iter().filter(|p| p.required).collect();
        assert_eq!(required_pieces.len(), 2); // Gold and Silver should be required

        let gold_piece = required_pieces
            .iter()
            .find(|p| p.piece_type == PieceType::Gold);
        let silver_piece = required_pieces
            .iter()
            .find(|p| p.piece_type == PieceType::Silver);

        assert!(gold_piece.is_some());
        assert!(silver_piece.is_some());

        assert_eq!(gold_piece.unwrap().weight, 10);
        assert_eq!(silver_piece.unwrap().weight, 9);
    }

    #[test]
    fn test_mino_castle_optional_pieces() {
        let mino_castle = get_mino_castle();
        let optional_pieces: Vec<&CastlePiece> =
            mino_castle.pieces.iter().filter(|p| !p.required).collect();
        assert_eq!(optional_pieces.len(), 3); // Three pawns should be optional

        for piece in optional_pieces {
            assert_eq!(piece.piece_type, PieceType::Pawn);
            assert_eq!(piece.weight, 6);
        }
    }
}
