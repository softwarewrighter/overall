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

### UI Design: Status Icon System

**CRITICAL**: The UI uses PNG icons from `static/icons/` for status visualization. DO NOT regress to text/emoji icons.

#### Icon Files (static/icons/)
- `complete.png` - ✓ Green checkmark (all clean, no pending work)
- `local-changes.png` - 📝 White document (uncommitted files)
- `needs-sync.png` - 🔴 Red circle (unpushed commits or behind remote)
- `stale.png` - ⚠️ Gray warning (old branches/PRs, no recent activity)

#### Status Priority (Worst-Case Wins)

**CRITICAL**: When a repo has MULTIPLE statuses, show the most urgent action needed.

**Workflow priority** (check in this order):
1. **local-changes** (Priority 0 - YELLOW/WARNING): uncommitted_files > 0 ← **CHECK FIRST** (commit before push!)
2. **needs-sync** (Priority 1 - RED/CRITICAL): unpushed_commits > 0 OR behind_commits > 0 **OR** repo.branches has any branch with ahead > 0 or behind > 0
3. **stale** (Priority 2 - WHITE/INFO): unmerged_count > 0 (old feature branches)
4. **complete** (Priority 3 - GREEN/SUCCESS): No pending work

**Rationale**: Uncommitted files must be committed before pushing, so local-changes takes priority over needs-sync.

**CRITICAL - Dual Status Checking**:
- **LOCAL status**: From `/api/local-repos/status` endpoint (filesystem git working directory)
  - `uncommitted_files`, `unpushed_commits`, `behind_commits`
- **GITHUB status**: From `repos.json` (GitHub API branch data)
  - `repo.branches[].ahead`, `repo.branches[].behind`
- **MUST check BOTH sources** for needs-sync: A repo with clean local working directory may still have branches that are ahead/behind on GitHub!

**Example 1**: Repo has 15 uncommitted files AND 1 unpushed commit → Show **local-changes** (yellow)
**Example 2**: Tab has repos with local-changes, stale, and complete → Tab shows **local-changes** (yellow)
**Example 3**: Repo has clean working directory (0 uncommitted, 0 unpushed) BUT has branch with behindBy: 34 on GitHub → Show **needs-sync** (red)

#### Tab Icon Display Rules

**Tab icons show the worst-case status** across ALL repos in that tab:
- If ANY repo has needs-sync (priority 0) → tab shows needs-sync icon
- Else if ANY repo has local-changes (priority 1) → tab shows local-changes icon
- Else if ANY repo has stale (priority 2) → tab shows stale icon
- Else (all complete) → tab shows complete icon

**Tab hover text shows ONLY the worst-case reason**:
- needs-sync: "Has uncommitted, unpushed, or unfetched commits"
- local-changes: "Has local uncommitted changes"
- stale: "Has unmerged feature branches"
- complete: "All repositories up to date"

DO NOT show multiple reasons in hover text - only the single worst-case reason.

#### Row Icon Display Rules

Row icons show the specific status of that individual repository:
- Check local status first (needs-sync > local-changes)
- Then check GitHub status (stale if unmerged_count > 0)
- Show complete only if clean on both local and GitHub

#### Implementation Location

- Tab status calculation: `wasm-ui/src/lib.rs` in App component (tabs rendering section)
- Row status calculation: `wasm-ui/src/lib.rs` in RepoRow component
- Icon rendering: Use `<img>` tags with src="/icons/{status}.png"
- Example: `<img class="tab-status-icon" src="/icons/needs-sync.png" alt="Needs sync" />`

#### Common Bugs to Avoid

1. ❌ Using emoji/text instead of PNG icons
2. ❌ Showing multiple reasons in tab hover text (only show worst-case)
3. ❌ Not calculating worst-case priority correctly
4. ❌ Forgetting to exclude main/master/develop from unmerged_count
5. ❌ **Only checking LOCAL status** - MUST check both local_status AND repo.branches for needs-sync!

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

## Testing Strategy (CRITICAL - READ FIRST)

**See `docs/testing.md` for complete testing patterns and examples.**

### Core Principles

1. **Test EVERYTHING** - Every feature, edge case, error condition
2. **Dependency Injection** - Use traits and mocks to test without external dependencies
3. **TDD Always** - Write tests before implementation (Red → Green → Refactor)
4. **No Integration Dependency** - Tests must NOT require `gh` CLI, Ollama, or network access

