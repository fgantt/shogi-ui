import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import SvgPiece from './SvgPiece';
import { KING, GOLD, SILVER, KNIGHT, LANCE, ROOK, BISHOP, PAWN } from '../game/engine';
import './HelpPage.css';

const HelpPage: React.FC = () => {
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

  const renderMovementDiagram = (piece: any) => {
    const gridSize = 5;
    const center = Math.floor(gridSize / 2);
    
    return (
      <div className="movement-diagram" style={{ width: 'auto', height: 'auto' }}>
        <div className="diagram-grid" style={{ gridTemplateColumns: `repeat(${gridSize}, 1fr)` }}>
          {Array.from({ length: gridSize * gridSize }, (_, index) => {
            const row = Math.floor(index / gridSize);
            const col = index % gridSize;
            const relativeRow = row - center;
            const relativeCol = col - center;
            
            const isPiece = row === center && col === center;
            const isLegalMove = piece.movement.some(([dr, dc]: [number, number]) => 
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
                    hideText={true}
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
        <button 
          className={`nav-tab ${activeSection === 'kifu-notation' ? 'active' : ''}`}
          onClick={() => setActiveSection('kifu-notation')}
        >
          Kifu Notation
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

        {activeSection === 'kifu-notation' && (
          <div className="content-section">
            <h2>Kifu Notation Explained</h2>
            <p>A kifu is a record of a shogi game, documenting each move made by both players. Understanding the notation allows you to follow along with game analysis and improve your own playing. There are several notation systems, but they share common elements.</p>

            <h3>Core Components of a Move</h3>
            <p>A standard move notation includes the following parts, though some may be omitted if the move is unambiguous:</p>
            <ul>
              <li><strong>Player Indicator:</strong> Often, a symbol indicates which player is moving. Black (Sente, the first player) is typically represented by a black triangle (▲), while White (Gote, the second player) is shown with a white triangle (△).</li>
              <li><strong>Piece:</strong> The piece being moved is identified, usually by its initial in Western notation (e.g., P for Pawn, R for Rook) or its Japanese character (e.g., 歩 for Pawn).</li>
              <li><strong>Destination Coordinates:</strong> The square the piece moves to is always indicated. Shogi boards are a 9x9 grid. Files (columns) are numbered 1 to 9 from right to left, and ranks (rows) are numbered 1 to 9 from top to bottom (from Black's perspective). A move to the square at file 7, rank 6 would be written as 76 in Western notation or ７六 in Japanese.</li>
              <li><strong>Action/Movement Type:</strong> Symbols are used to clarify the type of move.
                <ul>
                  <li><strong>Normal Move:</strong> A hyphen (-) is often used in Western notation (e.g., P-76).</li>
                  <li><strong>Capture:</strong> An 'x' indicates a capture (e.g., Rx24).</li>
                  <li><strong>Drop:</strong> An asterisk (*) signifies a piece being dropped onto the board from hand (e.g., P*23). In Japanese notation, this is indicated by the character '打'.</li>
                </ul>
              </li>
              <li><strong>Promotion:</strong>
                <ul>
                  <li>A plus sign (+) is added if a piece is promoted (e.g., Bx27+). In Japanese notation, this is indicated by the character '成'.</li>
                  <li>An equals sign (=) is used if a player chooses not to promote a piece when given the option (e.g., Nx53=).</li>
                </ul>
              </li>
            </ul>

            <h3>Resolving Ambiguity</h3>
            <p>When two or more of the same piece could move to the same square, additional information is added to the notation to clarify which piece moved.</p>
            <ul>
              <li><strong>Origin Square:</strong> The starting square of the piece can be included (e.g., G77-78 to distinguish it from G68-78).</li>
              <li><strong>Directional Indicators:</strong> Japanese notation often uses characters to show the direction of movement or the relative position of the piece.
                <ul>
                  <li>右 (migi): Right</li>
                  <li>左 (hidari): Left</li>
                  <li>上 (agaru): Forward/Up</li>
                  <li>引 (hiku): Backward/Pull</li>
                  <li>寄 (yoru): Sideways</li>
                  <li>直 (choku): Straight (when a piece could also move diagonally)</li>
                </ul>
              </li>
            </ul>

            <h3>Special Notation</h3>
            <ul>
              <li><strong>Same Square Capture (同 dou):</strong> If a piece captures the opponent's piece on the same square where the opponent just moved, the destination coordinates can be replaced with the character '同', meaning "same." This always involves a capture.</li>
            </ul>

            <h3>KIF File Format</h3>
            <p>The KIF (Kifu) file format is a common text-based format used to record Shogi games, studies, and puzzles. It provides a structured way to represent all aspects of a game, from initial setup to the final move and comments.</p>
            <h4>Key Sections of a KIF File:</h4>
            <ul>
              <li><strong>Header:</strong> Contains general information about the game, such as player names (先手 for Sente/black, 後手 for Gote/white), handicap (手合割), start and end dates (開始日時, 終了日時), tournament name (棋戦), location (場所), and time controls (持ち時間, 秒読み).</li>
              <li><strong>Board Setup:</strong> Specifies the initial board position for non-standard games or puzzles, similar to SFEN. It shows pieces on the board and pieces in hand.</li>
              <li><strong>Moves:</strong> Describes individual moves, including destination, piece, promotion (成), and origin. Drops are indicated by 「打」.</li>
              <li><strong>Termination Moves:</strong> Records the reason for game termination, such as resignation (投了), four-fold repetition (千日手), checkmate (詰み), or loss on time (切れ負け).</li>
              <li><strong>Time Expended:</strong> Shows the time spent on a move and the total time expended by a player.</li>
              <li><strong>Variations:</strong> Allows for branching move sequences, starting with 「変化：」 followed by the move number from which the variation branches.</li>
              <li><strong>Comments:</strong> Lines starting with '#' are ignored by parsers and can contain general information. Comments on moves start with '*'.</li>
            </ul>
          </div>
        )}
      </div>
    </div>
  );
};

export default HelpPage;
