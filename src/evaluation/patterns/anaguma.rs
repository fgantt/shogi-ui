use crate::evaluation::castles::*;
use crate::types::*;

/// Anaguma castle pattern definition
pub fn get_anaguma_castle() -> CastlePattern {
    CastlePattern {
        name: "Anaguma",
        pieces: vec![
            CastlePiece {
                piece_type: PieceType::Gold,
                relative_pos: (-1, 0),
                required: true,
                weight: 10,
            },
            CastlePiece {
                piece_type: PieceType::Silver,
                relative_pos: (-2, 0),
                required: true,
                weight: 9,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-2, -1),
                required: false,
                weight: 7,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-2, 1),
                required: false,
                weight: 7,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-1, -1),
                required: false,
                weight: 6,
            },
            CastlePiece {
                piece_type: PieceType::Pawn,
                relative_pos: (-1, 1),
                required: false,
                weight: 6,
            },
        ],
        score: TaperedScore::new_tapered(220, 40),
        flexibility: 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anaguma_castle_pattern() {
        let anaguma_castle = get_anaguma_castle();
        assert_eq!(anaguma_castle.name, "Anaguma");
        assert_eq!(anaguma_castle.pieces.len(), 6);
        assert_eq!(anaguma_castle.flexibility, 3);
        assert_eq!(anaguma_castle.score.mg, 220);
        assert_eq!(anaguma_castle.score.eg, 40);
    }

    #[test]
    fn test_anaguma_castle_required_pieces() {
        let anaguma_castle = get_anaguma_castle();
        let required_pieces: Vec<&CastlePiece> = anaguma_castle
            .pieces
            .iter()
            .filter(|p| p.required)
            .collect();
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
    fn test_anaguma_castle_optional_pieces() {
        let anaguma_castle = get_anaguma_castle();
        let optional_pieces: Vec<&CastlePiece> = anaguma_castle
            .pieces
            .iter()
            .filter(|p| !p.required)
            .collect();
        assert_eq!(optional_pieces.len(), 4); // Four pawns should be optional

        for piece in optional_pieces {
            assert_eq!(piece.piece_type, PieceType::Pawn);
            assert!(piece.weight >= 6 && piece.weight <= 7);
        }
    }
}
