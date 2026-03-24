// SPDX-License-Identifier: AGPL-3.0-or-later
//! `PrimalBridge` — runtime coordinator for primal composition.
//!
//! Discovers live primals at startup, holds connections, and delegates
//! domain-specific calls with graceful degradation. If a primal is
//! absent, every method returns a sensible default rather than failing.
//!
//! ## Transport priority
//!
//! TCP is preferred when a `tcp_addr` is available (platform-agnostic,
//! works inside containers and on Graphene). Falls back to UDS when only
//! a socket path was discovered.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::client::PrimalClient;
use super::discovery::PrimalRegistry;
use super::envelope::{IpcError, JsonRpcResponse};
use super::ludospring::{
    DdaResult, DialogueResponse, EngagementResult, FlowResult, METHOD_BEGIN_SESSION,
    METHOD_COMPLETE_SESSION, METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW,
    METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE, METHOD_PUSH_SCENE, METHOD_VOICE_CHECK, VoiceNote,
};
use super::petaltongue::{InputEvent, METHOD_INTERACTION_POLL, METHOD_RENDER_SCENE};
use super::resilience::{CircuitBreaker, RetryPolicy, is_recoverable};
use super::squirrel::ChatResponse;
use super::squirrel::{METHOD_AI_CHAT, METHOD_AI_SUMMARIZE};
use super::{
    DOMAIN_AI, DOMAIN_COMPUTE, DOMAIN_DAG, DOMAIN_GAME, DOMAIN_LINEAGE, DOMAIN_STORAGE,
    DOMAIN_VISUALIZATION, METHOD_CERT_MINT, METHOD_COMPUTE_SUBMIT, METHOD_DAG_EVENT_APPEND,
    METHOD_DAG_FRONTIER_GET, METHOD_DAG_MERKLE_ROOT, METHOD_DAG_QUERY_VERTICES,
    METHOD_DAG_SESSION_COMPLETE, METHOD_DAG_SESSION_CREATE, METHOD_STORAGE_RETRIEVE,
    METHOD_STORAGE_STORE, PRIMAL_DOMAINS,
};

/// Maximum characters for degraded summary truncation.
///
/// Overridable via `ESOTERICWEBB_SUMMARY_LIMIT` environment variable.
fn degraded_summary_limit() -> usize {
    std::env::var("ESOTERICWEBB_SUMMARY_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200)
}

/// Status of a single primal connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    /// Primal name.
    pub name: String,
    /// Capability domain.
    pub domain: String,
    /// Whether an endpoint was discovered (UDS or TCP).
    pub discovered: bool,
    /// Whether the primal responded to a health check.
    pub healthy: bool,
    /// Transport used for the connection (if connected).
    pub transport: Option<String>,
}

/// Runtime coordinator that discovers and holds live primal connections.
#[derive(Debug)]
pub struct PrimalBridge {
    clients: HashMap<String, PrimalClient>,
    statuses: Vec<PrimalStatus>,
    circuits: HashMap<String, CircuitBreaker>,
    retry_policy: RetryPolicy,
}

