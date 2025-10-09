# Endgame Detection Implementation Plan

## Executive Summary

This document outlines the implementation plan to fix endgame detection in the shogi game engine. Currently, when a player (human or AI) is in checkmate, the game does not properly detect and handle the game-over condition, resulting in the AI searching indefinitely for moves that don't exist.

## Problem Statement

### Current Issues

1. **Missing UI Detection**: The checkmate detection code in `GamePage.tsx` (lines 450-455) is commented out with a TODO note.
2. **No Controller-Level Detection**: The `controller.ts` does not check for endgame conditions after moves.
3. **Poor AI Response**: When the AI is in checkmate, the search engine returns `None` or `"resign"`, but this is not properly handled by the UI.
4. **Incomplete Handling**: Only checkmate is partially handled; other endgame conditions (repetition, impasse, illegal moves, stalemate) are not implemented.

### Root Cause

The transition from the old game engine to `tsshogi` left endgame detection logic incomplete. The search engine correctly detects terminal positions (no legal moves), but this information is not propagated to the UI layer.

## Current Implementation Analysis

### Search Engine (Rust)

**File**: `src/search/search_integration.rs:121-124`
```rust
let legal_moves = self.move_generator.generate_legal_moves(board, player, captured_pieces);
if legal_moves.is_empty() {
    let is_check = board.is_king_in_check(player, captured_pieces);
    return Ok(if is_check { -100000 } else { 0 });
}
```

**Status**: ✅ Correctly detects checkmate (returns -100000) and stalemate (returns 0)

**File**: `src/bitboards.rs:326-332`
```rust
pub fn is_checkmate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
    self.is_king_in_check(player, captured_pieces) && !self.has_legal_moves(player, captured_pieces)
}

pub fn is_stalemate(&self, player: Player, captured_pieces: &CapturedPieces) -> bool {
    !self.is_king_in_check(player, captured_pieces) && !self.has_legal_moves(player, captured_pieces)
}
```

**Status**: ✅ Helper methods exist but are not exposed to WASM

### Controller (TypeScript)

**File**: `src/usi/controller.ts`

**Status**: ❌ No endgame detection logic

**Issues**:
- Does not check for checkmate after moves
- Does not detect repetition
- Does not detect impasse
- Handles `"resign"` from engine but doesn't trigger game over

### UI Layer (React)

**File**: `src/components/GamePage.tsx:450-455`
```typescript
//TODO(feg): With the switch to tsshogi, need to determine checkmate and repetition from the newPosition object.
// if (newPosition.isCheckmate()) {
//   setWinner(newPosition.turn === 0 ? 'player2' : 'player1');
// } else if (newPosition.isRepetition()) {
//   setWinner('draw');
// }
```

**Status**: ❌ Commented out, needs implementation

**File**: `src/components/CheckmateModal.tsx`

**Status**: ✅ Component exists and supports:
- Checkmate for player1/player2
- Draw by repetition
- But it's never triggered

## Implementation Plan

### Phase 1: Core Checkmate Detection (CRITICAL)

#### 1.1: Expose Checkmate Detection to WASM

**File**: `src/lib.rs` (or relevant WASM bindings)

**Task**: Add WASM bindings for endgame detection

```rust
#[wasm_bindgen]
impl WasmGameState {
    /// Check if the current position is checkmate for the specified player
    pub fn is_checkmate(&self, player: String) -> bool {
        let player = match player.as_str() {
            "black" => Player::Black,
            "white" => Player::White,
            _ => return false,
        };
        self.board.is_checkmate(player, &self.captured_pieces)
    }
    
    /// Check if the current position is stalemate for the specified player
    pub fn is_stalemate(&self, player: String) -> bool {
        let player = match player.as_str() {
            "black" => Player::Black,
            "white" => Player::White,
            _ => return false,
        };
        self.board.is_stalemate(player, &self.captured_pieces)
    }
    
    /// Check if the game is over for any reason
    pub fn is_game_over(&self) -> JsValue {
        // Check checkmate
        if self.board.is_checkmate(Player::Black, &self.captured_pieces) {
            return JsValue::from_str("checkmate_white_wins");
        }
        if self.board.is_checkmate(Player::White, &self.captured_pieces) {
            return JsValue::from_str("checkmate_black_wins");
        }
        
        // Check stalemate (counts as loss in shogi)
        if self.board.is_stalemate(Player::Black, &self.captured_pieces) {
            return JsValue::from_str("stalemate_black_loses");
        }
        if self.board.is_stalemate(Player::White, &self.captured_pieces) {
            return JsValue::from_str("stalemate_white_loses");
        }
        
        JsValue::from_str("ongoing")
    }
}
```

