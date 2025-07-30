import React, { useState } from 'react';
import { KANJI_MAP, ENGLISH_MAP } from '../utils/pieceMaps';

const padToTwoChars = (str) => str.length === 1 ? ` ${str}` : str;

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
            <th>Action</th>
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
                    ? padToTwoChars(KANJI_MAP[move.piece])
                    : padToTwoChars(ENGLISH_MAP[move.piece])}
                </span>
                {`  `}
                {move.from === 'drop'
                  ? `Drop to [${move.to[0] + 1}, ${move.to[1] + 1}]`
                  : `[${move.from[0] + 1}, ${move.from[1] + 1}] => [${move.to[0] + 1}, ${move.to[1] + 1}]`}
                {move.captured && `  `}
                {move.captured &&
                  (move.captured.includes(' / check')
                    ? `${pieceLabelType === 'kanji'
                        ? padToTwoChars(KANJI_MAP[move.captured.split(' / ')[0]])
                        : padToTwoChars(ENGLISH_MAP[move.captured.split(' / ')[0]])} / check`
                      : (move.captured === 'check'
                        ? 'check'
                        : (pieceLabelType === 'kanji'
                          ? padToTwoChars(KANJI_MAP[move.captured])
                          : padToTwoChars(ENGLISH_MAP[move.captured]))))}
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
