# Design Document: Aspiration Windows

## 1. Overview

This document provides a detailed design for implementing Aspiration Windows in the Shogi engine's iterative deepening search process. Aspiration Windows is a search optimization technique that leverages the score from the previous depth iteration to create a narrow search window, potentially leading to more frequent beta cutoffs and faster search completion.

## 2. Background and Motivation

### 2.1 Current Search Architecture

The current search implementation uses:
- **Iterative Deepening**: Searches from depth 1 to `max_depth` incrementally
- **Alpha-Beta Pruning**: Standard minimax with alpha-beta optimization
- **Transposition Table**: Caches search results for position reuse
- **Late Move Reductions (LMR)**: Reduces search depth for later moves
- **Null Move Pruning**: Skips a move to test position strength
- **Quiescence Search**: Continues search until quiet positions

### 2.2 Problem Statement

In iterative deepening, each depth iteration starts with a full-width alpha-beta window (`-∞`, `+∞`). However, the score from depth `d-1` is often a very good estimate for the score at depth `d`. This creates an opportunity for optimization.

### 2.3 Aspiration Windows Solution

Aspiration Windows uses the previous iteration's score to create a narrow search window around it. If the true score falls within this window, the search completes much faster due to more frequent beta cutoffs. If it falls outside (fail-high or fail-low), the search is repeated with a wider window.

## 3. Design Goals

### 3.1 Primary Goals
- **Performance Improvement**: Reduce total search time by 10-30% on average
- **Maintain Search Quality**: Ensure no degradation in playing strength
- **Minimal Code Complexity**: Integrate cleanly with existing search architecture
- **Configurable Parameters**: Allow tuning of window size and behavior

### 3.2 Secondary Goals
- **Robust Error Handling**: Graceful fallback to full-width search on failures
- **Comprehensive Logging**: Detailed statistics for performance analysis
- **Memory Efficiency**: Minimal additional memory overhead
- **Testing Support**: Easy to test and benchmark

## 4. Technical Design

### 4.1 Core Algorithm

```rust
// Pseudocode for aspiration window logic
for depth in 1..=max_depth {
    if depth == 1 {
        // First iteration: use full-width window
        alpha = -∞
        beta = +∞
    } else {
        // Subsequent iterations: use aspiration window
        delta = aspiration_window_size
        alpha = previous_score - delta
        beta = previous_score + delta
    }
    
    loop {
        result = search_at_depth(board, depth, alpha, beta)
        
        if result.score <= alpha {
            // Fail-low: widen window downward
            alpha = -∞
            continue
        }
        if result.score >= beta {
            // Fail-high: widen window upward  
            beta = +∞
            continue
        }
        
        // Success: score within window
        previous_score = result.score
        break
    }
}
```

### 4.2 Data Structures

