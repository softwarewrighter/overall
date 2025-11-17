# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GitHub Repository Manager (overall) - A Rust/Yew web application that tracks and prioritizes GitHub repositories across multiple users and organizations. It identifies unmerged branches ready for pull requests or merges, using local Ollama LLM for AI-powered analysis.

## Build Commands

### Full Build
```bash
./scripts/build-all.sh
```
Builds both the CLI (overall-cli) and WASM UI (wasm-ui) components.

### CLI Only
```bash
cargo build --release -p overall-cli
```

### WASM UI Only
```bash
cd wasm-ui && wasm-pack build --target web --release
```

### Development Mode
```bash
# Build with debug symbols
cargo build -p overall-cli
```

## Testing

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_list_repos

# Run with output
cargo test -- --nocapture

# Run tests in specific package
cargo test -p overall-cli
```

## Linting and Formatting

```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check prerequisites
./scripts/check-setup.sh
```

## Running the Application

### CLI Commands

The binary name is `overall` (built to `./target/release/overall`).

```bash
# Scan repositories for a GitHub user/org
./target/release/overall scan <owner> [--limit 50]

# List tracked repositories
./target/release/overall list

# Export data to JSON (for web UI)
./target/release/overall export [--output static/repos.json]

# Start web server
./target/release/overall serve [--port 8459] [--debug]
```

### Quick Development Workflow

```bash
# 1. Build everything
./scripts/build-all.sh

# 2. Scan repositories
./target/release/overall scan softwarewrighter

# 3. Export to JSON
./target/release/overall export

# 4. Start server
./scripts/serve.sh
# OR manually:
./target/release/overall serve --port 8459
```

## Architecture

### Workspace Structure

This is a Cargo workspace with two main packages:

- **overall-cli**: Backend Rust library and CLI binary
  - Location: `overall-cli/`
  - Binary: `src/main.rs`
  - Library modules: `github`, `storage`, `analysis`, `ai`, `server`, `models`

- **wasm-ui**: Frontend Yew/WASM application
  - Location: `wasm-ui/`
  - Single-file app: `src/lib.rs`
  - Built with wasm-pack

### Key Modules

#### GitHub Integration (`overall-cli/src/github/`)
- Uses `gh` CLI for all GitHub API interactions
- `commands.rs`: Executes gh CLI commands and parses JSON output
- Functions: `list_repos()`, `fetch_branches()`, `fetch_pull_requests()`, `fetch_commits()`, `create_pull_request()`

#### Storage (`overall-cli/src/storage/`)
- SQLite database via rusqlite
- Schema: `schema.sql` embedded via `include_str!`
- Database location: `~/.overall/overall.db`
- Tables: repositories, branches, pull_requests, commits, groups, repo_groups, local_repo_roots, local_repo_statuses

#### Server (`overall-cli/src/server/`)
- Axum web server with REST API
- Serves static files from `static/` directory
- API endpoints:
  - `GET /api/build-info` - Build metadata
  - `POST /api/groups` - Create/manage repository groups
  - `POST /api/groups/:id/repos/:repo_id` - Move repos between groups
  - `POST /api/repos/create-pr` - Create pull request
  - `POST /api/repos/create-all-prs` - Create PRs for all unmerged branches
  - `POST /api/local-repo-roots` - Manage local repository roots
  - `POST /api/local-repos/scan` - Scan local repositories

#### Models (`overall-cli/src/models/`)
- Shared data structures: Repository, Branch, Commit, PullRequest, Group
- BranchStatus enum: Unknown, NeedsSync, ReadyForPR, PROpen, Merged

### Data Flow

1. **Scan**: CLI executes `gh` commands → parses JSON → saves to SQLite
2. **Export**: Reads from SQLite → builds JSON with groups/repos/branches/PRs → writes to `static/repos.json`
3. **Serve**: Axum server serves static files + REST API for dynamic operations
4. **WASM UI**: Fetches `repos.json` → renders in Yew components → can call API endpoints

### External Dependencies

- **gh CLI**: Must be installed and authenticated (`gh auth login`)
- **ask CLI**: Optional, for AI analysis (from `~/.local/softwarewrighter/bin/`)
- **Ollama**: Optional, for local LLM (default model: phi3:3.8b)

## Common Patterns

### Database Access

Always use the Database wrapper in `storage/mod.rs`. Open database with:

```rust
let db_path = PathBuf::from(env::var("HOME")?).join(".overall").join("overall.db");
let db = Database::open_or_create(&db_path)?;
```

### GitHub API Calls

All GitHub operations go through `gh` CLI. Example pattern in `github/commands.rs`:

```rust
let output = Command::new("gh")
    .args(&["repo", "list", owner, "--json", "name,owner,..."])
    .output()?;
let repos: Vec<GhRepo> = serde_json::from_slice(&output.stdout)?;
```

### Error Handling

Uses `anyhow::Result` for most operations. Custom error types in `error.rs`.

### Logging

Uses `tracing` crate. Initialize in `main.rs`:
```rust
tracing_subscriber::fmt::init();
```

## Testing Notes

- Integration tests require `gh` CLI authenticated
- Tests in `github/mod.rs` expect the `softwarewrighter` account to have repositories
- Database tests use temporary files via `tempfile` crate
- WASM UI has no tests currently (all logic is in single file)

## Important Configuration

### Database Location
- Default: `~/.overall/overall.db`
- Created automatically on first run with schema from `storage/schema.sql`

### Static Files
- Location: `static/`
- Required files: `index.html`, `repos.json` (generated), `pkg/` (WASM build output)
- Icons: `static/icons/` directory

### Build Artifacts
- WASM output: `wasm-ui/pkg/` → copied to `static/pkg/`
- Binary: `target/release/overall`
- Web server serves from `./static/` directory

## Development Process (CRITICAL - READ FIRST)

**ALWAYS follow these processes**. See `docs/ai_agent_instructions.md` and `docs/learnings.md` for details.

### Test-Driven Development (TDD) - NON-NEGOTIABLE

1. **Write tests FIRST** - never write implementation before tests
2. **Red → Green → Refactor** cycle:
   - Red: Write failing test
   - Green: Implement minimal code to pass
   - Refactor: Improve while keeping tests green

### Checkpoint Process - Before EVERY Commit

```bash
# 1. Run ALL tests (MUST PASS)
cargo test --all

# 2. Fix ALL clippy warnings (ZERO warnings)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Format code
cargo fmt --all

# 4. Build (must succeed)
./scripts/build-all.sh

# 5. Review changes
git status
git diff

# 6. Stage and commit
git add <files>
git commit -m "..."
git push
```

**NEVER skip any step**. If a step fails, fix it before proceeding.

### API Development

When adding API endpoints:
1. Define types/contracts first
2. Write tests FIRST (success, validation, errors)
3. Implement to make tests pass
4. Run integration tests
5. Document endpoint
6. Only then update UI

**NEVER**:
- Add endpoints without tests
- Test only via UI
- Skip error handling
- Commit untested code

## Release Process

The build includes cache-busting for static assets. Build date, git commit, and version are embedded at compile time (check `overall-cli/src/server/mod.rs` for build info generation).

When building for release:
1. Run `./scripts/build-all.sh`
2. Binary is ready at `target/release/overall`
3. Static files are ready in `static/`
4. Can run `./target/release/overall serve` from any directory (will serve from `./static/`)

## Important References

- **Development Process**: `docs/ai_agent_instructions.md` - Full process guidelines
- **Learnings**: `docs/learnings.md` - Historical mistakes and solutions
- **Architecture**: `docs/architecture.md` - System design details
- **Status**: `docs/status.md` - Current progress
