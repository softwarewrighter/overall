# Product Requirements Document (PRD)

## Product Overview

**Product Name**: GitHub Repository Manager (working title: "overall")

**Vision**: A smart, AI-powered dashboard that helps developers manage their GitHub repositories by identifying branches ready for pull requests or merges, prioritizing work based on recent activity, and providing AI-driven suggestions for next steps.

**Target Users**:
- Individual developers managing multiple personal repositories
- Open source maintainers tracking projects across organizations
- Software engineers with repositories spread across personal and work accounts
- Primary user: softwarewrighter (GitHub user)

## Problem Statement

### Current Challenges

1. **Repository Sprawl**: Developers often have dozens of repositories across multiple GitHub accounts and organizations
2. **Lost Context**: It's hard to remember which branches have unmerged work
3. **Manual Tracking**: No easy way to see which branches are ready for PRs vs already in review
4. **Prioritization Difficulty**: Hard to decide which repository needs attention first
5. **Context Switching Cost**: Need to visit each repository individually to check status

### User Stories

**As a developer, I want to...**
- See all my repositories in one place, sorted by most recent activity
- Identify which branches are ready to become pull requests
- Know which pull requests are ready to merge
- Get AI suggestions on what to work on next
- Focus on my 50 most recently active repositories to avoid overwhelming data

**As an open source maintainer, I want to...**
- Track repositories across multiple organizations
- Quickly identify stale branches that need attention
- See which branches might have merge conflicts
- Get recommendations on prioritizing work

## Success Criteria

### Must Have (v1.0)
1. Display repositories from configured GitHub users/organizations
2. Show unmerged branches for each repository
3. Indicate which branches are ready for PRs vs already have PRs
4. Sort/filter by most recent activity (last 50 repos only)
5. Use local AI (Ollama) to analyze and suggest next steps
6. Persist data locally (SQLite) for offline viewing
7. One-click refresh to update from GitHub

### Should Have (v1.5)
8. Branch comparison (commits ahead/behind)
9. Merge conflict detection
10. Filter by language, organization, or status
11. AI analysis of individual repositories on demand
12. Configurable list of users/organizations to track

### Could Have (v2.0)
13. Automatic background refresh every N hours
14. GitHub PR creation directly from the UI
15. Branch deletion after merge
16. Team collaboration features
17. Export to CSV/JSON

### Won't Have (v1.0)
- Multi-user support
- Cloud synchronization
- Mobile app
- GitHub Actions integration
- Code review features

## Functional Requirements

### FR1: Repository Discovery

**Description**: Scan and display repositories from configured GitHub accounts

**Acceptance Criteria**:
- User can configure list of GitHub users/organizations to track
- System fetches up to 50 most recent repositories (by `pushedAt` date) per owner
- Repositories are stored in local SQLite database
- Display shows: owner, name, language, last activity date

**Implementation Notes**:
- Use `gh repo list <owner> --limit 50 --json pushedAt,name,owner,language,updatedAt`
- Sort by `pushedAt` descending before limiting to 50
- Default configuration includes: "softwarewrighter"

**Example**:
```
Configuration file: ~/.config/overall/owners.txt
softwarewrighter
other-org
personal-account
```

### FR2: Branch Analysis

**Description**: Identify unmerged branches and their status

**Acceptance Criteria**:
- For each repository, fetch all branches
- Determine default branch (main/master)
- Identify branches without merged PRs
- Show count of unmerged branches per repository
- Classify branches as:
  - Ready for PR (no PR exists, ahead of default)
  - In Review (PR exists, not merged)
  - Ready to Merge (PR approved, no conflicts)
  - Needs Update (behind default branch)
  - Has Conflicts (merge conflicts detected)

**Implementation Notes**:
- Use `gh api repos/{owner}/{repo}/branches`
- Use `gh pr list -R {owner}/{repo}` to check PR status
- Use `gh api repos/{owner}/{repo}/compare/{base}...{head}` for ahead/behind

