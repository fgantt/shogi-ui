# Castle Pattern Fixtures

This document describes the canonical castle pattern fixtures used for testing and benchmarking in Task 16.0.

## Fixture Overview

The castle fixtures are organized by theme and castle type:

| Fixture | Theme | Castle Type | Player | Description |
|---------|-------|-------------|--------|-------------|
| `mino_canonical_black` | Canonical | Mino | Black | Complete Mino castle in center file |
| `mino_canonical_white` | Canonical | Mino | White | Complete Mino castle in center file |
| `anaguma_canonical_black` | Canonical | Anaguma | Black | Complete Anaguma castle |
| `yagura_canonical_black` | Canonical | Yagura | Black | Complete Yagura castle |
| `mino_mirrored_left_black` | Mirrored | Mino | Black | Mino castle on left side (file 2) |
| `mino_mirrored_right_black` | Mirrored | Mino | Black | Mino castle on right side (file 6) |
| `anaguma_mirrored_left_black` | Mirrored | Anaguma | Black | Anaguma castle on left side |
| `anaguma_mirrored_right_black` | Mirrored | Anaguma | Black | Anaguma castle on right side |
| `mino_partial_missing_silver` | Partial | Mino | Black | Mino castle missing silver defender |
| `mino_partial_missing_pawns` | Partial | Mino | Black | Mino castle missing pawn wall |
| `anaguma_partial_missing_gold` | Partial | Anaguma | Black | Anaguma castle missing one gold |
| `mino_broken_breached_wall` | Broken | Mino | Black | Mino castle with breached pawn wall |
| `anaguma_broken_missing_defenders` | Broken | Anaguma | Black | Anaguma castle missing multiple defenders |
| `mino_attacked_rook_file` | Attacked | Mino | Black | Mino castle under rook file attack |
| `anaguma_attacked_infiltration` | Attacked | Anaguma | Black | Anaguma castle with infiltrating piece |
| `yagura_attacked_mating_net` | Attacked | Yagura | Black | Yagura castle in mating net |

## Fixture Themes

### Canonical
Complete, well-formed castles that should be recognized with high quality scores (≥ 0.7).

### Mirrored
Left/right variants of castles that test symmetry-aware recognition and caching.

### Partial
Incomplete castles missing some defenders, testing graded scoring (quality 0.3-0.8).

### Broken
Severely damaged castles with breached walls or missing critical defenders (quality < 0.5).

### Attacked
Castles under attack, testing infiltration detection and attack penalty integration.

## Usage in Tests

```rust
use shogi_engine::evaluation::castle_fixtures::castle_fixtures;

let fixtures = castle_fixtures();
for fixture in fixtures {
    let (board, king_pos) = (fixture.builder)(fixture.player);
    let evaluation = recognizer.evaluate_castle(&board, fixture.player, king_pos);
    // Assert quality based on theme
}
```

## Usage in Benchmarks

The fixtures are used in `benches/castle_recognition_cache_benchmarks.rs` to measure:
- Recognition throughput across all fixture types
- Cache hit rate with repeated evaluations
- Telemetry overhead
- Performance across game phases (opening/middlegame/endgame)

## Validation & Benchmarks

Run regression tests:
```bash
cargo test --test castle_pattern_regression_tests -- --nocapture
```

Run integration tests:
```bash
cargo test --test castle_attack_integration_tests -- --nocapture
```

Run benchmarks:
```bash
cargo bench --bench castle_recognition_cache_benchmarks --features legacy-tests
```

## Expected Quality Scores

- **Canonical**: quality ≥ 0.7, matched_pattern is Some
- **Partial**: quality 0.3-0.8
- **Broken**: quality < 0.5
- **Attacked**: infiltration_ratio > 0.0

## Index

See `tests/data/castle_pattern_fixtures_index.toml` for the complete fixture catalog.

