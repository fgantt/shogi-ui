# Implementation Plan: Late Move Reductions (LMR)

## 1. Objective

To implement Late Move Reductions (LMR), a search optimization technique that reduces the search depth for moves that are ordered later in the move list. This saves a significant amount of computation time by focusing the search on the most promising moves, thereby allowing the engine to search deeper overall.

## 2. Background

LMR is based on the heuristic that if the move ordering is effective, the best moves will be searched first. Moves that appear later in the sorted list are less likely to be good. LMR exploits this by searching these "late" moves with a reduced depth.

If a move searched with a reduced depth happens to be surprisingly good (i.e., it improves upon the current best score), the algorithm assumes it might be an important move and **re-searches** it at the full, original depth to ensure accuracy. This combination of optimistic reduction and selective re-search provides a good balance between speed and accuracy.

## 3. Core Logic and Implementation Plan

The logic for LMR will be implemented inside the main move loop within the `negamax` function in `src/search.rs`.

### Step 1: Modify the `negamax` Move Loop

**File:** `src/search.rs`

```rust
// In the `negamax` function, inside the move iteration loop:

let mut move_index = 0;
for move_ in sorted_moves {
    move_index += 1;
    let score;

    // 1. Define conditions for applying LMR
    let can_reduce = depth >= 3 && move_index > 3 && !move_.is_capture && !board.is_king_in_check(player.opposite(), &new_captured); // Don't reduce moves that give check

    if can_reduce {
        // 2. Apply reduction and perform a reduced-depth search
        // A simple reduction of 1 is a good starting point.
        let reduction = 1;
        
        // Search with a null window around alpha. This is faster and is only to check if the move is better than what we have.
        score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1 - reduction, -alpha - 1, -alpha, start_time, time_limit_ms, history, true);

        // 3. Re-search if the move was better than expected
        if score > alpha {
            // The move was promising, so we must re-search it at the full depth.
            score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms, history, true);
        }
    } else {
        // 4. Perform a full-depth search for promising moves (first few moves, captures, checks)
        score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, start_time, time_limit_ms, history, true);
    }

    // ... (rest of the loop: score updates, alpha/beta updates, etc.)
    if score > best_score {
        best_score = score;
        // ...
    }
}
```
*(Note: The above snippet is a conceptual guide. The `new_board` and `new_captured` would need to be created before this logic block inside the loop)*

### Step 2: Refine Reduction Logic (Advanced)

A static reduction of `1` is a good start. For more advanced tuning, the reduction amount can be made dynamic based on depth and move index.

```rust
// Example of a dynamic reduction formula
let mut reduction = (0.5 + (depth as f32).ln() * (move_index as f32).ln() / 2.0) as u8;
if reduction < 1 { reduction = 1; }
if reduction > depth - 2 { reduction = depth - 2; } // Don't reduce to zero or less
```
This formula can replace the static `let reduction = 1;`.

## 4. Dependencies and Considerations

*   **Move Ordering:** LMR is **highly dependent** on the quality of the move ordering. If `sort_moves` does not consistently place better moves first, LMR can be ineffective or even weaken the engine by reducing the search depth of a critical move.
*   **Exemptions:** It is crucial to exempt certain types of moves from reduction. These include:
    *   Captures
    *   Promotions
    *   Moves that give check
    *   Moves that escape check
    *   Killer moves and transposition table moves
*   **Re-Search:** The re-search step is not optional. Failing to re-search a move that beats alpha will lead to incorrect search results and significantly weaken the engine.
*   **Tuning:** The conditions for applying LMR (e.g., `move_index > 3`) and the reduction formula are sensitive parameters that require careful tuning and benchmarking.

## 5. Verification Plan

1.  **Benchmarking:** The primary goal is to increase search speed. Run the engine on a test suite of positions and measure the change in **nodes per second (NPS)**. A successful LMR implementation should yield a significant increase in NPS.
2.  **Strength Testing:** Conduct a match of at least 100 games between the engine with LMR and the version without it. The LMR-enabled version should show a clear improvement in win rate.
3.  **Logging and Counters:** Add internal counters to the `SearchEngine` struct to track:
    *   `late_moves_reduced`: How many times a reduction was applied.
    *   `researches_triggered`: How many times a reduced move required a full-depth re-search.
    This data is invaluable for tuning the reduction criteria. If the re-search rate is too high, the reductions are too aggressive. If it's too low, they might be too conservative.
4.  **Regression Testing:** Use a set of tactical puzzles where a specific move is required. Ensure that the LMR implementation does not cause the engine to miss these critical moves.