**Data Model**:
```rust
pub enum BranchStatus {
    ReadyForPR,
    InReview,
    ReadyToMerge,
    NeedsUpdate,
    HasConflicts,
}

pub struct Branch {
    name: String,
    sha: String,
    ahead_by: u32,
    behind_by: u32,
    status: BranchStatus,
    last_commit_date: DateTime<Utc>,
}
```

### FR3: Priority Ranking

**Description**: Calculate and display priority scores for repositories

**Acceptance Criteria**:
- Each repository has a priority score (0.0 - 1.0)
- Score factors in:
  - Recency (40%): Days since last push
  - Activity (30%): Number of commits in last 30 days
  - Branch readiness (20%): Number of branches ready for PR
  - AI recommendation (10%): AI-generated priority
- Repositories sorted by priority by default
- User can re-sort by any column

**Implementation Notes**:
- Calculate priority on each refresh
- Store in database for offline access
- Recalculate when AI analysis completes

**Formula**:
```
recency_score = exp(-0.1 * days_since_push)
activity_score = min(1.0, commits_last_30d / 50.0)
branch_score = min(1.0, ready_branches / 5.0)
ai_score = ai_priority / 10.0

priority = 0.4 * recency + 0.3 * activity + 0.2 * branch + 0.1 * ai
```

### FR4: AI-Powered Analysis

**Description**: Use local Ollama LLM to analyze repositories and suggest actions

**Acceptance Criteria**:
- Integration with `ask` CLI tool (from ~/.local/softwarewrighter/bin/)
- Analyze top 10 repositories by priority on each refresh
- Generate suggestions including:
  - Which branch to work on first
  - Whether branches appear feature-complete
  - Recommended next steps
  - Priority rating (1-10)
- Cache AI results for 24 hours
- Display AI suggestions in repository detail view

**Implementation Notes**:
- Use `ask -p ollama -m phi3:3.8b "<prompt>"`
- Prompt template includes: repo name, language, branch list, commit messages
- Parse response for structured data (priority, actions)
- Run analyses in parallel (max 3 concurrent)
- Timeout after 30 seconds per analysis

**Example Prompt**:
```
Analyze this GitHub repository:

Repository: softwarewrighter/overall
Language: Rust
Last activity: 2 days ago

Branches:
- feature/ai-integration: 5 commits ahead of main
- bugfix/parser-error: 2 commits ahead of main
- experiment/new-ui: 15 commits ahead of main

Recent commits (feature/ai-integration):
- Add Ollama integration
- Implement ask CLI wrapper
- Add tests for AI module

Provide:
1. Priority rating (1-10)
2. Which branch should be worked on first
3. Whether branches appear complete enough for PR
4. Recommended next steps

Format response as:
Priority: <number>
Focus: <branch-name>
Actions:
- <action 1>
- <action 2>
```

### FR5: User Interface

**Description**: Web-based UI built with Yew/WASM

**Acceptance Criteria**:

**Main View (Repository List)**:
- Table with columns:
  - Row number (for easy reference)
  - Repository name (clickable to detail view)
  - Owner
  - Language
  - Last activity (relative time, e.g., "2 days ago")
  - Priority score (color-coded: green >0.7, yellow 0.4-0.7, red <0.4)
  - Unmerged branches count (with status breakdown tooltip)
  - AI suggestions summary (truncated)
- Sortable columns (click header to sort)
- Filter controls:
  - Owner/organization dropdown
  - Language filter
  - Status filter (has ready PRs, has conflicts, etc.)
  - Search by repository name
- Actions:
  - Refresh button (re-scan GitHub)
  - Settings button (configure owners)
  - Export button (download as JSON)

**Detail View (Repository Details)**:
- Repository header:
  - Full name (owner/repo)
  - Description
  - Primary language
  - Stars, forks, watchers
  - Last activity timestamp
  - Link to GitHub (open in browser)
- Branches section:
  - List of unmerged branches
  - Each branch shows:
    - Name
    - Status badge (Ready for PR, In Review, etc.)
    - Ahead/behind counts
    - Last commit message and date
    - Conflict indicator
    - Actions: Create PR, View on GitHub
