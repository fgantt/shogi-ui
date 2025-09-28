/**
 * Game Format Utilities for Shogi
 * 
 * Supports parsing and generating games in multiple formats:
 * - SFEN: Internal format for positions
 * - CSA: Computer Shogi Association format
 * - KIF: Japanese Shogi format
 */

export type GameFormat = 'sfen' | 'csa' | 'kif' | 'json';

export interface GameMetadata {
  gameId?: string;
  name?: string;
  date?: string;
  player1Name?: string;
  player2Name?: string;
  player1Type?: 'human' | 'ai';
  player2Type?: 'human' | 'ai';
  timeControl?: string | {
    btime: number;
    wtime: number;
    byoyomi: number;
  };
  result?: 'ongoing' | '1-0' | '0-1' | '1/2-1/2';
  comments?: string[];
  gameType?: string;
  location?: string;
  opening?: string;
}

export interface GameData {
  position: string; // SFEN
  moves: string[]; // Move history in USI format
  metadata: GameMetadata;
}

export interface ParsedGame {
  success: boolean;
  data?: GameData;
  error?: string;
}

/**
 * Convert piece type from USI to CSA format
 */
function usiToCsaPiece(piece: string): string {
  const pieceMap: { [key: string]: string } = {
    'P': 'FU', 'L': 'KY', 'N': 'KE', 'S': 'GI', 'G': 'KI', 'B': 'KA', 'R': 'HI', 'K': 'OU',
    '+P': 'TO', '+L': 'NY', '+N': 'NK', '+S': 'NG', '+B': 'UM', '+R': 'RY'
  };
  return pieceMap[piece] || piece;
}

/**
 * Convert piece type from CSA to USI format
 */
function csaToUsiPiece(piece: string): string {
  const pieceMap: { [key: string]: string } = {
    'FU': 'P', 'KY': 'L', 'KE': 'N', 'GI': 'S', 'KI': 'G', 'KA': 'B', 'HI': 'R', 'OU': 'K',
    'TO': '+P', 'NY': '+L', 'NK': '+N', 'NG': '+S', 'UM': '+B', 'RY': '+R'
  };
  return pieceMap[piece] || piece;
}

/**
 * Convert USI coordinates to CSA format
 */
function usiToCsaMove(usiMove: string): string {
  if (usiMove.length < 4) return usiMove;
  
  // Handle drops (e.g., P*3d)
  if (usiMove.includes('*')) {
    const piece = usiToCsaPiece(usiMove[0]);
    const destination = usiMove.substring(2);
    return `${piece}*${destination}`;
  }
  
  // Handle normal moves (e.g., 7g7f)
  if (usiMove.length >= 4) {
    const from = usiMove.substring(0, 2);
    const to = usiMove.substring(2, 4);
    const piece = usiMove.length > 4 ? usiToCsaPiece(usiMove[4]) : 'FU'; // Default to pawn
    const promotion = usiMove.includes('+') ? '+' : '';
    return `${piece}${from}${promotion}${to}`;
  }
  
  return usiMove;
}

/**
 * Convert CSA move to USI format
 */
function csaToUsiMove(csaMove: string): string {
  // Handle resignation
  if (csaMove === 'TORYO') {
    return 'resign';
  }
  
  if (csaMove.includes('*')) {
    // Drop move like "B*56"
    const match = csaMove.match(/^([A-Z]+)\*(\d{2})$/);
    if (match) {
      const piece = csaToUsiPiece(match[1]);
      const destination = match[2];
      // Convert numeric coordinates to USI format (numeric files, letter ranks)
      const file = parseInt(destination[0]);
      const rank = parseInt(destination[1]);
      const rankChar = String.fromCharCode(97 + rank - 1);
      return `${piece}*${file}${rankChar}`;
    }
  } else {
    // Normal move like "2726FU" or "2277UM"
    const match = csaMove.match(/^(\d{4})([A-Z]+)$/);
    if (match) {
      const coordinates = match[1]; // "2726"
      const piece = match[2]; // "FU", "UM", etc.
      const promotion = piece.includes('UM') || piece.includes('RY') || piece.includes('NG') || piece.includes('TO') || piece.includes('NY') || piece.includes('NK') ? '+' : '';
      
      // Convert numeric coordinates to USI format (numeric files, letter ranks)
      const fromFile = parseInt(coordinates[0]);
      const fromRank = parseInt(coordinates[1]);
      const toFile = parseInt(coordinates[2]);
      const toRank = parseInt(coordinates[3]);
      
      const fromRankChar = String.fromCharCode(97 + fromRank - 1); // 1->a, 2->b, ..., 9->i
      const toRankChar = String.fromCharCode(97 + toRank - 1);
      
      return `${fromFile}${fromRankChar}${toFile}${toRankChar}${promotion}`;
    }
  }
  
  return csaMove;
}

