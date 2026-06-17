//! Core library for the `newc` C project manager.
//!
//! Provides all project operations — scaffolding, module management, header sync,
//! static analysis, build history, git integration, and the function library —
//! as pure Rust functions consumed by both the CLI and the GUI.

pub mod analysis;
pub mod build;
pub mod build_history;
pub mod config;
pub mod cref;
pub mod diag;
pub mod error;
pub mod export;
pub mod function_lib;
pub mod git;
pub mod grep;
pub mod header;
pub mod lint;
pub mod main_builder;
pub mod meta;
pub mod module;
pub mod notes;
pub mod project;
pub mod project_template;
pub mod refactor;
pub mod report;
pub mod scaffold;
pub mod stats;
pub mod sync;
pub mod templates;
pub mod user_template;
