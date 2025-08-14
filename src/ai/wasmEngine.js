import { ShogiEngine, PieceType, Player } from '../../pkg-bundler/shogi_engine.js';

let wasmEngine = null;
let isInitialized = false;

/**
 * Initialize the WebAssembly engine
 */
export async function initializeWasmEngine() {
    if (isInitialized) {
        return;
    }
    
    try {
        isInitialized = true;
    } catch (error) {
        console.error('Failed to initialize WebAssembly engine:', error);
        throw error;
    }
}

/**
 * Get the best move using the WebAssembly engine
 */
export async function getWasmAiMove(gameState, difficulty) {
    if (!isInitialized) {
        await initializeWasmEngine();
    }
    
    try {
        // Create new engine instance
        const engine = ShogiEngine.new();
        
        // Convert game state to engine format
        const engineState = convertGameStateToEngine(gameState);
        
        // Set up the engine with current position
        setupEnginePosition(engine, engineState);
        
        // Get time limit based on difficulty
        const timeLimit = getTimeLimit(difficulty);
        
        // Get best move
        const bestMove = engine.get_best_move(difficulty, timeLimit);
        
        if (bestMove) {
            // Convert engine move back to game format
            return convertEngineMoveToGame(bestMove);
        } else {
            console.warn('No move returned from WebAssembly engine');
            return null;
        }
        
    } catch (error) {
        console.error('Error in WebAssembly engine:', error);
        throw error;
    }
}

/**
 * Convert game state to engine format
 */
function convertGameStateToEngine(gameState) {
    try {
        const { board, currentPlayer, capturedPieces, moveHistory } = gameState;
        
        // Validate board structure
        if (!board || !Array.isArray(board) || board.length !== 9) {
            throw new Error('Invalid board structure: board must be a 9x9 array');
        }
        
        // Convert the board to a format the Rust engine can understand
        const engineBoard = [];
        
        for (let row = 0; row < 9; row++) {
            if (!board[row] || !Array.isArray(board[row]) || board[row].length !== 9) {
                throw new Error(`Invalid board row ${row}: must be an array of length 9`);
            }
            
            for (let col = 0; col < 9; col++) {
                const cell = board[8 - row][col];
                if (cell && typeof cell === 'object' && cell.type && cell.player) {
                    const pieceType = convertPieceTypeToEngine(cell.type);
                    const player = cell.player === 'player1' ? 'Black' : 'White';
                    engineBoard.push({
                        position: { row, col },
                        piece_type: pieceType,
                        player: player
                    });
                }
            }
        }
        
        return {
            board: engineBoard,
            currentPlayer: currentPlayer === 'player1' ? 'Black' : 'White',
            capturedPieces: convertCapturedPieces(capturedPieces),
            moveHistory: moveHistory || []
        };
    } catch (error) {
        console.error('Error converting game state:', error);
        console.error('Game state received:', gameState);
        throw new Error(`Failed to convert game state: ${error.message}`);
    }
}

/**
 * Convert piece types from game format to engine format
 */
function convertPieceTypeToEngine(pieceType) {
    const pieceMap = {
        'K': 'King',
        'R': 'Rook',
        'B': 'Bishop',
        'G': 'Gold',
        'S': 'Silver',
        'N': 'Knight',
        'L': 'Lance',
        'P': 'Pawn',
        '+P': 'PromotedPawn',
        '+L': 'PromotedLance',
        '+N': 'PromotedKnight',
        '+S': 'PromotedSilver',
        '+B': 'PromotedBishop',
        '+R': 'PromotedRook'
    };
    
    return pieceMap[pieceType] || 'Pawn';
}

/**
 * Convert captured pieces to engine format
 */
function convertCapturedPieces(capturedPieces) {
    const engineCaptured = [];
    
    // Convert player1 captured pieces
    if (capturedPieces.player1) {
        capturedPieces.player1.forEach(piece => {
            engineCaptured.push({
                piece_type: convertPieceTypeToEngine(piece.type),
                player: 'White' // These were captured from White
            });
        });
    }
    
    // Convert player2 captured pieces
    if (capturedPieces.player2) {
        capturedPieces.player2.forEach(piece => {
            engineCaptured.push({
                piece_type: convertPieceTypeToEngine(piece.type),
                player: 'Black' // These were captured from Black
            });
        });
    }
    
    return engineCaptured;
}

/**
 * Set up the engine with the current position
 */
function setupEnginePosition(engine, engineState) {
    try {
        // Set the board position
        engine.set_position(JSON.stringify(engineState.board));
        
        // Set current player
        engine.set_current_player(engineState.currentPlayer);
        
        // Set captured pieces
        engine.set_captured_pieces(JSON.stringify(engineState.capturedPieces));
        
    } catch (error) {
        console.error('Error setting up engine position:', error);
        throw error;
    }
}

