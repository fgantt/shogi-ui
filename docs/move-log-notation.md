# Shogi Move Notations
## Overview
Shogi has several different notations systems used to record games. Here’s a breakdown of the main ones:

1. KIF (棋譜, Japanese descriptive notation)

   - The most traditional style, used in newspapers, books, and professional records in Japan.

   - Combines kanji numbers + piece names.

   - Example:

     - ▲７六歩 → Black moves pawn to 7-6.

     - △３四歩 → White moves pawn to 3-4.

   - Can include origin square in parentheses: ▲７六歩(77).

   - Symbols:

     - ▲ = Black (先手 sente), △ = White (後手 gote).

2. Western / English Notation

   - Uses Arabic numerals for squares and English letters for pieces.

   - Example:

     - P-7f = pawn moves to 7f (same as ▲７六歩).

     - B*5e = bishop dropped on 5e.

   - Common in English-language Shogi books.

   - Pieces: P (pawn), L (lance), N (knight), S (silver), G (gold), B (bishop), R (rook), K (king), + prefix for promoted pieces (e.g., +B = horse, +R = dragon).

3. CSA Notation (Computer Shogi Association format)

   - A plain-text, computer-friendly format widely used in software and engines.

   - Each move = origin + destination square + piece.

   - Example:

     - +7776FU → Black pawn from 77 to 76.

     - -3334FU → White pawn from 33 to 34.

     - +0055KA → Black drops a bishop on 55.

   - + means Black, - means White.

4. USI (Universal Shogi Interface)

   - Protocol similar to chess’s UCI, used for shogi engines.

   - Coordinates only, no piece names.

   - Example:

     - 7g7f → pawn from 7g to 7f.

     - 3c3d → pawn from 3c to 3d.

     - B*5e → bishop drop on 5e.

   - Promotion marked with a + at the end: 2b3c+.


## Comparison


We’ll use “Black moves from 7七 → 7六” as the base example, then show promotion and drop.

