use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub CLI error: {0}")]
    GitHubCLI(String),

    #[error("GitHub API error: {status} - {message}")]
    GitHubAPI { status: u16, message: String },

    #[error("Git command error: {0}")]
    GitCommand(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("AI service unavailable: {0}")]
    AIUnavailable(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Invalid GitHub owner name: {0}")]
    InvalidOwner(String),
}

pub type Result<T> = std::result::Result<T, Error>;
