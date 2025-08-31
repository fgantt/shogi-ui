
import React from 'react';

interface MoveLogProps {
  moves: string[];
}

const MoveLog: React.FC<MoveLogProps> = ({ moves }) => {
  return (
    <div className="move-log">
      <h3>Move History</h3>
      <ol>
        {moves.map((move, index) => (
          <li key={index}>{move}</li>
        ))}
      </ol>
    </div>
  );
};

export default MoveLog;
