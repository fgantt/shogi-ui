# Cross-Language API Mapping (Rust ↔ TypeScript)

This document maps core Rust engine types and structures to their TypeScript counterparts used by the UI and Tauri bridge. It also outlines serialization conventions.

Core domain:
- Rust `types::core::Player` ↔ TS string unions where relevant (`'player1' | 'player2'` in UI state)
- Rust `types::core::Position` ↔ USI-like strings (e.g., `"7f"`) or JSON `{ row, col }`
- Rust `types::core::Move` ↔ USI strings (e.g., `"7g7f"`) or structured `{ from, to, promote }`
- Rust `types::board::CapturedPieces` ↔ TS `{ [piece: string]: number }`

Evaluation/Search configurations:
- Rust config types (`...Config`) map to persisted settings in UI; stats (`...Stats`) are runtime-only telemetry
- Time management: Rust `types::search::TimeManagementConfig` ↔ TS `GameSettings.minutesPerSide`, `byoyomiInSeconds`

Telemetry and logging:
- Rust telemetry via `crate::utils::telemetry` emits strings consumed in UI logs; TS uses lightweight string processing

Serialization conventions:
- Prefer USI-compatible strings for moves/positions when interoperating with UI
- For complex payloads, use JSON objects with explicit keys; avoid positional tuples

References:
- Rust: `src/types/core.rs`, `src/types/search.rs`, `src/evaluation/**`
- TypeScript: `src/types.ts`, `src/types/engine.ts`


