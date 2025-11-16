#!/usr/bin/env bash
set -euo pipefail

# Rust coverage helper. Prefers tarpaulin if available, otherwise prints guidance.

if command -v cargo-tarpaulin >/dev/null 2>&1; then
  echo "[COVERAGE] Running cargo tarpaulin (line coverage)..."
  cargo tarpaulin --timeout 120 --out Html --workspace
  echo "[COVERAGE] Report generated in tarpaulin-report/index.html"
  exit 0
fi

echo "[COVERAGE] cargo-tarpaulin not found."
echo "Install with: cargo install cargo-tarpaulin"
echo "Alternatively, use grcov with nightly + llvm-tools:"
echo "  rustup component add llvm-tools-preview"
echo "  cargo build"
echo "  RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test"
echo "  grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o coverage/"
exit 1


