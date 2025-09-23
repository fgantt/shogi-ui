# Implementation Plan: Opening Book

## 1. Objective

To integrate an opening book into the Rust WASM engine. This will allow the engine to play the first several moves of a game instantly based on a pre-computed library of standard openings. This saves calculation time, improves the quality of early-game play, and introduces strategic variety.

## 2. Background

An opening book is a database that maps board positions to one or more good moves. Instead of calculating moves from scratch in the opening phase, an engine can look up the current position in its book and play a recommended move immediately. This is based on established shogi theory and allows the engine to enter the mid-game in a strong, well-understood position.

The project already contains an `openingBook.json` file, which will be used as the data source for the Rust engine.

## 3. Core Logic and Implementation Plan

The plan involves creating a new module in Rust to handle the opening book, loading the data, and querying it before initiating a search.

### Step 1: Add `serde` for JSON Parsing

To parse the `openingBook.json` file, we need the `serde` and `serde_json` crates.

**File:** `Cargo.toml`

```toml
# Add these lines under [dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Step 2: Create the Opening Book Module

Create a new file `src/opening_book.rs` to encapsulate all logic related to the opening book.

**File:** `src/opening_book.rs`

```rust
use std::collections::HashMap;
use serde::Deserialize;
use crate::types::{Move, Position, PieceType, Player};

#[derive(Deserialize, Debug)]
struct BookMoveJson {
    from: [u8; 2],
    to: [u8; 2],
    piece_type: String, // Assuming piece type is in the JSON
    promotion: bool,
}

#[derive(Deserialize, Debug)]
struct BookEntryJson {
    moves: Vec<BookMoveJson>,
}

pub struct OpeningBook {
    book: HashMap<String, Vec<Move>>,
}

impl OpeningBook {
    pub fn new() -> Self {
        let json_data = include_str!("../ai/openingBook.json");
        let parsed_book: HashMap<String, BookEntryJson> = serde_json::from_str(json_data)
            .expect("Failed to parse openingBook.json");

        let mut book = HashMap::new();

        for (fen, entry) in parsed_book {
            let moves: Vec<Move> = entry.moves.into_iter().map(|m| {
                // Note: The JSON format may need to be adjusted to match this structure.
                // The current openingBook.json seems to have a different format.
                // This code assumes a structure that can be parsed into a Move object.
                let from_pos = Position::new(m.from[0], m.from[1]);
                let to_pos = Position::new(m.to[0], m.to[1]);
                let piece_type = PieceType::from_str(&m.piece_type).unwrap_or(PieceType::Pawn);

                Move {
                    from: Some(from_pos),
                    to: to_pos,
                    piece_type: piece_type,
                    player: Player::Black, // This needs to be determined from context or FEN
                    is_promotion: m.promotion,
                    is_capture: false, // This also needs to be determined
                    captured_piece: None,
                }
            }).collect();
            book.insert(fen, moves);
        }

        Self { book }
    }

    pub fn get_move(&self, fen: &str) -> Option<Move> {
        if let Some(moves) = self.book.get(fen) {
            if !moves.is_empty() {
                // For now, just return the first move. Could be randomized.
                return Some(moves[0].clone());
            }
        }
        None
    }
}
```

### Step 3: Integrate into the Main Engine Logic

Modify `src/lib.rs` to include and use the `OpeningBook`.

**File:** `src/lib.rs`

```rust
// Add the new module
pub mod opening_book;
use opening_book::OpeningBook;

// Add OpeningBook to the ShogiEngine struct
#[wasm_bindgen]
#[derive(Clone)]
pub struct ShogiEngine {
    // ... existing fields
    opening_book: OpeningBook,
}

// In ShogiEngine::new()
#[wasm_bindgen]
impl ShogiEngine {
    pub fn new() -> Self {
        // ...
        Self {
            // ... existing fields
            opening_book: OpeningBook::new(),
        }
    }

    // In get_best_move() or a similar top-level search function
    pub fn get_best_move(&mut self, depth: u8, time_limit_ms: u32, /*...*/) -> Option<Move> {
        // 1. Generate FEN for the current position
        let fen = self.board.to_fen(self.current_player, &self.captured_pieces);

        // 2. Check the opening book first
        if let Some(book_move) = self.opening_book.get_move(&fen) {
            // Need to update the move with the correct player
            let mut final_move = book_move.clone();
            final_move.player = self.current_player;
            return Some(final_move);
        }

        // 3. If no book move, proceed with the search
        // ... existing search logic ...
    }
}
```

## 4. Dependencies and Considerations

*   **JSON Format:** The existing `src/ai/openingBook.json` may not match the `BookMoveJson` struct. The JSON file or the parsing logic will need to be standardized. The current book seems to use a FEN-like key but the move format is an array of objects with `from`, `to`, and `name`. The `name` field likely implies the piece type. The parsing logic in `opening_book.rs` must be written to match the actual JSON structure.
*   **WASM Size:** Embedding the JSON file directly into the WASM binary using `include_str!` will increase its size. For a large opening book, it might be preferable to fetch it from JavaScript and pass it to the engine during initialization.
*   **Player Turn:** The `Move` object requires a `Player`. When parsing the book, the player to move is not immediately known. The FEN key itself contains the active player, so the parsing logic should extract this and create the `Move` objects accordingly, or the player should be set when the move is retrieved, as shown in the example.

## 5. Verification Plan

1.  **Unit Test:** Create a test in `src/opening_book.rs` that loads the book and asserts that a known FEN key returns the expected number of moves.
2.  **Integration Test:** In `src/lib.rs` tests, set up the board to a starting position that is in the opening book. Call `get_best_move` and assert that the returned move is one of the valid book moves and that the function returns almost instantly (i.e., without performing a search).
3.  **Gameplay Test:** Start a new game against the AI. Observe the first few moves. The engine should follow a standard opening sequence from its book. Check the logs to confirm that the moves are being identified as "book moves."
4.  **Logging:** Add a `debug_log` call in `get_best_move` when a book move is found and returned, e.g., `debug_utils::debug_log("Playing move from opening book.");`

