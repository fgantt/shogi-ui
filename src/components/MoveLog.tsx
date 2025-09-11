
import React from 'react';
import { formatMoveForDisplay } from '../utils/moveNotation';

interface MoveLogProps {
  moves: any[];
  notation: 'western' | 'kifu';
}

const MoveLog: React.FC<MoveLogProps> = ({ moves, notation }) => {
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
  
  for (let i = 0; i < actualMoves.length; i += 2) {
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
