# Project Learnings & Best Practices

This document captures key learnings and process improvements to avoid repeating avoidable errors.

## Rust WASM Development

### ✅ Conditional Compilation for WASM Code

**Problem**: Using `#[allow(dead_code)]` on WASM-specific code creates tech debt and code smell.

**Solution**: Use `#[cfg(target_arch = "wasm32")]` for all WASM-specific code:

```rust
#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, PartialEq)]
struct Repository {
    // ...
}

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
fn app() -> Html {
    // ...
}
```

**Benefits**:
- No dead code warnings
- Zero clippy violations
- Code only compiles for intended target
- Clear separation between WASM and native code

### ✅ Yew Callback Best Practices

**Problem**: Direct callback cloning in HTML attributes causes type mismatches.

**Wrong**:
```rust
<button onclick={props.on_close.clone()}>{ "✕" }</button>
```

**Correct**:
```rust
let on_close_button_click = {
    let on_close = props.on_close.clone();
    Callback::from(move |_| on_close.emit(()))
};

html! {
    <button onclick={on_close_button_click}>{ "✕" }</button>
}
```

### ✅ Boolean Then Methods

**Clippy Warning**: `unnecessary_lazy_evaluations`

**Wrong**:
```rust
.then(|| "active")
```

**Correct**:
```rust
.then_some("active")
```

## Build Process

### ✅ Build Info Generation

Always capture complete build metadata for traceability:

```bash
cat > static/build-info.json <<EOF
{
  "version": "0.1.0",
  "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "build_host": "$(hostname)",
  "git_commit": "$(git rev-parse HEAD)",
  "git_commit_short": "$(git rev-parse --short HEAD)",
  "git_branch": "$(git branch --show-current)"
}
EOF
```

**Why**: Provides full traceability for deployed builds (commit SHA, build time, host).

## UI/UX Design

### ✅ Scalability First

**Lesson**: Design for scale from the start.

**Card Grid Limitation**:
- Works well for 4-10 items
- Becomes unwieldy with 50+ items
- Hard to scan quickly

**List/Row Layout Benefits**:
- Scales to 100+ items
- Quick scanning with status indicators
- Efficient use of vertical space
- Hover effects guide user attention

### ✅ Status Indicators

Use clear, universal indicators:
- ⚠️ Warning (red): Issues requiring attention
- 📋 Info (blue): Informational status
- ✓ Success (green): All clear

### ✅ Progressive Disclosure

**Pattern**: Show summary, reveal details on demand

**Implementation**:
- List view: Compact rows with key info + status
- Detail modal: Full information when clicked
- Prevents information overload
- Faster navigation

### ✅ Footer Layout

**Best Practice**: Wide footer with left/right sections

```
[Copyright · License · Links]     [Build: SHA · Host · Timestamp]
```

**Why**:
- Legal info on left (standard placement)
- Technical info on right (less prominent)
- Easy to scan
- Professional appearance

## Git & Version Control

### ✅ Commit Message Structure

Follow this format for feature commits:

```
feat: Brief description (under 72 chars)

Detailed explanation of what changed and why.

Changes:
- Component/File 1:
  - Specific change
  - Another change

- Component/File 2:
  - Change details

Benefits:
- Why this matters
- What it enables

Technical Notes:
- Implementation details
- Breaking changes
- Dependencies

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Testing & Quality

### ✅ Pre-Commit Checklist

Always run before committing:

```bash
# 1. Run all tests
cargo test --all

# 2. Check for clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# 3. Format code
cargo fmt --all

# 4. Build WASM (if applicable)
./scripts/build-all.sh
```

### ✅ Screenshot Documentation

**Process**:
1. Use Playwright MCP for consistent screenshots
2. Capture both list view AND detail views
3. Name files descriptively:
   - `screenshot.png` - Main UI
   - `screenshot-modal.png` - Detail view
4. Update README.md with screenshots
5. Commit screenshots with code changes

**Why**: Visual documentation helps users understand features immediately.

## Error Handling

### ✅ WASM Fetch Error Handling

**Wrong** (type mismatch):
```rust
async fn fetch() -> Result<T, JsValue> {
    let response = Request::get("/api").send().await?; // Error!
}
```

**Correct** (explicit error conversion):
```rust
async fn fetch() -> Result<T, String> {
    let response = Request::get("/api")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch: {:?}", e))?;

    let data = response.json()
        .await
        .map_err(|e| format!("Failed to parse: {:?}", e))?;

    Ok(data)
}
```

## Process Improvements

### ✅ Always Use Scripts - NEVER Bare Commands

**CRITICAL**: Use scripts for ALL operations to ensure repeatability and reproducibility.

**NEVER**:
```bash
# ❌ WRONG - Bare command
./target/release/overall serve --port 8459 --debug

# ❌ WRONG - Direct cargo invocation
cargo build --release -p overall-cli

