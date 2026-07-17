// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific IPC delegations.
//!
//! Each primal domain gets a section of 1-3 line methods that map to
//! the generic call helpers in [`super`]. Keeping these delegations
//! in their own file prevents the bridge module from growing beyond
//! the 1000-line ecosystem limit as new domains are wired.
//!
//! ## Architecture (V6+ — direct primal composition)
//!
//! All calls go directly to the underlying primals:
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
    METHOD_CERT_MINT, METHOD_COMPUTE_SUBMIT, METHOD_CRYPTO_HASH, METHOD_CRYPTO_SIGN,
    METHOD_CRYPTO_VERIFY, METHOD_DAG_EVENT_APPEND, METHOD_DAG_FRONTIER_GET, METHOD_DAG_MERKLE_ROOT,
    METHOD_DAG_QUERY_VERTICES, METHOD_DAG_SESSION_COMPLETE, METHOD_DAG_SESSION_CREATE,
    METHOD_MESH_BONDS, METHOD_MESH_HEALTH, METHOD_MESH_QUERY, METHOD_MESH_TOPOLOGY,
    METHOD_STORAGE_RETRIEVE, METHOD_STORAGE_STORE, SIGNAL_NEST_COMMIT, SIGNAL_NEST_STORE,
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
    /// directly via biomeOS semantic methods.
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

    // ── Provenance / attribution domain (sweetGrass) ──────────

    /// Create an attribution braid (sweetGrass).
    ///
    /// Braids weave action → provenance → lineage → attribution into a
    /// single traceable record. Returns the braid ID if successful.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn attribution_create(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::PROVENANCE,
            "braid.create",
            params.clone(),
            &["braid_id", "id"],
        )
    }

    /// Query attribution records for a session or content item.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn attribution_query(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::PROVENANCE, "braid.query", params.clone())
    }

    // ── Crypto domain (bearDog) ─────────────────────────────

    /// Sign a payload with the session key (bearDog).
    ///
    /// Returns the signature bytes as a hex string. Degrades to `None`
    /// when bearDog is unavailable — the game continues without
    /// cryptographic sealing.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn crypto_sign(&mut self, params: &serde_json::Value) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::CRYPTO,
            METHOD_CRYPTO_SIGN,
            params.clone(),
            &["signature", "sig"],
        )
    }

    /// Verify a signed payload (bearDog).
    ///
    /// Returns `true` if the signature is valid, `false` if bearDog
    /// is unavailable (degradation: unsigned is assumed valid locally).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn crypto_verify(&mut self, params: &serde_json::Value) -> Result<bool, IpcError> {
        self.call_or_default(domain::CRYPTO, METHOD_CRYPTO_VERIFY, params.clone(), false)
    }

    /// Hash arbitrary data for content-addressable identity (bearDog).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn crypto_hash(&mut self, params: &serde_json::Value) -> Result<Option<String>, IpcError> {
        self.call_extract_id(
            domain::CRYPTO,
            METHOD_CRYPTO_HASH,
            params.clone(),
            &["hash", "digest"],
        )
    }

    // ── Mesh / topology domain (songBird) ───────────────────

    /// Query the live ecosystem topology from songBird.
    ///
    /// Returns the full mesh graph: gates, primals, bonds, health
    /// status. This is the primary data source for the first milestone
    /// (static site rendering of ecosystem topology).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn mesh_topology(&mut self) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::MESH, METHOD_MESH_TOPOLOGY, serde_json::Value::Null)
    }

    /// Query mesh-wide health from songBird.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn mesh_health(&mut self) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::MESH, METHOD_MESH_HEALTH, serde_json::Value::Null)
    }

    /// Query a specific primal's mesh status by name.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn mesh_query(&mut self, primal: &str) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(
            domain::MESH,
            METHOD_MESH_QUERY,
            serde_json::json!({ "primal": primal }),
        )
    }

    /// List active bonds between primals in the mesh.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn mesh_bonds(&mut self) -> Result<Option<serde_json::Value>, IpcError> {
        self.call_passthrough(domain::MESH, METHOD_MESH_BONDS, serde_json::Value::Null)
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
            "primal": crate::niche::NICHE_NAME,
            "socket": socket,
            "capabilities": ["narrative", "session", "mcp"],
            "methods": methods,
            "composition_tiers": ["nest", "meta"],
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
        match self.neural_api_call("lifecycle", crate::ipc::METHOD_PRIMAL_ANNOUNCE, params) {
            Ok(resp) if resp.error.is_none() => {
                tracing::info!("primal.announce accepted by biomeOS");
            }
            Ok(_) | Err(_) => {
                tracing::debug!("primal.announce not accepted — operating without registration");
            }
        }

        self.register_mesh_route(socket, methods);
    }

    /// Register with mesh router for cross-gate capability discovery (Wave 73+).
    ///
    /// Enables other gates in the mesh to discover and invoke esotericWebb's
    /// interactive product capabilities. Includes stability tier metadata
    /// (Wave 75 push model) so the router can prioritize propagation of
    /// stable methods. Gracefully degrades if the mesh router is unavailable
    /// (standalone/single-gate operation continues).
    fn register_mesh_route(&mut self, socket: &str, methods: &[&str]) {
        let params = serde_json::json!({
            "primal": crate::niche::NICHE_NAME,
            "gate": crate::niche::gate_id(),
            "socket": socket,
            "capabilities": crate::niche::CAPABILITIES,
            "methods": methods,
            "version": env!("CARGO_PKG_VERSION"),
            "stability_tiers": {
                "stable": [
                    "health.liveness", "health.readiness", "health.check",
                    "health.version", "health.drain", "identity.get",
                    "capabilities.list", "primal.announce", "primal.info",
                    "webb.health", "webb.liveness", "webb.readiness",
                    "webb.scene.current", "webb.narrative.status", "webb.content.list",
                    "session.start", "session.state", "session.actions",
                    "session.act", "session.history", "session.narrate",
                    "session.graph", "session.metrics", "method.describe"
                ],
                "evolving": ["tools.list", "tools.call"],
            },
            "propagation": "push",
        });
        match self.neural_api_call("routing", crate::ipc::METHOD_ROUTE_REGISTER, params) {
            Ok(resp) if resp.error.is_none() => {
                tracing::info!("route.register accepted — mesh-visible (push propagation)");
            }
            Ok(_) | Err(_) => {
                tracing::debug!("route.register unavailable — single-gate mode");
            }
        }
    }
}
