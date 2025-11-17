# Testing Strategy

## Philosophy

**Every bug raised, every edge case discovered, every scenario discussed must have a test.**

Tests are not optional. Tests are not just for happy paths. Tests define the contract of our system and prevent regressions.

## Core Principles

### 1. Test-Driven Development (TDD)

**ALWAYS write tests BEFORE implementation:**

1. **Red**: Write a failing test that describes the behavior you want
2. **Green**: Write the minimum code to make the test pass
3. **Refactor**: Improve the code while keeping tests green

### 2. Dependency Injection

**Never depend on external systems in tests:**

- ❌ Don't call real GitHub API (`gh` CLI)
- ❌ Don't require Ollama or network access
- ❌ Don't depend on filesystem state (except temp files)
- ✅ Use traits and mock implementations
- ✅ Inject dependencies via constructor/function parameters
- ✅ Create test fixtures with known state

### 3. Comprehensive Coverage

**Every feature needs tests for:**

1. **Happy path** - Normal successful operation
2. **Edge cases** - Empty inputs, boundary conditions, unusual states
3. **Error conditions** - Invalid inputs, missing data, constraint violations
4. **State transitions** - How operations change the database
5. **Combinations** - Multiple entities with different states

## Test Infrastructure

### Mock GitHub Client

ALL GitHub operations must go through a trait that can be mocked.

**File: `overall-cli/src/github/client_trait.rs`**

```rust
use anyhow::Result;
use crate::models::{Repository, Branch, PullRequest, Commit};

/// Trait for GitHub operations, allowing dependency injection in tests
pub trait GitHubClient {
    /// List repositories for an owner (user or org)
    fn list_repos(&self, owner: &str, limit: Option<u32>) -> Result<Vec<Repository>>;

    /// Fetch branches for a repository
    fn fetch_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>>;

    /// Fetch pull requests for a repository
    fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>>;

    /// Fetch commits for a branch
    fn fetch_commits(&self, owner: &str, repo: &str, branch: &str) -> Result<Vec<Commit>>;

    /// Create a pull request
    fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<PullRequest>;
}
```

**File: `overall-cli/src/github/real_client.rs`**

```rust
use super::client_trait::GitHubClient;
use super::commands; // Existing gh CLI wrapper functions
use anyhow::Result;
use crate::models::*;

/// Real GitHub client that uses gh CLI
pub struct RealGitHubClient;

impl GitHubClient for RealGitHubClient {
    fn list_repos(&self, owner: &str, limit: Option<u32>) -> Result<Vec<Repository>> {
        commands::list_repos(owner, limit)
    }

    fn fetch_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        commands::fetch_branches(owner, repo)
    }

    // ... implement other methods by delegating to existing commands module
}
```

**File: `overall-cli/src/test_support/mock_github.rs`**

