#!/usr/bin/env bash
# Serve the Overall web UI with live backend API
# Default port: 8459 (reserved for this project)

set -e

PORT="${1:-8459}"

echo "Starting Overall web server on port ${PORT}..."

# Kill any existing server on this port
lsof -ti:${PORT} | xargs -r kill -9 2>/dev/null || true

# Start the server
./target/release/overall serve --port ${PORT}
