import React from 'react';
import { useNavigate } from 'react-router-dom';
import './PracticePage.css';

const PracticePage = () => {
  const navigate = useNavigate();

  const practiceExercises = [
    {
      id: 'name-identification',
      title: 'Name Identification',
      description: 'Learn to identify Shogi pieces by their appearance and kanji characters, including promoted pieces.',
      icon: '🎯',
      difficulty: 'Beginner'
    },
    {
      id: 'movement-identification',
      title: 'Movement Identification',
      description: 'Practice recognizing the movement patterns of different Shogi pieces.',
      icon: '♟️',
      difficulty: 'Beginner'
    }
  ];

  const handleExerciseSelect = (exerciseId) => {
    navigate(`/practice/${exerciseId}`);
  };

  return (
    <div className="practice-page">
      <div className="practice-header">
        <button className="back-button" onClick={() => navigate('/')}>
          ← Back to Home
        </button>
        <h1>Practice Exercises</h1>
        <p>Choose an exercise to improve your Shogi skills</p>
      </div>

      <div className="practice-content">
        <div className="exercises-grid">
          {practiceExercises.map((exercise) => (
            <div 
              key={exercise.id} 
              className="exercise-card"
              onClick={() => handleExerciseSelect(exercise.id)}
            >
              <div className="exercise-icon">{exercise.icon}</div>
              <div className="exercise-info">
                <h3>{exercise.title}</h3>
                <p>{exercise.description}</p>
                <span className="difficulty-badge">{exercise.difficulty}</span>
              </div>
              <div className="exercise-arrow">→</div>
            </div>
          ))}
        </div>

        <div className="practice-tips">
          <h3>💡 Practice Tips</h3>
          <ul>
            <li>Start with Name Identification to learn piece recognition (now with 10 randomized questions including promoted pieces!)</li>
            <li>Practice regularly to build muscle memory</li>
            <li>Focus on understanding movement patterns</li>
            <li>Don't worry about speed - accuracy comes first</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default PracticePage;