```rust
use crate::github::client_trait::GitHubClient;
use crate::models::*;
use anyhow::{Result, bail};
use std::collections::HashMap;

/// Mock GitHub client for testing
pub struct MockGitHubClient {
    pub repos: HashMap<String, Vec<Repository>>,
    pub branches: HashMap<(String, String), Vec<Branch>>,
    pub pull_requests: HashMap<(String, String), Vec<PullRequest>>,
    pub commits: HashMap<(String, String, String), Vec<Commit>>,

    // Track what was called for verification
    pub created_prs: Vec<(String, String, String, String, String, String)>,

    // Expected calls for verification
    pub expect_create_pr: Vec<String>, // repo names that should have PR created
    pub expect_no_create_pr: Vec<String>, // repo names that should NOT have PR created
}

impl MockGitHubClient {
    pub fn new() -> Self {
        Self {
            repos: HashMap::new(),
            branches: HashMap::new(),
            pull_requests: HashMap::new(),
            commits: HashMap::new(),
            created_prs: Vec::new(),
            expect_create_pr: Vec::new(),
            expect_no_create_pr: Vec::new(),
        }
    }

    pub fn with_repo(mut self, owner: &str, repo: Repository) -> Self {
        self.repos.entry(owner.to_string())
            .or_insert_with(Vec::new)
            .push(repo);
        self
    }

    pub fn with_branches(mut self, owner: &str, repo: &str, branches: Vec<Branch>) -> Self {
        self.branches.insert((owner.to_string(), repo.to_string()), branches);
        self
    }

    pub fn with_pull_requests(mut self, owner: &str, repo: &str, prs: Vec<PullRequest>) -> Self {
        self.pull_requests.insert((owner.to_string(), repo.to_string()), prs);
        self
    }

    pub fn expect_create_pr_for(mut self, repo: &str) -> Self {
        self.expect_create_pr.push(repo.to_string());
        self
    }

    pub fn expect_no_create_pr_for(mut self, repo: &str) -> Self {
        self.expect_no_create_pr.push(repo.to_string());
        self
    }

    /// Verify that expectations were met (call at end of test)
    pub fn verify(&self) {
        for repo in &self.expect_create_pr {
            assert!(
                self.created_prs.iter().any(|(_, r, _, _, _, _)| r == repo),
                "Expected create_pr to be called for {}, but it wasn't",
                repo
            );
        }

        for repo in &self.expect_no_create_pr {
            assert!(
                !self.created_prs.iter().any(|(_, r, _, _, _, _)| r == repo),
                "Expected create_pr NOT to be called for {}, but it was",
                repo
            );
        }
    }
}

impl GitHubClient for MockGitHubClient {
    fn list_repos(&self, owner: &str, _limit: Option<u32>) -> Result<Vec<Repository>> {
        Ok(self.repos.get(owner).cloned().unwrap_or_default())
    }

    fn fetch_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>> {
        Ok(self.branches.get(&(owner.to_string(), repo.to_string()))
            .cloned()
            .unwrap_or_default())
    }

    fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        Ok(self.pull_requests.get(&(owner.to_string(), repo.to_string()))
            .cloned()
            .unwrap_or_default())
    }

    fn fetch_commits(&self, owner: &str, repo: &str, branch: &str) -> Result<Vec<Commit>> {
        Ok(self.commits.get(&(owner.to_string(), repo.to_string(), branch.to_string()))
            .cloned()
            .unwrap_or_default())
    }

    fn create_pull_request(
        &mut self,
        owner: &str,
        repo: &str,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<PullRequest> {
        self.created_prs.push((
            owner.to_string(),
            repo.to_string(),
            head.to_string(),
            base.to_string(),
            title.to_string(),
            body.to_string(),
        ));

        Ok(PullRequest {
            number: 123,
            title: title.to_string(),
            state: "open".to_string(),
            head_ref: head.to_string(),
            base_ref: base.to_string(),
            url: format!("https://github.com/{}/{}/pull/123", owner, repo),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}
```

### Database Test Fixtures

Use builder pattern to create test databases with known state.

**File: `overall-cli/src/test_support/fixtures.rs`**

```rust
use crate::models::*;
use crate::storage::Database;
use anyhow::Result;
use tempfile::{NamedTempFile, TempPath};

/// Test database with builder pattern for easy fixture creation
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

    /// Add a repository using builder pattern
    pub fn with_repo(self, builder: RepoBuilder) -> Result<Self> {
        let repo = builder.build();
        self.db.insert_repository(&repo)?;
        Ok(self)
    }

    /// Add a group and associate repos with it
    pub fn with_group(self, name: &str, repo_ids: Vec<i64>) -> Result<Self> {
        let group_id = self.db.create_group(name)?;
        for repo_id in repo_ids {
            self.db.add_repo_to_group(group_id, repo_id)?;
        }
        Ok(self)
    }
}

/// Builder for creating test repositories
pub struct RepoBuilder {
    name: String,
    owner: String,
    full_name: String,
    url: String,
    default_branch: String,
}

impl RepoBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            owner: "testowner".to_string(),
            full_name: format!("testowner/{}", name),
            url: format!("https://github.com/testowner/{}", name),
            default_branch: "main".to_string(),
        }
    }

    pub fn owner(mut self, owner: &str) -> Self {
        self.owner = owner.to_string();
        self.full_name = format!("{}/{}", owner, self.name);
        self.url = format!("https://github.com/{}/{}", owner, self.name);
        self
    }

    pub fn default_branch(mut self, branch: &str) -> Self {
        self.default_branch = branch.to_string();
        self
    }

    pub fn build(self) -> Repository {
        Repository {
            id: 0, // Will be assigned by database
            name: self.name,
            full_name: self.full_name,
            owner: self.owner,
            url: self.url,
            default_branch: self.default_branch,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_scanned_at: Some(chrono::Utc::now()),
        }
    }
}

/// Builder for creating test branches
pub struct BranchBuilder {
    name: String,
    is_default: bool,
    ahead_by: i32,
    behind_by: i32,
    has_open_pr: bool,
}

impl BranchBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_default: false,
            ahead_by: 0,
            behind_by: 0,
            has_open_pr: false,
        }
    }

    pub fn default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn ahead(mut self, commits: i32) -> Self {
        self.ahead_by = commits;
        self
    }

    pub fn behind(mut self, commits: i32) -> Self {
        self.behind_by = commits;
        self
    }

    pub fn with_open_pr(mut self) -> Self {
        self.has_open_pr = true;
        self
    }

    pub fn build(self, repo_id: i64) -> Branch {
        Branch {
            id: 0, // Will be assigned by database
            repository_id: repo_id,
            name: self.name,
            sha: "abc123".to_string(),
            protected: false,
            ahead_by: self.ahead_by,
            behind_by: self.behind_by,
            updated_at: chrono::Utc::now(),
        }
    }
}
```

