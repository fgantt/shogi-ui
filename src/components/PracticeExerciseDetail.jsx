import React, { useState, useEffect } from 'react';
import { useNavigate, useParams, useLocation } from 'react-router-dom';
import SvgPiece from './SvgPiece';
import { KING, GOLD, SILVER, ROOK, BISHOP, PAWN, KNIGHT, LANCE } from '../game/engine';
import './PracticeExerciseDetail.css';

const PracticeExerciseDetail = () => {
  const navigate = useNavigate();
  const { exerciseId } = useParams();
  const [currentQuestion, setCurrentQuestion] = useState(0);
  const [score, setScore] = useState(0);
  const [showFeedback, setShowFeedback] = useState(false);
  const [selectedAnswer, setSelectedAnswer] = useState(null);
  const [isCorrect, setIsCorrect] = useState(null);

  // Sample questions for piece identification
  const pieceIdentificationQuestions = [
    {
      piece: { type: KING, player: 1, promoted: false },
      question: 'What piece is this?',
      options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Rook (飛車)'],
      correctAnswer: 0
    },
    {
      piece: { type: GOLD, player: 1, promoted: false },
      question: 'What piece is this?',
      options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Bishop (角行)'],
      correctAnswer: 1
    },
    {
      piece: { type: SILVER, player: 1, promoted: false },
      question: 'What piece is this?',
      options: ['King (王将)', 'Gold General (金将)', 'Silver General (銀将)', 'Knight (桂馬)'],
      correctAnswer: 2
    },
    {
      piece: { type: ROOK, player: 1, promoted: false },
      question: 'What piece is this?',
      options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'],
      correctAnswer: 2
    },
    {
      piece: { type: BISHOP, player: 1, promoted: false },
      question: 'What piece is this?',
      options: ['King (王将)', 'Gold General (金将)', 'Rook (飛車)', 'Bishop (角行)'],
      correctAnswer: 3
    }
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

  const getQuestions = () => {
    switch (exerciseId) {
      case 'name-identification':
        return pieceIdentificationQuestions;
      case 'movement-identification':
        return movementIdentificationQuestions;
      default:
        return pieceIdentificationQuestions;
    }
  };

  const questions = getQuestions();
  const totalQuestions = questions.length;
  const progress = ((currentQuestion + 1) / totalQuestions) * 100;

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
        return 'Identify Shogi pieces by their appearance and kanji characters';
      case 'movement-identification':
        return 'Learn how different Shogi pieces move on the board';
      default:
        return 'Practice your Shogi skills';
    }
  };

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
