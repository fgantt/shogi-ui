# Telemetry Export and Advanced Metrics Analysis

## Overview

The telemetry export system provides comprehensive performance metrics export capabilities for analysis and monitoring. This feature enables exporting performance data in multiple formats (JSON, CSV, Markdown) and includes specialized exports for efficiency metrics, transposition table quality, hit rates by depth, scalability metrics, and cache effectiveness.

## Features

- **Multiple Export Formats**: JSON, CSV, and Markdown formats for different analysis needs
- **Performance Metrics Export**: Complete performance baseline and runtime metrics
- **Efficiency Metrics Export**: IID and LMR efficiency analysis (PRD Section 3.4 gap)
- **TT Entry Quality Distribution**: Exact/Beta/Alpha entry percentages (PRD Section 5.2 gap)
- **Hit Rate by Depth**: Transposition table hit rates stratified by depth (PRD Section 5.3 gap)
- **Scalability Metrics**: Parallel search scalability for regression analysis (PRD Section 7.3 gap)
- **Cache Effectiveness**: Cache hit rates and size monitoring (PRD Section 4.2 gap)

## Configuration

Telemetry export can be configured via `EngineConfig`:

```rust
let mut config = EngineConfig::default();
config.telemetry_export_enabled = true;
config.telemetry_export_path = "telemetry".to_string();
```

### Configuration Options

- `telemetry_export_enabled: bool` - Enable/disable automatic telemetry export (default: `false`)
- `telemetry_export_path: String` - Directory path for telemetry exports (default: `"telemetry"`)

## Usage

### Basic Export

```rust
use crate::search::performance_tuning::TelemetryExporter;
use crate::search::search_engine::SearchEngine;

let exporter = TelemetryExporter::new("telemetry");
let mut engine = SearchEngine::new();

// Run search
let board = BitboardBoard::starting_position();
let _ = engine.search(&board, 6, None);

// Export performance metrics
exporter.export_performance_metrics_to_json(&engine, "metrics.json")?;
exporter.export_performance_metrics_to_csv(&engine, "metrics.csv")?;
exporter.export_performance_metrics_to_markdown(&engine, "metrics.md")?;
```

### Advanced Metrics Export

```rust
// Export efficiency metrics (IID and LMR)
exporter.export_efficiency_metrics(&engine, "efficiency.json")?;

// Export TT entry quality distribution
exporter.export_tt_entry_quality_distribution(&engine, "tt_quality.json")?;

// Export hit rate by depth
exporter.export_hit_rate_by_depth(&engine, "hit_rate.json")?;

// Export scalability metrics
exporter.export_scalability_metrics(&engine, "scalability.json")?;

// Export cache effectiveness
exporter.export_cache_effectiveness(&engine, "cache.json")?;
```

### Disabling Export

```rust
let mut exporter = TelemetryExporter::new("telemetry");
exporter.set_enabled(false);

// Exports will return an error when disabled
let result = exporter.export_performance_metrics_to_json(&engine, "metrics.json");
assert!(result.is_err());
```

## Export Formats

### JSON Format

The JSON export includes:
- Timestamp
- Performance metrics (nodes per second, aspiration success rate, etc.)
- Baseline metrics (search, evaluation, TT, move ordering, parallel, memory)

Example:
```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "performance_metrics": {
    "nodes_per_second": 1000000.0,
    "aspiration_success_rate": 0.85,
    "health_score": 0.92
  },
  "baseline_metrics": {
    "search_metrics": { ... },
    "evaluation_metrics": { ... },
    "tt_metrics": { ... }
  }
}
```

### CSV Format

The CSV export provides a flat structure suitable for spreadsheet analysis:
- Metric name and value pairs
- All performance and baseline metrics in a single table

### Markdown Format

The Markdown export provides a human-readable report with:
- Performance metrics table
- Search metrics table
- Evaluation metrics table
- Transposition table metrics table
- Move ordering metrics table
- Parallel search metrics table
- Memory metrics table

## Specialized Exports

### Efficiency Metrics

Exports IID and LMR efficiency metrics including:
- IID efficiency rate, cutoff rate, overhead percentage
- LMR reductions applied, researches triggered, cutoffs

### TT Entry Quality Distribution

Exports transposition table entry quality distribution:
- Total entries, exact entries, beta entries, alpha entries
- Percentages for each entry type
- Overall hit rate and occupancy rate

### Hit Rate by Depth

Exports transposition table hit rates stratified by search depth:
- Overall hit rate
- Hit rate for each depth level (1-10)

### Scalability Metrics

Exports parallel search scalability metrics:
- Speedup for 4 and 8 cores
- Efficiency for 4 and 8 cores
- Comparison to linear speedup

### Cache Effectiveness

Exports cache effectiveness metrics:
- Evaluation cache hit rate
- Move ordering cache hit rates (PV, killer, cache)
- Cache memory usage

## Script Usage

The `scripts/export_telemetry.sh` script provides a convenient way to export telemetry:

```bash
# Basic usage
./scripts/export_telemetry.sh

# Custom export directory and search depth
./scripts/export_telemetry.sh --export-dir ./my_telemetry --depth 8

# Custom FEN position
./scripts/export_telemetry.sh --fen "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1"
```

## Best Practices

1. **Export Directory**: Use a dedicated directory for telemetry exports to avoid cluttering the project
2. **Naming Convention**: Use descriptive filenames with timestamps for tracking over time
3. **Regular Exports**: Export telemetry regularly during development to track performance changes
4. **CI Integration**: Use telemetry exports in CI to track performance regressions
5. **Analysis Tools**: Use CSV exports for spreadsheet analysis, JSON for programmatic analysis, Markdown for reports

## Limitations

- Export overhead is minimal but non-zero - disable in production if not needed
- Some metrics (e.g., hit rate by depth) use estimates when detailed tracking is not available
- Export directory must be writable by the process

## Future Enhancements

- Real-time streaming export
- Database integration for long-term storage
- Automated analysis and alerting
- Integration with performance dashboards
- Export compression for large datasets

