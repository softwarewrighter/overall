use crate::{
    models::{Branch, BranchStatus, Commit, PRState, PullRequest, Repository},
    Error, Result,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct GhRepository {
    name: String,
    owner: GhOwner,
    #[serde(rename = "pushedAt")]
    pushed_at: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "primaryLanguage")]
    primary_language: Option<GhLanguage>,
    description: Option<String>,
    #[serde(rename = "isFork")]
    is_fork: bool,
}

#[derive(Debug, Deserialize)]
struct GhOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhLanguage {
    name: String,
}

pub fn list_repos(owner: &str, limit: usize) -> Result<Vec<Repository>> {
    // Validate owner name
    validate_owner(owner)?;

    // Execute gh CLI command
    let output = Command::new("gh")
        .args([
            "repo",
            "list",
            owner,
            "--limit",
            &limit.to_string(),
            "--json",
            "name,owner,pushedAt,createdAt,updatedAt,primaryLanguage,description,isFork",
        ])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "gh CLI command failed: {}",
            stderr
        )));
    }

    // Parse JSON response
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let gh_repos: Vec<GhRepository> = serde_json::from_str(&stdout)?;

    // Convert to our Repository model
    let mut repos: Vec<Repository> = gh_repos
        .into_iter()
        .map(|gh_repo| {
            Ok(Repository {
                id: format!("{}/{}", gh_repo.owner.login, gh_repo.name),
                owner: gh_repo.owner.login,
                name: gh_repo.name,
                language: gh_repo.primary_language.map(|l| l.name),
                description: gh_repo.description,
                pushed_at: parse_github_timestamp(&gh_repo.pushed_at)?,
                created_at: parse_github_timestamp(&gh_repo.created_at)?,
                updated_at: parse_github_timestamp(&gh_repo.updated_at)?,
                is_fork: gh_repo.is_fork,
                priority: 0.0, // Will be calculated later
            })
        })
        .collect::<Result<Vec<Repository>>>()?;

    // Sort by pushed_at descending (most recent first)
    repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));

    Ok(repos)
}

fn validate_owner(owner: &str) -> Result<()> {
    if owner.is_empty() {
        return Err(Error::InvalidOwner("Owner cannot be empty".to_string()));
    }

    // GitHub usernames: alphanumeric, hyphens, max 39 chars
    if !owner.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(Error::InvalidOwner(format!(
            "Invalid owner name '{}': must be alphanumeric or hyphens",
            owner
        )));
    }

    if owner.len() > 39 {
        return Err(Error::InvalidOwner(format!(
            "Owner name too long: {} characters (max 39)",
            owner.len()
        )));
    }

    Ok(())
}

fn parse_github_timestamp(timestamp: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::GitHubCLI(format!("Failed to parse timestamp '{}': {}", timestamp, e)))
}

// Branch-related structures
#[derive(Debug, Deserialize)]
struct GhBranch {
    name: String,
    commit: GhBranchCommit,
}

#[derive(Debug, Deserialize)]
struct GhBranchCommit {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GhCommitDetails {
    #[allow(dead_code)]
    sha: String,
    commit: GhCommitInfo,
}

#[derive(Debug, Deserialize)]
struct GhCommitInfo {
    author: GhAuthor,
}

#[derive(Debug, Deserialize)]
struct GhAuthor {
    date: String,
}

#[derive(Debug, Deserialize)]
struct GhComparison {
    ahead_by: u32,
    behind_by: u32,
    #[allow(dead_code)]
    status: String,
}

pub fn fetch_branches(repo_id: &str) -> Result<Vec<Branch>> {
    // Parse repo_id (owner/name format)
    let parts: Vec<&str> = repo_id.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidOwner(format!(
            "Invalid repository ID: {}. Expected owner/name format",
            repo_id
        )));
    }

    // Fetch branches using gh API
    let output = Command::new("gh")
        .args(["api", &format!("repos/{}/branches", repo_id), "--paginate"])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "gh CLI command failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let gh_branches: Vec<GhBranch> = serde_json::from_str(&stdout)?;

    // Get default branch to compare against
    let default_branch = get_default_branch(repo_id)?;

    // Convert to our Branch model
    let branches: Vec<Branch> = gh_branches
        .into_iter()
        .enumerate()
        .map(|(idx, gh_branch)| {
            // Fetch commit details to get the date
            let commit_details = fetch_commit_details(repo_id, &gh_branch.commit.sha)?;
            let last_commit_date = commit_details;

            // Calculate ahead/behind if not the default branch
            let (ahead_by, behind_by) = if gh_branch.name == default_branch {
                (0, 0)
            } else {
                match compare_branches(repo_id, &default_branch, &gh_branch.name) {
                    Ok((ahead, behind)) => (ahead, behind),
                    Err(_) => (0, 0), // If comparison fails, assume no difference
                }
            };

            Ok(Branch {
                id: idx as i64,
                repo_id: repo_id.to_string(),
                name: gh_branch.name,
                sha: gh_branch.commit.sha,
                ahead_by,
                behind_by,
                status: BranchStatus::ReadyForPR, // Will be updated in Phase 1.3
                last_commit_date,
            })
        })
        .collect::<Result<Vec<Branch>>>()?;

    Ok(branches)
}

