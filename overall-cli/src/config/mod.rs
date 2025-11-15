//! Configuration management

use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub github: GitHubConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubConfig {
    pub owners: Vec<String>,
    pub repo_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: "1.0".to_string(),
            github: GitHubConfig {
                owners: vec!["softwarewrighter".to_string()],
                repo_limit: 50,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: Load from file in Phase 2
        Ok(Self::default())
    }
}
