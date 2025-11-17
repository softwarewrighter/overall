# Implementation Plan

## Project Timeline

**Target**: v1.0 MVP in 4-6 weeks

**Approach**: Iterative development following TDD principles from docs/process.md

## Development Phases

### Phase 0: Project Setup (Week 1)

**Goal**: Establish project structure, dependencies, and development environment

#### Tasks

1. **Initialize Project Structure**
   - Create Cargo workspace with multiple crates
   - Setup wasm-ui subdirectory
   - Configure Cargo.toml for workspace
   - Initialize git repository

2. **Setup Dependencies**
   ```toml
   # Root Cargo.toml
   [workspace]
   members = ["overall-cli", "wasm-ui"]

   [workspace.dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   chrono = { version = "0.4", features = ["serde"] }
   tokio = { version = "1.0", features = ["full"] }
   ```

3. **Create Build Scripts**
   - scripts/build-all.sh
   - scripts/run-web.sh
   - scripts/check-setup.sh
   - Make scripts executable

4. **Setup Documentation**
   - Run `proact .` to generate docs/ai_agent_instructions.md
   - Create docs/learnings.md (initially empty)
   - Add README.md with quickstart

5. **Verify Prerequisites**
   - Test gh CLI authentication
   - Test Ollama installation
   - Test ask CLI availability
   - Document any setup issues in learnings.md

**Deliverable**: Working project skeleton with passing build

**Test**: `./scripts/build-all.sh` completes successfully

---

### Phase 1: Core Backend - GitHub Integration (Week 1-2)

**Goal**: Fetch and parse GitHub repository data

#### Iteration 1.1: Repository Listing

**TDD Cycle**:

1. **RED**: Write test for listing repositories
   ```rust
   #[test]
   fn test_list_repos_returns_repositories() {
       let repos = list_repos("softwarewrighter").unwrap();
       assert!(!repos.is_empty());
   }
   ```

2. **GREEN**: Implement minimal version
   ```rust
   pub fn list_repos(owner: &str) -> Result<Vec<Repository>> {
       // Execute gh CLI command
       // Parse JSON output
       // Return repositories
   }
   ```

3. **REFACTOR**: Extract command execution, improve error handling

**Implementation**:
- src/github/mod.rs: Module definition
- src/github/commands.rs: Command pattern implementation
- src/github/models.rs: Repository, Branch, PR structs
- src/error.rs: Error types

**Tests**:
- Unit test: Command construction
- Unit test: JSON parsing
- Integration test: Real gh CLI call (softwarewrighter)

**Acceptance Criteria**:
- [ ] Can fetch repositories for given owner
- [ ] Parses JSON response correctly
- [ ] Handles errors gracefully (owner not found, network failure)
- [ ] Returns top 50 most recent repos sorted by pushedAt

#### Iteration 1.2: Branch Analysis

**TDD Cycle**:

1. **RED**: Test for fetching branches
   ```rust
   #[test]
   fn test_fetch_branches_for_repo() {
       let branches = fetch_branches("softwarewrighter/overall").unwrap();
       assert!(branches.iter().any(|b| b.name == "main"));
   }
   ```

2. **GREEN**: Implement branch fetching
3. **REFACTOR**: Extract branch parsing logic

**Implementation**:
- Fetch branches via gh API
- Compare branches to default branch
- Calculate ahead/behind counts
- Detect merge conflicts

**Tests**:
- Parse branch JSON
- Calculate ahead/behind correctly
- Handle repos with no branches

**Acceptance Criteria**:
- [ ] Fetches all branches for repository
- [ ] Identifies default branch
- [ ] Calculates ahead/behind for each branch
- [ ] Handles empty repositories

#### Iteration 1.3: Pull Request Status

**TDD Cycle**:

1. **RED**: Test for PR status detection
   ```rust
   #[test]
   fn test_classify_branch_ready_for_pr() {
       let branch = Branch { /* ahead, no PR */ };
       assert_eq!(classify_status(&branch), BranchStatus::ReadyForPR);
   }
   ```

