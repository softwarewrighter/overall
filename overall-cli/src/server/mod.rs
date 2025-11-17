// Copyright (c) 2025 Michael A Wright
// SPDX-License-Identifier: MIT

use crate::storage::Database;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::{Arc, Mutex}};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[derive(Clone)]
pub struct AppState {
    db: Arc<Mutex<Database>>,
    static_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveRepoRequest {
    repo_id: String,
    target_group_id: Option<i64>, // None means move to ungrouped
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateGroupRequest {
    group_name: String,
    repo_ids: Vec<String>,
    target_group_id: Option<i64>, // If None, create new group
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePRRequest {
    repo_id: String,
    branch_name: String,
    title: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatePRResponse {
    success: bool,
    pr_url: Option<String>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAllPRsRequest {
    repo_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PRResult {
    branch_name: String,
    success: bool,
    pr_url: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateAllPRsResponse {
    success: bool,
    results: Vec<PRResult>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddLocalRepoRootRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveLocalRepoRootRequest {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScanLocalReposRequest {
    // Empty - scans all enabled roots
}

pub async fn serve(port: u16, db_path: PathBuf, static_dir: PathBuf, _debug: bool) -> anyhow::Result<()> {
    let db = Database::open_or_create(&db_path)?;
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        static_dir: static_dir.clone(),
    };

    let app = Router::new()
        // API routes
        .route("/api/groups", get(list_groups))
        .route("/api/groups/add-repos", post(add_repos_to_group))
        .route("/api/groups/delete/:id", post(delete_group))
        .route("/api/repos/move", post(move_repo))
        .route("/api/repos/export", post(export_repos))
        .route("/api/pr/create", post(create_pr))
        .route("/api/pr/create-all", post(create_all_prs))
        // Local repos routes
        .route("/api/local-repos/roots", get(list_local_repo_roots))
        .route("/api/local-repos/roots", post(add_local_repo_root))
        .route("/api/local-repos/roots/:id", axum::routing::delete(remove_local_repo_root))
        .route("/api/local-repos/scan", post(scan_local_repos))
        .route("/api/local-repos/status", get(get_local_repos_status))
        // Static files
        .nest_service("/", ServeDir::new(static_dir))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_groups(State(state): State<AppState>) -> Response {
    let db = state.db.lock().unwrap();
    match db.get_all_groups() {
        Ok(groups) => Json(groups).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to list groups: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Helper function to regenerate repos.json from current database state
fn regenerate_repos_json(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;

    let db = state.db.lock().unwrap();

    // Build JSON structure with groups
    let mut export_data = json!({
        "groups": [],
        "ungrouped": []
    });

    // Export all groups
    let groups = db.get_all_groups()?;
    for group in &groups {
        let repos = db.get_repos_in_group(group.id).unwrap_or_default();
        let mut group_repos = Vec::new();

        for repo in repos {
            let branches = db.get_branches_for_repo(&repo.id).unwrap_or_default();
            let prs = db.get_pull_requests_for_repo(&repo.id).unwrap_or_default();

            let unmerged_count = branches
                .iter()
                .filter(|b| b.ahead_by > 0 && b.behind_by == 0)
                .count();
            let open_pr_count = prs
                .iter()
                .filter(|pr| matches!(pr.state, crate::models::PRState::Open))
                .count();

            group_repos.push(json!({
                "id": repo.id,
                "owner": repo.owner,
                "name": repo.name,
                "language": repo.language.unwrap_or_else(|| "Unknown".to_string()),
                "lastPush": repo.pushed_at.to_rfc3339(),
                "branches": branches.iter().map(|b| {
                    let commits = db.get_commits_for_branch(b.id).unwrap_or_default();
                    json!({
                        "name": b.name,
                        "sha": b.sha,
                        "aheadBy": b.ahead_by,
                        "behindBy": b.behind_by,
                        "status": b.status.to_string(),
                        "lastCommitDate": b.last_commit_date.to_rfc3339(),
                        "commits": commits.iter().map(|c| json!({
                            "sha": c.sha,
                            "message": c.message,
                            "authorName": c.author_name,
                            "authorEmail": c.author_email,
                            "authoredDate": c.authored_date.to_rfc3339(),
                            "committerName": c.committer_name,
                            "committerEmail": c.committer_email,
                            "committedDate": c.committed_date.to_rfc3339(),
                        })).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "pullRequests": prs.iter().map(|pr| json!({
                    "number": pr.number,
                    "title": pr.title,
                    "state": pr.state.to_string(),
                    "createdAt": pr.created_at.to_rfc3339(),
                    "updatedAt": pr.updated_at.to_rfc3339(),
                })).collect::<Vec<_>>(),
                "unmergedCount": unmerged_count,
                "prCount": open_pr_count,
            }));
        }

        export_data["groups"].as_array_mut().unwrap().push(json!({
            "id": group.id,
            "name": group.name,
            "repos": group_repos
        }));
    }

    // Export ungrouped repositories
    let ungrouped = db.get_ungrouped_repositories().unwrap_or_default();
    for repo in ungrouped {
        let branches = db.get_branches_for_repo(&repo.id).unwrap_or_default();
        let prs = db.get_pull_requests_for_repo(&repo.id).unwrap_or_default();

        let unmerged_count = branches
            .iter()
            .filter(|b| b.ahead_by > 0 && b.behind_by == 0)
            .count();
        let open_pr_count = prs
            .iter()
            .filter(|pr| matches!(pr.state, crate::models::PRState::Open))
            .count();

        export_data["ungrouped"].as_array_mut().unwrap().push(json!({
            "id": repo.id,
            "owner": repo.owner,
            "name": repo.name,
            "language": repo.language.unwrap_or_else(|| "Unknown".to_string()),
            "lastPush": repo.pushed_at.to_rfc3339(),
            "branches": branches.iter().map(|b| {
                let commits = db.get_commits_for_branch(b.id).unwrap_or_default();
                json!({
                    "name": b.name,
                    "sha": b.sha,
                    "aheadBy": b.ahead_by,
                    "behindBy": b.behind_by,
                    "status": b.status.to_string(),
                    "lastCommitDate": b.last_commit_date.to_rfc3339(),
                    "commits": commits.iter().map(|c| json!({
                        "sha": c.sha,
                        "message": c.message,
                        "authorName": c.author_name,
                        "authorEmail": c.author_email,
                        "authoredDate": c.authored_date.to_rfc3339(),
                        "committerName": c.committer_name,
                        "committerEmail": c.committer_email,
                        "committedDate": c.committed_date.to_rfc3339(),
                    })).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
            "pullRequests": prs.iter().map(|pr| json!({
                "number": pr.number,
                "title": pr.title,
                "state": pr.state.to_string(),
                "createdAt": pr.created_at.to_rfc3339(),
                "updatedAt": pr.updated_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
            "unmergedCount": unmerged_count,
            "prCount": open_pr_count,
        }));
    }

    // Write to static/repos.json
    let output_path = state.static_dir.join("repos.json");
    let json_str = serde_json::to_string_pretty(&export_data)?;
    std::fs::write(&output_path, json_str)?;

    Ok(())
}

async fn move_repo(
    State(state): State<AppState>,
    Json(req): Json<MoveRepoRequest>,
) -> Response {
    let result = {
        let db = state.db.lock().unwrap();
        if let Some(target_group_id) = req.target_group_id {
            db.move_repo_to_group(&req.repo_id, target_group_id)
        } else {
            db.remove_repo_from_all_groups(&req.repo_id)
        }
    };

    match result {
        Ok(()) => {
            // Regenerate repos.json after successful move
            if let Err(e) = regenerate_repos_json(&state) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Repository moved but failed to update repos.json: {}", e),
                    }),
                )
                    .into_response();
            }

            Json(ApiResponse {
                success: true,
                message: "Repository moved successfully".to_string(),
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to move repository: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn add_repos_to_group(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> Response {
    let group_id = if let Some(existing_group_id) = req.target_group_id {
        // Use existing group
        existing_group_id
    } else {
        // Create new group
        let db = state.db.lock().unwrap();
        let max_order = db.get_all_groups()
            .unwrap_or_default()
            .iter()
            .map(|g| g.display_order)
            .max()
            .unwrap_or(-1);

        match db.create_group(&req.group_name, max_order + 1) {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Failed to create group: {}", e),
                    }),
                )
                    .into_response();
            }
        }
    };

    // Add repos to the group
    {
        let db = state.db.lock().unwrap();
        for repo_id in &req.repo_ids {
            if let Err(e) = db.add_repo_to_group(repo_id, group_id) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Failed to add repository {}: {}", repo_id, e),
                    }),
                )
                    .into_response();
            }
        }
    }

    // Regenerate repos.json
    if let Err(e) = regenerate_repos_json(&state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Repositories added but failed to update repos.json: {}", e),
            }),
        )
            .into_response();
    }

    Json(ApiResponse {
        success: true,
        message: format!(
            "Successfully added {} repositories to group",
            req.repo_ids.len()
        ),
    })
    .into_response()
}

async fn delete_group(
    State(state): State<AppState>,
    Path(group_id): Path<i64>,
) -> Response {
    // Delete the group - repos will automatically become ungrouped due to CASCADE
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.delete_group(group_id) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to delete group: {}", e),
                }),
            )
                .into_response();
        }
    }

    // Regenerate repos.json
    if let Err(e) = regenerate_repos_json(&state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Group deleted but failed to update repos.json: {}", e),
            }),
        )
            .into_response();
    }

    Json(ApiResponse {
        success: true,
        message: "Successfully deleted group".to_string(),
    })
    .into_response()
}

async fn export_repos(State(state): State<AppState>) -> Response {
    use serde_json::json;

    let db = state.db.lock().unwrap();

    // Build JSON structure with groups
    let mut export_data = json!({
        "groups": [],
        "ungrouped": []
    });

    // Export all groups
    let groups = match db.get_all_groups() {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to get groups: {}", e),
                }),
            )
                .into_response()
        }
    };

    for group in &groups {
        let repos = db.get_repos_in_group(group.id).unwrap_or_default();
        let mut group_repos = Vec::new();

        for repo in repos {
            let branches = db.get_branches_for_repo(&repo.id).unwrap_or_default();
            let prs = db
                .get_pull_requests_for_repo(&repo.id)
                .unwrap_or_default();

            let unmerged_count = branches
                .iter()
                .filter(|b| b.ahead_by > 0 && b.behind_by == 0)
                .count();
            let open_pr_count = prs
                .iter()
                .filter(|pr| matches!(pr.state, crate::models::PRState::Open))
                .count();

            group_repos.push(json!({
                "id": repo.id,
                "owner": repo.owner,
                "name": repo.name,
                "language": repo.language.unwrap_or_else(|| "Unknown".to_string()),
                "lastPush": repo.pushed_at.to_rfc3339(),
                "branches": branches.iter().map(|b| {
                    let commits = db.get_commits_for_branch(b.id).unwrap_or_default();
                    json!({
                        "name": b.name,
                        "sha": b.sha,
                        "aheadBy": b.ahead_by,
                        "behindBy": b.behind_by,
                        "status": b.status.to_string(),
                        "lastCommitDate": b.last_commit_date.to_rfc3339(),
                        "commits": commits.iter().map(|c| json!({
                            "sha": c.sha,
                            "message": c.message,
                            "authorName": c.author_name,
                            "authorEmail": c.author_email,
                            "authoredDate": c.authored_date.to_rfc3339(),
                            "committerName": c.committer_name,
                            "committerEmail": c.committer_email,
                            "committedDate": c.committed_date.to_rfc3339(),
                        })).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "pullRequests": prs.iter().map(|pr| json!({
                    "number": pr.number,
                    "title": pr.title,
                    "state": pr.state.to_string(),
                    "createdAt": pr.created_at.to_rfc3339(),
                    "updatedAt": pr.updated_at.to_rfc3339(),
                })).collect::<Vec<_>>(),
                "unmergedCount": unmerged_count,
                "prCount": open_pr_count,
            }));
        }

        export_data["groups"].as_array_mut().unwrap().push(json!({
            "id": group.id,
            "name": group.name,
            "repos": group_repos
        }));
    }

    // Export ungrouped repositories
    let ungrouped = db.get_ungrouped_repositories().unwrap_or_default();
    for repo in ungrouped {
        let branches = db.get_branches_for_repo(&repo.id).unwrap_or_default();
        let prs = db
            .get_pull_requests_for_repo(&repo.id)
            .unwrap_or_default();

        let unmerged_count = branches
            .iter()
            .filter(|b| b.ahead_by > 0 && b.behind_by == 0)
            .count();
        let open_pr_count = prs
            .iter()
            .filter(|pr| matches!(pr.state, crate::models::PRState::Open))
            .count();

        export_data["ungrouped"].as_array_mut().unwrap().push(json!({
            "id": repo.id,
            "owner": repo.owner,
            "name": repo.name,
            "language": repo.language.unwrap_or_else(|| "Unknown".to_string()),
            "lastPush": repo.pushed_at.to_rfc3339(),
            "branches": branches.iter().map(|b| {
                let commits = db.get_commits_for_branch(b.id).unwrap_or_default();
                json!({
                    "name": b.name,
                    "sha": b.sha,
                    "aheadBy": b.ahead_by,
                    "behindBy": b.behind_by,
                    "status": b.status.to_string(),
                    "lastCommitDate": b.last_commit_date.to_rfc3339(),
                    "commits": commits.iter().map(|c| json!({
                        "sha": c.sha,
                        "message": c.message,
                        "authorName": c.author_name,
                        "authorEmail": c.author_email,
                        "authoredDate": c.authored_date.to_rfc3339(),
                        "committerName": c.committer_name,
                        "committerEmail": c.committer_email,
                        "committedDate": c.committed_date.to_rfc3339(),
                    })).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
            "pullRequests": prs.iter().map(|pr| json!({
                "number": pr.number,
                "title": pr.title,
                "state": pr.state.to_string(),
                "createdAt": pr.created_at.to_rfc3339(),
                "updatedAt": pr.updated_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
            "unmergedCount": unmerged_count,
            "prCount": open_pr_count,
        }));
    }

    // Write to static/repos.json
    let output_path = state.static_dir.join("repos.json");
    let json_str = serde_json::to_string_pretty(&export_data).unwrap();
    if let Err(e) = std::fs::write(&output_path, json_str) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to write repos.json: {}", e),
            }),
        )
            .into_response();
    }

    Json(ApiResponse {
        success: true,
        message: "Export completed successfully".to_string(),
    })
    .into_response()
}

async fn create_pr(
    State(_state): State<AppState>,
    Json(req): Json<CreatePRRequest>,
) -> Response {
    // Call github::create_pull_request
    let title_ref = req.title.as_deref();
    let body_ref = req.body.as_deref();

    match crate::github::create_pull_request(&req.repo_id, &req.branch_name, title_ref, body_ref) {
        Ok(pr_url) => Json(CreatePRResponse {
            success: true,
            pr_url: Some(pr_url),
            message: "Pull request created successfully".to_string(),
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreatePRResponse {
                success: false,
                pr_url: None,
                message: format!("Failed to create pull request: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn create_all_prs(
    State(state): State<AppState>,
    Json(req): Json<CreateAllPRsRequest>,
) -> Response {
    let db = state.db.lock().unwrap();

    // Get all branches for this repo
    let branches = match db.get_branches_for_repo(&req.repo_id) {
        Ok(branches) => branches,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CreateAllPRsResponse {
                    success: false,
                    results: vec![],
                    message: format!("Failed to get branches: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Release the lock before making external calls
    drop(db);

    // Filter branches with unmerged work (ahead > 0)
    let branches_to_pr: Vec<_> = branches
        .into_iter()
        .filter(|b| b.ahead_by > 0)
        .collect();

    if branches_to_pr.is_empty() {
        return Json(CreateAllPRsResponse {
            success: true,
            results: vec![],
            message: "No branches with unmerged work found".to_string(),
        })
        .into_response();
    }

    // Create PRs for each branch
    let mut results = Vec::new();
    for branch in branches_to_pr {
        match crate::github::create_pull_request(&req.repo_id, &branch.name, None, None) {
            Ok(pr_url) => {
                results.push(PRResult {
                    branch_name: branch.name,
                    success: true,
                    pr_url: Some(pr_url),
                    error: None,
                });
            }
            Err(e) => {
                results.push(PRResult {
                    branch_name: branch.name,
                    success: false,
                    pr_url: None,
                    error: Some(format!("{}", e)),
                });
            }
        }
    }

    let success_count = results.iter().filter(|r| r.success).count();
    let total_count = results.len();

    Json(CreateAllPRsResponse {
        success: true,
        results,
        message: format!("Created {} of {} PRs successfully", success_count, total_count),
    })
    .into_response()
}

// Local repository management handlers

async fn list_local_repo_roots(State(state): State<AppState>) -> Response {
    let db = state.db.lock().unwrap();
    match db.get_all_local_repo_roots() {
        Ok(roots) => Json(roots).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to list local repo roots: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn add_local_repo_root(
    State(state): State<AppState>,
    Json(req): Json<AddLocalRepoRootRequest>,
) -> Response {
    // Expand tilde in path
    let expanded_path = if req.path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            req.path.replacen("~", &home, 1)
        } else {
            req.path
        }
    } else {
        req.path
    };

    let db = state.db.lock().unwrap();
    match db.add_local_repo_root(&expanded_path) {
        Ok(_id) => Json(ApiResponse {
            success: true,
            message: "Local repository root added successfully".to_string(),
        })
        .into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            // Check if it's a UNIQUE constraint violation
            if error_msg.contains("UNIQUE constraint failed") {
                (
                    StatusCode::CONFLICT,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Path '{}' is already configured", expanded_path),
                    }),
                )
                    .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Failed to add local repo root: {}", e),
                    }),
                )
                    .into_response()
            }
        }
    }
}

async fn remove_local_repo_root(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Response {
    let db = state.db.lock().unwrap();
    match db.remove_local_repo_root(id) {
        Ok(()) => Json(ApiResponse {
            success: true,
            message: "Local repository root removed successfully".to_string(),
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to remove local repo root: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn scan_local_repos(
    State(state): State<AppState>,
    Json(_req): Json<ScanLocalReposRequest>,
) -> Response {
    use std::path::Path;

    let db = state.db.lock().unwrap();

    // Get all enabled repo roots
    let roots = match db.get_all_local_repo_roots() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to get repo roots: {}", e),
                }),
            )
                .into_response()
        }
    };

    drop(db); // Release lock before doing git operations

    let mut scan_results = Vec::new();
    let mut total_repos = 0;

    for root in roots.iter().filter(|r| r.enabled) {
        let root_path = Path::new(&root.path);

        // Scan for git repos
        match crate::local_git::scan_for_git_repos(root_path) {
            Ok(repo_paths) => {
                for repo_path in repo_paths {
                    total_repos += 1;

                    // Get repo status
                    match crate::local_git::get_repo_status(&repo_path) {
                        Ok(status) => {
                            // Save to database
                            let db = state.db.lock().unwrap();
                            if let Err(e) = db.save_local_repo_status(&status) {
                                scan_results.push(format!(
                                    "Error saving status for {}: {}",
                                    repo_path.display(),
                                    e
                                ));
                            } else {
                                scan_results.push(format!("Scanned: {}", status.repo_id));
                            }
                        }
                        Err(e) => {
                            scan_results.push(format!(
                                "Error scanning {}: {}",
                                repo_path.display(),
                                e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                scan_results.push(format!("Error scanning {}: {}", root.path, e));
            }
        }
    }

    Json(ApiResponse {
        success: true,
        message: format!(
            "Scanned {} repositories. Results: {}",
            total_repos,
            scan_results.join("; ")
        ),
    })
    .into_response()
}

async fn get_local_repos_status(State(state): State<AppState>) -> Response {
    let db = state.db.lock().unwrap();
    match db.get_all_local_repo_statuses() {
        Ok(statuses) => Json(statuses).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to get local repo statuses: {}", e),
            }),
        )
            .into_response(),
    }
}
