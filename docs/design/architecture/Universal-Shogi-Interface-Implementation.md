# Universal Shogi Interface (USI) Implementation Details

This document outlines the message flow for each USI command within this application, detailing how the command travels through the various layers and components. Our implementation translates the text-based USI protocol into a series of method calls and events across different architectural layers.

## Architecture Layers

The application is divided into the following layers:

1.  **GUI Layer (React Components):** The user-facing components, primarily `GamePage.tsx`, which renders the game board and handles user input.
2.  **Controller Layer (`ShogiController`):** A TypeScript class (`src/usi/controller.ts`) that acts as the presentation logic layer. It manages the game state using the `tsshogi` library, processes user actions, and orchestrates communication with the engine adapter.
3.  **Engine Adapter Layer (`WasmEngineAdapter`):** A TypeScript class (`src/usi/engine.ts`) that provides a standardized interface for communicating with a USI engine. It abstracts the underlying communication mechanism, which in this case is a Web Worker.
4.  **Worker Layer (`ai.worker.ts`):** A Web Worker (`src/ai/ai.worker.ts`) that hosts the WebAssembly (Wasm) shogi engine. It receives USI commands from the main thread, translates them into calls to the Wasm module, and posts results back.
5.  **Wasm Engine Layer (`ShogiEngine`):** The core shogi engine written in Rust (`src/lib.rs`) and compiled to WebAssembly. It contains the game logic, search algorithms, and evaluation functions.

---

## GUI to Engine Command Flow