## Test Scenarios

### Scenario 1: PR Creation Logic

**Test all combinations of repo states:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TestDatabase, RepoBuilder, MockGitHubClient};

    #[test]
    fn test_create_all_prs_skips_repos_with_open_prs() -> Result<()> {
        // Setup: Two repos, one with open PR, one ready for PR
        let db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("repo-with-pr"))?
            .with_repo(RepoBuilder::new("repo-ready"))?;

        // Insert branches and PRs
        let repo1_id = db.db.get_repository_id("testowner", "repo-with-pr")?;
        let repo2_id = db.db.get_repository_id("testowner", "repo-ready")?;

        db.db.insert_branch(&BranchBuilder::new("feature-1")
            .ahead(3)
            .build(repo1_id))?;
        db.db.insert_pull_request(&PullRequest {
            repository_id: repo1_id,
            number: 123,
            state: "open".to_string(),
            head_ref: "feature-1".to_string(),
            // ... other fields
        })?;

        db.db.insert_branch(&BranchBuilder::new("feature-2")
            .ahead(5)
            .build(repo2_id))?;

        // Setup mock GitHub - expect PR only for repo-ready
        let mock_gh = MockGitHubClient::new()
            .expect_no_create_pr_for("repo-with-pr")
            .expect_create_pr_for("repo-ready");

        // Execute
        let result = create_all_prs(&db.db, &mock_gh)?;

        // Verify
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped.contains(&"repo-with-pr".to_string()));
        assert_eq!(result.created.len(), 1);
        assert!(result.created.contains(&"repo-ready".to_string()));

        mock_gh.verify(); // Verify expectations were met

        Ok(())
    }

    #[test]
    fn test_create_all_prs_respects_group_boundaries() -> Result<()> {
        // Setup: Three repos in two groups
        let db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("repo1"))?
            .with_repo(RepoBuilder::new("repo2"))?
            .with_repo(RepoBuilder::new("repo3"))?
            .with_group("Group A", vec![1, 2])?
            .with_group("Group B", vec![3])?;

        // All have branches ready for PR
        for i in 1..=3 {
            db.db.insert_branch(&BranchBuilder::new(&format!("feature-{}", i))
                .ahead(i)
                .build(i))?;
        }

        let mock_gh = MockGitHubClient::new()
            .expect_create_pr_for("repo1")
            .expect_create_pr_for("repo2")
            .expect_no_create_pr_for("repo3"); // Different group

        // Execute for Group A only
        let result = create_all_prs_for_group(&db.db, &mock_gh, 1)?;

        // Verify
        assert_eq!(result.created.len(), 2);
        assert!(!result.created.contains(&"repo3".to_string()));

        mock_gh.verify();

        Ok(())
    }

    #[test]
    fn test_create_pr_validates_target_branch_exists() -> Result<()> {
        let db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("test-repo"))?;

        let repo_id = db.db.get_repository_id("testowner", "test-repo")?;

        // Feature branch exists, but main doesn't
        db.db.insert_branch(&BranchBuilder::new("feature")
            .ahead(3)
            .build(repo_id))?;

        let mock_gh = MockGitHubClient::new();

        // Execute - should fail validation
        let result = create_pull_request(
            &db.db,
            &mock_gh,
            "testowner",
            "test-repo",
            "feature",
            "main", // Doesn't exist!
        );

        // Verify
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Target branch"));

        Ok(())
    }
}
```

### Scenario 2: Status Calculation

**Test priority logic with all combinations:**

```rust
#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn test_status_priority_uncommitted_beats_unpushed() -> Result<()> {
        let status = LocalRepoStatus {
            uncommitted_files: 5,
            unpushed_commits: 3,
            behind_commits: 0,
        };

        let priority = calculate_status_priority(&status, &[], 0);

        assert_eq!(priority, StatusPriority::LocalChanges);
        Ok(())
    }

    #[test]
    fn test_status_priority_checks_github_branches_for_sync() -> Result<()> {
        // Clean local working directory
        let status = LocalRepoStatus {
            uncommitted_files: 0,
            unpushed_commits: 0,
            behind_commits: 0,
        };

        // But GitHub branch is behind
        let branches = vec![
            Branch {
                name: "feature".to_string(),
                ahead_by: 0,
                behind_by: 34, // Behind!
                // ... other fields
            }
        ];

        let priority = calculate_status_priority(&status, &branches, 0);

        // Should be NeedsSync because of GitHub branch status
        assert_eq!(priority, StatusPriority::NeedsSync);
        Ok(())
    }

    #[test]
    fn test_status_complete_only_when_all_clean() -> Result<()> {
        let status = LocalRepoStatus {
            uncommitted_files: 0,
            unpushed_commits: 0,
            behind_commits: 0,
        };

        let branches = vec![]; // No unmerged branches
        let unmerged_count = 0;

        let priority = calculate_status_priority(&status, &branches, unmerged_count);

        assert_eq!(priority, StatusPriority::Complete);
        Ok(())
    }
}
```

### Scenario 3: API Endpoint Testing

**Test HTTP endpoints with mocked dependencies:**

```rust
#[cfg(test)]
mod api_tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn test_create_pr_endpoint_validates_input() -> Result<()> {
        let db = TestDatabase::new()?;
        let gh = Arc::new(Mutex::new(MockGitHubClient::new()));
        let app = create_app(db.db, gh);
        let client = TestClient::new(app);