# ❌ WRONG - Manual process
cd wasm-ui && wasm-pack build
```

**ALWAYS**:
```bash
# ✅ CORRECT - Use scripts
./scripts/serve.sh --debug
./scripts/build-all.sh
./scripts/check-setup.sh
```

**Why This Matters**:
- **Reproducibility**: Scripts ensure same process every time
- **Documentation**: Scripts serve as executable documentation
- **Consistency**: All developers use same commands
- **Maintenance**: Update one script, everyone benefits
- **Debugging**: Scripts can include logging, error handling
- **Automation**: Scripts work in CI/CD pipelines

**Script Development Guidelines**:
1. If you find yourself typing the same command twice, create a script
2. Scripts should have clear usage comments at the top
3. Scripts should support common flags (--debug, --help, etc.)
4. Scripts should validate prerequisites before running
5. Scripts should provide clear error messages
6. Never bypass scripts by running commands directly

**If a script doesn't support what you need**:
- Update the script to add the feature
- Don't bypass it with bare commands
- Document the new option in the script's usage comments

### ✅ Test-Driven Development (TDD) - NON-NEGOTIABLE

**CRITICAL**: Always follow TDD. Never write implementation code before tests.

**The TDD Cycle (Red-Green-Refactor)**:
1. **Red**: Write a failing test first
2. **Green**: Write minimal code to make it pass
3. **Refactor**: Improve code while keeping tests green

**Examples**:

❌ **WRONG - Implementation First**:
```rust
// Added new API endpoint
async fn sync_all_repos(...) -> Response {
    // 100 lines of implementation
}

// No tests written
// Manually tested in UI
// Committed with "works on my machine"
```

✅ **CORRECT - Test First**:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sync_all_repos_success() {
        // Setup test database
        // Call sync_all_repos
        // Assert correct behavior
    }

    #[tokio::test]
    async fn test_sync_all_repos_handles_errors() {
        // Test error cases
    }
}

// ONLY AFTER tests are written:
async fn sync_all_repos(...) -> Response {
    // Implementation
}
```

**Why This Matters**:
- Prevents broken code from being committed
- Forces thinking about edge cases upfront
- Provides regression protection
- Documents expected behavior
- Catches integration issues early

**Consequences of Skipping TDD**:
- UI features that don't work
- API endpoints with no error handling
- Broken builds pushed to main
- Wasted time debugging in production
- Loss of confidence in codebase

### ✅ Checkpoint Process - ALWAYS REQUIRED

**Before ANY commit**, execute this exact sequence:

```bash
# 1. Run ALL tests (MUST PASS)
cargo test --all

# 2. Fix ALL clippy warnings (ZERO warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Format ALL code (auto-fixes)
cargo fmt --all

# 4. Build WASM if applicable (MUST succeed)
./scripts/build-all.sh

# 5. Run project-specific checks
# - Check for TODO limits (max 3 per file)
# - Verify file sizes (< 500 lines)
# - Run sw-checklist if available
# - Run markdown-checker on docs

# 6. Review git status
git status
git diff

# 7. Stage ONLY related files
git add <specific-files>

# 8. Commit with detailed message
git commit -m "..."

# 9. Push immediately
git push
```

**NEVER skip any step**. If a step fails, fix it before proceeding.

### ✅ API Development Process

**When adding new API endpoints**:

1. **Define the contract** (types, request/response)
2. **Write tests FIRST**:
   ```rust
   #[tokio::test]
   async fn test_new_endpoint_success() { }

   #[tokio::test]
   async fn test_new_endpoint_validation() { }

   #[tokio::test]
   async fn test_new_endpoint_error_handling() { }
   ```
3. **Implement endpoint** to make tests pass
4. **Run integration tests** with actual server
5. **Document** the endpoint (OpenAPI/comments)
6. **Only then** update UI to use it

**NEVER**:
- Add endpoints without tests
- Test only via UI
- Skip error handling
- Assume "it works" without automated verification

### ✅ Quality Gates - Must Pass Before Commit

**Automated Checks**:
- ✅ All tests pass (`cargo test --all`)
- ✅ Zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- ✅ Code formatted (`cargo fmt --all --check`)
- ✅ Builds successfully (`cargo build --release`)
- ✅ Documentation builds (`cargo doc --no-deps`)

**Manual Checks**:
- ✅ Git status reviewed (no accidental files)
- ✅ Commit message follows convention
- ✅ Changes are logical and atomic
- ✅ No debugging code left in
- ✅ No commented-out code blocks

**If ANY check fails, DO NOT commit**. Fix the issue first.

### ✅ Avoid Dead Code Attributes

**Never use**: `#[allow(dead_code)]`

**Instead**:
1. Use conditional compilation (`#[cfg(...)]`)
2. Ensure code is actually used
3. Remove unused code
4. Configure test exclusions properly

