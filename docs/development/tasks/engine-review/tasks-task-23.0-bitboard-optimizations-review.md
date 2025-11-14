## Relevant Files

- `src/bitboards.rs` - Core `BitboardBoard`, move legality, attack queries, board cloning, hashing.
- `src/bitboards/sliding_moves.rs` - Magic-bitboard engine, ray-cast fallback, sliding move builders.
- `src/bitboards/integration.rs` - `BitScanningOptimizer`, adaptive popcount/bitscan dispatch, API glue.
- `src/bitboards/attack_patterns.rs` - Precomputed attack tables and metadata for non-sliding pieces.
- `src/bitboards/cache_opt.rs` - Cache-aware popcount/bit-scan helpers and prefetch toggles.
- `src/bitboards/branch_opt.rs` - Branch-hinted popcount/bitscan helpers (`likely`/`unlikely` utilities).
- `src/bitboards/api.rs` - Public bitboard API surface, helper exports, platform abstractions.
- `src/types.rs` - `Bitboard` alias, `MagicTable`, `MagicBitboard`, search-related structs referencing bitboards.
- `tests/bitboards/` - Unit and integration tests for bitboard board state, attack generation, hashing.
- `benches/bitscan_comprehensive_benchmarks.rs` - Popcount/bit-scan criterion suites referenced in the review.
- `benches/attack_pattern_performance_benchmarks.rs` - Attack generation benchmark harness.

### Notes

- Include unit tests alongside the modules they verify (e.g., `src/bitboards.rs` ↔ `src/bitboards.rs` tests module or `tests/bitboards/`).
- Prefer Criterion benchmarks under `benches/` to capture regressions in move generation, attack detection, and cloning cost.
- Ensure new configuration toggles are documented and wired into existing telemetry/logging helpers for engine instrumentation.
- Follow the bitboard module’s existing patterns for unsafe/platform-specific code (feature flags for SIMD, BMI1, etc.).

## Tasks

- [ ] 1.0 Board State Encoding & Hash Integrity
  - [ ] 1.1 Replace `piece_positions: HashMap<Position, Piece>` with a fixed `[Option<Piece>; 81]` (or equivalent) backed directly by bitboards to eliminate per-square hashing.
  - [ ] 1.2 Update board accessors (`get_piece`, `is_occupied`, iteration helpers) to read from the fixed array and ensure bitboard/state stay in sync via centralized setters.
  - [ ] 1.3 Introduce a full Zobrist-style hash that covers side-to-move, pieces in hand, drops, and occupancy; thread it through `get_position_id`, repetition detection, and TT probes.
  - [ ] 1.4 Rework `clone`, `is_legal_move`, and other copy-heavy call sites to avoid cloning large attack tables by sharing them (Arc/static) and reusing captured-piece buffers.
  - [ ] 1.5 Add regression tests validating hash uniqueness (different hands/players produce different hashes) and that board cloning preserves bitboards without duplicating tables.
  - [ ] 1.6 Document the new encoding and hashing approach in `task-23.0` notes plus update any developer docs referencing the HashMap-based storage.

- [ ] 2.0 Sliding Move Infrastructure Hardening
  - [ ] 2.1 Refactor magic-table ownership so initialization stores tables in a shared singleton/Arc and the board holds lightweight references; prevent `Option::take()` from invalidating future setups.
  - [ ] 2.2 Implement the ray-cast fallback to correctly generate rook/bishop/promoted sliding moves when magic data is unavailable, using occupancy masks rather than returning empty boards.
  - [ ] 2.3 Add runtime warnings/telemetry when the engine runs without magic support or falls back to ray-cast generation, including counters exposed via `debug_utils`.
  - [ ] 2.4 Ensure `sliding_moves.rs` iterates attack bitboards via bit scans (not 81-square loops) for both magic and fallback paths, sharing helper utilities.
  - [ ] 2.5 Extend tests to cover magic-enabled, fallback-only, and mixed scenarios (missing table entries, invalidations) to guarantee sliding move correctness across platforms.

- [ ] 3.0 Bitboard-Centric Attack & Move Iteration
  - [ ] 3.1 Rewrite `is_square_attacked_by` to iterate attackers by bitboard (e.g., per-piece masks + attack tables) instead of nested 9×9 loops with HashMap lookups.
  - [ ] 3.2 Update `piece_attacks_square` and drop-specific helpers to leverage precomputed attack tables for non-sliding pieces plus bit scans for sliding pieces.
  - [ ] 3.3 Replace all 0..81 loops in move generators (`generate_sliding_moves`, drop move builders, check detection) with `while attacks != 0 { idx = attacks.trailing_zeros(); ... }`.
  - [ ] 3.4 Add SEE/perf-critical helpers (e.g., iterator wrappers) that yield target squares from a bitboard, ensuring they integrate with pruning/search modules.
  - [ ] 3.5 Create regression tests comparing old vs. new attack results on representative positions (dense opening, sparse endgame, drop-heavy midgame) to confirm parity.
  - [ ] 3.6 Measure and record node/time improvements from bit-iteration rewrites, feeding results back into the review appendix.

- [ ] 4.0 Adaptive Bit-Scan & Branch Optimization Fixes
  - [ ] 4.1 Correct `BitScanningOptimizer::estimate_bit_count` so it counts high/low halves independently (or simply uses `bb.count_ones()` thresholds) to avoid misclassifying dense boards.
  - [ ] 4.2 Replace the no-op `likely`/`unlikely` helpers with cfg-gated wrappers around `core::intrinsics::{likely, unlikely}` (or compiler hints) and provide safe fallbacks for non-nightly targets.
  - [ ] 4.3 Audit adaptive dispatch points to ensure the corrected estimator picks the intended cache/bmi/debruijn paths; add logging counters for chosen strategies.
  - [ ] 4.4 Update public API surfaces (`bitboards::api`, `integration.rs`) so downstream callers can select or override scanning strategies (e.g., via config flags).
  - [ ] 4.5 Expand unit tests covering estimator edge cases (bits only in high half, dense boards, empty boards) and branch-hint wrappers on all supported targets.
  - [ ] 4.6 Document configuration/tuning guidance for adaptive scanning in the module docs and PRD follow-up.

- [ ] 5.0 Benchmarks, Telemetry, and Regression Safeguards
  - [ ] 5.1 Extend Criterion suites to benchmark board cloning, legal move generation, attack detection, and sliding move throughput before/after optimizations.
  - [ ] 5.2 Add telemetry counters for board clones, ray-cast fallback usage, attack-table initialization time/memory, and hash collisions; surface via debug logs or metrics exports.
  - [ ] 5.3 Capture benchmark results (node count, time per move generation) in the `task-23.0` documentation to quantify impact.
  - [ ] 5.4 Create integration tests ensuring platform-specific code paths (SIMD/BMI, fallback) remain functional in CI, including wasm/ARM builds if applicable.
  - [ ] 5.5 Update developer docs/readmes to explain how to run the new benchmarks, interpret telemetry, and configure feature flags.

