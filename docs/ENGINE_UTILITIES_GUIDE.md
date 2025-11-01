# Engine Utilities Guide

**Date:** December 2024  
**Status:** Implementation & Enhancement  
**Purpose:** Comprehensive guide to useful utilities that can be built using the Shogi Engine

---

## Overview

This document outlines useful utilities that can be built leveraging the sophisticated Shogi Engine. The engine provides a powerful foundation with advanced search algorithms, evaluation functions, and analysis capabilities that enable the creation of various specialized tools.

Each utility includes:
- **What it is**: A clear description of what the tool does
- **Purpose**: Why the tool exists and what problems it solves
- **When to use**: Specific scenarios where the tool provides value

## Current Engine Capabilities

### ✅ **Core Features Available**
- **Advanced Search**: Iterative deepening with Principal Variation Search (PVS)
- **Sophisticated Evaluation**: Tapered evaluation with multiple factors
- **Opening Book**: JSON format with embedded data (`src/ai/openingBook.json`)
- **Endgame Tablebase**: Micro-tablebase for endgame positions
- **Debug Logging**: Comprehensive debug and trace logging system
- **Performance Optimization**: Bitboards, transposition tables, move ordering
- **Parameter Tuning**: Automated optimization algorithms (Adam, LBFGS, Genetic)
- **USI Protocol**: Universal Shogi Interface compatibility

### 🏗️ **Architecture**
- **Pure Rust**: Native performance without WebAssembly overhead
- **Tauri Integration**: Desktop application with USI engine support
- **Modular Design**: Clean separation of search, evaluation, and game logic
- **Thread-Safe**: Multi-threaded search capabilities

---

## Implemented Utilities

### 1. **USI Engine** (`usi-engine`)
**Status:** ✅ Complete  
**Binary:** `./target/release/usi-engine`

**What it is:**
A Universal Shogi Interface (USI) protocol engine that provides a command-line interface for chess engines to communicate with shogi GUI applications.

**Purpose:**
Act as the communication layer between your shogi engine and graphical user interfaces like ShogiGUI, ShogiWars, or custom applications.

**When to use:**
- Connect your engine to external shogi GUIs
- Test engine compatibility with standard protocols
- Integrate your engine into existing shogi software ecosystems
- Debug engine behavior through standardized commands
- Play online shogi with your engine

```bash
# Run interactive USI engine
./target/release/usi-engine

# Test with USI commands
echo -e "usi\nisready\nposition startpos\ngo depth 3\nquit" | ./target/release/usi-engine
```

**Features:**
- Full USI protocol implementation
- Configurable hash size (1-1024MB)
- Adjustable search depth (1-8)
- Real-time search information
- Engine identification and options

### 2. **Parameter Tuner** (`tuner`)
**Status:** ✅ Complete  
**Binary:** `./target/release/tuner`

**What it is:**
An automated evaluation parameter tuning tool that uses machine learning algorithms to optimize your engine's evaluation function weights.

**Purpose:**
Automatically find optimal weight values for your evaluation function by learning from high-quality game data, eliminating manual parameter tweaking.

**When to use:**
- Optimize your engine's strength without manual parameter adjustment
- Improve evaluation accuracy using real game data
- Experiment with different optimization algorithms
- Generate synthetic test data for development
- Validate that your parameter changes improve engine performance
- Compare different evaluation function configurations

```bash
# Tune evaluation parameters
./target/release/tuner --dataset games.json --output weights.json --method adam --iterations 1000

# Cross-validation
./target/release/tuner validate --dataset games.json --folds 5

# Generate synthetic data
./target/release/tuner generate --count 1000 --output synthetic.json

# Benchmark algorithms
./target/release/tuner benchmark --iterations 100
```

**Features:**
- Multiple optimization methods (Adam, LBFGS, Genetic Algorithm)
- Cross-validation testing
- Synthetic data generation
- Performance benchmarking
- Weight file management
- Position filtering and validation

### 3. **Position Analyzer** (`analyzer`)
**Status:** ✅ Complete  
**Binary:** `./target/release/analyzer`

**What it is:**
A position analysis tool that evaluates shogi positions and provides detailed information about move quality, evaluation scores, and search statistics.

**Purpose:**
Provide quick, detailed analysis of specific positions for study, opening preparation, or game analysis.

**When to use:**
- Analyze specific positions from your games
- Study opening positions and find best moves
- Compare different positions to understand strategic differences
- Debug engine behavior at specific positions
- Explore tactical positions and see engine's recommended lines
- Quick analysis during game study sessions