**Why**: Dead code attributes hide real issues and accumulate tech debt.

### ✅ ISO Timestamps in UI

**Always display** ISO timestamps in footer/build info:
- Full traceability
- Unambiguous timezone (UTC)
- Sortable format
- International standard

**Format**: `2025-11-15T23:15:29Z`

## Lessons Learned

1. **TDD is Non-Negotiable**: Never skip writing tests first - it ALWAYS causes problems
2. **Checkpoint Process is Mandatory**: Run tests, clippy, fmt before EVERY commit
3. **API Endpoints Need Tests**: Never add server endpoints without automated tests
4. **Manual UI Testing is Insufficient**: Broken APIs only discovered when tests would have caught them immediately
5. **Scale from Day 1**: Design UI for 10x the current data volume
6. **No Tech Debt**: Never use `allow(dead_code)` - fix the root cause
7. **Visual Feedback**: Status indicators must be instantly recognizable
8. **Build Traceability**: Always capture commit SHA, host, and timestamp
9. **Progressive Disclosure**: Show summary first, details on demand
10. **Conditional Compilation**: Use `#[cfg(...)]` for target-specific code
11. **Screenshot Everything**: Document UI changes visually
12. **Type Safety**: Let the compiler catch errors early (no JsValue leaks)

## Critical Failures to Remember

### 2025-11-17: Stale Data Retention Bug (Sync Not Clearing Old Data)

**What Happened**:
- Implemented `sync_single_repo()` API endpoint to refresh repository data
- Endpoint fetched new branches/PRs from GitHub
- **BUT**: Did not clear old data from database before saving new data
- Result: Deleted branches still showed in UI after "refresh"
- User deleted 2 branches on GitHub, refreshed UI, still saw all 3 branches

**Root Cause**:
- Copied pattern from `fetch_branches()` which only fetches, doesn't save
- Did not reference `main.rs` scan command which does it correctly
- scan command: fetch → **clear old** → save new
- sync endpoint: fetch → save new (WRONG - leaves stale data)

**How It Should Work**:
```rust
// CORRECT Pattern (from main.rs scan command)
let branches = github::fetch_branches(&repo_id)?;
db.clear_branches_for_repo(&repo_id)?;  // ← CRITICAL
for branch in &branches {
    db.save_branch(branch)?;
}
```

**What We Did Wrong**:
```rust
// WRONG - Missing clear step
let branches = github::fetch_branches(repo_id)?;
// Missing: db.clear_branches_for_repo(repo_id)?;
// Just saves new data, leaving old data in place
```

**Prevention Strategies**:

1. **Write Tests for Data Lifecycle**:
   - Test: Add 3 items → clear → verify 0 items
   - Test: Add old data → sync with 1 item → verify exactly 1 item (not 4)
   - Pattern: `test_clear_removes_all_old_data()`

2. **Always Reference Working Code**:
   - When implementing similar functionality (sync vs scan)
   - Read the working implementation FIRST
   - Copy the entire pattern, not just pieces
   - Document why each step exists

3. **Test with Real Deletion Scenarios**:
   - Don't just test additions
   - Test deletions (delete branch on GitHub, sync, verify gone)
   - Test updates (change branch SHA, sync, verify updated)
   - Test edge cases (delete all branches, sync, verify empty)

4. **Database Operations Always Follow This Pattern**:
   ```rust
   // 1. Fetch new data from source
   let data = source.fetch()?;

   // 2. Clear old data (prevents stale data retention)
   db.clear_old_data()?;

   // 3. Save new data
   for item in data {
       db.save(item)?;
   }
   ```

5. **Document Critical Steps**:
   ```rust
   // Clear old branches BEFORE saving new ones
   // This ensures deleted branches are removed from the database
   db.clear_branches_for_repo(repo_id)?;
   ```

**Test That Would Have Caught This**:
```rust
#[test]
fn test_sync_removes_deleted_branches() {
    // Setup: Save 3 branches to DB
    for i in 1..=3 {
        db.save_branch(&branch_i)?;
    }

    // Simulate GitHub now has only 1 branch (2 were deleted)
    let github_branches = vec![branch_1];

    // Sync
    sync_single_repo(repo_id)?;

    // CRITICAL: Should have exactly 1, not 3
    let db_branches = db.get_branches(repo_id)?;
    assert_eq!(db_branches.len(), 1, "Deleted branches not removed!");
}
```

**Why This Matters**:
- Shows stale/deleted data to users (confusing)
- Users make decisions based on incorrect data
- "Refresh" feature that doesn't actually refresh is worse than no feature
- Erodes trust in the entire application
- Can cause cascading errors (trying to create PRs for deleted branches)

### 2025-11-17: Lost Code During Revert - NEVER Revert Without Backup

