# Endgame Detection - Testing Instructions

**Status**: Ready for Testing  
**Build**: ✅ Successful  
**Date**: October 9, 2025

---

## Quick Start

```bash
cd /Users/fgantt/projects/vibe/shogi-game/worktrees/usi
npm run dev
```

Then open your browser to `http://localhost:5173` (or the port shown in the terminal).

---

## Test Scenarios

### 🔴 Test 1: Human vs AI - AI Gets Checkmated (CRITICAL)

**Purpose**: Verify AI doesn't loop infinitely when checkmated and modal appears correctly.

**Steps**:
1. Start a new game:
   - Player 1 (Sente): Human
   - Player 2 (Gote): AI
   - Difficulty: Easy (Level 3) for faster testing
2. Play moves to checkmate the AI (White/Gote)
3. When AI is checkmated, observe:
   - Console shows: `[CONTROLLER] NO LEGAL MOVES - GAME OVER!`
   - Console shows: `[GAMEPAGE] GAME OVER EVENT RECEIVED!`
   - Modal appears immediately
   - No infinite search loop or hanging

**Expected Results**:
- ✅ Modal appears with title "Checkmate!" and emoji 👑
- ✅ Message shows: "Sente (Player 1) wins by checkmate (Tsumi / 詰み)!"
- ✅ Console logs show proper event propagation
- ✅ No infinite loop or performance issues
- ✅ "New Game" and "Review Position" buttons are clickable

**Console Indicators**:
```
[CONTROLLER] Legal moves available: false
[CONTROLLER] NO LEGAL MOVES - GAME OVER! Winner: player1, Type: checkmate
[GAMEPAGE] GAME OVER EVENT RECEIVED!
[GAMEPAGE] Setting winner to: player1
[GAMEPAGE] Endgame type: checkmate
```

---

### 🔴 Test 2: AI vs Human - Human Gets Checkmated (CRITICAL)

**Purpose**: Verify human checkmate is detected correctly.

**Steps**:
1. Start a new game:
   - Player 1 (Sente): AI
   - Player 2 (Gote): Human
   - Difficulty: Easy
2. Allow AI to checkmate you (or use a position with checkmate)
3. Observe modal and console

**Expected Results**:
- ✅ Modal appears when human is checkmated
- ✅ Correct winner is displayed (AI/Player 1)
- ✅ Game stops accepting moves
- ✅ Modal shows "Sente (Player 1) wins by checkmate (Tsumi / 詰み)!"

---

### 🔴 Test 3: Human vs Human - Checkmate (CRITICAL)

**Purpose**: Verify checkmate detection works without AI.

**Steps**:
1. Start a new game:
   - Player 1: Human
   - Player 2: Human
2. Play a game to checkmate
3. Observe modal

**Expected Results**:
- ✅ Modal appears immediately upon checkmate
- ✅ Correct winner displayed
- ✅ Proper Japanese terminology shown

---

### 🟡 Test 4: Four-Fold Repetition - Draw (HIGH PRIORITY)

**Purpose**: Verify Sennichite (千日手) repetition detection.

**Steps**:
1. Start any game mode
2. Create a repeating sequence of moves (e.g., moving pieces back and forth)
3. Repeat the exact same position 4 times
4. Observe what happens on the 4th repetition

**Expected Results**:
- ✅ After 4th occurrence of same position, game ends
- ✅ Modal shows title "Draw" with emoji 🤝
- ✅ Message: "The game is a draw by four-fold repetition (Sennichite / 千日手)."
- ✅ Console shows: `[CONTROLLER] FOUR-FOLD REPETITION DETECTED (Sennichite)!`

**Console Indicators**:
```
[CTRL-...] Position history updated. Count for current position: 1
[CTRL-...] Position history updated. Count for current position: 2
[CTRL-...] Position history updated. Count for current position: 3
[CTRL-...] FOUR-FOLD REPETITION DETECTED (Sennichite)!
```

**Note**: Position must be EXACTLY the same (including whose turn it is and all pieces in hand).

---

### 🟡 Test 5: AI Resignation (HIGH PRIORITY)

**Purpose**: Verify AI resignation is handled correctly.

**Steps**:
1. Start Human vs AI game
2. Create an overwhelming advantage (capture many pieces)
3. Continue playing until AI is in a hopeless position
4. Wait for AI to return "resign" move

