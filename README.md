# GitHub Repository Manager (overall)

A Rust/Yew web application that helps developers track and prioritize their GitHub repositories across multiple users and organizations.

## Features

- Scan repositories from GitHub users/organizations
- Identify unmerged branches ready for pull requests or merges
- Calculate priority scores based on activity and recency
- AI-powered analysis using local Ollama LLM
- Web-based UI for browsing and filtering repositories
- Local-first SQLite storage

## Prerequisites

- Rust (latest stable)
- wasm-pack
- gh CLI (authenticated)
- Ollama (with phi3:3.8b model)
- ask CLI (from ~/.local/softwarewrighter/bin/)

## Quick Start

```bash
# Check prerequisites
./scripts/check-setup.sh

# Authenticate gh CLI (if not already)
gh auth login

# Pull Ollama model (if not already)
ollama pull phi3:3.8b

# Build the project
./scripts/build-all.sh

# Run the CLI
./target/release/overall scan softwarewrighter

# Start web UI
./scripts/run-web.sh
```

## Project Structure

```
overall/
+-- overall-cli/          # Backend Rust library and CLI
|   +-- src/
|       +-- github/       # GitHub API integration
|       +-- storage/      # SQLite database
|       +-- analysis/     # Priority calculation
|       +-- ai/           # Ollama integration
|       +-- models/       # Data models
+-- wasm-ui/              # Frontend Yew/WASM application
|   +-- src/
|       +-- components/   # UI components
|       +-- api/          # Backend calls
+-- static/               # Web assets
+-- scripts/              # Build and run scripts
+-- docs/                 # Documentation
```

## Development

### Building

```bash
# Build everything
./scripts/build-all.sh

# Build CLI only
cargo build --release -p overall-cli

# Build WASM UI only
cd wasm-ui && wasm-pack build --target web --release
```

### Testing

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_list_repos

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check prerequisites
./scripts/check-setup.sh
```

## Documentation

- [Architecture](docs/architecture.md) - System design and components
- [Product Requirements](docs/prd.md) - Features and requirements
- [Design Decisions](docs/design.md) - Technical decisions
- [Implementation Plan](docs/plan.md) - Development roadmap
- [Status](docs/status.md) - Current progress

## Current Status

**Phase**: Phase 0 Complete - Project Setup

**Version**: 0.1.0-dev

**Next**: Phase 1 - GitHub Integration

See [docs/status.md](docs/status.md) for detailed progress.

## Configuration

Default configuration (created on first run):

```toml
# ~/.config/overall/config.toml
[github]
owners = ["softwarewrighter"]
repo_limit = 50
```

## License

MIT