2. **GREEN**: Implement classification logic
3. **REFACTOR**: Simplify conditionals

**Implementation**:
- Fetch PRs for repository
- Match branches to PRs
- Classify branch status
- Handle edge cases (closed PR, merged PR)

**Tests**:
- Branch with no PR -> ReadyForPR
- Branch with open PR -> InReview
- Branch with approved PR -> ReadyToMerge
- Branch behind default -> NeedsUpdate

**Acceptance Criteria**:
- [ ] Correctly identifies branches without PRs
- [ ] Matches branches to existing PRs
- [ ] Classifies PR states accurately
- [ ] Handles multiple PRs for same branch

**Milestone**: Backend can fetch and analyze GitHub data

**Demo**: CLI command that prints repository analysis

---

### Phase 2: Data Storage (Week 2)

**Goal**: Persist GitHub data locally in SQLite

#### Iteration 2.1: Database Schema

**TDD Cycle**:

1. **RED**: Test database creation
   ```rust
   #[test]
   fn test_create_database() {
       let temp = tempdir().unwrap();
       let db = Database::create(temp.path()).unwrap();
       assert!(db.connection().is_ok());
   }
   ```

2. **GREEN**: Create schema, initialize DB
3. **REFACTOR**: Extract SQL to separate file

**Implementation**:
- src/storage/mod.rs: Database wrapper
- src/storage/schema.sql: Table definitions
- src/storage/migrations.rs: Version management
- Use rusqlite crate

**Schema Tables**:
- repositories
- branches
- pull_requests
- ai_analysis
- config

**Tests**:
- Database creation
- Table existence
- Index creation

**Acceptance Criteria**:
- [ ] Database file created at specified path
- [ ] All tables created with correct schema
- [ ] Indexes created
- [ ] WAL mode enabled

#### Iteration 2.2: Repository CRUD

**TDD Cycle**:

1. **RED**: Test saving and loading repository
   ```rust
   #[test]
   fn test_save_and_load_repository() {
       let db = test_database();
       let repo = test_repo();
       db.save_repository(&repo).unwrap();
       let loaded = db.load_repository(&repo.id).unwrap();
       assert_eq!(loaded.name, repo.name);
   }
   ```

2. **GREEN**: Implement insert/select
3. **REFACTOR**: Use prepared statements

**Implementation**:
- Repository pattern (trait + impl)
- CRUD operations: create, read, update, delete
- Bulk insert for efficiency
- Transactions for consistency

**Tests**:
- Save single repository
- Save multiple repositories (bulk)
- Update existing repository
- Delete repository cascades to branches

**Acceptance Criteria**:
- [ ] Can save repository to database
- [ ] Can load repository by ID
- [ ] Can update existing repository
- [ ] Can delete repository and related data

#### Iteration 2.3: Branch and PR Storage

**Implementation**:
- Branch CRUD operations
- PR CRUD operations
- Foreign key constraints
- Cascade deletes

**Tests**:
- Save branches for repository
- Load branches with repository
- Update branch status
- Delete branches when repo deleted

**Acceptance Criteria**:
- [ ] Can save branches linked to repository
- [ ] Can query branches by repository
- [ ] Can update branch metadata
- [ ] Foreign keys enforced

**Milestone**: All GitHub data persisted locally

**Demo**: Scan repositories, close app, reopen -> data still there

---

### Phase 3: Priority Calculation and AI Integration (Week 3)

**Goal**: Calculate priority scores and integrate Ollama AI

#### Iteration 3.1: Priority Calculation

**TDD Cycle**:

1. **RED**: Test priority formula
   ```rust
   #[test]
   fn test_calculate_priority_recent_active() {
       let repo = Repository {
           pushed_at: Utc::now() - Duration::days(1),
           // ... high activity
       };
       let priority = calculate_priority(&repo, &branches, None);
       assert!(priority > 0.7);
   }
   ```

2. **GREEN**: Implement formula
3. **REFACTOR**: Extract components (recency, activity, etc.)

