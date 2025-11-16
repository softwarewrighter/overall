//! GitHub API integration via gh CLI

pub mod commands;

use crate::{
    models::{Branch, BranchStatus, Commit, PullRequest, Repository},
    Result,
};

/// List repositories for a given owner (user or organization)
pub fn list_repos(owner: &str, limit: usize) -> Result<Vec<Repository>> {
    commands::list_repos(owner, limit)
}

/// Fetch all branches for a repository
pub fn fetch_branches(repo_id: &str) -> Result<Vec<Branch>> {
    commands::fetch_branches(repo_id)
}

/// Fetch all pull requests for a repository
pub fn fetch_pull_requests(repo_id: &str) -> Result<Vec<PullRequest>> {
    commands::fetch_pull_requests(repo_id)
}

/// Fetch commits for a specific branch
pub fn fetch_commits(repo_id: &str, branch_name: &str, branch_id: i64) -> Result<Vec<Commit>> {
    commands::fetch_commits(repo_id, branch_name, branch_id)
}

/// Classify branch status based on PR state
pub fn classify_branch_status(
    branch: &Branch,
    prs: &[PullRequest],
    default_branch: &str,
) -> BranchStatus {
    commands::classify_branch_status(branch, prs, default_branch)
}

/// Create a pull request for a branch
/// Returns the PR URL on success
pub fn create_pull_request(
    repo_id: &str,
    branch_name: &str,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<String> {
    commands::create_pull_request(repo_id, branch_name, title, body)
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

    #[test]
    fn test_fetch_branches_returns_branches() {
        // Use an existing softwarewrighter project for testing
        let result = fetch_branches("softwarewrighter/proact");

        if let Err(e) = &result {
            eprintln!("Error fetching branches: {}", e);
        }
        assert!(result.is_ok(), "Should successfully fetch branches");

        let branches = result.unwrap();
        assert!(
            !branches.is_empty(),
            "Repository should have at least one branch"
        );

        // Verify branch structure
        let branch = &branches[0];
        assert!(!branch.name.is_empty());
        assert!(!branch.sha.is_empty());
        assert_eq!(branch.repo_id, "softwarewrighter/proact");
    }

    #[test]
    fn test_fetch_branches_has_main_branch() {
        let result = fetch_branches("softwarewrighter/proact");
        assert!(result.is_ok());

        let branches = result.unwrap();
        let main_branch = branches.iter().find(|b| b.name == "main");
        assert!(main_branch.is_some(), "Should have main branch");

        // Main branch should have 0 ahead/behind of itself
        let main = main_branch.unwrap();
        assert_eq!(main.ahead_by, 0, "Main should be 0 ahead of itself");
        assert_eq!(main.behind_by, 0, "Main should be 0 behind of itself");
    }

    #[test]
    fn test_fetch_branches_invalid_repo() {
        let result = fetch_branches("invalid-format");
        assert!(result.is_err(), "Invalid format should return error");

        let result = fetch_branches("softwarewrighter/nonexistent-repo-12345");
        assert!(result.is_err(), "Nonexistent repo should return error");
    }
}
