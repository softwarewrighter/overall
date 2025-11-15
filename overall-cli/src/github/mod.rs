//! GitHub API integration via gh CLI

pub mod commands;

use crate::{models::Repository, Result};

/// List repositories for a given owner (user or organization)
pub fn list_repos(owner: &str, limit: usize) -> Result<Vec<Repository>> {
    commands::list_repos(owner, limit)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_repos_returns_repositories() {
        // This will be implemented in Phase 1.1
        // For now, we'll skip this test until gh is authenticated
    }
}
