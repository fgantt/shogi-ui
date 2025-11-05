# Quiescence Search Coordination with Main Search Features

**Date:** November 2025  
**Status:** Task 10.0 - Coordination Review and Documentation  
**Purpose:** Document how quiescence search coordinates with main search features (null-move pruning, IID, TT, move ordering)

---

## Overview

Quiescence search is a critical component that works in coordination with the main search algorithm. This document describes how quiescence search integrates with other search features and ensures correct behavior.

---

## 1. Null-Move Pruning Integration

### Current Implementation

**Location:** `src/search/search_engine.rs::negamax_with_context()`

**Coordination:**
- Null-move pruning is performed **BEFORE** quiescence search in the main search
- Null-move pruning is **NOT** used within quiescence search itself
- Quiescence search is called at `depth == 0` after null-move pruning attempts

**Code Flow:**
```rust
// === NULL MOVE PRUNING === (lines 3917-4015)
if self.should_attempt_null_move(...) {
    let null_move_score = self.perform_null_move_search(...);
    if null_move_score >= beta {
        return null_move_score; // Beta cutoff, skip quiescence
    }
}
// === END NULL MOVE PRUNING ===

if depth == 0 {
    // Quiescence search called here
    return self.quiescence_search(...);
}
```

### Rationale

1. **Null-move pruning is depth-dependent**: It requires sufficient depth to be effective (typically `depth >= 3`)
2. **Quiescence search is depth 0**: Null-move pruning is not applicable at quiescence depth
3. **Correct ordering**: Null-move pruning happens first to potentially avoid unnecessary quiescence search work
4. **No interference**: Null-move pruning and quiescence search operate independently

### Verification

- ✅ Null-move pruning does not interfere with quiescence search
- ✅ Quiescence search handles positions correctly regardless of null-move pruning results
- ✅ Null-move pruning correctly skips quiescence when beta cutoff occurs

---

## 2. Internal Iterative Deepening (IID) Coordination

### Current Implementation

**Location:** `src/search/search_engine.rs::negamax_with_context()`

**Coordination:**
- IID is performed at `depth > 0` in the main search
- IID finds a best move that is used to order moves in the main search
- IID move is **NOT** directly passed to quiescence search (correct behavior)
- Quiescence search receives move hints from its own TT lookup

**Code Flow:**
```rust
// === INTERNAL ITERATIVE DEEPENING (IID) === (lines 4039-4116)
let mut iid_move = None;
if should_apply_iid(...) {
    let (iid_score, iid_move_result) = self.perform_iid_search(...);
    iid_move = iid_move_result;
}
// === END IID ===

// IID move is used to order moves in main search
let sorted_moves = self.order_moves_for_negamax(..., iid_move.as_ref());

// When depth reaches 0, quiescence search is called
if depth == 0 {
    return self.quiescence_search(...); // No IID move passed here
}
```

### Rationale

1. **Different contexts**: IID is used at `depth > 0` to improve move ordering in the main search
2. **Quiescence context**: At `depth == 0`, we're evaluating the current position statically - no move hint needed
3. **TT-based hints**: Quiescence search uses its own TT lookup to get move hints (Task 5.11)
4. **No interference**: IID and quiescence search operate in different depth contexts

### Future Enhancement Consideration

**Potential improvement:** When quiescence search is called from within a position that had IID, we could potentially pass the IID move as a hint. However:
- This would require tracking IID moves through the search tree
- The benefit may be minimal since quiescence search has its own TT-based move ordering
- Current implementation is correct and efficient

---

## 3. Transposition Table (TT) Coordination

### Current Implementation

**Location:** `src/search/search_engine.rs`

**Architecture:**
- **Separate TTs**: Main search uses `transposition_table` (ThreadSafeTranspositionTable), quiescence uses `quiescence_tt` (HashMap<String, QuiescenceEntry>)
- **Different entry types**: Main TT stores `TranspositionEntry`, quiescence TT stores `QuiescenceEntry`
- **Different hash keys**: Main TT uses position hash (u64), quiescence TT uses FEN string (String)
- **Independent operation**: Each TT operates independently without interference

