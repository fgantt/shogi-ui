import { useState, useEffect } from 'react';
import { getInitialGameState, movePiece, dropPiece, getLegalMoves, getLegalDrops, completeMove, isKingInCheck, isCheckmate, PLAYER_1, PLAYER_2, getAttackedSquares, generateStateHash } from './game/engine';
import { getAiMove } from './ai/computerPlayer';
import Board from './components/Board';
import CapturedPieces from './components/CapturedPieces';
import PromotionModal from './components/PromotionModal';
import GameControls from './components/GameControls';
import SettingsPanel from './components/SettingsPanel';
import MoveLog from './components/MoveLog';
import CheckmateModal from './components/CheckmateModal';
import StartGameModal from './components/StartGameModal';
import './App.css';
import './styles/shogi.css';
import './styles/settings.css';

function App() {
  const [gameState, setGameState] = useState(getInitialGameState());
  const [humanPlayer, setHumanPlayer] = useState(PLAYER_1);
  const [aiPlayer, setAiPlayer] = useState(PLAYER_2);
  const [selectedPiece, setSelectedPiece] = useState(null); // { row, col, piece }
  const [selectedCapturedPiece, setSelectedCapturedPiece] = useState(null); // { type }
  const [legalMoves, setLegalMoves] = useState([]); // Array of [row, col]
  const [legalDropSquares, setLegalDropSquares] = useState([]); // Array of [row, col] for drops
  const [aiDifficulty, setAiDifficulty] = useState('medium'); // easy, medium, hard
  const [lastMove, setLastMove] = useState(null); // { from: [r,c], to: [r,c] }
  const [pieceLabelType, setPieceLabelType] = useState('kanji'); // 'kanji' or 'english'
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isStartModalOpen, setIsStartModalOpen] = useState(true);
  const [attackedSquares, setAttackedSquares] = useState({ player1: new Set(), player2: new Set() });
  const [showAttackedPieces, setShowAttackedPieces] = useState(true);
  const [showPieceTooltips, setShowPieceTooltips] = useState(false);
  const [checkmateWinner, setCheckmateWinner] = useState(null);
  const [isThinking, setIsThinking] = useState(false);
  const [repetitionCount, setRepetitionCount] = useState({});

  const [wallpaperList, setWallpaperList] = useState([]);
  const [boardBackgroundList, setBoardBackgroundList] = useState([]);
  const [currentWallpaper, setCurrentWallpaper] = useState('');
  const [currentBoardBackground, setCurrentBoardBackground] = useState('');

  useEffect(() => {
    const importWallpapers = async () => {
      const modules = import.meta.glob('/public/wallpapers/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      setWallpaperList(paths);
      if (paths.length > 0) {
        const initialWallpaper = paths[Math.floor(Math.random() * paths.length)];
        document.body.style.backgroundImage = `url('${initialWallpaper}')`;
        setCurrentWallpaper(initialWallpaper);
      }
    };
    const importBoardBackgrounds = async () => {
      const modules = import.meta.glob('/public/boards/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      setBoardBackgroundList(paths);
      if (paths.length > 0) {
        const initialBoardBackground = paths[Math.floor(Math.random() * paths.length)];
        document.querySelector('.board').style.backgroundImage = `url('${initialBoardBackground}')`;
        setCurrentBoardBackground(initialBoardBackground);
      }
    };
    importWallpapers();
    importBoardBackgrounds();
  }, []);

  useEffect(() => {
    updateAttackedSquares();
  }, [gameState.board]);

  useEffect(() => {
    if (gameState.currentPlayer === aiPlayer && !checkmateWinner) {
      setIsThinking(true);
      getAiMove(gameState, aiDifficulty)
        .then(aiMove => {
          setGameState(currentGameState => {
            let nextState;
            if (aiMove) {
              let attemptedNextState;
              if (aiMove.from === 'drop') {
                attemptedNextState = dropPiece(currentGameState, aiMove.type, aiMove.to);
              } else if (Array.isArray(aiMove.from)) {
                attemptedNextState = movePiece(currentGameState, aiMove.from, aiMove.to, aiMove.promote);
              } else {
                console.error("AI returned an unexpected move format:", aiMove);
                attemptedNextState = currentGameState; // Fallback to current state
              }

              if (attemptedNextState !== currentGameState) {
                // Move was successful
                nextState = attemptedNextState;
                setLastMove({ from: aiMove.from, to: aiMove.to });
              } else {
                // Move was illegal, keep current state and switch player to human
                nextState = { ...currentGameState, currentPlayer: humanPlayer };
              }
            } else {
              // AI found no legal moves, switch turn back to human
              nextState = { ...currentGameState, currentPlayer: humanPlayer };
            }

            // Check for checkmate after AI's move
            if (isCheckmate(nextState)) {
              setCheckmateWinner(nextState.currentPlayer === humanPlayer ? aiPlayer : humanPlayer);
            }

            const hash = generateStateHash(nextState);
            const newRepetitionCount = { ...repetitionCount, [hash]: (repetitionCount[hash] || 0) + 1 };
            setRepetitionCount(newRepetitionCount);

            if (newRepetitionCount[hash] >= 4) {
              const isCheck = isKingInCheck(nextState.board, nextState.currentPlayer);
              if (isCheck) {
                // Perpetual check, the player who made the move loses
                setCheckmateWinner(nextState.currentPlayer === humanPlayer ? aiPlayer : humanPlayer);
              } else {
                // Repetition draw
                setCheckmateWinner('draw');
              }
            }

            return nextState;
          });
        })
        .catch(error => {
          console.error("Error getting AI move:", error);
        })
        .finally(() => {
          setIsThinking(false);
        });
    }
  }, [gameState.currentPlayer, aiDifficulty, checkmateWinner, humanPlayer, aiPlayer]);

  const setRandomWallpaper = () => {
    if (wallpaperList.length > 0) {
      const randomIndex = Math.floor(Math.random() * wallpaperList.length);
      const newWallpaper = wallpaperList[randomIndex];
      document.body.style.backgroundImage = `url('${newWallpaper}')`;
      setCurrentWallpaper(newWallpaper);
    }
  };

  const setRandomBoardBackground = () => {
    if (boardBackgroundList.length > 0) {
      const randomIndex = Math.floor(Math.random() * boardBackgroundList.length);
      const newBoardBackground = boardBackgroundList[randomIndex];
      document.querySelector('.board').style.backgroundImage = `url('${newBoardBackground}')`;
      setCurrentBoardBackground(newBoardBackground);
    }
  };

  const updateAttackedSquares = () => {
    const player1Attacks = getAttackedSquares(gameState.board, PLAYER_1);
    const player2Attacks = getAttackedSquares(gameState.board, PLAYER_2);
    setAttackedSquares({ player1: player1Attacks, player2: player2Attacks });
  };

  const handlePlayerMove = (newGameState, from, to) => {
    const hash = generateStateHash(newGameState);
    const newRepetitionCount = { ...repetitionCount, [hash]: (repetitionCount[hash] || 0) + 1 };
    setRepetitionCount(newRepetitionCount);

    if (newRepetitionCount[hash] >= 4) {
      const isCheck = isKingInCheck(newGameState.board, newGameState.currentPlayer);
      if (isCheck) {
        // Perpetual check, the player who made the move loses
        setCheckmateWinner(newGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1);
      } else {
        // Repetition draw
        setCheckmateWinner('draw');
      }
      return;
    }

    setGameState(newGameState);
    setSelectedPiece(null);
    setSelectedCapturedPiece(null);
    setLegalMoves([]);
    setLegalDropSquares([]); // Clear legal drop squares after a move
    setLastMove({ from: from, to: to });

    // Check for checkmate after player's move
    if (isCheckmate(newGameState)) {
      setCheckmateWinner(newGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1);
      return; // Stop game if checkmate
    }
  };

  const handleSquareClick = (row, col) => {
    if (isThinking || gameState.currentPlayer !== humanPlayer) return;
    const pieceAtClick = gameState.board[row][col];

    // Deselect if the already selected piece is clicked again
    if (selectedPiece && selectedPiece.row === row && selectedPiece.col === col) {
      setSelectedPiece(null);
      setLegalMoves([]);
      setLegalDropSquares([]);
      return;
    }

    if (selectedCapturedPiece) {
      // A captured piece is selected, attempt to drop it
      if (legalDropSquares.some(square => square[0] === row && square[1] === col)) {
        const newGameState = dropPiece(gameState, selectedCapturedPiece.type, [row, col]);
        if (newGameState !== gameState) { // Only proceed if the drop was successful
          handlePlayerMove(newGameState, 'drop', [row, col]);
        }
      }
    } else if (selectedPiece) {
      // A piece on the board is already selected, attempt to move it
      if (legalMoves.some(move => {
        
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
      const filteredMoves = moves.filter(to => {
        const simulatedGameState = movePiece(gameState, [row, col], to);
        return !isKingInCheck(simulatedGameState.board, gameState.currentPlayer); // Only keep moves that don't result in current player's king being in check
      });
      setLegalMoves(filteredMoves);
      setLegalDropSquares([]); // Clear legal drop squares when a board piece is selected
      
    }
  };

  const handleDragStart = (row, col) => {
    if (isThinking || gameState.currentPlayer !== humanPlayer) return;
    const pieceAtDragStart = gameState.board[row][col];
    if (pieceAtDragStart && pieceAtDragStart.player === gameState.currentPlayer) {
      setSelectedPiece({ row, col, piece: pieceAtDragStart });
      setSelectedCapturedPiece(null); // Clear any selected captured piece
      const moves = getLegalMoves(pieceAtDragStart, row, col, gameState.board);
      const filteredMoves = moves.filter(to => {
        const simulatedGameState = movePiece(gameState, [row, col], to);
        return !isKingInCheck(simulatedGameState.board, gameState.currentPlayer); // Only keep moves that don't result in current player's king being in check
      });
      setLegalMoves(filteredMoves);
      console.log("Legal moves for dragged piece:", filteredMoves);
    }
  };

  const handleDrop = (row, col) => {
    if (isThinking || gameState.currentPlayer !== humanPlayer) return;
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
    if (isThinking || gameState.currentPlayer !== humanPlayer) return;
    if (selectedCapturedPiece && selectedCapturedPiece.type === pieceType) {
      // If the same captured piece is clicked again, deselect it
      setSelectedCapturedPiece(null);
      setLegalDropSquares([]);
    } else {
      setSelectedCapturedPiece({ type: pieceType });
      setSelectedPiece(null); // Clear any selected board piece
      setLegalMoves([]); // Clear legal moves
      setLegalDropSquares(getLegalDrops(gameState, pieceType)); // Set legal drop squares
    }
  };

  const handleCapturedPieceDragStart = (pieceType) => {
    if (isThinking || gameState.currentPlayer !== humanPlayer) return;
    if (selectedCapturedPiece && selectedCapturedPiece.type === pieceType) {
      // If the same captured piece is dragged again, deselect it
      setSelectedCapturedPiece(null);
      setLegalDropSquares([]);
    } else {
      setSelectedCapturedPiece({ type: pieceType });
      setSelectedPiece(null); // Clear any selected board piece
      setLegalMoves([]); // Clear legal moves
      setLegalDropSquares(getLegalDrops(gameState, pieceType)); // Set legal drop squares
    }
  };

  const handlePromotionChoice = (promote) => {
    const { from, to } = gameState.promotionPending;
    const newGameState = completeMove(gameState, from, to, promote);
    handlePlayerMove(newGameState, from, to);
  };

  const handleStartGame = (settings) => {
    setAiDifficulty(settings.difficulty);
    setPieceLabelType(settings.pieceSet);
    setHumanPlayer(settings.player);
    setAiPlayer(settings.player === PLAYER_1 ? PLAYER_2 : PLAYER_1);
    setGameState(getInitialGameState());
    setSelectedPiece(null);
    setSelectedCapturedPiece(null);
    setLegalMoves([]);
    setLegalDropSquares([]);
    setLastMove(null);
    setCheckmateWinner(null);
    setRepetitionCount({});
    setIsStartModalOpen(false);
  };

  const handleNewGame = () => {
    setIsStartModalOpen(true);
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
  };

  const handleOpenSettings = () => {
    setIsSettingsOpen(true);
  };

  const handleCloseSettings = () => {
    setIsSettingsOpen(false);
  };

  const handleSelectWallpaper = (wallpaper) => {
    document.body.style.backgroundImage = `url('${wallpaper}')`;
    setCurrentWallpaper(wallpaper);
  };

  const handleSelectBoardBackground = (boardBackground) => {
    document.querySelector('.board').style.backgroundImage = `url('${boardBackground}')`;
    setCurrentBoardBackground(boardBackground);
  };

  return (
    <div className="app">
      <h1>Shogi Vibe</h1>
      <GameControls
        onNewGame={handleNewGame}
        onUndoMove={handleUndoMove}
        onDifficultyChange={handleDifficultyChange}
        onPieceLabelTypeChange={setPieceLabelType}
        pieceLabelType={pieceLabelType}
        onOpenSettings={handleOpenSettings}
        aiDifficulty={aiDifficulty}
        isThinking={isThinking}
      />
      <div className="game-container">
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_2]}
          player={PLAYER_2}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
          pieceLabelType={pieceLabelType}
          selectedCapturedPiece={selectedCapturedPiece}
          isThinking={isThinking}
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
          selectedPiece={selectedPiece}
          attackedSquares={attackedSquares}
          showAttackedPieces={showAttackedPieces}
          showPieceTooltips={showPieceTooltips}
          isThinking={isThinking}
        />
        <CapturedPieces
          pieces={gameState.capturedPieces[PLAYER_1]}
          player={PLAYER_1}
          onPieceClick={handleCapturedPieceClick}
          onPieceDragStart={handleCapturedPieceDragStart}
          pieceLabelType={pieceLabelType}
          selectedCapturedPiece={selectedCapturedPiece}
          isThinking={isThinking}
        />
      </div>

      <MoveLog moves={gameState.moveHistory} pieceLabelType={pieceLabelType} />

      {gameState.promotionPending && (
        <PromotionModal onPromote={handlePromotionChoice} />
      )}

      {isSettingsOpen && (
        <SettingsPanel
          aiDifficulty={aiDifficulty}
          onDifficultyChange={setAiDifficulty}
          pieceLabelType={pieceLabelType}
          onPieceLabelTypeChange={setPieceLabelType}
          wallpaperList={wallpaperList}
          onSelectWallpaper={handleSelectWallpaper}
          boardBackgroundList={boardBackgroundList}
          onSelectBoardBackground={handleSelectBoardBackground}
          onClose={handleCloseSettings}
          currentWallpaper={currentWallpaper}
          currentBoardBackground={currentBoardBackground}
          showAttackedPieces={showAttackedPieces}
          onShowAttackedPiecesChange={setShowAttackedPieces}
          showPieceTooltips={showPieceTooltips}
          onShowPieceTooltipsChange={setShowPieceTooltips}
        />
      )}

      {checkmateWinner && (
        <CheckmateModal
          winner={checkmateWinner}
          onDismiss={() => setCheckmateWinner(null)}
          onNewGame={handleNewGame}
        />
      )}

      {isStartModalOpen && (
        <StartGameModal
          isOpen={isStartModalOpen}
          onClose={() => setIsStartModalOpen(false)}
          onStartGame={handleStartGame}
        />
      )}
    </div>
  );
}

export default App;
