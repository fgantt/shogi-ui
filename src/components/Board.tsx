import React, { useRef, useImperativeHandle, forwardRef } from 'react';
import { ImmutablePosition, Square } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

interface BoardProps {
  position: ImmutablePosition;
  onSquareClick: (row: number, col: number) => void;
  onDragStart?: (row: number, col: number) => void;
  onDragEnd?: (row: number, col: number) => void;
  onDragOver?: (row: number, col: number) => void;
  selectedSquare: Square | null;
  legalMoves: Square[];
  lastMove: { from: Square | null; to: Square | null } | null;
  isSquareAttacked?: (square: Square) => boolean;
  isInCheck?: boolean;
  kingInCheckSquare?: Square | null;
  attackingPieces?: Square[];
  boardBackground?: string;
  pieceThemeType?: string;
  showPieceTooltips?: boolean;
  notation?: 'western' | 'kifu' | 'usi' | 'csa';
  promotionModalContent?: React.ReactNode;
  promotionTargetUsi?: string;
  thinkingMove?: string | null;
}

// Define the ref type that will be exposed
export interface BoardRef {
  getSquareRef: (usi: string) => HTMLDivElement | undefined;
}

// Helper to map tsshogi color to our Player type
function toOurPlayer(color: string): 'player1' | 'player2' {
    return color === 'black' ? 'player1' : 'player2';
}