fn fetch_commit_details(repo_id: &str, sha: &str) -> Result<DateTime<Utc>> {
    let output = Command::new("gh")
        .args(["api", &format!("repos/{}/commits/{}", repo_id, sha)])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "Failed to fetch commit details: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let commit: GhCommitDetails = serde_json::from_str(&stdout)?;
    parse_github_timestamp(&commit.commit.author.date)
}

fn get_default_branch(repo_id: &str) -> Result<String> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}", repo_id),
            "--jq",
            ".default_branch",
        ])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "Failed to get default branch: {}",
            stderr
        )));
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?
        .trim()
        .to_string();

    Ok(branch)
}

fn compare_branches(repo_id: &str, base: &str, head: &str) -> Result<(u32, u32)> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/compare/{}...{}", repo_id, base, head),
        ])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "Failed to compare branches: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let comparison: GhComparison = serde_json::from_str(&stdout)?;

    // ahead_by means head is ahead of base (commits in head not in base)
    // behind_by means head is behind base (commits in base not in head)
    Ok((comparison.ahead_by, comparison.behind_by))
}

// Commit-related structures
#[derive(Debug, Deserialize)]
struct GhCommitFull {
    sha: String,
    commit: GhCommitFullInfo,
}

#[derive(Debug, Deserialize)]
struct GhCommitFullInfo {
    message: String,
    author: GhCommitAuthor,
    committer: GhCommitCommitter,
}

#[derive(Debug, Deserialize)]
struct GhCommitAuthor {
    name: String,
    email: String,
    date: String,
}

#[derive(Debug, Deserialize)]
struct GhCommitCommitter {
    name: String,
    email: String,
    date: String,
}

pub fn fetch_commits(repo_id: &str, branch_name: &str, branch_id: i64) -> Result<Vec<Commit>> {
    // Fetch commits for the branch
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/commits?sha={}", repo_id, branch_name),
            "--paginate",
        ])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "Failed to fetch commits: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let gh_commits: Vec<GhCommitFull> = serde_json::from_str(&stdout)?;

    // Convert to our Commit model (limit to first 50 commits to avoid overwhelming the UI)
    let commits: Vec<Commit> = gh_commits
        .into_iter()
        .take(50)
        .enumerate()
        .map(|(idx, gh_commit)| {
            Ok(Commit {
                id: idx as i64,
                branch_id,
                sha: gh_commit.sha,
                message: gh_commit.commit.message,
                author_name: gh_commit.commit.author.name,
                author_email: gh_commit.commit.author.email,
                authored_date: parse_github_timestamp(&gh_commit.commit.author.date)?,
                committer_name: gh_commit.commit.committer.name,
                committer_email: gh_commit.commit.committer.email,
                committed_date: parse_github_timestamp(&gh_commit.commit.committer.date)?,
            })
        })
        .collect::<Result<Vec<Commit>>>()?;

    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_owner_valid() {
        assert!(validate_owner("softwarewrighter").is_ok());
        assert!(validate_owner("rust-lang").is_ok());
        assert!(validate_owner("test123").is_ok());
    }

    #[test]
    fn test_validate_owner_invalid() {
        assert!(validate_owner("").is_err());
        assert!(validate_owner("invalid@name").is_err());
        assert!(validate_owner("invalid.name").is_err());
        assert!(validate_owner(&"a".repeat(40)).is_err());
    }

    #[test]
    fn test_parse_github_timestamp() {
        let result = parse_github_timestamp("2023-11-15T12:00:00Z");
        assert!(result.is_ok());
    }
}

