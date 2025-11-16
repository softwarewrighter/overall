// Copyright (c) 2025 Michael A Wright
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};
use overall_cli::{github, storage::Database};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "overall")]
#[command(about = "GitHub Repository Manager - Track and prioritize your repositories")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan repositories for a GitHub user or organization
    Scan {
        /// GitHub user or organization name
        owner: String,

        /// Maximum number of repositories to fetch
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// List all tracked repositories
    List,
    /// Export data to JSON for UI consumption
    Export {
        /// Output file path
        #[arg(short, long, default_value = "static/repos.json")]
        output: PathBuf,
    },
    /// Start web UI server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8459")]
        port: u16,
    },
}

fn get_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".overall").join("overall.db")
}

fn ensure_db_dir() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Ensure database directory exists
    if let Err(e) = ensure_db_dir() {
        eprintln!("Error creating database directory: {}", e);
        std::process::exit(1);
    }

    match cli.command {
        Some(Commands::Scan { owner, limit }) => {
            println!("Scanning repositories for: {} (limit: {})", owner, limit);

            // Open database
            let db_path = get_db_path();
            let db = match Database::open_or_create(&db_path) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            // Fetch repositories
            let repos = match github::list_repos(&owner, limit) {
                Ok(repos) => repos,
                Err(e) => {
                    eprintln!("Error fetching repositories: {}", e);
                    std::process::exit(1);
                }
            };

            println!("\nFound {} repositories\n", repos.len());

            // Process each repository
            for (i, repo) in repos.iter().enumerate() {
                println!(
                    "[{}/{}] Processing {}...",
                    i + 1,
                    repos.len(),
                    repo.id
                );

                // Save repository
                if let Err(e) = db.save_repository(repo) {
                    eprintln!("  Error saving repository: {}", e);
                    continue;
                }

                // Fetch and save branches
                print!("  Fetching branches...");
                match github::fetch_branches(&repo.id) {
                    Ok(branches) => {
                        println!(" found {}", branches.len());

                        // Clear old branches
                        if let Err(e) = db.clear_branches_for_repo(&repo.id) {
                            eprintln!("  Error clearing old branches: {}", e);
                        }

                        // Save branches and fetch commits for unmerged branches
                        for branch in &branches {
                            match db.save_branch(branch) {
                                Ok(branch_id) => {
                                    // Fetch commits for branches with unmerged changes
                                    if branch.ahead_by > 0 {
                                        print!("  Fetching commits for {}...", branch.name);
                                        match github::fetch_commits(&repo.id, &branch.name, branch_id) {
                                            Ok(commits) => {
                                                println!(" found {}", commits.len());

                                                // Clear old commits for this branch
                                                if let Err(e) = db.clear_commits_for_branch(branch_id) {
                                                    eprintln!("    Error clearing old commits: {}", e);
                                                }

                                                // Save commits
                                                for commit in &commits {
                                                    if let Err(e) = db.save_commit(commit) {
                                                        eprintln!("    Error saving commit {}: {}", &commit.sha[..7], e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("\n    Error fetching commits: {}", e);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("  Error saving branch {}: {}", branch.name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\n  Error fetching branches: {}", e);
                    }
                }

                // Fetch and save pull requests
                print!("  Fetching pull requests...");
                match github::fetch_pull_requests(&repo.id) {
                    Ok(prs) => {
                        println!(" found {}", prs.len());

                        // Clear old PRs
                        if let Err(e) = db.clear_pull_requests_for_repo(&repo.id) {
                            eprintln!("  Error clearing old PRs: {}", e);
                        }

                        // Save PRs
                        for pr in &prs {
                            if let Err(e) = db.save_pull_request(pr) {
                                eprintln!("  Error saving PR #{}: {}", pr.number, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\n  Error fetching pull requests: {}", e);
                    }
                }

                println!();
            }

            println!("✓ Scan complete! Data saved to {}", db_path.display());
        }
        Some(Commands::List) => {
            println!("Listing repositories...");

            let db_path = get_db_path();
            let db = match Database::open_or_create(&db_path) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            match db.get_all_repositories() {
                Ok(repos) => {
                    if repos.is_empty() {
                        println!("No repositories found. Run 'overall scan <owner>' to fetch repositories.");
                        return;
                    }

                    println!("\nFound {} repositories:\n", repos.len());
                    for (i, repo) in repos.iter().enumerate() {
                        println!("{}. {}", i + 1, repo.id);
                        println!(
                            "   Language: {}",
                            repo.language.as_ref().unwrap_or(&"Unknown".to_string())
                        );
                        println!(
                            "   Last push: {}",
                            repo.pushed_at.format("%Y-%m-%d %H:%M:%S")
                        );

                        // Get branch count
                        if let Ok(branches) = db.get_branches_for_repo(&repo.id) {
                            println!("   Branches: {}", branches.len());
                        }

                        // Get PR count
                        if let Ok(prs) = db.get_pull_requests_for_repo(&repo.id) {
                            let open_prs = prs.iter().filter(|pr| matches!(pr.state, overall_cli::models::PRState::Open)).count();
                            if open_prs > 0 {
                                println!("   Open PRs: {}", open_prs);
                            }
                        }

                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Error loading repositories: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Export { output }) => {
            println!("Exporting data to {}...", output.display());

            let db_path = get_db_path();
            let db = match Database::open_or_create(&db_path) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            // Build JSON structure with groups
            use serde_json::json;
            let mut export_data = json!({
                "groups": [],
                "ungrouped": []
            });

            // Export all groups
            let groups = db.get_all_groups().unwrap_or_default();
            for group in &groups {
                let repos = db.get_repos_in_group(group.id).unwrap_or_default();
                let mut group_repos = Vec::new();

                for repo in repos {
                    let branches = db.get_branches_for_repo(&repo.id).unwrap_or_default();
                    let prs = db.get_pull_requests_for_repo(&repo.id).unwrap_or_default();

                    let unmerged_count = branches.iter().filter(|b| b.ahead_by > 0 && b.behind_by == 0).count();
                    let open_pr_count = prs.iter().filter(|pr| matches!(pr.state, overall_cli::models::PRState::Open)).count();

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

                let unmerged_count = branches.iter().filter(|b| b.ahead_by > 0 && b.behind_by == 0).count();
                let open_pr_count = prs.iter().filter(|pr| matches!(pr.state, overall_cli::models::PRState::Open)).count();

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

            // Create output directory if needed
            if let Some(parent) = output.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Error creating output directory: {}", e);
                    std::process::exit(1);
                }
            }

            // Write JSON file
            let json_str = serde_json::to_string_pretty(&export_data).unwrap();
            if let Err(e) = std::fs::write(&output, json_str) {
                eprintln!("Error writing output file: {}", e);
                std::process::exit(1);
            }

            let total_groups = groups.len();
            let total_ungrouped = export_data["ungrouped"].as_array().unwrap().len();
            println!("✓ Exported {} groups and {} ungrouped repositories to {}",
                total_groups, total_ungrouped, output.display());
        }
        Some(Commands::Serve { port }) => {
            println!("Starting web server on port {}...", port);

            let db_path = get_db_path();
            let static_dir = std::path::PathBuf::from("static");

            // Check if static directory exists
            if !static_dir.exists() {
                eprintln!("Error: static directory not found");
                eprintln!("Run './target/release/overall export' first to generate static files");
                std::process::exit(1);
            }

            // Run the server using tokio runtime
            let runtime = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = runtime.block_on(overall_cli::server::serve(port, db_path, static_dir)) {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            println!("Use --help for usage information");
        }
    }
}
