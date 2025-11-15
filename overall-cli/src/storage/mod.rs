//! Local SQLite storage

use crate::{
    models::{Branch, BranchStatus, PRState, PullRequest, Repository},
    Result,
};
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
             ORDER BY name"
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
             ORDER BY number DESC"
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
        self.conn.execute(
            "DELETE FROM branches WHERE repo_id = ?1",
            params![repo_id],
        )?;
        Ok(())
    }

    pub fn clear_pull_requests_for_repo(&self, repo_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM pull_requests WHERE repo_id = ?1",
            params![repo_id],
        )?;
        Ok(())
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
}
