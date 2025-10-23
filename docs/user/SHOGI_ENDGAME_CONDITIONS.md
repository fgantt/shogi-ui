# Shogi Endgame Conditions

This document describes all possible endgame conditions in **shogi (Japanese chess)** as recognized in both casual and official (Japan Shogi Association) play.

---

## Overview

A shogi game can end in several ways: through checkmate, impasse, repetition, illegal move, time loss, or resignation. Unlike Western chess, shogi has unique rules regarding stalemate and specific illegal move conditions that are important for both players and engine implementations to understand.

---

## 🏁 1. Checkmate (Tsumi / 詰み)

### Definition
The opponent's king is attacked ("in check") and **has no legal move to escape**.

### Result
The player who delivers checkmate **wins**.

### Important Notes
- **Unlike chess, stalemate is not a draw** — it counts as a win for the player who delivers the final check.
- You can checkmate using drops (e.g., dropping a piece to deliver mate).
- This is the most definitive winning condition in shogi.

---

## 🙇‍♂️ 2. Resignation (Tōkyō / 投了)

### Definition
The losing player **resigns voluntarily**, often when checkmate is inevitable.

### Result
The player who did *not* resign **wins**.

### Important Notes
- Most professional and serious amateur games end this way.
- Players generally resign as soon as the situation becomes clearly hopeless.
- This is the most common way games end in practice, especially at higher levels of play.

---

## 🤝 3. Impasse (Jishōgi / 持将棋)

### Definition
Both kings have entered each other's camp (the opponent's three ranks), and neither side can reasonably deliver checkmate.

### Resolution Process

1. **Count Points** — Each player counts points based on their remaining pieces:
   - **King:** 0 points
   - **Rook/Dragon:** 5 points
   - **Bishop/Horse:** 5 points
   - **Gold, Silver, Knight, Lance, Pawn:** 1 point each

2. **Promotions** — Promoted pieces use these values:
   - Dragon (promoted Rook): 5 points
   - Horse (promoted Bishop): 5 points
   - All other promoted pieces: 1 point

3. **Determine Outcome:**
   - **If both players have 24 points or more** → The game is a **draw**
   - **If one side has less than 24 points** → That side **loses**

### Important Notes
- Common in professional games where both sides have advanced kings and locked positions.
- Sometimes called a "entering king" rule.
- The 24-point threshold is the standard rule, though some rulesets use different values.

---

## 🔁 4. Repetition (Sennichite / 千日手)

### Definition
The same board position (including who moves next and all pieces in hand) occurs **four times**.

### Result
- **Normally:** The game is a **draw**
- **Exception:** If the repetition includes **continuous checks by one player**, that player **loses**

### Important Notes
- This rule prevents perpetual check tactics.
- The position must be identical including:
  - All pieces on the board in the same positions
  - All pieces in hand for both players
  - The same player to move
- Professional games often result in a replay with colors reversed after sennichite.

---

## ❌ 5. Illegal Move (Hansoku-make / 反則負け)

### Definition
A player makes an illegal move according to shogi rules.

### Result
Immediate **loss** for the offending player.

### Common Examples of Illegal Moves

#### Nifu (二歩) — Double Pawn
- Dropping a pawn on a file that already has an unpromoted pawn belonging to the same player.
- **Important:** This only applies to unpromoted pawns. Promoted pawns (tokin) don't count.

#### Uchifuzume (打ち歩詰め) — Pawn Drop Mate
- Dropping a pawn to give **immediate checkmate**.
- **Note:** This is illegal ONLY if it gives immediate mate. Pawn drops that give check (but not mate) are legal.
- Moving a pawn to give checkmate IS legal — only dropping is prohibited.

#### Incorrect Piece Movement
- Moving a piece in a way that violates its movement rules.
- Example: Moving a knight backward or sideways.

#### Mandatory Promotion Violations
- Failing to promote when required.
- Example: A pawn, lance, or knight reaching a rank where it would have no legal moves if not promoted.

#### Illegal Promotions
- Promoting a piece that doesn't meet promotion conditions.
- Promoting a piece that cannot promote (e.g., gold, king).

#### Moving into Check
- Moving the king into check.
- Making any move that leaves one's own king in check.

### Important Notes
- In professional and serious amateur play, an illegal move ends the game instantly.
- In casual play, players might allow taking back the move.
- Engine implementations must carefully validate all moves, especially drops.

---

## ⏱️ 6. Time Loss (Jikan-gire / 時間切れ)

### Definition
A player runs out of **main time or byōyomi (countdown time)**.

### Result
The player whose clock hits zero **loses**.