**Priority**: HIGH
**Estimated Effort**: 2-3 hours

#### 1.2: Implement Controller-Level Detection

**File**: `src/usi/controller.ts`

**Task**: Add game-over detection after moves

```typescript
private checkGameOver(position: ImmutablePosition): 'player1' | 'player2' | 'draw' | null {
  // Use tsshogi's built-in methods if available
  // Otherwise, check if current player has any legal moves
  
  const isBlackTurn = position.sfen.includes(' b ');
  const currentPlayer = isBlackTurn ? 'black' : 'white';
  
  // Check for checkmate
  if (position.isCheckmate && position.isCheckmate()) {
    // Current player is checkmated
    return isBlackTurn ? 'player2' : 'player1';
  }
  
  // Check for repetition (if tsshogi supports it)
  if (position.isRepetition && position.isRepetition()) {
    return 'draw';
  }
  
  // Check if current player has no legal moves
  const allPieces = position.board.getAllSquares(); // Get all squares with pieces
  let hasAnyLegalMove = false;
  
  for (const square of allPieces) {
    const piece = position.board.at(square);
    if (piece && piece.color === (isBlackTurn ? Color.BLACK : Color.WHITE)) {
      const moves = position.getLegalMovesFrom(square);
      if (moves.length > 0) {
        hasAnyLegalMove = true;
        break;
      }
    }
  }
  
  if (!hasAnyLegalMove) {
    // No legal moves - check if in check (checkmate) or not (stalemate = loss in shogi)
    const isInCheck = position.isCheck && position.isCheck();
    // In shogi, stalemate is a loss for the player who can't move
    return isBlackTurn ? 'player2' : 'player1';
  }
  
  return null;
}

private emitStateChanged(): void {
  const newPosition = this.record.position;
  this.emit('stateChanged', newPosition);
  
  // Check for game over
  const winner = this.checkGameOver(newPosition);
  if (winner) {
    this.emit('gameOver', { winner, position: newPosition });
  }
}
```

**Priority**: HIGH
**Estimated Effort**: 3-4 hours

#### 1.3: Update UI to Handle Game Over

**File**: `src/components/GamePage.tsx`

**Task**: Replace the commented-out TODO with proper detection

```typescript
// Listen for game over events from controller
useEffect(() => {
  const handleGameOver = (data: { winner: 'player1' | 'player2' | 'draw', position: ImmutablePosition }) => {
    console.log('Game over detected:', data.winner);
    setWinner(data.winner);
    
    // Play game over sound if available
    // playGameOverSound();
  };

  controller.on('gameOver', handleGameOver);

  return () => {
    controller.off('gameOver', handleGameOver);
  };
}, [controller]);
```

**Alternative Approach**: Check after each state change

