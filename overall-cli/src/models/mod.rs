use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: String,
    pub owner: String,
    pub name: String,
    pub language: Option<String>,
    pub description: Option<String>,
    pub pushed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_fork: bool,
    pub priority: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    pub id: i64,
    pub repo_id: String,
    pub name: String,
    pub sha: String,
    pub ahead_by: u32,
    pub behind_by: u32,
    pub status: BranchStatus,
    pub last_commit_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BranchStatus {
    ReadyForPR,
    InReview,
    ReadyToMerge,
    NeedsUpdate,
    HasConflicts,
}

impl fmt::Display for BranchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BranchStatus::ReadyForPR => write!(f, "ReadyForPR"),
            BranchStatus::InReview => write!(f, "InReview"),
            BranchStatus::ReadyToMerge => write!(f, "ReadyToMerge"),
            BranchStatus::NeedsUpdate => write!(f, "NeedsUpdate"),
            BranchStatus::HasConflicts => write!(f, "HasConflicts"),
        }
    }
}

impl FromStr for BranchStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ReadyForPR" => Ok(BranchStatus::ReadyForPR),
            "InReview" => Ok(BranchStatus::InReview),
            "ReadyToMerge" => Ok(BranchStatus::ReadyToMerge),
            "NeedsUpdate" => Ok(BranchStatus::NeedsUpdate),
            "HasConflicts" => Ok(BranchStatus::HasConflicts),
            _ => Err(format!("Unknown branch status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequest {
    pub id: i64,
    pub repo_id: String,
    pub branch_id: Option<i64>,
    pub number: u32,
    pub state: PRState,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PRState {
    Open,
    Closed,
    Merged,
}

impl fmt::Display for PRState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PRState::Open => write!(f, "Open"),
            PRState::Closed => write!(f, "Closed"),
            PRState::Merged => write!(f, "Merged"),
        }
    }
}

impl FromStr for PRState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(PRState::Open),
            "Closed" => Ok(PRState::Closed),
            "Merged" => Ok(PRState::Merged),
            _ => Err(format!("Unknown PR state: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commit {
    pub id: i64,
    pub branch_id: i64,
    pub sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_date: DateTime<Utc>,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIAnalysis {
    pub id: i64,
    pub repo_id: String,
    pub priority: u8,
    pub focus_branch: Option<String>,
    pub actions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRepoRoot {
    pub id: i64,
    pub path: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRepoStatus {
    pub id: i64,
    pub repo_id: String,
    pub local_path: String,
    pub current_branch: Option<String>,
    pub uncommitted_files: u32,
    pub unpushed_commits: u32,
    pub behind_commits: u32,
    pub is_dirty: bool,
    pub last_checked: DateTime<Utc>,
}