**What Happened**:
- Had broken code mixed with working changes in working directory
- Decided to revert some changes to "fix" the problem
- **Lost new working code entirely** by reverting without backup
- Had to re-implement features that were already partially done
- Wasted time recreating code that existed moments before

**Root Cause**:
- **Panic response**: Saw broken code, immediately reverted without thinking
- **No backup**: Didn't save working changes before reverting
- **Lost context**: Couldn't remember what was good vs bad in the revert
- **No git stash**: Could have stashed changes before reverting

**THE FIX - MANDATORY PROCESS**:

**BEFORE reverting ANYTHING**:
```bash
# 1. ALWAYS create backup branch first
git checkout -b backup/before-revert-$(date +%Y%m%d-%H%M%S)
git add -A
git commit -m "Backup before reverting [description]"

# 2. Return to working branch
git checkout main

# 3. NOW you can safely revert
git revert <commit>
# OR
git reset --hard HEAD~1

# 4. If you need the old code later:
git checkout backup/before-revert-20251117-054500 -- path/to/file.rs
```

**Even Better - Fix Instead of Revert**:
```bash
# PREFER: Fix the broken code
# 1. Identify what's broken
# 2. Fix just that part
# 3. Keep the working changes
# 4. Commit the fix

# Instead of throwing away everything
```

**Why This Matters**:
- Reverting loses ALL changes, good and bad
- Can't distinguish between broken code and working features
- Forces re-implementation of code that was already done
- Wastes time and introduces new bugs in re-implementation
- Creates frustration and destroys momentum

**Prevention Strategies**:

1. **Use Git Stash**:
   ```bash
   # Save current changes without committing
   git stash save "WIP: status priority fix"

   # Apply later when ready
   git stash pop
   ```

2. **Use Backup Branches**:
   - Create backup before any destructive operation
   - Cheap to create, invaluable when needed
   - Can cherry-pick good changes later

3. **Fix Forward, Not Backward**:
   - Identify the broken part
   - Fix just that part
   - Keep everything else
   - Commit the fix

4. **Incremental Commits**:
   - Commit working features immediately
   - Don't accumulate changes
   - Smaller commits = easier to revert specific things

**Commitment**:
- **NEVER** run `git revert` or `git reset --hard` without backup first
- **ALWAYS** prefer fixing broken code over reverting
- **USE** `git stash` for quick WIP saves
- **CREATE** backup branches before destructive operations
- **THINK** before reverting: "What good code am I about to lose?"

### 2025-11-17: Missing WASM Build - CRITICAL PROCESS FAILURE

**What Happened**:
- Made changes to `wasm-ui/src/lib.rs` fixing status priority logic
- **DID NOT run `./scripts/build-all.sh` to rebuild WASM**
- Started server with old, broken WASM still deployed in `static/wasm/`
- User saw completely broken UI with wrong status icons and logic
- Wasted user time and caused major frustration

**Root Cause - PROCESS VIOLATION**:
- **FORGOT** that WASM changes require explicit rebuild
- **SKIPPED** the build-all step before starting server
- **ASSUMED** code changes would "just work" without rebuilding
- **FAILED** to verify what was actually deployed vs what was edited

**Why This Keeps Happening**:
1. **Rust Incremental Build Confusion**: Regular `cargo build` auto-rebuilds backend, but WASM requires `wasm-pack build`
2. **Hidden Deployment**: WASM artifacts in `static/wasm/` are "out of sight, out of mind"
3. **No Verification Step**: Start server without checking build timestamps
4. **Muscle Memory Failure**: Forgetting the 2-step process (build WASM → start server)

**THE FIX - MANDATORY PROCESS**:

**NEVER** start the server without first running:
```bash
./scripts/build-all.sh
```

**ALWAYS** follow this exact sequence:
```bash
# 1. Make code changes
vim wasm-ui/src/lib.rs

# 2. BUILD FIRST (NON-NEGOTIABLE!)
./scripts/build-all.sh

# 3. ONLY THEN start server
./scripts/serve.sh
```

**Prevention Strategies**:

1. **Update serve.sh to Check Build Freshness**:
   - Script should check if WASM is older than source files
   - Refuse to start if WASM is stale
   - Force explicit rebuild

2. **Add Build Verification**:
   ```bash
   # In serve.sh, before starting server:
   if [ wasm-ui/src/lib.rs -nt static/wasm/wasm_ui_bg.wasm ]; then
       echo "ERROR: WASM is out of date. Run ./scripts/build-all.sh first!"
       exit 1
   fi
   ```

3. **Single Command Workflow**:
   - Create `./scripts/dev.sh` that builds AND serves
   - Eliminates the 2-step process
   - Can't forget to build

4. **Visual Reminder in Terminal**:
   - Add to shell prompt or script output:
   - "🔨 Remember: Changes to wasm-ui/ require ./scripts/build-all.sh!"

