import { useState } from 'react';
import { getInitialGameState } from './game/engine';
import Board from './components/Board';
import './App.css';
import './styles/shogi.css';

function App() {
  const [gameState, setGameState] = useState(getInitialGameState());

  return (
    <div className="app">
      <h1>Shogi Game</h1>
      <Board board={gameState.board} />
    </div>
  );
}

export default App;

