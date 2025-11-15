use crate::types::core::{PieceType, Player, Position};

/// Represents a relative offset from the king's square using player-centric axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativeOffset {
    pub rank: i8,
    pub file: i8,
}

impl RelativeOffset {
    pub const fn new(rank: i8, file: i8) -> Self {
        Self { rank, file }
    }

    /// Mirror the offset across the king's file (left/right swap).
    pub const fn mirrored(self) -> Self {
        Self {
            rank: self.rank,
            file: -self.file,
        }
    }

    /// Convert the offset into an absolute board position for the given player and king square.
    pub fn to_absolute(self, king: Position, player: Player) -> Option<Position> {
        let (rank_delta, file_delta) = match player {
            Player::Black => (self.rank, self.file),
            Player::White => (-self.rank, -self.file),
        };

        let new_rank = king.row as i8 + rank_delta;
        let new_file = king.col as i8 + file_delta;

        if (0..9).contains(&new_rank) && (0..9).contains(&new_file) {
            Some(Position::new(new_rank as u8, new_file as u8))
        } else {
            None
        }
    }
}

/// Groups of interchangeable defenders that fulfil the same structural role in a castle.
#[derive(Debug, Clone, Copy)]
pub enum CastlePieceClass {
    Exact(PieceType),
    AnyOf(&'static [PieceType]),
}

impl CastlePieceClass {
    pub fn matches(self, piece_type: PieceType) -> bool {
        match self {
            CastlePieceClass::Exact(expected) => expected == piece_type,
            CastlePieceClass::AnyOf(candidates) => {
                candidates.iter().copied().any(|p| p == piece_type)
            }
        }
    }
}

/// Convenience constructor for an exact piece requirement.
pub const fn exact(piece: PieceType) -> CastlePieceClass {
    CastlePieceClass::Exact(piece)
}

/// Family of defenders that guard the king like a gold: Gold itself plus promoted silvers.
pub const GOLD_FAMILY: &[PieceType] = &[PieceType::Gold, PieceType::PromotedSilver];

/// Family of flexible silver defenders, accepting the promoted form as well.
pub const SILVER_FAMILY: &[PieceType] = &[PieceType::Silver, PieceType::PromotedSilver];

/// Family of pawn-based defenders, including promoted and drop pawns that can still shield the king.
pub const PAWN_WALL_FAMILY: &[PieceType] = &[PieceType::Pawn, PieceType::PromotedPawn];

/// Family of lance defenders protecting the outer file.
pub const LANCE_FAMILY: &[PieceType] = &[PieceType::Lance, PieceType::PromotedLance];

/// Family of knight defenders covering jump squares around the king.
pub const KNIGHT_FAMILY: &[PieceType] = &[PieceType::Knight, PieceType::PromotedKnight];

pub const KING_ZONE_RING: [RelativeOffset; 8] = [
    RelativeOffset::new(-1, -1),
    RelativeOffset::new(-1, 0),
    RelativeOffset::new(-1, 1),
    RelativeOffset::new(0, -1),
    RelativeOffset::new(0, 1),
    RelativeOffset::new(1, -1),
    RelativeOffset::new(1, 0),
    RelativeOffset::new(1, 1),
];

pub const FORWARD_SHIELD_ARC: [RelativeOffset; 3] = [
    RelativeOffset::new(-1, -1),
    RelativeOffset::new(-1, 0),
    RelativeOffset::new(-1, 1),
];

pub const BUFFER_RING: [RelativeOffset; 9] = [
    RelativeOffset::new(-2, -2),
    RelativeOffset::new(-2, -1),
    RelativeOffset::new(-2, 0),
    RelativeOffset::new(-2, 1),
    RelativeOffset::new(-2, 2),
    RelativeOffset::new(-1, -2),
    RelativeOffset::new(-1, 2),
    RelativeOffset::new(0, -2),
    RelativeOffset::new(0, 2),
];

pub const PAWN_WALL_ARC: [RelativeOffset; 3] = [
    RelativeOffset::new(-2, -2),
    RelativeOffset::new(-1, -2),
    RelativeOffset::new(0, -2),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlePieceRole {
    PrimaryDefender,
    SecondaryDefender,
    PawnShield,
    Buffer,
}

/// Descriptor used during pattern construction before variants are expanded.
#[derive(Debug, Clone, Copy)]
pub struct CastlePieceDescriptor {
    pub class: CastlePieceClass,
    pub offset: RelativeOffset,
    pub required: bool,
    pub weight: u8,
    pub role: CastlePieceRole,
}

impl CastlePieceDescriptor {
    pub const fn new(
        class: CastlePieceClass,
        offset: RelativeOffset,
        required: bool,
        weight: u8,
        role: CastlePieceRole,
    ) -> Self {
        Self {
            class,
            offset,
            required,
            weight,
            role,
        }
    }

    pub const fn mirrored(self) -> Self {
        Self {
            class: self.class,
            offset: self.offset.mirrored(),
            required: self.required,
            weight: self.weight,
            role: self.role,
        }
    }
}

/// Helper to mirror an entire shell of descriptors.
pub fn mirror_descriptors(descriptors: &[CastlePieceDescriptor]) -> Vec<CastlePieceDescriptor> {
    descriptors.iter().map(|d| d.mirrored()).collect()
}
