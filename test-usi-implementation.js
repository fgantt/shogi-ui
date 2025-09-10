// Simple test script to verify the new USI implementation
import { WasmUsiEngineAdapter } from './src/usi/engine.js';

async function testUsiImplementation() {
  console.log('Testing new USI implementation...');
  
  try {
    // Create the adapter
    const adapter = new WasmUsiEngineAdapter();
    console.log('✓ WasmUsiEngineAdapter created');
    
    // Initialize
    await adapter.init();
    console.log('✓ Engine initialized');
    
    // Check readiness
    await adapter.isReady();
    console.log('✓ Engine is ready');
    
    // Set position
    await adapter.setPosition('lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1', []);
    console.log('✓ Position set');
    
    // Start a search
    console.log('Starting search...');
    adapter.go({ btime: 5000, wtime: 5000, byoyomi: 1000 });
    
    // Listen for bestmove
    adapter.on('bestmove', ({ move }) => {
      console.log('✓ Best move received:', move);
      process.exit(0);
    });
    
    // Timeout after 10 seconds
    setTimeout(() => {
      console.log('❌ Test timed out');
      process.exit(1);
    }, 10000);
    
  } catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
  }
}

testUsiImplementation();
