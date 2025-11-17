# Design Document

## Design Decisions and Rationale

This document captures the key technical design decisions made for the GitHub Repository Manager project, along with the reasoning behind each choice.

## Technology Choices

### Decision 1: Yew Framework for Frontend

**Choice**: Use Yew (Rust WASM framework) instead of React/Vue/Svelte

**Rationale**:
1. **Single Language**: Entire codebase in Rust (frontend + backend)
2. **Type Safety**: Compile-time guarantees across full stack
3. **Performance**: WASM performance comparable to native
4. **Learning Goal**: Gain experience with Rust web development
5. **Minimal JavaScript**: Aligns with project philosophy from docs/ai_agent_instructions.md

**Trade-offs**:
- Smaller ecosystem than React
- Steeper learning curve for web developers
- Larger initial binary size (~2MB compressed)
- Limited third-party component libraries

**Alternatives Considered**:
- **Leptos**: More modern, but less mature than Yew
- **React**: Larger ecosystem, but requires JavaScript
- **Tauri + Yew**: Considered for desktop app, deferred to v2.0

**References**:
- Yew docs: https://yew.rs
- Yew vs Leptos comparison: https://www.lpalmieri.com/posts/yew-vs-leptos/

### Decision 2: gh CLI for GitHub API Access

**Choice**: Use `gh` CLI tool via std::process::Command instead of REST/GraphQL API directly

**Rationale**:
1. **Authentication Handled**: gh manages OAuth tokens, no credential storage needed
2. **Rate Limiting**: gh respects rate limits automatically
3. **Tested Tool**: gh CLI is well-maintained by GitHub
4. **User Familiarity**: Developers already have gh installed and authenticated
5. **Simpler Code**: No need to implement OAuth flow or API client

**Trade-offs**:
- Dependency on external binary
- Slightly slower than direct API calls
- Less control over request formatting
- Requires parsing JSON output

**Alternatives Considered**:
- **octocrab crate**: Rust GitHub API client, but requires credential management
- **graphql_client**: More efficient queries, but complex setup
- **reqwest + manual**: Full control, but significant development overhead

**Implementation**:
```rust
use std::process::Command;

pub fn list_repos(owner: &str) -> Result<Vec<Repository>> {
    let output = Command::new("gh")
        .args(&["repo", "list", owner, "--limit", "50", "--json",
                "name,owner,pushedAt,language,description"])
        .output()?;

    let json = String::from_utf8(output.stdout)?;
    let repos: Vec<Repository> = serde_json::from_str(&json)?;
    Ok(repos)
}
```

### Decision 3: Local Ollama + ask CLI for AI Analysis

**Choice**: Use local Ollama via `ask` CLI instead of cloud APIs (OpenAI, Anthropic)

**Rationale**:
1. **Privacy**: Code and repository data stays local
2. **Cost**: No per-request charges
3. **Offline**: Works without internet after models downloaded
4. **Control**: User chooses models and prompts
5. **Tool Reuse**: ask CLI already built and available

**Trade-offs**:
- Requires Ollama installation and running service
- Slower inference than cloud APIs (especially large models)
- Model quality varies (phi3 vs GPT-4)
- Limited context window (phi3: 4k tokens)

**Alternatives Considered**:
- **OpenAI API**: Better quality, but costs money and sends code to cloud
- **Anthropic API**: Good for code, but requires API key and network
- **Embedded LLM**: llama.cpp in Rust, but complex integration
- **No AI**: Simpler, but loses key differentiator

**Model Selection**:
- **Default**: phi3:3.8b (fast, good quality, 4k context)
- **Alternative**: codellama:7b (code-specialized, slower)
- **Future**: deepseek-coder:6.7b (excellent for code, larger)

### Decision 4: SQLite for Local Storage

**Choice**: Use SQLite with rusqlite instead of JSON files or PostgreSQL

**Rationale**:
1. **Single File**: Easy backup, migration, portable
2. **ACID**: Transactions, crash recovery
3. **Performance**: Fast queries, indexes
4. **Embeddable**: No separate server process
5. **Mature**: Well-tested, stable

**Trade-offs**:
- Less scalable than PostgreSQL (but sufficient for 500 repos)
- Single writer at a time (not an issue for local app)
- No built-in replication

**Alternatives Considered**:
- **JSON files**: Simple, but slow queries, no transactions
- **PostgreSQL**: More scalable, but requires separate server
- **sled**: Rust-native, but less mature
- **Browser LocalStorage**: Limited size, frontend-only

