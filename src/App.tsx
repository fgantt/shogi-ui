import { Routes, Route } from 'react-router-dom';
import HomePage from './components/HomePage';
import GamePage from './components/GamePage';
import PracticePage from './components/PracticePage';
import PracticeExerciseDetail from './components/PracticeExerciseDetail';
import HelpPage from './components/HelpPage';
import AboutPage from './components/AboutPage';

import { ShogiController } from './usi/controller';
import { WasmEngineAdapter } from './usi/engine';
import { ShogiControllerProvider } from './context/ShogiControllerContext';

import './App.css';
import './styles/shogi.css';
import './styles/settings.css';
import { useEffect } from 'react';

const wasmEngineAdapter = new WasmEngineAdapter('./ai/ai.worker.ts');
const shogiController = new ShogiController(wasmEngineAdapter);

function App() {
  const [isControllerInitialized, setIsControllerInitialized] = useState(false);

  useEffect(() => {
    if (!isControllerInitialized) {
      shogiController.initialize().then(() => {
        setIsControllerInitialized(true);
      });
    }

    // Initialize default wallpaper for all routes
    const initializeDefaultWallpaper = async () => {
      const modules = import.meta.glob('/public/wallpapers/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      if (paths.length > 0) {
        // Set photo1.jpg as the default wallpaper
        const defaultWallpaper = '/wallpapers/photo1.jpg';
        const initialWallpaper = paths.includes(defaultWallpaper) ? defaultWallpaper : paths[0];
        document.body.style.backgroundImage = `url('${initialWallpaper}')`;
        document.body.style.backgroundSize = 'cover';
        document.body.style.backgroundRepeat = 'no-repeat';
        document.body.style.backgroundPosition = 'center center';
        document.body.style.backgroundAttachment = 'fixed';
      }
    };
    
    initializeDefaultWallpaper();
  }, [isControllerInitialized]);

  return (
    <div className="app">
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route 
          path="/game" 
          element={
            <ShogiControllerProvider controller={shogiController}>
              <GamePage />
            </ShogiControllerProvider>
          } 
        />
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