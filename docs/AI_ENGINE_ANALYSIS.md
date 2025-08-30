# Shogi AI Engine Analysis

This document provides a detailed analysis of the two computer player AI engines implemented in this project: the original JavaScript-based engine and the newer Rust-based WebAssembly (WASM) engine.

## 1. WebAssembly (Rust) Engine

The WebAssembly engine is the primary, high-performance AI for the application. It is written in Rust and compiled to WASM for execution in the browser, providing near-native performance for computationally intensive tasks.

### How It Works

1.  **Core Logic (Rust):** The engine's core logic resides in the `src/` directory, with `lib.rs` as the main entry point.
    *   **`search.rs`**: Implements a sophisticated search algorithm to find the best move. It uses **iterative deepening** combined with a **principal variation search (PVS)**, which is an optimization of the standard alpha-beta pruning algorithm.
    *   **`evaluation.rs`**: Contains the static board evaluation function. It analyzes a given board position and assigns a numerical score based on various heuristics.
    *   **`bitboards.rs`**: Represents the game board using bitboards, a highly efficient data structure for representing board states and calculating piece attacks, which significantly speeds up move generation and evaluation.
    *   **`moves.rs`**: Handles move generation.
    *   **`types.rs`**: Defines the core data structures (Piece, Player, Move, etc.).

2.  **JS/TS Interface (`wasmEngine.ts`):** A TypeScript wrapper (`src/ai/wasmEngine.ts`) provides a clean interface between the React application and the compiled WASM module. It handles:
    *   Initializing the WASM module.
    *   Converting the JavaScript `GameState` object into a format the Rust engine can understand (JSON strings).
    *   Calling the exported `get_best_move` function from the Rust code.
    *   Converting the returned move from the WASM module back into a format the JavaScript game logic can use.

3.  **Execution:** The AI runs in a Web Worker (`src/ai/ai.worker.ts`), ensuring that the complex calculations do not block the main UI thread, which keeps the application responsive.

### Features

*   **High Performance:** Rust and WebAssembly provide performance far superior to what is achievable with JavaScript, allowing for deeper search depths in the same amount of time.
*   **Advanced Search Algorithm:** Implements iterative deepening with Principal Variation Search (a variant of NegaMax with Alpha-Beta pruning).
*   **Advanced Evaluation:** The evaluation function is comprehensive, considering:
    *   Material balance (value of pieces).
    *   Positional value (using Piece-Square Tables, or PSTs).
    *   King safety.
    *   Mobility (number of available moves).
    *   Pawn structure.
    *   Piece coordination (e.g., connected rooks).
*   **Efficient Board Representation:** Uses bitboards for fast state manipulation and move generation.
*   **Non-Blocking:** Runs in a Web Worker to prevent UI freezes.
*   **Transposition Tables:** Caches previously evaluated positions to avoid re-computing scores for the same position reached through different move orders.

### Deficiencies

*   **Complexity:** The build process is more complex, requiring `wasm-pack` to compile the Rust code whenever changes are made.
*   **Integration Overhead:** Requires a data conversion layer to translate game state between JavaScript and Rust, which can be error-prone if not managed carefully.
*   **Debugging:** Debugging WASM code can be more challenging than debugging JavaScript.

### Potential Improvements

*   **Quiescence Search:** The current search could be enhanced with a quiescence search to better evaluate "noisy" positions (positions with many captures or checks) by extending the search until the position is stable.
*   **Opening Book:** While a `openingBook.json` exists, the WASM engine does not currently use it. Integrating an opening book directly into the Rust search would improve early-game play.
*   **Endgame Tablebases:** For positions with very few pieces, pre-calculated endgame tablebases could be used to find the perfect move instantly.

## 2. JavaScript (TypeScript) Engine

The JavaScript engine was the original AI implementation. It is written entirely in TypeScript and runs directly in the browser.

### How It Works

