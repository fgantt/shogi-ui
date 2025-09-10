import { useState, useEffect } from 'react';
import { useShogiController } from '../context/ShogiControllerContext';
import { ImmutablePosition, Square, PieceType as TsshogiPieceType, isPromotableRank, Color } from 'tsshogi';
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

// Helper function to check if a piece is already promoted
const isPiecePromoted = (pieceType: TsshogiPieceType): boolean => {
  return [
    TsshogiPieceType.PROM_PAWN,
    TsshogiPieceType.PROM_LANCE,
    TsshogiPieceType.PROM_KNIGHT,
    TsshogiPieceType.PROM_SILVER,
    TsshogiPieceType.HORSE, // promoted bishop
    TsshogiPieceType.DRAGON  // promoted rook
  ].includes(pieceType);
};

const GamePage = () => {
  const controller = useShogiController();
  const [position, setPosition] = useState<ImmutablePosition | null>(null);
  const [renderKey, setRenderKey] = useState(0); // Force re-render counter
  const [selectedSquare, setSelectedSquare] = useState<Square | null>(null);
  const [legalMoves, setLegalMoves] = useState<Square[]>([]);
  const [lastMove, setLastMove] = useState<{ from: Square | null; to: Square | null } | null>(null);
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
      // For now, use hardcoded lists since import.meta.glob is not available
      const wallpaperPaths = [
        '/wallpapers/beautiful-japanese-garden.jpg',
        '/wallpapers/beautiful-natural-landscape.jpg',
        '/wallpapers/fuji1.jpg',
        '/wallpapers/koi.jpg',
        '/wallpapers/maple.jpg',
        '/wallpapers/mountain-house.jpeg',
        '/wallpapers/photo1.jpg',
        '/wallpapers/shogi-background-placeholder.svg',
        '/wallpapers/wave.jpg',
        '/wallpapers/woman-with-kimono-wagasa-umbrella.jpg'
      ];
      
      const boardPaths = [
        '/boards/koi-bw.jpg',
        '/boards/marble-calacatta.jpg',
        '/boards/marble.jpg',
        '/boards/quartz-1.jpg',
        '/boards/quartz-2.jpg',
        '/boards/stars-1.jpg',
        '/boards/stars-2.jpg',
        '/boards/wood-agathis-1.jpg',
        '/boards/wood-agathis-2.jpg',
        '/boards/wood-bambo.jpg',
        '/boards/wood-boxwood-1.jpg',
        '/boards/wood-boxwood-2.jpg',
        '/boards/wood-boxwood-3.jpg',
        '/boards/wood-boxwood-4.jpg',
        '/boards/wood-cherry-1.jpg',
        '/boards/wood-cherry-2.jpg',
        '/boards/wood-cherry-3.jpg',
        '/boards/wood-cypress-1.jpg',
        '/boards/wood-ginkgo-1.jpg',
        '/boards/wood-ginkgo-2.jpg',
        '/boards/wood-ginkgo-3.jpg',
        '/boards/wood-hiba-1.jpeg',
        '/boards/wood-hickory-1.jpg',
        '/boards/wood-katsura-1.png',
        '/boards/wood-mahogany-1.jpg',
        '/boards/wood-maple-1.jpg',
        '/boards/wood-maple-2.webp',
        '/boards/wood-pecan-1.jpg',
        '/boards/wood-pecan-2.jpg',
        '/boards/wood-red-spruce-1.jpg'
      ];

      setWallpaperList(wallpaperPaths);
      setBoardBackgroundList(boardPaths);
    };

    loadAssets();
  }, []);

  useEffect(() => {
    const onStateChanged = (newPosition: ImmutablePosition) => {
      // Force a re-render by updating both position and render key
      // The position object from tsshogi is mutable, so we need to trigger React's re-render
      setPosition(newPosition);
      setRenderKey(prev => prev + 1);
      
      // Update last move for highlighting
      const lastMoveData = controller.getLastMove();
      setLastMove(lastMoveData);
      
      //TODO(feg): With the switch to tsshogi, need to determine checkmate and repetition from the newPosition object.
      // if (newPosition.isCheckmate()) {
      //   setWinner(newPosition.turn === 0 ? 'player2' : 'player1');
      // } else if (newPosition.isRepetition()) {
      //   setWinner('draw');
      // }
    };

    controller.on('stateChanged', onStateChanged);
    setPosition(controller.getPosition());

    return () => {
      controller.off('stateChanged', onStateChanged);
    };
  }, [controller]);

  const handleSquareClick = (row: number, col: number) => {
    if (!position) return;
    const clickedSquare = Square.newByXY(col, row);
    if (!clickedSquare) return;

    // Deselect if clicking the same square
    if (selectedSquare?.equals(clickedSquare)) {
      setSelectedSquare(null);
      setLegalMoves([]);
      return;
    }

    // If a piece is selected, try to move
    if (selectedSquare) {
      const piece = position.board.at(selectedSquare);
      if (!piece) {
        setSelectedSquare(null);
        setLegalMoves([]);
        return;
      }

      // Check if the move is eligible for promotion
      const currentColor = position.sfen.includes(' b ') ? Color.BLACK : Color.WHITE;
      const isFromPromotable = isPromotableRank(currentColor, selectedSquare.rank);
      const isToPromotable = isPromotableRank(currentColor, clickedSquare.rank);
      const canPromote = !isPiecePromoted(piece.type) && // Piece is not already promoted
                        piece.type !== TsshogiPieceType.KING && 
                        piece.type !== TsshogiPieceType.GOLD && 
                        (isFromPromotable || isToPromotable);

      if (canPromote) {
        // Show promotion modal instead of making the move directly
        setPromotionMove({ from: selectedSquare, to: clickedSquare });
        setSelectedSquare(null);
        setLegalMoves([]);
      } else {
        // Make the move directly
        const moveUsi = `${selectedSquare.usi}${clickedSquare.usi}`;
        controller.handleUserMove(moveUsi);
        setSelectedSquare(null);
        setLegalMoves([]);
      }
    } else {
      // No piece selected, so select one
      const piece = position.board.at(clickedSquare);
      if (piece && piece.color === (position.sfen.includes(' b ') ? 'black' : 'white')) {
        setSelectedSquare(clickedSquare);
        // Get legal moves for the selected piece
        const moves = controller.getLegalMovesForSquare(clickedSquare);
        setLegalMoves(moves);
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
    const sfen = controller.getPosition().sfen;
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
    const isPlayer1Turn = position?.sfen.includes(' b ');
    const isPlayer2Turn = position?.sfen.includes(' w ');

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
        <Board key={renderKey} position={position} onSquareClick={handleSquareClick} selectedSquare={selectedSquare} legalMoves={legalMoves} lastMove={lastMove} />
      </div>
      <div className="side-panel">
        <GameControls 
          onNewGame={handleNewGame} 
          onOpenSettings={() => setIsSettingsOpen(true)} 
          onOpenSaveModal={() => setIsSaveModalOpen(true)}
          onOpenLoadModal={() => setIsLoadModalOpen(true)}
        />
        <CapturedPieces captured={position.blackHand as any} player={'player1'} onPieceClick={(pieceType) => handleCapturedPieceClick(pieceType, 'player1')} selectedCapturedPiece={selectedCapturedPiece} />
        <CapturedPieces captured={position.whiteHand as any} player={'player2'} onPieceClick={(pieceType) => handleCapturedPieceClick(pieceType, 'player2')} selectedCapturedPiece={selectedCapturedPiece} />
        <MoveLog moves={controller.getRecord().moves.map(m => {
          if ('usi' in m.move) {
            return m.move.usi;
          } else {
            return m.displayText;
          }
        })} />
      </div>
      {isSettingsOpen && <SettingsPanel 
        pieceLabelType={pieceLabelType as any}
        onPieceLabelTypeChange={handleSettingChange(setPieceLabelType, 'shogi-piece-label-type')}
        notation={notation as any}
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
        aiDifficulty={1 as any}
        onDifficultyChange={() => {}}
      />}
      {promotionMove && <PromotionModal onPromote={handlePromotion} />}
      {winner && <CheckmateModal winner={winner} onNewGame={handleNewGame} onDismiss={handleDismiss} />}
      <SaveGameModal isOpen={isSaveModalOpen} onClose={() => setIsSaveModalOpen(false)} onSave={handleSaveGame} />
      <LoadGameModal isOpen={isLoadModalOpen} onClose={() => setIsLoadModalOpen(false)} onLoad={handleLoadGame} onDelete={handleDeleteGame} savedGames={savedGames} />
    </div>
  );
};

export default GamePage;