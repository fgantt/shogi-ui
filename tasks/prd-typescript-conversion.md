# PRD: Convert Project to TypeScript

## 1. Introduction/Overview

This document outlines the requirements for converting the Shogi game project from JavaScript to TypeScript. Given the complexity of the game logic and the critical interface between the React UI and the Wasm engine, this conversion will significantly improve code quality, reduce bugs, and enhance developer productivity. The goal is to perform an incremental migration, starting with the most critical parts of the codebase—the core game logic and state management—before converting the entire application.

## 2. Goals

The primary objectives of this conversion are to:

*   **Improve Stability:** Leverage TypeScript's static typing to catch errors during development, reducing the number of runtime bugs.
*   **Enhance Developer Experience:** Provide developers with benefits like IDE autocompletion, type-aware feedback, and safer refactoring capabilities.
*   **Strengthen Wasm Integration:** Create a type-safe boundary between the frontend and the Rust/Wasm game engine by using generated type definitions.

## 3. User Stories

*   **As a developer, I want to see type definitions for the game state and component props so that I can catch errors during development and get autocomplete in my IDE, reducing bugs and speeding up my workflow.**

## 4. Functional Requirements

### Phase 1: Core Logic and Setup

1.  **Environment Setup:**
    *   Install the necessary development dependencies: `typescript`, `@types/react`, `@types/react-dom`, and `@types/node`.
    *   Initialize a `tsconfig.json` file using `npx tsc --init`.
    *   Configure `tsconfig.json` to support a gradual migration. Key settings should include `"strict": true` and `"allowJs": true`.

2.  **Core Logic Conversion:**
    *   Begin by converting the core game logic files.
    *   Rename all `.js` files in `src/game/` and `src/ai/` to `.ts`.
    *   Define and apply TypeScript interfaces for all core data structures, such as `Piece`, `Move`, and `GameState`.
    *   Work through the files to resolve all TypeScript compiler errors until the code is type-safe.

3.  **Wasm Integration:**
    *   Update the `wasm-pack` build command to generate TypeScript definition files (`.d.ts`) for the Rust engine.
    *   Ensure the Wasm module is imported and utilized in a type-safe manner within the newly converted `.ts` files.

4.  **Verification Script:**
    *   Add a new script to `package.json` for type-checking the project, for example: `"type-check": "tsc --noEmit"`.

### Phase 2: UI and Application Conversion

5.  **Component Conversion:**
    *   Once the core logic is typed, proceed to convert the React components.
    *   Rename `.jsx` files in `src/components/` to `.tsx`, adding types for props, state, and event handlers.

6.  **Finalization:**
    *   Convert the remaining application files, such as `src/App.jsx` and `src/main.jsx`, to `.tsx`.

## 5. Non-Goals (Out of Scope)

*   **"Big Bang" Migration:** The entire codebase will **not** be converted in a single pull request. The migration must be incremental, starting with Phase 1.
*   **Immediate Type Perfection:** Using `any` as a temporary escape hatch is acceptable to ensure the initial conversion can be completed efficiently. The goal is progress, not perfection, in the first pass.
*   **Logic Refactoring:** This task is strictly for adding types to the existing codebase. No business logic should be refactored or changed.

## 6. Design Considerations

*   Not applicable. This is a technical migration and does not directly involve UI or design changes.

## 7. Technical Considerations

*   The `tsconfig.json` file is the central configuration for the TypeScript compiler. Ensure it is configured correctly for a Vite + React project (e.g., `"jsx": "react-jsx"`).
*   Vite provides excellent out-of-the-box support for TypeScript, which should simplify the build process integration.

## 8. Success Metrics

The initial phase of the conversion will be considered successful when all of the following criteria are met:

*   The project successfully builds without any TypeScript errors using the `npm run build` command.
*   All files within the initial scope (`src/game/` and `src/ai/`) have been converted to `.ts`.
*   The application runs correctly in the browser with no new runtime errors introduced by the conversion.
*   The `wasm-pack` build process is successfully configured to generate TypeScript definitions.
*   A `type-check` script exists in `package.json` and executes without errors.

## 9. Open Questions

*   None at this time.
