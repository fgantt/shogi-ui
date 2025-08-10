import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import SvgPiece from './SvgPiece';
import { KING, GOLD, SILVER, KNIGHT, LANCE, ROOK, BISHOP, PAWN } from '../game/engine';
import './HelpPage.css';

const HelpPage = () => {
  const navigate = useNavigate();
  const [activeSection, setActiveSection] = useState('about');

  const pieceData = [
    {
      name: 'King (王将/玉将)',
      type: KING,
      description: 'Moves one square in any direction (horizontally, vertically, or diagonally).',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [1, -1],  [1, 0],  [1, 1]
      ],
      promoted: false
    },
    {
      name: 'Gold General (金将)',
      type: GOLD,
      description: 'Moves one square orthogonally or one square diagonally forward. Cannot move diagonally backward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [0, 0],  [1, 0]
      ],
      promoted: false
    },
    {
      name: 'Silver General (銀将)',
      type: SILVER,
      description: 'Moves one square forward or one square diagonally in any direction. Cannot move sideways or directly backward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, 0],  [1, -1],  [1, 1]
      ],
      promoted: false
    },
    {
      name: 'Knight (桂馬)',
      type: KNIGHT,
      description: 'Moves two squares forward and then one square sideways (L-shape, but only forward). Can jump over other pieces.',
      movement: [
        [-2, -1], [-2, 1]
      ],
      promoted: false
    },
    {
      name: 'Lance (香車)',
      type: LANCE,
      description: 'Moves any number of squares directly forward. Cannot move backward or sideways.',
      movement: [
        [-1, 0], [-2, 0], [-3, 0], [-4, 0]
      ],
      promoted: false
    },
    {
      name: 'Rook (飛車)',
      type: ROOK,
      description: 'Moves any number of squares horizontally or vertically.',
      movement: [
        [-1, 0], [-2, 0], [-3, 0], [-4, 0],
        [0, -1], [0, -2], [0, -3], [0, -4],
        [0, 1],  [0, 2],  [0, 3],  [0, 4],
        [1, 0],  [2, 0],  [3, 0],  [4, 0]
      ],
      promoted: false
    },
    {
      name: 'Bishop (角行)',
      type: BISHOP,
      description: 'Moves any number of squares diagonally.',
      movement: [
        [-1, -1], [-2, -2], [-3, -3], [-4, -4],
        [-1, 1],  [-2, 2],  [-3, 3],  [-4, 4],
        [1, -1],  [2, -2],  [3, -3],  [4, -4],
        [1, 1],   [2, 2],   [3, 3],   [4, 4]
      ],
      promoted: false
    },
    {
      name: 'Pawn (歩兵)',
      type: PAWN,
      description: 'Moves one square directly forward. Captures by moving forward onto an opponent\'s piece.',
      movement: [
        [-1, 0]
      ],
      promoted: false
    }
  ];

  const promotedPieceData = [
    {
      name: 'Promoted Silver (成銀)',
      type: SILVER,
      description: 'Promoted Silver moves like a Gold General - one square orthogonally or one square diagonally forward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [0, 0],  [1, 0]
      ],
      promoted: true
    },
    {
      name: 'Promoted Knight (成桂)',
      type: KNIGHT,
      description: 'Promoted Knight moves like a Gold General - one square orthogonally or one square diagonally forward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [0, 0],  [1, 0]
      ],
      promoted: true
    },
    {
      name: 'Promoted Lance (成香)',
      type: LANCE,
      description: 'Promoted Lance moves like a Gold General - one square orthogonally or one square diagonally forward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [0, 0],  [1, 0]
      ],
      promoted: true
    },
    {
      name: 'Dragon King (龍王)',
      type: ROOK,
      description: 'Dragon King combines Rook and King movements - moves any number of squares horizontally/vertically, plus one square diagonally.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [1, -1],  [1, 0],  [1, 1],
        [-2, 0], [-3, 0], [-4, 0],
        [0, -2], [0, -3], [0, -4],
        [0, 2],  [0, 3],  [0, 4],
        [2, 0],  [3, 0],  [4, 0]
      ],
      promoted: true
    },
    {
      name: 'Dragon Horse (龍馬)',
      type: BISHOP,
      description: 'Dragon Horse combines Bishop and King movements - moves any number of squares diagonally, plus one square orthogonally.',
      movement: [
        [-1, -1], [-2, -2], [-3, -3], [-4, -4],
        [-1, 1],  [-2, 2],  [-3, 3],  [-4, 4],
        [1, -1],  [2, -2],  [3, -3],  [4, -4],
        [1, 1],   [2, 2],   [3, 3],   [4, 4],
        [-1, 0], [0, -1], [0, 1], [1, 0]
      ],
      promoted: true
    },
    {
      name: 'Promoted Pawn (と金)',
      type: PAWN,
      description: 'Promoted Pawn moves like a Gold General - one square orthogonally or one square diagonally forward.',
      movement: [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1],           [0, 1],
        [0, 0],  [1, 0]
      ],
      promoted: true
    }
  ];

  const renderMovementDiagram = (piece) => {
    const gridSize = 5;
    const center = Math.floor(gridSize / 2);
    
    return (
      <div className="movement-diagram">
        <div className="diagram-grid" style={{ gridTemplateColumns: `repeat(${gridSize}, 1fr)` }}>
          {Array.from({ length: gridSize * gridSize }, (_, index) => {
            const row = Math.floor(index / gridSize);
            const col = index % gridSize;
            const relativeRow = row - center;
            const relativeCol = col - center;
            
            const isPiece = row === center && col === center;
            const isLegalMove = piece.movement.some(([dr, dc]) => 
              dr === relativeRow && dc === relativeCol
            );
            
            let className = 'diagram-square';
            if (isPiece) className += ' piece-square';
            else if (isLegalMove) className += ' legal-move';
            
            return (
              <div key={index} className={className}>
                {isPiece && (
                  <SvgPiece 
                    piece={{ type: piece.type, player: 1, promoted: piece.promoted }}
                    size={30}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  return (
    <div className="help-page">
      <div className="help-header">
        <button className="back-button" onClick={() => navigate('/')}>
          ← Back to Home
        </button>
        <h1>Shogi Help & Rules</h1>
      </div>

      <div className="help-navigation">
        <button 
          className={`nav-tab ${activeSection === 'about' ? 'active' : ''}`}
          onClick={() => setActiveSection('about')}
        >
          About Shogi
        </button>
        <button 
          className={`nav-tab ${activeSection === 'rules' ? 'active' : ''}`}
          onClick={() => setActiveSection('rules')}
        >
          Rules
        </button>
        <button 
          className={`nav-tab ${activeSection === 'pieces' ? 'active' : ''}`}
          onClick={() => setActiveSection('pieces')}
        >
          Piece Movements
        </button>
      </div>

      <div className="help-content">
        {activeSection === 'about' && (
          <div className="content-section">
            <h2>About Shogi</h2>
            <p>
              Shogi, also known as Japanese Chess, is a strategic board game that shares similarities 
              with Western chess but has unique and fascinating differences. It's played on a 9x9 board 
              and features the distinctive rule of dropping captured pieces back onto the board.
            </p>
            <p>
              Unlike Western chess, all pieces in Shogi are the same color and are differentiated by 
              their shape and the kanji (Japanese characters) written on them. Players tell their pieces 
              from their opponent's by the direction they are pointing.
            </p>
            <p>
              The most unique aspect of Shogi is the "dropping" rule - when you capture an opponent's 
              piece, it becomes part of your "pieces in hand" and can be dropped onto any empty square 
              on the board on a subsequent turn.
            </p>
          </div>
        )}

        {activeSection === 'rules' && (
          <div className="content-section">
            <h2>Basic Rules</h2>
            <div className="rules-grid">
              <div className="rule-card">
                <h3>Objective</h3>
                <p>Checkmate the opponent's King by placing it in a position where it is under attack and has no legal move to escape.</p>
              </div>
              <div className="rule-card">
                <h3>Setup</h3>
                <p>Each player starts with 20 pieces arranged in three rows. The King is in the center of the back row.</p>
              </div>
              <div className="rule-card">
                <h3>Promotion</h3>
                <p>Most pieces can be promoted when they enter, exit, or pass through the last three ranks on the opponent's side.</p>
              </div>
              <div className="rule-card">
                <h3>Dropping</h3>
                <p>Captured pieces become part of your hand and can be dropped onto any empty square on the board.</p>
              </div>
            </div>
          </div>
        )}

        {activeSection === 'pieces' && (
          <div className="content-section">
            <h2>Piece Movements</h2>
            <p>Click on a piece to see its movement pattern:</p>
            
            <h3>Regular Pieces</h3>
            <div className="pieces-grid">
              {pieceData.map((piece, index) => (
                <div key={index} className="piece-card">
                  <div className="piece-header">
                    <SvgPiece 
                      piece={{ type: piece.type, player: 1, promoted: false }}
                      size={40}
                    />
                    <h3>{piece.name}</h3>
                  </div>
                  <p className="piece-description">{piece.description}</p>
                  {renderMovementDiagram(piece)}
                </div>
              ))}
            </div>

            <h3>Promoted Pieces</h3>
            <div className="pieces-grid">
              {promotedPieceData.map((piece, index) => (
                <div key={index} className="piece-card">
                  <div className="piece-header">
                    <SvgPiece 
                      piece={{ type: piece.type, player: 1, promoted: true }}
                      size={40}
                    />
                    <h3>{piece.name}</h3>
                  </div>
                  <p className="piece-description">{piece.description}</p>
                  {renderMovementDiagram(piece)}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default HelpPage;
