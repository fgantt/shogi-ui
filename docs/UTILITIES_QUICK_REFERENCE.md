# Engine Utilities Quick Reference

## 🚀 Available Utilities

### **USI Engine** (`usi-engine`)
```bash
# Run interactive engine
./target/release/usi-engine

# Quick test
echo "usi" | ./target/release/usi-engine

# Full analysis
echo -e "usi\nisready\nposition startpos\ngo depth 3\nquit" | ./target/release/usi-engine
```

### **Parameter Tuner** (`tuner`)
```bash
# Basic tuning
./target/release/tuner --dataset games.json --output weights.json --method adam

# Cross-validation
./target/release/tuner validate --dataset games.json --folds 5

# Generate test data
./target/release/tuner generate --count 1000 --output synthetic.json

# Benchmark algorithms
./target/release/tuner benchmark --iterations 100
```

### **Position Analyzer** (`analyzer`)
```bash
# Analyze starting position
./target/release/analyzer startpos --depth 6

# Verbose analysis
./target/release/analyzer --verbose --depth 4

# Compare positions
./target/release/analyzer compare "startpos" "sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"
```

### **Engine Strength Tester** (`strength-tester`)
```bash
# Test engine with self-play
./target/release/strength-tester --games 10 --depth 3 --verbose

# Run multiple games
./target/release/strength-tester --games 50 --depth 4

# Compare configurations
./target/release/strength-tester compare --config1 config1.json --config2 config2.json
```

### **Move Quality Assessor** (`move-assessor`)
```bash
# Analyze game moves
./target/release/move-assessor --input game.kif --depth 6 --output analysis.json

# Find blunders
./target/release/move-assessor --input game.kif find-blunders --threshold 200

# Verbose output
./target/release/move-assessor --input game.kif --depth 4 --verbose
```

### **Tactical Puzzle Generator** (`puzzle-gen`)
```bash
# Generate puzzles from games
./target/release/puzzle-gen --input games.json --output puzzles.json --count 50

# Create specific pattern puzzles
./target/release/puzzle-gen --input games.json --pattern "fork" --count 50

# Extract by difficulty
./target/release/puzzle-gen --input games.json --difficulty "medium" --count 100

# Extract from KIF games
./target/release/puzzle-gen extract --input game.kif --output puzzles.json --count 20
```

### **Performance Profiler** (`profiler`)
```bash
# Profile engine performance and save JSON report
./target/release/profiler --position startpos --depth 8 --output profile.json --verbose

# Compare two saved profiles
./target/release/profiler compare --config1 profile_default.json --config2 profile_optimized.json
```

## 🔧 Build Commands

```bash
# Build all utilities
cargo build --release

# Build specific utility
cargo build --release --bin usi-engine
cargo build --release --bin tuner
cargo build --release --bin analyzer
cargo build --release --bin strength-tester
cargo build --release --bin move-assessor
cargo build --release --bin puzzle-gen
cargo build --release --bin profiler
```

## 📊 Engine Capabilities

- **Search Depth**: 1-8 levels
- **Hash Size**: 1-1024MB
- **Time Control**: Configurable milliseconds
- **Opening Book**: JSON format with embedded data
- **Endgame Tablebase**: Micro-tablebase support
- **Debug Mode**: Comprehensive logging

## 🎯 Next Utilities (Planned)

1. **Game Database Analyzer** - Bulk analysis of game collections
2. **Opening Book Manager** - Convert and manage opening books
3. **Interactive Analysis Mode** - Real-time position analysis
4. **Enhanced Tactical Pattern Detection** - Full integration improvements

## 📁 File Locations

- **Binaries**: `./target/release/`
- **Source Code**: `src/bin/`
- **Documentation**: `docs/ENGINE_UTILITIES_GUIDE.md`
- **Opening Book**: `src/ai/openingBook.json`
- **Examples**: `examples/`

## 🆘 Troubleshooting

### Common Issues
- **Permission Denied**: Run `chmod +x ./target/release/*`
- **Missing Dependencies**: Run `cargo build --release`
- **Memory Issues**: Reduce hash size or search depth
- **Slow Performance**: Increase hash size or reduce depth

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/analyzer --verbose --depth 3
```

---

**Quick Reference** | **Last Updated**: December 2024
