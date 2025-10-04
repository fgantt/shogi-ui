# Troubleshooting Guide for Advanced Alpha-Beta Pruning

## Overview

This document provides a comprehensive troubleshooting guide for common issues encountered with the advanced alpha-beta pruning implementation in the Shogi engine. It includes diagnostic procedures, solutions, and preventive measures for various problems.

## Table of Contents

1. [Common Issues and Symptoms](#common-issues-and-symptoms)
2. [Diagnostic Procedures](#diagnostic-procedures)
3. [Performance Issues](#performance-issues)
4. [Correctness Issues](#correctness-issues)
5. [Memory Issues](#memory-issues)
6. [Integration Issues](#integration-issues)
7. [Parameter Tuning Issues](#parameter-tuning-issues)
8. [Debugging Tools and Techniques](#debugging-tools-and-techniques)
9. [Preventive Measures](#preventive-measures)
10. [Emergency Procedures](#emergency-procedures)

## Common Issues and Symptoms

### Issue Categories

#### 1. Performance Issues
- **Slow Search**: Search taking longer than expected
- **Low Pruning Rate**: Pruning not effective enough
- **High Memory Usage**: Excessive memory consumption
- **Cache Inefficiency**: Low cache hit rates

#### 2. Correctness Issues
- **Tactical Losses**: Missing tactical sequences
- **Missed Best Moves**: Not finding the best move
- **Inconsistent Results**: Different results for same position
- **Search Instability**: Unstable search behavior

#### 3. Integration Issues
- **Compilation Errors**: Code compilation failures
- **Runtime Crashes**: Application crashes during search
- **API Incompatibilities**: Interface compatibility issues
- **Configuration Problems**: Parameter configuration issues

#### 4. Parameter Issues
- **Over-Pruning**: Too aggressive pruning
- **Under-Pruning**: Too conservative pruning
- **Parameter Conflicts**: Conflicting parameter values
- **Tuning Difficulties**: Difficulty finding optimal parameters

## Diagnostic Procedures

### Step-by-Step Diagnosis

#### 1. Initial Assessment
```rust
pub struct InitialAssessment {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub affected_components: Vec<Component>,
    pub reproduction_steps: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
}
```

#### 2. System Health Check
```rust
pub struct SystemHealthCheck {
    pub memory_usage: MemoryUsage,
    pub cpu_usage: CpuUsage,
    pub cache_performance: CachePerformance,
    pub search_performance: SearchPerformance,
    pub error_logs: Vec<ErrorLog>,
}
```

#### 3. Component Analysis
```rust
pub struct ComponentAnalysis {
    pub pruning_manager: PruningManagerAnalysis,
    pub search_engine: SearchEngineAnalysis,
    pub parameter_config: ParameterConfigAnalysis,
    pub performance_metrics: PerformanceMetricsAnalysis,
}
```

### Diagnostic Tools

```rust
pub struct DiagnosticTools {
    pub health_checker: HealthChecker,
    pub performance_analyzer: PerformanceAnalyzer,
    pub error_detector: ErrorDetector,
    pub parameter_validator: ParameterValidator,
    pub search_validator: SearchValidator,
}
```

## Performance Issues

### Issue: Slow Search Performance

#### Symptoms
- Search time significantly longer than expected
- Low nodes per second
- High CPU usage
- Poor scalability with depth

#### Diagnostic Steps
```rust
// 1. Check basic performance metrics
let performance = engine.get_performance_metrics();
println!("Nodes per second: {}", performance.nodes_per_second);
println!("Search time: {}ms", performance.search_time_ms);

// 2. Check pruning effectiveness
let pruning_stats = engine.get_pruning_statistics();
println!("Pruning rate: {:.2}%", pruning_stats.get_pruning_rate() * 100.0);

// 3. Check cache performance
let cache_stats = engine.get_cache_statistics();
println!("Cache hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
```

#### Common Causes
1. **Low Pruning Rate**: Pruning not effective enough
2. **Cache Misses**: Poor cache performance
3. **Parameter Issues**: Suboptimal parameters
4. **Memory Pressure**: High memory usage causing slowdowns

#### Solutions
```rust
// Solution 1: Optimize pruning parameters
let optimized_params = PruningParameters {
    futility_margin: [0, 120, 240, 360, 480, 600, 720, 840], // More aggressive
    lmr_move_threshold: 4, // More aggressive
    delta_margin: 250, // More aggressive
    ..Default::default()
};

// Solution 2: Increase cache sizes
engine.optimize_cache_sizes();

// Solution 3: Enable adaptive parameters
let adaptive_params = PruningParameters {
    adaptive_enabled: true,
    position_dependent_margins: true,
    ..Default::default()
};
```

### Issue: Low Pruning Rate

#### Symptoms
- Pruning rate below 20%
- High number of nodes searched
- Poor tree reduction
- Ineffective pruning techniques

#### Diagnostic Steps
```rust
// 1. Check individual pruning technique rates
let pruning_breakdown = engine.get_pruning_breakdown();
println!("Futility pruning: {:.2}%", pruning_breakdown.futility_rate * 100.0);
println!("Delta pruning: {:.2}%", pruning_breakdown.delta_rate * 100.0);
println!("Razoring: {:.2}%", pruning_breakdown.razoring_rate * 100.0);

// 2. Check position characteristics
let position_analysis = engine.analyze_position();
println!("Position type: {:?}", position_analysis.position_type);
println!("Game phase: {:?}", position_analysis.game_phase);
```

#### Common Causes
1. **Conservative Parameters**: Parameters too conservative
2. **Tactical Position**: Position not suitable for pruning
3. **Shallow Search**: Search depth too shallow
4. **Check Position**: In check, pruning disabled

#### Solutions
```rust
// Solution 1: Make parameters more aggressive
let aggressive_params = PruningParameters {
    futility_margin: [0, 120, 240, 360, 480, 600, 720, 840],
    futility_depth_limit: 4, // Increase depth limit
    lmr_move_threshold: 2, // Lower threshold
    lmr_max_reduction: 4, // Increase max reduction
    delta_margin: 250, // Increase margin
    razoring_margin: 350, // Increase margin
    ..Default::default()
};

// Solution 2: Enable position-dependent parameters
let adaptive_params = PruningParameters {
    adaptive_enabled: true,
    position_dependent_margins: true,
    ..Default::default()
};
```

### Issue: High Memory Usage

#### Symptoms
- Memory usage above 100 MB
- Memory growth over time
- Cache size issues
- Memory leaks

#### Diagnostic Steps
```rust
// 1. Check memory usage
let memory_usage = engine.get_memory_usage();
println!("Total memory: {} MB", memory_usage.total_mb);
println!("Cache memory: {} MB", memory_usage.cache_mb);

// 2. Check cache sizes
let cache_stats = engine.get_cache_statistics();
println!("Pruning cache size: {}", cache_stats.pruning_cache_size);
println!("Position cache size: {}", cache_stats.position_cache_size);

// 3. Check for memory leaks
let memory_growth = engine.analyze_memory_growth();
println!("Memory growth rate: {} MB/s", memory_growth.growth_rate);
```

#### Common Causes
1. **Large Cache Sizes**: Caches too large
2. **Memory Leaks**: Objects not properly cleaned up
3. **Cache Growth**: Caches growing without bounds
4. **Inefficient Data Structures**: Poor memory usage

#### Solutions
```rust
// Solution 1: Reduce cache sizes
engine.set_cache_limits(10000, 5000, 2000); // pruning, position, check

// Solution 2: Enable cache cleanup
engine.enable_automatic_cache_cleanup();

// Solution 3: Implement memory monitoring
engine.enable_memory_monitoring();
```

## Correctness Issues

### Issue: Tactical Losses

#### Symptoms
- Missing forced mate sequences
- Missing tactical combinations
- Incorrect evaluation of tactical positions
- Poor performance in tactical tests

#### Diagnostic Steps
```rust
// 1. Test on known tactical positions
let tactical_tests = load_tactical_test_suite();
for test in tactical_tests {
    let result = engine.search(test.position, test.time_limit);
    if result.best_move != test.expected_move {
        println!("Tactical loss: {}", test.name);
    }
}

// 2. Check pruning decisions in tactical positions
let pruning_analysis = engine.analyze_pruning_decisions(tactical_position);
println!("Pruning rate in tactical position: {:.2}%", pruning_analysis.pruning_rate);
```

#### Common Causes
1. **Over-Pruning**: Too aggressive pruning in tactical positions
2. **Incorrect Parameters**: Parameters not suitable for tactical positions
3. **Missing Safety Checks**: Pruning not properly disabled in check
4. **Tactical Move Pruning**: Pruning tactical moves

#### Solutions
```rust
// Solution 1: Make parameters more conservative for tactical positions
let tactical_params = PruningParameters {
    futility_depth_limit: 2, // More conservative
    lmr_max_reduction: 2, // More conservative
    delta_margin: 150, // More conservative
    razoring_margin: 250, // More conservative
    ..Default::default()
};

// Solution 2: Add tactical position detection
impl PruningManager {
    fn is_tactical_position(&self, state: &SearchState) -> bool {
        // Detect tactical positions
        state.static_eval.abs() > 500 || // Large evaluation
        state.is_in_check || // In check
        self.has_recent_captures(state) || // Recent captures
        self.has_forced_moves(state) // Forced moves
    }
    
    fn should_prune(&mut self, state: &SearchState, mv: &Move) -> PruningDecision {
        if self.is_tactical_position(state) {
            return PruningDecision::Search; // Don't prune in tactical positions
        }
        // ... normal pruning logic
    }
}
```

### Issue: Missed Best Moves

#### Symptoms
- Engine not finding the best move
- Suboptimal move selection
- Inconsistent move quality
- Poor performance in position tests

#### Diagnostic Steps
```rust
// 1. Compare with known best moves
let position_tests = load_position_test_suite();
for test in position_tests {
    let result = engine.search(test.position, test.time_limit);
    if result.best_move != test.best_move {
        println!("Missed best move: {}", test.name);
        println!("Expected: {}, Got: {}", 
                 test.best_move.to_string(), 
                 result.best_move.to_string());
    }
}

// 2. Analyze move ordering
let move_ordering_analysis = engine.analyze_move_ordering(test.position);
println!("Best move rank: {}", move_ordering_analysis.best_move_rank);
```

#### Common Causes
1. **Poor Move Ordering**: Best move not ordered first
2. **Pruning Best Moves**: Best move being pruned
3. **Insufficient Search Depth**: Not searching deep enough
4. **Evaluation Issues**: Poor position evaluation

#### Solutions
```rust
// Solution 1: Improve move ordering
impl SearchEngine {
    fn order_moves(&self, moves: Vec<Move>, state: &SearchState) -> Vec<Move> {
        let mut ordered_moves = moves;
        
        // Prioritize tactical moves
        ordered_moves.sort_by(|a, b| {
            let a_score = self.score_move(a, state);
            let b_score = self.score_move(b, state);
            b_score.cmp(&a_score)
        });
        
        ordered_moves
    }
}

// Solution 2: Add best move protection
impl PruningManager {
    fn should_prune(&mut self, state: &SearchState, mv: &Move) -> PruningDecision {
        // Don't prune the first few moves (likely to be best)
        if state.move_number <= 2 {
            return PruningDecision::Search;
        }
        
        // Don't prune moves that improve alpha significantly
        if let Some(best_move) = state.best_move {
            if mv == &best_move {
                return PruningDecision::Search;
            }
        }
        
        // ... normal pruning logic
    }
}
```

## Memory Issues

### Issue: Memory Leaks

#### Symptoms
- Memory usage growing over time
- Application crashes due to memory exhaustion
- Poor performance after long runs
- Cache sizes growing without bounds

#### Diagnostic Steps
```rust
// 1. Monitor memory usage over time
let memory_monitor = MemoryMonitor::new();
for _ in 0..1000 {
    engine.search(test_position, 1000);
    let current_memory = memory_monitor.get_current_usage();
    println!("Memory usage: {} MB", current_memory);
}

// 2. Check for cache growth
let cache_monitor = CacheMonitor::new();
for _ in 0..1000 {
    engine.search(test_position, 1000);
    let cache_sizes = cache_monitor.get_cache_sizes();
    println!("Cache sizes: {:?}", cache_sizes);
}
```

#### Common Causes
1. **Unbounded Cache Growth**: Caches growing without limits
2. **Object Retention**: Objects not being garbage collected
3. **Circular References**: Circular references preventing cleanup
4. **Resource Leaks**: Resources not properly released

#### Solutions
```rust
// Solution 1: Implement cache size limits
impl PruningManager {
    fn should_cache_decision(&self, cache_key: u64, decision: PruningDecision) -> bool {
        if self.pruning_cache.len() >= self.max_cache_size {
            // Remove oldest entries
            self.cleanup_cache();
        }
        true
    }
    
    fn cleanup_cache(&mut self) {
        // Remove 20% of oldest entries
        let remove_count = self.pruning_cache.len() / 5;
        let mut keys_to_remove = Vec::new();
        
        for (key, _) in self.pruning_cache.iter().take(remove_count) {
            keys_to_remove.push(*key);
        }
        
        for key in keys_to_remove {
            self.pruning_cache.remove(&key);
        }
    }
}

// Solution 2: Implement automatic cleanup
impl SearchEngine {
    fn enable_automatic_cleanup(&mut self) {
        self.cleanup_interval = Some(10000); // Cleanup every 10k nodes
    }
    
    fn check_cleanup(&mut self) {
        if let Some(interval) = self.cleanup_interval {
            if self.nodes_searched % interval == 0 {
                self.cleanup_caches();
            }
        }
    }
}
```

## Integration Issues

### Issue: Compilation Errors

#### Symptoms
- Code not compiling
- Type errors
- Missing dependencies
- API incompatibilities

#### Common Causes
1. **Missing Imports**: Required modules not imported
2. **Type Mismatches**: Incorrect type usage
3. **API Changes**: Breaking changes in dependencies
4. **Configuration Issues**: Incorrect build configuration

#### Solutions
```rust
// Solution 1: Check imports
use crate::types::*;
use crate::search::*;
use crate::pruning::*;

// Solution 2: Fix type mismatches
let params = PruningParameters {
    futility_margin: [0, 100, 200, 300, 400, 500, 600, 700],
    futility_depth_limit: 3u8, // Explicit type
    ..Default::default()
};

// Solution 3: Update API usage
let pruning_manager = PruningManager::new(params);
let decision = pruning_manager.should_prune(&state, &mv);
```

### Issue: Runtime Crashes

#### Symptoms
- Application crashes during search
- Panic messages
- Segmentation faults
- Stack overflow

#### Diagnostic Steps
```rust
// 1. Enable panic handling
std::panic::set_hook(Box::new(|panic_info| {
    println!("Panic occurred: {:?}", panic_info);
    // Log panic information
}));

// 2. Add bounds checking
impl PruningManager {
    fn get_futility_margin(&self, depth: u8) -> i32 {
        let depth_index = (depth as usize).min(self.parameters.futility_margin.len() - 1);
        self.parameters.futility_margin[depth_index]
    }
}
```

#### Common Causes
1. **Array Bounds**: Accessing arrays out of bounds
2. **Null Pointers**: Dereferencing null pointers
3. **Stack Overflow**: Deep recursion
4. **Integer Overflow**: Arithmetic overflow

#### Solutions
```rust
// Solution 1: Add bounds checking
impl PruningManager {
    fn safe_get_futility_margin(&self, depth: u8) -> i32 {
        if depth as usize >= self.parameters.futility_margin.len() {
            return self.parameters.futility_margin.last().unwrap_or(&0).clone();
        }
        self.parameters.futility_margin[depth as usize]
    }
}

// Solution 2: Use safe arithmetic
impl PruningManager {
    fn safe_calculate_reduction(&self, depth: u8, move_number: u8) -> u8 {
        let base_reduction = self.parameters.lmr_base_reduction;
        let depth_factor = (depth.saturating_sub(self.parameters.lmr_depth_threshold)).min(3);
        let move_factor = (move_number.saturating_sub(self.parameters.lmr_move_threshold)).min(3);
        
        let reduction = base_reduction.saturating_add(depth_factor).saturating_add(move_factor);
        reduction.min(self.parameters.lmr_max_reduction)
    }
}
```

## Parameter Tuning Issues

### Issue: Over-Pruning

#### Symptoms
- High pruning rate (>50%)
- Tactical losses
- Missed best moves
- Poor performance in tests

#### Solutions
```rust
// Solution: Make parameters more conservative
let conservative_params = PruningParameters {
    futility_margin: [0, 80, 160, 240, 320, 400, 480, 560], // Smaller margins
    futility_depth_limit: 2, // Lower depth limit
    lmr_move_threshold: 4, // Higher threshold
    lmr_max_reduction: 2, // Lower max reduction
    delta_margin: 150, // Smaller margin
    razoring_margin: 250, // Smaller margin
    ..Default::default()
};
```

### Issue: Under-Pruning

#### Symptoms
- Low pruning rate (<20%)
- Slow search performance
- High node count
- Poor tree reduction

#### Solutions
```rust
// Solution: Make parameters more aggressive
let aggressive_params = PruningParameters {
    futility_margin: [0, 120, 240, 360, 480, 600, 720, 840], // Larger margins
    futility_depth_limit: 4, // Higher depth limit
    lmr_move_threshold: 2, // Lower threshold
    lmr_max_reduction: 4, // Higher max reduction
    delta_margin: 250, // Larger margin
    razoring_margin: 350, // Larger margin
    ..Default::default()
};
```

## Debugging Tools and Techniques

### Debugging Tools

```rust
pub struct DebuggingTools {
    pub performance_profiler: PerformanceProfiler,
    pub memory_analyzer: MemoryAnalyzer,
    pub search_tracer: SearchTracer,
    pub parameter_analyzer: ParameterAnalyzer,
    pub error_logger: ErrorLogger,
}
```

### Debugging Techniques

#### 1. Performance Profiling
```rust
// Profile search performance
let profiler = PerformanceProfiler::new();
profiler.start_profiling();

let result = engine.search(position, time_limit);

let profile = profiler.end_profiling();
println!("Search time: {}ms", profile.search_time);
println!("Nodes searched: {}", profile.nodes_searched);
println!("Pruning rate: {:.2}%", profile.pruning_rate);
```

#### 2. Search Tracing
```rust
// Trace search decisions
let tracer = SearchTracer::new();
tracer.enable_tracing();

let result = engine.search(position, time_limit);

let trace = tracer.get_trace();
for decision in trace.decisions {
    println!("Depth {}: Move {} -> {:?}", 
             decision.depth, 
             decision.move_.to_string(), 
             decision.pruning_decision);
}
```

#### 3. Memory Analysis
```rust
// Analyze memory usage
let memory_analyzer = MemoryAnalyzer::new();
memory_analyzer.start_analysis();

let result = engine.search(position, time_limit);

let analysis = memory_analyzer.end_analysis();
println!("Peak memory: {} MB", analysis.peak_memory);
println!("Cache memory: {} MB", analysis.cache_memory);
println!("Memory leaks: {}", analysis.leaks_detected);
```

## Preventive Measures

### Code Quality

#### 1. Input Validation
```rust
impl PruningManager {
    fn validate_parameters(&self, params: &PruningParameters) -> Result<(), String> {
        // Validate parameter ranges
        if params.futility_depth_limit > 8 {
            return Err("Futility depth limit too high".to_string());
        }
        
        if params.lmr_max_reduction > 5 {
            return Err("LMR max reduction too high".to_string());
        }
        
        // Validate parameter consistency
        if params.lmr_move_threshold > params.lmr_max_reduction {
            return Err("LMR move threshold > max reduction".to_string());
        }
        
        Ok(())
    }
}
```

#### 2. Error Handling
```rust
impl PruningManager {
    fn safe_should_prune(&mut self, state: &SearchState, mv: &Move) -> PruningDecision {
        match self.should_prune(state, mv) {
            Ok(decision) => decision,
            Err(e) => {
                eprintln!("Pruning error: {}", e);
                PruningDecision::Search // Safe fallback
            }
        }
    }
}
```

#### 3. Resource Management
```rust
impl PruningManager {
    fn cleanup_resources(&mut self) {
        // Clean up caches
        self.pruning_cache.clear();
        self.position_cache.clear();
        self.check_cache.clear();
        
        // Reset statistics
        self.statistics.reset();
        
        // Clear adaptive parameters
        self.adaptive_params = None;
    }
}
```

### Testing

#### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_futility_pruning() {
        let params = PruningParameters::default();
        let mut manager = PruningManager::new(params);
        
        let state = SearchState {
            depth: 2,
            static_eval: 100,
            alpha: 200,
            is_in_check: false,
            ..Default::default()
        };
        
        let quiet_move = Move { is_capture: false, gives_check: false, ..Default::default() };
        let decision = manager.should_prune(&state, &quiet_move);
        
        assert_eq!(decision, PruningDecision::Skip);
    }
}
```

#### 2. Integration Tests
```rust
#[test]
fn test_search_with_pruning() {
    let mut engine = SearchEngine::new();
    let position = load_test_position();
    
    let result = engine.search(position, 1000);
    
    assert!(result.best_move.is_some());
    assert!(result.score > i32::MIN);
}
```

#### 3. Performance Tests
```rust
#[test]
fn test_pruning_performance() {
    let mut engine = SearchEngine::new();
    let position = load_test_position();
    
    let start_time = std::time::Instant::now();
    let result = engine.search(position, 1000);
    let duration = start_time.elapsed();
    
    assert!(duration.as_millis() < 2000); // Should complete within 2 seconds
    assert!(engine.get_pruning_rate() > 0.2); // Should have >20% pruning rate
}
```

## Emergency Procedures

### Emergency Shutdown

```rust
impl SearchEngine {
    fn emergency_shutdown(&mut self) {
        // Stop search immediately
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Clean up resources
        self.cleanup_resources();
        
        // Log emergency shutdown
        eprintln!("Emergency shutdown triggered");
    }
}
```

### Recovery Procedures

```rust
impl SearchEngine {
    fn recover_from_error(&mut self, error: &str) -> Result<(), String> {
        // Log error
        eprintln!("Recovery from error: {}", error);
        
        // Reset to safe state
        self.reset_to_safe_state();
        
        // Validate configuration
        self.validate_configuration()?;
        
        // Test basic functionality
        self.test_basic_functionality()?;
        
        Ok(())
    }
    
    fn reset_to_safe_state(&mut self) {
        // Reset to default parameters
        self.pruning_manager.parameters = PruningParameters::default();
        
        // Clear caches
        self.pruning_manager.cleanup_resources();
        
        // Reset statistics
        self.pruning_manager.statistics.reset();
    }
}
```

## Conclusion

This troubleshooting guide provides comprehensive procedures for diagnosing and resolving common issues with the advanced alpha-beta pruning implementation. Key points:

1. **Systematic Approach**: Follow diagnostic procedures step by step
2. **Proper Tools**: Use appropriate debugging tools and techniques
3. **Prevention**: Implement preventive measures to avoid issues
4. **Testing**: Comprehensive testing to catch issues early
5. **Recovery**: Emergency procedures for critical situations

Regular monitoring, proper testing, and systematic debugging ensure the pruning system operates reliably and efficiently. When issues occur, follow the diagnostic procedures outlined in this guide to identify and resolve them quickly.
