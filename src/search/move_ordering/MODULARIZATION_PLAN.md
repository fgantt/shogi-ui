# Move Ordering Modularization Plan

## Current Status

- ✅ Module directory structure created (`src/search/move_ordering/`)
- ✅ 8 submodule files created with documentation
- ✅ Extraction plan documented in each module
- ⏳ Code extraction pending (requires incremental migration)

## File Size

- Original file: `src/search/move_ordering.rs` - 13,335 lines
- Target: Distributed across ~10 files (mod.rs + 8 submodules)

## Migration Strategy

### Phase 1: Structure Creation (COMPLETED)
- Created module directory
- Created submodule stubs with documentation
- Documented extraction requirements

### Phase 2: Code Extraction (PENDING)

**Important:** This must be done incrementally to maintain backward compatibility.

#### Step 1: Extract Statistics Module
1. Move `OrderingStats` struct to `statistics.rs`
2. Move all statistics-related structures (HotPathStats, HeuristicStats, etc.)
3. Move statistics update methods
4. Update imports in main file
5. Compile and test

#### Step 2: Extract Cache Module
1. Move `CacheConfig` and `CacheEvictionPolicy` to `cache.rs`
2. Move `MoveOrderingCacheEntry` to `cache.rs`
3. Move cache eviction logic
4. Move cache management methods
5. Update imports
6. Compile and test

#### Step 3: Extract History Heuristic Module
1. Move `HistoryEntry` to `history_heuristic.rs`
2. Move history table structures
3. Move history scoring/updating methods
4. Move history aging logic
5. Update imports
6. Compile and test

#### Step 4: Extract Killer Moves Module
1. Move killer move storage and management
2. Move killer move scoring methods
3. Update imports
4. Compile and test

#### Step 5: Extract Counter-Moves Module
1. Move counter-move table and management
2. Move counter-move scoring methods
3. Update imports
4. Compile and test

#### Step 6: Extract PV Ordering Module
1. Move PV move cache and retrieval
2. Move PV move scoring methods
3. Update imports
4. Compile and test

#### Step 7: Extract Capture Ordering Module
1. Move MVV/LVA scoring methods
2. Move capture move ordering logic
3. Update imports
4. Compile and test

#### Step 8: Extract SEE Calculation Module
1. Move SEE calculation methods
2. Move SEE cache management
3. Move helper methods (find_attackers_defenders, etc.)
4. Update imports
5. Compile and test

#### Step 9: Create mod.rs
1. Rename `move_ordering.rs` to `move_ordering_old.rs`
2. Create `move_ordering/mod.rs` with main `MoveOrdering` struct
3. Re-export all necessary types and functions
4. Update `MoveOrdering` impl to use module methods
5. Compile and test

#### Step 10: Update Imports
1. Update all `use` statements throughout codebase
2. Verify all code compiles
3. Run full test suite
4. Remove `move_ordering_old.rs`

## Module Dependencies

```
mod.rs
├── statistics (independent)
├── cache (used by many)
├── history_heuristic (independent)
├── killer_moves (independent)
├── counter_moves (independent)
├── pv_ordering (uses cache)
├── capture_ordering (uses see_calculation)
└── see_calculation (independent)
```

## Backward Compatibility

- All public APIs must remain unchanged
- All existing code must continue to work
- All tests must pass without modification
- No breaking changes allowed

## Testing Strategy

After each extraction step:
1. Compile the code
2. Run unit tests
3. Run integration tests
4. Verify no regressions

## Risk Mitigation

- Work incrementally (one module at a time)
- Compile and test after each step
- Keep old file as backup during migration
- Use git commits after each successful extraction
- Revert if any step breaks functionality

