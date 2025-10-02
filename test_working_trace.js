// Test using the existing debug infrastructure
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testWorkingTrace() {
    console.log('=== Shogi Engine Tracing Test (Using Existing Debug) ===\n');
    
    // Initialize the engine
    const engine = new WasmUsiHandler();
    
    // Test basic commands
    console.log('Testing position command...');
    const responses = engine.process_command('position startpos');
    console.log('Position command responses:', responses);
    console.log();
    
    // Enable debug mode using the existing USI command
    console.log('Enabling debug mode using USI command...');
    const debugResponses = engine.process_command('debug on');
    console.log('Debug command responses:', debugResponses);
    console.log();
    
    // Check debug status
    console.log('Debug enabled:', engine.is_debug_enabled());
    console.log();
    
    // Try a simple search with tracing
    console.log('Searching for best move with tracing...');
    console.log('This should show detailed timing and decision information:');
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
    
    // Disable debug mode
    console.log('Disabling debug mode...');
    const disableResponses = engine.process_command('debug off');
    console.log('Debug disable responses:', disableResponses);
    console.log('Debug enabled:', engine.is_debug_enabled());
    
    console.log('\n=== Test Complete ===');
}

// Run the test
testWorkingTrace().catch(console.error);
