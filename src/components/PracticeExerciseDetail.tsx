import React, { useState, useEffect } from 'react';
import { useNavigate, useParams, useLocation } from 'react-router-dom';
import SvgPiece from './SvgPiece';
import { PieceType } from 'tsshogi';
import './PracticeExerciseDetail.css';

const PracticeExerciseDetail: React.FC = () => {
  const navigate = useNavigate();
  const { exerciseId } = useParams<{ exerciseId: string }>();
  const { pathname } = useLocation();
  const [currentQuestion, setCurrentQuestion] = useState<number>(0);
  const [score, setScore] = useState<number>(0);
  const [showFeedback, setShowFeedback] = useState<boolean>(false);
  const [selectedAnswer, setSelectedAnswer] = useState<number | null>(null);
  const [isCorrect, setIsCorrect] = useState<boolean | null>(null);
  const [questions, setQuestions] = useState<any[]>([]); // TODO: Define a proper interface for questions
  const [pieceThemeType, setPieceThemeType] = useState<string>(localStorage.getItem('shogi-piece-label-type') || 'kanji');

  // Listen for storage changes to sync theme updates
  useEffect(() => {
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === 'shogi-piece-label-type' && e.newValue) {
        setPieceThemeType(e.newValue);
      }
    };

    window.addEventListener('storage', handleStorageChange);
    
    // Also listen for custom events (for same-tab updates)
    const handleThemeChange = (e: CustomEvent) => {
      setPieceThemeType(e.detail);
    };

    window.addEventListener('themeChange', handleThemeChange as EventListener);

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('themeChange', handleThemeChange as EventListener);
    };
  }, []);

  // All possible pieces for name identification (including promoted pieces)
  const allPieces = [
    { type: 'king' as PieceType, player: 'player1', promoted: false, name: 'King (王将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Rook (飛車)'] },
    { type: 'gold' as PieceType, player: 'player1', promoted: false, name: 'Gold General (金将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Bishop (角行)'] },
    { type: 'silver' as PieceType, player: 'player1', promoted: false, name: 'Silver General (銀将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Knight (桂馬)'] },
    { type: 'rook' as PieceType, player: 'player1', promoted: false, name: 'Rook (飛車)', options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'] },
    { type: 'bishop' as PieceType, player: 'player1', promoted: false, name: 'Bishop (角行)', options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'] },
    { type: 'knight' as PieceType, player: 'player1', promoted: false, name: 'Knight (桂馬)', options: ['King (王将)', 'Gold General (金将)', 'Knight (桂馬)', 'Lance (香車)'] },
    { type: 'lance' as PieceType, player: 'player1', promoted: false, name: 'Lance (香車)', options: ['King (王将)', 'Gold General (金将)', 'Knight (桂馬)', 'Lance (香車)'] },
    { type: 'pawn' as PieceType, player: 'player1', promoted: false, name: 'Pawn (歩兵)', options: ['King (王将)', 'Gold General (金将)', 'Pawn (歩兵)', 'Knight (桂馬)'] },
    { type: 'dragon' as PieceType, player: 'player1', promoted: true, name: 'Promoted Rook (龍王)', options: ['King (王将)', 'Promoted Rook (龍王)', 'Promoted Bishop (龍馬)', 'Gold General (金将)'] },
    { type: 'horse' as PieceType, player: 'player1', promoted: true, name: 'Promoted Bishop (龍馬)', options: ['King (王将)', 'Promoted Rook (龍王)', 'Promoted Bishop (龍馬)', 'Silver General (銀将)'] },
    { type: 'promSilver' as PieceType, player: 'player1', promoted: true, name: 'Promoted Silver (成銀)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Silver (成銀)', 'Promoted Knight (成桂)'] },
    { type: 'promKnight' as PieceType, player: 'player1', promoted: true, name: 'Promoted Knight (成桂)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Knight (成桂)', 'Promoted Lance (成香)'] },
    { type: 'promLance' as PieceType, player: 'player1', promoted: true, name: 'Promoted Lance (成香)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Knight (成桂)', 'Promoted Lance (成香)'] },
    { type: 'promPawn' as PieceType, player: 'player1', promoted: true, name: 'Promoted Pawn (と金)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Pawn (と金)', 'Promoted Silver (成銀)'] }
  ];

  // Promotion matching questions
  const promotionMatchingQuestions = [
    {
      piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Pawn (と金)', piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Lance (成香)', piece: { type: 'promLance' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 0,
      explanation: 'Pawns promote to Promoted Pawns (と金), which move like Gold Generals'
    },
    {
      piece: { type: 'knight' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Pawn (と金)', piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Lance (成香)', piece: { type: 'promLance' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 2,
      explanation: 'Knights promote to Promoted Knights (成桂), which move like Gold Generals'
    },
    {
      piece: { type: 'lance' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Pawn (と金)', piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Lance (成香)', piece: { type: 'promLance' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 3,
      explanation: 'Lances promote to Promoted Lances (成香), which move like Gold Generals'
    },
    {
      piece: { type: 'silver' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Pawn (と金)', piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Lance (成香)', piece: { type: 'promLance' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 1,
      explanation: 'Silver Generals promote to Promoted Silvers (成銀), which move like Gold Generals'
    },
    {
      piece: { type: 'rook' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Rook (龍王)', piece: { type: 'dragon' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Bishop (龍馬)', piece: { type: 'horse' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 0,
      explanation: 'Rooks promote to Promoted Rooks (龍王), which can move like Rooks plus one square diagonally'
    },
    {
      piece: { type: 'bishop' as PieceType, player: 'player1', promoted: false },
      question: 'What does this piece become when promoted?',
      options: [
        { text: 'Promoted Rook (龍王)', piece: { type: 'dragon' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Bishop (龍馬)', piece: { type: 'horse' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Silver (成銀)', piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true } },
        { text: 'Promoted Knight (成桂)', piece: { type: 'promKnight' as PieceType, player: 'player1', promoted: true } }
      ],
      correctAnswer: 1,
      explanation: 'Bishops promote to Promoted Bishops (龍馬), which can move like Bishops plus one square orthogonally'
    },
    {
      piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true },
      question: 'What piece promotes to this?',
      options: [
        { text: 'Pawn (歩兵)', piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false } },
        { text: 'Knight (桂馬)', piece: { type: 'knight' as PieceType, player: 'player1', promoted: false } },
        { text: 'Lance (香車)', piece: { type: 'lance' as PieceType, player: 'player1', promoted: false } },
        { text: 'Silver (銀将)', piece: { type: 'silver' as PieceType, player: 'player1', promoted: false } }
      ],
      correctAnswer: 0,
      explanation: 'Promoted Pawns (と金) come from Pawns (歩兵)'
    },
    {
      piece: { type: 'dragon' as PieceType, player: 'player1', promoted: true },
      question: 'What piece promotes to this?',
      options: [
        { text: 'Pawn (歩兵)', piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false } },
        { text: 'Rook (飛車)', piece: { type: 'rook' as PieceType, player: 'player1', promoted: false } },
        { text: 'Bishop (角行)', piece: { type: 'bishop' as PieceType, player: 'player1', promoted: false } },
        { text: 'Silver (銀将)', piece: { type: 'silver' as PieceType, player: 'player1', promoted: false } }
      ],
      correctAnswer: 1,
      explanation: 'Promoted Rooks (龍王) come from Rooks (飛車)'
    },
    {
      piece: { type: 'horse' as PieceType, player: 'player1', promoted: true },
      question: 'What piece promotes to this?',
      options: [
        { text: 'Pawn (歩兵)', piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false } },
        { text: 'Rook (飛車)', piece: { type: 'rook' as PieceType, player: 'player1', promoted: false } },
        { text: 'Bishop (角行)', piece: { type: 'bishop' as PieceType, player: 'player1', promoted: false } },
        { text: 'Silver (銀将)', piece: { type: 'silver' as PieceType, player: 'player1', promoted: false } }
      ],
      correctAnswer: 2,
      explanation: 'Promoted Bishops (龍馬) come from Bishops (角行)'
    },
    {
      piece: { type: 'promSilver' as PieceType, player: 'player1', promoted: true },
      question: 'What piece promotes to this?',
      options: [
        { text: 'Pawn (歩兵)', piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false } },
        { text: 'Knight (桂馬)', piece: { type: 'knight' as PieceType, player: 'player1', promoted: false } },
        { text: 'Lance (香車)', piece: { type: 'lance' as PieceType, player: 'player1', promoted: false } },
        { text: 'Silver (銀将)', piece: { type: 'silver' as PieceType, player: 'player1', promoted: false } }
      ],
      correctAnswer: 3,
      explanation: 'Promoted Silvers (成銀) come from Silver Generals (銀将)'
    }
  ];

  // Movement identification questions with grid diagrams
  const movementIdentificationQuestions = [
    {
      piece: { type: 'pawn' as PieceType, player: 'player1', promoted: false },
      question: 'How does a pawn move?',
      options: [
        { text: 'One square forward', diagram: 'pawn' },
        { text: 'One square in any direction', diagram: 'king' },
        { text: 'Two squares forward', diagram: 'knight' },
        { text: 'Diagonally forward', diagram: 'silver' }
      ],
      correctAnswer: 0
    },
    {
      piece: { type: 'knight' as PieceType, player: 'player1', promoted: false },
      question: 'How does a knight move?',
      options: [
        { text: 'One square forward', diagram: 'pawn' },
        { text: 'Two squares forward and one sideways', diagram: 'knight' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 1
    },
    {
      piece: { type: 'lance' as PieceType, player: 'player1', promoted: false },
      question: 'How does a lance move?',
      options: [
        { text: 'One square forward', diagram: 'pawn' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Diagonally', diagram: 'bishop' },
        { text: 'Any direction', diagram: 'king' }
      ],
      correctAnswer: 1
    },
    {
      piece: { type: 'silver' as PieceType, player: 'player1', promoted: false },
      question: 'How does a silver general move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Any number of squares diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 0
    },
    {
      piece: { type: 'gold' as PieceType, player: 'player1', promoted: false },
      question: 'How does a gold general move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Any number of squares diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 1
    },
    {
      piece: { type: 'bishop' as PieceType, player: 'player1', promoted: false },
      question: 'How does a bishop move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Any number of squares diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 3
    },
    {
      piece: { type: 'rook' as PieceType, player: 'player1', promoted: false },
      question: 'How does a rook move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares horizontally or vertically', diagram: 'rook' },
        { text: 'Any number of squares diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 2
    },
    {
      piece: { type: 'king' as PieceType, player: 'player1', promoted: false },
      question: 'How does a king move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares horizontally or vertically', diagram: 'rook' },
        { text: 'One square in any direction', diagram: 'king' }
      ],
      correctAnswer: 3
    },
    {
      piece: { type: 'promPawn' as PieceType, player: 'player1', promoted: true },
      question: 'How does a promoted pawn move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares forward', diagram: 'lance' },
        { text: 'Any number of squares diagonally', diagram: 'bishop' }
      ],
      correctAnswer: 1
    },
    {
      piece: { type: 'dragon' as PieceType, player: 'player1', promoted: true },
      question: 'How does a promoted rook (dragon king) move?',
      options: [
        { text: 'One square forward or diagonally', diagram: 'silver' },
        { text: 'One square orthogonally or diagonally forward', diagram: 'gold' },
        { text: 'Any number of squares horizontally/vertically + one square diagonally', diagram: 'promotedRook' },
        { text: 'Any number of squares diagonally + one square orthogonally', diagram: 'promotedBishop' }
      ],
      correctAnswer: 2
    }
  ];

  // Function to render movement diagram for answer choices
  const renderMovementDiagram = (diagramType: string) => {
    const gridSize = 5;
    const center = Math.floor(gridSize / 2);
    
    let movement: number[][] = [];
    let pieceType: PieceType = 'pawn';
    let promoted = false;
    
    // Define movement patterns for each diagram type
    switch (diagramType) {
      case 'pawn':
        movement = [[-1, 0]];
        pieceType = 'pawn';
        break;
      case 'knight':
        movement = [[-2, -1], [-2, 1]];
        pieceType = 'knight';
        break;
      case 'lance':
        movement = [[-1, 0], [-2, 0], [-3, 0], [-4, 0]];
        pieceType = 'lance';
        break;
      case 'silver':
        movement = [[-1, -1], [-1, 0], [-1, 1], [1, -1], [1, 1]];
        pieceType = 'silver';
        break;
      case 'gold':
        movement = [[-1, -1], [-1, 0], [-1, 1], [0, -1], [0, 1], [1, 0]];
        pieceType = 'gold';
        break;
      case 'bishop':
        movement = [[-1, -1], [-2, -2], [-3, -3], [-4, -4], [-1, 1], [-2, 2], [-3, 3], [-4, 4], [1, -1], [2, -2], [3, -3], [4, -4], [1, 1], [2, 2], [3, 3], [4, 4]];
        pieceType = 'bishop';
        break;
      case 'rook':
        movement = [[-1, 0], [-2, 0], [-3, 0], [-4, 0], [0, -1], [0, -2], [0, -3], [0, -4], [0, 1], [0, 2], [0, 3], [0, 4], [1, 0], [2, 0], [3, 0], [4, 0]];
        pieceType = 'rook';
        break;
      case 'king':
        movement = [[-1, -1], [-1, 0], [-1, 1], [0, -1], [0, 1], [1, -1], [1, 0], [1, 1]];
        pieceType = 'king';
        break;
      case 'promotedRook':
        movement = [[-1, -1], [-1, 0], [-1, 1], [0, -1], [0, 1], [1, -1], [1, 0], [1, 1], [-2, 0], [-3, 0], [-4, 0], [0, -2], [0, -3], [0, -4], [0, 2], [0, 3], [0, 4], [2, 0], [3, 0], [4, 0]];
        pieceType = 'rook';
        promoted = true;
        break;
      case 'promotedBishop':
        movement = [[-1, -1], [-2, -2], [-3, -3], [-4, -4], [-1, 1], [-2, 2], [-3, 3], [-4, 4], [1, -1], [2, -2], [3, -3], [4, -4], [1, 1], [2, 2], [3, 3], [4, 4], [-1, 0], [0, -1], [0, 1], [1, 0]];
        pieceType = 'bishop';
        promoted = true;
        break;
      default:
        movement = [[-1, 0]];
        pieceType = 'pawn';
    }
    
    return (
      <div className="movement-diagram">
        <div className={`diagram-grid ${pathname === '/practice/movement-identification' ? 'diagram-grid-centered' : ''}`} style={{ gridTemplateColumns: `repeat(${gridSize}, 1fr)` }}>
          {Array.from({ length: gridSize * gridSize }, (_, index) => {
            const row = Math.floor(index / gridSize);
            const col = index % gridSize;
            const relativeRow = row - center;
            const relativeCol = col - center;
            
            const isPiece = row === center && col === center;
            const isLegalMove = movement.some(([dr, dc]) => 
              dr === relativeRow && dc === relativeCol
            );
            
            let className = 'diagram-square';
            if (isPiece) className += ' piece-square';
            else if (isLegalMove) className += ' legal-move';
            
            return (
              <div key={index} className={className}>
                {isPiece && (
                  <SvgPiece 
                    piece={{ type: pieceType, player: 'player1', promoted: promoted }}
                    size={30}
                    hideText={true}
                    pieceThemeType={pieceThemeType}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  // Function to shuffle array and get random subset
  const shuffleAndSelect = (array: any[], count: number) => {
    const shuffled = [...array].sort(() => Math.random() - 0.5);
    return shuffled.slice(0, count);
  };

  // Initialize questions when component mounts or exercise changes
  useEffect(() => {
    if (exerciseId === 'name-identification') {
      // Generate 10 random questions from all pieces
      const selectedPieces = shuffleAndSelect(allPieces, 10);
      const generatedQuestions = selectedPieces.map((piece, index) => ({
        piece: { type: piece.type, player: piece.player, promoted: piece.promoted },
        question: 'What piece is this?',
        options: shuffleAndSelect(piece.options, 4).map(option => ({ text: option, diagram: null })), // Convert to new structure
        correctAnswer: 0, // First option is always the correct one after shuffling
        correctName: piece.name
      }));
      
      // Update correctAnswer index based on where the correct name ended up
      generatedQuestions.forEach(q => {
        q.correctAnswer = q.options.findIndex(option => option.text === q.correctName);
      });
      
      setQuestions(generatedQuestions);
    } else if (exerciseId === 'movement-identification') {
      // Generate 10 random questions from movement identification questions
      const selectedQuestions = shuffleAndSelect(movementIdentificationQuestions, 10);
      setQuestions(selectedQuestions);
    } else if (exerciseId === 'promotion-matching') {
      // Generate 10 random questions from promotion matching questions
      const selectedQuestions = shuffleAndSelect(promotionMatchingQuestions, 10);
      setQuestions(selectedQuestions);
    }
  }, [exerciseId]);

  const totalQuestions = questions.length;
  const progress = totalQuestions > 0 ? ((currentQuestion + 1) / totalQuestions) * 100 : 0;

  const handleAnswerSelect = (answerIndex: number) => {
    if (showFeedback) return; // Prevent multiple selections
    
    setSelectedAnswer(answerIndex);
    const correct = answerIndex === questions[currentQuestion].correctAnswer;
    setIsCorrect(correct);
    
    if (correct) {
      setScore(score + 1);
    }
    
    setShowFeedback(true);
  };

  const handleNextQuestion = () => {
    if (currentQuestion < totalQuestions - 1) {
      setCurrentQuestion(currentQuestion + 1);
      setShowFeedback(false);
      setSelectedAnswer(null);
      setIsCorrect(null);
    }
  };

  const handleFinishExercise = () => {
    navigate('/practice', { 
      state: { 
        completed: true, 
        score: score, 
        total: totalQuestions,
        exerciseId: exerciseId
      } 
    });
  };

  const getExerciseTitle = () => {
    switch (exerciseId) {
      case 'name-identification':
        return 'Piece Name Identification';
      case 'movement-identification':
        return 'Movement Pattern Recognition';
      case 'promotion-matching':
        return 'Promotion Matching';
      default:
        return 'Practice Exercise';
    }
  };

  const getExerciseDescription = () => {
    switch (exerciseId) {
      case 'name-identification':
        return 'Identify Shogi pieces by their appearance and kanji characters (including promoted pieces)';
      case 'movement-identification':
        return 'Learn how different Shogi pieces move on the board using only visual movement diagrams (including promoted pieces)';
      case 'promotion-matching':
        return 'Match Shogi pieces to their promoted versions. Learn which pieces can be promoted and what they become.';
      default:
        return 'Practice your Shogi skills';
    }
  };

  // Don't render until questions are loaded
  if (questions.length === 0) {
    return (
      <div className="practice-exercise-detail">
        <div className="exercise-header">
          <button className="back-button" onClick={() => navigate('/practice')}>
            ← Back to Practice
          </button>
          <h1>Loading...</h1>
        </div>
      </div>
    );
  }

  if (currentQuestion >= totalQuestions) {
    return (
      <div className="practice-exercise-detail">
        <div className="exercise-header">
          <button className="back-button" onClick={() => navigate('/practice')}>
            ← Back to Practice
          </button>
          <h1>Exercise Complete!</h1>
          <p>Great job! You've completed the exercise.</p>
        </div>
        
        <div className="exercise-content">
          <div className="question-card">
            <h2>Final Score: {score}/{totalQuestions}</h2>
            <p>Percentage: {Math.round((score / totalQuestions) * 100)}%</p>
            <button className="finish-button" onClick={handleFinishExercise}>
              Return to Practice Menu
            </button>
          </div>
        </div>
      </div>
    );
  }

  const currentQ = questions[currentQuestion];

  return (
    <div className="practice-exercise-detail">
      <div className="exercise-header">
        <button className="back-button" onClick={() => navigate('/practice')}>
          ← Back to Practice
        </button>
        <h1>{getExerciseTitle()}</h1>
        <p>{getExerciseDescription()}</p>
        
        <div className="progress-bar">
          <div className="progress-fill" style={{ width: `${progress}%` }}></div>
        </div>
        
        <div className="score-display">
          Question {currentQuestion + 1} of {totalQuestions} | Score: {score}/{totalQuestions}
        </div>
      </div>

      <div className="exercise-content">
        <div className="question-card">
          <div className="piece-display">
            <SvgPiece piece={currentQ.piece} size={80} pieceThemeType={pieceThemeType} />
          </div>
          
          <div className="question-text">
            {currentQ.question}
          </div>
          
          <div className="answer-options">
            {currentQ.options.map((option: any, index: number) => (
              <button
                key={index}
                className={`answer-option ${
                  showFeedback && index === currentQ.correctAnswer ? 'correct' : ''
                } ${
                  showFeedback && selectedAnswer === index && index !== currentQ.correctAnswer ? 'incorrect' : ''
                }`}
                onClick={() => handleAnswerSelect(index)}
                disabled={showFeedback}
              >
                {exerciseId === 'movement-identification' ? (
                  renderMovementDiagram(option.diagram)
                ) : exerciseId === 'promotion-matching' && option.piece ? (
                  <div className="piece-option">
                    <SvgPiece piece={option.piece} size={80} pieceThemeType={pieceThemeType} />
                  </div>
                ) : (
                  option.text
                )}
              </button>
            ))}
          </div>
          
          {showFeedback && (
            <div className="answer-feedback">
              {isCorrect ? (
                <div className="correct-answer">
                  ✓ Correct! Well done!
                </div>
              ) : (
                <div className="incorrect-answer">
                  ✗ Incorrect. The correct answer is: {exerciseId === 'movement-identification' ? currentQ.options[currentQ.correctAnswer].text : currentQ.options[currentQ.correctAnswer].text}
                </div>
              )}
              
              {exerciseId === 'promotion-matching' && currentQ.explanation && (
                <div className="explanation">
                  <strong>Explanation:</strong> {currentQ.explanation}
                </div>
              )}
              
              <div className="action-buttons">
                {currentQuestion < totalQuestions - 1 ? (
                  <button className="next-button" onClick={handleNextQuestion}>
                    Next Question
                  </button>
                ) : (
                  <button className="finish-button" onClick={handleFinishExercise}>
                    Finish Exercise
                  </button>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PracticeExerciseDetail;