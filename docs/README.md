# Shogi Engine Documentation

This directory contains all documentation for the Shogi game engine, organized by audience and purpose.

## 📁 Directory Structure

### 👥 User Documentation (`user/`)
Documentation intended for end users, developers using the engine, and API consumers.

- **`guides/`** - How-to guides, tutorials, and user manuals
  - `USER_GUIDE.md` - Main user guide for the Shogi game
  - `FAQ.md` - Frequently asked questions
  - `TROUBLESHOOTING_GUIDE.md` - Common issues and solutions
  - `USI-tsshogi-usage.md` - USI engine usage guide
  - `STANDALONE_USI_ENGINE.md` - Standalone engine documentation
  - `OPENING_BOOK_MIGRATION_GUIDE.md` - Opening book migration guide
  - `PERFORMANCE_TUNING_GUIDE.md` - Performance tuning for users
  - `DATA_PREPARATION_GUIDE.md` - Data preparation guide
  - `USI_MONITOR_FEATURE.md` - USI communication monitor feature guide

- **`api/`** - API references and code examples
  - `API_DOCUMENTATION.md` - Complete API reference
  - `CODE_EXAMPLES.md` - Code examples and snippets
  - `OPENING_BOOK_API_REFERENCE.md` - Opening book API reference
  - `OPENING_BOOK_EXAMPLES.md` - Opening book usage examples
  - `OPENING_BOOK_EXAMPLE.md` - Additional opening book examples

- **`reference/`** - Game rules, notation, and reference materials
  - `kifu_notation.md` - Kifu notation reference
  - `move-log-notation.md` - Move log notation
  - `fen_and_coordinates.md` - FEN notation and coordinate system
  - `Universal-Shogi-Interface.html` - USI protocol specification
  - `PROMOTION_MATCHING_EXERCISE.md` - Promotion rules exercise

### 🏗️ Design Documentation (`design/`)
Technical design documents, architecture decisions, and implementation plans.

- **`architecture/`** - System architecture and high-level design
  - `Universal-Shogi-Interface-Implementation.md` - USI implementation details
  - `README_WASM_IMPLEMENTATION.md` - WebAssembly implementation guide
  - `WEBASSEMBLY_BITBOARDS_IMPLEMENTATION.md` - WASM bitboards implementation
  - `WEBASSEMBLY_INTEGRATION_GUIDE.md` - WASM integration guide
  - `JS_vs_WASM_engine.md` - JavaScript vs WASM engine comparison
  - `computer_player_flow_diagram.md` - Computer player architecture
  - `Streaming between UI and Engine - Gemini.md` - UI-Engine communication
  - `INTEGRATION_GUIDE.md` - System integration guide
  - `MIGRATION_GUIDE.md` - Migration guide for system changes
  - `USI_REFACTOR_SUMMARY.md` - Summary of USI implementation refactor