### Test Infrastructure Requirements

#### Mock/Spy Pattern for GitHub Operations

ALL GitHub operations must go through a trait that can be mocked:

```rust
// overall-cli/src/github/trait.rs
pub trait GitHubClient {
    fn list_repos(&self, owner: &str) -> Result<Vec<Repository>>;
    fn fetch_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>>;
    fn create_pull_request(&self, request: &PrRequest) -> Result<PullRequest>;
    // ... etc
}

// Real implementation wraps gh CLI
pub struct RealGitHubClient;

// Test implementation returns canned data
pub struct MockGitHubClient {
    pub repos: Vec<Repository>,
    pub branches: HashMap<String, Vec<Branch>>,
    // ... etc
}
```

#### Database Test Fixtures

Use builder pattern for test data:

```rust
// overall-cli/src/test_support/fixtures.rs
pub struct TestDatabase {
    db: Database,
    temp_file: TempPath,
}

impl TestDatabase {
    pub fn new() -> Self { /* ... */ }

    pub fn with_repo(&mut self, builder: RepoBuilder) -> &mut Self {
        // Insert repo with specified state
    }

    pub fn with_group(&mut self, name: &str, repos: Vec<i64>) -> &mut Self {
        // Create group and associate repos
    }
}

pub struct RepoBuilder {
    name: String,
    owner: String,
    has_unmerged: bool,
    has_uncommitted: bool,
    has_unpushed: bool,
    // ... etc
}
```

### Required Test Coverage

Every feature must have tests for:

1. **Happy path** - Normal successful operation
2. **Edge cases** - Empty inputs, boundary conditions, unusual combinations
3. **Error conditions** - Invalid inputs, missing data, constraint violations
4. **State transitions** - How operations change system state
5. **Combinations** - Multiple repos with different states in different groups

### Example Test Scenarios

```rust
#[test]
fn test_create_pr_skips_repos_with_open_prs() {
    let mut db = TestDatabase::new()
        .with_repo(RepoBuilder::new("repo1").with_open_pr())
        .with_repo(RepoBuilder::new("repo2").ready_for_pr())
        .build();

    let mock_gh = MockGitHubClient::new()
        .expect_no_call_for("repo1")
        .expect_create_pr("repo2");

    let result = create_all_prs(&db, &mock_gh)?;

    assert_eq!(result.skipped, vec!["repo1"]);
    assert_eq!(result.created, vec!["repo2"]);
    mock_gh.verify();
}
```

### Test Organization

- **Unit tests**: In same file as implementation (`#[cfg(test)] mod tests`)
- **Integration tests**: In `tests/` directory (for cross-module scenarios)
- **Test support**: In `overall-cli/src/test_support/` (fixtures, builders, mocks)
- **Mock implementations**: In `overall-cli/src/github/mock.rs` and similar

### Running Tests

```bash
# Run all tests (MUST PASS before commit)
cargo test --all

# Run specific test
cargo test test_create_pr_skips_repos_with_open_prs

# Run with output
cargo test -- --nocapture

# Run tests in specific package
cargo test -p overall-cli
```

### Legacy Test Notes (TO BE MIGRATED)

- ❌ OLD: Integration tests require `gh` CLI authenticated
- ❌ OLD: Tests in `github/mod.rs` expect the `softwarewrighter` account to have repositories
- ✅ Database tests use temporary files via `tempfile` crate
- ❌ WASM UI has no tests currently (all logic is in single file) - NEEDS TESTS

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

### Always Use Scripts - NEVER Bare Commands

**Use scripts for ALL operations**:
- ✅ `./scripts/build-all.sh` - Build everything
- ✅ `./scripts/serve.sh [port] [--debug]` - Start server
- ✅ `./scripts/check-setup.sh` - Verify prerequisites
- ❌ NEVER run `cargo build`, `./target/release/overall`, or any direct commands

**Why**: Reproducibility, documentation, consistency, automation

**If a script doesn't support what you need**: Update the script, don't bypass it

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
- **Testing Strategy**: `docs/testing.md` - Test patterns, mocks, fixtures, and examples
- **Learnings**: `docs/learnings.md` - Historical mistakes and solutions
- **Architecture**: `docs/architecture.md` - System design details
- **Status**: `docs/status.md` - Current progress
