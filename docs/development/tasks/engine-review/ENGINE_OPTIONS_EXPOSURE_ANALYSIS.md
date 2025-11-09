# Engine Options Exposure Analysis

**Date:** December 2024  
**Context:** Review of Task 8.0 completion and engine options exposure

---

## Current State

### Currently Exposed Options (via USI)

1. **USI_Hash** - Transposition table size (1-1024 MB, default: 16 MB)
2. **depth** - Search depth limit (1-8, default: 5) ⚠️ **LIMITED TO 8**
3. **USI_Threads** - Thread count (1-32, default: CPU count)

### Issues with Current Depth Setting

1. **Too restrictive**: Limited to max depth 8, while:
   - **Apery** has no depth limit and searches to depth 40+
   - **YaneuraOu** allows depth 0-2147483647 (default: 0 = unlimited/adaptive)
   - Our engine's internal validation allows up to 50
   - Our presets use max_depth: 25-30

2. **Default should be 0**: Like YaneuraOu, default should be 0 (unlimited/adaptive) rather than 5

3. **Inconsistent**: USI handler limits to 8, but EngineConfig validation allows up to 50

---

## Recommended Options to Expose

Based on Task 1.0-8.0 improvements, here are options that should be exposed:

### High Priority (User-Facing Performance Controls)

#### 1. **Depth Limit** (Fix Current Implementation)
- **Name**: `MaxDepth` or keep `depth` but change behavior
- **Type**: `spin`
- **Default**: `0` (unlimited/adaptive, like YaneuraOu)
- **Range**: `0-100` (0 = unlimited, 1-100 = specific limit)
- **Description**: Maximum search depth. 0 means adaptive (no limit). Higher values ensure deeper analysis but slower play.

#### 2. **Time Check Frequency** (Task 8.0)
- **Name**: `TimeCheckFrequency`
- **Type**: `spin`
- **Default**: `1024`
- **Range**: `1-100000`
- **Description**: How often to check time limits (every N nodes). Lower values = more accurate time control but higher overhead. Higher values = faster but less responsive.

#### 3. **Time Safety Margin** (Task 8.0)
- **Name**: `TimeSafetyMargin`
- **Type**: `spin`
- **Default**: `100`
- **Range**: `0-10000`
- **Description**: Absolute safety margin in milliseconds to reserve for search completion and time check overhead.

#### 4. **Time Allocation Strategy** (Task 4.0)
- **Name**: `TimeAllocationStrategy`
- **Type**: `combo`
- **Default**: `Adaptive`
- **Values**: `Equal`, `Exponential`, `Adaptive`
- **Description**: How to allocate time across iterative deepening depths:
  - Equal: Divide time equally
  - Exponential: Later depths get exponentially more time
  - Adaptive: Use historical depth completion times

#### 5. **Enable Aspiration Windows** (Task 7.0)
- **Name**: `EnableAspirationWindows`
- **Type**: `check`
- **Default**: `true`
- **Description**: Enable aspiration window optimization for faster searches.

#### 6. **Aspiration Window Size** (Task 7.0)
- **Name**: `AspirationWindowSize`
- **Type**: `spin`
- **Default**: `25` (base_window_size)
- **Range**: `10-500`
- **Description**: Base window size for aspiration windows. Larger values = wider search, slower but more thorough.

### Medium Priority (Advanced Tuning)

#### 7. **Enable Check Position Optimization** (Task 4.0)
- **Name**: `EnableCheckOptimization`
- **Type**: `check`
- **Default**: `true`
- **Description**: Optimize search parameters when king is in check with few legal moves.

#### 8. **Enable Time Budget** (Task 4.0)
- **Name**: `EnableTimeBudget`
- **Type**: `check`
- **Default**: `true`
- **Description**: Enable time budget allocation per depth in iterative deepening.

#### 9. **Enable Position Type Tracking** (Task 7.0)
- **Name**: `EnablePositionTypeTracking`
- **Type**: `check`
- **Default**: `true`
- **Description**: Track aspiration window statistics by game phase (opening/middlegame/endgame).

### Low Priority (Debug/Development)

#### 10. **Enable Statistics Tracking** (Task 7.0)
- **Name**: `EnableStatistics`
- **Type**: `check`
- **Default**: `false` (production builds)
- **Description**: Enable detailed statistics tracking (compile-time feature flag controlled).

