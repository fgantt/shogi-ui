
# FEN Notation and Coordinate Systems

This document explains the FEN (Forsyth-Edwards Notation) string representation and the different coordinate systems used in this Shogi project.

## FEN (Forsyth-Edwards Notation)

The FEN string is a standard way of representing a Shogi board position in a single line of text. It captures the placement of all pieces, the current player, any captured pieces, and the move number.

The FEN string consists of four parts, separated by spaces:

1.  **Piece Placement:** This is the main part of the FEN string and describes the position of each piece on the board.
    *   The board is represented from top to bottom (rank 9 to 1).
    *   Each rank is separated by a `/`.
    *   Pieces are represented by single letters (e.g., `k` for King, `r` for Rook).
    *   Uppercase letters represent pieces belonging to Player 1 (Black/Sente).
    *   Lowercase letters represent pieces belonging to Player 2 (White/Gote).
    *   A number represents a sequence of empty squares. For example, `9` means a completely empty rank.
    *   Promoted pieces are prefixed with a `+`. For example, `+R` is a promoted Rook.

2.  **Active Player:** A single letter indicates whose turn it is to move.
    *   `b`: Black (Player 1) to move.
    *   `w`: White (Player 2) to move.

3.  **Captured Pieces:** This section lists the pieces that have been captured by each player.
    *   A `-` indicates that no pieces have been captured.
    *   Uppercase letters represent pieces captured by Player 1.
    *   Lowercase letters represent pieces captured by Player 2.

4.  **Move Number:** This is the final part of the FEN string and indicates the current move number.

### Example FEN String

`lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1`

*   **Piece Placement:** `lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL` (The initial board setup)
*   **Active Player:** `b` (Black's turn)
*   **Captured Pieces:** `-` (No captures)
*   **Move Number:** `1` (First move)

## Coordinate Systems

There are two coordinate systems used in this project:

1.  **Internal Board Representation (row, col):** This is a 0-indexed 2D array (or "matrix") that represents the Shogi board internally.
    *   The top-left square is `[0, 0]`.
    *   The bottom-right square is `[8, 8]`.
    *   This system is used by the game engine (`src/game/engine.js`) and the AI (`src/ai/ai.worker.js`) for all game logic and calculations.

2.  **Shogi Board Notation (file, rank):** This is the traditional way of representing Shogi coordinates, as seen on the game board UI.
    *   The top-right square is `11` (file 1, rank 1).
    *   The bottom-left square is `99` (file 9, rank 9).
    *   This system is primarily for display purposes in the user interface.

### Coordinate Conversion

The `from` and `to` coordinates in the `openingBook.json` file use the **internal board representation**. Here's how to convert between the two systems:

*   **Internal `[row, col]` to Shogi `[file, rank]`:**
    *   `file = 9 - col`
    *   `rank = row + 1`

*   **Shogi `[file, rank]` to Internal `[row, col]`:**
    *   `row = rank - 1`
    *   `col = 9 - file`

### Example Coordinate Conversion

Let's take the first move of the "Aggressive Rook" opening: `{ "from": [6, 7], "to": [5, 7] }`

*   **`from` coordinate:**
    *   Internal: `[6, 7]`
    *   Shogi: `file = 9 - 7 = 2`, `rank = 6 + 1 = 7`. This corresponds to the Shogi coordinate **27**.

*   **`to` coordinate:**
    *   Internal: `[5, 7]`
    *   Shogi: `file = 9 - 7 = 2`, `rank = 5 + 1 = 6`. This corresponds to the Shogi coordinate **26**.

So, the move `{ "from": [6, 7], "to": [5, 7] }` represents moving the piece from square **27** to **26** in traditional Shogi notation.
