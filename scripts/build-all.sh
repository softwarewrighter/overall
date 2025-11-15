#!/bin/bash
set -euo pipefail

echo "Building overall project..."
echo

# Build backend CLI (release mode)
echo "Building Rust CLI (release)..."
cargo build --release -p overall-cli

# Build WASM UI
echo
echo "Building WASM UI..."
cd wasm-ui

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack not found"
    echo "Install with: cargo install wasm-pack"
    exit 1
fi

wasm-pack build --target web --release

cd ..

# Create static directory if it doesn't exist
mkdir -p static/wasm

# Copy WASM artifacts
echo
echo "Copying WASM artifacts to static/wasm/..."
cp -r wasm-ui/pkg/* static/wasm/

# Generate build info
echo
echo "Generating build info..."
cat > static/build-info.json <<EOF
{
  "version": "0.1.0",
  "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "build_host": "$(hostname)",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_commit_short": "$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')"
}
EOF

echo
echo "Build complete!"
echo "  CLI binary: target/release/overall"
echo "  WASM UI: static/wasm/"
echo "  Build info: static/build-info.json"
