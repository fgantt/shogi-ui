use crate::types::*;
use crate::evaluation::castles::*;

/// Yagura castle pattern definition
pub fn get_yagura_castle() -> CastlePattern {
    CastlePattern {
        name: "Yagura",
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
                piece_type: PieceType::Lance,
                relative_pos: (0, 3),
                required: false,
                weight: 5,
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
        ],
        score: TaperedScore::new_tapered(160, 80),
        flexibility: 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yagura_castle_pattern() {
        let yagura_castle = get_yagura_castle();
        assert_eq!(yagura_castle.name, "Yagura");
        assert_eq!(yagura_castle.pieces.len(), 5);
        assert_eq!(yagura_castle.flexibility, 2);
        assert_eq!(yagura_castle.score.mg, 160);
        assert_eq!(yagura_castle.score.eg, 80);
    }

    #[test]
    fn test_yagura_castle_required_pieces() {
        let yagura_castle = get_yagura_castle();
        let required_pieces: Vec<&CastlePiece> = yagura_castle.pieces.iter().filter(|p| p.required).collect();
        assert_eq!(required_pieces.len(), 2); // Gold and Silver should be required
        
        let gold_piece = required_pieces.iter().find(|p| p.piece_type == PieceType::Gold);
        let silver_piece = required_pieces.iter().find(|p| p.piece_type == PieceType::Silver);
        
        assert!(gold_piece.is_some());
        assert!(silver_piece.is_some());
        
        assert_eq!(gold_piece.unwrap().weight, 10);
        assert_eq!(silver_piece.unwrap().weight, 9);
    }

    #[test]
    fn test_yagura_castle_optional_pieces() {
        let yagura_castle = get_yagura_castle();
        let optional_pieces: Vec<&CastlePiece> = yagura_castle.pieces.iter().filter(|p| !p.required).collect();
        assert_eq!(optional_pieces.len(), 3); // Lance and two pawns should be optional
        
        let lance_piece = optional_pieces.iter().find(|p| p.piece_type == PieceType::Lance);
        let pawn_pieces: Vec<&CastlePiece> = optional_pieces.iter().filter(|p| p.piece_type == PieceType::Pawn).cloned().collect();
        
        assert!(lance_piece.is_some());
        assert_eq!(pawn_pieces.len(), 2);
        
        assert_eq!(lance_piece.unwrap().weight, 5);
        for pawn in pawn_pieces {
            assert_eq!(pawn.weight, 6);
        }
    }
}