5. **Test Build Timestamp API**:
   - Add `/api/wasm-build-time` endpoint
   - UI shows warning if WASM is >5 minutes old
   - Visual indicator of stale builds

**What Should Have Happened**:
1. Edit `wasm-ui/src/lib.rs`
2. **RUN `./scripts/build-all.sh`** ← CRITICAL MISSING STEP
3. Verify build completed successfully
4. Start server
5. Test changes in browser

**Why This is UNACCEPTABLE**:
- Wastes user's time debugging issues that don't exist in code
- Destroys trust ("did you even test this?")
- Shows lack of process discipline
- Same mistake repeated multiple times = not learning

**Commitment**:
- **From now on**: ALWAYS check "is WASM built?" before starting server
- **Verify**: `ls -la static/wasm/wasm_ui_bg.wasm` timestamp is recent
- **When in doubt**: Run `./scripts/build-all.sh` again
- **Update scripts**: Add build freshness checks to prevent this

### 2025-11-17: Status Priority Logic Regression - DOCUMENTATION LIE

**What Happened**:
- Commit `b7a167d` message claimed: "Replace emoji icons with PNG files and fix priority logic"
- **BUT**: The commit did NOT actually fix the priority logic!
- Code still had broken logic: conflated all states into "critical" priority
- Did NOT check GitHub branch ahead/behind status
- Did NOT separate local-changes (yellow) from needs-sync (red)
- User saw repos with 3 different states all showing wrong status

**Root Cause Analysis**:

1. **Commit Message Lied**: Said "fix priority logic" but code showed otherwise
2. **Incomplete Fix**: Only fixed PNG icons, forgot to fix the logic
3. **No Test Coverage**: If tests existed, they would have caught the broken logic
4. **No Verification**: Didn't actually test the "fixed" logic before committing
5. **Working Directory Confusion**: Had uncommitted changes mixed with broken committed code

**The Broken Logic**:
```rust
// WRONG - What was actually committed
fn calculate_repo_status_priority(...) -> u8 {
    if let Some(status) = local_status {
        if status.uncommitted_files > 0
            || status.unpushed_commits > 0
            || status.behind_commits > 0
        {
            return 0; // ALL states = priority 0!
        }
    }
    if repo.unmerged_count > 0 {
        return 1;
    }
    2
}
```

**What Was DOCUMENTED in CLAUDE.md**:
- Priority 0: local-changes (uncommitted files) - YELLOW
- Priority 1: needs-sync (unpushed/behind OR branches ahead/behind) - RED
- Priority 2: stale (unmerged branches) - WHITE
- Priority 3: complete - GREEN
- **MUST check BOTH local status AND GitHub branch status**

**The Correct Logic** (now fixed):
```rust
fn calculate_repo_status_priority(...) -> u8 {
    // Check uncommitted files FIRST (yellow)
    if let Some(status) = local_status {
        if status.uncommitted_files > 0 {
            return 0; // local-changes
        }
    }

    // Check sync issues (red)
    if let Some(status) = local_status {
        if status.unpushed_commits > 0 || status.behind_commits > 0 {
            return 1; // needs-sync
        }
    }

    // CRITICAL: Also check GitHub branch status!
    for branch in &repo.branches {
        if branch.ahead > 0 || branch.behind > 0 {
            return 1; // needs-sync
        }
    }

    // Check stale branches (white)
    if repo.unmerged_count > 0 {
        return 2; // stale
    }

    3 // complete (green)
}
```

**Prevention Strategies**:

## CATASTROPHIC REGRESSION - Missing Refresh Features & Zero Test Coverage

**Date**: 2025-11-16
**Severity**: CRITICAL - Multiple features silently broken with no test failures
**Root Cause**: Massive monolithic file (2400+ lines), no UI tests, not reading learnings.md

### The Disaster

User reported:
1. ❌ Main view Refresh button appears to no-op
2. ❌ No last-refresh-completed ISO time shown
3. ❌ No refresh happens on repo details dialog open
4. ❌ No spinner shown during refresh
5. ❌ No API calls visible in network tab
6. ❌ Repo detail dialog refresh button completely MISSING

**The Shocking Truth**: NO TESTS FAILED because we have ZERO UI tests.

### Root Cause Analysis

#### 1. **File Too Large - Not Modular**
`wasm-ui/src/lib.rs` = **2403 lines** in a SINGLE file!

Violations of CLAUDE.md:
- ❌ "keep functions small and focused for easier maintenance"
- ❌ "Limit functions per module"
- ❌ "Limit modules per crate"
- ❌ "Small testable units are preferred"

**Impact**: When something breaks, EVERYTHING breaks. No isolation.

#### 2. **Zero UI Test Coverage**
We have extensive backend tests (MockGitHubClient, TestDatabase, fixtures), but:
- ❌ NO tests for button clicks
- ❌ NO tests for callbacks firing
- ❌ NO tests for state updates
- ❌ NO tests for API calls happening
- ❌ NO tests for spinners/loading states
- ❌ NO tests for timestamp display

