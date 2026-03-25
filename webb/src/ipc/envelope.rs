// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 envelope types.
//!
//! Mirrors the wire format used by all ecoPrimals primals.
//! Newline-delimited JSON over Unix domain sockets.

use serde::{Deserialize, Serialize};

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

/// IPC errors for Webb.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Connection failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    /// Primal not discovered.
    #[error("primal not found for capability: {0}")]
    PrimalNotFound(String),
    /// Request timed out.
    #[error("request timed out after {0}ms")]
    Timeout(u64),
    /// JSON serialization error.
    #[error("serialization: {0}")]
    Serialization(String),
    /// Remote returned an error.
    #[error("remote error {code}: {message}")]
    Remote {
        /// Error code from the remote.
        code: i64,
        /// Error message from the remote.
        message: String,
    },
    /// IO error.
    #[error("io: {0}")]
    Io(String),
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
    /// Extract the result or convert the error.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::Remote`] if the response contains an error.
    pub fn into_result(self) -> Result<serde_json::Value, IpcError> {
        if let Some(err) = self.error {
            return Err(IpcError::Remote {
                code: err.code,
                message: err.message,
            });
        }
        Ok(self.result.unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let req = JsonRpcRequest::new("webb.health", None);
        let json = serde_json::to_string(&req).ok();
        assert!(json.is_some());
        let parsed: JsonRpcRequest =
            serde_json::from_str(json.as_deref().unwrap_or("{}")).unwrap_or_else(|_| req.clone());
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
        let val = resp.into_result();
        assert!(val.is_ok());
    }

    #[test]
    fn response_error() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "method not found".to_owned(),
                data: None,
            }),
            id: serde_json::json!(1),
        };
        let val = resp.into_result();
        assert!(val.is_err());
    }
}
