# Material Value Sets

This guide summarizes the assets and workflows available when tuning or swapping material value tables for the engine. For hands-on, step-by-step instructions, see `docs/tuning/material-value-set-workflows.md`.

## Built-in Presets

| ID | File | Description |
|----|------|-------------|
| `research` | `resources/material/research.json` | Default production values used when `use_research_values = true`. |
| `classic` | `resources/material/classic.json` | Legacy table retained for regression comparisons and debugging. |

Each preset exposes:
- `board_values`: tapered middlegame/endgame scores for pieces on the board.
- `hand_values`: optional hand-piece overrides; missing entries fallback to board values.
- Metadata (`display_name`, `version`, `last_updated`, `source`) used for telemetry tagging.

## Configuration Surface

Add the following keys under `[tapered.material]` (TOML) or the equivalent JSON structure:

```toml
include_hand_pieces = true         # Count pieces in hand (default: true)
use_research_values = true         # Toggle between research/classic presets
values_path = "weights/material/experiment.json"  # Optional external file (JSON/TOML)
enable_fast_loop = false           # Opt-in popcount traversal guarded by regression tests
```

- When `values_path` is supplied the engine loads the external file regardless of `use_research_values`.
- File resolution is relative to the working directory. Failures emit a `MaterialEvaluator` debug log and fall back to the requested preset.
- The `enable_fast_loop` toggle activates the optimized traversal path (see Telemetry & Validation).

## Custom Table Lifecycle

1. Clone an existing preset using `MaterialValueSet::classic()` / `research()` or copy the JSON from `resources/material/`.
2. Edit values or metadata. All 14 piece types (including promoted variants) must appear in both `board_values` and `hand_values` (use `null` to inherit board values).
3. Save via `MaterialValueLoader::save` (JSON) or hand-author a TOML file following `resources/material/material_values.schema.json`.
4. Point the engine at the modified file using `values_path` and restart or hot-reload (dev builds only).

Refer to `docs/tuning/material-value-set-workflows.md` for CLI snippets and automation hooks.

## Telemetry & Validation

- Material telemetry snapshots (`EvaluationTelemetry.material`) publish:
  - per-piece board and hand contributions
  - hand balance deltas
  - phase-weighted totals
  - preset usage counters (`custom`, `research`, `classic`)
- Run `cargo test material_value_set_loading_tests` for loader round-trip coverage.
- Run `cargo test --features material_fast_loop material_delta` after changing tables or enabling the fast loop to ensure parity against the legacy traversal.
- Benchmark impacts with `cargo bench --bench material_evaluation_performance_benchmarks --features "legacy-tests,material_fast_loop"`.

## Migration Notes

- Earlier releases stored material values inline in `MaterialEvaluator`; those constants remain as the `classic` preset for compatibility.
- Configuration files lacking `include_hand_pieces` or `use_research_values` continue to compile; defaults mirror previous behaviour (hand pieces counted, research preset active).
- Third-party integrations should prefer `values_path` rather than copying presets to avoid drift.
