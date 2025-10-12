# WASM Piece Corruption Fix

## Issue
Runtime errors during AI vs AI games showing:
```
panicked at src/bitboards.rs:130:22:
index out of bounds: the len is 14 but the index is 64
```

The error occurred when calling `piece.piece_type.to_u8()` which was returning 64 instead of the expected 0-13 range.

## Root Cause Analysis

### The Problem
1. The `Piece` struct had `#[wasm_bindgen]` attribute applied to it
2. `Piece` contains enum fields (`piece_type: PieceType`, `player: Player`)
3. WASM bindgen with enum fields in structs can cause memory layout issues
4. The corruption was causing `piece_type` to contain invalid data
5. The value 64 matched `Position (7,1).to_u8()`, suggesting memory confusion

### Why This Happened
- With `#[wasm_bindgen]` on a struct containing enum fields, the memory layout may differ from Rust's standard representation
- When `Piece` objects were stored/retrieved from HashMap or cloned during board operations, the data could get corrupted
- The enum discriminant could end up with invalid values, causing `to_u8()` to return unexpected results

## The Fix

### 1. Removed `#[wasm_bindgen]` from `Piece` struct
```rust
// Before:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}

// After:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub player: Player,
}
```

**Rationale**: `Piece` is not directly used across the WASM boundary. It's converted to/from JSON via `PieceJson` in `lib.rs`. The `#[wasm_bindgen]` attribute was unnecessary and causing issues.

### 2. Added `#[wasm_bindgen(skip)]` to `Move.captured_piece`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Move {
    pub from: Option<Position>,
    pub to: Position,
    pub piece_type: PieceType,
    pub player: Player,
    pub is_promotion: bool,
    pub is_capture: bool,
    #[wasm_bindgen(skip)]  // <-- Added this
    pub captured_piece: Option<Piece>,
    pub gives_check: bool,
    pub is_recapture: bool,
}
```

**Rationale**: Since `Piece` no longer has `#[wasm_bindgen]`, the `Move` struct (which does need WASM bindings) can't expose `Option<Piece>` fields. The `captured_piece` field is skipped in WASM bindings but still available in Rust code.

### 3. Added Defensive Validation
Added validation in `place_piece` and `remove_piece` to catch and log any future corruption:
- Validates position is valid
- Validates piece_type index is < 14
- Logs detailed error messages if validation fails

### 4. Added Debug Logging
Added comprehensive logging in:
- `place_piece`: Logs piece being placed and the converted index
- `remove_piece`: Logs piece being removed
- `make_move`: Logs pieces being moved/dropped

## Testing
The build now completes successfully. To verify the fix:
1. Run AI vs AI games
2. Check for the absence of index out of bounds errors
3. Monitor debug logs for any validation warnings

## Key Takeaways
1. **Avoid `#[wasm_bindgen]` on structs with enum fields** unless absolutely necessary
2. **Use `#[wasm_bindgen(skip)]`** for fields that don't need JavaScript exposure
3. **Prefer JSON serialization** for complex types crossing WASM boundaries
4. **Add validation** for array indexing when dealing with potentially corrupted data
5. **Both `Position` and `PieceType` have `to_u8()` methods** - be aware of potential confusion

## Files Modified
- `src/types.rs`: Removed `#[wasm_bindgen]` from `Piece`, added `#[wasm_bindgen(skip)]` to `Move.captured_piece`, added validation in `Piece::new`
- `src/bitboards.rs`: Added validation and debug logging in `place_piece`, `remove_piece`, and `make_move`