```typescript
const onStateChanged = (newPosition: ImmutablePosition) => {
  setPosition(newPosition);
  setRenderKey(prev => prev + 1);
  
  // Update last move for highlighting
  const lastMoveData = controller.getLastMove();
  setLastMove(lastMoveData);
  
  // Check for game over
  const gameOverResult = checkGameOver(newPosition);
  if (gameOverResult) {
    setWinner(gameOverResult);
  }
  
  // ... rest of existing code
};

function checkGameOver(position: ImmutablePosition): 'player1' | 'player2' | 'draw' | null {
  // Determine current player
  const isBlackTurn = position.sfen.includes(' b ');
  
  // Get all legal moves for current player
  let hasLegalMoves = false;
  const allSquares = [];
  
  // Iterate through all board squares
  for (let row = 0; row < 9; row++) {
    for (let col = 0; col < 9; col++) {
      const square = Square.newByXY(col, row);
      if (square) {
        const piece = position.board.at(square);
        if (piece && 
            ((isBlackTurn && piece.color === Color.BLACK) || 
             (!isBlackTurn && piece.color === Color.WHITE))) {
          const moves = position.getLegalMovesFrom(square);
          if (moves.length > 0) {
            hasLegalMoves = true;
            break;
          }
        }
      }
    }
    if (hasLegalMoves) break;
  }
  
  // Check for legal drops as well
  if (!hasLegalMoves) {
    const captures = position.getCaptures(isBlackTurn ? Color.BLACK : Color.WHITE);
    // Try dropping pieces (implementation depends on tsshogi API)
    // ... check for legal drop moves
  }
  
  if (!hasLegalMoves) {
    // No legal moves available
    // In shogi, this is a loss for the current player
    return isBlackTurn ? 'player2' : 'player1';
  }
  
  return null;
}
```

**Priority**: HIGH
**Estimated Effort**: 2-3 hours

#### 1.4: Handle AI Resignation

**File**: `src/usi/controller.ts:41-58`

**Task**: Properly handle when AI returns `"resign"`

```typescript
engine.on('bestmove', ({ move: usiMove, sessionId: bestmoveSessionId }) => {
  if (usiMove === 'resign' || !usiMove) {
    // AI resigned or has no moves
    console.log('AI resigned or has no legal moves');
    const isBlackTurn = this.record.position.sfen.includes(' b ');
    const winner = isBlackTurn ? 'player2' : 'player1';
    this.emit('gameOver', { winner, position: this.record.position });
    this.emitStateChanged();
    return;
  }
  
  if (bestmoveSessionId === 'sente' || bestmoveSessionId === 'gote') {
    if (this.recommendationsEnabled && this.hasHumanPlayer() && !this.isCurrentPlayerAI()) {
      this.parseRecommendation(usiMove);
    } else {
      this.applyMove(usiMove);
      this.emit('aiMoveMade', { move: usiMove });
      this.emitStateChanged();
      if (this.isCurrentPlayerAI()) {
        this.requestEngineMove();
      }
    }
  }
});
```

**Priority**: HIGH
**Estimated Effort**: 1 hour

### Phase 2: Repetition Detection (HIGH)

#### 2.1: Implement Position History Tracking

**File**: `src/usi/controller.ts`

**Task**: Track position history for repetition detection

```typescript
export class ShogiController extends EventEmitter {
  private record: Record;
  private sessions: Map<string, EngineAdapter> = new Map();
  private positionHistory: Map<string, number> = new Map(); // position hash -> count
  // ... rest of fields

  private updatePositionHistory(): void {
    const currentSfen = this.record.position.sfen;
    const count = this.positionHistory.get(currentSfen) || 0;
    this.positionHistory.set(currentSfen, count + 1);
    
    // Check for four-fold repetition (sennichite)
    if (count + 1 >= 4) {
      console.log('Four-fold repetition detected (Sennichite)');
      this.emit('gameOver', { winner: 'draw', position: this.record.position });
    }
  }

  private applyMove(usiMove: string): void {
    const moveResult = this.record.doMove(usiMove);
    if (moveResult instanceof Error) {
      console.error('Failed to apply move:', moveResult);
      return;
    }
    
    // Update position history for repetition detection
    this.updatePositionHistory();
    
    // Synchronize all engines with new position
    // ... existing synchronization code
  }

  public async newGame(initialSfen?: string): Promise<void> {
    // Clear position history when starting new game
    this.positionHistory.clear();
    
    // ... rest of existing newGame code
  }
}
```

**Priority**: MEDIUM-HIGH
**Estimated Effort**: 3-4 hours

#### 2.2: Detect Perpetual Check (Advanced)

**Task**: Implement detection for continuous checks leading to repetition

**Note**: This requires tracking if each position in the repetition involves check

**Priority**: MEDIUM
**Estimated Effort**: 4-6 hours

### Phase 3: Impasse Detection (MEDIUM)

#### 3.1: Implement Jishōgi Detection

**File**: `src/bitboards.rs`