- **`implementation/`** - AI algorithms, optimization strategies, and implementation plans organized by subject
  - **`advanced-king-safety/`** - King safety algorithm design and implementation
    - `DESIGN_ADVANCED_KING_SAFETY.md` - King safety algorithm design
    - `IMPLEMENT_ADVANCED_KING_SAFETY.md` - King safety implementation plan
    - `tasks-DESIGN_ADVANCED_KING_SAFETY.md` - King safety design tasks
  - **`aspiration-windows/`** - Aspiration windows design and implementation
    - `DESIGN_ASPIRATION_WINDOWS.md` - Aspiration windows design
    - `IMPLEMENT_ASPIRATION_WINDOWS.md` - Aspiration windows implementation
    - `tasks-DESIGN_ASPIRATION_WINDOWS.md` - Aspiration windows design tasks
  - **`automated-tuning/`** - Automated tuning design and implementation
    - `DESIGN_AUTOMATED_TUNING.md` - Automated tuning design
    - `IMPLEMENT_AUTOMATED_TUNING.md` - Automated tuning implementation
    - `tasks-DESIGN_AUTOMATED_TUNING.md` - Automated tuning design tasks
    - `TUNING_SYSTEM_README.md` - Tuning system documentation
  - **`endgame-tablebases/`** - Endgame tablebases design and implementation
    - `DESIGN_ENDGAME_TABLEBASES.md` - Endgame tablebases design
    - `IMPLEMENT_ENDGAME_TABLEBASES.md` - Endgame tablebases implementation
    - `tasks-DESIGN_ENDGAME_TABLEBASES.md` - Endgame tablebases design tasks
    - `TABLEBASE_SYSTEM_README.md` - Tablebase system documentation
  - **`internal-iterative-deepening/`** - Internal iterative deepening design and implementation
    - `DESIGN_INTERNAL_ITERATIVE_DEEPENING.md` - Internal iterative deepening design
    - `IMPLEMENT_INTERNAL_ITERATIVE_DEEPENING.md` - Internal iterative deepening implementation
    - `tasks-DESIGN_INTERNAL_ITERATIVE_DEEPENING.md` - Internal iterative deepening design tasks
  - **`late-move-reductions/`** - Late move reductions design and implementation
    - `DESIGN_LATE_MOVE_REDUCTIONS.md` - Late move reductions design
    - `IMPLEMENT_LATE_MOVE_REDUCTIONS.md` - Late move reductions implementation
    - `tasks-DESIGN_LATE_MOVE_REDUCTIONS.md` - Late move reductions design tasks
  - **`null-move-pruning/`** - Null move pruning design and implementation
    - `DESIGN_NULL_MOVE_PRUNING.md` - Null move pruning design
    - `IMPLEMENT_NULL_MOVE_PRUNING.md` - Null move pruning implementation
    - `TASKS_NULL_MOVE_PRUNING.md` - Null move pruning tasks
  - **`opening-book/`** - Opening book design and implementation
    - `IMPLEMENT_OPENING_BOOK.md` - Opening book implementation
    - `tasks-IMPLEMENT_OPENING_BOOK.md` - Opening book implementation tasks
    - `OPENING_BOOK_PERFORMANCE_BENCHMARKS.md` - Opening book benchmarks
    - `OPENING_BOOK_POPULATION_SUMMARY.md` - Opening book population analysis
  - **`optimization-strategies/`** - General optimization strategies and analysis
    - `OPTIMIZATION_STRATEGIES_ANALYSIS.md` - Comprehensive optimization analysis
    - `OPTIMIZATION_EXAMPLES.md` - Optimization examples
  - **`performance-analysis/`** - Performance analysis and benchmarking
    - `AI_ENGINE_ANALYSIS.md` - AI engine analysis
    - `BENCHMARK_RESULTS.md` - Benchmark results
    - `PERFORMANCE_TAPERED_EVALUATION.md` - Tapered evaluation performance
    - `PERFORMANCE_TUNING_GUIDE.md` - Performance tuning guide
  - **`quiescence-search/`** - Quiescence search design and implementation
    - `DESIGN_QUIESCENCE_SEARCH.md` - Quiescence search design
    - `IMPLEMENT_QUIESCENCE_SEARCH.md` - Quiescence search implementation
    - `TASKS_QUIESCENCE_SEARCH.md` - Quiescence search tasks
  - **`simd-optimization/`** - SIMD optimization design and implementation
    - `DESIGN_SIMD.md` - SIMD optimization design
    - `IMPLEMENT_SIMD.md` - SIMD implementation plan
    - `tasks-DESIGN_SIMD.md` - SIMD design tasks
    - `SIMD_OPTIMIZATION_ANALYSIS.md` - SIMD optimization analysis
    - `SIMD_OPTIMIZATION_PLAN.md` - SIMD optimization plan
    - `SIMD_PERFORMANCE_ANALYSIS_REPORT.md` - SIMD performance analysis
    - `SIMD_PERFORMANCE_FINAL_ANALYSIS.md` - Final SIMD performance analysis
  - **`tapered-evaluation/`** - Tapered evaluation design and implementation
    - `DESIGN_TAPERED_EVALUATION.md` - Tapered evaluation design
    - `IMPLEMENT_TAPERED_EVALUATION.md` - Tapered evaluation implementation
    - `TASKS_TAPERED_EVALUATION.md` - Tapered evaluation tasks

### 🔧 Development Documentation (`development/`)
Development processes, project planning, and status tracking.

- **`tasks/`** - Project planning and task management
  - `create-prd.md` - PRD creation process
  - `generate-tasks.md` - Task generation process
  - `opening-book.md` - Opening book development tasks
  - `prd-multi-tier-architecture-refactor.md` - Architecture refactor PRD
  - `prd-shogi-game.md` - Main Shogi game PRD
  - `prd-typescript-conversion.md` - TypeScript conversion PRD
  - `process-task-list.md` - Task list processing
  - `shogi-rules.md` - Shogi rules implementation tasks
  - `tasks-prd-multi-tier-architecture-refactor.md` - Architecture refactor tasks
  - `tasks-prd-shogi-game.md` - Main game development tasks
  - `tasks-prd-typescript-conversion.md` - TypeScript conversion tasks
  - `IMPROVE_MEMORY_EFFICIENCY.md` - Memory efficiency improvements

- **`status/`** - Project status reports
  - `status_20250729.md` - Status report for July 29, 2025
  - `status_20250730.md` - Status report for July 30, 2025
  - `status_20250801.md` - Status report for August 1, 2025
  - `status_20250804.md` - Status report for August 4, 2025
  - `status_20250804_updated.md` - Updated status report for August 4, 2025
  - `status_20250806.md` - Status report for August 6, 2025
  - `status_20250807.md` - Status report for August 7, 2025

- **`processes/`** - Development processes and workflows
  - `generate-tasks.md` - Task generation process

### 📦 Archive (`archive/`)
Deprecated, outdated, or experimental documentation.

- `AI_DROP_IMPROVEMENTS.md` - Legacy AI improvements document

## 🚀 Quick Start

### For End Users
Start with `user/guides/USER_GUIDE.md` for the main user guide, or check `user/guides/FAQ.md` for common questions.

### For Developers
Begin with `design/architecture/README_WASM_IMPLEMENTATION.md` for system architecture, then explore `design/implementation/` for AI implementation details organized by subject area.

### For API Consumers
Check `user/api/API_DOCUMENTATION.md` for complete API reference and `user/api/CODE_EXAMPLES.md` for usage examples.

### For Contributors
Review `development/tasks/` for current development tasks and `development/status/` for project status.

## 📝 Documentation Standards

- **User Documentation**: Written for end users with clear, step-by-step instructions
- **Design Documentation**: Technical documents for developers and architects
- **Development Documentation**: Project management and process documentation
- **Archive**: Clearly marked deprecated or experimental content

## 🔄 Maintenance

This documentation structure should be maintained as the project evolves:
- Move deprecated documents to `archive/`
- Update status reports regularly in `development/status/`
- Keep user guides current with feature changes
- Maintain design documents as architecture evolves
