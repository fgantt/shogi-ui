import React, { useState } from 'react';
import { KANJI_MAP, ENGLISH_MAP } from '../utils/pieceMaps';

const MoveLog = ({ moves, pieceLabelType }) => {
  const [sortOrder, setSortOrder] = useState('desc');

  const sortedMoves = [...moves].sort((a, b) => {
    if (sortOrder === 'asc') {
      return moves.indexOf(a) - moves.indexOf(b);
    } else {
      return moves.indexOf(b) - moves.indexOf(a);
    }
  });

  const toggleSortOrder = () => {
    setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
  };

  return (
    <div className="move-log">
      <h3>Move History</h3>
      <table className="move-table">
        <thead>
          <tr>
            <th onClick={toggleSortOrder} style={{ cursor: 'pointer' }}>
              Move {sortOrder === 'asc' ? '▲' : '▼'}
            </th>
            <th>Player</th>
            <th>Piece</th>
            <th>From</th>
            <th>To</th>
            <th>Captured</th>
            <th>Time</th>
          </tr>
        </thead>
        <tbody>
          {sortedMoves.map((move, index) => (
            <tr key={index}>
              <td>{moves.indexOf(move) + 1}</td>
              <td>{move.player === 'player1' ? 'Player 1' : 'Player 2'}</td>
              <td>
                <span style={{ color: move.promote ? '#b80000' : 'inherit' }}>
                  {pieceLabelType === 'kanji'
                    ? KANJI_MAP[move.piece]
                    : ENGLISH_MAP[move.piece]}
                </span>
              </td>
              <td>
                {move.from === 'drop'
                  ? `Drop`
                  : `[${move.from[0] + 1}, ${move.from[1] + 1}]`}
              </td>
              <td>{`[${move.to[0] + 1}, ${move.to[1] + 1}]`}</td>
              <td>
                {move.captured &&
                  (pieceLabelType === 'kanji'
                    ? KANJI_MAP[move.captured]
                    : ENGLISH_MAP[move.captured])}
              </td>
              <td>{move.timestamp}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default MoveLog;
