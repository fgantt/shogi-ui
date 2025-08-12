import React from 'react';
import { useNavigate } from 'react-router-dom';
import './HomePage.css';

const HomePage = () => {
  const navigate = useNavigate();

  const handleStartGame = () => {
    navigate('/game');
  };

  const handleOpenSettings = () => {
    navigate('/game', { state: { openSettings: true } });
  };

  return (
    <div className="home-page">
      <div className="home-content">
        <h1 className="home-title">Shogi Vibe</h1>
        <p className="home-subtitle">Experience the ancient art of Japanese chess</p>
        
        <div className="navigation-grid">
          <button 
            className="nav-button primary"
            onClick={handleStartGame}
          >
            <span className="button-icon">♔</span>
            <span className="button-text">Start New Game</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/help')}
          >
            <span className="button-icon">📖</span>
            <span className="button-text">Help</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/practice')}
          >
            <span className="button-icon">🎯</span>
            <span className="button-text">Practice</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={handleOpenSettings}
          >
            <span className="button-icon">⚙️</span>
            <span className="button-text">Settings</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/about')}
          >
            <span className="button-icon">ℹ️</span>
            <span className="button-text">About</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/wasm-demo')}
          >
            <span className="button-icon">🚀</span>
            <span className="button-text">WebAssembly Demo</span>
          </button>
        </div>
        
        <div className="home-footer">
          <p>Master the art of Shogi through practice and play</p>
        </div>
      </div>
    </div>
  );
};

export default HomePage;
