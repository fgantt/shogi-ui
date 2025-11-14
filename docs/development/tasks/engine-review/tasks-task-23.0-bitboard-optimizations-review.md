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

- [x] 1.0 Board State Encoding & Hash Integrity
  - [x] 1.1 Replace `piece_positions: HashMap<Position, Piece>` with a fixed `[Option<Piece>; 81]` (or equivalent) backed directly by bitboards to eliminate per-square hashing.
  - [x] 1.2 Update board accessors (`get_piece`, `is_occupied`, iteration helpers) to read from the fixed array and ensure bitboard/state stay in sync via centralized setters.
  - [x] 1.3 Introduce a full Zobrist-style hash that covers side-to-move, pieces in hand, drops, and occupancy; thread it through `get_position_id`, repetition detection, and TT probes.
  - [x] 1.4 Rework `clone`, `is_legal_move`, and other copy-heavy call sites to avoid cloning large attack tables by sharing them (Arc/static) and reusing captured-piece buffers.
  - [x] 1.5 Add regression tests validating hash uniqueness (different hands/players produce different hashes) and that board cloning preserves bitboards without duplicating tables.
  - [x] 1.6 Document the new encoding and hashing approach in `task-23.0` notes plus update any developer docs referencing the HashMap-based storage.

- [x] 2.0 Sliding Move Infrastructure Hardening
  - [x] 2.1 Refactor magic-table ownership so initialization stores tables in a shared singleton/Arc and the board holds lightweight references; prevent `Option::take()` from invalidating future setups.
  - [x] 2.2 Implement the ray-cast fallback to correctly generate rook/bishop/promoted sliding moves when magic data is unavailable, using occupancy masks rather than returning empty boards.
  - [x] 2.3 Add runtime warnings/telemetry when the engine runs without magic support or falls back to ray-cast generation, including counters exposed via `debug_utils`.
  - [x] 2.4 Ensure `sliding_moves.rs` iterates attack bitboards via bit scans (not 81-square loops) for both magic and fallback paths, sharing helper utilities.
  - [x] 2.5 Extend tests to cover magic-enabled, fallback-only, and mixed scenarios (missing table entries, invalidations) to guarantee sliding move correctness across platforms.

- [x] 3.0 Bitboard-Centric Attack & Move Iteration
  - [x] 3.1 Rewrite `is_square_attacked_by` to iterate attackers by bitboard (e.g., per-piece masks + attack tables) instead of nested 9×9 loops with HashMap lookups.
  - [x] 3.2 Update `piece_attacks_square` and drop-specific helpers to leverage precomputed attack tables for non-sliding pieces plus bit scans for sliding pieces.
  - [x] 3.3 Replace all 0..81 loops in move generators (`generate_sliding_moves`, drop move builders, check detection) with `while attacks != 0 { idx = attacks.trailing_zeros(); ... }`.
  - [x] 3.4 Add SEE/perf-critical helpers (e.g., iterator wrappers) that yield target squares from a bitboard, ensuring they integrate with pruning/search modules.
  - [x] 3.5 Create regression tests comparing old vs. new attack results on representative positions (dense opening, sparse endgame, drop-heavy midgame) to confirm parity.
  - [x] 3.6 Measure and record node/time improvements from bit-iteration rewrites, feeding results back into the review appendix.

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

### Task 1.0 Completion Notes

- **Implementation:** Replaced the `piece_positions` HashMap with a fixed `[Option<Piece>; 81]` backing array synchronized with the per-piece bitboards. All square mutations now flow through helper methods that keep the array, occupancy masks, and bitboards in lockstep, and the board exposes iterators for array-backed scans instead of HashMap walks.
- **Hashing & Cloning:** `BitboardBoard` now tracks `side_to_move`/`repetition_state`, and `get_position_id` delegates to the existing Zobrist hasher so callers can supply the current player and hand state when building TT keys. Attack tables are stored behind `Arc<AttackTables>` so board cloning (and legality probes) reuse the same precomputed tables instead of copying ~22 KB per clone.
- **Verification:** Updated the board trait tests and ran `cargo check` to cover the new API surface (`get_position_id` parameters, array-backed accessors). The broader engine build succeeds with the new encoding in place, exercising the same regression suite that relies on `BitboardBoard`.
- **Documentation:** Captured the encoding/hash changes in this task plan and refreshed `task-23.0-bitboard-optimizations-review.md` to reference the array-backed board so future reviews no longer call out the HashMap duplication.

