import React, { useState, useEffect } from 'react';
import { useNavigate, useParams, useLocation } from 'react-router-dom';
import SvgPiece from './SvgPiece';
import { 
  KING, GOLD, SILVER, ROOK, BISHOP, PAWN, KNIGHT, LANCE,
  PROMOTED_ROOK, PROMOTED_BISHOP, PROMOTED_SILVER, PROMOTED_KNIGHT, PROMOTED_LANCE, PROMOTED_PAWN
} from '../game/engine';
import './PracticeExerciseDetail.css';

const PracticeExerciseDetail = () => {
  const navigate = useNavigate();
  const { exerciseId } = useParams();
  const [currentQuestion, setCurrentQuestion] = useState(0);
  const [score, setScore] = useState(0);
  const [showFeedback, setShowFeedback] = useState(false);
  const [selectedAnswer, setSelectedAnswer] = useState(null);
  const [isCorrect, setIsCorrect] = useState(null);
  const [questions, setQuestions] = useState([]);

  // All possible pieces for name identification (including promoted pieces)
  const allPieces = [
    { type: KING, player: 1, promoted: false, name: 'King (王将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Rook (飛車)'] },
    { type: GOLD, player: 1, promoted: false, name: 'Gold General (金将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Bishop (角行)'] },
    { type: SILVER, player: 1, promoted: false, name: 'Silver General (銀将)', options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Knight (桂馬)'] },
    { type: ROOK, player: 1, promoted: false, name: 'Rook (飛車)', options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'] },
    { type: BISHOP, player: 1, promoted: false, name: 'Bishop (角行)', options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'] },
    { type: KNIGHT, player: 1, promoted: false, name: 'Knight (桂馬)', options: ['King (王将)', 'Gold General (金将)', 'Knight (桂馬)', 'Lance (香車)'] },
    { type: LANCE, player: 1, promoted: false, name: 'Lance (香車)', options: ['King (王将)', 'Gold General (金将)', 'Knight (桂馬)', 'Lance (香車)'] },
    { type: PAWN, player: 1, promoted: false, name: 'Pawn (歩兵)', options: ['King (王将)', 'Gold General (金将)', 'Pawn (歩兵)', 'Knight (桂馬)'] },
    { type: PROMOTED_ROOK, player: 1, promoted: true, name: 'Promoted Rook (龍王)', options: ['King (王将)', 'Promoted Rook (龍王)', 'Promoted Bishop (龍馬)', 'Gold General (金将)'] },
    { type: PROMOTED_BISHOP, player: 1, promoted: true, name: 'Promoted Bishop (龍馬)', options: ['King (王将)', 'Promoted Rook (龍王)', 'Promoted Bishop (龍馬)', 'Silver General (銀将)'] },
    { type: PROMOTED_SILVER, player: 1, promoted: true, name: 'Promoted Silver (成銀)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Silver (成銀)', 'Promoted Knight (成桂)'] },
    { type: PROMOTED_KNIGHT, player: 1, promoted: true, name: 'Promoted Knight (成桂)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Knight (成桂)', 'Promoted Lance (成香)'] },
    { type: PROMOTED_LANCE, player: 1, promoted: true, name: 'Promoted Lance (成香)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Knight (成桂)', 'Promoted Lance (成香)'] },
    { type: PROMOTED_PAWN, player: 1, promoted: true, name: 'Promoted Pawn (と金)', options: ['King (王将)', 'Gold General (金将)', 'Promoted Pawn (と金)', 'Promoted Silver (成銀)'] }
  ];

  // Sample questions for movement identification
  const movementIdentificationQuestions = [
    {
      piece: { type: PAWN, player: 1, promoted: false },
      question: 'How does a pawn move?',
      options: ['One square forward', 'One square in any direction', 'Two squares forward', 'Diagonally forward'],
      correctAnswer: 0
    },
    {
      piece: { type: KNIGHT, player: 1, promoted: false },
      question: 'How does a knight move?',
      options: ['One square forward', 'Two squares forward and one sideways', 'Any number of squares forward', 'Diagonally'],
      correctAnswer: 1
    },
    {
      piece: { type: LANCE, player: 1, promoted: false },
      question: 'How does a lance move?',
      options: ['One square forward', 'Any number of squares forward', 'Diagonally', 'Any direction'],
      correctAnswer: 1
    }
  ];

  // Function to shuffle array and get random subset
  const shuffleAndSelect = (array, count) => {
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
        options: shuffleAndSelect(piece.options, 4), // Shuffle the options too
        correctAnswer: 0, // First option is always the correct one after shuffling
        correctName: piece.name
      }));
      
      // Update correctAnswer index based on where the correct name ended up
      generatedQuestions.forEach(q => {
        q.correctAnswer = q.options.findIndex(option => option === q.correctName);
      });
      
      setQuestions(generatedQuestions);
    } else {
      setQuestions(movementIdentificationQuestions);
    }
  }, [exerciseId]);

  const totalQuestions = questions.length;
  const progress = totalQuestions > 0 ? ((currentQuestion + 1) / totalQuestions) * 100 : 0;

  const handleAnswerSelect = (answerIndex) => {
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
      default:
        return 'Practice Exercise';
    }
  };

  const getExerciseDescription = () => {
    switch (exerciseId) {
      case 'name-identification':
        return 'Identify Shogi pieces by their appearance and kanji characters (including promoted pieces)';
      case 'movement-identification':
        return 'Learn how different Shogi pieces move on the board';
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
            <SvgPiece piece={currentQ.piece} size={80} />
          </div>
          
          <div className="question-text">
            {currentQ.question}
          </div>
          
          <div className="answer-options">
            {currentQ.options.map((option, index) => (
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
                {option}
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
                  ✗ Incorrect. The correct answer is: {currentQ.options[currentQ.correctAnswer]}
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