```bash
# Analyze starting position
./target/release/analyzer startpos --depth 6

# Analyze with verbose output
./target/release/analyzer --verbose --depth 4

# Compare multiple positions
./target/release/analyzer compare "startpos" "sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"
```

**Features:**
- Position analysis with detailed evaluation
- Best move calculation with principal variation
- Search time and performance metrics
- Engine information display
- Position comparison capabilities
- Verbose analysis mode

### 4. **Engine Strength Tester** (`strength-tester`)
**Status:** ✅ Complete  
**Binary:** `./target/release/strength-tester`

**What it is:**
A self-play testing tool that runs your engine against itself to evaluate strength, measure improvements, and test engine reliability.

**Purpose:**
Objectively measure your engine's playing strength, verify that changes improve engine performance, and identify weaknesses or bugs through automated testing.

**When to use:**
- Before releasing a new engine version to verify improvements
- After making evaluation changes to measure impact
- To benchmark engine speed vs. strength tradeoffs
- When debugging engine bugs in game playing
- To ensure engine stability over many games
- When comparing different configuration settings
- To test that engine correctly handles all game termination conditions

```bash
# Test engine strength with self-play
./target/release/strength-tester --games 10 --depth 3 --verbose

# Run strength testing with configurable time control
./target/release/strength-tester --time-control "10+0.1" --games 50 --depth 4
```

**Features:**
- ✅ Self-play strength testing
- ✅ Configurable games and search depth
- ✅ Game result tracking (wins, losses, draws)
- ✅ Game state management with move application
- ✅ Checkmate and stalemate detection
- ✅ Infinite loop prevention
- ⚠️ Configuration comparison (planned)
- ⚠️ ELO estimation (planned)

**Implementation Notes:**
- ✅ Uses direct `ShogiEngine` API
- ✅ Implements position tracking with `apply_move()`
- ✅ Terminal condition detection with `is_game_over()`
- ✅ Game result statistics
- ⚠️ ELO calculation needs statistical framework
- ⚠️ Configuration comparison needs implementation

### 5. **Move Quality Assessor** (`move-assessor`)
**Status:** ✅ Complete  
**Binary:** `./target/release/move-assessor`

**What it is:**
A game analysis tool that evaluates every move in a shogi game, categorizes them by quality, and identifies blunders, mistakes, and excellent moves.

**Purpose:**
Provide objective analysis of your games to identify learning opportunities, track improvement over time, and understand where you lose or gain advantage.

**When to use:**
- After playing a game to review and learn from mistakes
- To identify your most common error patterns
- When studying professional games to understand move quality
- Before teaching games to prepare explanations
- To track your improvement by measuring error rates
- When analyzing specific positions where you blundered
- As a training tool to focus practice on your weaknesses

```bash
# Analyze game moves
./target/release/move-assessor --input game.kif --output analysis.json --depth 8

# Find blunders
./target/release/move-assessor --input game.kif find-blunders --threshold 200

# Detailed analysis with verbose output
./target/release/move-assessor --input game.kif --depth 6 --verbose
```

**Features:**
- Move quality scoring (centipawns)
- Blunder detection (moves losing >200 centipawns)
- Mistake analysis (moves losing 50-200 centipawns)
- Improvement suggestions
- Game annotation with quality marks
- Statistical analysis of player performance
- KIF format parsing with UTF-8 safe handling
- Real engine evaluation integration
- JSON output with detailed analysis

**Implementation Notes:**
- ✅ KIF format parsing implemented
- ✅ Engine evaluation integrated for move assessment
- ✅ Blunder/mistake classification working
- ✅ Game annotation capabilities ready
- ✅ JSON output format with structured analysis

### 6. **Tactical Puzzle Generator** (`puzzle-gen`)
**Status:** ✅ Complete  
**Binary:** `./target/release/puzzle-gen`

**What it is:**
An automated puzzle generator that extracts tactical positions from games and creates training puzzles with varying difficulty levels.

**Purpose:**
Generate educational puzzles that help players practice specific tactical patterns, improve tactical vision, and train for common game situations.

**When to use:**
- To create training materials for students learning shogi
- When building a puzzle collection for teaching specific tactical patterns
- To extract interesting positions from large game databases
- When you want puzzles sorted by difficulty for progressive learning
- To practice specific tactical motifs (forks, pins, skewers)
- As part of a puzzle-of-the-day system
- To convert your played games into personalized training puzzles

