# Overall - AI-Orchestrated Repository Manager

## Vision: 10x-100x Developer Productivity Through Hierarchical AI Delegation

### The Juggler Metaphor
Think of a juggler with an assistant. Each spinning plate or ball in flight represents a cloud LLM actively working on a repo. The assistant (this tool) feeds "the next thing" to the juggler (you), gradually increasing the number of things in-flight. The juggler periodically needs to re-spin plates (review PRs, provide direction) or catch dropping balls (handle blockers).

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│ Human (You)                                                      │
│ - Review PRs and approve merges on GitHub                       │
│ - Manual acceptance testing                                     │
│ - Provide high-level direction to Group LLMs                    │
└──────────────────────┬──────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────────┐
│ Overall Tool (Rust/WASM UI)                                      │
│ - Visualize all repos grouped by category                       │
│ - Show PR status, branch status, unmerged work                  │
│ - Drag & drop repos between groups                              │
│ - Chat interface per group                                      │
│ - Browser tab management (MCP/Playwright)                       │
└──────────────────────┬──────────────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┬──────────────┐
        │              │              │              │
┌───────▼──────┐ ┌────▼─────┐ ┌──────▼──────┐ ┌─────▼──────┐
│ Group LLM 1  │ │Group LLM 2│ │ Group LLM 3 │ │ Group LLM 4│
│ (Ollama)     │ │ (Ollama)  │ │  (Ollama)   │ │  (Ollama)  │
│              │ │           │ │             │ │            │
│ Manages:     │ │ Manages:  │ │  Manages:   │ │  Manages:  │
│ Emacs (4)    │ │ Games (6) │ │ CLI Tools(4)│ │ Web Apps(0)│
│              │ │           │ │             │ │            │
│ - Context    │ │ - Context │ │  - Context  │ │ - Context  │
│ - History    │ │ - History │ │  - History  │ │ - History  │
│ - Delegates  │ │ - Delegates│ │ - Delegates │ │ - Delegates│
│   round-robin│ │  round-robin│ │ round-robin│ │  round-robin
└───────┬──────┘ └────┬──────┘ └──────┬──────┘ └─────┬──────┘
        │             │                │              │
   ┌────┼────┬────┐   │           ┌───┼───┬──────┐   │
   │    │    │    │   ...         │   │   │      │   ...
┌──▼─┐┌─▼─┐┌▼──┐┌▼──┐         ┌──▼─┐┌▼──┐┌▼───┐┌▼──┐
│Tab1││Tab2││Tab3│Tab4│         │Tab1│Tab2│Tab3 │Tab4│
│Repo││Repo││Repo│Repo│         │Repo│Repo│Repo │Repo│
│LLM ││LLM ││LLM │LLM │         │LLM │LLM │LLM  │LLM │
│    ││    ││    │    │         │    │    │     │    │
│Cloud│Cloud│Cloud│Cloud        │Cloud│Cloud│Cloud│Cloud
│LLM ││LLM ││LLM │LLM │         │LLM │LLM │LLM  │LLM │
└────┘└────┘└────┘└────┘         └────┘└────┘└────┘└────┘
```

### Workflow: The Human-LLM Coordination Dance

#### Two Primary Workflows

**Workflow A: Continue Without Merging (Accumulate Work)**
- Cloud LLM completes task → pushes to feature branch
- Group LLM updates status → Shows "Feature branch updated" (🔧)
- Human reviews but doesn't merge yet
- Human provides next task → "Continue on this branch, add feature Y"
- Cloud LLM continues building on same feature branch
- Accumulates multiple related changes before single PR review/merge
- **Benefit**: Fewer PR reviews, larger coherent changesets

**Workflow B: Merge Then Continue (Overlap Work)**
- Cloud LLM completes task → creates PR
- Group LLM updates status → Shows "Ready for review" (📋)
- Human reviews and merges PR on GitHub
- **Meanwhile**: Cloud LLM already switched to NEW feature branch for next task
- Human finishes merge → Cloud LLM already working on next thing
- **Benefit**: Maximum parallelism, cloud LLM never idle

#### Detailed State Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. CLEAN STATE                                                  │
│    - All branches merged                                        │
│    - No open PRs                                                │
│    - Indicator: ✓ (green checkmark)                            │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     │ Group LLM: "Repo is clean, what's next?"
                     │ Suggests: [Status, Ideas, Roadmap]
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. HUMAN REVIEWS & DECIDES                                      │
│    - Sees suggestions in repo detail view                       │
│    - Options:                                                   │
│      a) Pick a suggestion: "Implement suggestion #2"            │
│      b) Provide new direction: "Add dark mode support"          │
│      c) Ask for more info: "What would suggestion #3 involve?"  │
│      d) Skip for now: "Not a priority, move to next repo"      │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     │ Human assigns task
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. WORK IN PROGRESS                                             │
│    - Group LLM delegates to Repo LLM (cloud)                    │
│    - Repo LLM creates feature branch                            │
│    - Makes commits                                              │
│    - Indicator: 🔧 (wrench - work in progress)                  │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     │ Work completes
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. FEATURE BRANCH UPDATED                                       │
│    - Commits pushed to feature branch                           │
│    - Group LLM updates status                                   │
│    - Indicator: 🔧 + commit count badge                         │
│    - Human sees: "3 new commits on feature-branch"              │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     │ Human decision point
                     │
        ┌────────────┴─────────────┐
        │                          │
        ▼                          ▼
┌──────────────────┐      ┌──────────────────┐
│ CONTINUE PATH    │      │ REVIEW PATH      │
│ (Workflow A)     │      │ (Workflow B)     │
└────────┬─────────┘      └────────┬─────────┘
         │                         │
         │ "Add more to           │ "Create PR"
         │  this branch"          │
         │                         │
         ▼                         ▼
┌──────────────────┐      ┌──────────────────┐
│ Back to step 3   │      │ 5. PR CREATED    │
│ (same branch)    │      │   - Ready for    │
│                  │      │     review       │
└──────────────────┘      │   - Indicator:📋 │
                          └────────┬─────────┘
                                   │
                                   │ Human reviews on GitHub
                                   │
                          ┌────────┴─────────┐
                          │                  │
                          ▼                  ▼
                  ┌──────────────┐   ┌─────────────┐
                  │ 6a. NEEDS    │   │ 6b. APPROVED│
                  │     CHANGES  │   │     & MERGED│
                  │   - Feedback │   │   - Clean!  │
                  │   - Indicator│   │   - Back to │
                  │     ⚠️       │   │     step 1  │
                  └──────┬───────┘   └─────────────┘
                         │
                         │ Group LLM relays
                         │ feedback to Repo LLM
                         │
                         ▼
                  ┌──────────────┐
                  │ Back to      │
                  │ step 3       │
                  │ (iterate)    │
                  └──────────────┘
```

