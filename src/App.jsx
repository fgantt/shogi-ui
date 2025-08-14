import { Routes, Route } from 'react-router-dom';
import HomePage from './components/HomePage';
import GamePage from './components/GamePage';
import PracticePage from './components/PracticePage';
import PracticeExerciseDetail from './components/PracticeExerciseDetail';
import HelpPage from './components/HelpPage';
import AboutPage from './components/AboutPage';
import WebAssemblyDemo from './components/WebAssemblyDemo';
import './App.css';
import './styles/shogi.css';
import './styles/settings.css';
import { useEffect } from 'react';
import { test_logging } from "../pkg-bundler/shogi_engine.js";

function App() {
  useEffect(() => {
    test_logging();
  }, []);

  return (
    <div className="app">
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/game" element={<GamePage />} />
        <Route path="/practice" element={<PracticePage />} />
        <Route path="/practice/:exerciseId" element={<PracticeExerciseDetail />} />
        <Route path="/help" element={<HelpPage />} />
        <Route path="/about" element={<AboutPage />} />
        <Route path="/wasm-demo" element={<WebAssemblyDemo />} />
      </Routes>
    </div>
  );
}

export default App;