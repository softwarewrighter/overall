# Claude Web Research Preview - Overall Project Notes

**Last Updated**: 2025-11-16
**Session**: Research Preview Documentation
**Project**: GitHub Repository Manager (overall)

## Executive Summary

The **Overall** project is a Rust/Yew web application for managing GitHub repositories across multiple users and organizations. The project has progressed significantly beyond initial planning phases and now includes advanced features like repository grouping, drag-and-drop organization, local repository monitoring, and pull request creation.

**Current Status**: Mid-development with functional backend, web server, and UI. Core features implemented, some testing/polish needed.

**Maturity Level**: Alpha/Beta - Core functionality works, needs wasm-pack setup and test fixes for full development workflow.

---

## Current Implementation Status

### ✅ Completed Features

#### Backend (Rust CLI)
- **GitHub Integration** (Phase 1 ✓)
  - Repository listing via `gh` CLI
  - Branch analysis with ahead/behind tracking
  - Pull request status detection
  - Commit history fetching

- **Data Storage** (Phase 2 ✓)
  - SQLite database with schema for repos, branches, PRs
  - Repository grouping system
  - Local repository tracking
  - Build info storage

- **Web Server** (Beyond original plan ✓)
  - Axum-based HTTP server (port 3000)
  - REST API for repository operations
  - Static file serving
  - CORS support
  - API endpoints:
    - `GET /repos.json` - Grouped repository data
    - `POST /api/repos/move` - Move repo between groups
    - `POST /api/pr/create` - Create single PR
    - `POST /api/pr/create-all` - Create PRs for all ready branches
    - `GET /api/local-repos/status` - Local repo status
    - `GET /api/local-repos/roots` - Local repo root paths
    - `POST /api/local-repos/roots` - Add root path
    - `POST /api/local-repos/scan` - Scan local repos

- **Local Git Monitoring** (New feature)
  - Scans local directories for git repositories
  - Detects uncommitted files
  - Tracks unpushed commits
  - Shows behind/ahead status

#### Frontend (Yew/WASM)
- **UI Components** (Phase 4 ✓)
  - Tabbed interface for repository groups
  - Repository list with status indicators
  - Modal detail view for branches/PRs
  - Add Repository dialog with SQL generation
  - Settings dialog for local repo configuration

- **Repository Grouping**
  - Create custom groups
  - Drag-and-drop repos between groups
  - "Ungrouped" tab for uncategorized repos
  - SQL script generation for group management

- **Branch Management**
  - Individual "Create PR" buttons per branch
  - "Create All PRs" button for repos with multiple ready branches
  - Branch status badges (ReadyForPR, InReview, NeedsUpdate)
  - Commit history display
  - Ahead/behind tracking

- **Local Repository Indicators**
  - 📝 Uncommitted files count
  - ⬆️ Unpushed commits count
  - ⬇️ Behind commits count
  - Visual status on each repo row

- **Polish**
  - Footer with copyright, license links, build info
  - Build info showing git commit SHA, host, timestamp
  - Relative time formatting ("2 hours ago")
  - Professional color-coded status indicators
  - Responsive hover effects

### 🚧 Partially Complete

- **Testing**
  - 9 tests passing
  - 5 GitHub integration tests failing (likely need `gh` CLI auth)
  - No UI tests (Playwright planned but not implemented)
  - Test coverage: Unknown, likely <50%

- **AI Integration** (Phase 3 - Not Started)
  - Ollama integration planned but not implemented
  - Priority calculation not implemented
  - AI analysis features not started

- **Documentation**
  - README.md exists but may be outdated
  - docs/status.md outdated (shows Phase 0)
  - docs/learnings.md well-maintained with best practices
  - API documentation missing

### ❌ Not Implemented

- **Phase 3 Features**
  - Priority score calculation
  - AI-powered repository analysis
  - Ollama/LLM integration via `ask` CLI

