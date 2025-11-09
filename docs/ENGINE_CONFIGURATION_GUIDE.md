# Engine Configuration Guide

This document describes the configurable features of the built-in Shogi engine and how to configure them.

## Overview

The built-in engine has **many configurable features** but currently only exposes **2 options** (hash and depth) to users. This guide shows what's available and how to configure it.

## Currently Exposed Options (Updated)

The following options can now be configured via USI protocol:

### 1. **USI_Hash**
- **Description**: Transposition table size in megabytes
- **Type**: spin (integer)
- **Default**: 64 MB
- **Range**: 1-1024 MB
- **Usage**: `setoption name USI_Hash value 128`

### 2. **depth**
- **Description**: Search depth (default maximum depth for the search)
- **Type**: spin (integer)
- **Default**: 5
- **Range**: 1-50
- **Usage**: `setoption name depth value 8`

### 3. **QuiescenceDepth**
- **Description**: Maximum depth for quiescence search (capture-only search)
- **Type**: spin (integer)
- **Default**: 6
- **Range**: 1-10
- **Usage**: `setoption name QuiescenceDepth value 8`

### 4. **EnableQuiescence**
- **Description**: Enable or disable quiescence search
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableQuiescence value true` or `value false`

### 5. **EnableNullMove**
- **Description**: Enable or disable null-move pruning (pruning optimization)
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableNullMove value true`

### 6. **NullMoveMinDepth**
- **Description**: Minimum depth before null-move pruning is applied
- **Type**: spin (integer)
- **Default**: 3
- **Range**: 1-10
- **Usage**: `setoption name NullMoveMinDepth value 4`

### 7. **EnableLMR**
- **Description**: Enable or disable Late Move Reduction (search optimization)
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableLMR value true`

### 8. **EnableIID**
- **Description**: Enable or disable Internal Iterative Deepening (search optimization)
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableIID value true`

### 9. **EnableAspirationWindows**
- **Description**: Enable or disable aspiration windows (search optimization)
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableAspirationWindows value true`

### 10. **EnableTablebase**
- **Description**: Enable or disable endgame tablebase lookups
- **Type**: check (boolean)
- **Default**: true
- **Usage**: `setoption name EnableTablebase value true`

## Additional Configuration Options (Not Yet Exposed)

The following advanced features exist internally but are **not yet exposed** to users:

### Search Algorithms

#### Quiescence Search (`QuiescenceConfig`)
- `max_depth`: Maximum depth for quiescence search (Default: 6)
- `enable_delta_pruning`: Enable delta pruning in quiescence (Default: true)
- `enable_futility_pruning`: Enable futility pruning (Default: true)
- `enable_selective_extensions`: Enable selective depth extensions (Default: true)
- `enable_tt`: Enable transposition table in quiescence (Default: true)
- `futility_margin`: Futility pruning margin (Default: 200)
- `delta_margin`: Delta pruning margin (Default: 200)
- `tt_size_mb`: Transposition table size for quiescence (Default: 32 MB)
- `tt_cleanup_threshold`: Number of entries before cleanup (Default: 100000)

#### Null-Move Pruning (`NullMoveConfig`)
- `reduction_factor`: Depth reduction factor (Default: 2)
- `max_pieces_threshold`: Maximum pieces on board (Default: 8)
- `enable_dynamic_reduction`: Dynamic depth reduction (Default: true)
- `enable_endgame_detection`: Detect endgame positions (Default: true)

#### Late Move Reduction (`LMRConfig`)
- `min_depth`: Minimum depth for LMR (Default: 3)
- `min_move_index`: Minimum move index before reduction (Default: 4)
- `base_reduction`: Base depth reduction (Default: 1)
- `max_reduction`: Maximum depth reduction (Default: 3)
- `enable_dynamic_reduction`: Dynamic reduction (Default: true)
- `enable_adaptive_reduction`: Adaptive LMR (Default: true)
- `enable_extended_exemptions`: Extended move exemptions (Default: true)

#### Internal Iterative Deepening (`IIDConfig`)
- `min_depth`: Minimum depth for IID (Default: 3)
- `iid_depth_ply`: Depth reduction for IID (Default: 2)
- `max_legal_moves`: Maximum legal moves to try IID (Default: 40)
- `time_overhead_threshold`: Time overhead threshold (Default: 0.20)
- `enable_time_pressure_detection`: Detect time pressure (Default: true)
- `enable_adaptive_tuning`: Enable adaptive configuration (Default: true)

#### Aspiration Windows (`AspirationWindowConfig`)
- `base_window_size`: Initial window size (Default: 25)
- `dynamic_scaling`: Enable dynamic scaling (Default: true)
- `max_window_size`: Maximum window size (Default: 150)
- `min_depth`: Minimum depth for aspiration (Default: 2)
- `enable_adaptive_sizing`: Adaptive window sizing (Default: true)
- `max_researches`: Maximum research attempts (Default: 2)
- `enable_statistics`: Track statistics (Default: false)

### Time Management (`TimeManagementConfig`)
- `time_per_move_ms`: Time budget per move in milliseconds
- `increment_ms`: Time increment per move
- `max_time_ms`: Maximum time per search
- `time_control`: Type of time control (fixed, increment, custom)

### Evaluation Configuration

#### Material Evaluation
- `include_hand_pieces`: Include captured pieces in material count (Default: true)
- `use_research_values`: Selects the material value preset. When `true` (default) the engine loads the research-tuned tables; when `false` it falls back to the classic legacy tables. Both presets include board and hand-piece tapered values and can be switched at runtime.
- `values_path`: Optional filesystem path (JSON or TOML) pointing to a custom material value set. When provided, it overrides `use_research_values`. Paths are resolved relative to the engine working directory unless absolute; failures fall back to the preset indicated by `use_research_values` and emit a debug log.
- `enable_fast_loop`: Experimental fast path that counts bitboards directly rather than scanning 81 squares. Disabled by default; enable after validating parity via the cross-check test (`cargo test --features material_fast_loop material_delta`).

**Migration Notes**

- Releases prior to 11.0 hard-coded material tables. The new presets (`research`, `classic`) preserve the previous behaviour; no config changes are required to maintain default output.
- External tooling that previously patched source files should use `values_path` instead.
- When upgrading existing configs, add `enable_fast_loop = false` explicitly if deterministic parity is required before running the new regression suites.

**Troubleshooting**

- Missing or unreadable `values_path` files trigger a warning in the debug log and fall back to the preset specified by `use_research_values`.
- Mismatched table lengths (fewer than 14 piece entries) surface as `MaterialValueSetError::Validation` during startup.
- To compare presets or custom tables, enable telemetry (`integrated_evaluator.enable_statistics()`) and inspect the material snapshot dump in debug logs.

#### Piece-Square Tables
- Piece-square table weights for all piece types
- Middle game vs endgame table values
- Configurable table values for different positions

#### Position Features (`PositionFeatureConfig`)
- `enable_king_safety`: King safety evaluation (Default: true)
- `enable_pawn_structure`: Pawn structure evaluation (Default: true)
- `enable_mobility`: Piece mobility evaluation (Default: true)
- `enable_center_control`: Center control evaluation (Default: true)
- `enable_development`: Development evaluation (Default: true)

#### Evaluation Weights (`EvaluationWeights`)
- `material_weight`: Weight for material (Default: 1.0)
- `position_weight`: Weight for position (Default: 1.0)
- `king_safety_weight`: Weight for king safety (Default: 1.0)
- `pawn_structure_weight`: Weight for pawn structure (Default: 0.8)
- `mobility_weight`: Weight for mobility (Default: 0.6)
- `center_control_weight`: Weight for center control (Default: 0.7)
- `development_weight`: Weight for development (Default: 0.5)

### Advanced Features

#### Multi-Level Transposition Table
- Multiple table levels with different hash sizes
- LRU replacement policy at each level
- Configurable memory allocation per level

#### Compressed Entry Storage
- Entry compression algorithms
- Variable compression ratios
- Memory vs speed tradeoffs

#### Predictive Prefetching
- Prefetch prediction models
- Access pattern analysis
- Cache warming strategies

#### Machine Learning Replacement Policies
- ML-based replacement decisions
- Position feature extraction
- Temporal pattern learning

#### Dynamic Table Sizing
- Runtime table resizing
- Access pattern analysis
- Performance-based adjustments

## Configuration Presets

The engine includes several preset configurations:

### Default
- Balanced settings
- All optimizations enabled
- Standard depth and time limits

### Aggressive
- Maximum search optimization
- Higher depths
- More aggressive pruning

### Conservative
- More careful search
- Less aggressive pruning
- Prioritizes accuracy over speed

### Fast
- Optimized for speed
- Lower depths
- Minimal pruning overhead

### Strongest
- Maximum strength
- Deep search
- Full evaluation features

## How to Use

### Via USI Protocol

```bash
# Set transposition table size to 128 MB
setoption name USI_Hash value 128