**Implementation**:
- src/analysis/priority.rs
- Recency score calculation
- Activity score calculation
- Branch readiness score
- Combined priority score

**Tests**:
- Recent repo -> high recency score
- Many commits -> high activity score
- Many ready branches -> high branch score
- Old inactive repo -> low priority

**Acceptance Criteria**:
- [ ] Priority score between 0.0 and 1.0
- [ ] Recent repos prioritized higher
- [ ] Active repos prioritized higher
- [ ] Repos with ready branches prioritized higher

#### Iteration 3.2: AI Integration (ask CLI)

**TDD Cycle**:

1. **RED**: Test ask CLI invocation
   ```rust
   #[test]
   fn test_query_ollama() {
       let response = query_ollama("What is 2+2?").unwrap();
       assert!(response.contains("4"));
   }
   ```

2. **GREEN**: Execute ask command
3. **REFACTOR**: Extract prompt building

**Implementation**:
- src/ai/mod.rs: AI module
- src/ai/ollama.rs: Ollama integration via ask CLI
- src/ai/prompts.rs: Prompt templates
- Execute `ask -p ollama -m phi3:3.8b "<prompt>"`

**Tests**:
- Mock test: Command construction
- Integration test: Real Ollama query (if available)
- Timeout test: Handle long-running queries
- Error test: Ollama not available

**Acceptance Criteria**:
- [ ] Can execute ask CLI command
- [ ] Can parse response
- [ ] Handles timeout (30s)
- [ ] Gracefully handles Ollama unavailable

#### Iteration 3.3: Repository Analysis Prompts

**TDD Cycle**:

1. **RED**: Test prompt generation
   ```rust
   #[test]
   fn test_build_analysis_prompt() {
       let prompt = build_prompt(&repo, &branches);
       assert!(prompt.contains("softwarewrighter/overall"));
       assert!(prompt.contains("Rust"));
   }
   ```

2. **GREEN**: Build structured prompt
3. **REFACTOR**: Template system

**Implementation**:
- Prompt template with repository context
- Include: name, language, branches, recent commits
- Request: priority, focus, actions
- Parse response into structured data

**Tests**:
- Prompt includes all required fields
- Response parsing extracts priority
- Response parsing extracts actions
- Handles malformed responses

**Acceptance Criteria**:
- [ ] Prompt includes repository metadata
- [ ] Prompt includes branch information
- [ ] Response parsed into AIAnalysis struct
- [ ] Fallback for parsing failures

#### Iteration 3.4: AI Analysis Caching

**Implementation**:
- Store AI analysis in ai_analysis table
- Check cache before querying Ollama
- Cache TTL: 24 hours
- Background refresh for stale cache

**Tests**:
- Fresh cache -> return cached result
- Stale cache -> re-query Ollama
- No cache -> query and cache result

**Acceptance Criteria**:
- [ ] AI analysis cached in database
- [ ] Cache checked before querying
- [ ] Stale results refreshed
- [ ] Cache TTL configurable

**Milestone**: Backend calculates priorities with AI assistance

**Demo**: CLI shows priority-sorted repos with AI suggestions

---

### Phase 4: Yew Frontend - Basic UI (Week 3-4)

**Goal**: Display repositories in web UI

#### Iteration 4.1: Yew Project Setup

**Tasks**:
- Initialize wasm-ui project
- Add Yew dependencies
- Create basic App component
- Setup wasm-pack build
- Create index.html loader

**File Structure**:
```
wasm-ui/
+-- src/
    +-- lib.rs              # Entry point
    +-- app.rs              # Main App component
    +-- components/         # UI components
    +-- models/             # View models
    +-- api/                # Backend calls
Cargo.toml
```

**Dependencies**:
```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
gloo = "0.11"
```

**Deliverable**: "Hello World" Yew app loads in browser

#### Iteration 4.2: Repository List Component

