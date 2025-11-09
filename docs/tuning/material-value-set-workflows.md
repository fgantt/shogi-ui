# Material Value Set Workflows

This document complements `docs/tuning/material-value-sets.md`, which provides a high-level overview of presets and configuration knobs.

## 1. Default Assets

Two canonical presets ship with the engine under `resources/material/`:

| File | Purpose |
|------|---------|
| `research.json` | Production-tuned values used by default (`use_research_values = true`). |
| `classic.json` | Legacy material tables retained for regression comparison. |
| `material_values.schema.json` | JSON schema describing the on-disk format (board and hand tapered scores for all piece types). |

Each file includes metadata (`id`, `display_name`, optional `description`, `version`, `last_updated`) alongside arrays of middlegame (`mg`) and endgame (`eg`) tapered values.

---

## 2. Loading Custom Tables

You can point the engine at an external table by adding the `values_path` field to material configuration sources (TOML, JSON, or CLI):

```toml
[tapered.material]
include_hand_pieces = true
use_research_values = true      # Fallback preset
values_path = "weights/material/custom_run.json"
```

- When `values_path` is present the file is loaded regardless of `use_research_values`.
- Accepted formats: **JSON** (preferred) and **TOML**.
- Errors (missing files, parse failures, schema violations) fall back to the preset implied by `use_research_values` and emit a `MaterialEvaluator` debug log entry.

---

## 3. Saving & Round-Trip Editing

Programmatic access is available through `MaterialValueLoader::save` and `MaterialValueSet::from_path`:

```rust
use shogi_engine::evaluation::material::{MaterialValueSet, MaterialEvaluationConfig};
use shogi_engine::evaluation::material_value_loader::MaterialValueLoader;

let mut value_set = MaterialValueSet::classic();
value_set.id = "experiment-042".into();
value_set.board_values[PieceType::Silver.as_index()].mg += 25;

MaterialValueLoader::save(&value_set, std::path::Path::new("weights/material/experiment-042.json"))?;

let config = MaterialEvaluationConfig {
    include_hand_pieces: true,
    values_path: Some("weights/material/experiment-042.json".into()),
    ..MaterialEvaluationConfig::default()
};
```

This enables tuning runs to generate candidate tables, serialize them, and feed them back into the engine without recompilation.

---

## 4. Integration with Tuning Tools

1. **Generate Baseline:** Copy one of the built-in presets and adjust values or metadata as needed.
2. **Tuning Manager:** Point the self-play/tuning job at the generated file via `values_path` (recommended location: `weights/material/<experiment>.json`).
3. **Automation Hooks:** The tuning pipeline can update the JSON file in place; the engine will reload the values on startup.
4. **Validation:** Use the `material_value_set_loading_tests` regression test and benchmark suite (`cargo test material_value`) to ensure the updated tables parse correctly and produce deterministic scores.

---

## 5. Troubleshooting

- **File not found:** Ensure the path is absolute or relative to the engine working directory. A debug log entry (`MaterialEvaluator` tag) indicates fallback to the preset.
- **Schema violations:** Run the JSON through `material_values.schema.json` using a schema validator or rely on the error surfaced by `MaterialValueSetError::Validation`.
- **Inconsistent scores:** Re-run the integration tests to check that board and hand arrays contain all 14 piece entries (including promoted variants).

---

## 6. Future Enhancements

- CLI helpers to scaffold new value files directly from the engine.
- Optional hot-reload during tuning sessions.
- Aggregated telemetry comparing preset contributions across large self-play batches.

For now, the documented workflow provides a safe and observable path for experimenting with material value tables without touching source code constants.

