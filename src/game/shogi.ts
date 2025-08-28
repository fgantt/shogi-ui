
import { 
    importKIF, 
    importKI2, 
    importCSA, 
    importJKF, 
    exportKIF, 
    exportKI2, 
    exportCSA, 
    exportJKFString,
    detectRecordFormat,
    RecordFormatType,
    Record,
    Position,
    Color,
    Piece as TsshogiPiece,
    Move as TsshogiMove,
    Square,
    PieceType as TsshogiPieceType
} from 'tsshogi';
import { GameState, Move, Piece, Player, PieceType } from '../types';

// Map our piece types to tsshogi piece types
const pieceTypeMap: Record<PieceType, TsshogiPieceType> = {
    'K': TsshogiPieceType.KING,
    'R': TsshogiPieceType.ROOK,
    'B': TsshogiPieceType.BISHOP,
    'G': TsshogiPieceType.GOLD,
    'S': TsshogiPieceType.SILVER,
    'N': TsshogiPieceType.KNIGHT,
    'L': TsshogiPieceType.LANCE,
    'P': TsshogiPieceType.PAWN,
    '+R': TsshogiPieceType.DRAGON,
    '+B': TsshogiPieceType.HORSE,
    '+S': TsshogiPieceType.PROM_SILVER,
    '+N': TsshogiPieceType.PROM_KNIGHT,
    '+L': TsshogiPieceType.PROM_LANCE,
    '+P': TsshogiPieceType.PROM_PAWN,
};

// Map tsshogi piece types to our piece types
const reversePieceTypeMap: Record<TsshogiPieceType, PieceType> = {
    [TsshogiPieceType.KING]: 'K',
    [TsshogiPieceType.ROOK]: 'R',
    [TsshogiPieceType.BISHOP]: 'B',
    [TsshogiPieceType.GOLD]: 'G',
    [TsshogiPieceType.SILVER]: 'S',
    [TsshogiPieceType.KNIGHT]: 'N',
    [TsshogiPieceType.LANCE]: 'L',
    [TsshogiPieceType.PAWN]: 'P',
    [TsshogiPieceType.DRAGON]: '+R',
    [TsshogiPieceType.HORSE]: '+B',
    [TsshogiPieceType.PROM_SILVER]: '+S',
    [TsshogiPieceType.PROM_KNIGHT]: '+N',
    [TsshogiPieceType.PROM_LANCE]: '+L',
    [TsshogiPieceType.PROM_PAWN]: '+P',
};

// Convert tsshogi piece to our internal piece
function toOurPiece(piece: any): Piece {
    return {
        type: reversePieceTypeMap[piece.type],
        // tsshogi's color representation: Color.BLACK = Black pieces, Color.WHITE = White pieces
        player: piece.color === Color.BLACK ? 'player1' : 'player2',
    };
}

// Convert our internal piece to tsshogi piece
function fromOurPiece(piece: Piece): any {
    return new TsshogiPiece(
        // tsshogi's color representation: Color.BLACK = Black pieces, Color.WHITE = White pieces
        // player1 (Black) should map to Color.BLACK in tsshogi
        piece.player === 'player1' ? Color.BLACK : Color.WHITE,
        pieceTypeMap[piece.type]
    );
}

// Convert tsshogi move to our internal move
function toOurMove(tsshogiMove: any, player: Player): Move {
    // Handle drops (when from is a PieceType)
    if (typeof tsshogiMove.from === 'string') {
        return {
            from: 'drop',
            to: [tsshogiMove.to.y, tsshogiMove.to.x] as [number, number],
            piece: reversePieceTypeMap[tsshogiMove.pieceType],
            player: player,
            promote: tsshogiMove.promote,
            timestamp: new Date().toISOString(),
        };
    } else {
        // Handle normal moves
        return {
            from: [tsshogiMove.from.y, tsshogiMove.from.x] as [number, number],
            to: [tsshogiMove.to.y, tsshogiMove.to.x] as [number, number],
            piece: reversePieceTypeMap[tsshogiMove.pieceType],
            player: player,
            promote: tsshogiMove.promote,
            timestamp: new Date().toISOString(),
        };
    }
}

// Convert our coordinates to tsshogi Square
function toTsshogiSquare(coords: [number, number]): Square {
    // internal: [row, col] where row 0=1段目, col 0=9筋 (White on top, Black on bottom)
    // tsshogi: file 0=9筋, rank 0=1段目
    // From board parsing: row = rank - 1, col = 9 - file
    // So: file = 9 - col, rank = row + 1
    return new Square(9 - coords[1], coords[0] + 1);
}

// Supported formats and their extensions
export const SUPPORTED_FORMATS = {
    kif: { name: 'KIF', extension: '.kif', mimeType: 'text/plain' },
    ki2: { name: 'KI2', extension: '.ki2', mimeType: 'text/plain' },
    csa: { name: 'CSA', extension: '.csa', mimeType: 'text/plain' },
    jkf: { name: 'JKF', extension: '.jkf', mimeType: 'application/json' },
    sfen: { name: 'SFEN', extension: '.sfen', mimeType: 'text/plain' }
} as const;

