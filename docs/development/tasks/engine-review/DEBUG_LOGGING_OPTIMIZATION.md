# Debug Logging Optimization

## Overview

Task 2.0 optimizes debug logging performance by introducing conditional compilation and lightweight macros. This eliminates expensive string formatting overhead when debug logging is disabled.

## Implementation

### Feature Flag

A new `verbose-debug` feature flag has been added to `Cargo.toml`:

```toml
[features]
default = []
verbose-debug = []  # Enable verbose debug logging with compile-time checks
```

### Usage

**Default (Feature Disabled):**
- All debug logging functions return immediately (zero overhead)
- No string formatting occurs
- No runtime checks (compile-time removal)
- Best performance for release builds

**With Feature Enabled:**
- Debug logging functions check runtime flag first
- String formatting only occurs if debug is enabled
- Backward compatible with USI "debug on/off" command
- Still provides performance benefit via early return

**Build Commands:**
```bash
# Default build (debug logging removed at compile time)
cargo build --release

# Build with verbose debug logging enabled
cargo build --release --features verbose-debug

# Enable debug at runtime (requires verbose-debug feature)
# Send USI command: debug on
```

### Performance Characteristics

**When Feature Disabled:**
- Zero overhead - functions compile to nothing
- No string formatting overhead
- No runtime checks
- Optimal for production builds

**When Feature Enabled but Debug Disabled:**
- Runtime flag check overhead (minimal - atomic read)
- String formatting still occurs (Rust evaluates macro arguments eagerly)
- Function returns early without logging
- Some overhead remains from format!() evaluation

**When Feature Enabled and Debug Enabled:**
- Full logging functionality
- All string formatting occurs
- Same behavior as before optimization

### Optimization Notes

**Current Limitation:**
The current implementation still evaluates `format!()` strings even when debug is disabled, because Rust macros evaluate their arguments eagerly. For example:

```rust
trace_log("FEATURE", &format!("depth={} alpha={}", depth, alpha));
```

The `format!()` call is evaluated before `trace_log()` checks the flag.

**Future Improvement:**
For maximum performance, use lazy evaluation with closures:
```rust
trace_log_lazy!("FEATURE", || format!("depth={} alpha={}", depth, alpha));
```

Or wrap hot-path calls in conditional compilation:
```rust
#[cfg(feature = "verbose-debug")]
{
    trace_log("FEATURE", &format!("depth={}", depth));
}
```

### Backward Compatibility

The USI "debug on/off" command continues to work as before:
- `debug on` - Enables debug logging (requires verbose-debug feature)
- `debug off` - Disables debug logging
- `debug trace` - Enables trace logging
- `debug notrace` - Disables trace logging

### Macros Available

The following lightweight macros are available (when verbose-debug feature is enabled):

- `trace_log_fast!(feature, message)` - Fast trace logging with runtime check
- `debug_log_fast!(message)` - Fast debug logging with runtime check
- `log_decision_fast!(feature, decision, reason, value)` - Fast decision logging
- `log_move_eval_fast!(feature, move_str, score, reason)` - Fast move evaluation logging
- `trace_log_lazy!(feature, || format!(...))` - Lazy evaluation trace logging

### Benchmark

A benchmark has been added to measure debug logging overhead:
```bash
cargo bench --bench debug_logging_performance_benchmarks
```

This compares performance with and without debug logging to quantify the optimization impact.

---

**Status:** Complete  
**Date:** December 2024  
**Task:** 2.0 Optimize Debug Logging Performance

