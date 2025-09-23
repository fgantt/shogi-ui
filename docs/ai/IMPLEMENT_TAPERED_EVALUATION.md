# Implementation Plan: Tapered Evaluation

## 1. Objective

To implement a tapered evaluation function in the Rust engine. This will allow the engine to use different weights for its evaluation terms based on the current phase of the game (e.g., opening, middlegame, endgame), leading to more nuanced and positionally aware strategic play.

## 2. Background

The value of pieces and positional features changes as the game progresses. For example, king safety is paramount in the middlegame when the board is full of attacking pieces, but it is less critical in many endgames. Conversely, the potential of a passed pawn to promote is far more valuable in the endgame than in the opening.

A tapered evaluation addresses this by calculating two scores for a position: a **middlegame score (`mg`)** and an **endgame score (`eg`)**. It then interpolates between these two scores based on a "game phase" value, producing a single, phase-adjusted evaluation.

## 3. Core Logic and Implementation Plan

This implementation requires a significant refactoring of `src/evaluation.rs`.

### Step 1: Define a Tapered Score and Game Phase

First, we need a way to represent the dual scores and calculate the game phase.

**File:** `src/evaluation.rs` (or a new `types.rs` if preferred)

```rust
// A simple struct to hold middlegame and endgame scores.
#[derive(Clone, Copy, Debug, Default)]
pub struct TaperedScore {
    pub mg: i32,
    pub eg: i32,
}

// Implement basic arithmetic for the struct to make combining scores easier.
impl std::ops::AddAssign for TaperedScore {
    fn add_assign(&mut self, other: Self) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

// The GamePhase will be a value from 0 (endgame) to 256 (opening).
const GAME_PHASE_MAX: i32 = 256;
```

### Step 2: Update the Main `evaluate` Function

The main `evaluate` function will be changed to calculate the game phase and then interpolate the final score.

**File:** `src/evaluation.rs`

```rust
// In `PositionEvaluator`

pub fn evaluate(&self, board: &BitboardBoard, player: Player, captured_pieces: &CapturedPieces) -> i32 {
    // 1. Calculate the game phase.
    // This can be based on the number and type of non-pawn, non-king pieces on the board.
    let game_phase = self.calculate_game_phase(board);

    // 2. Get the total tapered score.
    let mut total_score = TaperedScore::default();
    total_score += self.evaluate_material_and_position(board, player);
    total_score += self.evaluate_pawn_structure(board, player);
    total_score += self.evaluate_king_safety(board, player);
    // ... add other evaluation terms ...

    // 3. Interpolate the final score.
    let final_score = (total_score.mg * game_phase + total_score.eg * (GAME_PHASE_MAX - game_phase)) / GAME_PHASE_MAX;

    // Return score from the perspective of the current player.
    if player == board.side_to_move() { // Assuming side_to_move() exists
        final_score
    } else {
        -final_score
    }
}

fn calculate_game_phase(&self, board: &BitboardBoard) -> i32 {
    // A simple implementation: sum phase values for major pieces.
    // For example: Knight=1, Silver=1, Gold=2, Bishop=2, Rook=3
    // Max phase value would be something like (2*1 + 2*1 + 2*2 + 1*2 + 1*3) * 2 players = 24
    // Then scale this value to 0-256.
    // This needs to be carefully designed and tuned.
    // Placeholder:
    let phase_value = 128; // Assume mid-game for now
    phase_value
}
```

### Step 3: Update All Evaluation Components

Every evaluation function must be refactored to return a `TaperedScore`.

**File:** `src/evaluation.rs`

```rust
// Example for King Safety
fn evaluate_king_safety(&self, board: &BitboardBoard, player: Player) -> TaperedScore {
    let mut score = TaperedScore::default();
    // King safety is much more important in the middlegame.
    let king_safety_bonus = 100; // Example value
    score.mg = king_safety_bonus;
    score.eg = king_safety_bonus / 4; // Less important in the endgame
    score
}

// Example for Piece-Square Tables (PSTs)
// The `PieceSquareTables` struct must be updated to hold two tables for each piece.
struct PieceSquareTables {
    pawn_table_mg: [[i32; 9]; 9],
    pawn_table_eg: [[i32; 9]; 9],
    // ... other pieces
}

// The `get_value` function would then return a TaperedScore.
fn get_value(&self, piece_type: PieceType, pos: Position, player: Player) -> TaperedScore {
    // ... logic to get mg and eg values from the respective tables ...
    TaperedScore { mg: mg_value, eg: eg_value }
}
```

## 4. Dependencies and Considerations

*   **Major Refactoring:** This is a large-scale change that touches the entire evaluation system. It should be done carefully, one component at a time.
*   **Tuning:** This approach introduces a massive number of new parameters to tune (an `mg` and `eg` weight for every single evaluation term and PST entry). Initial values can be set where `mg` equals `eg`, and then tuned incrementally.
*   **Game Phase Calculation:** The logic for `calculate_game_phase` is critical. A poorly designed phase calculation will lead to incorrect evaluations. It should be based on the material present on the board, excluding kings and pawns.

## 5. Verification Plan

1.  **Unit Tests:**
    *   Create a test for `calculate_game_phase` with a starting position (should be max phase), a mid-game position, and an endgame position (should be low phase).
    *   Create a test for a specific endgame (e.g., a lone pawn about to promote). Assert that the `eg` part of the evaluation dominates the final score.
    *   Create a test for a middlegame position with kings under attack. Assert that the `mg` part of the king safety evaluation dominates.
2.  **Logging:** In debug mode, log the `game_phase`, total `mg` score, total `eg` score, and the final interpolated score. This is essential for debugging and observing how the evaluation changes as the game progresses.
3.  **Strength Testing:** This is a strategic enhancement. The new engine should demonstrate better planning, especially in transitioning from the middlegame to the endgame. A match against the previous version should show a clear improvement in strategic play and overall win rate.

