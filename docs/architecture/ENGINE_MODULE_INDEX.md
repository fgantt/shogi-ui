# Engine Module Index

This index provides a high-level map of the engine modules and their responsibilities. Each entry links to the Rust module or namespace and describes the public surface and key integration points.

- evaluation (namespace)
  - extractors: Feature extraction components (position, tactical, endgame, opening, castles, PST, loaders, king safety, attacks, patterns).
  - aggregators: Scoring/integration components (tapered eval, phase transition, integration, interpolation, caches, stats, performance, weights).
- search: Core search engine, time management, aspiration windows, pruning, transposition tables, parallel search (YBWC).
- bitboards: Board representation and bitboard operations, attack generation, magic bitboards and tables.
- tablebase: Micro tablebase, position cache, tablebase configuration and profiling.
- opening_book: Opening book loader/builder, indexing and streaming.
- types: Core domain types (`Player`, `PieceType`, `Position`, `Move`, `CapturedPieces`), evaluation/search config and stats.
- utils: Common helpers, telemetry (debug/trace), time utilities, small shared helpers.
- usi: Integration with USI protocol, controller.

Cross-language mapping:
- See `docs/development/CROSS_LANGUAGE_API_MAPPING.md` for Rust ↔ TypeScript type mappings and conventions.