#### 11. **Min Time Per Depth** (Task 4.0)
- **Name**: `MinTimePerDepth`
- **Type**: `spin`
- **Default**: `50`
- **Range**: `10-1000`
- **Description**: Minimum time to spend on each depth iteration in milliseconds.

#### 12. **Max Time Per Depth** (Task 4.0)
- **Name**: `MaxTimePerDepth`
- **Type**: `spin`
- **Default**: `0` (no limit)
- **Range**: `0-60000`
- **Description**: Maximum time to spend on each depth iteration (0 = no limit).

#### 13. **Enable Debug Telemetry** (Task 10.0)
- **Name**: `DebugLogging`
- **Type**: `check`
- **Default**: `false`
- **Description**: When enabled, emits evaluation telemetry (cache hit rates, interpolation counts, profiler averages) to the debug log to assist tuning sessions.

---

## Implementation Plan

### Phase 1: Fix Depth Option (Critical)

1. **Update USI handler** (`src/usi.rs`):
   ```rust
   "option name MaxDepth type spin default 0 min 0 max 100".to_string(),
   ```

2. **Update setoption handler** (`src/lib.rs`):
   - Change `depth` to accept `0` (unlimited)
   - Remove hardcoded max limit of 8
   - Map `0` to use engine's adaptive depth (no limit)
   - Update validation to allow 0-100

3. **Update engine logic**:
   - When depth = 0, don't set a limit in `IterativeDeepening`
   - Let time management control the depth naturally

### Phase 2: Expose High Priority Options

1. Add to USI `handle_usi()` response
2. Add handlers in `handle_setoption()`
3. Update UI `EngineOptionsModal.tsx` to support new options
4. Update engine configuration to use these options

### Phase 3: Expose Medium Priority Options

- Same process as Phase 2, but for advanced users

### Phase 4: Documentation

- Update `ENGINE_CONFIGURATION_GUIDE.md`
- Update USI protocol documentation
- Add tooltips/help text in UI

---

## Comparison with Other Engines

### Apery
- **Depth**: No depth option (unlimited)
- **Options**: Minimal, focuses on core search
- **Philosophy**: Let engine decide based on time

### YaneuraOu
- **Depth**: 0-2147483647 (default: 0 = unlimited/adaptive)
- **Options**: Extensive tuning options
- **Philosophy**: Expose many options for advanced users

### Our Engine (Current)
- **Depth**: 1-8 (default: 5) ⚠️ **TOO RESTRICTIVE**
- **Options**: Minimal
- **Philosophy**: Simple defaults, but needs more flexibility

### Our Engine (Recommended)
- **Depth**: 0-100 (default: 0 = adaptive)
- **Options**: Balance of essential options without overwhelming users
- **Philosophy**: Essential performance controls + advanced options for power users

---

## Recommendations Summary

### Immediate Actions

1. ✅ **Fix depth option**: Change to 0-100 range, default 0 (unlimited)
2. ✅ **Expose time management options**: TimeCheckFrequency, TimeSafetyMargin, TimeAllocationStrategy
3. ✅ **Expose aspiration window options**: EnableAspirationWindows, AspirationWindowSize

### Rationale

- **Depth limit of 8 is too low**: Engines regularly search to depth 20-40+ in complex positions
- **Default should be adaptive**: Users shouldn't need to set depth manually; engine should adapt
- **Time management is important**: New Task 4.0 and 8.0 features provide valuable controls
- **Aspiration windows are performance-critical**: Task 7.0 improvements should be accessible

### User Experience

- **Beginner users**: Leave defaults (depth = 0, all enabled)
- **Intermediate users**: Adjust depth limit if needed, tweak time settings
- **Advanced users**: Full access to all options for fine-tuning

---

## Files to Modify

1. `src/usi.rs` - Add options to USI response
2. `src/lib.rs` - Add setoption handlers
3. `src/components/EngineOptionsModal.tsx` - UI support for new options
4. `src/types/engine.ts` - Type definitions if needed
5. `docs/ENGINE_CONFIGURATION_GUIDE.md` - Documentation

---

## Next Steps

1. Review and approve this analysis
2. Implement Phase 1 (fix depth option)
3. Implement Phase 2 (high priority options)
4. Test with various USI clients
5. Update UI and documentation

