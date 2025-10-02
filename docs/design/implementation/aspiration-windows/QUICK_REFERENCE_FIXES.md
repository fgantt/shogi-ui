# Quick Reference: Aspiration Window Fixes

## Critical Code Changes

### 1. Fix Aspiration Window Complete Failure

**File**: `src/search.rs`  
**Location**: Lines 5147-5152  
**Current Code**:
```rust
} else {
    // Search failed completely
    crate::debug_utils::end_timing(&format!("aspiration_search_{}_{}", depth, researches), "ASPIRATION_WINDOW");
    crate::debug_utils::trace_log("ASPIRATION_WINDOW", "Search failed completely");
    break; // ❌ This is wrong
}
```

**Fixed Code**:
```rust
} else {
    // Search failed - widen window and retry instead of giving up
    crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
        "Search failed at research {}, widening window and retrying", researches));
    
    if researches >= search_engine.aspiration_config.max_researches {
        // Only fall back to full-width search after exhausting retries
        crate::debug_utils::trace_log("ASPIRATION_WINDOW", &format!(
            "Max researches ({}) reached, falling back to full-width search", researches));
        current_alpha = i32::MIN + 1;
        current_beta = i32::MAX - 1;
        researches += 1;
        continue;
    } else {
        // Widen window and retry
        search_engine.handle_fail_low(&mut current_alpha, &mut current_beta, 
                                    previous_scores.last().copied().unwrap_or(0), 
                                    search_engine.calculate_window_size(depth, 0, 0));
        researches += 1;
        continue;
    }
}
```

### 2. Fix Move Tracking Bug

**File**: `src/search.rs`  
**Location**: Line 2089  
**Current Code**:
```rust
let mut best_score = alpha; // ❌ This prevents moves below alpha from being tracked
```

**Fixed Code**:
```rust
let mut best_score = i32::MIN + 1; // ✅ Allow tracking of any move, even if below alpha
```

**Additional Fix**: Add after move evaluation loop (around line 2155):
```rust
if best_move.is_none() && !sorted_moves.is_empty() {
    // If no move was better than alpha, use the first move as fallback
    best_move = Some(sorted_moves[0].clone());
    crate::debug_utils::trace_log("SEARCH_AT_DEPTH", 
        "No move exceeded alpha, using first move as fallback");
}
```

### 3. Add Enhanced Debug Logging

**File**: `src/search.rs`  
**Location**: Move evaluation loop (around line 2118)  
**Add after existing logging**:
```rust
// Enhanced move evaluation logging
crate::debug_utils::log_move_eval("SEARCH_AT_DEPTH", &move_.to_usi_string(), score, 
    &format!("move {} of {} (alpha: {}, beta: {}, current_best_score: {})", 
        move_index + 1, sorted_moves.len(), alpha, beta, best_score));

if score > best_score {
    crate::debug_utils::log_decision("SEARCH_AT_DEPTH", "New best move", 
        &format!("Move {} improved score from {} to {} (alpha: {})", 
            move_.to_usi_string(), best_score, score, alpha), 
        Some(score));
    best_score = score;
    best_move = Some(move_.clone());
    
    // Log the new best move details
    crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
        "BEST_MOVE_UPDATE: {} -> {} (score: {}, alpha: {})", 
        move_.to_usi_string(), move_.to_usi_string(), score, alpha));
} else {
    crate::debug_utils::trace_log("SEARCH_AT_DEPTH", &format!(
        "Move {} scored {} (not better than current best: {})", 
        move_.to_usi_string(), score, best_score));
}
```

### 4. Add Search Result Validation

**File**: `src/search.rs`  
**Location**: Add new method to SearchEngine impl block  
**Add this method**:
```rust
fn validate_search_result(&self, result: Option<(Move, i32)>, 
                         depth: u8, alpha: i32, beta: i32) -> bool {
    match result {
        Some((ref move_, score)) => {
            // Validate score is within reasonable bounds
            if score < -50000 || score > 50000 {
                crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                    &format!("WARNING: Score {} is outside reasonable bounds", score));
                return false;
            }
            
            // Validate move is not empty
            if move_.to_usi_string().is_empty() {
                crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                    "WARNING: Empty move string in search result");
                return false;
            }
            
            // Log successful validation
            crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                &format!("Search result validated: move={}, score={}, depth={}", 
                    move_.to_usi_string(), score, depth));
            
            true
        },
        None => {
            crate::debug_utils::trace_log("SEARCH_VALIDATION", 
                &format!("WARNING: Search returned None at depth {} (alpha: {}, beta: {})", 
                    depth, alpha, beta));
            false
        }
    }
}
```

**Integration**: Add validation call in `search_at_depth` before returning (around line 2161):
```rust
let result = best_move.map(|m| (m, best_score));
if !self.validate_search_result(result, depth, alpha, beta) {
    crate::debug_utils::trace_log("SEARCH_AT_DEPTH", 
        "Search result validation failed, attempting recovery");
    // Recovery logic here
}
result
```

## Testing Commands

### Run Specific Tests
```bash
# Test aspiration window functionality
cargo test aspiration_window

# Test search at depth functionality
cargo test search_at_depth

# Run all search-related tests
cargo test search
```

### Test with Specific Position
```bash
# Test with the position from log analysis
cargo test -- --nocapture test_aspiration_window_with_real_position
```

## Validation Steps

### 1. Compile and Test
```bash
cargo check
cargo test
```

### 2. Test with Log Position
- Use position: `lnsgkgsnl/1r5b1/ppppppppp/9/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL w - 1`
- Verify aspiration window doesn't fail completely at depth 3
- Confirm move tracking returns valid moves
- Check that logging provides clear visibility

### 3. Performance Check
```bash
# Run performance benchmarks
cargo test --release aspiration_window_performance
```

## Expected Log Output

After fixes, you should see:
```
[5ms] [ASPIRATION_WINDOW] [5ms] Starting aspiration window search at depth 3 (alpha: 177, beta: 297)
[10ms] [SEARCH_AT_DEPTH] [10ms] Evaluating move 1: 4a4b (alpha: 177, beta: 297, current_best: None)
[15ms] [SEARCH_AT_DEPTH] [15ms] MOVE_EVAL: 4a4b -> 237 (move 1 of 30)
[20ms] [SEARCH_AT_DEPTH] [20ms] BEST_MOVE_UPDATE: 4a4b -> 4a4b (score: 237, alpha: 177)
[25ms] [ASPIRATION_WINDOW] [25ms] Search result: move=4a4b, score=237, alpha=177, beta=297
[30ms] [ASPIRATION_WINDOW] [30ms] DECISION: Success - Score 237 within window [177, 297]
[35ms] [SEARCH_VALIDATION] [35ms] Search result validated: move=4a4b, score=237, depth=3
```

## Rollback Plan

If issues arise, revert these specific changes:
1. Revert aspiration window loop to original break behavior
2. Revert best_score initialization to `alpha`
3. Remove enhanced logging
4. Remove validation calls

## Priority Order

1. **Fix aspiration window failure** (Critical)
2. **Fix move tracking bug** (Critical)
3. **Add enhanced logging** (High)
4. **Add validation** (Medium)

## Success Criteria

- [ ] Aspiration window search never fails completely
- [ ] Move tracking always returns valid moves
- [ ] Search completes successfully at depth 3
- [ ] Logging provides clear visibility into search process
- [ ] No performance regressions
