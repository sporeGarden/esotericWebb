// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 envelope types and semantic IPC error classification.
//!
//! Mirrors the wire format used by all ecoPrimals primals.
//! Newline-delimited JSON over Unix domain sockets or TCP.
//!
//! Error classification aligned with primalSpring `ipc/error.rs` —
//! converged ecosystem pattern with `is_retriable()`, `is_recoverable()`,
//! and `classify_io_error()` for circuit breaker / retry decisions.

use serde::{Deserialize, Serialize};

/// Standard JSON-RPC error code for method not found.
pub const ERROR_METHOD_NOT_FOUND: i64 = -32601;
/// Standard JSON-RPC error code for parse error.
pub const ERROR_PARSE: i64 = -32700;
/// Standard JSON-RPC error code for invalid params.
pub const ERROR_INVALID_PARAMS: i64 = -32602;
/// Standard JSON-RPC error code for internal error.
pub const ERROR_INTERNAL: i64 = -32603;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version — always "2.0".
    pub jsonrpc: String,
    /// Method name (capability identifier).
    pub method: String,
    /// Method parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request identifier.
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version — always "2.0".
    pub jsonrpc: String,
    /// Successful result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Matching request identifier.
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i64,
    /// Human-readable message.
    pub message: String,
    /// Optional structured data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Semantic IPC error — classifies failures by *what happened* rather
/// than where in the code path the failure occurred.
///
/// Aligned with primalSpring's `IpcError` for ecosystem-wide consistency
/// in circuit breaker and retry decisions.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// No socket or endpoint found for the requested capability domain.
    #[error("primal not found for domain '{domain}'")]
    PrimalNotFound {
        /// Capability domain that was searched.
        domain: String,
    },
    /// Endpoint exists but the connection was actively refused.
    #[error("connection refused: {0}")]
    ConnectionRefused(String),
    /// Connection was established but dropped mid-communication.
    #[error("connection reset: {0}")]
    ConnectionReset(String),
    /// Operation exceeded the configured timeout.
    #[error("timeout after {ms}ms")]
    Timeout {
        /// Milliseconds elapsed before timeout.
        ms: u64,
    },
    /// Wire-level protocol violation (malformed JSON, empty response).
    #[error("protocol error: {detail}")]
    ProtocolError {
        /// Human-readable description of the protocol violation.
        detail: String,
    },
    /// Server explicitly reported the method does not exist.
    #[error("method not found: {method}")]
    MethodNotFound {
        /// The method name or server message.
        method: String,
    },
    /// Server returned a JSON-RPC error that is not `MethodNotFound`.
    #[error("remote error {code}: {message}")]
    ApplicationError {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message from the server.
        message: String,
    },
    /// Failed to serialize a request or deserialize a typed result.
    #[error("serialization: {detail}")]
    Serialization {
        /// Human-readable description of the serialization failure.
        detail: String,
    },
}

impl IpcError {
    /// Whether a retry is likely to succeed (transient failures only).
    ///
    /// Narrower than [`is_recoverable`](Self::is_recoverable) — only
    /// connection resets and timeouts, where the same request may succeed
    /// on a second attempt without any external change.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::ConnectionReset(_) | Self::Timeout { .. })
    }

    /// Whether recovery is possible without operator intervention.
    ///
    /// Broader than [`is_retriable`](Self::is_retriable) — includes
    /// transient failures AND server errors that may resolve if the primal
    /// stabilizes. Excludes `MethodNotFound` (permanent capability gap)
    /// and `Serialization` (client bug).
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionRefused(_)
                | Self::ConnectionReset(_)
                | Self::Timeout { .. }
                | Self::ApplicationError { .. }
        )
    }

    /// Whether the server reported the method does not exist.
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::MethodNotFound { .. })
    }

    /// Whether this is a connection-level failure.
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::PrimalNotFound { .. } | Self::ConnectionRefused(_) | Self::ConnectionReset(_)
        )
    }
}

/// Classify a raw `io::Error` into a semantic [`IpcError`] variant.
#[must_use]
pub fn classify_io_error(err: &std::io::Error) -> IpcError {
    match err.kind() {
        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound => {
            IpcError::ConnectionRefused(err.to_string())
        }
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            IpcError::Timeout { ms: 0 }
        }
        _ => IpcError::ConnectionReset(err.to_string()),
    }
}

