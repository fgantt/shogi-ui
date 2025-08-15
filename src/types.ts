export type PieceType = 'K' | 'R' | 'B' | 'G' | 'S' | 'N' | 'L' | 'P' | '+R' | '+B' | '+S' | '+N' | '+L' | '+P';
export type Player = 'player1' | 'player2';

export type Coords = [number, number];

export interface Piece {
  type: PieceType;
  player: Player;
}

export interface Move {
  from: number[] | 'drop';
  to: number[];
  piece: PieceType;
  player: Player;
  promote?: boolean;
  captured?: string | null;
  timestamp: string;
}

export interface GameState {
  board: (Piece | null)[][];
  currentPlayer: Player;
  capturedPieces: {
    player1: Piece[];
    player2: Piece[];
  };
  moveHistory: Move[];
  isCheck: boolean;
  isCheckmate: boolean;
  isDraw: boolean;
  kingPositions: {
    player1: [number, number];
    player2: [number, number];
  };
  pastStates: GameState[];
  promotionPending?: {
    from: [number, number];
    to: [number, number];
    piece: Piece;
  } | null;
}