**TDD Approach**:
- Use Playwright for UI testing (from docs/process.md)
- Test: Page loads and displays "Repositories" header
- Test: Table with correct columns renders
- Test: Mock data displays in table

**Implementation**:
- components/repository_list.rs
- Table with sortable columns
- Row rendering with repository data
- Loading state
- Empty state

**Features**:
- Columns: #, Name, Owner, Language, Last Activity, Priority, Branches, AI Summary
- Clickable headers for sorting
- Row highlighting on hover
- Priority color coding

**Acceptance Criteria**:
- [ ] Table renders with all columns
- [ ] Displays mock repository data
- [ ] Sorting works on click
- [ ] Loading spinner shown initially
- [ ] Empty state when no data

#### Iteration 4.3: Backend Integration

**Implementation**:
- api/backend.rs: Calls to Rust backend via wasm-bindgen
- Load repositories from database
- Deserialize to RepositoryViewModel
- Update UI state

**Data Flow**:
```
Frontend (Yew) -> wasm-bindgen -> Backend (Rust) -> SQLite
                                                 -> Return JSON
                <- Deserialize <- JSON Response <-
```

**Tests**:
- Load repositories from backend
- Handle empty database
- Handle backend errors
- Display error messages

**Acceptance Criteria**:
- [ ] Frontend calls backend API
- [ ] Repositories displayed in UI
- [ ] Loading state shown during fetch
- [ ] Errors displayed to user

#### Iteration 4.4: Detail View

**Implementation**:
- components/repository_detail.rs
- Router integration (yew-router)
- Route: /repo/:owner/:name
- Display: branches, PRs, AI analysis, recent commits

**Features**:
- Repository header with metadata
- Branches section with status badges
- AI analysis section
- Recent commits list
- Back to list navigation

**Acceptance Criteria**:
- [ ] Clicking repository navigates to detail
- [ ] Detail view shows all repository data
- [ ] Branch statuses color-coded
- [ ] AI suggestions displayed
- [ ] Back button returns to list

**Milestone**: Working web UI displaying GitHub data

**Demo**: Open browser, see repositories, click for details

---

### Phase 5: Advanced Features (Week 4-5)

**Goal**: Add filtering, refresh, and configuration

#### Iteration 5.1: Filtering and Search

**Implementation**:
- components/filters.rs
- Filter by owner dropdown
- Filter by language dropdown
- Search by repository name
- Filter by status (ready for PR, conflicts, etc.)

**Tests** (Playwright):
- Select owner filter -> only matching repos shown
- Type in search -> matching repos shown
- Clear filters -> all repos shown

**Acceptance Criteria**:
- [ ] Owner filter works
- [ ] Language filter works
- [ ] Search input filters by name
- [ ] Multiple filters combine (AND logic)
- [ ] Filters persist across navigation

#### Iteration 5.2: Refresh Functionality

**Implementation**:
- Refresh button in UI
- Backend: Re-scan GitHub (gh CLI)
- Frontend: Show progress, update UI
- Background: Queue AI analyses

**Features**:
- Progress indicator (X of Y repos scanned)
- Incremental UI updates
- Cancel button
- Auto-refresh option (configurable)

**Tests**:
- Click refresh -> data updates
- Progress shown during scan
- Cancel stops scan
- Error handling (network failure)

**Acceptance Criteria**:
- [ ] Refresh button triggers GitHub scan
- [ ] Progress shown to user
- [ ] UI updates incrementally
- [ ] Can cancel in-progress scan
- [ ] Errors displayed with retry option

#### Iteration 5.3: Settings Page

**Implementation**:
- components/settings.rs
- Route: /settings
- Edit configuration (owners, model, refresh interval)
- Save to config.toml
- Validation

**Features**:
- List of tracked owners (add/remove)
- Model selection dropdown
- Refresh interval selector
- Database location
- Clear cache button

**Tests**:
- Add owner -> saved to config
- Remove owner -> removed from config
- Change model -> saved and used
- Clear cache -> database cleared

