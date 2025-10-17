# Shogi Vibe

A high-performance desktop Shogi (Japanese Chess) game with advanced AI, built with Tauri and React. Play against an intelligent computer opponent with multiple difficulty levels, opening book strategies, and beautiful themes.

## 🎮 Download & Install

**[Download Latest Release](https://github.com/yourusername/shogi-game/releases/latest)**

- **macOS**: Download `.dmg` installer
- **Windows**: Download `.exe` or `.msi` installer  
- **Linux**: Download `.deb`, `.AppImage`, or `.rpm` package

See [BUILDING.md](BUILDING.md) for developer build instructions.

## Table of Contents
- [About Shogi](#about-shogi)
- [Rules of Shogi](#rules-of-shogi)
- [Piece Kanji Legend](#piece-kanji-legend)
- [Features](#features)
- [Getting Started](#getting-started)
- [Distribution](#distribution)


## About Shogi
Shogi shares similarities with Western chess but has unique and fascinating differences, most notably the "drop" rule, where captured pieces can be brought back into play. This significantly changes the game's dynamics, making it a deeply strategic and complex game.

![Shogi Board Screenshot](board.png)

## Rules of Shogi

**1. The Board and Pieces:**
*   **Board:** Shogi is played on a 9x9 uncheckered board.
*   **Pieces:** Each player starts with 20 pieces:
    *   1 King (王将 / 玉将 - Ōshō / Gyokushō)
    *   2 Gold Generals (金将 - Kinshō)
    *   2 Silver Generals (銀将 - Ginshō)
    *   2 Knights (桂馬 - Keima)
    *   2 Lances (香車 - Kyōsha)
    *   1 Rook (飛車 - Hisha)
    *   1 Bishop (角行 - Kakugyō)
    *   9 Pawns (歩兵 - Fuhyō)
*   **Piece Identification:** Unlike Western chess, all pieces are the same color. They are differentiated by their shape (a pentagon, with the pointed end facing forward) and kanji (Japanese characters) written on them. Players tell their pieces from their opponent's by the direction they are pointing.

**2. Setup:**
Pieces are set up as follows, from the player's side:
*   **First Row (nearest to player):** Lance, Knight, Silver General, Gold General, King (center), Gold General, Silver General, Knight, Lance.
*   **Second Row:** Bishop (second square from left), Rook (second square from right).
*   **Third Row:** All nine Pawns.

**3. Objective:**
The goal of Shogi is to checkmate the opponent's King. This means placing the King in a position where it is under attack (in "check") and has no legal move to escape capture on the next turn.

**4. Movement of Pieces:**
Each piece has a unique way of moving and capturing:

*   **King (King):** Moves one square in any direction (horizontally, vertically, or diagonally).
*   **Gold General (Gold):** Moves one square orthogonally (forward, backward, left, right) or one square diagonally forward. It cannot move diagonally backward.
*   **Silver General (Silver):** Moves one square forward, or one square diagonally in any direction. It cannot move sideways or directly backward.
*   **Knight (Knight):** Moves two squares forward and then one square sideways (L-shape, but only forward). It's the only piece that can jump over other pieces.
*   **Lance (Lance):** Moves any number of squares directly forward. It cannot move backward or sideways.
*   **Rook (Rook):** Moves any number of squares horizontally or vertically.
*   **Bishop (Bishop):** Moves any number of squares diagonally.
*   **Pawn (Pawn):** Moves one square directly forward. It captures by moving forward onto an opponent's piece, unlike Western chess where pawns capture diagonally.

**5. Capturing Pieces:**
When a piece moves onto a square occupied by an opponent's piece, the opponent's piece is captured. The captured piece is removed from the board and becomes part of the capturer's "pieces in hand."

**6. Promotion:**
*   **Promotion Zone:** The last three ranks (rows) on the opponent's side of the board are the "promotion zone."
*   **Option to Promote:** When most pieces (all except the King and Gold General) make a move that starts, ends, or passes through the promotion zone, the player has the *option* to promote that piece.
*   **Compulsory Promotion:** If a Pawn or Lance reaches the last rank, or a Knight reaches either of the last two ranks, it *must* be promoted, as it would have no legal moves otherwise.
*   **Promoted Moves:** When a piece is promoted, it is flipped over to reveal its promoted side (often with red characters).
    *   **Promoted Pawn, Lance, Knight, Silver General:** All promote to move like a Gold General.
    *   **Promoted Rook (Dragon King):** Moves like a Rook *and* a King.
    *   **Promoted Bishop (Dragon Horse):** Moves like a Bishop *and* a King.
*   **Captured Promoted Pieces:** If a promoted piece is captured, it reverts to its unpromoted state when it becomes a piece in hand.

**7. Dropping Pieces (The Most Unique Rule!):**
*   Instead of moving a piece on the board, a player can choose to "drop" a captured piece (from their "pieces in hand") onto any empty square on the board.
*   **Restrictions on Drops:**
    *   A piece is always dropped in its unpromoted state, even if dropped into the promotion zone. It can be promoted on a subsequent move.
    *   A piece cannot be dropped onto a square from which it would have no legal moves (e.g., a Pawn on the last rank, a Knight on the last two ranks, or a Lance on the last rank).
    *   **"Nifu" (Two Pawns on a File):** A Pawn cannot be dropped into a file (column) where you already have another *unpromoted* Pawn. (You can have an unpromoted Pawn and a promoted Pawn on the same file.)
    *   **"Uchifu-zume" (Pawn Drop Checkmate):** A Pawn cannot be dropped to give an immediate checkmate to the opponent's King. It can give check, but not a direct checkmate. Other pieces can.

**8. Winning and Draws:**
*   **Checkmate:** The game ends when a player checkmates the opponent's King.
*   **Resignation:** A player can resign if they believe their position is hopeless.
*   **Draws:** Draws are rare in Shogi due to the "dropping" rule.
    *   **Sennichite (Repetition):** If the exact same board position (including pieces in hand) occurs four times with the same player to move, the game is a draw. However, if this is caused by a perpetual check (one player repeatedly checking the King), the player giving the checks loses.
    *   **Jishogi (Impasse):** If both Kings have entered or can no longer be prevented from entering their respective promotion zones, and neither player can checkmate, the game can be declared a draw by counting pieces. Kings don't count, Rooks and Bishops are 5 points, and all other pieces are 1 point. If both players have at least 24 points, it's a draw.

## Piece Kanji Legend

| Piece Type        | Kanji | English Equivalent |
| :---------------- | :---- | :----------------- |
| King              | 王 / 玉 | Ō / Gyoku          |
| Rook              | 飛    | Hi                 |
| Bishop            | 角    | Kaku               |
| Gold General      | 金    | Kin                |
| Silver General    | 銀    | Gin                |
| Knight            | 桂    | Kei                |
| Lance             | 香    | Kyō                |
| Pawn              | 歩    | Fu                 |
| Promoted Rook     | 竜    | Ryū (Dragon King)  |
| Promoted Bishop   | 馬    | Uma (Dragon Horse) |
| Promoted Silver   | 全    | Nari (Promoted Silver) |
| Promoted Knight   | 圭    | Nari (Promoted Knight) |
| Promoted Lance    | 杏    | Nari (Promoted Lance)  |
| Promoted Pawn     | と    | To (Tokin)         |

## Features

### 🚀 High Performance
*   **Native Desktop App:** Built with Tauri for native performance and small bundle size
*   **Rust Engine:** Core game logic written in Rust with optimized move generation
*   **Advanced Opening Book:** Professional-grade opening strategies with intelligent caching
*   **Fast AI:** Multiple difficulty levels with optimized search algorithms

### 🎯 Gameplay
*   Single-player mode against an AI opponent
*   Three AI difficulty levels: Easy, Medium, and Hard
*   Click-to-move and drag-and-drop piece movement
*   Visual highlighting of legal moves
*   Promotion prompts for optional promotions
*   Visual indicators for check and last move
*   Complete move history log
*   Undo move functionality

### 🎨 User Experience
*   Beautiful, modern UI with multiple themes
*   Realistic piece movement sounds with audio controls
*   Customizable board and piece themes
*   Responsive design for various screen sizes
*   Smooth animations and transitions

## Getting Started

### For End Users

Download the installer for your platform from the [Releases](https://github.com/yourusername/shogi-game/releases) page and install it like any other application.

### For Developers

**Prerequisites:**
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v20 or later)

**Setup:**

1.  **Clone the repository:**
    ```bash
    git clone <repository_url>
    cd shogi-game/worktrees/tauri
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Run in development mode:**
    ```bash
    npm run tauri:dev
    ```

    This will build the Rust engine and launch the desktop app with hot-reload enabled.

4.  **Build for production:**
    ```bash
    npm run tauri:build
    ```

    Installers will be created in `src-tauri/target/release/bundle/`

See [BUILDING.md](BUILDING.md) for more details.

## WebAssembly Engine

The core game logic of this Shogi application is powered by a Rust-based engine compiled to WebAssembly (WASM). This allows for high-performance move calculation and AI, running directly in the browser.

### Building the WASM module

The pre-compiled WebAssembly module is included in the `pkg/` directory. However, if you make changes to the Rust source code (`.rs` files), you will need to rebuild the module.

**Prerequisites:**

*   [Rust](https://www.rust-lang.org/tools/install)
*   [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

**Build command:**

To rebuild the WebAssembly module, run the following command from the project root:

```bash
wasm-pack build --target web
```

This command will compile the Rust code, generate the necessary JavaScript bindings, and place the output in the `pkg/` directory.

## Sound Effects System

The game includes an immersive audio system that provides realistic sound effects for piece movements, enhancing the traditional shogi playing experience.

### Features

*   **Realistic Clacking Sounds:** Authentic wooden piece placement sounds that mimic traditional shogi gameplay
*   **Configurable Audio Settings:** Toggle sound effects on/off in the settings panel (enabled by default)
*   **Comprehensive Coverage:** Sound effects for all move types including:
    - Regular piece movements (clicks and drags)
    - Captured piece drops
    - Promotion moves
    - AI opponent moves
*   **Cross-Browser Compatibility:** Works across modern browsers with graceful fallbacks
*   **Synthetic Sound Generation:** Built-in Web Audio API sound generation when audio files are unavailable

### Audio Technology

The sound system uses modern web audio technologies:

*   **Web Audio API:** For high-quality sound generation and playback
*   **AudioContext Management:** Handles browser audio context suspension and user interaction requirements
*   **Fallback System:** Generates synthetic clacking sounds using noise and envelope shaping
*   **Memory Efficient:** Minimal memory footprint with on-demand audio generation

### Settings

Sound effects can be controlled through the game's settings panel:

1. Open the settings panel (gear icon)
2. Find the "Piece Movement Sounds" section
3. Toggle the switch to enable/disable sounds
4. Settings are automatically saved and persist between sessions

### Custom Audio Files

To use custom sound files:

1. Place your audio file as `public/sounds/piece-move.mp3`
2. Recommended format: MP3, 0.1-0.3 seconds duration
3. The system will automatically load the custom file
4. If no file is found, the system falls back to synthetic sound generation

## Opening Book System

The Shogi engine includes a sophisticated opening book system designed for high performance in WebAssembly environments. The opening book provides instant move suggestions for early-game positions, improving both AI play quality and response time.

### Key Features

*   **Binary Format:** Efficient binary storage format optimized for fast lookups and minimal memory usage
*   **Lazy Loading:** Positions are loaded on-demand to conserve memory, especially important for large opening books
*   **LRU Caching:** Frequently accessed positions are cached for optimal performance
*   **Streaming Support:** Large opening books can be loaded in chunks to handle memory constraints
*   **Memory Monitoring:** Built-in memory usage tracking and automatic optimization
*   **WASM Optimization:** FNV-1a hashing and `Box<[u8]>` storage for optimal WebAssembly performance

### Opening Book API

The opening book provides several methods for move selection:

```rust
// Get the best move for a position
let best_move = engine.get_best_move(&fen_string);

// Get a random move (weighted by frequency)
let random_move = engine.get_random_move(&fen_string);

// Get all available moves for a position
let all_moves = engine.get_all_opening_book_moves(&fen_string);

// Get opening book statistics
let stats = engine.get_opening_book_stats();
```

### Performance Benefits

*   **Instant Early-Game Moves:** No calculation time for common opening positions
*   **Memory Efficient:** Lazy loading and streaming support for large databases
*   **High Performance:** O(1) lookup time with intelligent caching
*   **WASM Optimized:** Designed specifically for WebAssembly constraints

### Building with Opening Book

The build script automatically converts the JSON opening book to binary format:

```bash
./build.sh
```

This will:
1. Convert `src/ai/openingBook.json` to binary format
2. Build the WebAssembly module
3. Copy the binary opening book to output directories

### Memory Usage

The opening book system includes comprehensive memory monitoring:

```rust
let memory_stats = engine.get_memory_usage();
println!("Total memory usage: {} bytes", memory_stats.total_size);
println!("Memory efficiency: {:.1}%", memory_stats.memory_efficiency);

// Automatic optimization
let optimization_result = engine.optimize_memory_usage();
println!("Applied {} optimizations", optimization_result.optimizations_applied);
```

## Distribution

### Building Installers

To create distributable installers for end users:

```bash
npm run tauri:build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg` disk image (Universal binary - Intel + Apple Silicon)
- **Windows**: `.exe` and `.msi` installers
- **Linux**: `.deb`, `.AppImage`, and `.rpm` packages

### Distribution Options

1. **GitHub Releases** (Recommended for open source)
   - Upload installers to GitHub Releases
   - Users download appropriate installer for their platform
   - Free hosting with version tracking

2. **Direct Download**
   - Host installers on your own website
   - Link directly to download files

3. **App Stores**
   - Mac App Store (requires Apple Developer account)
   - Microsoft Store (requires one-time fee)
   - Snap Store / Flathub (free for Linux)

### Code Signing

For production distribution, consider code signing to avoid security warnings:

- **macOS**: Requires Apple Developer account ($99/year) and notarization
- **Windows**: Requires code signing certificate ($100-400/year)
- **Linux**: Generally not required

### Complete Guide

For comprehensive instructions including:
- Code signing setup
- Auto-updates configuration
- CI/CD automation
- Optimization tips
- Troubleshooting

See the [**Distribution Guide**](docs/DISTRIBUTION_GUIDE.md).

## Technology Stack

- **Desktop Framework:** [Tauri 2.x](https://tauri.app/)
- **Frontend:** React 19 + TypeScript + Vite
- **Engine:** Rust (standalone binary via USI protocol)
- **Styling:** CSS with custom themes
- **Build System:** Cargo + npm

## Project Structure

```
.
├── src/                    # React frontend source
│   ├── components/         # React components
│   ├── hooks/              # Custom React hooks
│   ├── services/           # USI engine communication
│   └── styles/             # CSS and themes
├── src-tauri/              # Tauri backend
│   ├── src/                # Rust Tauri application
│   └── Cargo.toml          # Tauri dependencies
├── src/                    # Rust engine source
│   ├── ai/                 # AI implementation
│   ├── bin/                # Binary entry points
│   └── lib.rs              # Engine library
├── docs/                   # Documentation
├── public/                 # Static assets
└── Cargo.toml              # Engine dependencies
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT License - see LICENSE file for details.