impl PrimalBridge {
    /// Discover live primals and connect to healthy ones.
    ///
    /// Tries TCP first (if a `tcp_addr` was discovered), falls back to UDS.
    pub fn discover() -> Self {
        let registry = PrimalRegistry::discover();
        let mut clients = HashMap::new();
        let mut statuses = Vec::new();

        for &(domain, name) in PRIMAL_DOMAINS {
            let endpoint = registry.by_domain.get(domain);
            let discovered = endpoint.is_some();
            let mut healthy = false;
            let mut transport_used: Option<String> = None;

            if let Some(ep) = endpoint {
                // Try TCP first — platform-agnostic, works in containers
                if let Some(ref addr) = ep.tcp_addr {
                    if let Ok(mut client) = PrimalClient::connect_tcp(addr, name) {
                        if client.health_liveness().unwrap_or(false) {
                            healthy = true;
                            transport_used = Some(format!("tcp:{addr}"));
                            clients.insert(domain.to_owned(), client);
                        }
                    }
                }

                // Fall back to UDS if TCP didn't succeed
                if !healthy {
                    if let Some(ref sock) = ep.socket_path {
                        if let Ok(mut client) = PrimalClient::connect(sock, name) {
                            if client.health_liveness().unwrap_or(false) {
                                healthy = true;
                                transport_used = Some(format!("uds:{}", sock.display()));
                                clients.insert(domain.to_owned(), client);
                            }
                        }
                    }
                }
            }

            statuses.push(PrimalStatus {
                name: name.to_owned(),
                domain: domain.to_owned(),
                discovered,
                healthy,
                transport: transport_used,
            });
        }

        let circuits = PRIMAL_DOMAINS
            .iter()
            .map(|&(domain, _)| (domain.to_owned(), CircuitBreaker::from_env()))
            .collect();
        Self {
            clients,
            statuses,
            circuits,
            retry_policy: RetryPolicy::from_env(),
        }
    }

    /// Create an empty bridge with no connections (for standalone mode).
    pub fn standalone() -> Self {
        let statuses = PRIMAL_DOMAINS
            .iter()
            .map(|&(domain, name)| PrimalStatus {
                name: name.to_owned(),
                domain: domain.to_owned(),
                discovered: false,
                healthy: false,
                transport: None,
            })
            .collect();
        let circuits = PRIMAL_DOMAINS
            .iter()
            .map(|&(domain, _)| (domain.to_owned(), CircuitBreaker::from_env()))
            .collect();

        Self {
            clients: HashMap::new(),
            statuses,
            circuits,
            retry_policy: RetryPolicy::from_env(),
        }
    }

    /// Inject a pre-connected client for a domain (used by launcher).
    pub fn inject(&mut self, domain: &str, client: PrimalClient, transport_label: &str) {
        let name = client.primal().to_owned();
        self.clients.insert(domain.to_owned(), client);
        for s in &mut self.statuses {
            if s.domain == domain {
                s.discovered = true;
                s.healthy = true;
                s.transport = Some(transport_label.to_owned());
                s.name.clone_from(&name);
            }
        }
    }

    /// Status of all primal connections.
    pub fn statuses(&self) -> &[PrimalStatus] {
        &self.statuses
    }

    /// Whether a specific domain has a healthy connection.
    pub fn has(&self, domain: &str) -> bool {
        self.clients.contains_key(domain)
    }

    /// Number of connected primals.
    pub fn connected_count(&self) -> usize {
        self.clients.len()
    }

