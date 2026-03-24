// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the AI primal (`ai.*` capabilities).
//!
//! Consumes `ai.*` methods for narration, inference, and summarization.
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency. Design derived from Squirrel's AI/MCP architecture.
//!
//! When a live [`PrimalClient`] connection exists, methods send real
//! JSON-RPC calls. Otherwise they return graceful degradation defaults.

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::envelope::IpcError;

/// AI primal JSON-RPC method names.
pub const METHOD_AI_CHAT: &str = "ai.chat";
/// AI inference.
pub const METHOD_AI_INFERENCE: &str = "ai.inference";
/// Context summarization.
pub const METHOD_AI_SUMMARIZE: &str = "ai.summarize";

/// Chat response from the AI primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated text.
    pub text: String,
    /// Model used for generation.
    pub model: String,
    /// Token count.
    pub tokens: u32,
}

/// Client for AI primal capabilities (resolved from `plasmidBin/`).
#[derive(Debug)]
pub struct SquirrelClient {
    available: bool,
}

impl SquirrelClient {
    /// Create a new client.
    pub const fn new(available: bool) -> Self {
        Self { available }
    }

    /// Whether the AI primal was discovered and is healthy.
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Generate narration text via live connection or degradation fallback.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn narrate(
        &self,
        prompt: &str,
        client: Option<&mut PrimalClient>,
    ) -> Result<ChatResponse, IpcError> {
        if let Some(c) = client {
            let params = serde_json::json!({
                "messages": [{"role": "user", "content": prompt}],
            });
            let resp = c.call(METHOD_AI_CHAT, params)?;
            if let Some(result) = resp.result {
                if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                    return Ok(chat);
                }
            }
        }

        if !self.available {
            return Ok(ChatResponse {
                text: format!("[AI primal unavailable — narration placeholder for: {prompt}]"),
                model: "none".to_owned(),
                tokens: 0,
            });
        }
        Ok(ChatResponse {
            text: format!("[narration for: {prompt}]"),
            model: "local".to_owned(),
            tokens: 0,
        })
    }

    /// Summarize context via live connection or degradation fallback.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn summarize(
        &self,
        context: &str,
        client: Option<&mut PrimalClient>,
    ) -> Result<String, IpcError> {
        if let Some(c) = client {
            let params = serde_json::json!({ "text": context });
            let resp = c.call(METHOD_AI_SUMMARIZE, params)?;
            if let Some(serde_json::Value::String(text)) = resp.result {
                return Ok(text);
            }
        }

        if !self.available {
            let truncated: String = context.chars().take(200).collect();
            return Ok(format!("[summary unavailable] {truncated}..."));
        }
        Ok(format!("[summary of {}-char context]", context.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_returns_placeholder() {
        let client = SquirrelClient::new(false);
        let resp = client.narrate("test prompt", None);
        assert!(resp.is_ok());
        let chat = resp.unwrap_or(ChatResponse {
            text: String::new(),
            model: String::new(),
            tokens: 0,
        });
        assert!(chat.text.contains("unavailable"));
    }

    #[test]
    fn available_returns_narration() {
        let client = SquirrelClient::new(true);
        let resp = client.narrate("describe the room", None);
        assert!(resp.is_ok());
    }

    #[test]
    fn summarize_degradation() {
        let client = SquirrelClient::new(false);
        let resp = client.summarize("a long context string", None);
        assert!(resp.is_ok());
    }
}
