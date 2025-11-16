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
pub mod models;
pub mod storage;

pub use error::{Error, Result};
