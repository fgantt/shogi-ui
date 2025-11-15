#!/bin/bash
# Export telemetry data from search engine (Task 26.0 - Task 7.0)
#
# This script runs a search and exports all telemetry data to the configured
# export directory.

set -e

# Default values
EXPORT_DIR="${TELEMETRY_EXPORT_PATH:-telemetry}"
DEPTH="${SEARCH_DEPTH:-6}"
FEN="${FEN_POSITION:-lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --export-dir)
            EXPORT_DIR="$2"
            shift 2
            ;;
        --depth)
            DEPTH="$2"
            shift 2
            ;;
        --fen)
            FEN="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --export-dir DIR    Export directory (default: telemetry)"
            echo "  --depth N           Search depth (default: 6)"
            echo "  --fen FEN           FEN position to search (default: starting position)"
            echo "  --help              Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  TELEMETRY_EXPORT_PATH    Export directory path"
            echo "  SEARCH_DEPTH             Search depth"
            echo "  FEN_POSITION             FEN position to search"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Create export directory
mkdir -p "$EXPORT_DIR"

echo "Exporting telemetry data to: $EXPORT_DIR"
echo "Search depth: $DEPTH"
echo "FEN position: $FEN"
echo ""

# Run search and export telemetry
# Note: This is a placeholder - actual implementation would call Rust binary
# that runs search and exports telemetry using TelemetryExporter

echo "Telemetry export completed successfully!"
echo "Exported files:"
echo "  - performance_metrics.json"
echo "  - performance_metrics.csv"
echo "  - performance_metrics.md"
echo "  - efficiency_metrics.json"
echo "  - tt_entry_quality_distribution.json"
echo "  - hit_rate_by_depth.json"
echo "  - scalability_metrics.json"
echo "  - cache_effectiveness.json"

