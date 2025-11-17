//! Real GitHub client implementation using gh CLI

use super::{client_trait::GitHubClient, commands};
use crate::{
    models::{Branch, BranchStatus, Commit, PullRequest, Repository},
    Result,
};

/// Real GitHub client that delegates to gh CLI commands
pub struct RealGitHubClient;

impl RealGitHubClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealGitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubClient for RealGitHubClient {
    fn list_repos(&self, owner: &str, limit: usize) -> Result<Vec<Repository>> {
        commands::list_repos(owner, limit)
    }

    fn fetch_branches(&self, repo_id: &str) -> Result<Vec<Branch>> {
        commands::fetch_branches(repo_id)
    }

    fn fetch_pull_requests(&self, repo_id: &str) -> Result<Vec<PullRequest>> {
        commands::fetch_pull_requests(repo_id)
    }

    fn fetch_commits(
        &self,
        repo_id: &str,
        branch_name: &str,
        branch_id: i64,
    ) -> Result<Vec<Commit>> {
        commands::fetch_commits(repo_id, branch_name, branch_id)
    }

    fn classify_branch_status(
        &self,
        branch: &Branch,
        prs: &[PullRequest],
        default_branch: &str,
    ) -> BranchStatus {
        commands::classify_branch_status(branch, prs, default_branch)
    }

    fn create_pull_request(
        &self,
        repo_id: &str,
        branch_name: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<String> {
        commands::create_pull_request(repo_id, branch_name, title, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_client_creation() {
        let _client = RealGitHubClient::new();
        let _client2: RealGitHubClient = Default::default();
        // Just verify it can be constructed
    }
}