### Shogi Notation Comparison Table
<div class="_tableContainer_1rjym_1"><div tabindex="-1" class="group w-fit _tableWrapper_1rjym_13 flex flex-col-reverse"><table data-start="290" data-end="921" class="w-fit min-w-(--thread-content-width)"><thead data-start="290" data-end="371"><tr data-start="290" data-end="371"><th data-start="290" data-end="307" data-col-size="sm">Piece / Action</th><th data-start="307" data-end="328" data-col-size="sm"><strong data-start="309" data-end="327">KIF (Japanese)</strong></th><th data-start="328" data-end="342" data-col-size="sm"><strong data-start="330" data-end="341">English</strong></th><th data-start="342" data-end="352" data-col-size="sm"><strong data-start="344" data-end="351">CSA</strong></th><th data-start="352" data-end="362" data-col-size="sm"><strong data-start="354" data-end="361">USI</strong></th><th data-start="362" data-end="371" data-col-size="sm">Notes</th></tr></thead><tbody data-start="455" data-end="921"><tr data-start="455" data-end="504"><td data-start="455" data-end="470" data-col-size="sm"><strong data-start="457" data-end="469">Pawn (歩)</strong></td><td data-start="470" data-end="477" data-col-size="sm">▲７六歩</td><td data-start="477" data-end="484" data-col-size="sm">P-7f</td><td data-start="484" data-end="494" data-col-size="sm">+7776FU</td><td data-start="494" data-end="501" data-col-size="sm">7g7f</td><td data-start="501" data-end="504" data-col-size="sm"></td></tr><tr data-start="505" data-end="555"><td data-start="505" data-end="521" data-col-size="sm"><strong data-start="507" data-end="520">Lance (香)</strong></td><td data-start="521" data-end="528" data-col-size="sm">▲９六香</td><td data-start="528" data-end="535" data-col-size="sm">L-9f</td><td data-start="535" data-end="545" data-col-size="sm">+9996KY</td><td data-start="545" data-end="552" data-col-size="sm">9i9f</td><td data-start="552" data-end="555" data-col-size="sm"></td></tr><tr data-start="556" data-end="607"><td data-start="556" data-end="573" data-col-size="sm"><strong data-start="558" data-end="572">Knight (桂)</strong></td><td data-start="573" data-end="580" data-col-size="sm">▲８八桂</td><td data-start="580" data-end="587" data-col-size="sm">N-8h</td><td data-start="587" data-end="597" data-col-size="sm">+8988KE</td><td data-start="597" data-end="604" data-col-size="sm">8i8h</td><td data-start="604" data-end="607" data-col-size="sm"></td></tr><tr data-start="608" data-end="659"><td data-start="608" data-end="625" data-col-size="sm"><strong data-start="610" data-end="624">Silver (銀)</strong></td><td data-start="625" data-end="632" data-col-size="sm">▲７八銀</td><td data-start="632" data-end="639" data-col-size="sm">S-7h</td><td data-start="639" data-end="649" data-col-size="sm">+7978GI</td><td data-start="649" data-end="656" data-col-size="sm">7i7h</td><td data-start="656" data-end="659" data-col-size="sm"></td></tr><tr data-start="660" data-end="709"><td data-start="660" data-end="675" data-col-size="sm"><strong data-start="662" data-end="674">Gold (金)</strong></td><td data-start="675" data-end="682" data-col-size="sm">▲６八金</td><td data-start="682" data-end="689" data-col-size="sm">G-6h</td><td data-start="689" data-end="699" data-col-size="sm">+6968KI</td><td data-start="699" data-end="706" data-col-size="sm">6i6h</td><td data-start="706" data-end="709" data-col-size="sm"></td></tr><tr data-start="710" data-end="783"><td data-start="710" data-end="727" data-col-size="sm"><strong data-start="712" data-end="726">Bishop (角)</strong></td><td data-start="727" data-end="734" data-col-size="sm">▲７七角</td><td data-start="734" data-end="741" data-col-size="sm">B-7g</td><td data-start="741" data-end="751" data-col-size="sm">+8877KA</td><td data-start="751" data-end="758" data-col-size="sm">8h7g</td><td data-start="758" data-end="783" data-col-size="sm">Often placed at start</td></tr><tr data-start="784" data-end="833"><td data-start="784" data-end="799" data-col-size="sm"><strong data-start="786" data-end="798">Rook (飛)</strong></td><td data-start="799" data-end="806" data-col-size="sm">▲２六飛</td><td data-start="806" data-end="813" data-col-size="sm">R-2f</td><td data-start="813" data-end="823" data-col-size="sm">+2826HI</td><td data-start="823" data-end="830" data-col-size="sm">2h2f</td><td data-start="830" data-end="833" data-col-size="sm"></td></tr><tr data-start="834" data-end="921"><td data-start="834" data-end="851" data-col-size="sm"><strong data-start="836" data-end="850">King (玉/王)</strong></td><td data-start="851" data-end="858" data-col-size="sm">▲５八玉</td><td data-start="858" data-end="865" data-col-size="sm">K-5h</td><td data-start="865" data-end="875" data-col-size="sm">+5958OU</td><td data-start="875" data-end="882" data-col-size="sm">5i5h</td><td data-start="882" data-end="921" data-col-size="sm">王 (White), 玉 (Black) in pro records</td></tr></tbody></table>

### Promoted Pieces