1.  **Core Logic (`ai.worker.ts`):** The main logic is in `src/ai/ai.worker.ts`. It also runs in a worker to avoid blocking the UI.
2.  **Search Algorithm:** It uses a **minimax algorithm with alpha-beta pruning**. Like the WASM engine, it employs iterative deepening to manage search time.
3.  **Evaluation Function:** The `evaluateBoard` function is quite sophisticated for a JS engine. It includes:
    *   Material balance.
    *   Piece-Square Tables (PSTs).
    *   King safety evaluation.
    *   Mobility scoring.
    *   Pawn structure analysis.
    *   An opening book (`openingBook.json`) is used to provide standard opening moves.
4.  **Game Logic (`engine.ts`):** It relies on helper functions from `src/game/engine.ts` to get legal moves, make moves on a temporary board, and check for game-ending conditions like checkmate.

### Features

*   **Simplicity:** Easy to understand, debug, and modify as it's written in the same language as the rest of the frontend.
*   **No Compilation Step:** Changes are reflected instantly with Vite's hot module replacement.
*   **Sophisticated for JS:** The evaluation function and search algorithm are well-featured for a pure JavaScript implementation, including an opening book, move ordering heuristics (MVV-LVA), and killer moves.

### Deficiencies

*   **Performance:** Significantly slower than the WASM engine. JavaScript's interpreted, single-threaded nature (within the worker) is a major bottleneck for the millions of calculations required in a deep chess search.
*   **Shallow Search Depth:** Due to performance limitations, the search depth is severely restricted, leading to weaker, less strategic moves compared to the WASM engine.
*   **Less Efficient Board Representation:** Uses a standard 2D array to represent the board, which is much slower for move generation and state updates than bitboards.

### Potential Improvements

*   The primary path for improvement for the JS engine is its replacement by the WASM engine. Enhancing it further would essentially mean re-implementing features already present and better optimized in the Rust version (like bitboards).

## 3. Comparative Analysis & Path to Parity

| Feature | WebAssembly (Rust) Engine | JavaScript Engine | Winner |
| :--- | :--- | :--- | :--- |
| **Performance** | Near-native speed. | Interpreted, much slower. | **WASM** |
| **Search Algorithm** | Iterative Deepening, PVS (Alpha-Beta). | Iterative Deepening, Alpha-Beta. | **Tie (in theory)** |
| **Search Depth** | Deep (e.g., 6+ plies). | Shallow (e.g., 3-4 plies). | **WASM** |
| **Evaluation** | Advanced (Material, PST, King Safety, Mobility, etc.). | Advanced (Similar heuristics). | **WASM** (due to speed allowing more complex evals) |
| **Board Representation**| Bitboards (very fast). | 2D Array (slow). | **WASM** |
| **Integration** | More complex (requires `wasm-pack`). | Simple (native JS/TS). | **JavaScript** |
| **Opening Book** | Not implemented. | Implemented. | **JavaScript** |
| **Overall Strength** | **Strong** | **Weak** | **WASM** |

### Path to Parity and Future Improvements

The concept of "parity" in this context is not about making the JavaScript engine as strong as the WASM engine, as that would be impractical and defeat the purpose of using WebAssembly. Instead, the goal is to **achieve feature parity by ensuring the superior WASM engine incorporates all the useful features of the JS engine and becomes the sole AI engine.**

The path forward involves these steps:

1.  **Deprecate the JavaScript Engine:** The `computerPlayer.ts` file should be simplified to remove the logic for choosing between engines. It should exclusively use the `WasmEngine`. The `ai.worker.ts` file and its complex logic can be removed, as the WASM engine runs independently.

2.  **Integrate Opening Book into WASM Engine:** The `openingBook.json` data should be loaded and utilized by the Rust code. This will improve the WASM engine's opening play, bringing it to parity with the JS engine's capabilities in this area.

3.  **Implement Quiescence Search in WASM:** Add a quiescence search to the Rust engine to handle tactical positions more accurately, further increasing its strength.

4.  **Consolidate AI Logic:** By focusing all AI development efforts on the Rust codebase, the project can build a much stronger, more efficient, and more capable computer opponent without the maintenance overhead of two separate engines.
