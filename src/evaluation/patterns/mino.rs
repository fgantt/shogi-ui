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
            RelativeOffset::new(-1, -1),
            true,
            10,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(SILVER_FAMILY),
            RelativeOffset::new(-2, -1),
            true,
            9,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, -2),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-1, -2),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(0, -2),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
    ]
}

fn high_mino_shell() -> Vec<CastlePieceDescriptor> {
    vec![
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(GOLD_FAMILY),
            RelativeOffset::new(0, -1),
            true,
            10,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(SILVER_FAMILY),
            RelativeOffset::new(-1, -2),
            true,
            9,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, -2),
            false,
            6,
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
            RelativeOffset::new(0, -2),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
    ]
}

pub fn get_mino_castle() -> CastlePattern {
    let base = base_shell();
    let high = high_mino_shell();

    let mut variants = Vec::new();
    variants.push(CastleVariant::from_descriptors("right-base", &base));
    variants.push(CastleVariant::from_descriptors(
        "left-base",
        &mirror_descriptors(&base),
    ));
    variants.push(CastleVariant::from_descriptors("right-high", &high));
    variants.push(CastleVariant::from_descriptors(
        "left-high",
        &mirror_descriptors(&high),
    ));

    CastlePattern {
        name: "Mino",
        variants,
        score: TaperedScore::new_tapered(180, 60),
        flexibility: 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mino_castle_variants() {
        let pattern = get_mino_castle();
        assert_eq!(pattern.name, "Mino");
        assert_eq!(pattern.variants.len(), 4);

        for variant in &pattern.variants {
            let required = variant.pieces.iter().filter(|piece| piece.required).count();
            assert!(required >= 2);
        }
    }
}
