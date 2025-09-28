# Computer Player Move Flow Diagram

This diagram shows the complete flow of how a computer player makes a move in the shogi game.

```mermaid
flowchart TD
    A[Game State Change] --> B{Is Current Player AI?}
    B -->|No| C[Wait for Human Player]
    B -->|Yes| D[Set isThinking = true]
    
    D --> E[Call getAiMove with gameState, difficulty, engineType]
    
    E --> F{Engine Type?}
    F -->|ai-wasm| G[WebAssembly Engine Path]
    F -->|ai-js| H[JavaScript Engine Path]
    F -->|Unknown| H
    
    %% WebAssembly Engine Path
    G --> I{WASM Engine Available?}
    I -->|Yes| J[Initialize WASM Engine if needed]
    I -->|No| K[Fallback to JavaScript Engine]
    
    J --> L[Convert Game State to WASM Format]
    L --> M[Setup Engine Position]
    M --> N[Get Time Limit based on Difficulty]
    N --> O[Call engine.get_best_move]
    O --> P{Move Found?}
    P -->|Yes| Q[Convert WASM Move to Game Format]
    P -->|No| R[Return null]
    
    %% JavaScript Engine Path
    H --> S[Initialize AI Worker if needed]
    S --> T[Set up Worker Message Handlers]
    T --> U[Post Message: gameState + difficulty]
    
    U --> V[AI Worker Processes Move]
    V --> W[Check Opening Book]
    W --> X[Generate Legal Moves]
    X --> Y[Apply Search Algorithm]
    Y --> Z[Return Best Move]
    
    Z --> AA[Worker Posts Message Back]
    AA --> BB[Resolve Promise with Move]
    
    %% Common Path After Move Generation
    Q --> CC[Return Move]
    R --> CC
    BB --> CC
    K --> CC
    
    CC --> DD{Move Format?}
    DD -->|Drop Move| EE[Execute dropPiece]
    DD -->|Regular Move| FF[Execute movePiece]
    DD -->|Invalid Format| GG[Log Error, Keep Current State]
    
    EE --> HH[Update Game State]
    FF --> HH
    GG --> HH
    
    HH --> II{Move Successful?}
    II -->|Yes| JJ[Update Last Move]
    II -->|No| KK[Switch Turn Back to Human]
    
    JJ --> LL[Check for Checkmate/Draw]
    KK --> LL
    
    LL --> MM{Game Over?}
    MM -->|Yes| NN[Set Winner/Game Over State]
    MM -->|No| OO[Switch to Next Player]
    
    NN --> PP[Set isThinking = false]
    OO --> PP
    
    PP --> QQ[Wait for Next Turn]
    
    %% Error Handling
    G --> RR[WASM Engine Error]
    RR --> SS[Log Error]
    SS --> K
    
    H --> TT[Worker Error]
    TT --> UU[Log Error]
    UU --> VV[Reject Promise]
    VV --> GG
    
    style A fill:#e1f5fe
    style B fill:#fff3e0
    style D fill:#f3e5f5
    style E fill:#e8f5e8
    style G fill:#e3f2fd
    style H fill:#fff8e1
    style CC fill:#f1f8e9
    style PP fill:#fce4ec
    style QQ fill:#e0f2f1
```

## Key Components

### 1. **Entry Point**
- Triggered when `gameState.currentPlayer` changes to an AI player
- Sets `isThinking = true` to show AI is calculating

### 2. **Engine Selection**
- **WebAssembly Engine (`ai-wasm`)**: High-performance Rust-based engine
- **JavaScript Engine (`ai-js`)**: Traditional JavaScript implementation with Web Worker

### 3. **WebAssembly Engine Flow**
- Converts game state to WASM format
- Sets up engine position
- Calculates time limit based on difficulty
- Calls `engine.get_best_move()` with difficulty level and time limit
- Converts result back to game format

### 4. **JavaScript Engine Flow**
- Uses Web Worker for non-blocking execution
- Checks opening book for known positions
- Generates legal moves
- Applies search algorithms (minimax, alpha-beta pruning)
- Returns best move via worker message

### 5. **Move Execution**
- Handles both regular moves and piece drops
- Updates game state
- Checks for game-ending conditions (checkmate, draw)
- Switches to next player

### 6. **Error Handling**
- WASM engine failures fall back to JavaScript
- Worker errors are logged and handled gracefully
- Invalid moves are logged and turn is switched back

### 7. **State Management**
- Maintains thinking state during AI calculation
- Updates game state after move execution
- Handles turn switching and game progression

## Performance Characteristics

- **WebAssembly Engine**: Fastest, used for high-difficulty play
- **JavaScript Engine**: More flexible, handles complex game logic
- **Web Worker**: Prevents UI blocking during AI calculation
- **Fallback System**: Ensures game continues even if preferred engine fails