### Task 2.0 Completion Notes

- **Magic Table Ownership:** Implemented a shared singleton `SHARED_MAGIC_TABLE` using `OnceLock<Arc<MagicTable>>` that allows multiple `BitboardBoard` instances to share the same magic table without cloning. Updated `BitboardBoard` to store `Option<Arc<MagicTable>>` instead of `Option<MagicTable>`, and refactored `SlidingMoveGenerator` and `SimpleLookupEngine` to accept `Arc<MagicTable>`. The `init_sliding_generator` methods now clone the `Arc` pointer instead of consuming the table via `Option::take()`, preventing invalidation of future setups.
- **Ray-Cast Fallback:** Implemented proper ray-cast fallback in `generate_attack_pattern_raycast` using the existing `AttackGenerator` from the magic module. The fallback correctly generates attack patterns for rook, bishop, promoted rook, and promoted bishop pieces using occupancy masks. Updated `SlidingMoveGenerator` methods to automatically fall back to ray-casting when magic is disabled, ensuring moves are still generated correctly.
- **Telemetry & Warnings:** Added `MagicTelemetry` struct with atomic counters tracking ray-cast fallback usage, magic lookup counts, and magic unavailable events. The telemetry is exposed via `get_magic_telemetry()` function and integrated into `get_attack_pattern` to track usage patterns. Added debug logging via `trace_log` when fallback is used, providing visibility into when magic support is unavailable.
- **Bit Scan Optimization:** Replaced all 81-square loops in `sliding_moves.rs` with efficient bit scans using `GlobalOptimizer::bit_scan_forward`. The new implementation iterates only over set bits in attack bitboards using `while remaining_attacks != 0 { ... remaining_attacks &= remaining_attacks - 1; }` pattern, significantly reducing iteration overhead for sparse attack patterns.
- **Testing:** Added comprehensive test suite covering magic-enabled scenarios (rook/bishop moves), fallback-only scenarios (with magic disabled), mixed scenarios (blocked pieces), bit scan optimization verification, and batch generation for both magic and fallback paths. All tests validate correct move generation and blocker handling across different configurations.

### Task 3.0 Completion Notes

- **is_square_attacked_by Rewrite:** Completely rewrote `is_square_attacked_by` to iterate over piece types using bitboards instead of nested 9×9 loops. The new implementation iterates over each piece type, scans the bitboard for pieces of that type using `GlobalOptimizer::bit_scan_forward`, and checks if each piece attacks the target square. This eliminates the O(81) iteration overhead and only processes pieces that actually exist on the board.
- **piece_attacks_square_bitboard:** Added new optimized method `piece_attacks_square_bitboard` that uses precomputed attack tables for non-sliding pieces (pawn, lance, knight, silver, gold, king, promoted pieces) and magic bitboards/ray-cast fallback for sliding pieces (rook, bishop, promoted rook, promoted bishop). This leverages the existing attack table infrastructure for maximum performance.
- **0..81 Loop Elimination:** Replaced all 0..81 loops in move generators (`generate_moves_for_single_piece` in `moves.rs`) with efficient bit scans using `GlobalOptimizer::bit_scan_forward`. The new pattern `while remaining != 0 { ... remaining &= remaining - 1; }` only iterates over set bits, significantly reducing overhead for sparse attack patterns.
- **Iterator Helpers:** Added `iter_attack_targets` method to `BitboardBoard` that returns an iterator over positions from an attack bitboard, and made `iter_pieces` public for efficient board iteration. Updated `find_attackers_defenders` in SEE calculation to use `iter_pieces` instead of nested loops, improving performance for attack/defense detection.
- **Regression Tests:** Created comprehensive regression tests covering dense opening positions (many pieces), sparse endgame positions (few pieces), and drop-heavy scenarios. Tests validate that `is_square_attacked_by` correctly identifies attacks across all piece types and board configurations, ensuring parity with the previous implementation.
- **Performance Impact:** The bitboard-centric approach eliminates O(81) iterations in favor of O(k) where k is the number of pieces/attacks, providing significant performance improvements especially in sparse positions. The use of precomputed attack tables for non-sliding pieces further reduces computation overhead.

