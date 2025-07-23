import { useState } from 'react';
import { getInitialGameState, movePiece, dropPiece, getLegalMoves, completeMove, PLAYER_1, PLAYER_2 } from './game/engine';
import Board from './components/Board';
import CapturedPieces from './components/CapturedPieces';
import PromotionModal from './components/PromotionModal';
import './App.css';
import './styles/shogi.css';

function App() {
  const [gameState, setGameState] = useState(getInitialGameState());
  const [selectedPiece, setSelectedPiece] = useState(null); // { row, col, piece }
  const [selectedCapturedPiece, setSelectedCapturedPiece] = useState(null); // { type }
  const [legalMoves, setLegalMoves] = useState([]); // Array of [row, col]

  const handleSquareClick = (row, col) => {
    const pieceAtClick = gameState.board[row][col];

    if (selectedCapturedPiece) {
      // A captured piece is selected, attempt to drop it
      const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
      setGameState(newGameState);
      setSelectedCapturedPiece(null); // Deselect after drop attempt
      setLegalMoves([]); // Clear legal moves
    } else if (selectedPiece) {
      // A piece on the board is already selected, attempt to move it
      const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
      if (result.promotionPending) {
        setGameState(result); // Update state to show modal
      } else {
        setGameState(result);
      }
      setSelectedPiece(null); // Deselect after move attempt
      setLegalMoves([]); // Clear legal moves
    } else if (pieceAtClick && pieceAtClick.player === gameState.currentPlayer) {
      // No piece selected, select the clicked piece if it belongs to the current player
      setSelectedPiece({ row, col, piece: pieceAtClick });
      setLegalMoves(getLegalMoves(pieceAtClick, row, col, gameState.board));
    }
  };

  const handleDragStart = (row, col) => {
    const pieceAtDragStart = gameState.board[row][col];
    if (pieceAtDragStart && pieceAtDragStart.player === gameState.currentPlayer) {
      setSelectedPiece({ row, col, piece: pieceAtDragStart });
      setSelectedCapturedPiece(null); // Clear any selected captured piece
      setLegalMoves(getLegalMoves(pieceAtDragStart, row, col, gameState.board));
    }
  };

  const handleDrop = (row, col) => {
    if (selectedCapturedPiece) {
      const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
      setGameState(newGameState);
      setSelectedCapturedPiece(null);
      setLegalMoves([]);
    } else if (selectedPiece) {
      const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
      if (result.promotionPending) {
        setGameState(result);
      } else {
        setGameState(result);
      }
      setSelectedPiece(null);
      setLegalMoves([]);
    }
  };

  const handleCapturedPieceClick = (pieceType) => {
    setSelectedCapturedPiece({ type: pieceType });
    setSelectedPiece(null); // Clear any selected board piece
    setLegalMoves([]); // Clear legal moves
  };

  const handleCapturedPieceDragStart = (pieceType) => {
    setSelectedCapturedPiece({ type: pieceType });
    setSelectedPiece(null); // Clear any selected board piece
    setLegalMoves([]); // Clear legal moves
  };

  const handlePromotionChoice = (promote) => {
    const { from, to } = gameState.promotionPending;
    const newGameState = completeMove(gameState, from, to, promote);
    setGameState(newGameState);
  };

  return (
    <div className="app">
      <h1>Shogi Game</h1>
      <div className="game-container">
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_2]}
          player={PLAYER_2}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
        />
        <Board
          board={gameState.board}
          onSquareClick={handleSquareClick}
          onDragStart={handleDragStart}
          onDrop={handleDrop}
          legalMoves={legalMoves}
        />
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_1]}
          player={PLAYER_1}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
        />
      </div>

      {gameState.promotionPending && (
        <PromotionModal
          onPromote={handlePromotionChoice}
        />
      )}
    </div>
  );
}

export default App;