**Schema Design Principles**:
1. Normalize data to avoid duplication
2. Foreign keys for referential integrity
3. Indexes on frequently queried columns
4. Cache AI results with timestamps
5. Store configuration in separate table

### Decision 5: Trunk vs Custom Build Scripts

**Choice**: Use custom build scripts (scripts/build-all.sh) instead of Trunk bundler

**Rationale**:
1. **Consistency**: Aligns with existing project pattern (from docs/process.md)
2. **Control**: Explicit steps (build CLI, build WASM, copy files)
3. **Familiarity**: Similar to other softwarewrighter projects
4. **Debugging**: Easier to debug build issues

**Trade-offs**:
- Manual script maintenance
- Less automatic optimization than Trunk
- No hot-reload during development
- Platform-specific scripts (bash)

**Alternatives Considered**:
- **Trunk**: Modern Yew bundler, but hides build steps
- **wasm-pack + webpack**: More complex configuration
- **cargo-make**: Task runner, but adds dependency

**Build Process**:
```bash
#!/bin/bash
# scripts/build-all.sh

set -euo pipefail

echo "Building Rust CLI..."
cargo build --release

echo "Building WASM UI..."
cd wasm-ui
wasm-pack build --target web --release

echo "Copying assets..."
cp -r pkg ../static/wasm/
echo "Build complete!"
```

## Architecture Patterns

### Pattern 1: Separation of Concerns

**Structure**:
```
overall/
+-- src/                    # Backend (Rust CLI)
|   +-- github/             # GitHub API integration
|   +-- ai/                 # Ollama integration
|   +-- storage/            # SQLite database
|   +-- analysis/           # Priority calculation
|   +-- lib.rs              # Public API
|   +-- main.rs             # CLI entry point
+-- wasm-ui/                # Frontend (Yew WASM)
    +-- src/
        +-- components/     # UI components
        +-- api/            # Backend calls
        +-- state/          # App state management
        +-- lib.rs          # Yew app entry point
```

**Rationale**:
- Clear boundaries between layers
- Frontend can be developed independently
- Backend can be reused for CLI or API server
- Easy to test each module in isolation

### Pattern 2: Repository Pattern for Data Access

**Implementation**:
```rust
// src/storage/repository.rs

pub trait RepositoryStorage {
    fn save(&self, repo: &Repository) -> Result<()>;
    fn find_by_id(&self, id: &str) -> Result<Option<Repository>>;
    fn find_all(&self) -> Result<Vec<Repository>>;
    fn find_recent(&self, limit: usize) -> Result<Vec<Repository>>;
    fn delete(&self, id: &str) -> Result<()>;
}

pub struct SqliteRepositoryStorage {
    conn: Connection,
}

impl RepositoryStorage for SqliteRepositoryStorage {
    fn save(&self, repo: &Repository) -> Result<()> {
        // SQL insert/update
    }
    // ... other methods
}
```

**Rationale**:
- Abstraction over storage implementation
- Easy to mock for testing
- Can swap SQLite for PostgreSQL later
- Clear API for data operations

### Pattern 3: Command Pattern for GitHub Operations

**Implementation**:
```rust
// src/github/commands.rs

pub trait GitHubCommand {
    type Output;
    fn execute(&self) -> Result<Self::Output>;
}

pub struct ListReposCommand {
    owner: String,
    limit: usize,
}

impl GitHubCommand for ListReposCommand {
    type Output = Vec<Repository>;

    fn execute(&self) -> Result<Vec<Repository>> {
        let output = Command::new("gh")
            .args(&["repo", "list", &self.owner])
            .output()?;
        // Parse and return
    }
}
```

**Rationale**:
- Encapsulates gh CLI invocations
- Easy to add new commands
- Testable with mocks
- Retryable on failure

### Pattern 4: Observer Pattern for UI Updates

**Implementation**:
```rust
// wasm-ui/src/state/mod.rs

use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct AppState {
    pub repositories: Vec<Repository>,
    pub loading: bool,
    pub error: Option<String>,
}

pub enum AppAction {
    RefreshStart,
    RefreshComplete(Vec<Repository>),
    RefreshError(String),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AppAction::RefreshStart => Rc::new(AppState {
                loading: true,
                ..(*self).clone()
            }),
            AppAction::RefreshComplete(repos) => Rc::new(AppState {
                repositories: repos,
                loading: false,
                error: None,
            }),
            AppAction::RefreshError(err) => Rc::new(AppState {
                loading: false,
                error: Some(err),
                ..(*self).clone()
            }),
        }
    }
}
```

**Rationale**:
- Centralized state management
- Predictable state updates
- Easy to debug state transitions
- Components automatically re-render