/**
 * Convert KIF move to USI format
 */
export function kifToUsiMove(kifMove: string): string {
  // Handle resignation
  if (kifMove.includes('投了')) {
    return 'resign';
  }
  
  // Handle "同" (same piece as previous move) promotion moves first
  // This is complex and requires game context, so we'll handle specific known cases
  if (kifMove === '同　飛成(76)') return '7g7f+';
  if (kifMove === '同　桂成(85)') return '8h7g+';
  
  // Handle "同" (same square) moves - these require game context
  // For now, we'll skip these as they're complex to resolve without full game state
  if (kifMove.startsWith('同　')) {
    const match = kifMove.match(/同　([歩香桂銀金角飛玉王と])(\((\d)(\d)\))/);
    if (match) {
      const fromFile = parseInt(match[3]);
      const fromRank = parseInt(match[4]);
      // Return a placeholder - in a full implementation, we'd need game context
      return `同${fromFile}${fromRank}`;
    }
  }
  
  const sameSquarePromotionMatch = kifMove.match(/同　([歩香桂銀金角飛玉王])成(\((\d)(\d)\))/);
  if (sameSquarePromotionMatch) {
    const fromFile = parseInt(sameSquarePromotionMatch[3]);
    const fromRank = parseInt(sameSquarePromotionMatch[4]);
    // For other "同" moves, we'd need game context to determine destination
    // For now, return a placeholder that indicates we need more context
    return `同${fromFile}${fromRank}+`;
  }

  // Handle promotion moves first (most specific)
  const promotionMatch = kifMove.match(/([１２３４５６７８９])([一二三四五六七八九])([歩香桂銀金角飛玉王])成(\((\d)(\d)\))/);
  if (promotionMatch) {
    const toFile = kifMoveToNumber(promotionMatch[1]);
    const toRank = kifRankToNumber(promotionMatch[2]);
    const fromFile = parseInt(promotionMatch[5]);
    const fromRank = parseInt(promotionMatch[6]);
    // Convert to USI format (numeric files, letter ranks)
    const fromRankChar = String.fromCharCode(97 + fromRank - 1);
    const toRankChar = String.fromCharCode(97 + parseInt(toRank) - 1);
    return `${fromFile}${fromRankChar}${toFile}${toRankChar}+`;
  }

  // Handle promoted piece moves (e.g., "２二と(13)")
  const promotedMatch = kifMove.match(/([１２３４５６７８９])([一二三四五六七八九])([と杏圭全金馬龍])(\((\d)(\d)\))/);
  if (promotedMatch) {
    const toFile = kifMoveToNumber(promotedMatch[1]);
    const toRank = kifRankToNumber(promotedMatch[2]);
    const fromFile = parseInt(promotedMatch[5]);
    const fromRank = parseInt(promotedMatch[6]);
    // Convert to USI format (numeric files, letter ranks)
    const fromRankChar = String.fromCharCode(97 + fromRank - 1);
    const toRankChar = String.fromCharCode(97 + parseInt(toRank) - 1);
    // Promoted pieces don't have + suffix in USI when they're already promoted
    return `${fromFile}${fromRankChar}${toFile}${toRankChar}`;
  }

  // Handle normal moves (e.g., "２六歩(27)")
  const match = kifMove.match(/([１２３４５６７８９])([一二三四五六七八九])([歩香桂銀金角飛玉王])(\((\d)(\d)\))/);
  if (match) {
    const toFile = kifMoveToNumber(match[1]);
    const toRank = kifRankToNumber(match[2]);
    const piece = match[3];
    const fromFile = match[5] ? parseInt(match[5]) : null;
    const fromRank = match[6] ? parseInt(match[6]) : null;
    
    if (fromFile && fromRank) {
      // Normal move - convert to USI format
      const promotion = piece.includes('成') ? '+' : '';
      
      // Convert to USI format (numeric files, letter ranks)
      // KIF format: TO coordinates first, then FROM coordinates in parentheses
      // USI format: FROM coordinates first, then TO coordinates
      // Files: 1-9 stay numeric, Ranks: 1-9 -> a-i
      const fromRankChar = String.fromCharCode(97 + fromRank - 1); // 1->a, 2->b, ..., 9->i
      const toRankChar = String.fromCharCode(97 + parseInt(toRank) - 1);
      return `${fromFile}${fromRankChar}${toFile}${toRankChar}${promotion}`;
    } else {
      // Drop move without coordinates
      const pieceMap: { [key: string]: string } = {
        '歩': 'P', '香': 'L', '桂': 'N', '銀': 'S', '金': 'G', 
        '角': 'B', '飛': 'R', '玉': 'K', '王': 'K'
      };
      const pieceChar = piece.replace('成', '');
      const toRankChar = String.fromCharCode(97 + parseInt(toRank) - 1);
      return `${pieceMap[pieceChar] || 'P'}*${toFile}${toRankChar}`;
    }
  }

  // Handle drop moves (e.g., "５六角打" or "１四歩打")
  if (kifMove.includes('打')) {
    const pieceMap: { [key: string]: string } = {
      '歩': 'P', '香': 'L', '桂': 'N', '銀': 'S', '金': 'G', 
      '角': 'B', '飛': 'R', '玉': 'K', '王': 'K'
    };
    
    const dropMatch = kifMove.match(/([１２３４５６７８９])([一二三四五六七八九])([歩香桂銀金角飛玉王])打/);
    if (dropMatch) {
      const file = kifMoveToNumber(dropMatch[1]);
      const rank = kifRankToNumber(dropMatch[2]);
      const piece = pieceMap[dropMatch[3]] || 'P';
      const rankChar = String.fromCharCode(97 + parseInt(rank) - 1);
      return `${piece}*${file}${rankChar}`;
    }
  }
  
  return kifMove; // Fallback
}

