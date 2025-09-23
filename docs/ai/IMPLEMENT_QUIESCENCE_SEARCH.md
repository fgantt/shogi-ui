# Implementation Plan: Quiescence Search

## 1. Objective

To implement a quiescence search (`q-search`) in the Rust AI engine. This feature will enhance the engine's tactical ability by extending the search for "noisy" moves (like captures and promotions) beyond the nominal search depth. This prevents the "horizon effect," where the engine might miss an immediate, game-altering capture that lies just beyond its fixed search limit.

## 2. Background

A standard alpha-beta search stops at a fixed depth. If a powerful move, like a queen capturing a rook, exists just one ply beyond that depth, the engine won't see it. The static evaluation at the search limit might look good, but it's misleading.

Quiescence search solves this by continuing to search only a limited set of tactical moves (e.g., captures) from leaf nodes until the position becomes "quiet." This ensures the static evaluation is only performed on stable positions, leading to much more accurate tactical calculations.

## 3. Core Logic and Implementation Plan

The implementation will primarily involve modifying `src/search.rs` to call a new quiescence search function at the end of the main search.

### Step 1: Modify `negamax` to Call Quiescence Search

In `src/search.rs`, locate the `negamax` function. The current implementation calls the evaluator directly when `depth == 0`. This will be changed to call the new `quiescence_search` function.

**File:** `src/search.rs`

```rust
// In `negamax` function:
// ...
if depth == 0 {
    // return self.evaluator.evaluate(board, player, captured_pieces); // OLD
    return self.quiescence_search(board, captured_pieces, player, alpha, beta, &start_time, time_limit_ms, 5); // NEW (max q-search depth of 5)
}
// ...
```

### Step 2: Implement the `quiescence_search` Function

Add the new `quiescence_search` function to the `SearchEngine` implementation in `src/search.rs`.

**File:** `src/search.rs`

```rust
// Add this new function within `impl SearchEngine`
fn quiescence_search(&self, board: &BitboardBoard, captured_pieces: &CapturedPieces, player: Player, mut alpha: i32, beta: i32, start_time: &TimeSource, time_limit_ms: u32, depth: u8) -> i32 {
    if self.should_stop(&start_time, time_limit_ms) { 
        return 0; 
    }

    // At max quiescence depth, return static evaluation
    if depth == 0 {
        return self.evaluator.evaluate(board, player, captured_pieces);
    }
    
    // 1. Calculate the "stand-pat" score. This is the score if we do nothing.
    let stand_pat = self.evaluator.evaluate(board, player, captured_pieces);
    
    // 2. Beta cutoff: If the stand-pat score is already too high, we can prune.
    if stand_pat >= beta {
        return beta;
    }
    
    // 3. Update alpha with the stand-pat score.
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    
    // 4. Generate only "noisy" moves (captures, promotions).
    let noisy_moves = self.generate_noisy_moves(board, player, captured_pieces);
    let sorted_noisy_moves = self.sort_moves(&noisy_moves, board); // Reuse existing move sorting

    for move_ in sorted_noisy_moves {
        if self.should_stop(&start_time, time_limit_ms) { break; }
        
        let mut new_board = board.clone();
        let mut new_captured = captured_pieces.clone();
        if let Some(captured) = new_board.make_move(&move_) {
            new_captured.add_piece(captured.piece_type, player);
        }
        
        // 5. Recursively call quiescence_search with decremented depth.
        let score = -self.quiescence_search(&new_board, &new_captured, player.opposite(), -beta, -alpha, &start_time, time_limit_ms, depth - 1);
        
        if score >= beta {
            return beta; // Beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }
    
    alpha
}
```

### Step 3: Implement `generate_noisy_moves`

In `src/moves.rs`, the `MoveGenerator` needs a method to generate only tactical moves. For now, we can define this as captures and promotions.

**File:** `src/moves.rs`

```rust
// In `MoveGenerator` struct:
// ...

// Add this new function
/// Generates moves that are considered "noisy" - captures and promotions.
pub fn generate_noisy_moves(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
    let mut moves = self.generate_legal_captures(board, player, captured_pieces);
    
    // You may also want to add checks and promotions here if not already included in captures.
    // For simplicity, we start with just captures.
    
    moves
}

/// Generates all legal capture moves for a given player.
pub fn generate_legal_captures(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Vec<Move> {
    let mut legal_moves = self.generate_pseudo_legal_moves(board, player, captured_pieces);
    legal_moves.retain(|m| m.is_capture);
    
    // Filter out moves that leave the king in check
    legal_moves.into_iter().filter(|m| {
        let mut temp_board = board.clone();
        temp_board.make_move(m);
        !temp_board.is_king_in_check(player, captured_pieces)
    }).collect()
}
```
*(Note: The `search.rs` file already contains a `generate_noisy_moves` that can be used, so no major changes are needed in `moves.rs`)*

## 4. Dependencies and Considerations

*   **Performance:** Quiescence search can increase the total search time. The `depth` parameter in the `quiescence_search` function acts as a safeguard to prevent infinitely long searches in rare cases. A depth of 5-8 is typical.
*   **Move Generation:** The efficiency of `generate_noisy_moves` is critical. It must be fast to avoid slowing down the engine.
*   **Move Ordering:** The effectiveness of q-search heavily depends on good move ordering. The existing `sort_moves` function should be used, which prioritizes captures (e.g., using MVV-LVA - Most Valuable Victim, Least Valuable Aggressor).

## 5. Verification Plan

1.  **Unit Tests:** Create specific test positions in `tests/` where the engine without q-search would make a blunder (e.g., moving a piece to a square where it can be captured for free). The test should assert that the engine with q-search avoids this blunder.
2.  **Benchmarking:** Run a suite of tactical puzzles (e.g., the "Nakamura test suite" or similar) against the engine before and after the change. The engine should solve a higher percentage of puzzles correctly with q-search enabled.
3.  **Self-Play:** Pit the new engine against the old version in a series of games. The version with quiescence search should have a significantly higher win rate, especially in sharp, tactical games.
4.  **Info Output:** When running in debug mode, log when the quiescence search is entered and the resulting score. This helps verify it is being triggered correctly.

