// Test script to demonstrate the tracing system
// This script shows how to enable tracing and see the debug output

import { WasmUsiHandler } from './pkg/shogi_engine.js';

async function testTracing() {
    console.log('=== Shogi Engine Tracing Test ===\n');
    
    // Initialize the engine
    const engine = new WasmUsiHandler();
    
    // Enable debug tracing
    console.log('Enabling debug tracing...');
    engine.set_debug_enabled(true);
    console.log('Debug enabled:', engine.is_debug_enabled());
    console.log();
    
    // Set up a simple position
    console.log('Setting up starting position...');
    const responses = engine.process_command('position startpos');
    console.log('Position command responses:', responses);
    console.log();
    
    // Make a move to create a more interesting position
    console.log('Making a move to create an interesting position...');
    const moveResponses = engine.process_command('position startpos moves 7g7f');
    console.log('Move command responses:', moveResponses);
    console.log();
    
    // Now search for a move with tracing enabled
    console.log('Searching for best move with tracing...');
    console.log('This will show detailed timing and decision information:');
    console.log('==========================================');
    
    // Use go_with_callback for the go command
    engine.go_with_callback('go depth 3', (info) => {
        console.log('Search info:', info);
    });
    console.log('==========================================');
    console.log();
    
    // Disable tracing
    console.log('Disabling debug tracing...');
    engine.set_debug_enabled(false);
    console.log('Debug enabled:', engine.is_debug_enabled());
    console.log();
    
    // Search again without tracing
    console.log('Searching again without tracing (should be quiet)...');
    engine.go_with_callback('go depth 2', (info) => {
        console.log('Search info:', info);
    });
    
    console.log('\n=== Test Complete ===');
}

// Run the test
testTracing().catch(console.error);
