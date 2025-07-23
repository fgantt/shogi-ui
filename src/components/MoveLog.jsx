import React from 'react';
import '../styles/shogi.css';

const MoveLog = ({ moves }) => {
  return (
    <div className="move-log">
      <h3>Move History</h3>
      <ul className="move-list">
        {moves.map((move, index) => (
          <li key={index}>
            {move.from === 'drop'
              ? `Drop ${move.piece} to [${move.to[0]}, ${move.to[1]}]`
              : `Move ${move.piece} from [${move.from[0]}, ${move.from[1]}] to [${move.to[0]}, ${move.to[1]}]`}
          </li>
        ))}
      </ul>
    </div>
  );
};

export default MoveLog;
