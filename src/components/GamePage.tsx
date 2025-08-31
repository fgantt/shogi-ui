
import React, { useState, useEffect } from 'react';
import { useShogiController } from '../context/ShogiControllerContext';
import { Position, Square } from 'tsshogi';
import Board from './Board';
import CapturedPieces from './CapturedPieces';
import GameControls from './GameControls';
import SettingsPanel from './SettingsPanel';
import MoveLog from './MoveLog';
import './GamePage.css';

const GamePage = () => {
  const controller = useShogiController();
  const [position, setPosition] = useState<Position | null>(null);
  const [selectedSquare, setSelectedSquare] = useState<Square | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  useEffect(() => {
    const onStateChanged = (newPosition: Position) => {
      setPosition(newPosition);
    };

    controller.on('stateChanged', onStateChanged);
    setPosition(controller.getPosition());

    return () => {
      controller.off('stateChanged', onStateChanged);
    };
  }, [controller]);

  const handleSquareClick = (row: number, col: number) => {
    if (!position) return;

    const clickedSquare = Square.fromRowCol(row, col);
    if (selectedSquare) {
      const move = `${selectedSquare.toUSI()}${clickedSquare.toUSI()}`;
      controller.handleUserMove(move);
      setSelectedSquare(null);
    } else {
      const piece = position.board[row][col];
      if (piece && piece.color === position.turn) {
        setSelectedSquare(clickedSquare);
      }
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
        <GameControls onNewGame={() => controller.newGame()} />
        <CapturedPieces captured={position.hand['black']} player={'player1'} />
        <CapturedPieces captured={position.hand['white']} player={'player2'} />
        <MoveLog moves={controller.getRecord().moves.map(m => m.toUSI())} />
      </div>
      <SettingsPanel isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />
    </div>
  );
};

export default GamePage;