- **Phase 5 Advanced Features**
  - Filtering by language/owner (UI exists, backend partial)
  - Search functionality
  - Auto-refresh capability
  - Settings persistence (only local repo settings)

- **Phase 6 Polish**
  - Comprehensive E2E testing
  - Performance testing
  - Screenshot documentation
  - User guide

---

## Project Architecture

### Technology Stack

**Backend**:
- Rust (latest stable)
- SQLite with rusqlite
- Axum web framework
- Tower HTTP middleware
- Serde for JSON serialization
- Chrono for date/time handling

**Frontend**:
- Yew 0.21 (Rust WASM framework)
- wasm-bindgen for JS interop
- gloo for utilities
- web-sys for Web APIs

**External Dependencies**:
- `gh` CLI for GitHub API access
- `ask` CLI for AI (planned, not implemented)
- Ollama for local LLM (planned, not implemented)

### File Structure

```
overall/
├── overall-cli/          # Backend Rust library and CLI
│   ├── src/
│   │   ├── github/       # GitHub API integration
│   │   ├── storage/      # SQLite database
│   │   ├── server/       # Web server (Axum)
│   │   ├── local_git.rs  # Local repo monitoring
│   │   ├── models/       # Data models
│   │   ├── config/       # Configuration
│   │   ├── analysis/     # Priority (not implemented)
│   │   └── ai/           # AI integration (stub)
├── wasm-ui/              # Frontend Yew application
│   └── src/
│       └── lib.rs        # Single-file UI (1913 lines)
├── static/               # Web assets
│   ├── index.html        # App entry point
│   ├── repos.json        # Pre-exported data
│   └── favicon.ico
├── scripts/              # Build and run scripts
│   ├── build-all.sh      # Build CLI + WASM
│   └── run-web.sh        # Start web server
└── docs/                 # Documentation
    ├── architecture.md
    ├── prd.md
    ├── design.md
    ├── plan.md
    ├── status.md         # Outdated
    └── learnings.md      # Current
```

### Data Flow

1. **Scan Workflow**:
   ```
   CLI → gh CLI → Parse JSON → SQLite → Export JSON → static/repos.json
   ```

2. **Web UI Workflow**:
   ```
   Browser → WASM App → Fetch /repos.json → Render UI
            ↓
         API Calls (/api/*) → Rust Server → SQLite
   ```

3. **PR Creation**:
   ```
   UI Button → POST /api/pr/create → gh pr create → Open in browser
   ```

4. **Local Monitoring**:
   ```
   Settings → Add Path → POST /api/local-repos/roots → SQLite
   Scan → POST /api/local-repos/scan → git commands → Update DB
   ```

---

## Build and Development Setup

### Current Build Status

✅ **Backend CLI**: Builds successfully with 1 warning
- Warning: `RemoveLocalRepoRootRequest` struct never constructed (minor)

❌ **Frontend WASM**: Cannot build - wasm-pack not installed

### Prerequisites

Required:
- Rust (latest stable) ✓
- `gh` CLI (authenticated) ❓ (Not verified)
- wasm-pack ❌ (Missing)

Optional (for AI features):
- Ollama with phi3:3.8b model
- `ask` CLI from ~/.local/softwarewrighter/bin/

### Build Commands

```bash
# Build backend CLI only
cargo build --release -p overall-cli

# Build everything (requires wasm-pack)
./scripts/build-all.sh

# Run web server
./scripts/run-web.sh
# Or: ./target/release/overall serve

# Run tests
cargo test --all

# Scan repositories
./target/release/overall scan softwarewrighter

# Export to JSON
./target/release/overall export
```

### Known Build Issues

1. **wasm-pack not installed**
   ```
   ERROR: wasm-pack not found
   Install with: cargo install wasm-pack
   ```