export type SupportedFormat = keyof typeof SUPPORTED_FORMATS;

// Detect format from file content
export function detectFormat(fileContent: string): SupportedFormat {
    const detected = detectRecordFormat(fileContent);
    
    switch (detected) {
        case RecordFormatType.KIF:
            return 'kif';
        case RecordFormatType.KI2:
            return 'ki2';
        case RecordFormatType.CSA:
            return 'csa';
        case RecordFormatType.JKF:
            return 'jkf';
        case RecordFormatType.SFEN:
        case RecordFormatType.USI:
            return 'sfen';
        default:
            // Default to KIF if we can't determine
            return 'kif';
    }
}

// Detect format from filename extension
export function detectFormatFromExtension(filename: string): SupportedFormat | null {
    const extension = filename.toLowerCase().split('.').pop();
    if (!extension) return null;
    
    switch (extension) {
        case 'kif':
        case 'kifu':
            return 'kif';
        case 'ki2':
            return 'ki2';
        case 'csa':
            return 'csa';
        case 'jkf':
            return 'jkf';
        case 'sfen':
        case 'usi':
            return 'sfen';
        default:
            return null;
    }
}

export function parseShogiFile(fileContent: string, format?: SupportedFormat): GameState {
    // Auto-detect format if not provided
    if (!format) {
        format = detectFormat(fileContent);
    }
    
    console.log("Parsing file with format:", format);
    
    try {
        let record: any;
        
        switch (format) {
            case 'kif':
                const kifResult = importKIF(fileContent);
                if (kifResult instanceof Error) {
                    throw kifResult;
                }
                record = kifResult;
                break;
            case 'ki2':
                const ki2Result = importKI2(fileContent);
                if (ki2Result instanceof Error) {
                    throw ki2Result;
                }
                record = ki2Result;
                break;
            case 'csa':
                const csaResult = importCSA(fileContent);
                if (csaResult instanceof Error) {
                    throw csaResult;
                }
                record = csaResult;
                break;
            case 'jkf':
                // For JKF, we need to parse the JSON first
                try {
                    const jkfData = JSON.parse(fileContent);
                    const jkfResult = importJKF(jkfData);
                    if (jkfResult instanceof Error) {
                        throw jkfResult;
                    }
                    record = jkfResult;
                } catch (parseError) {
                    throw new Error('Invalid JKF JSON format');
                }
                break;
            case 'sfen':
                // For SFEN, we need to create a record from the position
                const position = Position.newBySFEN(fileContent);
                if (!position) {
                    throw new Error('Invalid SFEN string');
                }
                record = new Record(position);
                break;
            default:
                throw new Error(`Unsupported format: ${format}`);
        }
        
        // Get the current position
        const position = record.position;
        
        // console.log("=== DEBUG: Initial Position ===");
        // console.log("Initial SFEN:", position.sfen);
        // console.log("Total moves in record:", record.moves.length);
        
        // Apply all moves to get to the final position
        let moveNode = record.first.next;
        let moveCount = 0;
        while (moveNode && moveNode.move) {
            // console.log(`Applying move ${moveCount + 1}:`, moveNode.move);
            // Apply this move to the position
            (position as any).doMove(moveNode.move);
            moveCount++;
            moveNode = moveNode.next;
        }
        
        // console.log(`Applied ${moveCount} moves`);
        
        // Now get the final board state after all moves
        const finalPosition = position;
        
        // Convert board - now with correct coordinate mapping
        const board: (Piece | null)[][] = Array(9).fill(null).map(() => Array(9).fill(null));
        
        // console.log("=== DEBUG: Board Parsing ===");
        // console.log("Final Position SFEN:", finalPosition.sfen);
        
        // Get all pieces from tsshogi and map them correctly
        // console.log("=== DEBUG: Coordinate Mapping ===");
        // console.log("Current SFEN:", finalPosition.sfen);
        
        // Calculate king positions
        const kingPositions = {
            player1: [-1, -1] as [number, number],
            player2: [-1, -1] as [number, number],
        };
        
        // Convert board
        finalPosition.board.listNonEmptySquares().forEach((square: Square) => { 
            const piece = finalPosition.board.at(square);
            board[square.rank - 1][9 - square.file] = toOurPiece(piece);
            if (piece.type === TsshogiPieceType.KING) {
                if (piece.color === Color.BLACK) {
                    kingPositions.player1 = [square.rank - 1, 9 - square.file];
                } else {
                    kingPositions.player2 = [square.rank - 1, 9 - square.file];
                }
            }
        });

        // Let me also check what pieces are in the hands
        // console.log("=== DEBUG: Hand Pieces ===");
        // console.log("Black hand pieces:", finalPosition.blackHand);
        // console.log("White hand pieces:", finalPosition.whiteHand);
        
        // Convert captured pieces
        const capturedPieces = {
            player1: [] as Piece[],
            player2: [] as Piece[]
        };
        
        // Iterate through hand pieces
        // console.log("=== DEBUG: Processing Black Hand ===");
        for (const [key, value] of finalPosition.blackHand.pieces.entries()) {
            for (let i = 0; i < value; i++) {
              capturedPieces.player1.push({
                type: reversePieceTypeMap[key],
                player: 'player1' as Player
              });
            }
        }
        
        // console.log("=== DEBUG: Processing White Hand ===");
        for (const [key, value] of finalPosition.whiteHand.pieces.entries()) {
            for (let i = 0; i < value; i++) {
              capturedPieces.player2.push({
                type: reversePieceTypeMap[key],
                player: 'player2' as Player
              });
            }
        }
        
        // console.log("=== DEBUG: Final Captured Pieces ===");
        // console.log("player1:", capturedPieces.player1);
        // console.log("player2:", capturedPieces.player2);
        
        // Convert move history
        const moves: Move[] = [];
        let currentNode = record.first.next;
        
        while (currentNode) {
            // Check if this node has a move
            if (currentNode.move && typeof currentNode.move === 'object') {
                const move = currentNode.move as any;
                const player: Player = currentNode.nextColor === Color.WHITE ? 'player1' : 'player2';
                
                const ourMove = toOurMove(move, player);
                moves.push(ourMove);
            }
            
            currentNode = currentNode.next;
        }
        
        
        return {
            board,
            currentPlayer: finalPosition.color === Color.WHITE ? 'player2' : 'player1',
            capturedPieces,
            moveHistory: moves,
            isCheck: finalPosition.checked,
            isCheckmate: false, // TODO: Implement proper checkmate detection
            isDraw: false, // TODO: Implement proper draw detection
            promotionPending: null,
            pastStates: [],
            kingPositions,
        };
        
    } catch (error) {
        console.error('Error parsing shogi file:', error);
        throw new Error(`Failed to parse ${format} file: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

export function generateShogiFile(gameState: GameState, format: SupportedFormat = 'kif'): string {
    try {
        // Create a new record with standard initial position
        const record = new Record();
        
        // Set up the initial position to standard
        const position = record.position;
        //(position as any).reset(0); // 0 = InitialPositionType.STANDARD
        
        // Clear the board first
        (position.board as any).clear();
        
        // Set up the board
        for (let r = 0; r < 9; r++) {
            for (let c = 0; c < 9; c++) {
                const piece = gameState.board[r][c];
                if (piece) {
                    const square = toTsshogiSquare([r, c]);
                    (position.board as any).set(square, fromOurPiece(piece));
                }
            }
        }
        
        // Set up captured pieces
        gameState.capturedPieces.player1.forEach((p) => {
            const key = pieceTypeMap[p.type];
            const current = position.blackHand.pieces.get(key) ?? 0;
            position.blackHand.pieces.set(key, current + 1);
          });
          
        for (const piece of gameState.capturedPieces.player2) {
            (position.whiteHand as any).add(pieceTypeMap[piece.type], 1);
        }
        
        // Set the current player
        (position as any).color = gameState.currentPlayer === 'player1' ? Color.BLACK : Color.WHITE;
        
        // Add move history
        for (const move of gameState.moveHistory) {
            if (move.from !== 'drop') {
                const fromSquare = toTsshogiSquare(move.from);
                const toSquare = toTsshogiSquare(move.to);
                const tsshogiMove = new TsshogiMove(
                    fromSquare,
                    toSquare,
                    move.promote || false,
                    move.player === 'player1' ? Color.BLACK : Color.WHITE,
                    pieceTypeMap[move.piece],
                    null // captured piece type
                );
                record.append(tsshogiMove);
            } else {
                // Handle drops
                const toSquare = toTsshogiSquare(move.to);
                const tsshogiMove = new TsshogiMove(
                    pieceTypeMap[move.piece],
                    toSquare,
                    move.promote || false,
                    move.player === 'player1' ? Color.BLACK : Color.WHITE,
                    pieceTypeMap[move.piece],
                    null // captured piece type
                );
                record.append(tsshogiMove);
            }
        }
        
        // Export in the requested format
        switch (format) {
            case 'kif':
                return exportKIF(record);
            case 'ki2':
                return exportKI2(record);
            case 'csa':
                return exportCSA(record);
            case 'jkf':
                return exportJKFString(record);
            case 'sfen':
                return record.position.sfen;
            default:
                throw new Error(`Unsupported export format: ${format}`);
        }
        
    } catch (error) {
        console.error('Error generating shogi file:', error);
        throw new Error(`Failed to generate ${format} file: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}
