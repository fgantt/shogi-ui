## Relevant Files

- `package.json` - To add new dev dependencies (`typescript`, `@types/*`) and the `type-check` script.
- `tsconfig.json` - New file to be created for configuring the TypeScript compiler.
- `src/types.ts` - New file to be created for shared game-related interfaces (e.g., `Piece`, `Move`, `GameState`).
- `src/game/engine.js` -> `src/game/engine.ts` - Core JavaScript game engine logic to be converted.
- `src/game/engine.test.jsx` -> `src/game/engine.test.tsx` - Unit tests for the game engine, to be updated.
- `src/ai/wasmEngine.js` -> `src/ai/wasmEngine.ts` - Wrapper for the Rust/Wasm module to be converted.
- `src/ai/computerPlayer.js` -> `src/ai/computerPlayer.ts` - AI player logic to be converted.
- `src/components/Board.jsx` -> `src/components/Board.tsx` - Example of a major UI component to be converted.
- `src/App.jsx` -> `src/App.tsx` - The root application component to be converted.
- `src/main.jsx` -> `src/main.tsx` - The main application entry point to be converted.

### Notes

- Unit tests should be updated alongside the files they are testing. They will need to be renamed from `.js`/`.jsx` to `.ts`/`.tsx`.
- Use `npm test` to run the full test suite and verify changes.

## Tasks

- [x] 1.0 Setup TypeScript Environment and Configuration
  - [x] 1.1 Install TypeScript and type definitions (`typescript`, `@types/react`, `@types/react-dom`, `@types/node`) using `npm install --save-dev`.
  - [x] 1.2 Create the `tsconfig.json` file in the project root by running `npx tsc --init`.
  - [x] 1.3 Configure `tsconfig.json` for a strict, gradual migration. Ensure these key options are set: `"target": "ESNext"`, `"jsx": "react-jsx"`, `"strict": true`, and `"allowJs": true`.

- [x] 2.0 Convert Core Game Logic and AI Engine to TypeScript
  - [x] 2.1 Rename `src/game/engine.js` to `src/game/engine.ts`.
  - [x] 2.2 Rename `src/ai/wasmEngine.js` to `src/ai/wasmEngine.ts`.
  - [x] 2.3 Rename `src/ai/computerPlayer.js` to `src/ai/computerPlayer.ts`.
  - [x] 2.4 Create `src/types.ts` and define the core shared interfaces: `Piece`, `Move`, `GameState`, etc.
  - [x] 2.5 Import the new interfaces into the converted files and add types to all function signatures, variables, and data structures.
  - [x] 2.6 Work through all files in `src/game/` and `src/ai/` to resolve any TypeScript compiler errors.
  - [x] 2.7 Rename and update corresponding test files (e.g., `engine.test.jsx` to `engine.test.tsx`) and fix type-related issues.

- [x] 3.0 Update WebAssembly Integration for Type Safety
  - [x] 3.1 Check the `wasm-pack` documentation for how to generate TypeScript definitions.
  - [x] 3.2 Update the `wasm-pack build` command in `package.json` or relevant build scripts to include the flag for generating TypeScript definitions.
  - [x] 3.3 Verify that the `pkg/` directory contains the new `.d.ts` file after a build.
  - [x] 3.4 Update the Wasm module import in `src/ai/wasmEngine.ts` to use the newly generated types for full type safety.

- [x] 4.0 Convert React Components and Application Files to TypeScript
  - [x] 4.1 Begin by renaming `src/main.jsx` to `src/main.tsx` and `src/App.jsx` to `src/App.tsx`.
  - [x] 4.2 Incrementally convert React components by renaming `.jsx` files to `.tsx` (e.g., `src/components/Board.jsx` -> `src/components/Board.tsx`).
  - [x] 4.3 For each component, add types for props, state, and event handlers.
  - [x] 4.4 Update all component-related test files from `.jsx` to `.tsx` and resolve any type errors.

- [x] 5.0 Verify Project-Wide Type Safety and Finalize Migration
  - [x] 5.1 Add a `type-check` script to `package.json`: `"type-check": "tsc --noEmit"`.
  - [x] 5.2 Run `npm run type-check` to ensure the entire project passes without TypeScript errors.
  - [x] 5.3 Run the full test suite with `npm test` to confirm no regressions were introduced. (Note: wasmEngine tests are failing with an unreachable error).
  - [x] 5.4 Launch the application with `npm run dev` and perform a manual smoke test to ensure all features work as expected.