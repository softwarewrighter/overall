# Architecture

## System Overview

The GitHub Repository Manager is a Rust/Yew web application that helps developers track and prioritize their GitHub repositories across multiple users and organizations. It identifies unmerged branches that are ready for pull requests or merges, using local AI (Ollama) to analyze projects and suggest next steps.

## High-Level Architecture

```
+------------------+     +-------------------+     +------------------+
|                  |     |                   |     |                  |
|  Yew Frontend    +---->+  Rust Backend     +---->+  GitHub API      |
|  (WASM)          |     |  (CLI/Service)    |     |  (gh CLI)        |
|                  |     |                   |     |                  |
+--------+---------+     +---------+---------+     +------------------+
         |                         |
         |                         |
         v                         v
+------------------+     +-------------------+
|                  |     |                   |
|  Browser         |     |  Local Storage    |
|  LocalStorage    |     |  (SQLite)         |
|                  |     |                   |
+------------------+     +---------+---------+
                                   |
                                   v
                         +-------------------+
                         |                   |
                         |  Ollama LLM       |
                         |  (ask CLI)        |
                         |                   |
                         +-------------------+
```

## Components

### 1. Yew Frontend (Web UI)

**Technology**: Rust + Yew framework (WASM)

**Responsibilities**:
- Display repository list with filtering and sorting
- Show branch status and merge readiness
- Present AI-generated suggestions
- Handle user interactions (refresh, prioritize, analyze)
- Responsive UI with real-time updates

**Key Features**:
- Table view with sortable columns:
  - Repository name/owner
  - Last activity timestamp
  - Number of unmerged branches
  - Branch statuses (ready for PR, ready to merge, in review)
  - AI suggestions summary
- Detail view for individual repositories:
  - Branch comparison details
  - Merge conflicts indication
  - Commit history
  - AI analysis results
- Filter controls:
  - By owner/organization
  - By activity date
  - By merge status
  - Show only top 50 most recent

**State Management**:
- Repository data cached in browser LocalStorage
- Periodic background refresh (configurable)
- Optimistic UI updates

### 2. Rust Backend (CLI/Service)

**Technology**: Rust with tokio async runtime

**Responsibilities**:
- Execute gh CLI commands to fetch repository data
- Parse and process GitHub API responses
- Manage local SQLite database
- Coordinate AI analysis via ask CLI
- Expose API for frontend (via wasm-bindgen)

**Key Modules**:

#### GitHub Integration Module
```rust
// src/github/mod.rs
- list_repos(owners: Vec<String>) -> Vec<Repository>
- get_branches(repo: &Repository) -> Vec<Branch>
- check_pr_status(branch: &Branch) -> PRStatus
- get_recent_commits(branch: &Branch) -> Vec<Commit>
```

#### Repository Analysis Module
```rust
// src/analysis/mod.rs
- analyze_branch(branch: &Branch) -> BranchAnalysis
- detect_merge_readiness(branch: &Branch) -> MergeStatus
- calculate_priority(repo: &Repository) -> f32
```

#### AI Integration Module
```rust
// src/ai/mod.rs
- query_ollama(prompt: String) -> String
- analyze_project(repo: &Repository) -> ProjectAnalysis
- suggest_next_steps(repo: &Repository) -> Vec<Suggestion>
```

#### Storage Module
```rust
// src/storage/mod.rs
- save_repositories(repos: Vec<Repository>)
- load_repositories() -> Vec<Repository>
- update_analysis(repo_id: String, analysis: Analysis)
```

### 3. GitHub API Integration (gh CLI)

**Tool**: GitHub CLI (gh)

**Usage**:
- `gh repo list <owner> --limit 50 --json <fields>` - List repositories
- `gh api repos/{owner}/{repo}/branches` - Get branch information
- `gh pr list -R <repo> --json <fields>` - Get pull request status
- `gh api repos/{owner}/{repo}/compare/{base}...{head}` - Compare branches

**Data Flow**:
1. Execute gh command via std::process::Command
2. Capture JSON output
3. Deserialize using serde_json
4. Transform to internal data structures

### 4. Local AI Integration (Ollama via ask CLI)

**Tool**: ask CLI (local Ollama interface)

**Usage**:
```bash
ask -p ollama -m <model> "Analyze this repository and suggest next steps..."
```

