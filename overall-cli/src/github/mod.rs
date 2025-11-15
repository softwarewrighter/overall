//! GitHub API integration via gh CLI

pub mod commands;

use crate::{models::Repository, Result};

/// List repositories for a given owner (user or organization)
pub fn list_repos(owner: &str, limit: usize) -> Result<Vec<Repository>> {
    commands::list_repos(owner, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_repos_returns_repositories() {
        // Integration test - requires gh CLI authenticated
        let result = list_repos("softwarewrighter", 10);

        assert!(result.is_ok(), "Should successfully list repositories");

        let repos = result.unwrap();
        assert!(
            !repos.is_empty(),
            "softwarewrighter should have repositories"
        );

        // Verify repository structure
        let repo = &repos[0];
        assert_eq!(repo.owner, "softwarewrighter");
        assert!(!repo.name.is_empty());
        assert!(!repo.id.is_empty());
        assert!(repo.id.contains('/'), "ID should be owner/name format");
    }

    #[test]
    fn test_list_repos_respects_limit() {
        let result = list_repos("softwarewrighter", 5);
        assert!(result.is_ok());

        let repos = result.unwrap();
        assert!(repos.len() <= 5, "Should respect the limit parameter");
    }

    #[test]
    fn test_list_repos_invalid_owner() {
        let result = list_repos("", 10);
        assert!(result.is_err(), "Empty owner should return error");

        let result = list_repos("invalid@name", 10);
        assert!(result.is_err(), "Invalid characters should return error");
    }

    #[test]
    fn test_list_repos_sorted_by_push_date() {
        let result = list_repos("softwarewrighter", 10);
        assert!(result.is_ok());

        let repos = result.unwrap();
        if repos.len() > 1 {
            // Verify repos are sorted by most recent pushed_at
            for i in 0..repos.len() - 1 {
                assert!(
                    repos[i].pushed_at >= repos[i + 1].pushed_at,
                    "Repositories should be sorted by pushed_at descending"
                );
            }
        }
    }
}
