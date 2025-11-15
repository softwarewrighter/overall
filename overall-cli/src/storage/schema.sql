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

-- Configuration table
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
