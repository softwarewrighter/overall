# Data Flow & Sequence Diagrams

This page documents the key data flows and workflows in the **overall** system through detailed sequence diagrams.

## Table of Contents

- [Initial Repository Scan](#initial-repository-scan)
- [Export to JSON](#export-to-json)
- [Web UI Loading](#web-ui-loading)
- [Local Repository Scan](#local-repository-scan)
- [Create Pull Request](#create-pull-request)
- [Refresh Repository Data](#refresh-repository-data)
- [AI Analysis Workflow](#ai-analysis-workflow)
- [Group Management](#group-management)

## Initial Repository Scan

This is the primary workflow for importing repositories from GitHub into the local database.

```mermaid
sequenceDiagram
    participant User
    participant CLI as overall CLI
    participant GH as gh CLI
    participant GitHub as GitHub API
    participant DB as SQLite

    User->>CLI: overall scan <owner>
    CLI->>CLI: Initialize database
    CLI->>GH: gh repo list <owner> --limit 50 --json ...
    GH->>GitHub: GET /users/<owner>/repos
    GitHub-->>GH: Repository list JSON
    GH-->>CLI: JSON output
    CLI->>CLI: Parse JSON to Repository models
    CLI->>DB: INSERT INTO repositories (...)

    loop For each repository
        CLI->>GH: gh api repos/<owner>/<repo>/branches
        GH->>GitHub: GET /repos/<owner>/<repo>/branches
        GitHub-->>GH: Branches JSON
        GH-->>CLI: JSON output
        CLI->>CLI: Parse to Branch models
        CLI->>DB: INSERT INTO branches (...)

        CLI->>GH: gh pr list -R <repo> --json ...
        GH->>GitHub: GET /repos/<owner>/<repo>/pulls
        GitHub-->>GH: Pull requests JSON
        GH-->>CLI: JSON output
        CLI->>CLI: Parse to PullRequest models
        CLI->>DB: INSERT INTO pull_requests (...)
    end

    CLI->>DB: Calculate priorities
    CLI->>DB: COMMIT transaction
    CLI-->>User: Scan complete: N repos, M branches
```

### Key Steps

1. **Authentication Check**: Verify `gh` CLI is authenticated
2. **Repository List**: Fetch top 50 most recently pushed repos
3. **Sort by Activity**: Order by `pushedAt` timestamp
4. **Parallel Branch Fetch**: For each repo, get all branches
5. **PR Status Check**: Identify which branches have open PRs
6. **Calculate Ahead/Behind**: Compare each branch to default branch
7. **Persist to DB**: Store all metadata in SQLite

### Performance Considerations

- **Rate Limiting**: Respects GitHub's 5000 req/hour limit
- **Batch Processing**: Groups API calls where possible
- **Incremental Updates**: Future enhancement to only fetch changes

## Export to JSON

Exports the database to a static JSON file for the web UI.

```mermaid
sequenceDiagram
    participant User
    participant CLI as overall CLI
    participant DB as SQLite
    participant FS as File System

    User->>CLI: overall export
    CLI->>DB: SELECT * FROM groups ORDER BY display_order
    DB-->>CLI: Group list

    loop For each group
        CLI->>DB: SELECT repos FROM repo_groups WHERE group_id = ?
        DB-->>CLI: Repository IDs

        loop For each repository
            CLI->>DB: SELECT * FROM repositories WHERE id = ?
            DB-->>CLI: Repository data

            CLI->>DB: SELECT * FROM branches WHERE repo_id = ?
            DB-->>CLI: Branch list

            CLI->>DB: SELECT * FROM pull_requests WHERE repo_id = ?
            DB-->>CLI: PR list

            CLI->>DB: SELECT * FROM local_repo_status WHERE repo_id = ?
            DB-->>CLI: Local status (if exists)
        end
    end

    CLI->>CLI: Build JSON structure:<br/>groups -> repos -> branches/PRs
    CLI->>FS: Write to static/repos.json
    FS-->>CLI: File written
    CLI-->>User: Export complete: static/repos.json
```

### JSON Structure

```json
{
  "groups": [
    {
      "id": 1,
      "name": "Active Projects",
      "repos": [
        {
          "id": "repo123",
          "owner": "username",
          "name": "project",
          "unmerged_count": 2,
          "branches": [
            {
              "name": "feature-x",
              "ahead": 5,
              "behind": 0,
              "status": "ReadyForPR"
            }
          ],
          "pull_requests": [ ... ]
        }
      ]
    }
  ],
  "last_export": "2025-11-17T12:00:00Z"
}
```

## Web UI Loading

Shows how the Yew frontend loads and displays repository data.

```mermaid
sequenceDiagram
    participant Browser
    participant Server as Axum Server
    participant UI as Yew App
    participant API as REST API
    participant LS as LocalStorage

    Browser->>Server: GET /
    Server-->>Browser: index.html + WASM bundle
    Browser->>UI: Initialize Yew app

    UI->>LS: Check for cached data
    alt Cache exists and fresh
        LS-->>UI: Cached repo data
        UI->>Browser: Render initial view
    else No cache or stale
        UI->>Server: GET /repos.json
        Server-->>UI: Repository data JSON
        UI->>LS: Cache data
        UI->>Browser: Render initial view
    end

    UI->>API: POST /api/local-repos/status
    API-->>UI: Local status for all repos
    UI->>UI: Calculate status icons
    UI->>Browser: Update status icons

    loop Auto-refresh every 60s
        UI->>Server: GET /repos.json
        Server-->>UI: Updated data
        UI->>API: POST /api/local-repos/status
        API-->>UI: Updated local status
        UI->>UI: Recalculate priorities
        UI->>Browser: Re-render updated rows
    end
```

### Loading Phases

1. **Static Load**: HTML, CSS, WASM bundle
2. **Initial Data**: Fetch `repos.json` from server
3. **Local Status**: Query backend for local git status
4. **Render**: Display repository table with status icons
5. **Auto-Refresh**: Poll for updates every 60 seconds

## Local Repository Scan

Scans local filesystem for git repositories and checks their status.

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant FS as File System
    participant Git as Git Commands
    participant DB as SQLite

    User->>UI: Click "Add Local Root"
    UI->>User: Show path input dialog
    User->>UI: Enter /home/user/projects
    UI->>API: POST /api/local-repo-roots<br/>{path: "/home/user/projects"}
    API->>DB: INSERT INTO local_repo_roots
    DB-->>API: Root added
    API-->>UI: Success

    User->>UI: Click "Scan Local Repos"
    UI->>API: POST /api/local-repos/scan

    loop For each local root
        API->>FS: Read directory entries
        FS-->>API: File/folder list

        loop For each subdirectory
            API->>FS: Check for .git folder
            alt Is git repository
                API->>Git: git remote get-url origin
                Git-->>API: GitHub URL
                API->>Git: git status --porcelain
                Git-->>API: Uncommitted files count
                API->>Git: git rev-list @{u}..
                Git-->>API: Unpushed commits count
                API->>Git: git rev-list ..@{u}
                Git-->>API: Behind commits count
                API->>Git: git branch --show-current
                Git-->>API: Current branch name

                API->>DB: INSERT OR REPLACE INTO local_repo_status
            end
        end
    end

    API-->>UI: Scan complete: N repos found
    UI->>UI: Refresh status icons
    UI->>User: Display updated statuses
```

### Status Detection

The system detects:
- **Uncommitted files**: `git status --porcelain | wc -l`
- **Unpushed commits**: `git rev-list @{u}.. | wc -l`
- **Behind commits**: `git rev-list ..@{u} | wc -l`
- **Dirty working tree**: `git diff-index --quiet HEAD`

### Status Priority

```mermaid
graph TD
    Start[Check Repository] --> Unpushed{Unpushed or<br/>Behind?}
    Unpushed -->|Yes| NeedsSync[Priority 0:<br/>NEEDS SYNC 🔴]
    Unpushed -->|No| Uncommitted{Uncommitted<br/>Files?}
    Uncommitted -->|Yes| LocalChanges[Priority 1:<br/>LOCAL CHANGES ⚠️]
    Uncommitted -->|No| Unmerged{Unmerged<br/>Branches?}
    Unmerged -->|Yes| Stale[Priority 2:<br/>STALE ℹ️]
    Unmerged -->|No| Complete[Priority 3:<br/>COMPLETE ✅]
```

## Create Pull Request

Workflow for creating a pull request from the UI.

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant GH as gh CLI
    participant GitHub as GitHub API
    participant DB as SQLite

    User->>UI: Click "Create PR" on branch row
    UI->>User: Show PR details dialog
    User->>UI: Enter title, description, base branch
    UI->>API: POST /api/repos/create-pr<br/>{repo_id, branch, title, body, base}

    API->>DB: SELECT branch status
    DB-->>API: Branch data

    alt Branch already has PR
        API-->>UI: Error: PR already exists
        UI->>User: Show error message
    else No existing PR
        API->>GH: gh pr create<br/>--title "..." --body "..."<br/>--base main --head feature
        GH->>GitHub: POST /repos/<owner>/<repo>/pulls
        GitHub-->>GH: PR created (#123)
        GH-->>API: PR number and URL

        API->>DB: INSERT INTO pull_requests<br/>(number, state, title, ...)
        DB-->>API: PR saved

        API->>DB: UPDATE branches SET status = 'PROpen'

        API-->>UI: Success: PR #123 created
        UI->>UI: Refresh repository data
        UI->>User: Show success + PR link
    end
```

### Validation Rules

Before creating a PR:
1. **No existing PR**: Check `pull_requests` table
2. **Ahead of base**: Branch must have commits not in base
3. **Valid base branch**: Typically `main`, `master`, or `develop`
4. **No merge conflicts**: GitHub will validate on creation

## Refresh Repository Data

User-initiated refresh of a specific repository.

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant CLI as CLI Module
    participant GH as gh CLI
    participant DB as SQLite

    User->>UI: Click refresh icon on repo row
    UI->>API: POST /api/repos/<repo_id>/refresh

    API->>CLI: Trigger refresh for repo
    CLI->>GH: gh api repos/<owner>/<repo>
    GH-->>CLI: Updated repo metadata
    CLI->>DB: UPDATE repositories SET pushed_at = ?, ...

    CLI->>GH: gh api repos/<owner>/<repo>/branches
    GH-->>CLI: Current branch list
    CLI->>DB: DELETE FROM branches WHERE repo_id = ?
    CLI->>DB: INSERT INTO branches (new data)

    CLI->>GH: gh pr list -R <owner>/<repo>
    GH-->>CLI: Current PR list
    CLI->>DB: DELETE FROM pull_requests WHERE repo_id = ?
    CLI->>DB: INSERT INTO pull_requests (new data)

    CLI->>CLI: Recalculate priority
    CLI->>DB: UPDATE repositories SET priority = ?

    CLI-->>API: Refresh complete
    API->>CLI: Export to JSON
    CLI->>FS: Write static/repos.json

    API-->>UI: Success
    UI->>UI: Refetch repos.json
    UI->>User: Display updated data
```

### Refresh Triggers

- **Manual**: User clicks refresh button
- **Automatic**: After creating PR or modifying groups
- **Periodic**: Optional background refresh (future)

## AI Analysis Workflow

How AI analysis is performed using local Ollama.

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant AI as AI Module
    participant Ask as ask CLI
    participant Ollama as Ollama LLM
    participant DB as SQLite

    User->>UI: Click "Analyze" on repository
    UI->>API: POST /api/repos/<repo_id>/analyze

    API->>DB: SELECT repo, branches, commits
    DB-->>API: Repository context

    API->>AI: analyze_project(repo_data)
    AI->>AI: Build analysis prompt:<br/>repo name, language, branches,<br/>activity, unmerged count

    AI->>Ask: ask -p ollama -m phi3:3.8b<br/>"Analyze this repository..."
    Ask->>Ollama: POST /api/generate
    Ollama->>Ollama: Generate analysis<br/>(~5-10 seconds)
    Ollama-->>Ask: Analysis text
    Ask-->>AI: Analysis result

    AI->>AI: Parse analysis:<br/>- Priority score (1-10)<br/>- Suggested actions<br/>- Branch recommendations

    AI->>DB: INSERT INTO ai_analysis<br/>(repo_id, result, created_at)
    DB-->>AI: Analysis cached

    AI-->>API: Analysis complete
    API-->>UI: Analysis results
    UI->>User: Display suggestions
```

### Analysis Prompt Template

```
Repository: {owner}/{name}
Language: {primary_language}
Last activity: {pushed_at}

Unmerged branches:
- {branch_name}: {ahead_by} commits ahead, {behind_by} behind
  Last commit: {last_commit_date}

Analyze this repository and suggest:
1. Which branch should be prioritized
2. Whether branches are feature-complete
3. Recommended next steps
4. Priority score (1-10, where 10 is highest)
```

### AI Features

- **Local Processing**: Runs on user's machine via Ollama
- **Privacy-First**: No data sent to cloud
- **Caching**: Results cached for 24 hours
- **Parallel Processing**: Max 3 concurrent analyses
- **Optional**: System works without AI

## Group Management

Creating and managing repository groups.

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant DB as SQLite

    User->>UI: Click "New Group"
    UI->>User: Show group name dialog
    User->>UI: Enter "High Priority"
    UI->>API: POST /api/groups<br/>{name: "High Priority"}

    API->>DB: INSERT INTO groups (name, display_order, created_at)
    DB-->>API: Group created (id=5)

    API->>CLI: Trigger export
    CLI->>DB: Build JSON with new group
    CLI->>FS: Write repos.json

    API-->>UI: Success: Group created
    UI->>UI: Add new tab
    UI->>User: Show empty group tab

    User->>UI: Drag repo to "High Priority" tab
    UI->>API: POST /api/groups/5/repos/repo123

    API->>DB: BEGIN TRANSACTION
    API->>DB: DELETE FROM repo_groups WHERE repo_id = 'repo123'
    API->>DB: INSERT INTO repo_groups<br/>(repo_id, group_id, added_at)
    API->>DB: COMMIT

    API->>CLI: Trigger export
    CLI->>FS: Write updated repos.json

    API-->>UI: Success: Repo moved
    UI->>UI: Refetch and re-render
    UI->>User: Show repo in new group
```

### Group Operations

| Operation | Endpoint | Description |
|-----------|----------|-------------|
| Create Group | `POST /api/groups` | Add new group with name |
| Delete Group | `DELETE /api/groups/:id` | Remove group (repos become ungrouped) |
| Reorder Groups | `PUT /api/groups/reorder` | Update display_order |
| Add Repo to Group | `POST /api/groups/:id/repos/:repo_id` | Move repo to group |
| Remove from Group | `DELETE /api/groups/:id/repos/:repo_id` | Make repo ungrouped |

## Data Consistency

### Transaction Boundaries

All mutations use SQLite transactions:

```sql
BEGIN TRANSACTION;
  -- Multiple related operations
  DELETE FROM repo_groups WHERE repo_id = ?;
  INSERT INTO repo_groups VALUES (?, ?, ?);
  UPDATE groups SET updated_at = ? WHERE id = ?;
COMMIT;
```

### Export Consistency

The export process ensures consistency:
1. **Single Transaction**: Read all data in one transaction
2. **Atomic Write**: Write JSON atomically (temp file + rename)
3. **Build Info**: Include timestamp and git commit hash
4. **Validation**: Verify JSON structure before serving

### Cache Invalidation

```mermaid
graph LR
    Mutation[Database Mutation] --> Export[Trigger Export]
    Export --> JSON[Write repos.json]
    JSON --> Invalidate[Invalidate Browser Cache]
    Invalidate --> Refetch[UI Refetches Data]
```

## Error Handling Flows

### GitHub API Error

```mermaid
sequenceDiagram
    participant CLI
    participant GH as gh CLI
    participant User

    CLI->>GH: gh repo list owner
    GH-->>CLI: Error: API rate limit exceeded
    CLI->>CLI: Parse error message
    CLI->>CLI: Extract retry-after header

    alt Retry possible
        CLI->>CLI: Sleep for retry-after seconds
        CLI->>GH: Retry request
    else Rate limit exceeded
        CLI-->>User: Error: Rate limited until XX:XX
    end
```

### Database Error

```mermaid
sequenceDiagram
    participant API
    participant DB as SQLite
    participant User

    API->>DB: INSERT INTO repositories ...
    DB-->>API: Error: UNIQUE constraint failed

    API->>API: Check error type

    alt Constraint violation
        API->>DB: UPDATE repositories ... WHERE id = ?
        DB-->>API: Success
        API-->>User: Repository updated
    else Database locked
        API->>API: Retry with backoff
    else Corruption
        API-->>User: Error: Database corrupted<br/>Run: overall repair-db
    end
```

## Related Documentation

- [Architecture Overview](Architecture-Overview) - High-level system architecture
- [GitHub Integration](GitHub-Integration) - Details on GitHub API usage
- [Storage Layer](Storage-Layer) - Database schema and queries
- [Web Server & API](Web-Server-API) - REST API endpoint documentation

---

[← Back to Home](Home)
