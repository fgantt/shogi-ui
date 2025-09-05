import React, { useState, useEffect } from 'react';
import { useShogiController } from '../context/ShogiControllerContext';
import { Position, Square, PieceType as TsshogiPieceType, Color } from 'tsshogi';
import Board from './Board';
import CapturedPieces from './CapturedPieces';
import GameControls from './GameControls';
import SettingsPanel from './SettingsPanel';
import MoveLog from './MoveLog';
import PromotionModal from './PromotionModal';
import CheckmateModal from './CheckmateModal';
import SaveGameModal from './SaveGameModal';
import LoadGameModal from './LoadGameModal';
import './GamePage.css';

const GamePage = () => {
  const controller = useShogiController();
  const [position, setPosition] = useState<Position | null>(null);
  const [selectedSquare, setSelectedSquare] = useState<Square | null>(null);
  const [selectedCapturedPiece, setSelectedCapturedPiece] = useState<TsshogiPieceType | null>(null);
  const [promotionMove, setPromotionMove] = useState<{ from: Square; to: Square } | null>(null);
  const [winner, setWinner] = useState<'player1' | 'player2' | 'draw' | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isSaveModalOpen, setIsSaveModalOpen] = useState(false);
  const [isLoadModalOpen, setIsLoadModalOpen] = useState(false);
  const [savedGames, setSavedGames] = useState<{[key: string]: string}>({});

  useEffect(() => {
    const games = JSON.parse(localStorage.getItem('shogi-saved-games') || '{}');
    setSavedGames(games);
  }, []);

  // Settings state
  const [pieceLabelType, setPieceLabelType] = useState(localStorage.getItem('shogi-piece-label-type') || 'kanji');
  const [notation, setNotation] = useState(localStorage.getItem('shogi-notation') || 'western');
  const [showAttackedPieces, setShowAttackedPieces] = useState(localStorage.getItem('shogi-show-attacked-pieces') === 'true');
  const [showPieceTooltips, setShowPieceTooltips] = useState(localStorage.getItem('shogi-show-piece-tooltips') === 'true');
  const [wallpaper, setWallpaper] = useState(localStorage.getItem('shogi-wallpaper') || '/wallpapers/photo1.jpg');
  const [boardBackground, setBoardBackground] = useState(localStorage.getItem('shogi-board-background') || '/boards/wood-kaya.jpg');
  const [wallpaperList, setWallpaperList] = useState<string[]>([]);
  const [boardBackgroundList, setBoardBackgroundList] = useState<string[]>([]);

  useEffect(() => {
    const loadAssets = async () => {
      const wallpaperModules = import.meta.glob('/public/wallpapers/*.{jpg,svg,jpeg,png,webp}');
      const boardModules = import.meta.glob('/public/boards/*.{jpg,svg,jpeg,png,webp}');
      
      const wallpaperPaths = Object.keys(wallpaperModules).map(path => path.replace('/public', ''));
      const boardPaths = Object.keys(boardModules).map(path => path.replace('/public', ''));

      setWallpaperList(wallpaperPaths);
      setBoardBackgroundList(boardPaths);
    };

    loadAssets();
  }, []);

  useEffect(() => {
    const onStateChanged = (newPosition: Position) => {
      setPosition(newPosition);
      if (newPosition.isCheckmate()) {
        setWinner(newPosition._color === 'black' ? 'player2' : 'player1');
      } else if (newPosition.isRepetition()) {
        setWinner('draw');
      }
    };

    controller.on('stateChanged', onStateChanged);
    setPosition(controller.getPosition());

    return () => {
      controller.off('stateChanged', onStateChanged);
    };
  }, [controller]);

  const handleSquareClick = (row: number, col: number) => {
    if (!position) return;
    const clickedSquare = Square.newByXY(8 - col, row);
    if (!clickedSquare) return;

    // Deselect if clicking the same square
    if (selectedSquare?.equals(clickedSquare)) {
      setSelectedSquare(null);
      return;
    }

    // If a piece is selected, try to move
    if (selectedSquare) {
      const moveUsi = `${selectedSquare.usi}${clickedSquare.usi}`;
      // This won't handle promotions correctly yet, but it will move the piece.
      controller.handleUserMove(moveUsi);
      setSelectedSquare(null);
    } else {
      // No piece selected, so select one
      const piece = position.board.at(clickedSquare);
      if (piece && piece.color === position._color) {
        setSelectedSquare(clickedSquare);
      }
    }
  };

  const handlePromotion = (promote: boolean) => {
    if (!promotionMove) return;

    const { from, to } = promotionMove;
    const move = `${from.usi}${to.usi}${promote ? '+' : ''}`;
    controller.handleUserMove(move);
    setPromotionMove(null);
  };

  const handleNewGame = () => {
    controller.newGame();
    setWinner(null);
  };

  const handleDismiss = () => {
    setWinner(null);
  };

  const handleSettingChange = (setter: (value: any) => void, key: string) => (value: any) => {
    setter(value);
    localStorage.setItem(key, value.toString());
  };

  const handleSaveGame = (name: string) => {
    const sfen = controller.getPosition().toSFEN();
    const newSavedGames = { ...savedGames, [name]: sfen };
    setSavedGames(newSavedGames);
    localStorage.setItem('shogi-saved-games', JSON.stringify(newSavedGames));
    setIsSaveModalOpen(false);
  };

  const handleLoadGame = (name: string) => {
    const sfen = savedGames[name];
    if (sfen) {
      controller.loadSfen(sfen);
    }
    setIsLoadModalOpen(false);
  };

  const handleDeleteGame = (name: string) => {
    const newSavedGames = { ...savedGames };
    delete newSavedGames[name];
    setSavedGames(newSavedGames);
    localStorage.setItem('shogi-saved-games', JSON.stringify(newSavedGames));
  };

  const handleCapturedPieceClick = (pieceType: TsshogiPieceType, player: 'player1' | 'player2') => {
    const isPlayer1Turn = position?._color === 'black';
    const isPlayer2Turn = position?._color === 'white';

    if ((isPlayer1Turn && player === 'player1') || (isPlayer2Turn && player === 'player2')) {
      setSelectedCapturedPiece(pieceType);
      setSelectedSquare(null);
    }
  };

  if (!position) {
    return <div>Loading...</div>;
  }

  return (
    <div className={`game-page`}>
      <div className="main-area">
        <Board position={position} onSquareClick={handleSquareClick} selectedSquare={selectedSquare} />
      </div>
      <div className="side-panel">
        <GameControls 
          onNewGame={handleNewGame} 
          onOpenSettings={() => setIsSettingsOpen(true)} 
          onOpenSaveModal={() => setIsSaveModalOpen(true)}
          onOpenLoadModal={() => setIsLoadModalOpen(true)}
        />
        <CapturedPieces captured={position.hand(Color.BLACK)} player={'player1'} onPieceClick={(pieceType) => handleCapturedPieceClick(pieceType, 'player1')} selectedCapturedPiece={selectedCapturedPiece} />
        <CapturedPieces captured={position.hand(Color.WHITE)} player={'player2'} onPieceClick={(pieceType) => handleCapturedPieceClick(pieceType, 'player2')} selectedCapturedPiece={selectedCapturedPiece} />
        <MoveLog moves={controller.getRecord().moves.map(m => {
          if ('usi' in m.move) {
            return m.move.usi;
          } else {
            return m.displayText;
          }
        })} />
      </div>
      {isSettingsOpen && <SettingsPanel 
        pieceLabelType={pieceLabelType}
        onPieceLabelTypeChange={handleSettingChange(setPieceLabelType, 'shogi-piece-label-type')}
        notation={notation}
        onNotationChange={handleSettingChange(setNotation, 'shogi-notation')}
        wallpaperList={wallpaperList}
        onSelectWallpaper={handleSettingChange(setWallpaper, 'shogi-wallpaper')}
        boardBackgroundList={boardBackgroundList}
        onSelectBoardBackground={handleSettingChange(setBoardBackground, 'shogi-board-background')}
        onClose={() => setIsSettingsOpen(false)}
        currentWallpaper={wallpaper}
        currentBoardBackground={boardBackground}
        showAttackedPieces={showAttackedPieces}
        onShowAttackedPiecesChange={handleSettingChange(setShowAttackedPieces, 'shogi-show-attacked-pieces')}
        showPieceTooltips={showPieceTooltips}
        onShowPieceTooltipsChange={handleSettingChange(setShowPieceTooltips, 'shogi-show-piece-tooltips')}
      />}
      {promotionMove && <PromotionModal onPromote={handlePromotion} />}
      {winner && <CheckmateModal winner={winner} onNewGame={handleNewGame} onDismiss={handleDismiss} />}
      <SaveGameModal isOpen={isSaveModalOpen} onClose={() => setIsSaveModalOpen(false)} onSave={handleSaveGame} />
      <LoadGameModal isOpen={isLoadModalOpen} onClose={() => setIsLoadModalOpen(false)} onLoad={handleLoadGame} onDelete={handleDeleteGame} savedGames={savedGames} />
    </div>
  );
};

export default GamePage;