- AI Analysis section:
  - Priority score with explanation
  - Recommended actions (bulleted list)
  - Next steps
  - Re-analyze button
- Recent commits section:
  - Last 10 commits on default branch
  - Commit message, author, date

**Settings View**:
- List of tracked owners (users/organizations)
- Add/remove owners
- Model selection (phi3:3.8b, codellama, etc.)
- Refresh interval (manual, hourly, daily)
- Database location
- Clear cache button

**Implementation Notes**:
- Use Yew components for modularity
- State management via Yew hooks (use_state, use_effect)
- Routing with yew-router
- Responsive design (CSS Grid/Flexbox)
- Loading states for async operations
- Error messages for failures

### FR6: Data Persistence

**Description**: Store repository data locally for offline access

**Acceptance Criteria**:
- SQLite database at `~/.local/share/overall/repo_manager.db`
- Tables: repositories, branches, pull_requests, ai_analysis, config
- Automatic schema migration on version changes
- Graceful handling of database corruption (recreate)
- Import/export functionality

**Implementation Notes**:
- Use rusqlite crate
- Schema defined in docs/architecture.md
- WAL mode for better concurrency
- Indexes on frequently queried columns
- Regular VACUUM on large datasets

### FR7: Configuration Management

**Description**: User-configurable settings persisted across sessions

**Acceptance Criteria**:
- Configuration file at `~/.config/overall/config.toml`
- Settings include:
  - List of GitHub owners to track
  - Ollama model name
  - Refresh interval
  - UI preferences (theme, default sort)
  - Database location
- Configuration validated on load
- Defaults provided for missing values

**Example config.toml**:
```toml
[github]
owners = ["softwarewrighter", "rust-lang", "yewstack"]

[ai]
platform = "ollama"
model = "phi3:3.8b"
timeout_seconds = 30
max_concurrent = 3

[ui]
theme = "dark"
default_sort = "priority"
items_per_page = 50

[storage]
database_path = "~/.local/share/overall/repo_manager.db"
cache_duration_hours = 24
```

## Non-Functional Requirements

### NFR1: Performance

- Repository scan completes in <60 seconds for 50 repos
- UI renders in <2 seconds after data load
- AI analysis completes in <30 seconds per repository
- Database queries respond in <100ms
- Frontend bundle size <2MB (WASM)

### NFR2: Reliability

- Handle network failures gracefully (retry with backoff)
- Recover from database corruption (recreate from cache)
- Continue functioning if Ollama unavailable (skip AI features)
- No data loss on crashes (transactions, WAL mode)

### NFR3: Usability

- Zero-configuration first run (works with gh already authenticated)
- Clear error messages with actionable suggestions
- Keyboard shortcuts for common actions
- Accessible UI (screen reader compatible)
- Mobile-friendly responsive design

### NFR4: Security

- No credentials stored by application
- Relies on gh CLI authentication
- Local-only data storage (no cloud sync)
- Ollama runs locally (no code sent to cloud)
- WASM sandboxing for frontend

### NFR5: Maintainability

- Follow docs/process.md development workflow
- Comprehensive test coverage (>80%)
- Documentation for all public APIs
- Consistent code style (rustfmt, clippy)
- Modular architecture (independent components)

### NFR6: Portability

- Runs on macOS, Linux, Windows
- Requires: Rust, gh CLI, Ollama, ask CLI
- Self-contained binary (no external dependencies except above)
- Database portable across platforms

## Dependencies and Prerequisites

### Required Software

1. **Rust** (latest stable): For building the application
2. **gh CLI** (authenticated): For GitHub API access
3. **Ollama**: For local AI analysis
4. **ask CLI**: For Ollama interaction (from ~/.local/softwarewrighter/bin/)

### Installation Steps

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install gh CLI
brew install gh  # macOS
# or https://github.com/cli/cli#installation

# Authenticate gh
gh auth login

