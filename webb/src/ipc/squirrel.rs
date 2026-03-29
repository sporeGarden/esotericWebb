// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the AI primal (`ai.*` capabilities).
//!
//! Consumes `ai.*` methods for narration, dialogue, and analysis.
//! Method names align with the biomeOS capability registry
//! (`ai.query`, `ai.suggest`, `ai.analyze`) which routes to Squirrel's
//! native methods (`query`, `suggest`, `analyze`).
//!
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency.
//!
//! When a live [`PrimalClient`] connection exists, methods send real
//! JSON-RPC calls. Otherwise they return graceful degradation defaults.

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::envelope::IpcError;

/// AI query (narration, dialogue, chat). Maps to Squirrel `query`.
pub const METHOD_AI_QUERY: &str = "ai.query";
/// AI suggest (summarization, short-form generation). Maps to Squirrel `suggest`.
pub const METHOD_AI_SUGGEST: &str = "ai.suggest";
/// AI analyze (voice checks, classification). Maps to Squirrel `analyze`.
pub const METHOD_AI_ANALYZE: &str = "ai.analyze";

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

/// NPC dialogue response (constructed from AI primal output).
///
/// When Webb calls `ai.query` with NPC personality context, it wraps
/// the response into this structure for the enrichment pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponse {
    /// The NPC's spoken text.
    pub text: String,
    /// Internal voice interjections (if any fired).
    pub voice_notes: Vec<VoiceNote>,
    /// Whether any passive checks triggered.
    pub passive_checks_fired: bool,
    /// Whether this response is a degradation placeholder (primal unavailable).
    #[serde(default)]
    pub degraded: bool,
}

/// A voice interjection from the internal voice system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceNote {
    /// Voice identifier (e.g. "logic", "empathy").
    pub voice_id: String,
    /// The voice's interjection text.
    pub text: String,
    /// Priority (lower = more important).
    pub priority: u32,
}

/// Client for AI primal capabilities (resolved from `plasmidBin/`).
#[derive(Debug)]
pub struct SquirrelClient {
    available: bool,
}

impl SquirrelClient {
    /// Create a new client.
    #[must_use]
    pub const fn new(available: bool) -> Self {
        Self { available }
    }

    /// Whether the AI primal was discovered and is healthy.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Generate narration text via live connection or degradation fallback.
    ///
    /// When a live `PrimalClient` is provided, sends a real JSON-RPC call.
    /// Otherwise returns a clearly-labeled degradation placeholder — no
    /// path pretends to be a real AI response.
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
            let resp = c.call(METHOD_AI_QUERY, params)?;
            if let Some(result) = resp.result {
                if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                    return Ok(chat);
                }
            }
        }

        Ok(ChatResponse {
            text: format!("[degraded: AI primal not connected — narration for: {prompt}]"),
            model: "degraded".to_owned(),
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
            let resp = c.call(METHOD_AI_SUGGEST, params)?;
            if let Some(serde_json::Value::String(text)) = resp.result {
                return Ok(text);
            }
        }

        let truncated: String = context.chars().take(200).collect();
        Ok(format!("[degraded: summary unavailable] {truncated}..."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_returns_degraded_placeholder() {
        let client = SquirrelClient::new(false);
        let resp = client.narrate("test prompt", None);
        assert!(resp.is_ok());
        let chat = resp.unwrap_or(ChatResponse {
            text: String::new(),
            model: String::new(),
            tokens: 0,
        });
        assert!(chat.text.contains("degraded"));
        assert_eq!(chat.model, "degraded");
    }

    #[test]
    fn available_without_client_still_degrades() {
        let client = SquirrelClient::new(true);
        let resp = client
            .narrate("describe the room", None)
            .unwrap_or(ChatResponse {
                text: String::new(),
                model: String::new(),
                tokens: 0,
            });
        assert!(resp.text.contains("degraded"));
        assert_eq!(resp.model, "degraded");
    }

    #[test]
    fn summarize_degradation() {
        let client = SquirrelClient::new(false);
        let resp = client.summarize("a long context string", None);
        assert!(resp.is_ok());
        let text = resp.unwrap_or_default();
        assert!(text.contains("degraded"));
    }

    #[test]
    fn dialogue_response_serde_round_trip() {
        let resp = DialogueResponse {
            text: "Hello traveler".to_owned(),
            voice_notes: vec![VoiceNote {
                voice_id: "logic".to_owned(),
                text: "Be careful.".to_owned(),
                priority: 1,
            }],
            passive_checks_fired: true,
            degraded: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: DialogueResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.text, "Hello traveler");
        assert!(!back.degraded);
        assert_eq!(back.voice_notes.len(), 1);
        assert!(back.passive_checks_fired);
    }

    #[test]
    fn dialogue_degraded_defaults_to_false() {
        let json = r#"{"text":"hi","voice_notes":[],"passive_checks_fired":false}"#;
        let resp: DialogueResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.degraded);
    }
}
