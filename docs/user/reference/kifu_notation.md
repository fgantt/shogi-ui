# Kifu Notation Explained

A kifu is a record of a shogi game, documenting each move made by both players. Understanding the notation allows you to follow along with game analysis and improve your own playing. There are several notation systems, but they share common elements.

## Core Components of a Move

*   **Player Indicator:** Often, a symbol indicates which player is moving. Black (Sente, the first player) is typically represented by a black triangle (▲), while White (Gote, the second player) is shown with a white triangle (△).
*   **Piece:** The piece being moved is identified, usually by its initial in Western notation (e.g., P for Pawn, R for Rook) or its Japanese character (e.g., 歩 for Pawn).
*   **Destination Coordinates:** The square the piece moves to is always indicated. Shogi boards are a 9x9 grid. Files (columns) are numbered 1 to 9 from right to left, and ranks (rows) are numbered 1 to 9 from top to bottom (from Black's perspective). A move to the square at file 7, rank 6 would be written as 76 in Western notation or ７六 in Japanese.
*   **Action/Movement Type:** Symbols are used to clarify the type of move.
    *   **Normal Move:** A hyphen (-) is often used in Western notation (e.g., P-76).
    *   **Capture:** An 'x' indicates a capture (e.g., Rx24).
    *   **Drop:** An asterisk (*) signifies a piece being dropped onto the board from hand (e.g., P*23). In Japanese notation, this is indicated by the character '打'.
*   **Promotion:**
    *   A plus sign (+) is added if a piece is promoted (e.g., Bx27+). In Japanese notation, this is indicated by the character '成'.
    *   An equals sign (=) is used if a player chooses not to promote a piece when given the option (e.g., Nx53=).

## Resolving Ambiguity

When two or more of the same piece could move to the same square, additional information is added to the notation to clarify which piece moved.

*   **Origin Square:** The starting square of the piece can be included (e.g., G77-78 to distinguish it from G68-78).
*   **Directional Indicators:** Japanese notation often uses characters to show the direction of movement or the relative position of the piece.
    *   右 (migi): Right
    *   左 (hidari): Left
    *   上 (agaru): Forward/Up
    *   引 (hiku): Backward/Pull
    *   寄 (yoru): Sideways
    *   直 (choku): Straight (when a piece could also move diagonally)

## Special Notation

*   **Same Square Capture (同 dou):** If a piece captures the opponent's piece on the same square where the opponent just moved, the destination coordinates can be replaced with the character '同', meaning "same." This always involves a capture.

## KIF File Format

The KIF (Kifu) file format is a common text-based format used to record Shogi games, studies, and puzzles. It provides a structured way to represent all aspects of a game, from initial setup to the final move and comments.

### Key Sections of a KIF File:

*   **Header:** Contains general information about the game, such as player names (先手 for Sente/black, 後手 for Gote/white), handicap (手合割), start and end dates (開始日時, 終了日時), tournament name (棋戦), location (場所), and time controls (持ち時間, 秒読み).
*   **Board Setup:** Specifies the initial board position for non-standard games or puzzles, similar to SFEN. It shows pieces on the board and pieces in hand.
*   **Moves:** Describes individual moves, including destination, piece, promotion (成), and origin. Drops are indicated by 「打」.
*   **Termination Moves:** Records the reason for game termination, such as resignation (投了), four-fold repetition (千日手), checkmate (詰み), or loss on time (切れ負け).
*   **Time Expended:** Shows the time spent on a move and the total time expended by a player.
*   **Variations:** Allows for branching move sequences, starting with 「変化：」 followed by the move number from which the variation branches.
*   **Comments:** Lines starting with '#' are ignored by parsers and can contain general information. Comments on moves start with '*'.