**Expected Results**:
- ✅ Modal appears when AI resigns
- ✅ Title: "Resignation" with emoji 🏳️
- ✅ Message shows which player resigned and who won
- ✅ Console shows: `[CONTROLLER] AI RESIGNED!`

**Alternative Test**: Set up a position with minimal AI pieces and see if it resigns.

---

### 🟢 Test 6: Stalemate / No Legal Moves (NICE TO HAVE)

**Purpose**: Verify stalemate counts as a loss (Shogi rule).

**Steps**:
1. Create a position where a player has no legal moves but is NOT in check
2. Make it that player's turn
3. Observe behavior

**Expected Results**:
- ✅ Game ends immediately
- ✅ Modal shows "No Legal Moves" with emoji 🚫
- ✅ Message explains: "In Shogi, this counts as a loss"
- ✅ Player with no moves loses

**Note**: This is very rare in actual play.

---

## Debugging Tips

### Enable Detailed Logging

The implementation includes comprehensive logging. Check your browser console for:

1. **Controller Events**:
   - `[CONTROLLER] Checking endgame conditions...`
   - `[CONTROLLER] Legal moves available: [true/false]`
   - `[CONTROLLER] NO LEGAL MOVES - GAME OVER! Winner: [player]`

2. **GamePage Events**:
   - `[GAMEPAGE] GAME OVER EVENT RECEIVED!`
   - `[GAMEPAGE] Setting winner to: [winner]`
   - `[GAMEPAGE] Endgame type: [type]`

3. **Position History**:
   - `[CTRL-...] Position history updated. Count for current position: [N]`
   - `[CTRL-...] Position history cleared for new game`

### Common Issues

**Modal Doesn't Appear**:
- Check console for `[GAMEPAGE] GAME OVER EVENT RECEIVED!`
- Verify `winner` state is being set
- Check if modal is hidden by CSS/z-index issues

**Infinite Loop**:
- Check console for repeated search requests
- Verify `checkEndgameConditions()` is being called
- Check if game over event is being emitted

**Wrong Winner**:
- Check console logs to see calculated winner
- Verify turn detection logic (`position.sfen.includes(' b ')`)

---

## Manual Testing Positions

### Quick Checkmate Position (for testing)

You can load a position close to checkmate using the "Load Game" feature:

**Near Checkmate SFEN**:
```
lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1
```

This is just the starting position, but you can create custom positions or play a few moves to reach checkmate quickly.

---

## Checklist

After testing, verify:

### Critical Features ✅
- [ ] Human vs AI checkmate detection works
- [ ] AI vs Human checkmate detection works  
- [ ] Human vs Human checkmate detection works
- [ ] No infinite AI search loops
- [ ] Modal appears correctly
- [ ] Correct winner displayed
- [ ] Japanese terminology shown

### High Priority Features ✅
- [ ] Four-fold repetition detected (Sennichite)
- [ ] AI resignation handled properly
- [ ] Stalemate detection works
- [ ] Modal shows correct endgame type for each scenario

### User Experience ✅
- [ ] Modal is readable and attractive
- [ ] Emojis display correctly
- [ ] Messages are clear and educational
- [ ] "New Game" button works after game over
- [ ] "Review Position" button works after game over
- [ ] No console errors

---

## Reporting Issues

If you find any issues, please note:

1. **What You Did**: Exact steps to reproduce
2. **What Happened**: Actual behavior
3. **What You Expected**: Expected behavior
4. **Console Logs**: Copy relevant console output
5. **Browser**: Which browser and version
6. **Position**: SFEN notation if applicable

---

## Success Criteria

The implementation is considered successful if:

✅ **All Critical Tests Pass**: Checkmate is detected in all game modes without infinite loops  
✅ **High Priority Tests Pass**: Repetition and resignation work correctly  
✅ **Modal Functions Correctly**: Displays proper information for each endgame type  
✅ **No Console Errors**: Clean execution with only informational logs  
✅ **Game Continues Properly**: Can start new games after game over

---

## Additional Notes

- The implementation uses extensive logging for debugging
- All endgame types include proper Japanese terminology
- Modal is designed to be educational about Shogi rules
- Code is production-ready pending successful testing

**Ready to Test!** 🎮

---

**Last Updated**: October 9, 2025  
**Implementation**: See `ENDGAME_DETECTION_IMPLEMENTATION_COMPLETE.md`