        // Send invalid request (missing fields)
        let response = client
            .post("/api/repos/create-pr")
            .json(&serde_json::json!({
                "owner": "test",
                // Missing repo, head, base
            }))
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_all_prs_endpoint_returns_summary() -> Result<()> {
        let db = TestDatabase::new()?
            .with_repo(RepoBuilder::new("repo1"))?
            .with_repo(RepoBuilder::new("repo2"))?;

        // Setup branches
        db.db.insert_branch(&BranchBuilder::new("feature-1")
            .ahead(3)
            .build(1))?;
        db.db.insert_branch(&BranchBuilder::new("feature-2")
            .ahead(5)
            .build(2))?;

        let gh = Arc::new(Mutex::new(MockGitHubClient::new()));
        let app = create_app(db.db, gh.clone());
        let client = TestClient::new(app);

        // Execute
        let response = client
            .post("/api/repos/create-all-prs")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let summary: PrCreationSummary = response.json().await;
        assert_eq!(summary.created.len(), 2);
        assert_eq!(summary.skipped.len(), 0);
        assert_eq!(summary.failed.len(), 0);

        Ok(())
    }
}
```

## Test Organization

### Directory Structure

```
overall-cli/
├── src/
│   ├── test_support/        # Shared test infrastructure
│   │   ├── mod.rs
│   │   ├── fixtures.rs      # TestDatabase, builders
│   │   └── mock_github.rs   # MockGitHubClient
│   ├── github/
│   │   ├── mod.rs
│   │   ├── client_trait.rs  # GitHubClient trait
│   │   ├── real_client.rs   # RealGitHubClient implementation
│   │   └── commands.rs      # gh CLI wrappers
│   ├── storage/
│   │   └── mod.rs           # Database with #[cfg(test)] tests
│   ├── server/
│   │   └── mod.rs           # API handlers with #[cfg(test)] tests
│   └── lib.rs
└── tests/                   # Integration tests
    ├── pr_creation.rs       # Cross-module PR creation scenarios
    ├── status_calculation.rs
    └── api_integration.rs