/**
 * Get time limit based on difficulty
 */
function getTimeLimit(difficulty) {
    switch (difficulty) {
        case 'easy':
            return 1000; // 1 second
        case 'medium':
            return 3000; // 3 seconds
        case 'hard':
            return 9000; // 9 seconds
        default:
            return 3000; // Default to medium
    }
}

/**
 * Convert engine move back to game format
 */
function convertEngineMoveToGame(engineMove) {
    try {
        if (engineMove.from === null || engineMove.from === undefined) {
            // Drop move
            const toCoord = [8 - engineMove.to.row, engineMove.to.col];
            return {
                from: 'drop',
                to: toCoord,
                pieceType: convertEnginePieceTypeToGame(engineMove.piece_type),
                isCapture: false,
                isPromotion: false,
                isCheck: false
            };
        } else {
            // Regular move
            const fromCoord = [8 - engineMove.from.col, engineMove.from.row];
            const toCoord = [8 - engineMove.to.col, engineMove.to.row];
            return {
                from: fromCoord,
                to: toCoord,
                type: 'move',
                isCapture: engineMove.is_capture || false,
                isCheck: false,
                promote: engineMove.is_promotion || false
            };
        }
    } catch (error) {
        console.error('Error converting engine move:', error);
        throw error;
    }
}

/**
 * Convert engine piece types back to game format
 */
function convertEnginePieceTypeToGame(pieceType) {
    const reversePieceMap = {
        [PieceType.King]: 'K',
        [PieceType.Rook]: 'R',
        [PieceType.Bishop]: 'B',
        [PieceType.Gold]: 'G',
        [PieceType.Silver]: 'S',
        [PieceType.Knight]: 'N',
        [PieceType.Lance]: 'L',
        [PieceType.Pawn]: 'P',
        [PieceType.PromotedPawn]: '+P',
        [PieceType.PromotedLance]: '+L',
        [PieceType.PromotedKnight]: '+N',
        [PieceType.PromotedSilver]: '+S',
        [PieceType.PromotedBishop]: '+B',
        [PieceType.PromotedRook]: '+R'
    };
    
    return reversePieceMap[pieceType] || 'P';
}

/**
 * Performance comparison between WebAssembly and JavaScript engines
 */
export async function benchmarkEngines(gameState, difficulty) {
    // Benchmark WebAssembly engine
    const wasmStart = performance.now();
    try {
        const wasmMove = await getWasmAiMove(gameState, difficulty);
        const wasmTime = performance.now() - wasmStart;
        
        return {
            wasm: {
                move: wasmMove,
                time: wasmTime
            }
        };
    } catch (error) {
        console.error('WebAssembly benchmark failed:', error);
        return {
            wasm: {
                move: null,
                time: -1,
                error: error.message
            }
        };
    }
}

/**
 * Get engine statistics
 */
export function getEngineStats() {
    return {
        isInitialized,
        engineType: 'WebAssembly',
        features: [
            'Bitboard representation',
            'Advanced search algorithms',
            'Position evaluation',
            'Move generation',
            'Transposition table',
            'Killer moves',
            'History heuristic'
        ]
    };
}

/**
 * Check if WebAssembly engine is available
 */
export function isWasmEngineAvailable() {
    return isInitialized;
}

/**
 * Get engine version and capabilities
 */
export function getEngineCapabilities() {
    return {
        maxSearchDepth: 20,
        supportsBitboards: true,
        supportsTranspositionTable: true,
        supportsKillerMoves: true,
        supportsHistoryHeuristic: true,
        supportsQuiescenceSearch: true,
        supportsNullMovePruning: true,
        supportsFutilityPruning: true
    };
}

/**
 * Reset the engine state
 */
export function resetEngine() {
    isInitialized = false;
    wasmEngine = null;
}

/**
 * Get detailed performance metrics
 */
export async function getPerformanceMetrics(gameState, difficulty) {
    const startTime = performance.now();
    const startMemory = performance.memory?.usedJSHeapSize || 0;
    
    try {
        const move = await getWasmAiMove(gameState, difficulty);
        const endTime = performance.now();
        const endMemory = performance.memory?.usedJSHeapSize || 0;
        
        return {
            move,
            executionTime: endTime - startTime,
            memoryUsed: endMemory - startMemory,
            engineType: 'WebAssembly',
            difficulty
        };
    } catch (error) {
        return {
            move: null,
            error: error.message,
            executionTime: performance.now() - startTime,
            engineType: 'WebAssembly',
            difficulty
        };
    }
}
