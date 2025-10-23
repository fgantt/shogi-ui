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
  - `SHOGI_ENDGAME_CONDITIONS.md` - Complete guide to all endgame conditions

### 🔧 API Documentation (`api/`)
Technical API references and developer documentation for engine components.

- **Bit-scanning API** - High-performance bit manipulation operations
  - `bit-scanning-api.md` - Core bit-scanning API documentation
  - `bit-scanning-api-reference.md` - Complete API reference
  - `bit-scanning-migration-guide.md` - Migration guide for bit-scanning updates
  - `bit-scanning-performance-guide.md` - Performance optimization guide

- **Move Ordering API** - Move prioritization and search optimization
  - `MOVE_ORDERING_API.md` - Complete move ordering API reference
  - `MOVE_ORDERING_BEST_PRACTICES.md` - Best practices for move ordering
  - `MOVE_ORDERING_PERFORMANCE_GUIDE.md` - Performance tuning guide
  - `MOVE_ORDERING_TROUBLESHOOTING.md` - Common issues and solutions

- **Evaluation Cache API** - Position evaluation caching system
  - `EVALUATION_CACHE_API.md` - Complete evaluation cache API
  - `EVALUATION_CACHE_BEST_PRACTICES.md` - Best practices for cache usage
  - `EVALUATION_CACHE_EXAMPLES.md` - Usage examples and code snippets
  - `EVALUATION_CACHE_TROUBLESHOOTING.md` - Common issues and solutions
  - `EVALUATION_CACHE_TUNING_GUIDE.md` - Performance tuning guide
  - `EVALUATION_CACHE_ADVANCED_INTEGRATION.md` - Advanced integration patterns

- **Transposition Tables** - Position caching and lookup systems
  - `TRANSPOSITION_TABLE_API_REFERENCE.md` - Transposition table API reference
  - `TR posITION_TABLE_API_REFERENCE.md` - Position table API reference

### 🚀 Release Management (`release/`)
Documentation for creating and managing releases.

- `RELEASE_CHECKLIST.md` - Step-by-step release process checklist

### 📦 Distribution (`distribution/`)
Documentation for packaging and distributing the application.

- `DISTRIBUTION_GUIDE.md` - Complete packaging and distribution guide
- `DISTRIBUTION_INDEX.md` - Distribution documentation index and navigation
- `PACKAGING_QUICK_START.md` - Fast track to creating your first release
- `WHERE_ARE_MY_INSTALLERS.md` - Guide to finding built installer files

### ⚙️ Implementation (`implementation/`)
Documentation for specific implementation features and changes.

- `THEME_CHANGES_SUMMARY.md` - Summary of theme system changes
- `THEME_IMPLEMENTATION_SUMMARY.md` - Detailed theme system implementation

### 🧹 Cleanup (`cleanup/`)
Documentation for codebase cleanup and maintenance tasks.

- `CLEANUP_PLAN.md` - WASM to Tauri migration cleanup plan

### 🏗️ Design Documentation (`design/`)
Technical design documents, architecture decisions, and implementation plans.

- **`architecture/`** - System architecture and high-level design
  - `Universal-Shogi-Interface-Implementation.md` - USI implementation details
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
  - **`endgame-detection/`** - Endgame detection and game-over conditions
    - `ENDGAME_DETECTION_IMPLEMENTATION_PLAN.md` - Complete implementation plan
    - `ENDGAME_DETECTION_TASKS.md` - Actionable task breakdown

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

- **`bug-fixes/`** - Bug fix documentation and analysis
  - `BUG_FIX_INFINITE_SEARCH_LOOP.md` - Fix for AI infinite search when checkmated

### 📦 Archive (`archive/`)
Deprecated, outdated, or experimental documentation.

- `AI_DROP_IMPROVEMENTS.md` - Legacy AI improvements document

## 🚀 Quick Start

### For End Users
Start with `user/guides/USER_GUIDE.md` for the main user guide, or check `user/guides/FAQ.md` for common questions.

### For Developers
Begin with `design/architecture/` for system architecture, then explore `design/implementation/` for AI implementation details organized by subject area.

### For API Consumers
Check `api/` for complete API references including:
- `api/bit-scanning-api.md` - Bit manipulation operations
- `api/MOVE_ORDERING_API.md` - Move ordering system
- `api/EVALUATION_CACHE_API.md` - Position evaluation caching
- `user/api/API_DOCUMENTATION.md` - Complete API reference
- `user/api/CODE_EXAMPLES.md` - Usage examples

### For Release Management
- `release/RELEASE_CHECKLIST.md` - Step-by-step release process
- `distribution/PACKAGING_QUICK_START.md` - Fast track to first release
- `distribution/WHERE_ARE_MY_INSTALLERS.md` - Finding built files

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
