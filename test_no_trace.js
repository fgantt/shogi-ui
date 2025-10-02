// Test without tracing to see if basic search works
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testNoTrace() {
    console.log('=== Shogi Engine Test (No Tracing) ===\n');
    
    // Initialize the engine
    const engine = new WasmUsiHandler();
    
    // Test basic commands
    console.log('Testing position command...');
    const responses = engine.process_command('position startpos');
    console.log('Position command responses:', responses);
    console.log();
    
    // Test debug enable/disable
    console.log('Testing debug enable/disable...');
    console.log('Debug enabled (initial):', engine.is_debug_enabled());
    
    // Keep tracing disabled for now
    engine.set_debug_enabled(false);
    console.log('Debug enabled (after disable):', engine.is_debug_enabled());
    console.log();
    
    // Try a simple search without tracing
    console.log('Searching for best move without tracing...');
    try {
        engine.go_with_callback('go depth 2', (info) => {
            console.log('Search info:', info);
        });
        console.log('Search completed successfully');
    } catch (error) {
        console.error('Search failed:', error);
    }
    
    console.log('\n=== Test Complete ===');
}

// Run the test
testNoTrace().catch(console.error);
