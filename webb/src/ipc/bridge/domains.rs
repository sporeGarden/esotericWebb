// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific IPC delegations.
//!
//! Each primal domain gets a section of 1-3 line methods that map to
//! the generic call helpers in [`super`]. Keeping these delegations
//! in their own file prevents the bridge module from growing beyond
//! the 1000-line ecosystem limit as new domains are wired.
//!
//! ## Architecture (V6 — ludoSpring decomposition)
//!
//! Webb no longer routes through ludoSpring. All calls go to the
//! underlying primals directly:
//!
//! - **AI domain** (Squirrel): narration, NPC dialogue, voice checks,
//!   summarization — using biomeOS semantic methods (`ai.query`,
//!   `ai.suggest`, `ai.analyze`).
//! - **Visualization** (petalTongue): scene rendering, input polling.
//! - **Compute** (ToadStool): GPU dispatch.
//! - **Storage** (NestGate): key-value store.
//! - **DAG** (rhizoCrypt): provenance, session lifecycle.
//! - **Lineage** (LoamSpine): certificates.
//!
//! Game science (flow, engagement, DDA) is now local via `science/`.

use crate::ipc::envelope::IpcError;
use crate::ipc::petaltongue::{InputEvent, METHOD_INTERACTION_POLL, METHOD_RENDER_SCENE};
use crate::ipc::primal_names::domain;
use crate::ipc::squirrel::{
    ChatResponse, DialogueResponse, METHOD_AI_ANALYZE, METHOD_AI_QUERY, METHOD_AI_SUGGEST,
    VoiceNote,
};
use crate::ipc::{
    METHOD_CERT_MINT, METHOD_COMPUTE_SUBMIT, METHOD_DAG_EVENT_APPEND, METHOD_DAG_FRONTIER_GET,
    METHOD_DAG_MERKLE_ROOT, METHOD_DAG_QUERY_VERTICES, METHOD_DAG_SESSION_COMPLETE,
    METHOD_DAG_SESSION_CREATE, METHOD_STORAGE_RETRIEVE, METHOD_STORAGE_STORE, SIGNAL_NEST_COMMIT,
    SIGNAL_NEST_STORE,
};

use super::{PrimalBridge, degraded_summary_limit};

impl PrimalBridge {
    // ── AI domain (Squirrel) ────────────────────────────────

