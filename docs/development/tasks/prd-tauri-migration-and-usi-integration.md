# PRD: Tauri Migration and USI Engine Integration

## 1. Introduction/Overview

The current web application, while functional, is limited by its WebAssembly (WASM) architecture. This prevents the full utilization of our native Rust engine's capabilities and, crucially, prohibits the integration of third-party USI-compliant Shogi engines.

This document outlines the requirements for migrating the application to a Tauri-based desktop framework. This architectural shift will create a more powerful, extensible, and high-performance platform. The core change is to refactor the built-in Rust engine into a standalone, command-line USI-compliant executable and to build a new interface that can manage any USI-compliant engine as a separate process.

## 2. Goals

*   **Platform Migration:** Transition the application from a web-only platform to a cross-platform desktop application (Windows, macOS, Linux) using Tauri.
*   **Standalone Engine Architecture:** Refactor the built-in Rust engine into a standalone, USI-compliant command-line executable that can be used by any compatible UI.
*   **Decoupled Engine Integration:** The UI will treat the built-in engine as it would any third-party USI engine, communicating with it via standard input/output. It will be included as the default engine.
*   **Eliminate WASM:** Completely remove the existing WASM-based bindings and integration.
*   **Engine Extensibility:** Allow users to add, manage, and play against multiple external, third-party USI-compliant engines.
*   **Enhanced User Experience:** Provide a dedicated and intuitive user interface for managing engine configurations.

## 3. User Stories

*   **As an advanced player,** I want to add my favorite engine (like Apery) to the application so that I can play against a stronger opponent and analyze my games with it.
*   **As a casual player,** I want the application to run smoothly on my desktop so that I can enjoy a responsive and seamless game experience without browser limitations.
*   **As an engine developer,** I want to easily integrate and test my USI-compliant engine with the application so that I can get rapid feedback on its performance and behavior.
*   **As a user,** I want to switch between the bundled built-in engine and my configured external engines so that I can experience different playing styles and levels of difficulty.

## 4. Functional Requirements

| ID    | Requirement                                                                                                                            |
| :---- | :------------------------------------------------------------------------------------------------------------------------------------- |
| FR1   | The application **MUST** be packaged as a desktop application for Windows, macOS, and Linux using Tauri.                                 |
| FR2   | The built-in Rust engine **MUST** be compiled as a standalone USI-compliant executable and bundled with the application using Tauri's sidecar feature. |
| FR3   | The system **MUST** provide a dedicated "Engine Management" UI screen.                                                                 |
| FR4   | From the Engine Management screen, a user **MUST** be able to add a new external USI-compliant engine by providing a name and a file path to its executable. |
| FR4.1 | Engine configurations (engine_id, name, path, custom parameters, last_used timestamp) **MUST** persist across application restarts using platform-appropriate storage (e.g., `~/.config/shogi-vibe/engines.json` on Linux/macOS, AppData on Windows). |
| FR4.2 | The system **MUST** validate that an added engine responds to the `usi` command with `id name` and `usiok` within 5 seconds before adding it to the configuration. |
| FR4.3 | The system **SHOULD** display engine metadata (name, author, supported options) after successful validation of a newly added engine. |
| FR5   | The Engine Management screen **MUST** display a list of all configured engines. The built-in engine **MUST** be included by default.      |
| FR6   | From the Engine Management screen, a user **MUST** be able to remove a previously configured external engine.                           |
| FR7   | The system **MUST** allow the user to select any configured engine (built-in or external) to play against before starting a game.      |
| FR8   | The system **MUST** handle all communication with **all** engines (both built-in and external) consistently using the USI protocol via standard input/output pipes. No engine should be treated differently from a communication perspective. |
| FR9   | The system **SHOULD** allow for engine-vs-engine matches where the user can spectate.                                                  |
| FR10  | The UI **SHOULD** allow users to configure common USI engine parameters (e.g., `Hash`, `Threads`) for each engine, including the built-in engine. |

## 5. Non-Goals (Out of Scope)