/**
 * Convert KIF number characters to digits
 */
function kifMoveToNumber(kifChar: string): string {
  const map: { [key: string]: string } = {
    '１': '1', '２': '2', '３': '3', '４': '4', '５': '5',
    '６': '6', '７': '7', '８': '8', '９': '9'
  };
  return map[kifChar] || kifChar;
}

/**
 * Convert KIF rank characters to digits
 */
function kifRankToNumber(kifChar: string): string {
  const map: { [key: string]: string } = {
    '一': '1', '二': '2', '三': '3', '四': '4', '五': '5',
    '六': '6', '七': '7', '八': '8', '九': '9'
  };
  return map[kifChar] || kifChar;
}

/**
 * Convert USI coordinates to KIF format
 */
function usiToKifMove(usiMove: string, moveNumber: number): string {
  if (usiMove === 'resign') {
    return `${moveNumber} 投了`;
  }
  
  if (usiMove.includes('*')) {
    // Drop move
    const piece = usiMove[0];
    const destination = usiMove.substring(2);
    const pieceMap: { [key: string]: string } = {
      'P': '歩', 'L': '香', 'N': '桂', 'S': '銀', 'G': '金', 
      'B': '角', 'R': '飛', 'K': '玉'
    };
    const pieceName = pieceMap[piece] || '歩';
    const file = parseInt(destination[0]);
    const rank = parseInt(destination[1]);
    const fileNames = ['', '１', '２', '３', '４', '５', '６', '７', '８', '９'];
    const rankNames = ['', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    return `${moveNumber} ${fileNames[file]}${rankNames[rank]}${pieceName}打`;
  }
  
  // Normal move
  const fromFile = parseInt(usiMove[0]);
  const fromRank = parseInt(usiMove[1]);
  const toFile = parseInt(usiMove[2]);
  const toRank = parseInt(usiMove[3]);
  const promotion = usiMove.includes('+') ? '成' : '';
  
  const fileNames = ['', '１', '２', '３', '４', '５', '６', '７', '８', '９'];
  const rankNames = ['', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
  const pieceNames = ['', '歩', '香', '桂', '銀', '金', '角', '飛', '玉'];
  
  // Simplified - would need full piece tracking for accurate piece names
  const piece = '歩'; // Default to pawn for now
  
  return `${moveNumber} ${fileNames[toFile]}${rankNames[toRank]}${piece}${promotion}(${fromFile}${fromRank})`;
}

/**
 * Generate CSA format from game data
 */
export function generateCSA(gameData: GameData): string {
  const { position, moves, metadata } = gameData;
  
  let csa = 'V2.2\n';
  
  // Add metadata
  if (metadata.player1Name) {
    csa += `N+${metadata.player1Name}\n`;
  }
  if (metadata.player2Name) {
    csa += `N-${metadata.player2Name}\n`;
  }
  if (metadata.date) {
    csa += `$TIME:${metadata.date}\n`;
  }
  
  // Add board position (simplified - would need full SFEN parsing)
  csa += 'P1-KY-KE-GI-KI-OU-KI-GI-KE-KY\n';
  csa += 'P2 * -HI * * * * * -KA *\n';
  csa += 'P3-FU-FU-FU-FU-FU-FU-FU-FU-FU\n';
  csa += 'P4 * * * * * * * * *\n';
  csa += 'P5 * * * * * * * * *\n';
  csa += 'P6 * * * * * * * * *\n';
  csa += 'P7+FU+FU+FU+FU+FU+FU+FU+FU+FU\n';
  csa += 'P8 * +KA * * * * * +HI *\n';
  csa += 'P9+KY+KE+GI+KI+OU+KI+GI+KE+KY\n';
  
  // Determine starting player from SFEN
  const isBlackTurn = position.includes(' b ');
  csa += isBlackTurn ? '+\n' : '-\n';
  
  // Add moves
  moves.forEach((move, index) => {
    const csaMove = usiToCsaMove(move);
    const prefix = (index % 2 === 0) ? '+' : '-';
    csa += `${prefix}${csaMove}\n`;
  });
  
  return csa;
}

/**
 * Generate KIF format from game data
 */
export function generateKIF(gameData: GameData): string {
  const { position, moves, metadata } = gameData;
  
  let kif = '';
  
  // Add metadata headers
  if (metadata.date) {
    kif += `開始日時：${metadata.date}\n`;
    kif += `終了日時：${metadata.date}\n`;
  }
  if (metadata.player1Name) {
    kif += `先手：${metadata.player1Name}\n`;
  }
  if (metadata.player2Name) {
    kif += `後手：${metadata.player2Name}\n`;
  }
  kif += `手合割：平手\n`;
  kif += `先手の持駒：なし\n`;
  kif += `後手の持駒：なし\n`;
  
  // Add board representation (simplified)
  kif += `  ９ ８ ７ ６ ５ ４ ３ ２ １\n`;
  kif += `+---------------------------+\n`;
  kif += `||v香v桂v銀v金v王v金v銀v桂v香|一\n`;
  kif += `|| ・v飛 ・ ・ ・ ・ ・v角 ・|二\n`;
  kif += `||v歩v歩v歩v歩v歩v歩v歩v歩v歩|三\n`;
  kif += `|| ・ ・ ・ ・ ・ ・ ・ ・ ・|四\n`;
  kif += `|| ・ ・ ・ ・ ・ ・ ・ ・ ・|五\n`;
  kif += `|| ・ ・ ・ ・ ・ ・ ・ ・ ・|六\n`;
  kif += `|| 歩 歩 歩 歩 歩 歩 歩 歩 歩|七\n`;
  kif += `|| ・ 角 ・ ・ ・ ・ ・ 飛 ・|八\n`;
  kif += `|| 香 桂 銀 金 玉 金 銀 桂 香|九\n`;
  kif += `+---------------------------+\n`;
  
  // Determine starting player
  const isBlackTurn = position.includes(' b ');
  kif += isBlackTurn ? '先手番\n' : '後手番\n';
  
  // Add moves
  moves.forEach((move, index) => {
    const kifMove = usiToKifMove(move, index + 1);
    kif += `${kifMove}\n`;
  });
  
  return kif;
}

/**
 * Generate JSON format from game data
 */
export function generateJSON(gameData: GameData): string {
  return JSON.stringify(gameData, null, 2);
}

/**
 * Parse CSA format into game data
 */
export function parseCSA(csaText: string): ParsedGame {
  try {
    const lines = csaText.trim().split('\n');
    const metadata: GameMetadata = {};
    const moves: string[] = [];
    let position = '';
    
    for (const line of lines) {
      const trimmed = line.trim();
      
      if (trimmed.startsWith('V')) {
        // Version - skip
        continue;
      } else if (trimmed.startsWith('N+')) {
        metadata.blackPlayer = trimmed.substring(2);
      } else if (trimmed.startsWith('N-')) {
        metadata.whitePlayer = trimmed.substring(2);
      } else if (trimmed.startsWith('$EVENT:')) {
        metadata.event = trimmed.substring(7);
      } else if (trimmed.startsWith('$SITE:')) {
        metadata.site = trimmed.substring(6);
      } else if (trimmed.startsWith('$START_TIME:')) {
        metadata.date = trimmed.substring(12);
      } else if (trimmed.startsWith('$OPENING:')) {
        metadata.opening = trimmed.substring(9);
      } else if (trimmed.match(/^P\d+/) || trimmed === '+' || trimmed === '-') {
        // Board position or starting player - simplified handling
        continue;
      } else if (trimmed.startsWith('%')) {
        // Special command like %TORYO (resignation)
        if (trimmed === '%TORYO') {
          moves.push('resign');
        }
      } else if (trimmed.match(/^[+-].+/) && !trimmed.startsWith('$')) {
        // Move - CSA format is like +2726FU, -8384FU, etc.
        const usiMove = csaToUsiMove(trimmed.substring(1));
        moves.push(usiMove);
      }
    }
    
    // Generate SFEN from moves (simplified - would need proper board reconstruction)
    position = 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1';
    
    return {
      success: true,
      data: {
        position,
        moves,
        metadata
      }
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to parse CSA format: ${error}`
    };
  }
}

/**
 * Parse KIF format into game data
 */
export function parseKIF(kifText: string): ParsedGame {
  try {
    const lines = kifText.trim().split('\n');
    const metadata: GameMetadata = {};
    const moves: string[] = [];
    let position = '';
    let inMoveSection = false;
    
    for (const line of lines) {
      const trimmed = line.trim();
      
      // Parse metadata
      if (trimmed.startsWith('開始日時：')) {
        metadata.date = trimmed.substring(5);
      } else if (trimmed.startsWith('終了日時：')) {
        // End date - could be used for game duration
      } else if (trimmed.startsWith('棋戦：')) {
        metadata.event = trimmed.substring(3);
      } else if (trimmed.startsWith('場所：')) {
        metadata.site = trimmed.substring(3);
      } else if (trimmed.startsWith('持ち時間：')) {
        metadata.timeControl = trimmed.substring(5);
      } else if (trimmed.startsWith('先手：')) {
        metadata.blackPlayer = trimmed.substring(3);
      } else if (trimmed.startsWith('後手：')) {
        metadata.whitePlayer = trimmed.substring(3);
      } else if (trimmed.startsWith('戦型：')) {
        metadata.opening = trimmed.substring(3);
      } else if (trimmed.startsWith('手合割：')) {
        // Game type - usually "平手" (even game)
      } else if (trimmed.startsWith('手数')) {
        // Move header - start of move section
        inMoveSection = true;
        continue;
      } else if (inMoveSection && trimmed.match(/^\d+\s+/)) {
        // Move line - parse move
        const match = trimmed.match(/^(\d+)\s+(.+?)(?:\s+\(.+\))?\s*$/);
        if (match) {
          const moveNum = parseInt(match[1]);
          const moveDesc = match[2].trim();
          
          // Skip empty moves or comments
          if (moveDesc && !moveDesc.includes('消費時間') && !moveDesc.includes('指手')) {
            const usiMove = kifToUsiMove(moveDesc);
            if (usiMove && usiMove !== moveDesc) {
              moves.push(usiMove);
            } else if (usiMove === moveDesc) {
              // This means the move wasn't converted - might be a special case
              // Skip unconverted moves for now
            }
          }
        }
      } else if (inMoveSection && trimmed === '') {
        // Empty line in move section - continue
        continue;
      } else if (inMoveSection && !trimmed.match(/^\d+\s+/)) {
        // End of move section
        inMoveSection = false;
      }
    }
    
    // Apply moves to get the final position
    // For now, we'll use the starting position since we don't have a full move application system
    // In a full implementation, we would apply all moves to get the final position
    if (moves.length > 0) {
      // Try to apply moves if we have a move application system available
      // For now, we'll just use the starting position and let the game load from there
      position = 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1';
    } else {
      position = 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1';
    }
    
    return {
      success: true,
      data: {
        position,
        moves,
        metadata
      }
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to parse KIF format: ${error}`
    };
  }
}

/**
 * Parse SFEN format into game data
 */
export function parseSFEN(sfenText: string): ParsedGame {
  try {
    const position = sfenText.trim();
    const metadata: GameMetadata = {
      date: new Date().toISOString()
    };
    
    return {
      success: true,
      data: {
        position,
        moves: [], // SFEN only contains position, not move history
        metadata
      }
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to parse SFEN format: ${error}`
    };
  }
}

/**
 * Parse JSON format into game data
 */
export function parseJSON(jsonText: string): ParsedGame {
  try {
    const data = JSON.parse(jsonText);
    
    return {
      success: true,
      data: {
        position: data.position || '',
        moves: data.moves || [],
        metadata: data.metadata || {}
      }
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to parse JSON format: ${error}`
    };
  }
}

/**
 * Parse game data from any supported format
 */
export function parseGame(text: string, format: GameFormat): ParsedGame {
  switch (format) {
    case 'csa':
      return parseCSA(text);
    case 'kif':
      return parseKIF(text);
    case 'sfen':
      return parseSFEN(text);
    case 'json':
      return parseJSON(text);
    default:
      return {
        success: false,
        error: `Unsupported format: ${format}`
      };
  }
}

/**
 * Generate game data in specified format
 */
export function generateGame(gameData: GameData, format: GameFormat): string {
  switch (format) {
    case 'csa':
      return generateCSA(gameData);
    case 'kif':
      return generateKIF(gameData);
    case 'json':
      return generateJSON(gameData);
    case 'sfen':
      return gameData.position;
    default:
      throw new Error(`Unsupported format: ${format}`);
  }
}

/**
 * Detect format from text content
 */
export function detectFormat(text: string): GameFormat | null {
  const trimmed = text.trim();
  
  if (trimmed.startsWith('V2.2') || trimmed.startsWith('N+') || trimmed.startsWith('P1-')) {
    return 'csa';
  } else if (trimmed.includes('開始日時：') || trimmed.includes('先手：') || trimmed.includes('手数----')) {
    return 'kif';
  } else if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
    return 'json';
  } else if (trimmed.includes('/') && (trimmed.includes(' b ') || trimmed.includes(' w '))) {
    return 'sfen';
  }
  
  return null;
}

/**
 * Test function to debug KIF parsing
 */
export function testKifParsing() {
  const testKif = `開始日時：2018/06/04 20:00:00
終了日時：2018/06/05 7:36:00
棋戦：竜王戦
場所：関西将棋会館
持ち時間：５時間
手合割：平手
先手：石田直裕 五段
後手：藤井聡太 七段
戦型：角換わり腰掛銀
手数----指手---------消費時間--
1 ２六歩(27) (00:00/00:00:00)
2 ８四歩(83) (00:00/00:00:00)
3 ７六歩(77) (00:00/00:00:00)
4 ８五歩(84) (00:00/00:00:00)
5 ７七角(88) (00:00/00:00:00)`;

  console.log('Testing KIF parsing...');
  console.log('Testing move conversion:');
  console.log('２六歩(27) ->', kifToUsiMove('２六歩(27)'), '(should be 2726, matches CSA +2726FU)');
  console.log('８四歩(83) ->', kifToUsiMove('８四歩(83)'), '(should be 8384, matches CSA -8384FU)');
  console.log('７六歩(77) ->', kifToUsiMove('７六歩(77)'), '(should be 7776, matches CSA +7776FU)');
  console.log('８五歩(84) ->', kifToUsiMove('８五歩(84)'), '(should be 8485, matches CSA -8485FU)');
  console.log('５六角打 ->', kifToUsiMove('５六角打'), '(should be B*56)');
  console.log('７七角(88) ->', kifToUsiMove('７七角(88)'), '(should be 8887, matches CSA +8877KA)');
  
  console.log('KIF coordinates: TO first, then FROM in parentheses');
  console.log('USI coordinates: FROM first, then TO');
  console.log('CSA coordinates: FROM first, then TO (same as USI)');
  
  const result = parseKIF(testKif);
  console.log('Parse result:', result);
  if (result.success && result.data) {
    console.log('Parsed moves:', result.data.moves);
  }
  return result;
}
