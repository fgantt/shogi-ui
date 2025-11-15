#!/bin/bash
# Run search engine with perf profiling (Linux) (Task 26.0 - Task 8.0)
#
# This script runs the search engine with perf profiling enabled for hot path analysis.

set -e

# Default values
DEPTH="${SEARCH_DEPTH:-6}"
FEN="${FEN_POSITION:-lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1}"
OUTPUT_DIR="${PERF_OUTPUT_DIR:-perf_data}"

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
            echo "  --output-dir DIR    Output directory for perf data (default: perf_data)"
            echo "  --help              Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  SEARCH_DEPTH             Search depth"
            echo "  FEN_POSITION             FEN position to search"
            echo "  PERF_OUTPUT_DIR          Output directory for perf data"
            echo ""
            echo "Note: This script requires perf to be installed and the process to have"
            echo "      permission to use perf (may require sudo or kernel.perf_event_paranoid=0)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if perf is available
if ! command -v perf &> /dev/null; then
    echo "Error: perf is not installed or not in PATH"
    echo "Install perf with: sudo apt-get install linux-perf (Debian/Ubuntu)"
    echo "                   sudo yum install perf (RHEL/CentOS)"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Running search with perf profiling..."
echo "Search depth: $DEPTH"
echo "FEN position: $FEN"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Run with perf
# Note: This is a placeholder - actual implementation would call Rust binary
# that runs search with external profiler enabled

perf record -o "$OUTPUT_DIR/perf.data" \
    -g --call-graph dwarf \
    -F 99 \
    -- \
    echo "Running search with depth $DEPTH and FEN: $FEN"

echo ""
echo "Perf profiling completed!"
echo "Data saved to: $OUTPUT_DIR/perf.data"
echo ""
echo "To analyze the results:"
echo "  perf report -i $OUTPUT_DIR/perf.data"
echo "  perf script -i $OUTPUT_DIR/perf.data > $OUTPUT_DIR/perf.script"
echo "  perf annotate -i $OUTPUT_DIR/perf.data"

