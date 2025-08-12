import { 
  getWasmAiMove, 
  isWasmEngineAvailable, 
  getEngineStats, 
  getEngineCapabilities,
  benchmarkEngines,
  getPerformanceMetrics
} from './wasmEngine.js';

// Mock game state for testing
const mockGameState = {
  board: [
    // Row 0 (top - player2)
    [
      { type: 'L', player: 'player2' }, // Lance
      { type: 'N', player: 'player2' }, // Knight
      { type: 'S', player: 'player2' }, // Silver
      { type: 'G', player: 'player2' }, // Gold
      { type: 'K', player: 'player2' }, // King
      { type: 'G', player: 'player2' }, // Gold
      { type: 'S', player: 'player2' }, // Silver
      { type: 'N', player: 'player2' }, // Knight
      { type: 'L', player: 'player2' }  // Lance
    ],
    // Row 1
    [
      null,
      { type: 'R', player: 'player2' }, // Rook
      null, null, null, null, null,
      { type: 'B', player: 'player2' }, // Bishop
      null
    ],
    // Row 2
    Array(9).fill().map(() => ({ type: 'P', player: 'player2' })), // Pawns
    // Rows 3-5 (empty)
    Array(3).fill().map(() => Array(9).fill(null)),
    // Row 6
    Array(9).fill().map(() => ({ type: 'P', player: 'player1' })), // Pawns
    // Row 7
    [
      null,
      { type: 'B', player: 'player1' }, // Bishop
      null, null, null, null, null,
      { type: 'R', player: 'player1' }, // Rook
      null
    ],
    // Row 8 (bottom - player1)
    [
      { type: 'L', player: 'player1' }, // Lance
      { type: 'N', player: 'player1' }, // Knight
      { type: 'S', player: 'player1' }, // Silver
      { type: 'G', player: 'player1' }, // Gold
      { type: 'K', player: 'player1' }, // King
      { type: 'G', player: 'player1' }, // Gold
      { type: 'S', player: 'player1' }, // Silver
      { type: 'N', player: 'player1' }, // Knight
      { type: 'L', player: 'player1' }  // Lance
    ]
  ],
  currentPlayer: 'player1',
  capturedPieces: {
    player1: [],
    player2: []
  },
  moveHistory: [],
  isCheck: false,
  isCheckmate: false
};

/**
 * Test WebAssembly engine initialization
 */
async function testInitialization() {
  console.log('Testing WebAssembly engine initialization...');
  
  try {
    const stats = getEngineStats();
    console.log('Engine stats:', stats);
    
    const capabilities = getEngineCapabilities();
    console.log('Engine capabilities:', capabilities);
    
    const isAvailable = isWasmEngineAvailable();
    console.log('Engine available:', isAvailable);
    
    return true;
  } catch (error) {
    console.error('Initialization test failed:', error);
    return false;
  }
}

/**
 * Test basic move generation
 */
async function testMoveGeneration() {
  console.log('Testing move generation...');
  
  try {
    const move = await getWasmAiMove(mockGameState, 'easy');
    console.log('Generated move:', move);
    
    if (move && (move.from === 'drop' || (Array.isArray(move.from) && Array.isArray(move.to)))) {
      console.log('✅ Move generation test passed');
      return true;
    } else {
      console.log('❌ Move generation test failed - invalid move format');
      return false;
    }
  } catch (error) {
    console.error('Move generation test failed:', error);
    return false;
  }
}

/**
 * Test performance benchmarking
 */
async function testPerformanceBenchmark() {
  console.log('Testing performance benchmark...');
  
  try {
    const results = await benchmarkEngines(mockGameState, 'medium');
    console.log('Benchmark results:', results);
    
    if (results.wasm && results.wasm.time > 0) {
      console.log('✅ Performance benchmark test passed');
      return true;
    } else {
      console.log('❌ Performance benchmark test failed');
      return false;
    }
  } catch (error) {
    console.error('Performance benchmark test failed:', error);
    return false;
  }
}

/**
 * Test performance metrics
 */
async function testPerformanceMetrics() {
  console.log('Testing performance metrics...');
  
  try {
    const metrics = await getPerformanceMetrics(mockGameState, 'easy');
    console.log('Performance metrics:', metrics);
    
    if (metrics.executionTime > 0) {
      console.log('✅ Performance metrics test passed');
      return true;
    } else {
      console.log('❌ Performance metrics test failed');
      return false;
    }
  } catch (error) {
    console.error('Performance metrics test failed:', error);
    return false;
  }
}

/**
 * Run all tests
 */
async function runAllTests() {
  console.log('🚀 Starting WebAssembly engine tests...\n');
  
  const tests = [
    { name: 'Initialization', fn: testInitialization },
    { name: 'Move Generation', fn: testMoveGeneration },
    { name: 'Performance Benchmark', fn: testPerformanceBenchmark },
    { name: 'Performance Metrics', fn: testPerformanceMetrics }
  ];
  
  let passed = 0;
  let total = tests.length;
  
  for (const test of tests) {
    console.log(`\n--- ${test.name} Test ---`);
    const result = await test.fn();
    if (result) passed++;
    console.log(`--- ${test.name} Test Complete ---\n`);
  }
  
  console.log(`\n📊 Test Results: ${passed}/${total} tests passed`);
  
  if (passed === total) {
    console.log('🎉 All tests passed! WebAssembly integration is working correctly.');
  } else {
    console.log('⚠️  Some tests failed. Check the logs above for details.');
  }
  
  return passed === total;
}

/**
 * Test different difficulties
 */
async function testDifficulties() {
  console.log('Testing different difficulty levels...');
  
  const difficulties = ['easy', 'medium', 'hard'];
  const results = {};
  
  for (const difficulty of difficulties) {
    try {
      console.log(`Testing ${difficulty} difficulty...`);
      const start = performance.now();
      
      const move = await getWasmAiMove(mockGameState, difficulty);
      const time = performance.now() - start;
      
      results[difficulty] = {
        move,
        time: time.toFixed(2) + 'ms',
        success: !!move
      };
      
      console.log(`${difficulty}: ${time.toFixed(2)}ms - ${move ? 'Success' : 'Failed'}`);
    } catch (error) {
      console.error(`${difficulty} difficulty test failed:`, error);
      results[difficulty] = {
        error: error.message,
        success: false
      };
    }
  }
  
  console.log('Difficulty test results:', results);
  return results;
}

// Export test functions for external use
export {
  testInitialization,
  testMoveGeneration,
  testPerformanceBenchmark,
  testPerformanceMetrics,
  runAllTests,
  testDifficulties,
  mockGameState
};

// Run tests if this file is executed directly
if (typeof window !== 'undefined') {
  // Browser environment - wait for page load
  window.addEventListener('load', () => {
    setTimeout(() => {
      console.log('WebAssembly engine tests ready. Call runAllTests() to start testing.');
    }, 1000);
  });
} else {
  // Node.js environment - run tests immediately
  runAllTests().catch(console.error);
}
