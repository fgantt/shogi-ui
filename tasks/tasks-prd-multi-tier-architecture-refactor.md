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

- [ ] **2.0 Implement the USI-compliant Engine and Controller**
  - [ ] 2.1 Finalize the `EngineAdapter` interface in `src/usi/engine.ts` to ensure it can handle the full USI communication lifecycle.
  - [ ] 2.2 Refactor the AI worker (`src/ai/ai.worker.ts`) to be a USI-compliant engine, responding to commands like `position`, `go`, and `isready`.
  - [ ] 2.3 Implement the `WasmEngineAdapter` in `src/usi/engine.ts` to manage communication with the updated worker.
  - [ ] 2.4 Implement the `ShogiController` in `src/usi/controller.ts` to manage the `tsshogi.Record`, handle moves, and emit state changes.

- [ ] **3.0 Refactor Game UI to use the `ShogiController`**
  - [ ] 3.1 In `src/App.tsx` or a similar top-level component, instantiate the `ShogiController` and `WasmEngineAdapter`.
  - [ ] 3.2 Provide the `ShogiController` instance to the game UI components, either via props or a React Context.
  - [ ] 3.3 Refactor `Board.tsx` to render the board state directly from the `tsshogi.Position` object provided by the controller.
  - [ ] 3.4 Update `Board.tsx` interaction logic to generate USI move strings (e.g., "7g7f", "P*5d") and call `controller.handleUserMove()`.
  - [ ] 3.5 Refactor `CapturedPieces.tsx` and `MoveLog.tsx` to source their data from the `ShogiController`.

- [ ] **4.0 Implement the Engine Management UI**
  - [ ] 4.1 Create the `EngineSettings.tsx` React component.
  - [ ] 4.2 Add a route in the React Router and a link in the main settings UI to navigate to the new panel.
  - [ ] 4.3 Implement the UI to list available engines (the built-in WASM engine will be the first entry).
  - [ ] 4.4 Add UI elements (e.g., a button and file input) to allow users to specify a path to an external engine executable.
  - [ ] 4.5 Implement the client-side logic to save the user's engine selection (e.g., in `localStorage`) and instantiate the correct `EngineAdapter` on application startup.

- [ ] **5.0 Ensure Comprehensive Test Coverage**
  - [ ] 5.1 Write unit tests for `ShogiController` (`src/usi/controller.test.ts`) to cover move validation, engine orchestration, and state changes.
  - [ ] 5.2 Write integration tests for `WasmEngineAdapter` (`src/usi/engine.test.ts`) against a mock worker to verify USI command passing.
  - [ ] 5.3 Write component tests for the refactored `Board.tsx` to ensure it renders correctly and sends the correct USI move strings on user interaction.
  - [ ] 5.4 Write component tests for the new `EngineSettings.tsx` to verify its UI and state management logic.

## Reminders and Todos (from Task 1.0)

The foundational refactoring from Task 1.0 has been completed. The old game state management (`GameState`, `Move`, `Piece` types, and the `game/engine.ts` logic) has been removed. The application is currently in a non-functional state, which is expected at this stage. The following points should be addressed in the subsequent tasks.

### Broken Functionality (To Be Fixed in Tasks 2.0 & 3.0)

*   **`GamePage.tsx`**: This component is currently a placeholder. It needs to be refactored to use the new `ShogiController` for state management and UI rendering.
*   **Modals**: `LoadGameModal.tsx` and `SaveGameModal.tsx` have been temporarily disabled by commenting out their core logic. They need to be re-implemented to work with the new `tsshogi` data structures and `ShogiController`.
*   **`MoveLog.tsx`**: This component has been replaced with a placeholder. It needs to be refactored to display moves from the `tsshogi.Record` object.
*   **AI Engines**: Both the JavaScript AI (`ai.worker.ts`) and the WASM AI (`wasmEngine.ts`) have been stripped of their old logic. The `ai.worker.ts` needs to be rewritten as a USI-compliant engine, and the `WasmEngineAdapter` will need to be implemented to communicate with it.

### Testing Environment

*   The test suite is currently failing. The errors `Worker is not defined` and `Record.newBySFEN is not a function` indicate issues with the `vitest` configuration.
*   **Todo**: Further investigate the `vitest` configuration to ensure it can handle Web Workers and correctly import the `tsshogi` library in a `jsdom` environment. This will be crucial for completing the testing tasks in 5.0.

### Next Steps

*   **Immediate Priority**: Begin Task 2.0 to implement the `ShogiController` and the `EngineAdapter` interface. This is the core of the new architecture.
*   **UI Refactoring**: Once the controller is in place, proceed with Task 3.0 to refactor the UI components to use the new controller.
*   **Testing**: As the new components and services are developed, the corresponding tests should be implemented and fixed to ensure the new architecture is robust.