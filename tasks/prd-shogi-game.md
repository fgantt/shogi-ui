# Product Requirements Document: Browser-Based Shogi Game

## 1. Introduction/Overview

This document outlines the requirements for a single-player, browser-based Shogi (Japanese Chess) game. The feature allows a user to play a full game of Shogi against a computer opponent. The primary goal is to create a functional, engaging, and accessible Shogi experience for a broad audience, from beginners to experienced players, without the need for installation or complex setup.

## 2. Goals

*   **Develop a fully playable Shogi game** that correctly implements all standard rules, including piece movement, captures, drops, and promotion.
*   **Provide an intuitive and user-friendly interface** that accommodates both click-based and drag-and-drop controls.
*   **Implement a computer opponent with multiple difficulty levels** to provide a suitable challenge for players of varying skill.
*   **Deliver clear visual feedback** for all significant game states and actions, such as legal moves, checks, and the last move made.

## 3. User Stories

*   **As a new player,** I want to see all legal moves for a piece I select so that I can learn the rules and make valid moves.
*   **As an experienced player,** I want to be able to move and drop pieces quickly using drag-and-drop so that I can play at a faster pace.
*   **As a player,** I want a prompt to appear when I can promote a piece so that I don't forget to take advantage of promotion.
*   **As a player,** I want to be able to undo my last move so that I can correct a mistake or explore a different line of play.
*   **As a player,** I want to play against an AI that offers different difficulty levels so that I can have a challenging and enjoyable game regardless of my skill level.
*   **As a player,** I want to see a log of all moves made so that I can review the game's progression.

## 4. Functional Requirements

1.  The system must render a 9x9 Shogi board with all 40 pieces in their correct starting positions.
2.  The system must allow the user to move pieces using either of the following methods:
    *   Clicking the piece to select it, then clicking a valid destination square.
    *   Dragging the piece from its origin square and dropping it on a valid destination square.
3.  The system must correctly enforce all Shogi movement rules for all pieces (King, Rook, Bishop, Gold General, Silver General, Knight, Lance, Pawn).
4.  The system must highlight all possible legal moves on the board for a currently selected piece.
5.  The system must implement the piece capture rule: when a piece is captured, it is removed from the board and added to the capturing player's "pieces in hand" area.
6.  The system must allow a player to "drop" a captured piece onto any valid empty square on the board, using either click-based or drag-and-drop controls.
7.  The system must enforce all rules for dropping pieces, including "Nifu" (two pawns on a file) and "Uchifu-zume" (pawn drop checkmate).
8.  When a player's piece makes a move that allows for promotion, the system must display a prompt asking the user if they wish to promote the piece (unless promotion is mandatory).
9.  The system must automatically perform mandatory promotions as required by the rules.
10. The system must implement a computer AI opponent with three selectable difficulty levels: Easy, Medium, and Hard.
11. The system must provide a "New Game" button that resets the board and all game state to the starting position.
12. The system must provide an "Undo Move" button that reverts the last move made by the player and the subsequent AI move.
13. The system must display a clear visual indicator (e.g., highlighting the King) when a player is in "check".
14. The system must visually indicate the last move made by the opponent.
15. The system must maintain and display a log of all moves made during the game.
16. The system must correctly detect and declare a checkmate, ending the game.

## 5. Non-Goals (Out of Scope)

*   Player vs. Player (PvP) mode of any kind (local or online).
*   Timed games, move clocks, or any time-based constraints.
*   User accounts, profiles, game history, or statistics.
*   The ability to save or load a game in progress.
*   In-game tutorials, a rulebook, or a help system.

## 6. Design Considerations

*   **Aesthetic:** The game should have a traditional Japanese aesthetic.
*   **Board:** The board should be styled to look like a traditional wood (e.g., kaya) Shogi board.
*   **Pieces:** Pieces should use classic, easily readable Kanji characters. Promoted pieces should be clearly distinguishable (e.g., red characters).
*   **Captured Pieces:** Captured pieces for each player must be displayed clearly in a dedicated area on their respective side of the board.
*   **Controls:** UI controls (New Game, Undo, AI Difficulty) should be intuitive and easily accessible.

## 7. Technical Considerations

*   The core game logic (rules, state management) should be decoupled from the UI rendering to simplify development and future maintenance.
*   The AI logic should be modular, allowing different difficulty algorithms to be swapped without impacting the core game engine.

## 8. Success Metrics

*   The game is fully playable and adheres to all functional requirements.
*   The "Undo Move" feature is used in less than 20% of completed games, suggesting players are making intentional moves.
*   The AI at the "Hard" difficulty level provides a legitimate challenge to experienced players.

## 9. Open Questions

*   What specific algorithms should be used for the "Easy", "Medium", and "Hard" AI difficulty levels? (e.g., Easy=random, Medium=minimax with depth 2, Hard=minimax with depth 4 + alpha-beta pruning?)
*   How should the move log be formatted and displayed to the user? (e.g., using Standard Shogi notation like `P-7f` or a more descriptive format?)
