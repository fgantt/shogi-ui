import React, { useRef, useEffect } from 'react';
import type { Move, PieceType, GameState } from '../types';
import { KING, ROOK, BISHOP, GOLD, SILVER, KNIGHT, LANCE, PAWN, PROMOTED_ROOK, PROMOTED_BISHOP, PROMOTED_SILVER, PROMOTED_KNIGHT, PROMOTED_LANCE, PROMOTED_PAWN } from '../game/engine';
import { getMoveString } from '../game/kifu';

const getPieceInitial = (pieceType: PieceType): string => {
  switch (pieceType) {
    case KING: return 'K';
    case ROOK: return 'R';
    case BISHOP: return 'B';
    case GOLD: return 'G';
    case SILVER: return 'S';
    case KNIGHT: return 'N';
    case LANCE: return 'L';
    case PAWN: return 'P';
    case PROMOTED_ROOK: return '+R';
    case PROMOTED_BISHOP: return '+B';
    case PROMOTED_SILVER: return '+S';
    case PROMOTED_KNIGHT: return '+N';
    case PROMOTED_LANCE: return '+L';
    case PROMOTED_PAWN: return '+P';
    default: return '';
  }
};

const getRankLetter = (row: number): string => {
  return String.fromCharCode('a'.charCodeAt(0) + row);
};

const formatMoveWestern = (move: Move, allMoves: Move[]): string => {
  let pieceInitial = getPieceInitial(move.piece);
  if (move.promote && pieceInitial.startsWith('+')) {
    pieceInitial = pieceInitial.substring(1);
  }
  const toFile = 9 - move.to[1];
  const toRank = getRankLetter(move.to[0]);

  let notation = '';

  if (move.from === 'drop') {
    notation = `${pieceInitial}*${toFile}${toRank}`;
  } else if (Array.isArray(move.from)) {
    const fromFile = 9 - move.from[1];
    const fromRank = getRankLetter(move.from[0]);
    const moveType = move.captured && !move.captured.includes('check') ? 'x' : '-';

    const ambiguousMoves = allMoves.filter(m =>
      m.piece === move.piece &&
      Array.isArray(m.to) && m.to[0] === move.to[0] && m.to[1] === move.to[1] &&
      m !== move
    );

    const fromNotation = ambiguousMoves.length > 0 ? `${fromFile}${fromRank}` : '';

    notation = `${pieceInitial}${fromNotation}${moveType}${toFile}${toRank}`;

    if (move.promote) {
      notation += '+';
    }
  }

  return notation;
};

interface MoveLogProps {
  moves: Move[];
  gameState: GameState;
  notation: 'western' | 'kifu';
}

const MoveLog: React.FC<MoveLogProps> = ({ moves, gameState, notation }) => {
  const tableBodyRef = useRef<HTMLTableSectionElement>(null);

  useEffect(() => {
    if (tableBodyRef.current) {
      tableBodyRef.current.scrollTop = tableBodyRef.current.scrollHeight;
    }
  }, [moves]);

  const formatMove = (move: Move, allMoves: Move[], lastMove: Move | null): string => {
    if (notation === 'kifu') {
        const playerChar = move.player === 'player1' ? '▲' : '△';
        return playerChar + getMoveString(move, gameState, lastMove);
    } else {
      return formatMoveWestern(move, allMoves);
    }
  };

  return (
    <div className="move-log">
      <h3>Move History</h3>
      <table className="move-table">
        <thead>
          <tr>
            <th></th>
            <th><span style={{ color: "black" }}>☗</span></th>
            <th><span style={{ color: "white" }}>☗</span></th>
          </tr>
        </thead>
        <tbody ref={tableBodyRef}>
          {Array.from({ length: Math.ceil(moves.length / 2) }).map((_, i) => {
            const player1Move = moves[i * 2];
            const player2Move = moves[i * 2 + 1];
            const lastMove = i > 0 ? moves[i * 2 -1] : null
            return (
              <tr key={i}>
                <td>{i + 1}</td>
                <td>{player1Move ? formatMove(player1Move, moves, lastMove) : ''}</td>
                <td>{player2Move ? formatMove(player2Move, moves, player1Move) : ''}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default MoveLog;