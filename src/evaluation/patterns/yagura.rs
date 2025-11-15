use crate::evaluation::castle_geometry::{
    CastlePieceClass, CastlePieceDescriptor, CastlePieceRole, RelativeOffset,
};
use crate::evaluation::castles::{
    mirror_descriptors, CastlePattern, CastleVariant, GOLD_FAMILY, KNIGHT_FAMILY, LANCE_FAMILY,
    PAWN_WALL_FAMILY, SILVER_FAMILY,
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
            true,
            7,
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
            CastlePieceClass::AnyOf(KNIGHT_FAMILY),
            RelativeOffset::new(-2, -3),
            false,
            6,
            CastlePieceRole::SecondaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(LANCE_FAMILY),
            RelativeOffset::new(0, -3),
            false,
            5,
            CastlePieceRole::SecondaryDefender,
        ),
    ]
}

fn advanced_shell() -> Vec<CastlePieceDescriptor> {
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
            RelativeOffset::new(-1, -2),
            true,
            9,
            CastlePieceRole::PrimaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-2, -2),
            true,
            7,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(PAWN_WALL_FAMILY),
            RelativeOffset::new(-3, -2),
            false,
            6,
            CastlePieceRole::PawnShield,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(KNIGHT_FAMILY),
            RelativeOffset::new(-1, -3),
            false,
            6,
            CastlePieceRole::SecondaryDefender,
        ),
        CastlePieceDescriptor::new(
            CastlePieceClass::AnyOf(LANCE_FAMILY),
            RelativeOffset::new(0, -2),
            false,
            5,
            CastlePieceRole::SecondaryDefender,
        ),
    ]
}

pub fn get_yagura_castle() -> CastlePattern {
    let base = base_shell();
    let advanced = advanced_shell();

    let mut variants = Vec::new();
    variants.push(CastleVariant::from_descriptors("left-base", &base));
    variants.push(CastleVariant::from_descriptors(
        "right-base",
        &mirror_descriptors(&base),
    ));
    variants.push(CastleVariant::from_descriptors("left-advanced", &advanced));
    variants.push(CastleVariant::from_descriptors(
        "right-advanced",
        &mirror_descriptors(&advanced),
    ));

    CastlePattern {
        name: "Yagura",
        variants,
        score: TaperedScore::new_tapered(160, 80),
        flexibility: 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yagura_castle_variants() {
        let pattern = get_yagura_castle();
        assert_eq!(pattern.name, "Yagura");
        assert_eq!(pattern.variants.len(), 4);

        for variant in &pattern.variants {
            let required = variant.pieces.iter().filter(|piece| piece.required).count();
            assert!(required >= 3);
        }
    }
}
