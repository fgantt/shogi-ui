## Relevant Files

- `src/bin/shogi_engine.rs` - **New:** Standalone USI-compliant engine binary that will be bundled as a Tauri sidecar.
- `src-tauri/src/main.rs` - **New:** Main entry point for the Tauri application backend.
- `src-tauri/src/engine_manager.rs` - **New:** Rust module to manage the lifecycle (spawn, monitor, terminate) of all USI engine processes (both built-in and external).
- `src-tauri/src/usi_protocol.rs` - **New:** Rust module to handle parsing and formatting of USI protocol commands sent to/from all engines.
- `src-tauri/src/engine_validator.rs` - **New:** Rust module to validate USI engines during the add process (send `usi`, parse metadata, check `usiok`).
- `src-tauri/src/engine_storage.rs` - **New:** Rust module to handle persistence of engine configurations to/from JSON file.
- `src-tauri/src/state.rs` - **New:** Rust module to define and manage the shared application state, including the list of configured engines.
- `src-tauri/src/commands.rs` - **New:** Rust module to define all `#[tauri::command]` functions exposed to the frontend.
- `src/components/EngineManagementPage.tsx` - **New:** The main React component for the "Engine Management" screen, allowing users to add, view, and remove engines.
- `src/components/EngineManagementPage.test.tsx` - **New:** Unit tests for the `EngineManagementPage` component.
- `src/hooks/useTauriEvents.ts` - **New:** A custom React hook to subscribe to and handle events from the Rust backend (e.g., engine output).
- `src/App.tsx` - **Modified:** To add a new route for the `EngineManagementPage`.
- `src/components/GamePage.tsx` - **Modified:** To replace WASM worker logic with `invoke` calls to the Tauri backend for game interaction.
- `src/components/UsiMonitor.tsx` - **Modified:** To use data from Tauri events instead of props from the old system.
- `src/components/EngineSettings.tsx` - **Deleted/Replaced:** The existing engine settings component will be replaced by `EngineManagementPage.tsx`.
- `Cargo.toml` - **Modified:** To add Tauri and other Rust dependencies (e.g., `tokio`, `serde`), and define the new binary target.
- `src-tauri/Cargo.toml` - **Modified:** To configure the sidecar binary in the build configuration.
- `src-tauri/tauri.conf.json` - **Modified:** To configure the built-in engine as a sidecar bundle.
- `package.json` - **Modified:** To add scripts for running the Tauri development server and building the application.

### Notes

- The new Rust backend code will live in the `src-tauri` directory, which will be created by the Tauri CLI.
- Unit tests for React components should be created alongside the component files.
- Use `npm run tauri dev` to start the application in development mode.

## Tasks

- [x] 1.0 **Setup Tauri and Basic Application Scaffolding**
  - [x] 1.1 Add the Tauri CLI to the project's dev dependencies (`npm install -D @tauri-apps/cli`).
  - [x] 1.2 Initialize Tauri in the project, creating the `src-tauri` directory and `tauri.conf.json`.
  - [x] 1.3 Configure `tauri.conf.json` to point to the Vite dev server URL (`http://localhost:5173`) and the output directory (`../dist`).
  - [x] 1.4 Update `package.json` with a `tauri:dev` script that runs `npm run dev` and `tauri dev` concurrently.
  - [x] 1.5 Verify that the existing React application loads and runs inside a basic Tauri window.

- [x] 2.0 **Create Standalone Built-in Engine Binary**
  - [x] 2.1 Create `src/bin/shogi_engine.rs` as a new binary target that wraps the existing engine with USI stdio communication.
  - [x] 2.2 Implement a USI protocol handler in the binary that reads from stdin and writes to stdout, using the existing `ShogiEngine` and move generation logic.
  - [x] 2.3 Update `Cargo.toml` to define the new binary target (`[[bin]]` section with `name = "shogi-engine"` and `path = "src/bin/shogi_engine.rs"`).
  - [x] 2.4 Configure `src-tauri/tauri.conf.json` to bundle the engine binary as a sidecar (in the `tauri.bundle.externalBin` array).
  - [x] 2.5 Test the standalone engine binary manually via command line to ensure it responds correctly to USI commands (`usi`, `isready`, `position`, `go`, etc.).

