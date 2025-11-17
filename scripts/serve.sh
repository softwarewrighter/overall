#!/usr/bin/env bash
# Serve the Overall web UI with live backend API
# Default port: 8459 (reserved for this project)
#
# Usage:
#   ./scripts/serve.sh           # Start on default port 8459
#   ./scripts/serve.sh 8080      # Start on custom port
#   ./scripts/serve.sh --debug   # Start with debug logging
#   ./scripts/serve.sh 8080 --debug  # Custom port with debug

set -e

PORT="8459"
DEBUG_FLAG=""

# Parse arguments
for arg in "$@"; do
    if [ "$arg" = "--debug" ]; then
        DEBUG_FLAG="--debug"
    elif [[ "$arg" =~ ^[0-9]+$ ]]; then
        PORT="$arg"
    fi
done

echo "Starting Overall web server on port ${PORT}..."
if [ -n "$DEBUG_FLAG" ]; then
    echo "Debug mode enabled"
fi

# CRITICAL: Check if WASM needs rebuilding
echo "Checking WASM build freshness..."

WASM_SOURCE="wasm-ui/src/lib.rs"
WASM_OUTPUT="static/wasm/wasm_ui_bg.wasm"

if [ ! -f "$WASM_OUTPUT" ]; then
    echo "❌ ERROR: WASM not built!"
    echo "   Missing: $WASM_OUTPUT"
    echo "   Run: ./scripts/build-all.sh"
    exit 1
fi

if [ "$WASM_SOURCE" -nt "$WASM_OUTPUT" ]; then
    echo "❌ ERROR: WASM is out of date!"
    echo "   Source modified: $WASM_SOURCE"
    echo "   WASM built:      $WASM_OUTPUT"
    echo ""
    echo "   Your source code is newer than the deployed WASM."
    echo "   The server would serve OLD, STALE code."
    echo ""
    echo "   Run: ./scripts/build-all.sh"
    echo ""
    exit 1
fi

# Also check other critical source files
for src_file in wasm-ui/src/*.rs wasm-ui/Cargo.toml; do
    if [ -f "$src_file" ] && [ "$src_file" -nt "$WASM_OUTPUT" ]; then
        echo "❌ ERROR: WASM is out of date!"
        echo "   Source modified: $src_file"
        echo "   Run: ./scripts/build-all.sh"
        exit 1
    fi
done

echo "✅ WASM build is fresh"

# Kill any existing server on this port
lsof -ti:${PORT} | xargs -r kill -9 2>/dev/null || true

# Start the server
./target/release/overall serve --port ${PORT} ${DEBUG_FLAG}
