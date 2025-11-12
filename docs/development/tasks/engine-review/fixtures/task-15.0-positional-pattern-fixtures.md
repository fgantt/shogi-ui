# Task 15.0 Positional Pattern Fixtures

This catalog captures the canonical shogi scenarios introduced while completing Task 15.0.  
Each fixture is exercised by `tests/positional_patterns_regression_tests.rs` and the Criterion
benchmarks in `benches/positional_patterns_performance_benchmarks.rs`.

| Fixture | Theme | Advantage Reference | Expected Delta (mg) | Description |
|---------|-------|---------------------|---------------------|-------------|
| `central_file_bind` | Central fight | Black | ≥ 40 cp | Black doubles a rook and minor pieces on the 5th file with an extra pawn in hand, binding White’s central pawns. |
| `castle_gap_exposed_king` | Castle weakness | Black | ≥ 30 cp | White’s king-side wall is punctured; Black’s rook and bishop coordinate on the exposed file while White’s defenders are misplaced. |
| `space_clamp_double_lance` | Space clamp | Black | ≥ 35 cp | Black pushes a pawn trio deep into White’s camp, anchors lances on both wings, and keeps two pawns in hand to maintain the clamp. |

> **Delta definition:**  
> The regression suite measures `delta = score_black.mg − score_white.mg`.  
> Positive values favour Black, negative values favour White.

## Board Construction Summary

Fixtures are constructed programmatically via `shogi_engine::evaluation::positional_fixtures`.
Key placements (row 0 is White’s home rank, columns are zero-indexed):

- **`central_file_bind`**
  - Black: K(8,4), R(4,4), B(5,3), S(5,5), G(4,3), P(6,4), P(5,4); one pawn in hand.
  - White: K(0,4), S(2,4), P(3,4), P(3,3), B(1,6).

- **`castle_gap_exposed_king`**
  - Black: K(8,4), R(3,4), B(2,2), G(4,4), P(4,5), P(2,4).
  - White: K(0,4), G(1,5), S(1,3).

- **`space_clamp_double_lance`**
  - Black: K(8,4), P(3,2), P(3,4), P(3,6), L(2,0), L(2,8), R(4,6), S(5,5); two pawns in hand.
  - White: K(0,4), G(6,4), S(5,4), P(5,3), P(5,5).

## Validation & Benchmarks

- **Regression tests:** `cargo test positional_patterns_regression_tests -- --nocapture`  
  (See repository notes regarding the rustc ICE affecting `cargo test positional_patterns`;  
  the targeted suite runs successfully.)
- **Benchmarks:** `cargo bench positional_patterns_performance_benchmarks`
  compares fresh vs. reused analyzers across all fixtures.

These fixtures double as documentation and executable guarantees that the redesigned positional
heuristics track control, castle integrity, and territorial clamps with shogi-specific accuracy.