**Analysis Prompts**:
- Repository purpose and technology stack
- Branch purpose and completion status
- Merge conflict resolution suggestions
- Priority recommendations based on activity
- Next steps for development

**Models**:
- Default: phi3:3.8b (fast, good for code analysis)
- Optional: codellama, deepseek-coder (specialized code models)

### 5. Data Storage (SQLite)

**Schema**:

```sql
-- Repositories table
CREATE TABLE repositories (
    id TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    last_activity TIMESTAMP,
    is_fork BOOLEAN,
    language TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    pushed_at TIMESTAMP
);

-- Branches table
CREATE TABLE branches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sha TEXT NOT NULL,
    ahead_by INTEGER,
    behind_by INTEGER,
    has_conflicts BOOLEAN,
    last_commit_date TIMESTAMP,
    FOREIGN KEY (repo_id) REFERENCES repositories(id)
);

-- Pull requests table
CREATE TABLE pull_requests (
    id INTEGER PRIMARY KEY,
    repo_id TEXT NOT NULL,
    branch_id INTEGER,
    number INTEGER NOT NULL,
    state TEXT NOT NULL,
    title TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (repo_id) REFERENCES repositories(id),
    FOREIGN KEY (branch_id) REFERENCES branches(id)
);

-- AI analysis cache
CREATE TABLE ai_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id TEXT NOT NULL,
    branch_id INTEGER,
    analysis_type TEXT NOT NULL,
    result TEXT NOT NULL,
    created_at TIMESTAMP,
    FOREIGN KEY (repo_id) REFERENCES repositories(id),
    FOREIGN KEY (branch_id) REFERENCES branches(id)
);

-- Configuration
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

## Data Flow

### Initial Repository Scan

1. User provides list of GitHub users/organizations
2. For each owner:
   - Execute `gh repo list <owner> --limit 50 --json ...`
   - Sort by `pushedAt` to get 50 most recent
3. Store repositories in SQLite
4. For each repository:
   - Fetch all branches
   - Identify branches without PRs
   - Calculate ahead/behind status vs default branch
5. Persist branch data
6. Trigger background AI analysis for top 10 most active repos

### Branch Analysis Workflow

1. Identify unmerged branches (no PR or PR not merged)
2. For each unmerged branch:
   - Check if ahead of base (has new commits)
   - Check if behind base (needs update)
   - Detect merge conflicts via GitHub API
   - Classify status:
     - `ReadyForPR`: Ahead, no conflicts, no existing PR
     - `ReadyToMerge`: PR exists, approved, no conflicts
     - `NeedsUpdate`: Behind base branch
     - `HasConflicts`: Merge conflicts detected
     - `InReview`: PR exists, pending review

### AI Analysis Workflow

1. Select repository for analysis
2. Prepare context prompt:
   ```
   Repository: {owner}/{name}
   Language: {primary_language}
   Last activity: {pushed_at}

   Unmerged branches:
   - {branch_name}: {ahead_by} commits ahead, {behind_by} behind

   Analyze this repository and suggest:
   1. Which branch should be prioritized
   2. Whether branches are feature-complete
   3. Recommended next steps
   ```
3. Execute: `ask -p ollama -m phi3:3.8b "<prompt>"`
4. Parse response and extract:
   - Priority score (1-10)
   - Suggested actions
   - Branch recommendations
5. Cache results in ai_analysis table

### Priority Calculation

Priority score (0.0 - 1.0) based on:
- **Recency** (40%): Days since last push (exponential decay)
- **Activity** (30%): Number of commits in last 30 days
- **Branch count** (20%): Number of unmerged branches ready for PR
- **AI recommendation** (10%): Normalized AI priority score

Formula:
```rust
fn calculate_priority(repo: &Repository) -> f32 {
    let recency = (-0.1 * days_since_push).exp();
    let activity = (commits_last_30d as f32 / 50.0).min(1.0);
    let branches = (ready_branches as f32 / 5.0).min(1.0);
    let ai_score = ai_priority / 10.0;

    0.4 * recency + 0.3 * activity + 0.2 * branches + 0.1 * ai_score
}
```

## Technology Stack

### Frontend
- **Yew**: Rust framework for building web UIs with WASM
- **wasm-bindgen**: Rust/JavaScript interop
- **web-sys**: Web API bindings
- **yew-router**: Client-side routing
- **gloo**: Web platform utilities

### Backend
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **rusqlite**: SQLite database access
- **chrono**: Date/time handling
- **reqwest**: HTTP client (if needed beyond gh CLI)

### Build Tools
- **wasm-pack**: WASM build tool
- **trunk**: Yew application bundler (alternative to custom scripts)
- **cargo**: Rust build system

### External Dependencies
- **gh CLI**: GitHub API access (must be installed and authenticated)
- **ask CLI**: Ollama LLM access (from ~/.local/softwarewrighter/bin/)
- **Ollama**: Local LLM server (must be running)

## Deployment Architecture

### Local Development
```
Developer Machine:
  - Ollama service (localhost:11434)
  - SQLite database (./repo_manager.db)
  - Development server (trunk serve or custom script)
  - gh CLI (authenticated)
  - ask CLI (in PATH)