These commands originate from the GUI (or the controller acting on the GUI's behalf) and are sent to the engine.

### `usi`

-   **Description:** Tells the engine to enter USI mode.
-   **Flow:**
    1.  **`WasmEngineAdapter.init()`:** This method is called when the application initializes.
    2.  It calls `postCommand('usi')`, sending a `{ command: 'usi' }` message to the worker.
    3.  **`ai.worker.ts`:** The `handleMessage` function receives the command.
    4.  It posts back `{ command: 'id', ... }` and `{ command: 'usiok' }` messages (see Engine to GUI flow).

### `debug`

-   **Description:** Toggles debug mode in the engine.
-   **Current Status:** Not implemented.

### `isready`

-   **Description:** Used to synchronize the GUI and engine.
-   **Flow:**
    1.  **`WasmEngineAdapter.isReady()`:** This method is called during initialization and can be used to ping the engine.
    2.  It calls `postCommand('isready')`, sending a `{ command: 'isready' }` message to the worker.
    3.  **`ai.worker.ts`:** The `handleMessage` function receives the command.
    4.  It immediately posts back a `{ command: 'readyok' }` message.

### `setoption`

-   **Description:** Sets an internal engine parameter.
-   **Flow:**
    1.  **`WasmEngineAdapter.setOptions()`:** Takes an object of options.
    2.  It iterates through the options and calls `postCommand('setoption', { name, value })` for each.
    3.  **`ai.worker.ts`:** The `handleMessage` function receives the command. The `setoption` case is currently a stub and does nothing.

### `register`

-   **Description:** Handles engine registration.
-   **Current Status:** Not applicable/Not implemented.

### `usinewgame`

-   **Description:** Sent before starting a new game.
-   **Flow:**
    1.  **`ShogiController.newGame()`:** Called when the user starts a new game.
    2.  It calls `this.engine.newGame()`, which is the `WasmEngineAdapter`.
    3.  **`WasmEngineAdapter.newGame()`:** Calls `postCommand('usinewgame')`.
    4.  **`ai.worker.ts`:** The `handleMessage` function receives the command. The `usinewgame` case is currently a stub. In a stateful engine, this would be the place to reset engine state (e.g., clear hash tables).

### `position`

-   **Description:** Sets the board position.
-   **Parameters:** A string, typically `sfen <sfenstring> [moves <move1> ...]`.
-   **Flow:**
    1.  **`ShogiController.requestEngineMove()`:** This is called after a user makes a move.
    2.  It gets the current position SFEN from its internal `tsshogi.Record`.
    3.  It calls `this.engine.setPosition(sfen, [])`.
    4.  **`WasmEngineAdapter.setPosition()`:** Formats the arguments into the USI `position` string and calls `postCommand('position', { position: ... })`.
    5.  **`ai.worker.ts`:** The `handleMessage` function receives the command.
    6.  The `handlePosition` helper function is called. It parses the SFEN string, creates JSON objects for the board, player, and captured pieces, and calls `engine.set_position_from_info()` on the Wasm module to update the engine's internal state.

### `go`

-   **Description:** Starts the engine's search.
-   **Parameters:** An object with time control options, e.g., `{ btime, wtime, byoyomi }`.
-   **Flow:**
    1.  **`ShogiController.requestEngineMove()`:** After setting the position, it calls `this.engine.go(options)`.
    2.  **`WasmEngineAdapter.go()`:** Calls `postCommand('go', options)`.
    3.  **`ai.worker.ts`:** The `handleMessage` function receives the command.
    4.  It calls `engine.get_best_move(...)` on the Wasm `ShogiEngine` instance, passing difficulty and time limits.
    5.  The Wasm engine executes its search and returns a `Move` object.
    6.  The worker then calls the `moveToUsi` helper to convert the `Move` object into a USI-formatted string.
    7.  The worker posts a `bestmove` message back to the main thread.

### `stop`

-   **Description:** Stops the engine's search prematurely.
-   **Current Status:** Not implemented. The `WasmEngineAdapter` has a `stop` method, but the worker and Rust engine do not currently have logic to handle it.

### `ponderhit`

-   **Description:** Tells the engine that the expected ponder move was played.
-   **Current Status:** Not implemented.

### `gameover`

-   **Description:** Informs the engine that the game has ended.
-   **Current Status:** Not implemented.

### `quit`

-   **Description:** Shuts down the engine.
-   **Flow:**
    1.  **`ShogiController.quit()`:** Called when the component unmounts or the application closes.
    2.  It calls `this.engine.quit()`.
    3.  **`WasmEngineAdapter.quit()`:** Calls `postCommand('quit')` and then immediately calls `this.worker.terminate()` to kill the worker.
    4.  **`ai.worker.ts`:** The `handleMessage` function receives the `quit` command and calls `self.close()`.

---

## Engine to GUI Command Flow

These commands originate from the engine and are sent to the GUI.

### `id`

-   **Description:** Provides the engine's name and author.
-   **Current Status:** Not implemented in the engine. The worker should send this after receiving the `usi` command.

### `usiok`

-   **Description:** Acknowledges the `usi` command.
-   **Flow:**
    1.  **`ai.worker.ts`:** After receiving a `usi` command, the worker posts back `{ command: 'usiok' }`.
    2.  **`WasmEngineAdapter`:** The `onmessage` handler receives the message.
    3.  It emits a `usiok` event via its `EventEmitter` interface.
    4.  The `init()` method, which was waiting for this event, resolves its promise.

### `readyok`

-   **Description:** Acknowledges the `isready` command.
-   **Flow:**
    1.  **`ai.worker.ts`:** After receiving an `isready` command, the worker posts back `{ command: 'readyok' }`.
    2.  **`WasmEngineAdapter`:** The `onmessage` handler receives the message.
    3.  It emits a `readyok` event.
    4.  The `isReady()` method, which was waiting for this event, resolves its promise.

### `bestmove`

-   **Description:** The engine's best move after a search.
-   **Parameters:** A USI-formatted move string (e.g., `7g7f`, `P*5e`).
-   **Flow:**
    1.  **`ai.worker.ts`:** After the `get_best_move` call returns from Wasm, the worker formats the result into a USI string.
    2.  It posts back `{ command: 'bestmove', move: moveString }`.
    3.  **`WasmEngineAdapter`:** The `onmessage` handler receives the message.
    4.  It emits a `bestmove` event with the move string as an argument: `this.emit('bestmove', { move: moveString })`.
    5.  **`ShogiController`:** The `bestmove` event listener (defined in the constructor) is triggered.
    6.  It calls `this.applyMove(usiMove)` to update its internal game state.
    7.  It calls `this.emitStateChanged()` to notify the UI.
    8.  **`GamePage.tsx`:** The `onStateChanged` listener updates the component's state, causing a re-render to show the engine's move.

### `checkmate`

-   **Description:** Sent in response to a `go mate` command.
-   **Current Status:** Not implemented, as `go mate` is not supported.

### `copyprotection`

-   **Description:** Handles copy protection checks.
-   **Current Status:** Not applicable/Not implemented.

### `registration`

-   **Description:** Handles registration checks.
-   **Current Status:** Not applicable/Not implemented.

### `info`

-   **Description:** Provides search information during calculation (PV, score, depth, etc.).
-   **Current Status:** Not implemented. The Rust engine would need to be modified to calculate this information and a mechanism to stream it back to the worker during the search would be required.

### `option`

-   **Description:** Declares configurable engine options at startup.
-   **Current Status:** Not implemented. The Rust engine would need to define its options, and the worker would need to send them to the GUI after the `usi` command.
