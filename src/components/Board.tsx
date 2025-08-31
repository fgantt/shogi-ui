import React from 'react';
import { Position, Piece, Color, PieceType, Square } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

interface BoardProps {
  position: Position;
  onSquareClick: (row: number, col: number) => void;
  selectedSquare: Square | null;
}

// Helper to map tsshogi piece kind to our PieceType
function toOurPieceType(kind: PieceType): string {
    switch (kind) {
        case 'PAWN': return 'pawn';
        case 'LANCE': return 'lance';
        case 'KNIGHT': return 'knight';
        case 'SILVER': return 'silver';
        case 'GOLD': return 'gold';
        case 'BISHOP': return 'bishop';
        case 'ROOK': return 'rook';
        case 'KING': return 'king';
        case 'PROM_PAWN': return 'promPawn';
        case 'PROM_LANCE': return 'promLance';
        case 'PROM_KNIGHT': return 'promKnight';
        case 'PROM_SILVER': return 'promSilver';
        case 'HORSE': return 'horse';
        case 'DRAGON': return 'dragon';
        default: return '';
    }
}

// Helper to map tsshogi color to our Player type
function toOurPlayer(color: Color): 'player1' | 'player2' {
    return color === 'black' ? 'player1' : 'player2';
}

const Board: React.FC<BoardProps> = ({ position, onSquareClick, selectedSquare }) => {
  const columnNumbers = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const rowLetters = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
  const kifuRowLabels = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

  const isSelected = (row: number, col: number): boolean => {
    if (!selectedSquare) return false;
    return selectedSquare.row === row && selectedSquare.col === col;
  };

  return (
    <div className={`shogi-board-container`}>
      <div className="column-numbers">
        {columnNumbers.map((num) => (
          <div key={num} className="column-number-cell">
            {num}
          </div>
        ))}
      </div>
      <div className="board-and-row-numbers">
        <div className="board">
          {position.board.map((row, rowIndex) => (
            <div key={rowIndex} className="board-row">
              {row.map((piece, colIndex) => (
                <div
                  key={colIndex}
                  className={`board-square ${isSelected(rowIndex, colIndex) ? 'selected' : ''}`}
                  onClick={() => onSquareClick(rowIndex, colIndex)}>
                  {piece && (
                    <PieceComponent
                      type={toOurPieceType(piece.kind)}
                      player={toOurPlayer(piece.color)}
                      pieceLabelType={'kanji'} // Hardcoded for now
                      isSelected={isSelected(rowIndex, colIndex)}
                    />
                  )}
                </div>
              ))}
            </div>
          ))}
        </div>
        <div className="row-numbers">
          {kifuRowLabels.map((label) => (
            <div key={label} className="row-number-cell">
              {label}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Board;