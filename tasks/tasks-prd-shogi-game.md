## Relevant Files

- `src/game/engine.js` - Core Shogi game logic, including rules for movement, drops, promotion, check, and checkmate.
- `src/game/engine.test.js` - Unit tests for the core game logic.
- `src/App.jsx` - Main application component to manage overall game state and layout.
- `src/components/Board.jsx` - Renders the 9x9 game board and its squares.
- `src/components/Piece.jsx` - Renders a single Shogi piece with its traditional Kanji representation.
- `src/components/CapturedPieces.jsx` - Displays the pieces each player has captured and can drop.
- `src/components/GameControls.jsx` - Contains UI buttons like "New Game," "Undo Move," and the AI difficulty selector.
- `src/components/PromotionModal.jsx` - A modal dialog that prompts the user to promote a piece.
- `src/components/MoveLog.jsx` - Displays a running list of all moves made in the game.
- `src/ai/computerPlayer.js` - Logic for the computer opponent, including different difficulty algorithms.
- `src/ai/computerPlayer.test.js` - Unit tests for the AI's decision-making logic.
- `src/styles/shogi.css` - CSS file for styling the board, pieces, and all game-related components with a traditional Japanese aesthetic.

### Notes

- Unit tests should be created alongside the code files they are testing.
- You can run tests using the command provided by your Vite/React setup, likely `npm test`.

## Tasks

- [x] 1.0 Setup Core Game Logic and State Management
  - [x] 1.1 Define data structures for the game state (board layout, pieces, captured pieces, current turn).
  - [x] 1.2 Create a `game/engine.js` module to encapsulate all game rules.
  - [x] 1.3 Implement the initial board setup function.
  - [x] 1.4 Implement movement logic and validation for all piece types (Pawn, Lance, Knight, etc.).
  - [x] 1.5 Implement the logic for capturing pieces and adding them to the player's hand.
  - [x] 1.6 Implement the logic for dropping pieces, including validation for legal squares and "Nifu".
  - [x] 1.7 Implement the logic for piece promotion, including mandatory promotions.
  - [x] 1.8 Implement check and checkmate detection logic.
  - [x] 1.9 Write unit tests in `game/engine.test.js` to cover all rule implementations.

- [ ] 2.0 Develop Board and Piece Components
  - [x] 2.1 Create the `Board.jsx` component to render the 9x9 grid.
  - [x] 2.2 Create the `Piece.jsx` component to display a piece's Kanji, handling promoted states (e.g., with red text).
  - [x] 2.3 Create the `CapturedPieces.jsx` component to display pieces in a player's hand.
  - [x] 2.4 Style the board and pieces in `styles/shogi.css` to match the traditional Japanese aesthetic.
  - [x] 2.5 Integrate the components so the board correctly renders the initial game state from the game engine.

- [x] 3.0 Implement User Interaction for Piece Movement and Drops
  - [x] 3.1 Add state to `App.jsx` to track the currently selected piece.
  - [x] 3.2 Implement click-to-move: on piece click, select it; on square click, attempt to move the selected piece.
  - [x] 3.3 Implement drag-and-drop functionality for moving pieces on the board.
  - [x] 3.4 Implement both click-to-drop and drag-and-drop for pieces from the `CapturedPieces` component.
  - [x] 3.5 When a piece is selected, call the game engine to get legal moves and highlight the valid destination squares on the board.
  - [x] 3.6 Create the `PromotionModal.jsx` component.
  - [x] 3.7 When a move results in a promotion choice, display the promotion modal.

- [ ] 4.0 Build Game Controls and UI Feedback
  - [x] 4.1 Create the `GameControls.jsx` component.
  - [x] 4.2 Add a "New Game" button that resets the game state.
  - [x] 4.3 Implement an "Undo Move" button. This requires storing move history in the game state.
  - [x] 4.4 Add a dropdown to select AI difficulty (Easy, Medium, Hard).
  - [x] 4.5 Implement a visual indicator for when a king is in check.
  - [x] 4.6 Implement a visual indicator to highlight the opponent's last move.
  - [x] 4.7 Create and populate the `MoveLog.jsx` component to display move history.

- [ ] 5.0 Create the AI Opponent
  - [ ] 5.1 Create the `ai/computerPlayer.js` module.
  - [ ] 5.2 Implement the "Easy" difficulty level, which selects a random legal move.
  - [ ] 5.3 Implement the "Medium" difficulty level (e.g., using minimax with a shallow search depth).
  - [ ] 5.4 Implement the "Hard" difficulty level (e.g., minimax with alpha-beta pruning).
  - [ ] 5.5 Integrate the AI into the game flow: after the player's move is completed, trigger the AI to make its move based on the selected difficulty.
  - [ ] 5.6 Write unit tests in `ai/computerPlayer.test.js` for the AI's move-selection logic.