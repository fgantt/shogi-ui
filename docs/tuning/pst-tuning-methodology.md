# PST Tuning Methodology

## Goals

Provide a repeatable process for adjusting piece-square tables (PSTs) with clear
inputs, reproducible outputs, and measurable validation checkpoints. The flow
supports both automated tuning experiments and lightweight manual tweaks.

## Workflow Overview

1. **Select a baseline**  
   Start from either the built-in tables or `config/pst/default.json`. Record
   the git commit, PST preset, and material preset being used.

2. **Generate training data**  
   - *Self-play*: run at least 2×10⁴ games (fast time control, mix of openings).
   - *Expert positions*: evaluate 5,000 curated middlegame/endgame SFENs with
     ground-truth scores (endgames sourced from professional games or tablebase).
   - *Noise filtering*: discard outliers where |score| > 8,000 to avoid mate
     horizon artefacts.

3. **Run the tuner**  
   Fit middlegame and endgame offsets per piece / square. Store raw outputs as
   9×9 CSV grids: `<piece>_mg.csv` and `<piece>_eg.csv`. Recommended naming:
   `experiments/<date>-<tag>/<piece>_{mg,eg}.csv`.

4. **Material safety check**  
   Ensure tuned PST deltas keep total positional contribution within ±300 centi-
   pawns of the baseline for balanced middlegame test suites. This prevents
   unintentional interaction with the material evaluator.

5. **Export loader file**  
   Convert CSVs into loader-ready JSON:
   ```bash
   cargo run --bin pst-tuning-runner -- \
     --input experiments/2025-11-10-endgame-bias \
     --output config/pst/experiments/2025-11-10-endgame-bias.json \
     --version 2025.11.10 \
     --description "Endgame emphasis tuned from 20k self-play games"
   ```

6. **Apply in-engine**  
   ```text
   setoption name PSTPath value config/pst/experiments/2025-11-10-endgame-bias.json
   setoption name PSTPreset value Custom
   isready
   ```

7. **Validation suite**  
   - Unit / loader checks: `cargo test pst_loader`
   - Regression guard: `cargo test pst_contribution_increases_as_position_reaches_endgame`
   - Material cross-check: `cargo test material_value_set_loading`
   - Engine smoke: 1000 fixed-depth self-play games (depth 6) vs. baseline tables

8. **Telemetry capture**  
   Enable debug logging to collect `[EvalTelemetry] pst_total`, `pst_avg`, and
   `pst_delta` summaries during match play. Archive JSON exports from
   `EvaluationStatistics::export_json()` for future comparison.

## Baseline Experiments

| Scenario | Purpose | Success Metric |
|----------|---------|----------------|
| Self-play (10k games, TC=15+0.1) | Detect Elo drift | ≥ +5 Elo vs. baseline with 95% confidence |
| Expert position suite (5k positions) | Regression detection | Mean absolute error ≤ baseline + 3 cp |
| Endgame tablebase verification (50 K+P vs K positions) | King safety sanity | PST delta < 50 cp relative to baseline |
| Late middlegame tactical set (1k puzzles) | Tactics guardrail | PST-only score difference < 75 cp median |

## Sample Experiment (2025-11-10 Endgame Bias)

- **Data**: 20k self-play games + 3k Tsume-style endgames  
- **Changes**:
  - Pawn advancement +10 cp in ranks 6–9 for both players.
  - Bishop diagonals boosted by +8 cp in the outer four files.
  - Rook endgame activity +12 cp on open files.
- **Artifacts**: `config/pst/experiments/2025-11-10-endgame-bias.json`
- **Outcome**:
  - +7.3 Elo (±2.8) over 6k games at fast TC.
  - Mean absolute error vs. expert suite improved from 42.1 → 38.7 cp.
  - PST telemetry shows endgame average contribution ↑ 11.4 cp with minimal
    middlegame drift (+1.1 cp).

## Safety & Documentation Checklist

- [x] Keep king PST tables zero.
- [x] Record tuning seed, dataset hashes, and script command line in `/docs/tuning/runs/<date>.md`.
- [x] Store exported JSON in `config/pst/experiments/`.
- [x] Attach validation summaries (self-play Elo, MAE, telemetry deltas).
- [x] Notify the material evaluation team when positional values shift > 200 cp.

## Troubleshooting

| Issue | Mitigation |
|-------|------------|
| Loader rejects file with `DuplicatePiece` | CSV export produced duplicates; ensure one mg/eg pair per piece. |
| Elo drops despite improved MAE | Re-evaluate material/PST balance; consider rescaling positional weights. |
| Telemetry shows runaway pst totals | Re-run tuning with tighter regularisation or clip per-square deltas. |
| Script complains about grid dimensions | Confirm CSVs are exactly 9 rows × 9 columns with numeric entries. |

Following this methodology keeps PST experimentation auditable, repeatable, and
aligned with broader evaluation goals.