*   The existing WASM-based web application and its associated bindings will be entirely removed, not maintained.
*   An in-app marketplace or automatic downloader/installer for third-party engines will not be included in this release.

## 6. Design Considerations

*   The existing web frontend UI will be reused and adapted for the Tauri application to maintain a consistent user experience.
*   A new "Engine Management" screen must be designed and implemented. It should include:
    *   A list of all configured engines with their status (e.g., "Ready," "Error," "Thinking").
    *   An "Add Engine" button that opens a modal or form.
    *   The "Add Engine" form must include fields for a custom Engine Name and a file picker for the executable's path.
    *   Each external engine in the list must have a "Remove" or "Delete" option.
    *   Each engine should have a "Configure" option to set parameters as specified in FR10.

## 7. Technical Considerations

*   The Rust backend (Tauri core) will be responsible for spawning and managing the lifecycle of **all** USI engine processes, including the bundled built-in engine and any user-added external engines.
*   The built-in Rust engine will be compiled as a separate binary (e.g., `src/bin/shogi_engine.rs`) and included in the final application bundle using Tauri's `sidecar` feature. It will communicate via USI stdio like any external engine.
*   Communication between the TypeScript frontend and the Rust backend will be handled via Tauri's `invoke` and `event` system.
*   A robust module must be developed in the Rust backend to manage USI protocol communication, including sending commands, parsing responses, and handling engine state for all engines.
*   **Engine Configuration Persistence:**
    *   Engine configurations will be stored in a JSON file using Tauri's filesystem API or the Tauri Store plugin.
    *   Storage location: `~/.config/shogi-vibe/engines.json` (Linux/macOS) or `%APPDATA%\shogi-vibe\engines.json` (Windows).
    *   The built-in engine will be automatically registered on first launch if not present.
*   **Engine Validation and Health Checks:**
    *   When adding a new engine, the system must send `usi` command and wait up to 5 seconds for `usiok` response.
    *   Parse and store engine metadata from `id name` and `id author` responses.
    *   Implement health checks on application startup for all configured engines with timeout and error handling.
    *   Display clear error messages to users if engines fail to start, crash, or become unresponsive, including any stderr output from the engine.
    *   Implement graceful degradation if the built-in engine fails (e.g., disable AI mode until resolved).
*   **Timeout and Error Handling:**
    *   All engine communication should have configurable timeouts (default: 5 seconds for initialization, 60 seconds for moves).
    *   Implement watchdog timers for engine processes to detect hangs or crashes.
    *   Log all engine communication and errors for debugging purposes.
*   All WASM-related code and dependencies (`wasm-bindgen`, `js-sys`, etc.) will be removed from the project.

## 8. Success Metrics

*   The application successfully builds, packages, and runs on Windows, macOS, and Linux.
*   The built-in engine is successfully compiled into a standalone USI executable.
*   Users can successfully play full games against both the bundled built-in engine and at least one configured external engine.
*   Performance benchmarks show a measurable improvement (e.g., >15%) in engine search speed and UI responsiveness compared to the old WASM implementation.
*   Positive qualitative feedback is received from advanced players and developers via community channels.

## 9. Open Questions

*   ~~What is the desired behavior if an external engine fails to start, crashes, or becomes unresponsive? How is this communicated to the user?~~ **Resolved:** An error message should be shown with any stderr output from the engine. Implemented via FR4.2, FR4.3 and technical considerations.
*   ~~What is the complete list of USI `setoption` parameters we should support in the UI? Should we dynamically query the engine for its supported options?~~ **Resolved:** Yes, dynamically query each engine for its supported options during validation.
*   ~~How should the application handle engines that require specific startup procedures or command-line arguments beyond the standard USI protocol?~~ **Resolved:** Use defaults if possible. Future enhancement could allow custom command-line arguments per engine.
*   ~~Should the built-in engine be embedded in the Tauri process or treated as a separate sidecar binary?~~ **Resolved:** The built-in engine will be a standalone sidecar binary communicating via USI stdio, ensuring consistency with external engines and true architectural decoupling.
