//! Test support infrastructure
//!
//! This module provides:
//! - Mock/spy implementations of external dependencies (GitHub, etc.)
//! - Test fixtures and builders for creating test data
//! - Helper functions for common test scenarios
//!
//! Only compiled when cfg(test) is active

#[cfg(test)]
pub mod fixtures;

#[cfg(test)]
pub mod mock_github;

#[cfg(test)]
pub use fixtures::{BranchBuilder, RepoBuilder, TestDatabase};

#[cfg(test)]
pub use mock_github::MockGitHubClient;
