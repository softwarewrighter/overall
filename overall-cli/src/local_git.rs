//! Local git repository scanning and status detection

use crate::{models::LocalRepoStatus, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Scan a directory for git repositories
/// Returns a list of paths to git repositories found
pub fn scan_for_git_repos(root_path: &Path) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();

    if !root_path.exists() {
        return Err(crate::Error::GitCommand(format!(
            "Path does not exist: {}",
            root_path.display()
        )));
    }

    // Read directory entries
    let entries = std::fs::read_dir(root_path)
        .map_err(|e| crate::Error::GitCommand(format!("Failed to read directory: {}", e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            crate::Error::GitCommand(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Check if this directory is a git repository
        if path.is_dir() && path.join(".git").exists() {
            repos.push(path);
        }
    }

    Ok(repos)
}

/// Extract repo_id (owner/name) from a local git repository path
/// Expects paths like: ~/github/softwarewrighter/overall
/// Returns: softwarewrighter/overall
pub fn extract_repo_id(local_path: &Path) -> Option<String> {
    let components: Vec<_> = local_path.components().collect();
    if components.len() >= 2 {
        let name = components[components.len() - 1]
            .as_os_str()
            .to_str()?
            .to_string();
        let owner = components[components.len() - 2]
            .as_os_str()
            .to_str()?
            .to_string();
        Some(format!("{}/{}", owner, name))
    } else {
        None
    }
}

/// Get the current branch name for a repository
pub fn get_current_branch(repo_path: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::GitCommand(format!("Failed to get current branch: {}", e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let branch = String::from_utf8(output.stdout)
        .map_err(|e| crate::Error::GitCommand(format!("Invalid UTF-8 in branch name: {}", e)))?
        .trim()
        .to_string();

    Ok(Some(branch))
}

/// Count uncommitted files (modified, added, deleted)
pub fn count_uncommitted_files(repo_path: &Path) -> Result<u32> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::GitCommand(format!("Failed to get git status: {}", e)))?;

    if !output.status.success() {
        return Ok(0);
    }

    let status_lines = String::from_utf8(output.stdout)
        .map_err(|e| crate::Error::GitCommand(format!("Invalid UTF-8 in status: {}", e)))?;

    Ok(status_lines.lines().filter(|line| !line.is_empty()).count() as u32)
}

/// Get the number of commits ahead and behind the remote
/// Returns (ahead, behind)
pub fn get_ahead_behind(repo_path: &Path, branch: &str) -> Result<(u32, u32)> {
    // First, try to get the upstream branch
    let upstream_output = Command::new("git")
        .args([
            "rev-parse",
            "--abbrev-ref",
            &format!("{}@{{upstream}}", branch),
        ])
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::GitCommand(format!("Failed to get upstream: {}", e)))?;

    if !upstream_output.status.success() {
        // No upstream configured
        return Ok((0, 0));
    }

    let upstream = String::from_utf8(upstream_output.stdout)
        .map_err(|e| crate::Error::GitCommand(format!("Invalid UTF-8 in upstream: {}", e)))?
        .trim()
        .to_string();

    // Get ahead/behind counts
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{}...{}", branch, upstream),
        ])
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            crate::Error::GitCommand(format!("Failed to get ahead/behind counts: {}", e))
        })?;

    if !output.status.success() {
        return Ok((0, 0));
    }

    let counts = String::from_utf8(output.stdout)
        .map_err(|e| crate::Error::GitCommand(format!("Invalid UTF-8 in counts: {}", e)))?
        .trim()
        .to_string();

    let parts: Vec<&str> = counts.split_whitespace().collect();
    if parts.len() == 2 {
        let ahead = parts[0].parse::<u32>().unwrap_or(0);
        let behind = parts[1].parse::<u32>().unwrap_or(0);
        Ok((ahead, behind))
    } else {
        Ok((0, 0))
    }
}

/// Get the full status of a local git repository
pub fn get_repo_status(repo_path: &Path) -> Result<LocalRepoStatus> {
    let repo_id = extract_repo_id(repo_path)
        .ok_or_else(|| crate::Error::GitCommand("Failed to extract repo ID".to_string()))?;

    let current_branch = get_current_branch(repo_path)?;
    let uncommitted_files = count_uncommitted_files(repo_path)?;

    let (unpushed_commits, behind_commits) = if let Some(ref branch) = current_branch {
        get_ahead_behind(repo_path, branch)?
    } else {
        (0, 0)
    };

    let is_dirty = uncommitted_files > 0 || unpushed_commits > 0;

    Ok(LocalRepoStatus {
        id: 0, // Will be set by database
        repo_id,
        local_path: repo_path.to_string_lossy().to_string(),
        current_branch,
        uncommitted_files,
        unpushed_commits,
        behind_commits,
        is_dirty,
        last_checked: Utc::now(),
    })
}

/// Fetch remote updates for a repository
pub fn fetch_remote(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["fetch", "--all"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::GitCommand(format!("Failed to fetch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::GitCommand(format!(
            "Git fetch failed: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_repo_id() {
        let path = PathBuf::from("/Users/mike/github/softwarewrighter/overall");
        assert_eq!(
            extract_repo_id(&path),
            Some("softwarewrighter/overall".to_string())
        );

        let path = PathBuf::from("/Users/mike/github/sw-cli-tools/repo-tool");
        assert_eq!(
            extract_repo_id(&path),
            Some("sw-cli-tools/repo-tool".to_string())
        );
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path");
        let result = scan_for_git_repos(&path);
        assert!(result.is_err());
    }
}