# Set search depth to 8
setoption name depth value 8

# Disable null-move pruning
setoption name EnableNullMove value false

# Enable deeper quiescence search
setoption name QuiescenceDepth value 8

# Disable aspiration windows for slower but more thorough search
setoption name EnableAspirationWindows value false

# Disable tablebase lookups
setoption name EnableTablebase value false
```

### Via Code (Rust)

```rust
use shogi_engine::types::*;

// Create engine with custom configuration
let mut engine = ShogiEngine::new();

// Configure via setoption handler
engine.handle_setoption(&["name", "depth", "value", "8"]);

// Or directly configure search engine
if let Ok(mut search_engine) = engine.search_engine.lock() {
    let mut config = search_engine.get_engine_config();
    config.max_depth = 10;
    config.quiescence.max_depth = 8;
    search_engine.update_engine_config(config).unwrap();
}
```

## Performance Impact

Different configurations have different performance impacts:

- **Aggressive Pruning**: Faster search but may miss tactics
- **Deeper Search**: Better strength but slower
- **Large Hash**: Better transposition table hits but more memory
- **Tablebase Enabled**: Perfect endgame but requires tablebase data

## Recommendations

### For Casual Play (Fast)
- Depth: 5-6
- Hash: 64-128 MB
- Enable all optimizations
- Tablebase: Enabled

### For Competitive Play (Strong)
- Depth: 8-12
- Hash: 256-512 MB
- Enable all optimizations
- Tablebase: Enabled
- Deeper quiescence (8-10)

### For Analysis (Deepest)
- Depth: 15+
- Hash: 512-1024 MB
- Enable all optimizations
- Tablebase: Enabled
- Max quiescence depth

## Future Enhancements

Future versions may expose more options:

- Evaluation weights (material, position, safety)
- Time management controls
- Move ordering preferences
- Opening book selection strategy
- Pattern recognition weights
- Individual feature enable/disable

## See Also

- [README.md](../README.md) - General engine information
- [BUILDING.md](../BUILDING.md) - Building instructions
- [ARCHITECTURE.md](architecture/ENGINE_ARCHITECTURE.md) - Engine architecture

