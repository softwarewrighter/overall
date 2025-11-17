//! Trait for GitHub operations, enabling dependency injection for testing

use crate::{
    models::{Branch, BranchStatus, Commit, PullRequest, Repository},
    Result,
};

/// Trait for GitHub client operations
///
/// This trait abstracts all GitHub API interactions, allowing us to:
/// - Inject real implementations (via `gh` CLI) in production
/// - Inject mock implementations in tests (without network/auth dependencies)
/// - Verify behavior through spy/mock patterns
pub trait GitHubClient {
    /// List repositories for an owner (user or organization)
    fn list_repos(&self, owner: &str, limit: usize) -> Result<Vec<Repository>>;

    /// Fetch all branches for a repository
    fn fetch_branches(&self, repo_id: &str) -> Result<Vec<Branch>>;

    /// Fetch all pull requests for a repository
    fn fetch_pull_requests(&self, repo_id: &str) -> Result<Vec<PullRequest>>;

    /// Fetch commits for a specific branch
    fn fetch_commits(
        &self,
        repo_id: &str,
        branch_name: &str,
        branch_id: i64,
    ) -> Result<Vec<Commit>>;

    /// Classify branch status based on PR state
    fn classify_branch_status(
        &self,
        branch: &Branch,
        prs: &[PullRequest],
        default_branch: &str,
    ) -> BranchStatus;

    /// Create a pull request for a branch
    /// Returns the PR URL on success
    fn create_pull_request(
        &self,
        repo_id: &str,
        branch_name: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<String>;
}
