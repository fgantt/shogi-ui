## Relevant Files

- `shogi-engine/src/main.rs` - **New:** Main entry point for the standalone USI-compliant shogi engine binary.
- `shogi-engine/Cargo.toml` - **New:** Cargo manifest for the `shogi-engine` crate.
- `src-tauri/src/main.rs` - **New:** Main entry point for the Tauri application backend.
- `src-tauri/src/engine_manager.rs` - **New:** Rust module to manage the lifecycle (spawn, monitor, terminate) of all USI engine processes, including the built-in one.
- `src-tauri/src/usi_protocol.rs` - **New:** Rust module to handle parsing and formatting of USI protocol commands.
- `src-tauri/src/state.rs` - **New:** Rust module to define and manage the shared application state, including the list of configured engines.
- `src-tauri/src/commands.rs` - **New:** Rust module to define all `#[tauri::command]` functions exposed to the frontend.
- `src/components/EngineManagementPage.tsx` - **New:** The main React component for the "Engine Management" screen.
- `src/components/EngineManagementPage.test.tsx` - **New:** Unit tests for the `EngineManagementPage` component.
- `src/hooks/useTauriEvents.ts` - **New:** A custom React hook to subscribe to and handle events from the Rust backend (e.g., engine output).
- `src/App.tsx` - **Modified:** To add a new route for the `EngineManagementPage`.
- `src/components/GamePage.tsx` - **Modified:** To replace WASM worker logic with `invoke` calls to the Tauri backend for interacting with the selected USI engine process.
- `src/components/UsiMonitor.tsx` - **Modified:** To use data from Tauri events instead of props from the old system.
- `src/lib.rs` - **Heavily Modified:** All engine and USI logic will be moved to the new `shogi-engine` crate. WASM bindings will be removed.
- `src/components/EngineSettings.tsx` - **Deleted/Replaced:** The existing engine settings component will be replaced by `EngineManagementPage.tsx`.
- `Cargo.toml` - **Modified:** To add the new `shogi-engine` binary to the workspace, add Tauri dependencies, and remove WASM dependencies.
- `tauri.conf.json` - **Modified:** To configure the `shogi-engine` binary as a sidecar executable.
- `package.json` - **Modified:** To add scripts for running the Tauri development server and building the application.

### Notes

- The built-in engine will be a new binary crate located at `shogi-engine/`.
- The Tauri backend code will live in the `src-tauri` directory.
- All communication with shogi engines (both built-in and external) will be handled by `engine_manager.rs` via the USI protocol.
- There will be no remaining WASM integration.

## Tasks

- [x] 1.0 **Setup Tauri and Basic Application Scaffolding**
  - [x] 1.1 Add the Tauri CLI to the project's dev dependencies (`npm install -D @tauri-apps/cli`).
  - [x] 1.2 Initialize Tauri in the project, creating the `src-tauri` directory and `tauri.conf.json`.
  - [x] 1.3 Configure `tauri.conf.json` to point to the Vite dev server URL (`http://localhost:5173`) and the output directory (`../dist`).
  - [x] 1.4 Update `package.json` with a `tauri:dev` script that runs `npm run dev` and `tauri dev` concurrently.
  - [x] 1.5 Verify that the existing React application loads and runs inside a basic Tauri window.

- [ ] 2.0 **Refactor Rust Engine into a Standalone USI Executable**
  - [ ] 2.1 Create a new binary crate (`shogi-engine`) for the standalone USI engine.
  - [ ] 2.2 Move the core shogi logic from `src/lib.rs`, `src/usi.rs`, and related modules into the new `shogi-engine` crate.
  - [ ] 2.3 Implement a `main` function in the new crate that handles standard input/output to communicate using the USI protocol.
  - [ ] 2.4 Update the main `Cargo.toml` to define the new binary and ensure it's part of the workspace.
  - [ ] 2.5 Configure `tauri.conf.json` to package the compiled `shogi-engine` binary as a sidecar, making it available to the Tauri backend at runtime.
  - [ ] 2.6 Remove all WASM-related code, bindings (`wasm-bindgen`), and dependencies from the project.
  - [ ] 2.7 The Tauri backend will treat the built-in engine as just another USI process, managed by the `engine_manager.rs`. It will be configured by default.

- [ ] 3.0 **Implement Backend USI Engine Process Manager**
  - [ ] 3.1 In `engine_manager.rs`, create a struct to represent a USI engine, holding its process handle, stdin writer, and stdout reader.
  - [ ] 3.2 Implement a function to spawn an engine process (either the sidecar or an external binary) using `tokio::process::Command`.
  - [ ] 3.3 Implement an async task for each spawned engine that continuously reads its stdout and emits its output to the frontend as a Tauri event (e.g., `usi-message::{engine_id}`).
  - [ ] 3.4 Create Tauri commands for `add_engine`, `remove_engine`, and `get_engines` that manage a list of engine configurations stored in the shared state. The built-in engine should be added by default.
  - [ ] 3.5 Create a Tauri command `send_usi_command(engine_id, command)` that writes a given USI command string to the specified engine's stdin.

- [ ] 4.0 **Overhaul Frontend for New USI Architecture**
  - [ ] 4.1 Create the `EngineManagementPage.tsx` component with a UI to list, add (by path), and remove engines.
  - [ ] 4.2 Use Tauri's `invoke` API within `EngineManagementPage.tsx` to call the backend engine management commands.
  - [ ] 4.3 Implement the `useTauriEvents.ts` hook to listen for `usi-message::{engine_id}` events and update the application's state.
  - [ ] 4.4 Integrate the `UsiMonitor.tsx` component to display messages received via the Tauri event listener.
  - [ ] 4.5 Add a dropdown/selector to the `GamePage.tsx` to allow users to choose from any of the configured USI engines (including the default built-in one).
  - [ ] 4.6 Update the game logic to send USI commands to the currently selected engine via the `send_usi_command` Tauri command.

- [ ] 5.0 **Implement Engine-vs-Engine Gameplay Logic**
  - [ ] 5.1 Add a "Engine vs. Engine" mode to the game setup screen.
  - [ ] 5.2 Create a backend loop in Rust, triggered by a Tauri command, that manages the game state for an engine-vs-engine match.
  - [ ] 5.3 The loop will send the current position to the active engine, wait for its `bestmove` response, update the board, and then repeat for the other engine.
  - [ ] 5.4 The backend will emit game state updates (e.g., new move, board position) to the frontend, allowing the user to spectate the match in real-time.
