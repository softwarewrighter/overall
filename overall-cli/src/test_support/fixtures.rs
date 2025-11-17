//! Test fixtures and builders for creating test data

use crate::models::*;
use crate::storage::Database;
use crate::Result;
use chrono::{DateTime, Utc};
use tempfile::{NamedTempFile, TempPath};

/// Test database wrapper with builder pattern
///
/// Provides a fresh SQLite database for each test with helper methods
/// to populate it with test data in a fluent, readable way.
pub struct TestDatabase {
    pub db: Database,
    _temp_file: TempPath,
}

impl TestDatabase {
    /// Create a new test database with schema initialized
    pub fn new() -> Result<Self> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.into_temp_path();
        let db = Database::open_or_create(&temp_path)?;

        Ok(Self {
            db,
            _temp_file: temp_path,
        })
    }

    /// Add a repository using a builder
    pub fn with_repo(self, builder: RepoBuilder) -> Result<Self> {
        let repo = builder.build();
        self.db.save_repository(&repo)?;
        Ok(self)
    }

    /// Add a branch for a repository
    pub fn with_branch(self, repo_id: &str, builder: BranchBuilder) -> Result<Self> {
        let branch = builder.build(repo_id);
        self.db.save_branch(&branch)?;
        Ok(self)
    }

    /// Add a pull request for a repository
    pub fn with_pull_request(self, _repo_id: &str, pr: PullRequest) -> Result<Self> {
        self.db.save_pull_request(&pr)?;
        Ok(self)
    }

    /// Create a group and optionally add repos to it
    pub fn with_group(self, name: &str, display_order: i32, repo_ids: Vec<&str>) -> Result<Self> {
        let group_id = self.db.create_group(name, display_order)?;
        for repo_id in repo_ids {
            self.db.add_repo_to_group(repo_id, group_id)?;
        }
        Ok(self)
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new().expect("Failed to create test database")
    }
}

/// Builder for creating test repositories
///
/// Use this to create repositories with specific characteristics for testing.
///
/// # Example
/// ```ignore
/// let repo = RepoBuilder::new("my-repo")
///     .owner("octocat")
///     .with_language("Rust")
///     .is_fork(false)
///     .build();
/// ```
#[derive(Clone)]
pub struct RepoBuilder {
    name: String,
    owner: String,
    language: Option<String>,
    description: Option<String>,
    pushed_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_fork: bool,
    priority: f32,
}

impl RepoBuilder {
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            owner: "testowner".to_string(),
            language: None,
            description: None,
            pushed_at: now,
            created_at: now,
            updated_at: now,
            is_fork: false,
            priority: 0.0,
        }
    }

    pub fn owner(mut self, owner: &str) -> Self {
        self.owner = owner.to_string();
        self
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = Some(language.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn pushed_at(mut self, time: DateTime<Utc>) -> Self {
        self.pushed_at = time;
        self
    }

    pub fn is_fork(mut self, is_fork: bool) -> Self {
        self.is_fork = is_fork;
        self
    }

    pub fn priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Repository {
        Repository {
            id: format!("{}/{}", self.owner, self.name),
            owner: self.owner,
            name: self.name,
            language: self.language,
            description: self.description,
            pushed_at: self.pushed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_fork: self.is_fork,
            priority: self.priority,
        }
    }
}

/// Builder for creating test branches
///
/// Use this to create branches in specific states for testing.
///
/// # Example
/// ```ignore
/// let branch = BranchBuilder::new("feature-auth")
///     .ahead(5)
///     .behind(0)
///     .with_status(BranchStatus::ReadyForPR)
///     .build("owner/repo");
/// ```
#[derive(Clone)]
pub struct BranchBuilder {
    name: String,
    sha: String,
    ahead_by: u32,
    behind_by: u32,
    status: BranchStatus,
    last_commit_date: DateTime<Utc>,
}

impl BranchBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sha: "abc123def456".to_string(),
            ahead_by: 0,
            behind_by: 0,
            status: BranchStatus::ReadyForPR,
            last_commit_date: Utc::now(),
        }
    }

    pub fn sha(mut self, sha: &str) -> Self {
        self.sha = sha.to_string();
        self
    }

    pub fn ahead(mut self, commits: u32) -> Self {
        self.ahead_by = commits;
        self
    }

    pub fn behind(mut self, commits: u32) -> Self {
        self.behind_by = commits;
        self
    }

    pub fn with_status(mut self, status: BranchStatus) -> Self {
        self.status = status;
        self
    }

    pub fn last_commit_date(mut self, date: DateTime<Utc>) -> Self {
        self.last_commit_date = date;
        self
    }

    pub fn build(self, repo_id: &str) -> Branch {
        Branch {
            id: 0, // Will be assigned by database
            repo_id: repo_id.to_string(),
            name: self.name,
            sha: self.sha,
            ahead_by: self.ahead_by,
            behind_by: self.behind_by,
            status: self.status,
            last_commit_date: self.last_commit_date,
        }
    }
}