### Important Notes
- In official time control formats, this is enforced strictly.
- Common time controls include:
  - **Main time + byōyomi:** Each player has a main time bank, then enters byōyomi (e.g., 30 seconds per move).
  - **Sudden death:** Fixed time with no increment.
  - **Fischer increment:** Time added after each move.
- Byōyomi violations (taking more than the allotted time) result in immediate loss.

---

## 💀 7. No Legal Move (Forced Loss without Check)

### Definition
A player has **no legal moves left** but is **not in check**.

### Result
The player with no legal moves **loses**.

### Important Notes
- This is extremely rare in practice.
- **There is no stalemate draw rule in shogi** — if you have no legal moves, you lose.
- This can theoretically occur if:
  - All your pieces are pinned or blocked.
  - Any king move would be into check.
  - You have no pieces in hand to drop.

---

## Summary Table

| Condition | Japanese Term | Result Type | Typical Outcome |
|-----------|--------------|-------------|-----------------|
| Checkmate | 詰み (Tsumi) | Win/Loss | Win for attacker |
| Resignation | 投了 (Tōkyō) | Win/Loss | Win for non-resigner |
| Impasse | 持将棋 (Jishōgi) | Draw or Win/Loss | Usually Draw |
| Repetition | 千日手 (Sennichite) | Draw or Loss | Draw (unless perpetual check) |
| Illegal Move | 反則負け (Hansoku-make) | Loss | Loss for offender |
| Time Loss | 時間切れ (Jikan-gire) | Loss | Loss for slower player |
| No Legal Move | — | Loss | Loss for player unable to move |

---

## Implementation Considerations

### For Engine Developers

1. **Checkmate Detection**
   - Must efficiently detect when a king is in check and has no legal escape.
   - Should consider both moves and drops when checking for escapes.

2. **Illegal Move Prevention**
   - Validate all moves before they are made.
   - Special attention to:
     - Nifu detection (track pawn positions by file).
     - Uchifuzume detection (check if pawn drop gives immediate mate).
     - Mandatory promotion enforcement.

3. **Repetition Detection**
   - Maintain a position history with Zobrist hashing or similar.
   - Track the move sequence to identify four-fold repetition.
   - Detect perpetual check patterns.

4. **Impasse (Jishōgi) Evaluation**
   - Implement the 24-point counting system.
   - Detect when both kings have entered enemy territory (promotion zone).
   - Allow claiming jishōgi when conditions are met.

5. **Time Management**
   - Implement time controls with byōyomi support.
   - Handle time forfeit conditions.

### For Players

- **Uchifuzume Rule:** Remember that pawn drops for mate are illegal, but pawn moves for mate are fine.
- **Nifu Rule:** Always check the file before dropping a pawn.
- **No Stalemate:** Unlike chess, running out of moves is a loss, not a draw.
- **Repetition:** Be aware that repeating positions may lead to a draw (or loss if checking continuously).

---

## Special Cases and Variants

### Computer Shogi Considerations

- **Mate Finding:** Computer engines are expected to find checkmate when it exists.
- **Illegal Move Handling:** Engines must never suggest illegal moves.
- **Draw Evaluation:** Engines should properly evaluate impasse and repetition scenarios.

### Handicap Games

- **Jishōgi Rules:** May differ in handicap games, sometimes with adjusted point thresholds.
- **Time Controls:** Often modified to balance the handicap.

### Tournament-Specific Rules

- **Sennichite Replays:** In professional tournaments, games that end in repetition are often replayed with reversed colors.
- **Jishōgi Point Counts:** While 24 points is standard, some tournaments use 27 points (27-point rule).
- **Try Rule (Trying Rule):** Some rulesets allow a player whose king reaches the enemy's back rank with sufficient supporting material to claim a win, though this is less common.

---

## References

- **Japan Shogi Association (JSA):** Official rulebook and regulations.
- **Computer Shogi Association (CSA):** Standard protocols for computer implementations.
- **Professional Shogi Players:** Game records demonstrating endgame conditions in practice.

---

## Glossary

| English | Japanese | Romaji | Kanji |
|---------|----------|--------|-------|
| Checkmate | Tsumi | つみ | 詰み |
| Resignation | Tōkyō | とうりょう | 投了 |
| Impasse | Jishōgi | じしょうぎ | 持将棋 |
| Repetition | Sennichite | せんにちて | 千日手 |
| Illegal Move | Hansoku-make | はんそくまけ | 反則負け |
| Double Pawn | Nifu | にふ | 二歩 |
| Pawn Drop Mate | Uchifuzume | うちふづめ | 打ち歩詰め |
| Time Loss | Jikan-gire | じかんぎれ | 時間切れ |
| Byōyomi | Byōyomi | びょうよみ | 秒読み |

---

*Last Updated: October 8, 2025*

