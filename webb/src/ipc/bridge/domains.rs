// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific IPC delegations.
//!
//! Each primal domain gets a section of 1-3 line methods that map to
//! the generic call helpers in [`super`]. Keeping these delegations
//! in their own file prevents the bridge module from growing beyond
//! the 1000-line ecosystem limit as new domains are wired.

use crate::ipc::envelope::IpcError;
use crate::ipc::ludospring::{
    DdaResult, DialogueResponse, EngagementResult, FlowResult, METHOD_BEGIN_SESSION,
    METHOD_COMPLETE_SESSION, METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW,
    METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE, METHOD_PUSH_SCENE, METHOD_VOICE_CHECK, VoiceNote,
};
use crate::ipc::petaltongue::{InputEvent, METHOD_INTERACTION_POLL, METHOD_RENDER_SCENE};
use crate::ipc::primal_names::domain;
use crate::ipc::squirrel::{ChatResponse, METHOD_AI_CHAT, METHOD_AI_SUMMARIZE};
use crate::ipc::{
    METHOD_CERT_MINT, METHOD_COMPUTE_SUBMIT, METHOD_DAG_EVENT_APPEND, METHOD_DAG_FRONTIER_GET,
    METHOD_DAG_MERKLE_ROOT, METHOD_DAG_QUERY_VERTICES, METHOD_DAG_SESSION_COMPLETE,
    METHOD_DAG_SESSION_CREATE, METHOD_STORAGE_RETRIEVE, METHOD_STORAGE_STORE,
};

use super::{PrimalBridge, degraded_summary_limit};

impl PrimalBridge {
    // ── AI domain (Squirrel) ────────────────────────────────