**Why This Matters**: User asked about jest/similar UI testing framework MULTIPLE TIMES.
I never implemented it. Features silently broke. No test failed.

#### 3. **Not Reading learnings.md Periodically**
User specifically said: "learnings.md being for periodic reading, not just recording issues"

I keep ADDING to learnings.md but NEVER:
- ✓ Read it before starting work
- ✓ Implement the suggestions documented
- ✓ Check compliance with documented patterns
- ✓ Review it after each regression

**This is like writing a user manual and never reading it.**

### What Actually Happened (Investigation Results)

After checking git history and current code:

1. **Refresh button EXISTS** in code (line 268-296)
2. **Refresh handler EXISTS** and looks correct
3. **No last_refresh timestamp** - NEVER IMPLEMENTED (not a regression, just never built)
4. **Repo dialog refresh button** - NEVER EXISTED (not a regression, just missing feature)

**But user experienced**:
- Refresh button appears to no-op
- No network calls when clicking refresh
- No spinner during refresh

**Hypothesis**: The refresh handler is broken, but no test catches it because:
- No spy on `trigger_local_repo_scan()` to verify it's called
- No spy on `fetch_repos()` to verify it's called
- No assertion on state update
- No assertion on spinner visibility
- No assertion on timestamp update

### Required Fixes

#### Immediate (Before Any Other Work):

1. **Implement UI Testing Framework**
   - Research: `wasm-bindgen-test` for Rust/WASM
   - OR: Playwright/Puppeteer for E2E browser testing
   - OR: Both (unit + integration)

2. **Write Missing Tests**:
   ```rust
   #[wasm_bindgen_test]
   fn test_refresh_button_triggers_scan() {
       // GIVEN: App is rendered
       // WHEN: User clicks refresh button
       // THEN: trigger_local_repo_scan() is called
       // AND: fetch_repos() is called after 1 second
       // AND: Spinner is shown
       // AND: State is updated
       // AND: Last refresh timestamp is updated
   }

   #[wasm_bindgen_test]
   fn test_repo_dialog_has_refresh_button() {
       // GIVEN: Repo dialog is open
       // WHEN: User looks at dialog buttons
       // THEN: Refresh button exists
       // AND: Close button exists
   }
   ```

3. **Refactor lib.rs into Modules**:
   ```
   wasm-ui/src/
     lib.rs          (entry point, <100 lines)
     app.rs          (main App component)
     components/
       mod.rs
       repo_row.rs   (RepoRow component)
       repo_dialog.rs (RepoDetailDialog component)
       settings_dialog.rs
       add_repo_dialog.rs
     api/
       mod.rs
       github.rs     (fetch_repos, etc.)
       local.rs      (fetch_local_repo_statuses, etc.)
     utils/
       status.rs     (calculate_repo_status_priority)
   ```

4. **Add Coverage Reporting**:
   - Use `cargo-tarpaulin` or `cargo-llvm-cov`
   - Track percentage over time
   - FAIL CI if coverage drops

#### Strategic (Process Changes):

1. **Read learnings.md BEFORE Every Work Session**
   - Make it a checklist item in process.md
   - Verify compliance before coding

2. **Test-First for UI Features**
   - Write UI test FIRST
   - See it fail (RED)
   - Implement to make it pass (GREEN)
   - Refactor (REFACTOR)

3. **Enforce File Size Limits**
   - Add clippy lint for file line counts
   - Max 500 lines per file (configurable)
   - Force modularization

4. **Regular Audits**
   - Weekly: Check test coverage
   - Weekly: Review learnings.md compliance
   - Monthly: Check file sizes
   - Monthly: Check for missing tests

### User's Critical Questions - My Answers

**Q: "Are you adhering to: keep functions small and focused?"**
**A**: NO. `lib.rs` is 2403 lines. Massive violation.

**Q: "I asked earlier about jest or similar UI testing framework - are we doing this?"**
**A**: NO. I documented it but never implemented. Major failure.

**Q: "Why didn't tests notice when refresh button stopped working?"**
**A**: Because we have ZERO UI tests. Only backend/API tests.

**Q: "Why did no test fail when repo dialog refresh button disappeared?"**
**A**: Because we have ZERO tests for dialog button existence.

**Q: "Did you respond to my comment about learnings.md being for periodic reading?"**
**A**: I read it but didn't act on it. I keep adding to learnings.md without:
   - Reading it before work sessions
   - Implementing documented solutions
   - Auditing compliance

### Prevention Strategies:

## UI Testing Infrastructure - Deferred Decision

**Date**: 2025-11-17
**Status**: DEFERRED - Implement features now, decide testing approach when fresh

