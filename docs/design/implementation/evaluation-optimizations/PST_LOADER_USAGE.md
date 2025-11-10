# Piece-Square Table Loader Usage

## Overview

The PST loader lets you swap positional value sets without rebuilding the engine.  
Each table is described in JSON (9×9 grids for middlegame and endgame) and is
loaded on demand through new USI engine options:

- `PSTPreset` – selects one of the in-tree presets (`Builtin`, `Default`, `Custom`).
- `PSTPath` – absolute/relative path to a JSON file when `Custom` data is required.

The runtime wiring guarantees that invalid inputs produce clear error messages
and fall back to the previous working configuration.

## Presets and Default Files

| Preset   | Description | Notes |
|----------|-------------|-------|
| `Builtin` | Uses the baked-in Rust tables compiled with the engine. | Zero external dependencies; matches historical behaviour. |
| `Default` | Loads `config/pst/default.json`. | Mirrors the baked-in values in JSON form; a good starting point for tuning. |
| `Custom` | Requires an explicit `PSTPath`. | Use for experimental tables generated offline. |

Switch presets through USI:

```text
setoption name PSTPreset value Builtin
setoption name PSTPreset value Default
setoption name PSTPreset value Custom
```

When selecting `Custom`, provide the JSON path before searching:

```text
setoption name PSTPath value /abs/path/to/my_tables.json
# Optional: switching back to builtin clears the override
setoption name PSTPreset value Builtin
```

Passing an empty string to `PSTPath` clears the override and reverts to the
current preset (`Builtin` by default).

## JSON Schema

The loader expects a top-level object containing optional metadata plus a
`tables` map keyed by piece names.

```json
{
  "version": "1.0.0",
  "description": "aggressive middlegame tuning",
  "tables": {
    "pawn": {
      "mg": [[0, 1, 2, 3, 4, 3, 2, 1, 0], "... eight more rows ..."],
      "eg": [[2, 3, 4, 5, 6, 5, 4, 3, 2], "... eight more rows ..."]
    },
    "lance": { "mg": [[...]], "eg": [[...]] },
    "... remaining pieces ...",
    "promoted_rook": { "mg": [[...]], "eg": [[...]] }
  }
}
```

*Notes*:

- Keys are case-insensitive ASCII (e.g. `Promoted_Rook` is accepted).
- Each value must cover all 9×9 squares; the loader enforces completeness and
  rejects duplicate or missing pieces.
- King tables must contain only zeros (safety requirement for evaluation
  invariants).

## Applying and Verifying a Custom Table

1. Generate or edit a JSON file following the schema above.
2. Point the engine at the file:
   ```text
   setoption name PSTPath value /path/to/new_tables.json
   setoption name PSTPreset value Custom
   ```
3. Confirm load success in the engine log (`info string Loaded PST...`). On
   failure the engine retains the previous configuration and prints the error.
4. Run the quick validation suite:
   ```bash
   cargo test pst_loader
   cargo test pst_contribution_increases_as_position_reaches_endgame
   ```
5. (Optional) Capture telemetry for spot checks. With debug logging enabled,
   the search engine prints `[EvalTelemetry] pst_total`, `pst_avg`, and `pst_top`
   lines showing the new tables at work.

## Safety Guidelines

- Keep king tables zeroed to avoid destabilising evaluation heuristics.
- Store custom JSON files alongside experiment metadata (seed, tuning method,
  training data) for reproducibility.
- When scripting bulk experiments, prefer absolute paths to avoid working
  directory surprises.
- Re-run the loader tests after regenerating tables; they catch missing pieces
  or non-square data before a match.
- Combine PST changes with material presets cautiously. Record both presets in
  telemetry so regression analysis can separate positional vs. material effects.

## Troubleshooting

| Symptom | Resolution |
|---------|------------|
| `MissingCustomPath` error | Provide `setoption name PSTPath value <file>` before enabling the `Custom` preset. |
| `UnknownPiece` error | Verify all table keys match supported piece strings (e.g. `promoted_silver`). |
| Non-zero king error | Zero out every king table entry; king mobility is captured via other evaluators. |
| Old tables still active | Ensure `setoption` commands are issued *before* `isready`/`go`, and double-check that the engine reported a successful load. |

For advanced automation, integrate the loader with tuning scripts that emit JSON
files directly into `config/pst/` or a dedicated experiment directory, then use
the USI options above to replay results without recompilation.

