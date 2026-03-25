// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC client for the game science primal (`game.*` capabilities).
//!
//! Consumes `game.*` methods via JSON-RPC over Unix domain sockets.
//! The primal binary is resolved from `plasmidBin/` — no spring source
//! dependency. Design derived from ludoSpring's RPGPT science.
//!
//! When a live [`PrimalClient`] connection exists, methods send real
//! JSON-RPC calls. Otherwise they return graceful degradation defaults.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::PrimalClient;
use super::envelope::IpcError;
use super::squirrel::ChatResponse;

/// Game science primal JSON-RPC method names.
pub const METHOD_EVALUATE_FLOW: &str = "game.evaluate_flow";
/// Engagement metrics.
pub const METHOD_ENGAGEMENT: &str = "game.engagement";
/// Dynamic difficulty adjustment.
pub const METHOD_DIFFICULTY_ADJUSTMENT: &str = "game.difficulty_adjustment";
/// Begin game session (provenance).
pub const METHOD_BEGIN_SESSION: &str = "game.begin_session";
/// Record game action (provenance).
pub const METHOD_RECORD_ACTION: &str = "game.record_action";
/// Complete game session (provenance).
pub const METHOD_COMPLETE_SESSION: &str = "game.complete_session";
/// NPC dialogue via AI primal.
pub const METHOD_NPC_DIALOGUE: &str = "game.npc_dialogue";
/// Narrate a game action.
pub const METHOD_NARRATE_ACTION: &str = "game.narrate_action";
/// Internal voice check.
pub const METHOD_VOICE_CHECK: &str = "game.voice_check";
/// Push scene to visualization primal.
pub const METHOD_PUSH_SCENE: &str = "game.push_scene";
/// Query DAG vertices.
pub const METHOD_QUERY_VERTICES: &str = "game.query_vertices";
/// Mint a certificate via the certificate primal.
pub const METHOD_MINT_CERTIFICATE: &str = "game.mint_certificate";

/// Flow evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    /// Flow state (0.0 = anxiety, 0.5 = flow, 1.0 = boredom).
    pub flow_score: f64,
    /// Whether the player is currently in flow.
    pub in_flow: bool,
}

/// Engagement metrics result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementResult {
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration ratio (unique areas visited / total).
    pub exploration_ratio: f64,
    /// Overall engagement score.
    pub engagement_score: f64,
}

/// DDA recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdaResult {
    /// Recommended difficulty adjustment (-1.0 to 1.0).
    pub adjustment: f64,
    /// Reason for the recommendation.
    pub reason: String,
}

/// NPC dialogue response.
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

/// A voice interjection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceNote {
    /// Voice identifier (e.g. "logic", "empathy").
    pub voice_id: String,
    /// The voice's interjection text.
    pub text: String,
    /// Priority (lower = more important).
    pub priority: u32,
}

/// Client for game science primal capabilities (resolved from `plasmidBin/`).
#[derive(Debug)]
pub struct LudoSpringClient {
    available: bool,
}

impl LudoSpringClient {
    /// Create a new client. Availability is determined at discovery time.
    #[must_use]
    pub const fn new(available: bool) -> Self {
        Self { available }
    }

