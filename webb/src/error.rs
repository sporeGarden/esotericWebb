// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed errors for esoteric-webb.
//!
//! Replaces pervasive `Result<_, String>` with structured variants so callers
//! can inspect failure categories without parsing human-readable messages.

use std::path::PathBuf;

/// Errors produced by the webb runtime.
#[derive(Debug, thiserror::Error)]
pub enum WebbError {
    /// IO failure (file read/write, socket bind, process spawn).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// YAML deserialization failure.
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON serialization/deserialization failure.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    /// Content directory not found.
    #[error("content directory not found: {0}")]
    ContentNotFound(PathBuf),

    /// Content validation produced issues.
    #[error("{count} validation issue(s): {summary}")]
    Validation {
        /// Number of issues found.
        count: usize,
        /// Semicolon-joined issue descriptions.
        summary: String,
    },

    /// Narrative graph has no start node.
    #[error("no start node in narrative graph")]
    NoStartNode,

    /// Binary not found in expected locations.
    #[error("binary not found: {name}")]
    BinaryNotFound {
        /// Primal name that was searched for.
        name: String,
    },

    /// Signal handler registration failure.
    #[error("signal: {0}")]
    Signal(String),

    /// General operational error with context.
    #[error("{0}")]
    Other(String),
}

/// Result type for webb operations.
pub type Result<T> = std::result::Result<T, WebbError>;

impl From<String> for WebbError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}