2. **GitHub integration tests failing** (5 tests)
   - Likely cause: `gh` CLI not authenticated or not available
   - Tests: `test_list_repos_*`, `test_fetch_branches_*`
   - Impact: Medium (doesn't block development, just testing)

---

## Recent Development History

### Latest Commits (Most Recent First)

1. **35cd75f** - feat: add local repository monitoring and web server functionality
   - Added local git repo scanning
   - Uncommitted/unpushed/behind tracking
   - Settings dialog for repo paths

2. **162ccae** - feat: Add repository grouping and Add Repository dialog
   - Drag-and-drop between groups
   - SQL script generation
   - Group management UI

3. **13295ce** - feat: Implement Phase 1 - GitHub data integration and real-time UI
   - Full branch/PR data integration
   - Real-time status updates
   - API endpoints for data

4. **0f0a46d** - docs: Add learnings.md with best practices
   - Documented Yew patterns
   - Rust/WASM best practices
   - Build process learnings

5. **0bf3419** - feat: Redesign UI with tabs, compact layout, footer, modal
   - Tabbed interface
   - Modal detail views
   - Professional footer

### Implementation Velocity

- **Phase 0** (Project Setup): Completed
- **Phase 1** (GitHub Integration): Completed
- **Phase 2** (Data Storage): Completed
- **Phase 3** (AI Integration): Not started
- **Phase 4** (Basic UI): Completed + extras
- **Phase 5** (Advanced Features): Partially complete
- **Phase 6** (Polish): Not started

**Observation**: Project skipped Phase 3 (AI) and jumped to advanced UI features. Current focus appears to be on practical repository management over AI-powered analysis.

---

## Code Quality Assessment

### Strengths

✅ **Well-structured**: Clear separation of concerns (backend/frontend)
✅ **Type-safe**: Full Rust type safety on both backend and WASM
✅ **Modern patterns**: Uses Yew functional components, hooks
✅ **Conditional compilation**: Proper use of `#[cfg(target_arch = "wasm32")]`
✅ **Documentation**: Good inline comments and doc comments
✅ **Best practices**: Following learnings.md guidelines
✅ **Error handling**: Proper Result types throughout

### Areas for Improvement

⚠️ **Large single-file UI**: wasm-ui/src/lib.rs is 1913 lines
  - Recommendation: Split into modules (components/, api/, models/)

⚠️ **Test coverage**: Only 9 tests passing, 5 failing
  - Recommendation: Fix GitHub tests or mark as integration-only

⚠️ **No UI tests**: Playwright planned but not implemented
  - Recommendation: Add basic smoke tests

⚠️ **Dead code warning**: `RemoveLocalRepoRootRequest` unused
  - Recommendation: Implement or remove

⚠️ **Documentation drift**: status.md outdated
  - Recommendation: Update to reflect current state

⚠️ **No build-info.json**: Expected by UI, not generated
  - Recommendation: Generate during build or use defaults

---

## Recommended Next Steps

### Immediate (Fix Blockers)

1. **Install wasm-pack**
   ```bash
   cargo install wasm-pack
   ```

2. **Fix or document GitHub test failures**
   - Option A: Set up `gh` CLI authentication
   - Option B: Mark tests as `#[ignore]` with comment explaining why

3. **Generate build-info.json**
   - Add to `scripts/build-all.sh`
   - Or create static default in `static/`

4. **Update docs/status.md**
   - Reflect actual progress (Phase 4+ complete)
   - Update metrics
   - Document new features

### Short-term (Polish Current Features)

5. **Split wasm-ui/src/lib.rs**
   - Create `components/` module
   - Create `api/` module
   - Create `models/` module
   - Target: <500 lines per file

6. **Add basic UI tests**
   - Test: App renders
   - Test: Tabs switch correctly
   - Test: Modal opens/closes
   - Use Playwright MCP per learnings.md

7. **Implement RemoveLocalRepoRootRequest**
   - Add DELETE endpoint
   - Add UI button to remove paths
   - Or remove struct if not needed

8. **Add API documentation**
   - Document all endpoints
   - Add request/response examples
   - OpenAPI spec (optional)

### Medium-term (Feature Completion)

9. **Decide on AI integration**
   - Phase 3 is entirely unimplemented
   - Options:
     - Skip AI for v1.0 (simpler, faster release)
     - Implement basic priority calculation (no LLM)
     - Full Ollama integration (as planned)

10. **Implement filtering/search**
    - Language filter
    - Owner filter
    - Name search
    - Status filter

11. **Add refresh functionality**
    - Re-scan GitHub button
    - Progress indicator
    - Incremental updates

12. **Settings persistence**
    - Save UI preferences
    - Theme (optional)
    - Default filters

### Long-term (Release Preparation)

13. **Comprehensive testing**
    - Achieve >80% test coverage
    - E2E workflow tests
    - Performance testing
    - Error scenario testing

14. **Documentation**
    - User guide
    - Installation guide
    - Troubleshooting section
    - Screenshots

15. **Performance optimization**
    - WASM bundle size analysis
    - Database query optimization
    - Lazy loading (if needed)

16. **Pre-commit process**
    - All tests passing
    - Zero clippy warnings (currently 1)
    - Code formatted
    - Markdown validated

---

## Critical Decisions Needed

### 1. AI Integration Strategy

**Question**: Implement Phase 3 (AI integration) or skip for v1.0?

**Options**:
- **A) Skip AI**: Focus on core repository management features
  - Pros: Faster to v1.0, simpler codebase
  - Cons: Misses original vision, "priority" calculation missing