## Data Models

### Domain Models

```rust
// src/models/repository.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,              // owner/name
    pub owner: String,
    pub name: String,
    pub language: Option<String>,
    pub description: Option<String>,
    pub pushed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_fork: bool,
    pub priority: f32,           // Calculated 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: i64,
    pub repo_id: String,
    pub name: String,
    pub sha: String,
    pub ahead_by: u32,
    pub behind_by: u32,
    pub status: BranchStatus,
    pub last_commit_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStatus {
    ReadyForPR,
    InReview,
    ReadyToMerge,
    NeedsUpdate,
    HasConflicts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub repo_id: String,
    pub branch_id: Option<i64>,
    pub number: u32,
    pub state: PRState,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PRState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysis {
    pub id: i64,
    pub repo_id: String,
    pub priority: u8,            // 1-10
    pub focus_branch: Option<String>,
    pub actions: Vec<String>,
    pub created_at: DateTime<Utc>,
}
```

### View Models (for UI)

```rust
// wasm-ui/src/models/view_models.rs

#[derive(Clone, PartialEq)]
pub struct RepositoryViewModel {
    pub id: String,
    pub display_name: String,    // "owner/repo"
    pub language: String,        // "Rust" or "Unknown"
    pub last_activity: String,   // "2 days ago"
    pub priority_label: String,  // "High", "Medium", "Low"
    pub priority_color: String,  // "green", "yellow", "red"
    pub branch_count: usize,
    pub ready_for_pr: usize,
    pub in_review: usize,
    pub ai_summary: String,      // First line of AI suggestions
}

impl From<(Repository, Vec<Branch>, Option<AIAnalysis>)> for RepositoryViewModel {
    fn from((repo, branches, ai): (Repository, Vec<Branch>, Option<AIAnalysis>)) -> Self {
        let ready_count = branches.iter()
            .filter(|b| b.status == BranchStatus::ReadyForPR)
            .count();

        let review_count = branches.iter()
            .filter(|b| b.status == BranchStatus::InReview)
            .count();

        RepositoryViewModel {
            id: repo.id.clone(),
            display_name: format!("{}/{}", repo.owner, repo.name),
            language: repo.language.unwrap_or("Unknown".to_string()),
            last_activity: format_relative_time(repo.pushed_at),
            priority_label: priority_to_label(repo.priority),
            priority_color: priority_to_color(repo.priority),
            branch_count: branches.len(),
            ready_for_pr: ready_count,
            in_review: review_count,
            ai_summary: ai.and_then(|a| a.actions.first().cloned())
                           .unwrap_or("No analysis yet".to_string()),
        }
    }
}
```

## Error Handling Strategy

### Error Types

```rust
// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("GitHub CLI error: {0}")]
    GitHubCLI(String),

    #[error("GitHub API error: {status} - {message}")]
    GitHubAPI { status: u16, message: String },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("AI service unavailable: {0}")]
    AIUnavailable(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### Error Recovery Strategies

| Error Type | Recovery Strategy | User Message |
|------------|-------------------|--------------|
| GitHubCLI | Retry 3x, then fail | "Unable to connect to GitHub. Please check your gh CLI authentication." |
| GitHubAPI (Rate Limit) | Wait for reset time | "GitHub rate limit reached. Retrying in {minutes} minutes..." |
| Database | Attempt repair, recreate if needed | "Database error. Rebuilding from cache..." |
| AIUnavailable | Skip AI features | "AI analysis unavailable. Ollama service may not be running." |
| Config | Use defaults | "Configuration invalid. Using default settings." |

### Logging Strategy

```rust
// Use tracing crate for structured logging

use tracing::{info, warn, error, debug};

pub fn scan_repositories(owner: &str) -> Result<Vec<Repository>> {
    info!("Starting repository scan for {}", owner);

    match list_repos_command(owner).execute() {
        Ok(repos) => {
            info!("Found {} repositories for {}", repos.len(), owner);
            Ok(repos)
        }
        Err(e) => {
            error!("Failed to scan repositories for {}: {}", owner, e);
            Err(e)
        }
    }
}
```

## Security Considerations

### Authentication Flow

1. User authenticates `gh` CLI separately (one-time setup)
2. Application invokes `gh` commands, which use stored credentials
3. No credentials stored or handled by this application
4. Token refresh handled by gh CLI

**Security Benefits**:
- No credential storage in app
- No OAuth implementation needed
- Leverages gh CLI's security best practices
- Tokens stored in OS keychain by gh

### Data Privacy

1. **Local Storage Only**: All data stored in local SQLite database
2. **No Cloud Sync**: No data sent to external servers
3. **AI Privacy**: Ollama runs locally, no code sent to cloud LLMs
4. **WASM Sandbox**: Frontend runs in browser sandbox

### Input Validation

```rust
// Validate user inputs to prevent injection

