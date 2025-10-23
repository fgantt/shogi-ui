# Universal Shogi Interface (USI) Implementation Details

This document outlines the message flow for each USI command within this application, detailing how the command travels through the various layers and components. Our implementation uses Tauri for native desktop integration with USI-compliant engines.

## Architecture Layers

The application is divided into the following layers:

1.  **GUI Layer (React Components):** The user-facing components, primarily `GamePage.tsx`, which renders the game board and handles user input.
2.  **Controller Layer (`ShogiController`):** A TypeScript class (`src/usi/controller.ts`) that acts as the presentation logic layer. It manages the game state using the `tsshogi` library, processes user actions, and handles game logic.
3.  **Tauri Integration Layer:** Direct integration with Tauri backend for USI engine communication. Uses `@tauri-apps/api` to invoke Rust commands and listen to engine events.
4.  **Rust Engine Layer:** The core shogi engine written in Rust and compiled as a native binary. It contains the game logic, search algorithms, and evaluation functions, and communicates via USI protocol.

---

## GUI to Engine Command Flow

These commands originate from the GUI (or the controller acting on the GUI's behalf) and are sent to the engine.

### `usi`

-   **Description:** Tells the engine to enter USI mode.
-   **Flow:**
    1.  **Tauri Engine Spawn:** When the application initializes, it spawns a USI engine via Tauri commands.
    2.  **Engine Initialization:** The engine is initialized and ready to receive USI commands.
    3.  **USI Protocol:** The engine responds with `id` and `usiok` messages via Tauri events.

### `debug`

-   **Description:** Toggles debug mode in the engine.
-   **Current Status:** Not implemented.

### `isready`

-   **Description:** Used to synchronize the GUI and engine.
-   **Flow:**
    1.  **Tauri Engine Check:** The application checks if the engine is ready via Tauri commands.
    2.  **Engine Response:** The engine responds with `readyok` via Tauri events.

### `setoption`

-   **Description:** Sets an internal engine parameter.
-   **Flow:**
    1.  **Tauri Engine Options:** The application sends engine options via Tauri commands.
    2.  **Engine Configuration:** The engine receives and applies the configuration options.

### `register`

-   **Description:** Registers the engine with a name and author.
-   **Current Status:** Not implemented.

### `usinewgame`

-   **Description:** Tells the engine that a new game is starting.
-   **Flow:**
    1.  **`ShogiController.newGame()`:** Called when the user starts a new game.
    2.  **Tauri Engine Reset:** The application sends `usinewgame` command via Tauri.
    3.  **Engine State Reset:** The engine resets its internal state for the new game.

### `position`

-   **Description:** Sets the engine's internal board position.
-   **Flow:**
    1.  **`ShogiController.handleUserMove()`:** Called when the user makes a move.
    2.  **Position Update:** The controller gets the current position SFEN from its internal `tsshogi.Record`.
    3.  **Tauri Position Command:** The application sends the position command via Tauri.
    4.  **Engine State Update:** The engine updates its internal board state.

### `go`

-   **Description:** Starts the engine's search for the best move.
-   **Flow:**
    1.  **`ShogiController`:** After setting the position, it requests a move from the engine.
    2.  **Tauri Go Command:** The application sends the `go` command with search parameters via Tauri.
    3.  **Engine Search:** The engine performs its search algorithm.
    4.  **Best Move Response:** The engine returns the best move via Tauri events.

### `stop`

-   **Description:** Stops the engine's search prematurely.
-   **Current Status:** Implemented via Tauri engine management.

### `ponderhit`

-   **Description:** Tells the engine that the opponent made the expected move during pondering.
-   **Current Status:** Not implemented.

### `ponder`

-   **Description:** Starts pondering (thinking during the opponent's turn).
-   **Current Status:** Not implemented.

### `quit`

-   **Description:** Tells the engine to shut down.
-   **Flow:**
    1.  **`ShogiController.quit()`:** Called when the component unmounts or the application closes.
    2.  **Tauri Engine Cleanup:** The application terminates the engine via Tauri commands.
    3.  **Engine Shutdown:** The engine shuts down gracefully.

---

## Engine to GUI Command Flow

These commands originate from the engine and are sent to the GUI.

### `id`

-   **Description:** Provides the engine's name and author.
-   **Current Status:** Implemented via Tauri engine responses.

### `usiok`

-   **Description:** Acknowledges the `usi` command.
-   **Flow:**
    1.  **Tauri Engine Response:** The engine responds with `usiok` via Tauri events.
    2.  **Application Handling:** The application receives and processes the response.

### `readyok`

-   **Description:** Acknowledges the `isready` command.
-   **Flow:**
    1.  **Tauri Engine Response:** The engine responds with `readyok` via Tauri events.
    2.  **Application Handling:** The application receives and processes the response.

### `bestmove`

-   **Description:** Provides the engine's best move.
-   **Parameters:** A USI-formatted move string (e.g., `7g7f`, `P*5e`).
-   **Flow:**
    1.  **Tauri Engine Response:** The engine responds with the best move via Tauri events.
    2.  **Application Processing:** The application receives the move and applies it to the game state.
    3.  **UI Update:** The game board is updated to reflect the engine's move.

### `checkmate`

-   **Description:** Indicates that the current position is checkmate.
-   **Current Status:** Implemented via game state management.

### `checkmate_solution`

-   **Description:** Provides the sequence of moves leading to checkmate.
-   **Current Status:** Not implemented.

### `checkmate_handicap`

-   **Description:** Indicates that the handicap is too large for the engine to handle.
-   **Current Status:** Not implemented.

### `info`

-   **Description:** Provides search information during calculation (PV, score, depth, etc.).
-   **Current Status:** Implemented via Tauri engine events for search progress.

### `option`

-   **Description:** Declares configurable engine options at startup.
-   **Current Status:** Implemented via Tauri engine configuration.

---

## Tauri Integration Details

The application uses Tauri for native desktop integration with the following key components:

- **Engine Management:** Spawn and manage USI-compliant engines
- **Command Communication:** Send USI commands to engines via Tauri commands
- **Event Handling:** Listen to engine responses via Tauri events
- **Configuration:** Manage engine options and settings
- **Lifecycle Management:** Handle engine initialization, running, and cleanup

This architecture provides better performance and native integration compared to the previous WebAssembly-based approach.