#### Key Innovation: Status-Driven Attention Management

**Group LLM tracks repo states:**
- 🔧 WorkInProgress (2 repos) - Cloud LLMs actively coding
- 📋 ReadyForReview (3 repos) - Waiting for human review
- ⚠️ NeedsChanges (1 repo) - Human provided feedback
- ✓ Clean (4 repos) - Ready for next task assignment

**Human focus:**
- Priority 1: Review PRs (📋) → Unblock cloud LLMs
- Priority 2: Handle feedback (⚠️) → Provide clear direction
- Priority 3: Assign new work (✓) → Keep LLMs busy

**Goal: Minimize idle time for both human and LLMs**
- Cloud LLMs should rarely wait on human
- Human should rarely wait on cloud LLMs
- Maximize "plates spinning" without overwhelming human review capacity

### Key Features to Build

#### Phase 1: Foundation (✅ Mostly Complete)
- [x] Basic UI with groups and repos
- [x] Drag & drop repos between groups
- [x] REST API for repo management
- [x] Cache-busting and proper dev workflow
- [x] Port 8459 reserved for this project

#### Phase 2: GitHub Integration (PRIORITY)
- [ ] **Branch Management** (Critical: Many unmerged branches exist)
  - [x] Show all unmerged branches per repo
  - [x] Show commit details per branch (SHA, timestamp)
  - [ ] "Create PR" button per unmerged branch
  - [ ] "Create All PRs" button for repo (bulk action)
  - [ ] After PR exists: "View PR" button opens GitHub PR page
  - [ ] Status indicator: HasUnmergedBranches (🔧)

- [ ] **PR Management UI**
  - [ ] Show open PRs per repo
  - [ ] Click to open PR on GitHub
  - [ ] Status tracking:
    - [ ] ReadyForReview (📋 blue)
    - [ ] NeedsChanges (⚠️ yellow)
    - [ ] ReadyToMerge (✅ green)
  - [ ] Mark PR as reviewed/merged (updates status)
  - [ ] Refresh PR status (manual + auto-refresh)

- [ ] **Repo State Machine**
  - [ ] States: Clean → WorkInProgress → ReadyForReview → ReadyToMerge → Clean
  - [ ] Visual indicators for each state
  - [ ] Group-level rollup: "3 repos ready for review"

#### Phase 3: Ollama Integration
- [ ] **Group Chat Interface**
  - [ ] Chat UI embedded in group view
  - [ ] REST calls to Ollama API
  - [ ] Separate context/history per group
  - [ ] Store conversation in SQLite

- [ ] **Multiple Ollama Instances**
  - [ ] Configure multiple Ollama hosts (LAN)
  - [ ] Assign one Ollama instance per group
  - [ ] Health check and failover

#### Phase 4: Browser Tab Management (MCP/Playwright)
- [ ] **Playwright Integration**
  - [ ] Launch browser tabs programmatically
  - [ ] One tab per active repo
  - [ ] Navigate to cloud LLM (Claude Code, Cursor, etc.)
  - [ ] Inject context/instructions into chat

