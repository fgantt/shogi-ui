// Test to isolate the worker error
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testWorkerError() {
    console.log('=== Testing Worker Error Isolation ===\n');
    
    try {
        // Initialize the engine directly (not in worker)
        console.log('Initializing WASM engine directly...');
        const engine = new WasmUsiHandler();
        console.log('Engine initialized successfully');
        
        // Test basic functionality
        console.log('Testing basic commands...');
        const responses = engine.process_command('usi');
        console.log('USI command responses:', responses);
        
        const readyResponses = engine.process_command('isready');
        console.log('IsReady command responses:', readyResponses);
        
        const positionResponses = engine.process_command('position startpos');
        console.log('Position command responses:', positionResponses);
        
        console.log('All basic commands successful');
        
    } catch (error) {
        console.error('Error in direct engine test:', error);
        console.error('Error stack:', error.stack);
    }
    
    console.log('\n=== Test Complete ===');
}

testWorkerError().catch(console.error);