pub fn validate_owner(owner: &str) -> Result<()> {
    if owner.is_empty() {
        return Err(AppError::Config("Owner cannot be empty".to_string()));
    }

    // GitHub usernames: alphanumeric, hyphens, max 39 chars
    if !owner.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(AppError::Config("Invalid owner name".to_string()));
    }

    if owner.len() > 39 {
        return Err(AppError::Config("Owner name too long".to_string()));
    }

    Ok(())
}
```

## Performance Optimizations

### 1. Lazy Loading

- Load repository list first (fast)
- Fetch branch details on-demand (when user expands)
- AI analysis triggered in background, not blocking UI

### 2. Caching Strategy

```rust
pub struct CachePolicy {
    repository_ttl: Duration,      // 1 hour
    branch_ttl: Duration,          // 30 minutes
    ai_analysis_ttl: Duration,     // 24 hours
}

impl CachePolicy {
    pub fn is_fresh<T>(&self, cached: &CachedItem<T>) -> bool {
        let age = Utc::now() - cached.timestamp;
        match cached.item_type {
            ItemType::Repository => age < self.repository_ttl,
            ItemType::Branch => age < self.branch_ttl,
            ItemType::AIAnalysis => age < self.ai_analysis_ttl,
        }
    }
}
```

### 3. Batch Operations

```rust
// Batch GitHub API calls to reduce roundtrips

pub async fn fetch_all_branches(repos: &[Repository]) -> Result<Vec<(String, Vec<Branch>)>> {
    use futures::future::join_all;

    let futures: Vec<_> = repos.iter()
        .map(|repo| fetch_branches(&repo.id))
        .collect();

    join_all(futures).await
        .into_iter()
        .collect()
}
```

### 4. Database Indexing

```sql
CREATE INDEX idx_repositories_pushed_at ON repositories(pushed_at DESC);
CREATE INDEX idx_repositories_priority ON repositories(priority DESC);
CREATE INDEX idx_branches_repo_id ON branches(repo_id);
CREATE INDEX idx_branches_status ON branches(status);
CREATE INDEX idx_ai_analysis_repo_id ON ai_analysis(repo_id);
CREATE INDEX idx_ai_analysis_created_at ON ai_analysis(created_at DESC);
```

### 5. WASM Optimization

```toml
# Cargo.toml for wasm-ui

[profile.release]
opt-level = 'z'        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
panic = 'abort'        # Smaller binary
strip = true           # Remove debug symbols
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_priority() {
        let repo = Repository {
            pushed_at: Utc::now() - Duration::days(2),
            // ... other fields
        };

        let priority = calculate_priority(&repo, &[], None);
        assert!(priority > 0.5);
    }

    #[test]
    fn test_validate_owner() {
        assert!(validate_owner("softwarewrighter").is_ok());
        assert!(validate_owner("invalid@name").is_err());
        assert!(validate_owner("").is_err());
    }
}
```

### Integration Tests

```rust
// tests/github_integration.rs

#[test]
fn test_list_repos_integration() {
    let repos = list_repos("softwarewrighter").unwrap();
    assert!(!repos.is_empty());
    assert!(repos.iter().any(|r| r.owner == "softwarewrighter"));
}
```

### Mock Strategy

```rust
// src/github/mock.rs

pub struct MockGitHubClient {
    repos: Vec<Repository>,
}

impl GitHubCommand for MockGitHubClient {
    type Output = Vec<Repository>;