- [ ] **Tab Lifecycle Management**
  - [ ] Track which repos have active tabs
  - [ ] Close tabs when task complete
  - [ ] Reuse tabs for next round-robin task
  - [ ] Handle tab crashes/timeouts

#### Phase 5: Group LLM Orchestration
- [ ] **Task Queue per Group**
  - [ ] Queue of repos needing work
  - [ ] Round-robin scheduler
  - [ ] Priority/urgency markers

- [ ] **Delegation Logic**
  - [ ] Group LLM decides which repo gets next task
  - [ ] Formats instructions for Repo LLM
  - [ ] Monitors progress (PR created, tests passing, etc.)
  - [ ] Reports blockers to human

#### Phase 6: Scaling & Polish
- [ ] **Parallel Work Tracking**
  - [ ] Visual indicator of "plates spinning" (active work)
  - [ ] Estimated completion times
  - [ ] Bottleneck detection (waiting on human review)

- [ ] **Analytics**
  - [ ] Work completed per group
  - [ ] Average time from task → PR → merge
  - [ ] Identify which repos are bottlenecks

- [ ] **Safety & Controls**
  - [ ] Rate limiting (don't overwhelm human with PRs)
  - [ ] Pause/resume groups
  - [ ] Emergency stop all work

### Technology Stack

- **Backend**: Rust (axum, tokio)
- **Frontend**: Yew (WASM)
- **Database**: SQLite (repos, groups, chat history, task queue)
- **LLM Orchestration**:
  - Ollama (local, via REST API)
  - MCP/Playwright (browser automation)
- **Cloud LLMs**: Claude Code, Cursor, GitHub Copilot (via browser tabs)
- **Version Control**: GitHub API (PRs, branches, commits)

### Example User Journey

1. **Morning**: Open Overall, see 20 repos across 4 groups
2. **Review**: 3 PRs ready for review from yesterday's work
3. **Approve**: Merge 2 PRs, request changes on 1
4. **Delegate**: Chat with "Emacs" Group LLM:
   - "The emacs-agent PR looks good. Next, please add syntax highlighting to emacs-ai-api"
5. **Group LLM Works**: Opens tab for emacs-ai-api, starts working
6. **Meanwhile**: Check "CLI Tools" group, see 2 more PRs ready
7. **Afternoon**: 5 tabs open, each with a Repo LLM working in parallel
8. **End of Day**: 7 PRs created, 5 merged, 2 repos have unmerged branches needing review
9. **Next Day**: Repeat, gradually increasing parallel work from 5 → 10 → 20 repos

### Success Metrics

- **Throughput**: Number of PRs merged per day/week
- **Parallel Work**: Number of repos with active work in-flight
- **Review Time**: How long PRs wait for human review
- **Iteration Speed**: Time from feedback → updated PR
- **Human Cognitive Load**: Are you overwhelmed or in flow state?

### Open Questions

1. **How do we "talk to" cloud LLMs in browser tabs?**
   - Use Playwright to inject text into chat boxes?
   - Use browser extensions to intercept/automate?
   - Use official APIs if available?

2. **How does Group LLM know when work is done?**
   - Poll GitHub API for new PRs?
   - Watch for specific keywords in Repo LLM output?
   - Human explicitly marks task complete?

3. **How to handle conflicts/blockers?**
   - Group LLM escalates to human immediately?
   - Automatically skip to next repo in round-robin?
   - Maintain "blocked" queue?

4. **Chat history management?**
   - Store all conversations in SQLite?
   - Summarize old context to keep token usage low?
   - Allow human to edit/prune chat history?

5. **Multi-LAN Ollama coordination?**
   - How to discover Ollama instances on LAN?
   - Load balancing or fixed assignment?
   - What if an Ollama host goes down?

### Future Enhancements (Post-MVP)

#### Smart Conflict Detection & Merge Ordering
- **File-level conflict detection** using GitHub Compare API
  - Fetch changed files per branch: `gh api repos/{owner}/{repo}/compare/main...{branch}`
  - Detect overlapping file modifications between branches
  - Show conflict warnings in UI: "⚠️ May conflict with branch X"
- **Merge order recommendations**
  - Score branches by: freshness (-behind count), size (+ahead count), conflicts
  - Visual ranking: "#1 Recommended", "#2 Next", etc.
  - Color-coding: green (safe), yellow (check first), red (conflicts likely)
- **Advanced detection** (future)
  - Line-level conflict analysis from patch diffs
  - Git merge-tree simulation (requires local clone)
  - Dependency detection between branches

**Decision: Deferred to future**
- Creating PRs in any order is safe (GitHub handles conflict detection)
- Users can merge via GitHub UI or terminal as needed
- Tool doesn't need to clone repos locally
- Focus on expedient PR creation workflow first

---

**Next Immediate Steps**:
1. Add "Create PR" and "Create All PRs" buttons
2. Implement PR creation via gh CLI
3. Track PR URLs in database
4. Update UI to show "View PR" after PR exists
5. Add group chat UI (embedded in group view)
