# USI Communication Monitor Feature

## Overview

The USI Communication Monitor is a new debugging panel that allows you to monitor the communication between the UI and the USI engine in real-time. This feature is particularly useful for debugging engine integration issues and understanding the USI protocol flow.

## Features

### 1. Last Command Display
- **Last Sent Command**: Shows the most recent USI command sent from the UI to the engine
- **Last Received Command**: Shows the most recent USI response received from the engine

### 2. Communication History
- **Scrollable History**: A complete log of all USI communication
- **Direction Indicators**: 
  - `>` prefix for commands sent from UI to engine
  - `<` prefix for responses received from engine to UI
- **Timestamps**: Each entry shows the time when the communication occurred
- **Auto-scroll**: Automatically scrolls to show the latest communication

### 3. Toggle Visibility
- **Toggle Button**: Click "USI Monitor" button in the top-right corner to show/hide the panel
- **Close Button**: Click the "×" button in the panel header to hide it

## How to Use

1. **Start a Game**: Navigate to the game page
2. **Open Monitor**: Click the "USI Monitor" button in the top-right corner
3. **Make Moves**: Play moves normally - the monitor will show all USI communication
4. **View History**: Scroll through the communication history to see the complete conversation

## USI Commands Monitored

The monitor tracks all standard USI commands:

- `usi` - Initialize USI mode
- `isready` - Check if engine is ready
- `setoption` - Set engine options
- `usinewgame` - Start a new game
- `position` - Set board position
- `go` - Start searching for best move
- `stop` - Stop searching
- `quit` - Quit engine

## Technical Implementation

### Components
- **UsiMonitor.tsx**: React component for the monitor panel
- **UsiMonitor.css**: Styling for the monitor interface

### Event System
- **usiCommandSent**: Emitted when UI sends a command to engine
- **usiCommandReceived**: Emitted when engine sends a response to UI

### Integration
- Integrated into both compact and classic game layouts
- Uses the existing event system from WasmUsiEngineAdapter
- Automatically tracks all USI communication without affecting game performance

## Styling

The monitor panel features:
- Fixed positioning in the top-right corner
- Responsive design that adapts to mobile screens
- Color-coded commands (blue for sent, purple for received)
- Monospace font for easy reading of USI commands
- Smooth scrolling and hover effects

## Use Cases

1. **Debugging Engine Issues**: See exactly what commands are being sent and received
2. **Learning USI Protocol**: Understand how the USI protocol works in practice
3. **Performance Analysis**: Monitor the timing of engine responses
4. **Development**: Verify that custom engine modifications work correctly

## Future Enhancements

Potential improvements could include:
- Filtering options for specific command types
- Export functionality for communication logs
- Search within the communication history
- Performance metrics display
- Command syntax highlighting
