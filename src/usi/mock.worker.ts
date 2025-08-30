
// src/usi/mock.worker.ts
// A mock AI worker that simulates a real USI engine for testing purposes.
// It understands basic USI commands and returns a fixed or random legal move.

import { Position } from 'tsshogi';

let position = new Position();

self.onmessage = (e: MessageEvent) => {
  const { command, ...args } = e.data;

  switch (command) {
    case 'usi':
      self.postMessage({ command: 'id', name: 'MockEngine' });
      self.postMessage({ command: 'id', author: 'Test' });
      self.postMessage({ command: 'usiok' });
      break;
    
    case 'isready':
      self.postMessage({ command: 'readyok' });
      break;

    case 'usinewgame':
      position = new Position();
      break;

    case 'position':
      // Example: position: 'sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1 moves 7g7f'
      const parts = args.position.split(' moves ');
      const sfen = parts[0].replace('sfen ', '');
      position = Position.newBySFEN(sfen) ?? new Position();
      
      if (parts.length > 1 && parts[1]) {
        const moves = parts[1].split(' ');
        for (const moveStr of moves) {
          const move = position.createMoveByUSI(moveStr);
          if (move) {
            position.doMove(move);
          }
        }
      }
      break;

    case 'go':
      const legalMoves = position.generateMoves();
      if (legalMoves.length > 0) {
        // For deterministic testing, you could return a fixed move.
        // For this example, we just pick the first legal move.
        const bestMove = legalMoves[0].toUSI();
        self.postMessage({ command: 'bestmove', move: bestMove });
      } else {
        self.postMessage({ command: 'bestmove', move: 'resign' });
      }
      break;

    case 'quit':
      self.close();
      break;
  }
};
