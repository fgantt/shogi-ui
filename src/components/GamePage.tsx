import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
// import { getInitialGameState, movePiece, dropPiece, getLegalMoves, getLegalDrops, completeMove, isKingInCheck, isCheckmate, PLAYER_1, PLAYER_2, getAttackedSquares, generateStateHash, getAllCheckingPieces, checkSennichite } from '../game/engine';
// import { generateKifu, parseKifu } from '../game/kifu';
// import { getAiMove, initializeWasm } from '../ai/computerPlayer';
// import { GameState } from '../types';
import Board from './Board';
import CapturedPieces from './CapturedPieces';
import PromotionModal from './PromotionModal';
import GameControls from './GameControls';
import SettingsPanel from './SettingsPanel';
import MoveLog from './MoveLog';
import CheckmateModal from './CheckmateModal';
import StartGameModal from './StartGameModal';
import SaveGameModal from './SaveGameModal';
import LoadGameModal from './LoadGameModal';
import './GamePage.css';

const GamePage = () => {
  const navigate = useNavigate();
  const location = useLocation();
  // const [gameState, setGameState] = useState(getInitialGameState());
  const [player1Type, setPlayer1Type] = useState<'human' | 'ai-js' | 'ai-wasm'>('human');
  const [player2Type, setPlayer2Type] = useState<'human' | 'ai-js' | 'ai-wasm'>('ai-wasm'); // Default to Jaguar for Player 2
  const [selectedPiece, setSelectedPiece] = useState(null); // { row, col, piece }
  const [selectedCapturedPiece, setSelectedCapturedPiece] = useState(null); // { type }
  const [legalMoves, setLegalMoves] = useState([]); // Array of [row, col]
  const [legalDropSquares, setLegalDropSquares] = useState([]); // Array of [row, col] for drops
  const [aiDifficulty, setAiDifficulty] = useState('medium'); // easy, medium, hard
  const [lastMove, setLastMove] = useState(null); // { from: [r,c], to: [r,c] }
  const [pieceLabelType, setPieceLabelType] = useState('kanji'); // 'kanji' or 'english'
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isStartModalOpen, setIsStartModalOpen] = useState(true);
  const [isSaveModalOpen, setIsSaveModalOpen] = useState(false);
  const [isLoadModalOpen, setIsLoadModalOpen] = useState(false);
  const [attackedSquares, setAttackedSquares] = useState({ player1: new Set(), player2: new Set() });
  const [showAttackedPieces, setShowAttackedPieces] = useState(true);
  const [showPieceTooltips, setShowPieceTooltips] = useState(false);
  const [checkmateWinner, setCheckmateWinner] = useState(null);
  const [isThinking, setIsThinking] = useState(false);
  const [isGameOver, setIsGameOver] = useState(false);
  const [isLoadingGame, setIsLoadingGame] = useState(false);
  const [notation, setNotation] = useState<'western' | 'kifu'>('western');

  const playerNames = {
    'human': 'Human',
    'ai-js': 'Jaguar',
    'ai-wasm': 'Raptor'
  };
  
  const [checkingPieces, setCheckingPieces] = useState<[number, number][]>([]);

  const [wallpaperList, setWallpaperList] = useState([]);
  const [boardBackgroundList, setBoardBackgroundList] = useState([]);
  const [currentWallpaper, setCurrentWallpaper] = useState('');
  const [currentBoardBackground, setCurrentBoardBackground] = useState('');

  return (
    <div className={`game-page ${isGameOver ? 'game-over' : ''}`}>
        <h1>Shogi Game</h1>
        <p>The game is currently being refactored to use a new architecture. Please check back later.</p>
    </div>
  );
};

export default GamePage;