```

### Production (Local Desktop App)
```
User Machine:
  - Standalone WASM app (served via local web server)
  - SQLite database in user data directory
  - Background service for GitHub polling
  - Ollama service (localhost:11434)
  - gh CLI (authenticated)
```

### Future: Web Service
```
Cloud:
  - Frontend: Static hosting (GitHub Pages, Netlify)
  - Backend: Rust web service (Actix/Axum)
  - Database: PostgreSQL
  - AI: Cloud LLM API (OpenAI, Anthropic) or self-hosted Ollama
  - Auth: GitHub OAuth
```

## Security Considerations

1. **GitHub Authentication**:
   - Uses gh CLI which manages its own authentication
   - No credentials stored in application
   - User must authenticate gh separately

2. **Local AI**:
   - Ollama runs locally, no external API calls
   - No code/data sent to cloud services
   - User controls model selection and data privacy

3. **Data Storage**:
   - SQLite database stored locally
   - No sensitive data cached (only metadata)
   - Repository content not stored, only references

4. **WASM Isolation**:
   - Frontend runs in browser sandbox
   - Limited filesystem access
   - Backend operations via explicit API calls

## Performance Considerations

1. **Rate Limiting**:
   - GitHub API: 5000 requests/hour (authenticated)
   - Batch requests where possible
   - Cache results aggressively

2. **Incremental Updates**:
   - Only fetch new data since last sync
   - Use ETags for conditional requests
   - Background refresh on timer

3. **AI Analysis**:
   - Lazy evaluation (only analyze visible repos)
   - Cache results for 24 hours
   - Parallel analysis with semaphore (max 3 concurrent)

4. **Database Optimization**:
   - Indexes on repo_id, pushed_at, priority
   - Prepared statements for common queries
   - Periodic vacuum/optimize

## Scalability

### Current Scope (v1.0)
- Support 5-10 GitHub users/orgs
- Up to 500 total repositories
- Top 50 most recent repos prioritized
- Single user, local deployment

### Future Growth (v2.0+)
- Multi-user support
- Thousands of repositories
- Distributed caching
- Web service deployment
- Team collaboration features

## Error Handling

### GitHub API Errors
- Network failures: Retry with exponential backoff
- Rate limiting: Queue requests, respect retry-after
- Authentication: Prompt user to re-authenticate gh
- Not found: Mark repo as archived/deleted

### AI Analysis Errors
- Ollama unavailable: Skip AI features, show warning
- Model not found: Fall back to default model
- Timeout: Cancel after 30s, cache partial results

### Database Errors
- Corruption: Attempt repair, fallback to new DB
- Disk full: Warn user, disable caching
- Lock contention: Retry with timeout

## Monitoring and Observability

### Logging
```rust
// Use tracing crate
tracing::info!("Scanning repositories for {}", owner);
tracing::warn!("Rate limit approaching: {} requests remaining", remaining);
tracing::error!("Failed to analyze {}: {}", repo_name, error);
```

### Metrics
- Repositories scanned per session
- Branches analyzed
- AI queries executed
- Cache hit rate
- Average response times

### Health Checks
- GitHub API connectivity
- Ollama service availability
- Database accessibility
- Disk space availability

## Testing Strategy

### Unit Tests
- Repository parsing logic
- Priority calculation
- Date/time utilities
- Data transformations

### Integration Tests
- GitHub API interaction (mocked)
- SQLite operations
- AI prompt generation

### End-to-End Tests
- Full scan workflow
- UI interactions (via Playwright)
- Data persistence
- Error recovery

### Test Data
- Mock GitHub API responses
- Sample repositories with various states
- Pre-populated SQLite fixtures
