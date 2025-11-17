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
