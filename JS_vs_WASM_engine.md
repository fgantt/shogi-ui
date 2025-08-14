# JavaScript vs. WebAssembly Shogi Engine Comparison

This document outlines the differences between the JavaScript game engine and the WebAssembly (Rust) game engine used in this project.

## JavaScript Engine (`src/game/engine.js`)

The JavaScript engine is responsible for the core game logic and rules enforcement on the client-side.

### Features & Capabilities

*   **Game State Management:** Manages the board state, captured pieces, current player, move history, and game status (check, checkmate, draw).
*   **Move Generation:** Calculates all legal moves for a selected piece.
*   **Move Execution:** Handles piece movement, captures, and promotions.
*   **Rule Implementation:** Enforces all fundamental Shogi rules, including:
    *   Piece movement and capture
    *   Promotion (optional and mandatory)
    *   Drops (with restrictions like *nifu* and no-move drops)
    *   Check and checkmate detection
    *   Draw detection (Sennichite - repetition)
*   **FEN-like State Generation:** Can generate a string representation of the current game state, similar to Forsyth-Edwards Notation (FEN).
*   **No AI:** The engine itself does not contain any AI for computer opponents. It relies on a separate module (`src/ai/computerPlayer.js`) for AI logic, which in turn uses this engine to validate moves.

### Strengths

*   **Simplicity & Readability:** Being plain JavaScript, it's relatively easy to read, understand, and debug for web developers.
*   **Good for UI Interaction:** Excellent for handling user interactions and updating the UI in real-time, as it runs directly in the browser's main thread.
*   **No Compilation Step:** No special build process is required for the JavaScript engine itself, which simplifies the development workflow.

### Weaknesses

*   **Performance:** As an interpreted language, JavaScript is significantly slower than compiled languages like Rust. This is a major drawback for computationally intensive tasks like AI move calculation.
*   **Limited AI Capability:** While it can support a basic AI, it's not suitable for a strong computer opponent that requires deep search algorithms and complex evaluations. The performance limitations would lead to long thinking times for the AI.
*   **Lacks Advanced Engine Features:** Does not implement advanced chess engine techniques like bitboards, sophisticated evaluation functions, or advanced search algorithms.

## WebAssembly (Rust) Engine (`src/*.rs`)

The WebAssembly engine is a high-performance game engine written in Rust, compiled to WASM, and used for the AI opponent.

### Features & Capabilities

*   **High-Performance Architecture:**
    *   **Bitboards:** Uses a bitboard representation for the game state, which allows for extremely fast move generation and board manipulation.
    *   **Zobrist Hashing:** Implements Zobrist hashing for efficient transposition table lookups, avoiding re-computation of already analyzed positions.
*   **Advanced AI Search:**
    *   **Iterative Deepening:** The AI search doesn't just search to a fixed depth, but starts with a shallow search and progressively deepens, allowing it to be interrupted by time limits and still provide a good move.
    *   **Negamax with Alpha-Beta Pruning:** A highly efficient search algorithm that significantly reduces the number of nodes to be evaluated in the search tree.
    *   **Quiescence Search:** To avoid the horizon effect, it extends the search for "noisy" moves (like captures and checks) to ensure the evaluation is based on a stable position.
    *   **Advanced Move Ordering:** Uses techniques like killer moves and a history heuristic to prioritize promising moves, which dramatically improves the effectiveness of alpha-beta pruning.
*   **Sophisticated Evaluation:**
    *   **Piece-Square Tables:** The evaluation function considers not just the material balance but also the position of each piece.
    *   **Complex Evaluation Terms:** The engine evaluates multiple aspects of the position, including pawn structure, king safety, mobility, piece coordination, and center control.
*   **Complete Game Logic:** Implements all Shogi rules for move legality, captures, drops, and game termination.
*   **Directly Powers the AI:** The `get_best_move` function is the core of the AI, using the advanced search and evaluation to find the best possible move within a given time limit and difficulty setting.

### Strengths

*   **Exceptional Performance:** Being compiled from Rust to WebAssembly, it runs at near-native speed in the browser. This is crucial for the demanding computations of a strong chess AI.
*   **Strong AI Opponent:** The combination of advanced search algorithms and a detailed evaluation function allows for a much more challenging and human-like AI opponent.
*   **Scalability:** The engine's architecture is scalable. The AI's strength can be easily adjusted by changing the search depth or time limit.
*   **Memory Efficiency:** Rust's memory management features contribute to a more memory-efficient engine compared to a JavaScript equivalent with complex data structures.

### Weaknesses

*   **Complexity:** The codebase is significantly more complex than the JavaScript engine, requiring knowledge of Rust, bitboards, and advanced chess programming concepts.
*   **Compilation Required:** Any changes to the Rust code require a recompilation step using `wasm-pack`, which adds a layer to the development workflow.
*   **Bridging Overhead:** There is a small overhead in communication between the JavaScript environment and the WebAssembly module, though this is generally negligible compared to the performance gains.

## Summary of Differences