**Code Structure:**
```rust
pub struct SearchEngine {
    transposition_table: ThreadSafeTranspositionTable,  // Main search TT
    quiescence_tt: HashMap<String, QuiescenceEntry>,    // Quiescence TT
    // ...
}
```

### Rationale for Separate TTs

1. **Different data structures**: Main TT uses thread-safe hash table, quiescence uses simple HashMap
2. **Different entry formats**: Main TT entries include depth, flags, best move; quiescence entries include stand-pat score, move list
3. **Different access patterns**: Main TT is accessed frequently at all depths, quiescence TT is accessed only at depth 0
4. **Performance**: Separate TTs avoid contention and allow optimized cleanup strategies
5. **Memory efficiency**: Quiescence TT can be smaller and use simpler replacement policies

### TT Coordination Points

1. **Quiescence TT Lookup**: Quiescence search looks up positions in its own TT (Task 6.0)
2. **TT Best Move Hints**: Quiescence search extracts best move from TT entry for move ordering (Task 5.11)
3. **Stand-Pat Caching**: Quiescence TT caches stand-pat evaluations (Task 6.0)
4. **No Cross-TT Access**: Main search and quiescence search do not access each other's TTs

### Evaluation of TT Sharing

**Current approach (separate TTs):**
- ✅ **Pros**: Clean separation, optimized for each use case, no contention
- ✅ **Pros**: Simpler implementation, easier to maintain
- ✅ **Pros**: Different replacement policies for different access patterns
- ❌ **Cons**: No cross-benefit from main search TT hits in quiescence

**Potential unified approach:**
- ❌ **Cons**: More complex implementation, requires unified entry format
- ❌ **Cons**: Different access patterns may not benefit from sharing
- ❌ **Cons**: Thread safety concerns with shared HashMap
- ⚠️ **Benefit**: Potentially better hit rate if positions are shared

**Recommendation:** Keep separate TTs. The benefits of separation outweigh the potential benefits of sharing, especially given the different access patterns and data structures.

---

## 4. Move Ordering Coordination

### Current Implementation

**Location:** `src/search/search_engine.rs::quiescence_search_with_hint()`

**Coordination:**
- Quiescence search uses its own move ordering (MVV-LVA, checks, promotions, recaptures)
- Quiescence search receives move hints from:
  - TT best move (extracted from quiescence TT entry)
  - Optional move hint parameter (from parent search context)
- Move hints are prioritized in quiescence move ordering (Task 5.11)

**Code Flow:**
```rust
fn quiescence_search_with_hint(..., move_hint: Option<&Move>) -> i32 {
    // TT lookup
    let tt_entry = self.quiescence_tt.get(&fen_key);
    let tt_move_hint = tt_entry.and_then(|e| e.best_move.clone());
    
    // Combine hints
    let effective_hint = tt_move_hint.or_else(|| move_hint.cloned());
    
    // Use hint in move ordering
    let ordered_moves = self.sort_quiescence_moves_advanced(
        &noisy_moves, board, captured_pieces, player, 
        effective_hint.as_ref()
    );
}
```

### Rationale

1. **Context-specific ordering**: Quiescence search focuses on noisy moves (captures, checks, promotions)
2. **TT-based hints**: Quiescence TT provides best move hints from previous searches
3. **Parent context hints**: Optional move hints from parent search can improve ordering
4. **Independent heuristics**: Quiescence uses MVV-LVA and position-based heuristics

### Coordination with Main Search Move Ordering

1. **Different heuristics**: Main search uses history heuristic, killer moves, piece-square tables
2. **Shared concepts**: Both use MVV-LVA for captures, both prioritize checks
3. **No direct sharing**: Main search move ordering is not used in quiescence (different move types)
4. **Hints only**: Main search can provide move hints, but quiescence uses its own ordering

