
# GEMINI.md

## Project Overview

This is a browser-based Shogi (Japanese Chess) game built with React and Vite. It allows a single player to play against a computer opponent with varying difficulty levels. The AI is implemented in a web worker and uses a minimax algorithm with alpha-beta pruning, piece-square tables, and an opening book to make its moves.

## Building and Running

### Prerequisites

*   Node.js and npm

### Installation

1.  Clone the repository:
    ```bash
    git clone <repository_url>
    cd shogi-ui
    ```
2.  Install dependencies:
    ```bash
    npm install
    ```

### Running the Application

*   **Development:**
    ```bash
    npm run dev
    ```
    This will start the Vite development server and make the application available at `http://localhost:5173`.

*   **Production Build:**
    ```bash
    npm run build
    ```
    This will create a `dist` directory with the production-ready files.

### Testing

*   Run the test suite:
    ```bash
    npm test
    ```

### Linting

*   Check for linting errors:
    ```bash
    npm run lint
    ```

## Development Conventions

*   **Component-Based Architecture:** The UI is built with React components, located in the `src/components` directory.
*   **State Management:** The main application state is managed in the `App.jsx` component using React hooks (`useState`, `useEffect`).
*   **Game Logic:** The core Shogi game logic is separated into the `src/game/engine.js` file. This includes piece movement, move validation, and check/checkmate detection.
*   **AI:** The AI logic is offloaded to a web worker (`src/ai/ai.worker.js`) to prevent blocking the main UI thread. The AI uses a variety of techniques to determine the best move, including:
    *   Minimax algorithm with alpha-beta pruning
    *   Piece-square tables (PSTs) for positional evaluation
    *   Mobility scoring
    *   An opening book for the initial moves of the game
*   **Styling:** The application is styled using CSS, with separate files for different parts of the UI (e.g., `shogi.css`, `settings.css`).
*   **Testing:** The project uses Vitest for testing. Test files are located in the `src/game` directory and have a `.test.js` extension.
