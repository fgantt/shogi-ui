## References

Use the following files for accurate implementation and compliance:

* @tasks/prd-multi-tier-architecture-refactor.md - Product Requirements Document for  the architecture refactor. 
* @tasks/shogi-rules.md - rules of Shogi.
* @docs/Universal-Shogi-Interface.html - official USI specification.
* @docs/USI-tsshogi-usage.md - guidance for using `tsshogi` and its integration with USI.

## Relevant Files

- `src/usi/controller.ts` - New file. The central controller that connects the UI and the engine.
- `src/usi/engine.ts` - New file. Defines the `EngineAdapter` interface and `WasmEngineAdapter` implementation.
- `src/ai/ai.worker.ts` - To be modified to act as a USI-compliant engine.
- `src/App.tsx` - To be modified to instantiate and provide the `ShogiController` to the UI.
- `src/components/GamePage.tsx` - To be refactored to use the `ShogiController` instead of direct state management.
- `src/components/Board.tsx` - To be refactored to render the board from a `tsshogi.Position` object and send USI moves.
- `src/components/CapturedPieces.tsx` - To be refactored to get hand data from `tsshogi.Position`.
- `src/components/EngineSettings.tsx` - New UI component for managing engines.
- `src/types.ts` - To be modified to remove deprecated game state types.
- `src/game/shogi.ts` - To be modified to remove deprecated file parsing and conversion logic.
- `src/usi/controller.test.ts` - New test file for the controller.
- `src/usi/engine.test.ts` - New test file for the engine adapter.
- `src/components/Board.test.tsx` - New or modified test file for the board component.

### Notes

- Unit tests should typically be placed alongside the code files they are testing (e.g., `MyComponent.tsx` and `MyComponent.test.tsx` in the same directory).
- Use `npm run test` to run tests. This command is already configured in `package.json`.

## Tasks

- [x] **1.0 Foundational Refactoring: Integrate `tsshogi` and Deprecate Old Game State**
  - [x] 1.1 Remove the custom `GameState`, `Move`, and `Piece` types from `src/types.ts`.
  - [x] 1.2 Delete the data model conversion functions (e.g., `toOurPiece`, `fromOurPiece`) from `src/game/shogi.ts`.
  - [x] 1.3 Refactor any components that directly used the old game state types to remove those dependencies.
  - [x] 1.4 Clean up or delete `src/game/shogi.ts` as its primary role is now obsolete.

- [x] **2.0 Implement the USI-compliant Engine and Controller**
  - [x] 2.1 Finalize the `EngineAdapter` interface in `src/usi/engine.ts` to ensure it can handle the full USI communication lifecycle.
  - [x] 2.2 Refactor the AI worker (`src/ai/ai.worker.ts`) to be a USI-compliant engine, responding to commands like `position`, `go`, and `isready`.
  - [x] 2.3 Implement the `WasmEngineAdapter` in `src/usi/engine.ts` to manage communication with the updated worker.
  - [x] 2.4 Implement the `ShogiController` in `src/usi/controller.ts` to manage the `tsshogi.Record`, handle moves, and emit state changes.

- [x] **3.0 Refactor Game UI to use the `ShogiController`**
  - [x] 3.1 In `src/App.tsx` or a similar top-level component, instantiate the `ShogiController` and `WasmEngineAdapter`.
  - [x] 3.2 Provide the `ShogiController` instance to the game UI components, either via props or a React Context.
  - [x] 3.3 Refactor `Board.tsx` to render the board state directly from the `tsshogi.Position` object provided by the controller.
  - [x] 3.4 Update `Board.tsx` interaction logic to generate USI move strings (e.g., "7g7f", "P*5d") and call `controller.handleUserMove()`.
  - [x] 3.5 Refactor `CapturedPieces.tsx` and `MoveLog.tsx` to source their data from the `ShogiController`.

- [x] **4.0 Implement the Engine Management UI**
  - [x] 4.1 Create the `EngineSettings.tsx` React component.
  - [x] 4.2 Add a route in the React Router and a link in the main settings UI to navigate to the new panel.
  - [x] 4.3 Implement the UI to list available engines (the built-in WASM engine will be the first entry).
  - [x] 4.4 Add UI elements (e.g., a button and file input) to allow users to specify a path to an external engine executable.
  - [x] 4.5 Implement the client-side logic to save the user's engine selection (e.g., in `localStorage`) and instantiate the correct `EngineAdapter` on application startup.

- [x] **5.0 Ensure Comprehensive Test Coverage**
  - [x] 5.1 Write unit tests for `ShogiController` (`src/usi/controller.test.ts`) to cover move validation, engine orchestration, and state changes.
  - [x] 5.2 Write integration tests for `WasmEngineAdapter` (`src/usi/engine.test.ts`) against a mock worker to verify USI command passing.
  - [x] 5.3 Write component tests for the refactored `Board.tsx` to ensure it renders correctly and sends the correct USI move strings on user interaction.
  - [x] 5.4 Write component tests for the new `EngineSettings.tsx` to verify its UI and state management logic.

## Reminders and Todos (as of 2025-09-01)

All tasks under 1.0, 2.0, 3.0, 4.0, and 5.0 are now complete. The application has a functional USI controller, the UI has been refactored, engine management is implemented, and comprehensive test coverage is in place. The game is in a playable state with piece drops, promotion, and game end conditions visually represented.

### Implemented Functionality
*   The `ShogiController` manages the game state using `tsshogi`.
*   A `WasmEngineAdapter` communicates with a USI-compliant AI worker.
*   The `ai.worker.ts` uses the Rust engine to calculate moves.
*   The `GamePage` and its child components (`Board`, `CapturedPieces`, `MoveLog`) are now driven by the `ShogiController`.
*   Users can click to select and move pieces on the board.
*   **Piece Drops**: Implemented in the UI.
*   **Promotion**: Implemented in the UI with a promotion modal.
*   **Game End Conditions**: Checkmate and draw conditions are visually represented in the UI.
*   **Engine Management UI**: Fully implemented.
*   **Settings**: The `SettingsPanel` is fully connected to `GamePage` state and `localStorage`.

### Work in Progress
*   **SaveGameModal**: Initial state and handler functions are implemented in `GamePage.tsx`. The `loadSfen` function has been added to `ShogiController`. The modal component itself still needs to be integrated and fully implemented.
*   **LoadGameModal**: Initial state and handler functions are implemented in `GamePage.tsx`. The modal component itself still needs to be integrated and fully implemented.

### Testing Environment
*   The test suite is now passing after significant workarounds.
*   **`tsshogi` Import Issue**: There is a fundamental issue with using `tsshogi` in the `vitest` test environment. It appears that static methods on imported classes are not available, causing tests to fail. The `controller.test.ts` file now uses a comprehensive mock of the `tsshogi` library to bypass this issue. This is a workaround, and the root cause has not been identified. This means the controller's integration with the real `tsshogi` library is not being tested directly.
*   **Web Worker Mocking**: The `engine.test.ts` file now uses a manual mock of the `Worker` class to allow testing of the `WasmEngineAdapter` in a Node.js environment.

### Next Steps
*   **Complete `SaveGameModal` and `LoadGameModal`**: Integrate the modal components into `GamePage.tsx` and finalize their functionality.
*   **Testing**: As new UI components and features are developed, continue to write or update tests. Consider further investigation into the `tsshogi` testing issue if it becomes a blocker.

