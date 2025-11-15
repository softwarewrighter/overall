use clap::{Parser, Subcommand};

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
        Some(Commands::Scan { owner }) => {
            println!("Scanning repositories for: {}", owner);
            // TODO: Implement scanning
        }
        Some(Commands::List) => {
            println!("Listing repositories...");
            // TODO: Implement listing
        }
        Some(Commands::Serve { port }) => {
            println!("Starting web server on port {}...", port);
            // TODO: Implement web server
        }
        None => {
            println!("Use --help for usage information");
        }
    }
}
