use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIAnalysis {
    pub id: i64,
    pub repo_id: String,
    pub priority: u8,
    pub focus_branch: Option<String>,
    pub actions: Vec<String>,
    pub created_at: DateTime<Utc>,
}
