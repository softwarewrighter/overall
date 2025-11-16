-- Repositories table
CREATE TABLE IF NOT EXISTS repositories (
    id TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    language TEXT,
    description TEXT,
    pushed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_fork INTEGER NOT NULL,
    priority REAL NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_repositories_pushed_at ON repositories(pushed_at DESC);
CREATE INDEX IF NOT EXISTS idx_repositories_priority ON repositories(priority DESC);

-- Branches table
CREATE TABLE IF NOT EXISTS branches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sha TEXT NOT NULL,
    ahead_by INTEGER NOT NULL,
    behind_by INTEGER NOT NULL,
    status TEXT NOT NULL,
    last_commit_date TEXT NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_branches_repo_id ON branches(repo_id);

-- Pull requests table
CREATE TABLE IF NOT EXISTS pull_requests (
    id INTEGER PRIMARY KEY,
    repo_id TEXT NOT NULL,
    branch_id INTEGER,
    number INTEGER NOT NULL,
    state TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_prs_repo_id ON pull_requests(repo_id);

-- Commits table
CREATE TABLE IF NOT EXISTS commits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    branch_id INTEGER NOT NULL,
    sha TEXT NOT NULL,
    message TEXT NOT NULL,
    author_name TEXT NOT NULL,
    author_email TEXT NOT NULL,
    authored_date TEXT NOT NULL,
    committer_name TEXT NOT NULL,
    committer_email TEXT NOT NULL,
    committed_date TEXT NOT NULL,
    FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE,
    UNIQUE(branch_id, sha)
);

CREATE INDEX IF NOT EXISTS idx_commits_branch_id ON commits(branch_id);
CREATE INDEX IF NOT EXISTS idx_commits_committed_date ON commits(committed_date DESC);

-- Groups table
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_groups_display_order ON groups(display_order);

-- Repository group membership table
CREATE TABLE IF NOT EXISTS repo_groups (
    repo_id TEXT NOT NULL,
    group_id INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    PRIMARY KEY (repo_id, group_id),
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_repo_groups_repo_id ON repo_groups(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_groups_group_id ON repo_groups(group_id);

-- Configuration table
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Local repository root paths
CREATE TABLE IF NOT EXISTS local_repo_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_local_repo_roots_enabled ON local_repo_roots(enabled);

-- Local repository status tracking
CREATE TABLE IF NOT EXISTS local_repo_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id TEXT NOT NULL,
    local_path TEXT NOT NULL UNIQUE,
    current_branch TEXT,
    uncommitted_files INTEGER NOT NULL DEFAULT 0,
    unpushed_commits INTEGER NOT NULL DEFAULT 0,
    behind_commits INTEGER NOT NULL DEFAULT 0,
    is_dirty INTEGER NOT NULL DEFAULT 0,
    last_checked TEXT NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_local_repo_status_repo_id ON local_repo_status(repo_id);
CREATE INDEX IF NOT EXISTS idx_local_repo_status_last_checked ON local_repo_status(last_checked DESC);
