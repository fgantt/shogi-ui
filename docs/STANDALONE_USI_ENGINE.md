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
    option name USI_Hash type spin default 16 min 1 max 1024
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

4.  **Ask the engine to calculate the best move with a time limit:**
    ```
    go btime 10000 wtime 10000 byoyomi 1000
    ```
    *Expected Response:* The engine will search for a move and then output its best move in USI format. You will see `info` strings being printed while it is thinking.
    ```
    info depth 1 score cp 40 time 28 nodes 35 nps 1250 pv ...
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
-   `go [btime | wtime | byoyomi]`: The engine starts searching for the best move from the current position, respecting time controls.
-   `stop`: The engine will stop searching and return the best move found so far.
-   `setoption name <id> value <x>`: The engine can set options. Currently, only `USI_Hash` is supported.
-   `usinewgame`: The engine clears its internal state for a new game.
-   `quit`: The engine will exit gracefully.

### Supported Engine to GUI Responses

-   `id`: The engine sends its name and author.
-   `usiok`: The engine acknowledges the USI mode.
-   `readyok`: The engine signals it is ready for commands.
-   `bestmove`: The engine returns the best move found.
-   `info`: The engine streams search information (depth, score, pv, nodes, nps, time).
-   `option`: The engine declares configurable options at startup.

### Unsupported Commands

The following standard USI commands are not yet implemented:

-   `debug [on | off]`
-   `register`
-   `go ponder`
-   `ponderhit`
-   `gameover`

## Implementation Plan

To achieve a fully functional and compliant USI engine, the following features should be implemented in order.

1.  **Lower Priority Commands:**
    -   **Goal:** Implement the remaining USI commands for full compliance.
    -   **Plan:**
        -   `debug`: Add more verbose logging when enabled.
        -   `gameover`: Inform the engine of the game result, which can be used for learning.
        -   `go ponder` and `ponderhit`: Implement pondering logic.