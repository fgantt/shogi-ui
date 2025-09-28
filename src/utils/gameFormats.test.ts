import { describe, it, expect } from 'vitest';
import { 
  parseKIF, 
  parseCSA, 
  generateKIF, 
  generateCSA,
  kifToUsiMove,
  detectFormat 
} from './gameFormats';

// Test data from Shogi DB2 - real game between Ishida and Fujii
const testKIF = `開始日時：2018/06/04 20:00:00
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
5 ７七角(88) (00:00/00:00:00)
6 ３四歩(33) (00:00/00:00:00)
7 ６八銀(79) (00:00/00:00:00)
8 ３二金(41) (00:00/00:00:00)
9 ７八金(69) (00:00/00:00:00)
10 ７七角成(22) (00:00/00:00:00)
11 同　銀(68) (00:00/00:00:00)
12 ２二銀(31) (00:00/00:00:00)
13 ３八銀(39) (00:00/00:00:00)
14 ３三銀(22) (00:00/00:00:00)
15 ４六歩(47) (00:00/00:00:00)
16 ７四歩(73) (00:00/00:00:00)
17 ４七銀(38) (00:00/00:00:00)
18 ７二銀(71) (00:00/00:00:00)
19 ３六歩(37) (00:00/00:00:00)
20 ４二玉(51) (00:00/00:00:00)
21 ９六歩(97) (00:00/00:00:00)
22 １四歩(13) (00:00/00:00:00)
23 １六歩(17) (00:00/00:00:00)
24 ９四歩(93) (00:00/00:00:00)
25 ５六銀(47) (00:00/00:00:00)
26 ６四歩(63) (00:00/00:00:00)
27 ４八金(49) (00:00/00:00:00)
28 ７三桂(81) (00:00/00:00:00)
29 ６六歩(67) (00:00/00:00:00)
30 ６三銀(72) (00:00/00:00:00)
31 ３七桂(29) (00:00/00:00:00)
32 ８一飛(82) (00:00/00:00:00)
33 ２九飛(28) (00:00/00:00:00)
34 ６二金(61) (00:00/00:00:00)
35 ６八玉(59) (00:00/00:00:00)
36 ５四銀(63) (00:00/00:00:00)
37 ７九玉(68) (00:00/00:00:00)
38 ３一玉(42) (00:00/00:00:00)
39 ４五銀(56) (00:00/00:00:00)
40 ６三銀(54) (00:00/00:00:00)
41 ２五桂(37) (00:00/00:00:00)
42 ２二銀(33) (00:00/00:00:00)
43 ３四銀(45) (00:00/00:00:00)
44 ４二玉(31) (00:00/00:00:00)
45 ５六角打 (00:00/00:00:00)
46 ２四歩(23) (00:00/00:00:00)
47 １三桂成(25) (00:00/00:00:00)
48 同　桂(21) (00:00/00:00:00)
49 １五歩(16) (00:00/00:00:00)
50 同　歩(14) (00:00/00:00:00)
51 １四歩打 (00:00/00:00:00)
52 ６五歩(64) (00:00/00:00:00)
53 同　歩(66) (00:00/00:00:00)
54 ８六歩(85) (00:00/00:00:00)
55 同　歩(87) (00:00/00:00:00)
56 ７五歩(74) (00:00/00:00:00)
57 １五香(19) (00:00/00:00:00)
58 ３三歩打 (00:00/00:00:00)
59 ４三銀成(34) (00:00/00:00:00)
60 同　金(32) (00:00/00:00:00)
61 １三歩成(14) (00:00/00:00:00)
62 ７六歩(75) (00:00/00:00:00)
63 同　銀(77) (00:00/00:00:00)
64 ４七歩打 (00:00/00:00:00)
65 ６四歩(65) (00:00/00:00:00)
66 同　銀(63) (00:00/00:00:00)
67 ６三歩打 (00:00/00:00:00)
68 同　金(62) (00:00/00:00:00)
69 ２二と(13) (00:00/00:00:00)
70 ４八歩成(47) (00:00/00:00:00)
71 ７二銀打 (00:00/00:00:00)
72 ８六飛(81) (00:00/00:00:00)
73 ８七歩打 (00:00/00:00:00)
74 ７六飛(86) (00:00/00:00:00)
75 ７七歩打 (00:00/00:00:00)
76 同　飛成(76) (00:00/00:00:00)
77 同　金(78) (00:00/00:00:00)
78 ８五桂(73) (00:00/00:00:00)
79 ７六金(77) (00:00/00:00:00)
80 ７八歩打 (00:00/00:00:00)
81 同　玉(79) (00:00/00:00:00)
82 ７七歩打 (00:00/00:00:00)
83 ８八玉(78) (00:00/00:00:00)
84 ７八銀打 (00:00/00:00:00)
85 同　角(56) (00:00/00:00:00)
86 同　歩成(77) (00:00/00:00:00)
87 同　玉(88) (00:00/00:00:00)
88 ８六桂打 (00:00/00:00:00)
89 同　金(76) (00:00/00:00:00)
90 ７七銀打 (00:00/00:00:00)
91 同　桂(89) (00:00/00:00:00)
92 同　桂成(85) (00:00/00:00:00)
93 同　玉(78) (00:00/00:00:00)
94 ６六角打 (00:00/00:00:00)
95 ６七玉(77) (00:00/00:00:00)
96 ７七金打 (00:00/00:00:00)
97 投了 (00:00/00:00:00)`;

