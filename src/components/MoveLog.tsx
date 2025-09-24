
import React from 'react';
import { formatMoveForDisplay } from '../utils/moveNotation';

interface MoveLogProps {
  moves: any[];
  notation: 'western' | 'kifu' | 'usi' | 'csa';
  startColor?: 'black' | 'white';
}

const MoveLog: React.FC<MoveLogProps> = ({ moves, notation, startColor = 'black' }) => {
  // Filter out the initial "開始局面" (Starting phase) special move
  const actualMoves = moves.filter(move => {
    if ('move' in move && typeof move.move === 'object' && 'type' in move.move) {
      // This is a special move, check if it's the START type
      return move.move.type !== 'start';
    }
    return true; // Keep regular moves
  });

  // Group moves into pairs (black, white) for table display
  const movePairs: { black: string; white: string }[] = [];
  
  let startIndex = 0;
  if (startColor === 'white') {
    movePairs.push({
      black: '',
      white: actualMoves[0] ? formatMoveForDisplay(actualMoves[0], notation, false) : ''
    });
    startIndex = 1;
  }
  
  for (let i = startIndex; i < actualMoves.length; i += 2) {
    const blackMove = actualMoves[i] ? formatMoveForDisplay(actualMoves[i], notation, true) : '';
    const whiteMove = actualMoves[i + 1] ? formatMoveForDisplay(actualMoves[i + 1], notation, false) : '';
    
    movePairs.push({
      black: blackMove,
      white: whiteMove
    });
  }

  return (
    <div className="move-log">
      <h3>Move History</h3>
      <div className="move-table-container">
        <table className="move-table">
          <thead>
            <tr>
              <th></th>
              <th style={{ color: 'black' }}>☗</th>
              <th style={{ color: 'white' }}>☗</th>
            </tr>
          </thead>
          <tbody>
            {movePairs.map((pair, index) => (
              <tr key={index}>
                <td>{index + 1}</td>
                <td>{pair.black}</td>
                <td>{pair.white}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default MoveLog;
