#!/bin/bash
set -euo pipefail

echo "Checking prerequisites for overall project..."
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0

# Check Rust
echo -n "Checking Rust... "
if command -v rustc &> /dev/null; then
    VERSION=$(rustc --version)
    echo -e "${GREEN}OK${NC} ($VERSION)"
else
    echo -e "${RED}MISSING${NC}"
    echo "  Install from: https://rustup.rs"
    ERRORS=$((ERRORS + 1))
fi

# Check cargo
echo -n "Checking cargo... "
if command -v cargo &> /dev/null; then
    VERSION=$(cargo --version)
    echo -e "${GREEN}OK${NC} ($VERSION)"
else
    echo -e "${RED}MISSING${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check wasm-pack
echo -n "Checking wasm-pack... "
if command -v wasm-pack &> /dev/null; then
    VERSION=$(wasm-pack --version)
    echo -e "${GREEN}OK${NC} ($VERSION)"
else
    echo -e "${YELLOW}MISSING${NC}"
    echo "  Install: cargo install wasm-pack"
    echo "  Note: Required for building WASM UI"
fi

# Check gh CLI
echo -n "Checking gh CLI... "
if command -v gh &> /dev/null; then
    VERSION=$(gh --version | head -n1)
    echo -e "${GREEN}OK${NC} ($VERSION)"

    # Check authentication
    echo -n "Checking gh authentication... "
    if gh auth status &> /dev/null; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}NOT AUTHENTICATED${NC}"
        echo "  Run: gh auth login"
    fi
else
    echo -e "${RED}MISSING${NC}"
    echo "  Install from: https://cli.github.com"
    ERRORS=$((ERRORS + 1))
fi

# Check Ollama
echo -n "Checking Ollama... "
if command -v ollama &> /dev/null; then
    VERSION=$(ollama --version 2>&1 | head -n1 || echo "installed")
    echo -e "${GREEN}OK${NC} ($VERSION)"

    # Check if Ollama is running
    echo -n "Checking Ollama service... "
    if curl -s http://localhost:11434/api/tags &> /dev/null; then
        echo -e "${GREEN}RUNNING${NC}"

        # Check for phi3 model
        echo -n "Checking phi3:3.8b model... "
        if ollama list | grep -q "phi3:3.8b"; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${YELLOW}MISSING${NC}"
            echo "  Run: ollama pull phi3:3.8b"
        fi
    else
        echo -e "${YELLOW}NOT RUNNING${NC}"
        echo "  Start with: ollama serve"
    fi
else
    echo -e "${YELLOW}MISSING${NC}"
    echo "  Install from: https://ollama.ai"
    echo "  Note: Required for AI analysis features"
fi

# Check ask CLI
echo -n "Checking ask CLI... "
if command -v ask &> /dev/null; then
    VERSION=$(ask --version 2>&1 | head -n1 || echo "installed")
    echo -e "${GREEN}OK${NC} ($VERSION)"
else
    echo -e "${YELLOW}MISSING${NC}"
    echo "  Expected at: ~/.local/softwarewrighter/bin/ask"
    echo "  Note: Required for AI analysis features"
fi

echo
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}All critical prerequisites met!${NC}"
    exit 0
else
    echo -e "${RED}Missing $ERRORS critical prerequisite(s)${NC}"
    exit 1
fi