const testCSA = `V2.2
N+石田直裕 五段
N-藤井聡太 七段
$EVENT:竜王戦
$SITE:関西将棋会館
$START_TIME:2018/06/04 20:00:00
$END_TIME:2018/06/05 7:36:00
$OPENING:角換わり腰掛銀
P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
P2 * -HI *  *  *  *  * -KA * 
P3-FU-FU-FU-FU-FU-FU-FU-FU-FU
P4 *  *  *  *  *  *  *  *  * 
P5 *  *  *  *  *  *  *  *  * 
P6 *  *  *  *  *  *  *  *  * 
P7+FU+FU+FU+FU+FU+FU+FU+FU+FU
P8 * +KA *  *  *  *  * +HI * 
P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
+
+2726FU
-8384FU
+7776FU
-8485FU
+8877KA
-3334FU
+7968GI
-4132KI
+6978KI
-2277UM
+6877GI
-3122GI
+3938GI
-2233GI
+4746FU
-7374FU
+3847GI
-7172GI
+3736FU
-5142OU
+9796FU
-1314FU
+1716FU
-9394FU
+4756GI
-6364FU
+4948KI
-8173KE
+6766FU
-7263GI
+2937KE
-8281HI
+2829HI
-6162KI
+5968OU
-6354GI
+6879OU
-4231OU
+5645GI
-5463GI
+3725KE
-3322GI
+4534GI
-3142OU
+0056KA
-2324FU
+2513NK
-2113KE
+1615FU
-1415FU
+0014FU
-6465FU
+6665FU
-8586FU
+8786FU
-7475FU
+1915KY
-0033FU
+3443NG
-3243KI
+1413TO
-7576FU
+7776GI
-0047FU
+6564FU
-6364GI
+0063FU
-6263KI
+1322TO
-4748TO
+0072GI
-8186HI
+0087FU
-8676HI
+0077FU
-7677RY
+7877KI
-7385KE
+7776KI
-0078FU
+7978OU
-0077FU
+7888OU
-0078GI
+5678KA
-7778TO
+8878OU
-0086KE
+7686KI
-0077GI
+8977KE
-8577NK
+7877OU
-0066KA
+7767OU
-0077KI
%TORYO`;

const expectedFinalSFEN = 'l7l/2S2k1+P1/3gpgp2/p2s3p1/8L/PG1b1PPP1/1PgKP4/5+p3/L6R1 b 1R2S3N4P1b1n2p 97';

