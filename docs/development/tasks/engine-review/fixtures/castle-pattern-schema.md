# Castle Pattern Schema

This document summarizes the new symmetry-aware castle pattern model introduced for Task 16.0. Use it when adding or updating castle recognizers so new work aligns with the shared geometry helpers and defender families.

## Core Types

- `CastlePattern`
  - Holds the castle name, an array of `CastleVariant`s, the tapered base score, and the global `flexibility` budget used across variants.
- `CastleVariant`
  - Represents one orientation or shell for the castle. Each variant is identified by a short `id` (e.g., `"right-base"`, `"left-advanced"`).
  - Contains a list of `CastlePiece` entries produced from descriptors.
- `CastlePiece`
  - Created from `CastlePieceDescriptor` values.
  - Stores the defender `class`, relative offset (via `RelativeOffset`), `required` flag, and a `weight` used for match quality calculations.
- `RelativeOffset`
  - Expresses a piece location relative to the king using player-centric coordinates: negative ranks move toward the opponent; negative files move to the player's left.
  - `RelativeOffset::mirrored()` flips the file to build opposite-wing variants.

## Defender Classes & Families

Castle pieces now work with the `CastlePieceClass` enum:

- `Exact(PieceType)` — requires the specific piece type.
- `AnyOf(&'static [PieceType])` — accepts any member from a family slice.

Reusable defender families live in `castle_geometry.rs` and are re-exported from `castles.rs`:

- `GOLD_FAMILY` – `[Gold, PromotedSilver]`
- `SILVER_FAMILY` – `[Silver, PromotedSilver]`
- `PAWN_WALL_FAMILY` – `[Pawn, PromotedPawn]`
- `LANCE_FAMILY` – `[Lance, PromotedLance]`
- `KNIGHT_FAMILY` – `[Knight, PromotedKnight]`

Add new families here when castles need alternative defenders.

### Piece Roles

`CastlePieceRole` tags each descriptor with its tactical job:

- `PrimaryDefender` – core gold/silver shell pieces that must survive to keep the king safe.
- `SecondaryDefender` – outer guards (e.g., lance/knight posts) that absorb pressure and prevent infiltration.
- `PawnShield` – frozen pawn chains and drops that block direct attacks along the king’s front.
- `Buffer` – additional soft defenders (e.g., promoted pawns two files away) that provide depth to the castle. These are optional but improve shell integrity when present.

Accurate roles ensure the recognizer can report shell completeness and compute penalties for missing coverage.

### Zone Geometry Helpers

`castle_geometry.rs` also exposes three pre-defined offset rings used during evaluation:

- `KING_ZONE_RING` – the 3×3 halo around the king used to measure zone coverage and infiltration.
- `FORWARD_SHIELD_ARC` – the forward-facing arc where pawn shields are expected.
- `BUFFER_RING` – secondary squares two steps from the king that act as depth buffers.

The recognizer converts these offsets using `RelativeOffset::to_absolute` so the same constants work for both players.

## Building Variants

1. Define a descriptor shell with `CastlePieceDescriptor::new(class, offset, required, weight)` values.
2. Build primary variants with `CastleVariant::from_descriptors("right-base", &descriptors)`.
3. Generate mirrored variants via `mirror_descriptors(&descriptors)` and wrap with `CastleVariant::from_descriptors("left-base", &mirrored)`.
4. Compose `CastlePattern { name, variants, score, flexibility }`.

> Tip: Keep the descriptor arrays small and readable by grouping required defenders first, then optional shells (pawn walls, knight/lance guards, etc.).

## Match Quality Expectations

- Required defenders must be present for a variant to score.
- `flexibility` controls how many optional pieces can be missing before the variant is discarded.
- Match quality uses both piece count (60%) and weight coverage (40%). Weights should reflect the importance of each defender relative to the overall structure.
- Zone coverage, forward shield coverage, and buffer integrity are computed from the helpers above. These ratios, plus the pattern match quality, drive the graded castle score that KingSafety consumes.

## Unit Tests

When adding or updating patterns:

- Verify variant coverage (e.g., left/right, base/advanced).
- Ensure promoted defenders and mirrored orientations are accepted.
- Add integration tests similar to `test_recognize_anaguma_with_promoted_silver` to validate real positions.

## Updating the Recognizer Cache

`CastleRecognizer` now caches `CachedMatch` entries that store the winning pattern/variant and adjusted score. The cache key includes the king position and player. When patterns change significantly, consider invalidating existing caches or bumping configuration defaults to avoid stale matches.

## King Safety Configuration Integration

`KingSafetyConfig` exposes weights that combine pattern-derived ratios with live zone metrics:

- `pattern_coverage_weight` / `zone_coverage_weight`
- `pattern_shield_weight` / `zone_shield_weight`
- `exposure_zone_weight`, `exposure_shield_weight`, `exposure_primary_weight`
- `infiltration_penalty`, `exposed_king_penalty`, and per-missing defender penalties

Tweaking these values lets tuning adjust how aggressively the engine rewards complete castles versus penalising exposed kings or infiltration.

## Checklist for New Castles

- [ ] Add descriptor shells in a dedicated module under `src/evaluation/patterns/`.
- [ ] Use defender families instead of hard-coded promoted types when possible.
- [ ] Provide left/right variants through mirroring.
- [ ] Add unit tests for symmetry, promotion acceptance, and variant counts.
- [ ] Extend integration tests with realistic board fixtures covering intact, partial, and broken castles.