    /// Whether the game science primal was discovered and is healthy.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Evaluate flow state via live connection or degradation fallback.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails and no degradation is possible.
    pub fn evaluate_flow(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<FlowResult, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_EVALUATE_FLOW, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(flow) = serde_json::from_value::<FlowResult>(result) {
                    return Ok(flow);
                }
            }
        }
        Ok(FlowResult {
            flow_score: 0.5,
            in_flow: false,
        })
    }

    /// Get engagement metrics via live connection or degradation fallback.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn engagement(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<EngagementResult, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_ENGAGEMENT, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(eng) = serde_json::from_value::<EngagementResult>(result) {
                    return Ok(eng);
                }
            }
        }
        Ok(EngagementResult {
            actions_per_minute: 0.0,
            exploration_ratio: 0.0,
            engagement_score: 0.5,
        })
    }

    /// Get DDA recommendation via live connection or degradation fallback.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn difficulty_adjustment(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<DdaResult, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_DIFFICULTY_ADJUSTMENT, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(dda) = serde_json::from_value::<DdaResult>(result) {
                    return Ok(dda);
                }
            }
        }
        if !self.available {
            return Ok(DdaResult {
                adjustment: 0.0,
                reason: "game science primal unavailable — no adjustment".to_owned(),
            });
        }
        Ok(DdaResult {
            adjustment: 0.0,
            reason: "default".to_owned(),
        })
    }

    /// NPC dialogue via ludoSpring → Squirrel delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn npc_dialogue(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<DialogueResponse, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_NPC_DIALOGUE, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(d) = serde_json::from_value::<DialogueResponse>(result) {
                    return Ok(d);
                }
            }
        }
        Ok(DialogueResponse {
            text: "[game science primal unavailable — NPC dialogue degraded]".to_owned(),
            voice_notes: Vec::new(),
            passive_checks_fired: false,
            degraded: true,
        })
    }

    /// Narrate an action via ludoSpring → Squirrel delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn narrate_action(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<ChatResponse, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_NARRATE_ACTION, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                    return Ok(chat);
                }
            }
        }
        Ok(ChatResponse {
            text: "[game science primal unavailable — narration degraded]".to_owned(),
            model: "none".to_owned(),
            tokens: 0,
        })
    }

    /// Internal voice check via ludoSpring.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn voice_check(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<Vec<VoiceNote>, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_VOICE_CHECK, params.clone())?;
            if let Some(result) = resp.result {
                if let Ok(notes) = serde_json::from_value::<Vec<VoiceNote>>(result) {
                    return Ok(notes);
                }
            }
        }
        Ok(Vec::new())
    }

    /// Push scene to visualization via ludoSpring delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn push_scene(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<(), IpcError> {
        if let Some(c) = client {
            let _ = c.call(METHOD_PUSH_SCENE, params.clone())?;
        }
        Ok(())
    }

    /// Begin a game session via ludoSpring provenance delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn begin_session(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<Option<String>, IpcError> {
        if let Some(c) = client {
            let resp = c.call(METHOD_BEGIN_SESSION, params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("session_id")
                    .or_else(|| result.get("id"))
                    .and_then(Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Complete a game session via ludoSpring provenance delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails.
    pub fn complete_session(
        &self,
        params: &Value,
        client: Option<&mut PrimalClient>,
    ) -> Result<(), IpcError> {
        if let Some(c) = client {
            let _ = c.call(METHOD_COMPLETE_SESSION, params.clone())?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_client_degrades() {
        let client = LudoSpringClient::new(false);
        assert!(!client.is_available());
        let flow = client.evaluate_flow(&Value::Null, None).unwrap();
        assert!((flow.flow_score - 0.5).abs() < f64::EPSILON);
        assert!(!flow.in_flow);
    }

    #[test]
    fn available_client_returns_defaults() {
        let client = LudoSpringClient::new(true);
        assert!(client.is_available());
        let eng = client.engagement(&Value::Null, None).unwrap();
        assert!((eng.engagement_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn dda_unavailable_gives_unavailable_reason() {
        let client = LudoSpringClient::new(false);
        let dda = client.difficulty_adjustment(&Value::Null, None).unwrap();
        assert!((dda.adjustment).abs() < f64::EPSILON);
        assert!(dda.reason.contains("unavailable"));
    }

    #[test]
    fn dda_available_gives_default_reason() {
        let client = LudoSpringClient::new(true);
        let dda = client.difficulty_adjustment(&Value::Null, None).unwrap();
        assert!((dda.adjustment).abs() < f64::EPSILON);
        assert_eq!(dda.reason, "default");
    }

    #[test]
    fn npc_dialogue_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        let resp = client.npc_dialogue(&Value::Null, None).unwrap();
        assert!(resp.degraded);
        assert!(resp.text.contains("degraded"));
        assert!(resp.voice_notes.is_empty());
        assert!(!resp.passive_checks_fired);
    }

    #[test]
    fn narrate_action_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        let resp = client.narrate_action(&Value::Null, None).unwrap();
        assert_eq!(resp.model, "none");
        assert!(resp.text.contains("degraded"));
    }

    #[test]
    fn voice_check_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        let notes = client.voice_check(&Value::Null, None).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn push_scene_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        assert!(client.push_scene(&Value::Null, None).is_ok());
    }

    #[test]
    fn begin_session_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        let result = client.begin_session(&Value::Null, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn complete_session_degrades_without_client() {
        let client = LudoSpringClient::new(false);
        assert!(client.complete_session(&Value::Null, None).is_ok());
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
    fn degraded_defaults_to_false() {
        let json = r#"{"text":"hi","voice_notes":[],"passive_checks_fired":false}"#;
        let resp: DialogueResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.degraded);
    }

    #[test]
    fn flow_result_serde_round_trip() {
        let flow = FlowResult {
            flow_score: 0.7,
            in_flow: true,
        };
        let json = serde_json::to_string(&flow).unwrap();
        let back: FlowResult = serde_json::from_str(&json).unwrap();
        assert!((back.flow_score - 0.7).abs() < f64::EPSILON);
        assert!(back.in_flow);
    }

    #[test]
    fn engagement_result_serde_round_trip() {
        let eng = EngagementResult {
            actions_per_minute: 3.5,
            exploration_ratio: 0.8,
            engagement_score: 0.9,
        };
        let json = serde_json::to_string(&eng).unwrap();
        let back: EngagementResult = serde_json::from_str(&json).unwrap();
        assert!((back.actions_per_minute - 3.5).abs() < f64::EPSILON);
    }
}
