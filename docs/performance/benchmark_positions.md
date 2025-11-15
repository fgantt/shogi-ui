# Standard Benchmark Positions and Regression Suite

## Overview

The benchmark position set and regression suite (Task 26.0 - Task 5.0) provides a standardized set of positions for consistent performance testing and automated regression detection. This enables reliable performance comparisons across code changes.

## Standard Positions

The standard position set includes 5 positions covering different game phases:

1. **Starting Position** (Opening)
   - FEN: `lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1`
   - Expected depth: 5
   - Tests opening move generation and evaluation

2. **Mid-game Tactical Position** (MiddlegameTactical)
   - FEN: `lnsgkgsnl/1r5b1/ppppppppp/9/9/4P4/PPPP1PPPP/1B5R1/LNSGKGSNL w - 1`
   - Expected depth: 6
   - Tests search efficiency and move ordering in tactical positions

3. **Mid-game Positional Position** (MiddlegamePositional)
   - FEN: `lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1`
   - Expected depth: 6
   - Tests positional evaluation and long-term planning

4. **Endgame King Activity Position** (EndgameKingActivity)
   - FEN: `4k4/9/9/9/9/9/9/9/4K4 b - 1`
   - Expected depth: 7
   - Tests endgame evaluation and king safety

5. **Endgame Zugzwang Position** (EndgameZugzwang)
   - FEN: `4k4/9/9/9/9/9/9/9/4K4 w - 1`
   - Expected depth: 8
   - Tests deep search and zugzwang detection

## Usage

### Loading Standard Positions

```rust
use shogi_engine::search::performance_tuning::load_standard_positions;

let positions = load_standard_positions()?;
for position in &positions {
    println!("{}: {}", position.name, position.fen);
}
```

### Running a Single Position Benchmark

```rust
use shogi_engine::search::performance_tuning::{BenchmarkRunner, load_standard_positions};
use shogi_engine::search::search_engine::SearchEngine;

let mut engine = SearchEngine::new(None, 16);
let runner = BenchmarkRunner::new()
    .with_time_limit(10000); // 10 seconds

let positions = load_standard_positions()?;
let result = runner.run_position_benchmark(&positions[0], &mut engine)?;

println!("Position: {}", result.position_name);
println!("Time: {}ms", result.search_time_ms);
println!("Nodes: {}", result.nodes_searched);
println!("Nodes/sec: {:.2}", result.nodes_per_second);
```

### Running Regression Suite

```rust
use shogi_engine::search::performance_tuning::BenchmarkRunner;
use shogi_engine::search::search_engine::SearchEngine;
use std::path::PathBuf;

let mut engine = SearchEngine::new(None, 16);
let runner = BenchmarkRunner::new()
    .with_regression_threshold(5.0) // 5% threshold
    .with_baseline_path(PathBuf::from("docs/performance/baselines/latest.json"))
    .with_time_limit(10000);

let suite_result = runner.run_regression_suite(&mut engine)?;

println!("Total positions: {}", suite_result.total_positions);
println!("Regressions detected: {}", suite_result.regressions_detected);

for regression in &suite_result.regressions {
    println!("Regression in {}: {:.2}% slower",
        regression.position_name,
        regression.regression_percentage
    );
}
```

### Using the Script

```bash
# Run regression suite
./scripts/run_regression_suite.sh

# Run with custom baseline
./scripts/run_regression_suite.sh --baseline-path path/to/baseline.json

# Run with custom threshold
./scripts/run_regression_suite.sh --regression-threshold 10.0

# Run in test mode (fails on regressions)
./scripts/run_regression_suite.sh --regression-test
```

## Regression Detection

Regression detection compares current search times against baseline times:

- **Threshold**: Default 5% (configurable)
- **Detection**: Flags positions where current time > baseline time + threshold
- **Reporting**: Includes position name, baseline time, current time, and regression percentage

### Regression Test Result

```rust
use shogi_engine::search::performance_tuning::RegressionTestResult;

let result = RegressionTestResult::new(
    "Position1".to_string(),
    100,  // baseline time (ms)
    110,  // current time (ms)
    5.0,  // threshold (%)
);

assert!(result.regression_detected); // 10% > 5% threshold
assert_eq!(result.regression_percentage, 10.0);
```

## Baseline Format

Baseline files are JSON maps of position names to search times:

```json
{
  "Starting Position": 150,
  "Mid-game Tactical Position": 200,
  "Mid-game Positional Position": 180,
  "Endgame King Activity Position": 120,
  "Endgame Zugzwang Position": 300
}
```

## Integration with CI

The regression suite can be integrated into CI pipelines:

```bash
# In CI workflow
./scripts/run_regression_suite.sh \
    --baseline-path docs/performance/baselines/latest.json \
    --regression-test

# Exit code will be non-zero if regressions detected
if [ $? -ne 0 ]; then
    echo "Performance regression detected!"
    exit 1
fi
```

## Best Practices

1. **Establish Baseline**: Run regression suite on known good commit to establish baseline
2. **Regular Testing**: Run regression suite before merging PRs
3. **Threshold Tuning**: Adjust threshold based on normal variance (typically 5-10%)
4. **Position Updates**: Add new positions as needed for comprehensive coverage
5. **Baseline Updates**: Update baseline when performance improvements are verified

## Limitations

- Search times may vary due to system load
- Baseline comparison requires consistent hardware
- Position complexity may affect timing accuracy
- Deep positions may timeout with default time limits

## Future Enhancements

- Statistical significance testing for regression detection
- Historical trend analysis across multiple baselines
- Automatic baseline updates on performance improvements
- Position-specific thresholds based on variance
- Integration with performance profiling for detailed analysis

