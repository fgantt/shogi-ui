# Implementation Plan: Null Move Pruning

## 1. Objective

To implement Null Move Pruning (NMP), a powerful search heuristic, into the Rust engine. The goal is to significantly increase search efficiency by aggressively pruning large portions of the search tree, allowing the engine to search deeper in the same amount of time.

## 2. Background

Null Move Pruning is based on a simple but powerful idea: if you can give the opponent a free move (a "null move") and your position is *still* so good that it causes a beta cutoff, then your original position must be extremely strong. In such cases, it's highly likely that the current search branch will not yield a better move, so we can prune it and stop searching this line further.

This technique is most effective in positions that are not in "zugzwang" (a situation where being forced to move is a disadvantage). In shogi, as in chess, true zugzwang is rare, especially in the middlegame, making NMP a very effective technique.

## 3. Core Logic and Implementation Plan

The implementation will be primarily contained within the `negamax` function in `src/search.rs`.

### Step 1: Add Null Move Pruning Logic to `negamax`

**File:** `src/search.rs`

Modify the `negamax` function signature to prevent recursive null move searches. Then, add the core NMP logic.

```rust
// In `impl SearchEngine`

// Modify the function signature to accept a `can_null_move` flag.
fn negamax(&mut self, board: &mut BitboardBoard, captured_pieces: &CapturedPieces, player: Player, depth: u8, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, history: &mut Vec<String>, can_null_move: bool) -> i32 {
    // ... (existing initial checks like transposition table lookup)

    // === START NULL MOVE PRUNING LOGIC ===

    // 1. Conditions to disable NMP
    let is_in_check = board.is_king_in_check(player, captured_pieces);
    if can_null_move && !is_in_check && depth >= 3 {
        // 2. Define the depth reduction factor (R)
        // A simple static reduction is easiest to implement first.
        let r = 2;

        // 3. Make the "null move" by switching the player and calling search.
        // We pass `can_null_move = false` to prevent recursive null moves.
        let null_move_score = -self.negamax(board, captured_pieces, player.opposite(), depth - 1 - r, -beta, -beta + 1, start_time, time_limit_ms, history, false);

        // 4. Check for cutoff.
        // If the score is >= beta, it means our position is too good. We can prune.
        if null_move_score >= beta {
            return beta; // Prune this branch
        }
    }
    // === END NULL MOVE PRUNING LOGIC ===

    // ... (rest of the existing negamax function, including move generation and iteration)
    // ...
}
```

### Step 2: Update `negamax` Calls

All internal calls to `negamax` must be updated to pass the new `can_null_move` flag.

**File:** `src/search.rs`

```rust
// In `search_at_depth`:
// Initial call should allow null moves.
let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, &start_time, time_limit_ms, &mut history, true);

// In `negamax` (recursive call):
// When iterating through regular moves, the recursive call should also allow null moves.
let score = -self.negamax(&mut new_board, &new_captured, player.opposite(), depth - 1, -beta, -alpha, &start_time, time_limit_ms, history, true);
```

## 4. Dependencies and Considerations

*   **Check Detection:** NMP should **never** be used if the current player is in check, as being in check removes all other legal moves, creating a zugzwang-like situation where passing the turn is not a valid comparison. The `is_king_in_check` check is critical.
*   **Endgame:** NMP can be unreliable in the endgame where zugzwang is more common. A more advanced implementation could disable NMP if the number of pieces on the board is below a certain threshold.
*   **Depth:** NMP is not effective at very shallow search depths. The `depth >= 3` check prevents it from being used where it provides little benefit and could be misleading.
*   **Depth Reduction (R):** The choice of `R` is important. A static `R=2` is a safe starting point. More advanced engines use a dynamic `R` that increases with depth (e.g., `R = 2 + depth / 6`).
*   **Quiescence Search:** NMP should not be used in the quiescence search, as q-search already deals with a limited move set in tactical positions.

## 5. Verification Plan

1.  **Benchmarking (Primary):** The main goal of NMP is speed. The engine should be benchmarked on a standard test suite (like a collection of mid-game positions). The key metric to watch is **nodes per second (NPS)**. With NMP, the engine should be able to search significantly more nodes per second, and as a result, reach greater depths in the same amount of time.
2.  **Strength Testing:** Pit the new engine against the old version in a match of 50-100 games. The version with NMP should have a statistically significant higher win rate.
3.  **Logging:** Add debug logs to count how many times a null-move cutoff occurs during a search. This will confirm the feature is working and provide data for tuning the conditions (like depth and `R` value).
    ```rust
    // In negamax, after a successful cutoff
    if null_move_score >= beta {
        // self.null_move_cutoffs += 1; // If tracking stats
        return beta;
    }
    ```
4.  **Zugzwang Tests:** Create unit tests for known zugzwang positions. Verify that the engine does not perform a null move search in these positions and correctly finds the optimal (and often only) move.