### The Requirement
- Rust-first project (no JavaScript/TypeScript/Python in codebase)
- Need UI tests for regressions (button clicks, spinners, state changes)
- TDD preferred but not at expense of massive complexity

### Options Researched

1. **fantoccini/thirtyfour** (Rust WebDriver clients)
   - ✅ Pure Rust
   - ✅ Real browser automation
   - ❌ Requires geckodriver/chromedriver (external dependency)
   - ❌ Setup complexity unknown

2. **reqwest + web_sys simulation**
   - ✅ Pure Rust
   - ✅ Can fetch rendered HTML, verify elements
   - ✅ Can inject test harness code into WASM
   - ❌ Requires building custom test infrastructure

3. **wasm-bindgen-test only**
   - ✅ Pure Rust, minimal setup
   - ✅ Good for pure logic
   - ❌ Very limited for UI interactions

### Decision: Defer Until Fresh
User is tired, concerned about node_modules/JVM bloat. Valid concerns.

**Immediate action**:
- Implement missing UI features
- Run fmt/clippy/test
- Checkpoint to GitHub
- Document manual test steps

**Future action** (when fresh):
- Evaluate fantoccini setup complexity
- Check if geckodriver can be global install (not in project)
- Decide if custom reqwest+web_sys approach is worth building
- Or accept manual testing with rigorous checklists

### Lessons
1. Don't drag in JavaScript when user explicitly said "Rust-first"
2. Don't give up on Rust solutions too quickly
3. It's OK to defer testing infrastructure decisions when tired
4. Features + manual testing > no features + perfect tests

1. **Write Tests for Status Priority**:
   ```rust
   #[test]
   fn test_status_priority_uncommitted_beats_unpushed() {
       let repo = test_repo();
       let status = LocalRepoStatus {
           uncommitted_files: 5,
           unpushed_commits: 3,
           behind_commits: 0,
       };
       assert_eq!(calculate_repo_status_priority(&repo, Some(&status)), 0);
   }

   #[test]
   fn test_status_priority_checks_github_branches() {
       let mut repo = test_repo();
       repo.branches.push(BranchInfo {
           ahead: 0,
           behind: 34, // Behind on GitHub!
           // ... other fields
       });
       let status = LocalRepoStatus {
           uncommitted_files: 0,
           unpushed_commits: 0,
           behind_commits: 0, // Local is clean
       };
       // Should be needs-sync (priority 1) because of GitHub branch
       assert_eq!(calculate_repo_status_priority(&repo, Some(&status)), 1);
   }
   ```

2. **Verify Commit Messages**:
   - If message says "fix X", verify X is actually fixed
   - Before commit: diff the changes and confirm they match message
   - Don't copy-paste commit messages - write them based on actual changes

3. **Test All Edge Cases**:
   - Repo with clean local but branches behind on GitHub
   - Repo with uncommitted AND unpushed (should show uncommitted)
   - Repo with only stale branches (no local/sync issues)
   - Tab with mix of all statuses (should show worst-case)

4. **Compare Implementation to Documentation**:
   - Read CLAUDE.md requirements BEFORE implementing
   - After implementing: re-read and verify compliance
   - If docs say "MUST check both sources", code must check both sources

**Why This Regression is CRITICAL**:
- Documented behavior != actual behavior
- User can't trust documentation
- Core feature (status prioritization) completely broken
- Shows lack of verification before claiming "fixed"
- Same issue user reported before - regression of a fix!

**Commitment**:
- Never claim something is "fixed" in commit message without testing it
- Write tests for status priority logic (all edge cases)
- Verify implementation matches documentation before committing
- When fixing a bug user reported, test THE EXACT scenario they reported

### 2025-11-16: Skipped TDD and Checkpoint Process

**What Happened**:
- Added new API endpoints (`sync_all_repos`, `sync_single_repo`) without writing tests
- Added database methods (`get_config`, `set_config`, `get_repositories_updated_since`) without tests
- Modified UI code without verifying API endpoints work
- Attempted to commit documentation (CLAUDE.md) while WIP code remained
- Skipped running tests, clippy, and formatting before commit

**Consequences**:
- API endpoints may be broken (untested)
- UI features likely don't work
- Database methods unverified
- Mixed concerns in git history (docs + untested features)
- Violation of TDD principles

**What Should Have Happened**:
1. Write tests for new database methods FIRST
2. Implement database methods to pass tests
3. Write tests for API endpoints FIRST
4. Implement endpoints to pass tests
5. Run full checkpoint process (tests, clippy, fmt, build)
6. ONLY THEN update UI
7. Verify end-to-end with integration tests
8. Commit features separately from documentation

**Prevention**:
- Always read `docs/ai_agent_instructions.md` before starting work
- Follow TDD cycle religiously: Red → Green → Refactor
- Run checkpoint process before ANY commit
- Never commit untested code
- Separate documentation commits from feature commits
- When in doubt, write a test first

