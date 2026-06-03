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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_variant() {
        let err = WebbError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone"));
        assert!(err.to_string().contains("io:"));
    }

    #[test]
    fn display_content_not_found() {
        let err = WebbError::ContentNotFound(PathBuf::from("/missing"));
        assert!(err.to_string().contains("/missing"));
    }

    #[test]
    fn display_validation() {
        let err = WebbError::Validation {
            count: 3,
            summary: "a; b; c".to_owned(),
        };
        let msg = err.to_string();
        assert!(msg.contains("3 validation"));
        assert!(msg.contains("a; b; c"));
    }

    #[test]
    fn display_no_start_node() {
        let err = WebbError::NoStartNode;
        assert!(err.to_string().contains("no start node"));
    }

    #[test]
    fn display_binary_not_found() {
        let err = WebbError::BinaryNotFound {
            name: "rhizocrypt".to_owned(),
        };
        assert!(err.to_string().contains("rhizocrypt"));
    }

    #[test]
    fn display_signal() {
        let err = WebbError::Signal("SIGTERM".to_owned());
        assert!(err.to_string().contains("SIGTERM"));
    }

    #[test]
    fn display_other() {
        let err = WebbError::Other("something failed".to_owned());
        assert_eq!(err.to_string(), "something failed");
    }

    #[test]
    fn from_string_produces_other() {
        let err: WebbError = "custom error".to_owned().into();
        assert!(matches!(err, WebbError::Other(s) if s == "custom error"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: WebbError = io_err.into();
        assert!(matches!(err, WebbError::Io(_)));
    }
}