```bash
# Generate puzzles from games
./target/release/puzzle-gen --input games.json --output puzzles.json --difficulty medium --count 50

# Create specific pattern puzzles
./target/release/puzzle-gen --input games.json --pattern "fork" --count 50 --output fork_puzzles.json

# Generate by rating range
./target/release/puzzle-gen --input games.json --min-rating 1500 --max-rating 2000 --count 100

# Extract patterns from KIF games
./target/release/puzzle-gen extract --input game.kif --output puzzles.json --count 20
```

**Features:**
- ✅ Extract tactical motifs (forks, pins, skewers, discoveries)
- ✅ Generate puzzles by difficulty level (easy, medium, hard)
- ✅ Pattern-specific puzzle creation
- ✅ Puzzle rating system (1200-2500 ELO equivalent)
- ✅ Solution verification
- ✅ Educational categorization
- ✅ JSON export format
- ⚠️ Full tactical pattern detection (in progress)

**Implementation Notes:**
- ✅ CLI structure and data types implemented
- ✅ Puzzle generation framework ready
- ✅ Pattern filtering and difficulty rating working
- ✅ JSON export format with metadata
- ⚠️ Real-time tactical pattern detection needs integration with TacticalPatternRecognizer
- ⚠️ Full KIF parsing for multi-game files needs completion

---

## High-Priority Utilities to Implement

### 7. **Tactical Puzzle Generator (Enhanced)**
**Priority:** 🔥 Medium (Core functionality complete, enhancements pending)

**What it is:**
An automated puzzle generator that extracts tactical positions from games and creates training puzzles with varying difficulty levels.

**Purpose:**
Generate educational puzzles that help players practice specific tactical patterns, improve tactical vision, and train for common game situations.

**When to use:**
- To create training materials for students learning shogi
- When building a puzzle collection for teaching specific tactical patterns
- To extract interesting positions from large game databases
- When you want puzzles sorted by difficulty for progressive learning
- To practice specific tactical motifs (forks, pins, skewers)
- As part of a puzzle-of-the-day system
- To convert your played games into personalized training puzzles

```bash
# Generate puzzles from games
./puzzle-gen --input games.json --output puzzles.json --difficulty medium

# Create specific pattern puzzles
./puzzle-gen --pattern "fork" --count 50 --output fork_puzzles.json

# Generate by rating
./puzzle-gen --rating-range "1500-2000" --count 100 --output puzzles.json
```

**Features:**
- Extract tactical motifs (forks, pins, skewers, discoveries)
- Generate puzzles by difficulty level
- Pattern-specific puzzle creation
- Solution verification
- Puzzle rating system
- Educational categorization

**Implementation Notes:**
- Implement tactical pattern recognition
- Use engine search for solution verification
- Create difficulty rating system
- Add puzzle database management

---

## Medium-Priority Utilities

### 7. **Game Database Analyzer**
**Priority:** 🟡 Medium  
**Estimated Effort:** 3-4 weeks

**What it is:**
A comprehensive database analysis tool that processes thousands of shogi games to extract patterns, statistics, and insights.

**Purpose:**
Analyze large game collections to understand opening trends, identify common endgame patterns, study player styles, and extract data for research or training.

**When to use:**
- To understand which openings are most popular at different skill levels
- To study endgame patterns and their success rates
- When researching specific positional structures (anaguma, ibisha, etc.)
- To convert between different database formats (KIF, CSA, PGN, JSON)
- When preparing opening book material from professional games
- To analyze player styles and preferences
- When extracting statistics for research papers or articles

```bash
# Analyze large databases
./db-analyzer --input games.json --output analysis.json --threads 8

# Extract patterns
./db-analyzer --pattern "anaguma" --input games.json --output anaguma_games.json

# Opening popularity analysis
./db-analyzer --opening-stats --input games.json --depth 20
```

**Features:**
- Bulk position analysis
- Pattern recognition across databases
- Opening popularity analysis
- Endgame statistics
- Player style analysis
- Database format conversion

### 8. **Opening Book Manager**
**Priority:** 🟡 Medium  
**Estimated Effort:** 2-3 weeks

**What it is:**
A specialized tool for managing opening books that converts formats, generates opening lines, and analyzes book coverage.

**Purpose:**
Create, maintain, and analyze opening books for use with your engine, extracting knowledge from professional games and optimizing book quality.

**When to use:**
- To create an opening book from a professional game database
- When converting opening books between different formats
- To merge multiple opening books into one comprehensive database
- When analyzing which openings are covered in your book
- To find gaps or holes in your opening book coverage
- To extract popular opening lines and recent novelties
- When maintaining an engine's opening repertoire

