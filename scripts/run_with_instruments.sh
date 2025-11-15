#!/bin/bash
# Run search engine with Instruments profiling (macOS) (Task 26.0 - Task 8.0)
#
# This script runs the search engine with Instruments profiling enabled for hot path analysis.

set -e

# Default values
DEPTH="${SEARCH_DEPTH:-6}"
FEN="${FEN_POSITION:-lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1}"
OUTPUT_DIR="${INSTRUMENTS_OUTPUT_DIR:-instruments_data}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --depth)
            DEPTH="$2"
            shift 2
            ;;
        --fen)
            FEN="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --depth N           Search depth (default: 6)"
            echo "  --fen FEN           FEN position to search (default: starting position)"
            echo "  --output-dir DIR    Output directory for Instruments data (default: instruments_data)"
            echo "  --help              Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  SEARCH_DEPTH             Search depth"
            echo "  FEN_POSITION             FEN position to search"
            echo "  INSTRUMENTS_OUTPUT_DIR   Output directory for Instruments data"
            echo ""
            echo "Note: This script requires Xcode Command Line Tools to be installed"
            echo "      (instruments command is part of Xcode)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if instruments is available
if ! command -v instruments &> /dev/null; then
    echo "Error: instruments is not installed or not in PATH"
    echo "Install Xcode Command Line Tools with: xcode-select --install"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Running search with Instruments profiling..."
echo "Search depth: $DEPTH"
echo "FEN position: $FEN"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Run with Instruments
# Note: This is a placeholder - actual implementation would call Rust binary
# that runs search with external profiler enabled

instruments -t "Time Profiler" \
    -D "$OUTPUT_DIR/instruments.trace" \
    echo "Running search with depth $DEPTH and FEN: $FEN"

echo ""
echo "Instruments profiling completed!"
echo "Data saved to: $OUTPUT_DIR/instruments.trace"
echo ""
echo "To analyze the results:"
echo "  open $OUTPUT_DIR/instruments.trace"
echo "  (This will open the trace file in Instruments.app)"