| Feature                  | JavaScript Engine                               | WebAssembly (Rust) Engine                          |
| ------------------------ | ----------------------------------------------- | -------------------------------------------------- |
| **Primary Use Case**     | UI interaction, move validation for human player | Powering the AI opponent                           |
| **Performance**          | Slower (interpreted)                            | Near-native (compiled)                             |
| **Board Representation** | 2D Array                                        | Bitboards                                          |
| **AI Capability**        | Very limited, basic AI at best                  | Strong, with advanced search and evaluation        |
| **Search Algorithm**     | N/A                                             | Iterative Deepening, Negamax with Alpha-Beta       |
| **Evaluation**           | N/A                                             | Complex, with Piece-Square Tables and heuristics   |
| **Development Workflow** | Simple, no compilation                          | More complex, requires `wasm-pack` compilation     |
| **Code Complexity**      | Low to moderate                                 | High                                               |

In essence, the two engines are specialized for different tasks. The **JavaScript engine** serves as a straightforward and easy-to-maintain rulebook for the user-facing part of the game, while the **WebAssembly engine** is a high-performance powerhouse designed to provide a formidable AI opponent.

## Engine Parity Assessment

Bringing the JavaScript engine to parity with the WebAssembly (Rust) engine would be a monumental task, essentially requiring a complete rewrite of the JavaScript engine to replicate the architecture and advanced algorithms of a high-performance chess engine.

Here’s a breakdown of what it would take:

### 1. Fundamental Architectural Overhaul: Bitboards

*   **What it is:** The single most significant change would be to replace the simple 2D array board representation with **bitboards**. A bitboard is a data structure that uses a 64-bit integer (or larger) to represent the entire board, with each bit corresponding to a square. You would have separate bitboards for each piece type and player.
*   **Why it's necessary:** Bitboards allow for incredibly fast operations. For example, generating all possible moves for a piece can be done with a few bitwise operations (AND, OR, XOR, shifts), which are orders of magnitude faster than iterating through a 2D array.
*   **Effort:** This is a massive change that would touch every part of the engine, from move generation to move execution and evaluation.

### 2. Implementing an Advanced AI Search Algorithm

The current JavaScript engine has no AI search. To match the WASM engine, you would need to implement a sophisticated search function from scratch. This includes:

*   **Negamax with Alpha-Beta Pruning:** This is the standard algorithm for game AI. It explores the tree of possible moves, "pruning" branches that are guaranteed to be worse than already-found moves, which drastically reduces the search space.
*   **Iterative Deepening:** Instead of searching to a fixed depth (e.g., 4 moves ahead), you would implement a search that starts at depth 1, then 2, then 3, and so on. This allows the AI to produce a good move even if it's interrupted by a time limit.
*   **Quiescence Search:** To avoid making blunders based on an incomplete tactical sequence (the "horizon effect"), you'd need a special search that continues exploring captures and checks until the position is "quiet."
*   **Transposition Tables with Zobrist Hashing:** To avoid re-analyzing the same board position multiple times, you'd implement a transposition table (a large hash map). This requires generating a unique hash for each board state, typically with Zobrist hashing.

### 3. Building a Sophisticated Evaluation Function

A strong AI doesn't just search deeply; it needs to accurately evaluate the board positions it sees. This means replacing the current lack of evaluation with a complex function that considers:

*   **Piece-Square Tables (PSTs):** These are tables that assign a score to each piece depending on its position on the board (e.g., a knight in the center is more valuable than a knight in the corner).
*   **Game-Specific Heuristics:** You would need to write code to evaluate many other positional factors, such as:
    *   **King Safety:** How exposed is the king? Are there friendly pieces shielding it?
    *   **Pawn Structure:** Are there connected pawns, isolated pawns, or doubled pawns?
    *   **Mobility:** How many legal moves do your pieces have?
    *   **Piece Coordination:** Are your rooks connected? Do you have a bishop pair?
    *   **Center Control:** How well do you control the center of the board?

### 4. Performance Optimization

Even after implementing all the above, the engine would still be running in JavaScript, an interpreted language. You would face a constant battle against the language's performance limitations. This would involve:

*   **Minimizing Object Creation:** Creating new objects (like `move` or `piece` objects) inside the main search loop is very slow. You would need to pre-allocate and reuse objects.
*   **Manual Memory Management:** Carefully managing the transposition table and other data structures to avoid excessive memory usage and garbage collection pauses, which can be fatal for a time-sensitive AI.
*   **Intensive Profiling:** Constantly profiling the code to find and eliminate bottlenecks.

### Conclusion: Is It Worth It?

Bringing the JavaScript engine to parity with the WASM engine would be an immense and highly challenging software engineering project. You would essentially be building a state-of-the-art chess engine from the ground up, in a language that is not well-suited for such computationally intensive tasks.

The current architecture, where **JavaScript handles the UI and user interaction while WebAssembly handles the heavy-lifting of AI computation**, is the standard and most practical approach for high-performance web applications like this one. It leverages the strengths of both technologies, resulting in a responsive user experience and a powerful AI opponent without the near-insurmountable challenge of building a high-performance engine in pure JavaScript.
