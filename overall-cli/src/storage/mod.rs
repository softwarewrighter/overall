//! Local SQLite storage

use crate::{
    models::{Branch, BranchStatus, Commit, Group, PRState, PullRequest, Repository},
    Result,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA_SQL: &str = include_str!("schema.sql");

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_or_create(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create schema if not exists
        conn.execute_batch(SCHEMA_SQL)?;

        Ok(Database { conn })
    }

    pub fn save_repository(&self, repo: &Repository) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO repositories (id, owner, name, language, description, pushed_at, created_at, updated_at, is_fork, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &repo.id,
                &repo.owner,
                &repo.name,
                &repo.language,
                &repo.description,
                &repo.pushed_at.to_rfc3339(),
                &repo.created_at.to_rfc3339(),
                &repo.updated_at.to_rfc3339(),
                repo.is_fork as i32,
                repo.priority,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_repositories(&self) -> Result<Vec<Repository>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, owner, name, language, description, pushed_at, created_at, updated_at, is_fork, priority
             FROM repositories
             ORDER BY priority DESC, pushed_at DESC"
        )?;

        let repos = stmt
            .query_map([], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    language: row.get(3)?,
                    description: row.get(4)?,
                    pushed_at: row.get::<_, String>(5)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    created_at: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    updated_at: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    is_fork: row.get::<_, i32>(8)? != 0,
                    priority: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    pub fn save_branch(&self, branch: &Branch) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO branches (repo_id, name, sha, ahead_by, behind_by, status, last_commit_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &branch.repo_id,
                &branch.name,
                &branch.sha,
                branch.ahead_by as i64,
                branch.behind_by as i64,
                branch.status.to_string(),
                &branch.last_commit_date.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_branches_for_repo(&self, repo_id: &str) -> Result<Vec<Branch>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, repo_id, name, sha, ahead_by, behind_by, status, last_commit_date
             FROM branches
             WHERE repo_id = ?1
             ORDER BY name",
        )?;

        let branches = stmt
            .query_map([repo_id], |row| {
                let status_str: String = row.get(6)?;
                Ok(Branch {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    name: row.get(2)?,
                    sha: row.get(3)?,
                    ahead_by: row.get::<_, i64>(4)? as u32,
                    behind_by: row.get::<_, i64>(5)? as u32,
                    status: status_str.parse().unwrap_or(BranchStatus::ReadyForPR),
                    last_commit_date: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(branches)
    }

    pub fn save_pull_request(&self, pr: &PullRequest) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO pull_requests (repo_id, branch_id, number, state, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &pr.repo_id,
                pr.branch_id,
                pr.number as i64,
                pr.state.to_string(),
                &pr.title,
                &pr.created_at.to_rfc3339(),
                &pr.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_pull_requests_for_repo(&self, repo_id: &str) -> Result<Vec<PullRequest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, repo_id, branch_id, number, state, title, created_at, updated_at
             FROM pull_requests
             WHERE repo_id = ?1
             ORDER BY number DESC",
        )?;

        let prs = stmt
            .query_map([repo_id], |row| {
                let state_str: String = row.get(4)?;
                Ok(PullRequest {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    branch_id: row.get(2)?,
                    number: row.get::<_, i64>(3)? as u32,
                    state: state_str.parse().unwrap_or(PRState::Closed),
                    title: row.get(5)?,
                    created_at: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    updated_at: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(prs)
    }

    pub fn clear_branches_for_repo(&self, repo_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM branches WHERE repo_id = ?1", params![repo_id])?;
        Ok(())
    }

    pub fn clear_pull_requests_for_repo(&self, repo_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM pull_requests WHERE repo_id = ?1",
            params![repo_id],
        )?;
        Ok(())
    }

    pub fn save_commit(&self, commit: &Commit) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO commits (branch_id, sha, message, author_name, author_email, authored_date, committer_name, committer_email, committed_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                commit.branch_id,
                &commit.sha,
                &commit.message,
                &commit.author_name,
                &commit.author_email,
                &commit.authored_date.to_rfc3339(),
                &commit.committer_name,
                &commit.committer_email,
                &commit.committed_date.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_commits_for_branch(&self, branch_id: i64) -> Result<Vec<Commit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, branch_id, sha, message, author_name, author_email, authored_date, committer_name, committer_email, committed_date
             FROM commits
             WHERE branch_id = ?1
             ORDER BY committed_date DESC"
        )?;

        let commits = stmt
            .query_map([branch_id], |row| {
                Ok(Commit {
                    id: row.get(0)?,
                    branch_id: row.get(1)?,
                    sha: row.get(2)?,
                    message: row.get(3)?,
                    author_name: row.get(4)?,
                    author_email: row.get(5)?,
                    authored_date: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    committer_name: row.get(7)?,
                    committer_email: row.get(8)?,
                    committed_date: row.get::<_, String>(9)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(commits)
    }

    pub fn clear_commits_for_branch(&self, branch_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM commits WHERE branch_id = ?1",
            params![branch_id],
        )?;
        Ok(())
    }

    // Group management methods
    pub fn create_group(&self, name: &str, display_order: i32) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO groups (name, display_order, created_at) VALUES (?1, ?2, ?3)",
            params![name, display_order, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, display_order, created_at FROM groups ORDER BY display_order",
        )?;

        let groups = stmt
            .query_map([], |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_order: row.get(2)?,
                    created_at: row.get::<_, String>(3)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(groups)
    }

    pub fn add_repo_to_group(&self, repo_id: &str, group_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at) VALUES (?1, ?2, ?3)",
            params![repo_id, group_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn remove_repo_from_group(&self, repo_id: &str, group_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM repo_groups WHERE repo_id = ?1 AND group_id = ?2",
            params![repo_id, group_id],
        )?;
        Ok(())
    }

    pub fn get_repos_in_group(&self, group_id: i64) -> Result<Vec<Repository>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.owner, r.name, r.language, r.description, r.pushed_at, r.created_at, r.updated_at, r.is_fork, r.priority
             FROM repositories r
             INNER JOIN repo_groups rg ON r.id = rg.repo_id
             WHERE rg.group_id = ?1
             ORDER BY r.pushed_at DESC"
        )?;

        let repos = stmt
            .query_map([group_id], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    language: row.get(3)?,
                    description: row.get(4)?,
                    pushed_at: row.get::<_, String>(5)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    created_at: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    updated_at: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    is_fork: row.get::<_, i32>(8)? != 0,
                    priority: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    pub fn get_ungrouped_repositories(&self) -> Result<Vec<Repository>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, owner, name, language, description, pushed_at, created_at, updated_at, is_fork, priority
             FROM repositories
             WHERE id NOT IN (SELECT repo_id FROM repo_groups)
             ORDER BY pushed_at DESC"
        )?;

        let repos = stmt
            .query_map([], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    language: row.get(3)?,
                    description: row.get(4)?,
                    pushed_at: row.get::<_, String>(5)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    created_at: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    updated_at: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    is_fork: row.get::<_, i32>(8)? != 0,
                    priority: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    pub fn delete_group(&self, group_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM groups WHERE id = ?1", params![group_id])?;
        Ok(())
    }

    pub fn rename_group(&self, group_id: i64, new_name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE groups SET name = ?1 WHERE id = ?2",
            params![new_name, group_id],
        )?;
        Ok(())
    }

    pub fn move_repo_to_group(&self, repo_id: &str, target_group_id: i64) -> Result<()> {
        // Remove from all groups first
        self.conn.execute(
            "DELETE FROM repo_groups WHERE repo_id = ?1",
            params![repo_id],
        )?;

        // Add to target group
        self.conn.execute(
            "INSERT INTO repo_groups (repo_id, group_id, added_at) VALUES (?1, ?2, ?3)",
            params![repo_id, target_group_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn remove_repo_from_all_groups(&self, repo_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM repo_groups WHERE repo_id = ?1",
            params![repo_id],
        )?;
        Ok(())
    }

    // Local repository root paths management
    pub fn add_local_repo_root(&self, path: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO local_repo_roots (path, enabled, created_at)
             VALUES (?1, ?2, ?3)",
            params![path, 1, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_local_repo_roots(&self) -> Result<Vec<crate::models::LocalRepoRoot>> {
        use crate::models::LocalRepoRoot;

        let mut stmt = self.conn.prepare(
            "SELECT id, path, enabled, created_at
             FROM local_repo_roots
             ORDER BY created_at DESC",
        )?;

        let roots = stmt
            .query_map([], |row| {
                Ok(LocalRepoRoot {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    created_at: row.get::<_, String>(3)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(roots)
    }

    pub fn remove_local_repo_root(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM local_repo_roots WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_local_repo_root(&self, id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE local_repo_roots SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )?;
        Ok(())
    }

    // Local repository status management
    pub fn save_local_repo_status(&self, status: &crate::models::LocalRepoStatus) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO local_repo_status
             (repo_id, local_path, current_branch, uncommitted_files, unpushed_commits, behind_commits, is_dirty, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &status.repo_id,
                &status.local_path,
                &status.current_branch,
                status.uncommitted_files as i64,
                status.unpushed_commits as i64,
                status.behind_commits as i64,
                status.is_dirty as i32,
                &status.last_checked.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_local_repo_status(
        &self,
        repo_id: &str,
    ) -> Result<Option<crate::models::LocalRepoStatus>> {
        use crate::models::LocalRepoStatus;

        let mut stmt = self.conn.prepare(
            "SELECT id, repo_id, local_path, current_branch, uncommitted_files, unpushed_commits, behind_commits, is_dirty, last_checked
             FROM local_repo_status
             WHERE repo_id = ?1"
        )?;

        let mut rows = stmt.query(params![repo_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(LocalRepoStatus {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                local_path: row.get(2)?,
                current_branch: row.get(3)?,
                uncommitted_files: row.get::<_, i64>(4)? as u32,
                unpushed_commits: row.get::<_, i64>(5)? as u32,
                behind_commits: row.get::<_, i64>(6)? as u32,
                is_dirty: row.get::<_, i32>(7)? != 0,
                last_checked: row.get::<_, String>(8)?.parse().map_err(|_| {
                    rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                })?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_local_repo_statuses(&self) -> Result<Vec<crate::models::LocalRepoStatus>> {
        use crate::models::LocalRepoStatus;

        let mut stmt = self.conn.prepare(
            "SELECT id, repo_id, local_path, current_branch, uncommitted_files, unpushed_commits, behind_commits, is_dirty, last_checked
             FROM local_repo_status
             ORDER BY last_checked DESC"
        )?;

        let statuses = stmt
            .query_map([], |row| {
                Ok(LocalRepoStatus {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    local_path: row.get(2)?,
                    current_branch: row.get(3)?,
                    uncommitted_files: row.get::<_, i64>(4)? as u32,
                    unpushed_commits: row.get::<_, i64>(5)? as u32,
                    behind_commits: row.get::<_, i64>(6)? as u32,
                    is_dirty: row.get::<_, i32>(7)? != 0,
                    last_checked: row.get::<_, String>(8)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(statuses)
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM config WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_repositories_updated_since(&self, since: &str) -> Result<Vec<Repository>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, owner, name, language, description, pushed_at, created_at, updated_at, is_fork, priority
             FROM repositories
             WHERE pushed_at > ?1
             ORDER BY priority DESC, pushed_at DESC"
        )?;

        let repos = stmt
            .query_map([since], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    owner: row.get(1)?,
                    name: row.get(2)?,
                    language: row.get(3)?,
                    description: row.get(4)?,
                    pushed_at: row.get::<_, String>(5)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    created_at: row.get::<_, String>(6)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    updated_at: row.get::<_, String>(7)?.parse().map_err(|_| {
                        rusqlite::Error::InvalidParameterName("Invalid date".to_string())
                    })?,
                    is_fork: row.get::<_, i32>(8)? != 0,
                    priority: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(repos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_create_database() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open_or_create(&db_path);
        assert!(db.is_ok());
    }

    #[test]
    fn test_save_and_load_repository() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open_or_create(&db_path).unwrap();

        let repo = Repository {
            id: "test/repo".to_string(),
            owner: "test".to_string(),
            name: "repo".to_string(),
            language: Some("Rust".to_string()),
            description: Some("Test repo".to_string()),
            pushed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_fork: false,
            priority: 0.5,
        };

        db.save_repository(&repo).unwrap();

        let repos = db.get_all_repositories().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].id, "test/repo");
        assert_eq!(repos[0].owner, "test");
    }

    #[test]
    fn test_config_get_set() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open_or_create(&db_path).unwrap();

        // Test setting and getting a config value
        db.set_config("test_key", "test_value").unwrap();
        let value = db.get_config("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test updating an existing config value
        db.set_config("test_key", "new_value").unwrap();
        let value = db.get_config("test_key").unwrap();
        assert_eq!(value, Some("new_value".to_string()));

        // Test getting a non-existent key
        let value = db.get_config("non_existent").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_get_repositories_updated_since() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open_or_create(&db_path).unwrap();

        let now = Utc::now();
        let old_time = now - chrono::Duration::days(5);
        let recent_time = now - chrono::Duration::hours(1);

        // Create an old repository
        let old_repo = Repository {
            id: "test/old-repo".to_string(),
            owner: "test".to_string(),
            name: "old-repo".to_string(),
            language: Some("Rust".to_string()),
            description: Some("Old repo".to_string()),
            pushed_at: old_time,
            created_at: old_time,
            updated_at: old_time,
            is_fork: false,
            priority: 0.5,
        };

        // Create a recent repository
        let recent_repo = Repository {
            id: "test/recent-repo".to_string(),
            owner: "test".to_string(),
            name: "recent-repo".to_string(),
            language: Some("Rust".to_string()),
            description: Some("Recent repo".to_string()),
            pushed_at: recent_time,
            created_at: recent_time,
            updated_at: recent_time,
            is_fork: false,
            priority: 0.5,
        };

        db.save_repository(&old_repo).unwrap();
        db.save_repository(&recent_repo).unwrap();

        // Get repositories updated since 2 days ago
        let cutoff = (now - chrono::Duration::days(2)).to_rfc3339();
        let repos = db.get_repositories_updated_since(&cutoff).unwrap();

        // Should only return the recent repository
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].id, "test/recent-repo");

        // Get repositories updated since 10 days ago
        let cutoff = (now - chrono::Duration::days(10)).to_rfc3339();
        let repos = db.get_repositories_updated_since(&cutoff).unwrap();

        // Should return both repositories
        assert_eq!(repos.len(), 2);
    }
}
