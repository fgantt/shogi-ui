# Standalone USI Engine Documentation

This document provides instructions on how to run and interact with the standalone Universal Shogi Interface (USI) engine, and outlines the current implementation status and future development plan.

## How to Run the Engine

The engine is built and run using Cargo, the Rust build tool and package manager.

From the root of the project directory, execute the following command in your terminal:

```bash
cargo run --bin usi-engine
```

The engine will compile and start, then wait for USI commands from standard input.

## Sample Session

Here is a sample interactive session to test the basic functionality of the engine. Type each command and press Enter.

1.  **Initialize the engine:**
    ```
    usi
    ```
    *Expected Response:*
    ```
    id name Shogi Engine
    id author Gemini
    usiok
    ```

2.  **Check if the engine is ready:**
    ```
    isready
    ```
    *Expected Response:*
    ```
    readyok
    ```

3.  **Set the board to the starting position and make a move (e.g., P-7f):
    ```
    position startpos moves 7g7f
    ```
    *Expected Response:*
    ```
    info string Board state updated.
    ```

4.  **Ask the engine to calculate the best move:**
    ```
    go
    ```
    *Expected Response:* The engine will search for a move and then output its best move in USI format. The exact move may vary.
    ```
    bestmove 8c8d
    ```

5.  **Quit the engine:**
    ```
    quit
    ```

## USI Command Support Status

This section details which USI commands are supported, which are not, and the plan to implement the remaining features.

### Supported Commands

-   `usi`: The engine correctly identifies itself and acknowledges the USI mode.
-   `isready`: The engine correctly responds when it is ready to receive commands.
-   `position [startpos | sfen ... ] moves ...`: The engine can set the board to the starting position or a custom position from an SFEN string, and then apply a sequence of moves.
-   `go`: The engine starts searching for the best move from the current position. **Note:** Time control parameters (`btime`, `wtime`, `byoyomi`, etc.) are currently ignored.
-   `quit`: The engine will exit gracefully.

### Unsupported Commands

The following standard USI commands are not yet implemented:

-   `debug [on | off]`
-   `setoption name <id> [value <x>]`
-   `register`
-   `usinewgame`
-   `go` (with time controls, depth, nodes, etc.)
-   `stop`
-   `ponderhit`
-   `gameover`

Additionally, the engine does not yet send the following responses:

-   `info ...` (search depth, score, PV, etc.)
-   `option ...` (to declare configurable options)

## Implementation Plan

To achieve a fully functional and compliant USI engine, the following features should be implemented in order. This plan prioritizes features that provide the most utility for use in a standard GUI.

1.  **Implement `go` with Time Controls:**
    -   **Goal:** Make the engine respect the `btime`, `wtime`, and `byoyomi` parameters in the `go` command.
    -   **Plan:** Modify the `handle_go` method to parse these time parameters. The search function (`IterativeDeepening::search`) already accepts a `time_limit_ms`, so the main task is to calculate the appropriate time for the current move based on the remaining time and byoyomi, and pass it to the search.

2.  **Implement `info` Command Streaming:**
    -   **Goal:** Have the engine provide search information (depth, score, principal variation) while it is thinking.
    -   **Plan:** The search loop (`IterativeDeepening::search`) needs to be modified. After each completed depth, it should print an `info` string to stdout with the current search statistics. This will likely require passing a callback or a channel to the search function to send updates without tightly coupling it to `println!`.

3.  **Implement `stop` Command:**
    -   **Goal:** Allow the GUI to stop the engine's search prematurely.
    -   **Plan:** This requires a mechanism for the main command loop to signal the search thread to stop. An `AtomicBool` flag that is checked periodically within the search algorithm is a standard way to achieve this. When the `stop` command is received, the main thread sets the flag to `true`, and the search function will exit gracefully and return the best move found so far.

4.  **Implement `setoption` and `option`:**
    -   **Goal:** Allow users to configure engine parameters like `USI_Hash`.
    -   **Plan:**
        1.  Define configurable options within the `ShogiEngine` struct.
        2.  At startup (after the `usi` command), print an `option` command for each configurable parameter.
        3.  Implement the `handle_setoption` method to parse the `setoption` command and update the corresponding value in the `ShogiEngine`.

5.  **Implement `usinewgame`:**
    -   **Goal:** Ensure the engine state (e.g., transposition table, history heuristics) is cleared between games.
    -   **Plan:** Add a `clear()` or `reset()` method to the relevant search structures. The `handle_usinewgame` method will call this to prepare the engine for a new game.

6.  **Lower Priority Commands:**
    -   **Goal:** Implement the remaining USI commands for full compliance.
    -   **Plan:**
        -   `ponderhit` and `go ponder`: Implement pondering logic.
        -   `gameover`: Inform the engine of the game result, which can be used for learning.
        -   `debug`: Add more verbose logging when enabled.
