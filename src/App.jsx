import { useState } from 'react';
import { getInitialGameState, movePiece, dropPiece, getLegalMoves, completeMove, PLAYER_1, PLAYER_2 } from './game/engine';
import Board from './components/Board';
import CapturedPieces from './components/CapturedPieces';
import PromotionModal from './components/PromotionModal';
import GameControls from './components/GameControls';
import MoveLog from './components/MoveLog';
import './App.css';
import './styles/shogi.css';

function App() {
  const [gameState, setGameState] = useState(getInitialGameState());
  const [selectedPiece, setSelectedPiece] = useState(null); // { row, col, piece }
  const [selectedCapturedPiece, setSelectedCapturedPiece] = useState(null); // { type }
  const [legalMoves, setLegalMoves] = useState([]); // Array of [row, col]
  const [aiDifficulty, setAiDifficulty] = useState('easy'); // easy, medium, hard
  const [lastMove, setLastMove] = useState(null); // { from: [r,c], to: [r,c] }

  const handleSquareClick = (row, col) => {
    const pieceAtClick = gameState.board[row][col];

    if (selectedCapturedPiece) {
      // A captured piece is selected, attempt to drop it
      const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
      setGameState(newGameState);
      setSelectedCapturedPiece(null); // Deselect after drop attempt
      setLegalMoves([]); // Clear legal moves
      setLastMove({ from: 'drop', to: [row, col] });
    } else if (selectedPiece) {
      // A piece on the board is already selected, attempt to move it
      const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
      if (result.promotionPending) {
        setGameState(result); // Update state to show modal
      } else {
        setGameState(result);
        setLastMove({ from: [selectedPiece.row, selectedPiece.col], to: [row, col] });
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
      setLastMove({ from: 'drop', to: [row, col] });
    } else if (selectedPiece) {
      const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
      if (result.promotionPending) {
        setGameState(result);
      } else {
        setGameState(result);
        setLastMove({ from: [selectedPiece.row, selectedPiece.col], to: [row, col] });
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
    setLastMove({ from, to });
  };

  const handleNewGame = () => {
    setGameState(getInitialGameState());
    setSelectedPiece(null);
    setSelectedCapturedPiece(null);
    setLegalMoves([]);
    setLastMove(null);
  };

  const handleUndoMove = () => {
    if (gameState.pastStates.length > 0) {
      const previousState = gameState.pastStates[gameState.pastStates.length - 1];
      setGameState(previousState);
      setSelectedPiece(null);
      setSelectedCapturedPiece(null);
      setLegalMoves([]);
      setLastMove(previousState.moveHistory.length > 0 ? previousState.moveHistory[previousState.moveHistory.length - 1] : null);
    }
  };

  const handleDifficultyChange = (difficulty) => {
    setAiDifficulty(difficulty);
    // Potentially reset game or AI here if needed
  };

  return (
    <div className="app">
      <h1>Shogi Game</h1>
      <GameControls onNewGame={handleNewGame} onUndoMove={handleUndoMove} onDifficultyChange={handleDifficultyChange} />
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
          isCheck={gameState.isCheck}
          kingPosition={gameState.kingPositions[gameState.currentPlayer]}
          lastMove={lastMove}
        />
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_1]}
          player={PLAYER_1}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
        />
      </div>

      <MoveLog moves={gameState.moveHistory} />

      {gameState.promotionPending && (
        <PromotionModal
          onPromote={handlePromotionChoice}
        />
      )}
    </div>
  );
}

export default App;