#### 4.2.1 AspirationWindowConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspirationWindowConfig {
    /// Enable aspiration windows
    pub enabled: bool,
    /// Base window size in centipawns
    pub base_window_size: i32,
    /// Dynamic window scaling factor
    pub dynamic_scaling: bool,
    /// Maximum window size (safety limit)
    pub max_window_size: i32,
    /// Minimum depth to apply aspiration windows
    pub min_depth: u8,
    /// Enable adaptive window sizing
    pub enable_adaptive_sizing: bool,
    /// Maximum number of re-searches per depth
    pub max_researches: u8,
    /// Enable fail-high/fail-low statistics
    pub enable_statistics: bool,
}
```

#### 4.2.2 AspirationWindowStats

```rust
#[derive(Debug, Clone, Default)]
pub struct AspirationWindowStats {
    /// Total searches performed
    pub total_searches: u64,
    /// Successful searches (no re-search needed)
    pub successful_searches: u64,
    /// Fail-low occurrences
    pub fail_lows: u64,
    /// Fail-high occurrences  
    pub fail_highs: u64,
    /// Total re-searches performed
    pub total_researches: u64,
    /// Average window size used
    pub average_window_size: f64,
    /// Time saved (estimated)
    pub estimated_time_saved_ms: u64,
    /// Nodes saved (estimated)
    pub estimated_nodes_saved: u64,
}
```

### 4.3 Integration Points

#### 4.3.1 SearchEngine Modifications

The `SearchEngine` struct will be extended with:

```rust
pub struct SearchEngine {
    // ... existing fields ...
    aspiration_config: AspirationWindowConfig,
    aspiration_stats: AspirationWindowStats,
    previous_scores: Vec<i32>, // Track scores from previous depths
}
```

#### 4.3.2 IterativeDeepening Modifications

The `IterativeDeepening::search` method will be modified to:

1. **Track Previous Scores**: Maintain a history of scores from previous depths
2. **Calculate Window Size**: Determine appropriate window size based on configuration
3. **Implement Re-search Logic**: Handle fail-high/fail-low scenarios
4. **Collect Statistics**: Track performance metrics for analysis

### 4.4 Window Size Calculation

#### 4.4.1 Static Window Size

```rust
fn calculate_static_window_size(&self, depth: u8) -> i32 {
    if depth < self.aspiration_config.min_depth {
        return i32::MAX; // Use full-width window
    }
    self.aspiration_config.base_window_size
}
```

#### 4.4.2 Dynamic Window Size

```rust
fn calculate_dynamic_window_size(&self, depth: u8, previous_score: i32) -> i32 {
    let base_size = self.aspiration_config.base_window_size;
    
    if !self.aspiration_config.dynamic_scaling {
        return base_size;
    }
    
    // Scale based on depth
    let depth_factor = 1.0 + (depth as f64 - 1.0) * 0.1;
    
    // Scale based on score magnitude (more volatile scores = larger window)
    let score_factor = 1.0 + (previous_score.abs() as f64 / 1000.0) * 0.2;
    
    let dynamic_size = (base_size as f64 * depth_factor * score_factor) as i32;
    
    // Apply limits
    dynamic_size.min(self.aspiration_config.max_window_size)
}
```

#### 4.4.3 Adaptive Window Size

```rust
fn calculate_adaptive_window_size(&self, depth: u8, recent_failures: u8) -> i32 {
    let base_size = self.calculate_dynamic_window_size(depth, 0);
    
    if !self.aspiration_config.enable_adaptive_sizing {
        return base_size;
    }
    
    // Increase window size if recent failures
    let failure_factor = 1.0 + (recent_failures as f64 * 0.3);
    let adaptive_size = (base_size as f64 * failure_factor) as i32;
    
    adaptive_size.min(self.aspiration_config.max_window_size)
}
```

### 4.5 Re-search Logic

#### 4.5.1 Fail-Low Handling

```rust
fn handle_fail_low(&mut self, alpha: &mut i32, beta: &mut i32, 
                   previous_score: i32, window_size: i32) {
    self.aspiration_stats.fail_lows += 1;
    
    // Widen window downward
    *alpha = i32::MIN + 1;
    *beta = previous_score + window_size;
    
    // Log for debugging
    crate::debug_utils::debug_log(&format!(
        "Aspiration: Fail-low at depth {}, widening window to [{}, {}]",
        depth, *alpha, *beta
    ));
}
```

#### 4.5.2 Fail-High Handling

```rust
fn handle_fail_high(&mut self, alpha: &mut i32, beta: &mut i32,
                    previous_score: i32, window_size: i32) {
    self.aspiration_stats.fail_highs += 1;
    
    // Widen window upward
    *alpha = previous_score - window_size;
    *beta = i32::MAX - 1;
    
    // Log for debugging
    crate::debug_utils::debug_log(&format!(
        "Aspiration: Fail-high at depth {}, widening window to [{}, {}]",
        depth, *alpha, *beta
    ));
}
```

### 4.6 Modified Search Flow

#### 4.6.1 IterativeDeepening::search

```rust
pub fn search(&mut self, search_engine: &mut SearchEngine, 
              board: &BitboardBoard, captured_pieces: &CapturedPieces, 
              player: Player) -> Option<(Move, i32)> {
    let start_time = TimeSource::now();
    let mut best_move = None;
    let mut best_score = 0;
    let mut previous_scores = Vec::new();
    let search_time_limit = self.time_limit_ms.saturating_sub(100);

    for depth in 1..=self.max_depth {
        if self.should_stop(&start_time, search_time_limit) { break; }
        
        let elapsed_ms = start_time.elapsed_ms();
        let remaining_time = search_time_limit.saturating_sub(elapsed_ms);

        // Calculate aspiration window parameters
        let (alpha, beta) = if depth == 1 || !search_engine.aspiration_config.enabled {
            // First depth or disabled: use full-width window
            (i32::MIN + 1, i32::MAX - 1)
        } else {
            // Use aspiration window based on previous score
            let previous_score = previous_scores.last().copied().unwrap_or(0);
            let window_size = search_engine.calculate_window_size(depth, previous_score);
            (previous_score - window_size, previous_score + window_size)
        };

        // Perform search with aspiration window
        let mut search_result = None;
        let mut researches = 0;
        let mut current_alpha = alpha;
        let mut current_beta = beta;

        loop {
            if researches >= search_engine.aspiration_config.max_researches {
                // Fall back to full-width search
                current_alpha = i32::MIN + 1;
                current_beta = i32::MAX - 1;
            }

            if let Some((move_, score)) = search_engine.search_at_depth(
                board, captured_pieces, player, depth, remaining_time,
                current_alpha, current_beta
            ) {
                search_result = Some((move_, score));
                
                if score <= current_alpha {
                    // Fail-low: widen window downward
                    search_engine.handle_fail_low(&mut current_alpha, &mut current_beta, 
                                                previous_scores.last().copied().unwrap_or(0), 
                                                search_engine.calculate_window_size(depth, 0));
                    researches += 1;
                    continue;
                }
                
                if score >= current_beta {
                    // Fail-high: widen window upward
                    search_engine.handle_fail_high(&mut current_alpha, &mut current_beta,
                                                 previous_scores.last().copied().unwrap_or(0),
                                                 search_engine.calculate_window_size(depth, 0));
                    researches += 1;
                    continue;
                }
                
                // Success: score within window
                best_move = Some(move_);
                best_score = score;
                previous_scores.push(score);
                break;
            } else {
                // Search failed completely
                break;
            }
        }

        // Update statistics
        search_engine.update_aspiration_stats(researches > 0, researches);

        // ... existing info reporting logic ...
    }

    best_move.map(|m| (m, best_score))
}
```

#### 4.6.2 SearchEngine::search_at_depth

The `search_at_depth` method needs to be modified to accept alpha and beta parameters:

```rust
pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, 
                      player: Player, depth: u8, time_limit_ms: u32, 
                      alpha: i32, beta: i32) -> Option<(Move, i32)> {
    // ... existing implementation with alpha/beta parameters ...
}
```

## 5. Configuration and Tuning

### 5.1 Default Configuration

```rust
impl Default for AspirationWindowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_window_size: 50,        // 50 centipawns
            dynamic_scaling: true,
            max_window_size: 200,        // 200 centipawns
            min_depth: 2,                // Start at depth 2
            enable_adaptive_sizing: true,
            max_researches: 2,           // Allow up to 2 re-searches
            enable_statistics: true,
        }
    }
}
```

### 5.2 Tuning Guidelines

#### 5.2.1 Window Size Tuning
- **Too Small**: High re-search rate, negating performance gains
- **Too Large**: Fewer beta cutoffs, minimal performance improvement
- **Optimal Range**: 25-75 centipawns for most positions

#### 5.2.2 Depth Threshold
- **Too Early**: May cause instability in shallow searches
- **Too Late**: Misses optimization opportunities
- **Recommended**: Start at depth 2-3

#### 5.2.3 Re-search Limits
- **Too Low**: May fall back to full-width too often
- **Too High**: May waste time on excessive re-searches
- **Recommended**: 1-3 re-searches maximum

### 5.3 Performance Monitoring

#### 5.3.1 Key Metrics
- **Success Rate**: Percentage of searches that don't require re-search
- **Re-search Rate**: Average number of re-searches per depth
- **Time Savings**: Estimated time saved compared to full-width search
- **Node Reduction**: Estimated nodes saved due to beta cutoffs

#### 5.3.2 Adaptive Tuning

```rust
impl AspirationWindowStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_searches == 0 { 0.0 }
        else { self.successful_searches as f64 / self.total_searches as f64 }
    }
    
    pub fn research_rate(&self) -> f64 {
        if self.total_searches == 0 { 0.0 }
        else { self.total_researches as f64 / self.total_searches as f64 }
    }
    
    pub fn efficiency(&self) -> f64 {
        // Higher is better: more time saved, fewer re-searches
        let time_savings = self.estimated_time_saved_ms as f64;
        let research_penalty = self.total_researches as f64 * 10.0; // Penalty for re-searches
        time_savings - research_penalty
    }
}
```

## 6. Error Handling and Fallbacks

### 6.1 Graceful Degradation

The implementation includes multiple fallback mechanisms:

1. **Configuration Disabled**: Falls back to full-width search
2. **Excessive Re-searches**: Falls back to full-width search after max_researches
3. **Invalid Parameters**: Uses safe defaults
4. **Search Failures**: Returns previous best result

### 6.2 Error Recovery

```rust
fn handle_search_error(&mut self, error: SearchError) -> SearchResult {
    match error {
        SearchError::Timeout => {
            // Return best result so far
            self.get_best_result()
        },
        SearchError::InvalidWindow => {
            // Fall back to full-width search
            self.search_with_full_window()
        },
        SearchError::ExcessiveResearches => {
            // Use last successful result
            self.get_last_successful_result()
        }
    }
}
```

## 7. Testing Strategy

### 7.1 Unit Tests

- **Window Size Calculation**: Test static, dynamic, and adaptive sizing
- **Re-search Logic**: Test fail-high and fail-low handling
- **Configuration Validation**: Test parameter validation and defaults
- **Statistics Collection**: Test metric calculation and accuracy

### 7.2 Integration Tests

- **Search Quality**: Compare move quality with and without aspiration windows
- **Performance Benchmarks**: Measure time and node count improvements
- **Edge Cases**: Test with extreme positions and time limits
- **Memory Usage**: Verify minimal memory overhead

### 7.3 Performance Tests

- **Position Suite**: Test on standard Shogi test positions
- **Time Control**: Test with various time limits
- **Depth Scaling**: Test performance across different search depths
- **Statistical Analysis**: Verify performance improvements are statistically significant

## 8. Implementation Plan

### 8.1 Phase 1: Core Implementation
1. Add configuration structures to `types.rs`
2. Extend `SearchEngine` with aspiration window fields
3. Modify `search_at_depth` to accept alpha/beta parameters
4. Implement basic aspiration window logic in `IterativeDeepening::search`

### 8.2 Phase 2: Advanced Features
1. Implement dynamic and adaptive window sizing
2. Add comprehensive statistics collection
3. Implement re-search logic with proper error handling
4. Add performance monitoring and tuning utilities

### 8.3 Phase 3: Testing and Optimization
1. Create comprehensive test suite
2. Perform performance benchmarking
3. Tune default parameters based on test results
4. Add configuration presets for different playing styles

### 8.4 Phase 4: Integration and Documentation
1. Integrate with existing search optimizations
2. Add logging and debugging support
3. Create user documentation
4. Performance validation and final tuning

## 9. Expected Performance Impact

### 9.1 Performance Improvements
- **Time Reduction**: 10-30% faster search on average
- **Node Reduction**: 15-40% fewer nodes searched
- **Memory Efficiency**: Minimal additional memory overhead
- **Quality Maintenance**: No degradation in playing strength

### 9.2 Position-Dependent Benefits
- **Tactical Positions**: Moderate improvement (10-15%)
- **Positional Positions**: High improvement (20-35%)
- **Endgame Positions**: Variable improvement (5-25%)
- **Complex Positions**: Lower improvement due to more re-searches

### 9.3 Hardware Considerations
- **Fast Processors**: Higher benefit due to better beta cutoff utilization
- **Memory-Constrained Systems**: Lower benefit due to cache effects
- **Multi-threaded Search**: Compatible with parallel search implementations

## 10. Future Enhancements

### 10.1 Advanced Features
- **Machine Learning Tuning**: Use ML to optimize window sizes
- **Position-Specific Windows**: Different windows for different position types
- **Multi-PV Integration**: Aspiration windows for multiple principal variations
- **Transposition Table Integration**: Use TT data to improve window sizing

### 10.2 Research Directions
- **Adaptive Algorithms**: Self-tuning window size algorithms
- **Statistical Models**: Predictive models for optimal window sizing
- **Hardware Optimization**: SIMD-optimized window calculations
- **Parallel Search**: Aspiration windows in multi-threaded search

## 11. Conclusion

Aspiration Windows represents a well-established optimization technique that can provide significant performance improvements with minimal risk to search quality. The proposed design integrates cleanly with the existing search architecture while providing comprehensive configuration options and robust error handling.

The implementation follows the established patterns in the codebase and maintains compatibility with existing search optimizations. With proper tuning and testing, aspiration windows should provide measurable performance improvements across a wide range of Shogi positions.