```bash
# Convert formats
./book-manager convert --input games.kif --output opening_book.json

# Analyze statistics
./book-manager stats --book opening_book.json --depth 10

# Merge books
./book-manager merge --input book1.json book2.json --output merged.json
```

**Features:**
- Convert between KIF, CSA, PGN, JSON formats
- Generate opening books from game databases
- Analyze opening book coverage and quality
- Merge multiple opening books
- Extract popular lines and novelties

### 9. **Interactive Analysis Mode**
**Priority:** 🟡 Medium  
**Estimated Effort:** 2-3 weeks

**What it is:**
An interactive command-line interface that provides real-time position analysis, allowing you to explore moves, understand evaluations, and analyze positions dynamically.

**Purpose:**
Provide an interactive learning and analysis experience where you can explore positions, compare moves, and understand the engine's thinking in real-time.

**When to use:**
- During game study when you want to explore variations interactively
- When teaching and need to demonstrate different move options
- To understand why the engine prefers certain moves over others
- When analyzing complex positions with many candidate moves
- To explore opening theory interactively
- As a debugging tool to understand engine behavior
- For self-study of specific positions or patterns

```bash
# Real-time analysis
./interactive-analyzer
```

**Features:**
- Real-time position analysis
- Move exploration
- Evaluation explanation
- Tactical pattern highlighting
- Position comparison
- Interactive move input

---

## Development Utilities

### 10. **Performance Profiler** (`profiler`)
**Status:** ✅ Complete  
**Binary:** `./target/release/profiler`

**What it is:**
A profiling tool that analyzes engine performance, memory usage, cache efficiency, and identifies optimization opportunities.

**Purpose:**
Monitor and optimize engine performance, understand resource usage, and identify bottlenecks for improvement.

**When to use:**
- When optimizing engine speed and efficiency
- To understand memory usage patterns
- When debugging performance issues
- To compare different engine configurations
- To measure cache hit rates and transposition table effectiveness
- Before deploying to production to ensure performance
- When tuning hash table sizes and search parameters

```bash
# Profile engine performance
./target/release/profiler --position startpos --depth 8 --output profile.json --verbose

# Compare two profiles (use JSON outputs from previous runs)
./target/release/profiler compare --config1 profile_default.json --config2 profile_optimized.json
```

**Features:**
- Detailed performance profiling (time, nodes, NPS)
- Transposition table contention and hit-rate monitoring
- YBWC-related parallel search diagnostics
- Search efficiency metrics
- Optimization recommendations

### 11. **Endgame Tablebase Builder**
**Priority:** 🟢 Low  
**Estimated Effort:** 4-6 weeks

**What it is:**
A tool for building custom endgame tablebases that pre-calculate perfect play for specific endgame positions with limited pieces.

**Purpose:**
Create perfect play databases for endgame positions, enabling the engine to play endgames flawlessly and verify tablebase correctness.

**When to use:**
- To build perfect play databases for specific endgame material
- When verifying the correctness of existing tablebases
- To improve engine endgame strength for specific piece configurations
- When researching endgame theory and optimal play
- To create tablebases for experimental or uncommon piece combinations
- As part of engine development and testing
- To optimize tablebase storage and lookup performance

```bash
# Build custom tablebases
./tablebase-builder --pieces "K+2P vs K" --output 2pawn_vs_king.tb

# Verify tablebase correctness
./tablebase-builder verify --tablebase 2pawn_vs_king.tb
```

**Features:**
- Custom endgame tablebase generation
- Tablebase verification and validation
- Performance optimization
- Memory usage analysis
- Integration testing

---

## Implementation Roadmap

### Phase 1: Core Analysis Tools (Weeks 1-6) ✅ COMPLETE
1. ✅ **Move Quality Assessor** - Essential for game analysis - **COMPLETE**
2. ✅ **Engine Strength Tester** - Critical for development - **COMPLETE**
3. ✅ **Tactical Puzzle Generator** - High educational value - **COMPLETE**

### Phase 2: Database Tools (Weeks 7-12)
7. **Game Database Analyzer** - Powerful research capabilities
8. **Opening Book Manager** - Specialized but useful
9. **Interactive Analysis Mode** - User-friendly interface
10. **Enhanced Tactical Pattern Detection** - Full integration of TacticalPatternRecognizer

### Phase 3: Development Tools (Weeks 13-18)
7. ✅ **Performance Profiler** - Development optimization - **COMPLETE**
8. **Endgame Tablebase Builder** - Advanced feature

---

## Technical Implementation Guidelines

