# GitHub Repository Manager (overall) - Wiki

Welcome to the **overall** project documentation! This wiki provides comprehensive architecture documentation, diagrams, and detailed component descriptions.

## Project Overview

**overall** is a Rust/Yew web application that tracks and prioritizes GitHub repositories across multiple users and organizations. It identifies unmerged branches ready for pull requests or merges, using local Ollama LLM for AI-powered analysis.

### Key Features

- **Multi-Organization Tracking**: Monitor repositories across multiple GitHub users/organizations
- **Branch Analysis**: Automatically identify branches ready for pull requests or merges
- **Local Repository Integration**: Track local git working directory status
- **AI-Powered Insights**: Uses local Ollama LLM to analyze projects and suggest next steps
- **Priority System**: Intelligent prioritization based on recency, activity, and branch status
- **Visual Status Icons**: Traffic-light priority system for urgent actions
- **Web UI**: Modern Yew/WASM frontend with responsive design

## Architecture Documentation

### Core Architecture

- **[Architecture Overview](Architecture-Overview)** - High-level system architecture with block diagrams
- **[Data Flow](Data-Flow)** - Sequence diagrams showing key workflows and data movement
- **[Storage Layer](Storage-Layer)** - Database schema and data model with ER diagrams

### Component Deep Dives

- **[GitHub Integration](GitHub-Integration)** - How we interact with GitHub API via `gh` CLI
- **[Web Server & API](Web-Server-API)** - Axum server architecture and REST API endpoints
- **[UI Components](UI-Components)** - Yew/WASM frontend architecture and component hierarchy

## Quick Start

### Building the Project

```bash
# Full build (CLI + WASM UI)
./scripts/build-all.sh

# CLI only
cargo build --release -p overall-cli

# WASM UI only
cd wasm-ui && wasm-pack build --target web --release
```

### Running the Application

```bash
# 1. Scan repositories
./target/release/overall scan <github-username>

# 2. Export to JSON
./target/release/overall export

# 3. Start web server
./scripts/serve.sh
```

Visit `http://localhost:8459` to view the web UI.

## Technology Stack

### Backend
- **Rust** - Systems programming language
- **Axum** - Web server framework
- **SQLite** (rusqlite) - Local database
- **serde/serde_json** - Serialization

### Frontend
- **Yew** - Rust framework for building web UIs with WebAssembly
- **wasm-pack** - WASM build tool
- **web-sys** - Web API bindings

### External Tools
- **gh CLI** - GitHub API access
- **Ollama** - Local LLM (optional)
- **ask CLI** - LLM interface (optional)

## Project Structure

```
overall/
├── overall-cli/          # Backend Rust library and CLI binary
│   ├── src/
│   │   ├── main.rs      # CLI entry point
│   │   ├── github/      # GitHub API integration
│   │   ├── storage/     # SQLite database layer
│   │   ├── analysis/    # Branch analysis logic
│   │   ├── ai/          # Ollama LLM integration
│   │   ├── server/      # Axum web server
│   │   └── models/      # Shared data structures
│   └── Cargo.toml
├── wasm-ui/             # Frontend Yew/WASM application
│   ├── src/lib.rs       # Main UI component
│   └── Cargo.toml
├── static/              # Static web assets
│   ├── index.html
│   ├── icons/          # Status icon PNGs
│   └── pkg/            # WASM build output
├── scripts/            # Build and deployment scripts
├── docs/               # Additional documentation
└── wiki/               # This wiki (for GitHub wiki sync)
```

## Development Process

See **[CLAUDE.md](../CLAUDE.md)** for detailed development guidelines including:
- Test-Driven Development (TDD) requirements
- Checkpoint process before commits
- Testing strategy and mock patterns
- Code style and linting rules

## Contributing

1. Read [Architecture Overview](Architecture-Overview) to understand the system
2. Review [CLAUDE.md](../CLAUDE.md) for development processes
3. Write tests first (TDD)
4. Run checkpoint process before committing:
   ```bash
   cargo test --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all
   ./scripts/build-all.sh
   ```

## Additional Resources

- **Repository**: [softwarewrighter/overall](https://github.com/softwarewrighter/overall)
- **Issue Tracker**: [GitHub Issues](https://github.com/softwarewrighter/overall/issues)
- **Main Documentation**: See `docs/` directory in repository

---

**Last Updated**: 2025-11-17
**Maintainer**: softwarewrighter
