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

1. **Scale from Day 1**: Design UI for 10x the current data volume
2. **No Tech Debt**: Never use `allow(dead_code)` - fix the root cause
3. **Visual Feedback**: Status indicators must be instantly recognizable
4. **Build Traceability**: Always capture commit SHA, host, and timestamp
5. **Progressive Disclosure**: Show summary first, details on demand
6. **Conditional Compilation**: Use `#[cfg(...)]` for target-specific code
7. **Screenshot Everything**: Document UI changes visually
8. **Type Safety**: Let the compiler catch errors early (no JsValue leaks)