**Acceptance Criteria**:
- [ ] Can add/remove GitHub owners
- [ ] Can change AI model
- [ ] Can configure refresh interval
- [ ] Settings persist across sessions
- [ ] Validation prevents invalid inputs

**Milestone**: Full-featured v1.0 UI

**Demo**: Complete workflow from scan to analysis to PR creation

---

### Phase 6: Polish and Release (Week 5-6)

**Goal**: Final testing, documentation, and v1.0 release

#### Iteration 6.1: End-to-End Testing

**Tasks**:
- Write comprehensive Playwright tests
- Test full user workflows
- Test error scenarios
- Performance testing

**Test Scenarios**:
1. First-time setup
2. Scan repositories
3. View priorities
4. Filter repositories
5. View details
6. Refresh data
7. Configure settings
8. Handle errors (Ollama down, GitHub rate limit)

**Acceptance Criteria**:
- [ ] All user workflows tested
- [ ] Error scenarios covered
- [ ] Performance acceptable (<60s scan)
- [ ] No critical bugs

#### Iteration 6.2: Documentation

**Tasks**:
- Update README.md with screenshots
- Write installation guide
- Write usage guide
- Document configuration options
- Create troubleshooting section

**Documents**:
- README.md: Overview, quickstart, screenshots
- docs/INSTALL.md: Detailed installation
- docs/USER_GUIDE.md: How to use the app
- docs/TROUBLESHOOTING.md: Common issues

**Acceptance Criteria**:
- [ ] README complete with screenshots
- [ ] Installation guide tested
- [ ] User guide covers all features
- [ ] Troubleshooting includes common errors

#### Iteration 6.3: Pre-commit Process

**Tasks** (from docs/process.md):
1. Run all tests: `cargo test`
2. Fix linting: `cargo clippy --all-targets --all-features -- -D warnings`
3. Format code: `cargo fmt --all`
4. Validate markdown: `markdown-checker -f "**/*.md"`
5. Update documentation
6. Final review
7. Commit and push

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] Zero clippy warnings
- [ ] Code formatted
- [ ] Markdown validated
- [ ] Documentation up-to-date
- [ ] No TODO/FIXME comments

#### Iteration 6.4: Release Preparation

**Tasks**:
1. Update version in Cargo.toml (1.0.0)
2. Write CHANGELOG.md
3. Create git tag (v1.0.0)
4. Build release artifacts
5. Package distribution
6. Test installation on clean system

**Release Checklist**:
- [ ] Version bumped to 1.0.0
- [ ] CHANGELOG.md complete
- [ ] Git tag created
- [ ] Release build tested
- [ ] Package created
- [ ] Installation tested on clean system

**Deliverable**: overall v1.0.0 release

---

## Development Workflow

### Daily Process

1. **Start**: Review plan.md and status.md
2. **Plan**: Select next iteration task
3. **TDD Cycle**:
   - Write failing test (RED)
   - Implement minimal code (GREEN)
   - Refactor and improve (REFACTOR)
4. **Test**: Run cargo test frequently
5. **Commit**: Follow pre-commit process
6. **Update**: Update status.md with progress

### Weekly Process

1. **Monday**: Review weekly goals
2. **Wednesday**: Mid-week checkpoint
3. **Friday**: Week review, update plan if needed
4. **Weekend**: Optional exploration/prototyping

### Tools to Use

From docs/tools.md:
- `proact .` - Generate/update AI instructions
- `markdown-checker -f "**/*.md"` - Validate docs
- `ask "<query>"` - Quick AI assistance
- Playwright MCP - UI testing

### Quality Gates

Before each commit:
1. All tests pass
2. Zero clippy warnings
3. Code formatted
4. Markdown validated
5. Documentation updated
6. No debug code

### Metrics to Track

In status.md:
- Features completed
- Tests written
- Test coverage
- Clippy warnings (target: 0)
- Build time
- Lines of code
- TODO count (max 3 per file)

## Risk Mitigation

### Risk 1: gh CLI Changes

**Risk**: GitHub CLI changes API or JSON format

