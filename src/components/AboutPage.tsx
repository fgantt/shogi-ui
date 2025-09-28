import React from 'react';
import { useNavigate } from 'react-router-dom';
import './AboutPage.css';

const AboutPage: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="about-page">
      <div className="about-header">
        <button className="back-button" onClick={() => navigate('/')}>
          ← Back to Home
        </button>
        <h1>About Shogi Vibe</h1>
      </div>

      <div className="about-content">
        <div className="about-section">
          <h2>About the Application</h2>
          <p>
            Shogi Vibe is a modern, interactive implementation of the traditional Japanese chess game, 
            designed to provide an engaging and educational experience for players of all skill levels.
          </p>
          <p>
            This application combines the strategic depth of Shogi with modern web technologies, 
            offering features like AI opponents, customizable board themes, and interactive learning tools.
          </p>
          <p>
            Whether you're a complete beginner or an experienced Shogi player, Shogi Vibe provides 
            the tools and environment to enjoy and improve your game.
          </p>
        </div>

        <div className="about-section">
          <h2>Features</h2>
          <div className="features-grid">
            <div className="feature-item">
              <span className="feature-icon japanese-gameplay">🎌</span>
              <h3>Interactive Gameplay</h3>
              <p>Full Shogi game implementation with drag-and-drop piece movement</p>
            </div>
            <div className="feature-item">
              <span className="feature-icon japanese-ai">🤖</span>
              <h3>AI Opponents</h3>
              <p>Multiple difficulty levels with intelligent computer players</p>
            </div>
            <div className="feature-item">
              <span className="feature-icon japanese-themes">🌸</span>
              <h3>Customizable Themes</h3>
              <p>Beautiful board backgrounds and wallpapers to personalize your experience</p>
            </div>
            <div className="feature-item">
              <span className="feature-icon japanese-learning">📜</span>
              <h3>Learning Resources</h3>
              <p>Comprehensive help section and practice exercises</p>
            </div>
          </div>
        </div>

        <div className="about-section">
          <h2>Attributions</h2>
          <div className="attributions">
            <div className="attribution-item">
              <h3>Game Rules & Documentation</h3>
              <p>Based on traditional Shogi rules and comprehensive documentation</p>
            </div>
            <div className="attribution-item">
              <h3>Visual Assets</h3>
              <p>Board backgrounds and wallpapers sourced from various free-to-use image collections</p>
            </div>
            <div className="attribution-item">
              <h3>Technology Stack</h3>
              <p>Built with React, Vite, and modern web technologies</p>
            </div>
          </div>
        </div>

        <div className="about-section">
          <h2>Contact & Support</h2>
          <p>
            This is an open-source project created for educational and entertainment purposes. 
            For questions, suggestions, or contributions, please refer to the project repository.
          </p>
        </div>
      </div>
    </div>
  );
};

export default AboutPage;
