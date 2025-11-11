# Tactical Pattern Validation Playbook

This note captures the regression checks and performance signals that back the
Task 14.0 tactical pattern recognition work. Use it when validating local
changes or wiring automated CI/benchmark jobs.

## Regression Tests

- `cargo test --test tactical_patterns_accuracy_tests`
  - Covers forks, pins, skewers, discovered attacks, back-rank threats, and
    drop-based motifs with blocker-aware fixtures.
  - Includes `integrated_evaluator_respects_tactical_polarity` to ensure the
    top-level evaluator penalises defenders and rewards attackers consistently.
- `cargo test --test tactical_corpus_validation_tests`
  - Loads `tests/data/tactical_corpus.toml`, a curated SFEN corpus containing
    both historical failures and success cases (fork blockers, back-rank
    exposure, skewer penalties, discovered bonuses, drop motifs, and a quiet
    baseline).
  - Asserts motif polarity and rough symmetry between players with a
    ±80 cp tolerance to account for blended motif weights.

Run both tests in CI to guard against tactical regressions:

```bash
cargo test --test tactical_patterns_accuracy_tests
cargo test --test tactical_corpus_validation_tests
```

## Performance Benchmarks

- `cargo bench --bench tactical_patterns_performance_benchmarks`
  - `tactical_recognizer_construction/*`: recognizer instantiation and config
    swaps (baseline < 5 µs per iteration on Apple M2).
  - `tactical_single_position/*`: single-pass evaluation for each corpus entry.
  - `tactical_batch_evaluation/*`: iterates the full corpus once (and 16×) to
    spotlight allocation spikes or enumeration regressions. The single-pass
    batch stays under ~200 µs on Apple M2; raise alerts if it regresses by more
    than 20 %.

Benchmark output lives under `target/criterion/`. Capture a baseline (`cargo
bench`) after major changes and compare future runs with `cargo bench --baseline
<name>` to surface slowdowns.

## CI Integration

- Unit/validation tests: add the commands above to the default CI test stage.
  They run in under 1 s combined and catch polarity, scaling, and FEN-regression
  issues.
- Benchmarks: schedule `cargo bench --bench
  tactical_patterns_performance_benchmarks` in nightly or gated jobs. Fail the
  pipeline if:
  - Recognizer construction exceeds 10 µs.
  - Single-position evaluations regress by >20 %.
  - Batch iteration exceeds the historical p95 recorded in Criterion (store
    baselines in artifact storage or compare against previous run summaries).

Document benchmark thresholds in CI job definitions so alerting expectations are
clear. Update this playbook whenever new motifs, corpus entries, or thresholds
are introduced.