- **B) Simple priority**: Calculate priority without LLM
  - Pros: Meets core need, no external dependencies
  - Cons: Less intelligent than planned

- **C) Full AI integration**: Ollama + ask CLI as planned
  - Pros: Meets original vision, powerful insights
  - Cons: Complex setup, dependencies, slower

**Recommendation**: Option B - Implement simple priority calculation based on recency + activity + branch count. Add AI as v1.5+ feature.

### 2. UI Architecture

**Question**: Keep single-file UI or refactor into modules?

**Current**: 1913 lines in wasm-ui/src/lib.rs

**Recommendation**: Refactor now before it grows further. Target structure:
```
wasm-ui/src/
├── lib.rs              # Entry point, exports
├── app.rs              # Main App component
├── components/
│   ├── mod.rs
│   ├── repo_row.rs
│   ├── repo_detail.rs
│   ├── add_dialog.rs
│   └── settings.rs
├── models/
│   ├── mod.rs
│   ├── repository.rs
│   └── build_info.rs
└── api/
    ├── mod.rs
    └── backend.rs
```

### 3. Testing Strategy

**Question**: How to handle failing GitHub tests?

**Options**:
- **A) Fix now**: Set up `gh` CLI, run tests
- **B) Mark as integration tests**: Separate from unit tests
- **C) Mock gh CLI**: Create test doubles

**Recommendation**: Option B short-term (mark with `#[ignore]`), Option A long-term (CI setup with gh auth).

---

## Risk Assessment

### High Priority Risks

🔴 **wasm-pack missing**: Blocks frontend development
- **Mitigation**: Install immediately (`cargo install wasm-pack`)
- **Status**: Known, easy fix

🔴 **Documentation drift**: README/status outdated, confuses users
- **Mitigation**: Update docs to match implementation
- **Status**: Known, medium effort

### Medium Priority Risks

🟡 **Large single file**: wasm-ui/src/lib.rs hard to maintain
- **Mitigation**: Refactor into modules
- **Status**: Technical debt, increasing

🟡 **Test failures**: 5 GitHub tests failing
- **Mitigation**: Fix or document
- **Status**: Known, low impact on development

🟡 **No UI tests**: Regression risk for frontend
- **Mitigation**: Add Playwright tests
- **Status**: Planned, not critical

### Low Priority Risks

