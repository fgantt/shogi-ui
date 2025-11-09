## Relevant Files

- `src/evaluation/tapered_eval.rs` - Coordinates tapered evaluation workflow, phase caching, and integration touchpoints.
- `src/evaluation/phase_transition.rs` - Hosts interpolation kernels (linear, cubic, sigmoid, smoothstep) and validation helpers.
- `src/types.rs` - Defines `TaperedScore`, configuration structs, phase constants, and piece phase weights.
- `src/evaluation/advanced_interpolation.rs` - Experimental spline/multi-phase interpolator implementations pending production integration.
- `src/evaluation/integration.rs` - `IntegratedEvaluator` pipeline that invokes tapered evaluation within broader feature extractors.
- `src/evaluation/performance.rs` - Profiling infrastructure and optimized evaluator path for benchmarking.
- `src/evaluation/config.rs` - Configuration presets referencing tapered evaluation settings.
- `src/evaluation/statistics.rs` - Aggregated evaluation metrics and statistics reporting.
- `tests/` - Existing interpolation correctness coverage (partially feature gated) requiring expansion.

### Notes

- Unit tests should live alongside corresponding modules under `src/evaluation/` whenever possible.
- Integration and regression coverage can target representative shogi positions to capture hand-piece and promotion dynamics.
- Enable or restructure gated tests so CI exercises all interpolation modes without the `legacy-tests` feature flag.
- Use `cargo test --all-features` when validating optional feature paths locally; default CI should rely on `cargo test`.

## Tasks

- [ ] 1.0 Phase Classification Accuracy Enhancements
- [ ] 2.0 Interpolation Fidelity Corrections
- [ ] 3.0 Advanced Interpolator Productionization
- [ ] 4.0 Validation & Test Coverage Expansion
- [ ] 5.0 Observability & Instrumentation Surfacing