<table data-start="951" data-end="1485" class="w-fit min-w-(--thread-content-width)"><thead data-start="951" data-end="1032"><tr data-start="951" data-end="1032"><th data-start="951" data-end="968" data-col-size="sm">Promoted Piece</th><th data-start="968" data-end="989" data-col-size="sm"><strong data-start="970" data-end="988">KIF (Japanese)</strong></th><th data-start="989" data-end="1003" data-col-size="sm"><strong data-start="991" data-end="1002">English</strong></th><th data-start="1003" data-end="1013" data-col-size="sm"><strong data-start="1005" data-end="1012">CSA</strong></th><th data-start="1013" data-end="1023" data-col-size="sm"><strong data-start="1015" data-end="1022">USI</strong></th><th data-start="1023" data-end="1032" data-col-size="sm">Notes</th></tr></thead><tbody data-start="1116" data-end="1485"><tr data-start="1116" data-end="1172"><td data-start="1116" data-end="1136" data-col-size="sm">Promoted Pawn (と)</td><td data-start="1136" data-end="1143" data-col-size="sm">▲７六と</td><td data-start="1143" data-end="1151" data-col-size="sm">+P-7f</td><td data-start="1151" data-end="1161" data-col-size="sm">+7776TO</td><td data-start="1161" data-end="1169" data-col-size="sm">7g7f+</td><td data-start="1169" data-end="1172" data-col-size="sm"></td></tr><tr data-start="1173" data-end="1232"><td data-start="1173" data-end="1195" data-col-size="sm">Promoted Lance (成香)</td><td data-start="1195" data-end="1203" data-col-size="sm">▲９六成香</td><td data-start="1203" data-end="1211" data-col-size="sm">+L-9f</td><td data-start="1211" data-end="1221" data-col-size="sm">+9996NY</td><td data-start="1221" data-end="1229" data-col-size="sm">9i9f+</td><td data-start="1229" data-end="1232" data-col-size="sm"></td></tr><tr data-start="1233" data-end="1293"><td data-start="1233" data-end="1256" data-col-size="sm">Promoted Knight (成桂)</td><td data-start="1256" data-end="1264" data-col-size="sm">▲８八成桂</td><td data-start="1264" data-end="1272" data-col-size="sm">+N-8h</td><td data-start="1272" data-end="1282" data-col-size="sm">+8988NK</td><td data-start="1282" data-end="1290" data-col-size="sm">8i8h+</td><td data-start="1290" data-end="1293" data-col-size="sm"></td></tr><tr data-start="1294" data-end="1354"><td data-start="1294" data-end="1317" data-col-size="sm">Promoted Silver (成銀)</td><td data-start="1317" data-end="1325" data-col-size="sm">▲７八成銀</td><td data-start="1325" data-end="1333" data-col-size="sm">+S-7h</td><td data-start="1333" data-end="1343" data-col-size="sm">+7978NG</td><td data-start="1343" data-end="1351" data-col-size="sm">7i7h+</td><td data-start="1351" data-end="1354" data-col-size="sm"></td></tr><tr data-start="1355" data-end="1420"><td data-start="1355" data-end="1384" data-col-size="sm">Horse (馬, promoted bishop)</td><td data-start="1384" data-end="1391" data-col-size="sm">▲６六馬</td><td data-start="1391" data-end="1399" data-col-size="sm">+B-6f</td><td data-start="1399" data-end="1409" data-col-size="sm">+7766UM</td><td data-start="1409" data-end="1417" data-col-size="sm">7g6f+</td><td data-start="1417" data-end="1420" data-col-size="sm"></td></tr><tr data-start="1421" data-end="1485"><td data-start="1421" data-end="1449" data-col-size="sm">Dragon (竜, promoted rook)</td><td data-start="1449" data-end="1456" data-col-size="sm">▲２六竜</td><td data-start="1456" data-end="1464" data-col-size="sm">+R-2f</td><td data-start="1464" data-end="1474" data-col-size="sm">+2826RY</td><td data-start="1474" data-end="1482" data-col-size="sm">2h2f+</td><td data-start="1482" data-end="1485" data-col-size="sm"></td></tr></tbody></table>

### Drops (piece placed from hand)