const Board = forwardRef<BoardRef, BoardProps>(({ position, onSquareClick, onDragStart, onDragEnd, onDragOver, selectedSquare, legalMoves, lastMove, isSquareAttacked, isInCheck, kingInCheckSquare, attackingPieces, boardBackground, pieceThemeType, showPieceTooltips, notation = 'kifu', promotionModalContent, promotionTargetUsi, thinkingMove }, ref) => {
  const columnNumbers = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const squareRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  // Expose a method to get a specific square ref
  useImperativeHandle(ref, () => ({
    getSquareRef: (usi: string) => squareRefs.current.get(usi),
  }));
  
  // Row labels based on notation
  const getRowLabels = () => {
    switch (notation) {
      case 'western':
        return ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
      case 'csa':
        return ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
      case 'usi':
        return ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
      case 'kifu':
      default:
        return ['一', '二', '三', '四', '五', '六', '七', '八', '九'];
    }
  };
  
  const rowLabels = getRowLabels();

  const isSelected = (row: number, col: number): boolean => {
    if (!selectedSquare) return false;
    // Convert row/col to tsshogi coordinates for comparison
    const square = Square.newByXY(col, row);
    return square ? selectedSquare.equals(square) : false;
  };

  const isLegalMove = (row: number, col: number): boolean => {
    const square = Square.newByXY(col, row);
    if (!square) return false;
    return legalMoves.some(move => move.equals(square));
  };

  const isLastMove = (row: number, col: number): boolean => {
    if (!lastMove) return false;
    const square = Square.newByXY(col, row);
    if (!square) return false;
    
    return (lastMove.from ? lastMove.from.equals(square) : false) || 
           (lastMove.to ? lastMove.to.equals(square) : false);
  };

  const isInCheckSquare = (row: number, col: number): boolean => {
    if (!isInCheck || !kingInCheckSquare) return false;
    const square = Square.newByXY(col, row);
    if (!square) return false;
    return square.equals(kingInCheckSquare);
  };

  // Helper function to convert square coordinates to pixel coordinates
  const squareToPixel = (square: Square) => {
    // Use the same coordinate system as the board rendering
    // The board uses Square.newByXY(colIndex, rowIndex) where:
    // colIndex 0-8 goes left to right (0 = leftmost = file 9, 8 = rightmost = file 1)
    // rowIndex 0-8 goes top to bottom (0 = top = rank 1, 8 = bottom = rank 9)
    
    // Convert from USI string format (e.g., "2e", "1f") to board coordinates
    const usiString = square.usi;
    const file = parseInt(usiString[0], 10); // 1-9
    const rankChar = usiString[1]; // a-i
    const rank = rankChar.charCodeAt(0) - 'a'.charCodeAt(0) + 1; // 1-9
    
    // Convert to board colIndex and rowIndex
    const colIndex = 9 - file; // file 1 -> col 8, file 9 -> col 0
    const rowIndex = rank - 1; // rank 1 -> row 0, rank 9 -> row 8
    
    const x = colIndex * 70 + 35; // 70px per square, center at 35px
    const y = rowIndex * 76 + 38; // 76px per square, center at 38px
    
    return { x, y };
  };

  // Parse thinking move USI string to get from and to squares
  // Uses the same approach as the check indicator - leverage tsshogi's createMoveByUSI
  const parseThinkingMove = (moveUsi: string | null | undefined): { from: Square | null; to: Square | null } => {
    if (!moveUsi || moveUsi.length < 4) {
      return { from: null, to: null };
    }

    // Handle drop moves (e.g., "P*2e") - don't show arrow for drops
    if (moveUsi.includes('*')) {
      return { from: null, to: null };
    }

    try {
      // Use tsshogi's createMoveByUSI to parse the move - same approach as the rest of the codebase
      const move = position.createMoveByUSI(moveUsi);
      
      if (move && 'from' in move && 'to' in move) {
        // Regular move has from and to squares
        return { from: move.from, to: move.to };
      }
      
      return { from: null, to: null };
    } catch (e) {
      console.error('[Board] Error parsing thinking move:', moveUsi, e);
      return { from: null, to: null };
    }
  };

  const thinkingMoveSquares = parseThinkingMove(thinkingMove);

  // Debug logging for thinking move
  if (thinkingMove) {
    console.log('[Board] Thinking move received:', thinkingMove);
    console.log('[Board] Parsed squares:', thinkingMoveSquares);
  }

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
        <div 
          className="board"
          style={boardBackground ? {
            backgroundImage: `url('${boardBackground}')`,
            backgroundSize: 'cover',
            backgroundRepeat: 'no-repeat',
            backgroundPosition: 'center center'
          } : undefined}
        >
          {Array.from({ length: 9 }, (_, rowIndex) => (
            <div key={rowIndex} className="board-row">
              {Array.from({ length: 9 }, (_, colIndex) => {
                // Convert row/col to tsshogi Square
                const square = Square.newByXY(colIndex, rowIndex); // tsshogi uses 0-8 coordinates
                const piece = square ? position.board.at(square) : null;
                
                const classNames = [
                  'board-square',
                  isSelected(rowIndex, colIndex) ? 'selected' : '',
                  isLegalMove(rowIndex, colIndex) ? 'legal-move' : '',
                  isLastMove(rowIndex, colIndex) ? 'last-move' : '',
                  isInCheckSquare(rowIndex, colIndex) ? 'in-check' : '',
                ].filter(Boolean).join(' ');

                return (
                  <div
                    key={colIndex}
                    ref={el => {
                      const square = Square.newByXY(colIndex, rowIndex);
                      const usi = square?.usi;
                      if (usi) {
                        if (el) {
                          squareRefs.current.set(usi, el);
                        } else {
                          squareRefs.current.delete(usi);
                        }
                      }
                    }}
                    data-testid={`square-${rowIndex}-${colIndex}`}
                    className={classNames}
                    onClick={() => onSquareClick(rowIndex, colIndex)}
                    onDragOver={(e) => {
                      e.preventDefault();
                      onDragOver?.(rowIndex, colIndex);
                    }}
                    onDrop={(e) => {
                      e.preventDefault();
                      onDragEnd?.(rowIndex, colIndex);
                    }}>
                    {piece && (
                      <PieceComponent
                        type={piece.type}
                        player={toOurPlayer(piece.color)}
                        pieceThemeType={pieceThemeType || 'kanji'}
                        isSelected={isSelected(rowIndex, colIndex)}
                        isAttacked={isSquareAttacked ? isSquareAttacked(square) : false}
                        showTooltips={showPieceTooltips || false}
                        onClick={() => {
                          onSquareClick(rowIndex, colIndex)
                        }}
                        onDragStart={() => {
                          onDragStart?.(rowIndex, colIndex);
                        }}
                      />
                    )}
                    {/* Render promotion modal content if this is the target square */}
                    {promotionTargetUsi && square?.usi === promotionTargetUsi && promotionModalContent}
                    {/* Promotion zone intersection dots */}
                    {((rowIndex === 2 && (colIndex === 2 || colIndex === 5)) || 
                      (rowIndex === 5 && (colIndex === 2 || colIndex === 5))) && (
                      <div className="intersection-dot"></div>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
          {/* Red lines for check indicators */}
          {isInCheck && kingInCheckSquare && attackingPieces && attackingPieces.length > 0 && (
            <svg className="check-line-svg" width="630" height="684">
              {attackingPieces.map((attackerSquare, index) => {
                const attackerPos = squareToPixel(attackerSquare);
                const kingPos = squareToPixel(kingInCheckSquare);
                
                return (
                  <g key={index}>
                    <line
                      x1={attackerPos.x}
                      y1={attackerPos.y}
                      x2={kingPos.x}
                      y2={kingPos.y}
                      stroke="red"
                      strokeWidth="3"
                      strokeOpacity="0.8"
                    />
                  </g>
                );
              })}
            </svg>
          )}
          
          {/* Thinking arrow for engine moves */}
          {thinkingMoveSquares.from && thinkingMoveSquares.to && (
            <svg className="thinking-arrow-svg" width="630" height="684">
              <defs>
                <marker
                  id="arrowhead-thinking"
                  markerWidth="6"
                  markerHeight="6"
                  refX="5"
                  refY="2"
                  orient="auto"
                  markerUnits="strokeWidth"
                >
                  {/* Open arrowhead - just two lines forming a V shape */}
                  <polyline
                    points="0 0, 5 2, 0 4"
                    fill="none"
                    stroke="rgba(0, 150, 255, 0.8)"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </marker>
              </defs>
              {(() => {
                const fromPos = squareToPixel(thinkingMoveSquares.from);
                const toPos = squareToPixel(thinkingMoveSquares.to);
                
                // Calculate arrow direction to shorten it slightly before the arrowhead
                const dx = toPos.x - fromPos.x;
                const dy = toPos.y - fromPos.y;
                const length = Math.sqrt(dx * dx + dy * dy);
                const shortenBy = 10; // Shorten by 10 pixels to make room for smaller arrowhead
                const ratio = (length - shortenBy) / length;
                const endX = fromPos.x + dx * ratio;
                const endY = fromPos.y + dy * ratio;
                
                return (
                  <line
                    x1={fromPos.x}
                    y1={fromPos.y}
                    x2={endX}
                    y2={endY}
                    stroke="rgba(0, 150, 255, 0.8)"
                    strokeWidth="2.5"
                    strokeLinecap="round"
                    markerEnd="url(#arrowhead-thinking)"
                  />
                );
              })()}
            </svg>
          )}
          
        </div>
        <div className="row-numbers">
          {rowLabels.map((label) => (
            <div key={label} className="row-number-cell">
              {label}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
});

export default Board;