**Mitigation**:
- Version check in code
- Parse with optional fields
- Integration tests catch breakage
- Fallback to older format if possible

### Risk 2: Ollama Unavailable

**Risk**: User doesn't have Ollama or model installed

**Mitigation**:
- Detect at startup
- Gracefully disable AI features
- Show helpful error message
- App still useful without AI

### Risk 3: GitHub Rate Limiting

**Risk**: Hit GitHub API rate limits (5000/hour)

**Mitigation**:
- Monitor remaining requests
- Queue requests if near limit
- Cache aggressively
- Respect retry-after headers

### Risk 4: Database Corruption

**Risk**: SQLite database corrupted

**Mitigation**:
- Use WAL mode
- Regular backups (automatic)
- Graceful recovery (recreate)
- Warn user, offer re-scan

### Risk 5: WASM Binary Size

**Risk**: Frontend bundle too large (>5MB)

**Mitigation**:
- Profile with wasm-pack
- Optimize for size (opt-level = 'z')
- Code splitting (future)
- Accept 2-3MB for v1.0

## Success Criteria

### MVP (v1.0) Complete When:

1. **Functionality**:
   - [x] Scans repositories from configured owners
   - [x] Displays top 50 most recent
   - [x] Shows unmerged branch status
   - [x] Calculates priority scores
   - [x] AI analysis for top repos
   - [x] Filter and search
   - [x] Refresh on demand

2. **Quality**:
   - [x] >80% test coverage
   - [x] Zero clippy warnings
   - [x] All code formatted
   - [x] Documentation complete
   - [x] No critical bugs

3. **Performance**:
   - [x] Scan 50 repos in <60s
   - [x] UI renders in <2s
   - [x] AI analysis <30s per repo
   - [x] Database queries <100ms

4. **Usability**:
   - [x] Zero-config first run
   - [x] Clear error messages
   - [x] Intuitive UI
   - [x] Responsive design

## Post-v1.0 Roadmap

### v1.5 (Target: +2 weeks)
- Direct PR creation via gh CLI
- Branch deletion after merge
- Export data (CSV, JSON)
- Automatic background refresh

### v2.0 (Target: +2 months)
- Web service deployment
- Multi-user support
- Team dashboards
- Cloud LLM option (OpenAI, Anthropic)

### v3.0 (Target: +6 months)
- Tauri desktop app
- Notifications
- GitHub Actions integration
- Code quality metrics

## Appendix: Iteration Template

For each iteration, follow this structure:

```markdown
### Iteration X.Y: [Feature Name]

**Goal**: [What are we building?]

**TDD Cycle**:
1. RED: [Failing test description]
2. GREEN: [Minimal implementation]
3. REFACTOR: [Improvements]

**Implementation**:
- [File/module to create]
- [Key functions/types]

**Tests**:
- [Test case 1]
- [Test case 2]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]
```

## Refresh Features Implementation Plan

### Current Status: APIs vs Features

#### Backend APIs - COMPLETE ✅

All required backend APIs already exist (see `docs/design.md` for full documentation):

1. **POST /api/local-repos/scan** ✅ EXISTS
   - Scans all local repositories for status
   - Used by main refresh button
   - Returns success/error

2. **POST /api/repos/sync** ✅ EXISTS
   - Syncs single repository from GitHub
   - Required for dialog refresh
   - Request body: `{"repo_id": "owner/repo"}`
   - Response: `{"success": true, "message": "..."}`

3. **GET /repos.json** ✅ EXISTS
   - Static cached repository data
   - Generated by export command
   - Loaded by UI on page load and refresh

**Conclusion**: Backend is complete. No new endpoints needed.

---

#### Frontend Features - MISSING ❌

The following UI features need implementation (priority order per user):