🟢 **AI features missing**: Doesn't block core functionality
- **Mitigation**: Decide on strategy, implement or defer
- **Status**: Strategic decision needed

🟢 **Performance unknown**: No benchmarks yet
- **Mitigation**: Test with large datasets
- **Status**: Not critical for alpha

---

## Success Metrics

### What's Working Well

✅ Repository scanning and storage
✅ Web UI with modern design
✅ Repository grouping with drag-and-drop
✅ Local repository monitoring
✅ Pull request creation from UI
✅ Build info tracking
✅ Professional footer and polish

### What Needs Improvement

📊 Test coverage: Unknown, likely <50% (Target: >80%)
📊 Documentation completeness: 60% (Target: 100%)
📊 Code organization: Single large file (Target: Modular)
📊 Build success rate: 50% (CLI ✓, WASM ✗) (Target: 100%)

### Current vs. Planned

| Feature | Planned | Actual | Status |
|---------|---------|--------|--------|
| GitHub integration | ✓ | ✓ | Complete |
| SQLite storage | ✓ | ✓ | Complete |
| Yew UI | ✓ | ✓ | Complete + extras |
| AI analysis | ✓ | ✗ | Not started |
| Priority calculation | ✓ | ✗ | Not started |
| Repository grouping | ✗ | ✓ | Bonus feature |
| Local repo monitoring | ✗ | ✓ | Bonus feature |
| PR creation | ✗ | ✓ | Bonus feature |
| Web server | ✗ | ✓ | Bonus feature |
| Filtering/search | ✓ | Partial | In progress |
| Settings | ✓ | Partial | Local repos only |

**Observation**: Project pivoted from AI-focused to practical management tool. Core GitHub features + bonus UI features implemented. AI features deferred.

---

## Conclusion

The **Overall** project is in a **healthy mid-development state** with significant progress beyond initial plans. The focus has shifted from AI-powered analysis to practical repository management with an excellent web UI.

**Strengths**:
- Solid Rust backend with SQLite
- Modern Yew/WASM frontend
- Advanced features (grouping, local monitoring, PR creation)
- Good documentation practices (learnings.md)

**Blockers**:
- wasm-pack installation needed
- Documentation outdated
- 5 failing tests

**Recommendation**:
1. Install wasm-pack (10 min)
2. Update documentation (1-2 hours)
3. Decide on AI strategy (v1.0 or later?)
4. Refactor UI into modules (2-4 hours)
5. Fix or document test failures (1 hour)

**Timeline to v1.0**:
- With AI: 3-4 weeks
- Without AI: 1-2 weeks (recommended)

The project has demonstrated strong execution and exceeded initial scope in UI/UX while deferring complex AI features. This is a reasonable trade-off for delivering value sooner.

---

## Additional Resources

### Key Documentation Files

- `README.md` - Project overview and quickstart
- `docs/architecture.md` - System design
- `docs/prd.md` - Product requirements
- `docs/design.md` - Technical decisions
- `docs/plan.md` - Implementation roadmap (outdated)
- `docs/status.md` - Progress tracking (outdated)
- `docs/learnings.md` - Best practices and lessons

### Important Code Locations

- Backend entry: `overall-cli/src/main.rs`
- GitHub integration: `overall-cli/src/github/mod.rs`
- Database: `overall-cli/src/storage/mod.rs`
- Web server: `overall-cli/src/server/mod.rs`
- Local monitoring: `overall-cli/src/local_git.rs`
- Frontend: `wasm-ui/src/lib.rs` (all 1913 lines)

### Useful Commands

```bash
# Development
cargo run -- scan softwarewrighter
cargo run -- export
cargo run -- serve

# Build
./scripts/build-all.sh
cargo build --release -p overall-cli

# Test
cargo test --all
cargo clippy --all-targets --all-features

# Format
cargo fmt --all
```

---

**Generated**: 2025-11-16
**By**: Claude (Web Research Preview)
**Session**: Initial project analysis and documentation