    /// Generate narration via the AI primal (`ai.query`).
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
        self.call_or_default(domain::AI, METHOD_AI_QUERY, params, default)
    }

    /// Summarize context via the AI primal (`ai.suggest`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_summarize(&mut self, context: &str) -> Result<String, IpcError> {
        if self.has(domain::AI) {
            let params = serde_json::json!({ "text": context });
            if let Ok(resp) = self.resilient_call(domain::AI, METHOD_AI_SUGGEST, params) {
                if let Some(serde_json::Value::String(text)) = resp.result {
                    return Ok(text);
                }
            }
        }
        let truncated: String = context.chars().take(degraded_summary_limit()).collect();
        Ok(format!("[degraded: summary unavailable] {truncated}..."))
    }

    /// NPC dialogue via AI primal (`ai.query` with NPC personality context).
    ///
    /// Webb formats the NPC context into a prompt and calls Squirrel
    /// directly. No ludoSpring mediation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn npc_dialogue(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<DialogueResponse, IpcError> {
        if !self.has(domain::AI) {
            return Ok(DialogueResponse {
                text: "[AI primal unavailable — NPC dialogue degraded]".to_owned(),
                voice_notes: Vec::new(),
                passive_checks_fired: false,
                degraded: true,
            });
        }

        let npc_id = params
            .get("npc_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let context = params
            .get("context")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let prompt =
            format!("You are NPC '{npc_id}' in a CRPG. Respond in character.\nContext: {context}");

        let query_params = serde_json::json!({
            "messages": [{"role": "user", "content": prompt}],
        });
        match self.resilient_call(domain::AI, METHOD_AI_QUERY, query_params) {
            Ok(resp) => {
                if let Some(result) = resp.result {
                    if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                        return Ok(DialogueResponse {
                            text: chat.text,
                            voice_notes: Vec::new(),
                            passive_checks_fired: false,
                            degraded: false,
                        });
                    }
                }
                Ok(DialogueResponse {
                    text: "[AI response could not be parsed]".to_owned(),
                    voice_notes: Vec::new(),
                    passive_checks_fired: false,
                    degraded: true,
                })
            }
            Err(_) => Ok(DialogueResponse {
                text: "[AI primal unavailable — NPC dialogue degraded]".to_owned(),
                voice_notes: Vec::new(),
                passive_checks_fired: false,
                degraded: true,
            }),
        }
    }

    /// Narrate an action via AI primal (`ai.suggest`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn narrate_action(&mut self, params: &serde_json::Value) -> Result<ChatResponse, IpcError> {
        let action = params
            .get("action")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let outcome = params
            .get("outcome")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let suggest_params = serde_json::json!({
            "text": format!("Narrate: {action}. Outcome: {outcome}"),
        });
        self.call_or_default(
            domain::AI,
            METHOD_AI_SUGGEST,
            suggest_params,
            ChatResponse {
                text: "[AI primal unavailable — narration degraded]".to_owned(),
                model: "none".to_owned(),
                tokens: 0,
            },
        )
    }

    /// Internal voice check via AI primal (`ai.analyze`).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn voice_check(&mut self, params: &serde_json::Value) -> Result<Vec<VoiceNote>, IpcError> {
        self.call_or_default(domain::AI, METHOD_AI_ANALYZE, params.clone(), Vec::new())
    }

    // ── Visualization domain (petalTongue) ─────────────────

    /// Push a scene payload for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn render_scene(&mut self, scene: &serde_json::Value) -> Result<(), IpcError> {
        self.call_fire(domain::VISUALIZATION, METHOD_RENDER_SCENE, scene.clone())
    }

    /// Poll for player input events from petalTongue.
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

    // ── Compute domain (ToadStool) ────────────────────────

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

    // ── Storage domain (NestGate) ─────────────────────────

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

    // ── Lineage domain (LoamSpine) ──────────────────────────

    /// Mint a certificate via `LoamSpine`.
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

    // ── Signal dispatch (Wave 17 orchestration collapse) ─────

    /// Atomic provenance step via `nest.store` signal.
    ///
    /// When biomeOS routes the signal, it decomposes into:
    /// `NestGate.content.put → rhizoCrypt.dag.event.append → loamSpine.spine.seal → sweetGrass.braid.create`
    ///
    /// Falls back to direct `dag.event.append` if the signal is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if all paths fail.
    pub fn nest_store(&mut self, params: &serde_json::Value) -> Result<bool, IpcError> {
        if self.has_neural_api() {
            let signal_params = serde_json::json!({
                "signal": SIGNAL_NEST_STORE,
                "payload": params,
            });
            match self.neural_api_call("nest", SIGNAL_NEST_STORE, signal_params) {
                Ok(resp) if resp.error.is_none() => return Ok(true),
                Ok(_) | Err(_) => {
                    tracing::debug!(
                        "nest.store signal unavailable — falling back to dag.event.append"
                    );
                }
            }
        }
        self.provenance_append(params)
    }

    /// Atomic session finalization via `nest.commit` signal.
    ///
    /// When biomeOS routes the signal, it decomposes into:
    /// `rhizoCrypt.dehydrate → bearDog.sign → NestGate.store → loamSpine.seal`
    ///
    /// Falls back to direct `dag.session.complete` if the signal is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if all paths fail.
    pub fn nest_commit(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        if self.has_neural_api() {
            let signal_params = serde_json::json!({
                "signal": SIGNAL_NEST_COMMIT,
                "payload": params,
            });
            match self.neural_api_call("nest", SIGNAL_NEST_COMMIT, signal_params) {
                Ok(resp) if resp.error.is_none() => return Ok(()),
                Ok(_) | Err(_) => {
                    tracing::debug!(
                        "nest.commit signal unavailable — falling back to dag.session.complete"
                    );
                }
            }
        }
        self.dag_session_complete(params)
    }

    /// Self-announce to biomeOS (outbound `primal.announce`).
    ///
    /// Registers esotericWebb's socket, capabilities, methods, composition tiers,
    /// cost hints, and latency estimates with the orchestration layer.
    /// Aligned with Wave 45 announce schema (Songbird/BearDog key alignment).
    /// Falls back silently if biomeOS is unavailable.
    pub fn announce_self(&mut self, socket: &str, methods: &[&str]) {
        if !self.has_neural_api() {
            tracing::debug!("No neural-api connection — skipping primal.announce");
            return;
        }
        let params = serde_json::json!({
            "primal": "esotericwebb",
            "socket": socket,
            "capabilities": ["narrative", "session", "mcp"],
            "methods": methods,
            "signal_tiers": ["nest", "meta"],
            "version": env!("CARGO_PKG_VERSION"),
            "cost_hints": {
                "session.act": "low",
                "session.start": "medium",
                "tools.call": "medium",
                "webb.scene.current": "low",
            },
            "latency_estimates": {
                "session.act": "< 10ms",
                "session.start": "< 50ms",
                "tools.call": "< 100ms",
                "webb.scene.current": "< 5ms",
            },
        });
        match self.neural_api_call("lifecycle", "primal.announce", params) {
            Ok(resp) if resp.error.is_none() => {
                tracing::info!("primal.announce accepted by biomeOS");
            }
            Ok(_) | Err(_) => {
                tracing::debug!("primal.announce not accepted — operating without registration");
            }
        }
    }
}
