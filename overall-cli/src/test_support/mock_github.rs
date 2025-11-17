//! Mock GitHub client for testing

use crate::github::client_trait::GitHubClient;
use crate::models::*;
use crate::Result;
use std::cell::RefCell;
use std::collections::HashMap;

/// Mock GitHub client for testing
///
/// This mock allows tests to:
/// - Provide canned responses for GitHub operations
/// - Track what operations were called
/// - Verify expectations about what should/shouldn't be called
#[derive(Default)]
pub struct MockGitHubClient {
    // Canned data to return
    pub repos: HashMap<String, Vec<Repository>>,
    pub branches: HashMap<String, Vec<Branch>>,
    pub pull_requests: HashMap<String, Vec<PullRequest>>,
    pub commits: HashMap<(String, String), Vec<Commit>>,

    // Tracking what was called (using RefCell for interior mutability in trait methods)
    pub created_prs: RefCell<Vec<CreatedPR>>,

    // Expectations for verification
    pub expect_create_pr: Vec<String>, // repo_ids that should have PR created
    pub expect_no_create_pr: Vec<String>, // repo_ids that should NOT have PR created
}

/// Record of a created PR for verification
#[derive(Debug, Clone)]
pub struct CreatedPR {
    pub repo_id: String,
    pub branch_name: String,
    pub title: String,
    pub body: String,
}

impl MockGitHubClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a repository to the mock's canned data
    pub fn with_repo(mut self, owner: &str, repo: Repository) -> Self {
        self.repos.entry(owner.to_string()).or_default().push(repo);
        self
    }

    /// Add branches for a repository
    pub fn with_branches(mut self, repo_id: &str, branches: Vec<Branch>) -> Self {
        self.branches.insert(repo_id.to_string(), branches);
        self
    }

    /// Add pull requests for a repository
    pub fn with_pull_requests(mut self, repo_id: &str, prs: Vec<PullRequest>) -> Self {
        self.pull_requests.insert(repo_id.to_string(), prs);
        self
    }

    /// Add commits for a branch
    pub fn with_commits(mut self, repo_id: &str, branch_name: &str, commits: Vec<Commit>) -> Self {
        self.commits
            .insert((repo_id.to_string(), branch_name.to_string()), commits);
        self
    }

    /// Expect that create_pr will be called for this repo
    pub fn expect_create_pr_for(mut self, repo_id: &str) -> Self {
        self.expect_create_pr.push(repo_id.to_string());
        self
    }

    /// Expect that create_pr will NOT be called for this repo
    pub fn expect_no_create_pr_for(mut self, repo_id: &str) -> Self {
        self.expect_no_create_pr.push(repo_id.to_string());
        self
    }

    /// Verify that all expectations were met
    ///
    /// Call this at the end of your test to ensure the mock was used as expected
    pub fn verify(&self) {
        let created_prs = self.created_prs.borrow();

        // Check that all expected PRs were created
        for expected_repo in &self.expect_create_pr {
            assert!(
                created_prs.iter().any(|pr| pr.repo_id == *expected_repo),
                "Expected create_pr to be called for '{}', but it wasn't. Created PRs: {:?}",
                expected_repo,
                created_prs
            );
        }

        // Check that no unexpected PRs were created
        for unexpected_repo in &self.expect_no_create_pr {
            assert!(
                !created_prs.iter().any(|pr| pr.repo_id == *unexpected_repo),
                "Expected create_pr NOT to be called for '{}', but it was. Created PRs: {:?}",
                unexpected_repo,
                created_prs
            );
        }
    }

    /// Get all created PRs for inspection
    pub fn get_created_prs(&self) -> Vec<CreatedPR> {
        self.created_prs.borrow().clone()
    }
}

impl GitHubClient for MockGitHubClient {
    fn list_repos(&self, owner: &str, _limit: usize) -> Result<Vec<Repository>> {
        Ok(self.repos.get(owner).cloned().unwrap_or_default())
    }

    fn fetch_branches(&self, repo_id: &str) -> Result<Vec<Branch>> {
        Ok(self.branches.get(repo_id).cloned().unwrap_or_default())
    }

    fn fetch_pull_requests(&self, repo_id: &str) -> Result<Vec<PullRequest>> {
        Ok(self.pull_requests.get(repo_id).cloned().unwrap_or_default())
    }

    fn fetch_commits(
        &self,
        repo_id: &str,
        branch_name: &str,
        _branch_id: i64,
    ) -> Result<Vec<Commit>> {
        Ok(self
            .commits
            .get(&(repo_id.to_string(), branch_name.to_string()))
            .cloned()
            .unwrap_or_default())
    }

    fn classify_branch_status(
        &self,
        branch: &Branch,
        prs: &[PullRequest],
        default_branch: &str,
    ) -> BranchStatus {
        // Use the real classification logic for consistency
        crate::github::commands::classify_branch_status(branch, prs, default_branch)
    }

    fn create_pull_request(
        &self,
        repo_id: &str,
        branch_name: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<String> {
        // Record that this was called
        let default_title = branch_name.replace(['-', '_'], " ");
        let default_body = "Created via Overall";

        self.created_prs.borrow_mut().push(CreatedPR {
            repo_id: repo_id.to_string(),
            branch_name: branch_name.to_string(),
            title: title.unwrap_or(&default_title).to_string(),
            body: body.unwrap_or(default_body).to_string(),
        });

        // Return a fake PR URL
        Ok(format!("https://github.com/{}/pull/123", repo_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_mock_client_returns_canned_repos() {
        let repo = Repository {
            id: "owner/repo".to_string(),
            owner: "owner".to_string(),
            name: "repo".to_string(),
            language: None,
            description: None,
            pushed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_fork: false,
            priority: 0.0,
        };

        let mock = MockGitHubClient::new().with_repo("owner", repo.clone());

        let result = mock.list_repos("owner", 10).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "owner/repo");
    }

    #[test]
    fn test_mock_client_tracks_created_prs() {
        let mock = MockGitHubClient::new();

        let _url = mock
            .create_pull_request("owner/repo", "feature-branch", None, None)
            .unwrap();

        let created = mock.get_created_prs();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].repo_id, "owner/repo");
        assert_eq!(created[0].branch_name, "feature-branch");
    }

    #[test]
    fn test_mock_client_verify_expectations_pass() {
        let mock = MockGitHubClient::new()
            .expect_create_pr_for("owner/repo1")
            .expect_no_create_pr_for("owner/repo2");

        // Create PR for repo1
        let _url = mock
            .create_pull_request("owner/repo1", "feature", None, None)
            .unwrap();

        // Verify should pass
        mock.verify();
    }

    #[test]
    #[should_panic(expected = "Expected create_pr to be called")]
    fn test_mock_client_verify_fails_on_missing_call() {
        let mock = MockGitHubClient::new().expect_create_pr_for("owner/repo");

        // Don't create the PR - verification should fail
        mock.verify();
    }

    #[test]
    #[should_panic(expected = "Expected create_pr NOT to be called")]
    fn test_mock_client_verify_fails_on_unexpected_call() {
        let mock = MockGitHubClient::new().expect_no_create_pr_for("owner/repo");

        // Create the PR anyway - verification should fail
        let _url = mock
            .create_pull_request("owner/repo", "feature", None, None)
            .unwrap();

        mock.verify();
    }
}
