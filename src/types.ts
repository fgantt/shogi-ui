export interface GameSettings {
  difficulty: 'easy' | 'medium' | 'hard';
  player1Type: 'human' | 'ai';
  player2Type: 'human' | 'ai';
}

export interface GameState {
  board: any; // This will be the actual board type from tsshogi
  currentPlayer: 'player1' | 'player2';
  capturedPieces: { [key: string]: number };
  gameStatus: 'playing' | 'checkmate' | 'stalemate' | 'draw';
  lastMove: { from: any; to: any } | null; // Square type from tsshogi
  moveHistory: string[];
  isThinking: boolean;
  winner: 'player1' | 'player2' | 'draw' | null;
  difficulty: 'easy' | 'medium' | 'hard';
  engineType: 'ai-js' | 'ai-wasm';
  pieceSet: 'kanji' | 'international';
  showAttackedPieces: boolean;
  showPieceTooltips: boolean;
  currentWallpaper: string;
  currentBoardBackground: string;
  player1Type: 'human' | 'ai';
  player2Type: 'human' | 'ai';
}