# 3. Install Ollama
brew install ollama  # macOS
# or https://ollama.ai/download

# Start Ollama service
ollama serve

# Pull model
ollama pull phi3:3.8b

# 4. Verify ask CLI is in PATH
which ask  # Should be ~/.local/softwarewrighter/bin/ask

# 5. Clone and build this project
git clone https://github.com/softwarewrighter/overall.git
cd overall
cargo build --release

# 6. Run
./target/release/overall
```

## User Workflows

### Workflow 1: Initial Setup

1. User installs prerequisites (Rust, gh, Ollama, ask)
2. User authenticates gh CLI
3. User runs `overall --setup`
4. System prompts for GitHub owners to track
5. System creates configuration file and database
6. System performs initial scan
7. System displays results in browser

### Workflow 2: Daily Check

1. User opens application (browser tab or standalone)
2. System loads cached data from database
3. User clicks "Refresh" to update from GitHub
4. System fetches latest repository data
5. System analyzes new/changed repositories with AI
6. System updates priority rankings
7. User reviews top-priority repositories
8. User clicks repository to see details
9. User decides to create PR or merge branch

### Workflow 3: Creating a PR from a Branch

1. User sees branch marked "Ready for PR"
2. User clicks on repository to view details
3. User reviews branch status and AI suggestions
4. User clicks "Create PR" button
5. System opens GitHub in browser with PR creation form
6. User completes PR details and submits
7. User refreshes application to see updated status

### Workflow 4: AI-Guided Prioritization

1. User opens application with many repositories
2. System shows priority-sorted list
3. User clicks on top repository
4. User reads AI analysis section
5. AI suggests: "Focus on feature/ai-integration branch, appears ready for PR"
6. User navigates to that branch on GitHub
7. User creates PR based on AI recommendation
8. User marks task complete and moves to next priority

## Metrics and Analytics

### Usage Metrics (Local Only)
- Number of repositories tracked
- Scan frequency (refreshes per week)
- AI analyses performed
- PRs created (via link tracking)
- Time spent in application

### Performance Metrics
- Scan duration (50 repos)
- AI analysis duration (per repo)
- Database query latency
- UI render time

### Quality Metrics
- Test coverage percentage
- Clippy warnings count
- Build time
- Binary size

## Future Enhancements (Beyond v1.0)

### v1.5: Enhanced Analysis
- Commit message quality analysis
- Code churn detection
- Contributor activity tracking
- Dependency vulnerability scanning

### v2.0: Collaboration
- Multi-user support
- Team dashboards
- Shared priority rankings
- PR assignment recommendations

### v2.5: Automation
- Automatic PR creation
- Auto-merge when approved
- Branch cleanup after merge
- Scheduled scans

### v3.0: Intelligence
- Learn from user decisions
- Predict merge conflicts
- Recommend reviewers
- Estimate completion time

## Open Questions

1. **Q**: Should we support private repositories only, or public too?
   **A**: Both, but prioritize repos user has push access to

2. **Q**: How to handle very large organizations (>50 repos)?
   **A**: Limit to 50 most recent, allow user to configure filters

3. **Q**: What if Ollama is not installed?
   **A**: Application works without AI features, shows warning

4. **Q**: Should we create PRs directly or just open GitHub?
   **A**: v1.0 just opens GitHub, v1.5 can create PRs via gh CLI

5. **Q**: How to handle rate limiting from GitHub?
   **A**: Respect rate limits, queue requests, show progress

## Glossary

- **gh CLI**: GitHub command-line interface
- **Ollama**: Local LLM inference server
- **ask CLI**: Command-line tool for querying Ollama
- **WASM**: WebAssembly, compiled Rust code for browser
- **Yew**: Rust framework for building web UIs
- **Unmerged branch**: Branch without a merged pull request
- **Priority score**: Calculated ranking (0.0-1.0) for repository urgency
- **Ready for PR**: Branch ahead of default, no existing PR
- **In Review**: Branch has open PR, not merged
- **softwarewrighter**: Primary GitHub user/organization for this project