describe('Game Formats', () => {
  describe('Format Detection', () => {
    it('should detect KIF format', () => {
      expect(detectFormat(testKIF)).toBe('kif');
    });

    it('should detect CSA format', () => {
      expect(detectFormat(testCSA)).toBe('csa');
    });
  });

  describe('KIF Move Conversion', () => {
    it('should convert basic pawn moves correctly', () => {
      // Test first 10 moves from the game
      const expectedMoves = [
        '2726', // ２六歩(27) -> +2726FU
        '8384', // ８四歩(83) -> -8384FU  
        '7776', // ７六歩(77) -> +7776FU
        '8485', // ８五歩(84) -> -8485FU
        '8877', // ７七角(88) -> +8877KA
        '3334', // ３四歩(33) -> -3334FU
        '7968', // ６八銀(79) -> +7968GI
        '4132', // ３二金(41) -> -4132KI
        '6978', // ７八金(69) -> +6978KI
        '2277+' // ７七角成(22) -> -2277UM
      ];

      const kifMoves = [
        '２六歩(27)',
        '８四歩(83)', 
        '７六歩(77)',
        '８五歩(84)',
        '７七角(88)',
        '３四歩(33)',
        '６八銀(79)',
        '３二金(41)',
        '７八金(69)',
        '７七角成(22)'
      ];

      kifMoves.forEach((kifMove, index) => {
        const result = kifToUsiMove(kifMove);
        console.log(`Move ${index + 1}: ${kifMove} -> ${result} (expected: ${expectedMoves[index]})`);
        expect(result).toBe(expectedMoves[index]);
      });
    });

    it('should convert drop moves correctly', () => {
      const dropMoves = [
        '５六角打', // Should be B*56
        '１四歩打', // Should be P*14
        '３三歩打', // Should be P*33
        '４七歩打', // Should be P*47
        '６三歩打', // Should be P*63
        '７二銀打', // Should be S*72
        '８七歩打', // Should be P*87
        '７七歩打', // Should be P*77
        '７八歩打', // Should be P*78
        '７七歩打', // Should be P*77 (again)
        '７八銀打', // Should be S*78
        '８六桂打', // Should be N*86
        '７七銀打', // Should be S*77
        '６六角打', // Should be B*66
        '７七金打'  // Should be G*77
      ];

      const expectedDropMoves = [
        'B*56',
        'P*14', 
        'P*33',
        'P*47',
        'P*63',
        'S*72',
        'P*87',
        'P*77',
        'P*78',
        'P*77',
        'S*78',
        'N*86',
        'S*77',
        'B*66',
        'G*77'
      ];

      dropMoves.forEach((kifMove, index) => {
        const result = kifToUsiMove(kifMove);
        console.log(`Drop move ${index + 1}: ${kifMove} -> ${result} (expected: ${expectedDropMoves[index]})`);
        expect(result).toBe(expectedDropMoves[index]);
      });
    });

    it('should convert promotion moves correctly', () => {
      const promotionMoves = [
        '１三桂成(25)', // Should be 2513+
        '４三銀成(34)', // Should be 3443+
        '１三歩成(14)', // Should be 1413+
        '同　飛成(76)', // Should be 7677+
        '同　桂成(85)'  // Should be 8577+
      ];

      const expectedPromotionMoves = [
        '2513+',
        '3443+',
        '1413+',
        '7677+',
        '8577+'
      ];

      promotionMoves.forEach((kifMove, index) => {
        const result = kifToUsiMove(kifMove);
        console.log(`Promotion move ${index + 1}: ${kifMove} -> ${result} (expected: ${expectedPromotionMoves[index]})`);
        expect(result).toBe(expectedPromotionMoves[index]);
      });
    });

    it('should handle resignation', () => {
      expect(kifToUsiMove('投了')).toBe('resign');
    });
  });

  describe('KIF Parsing', () => {
    it('should parse KIF game correctly', () => {
      const result = parseKIF(testKIF);
      
      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.data!.metadata.blackPlayer).toBe('石田直裕 五段');
      expect(result.data!.metadata.whitePlayer).toBe('藤井聡太 七段');
      expect(result.data!.metadata.event).toBe('竜王戦');
      expect(result.data!.metadata.site).toBe('関西将棋会館');
      
      // console.log(`Parsed ${result.data!.moves.length} moves (expected 97)`);
      // console.log('Last few moves:', result.data!.moves.slice(-5));
      
      expect(result.data!.moves.length).toBe(97);
      expect(result.data!.moves[96]).toBe('resign'); // Last move should be resignation
    });

    it('should parse first 10 moves correctly', () => {
      const result = parseKIF(testKIF);
      expect(result.success).toBe(true);
      
      const expectedFirstMoves = [
        '2726', '8384', '7776', '8485', '8877',
        '3334', '7968', '4132', '6978', '2277+'
      ];

      const actualFirstMoves = result.data!.moves.slice(0, 10);
      
      expectedFirstMoves.forEach((expected, index) => {
        console.log(`Move ${index + 1}: expected ${expected}, got ${actualFirstMoves[index]}`);
        expect(actualFirstMoves[index]).toBe(expected);
      });
    });
  });

  describe('CSA Parsing', () => {
    it('should parse CSA game correctly', () => {
      const result = parseCSA(testCSA);
      
      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.data!.metadata.blackPlayer).toBe('石田直裕 五段');
      expect(result.data!.metadata.whitePlayer).toBe('藤井聡太 七段');
      expect(result.data!.metadata.event).toBe('竜王戦');
      expect(result.data!.metadata.site).toBe('関西将棋会館');
      
      // console.log(`CSA parsed ${result.data!.moves.length} moves (expected 97)`);
      // console.log('Last few CSA moves:', result.data!.moves.slice(-5));
      
      expect(result.data!.moves.length).toBe(97);
      expect(result.data!.moves[96]).toBe('resign');
    });

    it('should parse first 10 CSA moves correctly', () => {
      const result = parseCSA(testCSA);
      expect(result.success).toBe(true);
      
      const expectedFirstMoves = [
        '2726', '8384', '7776', '8485', '8877',
        '3334', '7968', '4132', '6978', '2277+'
      ];

      const actualFirstMoves = result.data!.moves.slice(0, 10);
      
      expectedFirstMoves.forEach((expected, index) => {
        console.log(`CSA Move ${index + 1}: expected ${expected}, got ${actualFirstMoves[index]}`);
        expect(actualFirstMoves[index]).toBe(expected);
      });
    });
  });

  describe('Cross-Format Consistency', () => {
    it('should produce identical moves from KIF and CSA formats', () => {
      const kifResult = parseKIF(testKIF);
      const csaResult = parseCSA(testCSA);
      
      expect(kifResult.success).toBe(true);
      expect(csaResult.success).toBe(true);
      
      const kifMoves = kifResult.data!.moves;
      const csaMoves = csaResult.data!.moves;
      
      expect(kifMoves.length).toBe(csaMoves.length);
      
      // Check first 20 moves for consistency
      for (let i = 0; i < Math.min(20, kifMoves.length); i++) {
        // console.log(`Move ${i + 1}: KIF=${kifMoves[i]}, CSA=${csaMoves[i]}`);
        expect(kifMoves[i]).toBe(csaMoves[i]);
      }
    });

    it('should produce identical metadata from KIF and CSA formats', () => {
      const kifResult = parseKIF(testKIF);
      const csaResult = parseCSA(testCSA);
      
      expect(kifResult.success).toBe(true);
      expect(csaResult.success).toBe(true);
      
      const kifMetadata = kifResult.data!.metadata;
      const csaMetadata = csaResult.data!.metadata;
      
      expect(kifMetadata.blackPlayer).toBe(csaMetadata.blackPlayer);
      expect(kifMetadata.whitePlayer).toBe(csaMetadata.whitePlayer);
      expect(kifMetadata.event).toBe(csaMetadata.event);
      expect(kifMetadata.site).toBe(csaMetadata.site);
    });
  });
});