- [x] 3.0 **Implement Backend USI Engine Process Manager**
  - [x] 3.1 In `engine_manager.rs`, create a struct to represent any USI engine (built-in or external), holding its process handle, stdin writer, and stdout reader.
  - [x] 3.2 Implement a function to spawn any engine process using `tokio::process::Command`, supporting both the sidecar built-in engine and external engine executables.
  - [x] 3.3 Implement an async task for each spawned engine that continuously reads its stdout and emits its output to the frontend as a Tauri event (e.g., `usi-message::{engine_id}`).
  - [x] 3.4 Implement timeout handling (5 seconds for init, 60 seconds for moves) and watchdog timers to detect engine hangs or crashes.
  - [x] 3.5 Implement error handling to capture and log stderr output from engines, and emit error events to the frontend.
  - [x] 3.6 Create a Tauri command `send_usi_command(engine_id, command)` that writes a given USI command string to the specified engine's stdin.

- [x] 3.5 **Implement Engine Validation and Persistence**
  - [x] 3.5.1 Create `engine_validator.rs` module with a function to validate USI engines by sending `usi` command and waiting up to 5 seconds for `usiok`.
  - [x] 3.5.2 Parse and extract engine metadata from `id name`, `id author`, and `option` responses during validation.
  - [x] 3.5.3 Create `engine_storage.rs` module to handle persistence of engine configurations to/from JSON file.
  - [x] 3.5.4 Implement platform-appropriate storage paths: `~/.config/shogi-vibe/engines.json` (Linux/macOS) or `%APPDATA%\shogi-vibe\engines.json` (Windows).
  - [x] 3.5.5 Create a Tauri command `add_engine(name, path)` that validates the engine, extracts metadata, saves configuration, and returns the engine info or error.
  - [x] 3.5.6 Create Tauri commands `remove_engine(engine_id)` and `get_engines()` that manage the persisted engine list.
  - [x] 3.5.7 Implement automatic registration of the built-in engine on first application launch if not already present in the configuration.
  - [x] 3.5.8 Implement health checks on application startup for all configured engines, marking engines as unavailable if they fail to respond.

- [x] 4.0 **Overhaul Frontend for Engine Management and Game Interaction**
  - [x] 4.1 Create the `EngineManagementPage.tsx` component with a UI to list, add, and remove engines, based on the PRD.
  - [x] 4.2 Display engine status (Ready, Error, Thinking, Unavailable) and metadata (name, author, supported options) in the engine list.
  - [x] 4.3 Use Tauri's `invoke` API within `EngineManagementPage.tsx` to call the backend engine management commands.
  - [x] 4.4 Implement error handling and display clear error messages when engine validation fails, including any stderr output from the engine.
  - [x] 4.5 Show a loading/validation indicator while adding a new engine.
  - [x] 4.6 Implement the `useTauriEvents.ts` hook to listen for `usi-message` and engine error events, and update the application's state.
  - [x] 4.7 Integrate the `UsiMonitor.tsx` component to display messages received via the Tauri event listener.
  - [x] 4.8 Add a dropdown/selector to the `GamePage.tsx` to allow users to choose between the built-in engine and any configured external engine before starting a game.
  - [x] 4.9 Update the game logic to send commands to the currently selected engine via the appropriate Tauri command.

- [ ] 5.0 **Implement Engine-vs-Engine Gameplay Logic**
  - [ ] 5.1 Add a "Engine vs. Engine" mode to the game setup screen.
  - [ ] 5.2 Create a backend loop in Rust, triggered by a Tauri command, that manages the game state for an engine-vs-engine match.
  - [ ] 5.3 The loop will send the current position to the active engine, wait for its `bestmove` response, update the board, and then repeat for the other engine.
  - [ ] 5.4 The backend will emit game state updates (e.g., new move, board position) to the frontend, allowing the user to spectate the match in real-time.
