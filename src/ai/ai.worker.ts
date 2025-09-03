import { get_best_move_from_sfen } from '../../pkg-bundler/shogi_engine.js'; // Keep this import for now

let position_sfen: string;
let moves: string[] = [];

let wasmModule: typeof import('../../pkg-bundler/shogi_engine.js'); // Declare a type for the dynamically imported module

// Initialize the wasm module
async function initWasm() {
  try {
    console.log('AI Worker: Initializing WASM module...');
    // Dynamically import the wasm module
    wasmModule = await import('../../pkg-bundler/shogi_engine.js');
    await wasmModule.default(); // Call the default export to initialize
    console.log('AI Worker: WASM module initialized.');
  } catch (error) {
    console.error('AI Worker: Failed to initialize WASM module:', error);
    throw error;
  }
}

const wasmReady = initWasm();

self.onmessage = async (e: MessageEvent) => {
  const { command, ...options } = e.data;

  console.log('AI Worker: Received command:', command);
  console.log('AI Worker: Awaiting wasmReady...');
  await wasmReady;
  console.log('AI Worker: wasmReady resolved.');

  switch (command) {
    case 'usi':
      self.postMessage({ command: 'usiok' });
      console.log('AI Worker: Sent usiok.');
      break;
    case 'isready':
      self.postMessage({ command: 'readyok' });
      console.log('AI Worker: Sent readyok.');
      break;
    case 'setoption':
      // Options are not yet implemented in the wasm engine
      break;
    case 'usinewgame':
      // Handled by resetting position
      break;
    case 'position':
      handlePosition(options.position);
      break;
    case 'go':
      // Use wasmModule.get_best_move_from_sfen
      const bestMove = wasmModule.get_best_move_from_sfen(position_sfen, options.difficulty || 5, options.time_limit_ms || 5000);
      self.postMessage({ command: 'bestmove', move: bestMove });
      break;
    case 'stop':
      // Stop is not yet implemented
      break;
    case 'quit':
      self.close();
      break;
  }
};

function handlePosition(position: string) {
  const parts = position.split(' ');
  if (parts[0] === 'sfen') {
    position_sfen = parts.slice(1, 7).join(' ');
    if (parts.length > 8 && parts[7] === 'moves') {
      moves = parts.slice(8);
    } else {
      moves = [];
    }
  }
  // The wasm engine works with a single SFEN, so we need to apply the moves
  // This is not ideal, but it's how the current wasm interface is designed
  // A better approach would be for the wasm module to handle moves internally
  // For now, we will rely on tsshogi in the controller to manage the record
}

async function handleGo(options: any): Promise<string | null> {
    // The wasm function needs the full SFEN including moves, but our rust code does not support that.
    // The controller sends the root sfen and the moves separately.
    // The rust function get_best_move_from_sfen only takes a sfen.
    // The controller is responsible for managing the full game state and giving us the current position.
    const difficulty = 5; // Hardcoded for now
    const time_limit_ms = options.btime || 5000;

    // Use wasmModule.get_best_move_from_sfen
    const bestMove = wasmModule.get_best_move_from_sfen(position_sfen, difficulty, time_limit_ms);

    return bestMove || null;
}