**PRIORITY 1 (MOST URGENT)**: Spinner Before Dialog Opens
- ❌ Full-screen spinner overlay when repo clicked
- ❌ POST to `/api/repos/sync` with repo_id BEFORE opening dialog
- ❌ Wait for GitHub refresh to complete
- ❌ Then fetch fresh `/repos.json`
- ❌ Then open dialog with fresh data
- **File**: `wasm-ui/src/lib.rs` - `on_repo_click` callback (around line 226)
- **User Quote**: "3 (spinner) should happen BEFORE 2 (dialog appears)"

**PRIORITY 2**: Spinner on Main Refresh Button
- ❌ Rotating spinner icon while refresh in progress
- ❌ Button disabled during refresh
- ❌ Returns to normal after completion
- **File**: `wasm-ui/src/lib.rs` - `on_refresh` callback (around line 268)

**PRIORITY 3**: Last Refresh Timestamp
- ❌ Display relative time in header ("just now", "2 minutes ago")
- ❌ Hover shows full ISO timestamp
- ❌ Updates after any refresh operation
- **File**: `wasm-ui/src/lib.rs` - App component state and header rendering
- **Helper Function**: `format_relative_time(dt: &DateTime<Utc>) -> String`

**PRIORITY 4**: Refresh Button in Dialog
- ❌ Button next to close (✕) in dialog header
- ❌ Refreshes that specific repository
- ❌ Shows spinner in dialog during refresh
- ❌ Updates dialog data after refresh
- **File**: `wasm-ui/src/lib.rs` - RepoDetailDialog component (around line 720)

---

#### Implementation Order

Per user directive: "3 (spinner) should happen BEFORE 2 (dialog appears)"

1. **Phase 1a**: Spinner overlay before dialog opens (PRIORITY 1)
   - Add `loading_repo: Option<String>` state
   - Show overlay immediately on repo click
   - POST to `/api/repos/sync` (API EXISTS ✅)
   - Fetch `/repos.json` after sync completes
   - Hide spinner and open dialog

2. **Phase 1b**: Main button spinner (PRIORITY 2)
   - Add `refreshing: bool` state
   - Show spinner during scan operation
   - Disable button during refresh

3. **Phase 2**: Last refresh timestamp (PRIORITY 3)
   - Add `last_refresh: Option<DateTime<Utc>>` state
   - Display in header with relative time formatting
   - Update after any refresh

4. **Phase 3**: Dialog refresh button (PRIORITY 4)
   - Add button to dialog header
   - Wire to `/api/repos/sync` (API EXISTS ✅)
   - Show spinner in dialog during refresh

---

#### CSS Requirements

The following CSS needs to be added to `static/index.html`:

```css
/* Spinner animation */
.spinner {
  display: inline-block;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Full-screen spinner overlay */
.spinner-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.spinner-container {
  text-align: center;
  color: white;
}

.spinner-large {
  font-size: 48px;
  animation: spin 1s linear infinite;
}

.spinner-text {
  margin-top: 16px;
  font-size: 18px;
}

/* Last refresh timestamp */
.last-refresh {
  font-size: 14px;
  color: #666;
  margin-right: 12px;
}
```

---

#### Detailed Implementation Guide

See `docs/todo-refresh-features.md` for:
- Step-by-step implementation instructions
- Code examples for each phase
- Testing checklist
- Timeline estimates (~1.5-2 hours total)

---

## Notes for AI Coding Agents

When implementing this plan:

1. **Follow TDD**: Always write tests first
2. **Use TodoWrite**: Track progress on each iteration
3. **Commit Frequently**: After each RED/GREEN/REFACTOR cycle
4. **Update Docs**: Keep docs/learnings.md current with issues
5. **Run Pre-commit**: Before every commit (docs/process.md)
6. **Ask Questions**: Use AskUserQuestion when unclear
7. **Reference Files**: Use file_path:line_number format in messages

Example iteration workflow:
```
1. Create todos for iteration tasks
2. Mark first task in_progress
3. Write failing test (RED)
4. Run test, confirm failure
5. Implement minimal code (GREEN)
6. Run test, confirm pass
7. Refactor and improve
8. Run full test suite
9. Mark task completed
10. Commit with detailed message
11. Move to next task
```