// PR-related structures
#[derive(Debug, Deserialize)]
struct GhPR {
    number: u32,
    state: String,
    title: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "headRefName")]
    #[allow(dead_code)]
    head_ref_name: String,
}

pub fn fetch_pull_requests(repo_id: &str) -> Result<Vec<PullRequest>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "-R",
            repo_id,
            "--json",
            "number,state,title,createdAt,updatedAt,headRefName",
            "--limit",
            "100",
        ])
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHubCLI(format!(
            "gh CLI command failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    let gh_prs: Vec<GhPR> = serde_json::from_str(&stdout)?;

    let prs: Vec<PullRequest> = gh_prs
        .into_iter()
        .enumerate()
        .map(|(idx, gh_pr)| {
            let state = match gh_pr.state.as_str() {
                "OPEN" => PRState::Open,
                "CLOSED" => PRState::Closed,
                "MERGED" => PRState::Merged,
                _ => PRState::Closed,
            };

            Ok(PullRequest {
                id: idx as i64,
                repo_id: repo_id.to_string(),
                branch_id: None, // Will be matched later
                number: gh_pr.number,
                state,
                title: gh_pr.title,
                created_at: parse_github_timestamp(&gh_pr.created_at)?,
                updated_at: parse_github_timestamp(&gh_pr.updated_at)?,
            })
        })
        .collect::<Result<Vec<PullRequest>>>()?;

    Ok(prs)
}

pub fn classify_branch_status(
    branch: &Branch,
    prs: &[PullRequest],
    default_branch: &str,
) -> BranchStatus {
    // Is this the default branch?
    if branch.name == default_branch {
        return BranchStatus::ReadyForPR; // Default branch doesn't need PR
    }

    // Find PR for this branch
    let pr = prs.iter().find(|pr| {
        // Match by branch name (headRefName in PR)
        pr.branch_id.is_none() // Simple match for now
    });

    match pr {
        Some(pr) if pr.state == PRState::Open => {
            if branch.behind_by > 0 {
                BranchStatus::NeedsUpdate
            } else {
                BranchStatus::InReview
            }
        }
        Some(_pr) if _pr.state == PRState::Merged => BranchStatus::ReadyForPR, // Stale branch
        _ => BranchStatus::ReadyForPR, // No PR or no need for PR
    }
}

/// Create a pull request for a branch
/// Returns the PR URL on success
pub fn create_pull_request(
    repo_id: &str,
    branch_name: &str,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<String> {
    // Validate repo_id format
    if !repo_id.contains('/') {
        return Err(Error::GitHubCLI(format!(
            "Invalid repo_id format: {}. Expected owner/repo",
            repo_id
        )));
    }

    // Build gh pr create command
    let mut args = vec!["pr", "create", "--repo", repo_id, "--head", branch_name];

    // Use provided title or generate from branch name
    let default_title = branch_name.replace(['-', '_'], " ");
    let pr_title = title.unwrap_or(&default_title);
    args.push("--title");
    args.push(pr_title);

    // Add body if provided
    let default_body = "Created via Overall";
    let pr_body = body.unwrap_or(default_body);
    args.push("--body");
    args.push(pr_body);

    // Execute command
    let output = Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| Error::GitHubCLI(format!("Failed to execute gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if PR already exists - gh CLI includes the URL in the error message
        if stderr.contains("already exists") {
            // Extract the PR URL from the error message (usually on the last line)
            if let Some(url_line) = stderr.lines().last() {
                let url = url_line.trim();
                if url.starts_with("https://github.com") {
                    // Return the existing PR URL as success
                    return Ok(url.to_string());
                }
            }
        }

        return Err(Error::GitHubCLI(format!(
            "Failed to create PR for branch {}: {}",
            branch_name, stderr
        )));
    }

    // Parse stdout to get PR URL
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| Error::GitHubCLI(format!("Invalid UTF-8 in response: {}", e)))?;

    // gh pr create outputs the PR URL on stdout
    let pr_url = stdout.trim().to_string();

    Ok(pr_url)
}
