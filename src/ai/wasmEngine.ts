import { ShogiEngine, PieceType as WasmPieceType } from '../../pkg-bundler/shogi_engine.js';
import type { GameState, Piece, PieceType } from '../types';

let isInitialized = false;

/**
 * Initialize the WebAssembly engine
 */
export async function initializeWasmEngine(): Promise<void> {
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
export async function getWasmAiMove(gameState: GameState, difficulty: string | number): Promise<any> {
    if (!isInitialized) {
        await initializeWasmEngine();
    }
    
    try {
        const engine = ShogiEngine.new();
        const engineState = convertGameStateToEngine(gameState);
        setupEnginePosition(engine, engineState);
        const timeLimit = getTimeLimit(difficulty);
        
        let difficultyLevel: number;
        if (typeof difficulty === 'string') {
            switch (difficulty) {
                case 'easy':
                    difficultyLevel = 1;
                    break;
                case 'medium':
                    difficultyLevel = 2;
                    break;
                case 'hard':
                    difficultyLevel = 3;
                    break;
                default:
                    difficultyLevel = 2;
            }
        } else {
            difficultyLevel = difficulty;
        }

        const bestMove = engine.get_best_move(difficultyLevel, timeLimit);
        
        if (bestMove) {
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
function convertGameStateToEngine(gameState: GameState): any {
    try {
        const { board, currentPlayer, capturedPieces, moveHistory } = gameState;
        
        if (!board || !Array.isArray(board) || board.length !== 9) {
            throw new Error('Invalid board structure: board must be a 9x9 array');
        }
        
        const engineBoard: { position: { row: number, col: number }, piece_type: string, player: string }[] = [];
        
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
    } catch (error: any) {
        console.error('Error converting game state:', error);
        console.error('Game state received:', gameState);
        throw new Error(`Failed to convert game state: ${error.message}`);
    }
}

/**
 * Convert piece types from game format to engine format
 */
function convertPieceTypeToEngine(pieceType: PieceType): string {
    const pieceMap: Record<PieceType, string> = {
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
function convertCapturedPieces(capturedPieces: GameState['capturedPieces']): any[] {
    const engineCaptured: { piece_type: string, player: string }[] = [];
    
    if (capturedPieces.player1) {
        capturedPieces.player1.forEach((piece: Piece) => {
            engineCaptured.push({
                piece_type: convertPieceTypeToEngine(piece.type),
                                player: 'Black'
            });
        });
    }
    
    if (capturedPieces.player2) {
        capturedPieces.player2.forEach((piece: Piece) => {
            engineCaptured.push({
                piece_type: convertPieceTypeToEngine(piece.type),
                player: 'White'
            });
        });
    }
    
    return engineCaptured;
}

/**
 * Set up the engine with the current position
 */
function setupEnginePosition(engine: ShogiEngine, engineState: any): void {
    try {
        engine.set_position(JSON.stringify(engineState.board));
        engine.set_current_player(engineState.currentPlayer);
        engine.set_captured_pieces(JSON.stringify(engineState.capturedPieces));
    } catch (error) {
        console.error('Error setting up engine position:', error);
        throw error;
    }
}

/**
 * Get time limit based on difficulty
 */
function getTimeLimit(difficulty: number | string): number {
    switch (difficulty) {
        case 'easy':
        case 1:
            return 1000;
        case 'medium':
        case 2:
            return 3000;
        case 'hard':
        case 3:
            return 9000;
        default:
            return 3000;
    }
}

/**
 * Convert engine move back to game format
 */
function convertEngineMoveToGame(engineMove: any): any {
    try {
        if (engineMove.from === null || engineMove.from === undefined) {
            const toCoord: [number, number] = [8 - engineMove.to.row, engineMove.to.col];
            return {
                from: 'drop',
                to: toCoord,
                type: convertEnginePieceTypeToGame(engineMove.piece_type),
            };
        } else {
                        const fromCoord: [number, number] = [8 - engineMove.from.row, engineMove.from.col];
            const toCoord: [number, number] = [8 - engineMove.to.row, engineMove.to.col];
            return {
                from: fromCoord,
                to: toCoord,
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
function convertEnginePieceTypeToGame(pieceType: WasmPieceType): PieceType {
    const reversePieceMap: { [key in WasmPieceType]: PieceType } = {
        [WasmPieceType.King]: 'K',
        [WasmPieceType.Rook]: 'R',
        [WasmPieceType.Bishop]: 'B',
        [WasmPieceType.Gold]: 'G',
        [WasmPieceType.Silver]: 'S',
        [WasmPieceType.Knight]: 'N',
        [WasmPieceType.Lance]: 'L',
        [WasmPieceType.Pawn]: 'P',
        [WasmPieceType.PromotedPawn]: '+P',
        [WasmPieceType.PromotedLance]: '+L',
        [WasmPieceType.PromotedKnight]: '+N',
        [WasmPieceType.PromotedSilver]: '+S',
        [WasmPieceType.PromotedBishop]: '+B',
        [WasmPieceType.PromotedRook]: '+R'
    };
    
    return reversePieceMap[pieceType] || 'P';
}

/**
 * Get detailed performance metrics
 */
export async function getPerformanceMetrics(gameState: GameState, difficulty: number): Promise<any> {
    const startTime = performance.now();
    const startMemory = (performance as any).memory?.usedJSHeapSize || 0;
    
    try {
        const move = await getWasmAiMove(gameState, difficulty);
        const endTime = performance.now();
        const endMemory = (performance as any).memory?.usedJSHeapSize || 0;
        
        return {
            move,
            executionTime: endTime - startTime,
            memoryUsed: endMemory - startMemory,
            engineType: 'WebAssembly',
            difficulty
        };
    } catch (error: any) {
        return {
            move: null,
            error: error.message,
            executionTime: performance.now() - startTime,
            engineType: 'WebAssembly',
            difficulty
        };
    }
}

/**
 * Check if WebAssembly engine is available
 */
export function isWasmEngineAvailable(): boolean {
    return isInitialized;
}

/**
 * Reset the engine state
 */
export function resetEngine(): void {
    isInitialized = false;
}
