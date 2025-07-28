import { useState, useEffect } from 'react';
import { getInitialGameState, movePiece, dropPiece, getLegalMoves, getLegalDrops, completeMove, PLAYER_1, PLAYER_2 } from './game/engine';
import { getAiMove } from './ai/computerPlayer';
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
  const [legalDropSquares, setLegalDropSquares] = useState([]); // Array of [row, col] for drops
  const [aiDifficulty, setAiDifficulty] = useState('easy'); // easy, medium, hard
  const [lastMove, setLastMove] = useState(null); // { from: [r,c], to: [r,c] }
  const [pieceLabelType, setPieceLabelType] = useState('kanji'); // 'kanji' or 'english'

  const [wallpaperList, setWallpaperList] = useState([]);

  useEffect(() => {
    const importWallpapers = async () => {
      const modules = import.meta.glob('/public/wallpapers/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      setWallpaperList(paths);
    };
    importWallpapers();
  }, []);

  const setRandomWallpaper = () => {
    if (wallpaperList.length > 0) {
      const randomIndex = Math.floor(Math.random() * wallpaperList.length);
      document.body.style.backgroundImage = `url('${wallpaperList[randomIndex]}')`;
    }
  };

  useEffect(() => {
    setRandomWallpaper();
  }, [wallpaperList]); // Set wallpaper when wallpaperList changes

  console.log("App.jsx - pieceLabelType:", pieceLabelType);

  const handlePlayerMove = (newGameState, from, to) => {
    setGameState(newGameState);
    setSelectedPiece(null);
    setSelectedCapturedPiece(null);
    setLegalMoves([]);
    setLegalDropSquares([]); // Clear legal drop squares after a move
    setLastMove({ from: from, to: to });

    // AI makes a move after player
    setTimeout(() => {
      const aiMove = getAiMove(newGameState, aiDifficulty);
      if (aiMove) {
        let finalAiGameState;
        if (aiMove.from === 'drop') {
          finalAiGameState = dropPiece(newGameState, aiMove.type, aiMove.to);
        } else {
          finalAiGameState = movePiece(newGameState, aiMove.from, aiMove.to);
        }
        setGameState(finalAiGameState);
        setLastMove({ from: aiMove.from, to: aiMove.to });
      }
    }, 500); // Delay AI move for better UX
  };

  const handleSquareClick = (row, col) => {
    const pieceAtClick = gameState.board[row][col];

    if (selectedCapturedPiece) {
      // A captured piece is selected, attempt to drop it
      if (legalDropSquares.some(square => square[0] === row && square[1] === col)) {
        const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
        handlePlayerMove(newGameState, 'drop', [row, col]);
      }
    } else if (selectedPiece) {
      // A piece on the board is already selected, attempt to move it
      if (legalMoves.some(move => {
        console.log("handleSquareClick - Checking legal move: ", move, "against clicked: ", [row, col]);
        return move[0] === row && move[1] === col;
      })) {
        const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
        if (result.promotionPending) {
          setGameState(result); // Update state to show modal
        } else {
          handlePlayerMove(result, [selectedPiece.row, selectedPiece.col], [row, col]);
        }
      }
    } else if (pieceAtClick && pieceAtClick.player === gameState.currentPlayer) {
      // No piece selected, select the clicked piece if it belongs to the current player
      setSelectedPiece({ row, col, piece: pieceAtClick });
      const moves = getLegalMoves(pieceAtClick, row, col, gameState.board);
      setLegalMoves(moves);
      console.log("Legal moves for selected piece:", moves);
    }
  };

  const handleDragStart = (row, col) => {
    const pieceAtDragStart = gameState.board[row][col];
    if (pieceAtDragStart && pieceAtDragStart.player === gameState.currentPlayer) {
      setSelectedPiece({ row, col, piece: pieceAtDragStart });
      setSelectedCapturedPiece(null); // Clear any selected captured piece
      setLegalMoves(getLegalMoves(pieceAtDragStart, row, col, gameState.board));
      console.log("Legal moves for dragged piece:", getLegalMoves(pieceAtDragStart, row, col, gameState.board));
    }
  };

  const handleDrop = (row, col) => {
    if (selectedCapturedPiece) {
      const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
      handlePlayerMove(newGameState, 'drop', [row, col]);
    } else if (selectedPiece) {
      if (legalMoves.some(move => {
        console.log("handleDrop - Checking legal move: ", move, "against dropped: ", [row, col]);
        return move[0] === row && move[1] === col;
      })) {
        const result = movePiece(gameState, [selectedPiece.row, selectedPiece.col], [row, col]);
        if (result.promotionPending) {
          setGameState(result);
        } else {
          handlePlayerMove(result, [selectedPiece.row, selectedPiece.col], [row, col]);
        }
      }
    }
  };

  const handleCapturedPieceClick = (pieceType) => {
    setSelectedCapturedPiece({ type: pieceType });
    setSelectedPiece(null); // Clear any selected board piece
    setLegalMoves([]); // Clear legal moves
    setLegalDropSquares(getLegalDrops(gameState, pieceType)); // Set legal drop squares
  };

  const handleCapturedPieceDragStart = (pieceType) => {
    setSelectedCapturedPiece({ type: pieceType });
    setSelectedPiece(null); // Clear any selected board piece
    setLegalMoves([]); // Clear legal moves
    setLegalDropSquares(getLegalDrops(gameState, pieceType)); // Set legal drop squares
  };

  const handlePromotionChoice = (promote) => {
    const { from, to } = gameState.promotionPending;
    const newGameState = completeMove(gameState, from, to, promote);
    handlePlayerMove(newGameState, from, to);
  };

  const handleNewGame = () => {
    setGameState(getInitialGameState());
    setSelectedPiece(null);
    setSelectedCapturedPiece(null);
    setLegalMoves([]);
    setLastMove(null);
    setRandomWallpaper();
  };

  const handleUndoMove = () => {
    if (gameState.pastStates.length > 0) {
      const previousState = gameState.pastStates[gameState.pastStates.length - 1];
      setGameState(previousState);
      setSelectedPiece(null);
      setSelectedCapturedPiece(null);
      setLegalMoves([]);
      setLastMove(previousState.moveHistory.length > 0 ? { from: previousState.moveHistory[previousState.moveHistory.length - 1].from, to: previousState.moveHistory[previousState.moveHistory.length - 1].to } : null);
    }
  };

  const handleDifficultyChange = (difficulty) => {
    setAiDifficulty(difficulty);
    // Potentially reset game or AI here if needed
  };

  return (
    <div className="app">
      <h1>Shogi Vibe</h1>
      <GameControls onNewGame={handleNewGame} onUndoMove={handleUndoMove} onDifficultyChange={handleDifficultyChange} onPieceLabelTypeChange={setPieceLabelType} pieceLabelType={pieceLabelType} />
      <div className="game-container">
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_2]}
          player={PLAYER_2}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
          pieceLabelType={pieceLabelType}
        />
        <Board
          board={gameState.board}
          onSquareClick={handleSquareClick}
          onDragStart={handleDragStart}
          onDrop={handleDrop}
          legalMoves={legalMoves}
          legalDropSquares={legalDropSquares}
          isCheck={gameState.isCheck}
          kingPosition={gameState.kingPositions[gameState.currentPlayer]}
          lastMove={lastMove}
          pieceLabelType={pieceLabelType}
        />
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_1]}
          player={PLAYER_1}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
          pieceLabelType={pieceLabelType}
        />
      </div>

      <MoveLog moves={gameState.moveHistory} pieceLabelType={pieceLabelType} />

      {gameState.promotionPending && (
        <PromotionModal
          onPromote={handlePromotionChoice}
        />
      )}
    </div>
  );
}

export default App;