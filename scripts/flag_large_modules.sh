#!/usr/bin/env bash
set -euo pipefail

# Flag Rust modules exceeding a size threshold (lines of code).
THRESHOLD="${1:-2000}"
echo "[SIZE CHECK] Flagging Rust files with > ${THRESHOLD} lines"
echo

# Exclude target and node_modules by default; rely on repo root execution
git ls-files '*.rs' | while read -r file; do
  lines=$(wc -l < "$file" | tr -d ' ')
  if [ "$lines" -gt "$THRESHOLD" ]; then
    echo "$lines  $file"
  fi
done

echo
echo "[HINT] Consider extracting helpers into submodules and re-exporting stable surfaces."


