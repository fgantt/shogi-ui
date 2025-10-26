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

## 🔧 Build Commands

```bash
# Build all utilities
cargo build --release

# Build specific utility
cargo build --release --bin usi-engine
cargo build --release --bin tuner
cargo build --release --bin analyzer
```

## 📊 Engine Capabilities

- **Search Depth**: 1-8 levels
- **Hash Size**: 1-1024MB
- **Time Control**: Configurable milliseconds
- **Opening Book**: JSON format with embedded data
- **Endgame Tablebase**: Micro-tablebase support
- **Debug Mode**: Comprehensive logging

## 🎯 Next Utilities (Planned)

1. **Move Quality Assessor** - Analyze game moves for blunders/mistakes
2. **Engine Strength Tester** - Self-play testing and ELO estimation
3. **Tactical Puzzle Generator** - Extract puzzles from games
4. **Game Database Analyzer** - Bulk analysis of game collections
5. **Opening Book Manager** - Convert and manage opening books
6. **Interactive Analysis Mode** - Real-time position analysis

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