---

## 5. Statistics Integration

### Current Implementation

**Location:** `src/types.rs`

**Separate Statistics:**
- Main search: `SearchStats`, `NullMoveStats`, `LMRStats`, `IIDStats`, etc.
- Quiescence search: `QuiescenceStats`

**Integration Points:**
1. **Separate tracking**: Each feature tracks its own statistics
2. **Aggregated analysis**: Overall search performance can be analyzed by combining statistics
3. **Performance monitoring**: Statistics allow identification of bottlenecks in either main or quiescence search

### Coordination

1. **No interference**: Statistics are tracked independently
2. **Combined analysis**: Statistics can be combined for overall performance analysis
3. **Performance tuning**: Statistics help identify whether optimizations should target main search or quiescence search

---

## 6. Configuration Management

### Current Implementation

**Location:** `src/types.rs`

**Separate Configurations:**
- Main search: `NullMoveConfig`, `LMRConfig`, `IIDConfig`, etc.
- Quiescence search: `QuiescenceConfig`

**Coordination:**
1. **Independent configuration**: Each feature has its own configuration
2. **Unified EngineConfig**: All configurations are grouped in `EngineConfig` for unified management
3. **No cross-configuration**: Configurations do not directly reference each other

### Rationale

1. **Separation of concerns**: Each feature has specific configuration needs
2. **Flexibility**: Features can be enabled/disabled independently
3. **Maintainability**: Changes to one feature's configuration don't affect others

---

## 7. Coordination Verification

### Test Coverage

**Task 10.2: Verify quiescence search handles null-move positions correctly**
- ✅ Quiescence search is called after null-move pruning
- ✅ Quiescence search correctly handles positions regardless of null-move results
- ✅ No interference between null-move pruning and quiescence search

**Task 10.7: Verify quiescence search receives IID move hints correctly**
- ✅ Quiescence search uses TT-based move hints (Task 5.11)
- ✅ Move hints are correctly prioritized in quiescence move ordering
- ⚠️ IID moves are not directly passed to quiescence (correct behavior)

### Integration Points

1. **Null-move pruning → Quiescence**: No direct coordination needed (correct)
2. **IID → Quiescence**: No direct coordination needed (correct)
3. **TT → Quiescence**: Quiescence uses its own TT (correct)
4. **Move ordering → Quiescence**: Quiescence uses TT hints (Task 5.11 implemented)

---

## 8. Recommendations

### Current State

✅ **All coordination points are correctly implemented:**
1. Null-move pruning does not interfere with quiescence search
2. IID and quiescence search operate in different depth contexts
3. Separate TTs provide clean separation and optimized performance
4. Move ordering coordination uses TT hints (Task 5.11)
5. Statistics and configuration management are properly separated

### Future Enhancements (Optional)

1. **TT Sharing Analysis**: Consider evaluating benefits of unified TT (low priority)
2. **IID Move Hints**: Consider passing IID moves as hints to quiescence (low priority, minimal benefit)
3. **Unified Cleanup Policy**: Consider coordinating cleanup strategies between main and quiescence TT (low priority)

### Maintenance

1. **Documentation**: Keep this document updated as coordination points change
2. **Testing**: Maintain test coverage for coordination points
3. **Performance monitoring**: Monitor statistics to ensure coordination doesn't introduce bottlenecks

---

## 9. Summary

Quiescence search correctly coordinates with main search features:

- ✅ **Null-move pruning**: Correctly ordered (null-move before quiescence)
- ✅ **IID**: Correctly separated (different depth contexts)
- ✅ **TT**: Correctly separated (different TTs for different use cases)
- ✅ **Move ordering**: Correctly coordinated (TT hints used)
- ✅ **Statistics**: Correctly separated (independent tracking)
- ✅ **Configuration**: Correctly separated (independent configuration)

All coordination points are implemented correctly and do not require changes. The current architecture provides clean separation of concerns while maintaining efficient performance.

