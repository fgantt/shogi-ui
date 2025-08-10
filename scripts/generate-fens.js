import { getInitialGameState, movePiece, generateStateHash, dropPiece } from '../src/game/engine.js';

// Helper function to convert Shogi coordinates (e.g., '77') to internal [row, col]
const shogiToInternal = (shogiCoord) => {
  const file = parseInt(shogiCoord[0], 10);
  const rank = parseInt(shogiCoord[1], 10);
  return [rank - 1, 9 - file];
};

const moves = [
  { from: '77', to: '76' },
  { from: '33', to: '34' },
  { from: '27', to: '26' },
  { from: '34', to: '35' },
  { from: '26', to: '25' },
  { from: '88', to: '22', promote: true },
  { from: '31', to: '42' },
  { from: '25', to: '24' },
  { from: 'drop', to: '23', pieceType: 'P' },
  { from: '24', to: '23', promote: true },
];

let gameState = getInitialGameState();
console.log('Initial FEN:', generateStateHash(gameState));

moves.forEach((move, index) => {
  console.log(`
Move ${index + 1}:`);
  console.log('Move data:', move);
  if (move.from === 'drop') {
    const to = shogiToInternal(move.to);
    console.log('Dropping piece:', move.pieceType, 'to:', to);
    gameState = dropPiece(gameState, move.pieceType, to);
  } else {
    const from = shogiToInternal(move.from);
    const to = shogiToInternal(move.to);
    console.log('Moving piece from:', from, 'to:', to, 'promote:', move.promote);
    gameState = movePiece(gameState, from, to, move.promote);
  }
  console.log('FEN after move:', generateStateHash(gameState));
});