/// Builder for creating test pull requests
///
/// # Example
/// ```ignore
/// let pr = PRBuilder::new(123, "feature-auth")
///     .state(PRState::Open)
///     .title("Add authentication")
///     .build("owner/repo");
/// ```
#[derive(Clone)]
pub struct PRBuilder {
    number: u32,
    state: PRState,
    title: String,
    branch_id: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PRBuilder {
    pub fn new(number: u32, title: &str) -> Self {
        let now = Utc::now();
        Self {
            number,
            state: PRState::Open,
            title: title.to_string(),
            branch_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn state(mut self, state: PRState) -> Self {
        self.state = state;
        self
    }

    pub fn branch_id(mut self, id: i64) -> Self {
        self.branch_id = Some(id);
        self
    }

    pub fn created_at(mut self, time: DateTime<Utc>) -> Self {
        self.created_at = time;
        self
    }

    pub fn updated_at(mut self, time: DateTime<Utc>) -> Self {
        self.updated_at = time;
        self
    }

    pub fn build(self, repo_id: &str) -> PullRequest {
        PullRequest {
            id: 0, // Will be assigned by database
            repo_id: repo_id.to_string(),
            branch_id: self.branch_id,
            number: self.number,
            state: self.state,
            title: self.title,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Builder for creating test commits
pub struct CommitBuilder {
    sha: String,
    message: String,
    author_name: String,
    author_email: String,
    authored_date: DateTime<Utc>,
    committer_name: String,
    committer_email: String,
    committed_date: DateTime<Utc>,
}

impl CommitBuilder {
    pub fn new(sha: &str, message: &str) -> Self {
        let now = Utc::now();
        Self {
            sha: sha.to_string(),
            message: message.to_string(),
            author_name: "Test Author".to_string(),
            author_email: "author@test.com".to_string(),
            authored_date: now,
            committer_name: "Test Committer".to_string(),
            committer_email: "committer@test.com".to_string(),
            committed_date: now,
        }
    }

    pub fn author(mut self, name: &str, email: &str) -> Self {
        self.author_name = name.to_string();
        self.author_email = email.to_string();
        self
    }

    pub fn authored_date(mut self, date: DateTime<Utc>) -> Self {
        self.authored_date = date;
        self
    }

    pub fn build(self, branch_id: i64) -> Commit {
        Commit {
            id: 0, // Will be assigned by database
            branch_id,
            sha: self.sha,
            message: self.message,
            author_name: self.author_name,
            author_email: self.author_email,
            authored_date: self.authored_date,
            committer_name: self.committer_name,
            committer_email: self.committer_email,
            committed_date: self.committed_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_builder() {
        let repo = RepoBuilder::new("test-repo")
            .owner("octocat")
            .with_language("Rust")
            .with_description("A test repo")
            .is_fork(false)
            .priority(10.0)
            .build();

        assert_eq!(repo.id, "octocat/test-repo");
        assert_eq!(repo.owner, "octocat");
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.language, Some("Rust".to_string()));
        assert_eq!(repo.description, Some("A test repo".to_string()));
        assert!(!repo.is_fork);
        assert_eq!(repo.priority, 10.0);
    }

    #[test]
    fn test_branch_builder() {
        let branch = BranchBuilder::new("feature-auth")
            .ahead(5)
            .behind(2)
            .with_status(BranchStatus::InReview)
            .build("owner/repo");

        assert_eq!(branch.name, "feature-auth");
        assert_eq!(branch.repo_id, "owner/repo");
        assert_eq!(branch.ahead_by, 5);
        assert_eq!(branch.behind_by, 2);
        assert_eq!(branch.status, BranchStatus::InReview);
    }

    #[test]
    fn test_pr_builder() {
        let pr = PRBuilder::new(123, "Add feature")
            .state(PRState::Open)
            .branch_id(42)
            .build("owner/repo");

        assert_eq!(pr.number, 123);
        assert_eq!(pr.title, "Add feature");
        assert_eq!(pr.state, PRState::Open);
        assert_eq!(pr.repo_id, "owner/repo");
        assert_eq!(pr.branch_id, Some(42));
    }

    #[test]
    fn test_database_with_repo() -> Result<()> {
        let test_db = TestDatabase::new()?.with_repo(RepoBuilder::new("test-repo"))?;

        let repos = test_db.db.get_all_repositories()?;
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "test-repo");

        Ok(())
    }

    #[test]
    fn test_database_with_branch() -> Result<()> {
        let test_db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("test-repo"))?
            .with_branch("testowner/test-repo", BranchBuilder::new("main"))?;

        let branches = test_db.db.get_branches_for_repo("testowner/test-repo")?;
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "main");

        Ok(())
    }

    #[test]
    fn test_database_with_group() -> Result<()> {
        let test_db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("repo1"))?
            .with_repo(RepoBuilder::new("repo2"))?
            .with_group("Test Group", 0, vec!["testowner/repo1", "testowner/repo2"])?;

        let groups = test_db.db.get_all_groups()?;
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Test Group");

        let repos = test_db.db.get_repos_in_group(groups[0].id)?;
        assert_eq!(repos.len(), 2);

        Ok(())
    }
}
