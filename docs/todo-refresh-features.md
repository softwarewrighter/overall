# TODO: Complete Refresh Features

**Date**: 2025-11-17
**Status**: Planned for next session

## Current State

### What Works ✅
- Main refresh button (🔄) exists and calls `/api/local-repos/scan`
- Backend has `/api/repos/sync` endpoint for single-repo refresh (line 128 in server/mod.rs)
- Dialog opens immediately with cached data

### What's Missing ❌
1. No spinner on main refresh button during 1-second wait
2. No last-refresh timestamp in UI header
3. Dialog doesn't trigger actual refresh (just re-reads stale JSON)
4. No spinner BEFORE dialog appears
5. No refresh button IN the dialog

## Implementation Plan

### Phase 1: Add Spinners (Visual Feedback)

#### 1.1: Add Spinner on Main Refresh Button
**Location**: `wasm-ui/src/lib.rs` - on_refresh callback (line 268)

**Changes**:
- Add `use_state` for `refreshing: bool`
- Set `refreshing.set(true)` before async call
- Set `refreshing.set(false)` after completion
- Update button to show spinner when refreshing:
  ```rust
  if *refreshing {
      html! { <button class="btn-refresh" disabled=true><span class="spinner">⟳</span></button> }
  } else {
      html! { <button class="btn-refresh" onclick={on_refresh}>{ "🔄" }</button> }
  }
  ```

**CSS**: Add rotating animation for `.spinner` class

#### 1.2: Add Spinner BEFORE Dialog Opens
**Location**: `wasm-ui/src/lib.rs` - on_repo_click callback (line 226)

**Current Flow** (WRONG):
1. User clicks repo
2. Dialog opens immediately with stale data
3. Background fetch happens (useless - just re-reads JSON)

**Correct Flow**:
1. User clicks repo
2. Show full-screen spinner overlay
3. POST to `/api/repos/sync` with repo_id
4. Wait for response
5. Fetch fresh `/repos.json`
6. Hide spinner
7. Open dialog with fresh data

**Changes**:
- Add `use_state` for `loading_repo: Option<String>`
- When repo clicked:
  - Set `loading_repo.set(Some(repo.id.clone()))`
  - Show spinner overlay
  - POST to `/api/repos/sync` with `repo_id`
  - Wait for completion
  - Fetch `/repos.json`
  - Set `loading_repo.set(None)`
  - Set `selected_repo.set(Some(repo))`

**HTML**:
```rust
if let Some(loading_id) = (*loading_repo).as_ref() {
    html! {
        <div class="spinner-overlay">
            <div class="spinner-container">
                <div class="spinner-large">⟳</div>
                <div class="spinner-text">{ format!("Refreshing {}...", loading_id) }</div>
            </div>
        </div>
    }
}
```

### Phase 2: Add Last Refresh Timestamp

**Location**: `wasm-ui/src/lib.rs` - App component state

**Changes**:
- Add `use_state` for `last_refresh: Option<DateTime<Utc>>`
- Update timestamp after any refresh completes
- Display in header next to refresh button

**HTML**:
```rust
<div class="header-actions">
    if let Some(last) = (*last_refresh).as_ref() {
        <span class="last-refresh" title={last.to_rfc3339()}>
            { format_relative_time(last) }
        </span>
    }
    <button class="btn-refresh" onclick={on_refresh}>{ "🔄" }</button>
</div>
```

**Helper Function**:
```rust
fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_seconds() < 10 {
        "just now".to_string()
    } else if duration.num_minutes() < 1 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_hours() < 1 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        format!("{} hours ago", duration.num_hours())
    }
}
```

### Phase 3: Add Refresh Button to Dialog

**Location**: `wasm-ui/src/lib.rs` - RepoDetailDialog component (around line 720)

**Changes**:
- Add refresh button next to close button in modal header
- On click: POST to `/api/repos/sync` with repo_id
- Show spinner IN dialog during refresh
- Update dialog data after refresh

**HTML**:
```rust
<div class="modal-header">
    <h2>{ &repo.id }</h2>
    <div class="modal-actions">
        <button class="refresh-button" onclick={on_dialog_refresh}>{ "🔄" }</button>
        <button class="close-button" onclick={on_close_button_click}>{ "✕" }</button>
    </div>
</div>
```

## API Contract

### POST /api/repos/sync
**Request Body**:
```json
{
  "repo_id": "owner/repo"
}
```

**Response**:
```json
{
  "success": true,
  "message": "Refreshed owner/repo"
}
```

**What it does**:
1. Fetch latest branches/PRs/commits from GitHub for this specific repo
2. Update database
3. Regenerate `/repos.json`
4. Return success

## Testing Checklist (Manual)

### Main Refresh Button Spinner
- [ ] Click main refresh (🔄)
- [ ] Button shows spinner immediately
- [ ] Button is disabled during refresh
- [ ] After ~1 second, button returns to normal
- [ ] Timestamp updates to "just now"

### Dialog Open with Spinner
- [ ] Click any repo row
- [ ] Full-screen spinner overlay appears immediately
- [ ] Spinner shows repo name being refreshed
- [ ] Network tab shows POST to `/api/repos/sync`
- [ ] After response, spinner disappears
- [ ] Dialog opens with fresh data

### Last Refresh Timestamp
- [ ] After any refresh, timestamp appears in header
- [ ] Shows "just now" immediately after refresh
- [ ] Updates to "X seconds ago", "X minutes ago", etc.
- [ ] Hover shows full ISO timestamp

### Dialog Refresh Button
- [ ] Open any repo dialog
- [ ] Refresh button (🔄) visible next to close (✕)
- [ ] Click refresh button
- [ ] Spinner appears in dialog
- [ ] Dialog data updates after refresh
- [ ] Timestamp in header updates

## Files to Modify

1. `wasm-ui/src/lib.rs`:
   - Add spinner states
   - Update on_refresh callback
   - Update on_repo_click callback
   - Add last_refresh state
   - Add format_relative_time function
   - Add dialog refresh button

2. `static/index.html`:
   - Add CSS for `.spinner` animation
   - Add CSS for `.spinner-overlay`
   - Add CSS for `.last-refresh`

3. No backend changes needed - `/api/repos/sync` already exists!

## Timeline Estimate

- Phase 1 (Spinners): 30-45 minutes
- Phase 2 (Timestamp): 15-20 minutes
- Phase 3 (Dialog button): 20-30 minutes
- Testing: 15-20 minutes
- **Total**: ~1.5-2 hours

## Priority Order

1. **Spinner before dialog** (most important - no visual feedback is confusing)
2. **Dialog refresh button** (second most important - no way to refresh dialog)
3. **Main button spinner** (nice to have - already works, just needs feedback)
4. **Timestamp** (nice to have - informational)