<table data-start="1530" data-end="1855" class="w-fit min-w-(--thread-content-width)"><thead data-start="1530" data-end="1607"><tr data-start="1530" data-end="1607"><th data-start="1530" data-end="1543" data-col-size="sm">Piece Drop</th><th data-start="1543" data-end="1564" data-col-size="sm"><strong data-start="1545" data-end="1563">KIF (Japanese)</strong></th><th data-start="1564" data-end="1578" data-col-size="sm"><strong data-start="1566" data-end="1577">English</strong></th><th data-start="1578" data-end="1588" data-col-size="sm"><strong data-start="1580" data-end="1587">CSA</strong></th><th data-start="1588" data-end="1598" data-col-size="sm"><strong data-start="1590" data-end="1597">USI</strong></th><th data-start="1598" data-end="1607" data-col-size="sm">Notes</th></tr></thead><tbody data-start="1687" data-end="1855"><tr data-start="1687" data-end="1757"><td data-start="1687" data-end="1699" data-col-size="sm">Pawn drop</td><td data-start="1699" data-end="1707" data-col-size="sm">▲５五歩打</td><td data-start="1707" data-end="1714" data-col-size="sm">P*5e</td><td data-start="1714" data-end="1724" data-col-size="sm">+0055FU</td><td data-start="1724" data-end="1731" data-col-size="sm">P*5e</td><td data-start="1731" data-end="1757" data-col-size="sm">打 (“drop”) in Japanese</td></tr><tr data-start="1758" data-end="1807"><td data-start="1758" data-end="1772" data-col-size="sm">Bishop drop</td><td data-start="1772" data-end="1780" data-col-size="sm">▲５五角打</td><td data-start="1780" data-end="1787" data-col-size="sm">B*5e</td><td data-start="1787" data-end="1797" data-col-size="sm">+0055KA</td><td data-start="1797" data-end="1804" data-col-size="sm">B*5e</td><td data-start="1804" data-end="1807" data-col-size="sm"></td></tr><tr data-start="1808" data-end="1855"><td data-start="1808" data-end="1820" data-col-size="sm">Rook drop</td><td data-start="1820" data-end="1828" data-col-size="sm">▲５五飛打</td><td data-start="1828" data-end="1835" data-col-size="sm">R*5e</td><td data-start="1835" data-end="1845" data-col-size="sm">+0055HI</td><td data-start="1845" data-end="1852" data-col-size="sm">R*5e</td><td data-start="1852" data-end="1855" data-col-size="sm"></td></tr></tbody></table>


### Summary:

- KIF = Japanese, human-readable with kanji pieces.
- English = Western, with letters & algebraic squares.
- CSA = machine-friendly, explicit piece codes.
- USI = ultra-compact, coordinates only, + for promotion, * for drop.

## KIF (棋譜) / KI2 Format Specification
### General

- Used for human-readable Japanese records.
- Uses Kanji numerals and piece names.
- KIF: includes metadata (players, tournament info, comments, time).
- KI2: lightweight, move-only version.

### Syntax

- Player symbols:
  - ▲ = Black (先手 sente)
  - △ = White (後手 gote)

- Coordinates:
  - File = 1–9, written in Arabic numerals.
  - Rank = 一–九 (kanji numbers).

- Piece names: 歩, 香, 桂, 銀, 金, 角, 飛, 玉/王.

- Promotion: add 成.

- Drops: add 打.

- If origin square disambiguation needed: append (file,rank) in parentheses.

### Examples
```
▲７六歩
△３四歩
▲６六馬(77)
△５五歩打
```

## English Notation Specification
### General

- Used in English-language Shogi books.
- Based on algebraic chess-style coordinates with files = 1–9, ranks = a–i (from Black’s perspective).
- Piece letters: P, L, N, S, G, B, R, K. Promoted pieces prefixed with +.

### Syntax

- Move = `<Piece><action><Destination>`

- Action:
  - - = move (from a square, implicit if unique).
  - * = drop.

- Promotion: add + at end.

### Examples
```
P-7f      (Pawn to 7f)
B*5e      (Bishop drop on 5e)
+R-2f     (Promoted rook (dragon) moves to 2f)
P-7f+     (Pawn promotes on 7f)
```

## CSA Format (Computer Shogi Association) Specification
### General

- Text-based, machine-readable.
- Each move = <Player><From><To><Piece><Promotion?>.
- Player: + (Black), - (White).
- Square coordinates: 2 digits each (file, rank). 00 = “from hand”.
- Piece codes (2 letters): FU, KY, KE, GI, KI, KA, HI, OU, TO, NY, NK, NG, UM, RY.
- Promotion: trailing +.

### Syntax
- `<sign><from><to><piece>[+]`

