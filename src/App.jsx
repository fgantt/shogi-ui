import { Routes, Route } from 'react-router-dom';
import HomePage from './components/HomePage';
import GamePage from './components/GamePage';
import PracticePage from './components/PracticePage';
import PracticeExerciseDetail from './components/PracticeExerciseDetail';
import HelpPage from './components/HelpPage';
import AboutPage from './components/AboutPage';
import './App.css';
import './styles/shogi.css';
import './styles/settings.css';

function App() {
  return (
    <div className="app">
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/game" element={<GamePage />} />
        <Route path="/practice" element={<PracticePage />} />
        <Route path="/practice/:exerciseId" element={<PracticeExerciseDetail />} />
        <Route path="/help" element={<HelpPage />} />
        <Route path="/about" element={<AboutPage />} />
      </Routes>
    </div>
  );
}

export default App;