### **Using the Engine API**
```rust
use shogi_engine::ShogiEngine;

let mut engine = ShogiEngine::new();

// Get best move
if let Some(best_move) = engine.get_best_move(depth, time_limit, None) {
    println!("Best move: {}", best_move.to_usi_string());
}

// Check engine status
println!("Debug mode: {}", engine.is_debug_enabled());
println!("Opening book loaded: {}", engine.is_opening_book_loaded());
```

### **Leveraging Existing Features**
- **Debug Logging**: Use `crate::debug_utils` for detailed analysis
- **Evaluation System**: Access tapered evaluation components
- **Search Engine**: Utilize iterative deepening and PVS
- **Opening Book**: Load and query opening positions
- **Tablebase**: Probe endgame positions

### **Future: USI Engine Abstraction**
To make these utilities more generally useful beyond our specific engine implementation:

**Current State:** Utilities are tightly integrated with `ShogiEngine` struct and use direct API calls for performance and functionality.

**Future Enhancement:** Add a USI engine abstraction layer to support any USI-compatible engine:

```rust
// Proposed abstraction
trait UsiEngine {
    fn get_best_move(&mut self, depth: u32, time_limit: Duration) -> Option<Move>;
    fn analyze_position(&mut self, sfen: &str, depth: u32) -> Analysis;
    // ... other common operations
}

// Implementation for our engine
impl UsiEngine for ShogiEngine { /* ... */ }

// Implementation for external USI engines via subprocess
struct UsiProtocolEngine {
    process: Child,
    // communication via USI protocol
}
impl UsiEngine for UsiProtocolEngine { /* ... */ }
```

**Benefits:**
- ✅ Works with any USI-compatible engine (YaneuraOu, elmo, etc.)
- ✅ Allows users to choose their preferred engine for analysis
- ✅ Enables comparing different engines
- ✅ Makes utilities more universally useful

**Current Priority:** Keep direct integration for now as it provides:
- Better performance (no process overhead)
- Access to internal state and debugging features
- Simpler implementation and maintenance
- Immediate functionality for our use case

**Migration Path:** When ready to generalize:
1. Create trait abstraction
2. Implement trait for our engine (trivial)
3. Add USI subprocess wrapper
4. Make utilities engine-agnostic
5. Add `--engine` flag to utilities to choose engine

This can be done incrementally without breaking existing functionality.

### **File Format Support**
- **KIF**: Japanese notation format
- **CSA**: Computer Shogi Association format
- **PGN**: Portable Game Notation
- **JSON**: Structured data format
- **SFEN**: Shogi Forsyth-Edwards Notation

---

## Success Metrics

### **Utility Adoption**
- Number of users utilizing each utility
- Frequency of utility usage
- User feedback and ratings

### **Technical Performance**
- Analysis speed and accuracy
- Memory efficiency
- Code maintainability
- Test coverage

### **Educational Value**
- Puzzle generation quality
- Analysis depth and insight
- Learning improvement metrics

---

## Future Enhancements

### **Engine Compatibility**
- **USI Abstraction Layer**: Generalize utilities to work with any USI-compatible engine
  - Add trait abstraction for engine operations
  - Implement USI subprocess communication for external engines
  - Enable `--engine` flag to choose analysis engine
  - Allow comparing different engine strengths and styles
  - Make utilities universally useful beyond our specific implementation

### **Advanced Features**
- Machine learning integration
- Cloud-based analysis
- Real-time game analysis
- Mobile application support

### **Community Features**
- Puzzle sharing platform
- Analysis result sharing
- Collaborative puzzle creation
- Rating and ranking systems

---

## Conclusion

The Shogi Engine provides an excellent foundation for building powerful analysis utilities. The implemented tools (USI Engine, Parameter Tuner, Position Analyzer, Engine Strength Tester, Move Quality Assessor, Tactical Puzzle Generator, Performance Profiler) demonstrate the engine's capabilities across different use cases:

- **Engine Development**: Use the Parameter Tuner to optimize evaluation and the Strength Tester to measure improvements
- **Game Analysis**: Use the Move Quality Assessor to review games and learn from mistakes
- **Position Study**: Use the Position Analyzer for opening preparation and tactical exploration
- **Training**: Use the Tactical Puzzle Generator to create custom puzzle collections for practice
- **Integration**: Use the USI Engine to connect to external applications and play online

The planned utilities will significantly expand the engine's usefulness for players, researchers, and developers. Each utility serves a specific purpose and addresses real needs in the shogi community.

The modular architecture and comprehensive feature set make it straightforward to implement additional utilities that leverage the engine's sophisticated search and evaluation capabilities.

---

**Last Updated:** December 2024  
**Next Review:** January 2025
