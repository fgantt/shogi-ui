import React from 'react';
import PieceComponent from './Piece';
import '../styles/shogi.css';
import type { Piece, Move } from '../types';

interface BoardProps {
  board: (Piece | null)[][];
  onSquareClick: (row: number, col: number) => void;
  onDragStart: (row: number, col: number) => void;
  onDrop: (row: number, col: number) => void;
  legalMoves: [number, number][];
  legalDropSquares: [number, number][];
  isCheck: boolean;
  kingPosition: [number, number] | null;
  lastMove: Move | null;
  pieceLabelType: string;
  notation: 'western' | 'kifu'; // Add this line
  selectedPiece: { row: number; col: number } | null;
  attackedSquares: { player1: Set<string>; player2: Set<string> };
  showAttackedPieces: boolean;
  showPieceTooltips: boolean;
  isThinking: boolean;
  checkingPieces: [number, number][];
  isGameOver: boolean;
}

const Board: React.FC<BoardProps> = ({ board, onSquareClick, onDragStart, onDrop, legalMoves, legalDropSquares, isCheck, kingPosition, lastMove, pieceLabelType, notation, selectedPiece, attackedSquares, showAttackedPieces, showPieceTooltips, isThinking, checkingPieces, isGameOver }) => {
  const isLegalMove = (row: number, col: number): boolean => {
    return legalMoves.some(move => move[0] === row && move[1] === col);
  };

  const isLegalDropSquare = (row: number, col: number): boolean => {
    return legalDropSquares.some(square => square[0] === row && square[1] === col);
  };

  const isKingSquare = (row: number, col: number): boolean => {
    return kingPosition ? kingPosition[0] === row && kingPosition[1] === col : false;
  };

  const isLastMoveSquare = (row: number, col: number): boolean => {
    if (!lastMove) return false;
    const { from, to } = lastMove;

    const isToSquare = Array.isArray(to) && to.length === 2 && to[0] === row && to[1] === col;
    const isFromSquare = Array.isArray(from) && from.length === 2 && from[0] === row && from[1] === col;

    return isFromSquare || isToSquare;
  };

  const renderCheckLine = () => {
    if (!isCheck || !checkingPieces || checkingPieces.length === 0 || !kingPosition) return null;

    return (
      <svg className="check-line-svg" width="630" height="684">
        {checkingPieces.map(([checkingRow, checkingCol], index) => {
          const [kingRow, kingCol] = kingPosition;

          const startX = checkingCol * 70 + 35;
          const startY = checkingRow * 76 + 38;
          const endX = kingCol * 70 + 35;
          const endY = kingRow * 76 + 38;

          return (
            <line
              key={index}
              x1={startX} y1={startY} x2={endX} y2={endY} stroke="red" strokeWidth="4"
            />
          );
        })}
      </svg>
    );
  };

  const columnNumbers = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const rowLetters = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
  const kifuRowLabels = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

  return (
    <div className={`shogi-board-container ${isThinking ? 'ai-thinking-overlay' : ''} ${isGameOver ? 'game-over' : ''}`}>
      <div className="column-numbers">
        {columnNumbers.map((num) => (
          <div key={num} className="column-number-cell">
            {num}
          </div>
        ))}
      </div>
      <div className="board-and-row-numbers">
        <div className="board">
          {renderCheckLine()}
          {board.map((row, rowIndex) => (
            <div key={rowIndex} className="board-row">
              {row.map((piece, colIndex) => (
                <div
                  key={colIndex}
                  className={`board-square ${isLegalMove(rowIndex, colIndex) ? 'legal-move' : ''} ${isLegalDropSquare(rowIndex, colIndex) ? 'legal-move' : ''} ${isKingSquare(rowIndex, colIndex) && isCheck ? 'in-check' : ''} ${isLastMoveSquare(rowIndex, colIndex) ? 'last-move' : ''}`}
                  onClick={() => onSquareClick(rowIndex, colIndex)}
                  onDragOver={(e) => e.preventDefault()} // Allow drop
                  onDrop={() => onDrop(rowIndex, colIndex)}
                >
                  {(rowIndex === 2 && colIndex === 2) && <div className="intersection-dot"></div>}
                  {(rowIndex === 2 && colIndex === 5) && <div className="intersection-dot"></div>}
                  {(rowIndex === 5 && colIndex === 2) && <div className="intersection-dot"></div>}
                  {(rowIndex === 5 && colIndex === 5) && <div className="intersection-dot"></div>}
                  {piece && (
                    <PieceComponent
                      type={piece.type}
                      player={piece.player}
                      onDragStart={() => onDragStart(rowIndex, colIndex)}
                      pieceLabelType={pieceLabelType}
                      isSelected={selectedPiece ? selectedPiece.row === rowIndex && selectedPiece.col === colIndex : false}
                      isAttacked={showAttackedPieces && attackedSquares[piece.player === 'player1' ? 'player2' : 'player1'].has(`${rowIndex},${colIndex}`)}
                      showTooltips={showPieceTooltips}
                    />
                  )}
                </div>
              ))}
            </div>
          ))}
        </div>
        <div className="row-numbers">
          {(notation === 'kifu' ? kifuRowLabels : rowLetters).map((label) => (
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
