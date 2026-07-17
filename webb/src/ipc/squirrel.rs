// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the AI primal (`ai.*` capabilities).
//!
//! Consumes `ai.*` methods for narration, dialogue, and analysis.
//! Method names align with the biomeOS capability registry
//! (`ai.query`, `ai.suggest`, `ai.analyze`) which routes to Squirrel's
//! native methods (`query`, `suggest`, `analyze`).
//!
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency. The `PrimalBridge` handles all IPC with graceful degradation.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for brevity")]
mod tests {
    use super::*;

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