impl From<JsonRpcError> for IpcError {
    fn from(err: JsonRpcError) -> Self {
        if err.code == ERROR_METHOD_NOT_FOUND {
            Self::MethodNotFound {
                method: err.message,
            }
        } else {
            Self::ApplicationError {
                code: err.code,
                message: err.message,
            }
        }
    }
}

impl JsonRpcRequest {
    /// Create a new request with the given method and params.
    #[must_use]
    pub fn new(method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            method: method.to_owned(),
            params,
            id: serde_json::Value::Number(serde_json::Number::from(1)),
        }
    }
}

impl JsonRpcResponse {
    /// Whether this response indicates success (no error field).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Extract the result or convert the error.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the response contains a JSON-RPC error.
    pub fn into_result(self) -> Result<serde_json::Value, IpcError> {
        if let Some(err) = self.error {
            return Err(IpcError::from(err));
        }
        Ok(self.result.unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let req = JsonRpcRequest::new("webb.health", None);
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "webb.health");
    }

    #[test]
    fn response_success() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: serde_json::json!(1),
        };
        assert!(resp.is_success());
        assert!(resp.into_result().is_ok());
    }

    #[test]
    fn response_error() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            error: Some(JsonRpcError {
                code: ERROR_METHOD_NOT_FOUND,
                message: "method not found".to_owned(),
                data: None,
            }),
            id: serde_json::json!(1),
        };
        assert!(!resp.is_success());
        let err = resp.into_result().unwrap_err();
        assert!(err.is_method_not_found());
    }

    #[test]
    fn from_jsonrpc_method_not_found() {
        let rpc_err = JsonRpcError {
            code: ERROR_METHOD_NOT_FOUND,
            message: "health.check".to_owned(),
            data: None,
        };
        let err = IpcError::from(rpc_err);
        assert!(err.is_method_not_found());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn from_jsonrpc_application_error() {
        let rpc_err = JsonRpcError {
            code: ERROR_INTERNAL,
            message: "internal".to_owned(),
            data: None,
        };
        let err = IpcError::from(rpc_err);
        assert!(!err.is_method_not_found());
        assert!(err.is_recoverable());
    }

    #[test]
    fn classify_io_refused() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = classify_io_error(&io_err);
        assert!(matches!(err, IpcError::ConnectionRefused(_)));
        assert!(err.is_connection_error());
    }

    #[test]
    fn classify_io_timed_out() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "slow");
        let err = classify_io_error(&io_err);
        assert!(matches!(err, IpcError::Timeout { .. }));
        assert!(err.is_retriable());
    }

    #[test]
    fn classify_io_broken_pipe() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe");
        let err = classify_io_error(&io_err);
        assert!(matches!(err, IpcError::ConnectionReset(_)));
        assert!(err.is_retriable());
    }

    #[test]
    fn primal_not_found_is_connection_error() {
        let err = IpcError::PrimalNotFound {
            domain: "ai".to_owned(),
        };
        assert!(err.is_connection_error());
        assert!(!err.is_retriable());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn serialization_not_recoverable() {
        let err = IpcError::Serialization {
            detail: "bad json".to_owned(),
        };
        assert!(!err.is_recoverable());
        assert!(!err.is_retriable());
    }

    #[test]
    fn display_all_variants() {
        let variants: Vec<IpcError> = vec![
            IpcError::PrimalNotFound {
                domain: "ai".to_owned(),
            },
            IpcError::ConnectionRefused("refused".to_owned()),
            IpcError::ConnectionReset("reset".to_owned()),
            IpcError::Timeout { ms: 5000 },
            IpcError::ProtocolError {
                detail: "bad".to_owned(),
            },
            IpcError::MethodNotFound {
                method: "foo.bar".to_owned(),
            },
            IpcError::ApplicationError {
                code: -32603,
                message: "internal".to_owned(),
            },
            IpcError::Serialization {
                detail: "parse".to_owned(),
            },
        ];
        for v in &variants {
            assert!(!v.to_string().is_empty());
        }
    }
}
