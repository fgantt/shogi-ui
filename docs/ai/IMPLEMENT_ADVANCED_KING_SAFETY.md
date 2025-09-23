# Implementation Plan: Advanced King Safety Evaluation

## 1. Objective

To significantly improve the engine's understanding of king safety by implementing a more sophisticated evaluation model. This model will go beyond simply counting nearby pieces and will instead recognize common castle structures, analyze their integrity, and evaluate the threat level from enemy attacking pieces.

## 2. Background

The current king safety evaluation is functional but basic. In shogi, the king's safety is not just about having defenders nearby; it's about the structural integrity of the "castle" (`kakoi`). A well-formed castle like a Mino or Anaguma provides robust protection. A strong engine must be able to distinguish between a solid castle and a weak, exposed king, and it must also quantify the danger posed by coordinated enemy attacks.

This implementation will add two main components to the evaluation:
1.  **Castle Recognition:** Scoring the player's defensive formation.
2.  **Attack Analysis:** Scoring the threat posed by the opponent.

## 3. Core Logic and Implementation Plan

This plan primarily involves creating new functions within `src/evaluation.rs` and integrating them into the main `evaluate` function, using the `TaperedScore` system.

### Step 1: Define Castle Patterns

We need a way to represent known castle formations. A struct holding bitmasks for the required pieces is a good approach.

**File:** `src/evaluation.rs` (or a new `castles.rs` module)

```rust
use crate::types::{PieceType, Player, Position, TaperedScore};
use crate::bitboards::BitboardBoard;

struct CastlePattern {
    name: &'static str,
    // A list of (PieceType, relative position from king) that form the castle.
    // This is a simplified representation. A more robust one might use bitmasks.
    pieces: Vec<(PieceType, i8, i8)>, // (PieceType, row_offset, col_offset)
    score: TaperedScore,
}

// Example for a simple Mino castle for Black
const MINO_CASTLE: CastlePattern = CastlePattern {
    name: "Mino",
    pieces: vec![
        (PieceType::Silver, -1, -1), // Assuming king is at origin
        (PieceType::Gold, 0, -1),
    ],
    score: TaperedScore { mg: 150, eg: 50 },
};

// A more robust approach using bitmasks relative to the king's square would be better.
```

### Step 2: Implement Castle Recognition Logic

Create a function that checks the board against a library of known castle patterns.

**File:** `src/evaluation.rs`

```rust
// In `PositionEvaluator`

fn evaluate_castle_structure(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let king_pos = board.find_king_position(player);
    if king_pos.is_none() { return TaperedScore::default(); }
    let king_pos = king_pos.unwrap();

    // A real implementation would have a list of castle patterns to check against.
    // For example, check for Mino:
    if self.is_castle_present(board, player, king_pos, &MINO_CASTLE) {
        return MINO_CASTLE.score;
    }

    // ... check for other castles (Anaguma, Yagura, etc.)

    TaperedScore::default() // No recognized castle
}

fn is_castle_present(&self, board: &BitboardBoard, player: Player, king_pos: Position, castle: &CastlePattern) -> bool {
    for (piece_type, dr, dc) in &castle.pieces {
        let check_pos = Position::new((king_pos.row as i8 + dr) as u8, (king_pos.col as i8 + dc) as u8);
        if let Some(piece) = board.get_piece(check_pos) {
            if !(piece.piece_type == *piece_type && piece.player == player) {
                return false; // Wrong piece or player
            }
        } else {
            return false; // Piece not found
        }
    }
    true
}
```

### Step 3: Implement Attack Analysis

This involves creating "attack maps" for the opponent's pieces and seeing how many of them target the king's zone.

**File:** `src/evaluation.rs`

```rust
// In `PositionEvaluator`

fn evaluate_king_attack(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let king_pos = board.find_king_position(player);
    if king_pos.is_none() { return TaperedScore::default(); }
    let king_pos = king_pos.unwrap();

    let opponent = player.opposite();
    let mut attack_score = 0;
    let mut num_attackers = 0;

    // Define the king's zone (e.g., a 3x3 square around the king)
    let king_zone = self.get_king_zone_bitboard(king_pos);

    // Iterate through opponent's pieces
    for piece_type in ALL_PIECE_TYPES { // Assuming such an array exists
        let piece_bitboard = board.get_piece_bitboard(piece_type, opponent);
        for pos in iter_bits(piece_bitboard) { // Assuming iter_bits yields Positions
            let attacks = self.move_generator.get_attacks(piece_type, pos, board, opponent);
            
            // Check if the attacks overlap with the king's zone
            if (attacks & king_zone).is_not_empty() {
                num_attackers += 1;
                // Add to score based on attacker's value
                attack_score += piece_type.base_value() / 10; // Example weighting
            }
        }
    }

    // Penalize heavily for multiple attackers
    if num_attackers > 1 {
        attack_score += (num_attackers - 1) * 50; // Exponentially increasing penalty
    }

    // King safety is less of a concern in the endgame
    TaperedScore { mg: -attack_score, eg: -attack_score / 4 }
}
```

### Step 4: Integrate into the Main Evaluation Function

Update `evaluate_king_safety` to use these new, more sophisticated functions.

**File:** `src/evaluation.rs`

```rust
// In `PositionEvaluator`

// This function will now be the main entry point for king safety.
fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let mut total_king_safety = TaperedScore::default();

    // Evaluate our castle
    total_king_safety += self.evaluate_castle_structure(board, player);

    // Evaluate opponent's attack on our king
    total_king_safety += self.evaluate_king_attack(board, player);

    total_king_safety
}
```

## 4. Dependencies and Considerations

*   **Domain Knowledge:** This feature requires significant knowledge of shogi castles. The patterns must be correctly defined. Researching common castles (Mino, Yagura, Anaguma, etc.) and their relative strengths is essential.
*   **Attack Tables:** For performance, `move_generator.get_attacks` should be very fast. This is typically done using pre-computed attack tables (e.g., an array that maps a square and piece type to an attack bitboard).
*   **Tuning:** The scores for each castle and the weights for attackers are critical tuning parameters. The initial values will be estimates that need to be refined through testing and automated tuning.
*   **Tapered Evaluation:** This feature is tightly coupled with the Tapered Evaluation system. The relative safety of a castle and the danger of an attack change dramatically between the middlegame and endgame.

## 5. Verification Plan

1.  **Unit Tests:**
    *   Create tests that set up perfect castle formations (Mino, Anaguma) and assert that `evaluate_castle_structure` returns a high positive score.
    *   Create tests with a king under a coordinated attack (e.g., from a Rook and Bishop) and assert that `evaluate_king_attack` returns a large negative score.
    *   Create tests for broken or incomplete castles and verify they score lower than complete ones.
2.  **Puzzle Testing:** Test the engine on a suite of checkmate-in-N (`tsume`) puzzles. A better understanding of king safety should allow the engine to find threats and mating nets more effectively.
3.  **Strength Testing:** The engine's defensive capabilities should be vastly improved. It should be much harder to launch a successful attack against it, and it should be better at building and maintaining its own castles. A match against the previous version should confirm this.