## GitHub Wiki Documentation Errors

### 2025-11-17: Three Classes of Wiki Link Errors

**Background**: Claude AI agents have been generating GitHub Wiki documentation with systematic errors that break links and diagrams. This section documents the three error patterns discovered across 14 repositories.

#### Error 1: Bad Internal Wiki Links (248 instances)

**Problem**: Internal wiki links incorrectly include `.md` extension
```markdown
❌ WRONG: [Architecture](Architecture.md)
✅ CORRECT: [Architecture](Architecture)
```

**Why This Matters**:
- GitHub Wiki interprets `Architecture.md` as a link to the raw markdown file
- Renders as plain text instead of formatted wiki page
- Breaks navigation between wiki pages

**Root Cause**: Confusion between:
- Wiki-to-wiki links (no extension)
- Repo file links (need full GitHub URL with extension)

**Prevention**:
- Internal wiki links NEVER have `.md` extension
- Only use bare page names: `[Page Title](Page-Name)`
- GitHub Wiki automatically handles the extension

#### Error 2: Broken Mermaid Diagrams (1,481 instances)

**Problem**: `<br/>` HTML tags used inside Mermaid diagram blocks
```mermaid
❌ WRONG:
participant Nginx as Nginx<br/>(Port 80)

✅ CORRECT:
participant Nginx as Nginx (Port 80)
```

**Why This Matters**:
- Mermaid doesn't support HTML tags
- Diagrams fail to render entirely
- Shows raw source code instead of diagram

**Prevention**:
- NEVER use HTML tags inside Mermaid code blocks
- Use plain text with parentheses for line breaks
- Mermaid has its own syntax for formatting

#### Error 3: Wiki-to-Repo File Links (51 instances)

**Problem**: Relative paths to repository files don't work in GitHub Wiki
```markdown
❌ WRONG: [Design Doc](../docs/design.md)
✅ CORRECT: [Design Doc](https://github.com/owner/repo/blob/main/docs/design.md)
```

**Why This Matters**:
- GitHub Wiki pages are served from separate `.wiki.git` repository
- Relative paths like `../docs/` resolve relative to wiki repo, not main repo
- Links to repository files break completely

**Root Cause**: GitHub's wiki architecture
- Main repo: `github.com/owner/repo`
- Wiki repo: `github.com/owner/repo.wiki.git`
- Relative paths don't cross repository boundaries

**Prevention**:
- Wiki-to-repo file links MUST use full GitHub URLs
- Format: `https://github.com/owner/repo/blob/main/path/to/file.md`
- Includes `/blob/main/` path component
- Preserves `.md` extension for repository files

#### Distinguishing Link Types

When creating GitHub Wiki documentation:

1. **Wiki-to-Wiki** (internal wiki navigation):
   - Format: `[Page](Page-Name)`
   - NO `.md` extension
   - Example: `[Architecture](Architecture)`

2. **Wiki-to-Repo** (links to repository files):
   - Format: `[File](https://github.com/owner/repo/blob/main/path/file.md)`
   - Full GitHub URL required
   - Includes `/blob/main/`
   - Keeps `.md` extension
   - Example: `[README](https://github.com/owner/repo/blob/main/README.md)`

3. **External Links** (other websites):
   - Format: `[Site](https://example.com)`
   - Standard markdown links
   - No special handling

#### Detection and Fixing

**Scripts Created**:
- `check-wiki-errors.sh` - Detects all three error types
- `fix-wiki-errors.sh` - Fixes mermaid and wiki-to-wiki links
- `find-repo-links.sh` - Identifies wiki-to-repo links needing full URLs
- `fix-repo-links.sh` - Converts relative repo links to full GitHub URLs

**Key Technique**: Wiki page inventory
- Build list of wiki page names (without `.md`)
- Use to distinguish wiki pages from repo files
- Only convert non-wiki links to full URLs

**Impact**:
- Fixed 1,729 errors across 14 repositories
- 248 bad wiki links
- 1,481 mermaid `<br/>` tags
- 51 wiki-to-repo relative links

#### Proactive Prevention

When generating GitHub Wiki documentation:

1. **NEVER** add `.md` to internal wiki links
2. **NEVER** use HTML tags in Mermaid diagrams
3. **ALWAYS** use full GitHub URLs for repository files
4. **VERIFY** link syntax before committing wiki pages
5. **TEST** links work correctly on actual GitHub Wiki

**Checklist for Wiki Documentation**:
```bash
# Check for errors before pushing wiki changes
./check-wiki-errors.sh

# Fix errors if found
./fix-wiki-errors.sh --dry-run  # Preview
./fix-wiki-errors.sh            # Apply

# Check for repo file links
./find-repo-links.sh
./fix-repo-links.sh --dry-run   # Preview
./fix-repo-links.sh             # Apply
```
