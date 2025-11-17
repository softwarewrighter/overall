//! GitHub Repository Manager
//!
//! Copyright (c) 2025 Michael A Wright
//! SPDX-License-Identifier: MIT
//!
//! This crate provides the core functionality for managing GitHub repositories,
//! including scanning, analysis, storage, and AI-powered prioritization.

pub mod ai;
pub mod analysis;
pub mod config;
pub mod error;
pub mod github;
pub mod local_git;
pub mod models;
pub mod server;
pub mod storage;

#[cfg(test)]
pub mod test_support;

pub use error::{Error, Result};
