# PRD: Multi-Tier Architecture Refactoring

## 1. Introduction/Overview

This document outlines the requirements for a foundational refactoring of the Shogi application. The project will convert the existing monolithic structure into a modular, multi-tier architecture consisting of a distinct UI layer, an engine layer, and an adapter layer to mediate between them.

This refactoring is critical for the long-term health of the application. It will solve key problems related to code maintainability and development velocity, improve performance by decoupling components, and introduce the highly-requested ability for players to use external, third-party Shogi engines.

## 2. Goals

*   **Architectural Purity:** Convert the application into a clean three-tier architecture (UI, Adapter, Engine).
*   **Standardization:** Ensure all communication between the UI and Engine layers strictly adheres to the Universal Shogi Interface (USI) protocol.
*   **Flexibility:** Allow for "plug-and-play" support for both the in-house WebAssembly engine and external USI-compatible engines.
*   **Maintainability:** Simplify the codebase by using `tsshogi` as the single source of truth for game state, removing redundant data models and conversion logic.
*   **Testability:** Ensure each layer is independently testable and has comprehensive unit test coverage.
*   **User Experience:** Provide a clear UI for users to manage and select their desired Shogi engine.

## 3. User Stories

This refactoring addresses the needs of both developers and players:

*   **Developer:** "As a developer, I want to work with a modular architecture so that I can easily test components in isolation and add new features without breaking existing ones."
*   **Advanced Player:** "As an advanced player, I want to connect my favorite high-strength USI engine to the application so that I can play against a more challenging opponent."
*   **Player:** "As a player, I want a reliable and smooth gameplay experience, free from bugs caused by tightly-coupled code."

## 4. Functional Requirements

### 4.1. Architecture
1.  The application codebase MUST be separated into three distinct layers: UI, USI Adapter, and Engine.
2.  The UI layer MUST NOT directly access any game logic or engine functionality.
3.  The Engine layer MUST NOT have any knowledge of the UI.
4.  All communication between the UI and Engine layers MUST be mediated by the USI Adapter.

### 4.2. Game State
1.  The `tsshogi` library shall be the single source of truth for game state.
2.  The application's custom `GameState`, `Move`, and `Piece` types MUST be removed and replaced with `tsshogi.Record`, `tsshogi.Position`, and `tsshogi.Piece`.
3.  The application MUST use `tsshogi` for all move validation, generation, and state changes (e.g., applying moves, updating hands).

### 4.3. USI Engine & Adapter
1.  The USI Adapter MUST implement the full lifecycle of the USI protocol (initialization, position setup, search commands, etc.).
2.  The system MUST provide an `EngineAdapter` interface that abstracts engine communication.
3.  The system MUST include a `WasmEngineAdapter` implementation for the existing in-house Rust/WASM engine.
4.  The system MUST support external USI-compatible engines.

### 4.4. User Interface
1.  The application MUST include a new **Engine Settings** panel within the settings UI.
2.  This panel MUST display a list of automatically detected or pre-configured engines.
3.  The panel MUST allow a user to add a new external engine by providing a file path to the engine's executable.
4.  The panel MUST allow the user to select which engine to use for gameplay.
5.  The UI MUST be updated to use the new `ShogiController` to get board state and send user moves.

### 4.5. Testing
1.  Each layer (UI, Adapter, Engine) MUST have corresponding unit tests.
2.  Tests for the adapter layer MUST use a mock engine to verify correct USI command sequencing.
3.  UI tests MUST verify that user interactions correctly trigger calls to the controller.

## 5. Non-Goals (Out of Scope)

*   A complete redesign of the existing application's visual theme. Minor UI changes for the engine settings are in scope, but the overall look and feel should be preserved.
*   Adding new game modes (e.g., variants of Shogi).
*   Changes to the fundamental rules of Shogi.
*   Implementing a server-side component for multi-client engine proxying in this phase (though the architecture should allow for it in the future).

## 6. Design Considerations

*   The new Engine Settings panel should be designed to be intuitive for users who may not be highly technical. It should clearly distinguish between the built-in engine and user-added engines.
*   Error handling is important. The UI should gracefully handle cases where a user-provided external engine fails to start or becomes unresponsive.

## 7. Technical Considerations

*   The `EngineAdapter` interface is the key technical enabler for this feature. See the implementation plan for details.
*   The existing WebAssembly engine worker (`ai.worker.ts`) will need to be updated to speak the USI protocol to communicate with the new `WasmEngineAdapter`.
*   Reference documents for implementation:
    *   `tasks/shogi-rules.md`
    *   `docs/Universal-Shogi-Interface.html`
    *   `docs/USI-tsshogi-usage.md`

## 8. Success Metrics

The success of this refactoring will be measured by:
1.  **Completion:** The new architecture is fully implemented and all functional requirements are met.
2.  **Test Coverage:** All new components have comprehensive unit tests that pass.
3.  **Engine Swapping:** The application can be demonstrably switched between the internal WASM engine and at least one major external USI engine (e.g., YaneuraOu) via the new UI.
4.  **Stability & Velocity:** A measurable reduction in regression bugs and/or an increase in development speed on subsequent features.

## 9. Open Questions

*   What is the desired behavior if a user-provided external engine fails to start, crashes, or violates the USI protocol during a game?
*   What is the initial list of third-party engines we want to officially test against and support?

## 10. Implementation Status & Next Steps

**Date:** 2025-09-03

### Completed Work

The foundational refactoring to a three-tier architecture is largely complete.

1.  **Architectural Separation:** The application is now structured into a UI layer (React components), a controller/adapter layer (`ShogiController`, `WasmEngineAdapter`), and an engine layer (the WASM engine in a Web Worker).
2.  **Singleton Controller:** The `ShogiController` has been refactored into a singleton instance that persists for the application's entire session. This solved a complex series of bugs related to React's Strict Mode lifecycle, which was previously creating and destroying engine instances.
3.  **Robust Worker Initialization:** The Web Worker engine now uses an internal command queue. This resolved a race condition where USI commands from the main thread were being sent before the worker was ready to receive them.
4.  **USI Handshake:** The initial USI protocol handshake (`usi`, `isready`, `usinewgame`) between the adapter and the engine is now completing successfully.

### Current Status

We are in the final stage of debugging the application's initial startup. While the architectural refactoring is stable and the engine communication is established, the UI fails to render the board on the very first load.

*   **Current Blocker:** A `TypeError` occurs immediately after the USI `usinewgame` command is processed.
*   **Error Message:** `Uncaught (in promise) TypeError: this.record.position.toSFEN is not a function`
*   **Root Cause:** This error indicates that our code has an incorrect assumption about the data structure of the `tsshogi` library. The `ShogiController` attempts to call a `.toSFEN()` method on the `record.position` object, but that method does not exist there.

### Next Steps

The path to completing the refactoring is clear and well-defined.

1.  **Investigate `tsshogi` Objects:** We have just added `console.log` statements to the `ShogiController` to inspect the true structure of the `record` and `record.position` objects provided by the `tsshogi` library.
2.  **Correct Method Calls:** Based on the logged object structure, we will correct the code in `ShogiController` (e.g., changing `this.record.position.toSFEN()` to the correct method, which may be `this.record.toSFEN()`).
3.  **Verify Render:** Once the `TypeError` is resolved, the controller will be able to correctly emit the initial board state, and the `GamePage` component should render the board successfully.
4.  **Final Validation:** Perform a final round of testing to ensure all functional requirements of the refactoring have been met and the application is stable.