```

### Unit vs Integration Tests

**Unit tests** (`#[cfg(test)] mod tests` in same file):
- Test single function/method in isolation
- Use mocks for all dependencies
- Fast, focused, numerous

**Integration tests** (`tests/` directory):
- Test multiple modules working together
- Test end-to-end workflows
- Use shared test fixtures
- Fewer, but more comprehensive

## Running Tests

```bash
# Run all tests (REQUIRED before commit)
cargo test --all

# Run specific test
cargo test test_create_pr_skips_repos_with_open_prs

# Run all tests in a module
cargo test storage::tests

# Run integration tests only
cargo test --test pr_creation

# Run with output
cargo test -- --nocapture

# Run specific package
cargo test -p overall-cli
```

## Test Coverage Checklist

For EVERY feature, ensure you have tests for:

- [ ] Happy path - feature works as expected
- [ ] Empty inputs - no repos, no branches, no PRs
- [ ] Single item - one repo, one branch, one PR
- [ ] Multiple items - multiple repos/branches/PRs
- [ ] Edge cases - exactly at boundary conditions
- [ ] Invalid inputs - null, empty, wrong types
- [ ] Missing dependencies - target branch doesn't exist
- [ ] Conflicting states - repo in multiple groups
- [ ] Error recovery - what happens on failure
- [ ] State verification - database reflects changes correctly

## Migration Plan

### Phase 1: Infrastructure (CURRENT)
- [x] Create `GitHubClient` trait
- [x] Create `RealGitHubClient` implementation
- [x] Create `MockGitHubClient` for tests
- [x] Create `TestDatabase` fixture
- [x] Create `RepoBuilder` and `BranchBuilder`

### Phase 2: Refactor to Use Traits
- [ ] Update all functions to accept `&dyn GitHubClient` parameter
- [ ] Update CLI commands to create `RealGitHubClient` and pass it in
- [ ] Update server handlers to accept injected client
- [ ] Verify existing integration tests still pass

### Phase 3: Write Missing Tests
- [ ] PR creation scenarios (all combinations)
- [ ] Status calculation edge cases
- [ ] API endpoint validation
- [ ] Group management
- [ ] Error handling paths

### Phase 4: Remove Integration Dependencies
- [ ] Mark old integration tests as `#[ignore]`
- [ ] Add comment explaining they're for manual verification only
- [ ] Ensure CI/CD doesn't require `gh` CLI

## Common Pitfalls

### ❌ Don't: Test implementation details
```rust
// Bad - testing how it's done
assert!(result.internal_cache.contains("foo"));
```

### ✅ Do: Test behavior and contracts
```rust
// Good - testing what it does
assert_eq!(result.status, "success");
assert_eq!(result.created_prs.len(), 2);
```

### ❌ Don't: Use real external dependencies
```rust
// Bad - calls real GitHub API
let repos = github::list_repos("octocat", None)?;
```

### ✅ Do: Use mocked dependencies
```rust
// Good - uses injected mock
let mock_gh = MockGitHubClient::new()
    .with_repo("octocat", test_repo);
let repos = mock_gh.list_repos("octocat", None)?;
```

### ❌ Don't: Share mutable state between tests
```rust
// Bad - tests can interfere with each other
static mut SHARED_DB: Option<Database> = None;
```

### ✅ Do: Create fresh state for each test
```rust
// Good - each test gets its own database
let db = TestDatabase::new()?;
```

## Conclusion

**Every bug is a missing test. Every edge case is a missing test. Every "what if" is a missing test.**

Write the tests. Make them comprehensive. Make them maintainable. Make them fast.

The code will thank you. Your future self will thank you. Your users will thank you.
