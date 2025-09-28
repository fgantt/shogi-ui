# Implementation Plan: Endgame Tablebases

## 1. Objective

To integrate endgame tablebases into the Rust engine to achieve perfect play in specific, low-piece-count endgame scenarios. This will give the engine infallible endgame technique in the positions covered.

## 2. Background

Endgame tablebases are pre-calculated databases containing the outcome (win, loss, or draw) and the optimal move for every possible position with a small number of pieces. When a position is found in the tablebase, an engine can play the perfect move instantly without needing to search.

**Constraint:** Full tablebases (e.g., for all 5 or 7-piece endgames) are terabytes in size and are not feasible for a web-based application. Therefore, this plan focuses on two potential, limited-scope approaches:

1.  **API-Based Probing:** Using a remote server to look up positions.
2.  **Embedded Micro-Tablebases:** Embedding a very small tablebase for a few critical endgames directly into the WASM module.

This plan will primarily detail the **Embedded Micro-Tablebase** approach, as it is more self-contained and realistic for the current architecture.

## 3. Core Logic and Implementation Plan (Embedded Micro-Tablebase)

This approach will focus on a single, critical endgame: **King + promoted piece vs. lone King (K+X v K)**. This is a common winning endgame that is simple enough to be encoded in a small tablebase.

### Step 1: Generate or Define the Micro-Tablebase

A full tablebase generator is out of scope. Instead, we will manually define the logic for a few simple checkmates, like a King and Gold vs. King (`K+G v K`) endgame, which follows a known pattern.

We will create a new module `src/tablebase.rs` to house this logic.

**File:** `src/tablebase.rs`

```rust
use crate::types::{BitboardBoard, CapturedPieces, Player, Move, PieceType};

/// A structure to handle simple, hardcoded endgame scenarios.
pub struct MicroTablebase;

impl MicroTablebase {
    pub fn new() -> Self {
        MicroTablebase
    }

    /// Probes the tablebase for the current position.
    /// Returns an optimal move if the position matches a known endgame.
    pub fn probe(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> Option<Move> {
        // Only probe if there are exactly 3 pieces on the board and no pieces in hand.
        if board.count_all_pieces() == 3 && captured_pieces.is_empty() {
            return self.probe_kg_vs_k(board, player);
        }
        None
    }

    /// Solves for King + Gold vs lone King.
    fn probe_kg_vs_k(&self, board: &BitboardBoard, player: Player) -> Option<Move> {
        // 1. Identify the pieces and their positions.
        let mut pieces = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                if let Some(piece) = board.get_piece(crate::types::Position::new(r, c)) {
                    pieces.push((piece, crate::types::Position::new(r, c)));
                }
            }
        }

        if pieces.len() != 3 { return None; }

        // 2. Find the lone king and the attacking king + gold.
        // ... (logic to identify which side is which)

        // 3. If it is the attacking player's turn, calculate the optimal move.
        // This involves moving the king to corral the lone king and using the gold
        // to deliver the final checkmate, following a standard algorithm.
        // This logic would be hardcoded here.
        // For example, move the king to reduce the opponent king's space, then deliver mate.

        // This is a placeholder for the actual mating algorithm.
        // A real implementation would calculate the correct move based on the geometry.
        // For now, we return None to indicate the concept.
        None
    }
}
```

### Step 2: Integrate Tablebase Probing into the Search

In `src/lib.rs`, the `ShogiEngine` will own an instance of the `MicroTablebase`. The `get_best_move` function will be modified to probe the tablebase before starting a search.

**File:** `src/lib.rs`

```rust
// Add the new module
pub mod tablebase;
use tablebase::MicroTablebase;

// Add MicroTablebase to the ShogiEngine struct
#[wasm_bindgen]
#[derive(Clone)]
pub struct ShogiEngine {
    // ... existing fields
    tablebase: MicroTablebase,
}

// In ShogiEngine::new()
#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        // ...
        Self {
            // ... existing fields
            tablebase: MicroTablebase::new(),
        }
    }

    // In get_best_move()
    pub fn get_best_move(&mut self, depth: u8, time_limit_ms: u32, /*...*/) -> Option<Move> {
        // 1. Probe the tablebase before doing anything else.
        if let Some(tb_move) = self.tablebase.probe(&self.board, self.current_player, &self.captured_pieces) {
            debug_utils::debug_log("Playing move from endgame tablebase.");
            return Some(tb_move);
        }

        // 2. Check opening book (as implemented previously).
        // ...

        // 3. If no tablebase or book move, proceed with search.
        // ... existing search logic ...
    }
}
```

## 4. Dependencies and Considerations

*   **Complexity:** Implementing the mating algorithms, even for simple endgames, is non-trivial. It requires a deep understanding of endgame theory to encode the logic correctly.
*   **Scope:** This approach is only practical for a handful of the most common and simple checkmating patterns. It will not cover the vast majority of endgame positions.
*   **Alternative (API-Based):** A more powerful but architecturally complex alternative is to use a web service. This would involve:
    *   Making an asynchronous `fetch` call from JavaScript to a server (e.g., `https://shogi-tablebase.com/probe?fen=...`).
    *   This would require the search function to be `async` and for the main thread to `await` the result before starting a normal search. This is a major refactoring.
    *   It also introduces an external dependency and requires an internet connection.

Given the project's current structure, the embedded micro-tablebase is the most feasible starting point.

## 5. Verification Plan

1.  **Unit Tests:** Create test cases in `src/tablebase.rs` for specific `K+G v K` positions. For each position, assert that `probe()` returns the known optimal mating move.
2.  **Integration Test:** In `src/lib.rs` tests, set up a board in a `K+G v K` position. Call `get_best_move` and verify that it returns the tablebase move instantly, without initiating a deeper search.
3.  **Gameplay Test:** Play a game until a supported endgame is reached. Verify that the engine executes the mating sequence perfectly and efficiently.
4.  **Logging:** Add a `debug_log` call when a tablebase move is played to confirm it is working as expected during live gameplay.