**Task**: Add impasse detection logic

```rust
impl BitboardBoard {
    /// Check if both kings are in the promotion zone (impasse condition)
    pub fn is_impasse_condition(&self) -> bool {
        let black_king_pos = self.find_king_position(Player::Black);
        let white_king_pos = self.find_king_position(Player::White);
        
        if let (Some(black_pos), Some(white_pos)) = (black_king_pos, white_pos) {
            // Black king in white's camp (ranks 0-2) AND white king in black's camp (ranks 6-8)
            return black_pos.row <= 2 && white_pos.row >= 6;
        }
        false
    }
    
    /// Count points for impasse resolution (24-point rule)
    pub fn count_impasse_points(&self, player: Player, captured_pieces: &CapturedPieces) -> i32 {
        let mut points = 0;
        
        // Count pieces on board
        for (pos, piece) in &self.piece_positions {
            if piece.player == player {
                points += match piece.piece_type {
                    PieceType::Rook | PieceType::Dragon => 5,
                    PieceType::Bishop | PieceType::Horse => 5,
                    PieceType::King => 0,
                    _ => 1, // Gold, Silver, Knight, Lance, Pawn, and all promoted pieces
                };
            }
        }
        
        // Count captured pieces (pieces in hand)
        // Note: This depends on how captured_pieces is structured
        // points += captured_pieces.count_points(player);
        
        points
    }
    
    /// Check impasse result (requires 24+ points for draw)
    pub fn check_impasse_result(&self, captured_pieces: &CapturedPieces) -> Option<ImpasseResult> {
        if !self.is_impasse_condition() {
            return None;
        }
        
        let black_points = self.count_impasse_points(Player::Black, captured_pieces);
        let white_points = self.count_impasse_points(Player::White, captured_pieces);
        
        Some(ImpasseResult {
            black_points,
            white_points,
            outcome: if black_points >= 24 && white_points >= 24 {
                ImpasseOutcome::Draw
            } else if black_points < 24 {
                ImpasseOutcome::WhiteWins
            } else {
                ImpasseOutcome::BlackWins
            }
        })
    }
}

pub struct ImpasseResult {
    pub black_points: i32,
    pub white_points: i32,
    pub outcome: ImpasseOutcome,
}

pub enum ImpasseOutcome {
    Draw,
    BlackWins,
    WhiteWins,
}
```

**Priority**: MEDIUM
**Estimated Effort**: 4-6 hours

#### 3.2: Expose Impasse to WASM and UI

**Task**: Add WASM bindings and controller logic for impasse

**Priority**: MEDIUM
**Estimated Effort**: 2-3 hours

### Phase 4: Illegal Move Detection (HIGH)

#### 4.1: Enhanced Move Validation

**Current Status**: Basic illegal move prevention exists in move generation

**Task**: Add specific illegal move detection and reporting

- Nifu (double pawn) detection
- Uchifuzume (pawn drop mate) detection
- Mandatory promotion enforcement
- Moving into check detection

**File**: `src/moves.rs` and `src/bitboards.rs`

**Priority**: MEDIUM-HIGH
**Estimated Effort**: 6-8 hours (comprehensive)

### Phase 5: UI Enhancements (LOW)

#### 5.1: Enhanced CheckmateModal

**Task**: Update modal to support all endgame conditions

```typescript
interface CheckmateModalProps {
  winner: 'player1' | 'player2' | 'draw' | null;
  endgameType?: 'checkmate' | 'resignation' | 'repetition' | 'impasse' | 'illegal' | 'time' | 'stalemate';
  details?: string;
  onDismiss: () => void;
  onNewGame: () => void;
}
```

**Priority**: LOW
**Estimated Effort**: 2-3 hours

#### 5.2: Game Over Animations and Sounds

**Task**: Add visual/audio feedback for game over

**Priority**: LOW
**Estimated Effort**: 2-3 hours

## Testing Plan

### Unit Tests

1. **Checkmate Detection**
   - Test various checkmate positions
   - Test that non-checkmate positions don't trigger
   - Test both Black and White checkmates

2. **Stalemate Detection**
   - Test stalemate positions
   - Verify stalemate counts as loss (not draw) in shogi

