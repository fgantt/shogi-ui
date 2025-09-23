# Implementation Plan: Internal Iterative Deepening (IID)

## 1. Objective

To implement Internal Iterative Deepening (IID), a search technique used to improve move ordering at critical nodes within the search tree. By finding a better move to search first, IID increases the effectiveness of alpha-beta pruning, leading to a more efficient search.

## 2. Background

The efficiency of an alpha-beta search is highly dependent on its move ordering. If the best move is searched first, the algorithm can prune the remaining moves much more effectively. While heuristics like the history heuristic and killer moves are good, they are not always perfect.

IID addresses this by performing a very shallow, quick search at a specific node *before* starting the main, deeper search from that node. The best move found during this shallow IID search is then promoted to be the first move searched in the main search. This significantly increases the probability that the first move searched is the actual best move, leading to more cutoffs.

## 3. Core Logic and Implementation Plan

The logic for IID will be added to the `negamax` function in `src/search.rs`.

### Step 1: Add IID Logic to `negamax`

**File:** `src/search.rs`

```rust
// In the `negamax` function:

// ... after transposition table lookup ...

let mut best_move_from_iid = None;

// 1. Conditions for applying IID
// Only apply IID if the TT didn't provide a move, and we are deep enough in the search.
if depth >= 4 && tt_move.is_none() { 
    // 2. Perform a shallow search (IID)
    let iid_depth = if depth > 6 { 3 } else { 2 }; // Use a small, fixed depth for IID
    
    if let Some((move_, _)) = self.search_at_depth(board, captured_pieces, player, iid_depth, time_limit_ms, alpha, beta) {
        best_move_from_iid = Some(move_);
    }
}

// 3. Modify move sorting to use the IID result
// The `sort_moves` function will need to be adapted to prioritize `best_move_from_iid` if it exists.
let sorted_moves = self.sort_moves(&legal_moves, board, best_move_from_iid.as_ref());

// 4. Proceed with the main search loop
for move_ in sorted_moves {
    // ... existing search logic ...
}
```

### Step 2: Modify `sort_moves` to Prioritize the IID Move

The `score_move` function needs to be updated to give a huge bonus to the move found by IID.

**File:** `src/search.rs`

```rust
// In `impl SearchEngine`

// Modify the sort_moves signature to accept an optional move
fn sort_moves(&self, moves: &[Move], board: &BitboardBoard, priority_move: Option<&Move>) -> Vec<Move> {
    let mut scored_moves: Vec<(Move, i32)> = moves.iter().map(|m| 
        (m.clone(), self.score_move(m, board, priority_move))
    ).collect();
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
    scored_moves.into_iter().map(|(m, _)| m).collect()
}

// Modify the score_move signature and logic
fn score_move(&self, move_: &Move, _board: &BitboardBoard, priority_move: Option<&Move>) -> i32 {
    // Give a massive score to the priority move to ensure it's searched first.
    if let Some(p_move) = priority_move {
        if self.moves_equal(move_, p_move) {
            return i32::MAX;
        }
    }

    let mut score = 0;
    // ... existing scoring logic (captures, promotions, killer moves, etc.)
    score
}
```

## 4. Dependencies and Considerations

*   **Performance Overhead:** IID adds a small search within a larger search, which introduces overhead. It's crucial to only apply it under specific conditions (e.g., sufficient depth, no TT move available) where the potential benefit of improved move ordering outweighs the cost of the shallow search.
*   **IID Depth:** The depth of the internal search is a key parameter. If it's too shallow (e.g., 1-ply), it won't provide reliable information. If it's too deep, it will slow down the overall search too much. A fixed depth of 2 or 3, or a depth relative to the main search (`depth - 2`), are common strategies.
*   **Transposition Table:** IID is most useful when the transposition table does *not* already have a good move for the current position. If a TT move exists, it should be searched first, and IID should be skipped.

## 5. Verification Plan

1.  **Benchmarking:** The primary goal of IID is to make the search more efficient. The number of nodes searched to reach a given depth should decrease, especially in complex positions where move ordering is difficult. This should be measured on a standard test suite.
2.  **Strength Testing:** A more efficient search allows the engine to reach greater depths in the same amount of time. A match against the previous version should show a measurable increase in playing strength.
3.  **Logging and Counters:** Add internal counters to track:
    *   `iid_triggered`: How many times the IID search was performed.
    *   `iid_move_improved_alpha`: How many times the move found by IID was the one that first raised `alpha` in the main search.
    This data will help verify that IID is actively improving the move ordering.

