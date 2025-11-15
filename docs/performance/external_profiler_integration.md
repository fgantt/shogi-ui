# External Profiler Integration and Hot Path Analysis

## Overview

The external profiler integration system provides hooks for integrating with system-level profilers like `perf` (Linux) and Instruments (macOS). This enables detailed hot path analysis and performance optimization by identifying bottlenecks in the search engine's critical paths.

## Features

- **External Profiler Trait**: Unified interface for different profiler implementations
- **Perf Profiler**: Linux-compatible profiler for `perf` integration
- **Instruments Profiler**: macOS-compatible profiler for Instruments integration
- **Hot Path Markers**: Automatic markers in critical paths (evaluation, move ordering, TT operations)
- **Marker Export**: JSON export of profiling markers for analysis
- **Script Integration**: Convenient scripts for running with external profilers

## Architecture

### ExternalProfiler Trait

The `ExternalProfiler` trait provides a unified interface for profiler implementations:

```rust
pub trait ExternalProfiler: Send + Sync {
    fn start_region(&self, name: &str);
    fn end_region(&self, name: &str);
    fn mark(&self, label: &str);
    fn export_markers(&self) -> Result<serde_json::Value, String>;
    fn is_enabled(&self) -> bool;
}
```

### Profiler Implementations

- **PerfProfiler**: Generates perf-compatible markers for Linux
- **InstrumentsProfiler**: Generates Instruments-compatible markers for macOS

Both implementations track markers with timestamps and export them to JSON format.

## Usage

### Basic Setup

```rust
use crate::search::performance_tuning::{PerfProfiler, InstrumentsProfiler};
use crate::search::search_engine::SearchEngine;
use std::sync::Arc;

// Create and enable profiler
let mut profiler = PerfProfiler::new(); // or InstrumentsProfiler::new()
profiler.enable();

// Enable external profiling in search engine
let mut engine = SearchEngine::new(None, 16);
engine.enable_external_profiling(Arc::new(profiler));

// Run search - markers will be automatically added to hot paths
let board = BitboardBoard::starting_position();
let _ = engine.search(&board, 6, None);

// Export markers for analysis
let markers = engine.export_profiling_markers()?;
```

### Hot Path Markers

The following hot paths are automatically instrumented:

1. **Evaluation** (`evaluate_position`): Region markers around position evaluation
2. **Move Ordering** (`order_moves_for_negamax`): Region markers around move ordering
3. **TT Operations**: Point markers for TT probe and store operations

### Marker Types

- **RegionStart/RegionEnd**: Marks the beginning and end of a code region
- **Point**: Marks a specific point in time

## Scripts

### Linux (perf)

```bash
# Basic usage
./scripts/run_with_perf.sh

# Custom depth and FEN
./scripts/run_with_perf.sh --depth 8 --fen "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"

# Custom output directory
./scripts/run_with_perf.sh --output-dir ./my_perf_data
```

The script generates `perf.data` which can be analyzed with:
- `perf report -i perf_data/perf.data`
- `perf script -i perf_data/perf.data > perf_data/perf.script`
- `perf annotate -i perf_data/perf.data`

### macOS (Instruments)

```bash
# Basic usage
./scripts/run_with_instruments.sh

# Custom depth and FEN
./scripts/run_with_instruments.sh --depth 8 --fen "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"

# Custom output directory
./scripts/run_with_instruments.sh --output-dir ./my_instruments_data
```

The script generates `instruments.trace` which can be opened in Instruments.app:
- `open instruments_data/instruments.trace`

## Marker Export Format

Markers are exported as JSON with the following structure:

```json
{
  "profiler": "perf",
  "markers": [
    {
      "name": "evaluate_position",
      "timestamp_ns": 1234567890,
      "type": "region_start"
    },
    {
      "name": "evaluate_position",
      "timestamp_ns": 1234568900,
      "type": "region_end"
    },
    {
      "name": "tt_probe",
      "timestamp_ns": 1234569000,
      "type": "point"
    }
  ],
  "total_markers": 3
}
```

## Integration with External Profilers

### Perf (Linux)

The profiler markers are designed to work with `perf`'s annotation system. When running with `perf record`, the markers will appear as annotations in the profiler output.

### Instruments (macOS)

The profiler markers are designed to work with Instruments' time profiler. Markers will appear as custom events in the Instruments timeline.

## Best Practices

1. **Enable Only When Needed**: External profiling adds overhead - disable in production
2. **Use Appropriate Depth**: Use shallow depths (4-6) for profiling to reduce noise
3. **Analyze Hot Paths**: Focus on regions with high marker frequency
4. **Compare Baselines**: Use markers to compare performance before/after optimizations
5. **Export Regularly**: Export markers after each profiling session for trend analysis

## Limitations

- Profiler overhead is non-zero - should be disabled in production
- Markers use relative timestamps (from profiler start time)
- Some hot paths may not be fully instrumented (e.g., parallel search internals)
- External profiler integration requires system-level profiler tools (perf/Instruments)

## Future Enhancements

- Real-time marker streaming to external profiler
- Integration with more profilers (VTune, Valgrind)
- Automatic hot path identification from marker frequency
- Marker visualization tools
- Integration with performance dashboards

