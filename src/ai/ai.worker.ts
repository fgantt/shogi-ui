console.log('AI Worker: Script loading.');

let wasmModule: typeof import('../../pkg-bundler/shogi_engine.js');
let isWasmReady = false;
const commandQueue: MessageEvent[] = [];

// Assign the message handler immediately to prevent race conditions.
self.onmessage = (e: MessageEvent) => {
  console.log('AI Worker: MESSAGE RECEIVED:', e.data);
  console.log('AI Worker: Received command:', e.data.command);
  if (isWasmReady) {
    handleMessage(e);
  } else {
    console.log('AI Worker: WASM not ready. Queuing command.');
    commandQueue.push(e);
  }
};
console.log('AI Worker: onmessage handler assigned.');

// Initialize the wasm module
async function initWasm() {
  try {
    console.log('AI Worker: Initializing WASM module...');
    wasmModule = await import('../../pkg-bundler/shogi_engine.js');
    console.log('AI Worker: WASM module initialized.');
    isWasmReady = true;
    processCommandQueue();
  } catch (error) {
    console.error('AI Worker: Failed to initialize WASM module:', error);
  }
}

// Process the queue of commands that arrived before WASM was ready
function processCommandQueue() {
  console.log(`AI Worker: Processing command queue with ${commandQueue.length} commands.`);
  while(commandQueue.length > 0) {
    const event = commandQueue.shift();
    if (event) {
      handleMessage(event);
    }
  }
}

// Main message handler
function handleMessage(e: MessageEvent) {
  const { command, ...options } = e.data;
  console.log('AI Worker: Handling command:', command);

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
      break;
    case 'usinewgame':
      break;
    case 'position':
      handlePosition(options.position);
      break;
    case 'go':
      const bestMove = wasmModule.get_best_move_from_sfen(position_sfen, options.difficulty || 5, options.time_limit_ms || 5000);
      self.postMessage({ command: 'bestmove', move: bestMove });
      break;
    case 'stop':
      break;
    case 'quit':
      self.close();
      break;
  }
}

let position_sfen: string;
let moves: string[] = [];
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
}

// Start the initialization
initWasm();

console.log('AI Worker: Script evaluation complete.');