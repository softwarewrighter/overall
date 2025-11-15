use crate::{models::Repository, Error, Result};
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
    let repos: Vec<Repository> = gh_repos
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
