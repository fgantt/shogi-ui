// Simple worker test without debugging
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testWorkerSimple() {
    console.log('=== Testing Worker Simple (No Debug) ===\n');
    
    try {
        // Test direct engine first
        console.log('Testing direct engine...');
        const engine = new WasmUsiHandler();
        console.log('Engine initialized successfully');
        
        // Test basic commands
        const responses = engine.process_command('position startpos');
        console.log('Position command responses:', responses);
        
        // Test a simple search
        console.log('Testing simple search...');
        try {
            engine.go_with_callback('go depth 2', (info) => {
                console.log('Search info:', info);
            });
            console.log('Search completed successfully');
        } catch (error) {
            console.error('Search failed:', error);
        }
        
    } catch (error) {
        console.error('Test failed:', error);
        console.error('Error stack:', error.stack);
    }
    
    console.log('\n=== Test Complete ===');
}

testWorkerSimple().catch(console.error);
