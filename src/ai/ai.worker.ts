console.log('AI Worker: Script loading.');

let wasmModule: typeof import('../../pkg-bundler/shogi_engine.js');
let engine: any;
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
    engine = wasmModule.ShogiEngine.new();
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

function posToUsi(pos) {
  const file = 9 - pos.col;
  const rank = String.fromCharCode('a'.charCodeAt(0) + pos.row);
  return `${file}${rank}`;
}

function moveToUsi(move) {
  if (!move) return 'resign';

  const to = posToUsi(move.to);

  // Handle drops
  if (move.from === null || move.from === undefined) {
    const pieceMap = ['P', 'L', 'N', 'S', 'G', 'B', 'R']; // Corresponds to PieceType enum order
    const pieceChar = pieceMap[move.piece_type];
    return `${pieceChar}*${to}`;
  }

  // Handle normal moves
  const from = posToUsi(move.from);
  const promotion = move.is_promotion ? '+' : '';
  return `${from}${to}${promotion}`;
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
      const bestMove = engine.get_best_move(options.difficulty || 5, options.time_limit_ms || 5000);
      const moveString = moveToUsi(bestMove);
      self.postMessage({ command: 'bestmove', move: moveString });
      break;
    case 'stop':
      break;
    case 'quit':
      self.close();
      break;
  }
}

function handlePosition(position: string) {
  const parts = position.split(' ');
  if (parts[0] !== 'sfen') return;

  const sfenBoard = parts[1];
  const sfenPlayer = parts[2];
  const sfenHand = parts[3];

  const player = sfenPlayer === 'b' ? 'Black' : 'White';

  const piecesJson = [];
  const ranks = sfenBoard.split('/');
  for (let r = 0; r < ranks.length; r++) {
    let col = 0;
    let promoted = false;
    for (const char of ranks[r]) {
      if (promoted) {
        const pieceType = getPieceType(char, true);
        piecesJson.push({ 
          position: { row: r, col }, 
          piece_type: pieceType, 
          player: isUpperCase(char) ? 'Black' : 'White' 
        });
        promoted = false;
        col++;
        continue;
      }

      if (char === '+') {
        promoted = true;
        continue;
      }

      const num = parseInt(char);
      if (!isNaN(num)) {
        col += num;
      } else {
        const pieceType = getPieceType(char, false);
        piecesJson.push({ 
          position: { row: r, col }, 
          piece_type: pieceType, 
          player: isUpperCase(char) ? 'Black' : 'White' 
        });
        col++;
      }
    }
  }

  const capturedJson = [];
  if (sfenHand !== '-') {
    let count = 1;
    for (const char of sfenHand) {
      const num = parseInt(char);
      if (!isNaN(num)) {
        count = num;
      } else {
        const pieceType = getPieceType(char, false);
        for (let i = 0; i < count; i++) {
          capturedJson.push({ 
            piece_type: pieceType, 
            player: isUpperCase(char) ? 'Black' : 'White' 
          });
        }
        count = 1;
      }
    }
  }

  const info = {
    board_json: JSON.stringify(piecesJson),
    captured_json: JSON.stringify(capturedJson),
    player: player,
  };

  engine.set_position_from_info(JSON.stringify(info));
}

function getPieceType(char, promoted) {
  const c = char.toLowerCase();
  switch (c) {
    case 'p': return promoted ? 'PromotedPawn' : 'Pawn';
    case 'l': return promoted ? 'PromotedLance' : 'Lance';
    case 'n': return promoted ? 'PromotedKnight' : 'Knight';
    case 's': return promoted ? 'PromotedSilver' : 'Silver';
    case 'g': return 'Gold';
    case 'b': return promoted ? 'PromotedBishop' : 'Bishop';
    case 'r': return promoted ? 'PromotedRook' : 'Rook';
    case 'k': return 'King';
    default: throw new Error(`Unknown piece type: ${char}`);
  }
}

function isUpperCase(char) {
  return char === char.toUpperCase();
}

// Start the initialization
initWasm();

console.log('AI Worker: Script evaluation complete.');