    fn execute(&self) -> Result<Self::Output> {
        Ok(self.repos.clone())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_with_mock() {
        let mock = MockGitHubClient {
            repos: vec![/* test data */],
        };

        let result = process_repositories(mock);
        assert_eq!(result.len(), 1);
    }
}
```

## Configuration Management

### Configuration File Format

```toml
# ~/.config/overall/config.toml

version = "1.0"

[github]
# List of GitHub users/organizations to track
owners = [
    "softwarewrighter",
    # "other-org",
]

# Limit per owner (max 50)
repo_limit = 50

[ai]
# AI platform (currently only "ollama")
platform = "ollama"

# Ollama model name
model = "phi3:3.8b"

# Analysis timeout in seconds
timeout = 30

# Maximum concurrent analyses
max_concurrent = 3

# AI analysis cache duration in hours
cache_hours = 24

[storage]
# Database file path (supports ~/ expansion)
database_path = "~/.local/share/overall/repo_manager.db"

[ui]
# UI theme ("light" or "dark")
theme = "dark"

# Default sort column
default_sort = "priority"

# Items per page
items_per_page = 50

[refresh]
# Auto-refresh interval ("manual", "hourly", "daily")
interval = "manual"
```

### Configuration Loading

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub github: GitHubConfig,
    pub ai: AIConfig,
    pub storage: StorageConfig,
    pub ui: UIConfig,
    pub refresh: RefreshConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or(AppError::Config("Cannot find home directory".to_string()))?;
        Ok(home.join(".config/overall/config.toml"))
    }

    fn validate(&self) -> Result<()> {
        if self.github.owners.is_empty() {
            return Err(AppError::Config("No GitHub owners configured".to_string()));
        }

        for owner in &self.github.owners {
            validate_owner(owner)?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: "1.0".to_string(),
            github: GitHubConfig {
                owners: vec!["softwarewrighter".to_string()],
                repo_limit: 50,
            },
            ai: AIConfig {
                platform: "ollama".to_string(),
                model: "phi3:3.8b".to_string(),
                timeout: 30,
                max_concurrent: 3,
                cache_hours: 24,
            },
            storage: StorageConfig {
                database_path: "~/.local/share/overall/repo_manager.db".to_string(),
            },
            ui: UIConfig {
                theme: "dark".to_string(),
                default_sort: "priority".to_string(),
                items_per_page: 50,
            },
            refresh: RefreshConfig {
                interval: "manual".to_string(),
            },
        }
    }
}
```

## Deployment Considerations

### Binary Distribution

1. **Release Build**:
   ```bash
   ./scripts/build-all.sh
   ```

2. **Package Structure**:
   ```
   overall-v1.0.0/
   +-- overall                  # CLI binary
   +-- static/
       +-- index.html           # WASM loader
       +-- wasm/
           +-- overall_ui_bg.wasm
           +-- overall_ui.js
   ```

3. **Installation**:
   ```bash
   # Using sw-install
   sw-install -p ~/projects/overall

   # Manual
   cp overall ~/.local/bin/
   cp -r static ~/.local/share/overall/
   ```

### Environment Setup

Prerequisites checklist:
- [ ] Rust installed (rustc --version)
- [ ] gh CLI installed and authenticated (gh auth status)
- [ ] Ollama installed and running (ollama list)
- [ ] ask CLI in PATH (which ask)

## Future Enhancements (Design Notes)

### v1.5: Web Service Architecture

```
Browser -> Actix Web -> Service Layer -> GitHub API
                     -> Storage Layer -> PostgreSQL
                     -> AI Layer -> Ollama (self-hosted)
```

### v2.0: Tauri Desktop App

Replace WASM frontend with Tauri:
- Native performance
- OS integration (notifications, tray icon)
- Automatic updates
- Single binary distribution

### v3.0: Multi-user Collaboration

Add:
- User authentication
- Shared team dashboards
- Role-based access control
- Real-time updates (WebSockets)

---

## REST API Endpoints

**Source**: `overall-cli/src/server/mod.rs` lines 122-139

### Groups Management
- `GET /api/groups` - List all repository groups
- `POST /api/groups/add-repos` - Add repositories to a group
- `POST /api/groups/delete/:id` - Delete a group

### Repository Operations
- `POST /api/repos/move` - Move repository between groups
- `POST /api/repos/export` - Export repositories to JSON
- `POST /api/repos/sync-all` - Sync all repositories from GitHub
- `POST /api/repos/sync` - Sync single repository from GitHub ✅ **EXISTS**

### Pull Request Management
- `POST /api/pr/create` - Create pull request for single branch
- `POST /api/pr/create-all` - Create pull requests for all branches in repo

### Build Info
- `GET /api/build-info` - Get build metadata (version, commit, date)

### Local Repository Monitoring
- `GET /api/local-repos/roots` - List local repository roots
- `POST /api/local-repos/roots` - Add local repository root path
- `POST /api/local-repos/roots/toggle/:id` - Enable/disable root
- `POST /api/local-repos/scan` - Scan local repositories for uncommitted/unpushed changes
- `GET /api/local-repos/status` - Get status of all local repositories

### Static Files
- `GET /repos.json` - Cached repository data (generated by export)
- `GET /build-info.json` - Build metadata
- `GET /icons/*.png` - Status icons (needs-sync, local-changes, stale, complete)
- `GET /wasm/*` - WebAssembly UI files
