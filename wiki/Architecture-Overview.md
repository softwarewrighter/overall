# Architecture Overview

This page provides a high-level overview of the **overall** system architecture, including component diagrams and module relationships.

## Table of Contents

- [System Architecture](#system-architecture)
- [Component Diagram](#component-diagram)
- [Module Structure](#module-structure)
- [Deployment Architecture](#deployment-architecture)
- [Technology Stack](#technology-stack)

## System Architecture

The GitHub Repository Manager follows a **client-server architecture** with a Rust backend and WebAssembly frontend:

```mermaid
graph TB
    subgraph "Browser"
        UI[Yew Frontend WASM]
        LS[LocalStorage]
    end

    subgraph "Backend Service"
        Server[Axum Web Server :8459]
        CLI[CLI Commands]
        Analysis[Branch Analysis]
        AI[AI Integration]
    end

    subgraph "Data Layer"
        DB[(SQLite Database ~/.overall/overall.db)]
        JSON[repos.json Static Export]
    end

    subgraph "External Services"
        GH[GitHub API via gh CLI]
        Ollama[Ollama LLM localhost:11434]
    end

    UI -->|REST API| Server
    UI -->|Fetch| JSON
    UI -->|Cache| LS

    Server -->|Serve Static| UI
    Server -->|Read/Write| DB
    Server -->|Execute| CLI

    CLI -->|Scan/Export| DB
    CLI -->|Query| GH

    Analysis -->|Analyze| DB
    AI -->|Query| Ollama

    Server -->|Create PR| GH
    Analysis -->|AI Suggestions| AI
```

### Key Architectural Principles

1. **Separation of Concerns**: Clear boundaries between CLI, server, and UI layers
2. **Data-Driven**: SQLite as single source of truth for repository data
3. **Offline-First**: Static JSON export allows UI to work without backend
4. **External Integration**: All GitHub operations through `gh` CLI (no direct API calls)
5. **Local AI**: Optional Ollama integration for privacy-focused analysis

## Component Diagram

### High-Level Components

```mermaid
graph LR
    subgraph "Frontend Layer"
        YUI[Yew UI Components]
        Router[Yew Router]
        State[Component State]
    end

    subgraph "Backend Layer"
        WS[Web Server]
        API[REST API]
        GHM[GitHub Module]
        SM[Storage Module]
        AM[Analysis Module]
        AIM[AI Module]
    end

    subgraph "Data Layer"
        SQLite[(SQLite DB)]
        Files[Static Files]
    end

    subgraph "External"
        GHCLI[gh CLI]
        OLL[Ollama]
    end

    YUI --> Router
    YUI --> State
    Router -->|HTTP| API
    State -->|Fetch| Files

    API --> WS
    WS --> GHM
    WS --> SM
    WS --> AM

    GHM --> GHCLI
    SM --> SQLite
    AM --> SM
    AIM --> OLL
    AM --> AIM

    GHM -->|Store| SM
```

### Component Responsibilities

| Component | Responsibility | Technology |
|-----------|---------------|------------|
| **Yew UI** | Render tables, handle user interactions, manage UI state | Yew, WASM, web-sys |
| **Web Server** | Serve static files, handle REST API requests | Axum, tokio |
| **GitHub Module** | Execute `gh` CLI commands, parse JSON responses | std::process::Command, serde_json |
| **Storage Module** | Database CRUD operations, schema management | rusqlite, SQL |
| **Analysis Module** | Calculate branch status, priority scoring | Rust |
| **AI Module** | Query Ollama for project analysis | ask CLI, Ollama API |

## Module Structure

### Backend Modules (overall-cli)

```mermaid
graph TB
    Main[main.rs CLI Entry Point]

    subgraph "Core Modules"
        GH[github/ GitHub Integration]
        ST[storage/ Database Layer]
        AN[analysis/ Branch Analysis]
        AI[ai/ LLM Integration]
        SV[server/ Web Server]
        MD[models/ Data Structures]
    end

    Main --> GH
    Main --> ST
    Main --> SV

    SV --> GH
    SV --> ST
    SV --> AN

    GH --> MD
    ST --> MD
    AN --> ST
    AN --> MD
    AI --> MD

    GH -->|Commands| GHCLI[gh CLI]
    ST -->|SQL| DB[(SQLite)]
    AI -->|HTTP| OL[Ollama]
```

#### Module Details

**github/** - GitHub API Integration
- `commands.rs` - Execute gh CLI commands
- `mod.rs` - Public API and data transformations
- Parses JSON output into internal models
- Handles authentication through gh CLI

**storage/** - Data Persistence
- `mod.rs` - Database wrapper and operations
- `schema.sql` - SQL schema definition (embedded)
- CRUD operations for repos, branches, PRs, commits, groups
- Connection pooling and transaction management

**analysis/** - Repository Analysis
- Branch status calculation (ReadyForPR, NeedsSync, etc.)
- Priority scoring algorithm
- Merge readiness detection
- Conflict detection

**ai/** - AI Integration
- Ollama LLM queries via ask CLI
- Project analysis prompts
- Next-step suggestions
- Priority recommendations

**server/** - Web Server
- Axum routes and handlers
- REST API endpoints
- Static file serving
- Build info generation

**models/** - Shared Data Structures
- `Repository`, `Branch`, `Commit`, `PullRequest`
- `Group`, `LocalRepoStatus`
- `BranchStatus` enum
- Serialization/deserialization logic

### Frontend Module (wasm-ui)

```mermaid
graph TB
    Lib[lib.rs Main Component]

    subgraph "UI Components"
        App[App Component]
        Tabs[Tab System]
        Table[Repository Table]
        Row[Repository Rows]
        Dialogs[Dialog Modals]
    end

    subgraph "State Management"
        Data[RepoData State]
        Local[LocalStatus State]
        Refresh[Auto-Refresh]
    end

    subgraph "Utilities"
        Fetch[Fetch Helpers]
        Status[Status Calculation]
        Icons[Icon Rendering]
    end

    Lib --> App
    App --> Tabs
    App --> Data
    Tabs --> Table
    Table --> Row
    Row --> Icons

    Data --> Fetch
    Local --> Fetch
    Status --> Icons

    Refresh --> Fetch
```

#### Component Hierarchy

```
App (root)
├── TabBar
│   ├── Tab (each group)
│   │   └── StatusIcon
│   └── RefreshButton
├── RepositoryTable
│   ├── TableHeader (sortable)
│   └── RepositoryRow (foreach repo)
│       ├── StatusIcon
│       ├── RepoName
│       ├── BranchesColumn
│       ├── LastActivityColumn
│       └── ActionsColumn
│           ├── CreatePRButton
│           └── RefreshButton
└── DialogManager
    ├── AddGroupDialog
    ├── AddLocalRootDialog
    └── CreatePRDialog
```

## Deployment Architecture

### Local Development Environment

```mermaid
graph TB
    subgraph "Developer Machine"
        Browser[Web Browser localhost:8459]

        subgraph "Backend Process"
            Server[overall serve]
            DB[(~/.overall/overall.db)]
        end

        subgraph "External Tools"
            GH[gh CLI authenticated]
            OL[Ollama Service localhost:11434]
        end

        Browser -->|HTTP| Server
        Server -->|Read/Write| DB
        Server -->|Execute| GH
        Server -->|Query| OL
    end
```

### Production Deployment (Local Desktop)

```mermaid
graph TB
    subgraph "User Machine"
        Browser[Web Browser]

        subgraph "Application"
            Server[overall binary Background Service]
            DB[(User Data Directory overall.db)]
            Static[Static Files WASM + HTML/CSS]
        end

        subgraph "User Installed"
            GH[gh CLI]
            OL[Ollama Optional]
        end

        Browser -->|localhost:8459| Server
        Server -->|Serve| Static
        Server -->|Store| DB
        Server -->|Execute| GH
        Server -.->|Optional| OL
    end
```

### Future: Cloud Deployment

```mermaid
graph TB
    subgraph "User's Browser"
        UI[Web UI]
    end

    subgraph "Cloud Infrastructure"
        CDN[CDN Static Assets]
        LB[Load Balancer]

        subgraph "Application Tier"
            API1[API Server 1]
            API2[API Server 2]
        end

        subgraph "Data Tier"
            PG[(PostgreSQL)]
            Redis[(Redis Cache)]
        end

        subgraph "AI Tier"
            LLM[Cloud LLM API OpenAI/Anthropic]
        end
    end

    subgraph "External"
        GH[GitHub API OAuth]
    end

    UI -->|Static| CDN
    UI -->|API| LB
    LB --> API1
    LB --> API2
    API1 --> PG
    API2 --> PG
    API1 --> Redis
    API2 --> Redis
    API1 --> LLM
    API1 --> GH
    API2 --> GH
```

## Technology Stack

### Backend Stack

```mermaid
graph LR
    subgraph "Core"
        Rust[Rust 2021]
        Tokio[tokio Async Runtime]
    end

    subgraph "Web"
        Axum[axum Web Framework]
        Tower[tower Middleware]
        Hyper[hyper HTTP]
    end

    subgraph "Data"
        Rusqlite[rusqlite SQLite]
        Serde[serde Serialization]
    end

    subgraph "Utils"
        Chrono[chrono DateTime]
        Anyhow[anyhow Error Handling]
        Tracing[tracing Logging]
    end

    Rust --> Tokio
    Tokio --> Axum
    Axum --> Tower
    Tower --> Hyper

    Rust --> Rusqlite
    Rust --> Serde
    Rust --> Chrono
    Rust --> Anyhow
    Rust --> Tracing
```

### Frontend Stack

```mermaid
graph LR
    subgraph "Core"
        Rust2[Rust 2021]
        Yew[yew UI Framework]
    end

    subgraph "WASM"
        WasmPack[wasm-pack Build Tool]
        WasmBindgen[wasm-bindgen JS Interop]
        WebSys[web-sys Web APIs]
    end

    subgraph "Utils"
        Gloo[gloo Web Utilities]
        Serde2[serde JSON]
    end

    Rust2 --> Yew
    Yew --> WasmBindgen
    WasmBindgen --> WebSys
    Yew --> Gloo
    Yew --> Serde2

    WasmPack --> WasmBindgen
```

### Build Pipeline

```mermaid
graph LR
    Source[Source Code]

    subgraph "Backend Build"
        CargoBuild[cargo build]
        Binary[overall binary]
    end

    subgraph "Frontend Build"
        WasmBuild[wasm-pack build]
        WasmPkg[pkg/ output]
        CopyStatic[Copy to static/]
    end

    subgraph "Output"
        Release[Release Bundle]
    end

    Source --> CargoBuild
    Source --> WasmBuild

    CargoBuild --> Binary
    WasmBuild --> WasmPkg
    WasmPkg --> CopyStatic

    Binary --> Release
    CopyStatic --> Release
```

## Performance Characteristics

### Scalability Limits

| Resource | v1.0 Target | Notes |
|----------|------------|-------|
| GitHub Users | 5-10 | Can monitor multiple orgs |
| Total Repositories | 500+ | Top 50/user prioritized |
| Concurrent Users | 1 | Local desktop app |
| Database Size | <100MB | Metadata only |
| WASM Bundle | ~2MB | Optimized release build |

### Performance Optimizations

1. **GitHub API Rate Limiting**
   - 5000 requests/hour (authenticated)
   - Aggressive caching in SQLite
   - Batch requests where possible

2. **Database Indexing**
   - Indexes on `pushed_at`, `priority`, `repo_id`
   - Prepared statements for common queries
   - Foreign key cascades for cleanup

3. **Frontend Optimization**
   - Static JSON export (no backend required for viewing)
   - LocalStorage caching
   - Lazy loading for large repo lists
   - Optimized WASM bundle size

4. **AI Analysis**
   - Lazy evaluation (on-demand)
   - Result caching (24 hours)
   - Parallel processing (max 3 concurrent)

## Security Model

```mermaid
graph TB
    subgraph "Authentication"
        GHAuth[GitHub Authentication via gh CLI]
        NoStore[No Credentials Stored]
    end

    subgraph "Data Privacy"
        Local[Local-Only Data]
        Metadata[Metadata Only No Code Content]
        LocalAI[Local AI Ollama]
    end

    subgraph "Isolation"
        WASM[WASM Sandbox]
        SQLite[File Permissions User-Only]
    end

    GHAuth --> NoStore
    Local --> Metadata
    Local --> LocalAI
    WASM --> SQLite
```

### Security Features

- **No Credential Storage**: Uses `gh` CLI which manages its own secure authentication
- **Local Data Only**: SQLite database stored in user home directory with restricted permissions
- **Privacy-First AI**: Ollama runs locally, no data sent to cloud services
- **WASM Sandbox**: Frontend runs in browser security sandbox
- **Metadata Only**: Only stores repository metadata, never actual code content

## Related Documentation

- [Data Flow](Data-Flow) - Detailed sequence diagrams for key workflows
- [Storage Layer](Storage-Layer) - Database schema and ER diagrams
- [Web Server & API](Web-Server-API) - REST API endpoint documentation
- [GitHub Integration](GitHub-Integration) - GitHub API integration details
- [UI Components](UI-Components) - Frontend component architecture

---

[← Back to Home](Home)
