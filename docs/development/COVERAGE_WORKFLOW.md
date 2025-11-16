# Coverage Workflow (Rust + TypeScript)

This guide describes how to run and interpret coverage for the engine (Rust) and UI (TypeScript).

Rust (engine):
- Preferred (simple): `cargo tarpaulin` for line coverage.
  - Install: `cargo install cargo-tarpaulin`
  - Run: `bash scripts/run_rust_coverage.sh`
  - Output: `tarpaulin-report/index.html`
- Alternative (branch coverage): grcov + llvm-tools
  - `rustup component add llvm-tools-preview`
  - `RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test`
  - `grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o coverage/`

TypeScript (UI):
- Vitest coverage:
  - Run: `bash scripts/run_ts_coverage.sh` or `npm run test:coverage`
  - Output: text summary + coverage directory (depending on config)

CI guidance:
- Default CI job: `cargo test` + `npm run test`
- Coverage job (scheduled nightly/weekly):
  - Rust: tarpaulin/grcov job producing HTML artifact and badge metadata
  - TypeScript: Vitest coverage with thresholds enforced
- Surface deltas: Use CI annotations or PR comments to show coverage diffs


