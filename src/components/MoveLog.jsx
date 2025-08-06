import React, { useState } from 'react';
import { KANJI_MAP, ENGLISH_MAP } from '../utils/pieceMaps';
import { KING, ROOK, BISHOP, GOLD, SILVER, KNIGHT, LANCE, PAWN, PROMOTED_ROOK, PROMOTED_BISHOP, PROMOTED_SILVER, PROMOTED_KNIGHT, PROMOTED_LANCE, PROMOTED_PAWN, PLAYER_1, PLAYER_2 } from '../game/engine';

const padToTwoChars = (str) => str.length === 1 ? ` ${str}` : str;

const getPieceInitial = (pieceType) => {
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

const formatMove = (move, allMoves) => {
  let pieceInitial = getPieceInitial(move.piece);
  if (move.promote && pieceInitial.startsWith('+')) {
    pieceInitial = pieceInitial.substring(1);
  }
  const toFile = 9 - move.to[1]; // Convert 0-indexed col to 1-9 file (right to left)
  const toRank = move.to[0] + 1; // Convert 0-indexed row to 1-9 rank (top to bottom)

  let notation = '';

  if (move.from === 'drop') {
    notation = `${pieceInitial}*${toFile}${toRank}`;
  } else {
    const fromFile = 9 - move.from[1];
    const fromRank = move.from[0] + 1;
    const moveType = move.captured && !move.captured.includes('check') ? 'x' : '-';

    // Check for ambiguity
    const ambiguousMoves = allMoves.filter(m =>
      m.piece === move.piece &&
      m.to[0] === move.to[0] &&
      m.to[1] === move.to[1] &&
      m !== move // Exclude the current move itself
    );

    const fromNotation = ambiguousMoves.length > 0 ? `${fromFile}${fromRank}` : '';

    notation = `${pieceInitial}${fromNotation}${moveType}${toFile}${toRank}`;

    if (move.promote) {
      notation += '+';
    } else if (move.promotionDeclined) {
      notation += '=';
    }
  }

  return notation;
};

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
            <th>☗</th>
            <th><span style={{ color: "white" }}>☗</span></th>
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: Math.ceil(moves.length / 2) }).map((_, i) => {
            const player1Move = moves[i * 2];
            const player2Move = moves[i * 2 + 1];
            return (
              <tr key={i}>
                <td>{i + 1}</td>
                <td>{player1Move ? formatMove(player1Move, moves) : ''}</td>
                <td>{player2Move ? formatMove(player2Move, moves) : ''}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default MoveLog;