    /// Generate narration via the AI primal.
    ///
    /// Degrades to a placeholder if Squirrel is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_narrate(&mut self, prompt: &str) -> Result<ChatResponse, IpcError> {
        let params = serde_json::json!({
            "messages": [{"role": "user", "content": prompt}],
        });
        let default = ChatResponse {
            text: format!("[AI primal unavailable — narration placeholder for: {prompt}]"),
            model: "none".to_owned(),
            tokens: 0,
        };
        self.call_or_default(domain::AI, METHOD_AI_CHAT, params, default)
    }

    /// Summarize context via the AI primal.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_summarize(&mut self, context: &str) -> Result<String, IpcError> {
        if self.has(domain::AI) {
            let params = serde_json::json!({ "text": context });
            if let Ok(resp) = self.resilient_call(domain::AI, METHOD_AI_SUMMARIZE, params) {
                if let Some(serde_json::Value::String(text)) = resp.result {
                    return Ok(text);
                }
            }
        }
        let truncated: String = context.chars().take(degraded_summary_limit()).collect();
        Ok(format!("[degraded: summary unavailable] {truncated}..."))
    }

    // ── Visualization domain (`PetalTongue`) ─────────────────

    /// Push a scene payload for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn render_scene(&mut self, scene: &serde_json::Value) -> Result<(), IpcError> {
        self.call_fire(domain::VISUALIZATION, METHOD_RENDER_SCENE, scene.clone())
    }

    /// Poll for player input events from `petalTongue`.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn poll_input(&mut self) -> Result<Vec<InputEvent>, IpcError> {
        self.call_or_default(
            domain::VISUALIZATION,
            METHOD_INTERACTION_POLL,
            serde_json::Value::Null,
            Vec::new(),
        )
    }

    // ── Compute domain (`ToadStool`) ────────────────────────

    /// Submit a compute task.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn compute_submit(&mut self, task: &serde_json::Value) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::COMPUTE,
            METHOD_COMPUTE_SUBMIT,
            task.clone(),
            &["job_id"],
        )
    }

    // ── Storage domain (`NestGate`) ─────────────────────────

    /// Store a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn store(&mut self, key: &str, value: &serde_json::Value) -> Result<bool, IpcError> {
        if !self.has(domain::STORAGE) {
            return Ok(false);
        }
        let params = serde_json::json!({ "key": key, "value": value });
        let resp = self.resilient_call(domain::STORAGE, METHOD_STORAGE_STORE, params)?;
        Ok(resp.error.is_none())
    }

    /// Retrieve a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn retrieve(&mut self, key: &str) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(
            domain::STORAGE,
            METHOD_STORAGE_RETRIEVE,
            serde_json::json!({ "key": key }),
        )
    }

    // ── Game science domain (`LudoSpring`) ──────────────────

    /// Evaluate flow state.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn evaluate_flow(&mut self, params: &serde_json::Value) -> Result<FlowResult, IpcError> {
        self.call_or_default(
            domain::GAME,
            METHOD_EVALUATE_FLOW,
            params.clone(),
            FlowResult {
                flow_score: 0.5,
                in_flow: false,
            },
        )
    }

    /// Get engagement metrics.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn engagement(&mut self, params: &serde_json::Value) -> Result<EngagementResult, IpcError> {
        self.call_or_default(
            domain::GAME,
            METHOD_ENGAGEMENT,
            params.clone(),
            EngagementResult {
                actions_per_minute: 0.0,
                exploration_ratio: 0.0,
                engagement_score: 0.5,
            },
        )
    }

    /// Get DDA recommendation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn difficulty_adjustment(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<DdaResult, IpcError> {
        self.call_or_default(
            domain::GAME,
            METHOD_DIFFICULTY_ADJUSTMENT,
            params.clone(),
            DdaResult {
                adjustment: 0.0,
                reason: "game science primal unavailable — no adjustment".to_owned(),
            },
        )
    }

    /// NPC dialogue via ludoSpring → Squirrel delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn npc_dialogue(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<DialogueResponse, IpcError> {
        self.call_or_default(
            domain::GAME,
            METHOD_NPC_DIALOGUE,
            params.clone(),
            DialogueResponse {
                text: "[game science primal unavailable — NPC dialogue degraded]".to_owned(),
                voice_notes: Vec::new(),
                passive_checks_fired: false,
                degraded: true,
            },
        )
    }

    /// Narrate an action via ludoSpring → Squirrel delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn narrate_action(&mut self, params: &serde_json::Value) -> Result<ChatResponse, IpcError> {
        self.call_or_default(
            domain::GAME,
            METHOD_NARRATE_ACTION,
            params.clone(),
            ChatResponse {
                text: "[game science primal unavailable — narration degraded]".to_owned(),
                model: "none".to_owned(),
                tokens: 0,
            },
        )
    }

    /// Internal voice check via ludoSpring.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn voice_check(&mut self, params: &serde_json::Value) -> Result<Vec<VoiceNote>, IpcError> {
        self.call_or_default(domain::GAME, METHOD_VOICE_CHECK, params.clone(), Vec::new())
    }

    /// Push scene via ludoSpring → `petalTongue` delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn game_push_scene(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        self.call_fire(domain::GAME, METHOD_PUSH_SCENE, params.clone())
    }

    /// Begin a game session in the provenance system via ludoSpring.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn game_begin_session(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::GAME,
            METHOD_BEGIN_SESSION,
            params.clone(),
            &["session_id", "id"],
        )
    }

    /// Complete a game session in the provenance system via ludoSpring.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn game_complete_session(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        self.call_fire(domain::GAME, METHOD_COMPLETE_SESSION, params.clone())
    }

    // ── Provenance / DAG domain (rhizoCrypt) ────────────────

    /// Append a provenance vertex to the session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn provenance_append(&mut self, vertex: &serde_json::Value) -> Result<bool, IpcError> {
        if !self.has(domain::DAG) {
            return Ok(false);
        }
        let resp = self.resilient_call(domain::DAG, METHOD_DAG_EVENT_APPEND, vertex.clone())?;
        Ok(resp.error.is_none())
    }

    /// Create a new session on the DAG primal.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_session_create(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::DAG,
            METHOD_DAG_SESSION_CREATE,
            params.clone(),
            &["session_id", "id"],
        )
    }

    /// Append an event to a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_event_append(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if !self.has(domain::DAG) {
            return Ok(None);
        }
        let resp = self.resilient_call(domain::DAG, METHOD_DAG_EVENT_APPEND, params.clone())?;
        if let Some(result) = resp.result {
            for field in &["vertex_id", "id"] {
                if let Some(id) = result.get(*field).and_then(serde_json::Value::as_str) {
                    return Ok(Some(id.to_owned()));
                }
            }
            return Ok(Some("ok".to_owned()));
        }
        Ok(None)
    }

    /// Get the current frontier of a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_frontier_get(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::DAG, METHOD_DAG_FRONTIER_GET, params.clone())
    }

    /// Get the Merkle root of a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_merkle_root(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::DAG,
            METHOD_DAG_MERKLE_ROOT,
            params.clone(),
            &["root", "merkle_root"],
        )
    }

    /// Complete a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_session_complete(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        self.call_fire(domain::DAG, METHOD_DAG_SESSION_COMPLETE, params.clone())
    }

    /// Query vertices from a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_query_vertices(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::DAG, METHOD_DAG_QUERY_VERTICES, params.clone())
    }

    // ── Lineage domain (loamSpine) ──────────────────────────

    /// Mint a certificate via loamSpine.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn mint_certificate(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::LINEAGE, METHOD_CERT_MINT, params.clone())
    }
}
