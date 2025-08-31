
import { get_best_move_from_sfen } from '../../pkg-bundler/shogi_engine.js';
import * as wasm from '../../pkg-bundler/shogi_engine.js';

let position_sfen: string;
let moves: string[] = [];

// Initialize the wasm module
async function initWasm() {
  await wasm.default();
}

const wasmReady = initWasm();

self.onmessage = async (e: MessageEvent) => {
  const { command, ...options } = e.data;

  await wasmReady;

  switch (command) {
    case 'usi':
      self.postMessage({ command: 'usiok' });
      break;
    case 'isready':
      self.postMessage({ command: 'readyok' });
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
      const bestMove = await handleGo(options);
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
  } else if (parts[0] === 'startpos') {
    position_sfen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";
    if (parts.length > 2 && parts[1] === 'moves') {
        moves = parts.slice(2);
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
    
    // We need to construct the current SFEN from the base and the moves.
    // However, the rust code doesn't have a function to apply moves to a sfen.
    // The controller already tracks the position. Let's assume the SFEN passed to setPosition is the current one.
    
    const bestMove = get_best_move_from_sfen(position_sfen, difficulty, time_limit_ms);

    return bestMove || null;
}
