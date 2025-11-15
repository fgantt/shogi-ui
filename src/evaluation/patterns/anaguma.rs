use crate::evaluation::castle_geometry::{
    CastlePieceClass, CastlePieceDescriptor, CastlePieceRole, RelativeOffset,
};
use crate::evaluation::castles::{
    mirror_descriptors, CastlePattern, CastleVariant, GOLD_FAMILY, PAWN_WALL_FAMILY, SILVER_FAMILY,
};
use crate::types::evaluation::TaperedScore;

fn base_shell() -> Vec<CastlePieceDescriptor> {
    vec![
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(GOLD_FAMILY),
            RelativeOffset::new(-1, 0),
            true,
            10,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(SILVER_FAMILY),
            RelativeOffset::new(-2, 0),
            true,
            9,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, -1),
            false,
            7,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, 1),
            false,
            7,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-1, -1),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-1, 1),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
    ]
}

fn advanced_silver_shell() -> Vec<CastlePieceDescriptor> {
    vec![
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(GOLD_FAMILY),
            RelativeOffset::new(-1, 0),
            true,
            10,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(SILVER_FAMILY),
            RelativeOffset::new(-1, -1),
            true,
            9,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, -1),
            false,
            7,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, 0),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, 1),
            false,
            7,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-1, 1),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
    ]
}

pub fn get_anaguma_castle() -> CastlePattern {
    let base = base_shell();
    let silver_forward = advanced_silver_shell();

    let mut variants = Vec::new();
    variants.push(CastleVariant::from_descriptors("right-base", &base));
    variants.push(CastleVariant::from_descriptors(
        "left-base",
        &mirror_descriptors(&base),
    ));
    variants.push(CastleVariant::from_descriptors(
        "right-silver-forward",
        &silver_forward,
    ));
    variants.push(CastleVariant::from_descriptors(
        "left-silver-forward",
        &mirror_descriptors(&silver_forward),
    ));

    CastlePattern {
        name: "Anaguma",
        variants,
        score: TaperedScore::new_tapered(220, 40),
        flexibility: 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anaguma_castle_pattern_variants() {
        let pattern = get_anaguma_castle();
        assert_eq!(pattern.name, "Anaguma");
        assert_eq!(pattern.variants.len(), 4);

        for variant in &pattern.variants {
            let required = variant.pieces.iter().filter(|piece| piece.required).count();
            assert!(required >= 2);
        }
    }

    #[test]
    fn test_anaguma_mirror_offsets() {
        let base = base_shell();
        let mirrored = mirror_descriptors(&base);
        for (original, mirrored_piece) in base.iter().zip(mirrored.iter()) {
            assert_eq!(original.offset.rank, mirrored_piece.offset.rank);
            assert_eq!(original.offset.file, -mirrored_piece.offset.file);
        }
    }
}
