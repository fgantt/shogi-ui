# Implementation Plan: Aspiration Windows

## 1. Objective

To implement Aspiration Windows in the iterative deepening search process. This technique aims to speed up the alpha-beta search by starting with a narrow search window, which can lead to more frequent beta cutoffs.

## 2. Background

In an iterative deepening search, the score from a shallower depth (e.g., depth `d-1`) is often a very good estimate for the score at the next depth (`d`). A standard search starts with an infinitely wide window (`-infinity`, `+infinity`).

An Aspiration Window search leverages the score from the previous iteration. It "aspires" that the score at the new depth will be close to the old one. It starts with a small search window (e.g., `previous_score - delta`, `previous_score + delta`) around the previous score. If the true score falls within this narrow window, the search completes much faster. If it falls outside (a "fail-high" or "fail-low"), the search must be repeated with a wider window, but this is statistically less common.

## 3. Core Logic and Implementation Plan

The logic will be implemented in the main loop of the `IterativeDeepening::search` method in `src/search.rs`.

### Step 1: Modify `search_at_depth` to Accept a Window

The `search_at_depth` function currently initializes `alpha` and `beta` internally. It needs to be modified to accept them as parameters.

**File:** `src/search.rs`

```rust
// In `impl SearchEngine`
// Modify the function signature
pub fn search_at_depth(&mut self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, time_limit_ms: u32, mut alpha: i32, beta: i32) -> Option<(Move, i32)> {
    self.nodes_searched = 0;
    let start_time = TimeSource::now();
    // Remove the internal alpha/beta initialization
    // let mut alpha = i32::MIN + 1; // REMOVE
    // let beta = i32::MAX - 1;    // REMOVE

    // ... the rest of the function remains the same
}
```

### Step 2: Implement the Aspiration Loop in `IterativeDeepening::search`

**File:** `src/search.rs`

```rust
// In `impl IterativeDeepening`

pub fn search(&mut self, search_engine: &mut SearchEngine, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player) -> Option<(Move, i32)> {
    let start_time = TimeSource::now();
    let mut best_move = None;
    let mut best_score = 0; // Start with 0 for the first aspiration window
    let search_time_limit = self.time_limit_ms.saturating_sub(100);

    for depth in 1..=self.max_depth {
        if self.should_stop(&start_time, search_time_limit) { break; }
        
        let elapsed_ms = start_time.elapsed_ms();
        let remaining_time = search_time_limit.saturating_sub(elapsed_ms);

        // --- Aspiration Window Logic --- 
        let mut alpha;
        let mut beta;
        let delta = 50; // Aspiration window size (tunable)

        if depth > 1 {
            alpha = best_score - delta;
            beta = best_score + delta;
        } else {
            alpha = i32::MIN + 1;
            beta = i32::MAX - 1;
        }

        loop { // This loop handles re-searching on failure
            if let Some((move_, score)) = search_engine.search_at_depth(board, captured_pieces, player, depth, remaining_time, alpha, beta) {
                if score <= alpha { // Fail-low
                    // Search failed low, so widen the window downwards and re-search.
                    alpha = i32::MIN + 1;
                    continue; // Re-search with the new window
                } 
                if score >= beta { // Fail-high
                    // Search failed high, so widen the window upwards and re-search.
                    beta = i32::MAX - 1;
                    continue; // Re-search with the new window
                }

                // Search was successful (score was within the window)
                best_move = Some(move_);
                best_score = score;

                // ... (existing info string reporting logic) ...

                break; // Exit the re-search loop
            } else {
                // search_at_depth returned None, something went wrong or no moves
                return best_move.map(|m| (m, best_score));
            }
        }
        // --- End Aspiration Window Logic ---
    }
    best_move.map(|m| (m, best_score))
}
```

## 4. Dependencies and Considerations

*   **Parameter Tuning:** The size of the aspiration window (`delta`) is a critical parameter. If it's too small, the engine will perform too many re-searches, negating the performance gain. If it's too large, the window won't be tight enough to cause many extra cutoffs. A value between 25 and 50 centipawns is a common starting point.
*   **Score Stability:** Aspiration windows are most effective when scores are relatively stable between iterations. In highly tactical positions, scores can fluctuate wildly, leading to more re-searches.
*   **Code Complexity:** This change adds a `loop` and more complex logic to the main iterative deepening function, which can make it harder to debug.

## 5. Verification Plan

1.  **Benchmarking:** The primary goal is to reduce the total search time for a given depth. Run the engine on a test suite of positions and compare the total time or nodes searched to reach a certain depth. The version with aspiration windows should be faster on average.
2.  **Logging:** Add debug logs to track the aspiration window process:
    *   Log the initial `(alpha, beta)` window for each depth.
    *   Log when a "fail-low" or "fail-high" occurs, forcing a re-search.
    *   Log the new window used for the re-search.
    This data is essential for tuning the `delta` value. If re-searches are happening on more than 5-10% of iterations, the window might be too small.
3.  **Strength Testing:** While mainly a speed optimization, the time saved allows the engine to search deeper in real games. A match against the previous version should show a small but measurable increase in playing strength.