    /// Resilient call: circuit breaker + retry with exponential backoff.
    ///
    /// Returns `Ok(response)` on success, or the last error after exhausting
    /// retries. Non-recoverable errors are returned immediately.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "params is cloned per retry attempt; owned avoids extra clone at call sites"
    )]
    fn resilient_call(
        &mut self,
        domain: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcResponse, IpcError> {
        let circuit = self.circuits.get(domain);
        if circuit.is_some_and(|cb| !cb.is_allowed()) {
            return Err(IpcError::Io(format!(
                "circuit open for domain {domain} — skipping {method}"
            )));
        }

        let client = self
            .clients
            .get_mut(domain)
            .ok_or_else(|| IpcError::PrimalNotFound(domain.to_owned()))?;

        let mut last_err = None;
        let max = self.retry_policy.max_retries;

        for attempt in 0..=max {
            match client.call(method, params.clone()) {
                Ok(resp) => {
                    if let Some(cb) = self.circuits.get(domain) {
                        cb.record_success();
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    if !is_recoverable(&e) {
                        if let Some(cb) = self.circuits.get(domain) {
                            cb.record_failure();
                        }
                        return Err(e);
                    }
                    tracing::debug!(
                        domain,
                        method,
                        attempt,
                        error = %e,
                        "IPC call failed, will retry"
                    );
                    last_err = Some(e);
                    if attempt < max {
                        std::thread::sleep(self.retry_policy.delay_for_attempt(attempt));
                    }
                }
            }
        }

        if let Some(cb) = self.circuits.get(domain) {
            cb.record_failure();
        }
        Err(last_err.unwrap_or_else(|| IpcError::Io("all retries exhausted".to_owned())))
    }

    // ── AI domain (Squirrel) ────────────────────────────────

    /// Generate narration via the AI primal.
    ///
    /// Degrades to a placeholder if Squirrel is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_narrate(&mut self, prompt: &str) -> Result<ChatResponse, IpcError> {
        if self.has(DOMAIN_AI) {
            let params = serde_json::json!({
                "messages": [{"role": "user", "content": prompt}],
            });
            if let Ok(resp) = self.resilient_call(DOMAIN_AI, METHOD_AI_CHAT, params) {
                if let Some(result) = resp.result {
                    if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                        return Ok(chat);
                    }
                }
            }
        }
        Ok(ChatResponse {
            text: format!("[AI primal unavailable — narration placeholder for: {prompt}]"),
            model: "none".to_owned(),
            tokens: 0,
        })
    }

    /// Summarize context via the AI primal.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn ai_summarize(&mut self, context: &str) -> Result<String, IpcError> {
        if self.has(DOMAIN_AI) {
            let params = serde_json::json!({ "text": context });
            if let Ok(resp) = self.resilient_call(DOMAIN_AI, METHOD_AI_SUMMARIZE, params) {
                if let Some(serde_json::Value::String(text)) = resp.result {
                    return Ok(text);
                }
            }
        }
        let truncated: String = context.chars().take(degraded_summary_limit()).collect();
        Ok(format!("[degraded: summary unavailable] {truncated}..."))
    }

    // ── Visualization domain (PetalTongue) ──────────────────

    /// Push a scene payload for rendering.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn render_scene(&mut self, scene: &serde_json::Value) -> Result<(), IpcError> {
        if self.has(DOMAIN_VISUALIZATION) {
            let _ =
                self.resilient_call(DOMAIN_VISUALIZATION, METHOD_RENDER_SCENE, scene.clone())?;
        }
        Ok(())
    }

    // ── Compute domain (ToadStool) ──────────────────────────

    /// Submit a compute task.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn compute_submit(&mut self, task: &serde_json::Value) -> Result<Option<String>, IpcError> {
        if self.has(DOMAIN_COMPUTE) {
            let resp = self.resilient_call(DOMAIN_COMPUTE, METHOD_COMPUTE_SUBMIT, task.clone())?;
            if let Some(serde_json::Value::String(job_id)) =
                resp.result.as_ref().and_then(|r| r.get("job_id").cloned())
            {
                return Ok(Some(job_id));
            }
        }
        Ok(None)
    }

    // ── Storage domain (NestGate) ───────────────────────────

    /// Store a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn store(&mut self, key: &str, value: &serde_json::Value) -> Result<bool, IpcError> {
        if self.has(DOMAIN_STORAGE) {
            let params = serde_json::json!({ "key": key, "value": value });
            let resp = self.resilient_call(DOMAIN_STORAGE, METHOD_STORAGE_STORE, params)?;
            return Ok(resp.error.is_none());
        }
        Ok(false)
    }

    /// Retrieve a value.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn retrieve(&mut self, key: &str) -> Result<Option<serde_json::Value>, IpcError> {
        if self.has(DOMAIN_STORAGE) {
            let params = serde_json::json!({ "key": key });
            let resp = self.resilient_call(DOMAIN_STORAGE, METHOD_STORAGE_RETRIEVE, params)?;
            return Ok(resp.result);
        }
        Ok(None)
    }

    // ── Game science domain (LudoSpring) ────────────────────

    /// Evaluate flow state.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn evaluate_flow(&mut self, params: &serde_json::Value) -> Result<FlowResult, IpcError> {
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) = self.resilient_call(DOMAIN_GAME, METHOD_EVALUATE_FLOW, params.clone())
            {
                if let Some(result) = resp.result {
                    if let Ok(flow) = serde_json::from_value::<FlowResult>(result) {
                        return Ok(flow);
                    }
                }
            }
        }
        Ok(FlowResult {
            flow_score: 0.5,
            in_flow: false,
        })
    }

    /// Get engagement metrics.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn engagement(&mut self, params: &serde_json::Value) -> Result<EngagementResult, IpcError> {
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) = self.resilient_call(DOMAIN_GAME, METHOD_ENGAGEMENT, params.clone()) {
                if let Some(result) = resp.result {
                    if let Ok(eng) = serde_json::from_value::<EngagementResult>(result) {
                        return Ok(eng);
                    }
                }
            }
        }
        Ok(EngagementResult {
            actions_per_minute: 0.0,
            exploration_ratio: 0.0,
            engagement_score: 0.5,
        })
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
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) =
                self.resilient_call(DOMAIN_GAME, METHOD_DIFFICULTY_ADJUSTMENT, params.clone())
            {
                if let Some(result) = resp.result {
                    if let Ok(dda) = serde_json::from_value::<DdaResult>(result) {
                        return Ok(dda);
                    }
                }
            }
        }
        Ok(DdaResult {
            adjustment: 0.0,
            reason: "game science primal unavailable — no adjustment".to_owned(),
        })
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
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) = self.resilient_call(DOMAIN_GAME, METHOD_NPC_DIALOGUE, params.clone())
            {
                if let Some(result) = resp.result {
                    if let Ok(d) = serde_json::from_value::<DialogueResponse>(result) {
                        return Ok(d);
                    }
                }
            }
        }
        Ok(DialogueResponse {
            text: "[game science primal unavailable — NPC dialogue degraded]".to_owned(),
            voice_notes: Vec::new(),
            passive_checks_fired: false,
        })
    }

    /// Narrate an action via ludoSpring → Squirrel delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn narrate_action(&mut self, params: &serde_json::Value) -> Result<ChatResponse, IpcError> {
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) =
                self.resilient_call(DOMAIN_GAME, METHOD_NARRATE_ACTION, params.clone())
            {
                if let Some(result) = resp.result {
                    if let Ok(chat) = serde_json::from_value::<ChatResponse>(result) {
                        return Ok(chat);
                    }
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
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn voice_check(&mut self, params: &serde_json::Value) -> Result<Vec<VoiceNote>, IpcError> {
        if self.has(DOMAIN_GAME) {
            if let Ok(resp) = self.resilient_call(DOMAIN_GAME, METHOD_VOICE_CHECK, params.clone()) {
                if let Some(result) = resp.result {
                    if let Ok(notes) = serde_json::from_value::<Vec<VoiceNote>>(result) {
                        return Ok(notes);
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    /// Push scene via ludoSpring → petalTongue delegation.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn game_push_scene(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        if self.has(DOMAIN_GAME) {
            let _ = self.resilient_call(DOMAIN_GAME, METHOD_PUSH_SCENE, params.clone())?;
        }
        Ok(())
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
        if self.has(DOMAIN_GAME) {
            let resp = self.resilient_call(DOMAIN_GAME, METHOD_BEGIN_SESSION, params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("session_id")
                    .or_else(|| result.get("id"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Complete a game session in the provenance system via ludoSpring.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn game_complete_session(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        if self.has(DOMAIN_GAME) {
            let _ = self.resilient_call(DOMAIN_GAME, METHOD_COMPLETE_SESSION, params.clone())?;
        }
        Ok(())
    }

    // ── Provenance domain (RootPulse trio) ──────────────────

    /// Append a provenance vertex to the session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn provenance_append(&mut self, vertex: &serde_json::Value) -> Result<bool, IpcError> {
        if self.has(DOMAIN_DAG) {
            let resp = self.resilient_call(DOMAIN_DAG, METHOD_DAG_EVENT_APPEND, vertex.clone())?;
            return Ok(resp.error.is_none());
        }
        Ok(false)
    }

    // ── DAG domain (rhizoCrypt) — typed API ─────────────────

    /// Create a new session on the DAG primal.
    ///
    /// Returns the session ID if rhizoCrypt is connected, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_session_create(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if self.has(DOMAIN_DAG) {
            let resp =
                self.resilient_call(DOMAIN_DAG, METHOD_DAG_SESSION_CREATE, params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("session_id")
                    .or_else(|| result.get("id"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Append an event to a session DAG.
    ///
    /// Returns the vertex ID if successful.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_event_append(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<Option<String>, IpcError> {
        if self.has(DOMAIN_DAG) {
            let resp = self.resilient_call(DOMAIN_DAG, METHOD_DAG_EVENT_APPEND, params.clone())?;
            if let Some(result) = resp.result {
                if let Some(id) = result
                    .get("vertex_id")
                    .or_else(|| result.get("id"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(id.to_owned()));
                }
                return Ok(Some("ok".to_owned()));
            }
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
        if self.has(DOMAIN_DAG) {
            let resp = self.resilient_call(DOMAIN_DAG, METHOD_DAG_FRONTIER_GET, params.clone())?;
            return Ok(resp.result);
        }
        Ok(None)
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
        if self.has(DOMAIN_DAG) {
            let resp = self.resilient_call(DOMAIN_DAG, METHOD_DAG_MERKLE_ROOT, params.clone())?;
            if let Some(result) = resp.result {
                if let Some(root) = result
                    .get("root")
                    .or_else(|| result.get("merkle_root"))
                    .and_then(serde_json::Value::as_str)
                {
                    return Ok(Some(root.to_owned()));
                }
            }
        }
        Ok(None)
    }

    /// Complete a session DAG.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn dag_session_complete(&mut self, params: &serde_json::Value) -> Result<(), IpcError> {
        if self.has(DOMAIN_DAG) {
            let _ = self.resilient_call(DOMAIN_DAG, METHOD_DAG_SESSION_COMPLETE, params.clone())?;
        }
        Ok(())
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
        if self.has(DOMAIN_DAG) {
            let resp =
                self.resilient_call(DOMAIN_DAG, METHOD_DAG_QUERY_VERTICES, params.clone())?;
            return Ok(resp.result);
        }
        Ok(None)
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
        if self.has(DOMAIN_LINEAGE) {
            let resp = self.resilient_call(DOMAIN_LINEAGE, METHOD_CERT_MINT, params.clone())?;
            return Ok(resp.result);
        }
        Ok(None)
    }

    // ── Visualization domain (petalTongue) — direct ─────────

    /// Poll for player input events from petalTongue.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the call fails unexpectedly.
    pub fn poll_input(&mut self) -> Result<Vec<InputEvent>, IpcError> {
        if self.has(DOMAIN_VISUALIZATION) {
            let resp = self.resilient_call(
                DOMAIN_VISUALIZATION,
                METHOD_INTERACTION_POLL,
                serde_json::Value::Null,
            )?;
            if let Some(result) = resp.result {
                if let Ok(events) = serde_json::from_value::<Vec<InputEvent>>(result) {
                    return Ok(events);
                }
            }
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn standalone_bridge_has_no_connections() {
        let bridge = PrimalBridge::standalone();
        assert_eq!(bridge.connected_count(), 0);
        assert!(!bridge.has(DOMAIN_AI));
        assert!(!bridge.has(DOMAIN_VISUALIZATION));
        assert!(!bridge.has(DOMAIN_COMPUTE));
        assert!(!bridge.has(DOMAIN_STORAGE));
        assert_eq!(bridge.statuses().len(), PRIMAL_DOMAINS.len());
    }

    #[test]
    fn standalone_ai_narrate_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let resp = bridge.ai_narrate("test prompt").unwrap();
        assert!(resp.text.contains("unavailable"));
        assert_eq!(resp.model, "none");
    }

    #[test]
    fn standalone_ai_summarize_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let resp = bridge.ai_summarize("some context").unwrap();
        assert!(resp.contains("summary unavailable"));
    }

    #[test]
    fn standalone_render_scene_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.render_scene(&serde_json::json!({"type": "test"}));
        assert!(result.is_ok());
    }

    #[test]
    fn standalone_compute_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.compute_submit(&serde_json::json!({})).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_store_returns_false() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.store("key", &serde_json::json!("value")).unwrap();
        assert!(!result);
    }

    #[test]
    fn standalone_retrieve_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.retrieve("key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_evaluate_flow_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.evaluate_flow(&serde_json::Value::Null).unwrap();
        assert!((result.flow_score - 0.5).abs() < f64::EPSILON);
        assert!(!result.in_flow);
    }

    #[test]
    fn standalone_difficulty_adjustment_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .difficulty_adjustment(&serde_json::Value::Null)
            .unwrap();
        assert!((result.adjustment).abs() < f64::EPSILON);
    }

    #[test]
    fn standalone_provenance_returns_false() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .provenance_append(&serde_json::json!({"id": "v1"}))
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn standalone_dag_session_create_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.dag_session_create(&serde_json::json!({})).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_event_append_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_event_append(&serde_json::json!({"session_id": "s1", "data": {}}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_frontier_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_frontier_get(&serde_json::json!({"session_id": "s1"}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_dag_merkle_root_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .dag_merkle_root(&serde_json::json!({"session_id": "s1"}))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn standalone_engagement_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge.engagement(&serde_json::Value::Null).unwrap();
        assert!((result.engagement_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn standalone_npc_dialogue_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .npc_dialogue(&serde_json::json!({"npc_id": "test"}))
            .unwrap();
        assert!(result.text.contains("degraded"));
        assert!(result.voice_notes.is_empty());
    }

    #[test]
    fn standalone_narrate_action_degrades() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .narrate_action(&serde_json::json!({"action": "test"}))
            .unwrap();
        assert_eq!(result.model, "none");
    }

    #[test]
    fn standalone_voice_check_returns_empty() {
        let mut bridge = PrimalBridge::standalone();
        let result = bridge
            .voice_check(&serde_json::json!({"voice_id": "logic"}))
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn standalone_game_push_scene_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .game_push_scene(&serde_json::json!({"type": "test"}))
                .is_ok()
        );
    }

    #[test]
    fn standalone_game_begin_session_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .game_begin_session(&serde_json::json!({}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_game_complete_session_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(bridge.game_complete_session(&serde_json::json!({})).is_ok());
    }

    #[test]
    fn standalone_dag_session_complete_is_noop() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .dag_session_complete(&serde_json::json!({"session_id": "s1"}))
                .is_ok()
        );
    }

    #[test]
    fn standalone_dag_query_vertices_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .dag_query_vertices(&serde_json::json!({"session_id": "s1"}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_mint_certificate_returns_none() {
        let mut bridge = PrimalBridge::standalone();
        assert!(
            bridge
                .mint_certificate(&serde_json::json!({"npc": "test"}))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn standalone_poll_input_returns_empty() {
        let mut bridge = PrimalBridge::standalone();
        assert!(bridge.poll_input().unwrap().is_empty());
    }

    #[test]
    fn discover_with_no_sockets_is_standalone() {
        let bridge = PrimalBridge::discover();
        assert_eq!(bridge.statuses().len(), PRIMAL_DOMAINS.len());
        for s in bridge.statuses() {
            assert!(!s.healthy);
        }
    }
}
