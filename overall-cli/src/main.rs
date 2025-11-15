// Copyright (c) 2025 Michael A Wright
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};
use overall_cli::github;

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
    /// Start web UI server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan { owner, limit }) => {
            println!("Scanning repositories for: {} (limit: {})", owner, limit);

            match github::list_repos(&owner, limit) {
                Ok(repos) => {
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
                        if let Some(desc) = &repo.description {
                            println!("   Description: {}", desc);
                        }
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Error scanning repositories: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::List) => {
            println!("Listing repositories...");
            // TODO: Load from database (Phase 2)
        }
        Some(Commands::Serve { port }) => {
            println!("Starting web server on port {}...", port);
            // TODO: Implement web server (Phase 4)
        }
        None => {
            println!("Use --help for usage information");
        }
    }
}