3. **Repetition Detection**
   - Test four-fold repetition
   - Test three-fold repetition doesn't trigger
   - Test perpetual check detection

4. **Impasse Detection**
   - Test 24-point rule with various piece combinations
   - Test that only positions with both kings in promotion zone trigger

### Integration Tests

1. **Human vs Human**
   - Play to checkmate
   - Verify modal appears
   - Verify game stops accepting moves

2. **Human vs AI**
   - Get AI into checkmate
   - Verify AI resigns or returns no move
   - Verify modal appears
   - Verify no infinite search loop

3. **AI vs AI**
   - Run full games to completion
   - Verify proper game termination
   - Verify no infinite loops

### Manual Testing Checklist

- [ ] Checkmate detected in all game modes (H/H, H/AI, AI/AI)
- [ ] Checkmate modal displays correctly
- [ ] Game stops accepting moves after checkmate
- [ ] AI properly resigns when checkmated
- [ ] Repetition detection works (if implemented)
- [ ] Impasse detection works (if implemented)
- [ ] No infinite search loops
- [ ] New game button works after game over
- [ ] Review position button works after game over

## Implementation Timeline

### Sprint 1 (Critical - 2-3 days)
- Phase 1.1: Expose checkmate to WASM ✓
- Phase 1.2: Controller detection ✓
- Phase 1.3: UI handling ✓
- Phase 1.4: AI resignation ✓
- **Testing**: Basic checkmate in all modes

### Sprint 2 (High Priority - 2-3 days)
- Phase 2.1: Repetition tracking ✓
- Phase 2.2: Perpetual check (optional) 
- Phase 4.1: Enhanced move validation (start)
- **Testing**: Repetition detection

### Sprint 3 (Medium Priority - 2-3 days)
- Phase 3.1: Impasse detection ✓
- Phase 3.2: Impasse WASM/UI ✓
- Phase 4.1: Enhanced move validation (complete)
- **Testing**: Impasse and illegal moves

### Sprint 4 (Polish - 1-2 days)
- Phase 5.1: Enhanced modal ✓
- Phase 5.2: Animations/sounds ✓
- **Testing**: Full integration testing

## Success Criteria

### Must Have (Sprint 1)
- ✅ Checkmate properly detected in all game modes
- ✅ CheckmateModal displays when game ends
- ✅ AI does not loop infinitely when checkmated
- ✅ Game properly ends and no more moves can be made

### Should Have (Sprint 2-3)
- ✅ Repetition (Sennichite) detection
- ✅ Stalemate detection (counts as loss)
- ✅ Basic illegal move prevention

### Nice to Have (Sprint 4)
- ⭕ Impasse (Jishōgi) detection with 24-point rule
- ⭕ Enhanced modal with different endgame types
- ⭕ Game over animations/sounds
- ⭕ Comprehensive illegal move detection (Nifu, Uchifuzume)

## Risk Analysis

### Technical Risks

1. **tsshogi API Limitations**
   - **Risk**: tsshogi may not expose all needed methods
   - **Mitigation**: Implement detection logic independently if needed

2. **Performance Impact**
   - **Risk**: Checking for game over after every move may impact performance
   - **Mitigation**: Optimize checks, only run when likely (e.g., few pieces remain)

3. **WASM-TypeScript Bridge**
   - **Risk**: Complexity in passing endgame data between Rust and TypeScript
   - **Mitigation**: Use simple return types (strings, numbers)

### Schedule Risks

1. **Scope Creep**
   - **Risk**: Implementing all endgame conditions may take longer than expected
   - **Mitigation**: Focus on Phase 1 first, other phases are optional

2. **Testing Coverage**
   - **Risk**: Insufficient testing may leave edge cases
   - **Mitigation**: Create comprehensive test suite with known positions

## References

- [Shogi Endgame Conditions Documentation](../../../SHOGI_ENDGAME_CONDITIONS.md)
- Japan Shogi Association official rules
- tsshogi library documentation
- Current codebase analysis

---

**Document Version**: 1.0  
**Last Updated**: October 8, 2025  
**Status**: Draft  
**Owner**: Development Team

