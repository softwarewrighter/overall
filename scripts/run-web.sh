#!/bin/bash
set -euo pipefail

PORT="${1:-8080}"

# Check if static directory exists
if [ ! -d "static" ]; then
    echo "ERROR: static/ directory not found"
    echo "Run ./scripts/build-all.sh first"
    exit 1
fi

# Check if WASM files exist
if [ ! -d "static/wasm" ]; then
    echo "ERROR: static/wasm/ directory not found"
    echo "Run ./scripts/build-all.sh first"
    exit 1
fi

echo "Starting web server on port $PORT..."
echo "Open http://localhost:$PORT in your browser"
echo "Press Ctrl+C to stop"
echo

# Use Python's built-in HTTP server
if command -v python3 &> /dev/null; then
    cd static && python3 -m http.server "$PORT"
elif command -v python &> /dev/null; then
    cd static && python -m SimpleHTTPServer "$PORT"
else
    echo "ERROR: Python not found"
    echo "Install Python or use another web server"
    exit 1
fi
