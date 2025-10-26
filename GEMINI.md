
# Project Overview

This is a desktop Shogi (Japanese Chess) game application built with Tauri. The application features a React-based user interface and a Rust-powered game engine that runs as a separate USI-compatible process for high-performance game logic.

## Key Technologies

*   **Frontend:** React, Vite, `react-router-dom`, TypeScript
*   **Desktop Framework:** Tauri (Rust-based desktop app framework)
*   **Game Engine:** Rust USI engine (`usi-engine` binary)
*   **Styling:** CSS, with specific styles for the game board and components
*   **Communication:** USI protocol for engine-frontend communication

## Architecture

The application is structured as a desktop application using Tauri, which combines a React frontend with a Rust backend. The core game logic, including move generation, validation, and AI, is implemented in Rust as a standalone USI engine that communicates with the frontend via the USI protocol.

*   `src/App.tsx`: Defines the main application component and routing
*   `src/components/`: Contains the React components for the UI, such as the game board, pieces, and modals
*   `src-tauri/src/`: Contains the Tauri backend code for engine management and communication
*   `src/lib.rs`: The main Rust source file, which defines the Shogi engine and its API
*   `src/main.rs`: Entry point for the standalone USI engine binary
*   `src-tauri/`: Tauri configuration and backend code for desktop app functionality

# Building and Running

## Prerequisites

*   Node.js and npm (or yarn)
*   Rust and Cargo
*   Tauri CLI (`npm install -g @tauri-apps/cli` or use `npx`)

## Development

To run the application in development mode, use the following command:

```bash
npm run tauri:dev
```

This will build the USI engine, start the Vite development server, and launch the Tauri desktop application.

## Building for Production

To build the application for production, use the following command:

```bash
npm run tauri:build
```

This will create a desktop application bundle in `src-tauri/target/release/bundle/` with platform-specific installers.

## Testing

To run the test suite, use the following command:

```bash
npm run test
```

# Development Conventions

## Code Style

*   **TypeScript/JavaScript:** The project uses ESLint for code linting. The configuration can be found in `eslint.config.js`.
*   **Rust:** The Rust code follows standard Rust conventions.

## USI Engine Integration

The Rust game engine is compiled as a standalone USI-compatible binary (`usi-engine`). The Tauri backend manages the engine process and handles communication via the USI protocol. When making changes to the Rust engine code, you will need to rebuild the engine using `npm run build:engine`.

## AI Engine

The AI engine is implemented in Rust as a USI-compatible engine. The Tauri backend (`src-tauri/src/`) manages engine processes and handles communication between the React frontend and the Rust engine via the USI protocol. The engine supports multiple instances and can be configured with different options.

## Additional Notes

*   The application is distributed as a desktop application, not a web application
*   Engine processes are managed by the Tauri backend and communicate via USI protocol
*   The project supports multiple engine instances and engine management features