/**
 * Game settings used to initialize or reconfigure a session.
 * Cross-language mapping:
 * - Rust `types::search::TimeManagementConfig` ↔ `minutesPerSide`, `byoyomiInSeconds`
 * - Rust `types::core::Player` ↔ `player1Type`/`player2Type` (controller-level)
 */
export interface GameSettings {
  player1Type: 'human' | 'ai';
  player2Type: 'human' | 'ai';
  minutesPerSide: number;
  byoyomiInSeconds: number;
  initialSfen?: string;
  player1EngineId?: string | null;
  player2EngineId?: string | null;
  player1TempOptions?: {[key: string]: string};
  player2TempOptions?: {[key: string]: string};
  useTauriEngine?: boolean;
}

/**
 * Frontend game state snapshot.
 * Cross-language mapping:
 * - Rust `types::core::Position`/`Move` → serialized strings in `moveHistory`
 * - Rust board state serialized via Tauri bridge → `board` shape
 */
export interface GameState {
  board: any; // serialized board representation from backend
  currentPlayer: 'player1' | 'player2';
  capturedPieces: { [key: string]: number };
  gameStatus: 'playing' | 'checkmate' | 'stalemate' | 'draw';
  lastMove: { from: any; to: any } | null;
  moveHistory: string[];
  isThinking: boolean;
  winner: 'player1' | 'player2' | 'draw' | null;
  difficulty: 'easy' | 'medium' | 'hard';
  engineType: 'ai-js';
  pieceSet: 'kanji' | 'international';
  showAttackedPieces: boolean;
  showPieceTooltips: boolean;
  currentWallpaper: string;
  currentBoardBackground: string;
  player1Type: 'human' | 'ai';
  player2Type: 'human' | 'ai';
}
