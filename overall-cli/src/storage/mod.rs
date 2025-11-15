//! Local SQLite storage

use crate::{models::Repository, Result};
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