### CSA Piece Codes
| CSA Code | Japanese Name | English Name | Promoted Form | Notes                 |
| -------- | ------------- | ------------ | ------------- | --------------------- |
| **FU**   | 歩兵 (fu)       | Pawn         | **TO** (と金)   |                       |
| **KY**   | 香車 (kyōsha)   | Lance        | **NY** (成香)   |                       |
| **KE**   | 桂馬 (keima)    | Knight       | **NK** (成桂)   |                       |
| **GI**   | 銀将 (ginshō)   | Silver       | **NG** (成銀)   |                       |
| **KI**   | 金将 (kinshō)   | Gold         | —             | Gold does not promote |
| **KA**   | 角行 (kakugyō)  | Bishop       | **UM** (馬)    | 馬 = “Horse”           |
| **HI**   | 飛車 (hisha)    | Rook         | **RY** (竜)    | 竜 = “Dragon King”     |
| **OU**   | 王将 / 玉将       | King         | —             | No promotion          |

#### Promoted Codes
- TO → Promoted pawn (と金 / Tokin)
- NY → Promoted lance (成香 / Narikyō)
- NK → Promoted knight (成桂 / Narikei)
- NG → Promoted silver (成銀 / Narigin)
- UM → Promoted bishop (馬 / Uma, “Horse”)
- RY → Promoted rook (竜 / Ryū, “Dragon”)

### Examples
```
+7776FU    (Black pawn from 77 to 76)
-3334FU    (White pawn from 33 to 34)
+0055KA    (Black drops bishop at 55)
+8877KA+   (Black bishop from 88 to 77, promotes to UM)
```

## USI (Universal Shogi Interface) Specification
### General

- Engine protocol, compact coordinate-based.
- Similar to UCI (chess).
- Coordinates: algebraic, files 1–9, ranks a–i.
- Moves = `<from><to>[+]` or `<piece>*<to>`.

### Syntax

- Normal move: `<from><to>`
- Promotion: add +
- Drop: `<Piece>*<to>`
- Piece letters: P, L, N, S, G, B, R, K. Promoted = + prefix.

### Examples
```
7g7f     (Pawn from 7g to 7f)
3c3d     (Pawn from 3c to 3d)
B*5e     (Bishop drop at 5e)
2b3c+    (Bishop from 2b to 3c, promotes to UM)
```

## Complete Shogi Notation Cheat Sheet
| CSA Code      | KIF/KI2 (Japanese) | English (Western) | USI | Notes                             |
| ------------- | ------------------ | ----------------- | --- | --------------------------------- |
| **FU**        | 歩                  | P                 | P   | Pawn                              |
| **FU (drop)** | 歩打                 | P\*               | P\* | Pawn drop                         |
| **TO**        | と                  | +P                | +P  | Promoted pawn                     |
| **KY**        | 香                  | L                 | L   | Lance                             |
| **KY (drop)** | 香打                 | L\*               | L\* | Lance drop                        |
| **NY**        | 成香                 | +L                | +L  | Promoted lance                    |
| **KE**        | 桂                  | N                 | N   | Knight                            |
| **KE (drop)** | 桂打                 | N\*               | N\* | Knight drop                       |
| **NK**        | 成桂                 | +N                | +N  | Promoted knight                   |
| **GI**        | 銀                  | S                 | S   | Silver                            |
| **GI (drop)** | 銀打                 | S\*               | S\* | Silver drop                       |
| **NG**        | 成銀                 | +S                | +S  | Promoted silver                   |
| **KI**        | 金                  | G                 | G   | Gold (never promotes)             |
| **KI (drop)** | 金打                 | G\*               | G\* | Gold drop                         |
| **KA**        | 角                  | B                 | B   | Bishop                            |
| **KA (drop)** | 角打                 | B\*               | B\* | Bishop drop                       |
| **UM**        | 馬                  | +B                | +B  | Promoted bishop (Horse)           |
| **HI**        | 飛                  | R                 | R   | Rook                              |
| **HI (drop)** | 飛打                 | R\*               | R\* | Rook drop                         |
| **RY**        | 竜 (龍)              | +R                | +R  | Promoted rook (Dragon)            |
| **OU**        | 王 / 玉              | K                 | K   | King (never promotes)             |
| **OU (drop)** | 王打 / 玉打            | K\*               | K\* | King drop (rare, usually illegal) |

