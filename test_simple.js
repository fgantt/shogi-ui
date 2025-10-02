// Simple test to check basic functionality
import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testSimple() {
    console.log('=== Simple Shogi Engine Test ===\n');
    
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
    
    engine.set_debug_enabled(true);
    console.log('Debug enabled (after enable):', engine.is_debug_enabled());
    
    engine.set_debug_enabled(false);
    console.log('Debug enabled (after disable):', engine.is_debug_enabled());
    console.log();
    
    console.log('=== Test Complete ===');
}

// Run the test
testSimple().catch(console.error);
