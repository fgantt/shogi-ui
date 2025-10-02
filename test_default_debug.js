// Test that debugging is enabled by default
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testDefaultDebug() {
    console.log('=== Testing Default Debug State ===\n');
    
    // Initialize the engine
    const engine = new WasmUsiHandler();
    
    // Check debug status without enabling it
    console.log('Debug enabled by default:', engine.is_debug_enabled());
    console.log();
    
    // Test basic commands
    console.log('Testing position command...');
    const responses = engine.process_command('position startpos');
    console.log('Position command responses:', responses);
    console.log();
    
    // Try a simple search - should show debug output if enabled
    console.log('Searching for best move (should show debug output if enabled)...');
    console.log('==========================================');
    
    try {
        engine.go_with_callback('go depth 2', (info) => {
            console.log('Search info:', info);
        });
        console.log('Search completed successfully');
    } catch (error) {
        console.error('Search failed:', error);
    }
    
    console.log('==========================================');
    console.log();
    
    console.log('\n=== Test Complete ===');
}

// Run the test
testDefaultDebug().catch(console.error);
